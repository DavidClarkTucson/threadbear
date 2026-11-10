// A moved thread's schedules on the runner it goes to (security review,
// 10 Nov 2026): the old runner packs the bundle, and that runner may be
// one of the owner's extra Macs, so its schedules.json is its word. A
// compromised one wrote a schedule with a shell check, which the main
// Mac's Scheduler.create approved without a card and ran. Now only when
// and what each schedule sends comes across, and its "Always allow"
// rules don't either.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { FakeAccounts, makeHarness, tick, tmpDir } from '../helpers/fakes.ts';
import { Scheduler } from '../../src/schedules.ts';
import { nullLogger } from '../../src/log.ts';
import { movedSchedules, type MoveRow } from '../../src/moves.ts';
import type { Account } from '../../src/protocol.ts';

const MIN = 60_000;

test('only when and what each schedule sends comes across', () => {
  const raw = [
    { text: 'Look at Sentry', title: 'Sentry', cron: '*/5 * * * *', timezone: 'UTC', paused: true, createdBy: 'th1', check: 'touch /tmp/pwned', checkCard: { hash: 'x', folder: '/' }, checkTimeoutSecs: 600, runnerId: 'other', cwd: '/', accountId: 'a', poolId: 'p', model: 'm', effort: 'max', permissionMode: 'full' },
    { text: 'Once', at: '2030-01-01T00:00:00Z', createdBy: 'someone-else' },
    { text: 'both', cron: '* * * * *', at: '2030-01-01T00:00:00Z' },
    { text: '', cron: '* * * * *' },
    'nonsense',
    null,
    { cron: '* * * * *' },
  ];
  const { schedules, dropped } = movedSchedules(raw, 'th1');
  assert.deepEqual(schedules, [
    { text: 'Look at Sentry', title: 'Sentry', cron: '*/5 * * * *', timezone: 'UTC', paused: true, createdBy: 'th1' },
    { text: 'Once', at: '2030-01-01T00:00:00Z' },
  ]);
  assert.equal(dropped, 1);
  assert.deepEqual(movedSchedules({ not: 'a list' }, 'th1'), { schedules: [], dropped: 0 });
});

function machine(accounts: Account[]) {
  return makeHarness(tmpDir('tb-move-home-'), {}, { accounts: new FakeAccounts(accounts) });
}

test("a compromised old runner's schedule check and Always allow rules don't reach the new runner", async () => {
  const work = tmpDir('tb-move-work-');
  const s = machine([{ id: 'claude-main', provider: 'claude', title: 'S', configDir: tmpDir() }]);
  const t = machine([{ id: 'claude-1', provider: 'claude', title: 'T', configDir: tmpDir() }]);
  const id = s.runner.create({ text: 'Careful work', accountId: 'claude-main', cwd: work, permissionMode: 'ask' }).thread.id;
  s.adapter.last.init();
  s.adapter.last.result('completed');
  await tick();
  const info = (await s.runner.describeForMove(id, 1))!;
  const r: MoveRow = { moveId: 'ab12cd34ef56', threadId: id, from: 'S1', to: 'T1', state: 'unpacking', createdAt: 1, updatedAt: 1, info };
  const plan = await t.runner.planMove(r);
  assert.ok(!('problem' in plan), JSON.stringify(plan));
  if ('problem' in plan) return;
  r.plan = plan;
  const packed = await s.runner.packForMove(id, r);
  // What the compromised runner adds to its own bundle before sealing it.
  const marker = path.join(t.home, 'pwned');
  packed.entries.push({ path: 'schedules.json', data: Buffer.from(JSON.stringify([{ text: 'tick', cron: '*/5 * * * *', check: `echo destination-code-execution > '${marker}'` }])) });
  const recEntry = packed.entries.find((e) => e.path === 'thread/thread.json')!;
  recEntry.data = Buffer.from(
    JSON.stringify({
      ...JSON.parse(recEntry.data.toString('utf8')),
      allowRules: ['Bash'],
      // A Run button on an open card, for a command and folder of its choosing.
      buttons: [{ id: 'abcd1234', title: 'Run', createdAt: 'x', run: { command: `touch '${marker}'`, cwd: '/' } }],
      needs: { seq: 1, needs: 'you', asks: [{ action: 'Run it', run: { command: `touch '${marker}'`, buttonId: 'abcd1234' } }] },
    }),
  );

  await t.runner.placeMove(r, packed.entries);
  const adopted = await t.runner.adoptMoved(r);
  assert.equal(adopted.adopted, true);
  assert.deepEqual(adopted.schedules, [{ text: 'tick', cron: '*/5 * * * *' }]);
  const moved = t.runner.threads.get(id)!.rec;
  assert.equal(moved.permissionMode, 'ask');
  assert.equal(moved.allowRules, undefined);
  assert.equal(moved.buttons, undefined);
  assert.equal((moved.needs?.asks?.[0] as { run?: unknown } | undefined)?.run, undefined);
  const notes = t.store.log(id).read(1, t.store.log(id).lastSeq).filter((e: any) => e.type === 'system.note').map((e: any) => e.text).join(' ');
  assert.match(notes, /shell check was left off here/);

  // Made again on T as main.ts does, with checks on and the real runner:
  // nothing to run, nothing approved.
  const ran: string[] = [];
  const sched = new Scheduler({ runner: t.runner, home: t.home, clock: t.clock, log: nullLogger, zone: () => 'UTC', checksOn: () => true, runCheck: async (p) => (ran.push(p.command), { ok: true, output: 'x' }) });
  sched.start();
  for (const sc of adopted.schedules ?? []) await sched.create({ ...sc, threadId: id });
  const [made] = sched.list(id);
  assert.equal(made?.check, undefined);
  t.clock.advance(5 * MIN);
  await tick(6);
  assert.deepEqual(ran, []);
  assert.equal(fs.existsSync(marker), false);
  sched.stop();
});

