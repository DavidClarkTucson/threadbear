import { test } from 'node:test';
import assert from 'node:assert/strict';
import { sections, toRow } from '../src/inbox.ts';

const now = Date.parse('2026-11-06T12:00:00Z');
let n = 0;
function thread(over: Record<string, unknown> = {}): any {
  n++;
  return {
    id: `t${n}`,
    title: `Thread ${n}`,
    cwd: '/home/mike/dev/x',
    accountId: 'claude-main',
    provider: 'claude',
    permissionMode: 'full',
    status: 'idle',
    live: false,
    needsYou: false,
    createdAt: '2026-11-06T10:00:00Z',
    updatedAt: '2026-11-06T10:00:00Z',
    lastActivityAt: `2026-11-06T10:${String(n).padStart(2, '0')}:00Z`,
    preview: '',
    lastSeq: 10,
    lastSeenSeq: 10,
    ...over,
  };
}
const ids = (s: ReturnType<typeof sections>, key: string) => s.find((x) => x.key === key)?.rows.map((r) => r.threadId) ?? [];

test('threads go under the headings the Mac app uses', () => {
  const asks = thread({ needs: 'you', needsYou: true, asks: [{ action: 'Ship it?' }] });
  const running = thread({ status: 'running' });
  const done = thread();
  const asleep = thread({ snoozedUntil: '2026-11-07T00:00:00Z' });
  const starred = thread({ starredAt: '2026-11-06T09:00:00Z' });
  const s = sections([asks, running, done, asleep, starred], now);
  assert.deepEqual(ids(s, 'inbox'), [asks.id, done.id]);
  assert.deepEqual(ids(s, 'running'), [running.id]);
  assert.deepEqual(ids(s, 'hibernating'), [asleep.id]);
  assert.deepEqual(ids(s, 'starred'), [starred.id]);
});

test('a thread stuck on an answer comes first, then newest', () => {
  const older = thread();
  const newer = thread();
  const stuck = thread({ status: 'waiting' });
  assert.deepEqual(ids(sections([older, newer, stuck], now), 'inbox'), [stuck.id, newer.id, older.id]);
});

test('a thread with sub-agents still going is Running, whatever its last turn said', () => {
  const t = thread({ needs: 'you', background: [{ id: 'b', kind: 'agent', description: '', startedAt: '' }] });
  const s = sections([t], now);
  assert.deepEqual(ids(s, 'running'), [t.id]);
});

test('unread is anything after what was last seen', () => {
  assert.equal(toRow(thread({ lastSeq: 12, lastSeenSeq: 10 })).unread, true);
  assert.equal(toRow(thread()).unread, false);
});

test('an empty inbox still has its heading', () => {
  assert.deepEqual(
    sections([], now).map((s) => s.key),
    ['inbox'],
  );
});
