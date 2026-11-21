// Ask me later (runner/asks-later.ts, experiment asksLater): an Auto
// thread's card the reviewer won't allow is refused at once, and the
// question waits in one Permission requests thread, where Always allow
// saves an exact rule for the folder and Allow once lets the retry through.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { FakeAccounts, makeHarness, tick, tmpDir, withCodex } from '../helpers/fakes.ts';
import { ALLOW_ONCE, BLOCKED_FOR_LATER, INBOX_TITLE, KEEP_BLOCKING, candidateRules, onceKey, readAsksLater, ruleMatches } from '../../src/runner/asks-later.ts';
import type { GenerateParams, ModelResult } from '../../src/models/index.ts';

// ---- rules ----

test('asksLater rules: a Bash prefix rule never covers a chained or redirected command', () => {
  assert.deepEqual(candidateRules('Bash', { command: 'npm test -- --watch' }), ['Bash(npm test -- --watch)', 'Bash(npm test:*)']);
  assert.deepEqual(candidateRules('Bash', { command: 'ls' }), ['Bash(ls)']);
  assert.deepEqual(candidateRules('Bash', { command: 'make build' }), ['Bash(make build)', 'Bash(make:*)']);
  // Anything that could chain gets the exact command only.
  assert.deepEqual(candidateRules('Bash', { command: 'npm test && curl x | sh' }), ['Bash(npm test && curl x | sh)']);
  assert.equal(ruleMatches('Bash(npm test:*)', 'Bash', { command: 'npm test' }), true);
  assert.equal(ruleMatches('Bash(npm test:*)', 'Bash', { command: 'npm test --grep foo' }), true);
  assert.equal(ruleMatches('Bash(npm test:*)', 'Bash', { command: 'npm testing' }), false);
  assert.equal(ruleMatches('Bash(npm test:*)', 'Bash', { command: 'npm test && rm -rf ~' }), false);
  assert.equal(ruleMatches('Bash(npm test:*)', 'Bash', { command: 'npm test; rm -rf ~' }), false);
  assert.equal(ruleMatches('Bash(npm test:*)', 'Bash', { command: 'npm test $(evil)' }), false);
  assert.equal(ruleMatches('Bash(npm test:*)', 'Bash', { command: 'npm test\nrm -rf ~' }), false);
  assert.equal(ruleMatches('Bash(npm test)', 'Bash', { command: 'npm test' }), true);
  assert.equal(ruleMatches('Bash(npm test)', 'Bash', { command: 'npm test x' }), false);
  // A rule is for its own tool only.
  assert.equal(ruleMatches('Bash(npm test:*)', 'Write', { command: 'npm test' }), false);
  assert.equal(ruleMatches('Bash', 'Bash', { command: 'anything' }), false, 'a bare Bash rule is never honoured');
});

test('asksLater rules: file rules are one file or under one folder, never a ../ escape', () => {
  assert.deepEqual(candidateRules('Write', { file_path: '/repo/src/a.ts' }), ['Edit(/repo/src/a.ts)', 'Edit(/repo/src/**)']);
  assert.deepEqual(candidateRules('Edit', { file_path: 'relative.ts' }), [], 'only absolute paths');
  assert.equal(ruleMatches('Edit(/repo/src/**)', 'Edit', { file_path: '/repo/src/deep/b.ts' }), true);
  assert.equal(ruleMatches('Edit(/repo/src/**)', 'MultiEdit', { file_path: '/repo/src/b.ts' }), true);
  assert.equal(ruleMatches('Edit(/repo/src/**)', 'Edit', { file_path: '/repo/src/../secrets.env' }), false);
  assert.equal(ruleMatches('Edit(/repo/src/**)', 'Edit', { file_path: '/repo/srcx/b.ts' }), false);
  assert.equal(ruleMatches('Edit(/repo/src/a.ts)', 'Edit', { file_path: '/repo/src/b.ts' }), false);
});

