// The Terminal panel's shells (docs/protocol.md, "The Terminal panel"): one
// per thread, in its folder, through the pseudo-terminal helper in
// terminal/pty-helper.c, which this compiles the first time it's needed.
// Output goes only to the connection attached to the terminal; the last
// REPLAY_BYTES of it are kept so a panel opened again can redraw.
import { execFile, spawn as nodeSpawn } from 'node:child_process';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Logger } from './log.ts';
import { ErrorCode, RpcError } from './protocol.ts';

export const REPLAY_BYTES = 256 * 1024;
export const FLUSH_MS = 16;
export const MAX_MESSAGE_BYTES = 64 * 1024;
export const MAX_INPUT_BYTES = 64 * 1024;
export const IDLE_MS = 8 * 60 * 60_000;
export const MAX_LIVE = 16;
const BUILD_TIMEOUT_MS = 2 * 60_000;

export type TerminalEndReason = 'exited' | 'archived' | 'deleted' | 'idle' | 'runner' | 'evicted' | 'killed';

export type TerminalNotification =
  | { method: 'terminal.output'; params: { terminalId: string; threadId: string; data: string } }
  | { method: 'terminal.exited'; params: { terminalId: string; threadId: string; exitCode: number | null; reason: TerminalEndReason } };

// One connection's view of terminals: rpc.ts makes one per WebSocket.
export type TerminalSink = { send(n: TerminalNotification): void };

export type OpenResult = { terminalId: string; replay: string; alive: boolean; startedAt: string; exitCode?: number | null; reason?: TerminalEndReason };

// What the helper process looks like to this module (a ChildProcess, or a
// test's fake).
export type HelperProcess = {
  stdin: { write(b: Buffer): unknown; end(): unknown } | null;
  stdout: { on(ev: 'data', fn: (b: Buffer) => void): unknown } | null;
  stderr?: { on(ev: 'data', fn: (b: Buffer) => void): unknown } | null;
  on(ev: 'exit', fn: (code: number | null, signal: NodeJS.Signals | null) => void): unknown;
  on(ev: 'error', fn: (e: Error) => void): unknown;
  kill(signal?: NodeJS.Signals): unknown;
};

export type TerminalDeps = {
  home: string;
  log?: Logger;
  // The thread's folder and variables, or a throw (an RpcError) when it
  // can't have a terminal here.
  thread(threadId: string): { cwd: string; env: Record<string, string> };
  // The compiled helper's path; built once by default.
  helper?: () => Promise<string>;
  spawn?: (file: string, args: string[], opts: { cwd: string; env: NodeJS.ProcessEnv }) => HelperProcess;
  shell?: string;
  now?: () => number;
  flushMs?: number;
  idleMs?: number;
  maxLive?: number;
  replayBytes?: number;
};

type Term = {
  id: string;
  threadId: string;
  proc?: HelperProcess;
  alive: boolean;
  exitCode?: number | null;
  startedAt: string;
  replay: Buffer[];
  replaySize: number;
  pending: Buffer[];
  pendingSize: number;
  flushTimer?: NodeJS.Timeout;
  sink?: TerminalSink;
  detachedAt: number;
  // Why it was ended, when this side ended it (so the helper's exit says so).
  ending?: TerminalEndReason;
};

export class TerminalManager {
  private byThread = new Map<string, Term>();
  private byId = new Map<string, Term>();
  private helperPath?: Promise<string>;
  private idleTimer?: NodeJS.Timeout;

  private deps: TerminalDeps;

  constructor(deps: TerminalDeps) {
    this.deps = deps;
  }

  private now(): number {
    return this.deps.now?.() ?? Date.now();
  }

