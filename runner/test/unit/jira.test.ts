import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { jiraKeys, readJiraConfig, jiraTicket, titleWithTicket } from '../../src/jira.ts';

test('jira: finds ticket keys in a message, first ones first, once each', () => {
  assert.deepEqual(jiraKeys('MACG-3375 tech design'), ['MACG-3375']);
  assert.deepEqual(jiraKeys('see https://x.atlassian.net/browse/PI-517 and macg-12, PI-517 again'), ['PI-517', 'MACG-12']);
  assert.deepEqual(jiraKeys('no ticket here, just a-1 and 3375'), []);
  assert.deepEqual(jiraKeys('A-1 B1-2 C-3 D-4'), ['B1-2'], 'a one-letter project is not a Jira key');
  assert.equal(jiraKeys('AB-1 AC-2 AD-3 AE-4').length, 3, 'three at most');
  assert.deepEqual(jiraKeys('x'.repeat(200_000) + ' MACG-1'), ['MACG-1']);
});

test('jira: reads secrets/jira.json, and only with an https site and all three fields', () => {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'tb-jira-'));
  assert.equal(readJiraConfig(home), undefined, 'no file');
  fs.mkdirSync(path.join(home, 'secrets'));
  const file = path.join(home, 'secrets', 'jira.json');
  fs.writeFileSync(file, JSON.stringify({ site: 'https://example.atlassian.net/', email: 'me@example.com', token: 't' }));
  assert.deepEqual(readJiraConfig(home), { site: 'https://example.atlassian.net', email: 'me@example.com', token: 't' });
  fs.writeFileSync(file, JSON.stringify({ site: 'http://example.atlassian.net', email: 'me@example.com', token: 't' }));
  assert.equal(readJiraConfig(home), undefined, 'plain http would send the token in the clear');
  fs.writeFileSync(file, JSON.stringify({ site: 'https://example.atlassian.net', email: 'me@example.com' }));
  assert.equal(readJiraConfig(home), undefined, 'no token');
  fs.writeFileSync(file, '{not json');
  assert.equal(readJiraConfig(home), undefined, 'broken file');
});

test('jira: looks a ticket up and gives its summary; any failure is no ticket', async () => {
  const cfg = { site: 'https://example.atlassian.net', email: 'me@example.com', token: 'secret' };
  const asked: { url: string; auth: string | null }[] = [];
  const ok: typeof fetch = async (url, init) => {
    asked.push({ url: String(url), auth: new Headers(init?.headers).get('authorization') });
    return new Response(JSON.stringify({ key: 'MACG-3375', fields: { summary: '  Force reclassifications\n of URLs ' } }), { status: 200 });
  };
  assert.deepEqual(await jiraTicket(cfg, ['MACG-3375'], ok), { key: 'MACG-3375', summary: 'Force reclassifications of URLs' });
  assert.equal(asked[0]!.url, 'https://example.atlassian.net/rest/api/3/issue/MACG-3375?fields=summary');
  assert.equal(asked[0]!.auth, `Basic ${Buffer.from('me@example.com:secret').toString('base64')}`);

  const missing: typeof fetch = async (url) => (String(url).includes('UTF-8') ? new Response('{}', { status: 404 }) : ok(url));
  assert.deepEqual(await jiraTicket(cfg, ['UTF-8', 'MACG-3375'], missing), { key: 'MACG-3375', summary: 'Force reclassifications of URLs' }, 'a key that is not a ticket is skipped');
  assert.equal(await jiraTicket(cfg, ['MACG-1'], async () => new Response('', { status: 401 })), undefined);
  assert.equal(await jiraTicket(cfg, ['MACG-1'], async () => { throw new Error('offline'); }), undefined);
  assert.equal(await jiraTicket(cfg, [], ok), undefined);
});

test('jira: the title starts with the ticket key, once, and fits', () => {
  assert.equal(titleWithTicket('MACG-3375', 'Force reclassifications tech design'), 'MACG-3375 - force reclassifications tech design');
  assert.equal(titleWithTicket('MACG-3375', 'MACG-3375 force reclassifications tech design'), 'MACG-3375 - force reclassifications tech design', 'the model said the key too');
  assert.equal(titleWithTicket('PI-517', 'URL list for Guardian admins'), 'PI-517 - URL list for Guardian admins', 'an acronym keeps its capitals');
  const long = titleWithTicket('MACG-3375', 'Force reclassifications of every URL in the district tech design');
  assert.ok(long.length <= 60, long);
  assert.ok(long.startsWith('MACG-3375 - force reclassifications'), long);
  assert.ok(!long.endsWith(' '), long);
});
