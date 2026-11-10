// A Jira ticket named in a thread's first message, for its title
// (runner/titles.ts): "MACG-3375 tech design" is named "MACG-3375 - force
// reclassifications tech design", from the ticket's summary.
//
// The title model runs with no tools (models/claude-pool.ts), so the runner
// asks Jira itself, with an Atlassian API token kept in
// secrets/jira.json: { "site": "https://<org>.atlassian.net", "email",
// "token" }. Without that file, or when Jira doesn't answer, titles are
// written as before. The token is never logged.
import fs from 'node:fs';
import path from 'node:path';
import { LIMITS } from './protocol.ts';
import { credentialUrlProblem } from './secure-url.ts';

export type JiraConfig = { site: string; email: string; token: string };
export type JiraTicket = { key: string; summary: string };

// Keys looked up at most: "UTF-8" or "SHA-256" look like keys too, and each
// one that isn't a ticket is a call to Jira.
const KEYS_MAX = 3;
const TIMEOUT_MS = 5_000;
const SUMMARY_CHARS = 200;

// Ticket keys in a message, in order, each once: a project key of 2 to 10
// letters and digits starting with a letter, a dash, a number. Lowercase
// ones count too ("macg-12"). Bounded quantifiers, so a long run of letters
// costs a little at each position, not a rescan (linear-text.ts).
export function jiraKeys(text: string): string[] {
  const keys: string[] = [];
  for (const m of (text ?? '').matchAll(/\b[A-Za-z][A-Za-z0-9]{1,9}-\d{1,7}\b/g)) {
    const key = m[0].toUpperCase();
    if (!keys.includes(key)) keys.push(key);
    if (keys.length >= KEYS_MAX) break;
  }
  return keys;
}

// secrets/jira.json, or undefined when it's missing, broken, short of a
// field, or its site isn't https (the token goes with every call).
export function readJiraConfig(home: string): JiraConfig | undefined {
  let raw: unknown;
  try {
    raw = JSON.parse(fs.readFileSync(path.join(home, 'secrets', 'jira.json'), 'utf8'));
  } catch {
    return undefined;
  }
  const c = raw as Partial<Record<keyof JiraConfig, unknown>> | null;
  if (!c || typeof c.site !== 'string' || typeof c.email !== 'string' || typeof c.token !== 'string') return undefined;
  if (!c.site.trim() || !c.email.trim() || !c.token.trim()) return undefined;
  if (credentialUrlProblem(c.site)) return undefined;
  return { site: c.site.trim().replace(/\/+$/, ''), email: c.email.trim(), token: c.token.trim() };
}

// The first of `keys` Jira knows, with its summary on one line. A key it
// doesn't know (404), no access, a timeout or no network: the next key,
// then undefined.
export async function jiraTicket(cfg: JiraConfig, keys: string[], fetchImpl: typeof fetch = fetch): Promise<JiraTicket | undefined> {
  const auth = `Basic ${Buffer.from(`${cfg.email}:${cfg.token}`).toString('base64')}`;
  for (const key of keys.slice(0, KEYS_MAX)) {
    try {
      const res = await fetchImpl(`${cfg.site}/rest/api/3/issue/${encodeURIComponent(key)}?fields=summary`, {
        headers: { authorization: auth, accept: 'application/json' },
        signal: AbortSignal.timeout(TIMEOUT_MS),
        redirect: 'error',
      });
      if (!res.ok) continue;
      const body = (await res.json()) as { fields?: { summary?: unknown } };
      const summary = typeof body.fields?.summary === 'string' ? body.fields.summary.split(/\s+/).filter(Boolean).join(' ').slice(0, SUMMARY_CHARS) : '';
      if (summary) return { key, summary };
    } catch {
      // the next key, or none
    }
  }
  return undefined;
}

// "<KEY> - <title>": the model's title without the key wherever it wrote it
// too ("… for MACG-3375"), its first letter lowercased only when the next
// word is lowercase too, so "Force reclassifications" reads on from the dash
// but "Family Portal" and "URL list" keep their capitals; cut at a word to
// fit LIMITS.titleChars. A key is letters, digits and one dash, so it goes
// into the pattern as it is.
export function titleWithTicket(key: string, title: string): string {
  let rest = title
    .replace(new RegExp(`(?:\\b(?:for|on|in|of)\\s+)?\\b${key}\\b`, 'gi'), ' ')
    .split(/\s+/)
    .filter(Boolean)
    .join(' ')
    .replace(/^[:—–-]+\s*/, '');
  const [first, second] = rest.split(' ');
  if (first && first.length > 1 && first.slice(1) === first.slice(1).toLowerCase() && (!second || second === second.toLowerCase())) rest = first[0]!.toLowerCase() + rest.slice(1);
  let out = rest ? `${key} - ${rest}` : key;
  if (out.length > LIMITS.titleChars) {
    const cut = out.slice(0, LIMITS.titleChars + 1);
    const space = cut.lastIndexOf(' ');
    out = (space > key.length + 3 ? cut.slice(0, space) : cut.slice(0, LIMITS.titleChars)).trimEnd();
  }
  return out;
}
