// The sidebar's sections, from the runner's thread summaries. The rules are
// the website's (web/src/sections.ts), shared rather than copied, so the
// Mac, the phone, the website and this app agree on what's in the inbox.
import type { ThreadSummary } from '../../runner/src/protocol.ts';
import { build, hibernating, isSnoozed, type ThreadRow } from '../../web/src/sections.ts';

export type Row = ThreadRow & { summary: ThreadSummary };

export function toRow(t: ThreadSummary): Row {
  const working = t.background?.some((b) => b.kind === 'agent' || b.kind === 'other') ? 'background' : undefined;
  return {
    summary: t,
    threadId: t.id,
    title: t.title,
    preview: t.preview,
    cwd: t.cwd,
    status: t.status,
    needsYou: t.needsYou,
    unread: t.lastSeq > t.lastSeenSeq,
    lastActivityMs: Date.parse(t.lastActivityAt),
    starredAt: t.starredAt,
    archivedAt: t.archivedAt,
    snoozedUntil: t.snoozedUntil,
    snoozedUntilRestart: t.snoozedUntilRestart,
    wakeNote: t.wakeWhen?.topic,
    // The phone's and website's rows leave needs out while sub-agents run
    // (docs/protocol.md, "Needs you"), and the shared rules expect that.
    needs: working ? undefined : t.needs,
    pendingReviews: t.pendingReviews,
    waitingForRoom: t.waitingForRoom,
    working,
    asks: t.asks,
    projects: t.projects?.map((p) => ({ id: p.id, title: p.title })),
    share: t.share,
  };
}

export type Section = { key: string; title: string; rows: Row[] };

export function sections(threads: Iterable<ThreadSummary>, now = Date.now()): Section[] {
  const rows = [...threads].map(toRow);
  const built = build(rows, now, new Map(), 'needsYou');
  const snoozed = rows.filter((r) => !r.archivedAt && !r.starredAt && isSnoozed(r, now));
  const out: Section[] = [
    { key: 'starred', title: 'Starred', rows: built.starred },
    { key: 'inbox', title: 'Inbox', rows: built.inbox },
    { key: 'running', title: 'Running', rows: built.running },
    { key: 'hibernating', title: 'Hibernating', rows: hibernating(built.scheduled, snoozed, new Map()) },
  ];
  return out.filter((s) => s.rows.length > 0 || s.key === 'inbox');
}

// Whether the row's dot says it wants Mike: stuck on him, or new to him.
export function wantsYou(r: Row): boolean {
  return r.status === 'waiting' || r.needsYou || (r.needs === 'you' && r.status !== 'running');
}
