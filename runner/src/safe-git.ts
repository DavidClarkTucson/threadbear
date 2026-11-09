import { execFileSync } from 'node:child_process';

// git in a folder Threadbear hasn't trusted, or whose .git it didn't make.
// A folder from a zip, a tarball or a shared drive can carry its own
// .git/config, and `core.fsmonitor` there names a program git starts on
// anything that reads the working tree: `git ls-files`, `git status`,
// `git worktree add`. Settings given with -c beat the repo's own, so these
// turn it off (and the untracked cache, which feeds from it) whatever the
// folder says. Every git the runner runs in a thread's folder goes through
// these (docs/protocol.md, "Trusting a folder's patches").
export const UNTRUSTED_GIT_CONFIG: readonly string[] = ['-c', 'core.fsmonitor=false', '-c', 'core.untrackedCache=false'];

// The arguments for `git -C <dir> ...args` with those settings first.
export function untrustedGitArgs(dir: string, args: readonly string[]): string[] {
  return [...UNTRUSTED_GIT_CONFIG, '-C', dir, ...args];
}

// The runner's environment without any GIT_ variable: one set when the
// runner was started from inside a git hook (or by hand) could point git
// at another repo, or add config (GIT_CONFIG_COUNT, GIT_CONFIG_PARAMETERS)
// that comes after the -c settings above.
export function untrustedGitEnv(base: NodeJS.ProcessEnv = process.env): NodeJS.ProcessEnv {
  const env = { ...base };
  for (const k of Object.keys(env)) if (k.startsWith('GIT_')) delete env[k];
  return env;
}

// A git listing in `dir`, read whole: what claude-trust and tracked-packages
// use to see what a folder holds before it's trusted. Throws as
// execFileSync does when git fails.
export function untrustedGitSync(dir: string, args: readonly string[]): string {
  return execFileSync('git', untrustedGitArgs(dir, args), {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'ignore'],
    timeout: 10_000,
    maxBuffer: 64 * 1024 * 1024,
    env: untrustedGitEnv(),
  });
}
