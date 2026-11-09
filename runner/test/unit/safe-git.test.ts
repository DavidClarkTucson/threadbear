import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { tmpDir } from '../helpers/fakes.ts';
import { claudeCommands } from '../../src/claude-trust.ts';
import { trackedNodeModules } from '../../src/tracked-packages.ts';
import { git as worktreeGit } from '../../src/worktrees.ts';
import { changedFiles } from '../../src/move-git.ts';
import { untrustedGitSync } from '../../src/safe-git.ts';

// A folder that arrived with its own .git (a zip, a shared drive), whose
// .git/config names an fsmonitor program. Nothing the runner does before
// the folder is trusted may start it (security review, 9 Nov 2026: the
// Claude settings check's `git ls-files` did, in every mode, before the
// first turn).
function planted(): { repo: string; ran: () => number } {
  const root = tmpDir();
  const repo = path.join(root, 'from-a-zip');
  fs.mkdirSync(repo);
  const g = (...args: string[]) => execFileSync('git', ['-C', repo, ...args], { stdio: 'pipe' });
  g('init', '-q');
  fs.writeFileSync(path.join(repo, 'README.md'), 'hi\n');
  g('add', 'README.md');
  g('-c', 'user.email=t@t', '-c', 'user.name=t', 'commit', '-qm', 'first');
  // Untracked and changed files too, so every listing reads the working tree.
  fs.writeFileSync(path.join(repo, 'untracked.txt'), 'x\n');
  fs.writeFileSync(path.join(repo, 'README.md'), 'changed\n');
  const marker = path.join(root, 'fsmonitor-ran');
  const hook = path.join(root, 'fsmonitor.sh');
  fs.writeFileSync(hook, `#!/bin/sh\necho ran >> '${marker}'\nexit 1\n`, { mode: 0o755 });
  g('config', 'core.fsmonitor', hook);
  const ran = () => (fs.existsSync(marker) ? fs.readFileSync(marker, 'utf8').split('\n').filter(Boolean).length : 0);
  return { repo, ran };
}

test('the planted fsmonitor really runs under plain git, so the checks below mean something', () => {
  const { repo, ran } = planted();
  execFileSync('git', ['-C', repo, 'status', '--porcelain'], { stdio: 'pipe' });
  assert.ok(ran() > 0);
});

test("the Claude settings check lists a folder's files without starting its fsmonitor", () => {
  const { repo, ran } = planted();
  // A skill nested in the folder, so the listing has something to find.
  fs.mkdirSync(path.join(repo, 'sub', '.claude', 'skills', 's'), { recursive: true });
  fs.writeFileSync(path.join(repo, 'sub', '.claude', 'skills', 's', 'SKILL.md'), '---\nname: s\n---\nhi\n');
  claudeCommands(repo, tmpDir());
  claudeCommands(path.join(repo, 'sub'), tmpDir());
  assert.equal(ran(), 0);
});

test("tracked packages, worktree git and a move's changed files leave the fsmonitor off", async () => {
  const { repo, ran } = planted();
  trackedNodeModules(repo);
  const st = await worktreeGit(repo, ['status', '--porcelain', '--untracked-files=all']);
  assert.ok(st.ok, st.stderr);
  assert.match(st.stdout, /untracked\.txt/);
  assert.deepEqual((await changedFiles(repo)).sort(), ['README.md', 'untracked.txt']);
  assert.equal(ran(), 0);
});

test("git config the runner's environment carries can't turn the fsmonitor back on for a listing", () => {
  const { repo, ran } = planted();
  const hook = execFileSync('git', ['-C', repo, 'config', 'core.fsmonitor'], { stdio: 'pipe' }).toString().trim();
  const saved = { ...process.env };
  try {
    Object.assign(process.env, { GIT_CONFIG_COUNT: '1', GIT_CONFIG_KEY_0: 'core.fsmonitor', GIT_CONFIG_VALUE_0: hook, GIT_CONFIG_PARAMETERS: `'core.fsmonitor'='${hook}'` });
    assert.match(untrustedGitSync(repo, ['ls-files', '-o', '--exclude-standard']), /untracked\.txt/);
  } finally {
    for (const k of Object.keys(process.env)) if (!(k in saved)) delete process.env[k];
  }
  assert.equal(ran(), 0);
});
