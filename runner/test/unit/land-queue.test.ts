// tb land's queue (docs/thread-worktrees.md, "Landing together"): on 12 Nov
// a dozen landings each waited for a check of their own, 40 minutes and
// more. Now everyone waiting goes into the next check together.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { land, landTargetFromGit, type LandTarget } from '../../src/worktrees.ts';
import { joinQueue, queueDir, setRunning, waiting, waitingLine } from '../../src/land-queue.ts';
import { capture, commit, makeBranch, makeRepo, sh, until, write } from '../helpers/worktrees-harness.ts';

// A check that writes down which of the files it saw, and holds the first
// landing (a.txt alone) until the test lets it go, so the rest queue up.
function slowCheck(root: string, failIf = 'never-there.txt'): string {
  return `ls *.txt | tr '\\n' ' ' >> '${root}/checks'; echo >> '${root}/checks'; ` +
    `if test -f a.txt && ! test -f b.txt; then touch '${root}/checking'; while ! test -f '${root}/release'; do sleep 0.05; done; fi; ` +
    `! test -f ${failIf}`;
}

function branches(main: string, names: string[]): LandTarget[] {
  return names.map((n) => {
    const t = makeBranch(main, `${n}0000000`);
    commit(t.path, { [`${n}.txt`]: `${n}\n` }, `Add ${n}`);
    return t;
  });
}

async function queuedBehindA(root: string, main: string, others: LandTarget[], a: LandTarget, check: string) {
  const first = land({ ...a, check }, capture().opts);
  await until(() => fs.existsSync(path.join(root, 'checking')), "a's check");
  const caps = others.map(() => capture());
  const rest = others.map((t, i) => land({ ...t, check }, caps[i].opts));
  try {
    await until(() => caps.every((c) => c.out.some((l) => l.startsWith('Waiting for another landing'))), 'the others waiting');
  } finally {
    write(root, { release: '' });
  }
  return { first: await first, codes: await Promise.all(rest), caps };
}

test('everyone waiting lands in one check, each branch moved to what landed', async () => {
  const { root, main } = makeRepo();
  const [a, b, c, d] = branches(main, ['a', 'b', 'c', 'd']);
  const check = slowCheck(root);
  const { first, codes, caps } = await queuedBehindA(root, main, [b, c, d], a, check);
  assert.equal(first, 0);
  assert.deepEqual(codes, [0, 0, 0], caps.map((c) => c.err.join('\n')).join('\n'));
  // a's check, then one for the other three.
  assert.deepEqual(fs.readFileSync(path.join(root, 'checks'), 'utf8').trim().split('\n').map((l) => l.trim()), ['a.txt', 'a.txt b.txt c.txt d.txt']);
  for (const n of ['a', 'b', 'c', 'd']) assert.ok(fs.existsSync(path.join(main, `${n}.txt`)));
  assert.equal(sh(main, 'git log --format=%s main | head -4 | sort | tr "\\n" " "'), 'Add a Add b Add c Add d');
  // Each branch is now on main, so its next tb land lands only what's new.
  for (const t of [b, c, d]) {
    assert.equal(sh(t.path, `git log --oneline main..${t.branch}`), '');
    assert.equal(sh(t.path, 'git rev-parse --abbrev-ref HEAD'), t.branch);
    assert.equal(sh(t.path, 'git status --porcelain'), '');
  }
  const all = caps.map((c) => c.out.join('\n'));
  assert.equal(all.filter((o) => /In the same check, and landed with it: /.test(o)).length, 1, all.join('\n---\n'));
  assert.equal(all.filter((o) => /, in one check with /.test(o)).length, 2, all.join('\n---\n'));
  // Each says what was its own.
  for (const [i, n] of ['b', 'c', 'd'].entries()) assert.match(all[i], new RegExp(`Add ${n}`));
  assert.ok(all.some((o) => /the check for thread\/a0000000 has been running/.test(o)), all.join('\n---\n'));
  assert.deepEqual(waiting(path.join(main, '.git')), [], 'nobody left in the line');
});

