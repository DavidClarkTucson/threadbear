// Loose ends (experiment looseEnds; docs/protocol.md, "Loose ends"): what's
// still open on a thread whose last turn said it's done, checked by the
// runner so the done card can say whether Archive is safe without Mike
// asking the agent. The nightly retro counted 27 typed "anything else, or
// archive?" asks on 6 Nov 2026: Archive puts the thread away straight off,
// and he wanted someone's word that nothing was left first. The one real
// loose end in the threads it looked at was a branch not on main
// (938c1cd1), so these are the things a runner can see for itself.
import { execFile } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import type { LooseEnd, Schedule } from './protocol.ts';
import { git, MAIN, type WorktreeRecord } from './worktrees.ts';

// The thread's own worktree: commits on its branch that main doesn't have,
// and files in it that aren't committed. Untracked ones count, as a new
// file the agent wrote and never committed is lost when the worktree goes;
// ignored ones (build output, the linked node_modules) don't show at all.
export async function worktreeLooseEnds(w: WorktreeRecord): Promise<LooseEnd[]> {
  const out: LooseEnd[] = [];
  const ahead = await git(w.mainCheckout, ['rev-list', '--count', `${MAIN}..refs/heads/${w.branch}`]);
  const commits = ahead.ok ? Number.parseInt(ahead.stdout.trim(), 10) : 0;
  if (commits > 0) out.push({ kind: 'unlanded', text: `${commits === 1 ? '1 commit' : `${commits} commits`} on ${w.branch} ${commits === 1 ? "isn't" : "aren't"} on ${MAIN}` });
  const status = await git(w.path, ['status', '--porcelain', '--untracked-files=all']);
  const files = status.ok ? status.stdout.split('\n').filter((l) => l.trim()).length : 0;
  if (files > 0) out.push({ kind: 'uncommitted', text: `${files === 1 ? '1 file' : `${files} files`} in its worktree ${files === 1 ? "isn't" : "aren't"} committed` });
  return out;
}

// Files the thread changed (its Highlights' edited list) outside a
// worktree of its own: a thread in ~/dev/me/misc or a repo without
// worktrees shares its checkout, so only its own files count, not the
// whole folder's status, which holds other threads' work too. And the
// commits it made there that no remote has, in a repo that has one: his
// own repos push straight to main. The check-in of 11 Nov found most of
// his typed "anything else?" asks came from threads like these, where
// the card could only see threads and check-ins. `skip` is the thread's
// own worktree, which worktreeLooseEnds covers.
// `pushed` are the commits its Highlights saw it push: the ones that can
// be pushed and still not finished (unmergedLooseEnds).
export async function editedLooseEnds(edited: string[], commits: string[], skip?: string, pushed: string[] = [], prs: PrLookup = githubPr): Promise<LooseEnd[]> {
  const repos = new Map<string, string[]>();
  const tops = new Map<string, string | undefined>();
  const topOf = async (file: string): Promise<string | undefined> => {
    let dir = path.dirname(file);
    // A file since deleted, in a folder since deleted: its nearest folder.
    while (!fs.existsSync(dir) && path.dirname(dir) !== dir) dir = path.dirname(dir);
    if (!tops.has(dir)) {
      const r = await git(dir, ['rev-parse', '--show-toplevel']);
      tops.set(dir, r.ok ? r.stdout.trim() : undefined);
    }
    return tops.get(dir);
  };
  const inside = (file: string, dir: string) => file === dir || file.startsWith(dir + path.sep);
  for (const file of edited) {
    if (!path.isAbsolute(file) || (skip && inside(file, skip))) continue;
    const top = await topOf(file);
    if (!top || (skip && inside(top, skip))) continue;
    repos.set(top, [...(repos.get(top) ?? []), path.relative(top, file)]);
  }
  const out: LooseEnd[] = [];
  for (const [top, files] of repos) {
    const name = path.basename(top);
    const status = await git(top, ['status', '--porcelain', '--untracked-files=all', '--', ...files]);
    const n = status.ok ? status.stdout.split('\n').filter((l) => l.trim()).length : 0;
    if (n > 0) out.push({ kind: 'uncommitted', text: `${n === 1 ? '1 file' : `${n} files`} it changed in ${name} ${n === 1 ? "isn't" : "aren't"} committed` });
    // A commit the thread made is in this repo if git knows it here.
    const remotes = await git(top, ['remote']);
    if (!remotes.ok || !remotes.stdout.trim()) continue;
    let unpushed = 0;
    for (const sha of commits) {
      if (!(await git(top, ['cat-file', '-e', `${sha}^{commit}`])).ok) continue;
      const on = await git(top, ['branch', '-r', '--contains', sha]);
      if (on.ok && !on.stdout.trim()) unpushed++;
    }
    if (unpushed > 0) out.push({ kind: 'unpushed', text: `${unpushed === 1 ? '1 commit' : `${unpushed} commits`} it made in ${name} ${unpushed === 1 ? "isn't" : "aren't"} pushed` });
    out.push(...(await unmergedLooseEnds(top, pushed, prs)));
  }
  return out;
}

