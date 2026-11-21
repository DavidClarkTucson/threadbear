// Ask me later (experiment `asksLater`, David's fork, 21 Nov 2026): an Auto
// thread never waits on a card. Where the card reviewer (card-review.ts)
// would hand a card to the user, the action is refused at once and the
// agent carries on without it. The refused action goes to one Permission
// requests thread as a question card, which the user answers whenever
// they like: Always allow (a saved rule for that folder), Allow once (the
// thread is told to try again, and that one call then goes through) or
// Keep blocking. Saved rules are checked before the reviewer runs, so an
// unattended thread slowly stops being blocked on what the user wants.
//
// Why: David, 21 Nov 2026 (thread 80345583): "tasks can run unattended but
// slowly build out rules to allow things the user might want".
//
// Rules are exact, Claude Code style (`Bash(npm test)`, `Bash(npm test:*)`,
// `Edit(/repo/src/**)`, `WebFetch(domain:x.com)`), never words for the
// reviewer, which a page the agent read could argue around. Cards that
// hand out lasting access (NEVER_REVIEWED) or edit a file Threadbear runs
// as code (no Always allow on their card) can't become rules: they're
// offered Allow once and Keep blocking only, and only on a Mac.
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import type { Runner, ThreadState } from '../runner.ts';
import type { RunnerPart } from './part.ts';
import type { EventBody, Question } from '../protocol.ts';
import type { QuestionAnswer, ToolApprovalAnswer } from '../adapters/types.ts';
import { experimentOn } from '../experiments.ts';
import { remoteCardShowsAll } from '../tools.ts';
import type { CardForReview } from './card-review.ts';

export const ASKS_LATER = 'asksLater';
export const INBOX_TITLE = 'Permission requests';
const FILE = 'asks-later.json';
// An Allow once that isn't used within a day is dropped.
const ONCE_MS = 24 * 60 * 60_000;
// How much of a command or input the question shows.
const SHOW_CHARS = 600;

export const ALLOW_ONCE = 'Allow once';
export const KEEP_BLOCKING = 'Keep blocking';
const ALWAYS = 'Always allow ';

// What the agent is told in place of the card.
export const BLOCKED_FOR_LATER =
  "Threadbear blocked this: its safety check wouldn't allow it, and in this thread the user isn't asked straight away. It's been left in their Permission requests to answer later. Don't retry it or reword it to get past the check. Carry on with what you can do without it, and in your final reply say what you couldn't do and why.";

export type SavedRule = { folder: string; rule: string; addedAt: number };
export type OnceAllow = { threadId: string; key: string; until: number };
export type LaterAsk = {
  id: string;
  threadId: string;
  threadTitle: string;
  folder: string;
  name: string;
  title: string;
  shown: string;
  why?: string;
  rules: string[];
  key: string;
  macOnly: boolean;
  at: number;
  status: 'open' | 'allowed' | 'once' | 'blocked';
};
export type AsksLaterFile = { inboxId?: string; rules: SavedRule[]; once: OnceAllow[]; asks: LaterAsk[] };

// ---- Rules: pure, so they're tested on their own -------------------------

