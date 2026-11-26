// Auto for every thread (experiment autoEverywhere, runner/new-threads.ts):
// with it on, a new Claude thread starts in Auto whatever mode it was
// asked for, so what it's refused waits in Permission requests (Ask me
// later) instead of the thread running unchecked or waiting on a card.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { FakeAccounts, makeHarness, tmpDir, withCodex } from '../helpers/fakes.ts';

function setup(on: boolean) {
  const home = tmpDir();
  if (on) fs.writeFileSync(path.join(home, 'runner.json'), JSON.stringify({ experiments: { autoEverywhere: true } }));
  return makeHarness(home, {}, { accounts: new FakeAccounts(withCodex()) });
}

const mode = (h: ReturnType<typeof setup>, p: { accountId: string; permissionMode?: 'full' | 'ask' | 'auto' | 'plan' }) =>
  h.runner.create({ text: 'Tidy the build', ...p }).thread.permissionMode;

test('autoEverywhere: on, a Claude thread starts in Auto whatever it asked for', () => {
  const h = setup(true);
  assert.equal(mode(h, { accountId: 'claude-3', permissionMode: 'full' }), 'auto');
  assert.equal(mode(h, { accountId: 'claude-3', permissionMode: 'ask' }), 'auto');
  assert.equal(mode(h, { accountId: 'claude-3' }), 'auto');
});

test('autoEverywhere: Plan first, Codex threads and the experiment off keep the mode asked for', () => {
  const h = setup(true);
  assert.equal(mode(h, { accountId: 'claude-3', permissionMode: 'plan' }), 'plan');
  const codex = withCodex().find((a) => a.provider === 'codex')!.id;
  assert.equal(mode(h, { accountId: codex, permissionMode: 'full' }), 'full');
  const off = setup(false);
  assert.equal(mode(off, { accountId: 'claude-3', permissionMode: 'full' }), 'full');
});