  /** Attaches `sink` to the thread's shell, starting one when it has none. */
  async open(threadId: string, cols: number, rows: number, sink: TerminalSink): Promise<OpenResult> {
    const existing = this.byThread.get(threadId);
    if (existing) {
      // An ended shell's thread still has to be allowed one before it's shown again.
      this.deps.thread(threadId);
      this.attach(existing, sink);
      if (existing.alive) this.resize(existing.id, cols, rows);
      return this.result(existing);
    }
    return this.start(threadId, cols, rows, sink);
  }

  /** A new shell for the thread, ending the one it has. */
  async restart(threadId: string, cols: number, rows: number, sink: TerminalSink): Promise<OpenResult> {
    this.deps.thread(threadId);
    const old = this.byThread.get(threadId);
    if (old) this.end(old, 'killed', false);
    return this.start(threadId, cols, rows, sink);
  }

  input(terminalId: string, data: string): Record<string, never> {
    const t = this.mustLive(terminalId);
    const bytes = Buffer.from(data, 'base64');
    if (bytes.length > MAX_INPUT_BYTES) throw new RpcError(ErrorCode.badParams, `At most ${MAX_INPUT_BYTES} bytes of input a call.`);
    if (bytes.length) t.proc?.stdin?.write(frame(0, bytes));
    return {};
  }

  resize(terminalId: string, cols: number, rows: number): Record<string, never> {
    const t = this.mustLive(terminalId);
    const b = Buffer.alloc(4);
    b.writeUInt16BE(clampSize(cols), 0);
    b.writeUInt16BE(clampSize(rows), 2);
    t.proc?.stdin?.write(frame(1, b));
    return {};
  }

  detach(terminalId: string, sink: TerminalSink): Record<string, never> {
    const t = this.byId.get(terminalId);
    if (t && t.sink === sink) this.letGo(t);
    return {};
  }

  kill(terminalId: string): Record<string, never> {
    const t = this.byId.get(terminalId);
    if (!t) throw notFound();
    if (t.alive) this.end(t, 'killed', true);
    return {};
  }

  /** A connection has closed: its terminals keep running, detached. */
  dropSink(sink: TerminalSink): void {
    for (const t of this.byId.values()) if (t.sink === sink) this.letGo(t);
  }

  /** The thread was archived or deleted: its shell ends with it. */
  threadGone(threadId: string, reason: 'archived' | 'deleted'): void {
    const t = this.byThread.get(threadId);
    if (!t) return;
    if (t.alive) this.end(t, reason, true);
    if (reason === 'deleted') this.forget(t);
  }

  /** Ends shells nobody has been attached to for idleMs. */
  sweepIdle(): void {
    const idle = this.deps.idleMs ?? IDLE_MS;
    const now = this.now();
    for (const t of [...this.byId.values()]) {
      if (t.alive && !t.sink && now - t.detachedAt >= idle) this.end(t, 'idle', true);
    }
  }

  startSweeping(everyMs = 10 * 60_000): void {
    this.idleTimer ??= setInterval(() => this.sweepIdle(), everyMs);
    this.idleTimer.unref?.();
  }

  /** The runner is stopping: every shell ends. */
  stopAll(): void {
    if (this.idleTimer) clearInterval(this.idleTimer);
    this.idleTimer = undefined;
    for (const t of [...this.byId.values()]) if (t.alive) this.end(t, 'runner', true);
  }

  liveCount(): number {
    let n = 0;
    for (const t of this.byId.values()) if (t.alive) n++;
    return n;
  }

