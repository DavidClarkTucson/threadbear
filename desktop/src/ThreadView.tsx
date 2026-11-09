import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import Markdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { ThreadSummary } from '../../runner/src/protocol.ts';
import { modelLabel, folderName } from '../../web/src/format.ts';
import { call, errorText, onNotification } from './runner.ts';
import { addEvents, answerMessage, duration, emptyEvents, fold, questionAnswers, type Item, type Step } from './transcript.ts';

type Props = { thread: ThreadSummary; connected: boolean };

export function ThreadView({ thread, connected }: Props) {
  const [state, setState] = useState(emptyEvents);
  const [error, setError] = useState<string>();
  const [loaded, setLoaded] = useState(false);

  // The newest 500 events, then a subscription from the last of them, so
  // nothing that lands in between is lost. Again after a reconnect.
  useEffect(() => {
    if (!connected) return;
    let cancelled = false;
    const id = thread.id;
    const off = onNotification(({ method, params }) => {
      if (method === 'thread.event' && params.threadId === id) setState((s) => addEvents(s, [params.event]));
    });
    (async () => {
      try {
        const page = await call<{ events: any[]; lastSeq: number }>('threads.events', { threadId: id, limit: 500 });
        if (cancelled) return;
        setState(addEvents(emptyEvents(), page.events));
        const sub = await call<{ events: any[]; inProgress: any[] }>('threads.subscribe', { threadId: id, afterSeq: page.lastSeq });
        if (cancelled) return;
        setState((s) => addEvents(s, sub.events, sub.inProgress));
        setLoaded(true);
      } catch (e) {
        if (!cancelled) setError(errorText(e));
      }
    })();
    return () => {
      cancelled = true;
      off.then((f) => f());
      call('threads.unsubscribe', { threadId: id }).catch(() => {});
    };
  }, [thread.id, connected]);

  // Opening it reads it, as on the Mac.
  useEffect(() => {
    if (loaded && thread.lastSeq > thread.lastSeenSeq) call('threads.markSeen', { threadId: thread.id, seq: thread.lastSeq }).catch(() => {});
  }, [loaded, thread.id, thread.lastSeq, thread.lastSeenSeq]);

  const items = useMemo(() => fold(state), [state]);
  const scroller = useRef<HTMLDivElement>(null);
  const pinned = useRef(true);
  useLayoutEffect(() => {
    const el = scroller.current;
    if (el && pinned.current) el.scrollTop = el.scrollHeight;
  }, [items]);

  const send = async (text: string, answers?: { action: string; answer: string }[]) => {
    pinned.current = true;
    await call('threads.send', { threadId: thread.id, text, ...(answers ? { answers } : {}) });
  };

  const busy = thread.status === 'running' || thread.status === 'starting';
  return (
    <div className="thread">
      <header className="thread-head">
        <h1>{thread.title}</h1>
        <div className="thread-meta">
          {folderName(thread.cwd)} · {modelLabel(thread.provider, thread.modelInUse ?? thread.model)} · {thread.status}
        </div>
      </header>
      <div
        className="transcript"
        ref={scroller}
        onScroll={(e) => {
          const el = e.currentTarget;
          pinned.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
        }}
      >
        {error && <div className="note error">{error}</div>}
        {!loaded && !error && <div className="note">Loading…</div>}
        {items.map((item) => (
          <ItemView key={item.key} item={item} thread={thread} send={send} />
        ))}
        {busy && <div className="working">Working…</div>}
      </div>
      <Composer busy={busy} disabled={!connected} send={send} interrupt={() => call('threads.interrupt', { threadId: thread.id })} />
    </div>
  );
}

function ItemView({ item, thread, send }: { item: Item; thread: ThreadSummary; send: (t: string, a?: { action: string; answer: string }[]) => Promise<void> }) {
  switch (item.kind) {
    case 'user':
      return (
        <div className="user">
          <div className="bubble">{item.answers?.length ? item.answers.map((a) => a.answer).join(', ') : item.text}</div>
        </div>
      );
    case 'text':
      return (
        <div className={`reply${item.streaming ? ' streaming' : ''}`}>
          <Md text={item.text} />
        </div>
      );
    case 'reasoning':
      return <div className="reasoning">{item.text}</div>;
    case 'steps':
      return <Steps steps={item.steps} />;
    case 'approval':
      return <Approval item={item} threadId={thread.id} />;
    case 'question':
      return <QuestionCard item={item} threadId={thread.id} />;
    case 'note':
      return <div className={`note ${item.tone === 'error' ? 'error' : ''}`}>{item.text}</div>;
    case 'turnEnd':
      return <TurnEnd item={item} open={thread.asksSeq === item.seq} send={send} />;
  }
}

function Md({ text }: { text: string }) {
  return (
    <Markdown
      remarkPlugins={[remarkGfm]}
      components={{
        // Links open in the browser, never in the app's own window.
        a: ({ href, children }) => (
          <a
            href={href}
            onClick={(e) => {
              e.preventDefault();
              if (href && /^https?:/.test(href)) openUrl(href);
            }}
          >
            {children}
          </a>
        ),
      }}
    >
      {text}
    </Markdown>
  );
}