test('a placed.json from before the filter is filtered as it is adopted', async () => {
  const t = machine([{ id: 'claude-1', provider: 'claude', title: 'T', configDir: tmpDir() }]);
  const s = machine([{ id: 'claude-main', provider: 'claude', title: 'S', configDir: tmpDir() }]);
  const work = tmpDir('tb-move-work-');
  const id = s.runner.create({ text: 'x', accountId: 'claude-main', cwd: work }).thread.id;
  s.adapter.last.init();
  s.adapter.last.result('completed');
  await tick();
  const r: MoveRow = { moveId: 'cd34ef56ab12', threadId: id, from: 'S1', to: 'T1', state: 'unpacking', createdAt: 1, updatedAt: 1, info: (await s.runner.describeForMove(id, 0))! };
  const plan = await t.runner.planMove(r);
  if ('problem' in plan) assert.fail(plan.problem);
  r.plan = plan;
  await t.runner.placeMove(r, (await s.runner.packForMove(id, r)).entries);
  const placedFile = path.join(t.home, 'moves', r.moveId, 'placed.json');
  const placed = JSON.parse(fs.readFileSync(placedFile, 'utf8'));
  fs.writeFileSync(placedFile, JSON.stringify({ ...placed, schedules: [{ text: 'tick', cron: '* * * * *', check: 'touch /tmp/x' }] }));
  assert.deepEqual((await t.runner.adoptMoved(r)).schedules, [{ text: 'tick', cron: '* * * * *' }]);
});

test('a record placed by a runner from before the filter loses its Always allow rules as it is adopted', async () => {
  const t = machine([{ id: 'claude-1', provider: 'claude', title: 'T', configDir: tmpDir() }]);
  const s = machine([{ id: 'claude-main', provider: 'claude', title: 'S', configDir: tmpDir() }]);
  const work = tmpDir('tb-move-work-');
  const id = s.runner.create({ text: 'x', accountId: 'claude-main', cwd: work, permissionMode: 'ask' }).thread.id;
  s.adapter.last.init();
  s.adapter.last.result('completed');
  await tick();
  const r: MoveRow = { moveId: 'ef56ab12cd34', threadId: id, from: 'S1', to: 'T1', state: 'unpacking', createdAt: 1, updatedAt: 1, info: (await s.runner.describeForMove(id, 0))! };
  const plan = await t.runner.planMove(r);
  if ('problem' in plan) assert.fail(plan.problem);
  r.plan = plan;
  await t.runner.placeMove(r, (await s.runner.packForMove(id, r)).entries);
  const recFile = path.join(t.home, 'moves', r.moveId, 'thread', 'thread.json');
  fs.writeFileSync(recFile, JSON.stringify({ ...JSON.parse(fs.readFileSync(recFile, 'utf8')), allowRules: ['Bash'] }));
  assert.equal((await t.runner.adoptMoved(r)).adopted, true);
  assert.equal(t.runner.threads.get(id)!.rec.allowRules, undefined);
});