  private async start(threadId: string, cols: number, rows: number, sink: TerminalSink): Promise<OpenResult> {
    const { cwd, env } = this.deps.thread(threadId);
    const helper = await this.helper();
    // Room for this one, ending the shell that's been detached longest.
    const max = this.deps.maxLive ?? MAX_LIVE;
    while (this.liveCount() >= max) {
      const oldest = [...this.byId.values()].filter((t) => t.alive && !t.sink).sort((a, b) => a.detachedAt - b.detachedAt)[0];
      if (!oldest) throw new RpcError(ErrorCode.other, `${max} terminals are open already. Close one first.`);
      this.end(oldest, 'evicted', true);
    }
    // Another open may have started one while the helper was building.
    const raced = this.byThread.get(threadId);
    if (raced?.alive) {
      this.attach(raced, sink);
      return this.result(raced);
    }
    if (raced) this.forget(raced);

    const shell = this.deps.shell ?? (process.env.SHELL || os.userInfo().shell || '/bin/zsh');
    const t: Term = {
      id: `term_${crypto.randomBytes(8).toString('hex')}`,
      threadId,
      alive: true,
      startedAt: new Date(this.now()).toISOString(),
      replay: [],
      replaySize: 0,
      pending: [],
      pendingSize: 0,
      detachedAt: this.now(),
    };
    const spawn = this.deps.spawn ?? ((file: string, args: string[], opts: { cwd: string; env: NodeJS.ProcessEnv }): HelperProcess => nodeSpawn(file, args, { ...opts, stdio: ['pipe', 'pipe', 'pipe'] }));
    const proc = spawn(helper, [String(clampSize(cols)), String(clampSize(rows)), cwd, shell, '-l'], {
      cwd,
      env: { ...process.env, ...env, TERM_PROGRAM: 'Threadbear' },
    });
    t.proc = proc;
    proc.stdout?.on('data', (b: Buffer) => this.output(t, b));
    proc.stderr?.on('data', (b: Buffer) => this.output(t, b));
    proc.on('error', (e: Error) => {
      this.deps.log?.warn('terminal helper failed', { threadId, error: e.message });
      this.ended(t, null);
    });
    proc.on('exit', (code: number | null, signal: NodeJS.Signals | null) => this.ended(t, code ?? (signal ? null : 0)));
    this.byThread.set(threadId, t);
    this.byId.set(t.id, t);
    this.attach(t, sink);
    this.deps.log?.info('terminal started', { threadId, terminalId: t.id });
    return this.result(t);
  }

  private result(t: Term): OpenResult {
    this.flush(t);
    return {
      terminalId: t.id,
      replay: Buffer.concat(t.replay).toString('base64'),
      alive: t.alive,
      startedAt: t.startedAt,
      ...(t.alive ? {} : { exitCode: t.exitCode ?? null, reason: t.ending ?? 'exited' }),
    };
  }

  private attach(t: Term, sink: TerminalSink): void {
    // Output already queued belongs to whoever had it, so the new one
    // gets it in the replay instead of twice.
    this.flush(t);
    t.sink = sink;
  }

  private letGo(t: Term): void {
    this.flush(t);
    t.sink = undefined;
    t.detachedAt = this.now();
  }

  private output(t: Term, b: Buffer): void {
    t.replay.push(b);
    t.replaySize += b.length;
    const max = this.deps.replayBytes ?? REPLAY_BYTES;
    while (t.replaySize > max && t.replay.length) {
      const over = t.replaySize - max;
      const first = t.replay[0]!;
      if (first.length <= over) {
        t.replay.shift();
        t.replaySize -= first.length;
      } else {
        t.replay[0] = first.subarray(over);
        t.replaySize -= over;
      }
    }
    if (!t.sink) return;
    t.pending.push(b);
    t.pendingSize += b.length;
    if (t.pendingSize >= MAX_MESSAGE_BYTES) this.flush(t);
    else t.flushTimer ??= setTimeout(() => this.flush(t), this.deps.flushMs ?? FLUSH_MS);
  }

  private flush(t: Term): void {
    if (t.flushTimer) clearTimeout(t.flushTimer);
    t.flushTimer = undefined;
    if (!t.pendingSize) return;
    const all = Buffer.concat(t.pending);
    t.pending = [];
    t.pendingSize = 0;
    if (!t.sink) return;
    for (let i = 0; i < all.length; i += MAX_MESSAGE_BYTES) {
      t.sink.send({ method: 'terminal.output', params: { terminalId: t.id, threadId: t.threadId, data: all.subarray(i, i + MAX_MESSAGE_BYTES).toString('base64') } });
    }
  }

