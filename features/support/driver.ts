// What a spec can do to an app and read back. Both apps take the same JSON
// commands (app/README.md, "Seeing and driving it from a script") and give the
// same `dump`, so the steps never ask which app they're driving. Only the
// handful of things one app can't name the same way are methods here.
export type Row = {
  id: string;
  kind: string;
  text: string;
  expanded?: boolean;
  seq?: number;
  turn?: string;
  inGroup?: boolean;
  /** Its height once it's been drawn (GPUI only measures rows it has laid out). */
  height?: number;
  onScreen?: boolean;
  /** What the row's buttons say, on the apps that report them. */
  buttons?: string[];
  /** A reply's ```slack, ```email and ```draft boxes as drawn: the label, each
   * line, each formatted piece ("bold: Hi", "link: docs <https://x.dev>") and
   * what Copy says ("Copy" or "Copied"). */
  drafts?: { label: string; lines: string[]; styled: string[]; copy: string }[];
  /** A reply's closed ```bash blocks and their icons: "[put, copy] npm test", or "[copy] npm test". */
  shellBlocks?: string[];
  /** The chips a row draws among its words: "picture: Picture 1", "card:
   * Fix the button", "thread: Morning brief". */
  chips?: string[];
};

export type Thread = {
  id: string;
  title: string;
  status: string;
  needsYou: boolean;
  unread: boolean;
  /** The sidebar row as drawn (sidebar specs). */
  place?: string;
  bold?: boolean;
  mark?: string | null;
  trailing?: string;
  tile?: string;
  origin?: string | null;
  hovered?: boolean;
  hoverButtons?: string[] | null;
  background?: string | null;
  waiting?: string | null;
  subagents?: { id: string; depth: number; status: string; title: string; detail: string; selected: boolean }[];
  /** "On bruce" for another runner's thread ("On bruce, offline"). */
  runner?: string | null;
};

export type Dump = {
  connection: string;
  selection?: Thread | null;
  sidebar: {
    mode: string;
    sections: { name: string; collapsed?: boolean; count?: number | null; shown?: number | null; threads: Thread[] }[];
    snoozed: number;
    archived: number;
    /** The Inbox's sort: "needsYou", "newest" or "oldest". */
    sort?: string;
    /** Before the runner's first list: the grey loading rows are showing. */
    loading?: boolean;
  };
  /** The line under the bar when the thread can't go on where it is (LimitBanner). */
  limit?: { text: string; button?: string | null; carryOn?: string | null } | null;
  composer: { draft: string; text: string; images?: number; editing?: number | null; chip?: string; chipBadge?: string | null; mode?: string; card?: string[] | null; cardRuns?: string[] | null; cwd?: string; runner?: string | null; creating?: boolean };
  /** The new-thread page while it's showing. */
  newThread?: { heading: string; note: string; project?: string | null; rooms: { label: string; line: string; selected: boolean; note?: string | null }[] } | null;
  /** The slash palette over the composer while it's open. */
  palette?: {
    query: string;
    category: string;
    categories: { title: string; count: number }[];
    commands: string[];
    picked?: string | null;
    detail?: { title: string; usage: string; description?: string | null; now?: string | null; also?: string | null; origin: string; path?: string | null; buttons: string[] } | null;
    note?: string | null;
  } | null;
  /** The Move sheet while it's up. */
  moveSheet?: { title: string; subtitle: string; folder: string; login: string; problems: string[]; leftBehind: string[]; clone?: string | null; asking: boolean; canMove: boolean } | null;
  decisions?: string[] | null;
  transcript: {
    threadId?: string;
    state?: string;
    atBottom?: boolean;
    hasMoreAbove?: boolean;
    rows: Row[];
  };
  banner?: { style: string; text: string; detail?: string } | null;
  /** The sub-agent whose transcript is open in place of its thread's, by id. */
  subagent?: string | null;
  /** The error bar's text, when it's up. */
  error?: string | null;
  /** The window's panels, in one line ("row[threads (no title) 29% | thread 47% | ...]"). */
  dock?: string;
  /** The Threads panel is on screen: in the layout and not folded away with ⌘/. */
  threadsShowing?: boolean;
  /** The window's zoom, in percent. */
  zoom?: number;
  /** The thread the Archived note names, while it's up. */
  archivedNote?: string | null;
  /** "end" or "empty" while the end-of-inbox note is up. */
  inboxEndNote?: string | null;
  /** The panel bar's title line and its buttons' tips. */
  header?: { title?: string; renaming?: boolean; backTip?: string; forwardTip?: string; shared?: boolean };
  /** The activity bar and the right rail, as drawn. */
  rails?: { activity: RailButton[]; right: RailButton[] };
  /** What the activity bar's Runners, Updates and Connectors panels say, a line each. */
  runners?: PanelLines;
  updates?: PanelLines;
  connectors?: PanelLines;
  /** The Terminal panel: whether it's up, the open thread's shell and its screen, and every shell ("thread:<id> running in <folder>"). */
  terminal?: {
    shown: boolean;
    cwd: string;
    note?: string | null;
    shell: 'running' | 'exited' | 'none';
    waitingForReturn: boolean;
    /** The program has asked for pastes to be marked (bracketed paste): a shell at its prompt. */
    pastesMarked: boolean;
    focused: boolean;
    cols?: number | null;
    rows?: number | null;
    screen?: string[] | null;
    shells: string[];
  } | null;
};

/** `dot` is the accent dot (something only you can do), `quietDot` the grey one. */
export type RailButton = { title: string; badge?: number | null; dot?: boolean; quietDot?: boolean; active: boolean };

/** One of the activity bar's panels: whether it's on show, and what it says. */
export type PanelLines = { shown: boolean; lines: string[] };

export interface Driver {
  readonly name: 'swift' | 'gpui';
  start(): Promise<void>;
  /** Runs one command and returns its result line. Throws if it failed. */
  do(cmd: Record<string, unknown>, timeoutSeconds?: number): Promise<string>;
  dump(): Promise<Dump>;
  /** Saves a picture of the window, for a failed scenario. */
  shot(file: string): Promise<void>;
  /** Quits the app and opens it again, keeping its data and its settings. */
  relaunch(): Promise<void>;
  stop(): Promise<void>;
}

export const sleep = (ms: number) => new Promise<void>((done) => setTimeout(done, ms));

/** What a command answered, past the "ok" (the Swift app's lines start
 * "<cmd> ok ...", the GPUI app's just "ok ..."). */
export const okValue = (out: string) => out.replace(/^(\S+ )?ok ?/, '').trim();
