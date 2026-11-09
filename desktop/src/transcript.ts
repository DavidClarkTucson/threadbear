// A thread's events folded into what the transcript draws. No React here,
// so the tests run it bare. The rules are docs/protocol.md's "Events" and
// "Transient events": a later assistant.text with the same itemId replaces
// the earlier one where it stands, the persisted event replaces whatever
// its deltas built up, and a thread.rewound hides the events it names.
import type { Ask, PersistedEvent, ToolKind, TurnOutcome, NeedsWho, AskAnswer, ApprovalOption, ApprovalDecision, Question } from '../../runner/src/protocol.ts';

export type Live = { type: 'assistant.text.delta' | 'reasoning.delta' | 'tool.output.delta'; text: string };

export type Step = { itemId: string; title: string; kind: ToolKind; ok?: boolean; output?: string };

export type Item =
  | { kind: 'user'; key: string; seq: number; text: string; answers?: AskAnswer[]; origin?: string }
  | { kind: 'text'; key: string; itemId: string; text: string; streaming?: boolean; summary?: boolean }
  | { kind: 'reasoning'; key: string; itemId: string; text: string; streaming?: boolean }
  | { kind: 'steps'; key: string; steps: Step[] }
  | { kind: 'approval'; key: string; approvalId: string; title: string; options: ApprovalOption[]; decision?: ApprovalDecision }
  | { kind: 'question'; key: string; approvalId: string; questions: Question[]; decision?: ApprovalDecision; message?: string }
  | { kind: 'note'; key: string; text: string; tone: 'info' | 'error' }
  | {
      kind: 'turnEnd';
      key: string;
      seq: number;
      outcome: TurnOutcome;
      durationMs: number;
      needs?: NeedsWho;
      asks?: Ask[];
      next?: string[];
      error?: string;
    };

export type ThreadEvents = { events: Map<number, PersistedEvent>; live: Map<string, Live> };

export const emptyEvents = (): ThreadEvents => ({ events: new Map(), live: new Map() });

const isDelta = (type: string): type is Live['type'] => type.endsWith('.delta');

// Adds what threads.events, threads.subscribe or a thread.event brought.
// Returns a new value, for React.
export function addEvents(state: ThreadEvents, incoming: any[], inProgress: { itemId: string; type: string; text: string }[] = []): ThreadEvents {
  const events = new Map(state.events);
  const live = new Map(state.live);
  for (const p of inProgress) live.set(p.itemId, { type: p.type === 'reasoning' ? 'reasoning.delta' : 'assistant.text.delta', text: p.text });
  for (const e of incoming) {
    if (isDelta(e.type)) {
      const had = live.get(e.itemId);
      live.set(e.itemId, { type: e.type, text: (had?.text ?? '') + e.text });
    } else {
      events.set(e.seq, e as PersistedEvent);
      if ('itemId' in e && (e.type === 'assistant.text' || e.type === 'reasoning')) live.delete(e.itemId);
    }
  }
  return { events, live };
}

