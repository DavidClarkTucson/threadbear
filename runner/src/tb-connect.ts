// How tb reaches the runner. Inside a thread that's with the thread's own key
// (docs/protocol.md, "A thread's own key"), never the token. Outside one, or
// pointed at another runner's data dir (a dev runner, whose token it reads and
// whose threads it doesn't belong to), it's that data dir's token.
import path from 'node:path';
import { readToken, runnerUrl, type Config } from './config.ts';
import { RunnerClient, type RunnerClientOptions } from './client.ts';
import { isThreadKey } from './rpc-auth.ts';

const sameDir = (a: string, b: string) => path.resolve(a) === path.resolve(b);

export type TbCredential = { token: string; threadId?: string };

export function tbCredential(home: string, env: NodeJS.ProcessEnv = process.env): TbCredential {
  const key = env.THREADBEAR_THREAD_KEY;
  const threadId = env.THREADBEAR_THREAD_ID;
  if (isThreadKey(key) && threadId && sameDir(env.THREADBEAR_RUNNER_HOME ?? home, home)) return { token: key, threadId };
  return { token: readToken(home) };
}

const options = (c: TbCredential, opts: RunnerClientOptions): RunnerClientOptions => ({ ...opts, ...(c.threadId ? { threadId: c.threadId } : {}) });

export function connectTb(cfg: Config, url: string = runnerUrl(cfg), opts: RunnerClientOptions = {}): Promise<RunnerClient> {
  const c = tbCredential(cfg.home);
  return RunnerClient.connect(url, c.token, options(c, opts));
}

// For a caller that drives `opened` itself, with its own timeout.
export function newTbClient(cfg: Config, url: string = runnerUrl(cfg), opts: RunnerClientOptions = {}): RunnerClient {
  const c = tbCredential(cfg.home);
  return new RunnerClient(url, c.token, options(c, opts));
}

// Why tb's commands that use this Mac's keys straight from disk (tb runner,
// tb pair) shouldn't run here, or undefined if they may: they don't go through
// the RPC, so the runner can't refuse them itself. Asked of the runner, so a
// wiped environment doesn't count. If the runner can't be reached, only a
// command with no thread key in its environment goes ahead.
export async function ownerOnly(cfg: Config, env: NodeJS.ProcessEnv = process.env): Promise<string | undefined> {
  let client: RunnerClient;
  try {
    client = await connectTb(cfg);
  } catch {
    return isThreadKey(env.THREADBEAR_THREAD_KEY) ? "This needs Full access and the runner can't be reached to say whether this thread has it." : undefined;
  }
  try {
    const who = await client.call<{ limited?: boolean; mode?: string }>('auth.whoami');
    return who.limited ? `This thread is on ${who.mode}, and this command uses the runner's own keys. Ask the user to run it from a terminal.` : undefined;
  } catch {
    // An older runner, which has no auth.whoami.
    return undefined;
  } finally {
    client.close();
  }
}
