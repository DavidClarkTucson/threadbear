// The GPUI build folders on a shared host (desktop/scripts/remote.sh). They
// filled the cloud box on 9 Nov, and that box also runs a runner, whose saves
// then failed (THREADBEAR-1R). An age limit alone doesn't help when four
// checkouts are all in use, so low disk has to give room back as well.
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test, type TestContext } from 'node:test';
import assert from 'node:assert/strict';

const SCRIPTS = path.join(import.meta.dirname, '..', '..', '..', 'desktop', 'scripts');
const TRIM = path.join(SCRIPTS, 'trim-remote-cache.sh');
const HOUR = 3_600_000;

// A df that answers from how many target folders are left, so deleting one
// really does free room. Each is worth 6 GiB.
function host(t: TestContext, freeGib: number, folders: Record<string, number>) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tb-remote-cache-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const bin = path.join(root, 'bin');
  const base = path.join(root, 'home', '.cache', 'threadbear-check', 'gpui');
  fs.mkdirSync(bin);
  fs.mkdirSync(base, { recursive: true });
  const now = Date.now();
  for (const [name, hoursAgo] of Object.entries(folders)) {
    const p = path.join(base, name);
    fs.mkdirSync(p);
    fs.utimesSync(p, new Date(now - hoursAgo * HOUR), new Date(now - hoursAgo * HOUR));
  }
  const targets = Object.keys(folders).filter((n) => /^target(-|$)/.test(n)).length;
  fs.writeFileSync(path.join(bin, 'df'), `#!/bin/bash
n=$(ls -d target target-* 2>/dev/null | wc -l)
echo "Filesystem 1024-blocks Used Available Capacity Mounted"
f=$(( (${freeGib} + (${targets} - n) * 6) * 1048576 ))
[ "$f" -ge 0 ] || f=0
echo "fake 0 0 $f 0% /"
`, { mode: 0o755 });
  const env = { ...process.env, PATH: `${bin}:${process.env.PATH}`, HOME: path.join(root, 'home') };
  const left = () => fs.readdirSync(base).filter((n) => /^target(-|$)/.test(n)).sort();
  const trim = (own = 'aaa') => spawnSync('/bin/bash', [TRIM, own], { cwd: base, env, encoding: 'utf8', timeout: 20_000 });
  return { root, bin, base, env, left, trim };
}

const WORK = {
  'src-aaa': 0, 'target-aaa': 0, // this checkout
  'src-bbb': 30, 'target-bbb': 30, // idle the longest
  'src-ccc': 10, 'target-ccc': 10, // idle
  'src-ddd': 0.5, 'target-ddd': 0.5, // building now
};

test('a disk with room is left alone', (t) => {
  const h = host(t, 40, WORK);
  assert.equal(h.trim().status, 0);
  assert.deepEqual(h.left(), ['target-aaa', 'target-bbb', 'target-ccc', 'target-ddd']);
});

test('low on disk, the checkout idle longest goes first, and only until there is room', (t) => {
  const h = host(t, 8, WORK);
  const r = h.trim();
  assert.equal(r.status, 0, r.stderr);
  assert.deepEqual(h.left(), ['target-aaa', 'target-ccc', 'target-ddd']);
  assert.match(r.stderr, /target-bbb/);
});

test("this checkout's folder and one in use are never taken, however low the disk", (t) => {
  const h = host(t, 4, WORK);
  assert.equal(h.trim().status, 0);
  assert.deepEqual(h.left(), ['target-aaa', 'target-ddd']);
});

test('the old shared folder goes before any checkout loses its own', (t) => {
  const h = host(t, 8, { ...WORK, target: 5 });
  assert.equal(h.trim().status, 0);
  assert.deepEqual(h.left(), ['target-aaa', 'target-bbb', 'target-ccc', 'target-ddd']);
});

test("the last-used time is the checkout's copy, as a target folder's own time doesn't move during a build", (t) => {
  const h = host(t, 8, { 'src-aaa': 0, 'target-aaa': 0, 'src-eee': 0.5, 'target-eee': 50 });
  assert.equal(h.trim().status, 0);
  assert.deepEqual(h.left(), ['target-aaa', 'target-eee']);
});

test('a disk still nearly full refuses the build and says why', (t) => {
  const h = host(t, 2, { 'src-aaa': 0, 'target-aaa': 0, 'src-ddd': 0.5, 'target-ddd': 0.5 });
  const r = h.trim();
  assert.equal(r.status, 3);
  assert.match(r.stderr, /2 GiB free/);
  assert.deepEqual(h.left(), ['target-aaa', 'target-ddd']);
});

test('remote.sh stops before copying anything when the host refuses, and marks its target as used', (t) => {
  const h = host(t, 2, {});
  const log = path.join(h.root, 'rsync-ran');
  // An ssh that runs the command here, stdin and all.
  fs.writeFileSync(path.join(h.bin, 'ssh'), `#!/bin/bash\nshift\ncd "$HOME" && eval "$*"\n`, { mode: 0o755 });
  fs.writeFileSync(path.join(h.bin, 'rsync'), `#!/bin/bash\ntouch ${JSON.stringify(log)}\n`, { mode: 0o755 });
  const r = spawnSync('/bin/bash', [path.join(SCRIPTS, 'remote.sh'), 'somehost', '--', 'true'], { env: h.env, encoding: 'utf8', timeout: 20_000 });
  assert.equal(r.status, 3, r.stderr);
  assert.equal(fs.existsSync(log), false, 'nothing copied to a full disk');
  assert.equal(fs.readdirSync(h.base).filter((n) => n.startsWith('target-')).length, 1, 'its target folder exists, so the age limit sees it used');
  const own = fs.readdirSync(h.base).find((n) => n.startsWith('target-'))!;
  assert.ok(Date.now() - fs.statSync(path.join(h.base, own)).mtimeMs < HOUR);
});
