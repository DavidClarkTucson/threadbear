// A runner.json that doesn't parse is never written over (config.ts,
// RunnerJsonError). Before 9 Nov one trailing comma made the next start
// rewrite it with only a new runner id, the port and the default folder.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { RunnerJsonError, loadConfig, peekRunnerJson, readRunnerJson, recordListening, writeRunnerJson } from '../../src/config.ts';
import { ensureRunnerId } from '../../src/sync.ts';
import { experimentOn } from '../../src/experiments.ts';
import { markConfigStop } from '../../src/release.ts';
import { tmpDir } from '../helpers/fakes.ts';

const RUNNER = path.resolve(import.meta.dirname, '..', '..');
const BROKEN = '{\n  "sync": { "runnerId": "abc123def456", "url": "https://x.convex.cloud" },\n  "experiments": { "labReview": true },\n  "accounts": [{ "id": "claude-main" }],\n}\n';

function brokenHome(text = BROKEN): { home: string; file: string } {
  const home = tmpDir();
  const file = path.join(home, 'runner.json');
  fs.writeFileSync(file, text);
  return { home, file };
}

test('a runner.json that does not parse throws, naming the file and what is wrong', () => {
  const { home, file } = brokenHome();
  assert.throws(
    () => readRunnerJson(home),
    (e: unknown) => e instanceof RunnerJsonError && e.file === file && e.message.includes(file) && /JSON/.test(e.problem),
  );
});

test('nothing writes over a broken runner.json, and its runner id is kept', () => {
  const { home, file } = brokenHome();
  assert.throws(() => ensureRunnerId(home), RunnerJsonError);
  assert.throws(() => recordListening(home, 4999, '/x'), RunnerJsonError);
  assert.throws(() => writeRunnerJson(home, { port: 1 }), RunnerJsonError, 'even a caller that never read it first');
  assert.throws(() => loadConfig({ THREADBEAR_HOME: home, THREADBEAR_PORT: '0' }), RunnerJsonError);
  assert.equal(fs.readFileSync(file, 'utf8'), BROKEN, 'byte for byte as it was');
});

test('empty, unreadable-as-object and non-object files count as broken too', () => {
  for (const text of ['', '  \n', '[1, 2]', 'null', '"text"', '42']) {
    const { home, file } = brokenHome(text);
    assert.throws(() => readRunnerJson(home), RunnerJsonError, JSON.stringify(text));
    assert.throws(() => recordListening(home, 4999, '/x'), RunnerJsonError);
    assert.equal(fs.readFileSync(file, 'utf8'), text);
  }
  const { home } = brokenHome('');
  assert.throws(() => readRunnerJson(home), /it is empty/);
});

test('a missing runner.json is a fresh start: read as {}, and made on first write', () => {
  const home = tmpDir();
  assert.deepEqual(readRunnerJson(home), {});
  const id = ensureRunnerId(home);
  recordListening(home, 4999, '/x');
  assert.deepEqual(readRunnerJson(home), { sync: { runnerId: id }, port: 4999, defaults: { cwd: '/x' } });
});

test('read-only lookups keep the last good copy while the file is broken mid-run', () => {
  const home = tmpDir();
  const file = path.join(home, 'runner.json');
  fs.writeFileSync(file, JSON.stringify({ experiments: { someExperiment: true } }));
  assert.equal(experimentOn(home, 'someExperiment'), true);
  fs.writeFileSync(file, '{ "experiments": { "someExperiment": true }, }');
  assert.equal(experimentOn(home, 'someExperiment'), true, 'the last good copy, not {}');
  assert.deepEqual(peekRunnerJson(home), { experiments: { someExperiment: true } });
  // What peek hands out is a copy: changing it changes nothing kept.
  peekRunnerJson(home).experiments!.someExperiment = false;
  assert.equal(experimentOn(home, 'someExperiment'), true);
  // A home never read cleanly in this process peeks as {}.
  assert.deepEqual(peekRunnerJson(brokenHome().home), {});
});

test('the runner refuses to start on a broken runner.json, says why, and leaves the file alone', () => {
  const { home, file } = brokenHome();
  const r = spawnSync(process.execPath, [path.join(RUNNER, 'src', 'main.ts')], {
    env: { ...process.env, THREADBEAR_HOME: home, THREADBEAR_PORT: '0', THREADBEAR_SENTRY: 'off', THREADBEAR_SUPERVISED: '' },
    encoding: 'utf8',
    timeout: 60_000,
  });
  assert.equal(r.status, 78, r.stderr);
  assert.match(r.stderr, /runner failed to start: .*runner\.json isn't valid JSON/);
  assert.ok(r.stderr.includes(file), 'names the file');
  assert.doesNotMatch(r.stderr, /\n\s+at /, 'no stack trace');
  assert.equal(fs.readFileSync(file, 'utf8'), BROKEN);
  assert.deepEqual(fs.readdirSync(home), ['runner.json'], 'made nothing else either');
});

test('a boot stopped by a broken runner.json does not count towards a rollback', async () => {
  const home = tmpDir();
  const launch = await import(pathToFileURL(path.join(RUNNER, 'launch.mjs')).href);
  const dir = path.join(home, 'runner');
  fs.mkdirSync(dir, { recursive: true });
  const stateFile = path.join(dir, 'state.json');
  const now = Date.parse('2026-11-09T10:00:00Z');
  const ago = (s: number) => new Date(now - s * 1000).toISOString();
  const state = { current: 'B', previous: 'A', lastGood: 'A', boots: [{ release: 'B', at: ago(40), pid: 101 }, { release: 'B', at: ago(20), pid: 102 }] };
  fs.writeFileSync(stateFile, JSON.stringify(state));
  // Without the mark, the next boot is the third failed one and goes back.
  assert.ok(launch.planBoot(state, now, () => true).rolledBack);
  assert.equal(await markConfigStop(home, 101, 0), true);
  assert.equal(await markConfigStop(home, 102, 0), true);
  const after = JSON.parse(fs.readFileSync(stateFile, 'utf8'));
  assert.deepEqual(after.boots.map((b: { clean?: boolean }) => b.clean), [true, true]);
  assert.equal(launch.planBoot(after, now, () => true).rolledBack, null, 'stays on B');
  assert.equal(launch.planBoot(after, now, () => true).start, 'B');
});

test('markConfigStop waits for launch.mjs to record the boot, and touches only its own', async () => {
  const home = tmpDir();
  const dir = path.join(home, 'runner');
  fs.mkdirSync(dir, { recursive: true });
  const stateFile = path.join(dir, 'state.json');
  fs.writeFileSync(stateFile, JSON.stringify({ current: 'B', lastGood: 'A', boots: [{ release: 'B', at: 'x', pid: 7 }] }));
  let slept = 0;
  const sleep = async () => {
    slept++;
    // launch.mjs writes the boot just after the spawn.
    if (slept === 2) fs.writeFileSync(stateFile, JSON.stringify({ current: 'B', lastGood: 'A', boots: [{ release: 'B', at: 'x', pid: 7 }, { release: 'B', at: 'y', pid: 8 }] }));
  };
  assert.equal(await markConfigStop(home, 8, 1000, sleep), true);
  const after = JSON.parse(fs.readFileSync(stateFile, 'utf8'));
  assert.deepEqual(after.boots, [{ release: 'B', at: 'x', pid: 7 }, { release: 'B', at: 'y', pid: 8, clean: true }]);
  // No state.json (run from source): nothing to mark, nothing made.
  const bare = tmpDir();
  assert.equal(await markConfigStop(bare, 8, 0), false);
  assert.deepEqual(fs.readdirSync(bare), []);
});
