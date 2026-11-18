// runner.json accountCaps: a login capped at 50% counts as out once half
// of its weekly or 5-hour window is used, and is back when that resets.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import type { Account, Pool } from '../../src/protocol.ts';
import { backAt, limitWhy, loginOk, nextLogin, pickInPool } from '../../src/pools.ts';
import { accountRoom, pickAccount } from '../../src/usage.ts';
import { capMap } from '../../src/config.ts';
import { discoverAccounts } from '../../src/accounts.ts';
import { tmpDir } from '../helpers/fakes.ts';

const NOW = Date.parse('2026-11-17T08:00:00.000Z');
const RESET = '2026-11-20T08:00:00.000Z';

function login(id: string, weekly: number, cap?: number): Account {
  const a: Account = {
    id,
    provider: 'claude',
    title: id,
    configDir: `/tmp/${id}`,
    usage: { windows: [{ kind: 'weekly', title: 'Weekly', remainingPercent: weekly, resetsAt: RESET }], readAt: new Date(NOW).toISOString(), source: 'live' },
  };
  if (cap) a.usageCap = cap;
  return a;
}

const pool: Pool = { id: 'claude', title: 'Claude', accountIds: ['claude-main', 'claude-1'] };

test('accountRoom counts down to the cap', () => {
  assert.equal(accountRoom(login('a', 70, 50)), 20);
  assert.equal(accountRoom(login('a', 40, 50)), 0);
  assert.equal(accountRoom(login('a', 70)), 70);
});

test('a capped login past its cap is out until the window resets', () => {
  const capped = login('claude-1', 50.5, 50);
  assert.equal(loginOk(capped), false);
  assert.equal(backAt(capped, NOW), Date.parse(RESET));
  assert.match(limitWhy(capped, NOW), /hit its weekly limit \(capped at 50%\)/);
  assert.equal(loginOk(login('claude-1', 60, 50)), true);
  assert.match(limitWhy(login('claude-1', 60, 50), NOW), /down to 10% of its weekly limit \(capped at 50%\)/);
});

test('threads move off a capped login, or wait when none has room', () => {
  const accounts = [login('claude-main', 0.5), login('claude-1', 45, 50)];
  assert.equal(pickInPool(pool, accounts, { provider: 'claude' }), undefined);
  const next = nextLogin(pool, accounts, { accountId: 'claude-main', provider: 'claude' }, NOW);
  assert.ok('waitUntil' in next);
  const roomy = [login('claude-main', 30), login('claude-1', 90, 50)];
  // claude-1 has 40 usable against claude-main's 30, same reset: most room wins.
  assert.equal(pickInPool(pool, roomy, { provider: 'claude' }), 'claude-1');
  assert.equal(pickAccount(roomy, 'claude-main'), 'claude-1');
  assert.equal(pickAccount([login('claude-main', 30), login('claude-1', 70, 50)], 'claude-main'), 'claude-main');
});

test('capMap keeps caps from 1 to 99', () => {
  assert.deepEqual(capMap({ 'claude-1': 50, a: 0, b: 100, c: '50', d: 99.5 }), { 'claude-1': 50, d: 99.5 });
  assert.equal(capMap({ a: 0 }), undefined);
  assert.equal(capMap(undefined), undefined);
});

test('discoverAccounts puts the cap on the login', () => {
  const home = tmpDir();
  const accountsDir = path.join(home, 'accounts');
  fs.mkdirSync(path.join(accountsDir, 'claude-1'), { recursive: true });
  const found = discoverAccounts({ claudeMainDir: path.join(home, 'none'), claudeMainJson: path.join(home, 'none.json'), tokenStatsAccountsDir: path.join(home, 'nope'), accountsDir, userHome: home, pathEnv: '', accountCaps: { 'claude-1': 50 } } as never);
  assert.equal(found.find((a) => a.id === 'claude-1')?.usageCap, 50);
});