test('a group check that fails is narrowed down, and only the one that broke it is refused', async () => {
  const { root, main } = makeRepo();
  const [a, b, c, d] = branches(main, ['a', 'b', 'c', 'd']);
  commit(c.path, { 'bad.txt': 'bad\n' }, 'Break it');
  const check = slowCheck(root, 'bad.txt');
  const { first, codes, caps } = await queuedBehindA(root, main, [b, c, d], a, check);
  assert.equal(first, 0);
  assert.deepEqual(codes, [0, 1, 0], caps.map((x) => x.out.concat(x.err).join('\n')).join('\n---\n'));
  assert.match(caps[1].err.join('\n'), /^The check failed \(exit 1\), so main wasn't touched/);
  assert.ok(!fs.existsSync(path.join(main, 'bad.txt')));
  for (const n of ['a', 'b', 'd']) assert.ok(fs.existsSync(path.join(main, `${n}.txt`)));
  assert.ok(caps.some((x) => x.out.some((l) => /Checking them in smaller groups/.test(l))));
  // Every check that passed had no bad.txt in it: main only moved to those.
  const checks = fs.readFileSync(path.join(root, 'checks'), 'utf8').trim().split('\n');
  assert.ok(checks.length <= 6, checks.join('\n'));
  assert.equal(sh(c.path, 'git rev-parse --abbrev-ref HEAD'), c.branch, 'the failed one is back on its branch');
});

test("a queued landing whose tb has gone is dropped from the line, not landed", async () => {
  const { main } = makeRepo();
  const [a, b] = branches(main, ['a', 'b']);
  const common = path.join(main, '.git');
  const gone = joinQueue(common, { branch: b.branch, path: b.path, base: sh(main, 'git rev-parse main'), commit: sh(b.path, 'git rev-parse HEAD') });
  const file = path.join(queueDir(common), `${gone.id}.json`);
  fs.writeFileSync(file, JSON.stringify({ ...gone, pid: 2 ** 22 + 12345 }));
  assert.equal(await land({ ...a, check: 'true' }, capture().opts), 0);
  assert.ok(!fs.existsSync(path.join(main, 'b.txt')));
  assert.ok(!fs.existsSync(file));
});

test('the waiting line says where it is in the line and roughly how long', () => {
  const { main } = makeRepo();
  const common = path.join(main, '.git');
  const times = path.join(common, 'threadbear-land-times.log');
  fs.writeFileSync(times, [300, 360, 420].map((s) => JSON.stringify({ ok: true, seconds: s })).join('\n') + '\n');
  const now = Date.now();
  setRunning(common, { startedAt: now - 120_000, branches: ['thread/x', 'thread/y'] });
  const ahead = joinQueue(common, { branch: 'thread/z', path: '/z', base: 'b', commit: 'c' });
  const me = joinQueue(common, { branch: 'thread/me', path: '/me', base: 'b', commit: 'c' });
  if (me.at === ahead.at) me.at += 1;
  const line = waitingLine(common, times, me, now);
  assert.equal(line, ': the check for thread/x and 1 more has been running 2 min; 1 more waiting ahead of this one, all checked together with it next, so roughly 10 min to go (a check takes about 6 min here).');
  setRunning(common, undefined);
  assert.equal(waitingLine(common, times, { ...ahead, at: ahead.at - 1 }, now), '...');
});

test('a landing that asks for a different check is never put in another one', async () => {
  const { root, main } = makeRepo();
  const [a, b, c] = branches(main, ['a', 'b', 'c']);
  const check = slowCheck(root);
  const first = land({ ...a, check }, capture().opts);
  await until(() => fs.existsSync(path.join(root, 'checking')), "a's check");
  const cb = capture();
  const cc = capture();
  const rest = [land({ ...b, check: `${check}; true` }, cb.opts), land({ ...c, check }, cc.opts)];
  try {
    await until(() => [cb, cc].every((x) => x.out.some((l) => l.startsWith('Waiting for another landing'))), 'both waiting');
  } finally {
    write(root, { release: '' });
  }
  assert.equal(await first, 0);
  assert.deepEqual(await Promise.all(rest), [0, 0]);
  assert.equal(fs.readFileSync(path.join(root, 'checks'), 'utf8').trim().split('\n').length, 3, 'each checked with its own check');
});

test('two tb lands in one worktree take turns rather than abort each other', async () => {
  const { main } = makeRepo();
  const [a] = branches(main, ['a']);
  commit(main, { 'other.txt': 'x\n' }, 'Main moved');
  const one = capture();
  const two = capture();
  const codes = await Promise.all([land({ ...a, check: 'sleep 0.3' }, one.opts), land({ ...a, check: 'sleep 0.3' }, two.opts)]);
  assert.deepEqual(codes, [0, 0], one.err.concat(two.err).join('\n'));
  assert.ok(one.out.concat(two.out).some((l) => /Another tb land is running in this worktree/.test(l)));
  assert.ok(fs.existsSync(path.join(main, 'a.txt')));
});

test('a worktree left detached by a tb killed during a group check is put back by the next tb land', async () => {
  const { main } = makeRepo();
  const [a] = branches(main, ['a']);
  const own = sh(a.path, 'git rev-parse HEAD');
  const gitDir = sh(a.path, 'git rev-parse --absolute-git-dir');
  fs.writeFileSync(path.join(gitDir, 'threadbear-land-checking.json'), JSON.stringify({ branch: a.branch, own }));
  sh(a.path, 'git checkout -q --detach main');
  const c = capture();
  assert.equal(await land({ ...a, check: 'true' }, c.opts), 0, c.err.join('\n'));
  assert.match(c.out[0], /checked out again as it was/);
  assert.equal(sh(main, 'git rev-parse main'), own);
  assert.ok(!fs.existsSync(path.join(gitDir, 'threadbear-land-checking.json')));
});

test('a commit made empty on purpose lands with its group, as a rebase would keep it', async () => {
  const { root, main } = makeRepo();
  const [a, b, c] = branches(main, ['a', 'b', 'c']);
  // main will have an empty commit too, which --cherry-pick would match.
  sh(a.path, 'git commit -q --allow-empty -m "Empty on a"');
  sh(b.path, 'git commit -q --allow-empty -m "Empty on b"');
  const check = slowCheck(root);
  const { first, codes } = await queuedBehindA(root, main, [b, c], a, check);
  assert.equal(first, 0);
  assert.deepEqual(codes, [0, 0]);
  assert.match(sh(main, 'git log --format=%s main'), /Empty on b/);
  assert.equal(fs.readFileSync(path.join(root, 'checks'), 'utf8').trim().split('\n').length, 2, 'b and c in one check');
});

test('tb land outside a thread finds the branch of a worktree a killed group check left detached', async () => {
  const { main } = makeRepo();
  const [a] = branches(main, ['a']);
  const gitDir = sh(a.path, 'git rev-parse --absolute-git-dir');
  fs.writeFileSync(path.join(gitDir, 'threadbear-land-checking.json'), JSON.stringify({ branch: a.branch, own: sh(a.path, 'git rev-parse HEAD') }));
  sh(a.path, 'git checkout -q --detach main');
  const found = await landTargetFromGit(a.path);
  assert.equal(found.target?.branch, a.branch, found.reason ?? "");
  // A failed put-back keeps the marker for next time.
  write(a.path, { 'a.txt': 'local edit in the way\n' });
  assert.equal(await land({ ...a, check: 'true' }, capture().opts), 1);
  assert.ok(fs.existsSync(path.join(gitDir, 'threadbear-land-checking.json')));
});

test("a workflow red on GitHub for another's files leaves that one out of the group, and it hears why", async () => {
  const { root, main } = makeRepo();
  const [a, b, c] = branches(main, ['a', 'b', 'c']);
  const check = slowCheck(root);
  const hold = async (_root: string, files: string[]) =>
    fs.existsSync(path.join(root, 'red')) && files.includes('c.txt') ? 'Held: the workflow for c.txt is red on main.' : undefined;
  const first = land({ ...a, check }, capture().opts);
  await until(() => fs.existsSync(path.join(root, 'checking')), "a's check");
  const cb = capture();
  const cc = capture();
  const rest = [land({ ...b, check }, { ...cb.opts, hold }), land({ ...c, check }, { ...cc.opts, hold })];
  try {
    await until(() => [cb, cc].every((x) => x.out.some((l) => l.startsWith('Waiting for another landing'))), 'both waiting');
    write(root, { red: '' });
  } finally {
    write(root, { release: '' });
  }
  assert.equal(await first, 0);
  assert.deepEqual(await Promise.all(rest), [0, 1]);
  assert.match(cc.err.join('\n'), /Held: the workflow for c\.txt is red on main\./);
  assert.ok(fs.existsSync(path.join(main, 'b.txt')) && !fs.existsSync(path.join(main, 'c.txt')));
});
