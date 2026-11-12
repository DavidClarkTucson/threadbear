// Another runner's thread on this Mac (relay.ts) draws a Plan first plan as
// its plan card, another lab's review on its turn's end, and the done card's
// loose ends, as that runner's own Mac does. All three were missing in the
// nightly parity run's pictures (12 Nov): the plan folded into the turn's
// work, and the other two never left that runner's row.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { PLAN_GO_AHEAD, relayEvents, planOutcome, type Card, type RelayRow, type RelayThreadRow } from '../../src/relay.ts';
import fs from 'node:fs';
import type { LabReviewLedger } from '../../src/protocol.ts';

const T = 'thread-on-bruce';
const at = (s: number) => new Date(Date.parse('2026-11-12T10:00:00Z') + s * 1000).toISOString();
const PLAN: RelayRow = { seq: 4, role: 'assistant', text: '# Bots\n\n1. Add a bots table', plan: true, turn: 'aaaa1111', at: at(1) };
const start: RelayRow = { seq: 2, role: 'user', text: 'Plan the bots', turn: 'aaaa1111', at: at(0) };
const idle: RelayThreadRow = { runnerId: 'bruce', status: 'idle', needs: 'you' };

const outcome = (after: RelayRow[], row: RelayThreadRow | null = idle) => planOutcome([start, PLAN, ...after], 1, row);

// Read as text: loading runner/approvals.ts on its own trips over its
// import cycle with runner.ts.
test("the relay's Go ahead words are the runner's", () => {
  const src = fs.readFileSync(new URL('../../src/runner/approvals.ts', import.meta.url), 'utf8');
  assert.equal(src.match(/export const PLAN_GO_AHEAD = '([^']*)';/)?.[1], PLAN_GO_AHEAD);
});

test('a plan row is a plan card, not a reply folded into the work', () => {
  const ev = relayEvents(T, [start, PLAN], idle, [], false, 'bruce');
  const plan = ev.find((e) => e.type === 'plan.proposed');
  assert.ok(plan, 'a plan.proposed');
  assert.equal(plan.itemId, 'relay-4');
  assert.equal(plan.text, PLAN.text);
  assert.equal(ev.some((e) => e.type === 'assistant.text' && e.text === PLAN.text), false);
  assert.equal(ev.some((e) => e.type === 'plan.resolved'), false, 'nothing after it: still waiting');
});

test('a plan row from an older backend, with no flag, is a reply as before', () => {
  const { plan: _plan, ...asText } = PLAN;
  const ev = relayEvents(T, [start, asText], idle, [], false, 'bruce');
  assert.equal(ev.some((e) => e.type === 'plan.proposed'), false);
  assert.ok(ev.some((e) => e.type === 'assistant.text' && e.text === PLAN.text));
});

test("what became of a plan, from what came after it", () => {
  const turn = 'aaaa1111';
  // Codex ends its turn with a line after the plan, and end_turn is a tool call too.
  assert.equal(outcome([{ seq: 6, role: 'assistant', text: 'Ready for you.', turn, at: at(2) }, { seq: 7, role: 'tool', text: 'end_turn', toolKind: 'mcp', turn, at: at(3) }]), undefined);
  // Claude's card allowed: the work goes on in the same turn.
  assert.equal(outcome([{ seq: 6, role: 'tool', text: 'Read ~/x.ts', toolKind: 'read', turn, at: at(2) }]), 'go');
  // The runner's own Go ahead, sent as Mike.
  assert.equal(outcome([{ seq: 6, role: 'user', text: PLAN_GO_AHEAD, origin: 'runner', turn: 'bbbb2222', at: at(5) }]), 'go');
  // Anything else Mike writes sends it back.
  assert.equal(outcome([{ seq: 6, role: 'user', text: 'Use Postgres instead', turn: 'bbbb2222', at: at(5) }]), 'revise');
  // So does a newer plan.
  assert.equal(outcome([{ ...PLAN, seq: 8, text: '# Bots, take two' }]), 'revise');
  // Claude's card for it still open: waiting, whatever else the row has.
  const waiting: RelayThreadRow = { ...idle, status: 'waiting', pendingApproval: { approvalId: 'a1', kind: 'approval', name: 'ExitPlanMode', title: 'Finish the plan', options: ['deny'], planSeq: 4 } };
  assert.equal(outcome([{ seq: 6, role: 'tool', text: 'Read ~/x.ts', toolKind: 'read', turn, at: at(2) }], waiting), undefined);
});

test("Claude's plan card goes with the plan's card, only once the plan is here as one", () => {
  const pending = { approvalId: 'a1', kind: 'approval' as const, seq: 5, name: 'ExitPlanMode', title: 'Finish the plan', options: ['deny'], planSeq: 4 };
  const row: RelayThreadRow = { ...idle, status: 'waiting', pendingApproval: pending };
  const card: Card = { pending, slot: 5 * 8 + 1 };
  const withPlan = relayEvents(T, [start, PLAN], row, [card], false, 'bruce').find((e) => e.type === 'approval.requested');
  assert.equal(withPlan?.itemId, 'relay-4');
  // Without the plan row (an older backend's, or not read yet) the card
  // stands on its own: one naming a plan that isn't drawn would show nothing.
  const { plan: _plan, ...asText } = PLAN;
  const without = relayEvents(T, [start, asText], row, [card], false, 'bruce').find((e) => e.type === 'approval.requested');
  assert.equal(without?.itemId, undefined);
});

test("another lab's review goes on the last turn's end", () => {
  const review: LabReviewLedger = { reviewer: 'Codex (codex-1)', independent: true, rounds: 1, threadId: 'r1', passed: false, findings: [{ id: 1, round: 1, severity: 'high', claim: 'Drops the session on refresh.', outcome: 'fixed' }] };
  const reply: RelayRow = { seq: 6, role: 'assistant', text: 'Fixed it.', turn: 'aaaa1111', at: at(3) };
  const ev = relayEvents(T, [start, reply], { ...idle, needs: 'nothing', review }, [], false, 'bruce');
  const end = ev.filter((e) => e.type === 'turn.completed').at(-1);
  assert.deepEqual(end?.review, review);
});
