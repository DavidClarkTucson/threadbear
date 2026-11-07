// runner/scripts/deploy-backend.mjs on a Mac with a runner key: only the
// main Mac's runner deploys the backend, even with a deploy key left over
// from an older setup (docs/protocol.md, "Runner keys").
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { tmpDir } from '../helpers/fakes.ts';

test("a Mac with a runner key doesn't deploy the backend, deploy key or not", () => {
  const home = tmpDir();
  fs.mkdirSync(path.join(home, 'secrets'), { recursive: true });
  fs.writeFileSync(path.join(home, 'secrets', 'convex-key'), 'tbr_mine\n');
  fs.writeFileSync(path.join(home, 'secrets', 'convex-deploy.env'), 'CONVEX_DEPLOY_KEY=prod:opulent-moose-249|x\n');
  fs.writeFileSync(path.join(home, 'runner.json'), JSON.stringify({ sync: { url: 'https://opulent-moose-249.ap-southeast-2.convex.cloud' } }));
  const repo = path.resolve(import.meta.dirname, '../../..');
  const r = spawnSync(process.execPath, [path.join(repo, 'runner/scripts/deploy-backend.mjs'), '--home', home, '--repo', repo], { encoding: 'utf8', env: { ...process.env, PATH: '/usr/bin:/bin' } });
  assert.equal(r.status, 0, r.stderr);
  assert.match(r.stdout, /runner key of its own/);
  assert.equal(fs.existsSync(path.join(home, 'backend.json')), false, 'nothing deployed');
});

function dryRun(keyFile: string, syncUrl: string) {
  const home = tmpDir();
  fs.mkdirSync(path.join(home, 'secrets'), { recursive: true });
  fs.writeFileSync(path.join(home, 'secrets', 'convex-deploy.env'), keyFile);
  fs.writeFileSync(path.join(home, 'runner.json'), JSON.stringify({ sync: { url: syncUrl } }));
  const repo = path.resolve(import.meta.dirname, '../../..');
  return spawnSync(process.execPath, [path.join(repo, 'runner/scripts/deploy-backend.mjs'), '--home', home, '--repo', repo, '--dry-run'], { encoding: 'utf8', env: { ...process.env, PATH: '/usr/bin:/bin' } });
}

test('a self-hosted backend deploys with its URL and admin key, only to the one the runner mirrors to', () => {
  const key = 'CONVEX_SELF_HOSTED_URL=https://convex.example.com\nCONVEX_SELF_HOSTED_ADMIN_KEY=convex-self-hosted|0123abcd\n';
  const ok = dryRun(key, 'https://convex.example.com/');
  assert.equal(ok.status, 0, ok.stderr);
  assert.match(ok.stdout, /Would deploy the functions and website at \w+ to https:\/\/convex\.example\.com\./);
  assert.doesNotMatch(ok.stdout + ok.stderr, /0123abcd/, 'the key is never printed');
  const other = dryRun(key, 'https://x.convex.cloud');
  assert.equal(other.status, 1);
  assert.match(other.stderr, /is for https:\/\/convex\.example\.com, but the runner mirrors to https:\/\/x\.convex\.cloud/);
  const half = dryRun('CONVEX_SELF_HOSTED_URL=https://convex.example.com\n', 'https://convex.example.com');
  assert.equal(half.status, 1);
  assert.match(half.stderr, /no CONVEX_SELF_HOSTED_ADMIN_KEY= line/);
});
