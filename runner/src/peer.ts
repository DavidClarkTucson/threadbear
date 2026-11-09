// Which thread a process on this Mac belongs to, for the RPC server (docs/
// protocol.md, "Who's really calling"). Credentials alone can't say: anything
// a thread's shell runs can read the token file or wipe its own key from the
// environment. The kernel can say what it's a child of, so the server looks at
// the far end of the connection and walks up its parents to a thread's CLI.
import { execFile } from 'node:child_process';
import { lsofPath } from './handover.ts';
import { psTable, type ProcessRegistry } from './proc.ts';
import type { Logger } from './log.ts';

export type PeerDeps = {
  registry: Pick<ProcessRegistry, 'threadOfPid'>;
  // A thread this runner or a draining one really has (a registry entry can
  // be a job's label, like models:<account>).
  known: (threadId: string) => boolean;
  log: Logger;
  // Tests pass stand-ins.
  lsof?: (port: number) => Promise<number[]>;
  table?: () => Promise<{ pid: number; ppid: number }[]>;
};

// The processes with an established TCP connection using this port.
function lsofPids(port: number): Promise<number[]> {
  return new Promise((resolve, reject) => {
    execFile(lsofPath(), ['-nP', `-iTCP:${port}`, '-sTCP:ESTABLISHED', '-Fp'], { timeout: 3000 }, (err, stdout) => {
      // lsof exits 1 when nothing matched; what it printed still counts.
      if (err && (err as { code?: unknown }).code !== 1 && !stdout) return reject(err);
      resolve([...stdout.matchAll(/^p(\d+)$/gm)].map((m) => Number(m[1])));
    });
  });
}

// The thread the connection from this client port belongs to, or undefined
// when it isn't under a thread's CLI or the lookup can't tell (no lsof, on
// a machine without it): then the credential presented decides.
export async function threadOfPort(port: number, deps: PeerDeps): Promise<string | undefined> {
  try {
    const pids = await (deps.lsof ?? lsofPids)(port);
    const peer = pids.find((p) => p !== process.pid);
    if (peer === undefined) return undefined;
    const parents = new Map((await (deps.table ?? psTable)()).map((r) => [r.pid, r.ppid]));
    const seen = new Set<number>();
    for (let p: number | undefined = peer; p !== undefined && p > 1 && !seen.has(p); p = parents.get(p)) {
      seen.add(p);
      const t = deps.registry.threadOfPid(p);
      if (t && deps.known(t)) return t;
    }
    return undefined;
  } catch (e) {
    deps.log.warn('could not tell which process connected to the RPC', { error: String(e) });
    return undefined;
  }
}
