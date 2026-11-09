import { test } from 'node:test';
import assert from 'node:assert/strict';
import { addEvents, answerMessage, emptyEvents, fold, questionAnswers } from '../src/transcript.ts';

let seq = 0;
const ev = (body: Record<string, unknown>) => ({ seq: ++seq, ts: '2026-11-06T00:00:00Z', threadId: 't1', ...body });
const kinds = (items: { kind: string }[]) => items.map((i) => i.kind);

test('a turn: message, work grouped into steps, reply and its end', () => {
  seq = 0;
  const s = addEvents(emptyEvents(), [
    ev({ type: 'user.message', text: 'hi', images: [] }),
    ev({ type: 'turn.started', turnId: 'a' }),
    ev({ type: 'tool.call', itemId: 'c1', name: 'Bash', kind: 'command', title: 'npm test', input: {} }),
    ev({ type: 'tool.call', itemId: 'c2', name: 'Read', kind: 'read', title: 'Read a.ts', input: {} }),
    ev({ type: 'tool.result', itemId: 'c1', ok: false, text: 'boom', truncated: false, images: [] }),
    ev({ type: 'assistant.text', itemId: 'm1', text: 'Done.' }),
    ev({ type: 'turn.completed', turnId: 'a', outcome: 'completed', durationMs: 1200, needs: 'nothing' }),
  ]);
  const items = fold(s);
  assert.deepEqual(kinds(items), ['user', 'steps', 'text', 'turnEnd']);
  const steps = items[1];
  assert.ok(steps.kind === 'steps');
  assert.equal(steps.steps.length, 2);
  assert.equal(steps.steps[0].ok, false);
  assert.equal(steps.steps[1].ok, undefined);
});

test('deltas stream, and the persisted text replaces them where they stand', () => {
  seq = 0;
  let s = addEvents(emptyEvents(), [ev({ type: 'user.message', text: 'hi', images: [] })]);
  s = addEvents(s, [
    { seq, ts: '', threadId: 't1', type: 'assistant.text.delta', itemId: 'm1', text: 'Hel' },
    { seq, ts: '', threadId: 't1', type: 'assistant.text.delta', itemId: 'm1', text: 'lo' },
  ]);
  let items = fold(s);
  assert.deepEqual(kinds(items), ['user', 'text']);
  assert.equal(items[1].kind === 'text' && items[1].text, 'Hello');
  assert.equal(items[1].kind === 'text' && items[1].streaming, true);

  s = addEvents(s, [ev({ type: 'assistant.text', itemId: 'm1', text: 'Hello there' })]);
  items = fold(s);
  assert.deepEqual(kinds(items), ['user', 'text']);
  assert.equal(items[1].kind === 'text' && items[1].text, 'Hello there');
  assert.equal(items[1].kind === 'text' && items[1].streaming, undefined);
});

test('reasoning that lands as text takes the reasoning row’s place', () => {
  seq = 0;
  let s = addEvents(emptyEvents(), [ev({ type: 'user.message', text: 'hi', images: [] })], [{ itemId: 'n1', type: 'reasoning', text: 'thinking' }]);
  assert.deepEqual(kinds(fold(s)), ['user', 'reasoning']);
  s = addEvents(s, [ev({ type: 'assistant.text', itemId: 'n1', text: 'the reply' })]);
  assert.deepEqual(kinds(fold(s)), ['user', 'text']);
});

test('a later assistant.text with the same itemId replaces the earlier one', () => {
  seq = 0;
  const s = addEvents(emptyEvents(), [
    ev({ type: 'assistant.text', itemId: 'm1', text: 'summary', summary: true }),
    ev({ type: 'tool.call', itemId: 'c1', name: 'x', kind: 'other', title: 'x', input: {} }),
    ev({ type: 'assistant.text', itemId: 'm1', text: 'the whole reply' }),
  ]);
  const items = fold(s);
  assert.deepEqual(kinds(items), ['text', 'steps']);
  assert.equal(items[0].kind === 'text' && items[0].text, 'the whole reply');
});

test('an edit hides the events it rewound', () => {
  seq = 0;
  const s = addEvents(emptyEvents(), [
    ev({ type: 'user.message', text: 'first try', images: [] }),
    ev({ type: 'assistant.text', itemId: 'm1', text: 'answer' }),
    ev({ type: 'thread.rewound', fromSeq: 1, toSeq: 2, files: { undone: [], failed: [], how: 'none' } }),
    ev({ type: 'user.message', text: 'second try', images: [], editOf: 1 }),
  ]);
  const items = fold(s);
  assert.deepEqual(kinds(items), ['user']);
  assert.equal(items[0].kind === 'user' && items[0].text, 'second try');
});

test('an approval shows whether it was answered', () => {
  seq = 0;
  const s = addEvents(emptyEvents(), [
    ev({ type: 'approval.requested', approvalId: 'p1', name: 'Bash', title: 'rm -rf build', input: {}, options: ['allow', 'deny'] }),
    ev({ type: 'approval.resolved', approvalId: 'p1', decision: 'deny' }),
  ]);
  const [item] = fold(s);
  assert.equal(item.kind === 'approval' && item.decision, 'deny');
});

test('the same events twice change nothing', () => {
  seq = 0;
  const batch = [ev({ type: 'user.message', text: 'hi', images: [] })];
  const s = addEvents(addEvents(emptyEvents(), batch), batch);
  assert.equal(fold(s).length, 1);
});

test('answering an ask sends the question and the answer', () => {
  assert.deepEqual(answerMessage({ action: 'Ship it?' }, 'Yes'), { text: 'Ship it?\nYes', answers: [{ action: 'Ship it?', answer: 'Yes' }] });
});

test('a question card, and what answering it sends', () => {
  seq = 0;
  const questions = [
    { question: 'Which colour?', multiSelect: false, options: [{ label: 'Blue' }, { label: 'Green' }] },
    { question: 'Where?', multiSelect: true, options: [{ label: 'Toolbar' }, { label: 'Menu' }] },
  ];
  let s = addEvents(emptyEvents(), [ev({ type: 'question.requested', approvalId: 'q1', questions })]);
  let [item] = fold(s);
  assert.equal(item.kind, 'question');
  assert.equal(item.kind === 'question' && item.decision, undefined);

  const sent = questionAnswers(questions, { 0: ['Green'], 1: ['Menu', 'Toolbar'] });
  assert.deepEqual(sent.answers, { 'Which colour?': 'Green', 'Where?': 'Toolbar, Menu' });
  assert.equal(sent.message, 'Green; Toolbar, Menu');
  assert.deepEqual(questionAnswers(questions, { 1: ['Menu'] }), { answers: { 'Where?': 'Menu' }, message: 'Menu' });

  s = addEvents(s, [ev({ type: 'approval.resolved', approvalId: 'q1', decision: 'answered', message: sent.message })]);
  [item] = fold(s);
  assert.equal(item.kind === 'question' && item.message, 'Green; Toolbar, Menu');
});