const FILE_TOOLS = new Set(['Edit', 'Write', 'MultiEdit', 'NotebookEdit']);
// Anything that can chain, redirect or substitute: a prefix rule never
// covers a command with one, as `npm test && curl ...` starts with `npm test`.
const SHELL_SPECIAL = /[;&|<>`$\n\r\\(){}]/;

function filePath(input: Record<string, unknown>): string | undefined {
  const p = input.file_path ?? input.notebook_path;
  return typeof p === 'string' && path.isAbsolute(p) ? path.resolve(p) : undefined;
}

function command(input: Record<string, unknown>): string | undefined {
  return typeof input.command === 'string' ? input.command.trim() : undefined;
}

function host(input: Record<string, unknown>): string | undefined {
  try {
    return typeof input.url === 'string' ? new URL(input.url).hostname.toLowerCase() : undefined;
  } catch {
    return undefined;
  }
}

// The rules the user can pick from for this call, narrowest first.
export function candidateRules(name: string, input: Record<string, unknown>): string[] {
  if (name === 'Bash') {
    const cmd = command(input);
    if (!cmd || cmd.includes(')')) return [];
    const out = [`Bash(${cmd})`];
    if (SHELL_SPECIAL.test(cmd)) return out;
    const words = cmd.split(/\s+/);
    const prefix = words.slice(0, words.length > 2 ? 2 : 1).join(' ');
    if (words.length > 1) out.push(`Bash(${prefix}:*)`);
    return out;
  }
  if (FILE_TOOLS.has(name)) {
    const file = filePath(input);
    if (!file || file.includes(')')) return [];
    return [`Edit(${file})`, `Edit(${path.dirname(file)}/**)`];
  }
  if (name === 'WebFetch') {
    const h = host(input);
    return h ? [`WebFetch(domain:${h})`] : [];
  }
  return /^[\w-]+$/.test(name) ? [name] : [];
}

// Whether a saved rule covers this call.
export function ruleMatches(rule: string, name: string, input: Record<string, unknown>): boolean {
  const m = /^([\w-]+)(?:\((.*)\))?$/s.exec(rule);
  if (!m) return false;
  const [, tool, arg] = m;
  if (arg === undefined) return tool === name && name !== 'Bash' && !FILE_TOOLS.has(name) && name !== 'WebFetch';
  if (tool === 'Bash') {
    if (name !== 'Bash') return false;
    const cmd = command(input);
    if (!cmd) return false;
    if (!arg.endsWith(':*')) return cmd === arg;
    const prefix = arg.slice(0, -2);
    return !SHELL_SPECIAL.test(cmd) && (cmd === prefix || cmd.startsWith(`${prefix} `));
  }
  if (tool === 'Edit') {
    if (!FILE_TOOLS.has(name)) return false;
    const file = filePath(input);
    if (!file) return false;
    if (!arg.endsWith('/**')) return file === arg;
    const dir = arg.slice(0, -3);
    return file.startsWith(`${dir}${path.sep}`);
  }
  if (tool === 'WebFetch') return name === 'WebFetch' && arg.startsWith('domain:') && host(input) === arg.slice(7);
  return false;
}

// What makes two calls "the same" for Allow once: a Bash call's command
// (its description and timeout vary when the agent tries again), a file
// edit's path and change, anything else its whole input.
export function onceKey(name: string, input: Record<string, unknown>): string {
  let body: unknown = input;
  if (name === 'Bash') body = command(input);
  else if (FILE_TOOLS.has(name)) body = { ...input, file_path: filePath(input) };
  return `${name}\n${JSON.stringify(body)}`;
}

// Where a thread's saved rules belong: a worktree thread's repo, not its slot.
export function ruleFolder(st: ThreadState): string {
  return st.rec.worktree?.mainCheckout ?? st.rec.cwd;
}

function shownInput(name: string, input: Record<string, unknown>): string {
  const cmd = name === 'Bash' ? command(input) : undefined;
  let text: string;
  if (cmd) text = cmd;
  else if (FILE_TOOLS.has(name)) text = filePath(input) ?? '';
  else {
    try {
      text = JSON.stringify(input);
    } catch {
      text = '';
    }
  }
  return text.length > SHOW_CHARS ? `${text.slice(0, SHOW_CHARS)}…` : text;
}

// The card the user answers in Permission requests.
export function laterQuestion(ask: LaterAsk): Question {
  const why = ask.why ? ` ${ask.why}` : '';
  const can = ask.rules.length ? '' : ' This kind of action can only ever be allowed once.';
  return {
    header: 'Blocked',
    question: `"${ask.threadTitle}" wanted to: ${ask.title}\n${ask.shown}\n\nThe safety check stopped it.${why}${can} In ${ask.folder}.`,
    multiSelect: false,
    options: [
      ...ask.rules.map((r) => ({ label: `${ALWAYS}${r}`, description: `From now on, in ${ask.folder}` })),
      { label: ALLOW_ONCE, description: 'The thread is told to try it again, and that one call goes through' },
      { label: KEEP_BLOCKING, description: 'Nothing changes; the thread carries on without it' },
    ],
  };
}

// ---- The store ------------------------------------------------------------

export function readAsksLater(home: string): AsksLaterFile {
  try {
    const raw = JSON.parse(fs.readFileSync(path.join(home, FILE), 'utf8')) as Partial<AsksLaterFile>;
    return { ...(raw.inboxId ? { inboxId: raw.inboxId } : {}), rules: raw.rules ?? [], once: raw.once ?? [], asks: raw.asks ?? [] };
  } catch {
    return { rules: [], once: [], asks: [] };
  }
}

function writeAsksLater(home: string, f: AsksLaterFile): void {
  const file = path.join(home, FILE);
  const tmp = `${file}.${process.pid}.tmp`;
  fs.writeFileSync(tmp, `${JSON.stringify(f, null, 1)}\n`, { mode: 0o600 });
  fs.renameSync(tmp, file);
}

// ---- The runner side ------------------------------------------------------

export const asksLaterPart = {
  asksLaterOn(this: Runner, st: ThreadState): boolean {
    return st.rec.permissionMode === 'auto' && st.rec.provider === 'claude' && experimentOn(this.cfg.home, ASKS_LATER);
  },

  // Before the reviewer: a call the user already allowed, once or always.
  // Returns the note for the transcript, or nothing.
  allowedEarlier(this: Runner, st: ThreadState, card: CardForReview & { savable?: boolean }): string | undefined {
    const home = this.cfg.home;
    const f = readAsksLater(home);
    const now = this.clock.now();
    const key = onceKey(card.name, card.input);
    const once = f.once.findIndex((o) => o.threadId === st.rec.id && o.key === key && o.until > now);
    if (once >= 0 || f.once.some((o) => o.until <= now)) {
      const used = once >= 0 ? f.once[once] : undefined;
      f.once = f.once.filter((o) => o !== used && o.until > now);
      writeAsksLater(home, f);
      if (used) return `Allowed once, as you said in ${INBOX_TITLE}: ${card.title}`;
    }
    if (!card.savable) return undefined;
    const folder = ruleFolder(st);
    const rule = f.rules.find((r) => r.folder === folder && ruleMatches(r.rule, card.name, card.input));
    return rule ? `Allowed by your rule ${rule.rule}: ${card.title}` : undefined;
  },

  // In place of the card: refuse, and leave the question for later.
  blockForLater(this: Runner, st: ThreadState, card: CardForReview & { savable?: boolean }, why: string | undefined): ToolApprovalAnswer {
    const home = this.cfg.home;
    const f = readAsksLater(home);
    const key = onceKey(card.name, card.input);
    // The same call blocked again while its question is still open: one card is enough.
    const dup = f.asks.find((a) => a.status === 'open' && a.threadId === st.rec.id && a.key === key);
    const ask: LaterAsk = dup ?? {
      id: crypto.randomUUID(),
      threadId: st.rec.id,
      threadTitle: st.rec.title || 'A thread',
      folder: ruleFolder(st),
      name: card.name,
      title: card.title,
      shown: shownInput(card.name, card.input),
      ...(why ? { why } : {}),
      rules: card.savable ? candidateRules(card.name, card.input) : [],
      key,
      macOnly: !card.savable || !remoteCardShowsAll(card.name, card.input, st.rec.cwd, card.title),
      at: this.clock.now(),
      status: 'open',
    };
    if (!dup) {
      f.asks.push(ask);
      writeAsksLater(home, f);
      this.postLaterAsk(ask);
    }
    this.emit(st, { type: 'system.note', text: `Blocked, and left in ${INBOX_TITLE} for you: ${card.title}.${why ? ` ${why}` : ''}` }, st.rt.turn?.turnId);
    return { decision: 'deny', message: BLOCKED_FOR_LATER };
  },

  // The one Permission requests thread, made the first time it's needed.
  laterInbox(this: Runner): ThreadState {
    const home = this.cfg.home;
    const f = readAsksLater(home);
    const had = f.inboxId ? this.threads.get(f.inboxId) : undefined;
    if (had && !had.rt.deleted) return had;
    const { st } = this.makeThread({ text: INBOX_TITLE, title: INBOX_TITLE, permissionMode: 'ask', worktree: false, origin: 'runner' });
    this.emit(st, { type: 'system.note', text: `When an Auto thread's safety check stops something, the thread carries on without it and the question waits here. Always allow saves a rule for that folder, so it isn't stopped again; Allow once has the thread try that one call again.` });
    this.markDirty(st);
    writeAsksLater(home, { ...readAsksLater(home), inboxId: st.rec.id });
    return st;
  },

  postLaterAsk(this: Runner, ask: LaterAsk): void {
    const st = this.laterInbox();
    // Unarchived and woken, so the card is seen.
    if (st.rec.archivedAt) this.park(st, { kind: 'unarchive' });
    const approvalId = crypto.randomUUID();
    const questions = [laterQuestion(ask)];
    const body: EventBody = { type: 'question.requested', approvalId, questions };
    this.addApproval(st, { approvalId, kind: 'question', main: false, title: questions[0].question.split('\n')[0], questions, ...(ask.macOnly ? { macOnly: true } : {}), resolve: (a) => this.answerLaterAsk(ask.id, a) }, body, new AbortController().signal);
  },

  answerLaterAsk(this: Runner, id: string, a: QuestionAnswer): void {
    // Closed unanswered (a restart, Stop): it stays open and comes back.
    const label = a.answers ? Object.values(a.answers)[0] : undefined;
    if (!label) return;
    const home = this.cfg.home;
    const f = readAsksLater(home);
    const ask = f.asks.find((x) => x.id === id);
    if (!ask || ask.status !== 'open') return;
    const now = this.clock.now();
    let tell: string | undefined;
    if (label.startsWith(ALWAYS) && ask.rules.includes(label.slice(ALWAYS.length))) {
      const rule = label.slice(ALWAYS.length);
      if (!f.rules.some((r) => r.folder === ask.folder && r.rule === rule)) f.rules.push({ folder: ask.folder, rule, addedAt: now });
      ask.status = 'allowed';
      tell = `I've allowed ${rule} in ${ask.folder} from now on, so the call Threadbear blocked earlier (${ask.title}) will go through. Run it again if it's still needed, then carry on.`;
    } else if (label === ALLOW_ONCE) {
      f.once.push({ threadId: ask.threadId, key: ask.key, until: now + ONCE_MS });
      ask.status = 'once';
      tell = `I've allowed the call Threadbear blocked earlier, once: ${ask.title}. Run it again exactly as before if it's still needed, then carry on.`;
    } else {
      ask.status = 'blocked';
    }
    writeAsksLater(home, f);
    if (!tell || !this.threads.has(ask.threadId)) return;
    try {
      this.send(ask.threadId, tell);
    } catch (e) {
      this.log.warn('asks later: could not tell the thread', { threadId: ask.threadId, error: String(e) });
    }
  },

  // After a restart the cards are gone, but the questions aren't.
  repostLaterAsks(this: Runner): void {
    if (!experimentOn(this.cfg.home, ASKS_LATER)) return;
    for (const ask of readAsksLater(this.cfg.home).asks) if (ask.status === 'open') this.postLaterAsk(ask);
  },
};

export type AsksLaterPart = RunnerPart<typeof asksLaterPart>;