function Steps({ steps }: { steps: Step[] }) {
  const [open, setOpen] = useState(false);
  const running = steps.some((s) => s.ok === undefined);
  const failed = steps.filter((s) => s.ok === false).length;
  return (
    <div className="steps">
      <button className="steps-head" onClick={() => setOpen(!open)}>
        <span className="chev">{open ? '▾' : '▸'}</span>
        <span className="steps-title">{steps.length === 1 ? steps[0].title : `${steps.length} steps`}</span>
        {running && <span className="steps-tag">running</span>}
        {failed > 0 && <span className="steps-tag bad">{failed} failed</span>}
      </button>
      {open && (
        <ul>
          {steps.map((s) => (
            <li key={s.itemId} className={s.ok === false ? 'bad' : ''}>
              <span className="step-kind">{s.kind}</span> {s.title}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function Approval({ item, threadId }: { item: Extract<Item, { kind: 'approval' }>; threadId: string }) {
  const [sending, setSending] = useState(false);
  const respond = async (decision: string) => {
    setSending(true);
    try {
      await call('approvals.respond', { threadId, approvalId: item.approvalId, decision });
    } finally {
      setSending(false);
    }
  };
  const label = { allow: 'Allow', allowAlways: 'Allow for this thread', deny: 'Deny' } as const;
  return (
    <div className="card">
      <div className="card-title">Allow {item.title}?</div>
      {item.decision ? (
        <div className="card-done">{item.decision === 'deny' ? 'Denied' : item.decision === 'expired' ? 'Expired' : 'Allowed'}</div>
      ) : (
        <div className="card-buttons">
          {item.options.map((o) => (
            <button key={o} disabled={sending} className={o === 'allow' ? 'primary' : ''} onClick={() => respond(o)}>
              {label[o]}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function QuestionCard({ item, threadId }: { item: Extract<Item, { kind: 'question' }>; threadId: string }) {
  const [chosen, setChosen] = useState<Record<number, string[]>>({});
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string>();
  const pick = (i: number, label: string, multi: boolean) =>
    setChosen((c) => {
      const had = c[i] ?? [];
      const next = multi ? (had.includes(label) ? had.filter((l) => l !== label) : [...had, label]) : [label];
      return { ...c, [i]: next };
    });
  const { answers, message } = questionAnswers(item.questions, chosen);
  const submit = async () => {
    setSending(true);
    try {
      await call('approvals.respond', { threadId, approvalId: item.approvalId, decision: 'answered', answers, message });
    } catch (e) {
      setError(errorText(e));
    } finally {
      setSending(false);
    }
  };
  return (
    <div className={`card${item.decision ? '' : ' you'}`}>
      {item.questions.map((q, i) => (
        <div key={i} className="ask">
          <div className="card-title">{q.question}</div>
          {!item.decision && (
            <div className="card-buttons">
              {q.options.map((o) => (
                <button key={o.label} className={chosen[i]?.includes(o.label) ? 'primary' : ''} title={o.description} onClick={() => pick(i, o.label, q.multiSelect)}>
                  {o.label}
                </button>
              ))}
            </div>
          )}
        </div>
      ))}
      {item.decision ? (
        <div className="card-done">{item.message || (item.decision === 'expired' ? 'Expired' : 'Answered')}</div>
      ) : (
        <div className="card-buttons">
          <button className="primary" disabled={sending || !message} onClick={submit}>
            Answer
          </button>
          {error && <span className="bad">{error}</span>}
        </div>
      )}
    </div>
  );
}

function TurnEnd({ item, open, send }: { item: Extract<Item, { kind: 'turnEnd' }>; open: boolean; send: (t: string, a?: { action: string; answer: string }[]) => Promise<void> }) {
  const asks = item.asks ?? [];
  return (
    <div className="turn-end">
      {asks.length > 0 && (
        <div className={`card${open ? ' you' : ''}`}>
          {asks.map((ask, i) => (
            <div key={i} className="ask">
              <div className="card-title">{ask.action}</div>
              {ask.why && (
                <div className="ask-why">
                  <Md text={ask.why} />
                </div>
              )}
              {open && ask.options && (
                <div className="card-buttons">
                  {ask.options.map((o, j) => {
                    const m = answerMessage(ask, o);
                    return (
                      <button key={o} className={j === ask.suggested ? 'primary' : ''} title={ask.optionNotes?.[j] || undefined} onClick={() => send(m.text, m.answers)}>
                        {o}
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
      <div className="turn-foot">
        {item.outcome === 'completed' ? 'Done' : item.outcome === 'interrupted' ? 'Stopped' : 'Failed'} in {duration(item.durationMs)}
        {item.error && <span className="bad"> · {item.error}</span>}
      </div>
    </div>
  );
}

function Composer({ busy, disabled, send, interrupt }: { busy: boolean; disabled: boolean; send: (t: string) => Promise<void>; interrupt: () => void }) {
  const [text, setText] = useState('');
  const [error, setError] = useState<string>();
  const submit = async () => {
    const t = text.trim();
    if (!t) return;
    try {
      await send(t);
      setText('');
      setError(undefined);
    } catch (e) {
      setError(errorText(e));
    }
  };
  return (
    <div className="composer">
      {error && <div className="note error">{error}</div>}
      <textarea
        value={text}
        placeholder={disabled ? 'Not connected' : 'Message'}
        disabled={disabled}
        rows={3}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            submit();
          }
        }}
      />
      <div className="composer-buttons">
        {busy && (
          <button onClick={interrupt} title="Stop the turn">
            Stop
          </button>
        )}
        <button className="primary" disabled={disabled || !text.trim()} onClick={submit} title="Send (Enter)">
          Send
        </button>
      </div>
    </div>
  );
}
