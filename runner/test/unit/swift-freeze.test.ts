// scripts/swift-freeze.sh keeps new code out of the Swift Mac app (app/,
// dock/) now Mike uses the GPUI one (12 Nov 2026): it refuses in his copy,
// and once WARN_FORKS is on, only warns in other people's. Each test plays it
// in a throwaway repo, as git's pre-commit hook and as Claude Code's
// PreToolUse hook.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const script = path.join(path.dirname(fileURLToPath(import.meta.url)), '../../../scripts/swift-freeze.sh');

function repo(origin: string, warnForks = false): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'swift-freeze-'));
  const git = (...args: string[]) => spawnSync('git', args, { cwd: dir, encoding: 'utf8' });
  git('init', '-q', '-b', 'main');
  git('remote', 'add', 'origin', origin);
  fs.mkdirSync(path.join(dir, 'scripts'));
  let text = fs.readFileSync(script, 'utf8');
  if (warnForks) text = text.replace(/^WARN_FORKS=0$/m, 'WARN_FORKS=1');
  fs.writeFileSync(path.join(dir, 'scripts/swift-freeze.sh'), text, { mode: 0o755 });
  return dir;
}

function stage(dir: string, file: string) {
  fs.mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
  fs.writeFileSync(path.join(dir, file), 'x\n');
  spawnSync('git', ['add', file], { cwd: dir });
}

const env = (extra: Record<string, string> = {}) => {
  const e: Record<string, string | undefined> = { ...process.env, ...extra };
  delete e.CLAUDE_PROJECT_DIR;
  delete e.THREADBEAR_SWIFT_OK;
  delete e.THREADBEAR_SWIFT_FREEZE_COPY;
  return { ...e, ...extra };
};

function commit(dir: string, extra: Record<string, string> = {}) {
  return spawnSync('scripts/swift-freeze.sh', ['--commit'], { cwd: dir, encoding: 'utf8', env: env(extra) });
}

function edit(dir: string, file: string, extra: Record<string, string> = {}) {
  const input = JSON.stringify({ tool_name: 'Edit', tool_input: { file_path: path.join(dir, file), old_string: 'a', new_string: 'b' } });
  return spawnSync(path.join(dir, 'scripts/swift-freeze.sh'), ['--tool'], {
    cwd: dir, encoding: 'utf8', input, env: env({ CLAUDE_PROJECT_DIR: dir, ...extra }),
  });
}

const MIKE = 'https://github.com/mikecann/threadbear.git';
const FORK = 'git@github.com:someone/threadbear.git';

test("Mike's copy refuses a commit that adds to the Swift app", () => {
  const dir = repo(MIKE);
  stage(dir, 'app/Sources/Threadbear/New.swift');
  stage(dir, 'desktop/gpui/src/new.rs');
  const r = commit(dir);
  assert.equal(r.status, 2);
  assert.match(r.stderr, /frozen/);
  assert.match(r.stderr, /app\/Sources\/Threadbear\/New\.swift/);
  assert.doesNotMatch(r.stderr, /new\.rs/);
});

test("Mike's copy refuses BearDock too, and an ssh origin counts as his", () => {
  const dir = repo('git@github.com:mikecann/threadbear.git');
  stage(dir, 'dock/Sources/BearDock/Drag.swift');
  assert.equal(commit(dir).status, 2);
});

