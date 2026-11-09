// The Terminal panel's shells (src/terminals.ts). Most of it with the helper
// faked; one test builds and runs the real one where a C compiler is here.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import { execFileSync } from 'node:child_process';
import { tmpDir } from '../helpers/fakes.ts';
import { ErrorCode, RpcError } from '../../src/protocol.ts';
import { TerminalManager, buildHelper, frame, type HelperProcess, type TerminalNotification, type TerminalSink } from '../../src/terminals.ts';

class FakeProc extends EventEmitter {
  written: Buffer[] = [];
  ended = false;
  killed: string[] = [];
  stdout = new EventEmitter();
  stderr = new EventEmitter();
  stdin = {
    write: (b: Buffer) => this.written.push(b),
    end: () => {
      this.ended = true;
    },
  };
  args: string[] = [];
  kill(sig?: string) {
    this.killed.push(sig ?? 'SIGTERM');
  }
  say(text: string) {
    this.stdout.emit('data', Buffer.from(text));
  }
  exit(code: number | null) {
    this.emit('exit', code, null);
  }
}

function sink(): TerminalSink & { got: TerminalNotification[]; text(): string } {
  const got: TerminalNotification[] = [];
  return {
    got,
    send: (n) => got.push(n),
    text: () => got.filter((n) => n.method === 'terminal.output').map((n) => Buffer.from((n.params as { data: string }).data, 'base64').toString()).join(''),
  };
}

function make(over: Partial<ConstructorParameters<typeof TerminalManager>[0]> = {}) {
  const procs: FakeProc[] = [];
  let now = 1_000_000;
  const refused = new Set<string>();
  const m = new TerminalManager({
    home: tmpDir(),
    thread: (id) => {
      if (refused.has(id)) throw new RpcError(ErrorCode.other, 'Open it on the Mac that has it.');
      return { cwd: `/work/${id}`, env: { THREADBEAR_THREAD_ID: id } };
    },
    helper: async () => '/fake/pty-helper',
    spawn: (_file, args) => {
      const p = new FakeProc();
      p.args = args;
      procs.push(p);
      return p as unknown as HelperProcess;
    },
    shell: '/bin/zsh',
    now: () => now,
    flushMs: 1,
    ...over,
  });
  return { m, procs, refused, tick: (ms: number) => (now += ms) };
}

const wait = (ms = 10) => new Promise((r) => setTimeout(r, ms));

test('open starts one shell per thread in its folder, and opening again attaches to it', async () => {
  const { m, procs } = make();
  const a = sink();
  const r1 = await m.open('t1', 100, 30, a);
  assert.equal(r1.alive, true);
  assert.deepEqual(procs[0]!.args, ['100', '30', '/work/t1', '/bin/zsh', '-l']);
  const r2 = await m.open('t1', 100, 30, sink());
  assert.equal(r2.terminalId, r1.terminalId);
  assert.equal(procs.length, 1);
});

test('output reaches only the attached connection, batched, and the replay keeps it for the next open', async () => {
  const { m, procs } = make();
  const a = sink();
  const { terminalId } = await m.open('t1', 80, 24, a);
  procs[0]!.say('hello ');
  procs[0]!.say('world');
  await wait();
  assert.equal(a.text(), 'hello world');
  assert.equal(a.got.length, 1, 'two writes in one batch');
  m.detach(terminalId, a);
  procs[0]!.say('!');
  await wait();
  assert.equal(a.text(), 'hello world', 'nothing after detach');
  const b = sink();
  const again = await m.open('t1', 80, 24, b);
  assert.equal(Buffer.from(again.replay, 'base64').toString(), 'hello world!');
});

test('the replay keeps only the last replayBytes', async () => {
  const { m, procs } = make({ replayBytes: 10 });
  await m.open('t1', 80, 24, sink());
  procs[0]!.say('0123456789');
  procs[0]!.say('abcde');
  const r = await m.open('t1', 80, 24, sink());
  assert.equal(Buffer.from(r.replay, 'base64').toString(), '56789abcde');
});

test('input and resize go to the helper as frames', async () => {
  const { m, procs } = make();
  const { terminalId } = await m.open('t1', 80, 24, sink());
  procs[0]!.written.length = 0;
  m.input(terminalId, Buffer.from('ls\r').toString('base64'));
  m.resize(terminalId, 120, 40);
  const size = Buffer.alloc(4);
  size.writeUInt16BE(120, 0);
  size.writeUInt16BE(40, 2);
  assert.deepEqual(procs[0]!.written, [frame(0, Buffer.from('ls\r')), frame(1, size)]);
});

test('too much input in one call is refused', async () => {
  const { m } = make();
  const { terminalId } = await m.open('t1', 80, 24, sink());
  assert.throws(() => m.input(terminalId, Buffer.alloc(64 * 1024 + 1).toString('base64')), (e: RpcError) => e.code === ErrorCode.badParams);
});

