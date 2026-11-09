// Keeps a thread that isn't on Full access from reading the runner's secrets
// with Claude's own tools (docs/protocol.md, "Who's really calling"). Defence in
// depth behind the RPC's own check of the caller's process: a shell that's
// allowed to run anything can spell a path in ways no pattern will see, and
// SECURITY.md says so. A Full access thread can already run anything as you,
// so it isn't checked.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const REASON = "The runner's secrets aren't readable from a thread without Full access. tb in this thread already works with its own key.";

const PATH_KEYS = ['file_path', 'path', 'notebook_path'] as const;

function expand(p: string): string {
  return p === '~' || p.startsWith('~/') ? path.join(os.homedir(), p.slice(1)) : p;
}

// The deepest part of a path that exists, with links followed, and the rest
// on the end: a Read of secrets/runner-token through a link in the path is
// that file.
function resolved(p: string): string {
  let head = path.resolve(expand(p));
  const tail: string[] = [];
  for (;;) {
    try {
      return path.join(fs.realpathSync.native(head), ...tail);
    } catch {
      const up = path.dirname(head);
      if (up === head) return path.join(head, ...tail);
      tail.unshift(path.basename(head));
      head = up;
    }
  }
}

const inside = (p: string, dir: string) => p === dir || p.startsWith(dir + path.sep);

// The reason a call is denied, or undefined if it's fine. `home` is the
// runner's data dir.
export function denySecrets(tool: string, input: Record<string, unknown>, home: string): string | undefined {
  const secrets = path.join(home, 'secrets');
  if (tool === 'Bash') {
    const command = typeof input.command === 'string' ? input.command : '';
    // Quotes and backslashes that don't change which path it is.
    const flat = command.replace(/\\(.)/g, '$1').replace(/["']/g, '');
    const home2 = os.homedir();
    const forms = [
      secrets,
      secrets.replace(home2, '~'),
      secrets.replace(home2, '$HOME'),
      secrets.replace(home2, '${HOME}'),
      '$THREADBEAR_HOME/secrets',
      '${THREADBEAR_HOME}/secrets',
      '$THREADBARE_HOME/secrets',
      '${THREADBARE_HOME}/secrets',
      'Threadbear/secrets',
    ];
    if (/\brunner-token\b/.test(flat) || forms.some((f) => flat.includes(f))) return REASON;
    return undefined;
  }
  const target = resolved(secrets);
  for (const k of PATH_KEYS) {
    const v = input[k];
    if (typeof v === 'string' && v && (inside(resolved(v), target) || inside(path.resolve(expand(v)), secrets))) return REASON;
  }
  // A Glob or Grep run from above the folder, with a pattern that goes in.
  const pattern = input.pattern;
  if ((tool === 'Glob' || tool === 'Grep') && typeof pattern === 'string' && /\bsecrets\b|runner-token/.test(pattern)) {
    const base = typeof input.path === 'string' && input.path ? input.path : '.';
    const dir = resolved(base);
    if (inside(target, dir) || inside(resolved(path.join(base, pattern.split('*')[0] || '.')), target)) return REASON;
  }
  return undefined;
}