test('the GPUI app, the shared test tools and deletions go through', () => {
  const dir = repo(MIKE);
  stage(dir, 'desktop/gpui/src/new.rs');
  stage(dir, 'app/Tools/mock-runner/server.ts');
  stage(dir, 'app/Tools/post-gesture.swift');
  assert.equal(commit(dir).status, 0);
  spawnSync('git', ['commit', '-qm', 'x', '--no-verify'], { cwd: dir, env: env({ GIT_AUTHOR_NAME: 't', GIT_AUTHOR_EMAIL: 't@t', GIT_COMMITTER_NAME: 't', GIT_COMMITTER_EMAIL: 't@t' }) });
  const old = repo(MIKE);
  stage(old, 'app/Sources/Old.swift');
  spawnSync('git', ['commit', '-qm', 'x', '--no-verify'], { cwd: old, env: env({ GIT_AUTHOR_NAME: 't', GIT_AUTHOR_EMAIL: 't@t', GIT_COMMITTER_NAME: 't', GIT_COMMITTER_EMAIL: 't@t' }) });
  spawnSync('git', ['rm', '-q', 'app/Sources/Old.swift'], { cwd: old });
  assert.equal(commit(old).status, 0, 'removing Swift code is fine');
});

test('Mike asking for a Swift change lets it through, by file or variable', () => {
  const dir = repo(MIKE);
  stage(dir, 'app/Sources/Threadbear/New.swift');
  assert.equal(commit(dir, { THREADBEAR_SWIFT_OK: '1' }).status, 0);
  fs.writeFileSync(path.join(dir, '.git/threadbear-swift-ok'), '');
  assert.equal(commit(dir).status, 0);
  assert.equal(edit(dir, 'app/Sources/Threadbear/New.swift').status, 0);
});

test("Mike's copy refuses an agent's edit to the Swift app before it's written", () => {
  const dir = repo(MIKE);
  const r = edit(dir, 'app/Sources/Threadbear/ThreadView.swift');
  assert.equal(r.status, 2);
  assert.match(r.stderr, /desktop\//);
  assert.equal(edit(dir, 'desktop/gpui/src/thread_view.rs').status, 0);
  assert.equal(edit(dir, 'app/Tools/mock-runner/server.ts').status, 0);
});

test("someone else's copy says nothing until the GPUI app is released to them", () => {
  const dir = repo(FORK);
  stage(dir, 'app/Sources/Threadbear/New.swift');
  const c = commit(dir);
  assert.equal(c.status, 0);
  assert.equal(c.stderr, '');
  const e = edit(dir, 'app/Sources/Threadbear/New.swift');
  assert.equal(e.status, 0);
  assert.equal(e.stdout, '');
});

test("once released, someone else's copy warns but doesn't refuse", () => {
  const dir = repo(FORK, true);
  stage(dir, 'app/Sources/Threadbear/New.swift');
  const c = commit(dir);
  assert.equal(c.status, 0);
  assert.match(c.stderr, /being retired in favour of the GPUI app/);
  const e = edit(dir, 'app/Sources/Threadbear/New.swift');
  assert.equal(e.status, 0);
  const out = JSON.parse(e.stdout);
  assert.equal(out.hookSpecificOutput.hookEventName, 'PreToolUse');
  assert.match(out.hookSpecificOutput.additionalContext, /desktop\//);
  assert.equal(out.hookSpecificOutput.permissionDecision, undefined, 'a warning never approves the edit');
});

test('finishing a merge brings in commits already made, so it passes', () => {
  const dir = repo(MIKE);
  stage(dir, 'app/Sources/Threadbear/New.swift');
  fs.writeFileSync(path.join(dir, '.git/MERGE_HEAD'), '');
  // MERGE_HEAD has to name a commit for rev-parse --verify.
  spawnSync('git', ['commit', '-qm', 'x', '--no-verify', '--allow-empty'], { cwd: dir, env: env({ GIT_AUTHOR_NAME: 't', GIT_AUTHOR_EMAIL: 't@t', GIT_COMMITTER_NAME: 't', GIT_COMMITTER_EMAIL: 't@t' }) });
  const head = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: dir, encoding: 'utf8' }).stdout.trim();
  fs.writeFileSync(path.join(dir, '.git/MERGE_HEAD'), head + '\n');
  stage(dir, 'app/Sources/Threadbear/Other.swift');
  assert.equal(commit(dir).status, 0);
});