test('a shell that exits tells the attached connection, and open then says it ended', async () => {
  const { m, procs } = make();
  const a = sink();
  const { terminalId } = await m.open('t1', 80, 24, a);
  procs[0]!.exit(3);
  assert.deepEqual(a.got.at(-1), { method: 'terminal.exited', params: { terminalId, threadId: 't1', exitCode: 3, reason: 'exited' } });
  const r = await m.open('t1', 80, 24, sink());
  assert.equal(r.alive, false);
  assert.equal(r.exitCode, 3);
  assert.equal(r.reason, 'exited');
  assert.throws(() => m.input(terminalId, 'aGk='), (e: RpcError) => e.code === ErrorCode.threadNotFound);
});

test('restart ends the old shell and starts a new one', async () => {
  const { m, procs } = make();
  const a = sink();
  const first = await m.open('t1', 80, 24, a);
  const second = await m.restart('t1', 80, 24, a);
  assert.notEqual(second.terminalId, first.terminalId);
  assert.equal(procs[0]!.ended, true, 'old helper hung up');
  assert.equal(procs.length, 2);
  assert.equal(second.alive, true);
});

test('archiving a thread ends its shell with that reason; deleting forgets it', async () => {
  const { m, procs } = make();
  const a = sink();
  const { terminalId } = await m.open('t1', 80, 24, a);
  m.threadGone('t1', 'archived');
  assert.equal(procs[0]!.ended, true);
  procs[0]!.exit(129);
  assert.deepEqual(a.got.at(-1), { method: 'terminal.exited', params: { terminalId, threadId: 't1', exitCode: 129, reason: 'archived' } });
  m.threadGone('t1', 'deleted');
  const r = await m.open('t1', 80, 24, sink());
  assert.equal(r.alive, true, 'a deleted thread\'s record is gone, so this is a new shell');
});

test('a shell left detached past idleMs is ended', async () => {
  const { m, procs, tick } = make({ idleMs: 1000 });
  const a = sink();
  const { terminalId } = await m.open('t1', 80, 24, a);
  tick(5000);
  m.sweepIdle();
  assert.equal(procs[0]!.ended, false, 'attached, so not idle');
  m.dropSink(a);
  tick(999);
  m.sweepIdle();
  assert.equal(procs[0]!.ended, false);
  tick(1);
  m.sweepIdle();
  assert.equal(procs[0]!.ended, true);
  procs[0]!.exit(129);
  const r = await m.open('t1', 80, 24, sink());
  assert.equal(r.reason, 'idle');
  assert.equal(r.terminalId, terminalId);
});

test('past maxLive the shell detached longest is ended to make room', async () => {
  const { m, procs, tick } = make({ maxLive: 2 });
  const a = sink();
  const b = sink();
  const t1 = await m.open('t1', 80, 24, a);
  tick(1);
  const t2 = await m.open('t2', 80, 24, b);
  m.detach(t1.terminalId, a);
  tick(1);
  m.detach(t2.terminalId, b);
  // Exits come later in real life; make room straight away here.
  procs[0]!.stdin.end = () => {
    procs[0]!.ended = true;
    procs[0]!.exit(129);
  };
  await m.open('t3', 80, 24, sink());
  assert.equal(procs[0]!.ended, true, 't1 was detached longest');
  assert.equal(procs[1]!.ended, false);
});

test('past maxLive with every shell attached, open is refused', async () => {
  const { m } = make({ maxLive: 1 });
  await m.open('t1', 80, 24, sink());
  await assert.rejects(m.open('t2', 80, 24, sink()), (e: RpcError) => e.code === ErrorCode.other);
});

test('a thread that can\'t have a terminal here is refused, even with an ended record', async () => {
  const { m, refused, procs } = make();
  refused.add('remote');
  await assert.rejects(m.open('remote', 80, 24, sink()), /Mac that has it/);
  assert.equal(procs.length, 0);
});

test('stopAll ends every shell with reason runner', async () => {
  const { m, procs } = make();
  const a = sink();
  await m.open('t1', 80, 24, a);
  m.stopAll();
  procs[0]!.exit(129);
  assert.equal((a.got.at(-1)!.params as { reason: string }).reason, 'runner');
});

const hasCc = (() => {
  try {
    execFileSync('cc', ['--version'], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
})();

test('the real helper: runs a shell, takes a resize, and hangs up when let go', { skip: !hasCc && 'no C compiler here' }, async () => {
  const home = tmpDir();
  const helper = await buildHelper(home);
  assert.equal(await buildHelper(home), helper, 'built once, then reused');
  const m = new TerminalManager({ home, thread: () => ({ cwd: home, env: {} }), helper: async () => helper, shell: '/bin/sh', flushMs: 1 });
  const a = sink();
  const { terminalId } = await m.open('t1', 80, 24, a);
  m.resize(terminalId, 123, 45);
  m.input(terminalId, Buffer.from('stty size; echo "di""d it"\n').toString('base64'));
  for (let i = 0; i < 100 && !a.text().includes('did it'); i++) await wait(20);
  assert.match(a.text(), /45 123/);
  assert.match(a.text(), /did it/);
  m.kill(terminalId);
  for (let i = 0; i < 100 && !a.got.some((n) => n.method === 'terminal.exited'); i++) await wait(20);
  const exited = a.got.find((n) => n.method === 'terminal.exited');
  assert.equal((exited?.params as { reason: string }).reason, 'killed');
});
