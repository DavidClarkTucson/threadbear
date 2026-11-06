import { useEffect, useMemo, useState } from 'react';
import type { ThreadSummary } from '../../runner/src/protocol.ts';
import { call, currentStatus, errorText, onNotification, onStatus, type Status } from './runner.ts';
import { sections } from './inbox.ts';
import { ConnectionNote, Sidebar } from './Sidebar.tsx';
import { ThreadView } from './ThreadView.tsx';

export function App() {
  const [status, setStatus] = useState<Status>();
  const [threads, setThreads] = useState<Map<string, ThreadSummary>>(new Map());
  const [selected, setSelected] = useState<string>();
  const [listError, setListError] = useState<string>();

  useEffect(() => {
    const offs = [
      onStatus(setStatus),
      onNotification(({ method, params }) => {
        if (method === 'thread.updated') setThreads((m) => new Map(m).set(params.thread.id, params.thread));
        else if (method === 'thread.deleted')
          setThreads((m) => {
            const next = new Map(m);
            next.delete(params.threadId);
            return next;
          });
      }),
    ];
    // The status may have moved on before we listened.
    currentStatus().then(setStatus);
    return () => offs.forEach((p) => p.then((off) => off()));
  }, []);

  // Each time it's connected (first time, or after the runner restarted),
  // the whole list again: thread.updated only says what changed since.
  const connected = status?.state === 'connected';
  useEffect(() => {
    if (!connected) return;
    call<{ threads: ThreadSummary[] }>('threads.list', { include: ['inbox', 'snoozed'] })
      .then(({ threads }) => {
        setThreads(new Map(threads.map((t) => [t.id, t])));
        setListError(undefined);
      })
      .catch((e) => setListError(errorText(e)));
  }, [connected, status]);

  const shown = useMemo(() => sections([...threads.values()].filter((t) => !t.archivedAt)), [threads]);
  const thread = selected ? threads.get(selected) : undefined;

  return (
    <div className="app">
      <Sidebar sections={shown} selected={selected} onSelect={setSelected} status={status} error={listError} />
      <main className="main">
        {thread ? (
          <ThreadView key={thread.id} thread={thread} connected={connected} />
        ) : (
          <div className="empty">{connected ? 'No thread open' : <ConnectionNote status={status} />}</div>
        )}
      </main>
    </div>
  );
}