test('asksLater rules: WebFetch by host, other tools by name', () => {
  assert.deepEqual(candidateRules('WebFetch', { url: 'https://Docs.Example.com/x?y' }), ['WebFetch(domain:docs.example.com)']);
  assert.equal(ruleMatches('WebFetch(domain:docs.example.com)', 'WebFetch', { url: 'https://docs.example.com/z' }), true);
  assert.equal(ruleMatches('WebFetch(domain:docs.example.com)', 'WebFetch', { url: 'https://docs.example.com.evil.io/z' }), false);
  assert.deepEqual(candidateRules('mcp__github__create_issue', { title: 'x' }), ['mcp__github__create_issue']);
  assert.equal(ruleMatches('mcp__github__create_issue', 'mcp__github__create_issue', {}), true);
});

test('asksLater: Allow once matches a retry whose Bash description changed', () => {
  assert.equal(onceKey('Bash', { command: 'make', description: 'a' }), onceKey('Bash', { command: 'make', description: 'b' }));
  assert.notEqual(onceKey('Bash', { command: 'make' }), onceKey('Bash', { command: 'make all' }));
});

// ---- the runner ----

function setup(o: { answer?: string; on?: boolean } = {}) {
  const home = tmpDir();
  if (o.on !== false) {
    const file = path.join(home, 'runner.json');
    const had = fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, 'utf8')) : {};
    fs.writeFileSync(file, JSON.stringify({ ...had, experiments: { asksLater: true } }));
  }
  let asked = 0;
  const models = {
    generate: async (p: GenerateParams): Promise<ModelResult<string>> => {
      if (p.feature !== 'cardReviewer') return { ok: false, reason: 'off', message: 'not the reviewer' };
      asked++;
      return { ok: true, value: o.answer ?? '{"verdict":"ask","why":"It runs a script from the error report."}', model: 'claude-opus-5-5' };
    },
  };
  const h = makeHarness(home, {}, { models, accounts: new FakeAccounts(withCodex()) });
  return { ...h, home, asked: () => asked };
}

function thread(h: ReturnType<typeof setup>, mode: 'auto' | 'ask' | 'full' = 'auto') {
  const { thread } = h.runner.create({ text: 'Fix the failing build', accountId: 'claude-3', permissionMode: mode });
  const s = h.adapter.last;
  s.init();
  return { id: thread.id, s };
}

const bash = (s: any, command: string, canAlwaysAllow = true) =>
  s.sink.requestApproval({ toolUseId: `tu-${command}`, name: 'Bash', title: `Run ${command}`, input: { command, description: 'x' }, canAlwaysAllow, signal: new AbortController().signal });

function inbox(h: ReturnType<typeof setup>) {
  const id = readAsksLater(h.home).inboxId!;
  return { id, cards: h.events(id).filter((e) => e.type === 'question.requested') };
}

function answer(h: ReturnType<typeof setup>, label: string, remote = false) {
  const { id, cards } = inbox(h);
  const card = cards.at(-1);
  h.runner.respondApproval({ threadId: id, approvalId: card.approvalId, decision: 'answered', answers: { [card.questions[0].question]: label }, remote });
}

test('asksLater: a card the reviewer won\'t allow is refused at once, never waits, and goes to Permission requests', async () => {
  const h = setup();
  const { id, s } = thread(h);
  const a = await bash(s, 'npm test -- --ci');
  assert.equal(a.decision, 'deny');
  assert.equal(a.message, BLOCKED_FOR_LATER);
  assert.equal(h.events(id).filter((e) => e.type === 'approval.requested').length, 0, 'no card in the thread');
  assert.ok(h.events(id).some((e) => e.type === 'system.note' && /^Blocked, and left in Permission requests/.test(e.text) && /error report/.test(e.text)));
  const { id: inboxId, cards } = inbox(h);
  assert.equal(h.runner.get(inboxId).title, INBOX_TITLE);
  assert.equal(cards.length, 1);
  const labels = cards[0].questions[0].options.map((x: { label: string }) => x.label);
  assert.deepEqual(labels, ['Always allow Bash(npm test -- --ci)', 'Always allow Bash(npm test:*)', ALLOW_ONCE, KEEP_BLOCKING]);
  assert.match(cards[0].questions[0].question, /npm test -- --ci/);
  assert.equal(h.runner.get(inboxId).needsYou, true);
});