export function fold({ events, live }: ThreadEvents): Item[] {
  const ordered = [...events.values()].sort((a, b) => a.seq - b.seq);
  const hidden: [number, number][] = ordered.flatMap((e) => (e.type === 'thread.rewound' ? [[e.fromSeq, e.toSeq] as [number, number]] : []));
  const isHidden = (seq: number) => hidden.some(([from, to]) => seq >= from && seq <= to);

  const items: Item[] = [];
  const byItem = new Map<string, number>(); // itemId → index of its text or reasoning row
  const stepOf = new Map<string, Step>();
  const resolved = new Map<string, { decision: ApprovalDecision; message?: string }>();
  for (const e of ordered) if (e.type === 'approval.resolved') resolved.set(e.approvalId, { decision: e.decision, message: e.message });

  const steps = (): Step[] => {
    const last = items[items.length - 1];
    if (last?.kind === 'steps') return last.steps;
    const fresh: Item = { kind: 'steps', key: `steps-${items.length}`, steps: [] };
    items.push(fresh);
    return fresh.steps;
  };

  for (const e of ordered) {
    if (isHidden(e.seq)) continue;
    switch (e.type) {
      case 'user.message':
        items.push({ kind: 'user', key: `u${e.seq}`, seq: e.seq, text: e.text, answers: e.answers, origin: e.origin });
        break;
      case 'assistant.text':
      case 'reasoning': {
        const row: Item =
          e.type === 'assistant.text'
            ? { kind: 'text', key: `t${e.itemId}`, itemId: e.itemId, text: e.text, summary: e.summary }
            : { kind: 'reasoning', key: `r${e.itemId}`, itemId: e.itemId, text: e.text };
        const at = byItem.get(e.itemId);
        if (at !== undefined) items[at] = row;
        else {
          byItem.set(e.itemId, items.length);
          items.push(row);
        }
        break;
      }
      case 'tool.call': {
        const step: Step = { itemId: e.itemId, title: e.title, kind: e.kind };
        stepOf.set(e.itemId, step);
        steps().push(step);
        break;
      }
      case 'tool.result': {
        const step = stepOf.get(e.itemId);
        if (step) {
          step.ok = e.ok;
          step.output = e.text;
        }
        break;
      }
      case 'approval.requested':
        items.push({ kind: 'approval', key: `a${e.approvalId}`, approvalId: e.approvalId, title: e.title, options: e.options, decision: resolved.get(e.approvalId)?.decision });
        break;
      case 'question.requested': {
        const r = resolved.get(e.approvalId);
        items.push({ kind: 'question', key: `q${e.approvalId}`, approvalId: e.approvalId, questions: e.questions, decision: r?.decision, message: r?.message });
        break;
      }
      case 'secret.requested':
        items.push({ kind: 'note', key: `s${e.seq}`, text: `Asked for ${e.label}`, tone: 'info' });
        break;
      case 'system.note':
        items.push({ kind: 'note', key: `n${e.seq}`, text: e.text, tone: 'info' });
        break;
      case 'error':
        items.push({ kind: 'note', key: `e${e.seq}`, text: e.message, tone: 'error' });
        break;
      case 'turn.completed':
        items.push({
          kind: 'turnEnd',
          key: `c${e.seq}`,
          seq: e.seq,
          outcome: e.outcome,
          durationMs: e.durationMs,
          needs: e.needs,
          asks: e.asks,
          next: e.next,
          error: e.error,
        });
        break;
      default:
        break;
    }
  }

  // Text still streaming: replaces a persisted row of the same item (a
  // narration block landing as text), else goes at the end, as it's the
  // newest thing in the thread.
  for (const [itemId, l] of live) {
    if (l.type === 'tool.output.delta') {
      const step = stepOf.get(itemId);
      if (step && step.ok === undefined) step.output = l.text;
      continue;
    }
    const row: Item =
      l.type === 'assistant.text.delta'
        ? { kind: 'text', key: `t${itemId}`, itemId, text: l.text, streaming: true }
        : { kind: 'reasoning', key: `r${itemId}`, itemId, text: l.text, streaming: true };
    const at = byItem.get(itemId);
    if (at !== undefined) items[at] = row;
    else items.push(row);
  }
  return items;
}

// What a click on one of a card's answers sends: the ask and the answer,
// as the Mac app writes them, and which ask it answers.
export function answerMessage(ask: Ask, answer: string): { text: string; answers: AskAnswer[] } {
  return { text: `${ask.action}\n${answer}`, answers: [{ action: ask.action, answer }] };
}

// What approvals.respond takes for a question card, as the Mac app makes
// it (Cells.swift, ApprovalText.answerSummary): each question's picks joined
// with ", ", keyed by the question, and the whole lot in one line for the
// transcript, in the order they were asked.
export function questionAnswers(questions: Question[], chosen: Record<number, string[]>): { answers: Record<string, string>; message: string } {
  const answers: Record<string, string> = {};
  questions.forEach((q, i) => {
    const picks = q.options.map((o) => o.label).filter((l) => chosen[i]?.includes(l));
    if (picks.length) answers[q.question] = picks.join(', ');
  });
  const message = questions.flatMap((q) => (answers[q.question] ? [answers[q.question]] : [])).join('; ');
  return { answers, message };
}

export function duration(ms: number): string {
  const s = Math.round(ms / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ${s % 60}s`;
  return `${Math.floor(m / 60)}h ${m % 60}m`;
}