  // This side ends a shell: closing the helper's stdin hangs it up, and the
  // helper's exit reports it. `wait` false forgets it at once (a restart).
  private end(t: Term, reason: TerminalEndReason, wait: boolean): void {
    t.ending ??= reason;
    try {
      t.proc?.stdin?.end();
    } catch {
      // already closed
    }
    // A helper that doesn't go is made to.
    const proc = t.proc;
    setTimeout(() => {
      if (t.alive) proc?.kill('SIGKILL');
    }, 3000).unref?.();
    if (!wait) {
      this.ended(t, null);
      this.forget(t);
    }
  }

  private ended(t: Term, exitCode: number | null): void {
    if (!t.alive) return;
    t.alive = false;
    t.exitCode = exitCode;
    this.flush(t);
    const reason = t.ending ?? 'exited';
    t.sink?.send({ method: 'terminal.exited', params: { terminalId: t.id, threadId: t.threadId, exitCode, reason } });
    this.deps.log?.info('terminal ended', { threadId: t.threadId, terminalId: t.id, reason, exitCode });
  }

  private forget(t: Term): void {
    this.byId.delete(t.id);
    if (this.byThread.get(t.threadId) === t) this.byThread.delete(t.threadId);
  }

  private mustLive(terminalId: string): Term {
    const t = this.byId.get(terminalId);
    if (!t || !t.alive) throw notFound();
    return t;
  }

  private helper(): Promise<string> {
    if (this.deps.helper) return this.deps.helper();
    this.helperPath ??= buildHelper(this.deps.home).catch((e) => {
      this.helperPath = undefined;
      throw e;
    });
    return this.helperPath;
  }
}

function notFound(): RpcError {
  return new RpcError(ErrorCode.threadNotFound, 'No such terminal: it has ended.');
}

function clampSize(n: number): number {
  return Math.max(1, Math.min(1000, Math.round(Number.isFinite(n) ? n : 80)));
}

export function frame(type: 0 | 1, payload: Buffer): Buffer {
  const head = Buffer.alloc(5);
  head[0] = type;
  head.writeUInt32BE(payload.length, 1);
  return Buffer.concat([head, payload]);
}

export const helperSource = (): string => fileURLToPath(new URL('./terminal/pty-helper.c', import.meta.url));

/** Compiles the helper into the data dir, named by its source's hash, unless it's there already. */
export async function buildHelper(home: string, source = helperSource()): Promise<string> {
  const hash = crypto.createHash('sha256').update(fs.readFileSync(source)).digest('hex').slice(0, 16);
  const out = path.join(home, 'tools', 'pty-helper', `pty-helper-${hash}`);
  if (fs.existsSync(out)) return out;
  fs.mkdirSync(path.dirname(out), { recursive: true });
  const tmp = `${out}.${process.pid}.tmp`;
  const libs = process.platform === 'darwin' ? [] : ['-lutil'];
  await new Promise<void>((resolve, reject) => {
    execFile('cc', ['-O', '-o', tmp, source, ...libs], { timeout: BUILD_TIMEOUT_MS }, (err, _out, stderr) => {
      if (err) {
        fs.rmSync(tmp, { force: true });
        const missing = (err as NodeJS.ErrnoException).code === 'ENOENT' || /xcode-select|command line tools/i.test(String(stderr));
        reject(new RpcError(ErrorCode.other, missing
          ? 'The terminal needs a C compiler to set itself up. Install Xcode\'s command line tools (xcode-select --install) and try again.'
          : `The terminal's helper wouldn't build: ${String(stderr).trim().slice(0, 300) || err.message}`));
        return;
      }
      fs.renameSync(tmp, out);
      resolve();
    });
  });
  return out;
}