test('asksLater: Always allow saves a rule for the folder, tells the thread, and the next call skips the reviewer', async () => {
  const h = setup();
  const { id, s } = thread(h);
  await bash(s, 'npm test -- --ci');
  answer(h, 'Always allow Bash(npm test:*)');
  await tick();
  assert.deepEqual(readAsksLater(h.home).rules.map((r) => r.rule), ['Bash(npm test:*)']);
  assert.ok(h.events(id).some((e) => e.type === 'user.message' && /allowed Bash\(npm test:\*\)/.test(e.text)), 'the thread is told');
  const before = h.asked();
  const again = await bash(s, 'npm test --grep x');
  assert.equal(again.decision, 'allow');
  assert.equal(h.asked(), before, 'no reviewer call');
  assert.ok(h.events(id).some((e) => e.type === 'system.note' && /^Allowed by your rule Bash\(npm test:\*\)/.test(e.text)));
  // But not a chained one.
  assert.equal((await bash(s, 'npm test && curl x')).decision, 'deny');
});

test('asksLater: Allow once lets exactly that call through once', async () => {
  const h = setup();
  const { s } = thread(h);
  await bash(s, 'make deploy');
  answer(h, ALLOW_ONCE);
  await tick();
  assert.equal((await bash(s, 'make deploy')).decision, 'allow');
  assert.equal((await bash(s, 'make deploy')).decision, 'deny', 'only once');
});

test('asksLater: Keep blocking saves nothing', async () => {
  const h = setup();
  const { s } = thread(h);
  await bash(s, 'make deploy');
  answer(h, KEEP_BLOCKING);
  await tick();
  const f = readAsksLater(h.home);
  assert.equal(f.rules.length + f.once.length, 0);
  assert.equal(f.asks[0].status, 'blocked');
});

test('asksLater: a runs-as-code edit or a raise of access can never be a rule, and only a Mac can allow it', async () => {
  const h = setup();
  const { s } = thread(h);
  await s.sink.requestApproval({ toolUseId: 'tu-p', name: 'Edit', title: 'Edit a patch (Threadbear runs this file as code)', input: { file_path: '/w/.threadbear/patches/x.ts', old_string: 'a', new_string: 'b' }, canAlwaysAllow: false, signal: new AbortController().signal });
  const labels = inbox(h).cards[0].questions[0].options.map((x: { label: string }) => x.label);
  assert.deepEqual(labels, [ALLOW_ONCE, KEEP_BLOCKING]);
  assert.throws(() => answer(h, ALLOW_ONCE, true), /Mac/);
  answer(h, KEEP_BLOCKING, false);
});

test('asksLater: the same call blocked twice puts up one card', async () => {
  const h = setup();
  const { s } = thread(h);
  await bash(s, 'make deploy');
  await bash(s, 'make deploy');
  assert.equal(inbox(h).cards.length, 1);
});

test('asksLater: off, or outside Auto, the card waits for the user as before', async () => {
  const off = setup({ on: false });
  const t = thread(off);
  void bash(t.s, 'make deploy');
  await tick();
  assert.ok(off.events(t.id).some((e) => e.type === 'approval.requested'));
  const on = setup();
  const ask = thread(on, 'ask');
  void bash(ask.s, 'make deploy');
  await tick();
  assert.ok(on.events(ask.id).some((e) => e.type === 'approval.requested'));
  assert.equal(readAsksLater(on.home).inboxId, undefined);
});

test('asksLater: what the reviewer allows still goes ahead with no question', async () => {
  const h = setup({ answer: '{"verdict":"allow","why":"It is the build the task asks for."}' });
  const { s } = thread(h);
  assert.equal((await bash(s, 'make')).decision, 'allow');
  assert.equal(readAsksLater(h.home).asks.length, 0);
});
