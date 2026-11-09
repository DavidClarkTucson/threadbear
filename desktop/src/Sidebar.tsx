import { folderName, ago } from '../../web/src/format.ts';
import { wantsYou, type Row, type Section } from './inbox.ts';
import type { Status } from './runner.ts';

type Props = { sections: Section[]; selected?: string; onSelect: (id: string) => void; status?: Status; error?: string };

export function Sidebar({ sections, selected, onSelect, status, error }: Props) {
  return (
    <nav className="side">
      <header className="side-head">
        <span className="brand">Threadbear (Tauri)</span>
        <span className={`conn conn-${status?.state ?? 'connecting'}`} title={status?.state === 'connected' ? `Runner ${status.hello.runnerVersion}` : undefined} />
      </header>
      {status?.state !== 'connected' && (
        <div className="side-note">
          <ConnectionNote status={status} />
        </div>
      )}
      {error && <div className="side-note error">{error}</div>}
      <div className="side-list">
        {sections.map((s) => (
          <section key={s.key}>
            <h2>
              {s.title} <span className="count">{s.rows.length}</span>
            </h2>
            {s.rows.length === 0 && <div className="side-empty">Nothing here</div>}
            {s.rows.map((r) => (
              <ThreadRowView key={r.threadId} row={r} selected={r.threadId === selected} onSelect={onSelect} />
            ))}
          </section>
        ))}
      </div>
    </nav>
  );
}

function ThreadRowView({ row, selected, onSelect }: { row: Row; selected: boolean; onSelect: (id: string) => void }) {
  const busy = row.status === 'running' || row.status === 'starting';
  const place = row.projects?.length ? row.projects[0].title : folderName(row.cwd);
  return (
    <button className={`row${selected ? ' selected' : ''}`} onClick={() => onSelect(row.threadId)} title={row.title}>
      <span className={`dot${busy ? ' busy' : wantsYou(row) ? ' you' : row.unread ? ' unread' : ''}`} />
      <span className="row-body">
        <span className="row-title">{row.title}</span>
        <span className="row-sub">
          {place} · {ago(row.lastActivityMs)}
        </span>
      </span>
    </button>
  );
}

export function ConnectionNote({ status }: { status?: Status }) {
  if (!status || status.state === 'connecting') return <span>Connecting to the runner…</span>;
  if (status.state === 'disconnected')
    return (
      <span>
        Can't reach the runner: {status.reason}. Trying again in {Math.ceil(status.retryInMs / 1000)}s.
      </span>
    );
  return <span>Connected to runner {status.hello.runnerVersion}</span>;
}