// Pushed isn't finished. On 11 Nov 2026 the examples thread (0222c475) said
// Done with its work pushed to three stacked draft PRs in convex/website,
// none merged, and the card showed a plain Done: Mike nearly archived it.
// So a commit it pushed that's on a remote branch but not the remote's
// default one is open, one line per branch, naming the PR when GitHub has
// one. A merged or closed PR isn't (a squash merge leaves the commits off
// main for good), and a commit no remote branch has any more (rebased and
// force-pushed) can't be told, so it's left out.
const MAX_UNMERGED = 3;
async function unmergedLooseEnds(top: string, pushed: string[], prs: PrLookup): Promise<LooseEnd[]> {
  if (!pushed.length) return [];
  const head = await git(top, ['symbolic-ref', '--short', 'refs/remotes/origin/HEAD']);
  let main = head.ok ? head.stdout.trim() : '';
  for (const guess of ['origin/main', 'origin/master']) {
    if (main) break;
    if ((await git(top, ['rev-parse', '--verify', '--quiet', `refs/remotes/${guess}`])).ok) main = guess;
  }
  if (!main) return [];
  const branches = new Set<string>();
  for (const sha of pushed) {
    if (!(await git(top, ['cat-file', '-e', `${sha}^{commit}`])).ok) continue;
    if ((await git(top, ['merge-base', '--is-ancestor', sha, main])).ok) continue;
    const on = await git(top, ['branch', '-r', '--format=%(refname:short)', '--contains', sha]);
    if (!on.ok) continue;
    for (const b of on.stdout.split('\n').map((l) => l.trim())) {
      // origin/<branch> only: other remotes are someone else's forks.
      if (b.startsWith('origin/') && b !== main && b !== 'origin/HEAD' && b !== 'origin') branches.add(b.slice('origin/'.length));
    }
  }
  if (!branches.size) return [];
  const name = path.basename(top);
  const into = main.slice('origin/'.length);
  const url = await git(top, ['remote', 'get-url', 'origin']);
  const repo = url.ok ? githubRepo(url.stdout.trim()) : undefined;
  const out: LooseEnd[] = [];
  for (const branch of [...branches].sort()) {
    const pr = repo ? await prs(repo, branch) : undefined;
    if (pr && pr.state !== 'OPEN') continue;
    out.push({
      kind: 'unmerged',
      text: pr ? `PR #${pr.number} in ${name} ${pr.isDraft ? 'is still a draft' : "isn't merged"}` : `Work it pushed to ${branch} in ${name} isn't on ${into}`,
    });
  }
  return out.slice(0, MAX_UNMERGED);
}

// "owner/name" for a github.com remote, else undefined.
export function githubRepo(url: string): string | undefined {
  const m = url.match(/^(?:https:\/\/(?:[^@/]+@)?github\.com\/|git@github\.com:|ssh:\/\/git@github\.com\/)([\w.-]+)\/([\w.-]+?)(?:\.git)?\/?$/);
  return m ? `${m[1]}/${m[2]}` : undefined;
}

export type Pr = { number: number; state: 'OPEN' | 'MERGED' | 'CLOSED'; isDraft: boolean };
export type PrLookup = (repo: string, branch: string) => Promise<Pr | undefined>;

// The newest PR from that branch, through the gh CLI, or undefined (no gh,
// not signed in, offline, no PR). Run outside the repo with -R, so the
// repo's own git config (an fsmonitor, a hook) never runs under gh: the
// runner's git goes through safe-git for the same reason. Kept a minute,
// as opening a thread checks again every few seconds.
const PR_TTL_MS = 60_000;
const prCache = new Map<string, { at: number; pr: Pr | undefined }>();
export const githubPr: PrLookup = async (repo, branch) => {
  const key = `${repo}#${branch}`;
  const hit = prCache.get(key);
  if (hit && Date.now() - hit.at < PR_TTL_MS) return hit.pr;
  const pr = await new Promise<Pr | undefined>((resolve) => {
    execFile(
      'gh',
      ['pr', 'list', '-R', repo, '--head', branch, '--state', 'all', '--limit', '1', '--json', 'number,state,isDraft'],
      { cwd: os.tmpdir(), timeout: 15_000, killSignal: 'SIGKILL', env: { ...process.env, GH_PROMPT_DISABLED: '1', GIT_TERMINAL_PROMPT: '0' } },
      (err, stdout) => {
        if (err) return resolve(undefined);
        try {
          const [p] = JSON.parse(String(stdout)) as Pr[];
          resolve(p && typeof p.number === 'number' ? p : undefined);
        } catch {
          resolve(undefined);
        }
      },
    );
  });
  prCache.set(key, { at: Date.now(), pr });
  return pr;
};

// What the agent said is still open (end_turn's open): things only it
// knows, like a reply worth sending or a decision waiting elsewhere.
export function agentLooseEnds(open: string[] | undefined): LooseEnd[] {
  return (open ?? []).map((text) => ({ kind: 'open' as const, text }));
}

// A thread this one started that hasn't finished: still working, or
// waiting on Mike. Hibernating or done ones aren't open business here.
export type ChildThread = { id: string; title: string; working: boolean; needsYou: boolean };

export function childLooseEnds(children: ChildThread[]): LooseEnd[] {
  const out: LooseEnd[] = [];
  for (const c of children) {
    if (c.working) out.push({ kind: 'thread', threadId: c.id, text: `"${c.title}", a thread it started, is still working` });
    else if (c.needsYou) out.push({ kind: 'thread', threadId: c.id, text: `"${c.title}", a thread it started, is waiting for you` });
  }
  return out;
}

// A schedule that will message this thread again (a check-in it booked).
// Not a reason to keep it: an archived thread comes back when one runs.
// It's on the card so Mike knows archiving it doesn't lose the check-in.
export function scheduleLooseEnds(threadId: string, schedules: Schedule[]): LooseEnd[] {
  return schedules
    .filter((s) => s.threadId === threadId && !s.paused && s.nextRunAt)
    .sort((a, b) => (a.nextRunAt! < b.nextRunAt! ? -1 : 1))
    .slice(0, 3)
    .map((s) => ({ kind: 'scheduled' as const, text: s.title, at: s.nextRunAt }));
}

// Whether any of them should stop a plain Archive: a booked check-in doesn't.
export function anyOpen(ends: LooseEnd[]): boolean {
  return ends.some((e) => e.kind !== 'scheduled');
}
