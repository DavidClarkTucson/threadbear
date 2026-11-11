# GPUI port: parity

What the GPUI app does compared with the Swift app, worked out from which
behaviour specs pass (`features/`). The table is written by
`node features/parity.mjs` from each machine's report. Don't edit it by hand.

A blank cell means no spec covers that yet, not that the feature is missing.
The full list of features is [features.md](features.md).

## The table

<!-- parity:start -->

122 of 329 features have a spec so far, across 220 scenarios.

Scenarios passing: Swift: 207 of 220, Mac: 208 of 220, Windows: 192 of 220, Linux: 208 of 220.


### 0. Foundations everything else needs

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `foundation.rpc` Runner RPC client | pass | pass | pass | pass |
| `foundation.zoom` Zoom | partly | pass | pass | pass |

### 1. Windows and panel dock

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `dock.layout` Layout tree | pass | pass | pass | pass |
| `dock.titlebar` No title bar | pending | pending | pending | pending |
| `dock.frame` Window frame memory | pending | pending | pending | pending |
| `dock.reset` Default layout and Reset | pass | pass | pass | pass |

### 2. Rails

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `rail.activity` Activity bar | pass | pass | pass | pass |
| `rail.dots` Rail dots | pass | pass | pass | pass |
| `rail.right` Right rail | pass | pass | pass | pass |

### 3. Sidebar and inbox

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `sidebar.sections` Sections | pass | pass | pass | pass |
| `sidebar.sort` Inbox sort | pass | pass | pass | pass |
| `sidebar.search` Thread search | pass | pass | pass | pass |
| `sidebar.newthread` New thread button | pass | pass | pass | pass |
| `sidebar.row` Thread row | pass | pass | pass | pass |
| `sidebar.row.marks` Row left marks | pass | pass | pass | pass |
| `sidebar.row.origin` Origin mark | pass | pass | pass | pass |
| `sidebar.row.status` Row right status | pass | pass | pass | pass |
| `sidebar.row.remote` Other-Mac icon | pass | pass | pass | pass |
| `sidebar.row.hoverbuttons` Hover buttons | pass | pass | pass | pass |
| `sidebar.row.menu` Row context menu | pass | pass | pass | pass |
| `sidebar.subagents` Sub-agent tree | pass | pass | pass | pass |
| `sidebar.archived.paging` Archived paging | pass | pass | pass | pass |
| `sidebar.skeleton` Loading skeleton | pass | pass | pass | pass |
| `sidebar.hibernate.presets` Hibernate presets | pass | pass | pass | pass |
| `sidebar.hibernate.pick` Pick a time | pending | pending | pending | pending |
| `sidebar.hibernate.wake` Wake when… | pass | pass | pass | pass |
| `sidebar.move` Move to another runner | pass | pass | pass | pass |

### 4. Thread header, banners, toasts

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `header.title` Title line | pass | pass | pass | pass |
| `header.backforward` Back and forward | pass | pass | pass | pass |
| `header.share` Share | pass | pass | pass | pass |
| `header.more` ... menu | pass | pass | pass | pass |
| `banner.connection` Connection banner | pass | pass | pass | pass |
| `banner.error` Error banner | pass | pass | pass | pass |
| `banner.limit` Limit banner | pass | pass | pending | pass |
| `toast.archived` Archived toast | pass | pass | pass | pass |
| `toast.inboxend` Inbox end toast | pass | pass | pass | pass |
| `thread.newpage` New-thread page | pass | pass | pass | pass |

### 5. Transcript row kinds

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `transcript.list` The transcript list | pass | pass | pass | pass |
| `transcript.user` Your bubble | pass | pass | pass | pass |
| `transcript.reply` Assistant reply | pass | pass | pass | pass |
| `transcript.thinking` Thinking line | pass | pass | pass | pass |
| `transcript.tool` Tool line | pass | pass | pass | pass |
| `transcript.tooloutput` Tool output styling | pass | pass | pass | pass |
| `transcript.diff` Diff row | pass | pass | pass | pass |
| `transcript.images` Images row | partly | pass | pass | pass |
| `transcript.approval` Approval card | pass | pass | pass | pass |
| `transcript.approval.stacked` Stacked approval | pass | pass | pass | pass |
| `transcript.approval.settled` Settled approval | pass | pass | pass | pass |
| `transcript.question` Question card | pass | pass | pass | pass |
| `transcript.secret` Secret card | pass | pass | pass | pass |
| `transcript.review` Review card | pass | pass | pass | pass |
| `transcript.turnfooter` Turn footer | pass | pass | pass | pass |
| `transcript.error` Error row | pass | pass | pass | pass |
| `transcript.workedfor` Worked-for group | pass | pass | pass | pass |
| `transcript.working` Working line | pass | pass | pass | pass |
| `transcript.changes` Changes card | pass | pass | pass | pass |
| `transcript.rowmenu` Row context menu | pass | pass | pass | pass |

### 7. Composer

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `composer.textbox` Text box | pass | pass | pass | pass |
| `composer.return` Return, Shift-Return | pass | pass | pass | pass |
| `composer.listcontinue` List continuation | pass | pass | pass | pass |
| `composer.picturechips` Picture chips | pass | pass | pass | pass |
| `composer.chiphover` Chip hover and × | pass | pass | pass | pass |
| `composer.filedrop` File path drop | pass | pass | pass | pass |
| `composer.dropanywhere` Drop anywhere on thread | pass | pass | pass | pass |
| `composer.runner` Runner chip | pass | pass | pass | pass |
| `composer.modelpicker` Model picker | pass | pass | pass | pass |
| `composer.permission` Permission chip | pass | pass | pass | pass |
| `composer.contextring` Context ring | pass | partly | partly | partly |
| `composer.send` Send and Stop | pass | pass | pass | pass |
| `composer.starting` Starting state | pass | pass | pass | pass |
| `composer.slash` Slash palette | pass | pass | pass | pass |
| `composer.slash.keys` Palette keys | pass | pass | pass | pass |

### 8. Cards in the composer

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `card.done` Done card | pass | pass | pass | pass |
| `card.needsyou` Needs you card | pass | pass | pass | pass |
| `card.collapse` Collapse | pass | pass | pass | pass |
| `card.somethingelse` Something else box | pass | pass | pass | pass |
| `card.returnflow` Return flow | pass | pass | pass | pass |
| `card.answers` Answer and follow-up state | pass | pass | pass | pass |

### 9. Highlights

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `highlights.panel` Panel and folds | pass | pass | pass | pass |
| `highlights.librarian` Librarian line | pass | pass | pass | pass |
| `highlights.needsyou` Needs you list | pass | pass | pass | pass |
| `highlights.projects` Project blocks | pass | pass | pass | pass |
| `highlights.scheduled` Scheduled list | pass | pass | pass | pass |
| `highlights.subagents` Sub-agents list | pass | pass | pass | pass |
| `highlights.started` Threads it started | pass | pass | pass | pass |
| `highlights.pinned` Pinned, Files, Links | pass | pass | pass | pass |
| `highlights.changes` Changes | pass | pass | pass | pass |
| `highlights.rowmenus` Row menus | pass | pass | pass | pass |

### 10. Patches panels and web panels

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `patches.list` Patches list | pass | pass | pass | pass |
| `patches.run` Patch chips run | pass | pass | pass | pass |
| `panel.list` List panel | pass | pass | pass | pass |
| `panel.image` Image panel | pass | pass | pass | pass |

### 12. File previews and folders

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `file.host` Preview host | pass | pass | pass | pass |
| `file.picture` Picture view | pass | pass | pass | pass |
| `file.textview` Text viewer | pass | pass | pass | pass |
| `file.markdown` Markdown preview | pass | pass | pass | pass |
| `folder.list` Folder list | pass | pass | pass | pass |
| `folder.pathbar` Path bar | pass | pass | pass | pass |
| `folder.menu` Folder menu | pass | pass | pass | pass |
| `folder.filespanel` Files panel | pass | pass | pass | pass |

### 13. Terminal

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `terminal.panel` Terminal panel | pending | pending | pending | pending |
| `cards.run` Run on the card | pending | pending | pending | pending |
| `terminal.putcommand` Put a command in the Terminal | pending | pending | pending | pending |

### 14. Projects, Librarian, Schedules, Usage, Runners, Connectors, Updates

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `projects.decisions` Decisions on the Projects page | pass | pending | pending | pending |
| `librarian.page` Librarian page | pass | pass | pending | pass |
| `librarian.log` Librarian log | pass | pass | pending | pass |
| `schedule.page` Scheduled page | pass | pass | pending | pass |
| `schedule.list` Scheduled list | pass | pass | pending | pass |
| `schedule.editor` Schedule editor | pass | pass | pending | pass |

### 15. Settings

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `settings.window` Settings window | pass | pass | pending | pass |
| `settings.general` General | pending | pass | pending | pass |
| `settings.models` Models | pending | pass | pending | pass |

### 17. Navigation, shortcuts, main menu

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `nav.history` Back/forward history | partly | pass | partly | pass |
| `nav.sidebuttons` Mouse side buttons | pass | pass | pass | pass |
| `nav.nextinbox` Next Inbox Thread | pass | pass | pass | pass |
| `menu.app` App menu | pass | pass | pass | pass |
| `menu.file` File menu | pass | pass | pass | pass |
| `menu.view` View menu | pass | pass | pass | pass |
| `menu.go` Go menu | pass | pass | pass | pass |
| `menu.thread` Thread menu | pass | pass | pass | pass |
| `menu.help` Help menu | pass | pass | pass | pass |
| `nav.shortcuts` Keyboard shortcuts | partly | pass | pass | pass |

<!-- parity:end -->

## The log

One entry a slice, newest first. Every entry has a `Verification:` line: a spec
run or a screenshot from the running app. A slice with no runtime check doesn't
count. (Ghostex's port log is where this shape comes from; its slices 214 to 268
built parsers and gates with no launch of the app, which is the trap.)

### 2026-11-11 Chips in Mike's messages (`rich.picturechips`, `rich.cardchips`, `transcript.user`)

- In Mike's bubble a "[Picture N]" that places one of the message's
  pictures is a chip, the photo symbol and "Picture N", as the Swift bubble
  draws it. Resting the pointer on it shows that picture in a card under
  it, fitted to a 276 by 170 box (this Mac's threads only: another Mac's
  path names a file there); a click opens it in the in-app viewer, as the
  pictures over the words do.
- A Notion card's link in his words is a chip with the card's title (cut
  short past 230 points) that opens the page through the open gate, with a
  card after half a second saying its title and address, as LinkPeekCard
  does. A `thread:` link is a chip with the thread's title and the speech
  bubble, which opens the thread, with the thread's own card: its title,
  where it is and what it's doing, and "Click to open" (ThreadChipCard).
- A bubble with chips is laid out a line at a time round them, words
  wrapping where they would, so it's as wide as its widest line, as the
  Swift bubble is; its words stay selectable a run at a time. One with no
  chips is the one selectable text it was.
- Finding them is `desktop/core/src/chips.rs`: InlinePictures' markers and
  CardMentions' forward scan, with their tests, and ThreadRefs (ids, the
  index, the card's line) ready for replies.
- Not here: a selection running across a chip from the words either side
  (each run selects on its own), dragging a picture chip out and a chip's
  right-click menu (`drag.fileout`, `rich.filemenu`), and chips in a
  message that answered asks, which keeps its question and answer layout.
- Both apps' dumps give a message's `chips` ("picture: Picture 1", "card:
  Fix the share button", "thread: Tidy the imports"); the GPUI app takes
  the Swift app's `peek` (list a row's chips, show one's card, press a
  picture's chip through a real click and say what opened).
- Files: `desktop/core/src/{chips,lib}.rs`,
  `desktop/gpui/src/{chips,rows,thread,automation,icons,main}.rs`,
  `app/Sources/Threadbear/AppDump.swift`, `features/message-chips.feature`
  and its steps, `features/support/driver.ts`.
- Verification: on Tinker Desk, `@rich.picturechips` and `@rich.cardchips`,
  GPUI 2 of 2 and Swift 2 of 2, with `@transcript.user`, `@transcript.edit`
  and `@composer.picturechips` on the GPUI app; core's tests. Pictures of
  both apps' bubble with a picture, a card and a thread chip, dark and
  light, laid over each other: the lines break at the same words, the chips
  are the Swift ones' size and place (the symbol was small, and the edge
  twice as strong on a 1x screen, both brought in); and the GPUI app's
  three cards under their chips.

### 2026-11-11 A project page's own panels (`panel.projectpage`)

- On a project's page the right rail and the pane beside it are the
  project's: the panels and patches its folders give, borrowed from its
  newest open thread that has any (else its newest archived one) and kept
  to what came from the project, as the Swift app's `paneThread` does. A
  project's page has no Highlights or Files, as they're a thread's; Patches
  lists the project's in one list and says they come from its folders and
  that every thread in it gets them. A page whose threads have nothing from
  it has no rail.
- Worked out once until the app next changes: the dock and the rail ask
  several times a frame, and finding the thread reads each of the
  project's threads' patches.
- Files: `desktop/gpui/src/projects.rs` (`pane_thread`); the lines in
  `shell.rs`, `rails.rs`, `highlights.rs`, `patches.rs` and `app.rs` that
  read it; a scenario in `features/projects-views.feature`.
- Then two small things the Swift app has: the Projects search's tooltip
  ("Search projects"), and a note, "Making <name>…", while a project made
  from a thread's menu is on its way, as there's no button there to spin.
- Verification: projects-views.feature's 10 scenarios on Tinker Desk against
  both apps (GPUI 10 of 10, Swift 9 of 9, the Highlights one pending there),
  with projects, rails, highlights and patches on the GPUI app (45 of 45);
  the Walkies page with its Patches pane side by side with the Swift app's.
  After the two small things, projects-views and projects on the GPUI app
  (17 of 17).
### 2026-11-11 Decisions on a project's page, and the folded Projects view (`projects.decisions`)

- With the projectDecisions experiment on, a project's page opens on its
  threads' (and its parts' threads') open cards of asks, newest first, under
  Decisions ("4 decisions from 3 threads"). Each card is its thread's: its
  title and how long ago, a line on what it's doing, its asks drawn by the
  composer card's own code (`AskHost` in composer_card.rs, which the
  composer's card now goes through too), and Open thread. A pick that
  answers the last open ask sends the thread's own `threads.send` with the
  answers; Something else opens its box, where Return sends once every ask
  has an answer and ⌘Return sends what's answered. Until the thread takes it
  the card is one line, "Sending to <title>…" with a spinner and what was
  answered, and takes no clicks; refused, the picks come back with the
  runner's reason.
- A project's card on the Projects page says how many decisions wait in it,
  and the sidebar's Projects view folds each project to one row (its icon,
  its name, its decisions in the needs-you orange or its threads' count, a
  chevron that shows its threads), folded by default and remembered
  (`projectsList.openRows`), Not in a project too.
- A thread summary's asks now keep their option notes, reply and links
  (core's `OpenAsk`), which the cards draw from.
- The answer rows the decisions share with the composer's card now take the
  Swift rows' spacing, measured against both of the Swift app's: a box with
  a small corner, 14 over the first row's words and 6 over the rest, a
  note in the small font just under its label, and 8 under the box. They
  were 7 points a row taller.
- Not here yet: the cards sliding in and out, and holding a pick while the
  thread's own box has a message on its way (the Swift app waits for that
  too; here only an answer sent from the page holds it).
- Files: `desktop/gpui/src/project_decisions.rs` (new); `composer_card.rs`
  (`AskHost`, the rows' spacing), `project_page.rs`, `projects.rs`,
  `projects_list.rs`, `app.rs`, `automation.rs` and `main.rs`; core's
  `projects.rs` and `inbox.rs`; `features/projects.feature`.
- Verification: projects.feature's 7 scenarios on Tinker Desk against the
  GPUI app, the 5 decisions ones no longer pending, with projects-views,
  composer-cards and cards (34 of 34); core's tests. The decisions page and
  a thread's composer card side by side with the Swift app's in dark and
  measured: the rows, the card and the folded rows within a point or two.
### 2026-11-11 Drafts in a reply, and their Copy (`rich.draft`, `rich.draft.copy`)

- A ```slack, ```email or ```draft fence in a reply is drawn as the draft it
  is: a rounded box like a code block's with its label ("Slack draft",
  "Email draft", "Draft") and Copy in a header, a line across the box under
  it, then every line where the agent put it, blank ones too. A Slack draft
  shows Slack's own formatting: *bold*, _italic_, ~strike~, `code` (13 pt
  monospaced in its pill, as the reply's), <url|label> links, mentions in
  bold, and "> " lines quiet behind a bar. An email or plain draft's bare
  web addresses are links. Links go through the open gate as the reply's do.
  A fence still streaming in stays code until its closing line comes, and
  one in a list or a quote is a box too.
- Copy puts the plain text (Slack's markers kept, links as "label (url)")
  and HTML beside it on the clipboard, so a paste into Slack or Mail keeps
  the bold and the links; it turns green with a tick, "Copied", for a moment
  and the "Copied to clipboard" bubble rises over it and fades. Its tooltip
  is the Swift one's.
- The parsing is `desktop/core/src/draft_block.rs`, a port of DraftBlock with
  its tests and the website's fence check. The box is the kit's block plugin
  (`draft_boxes.rs`), so the kit's own parse finds the fence and a streaming
  reply only reparses its tail, rather than the reply being split around it.
- Not here: selecting part of a draft's words (a selection across the reply
  takes the box whole, as its plain text; Copy and Copy message cover the
  rest), HTML on the clipboard on Windows and Linux (plain text there for
  now), and a link's hover peek in a draft, which comes with
  `rich.linkpeek`.
- Both apps' dumps give each reply's `drafts` (label, lines as drawn, the
  formatted pieces, what Copy says); both take `draft` (press a box's Copy
  through a real click) and `clipboard` with `"type": "html"`. The Swift
  app's Copy goes to the instance's own pasteboard when scripted. The mock
  runner's `mock.stream` takes `text`, to stream a reply of a spec's own.
- Files: `desktop/core/src/{draft_block,lib}.rs`,
  `desktop/gpui/src/{draft_boxes,rows,thread,automation,main}.rs`,
  `desktop/gpui/Cargo.toml` (NSPasteboard),
  `app/Sources/Threadbear/{AppDump,Automation,Automation+Commands}.swift`,
  `app/Sources/Threadbear/Transcript/TextKit.swift`, `app/README.md`,
  `app/Tools/mock-runner/server.mjs`, `features/drafts.feature` and its
  steps, `features/support/driver.ts`.
- Verification: on Tinker Desk, `@rich.draft` and `@rich.draft.copy`, GPUI 4
  of 4 and Swift 4 of 4; core's tests (with the integration tests) and the
  plugin's own tests against the kit's parser. Pictures of both apps side by
  side, dark and light, a Slack draft with every kind of formatting and an
  email draft at the end of a reply, laid over each other with
  pixel-overlay: the box, its lines, the header and Copy sit where Swift's
  do (Copy was 12 points left and the code at body size, both brought in);
  the Copied state and its bubble just after a press, matched to the Swift
  bubble's height and place. The line under the header is half a point on a
  Retina screen and a faint whole pixel on a 1x one, where GPUI drops half
  a pixel.

### 2026-11-11 Projects: the sidebar's view, the page of cards and a project's page (`projects.list`, `projects.home`, `projects.page`, `projects.parts`, `projects.newthread`, `header.projectchip`, `highlights.projects`)

- Projects in the activity bar opens the sidebar's Projects view and the
  Projects page beside it, as in the Swift app. The view: a heading per
  project (its icon, its open threads' count, a chevron under the pointer,
  lit while its page is open), its open threads under it as the inbox draws
  them (leaving out the project the heading names), its archived ones folded
  away five at a time, its parts nested under it; then Not in a project, and
  Archived projects folded away at the foot. New project at the top spins
  until the runner has made it.
- The page: a card per project in the Swift grid (as many columns as fit
  at 240 to 340 wide), its icon on a wide tile (a picture icon blurred
  behind it, made off the main thread, as GPUI has no blur), its open
  threads' count in the corner, what its threads are doing in the sidebar's
  colours with the working spinner (or when it was last active), its parts,
  and its state's first lines. The search looks through titles, state,
  goals, notes, folders' names, links and parts, accents aside; Return
  opens the top card, Escape clears it. A card's right-click is the
  project's menu.
- A project's page: its icon (a menu to choose a picture, go back to the
  librarian's, or show the original), the title edited in place (Return
  saves, Escape puts it back), who made it and when, "Part of" for a part,
  New Thread; then Parts, Goals, State, the Notion task board, Notes,
  Folders, Threads and Patches. Goals, state and notes edit in a box with
  Save (⌘Return) and Cancel (Escape), and show with their bold, code and
  links, every line kept as it was. Every change shows at once and goes
  back with the runner's reason if it says no; the icon, the board's
  refresh and New project spin until they land. The bar says Projects /
  the project, and its ... is the project's menu: Archive, the Notion page,
  Make a Part of or Make Its Own Project, Merge into, Copy project id and
  Delete, asking first where the Swift app does.
- The ways in: a thread's project tiles in its bar open the page (a dashed
  ring for the librarian's guess, its menu to keep it there, take it out or
  add it to another, shown at once); Projects ▸ in a row's menu and the
  bar's ... (a tick by each project it's in, New project…, Ask the
  librarian to look); Open project in Highlights' ⓘ; New Thread on the page
  starts the new-thread box in the project and its first folder. Back and
  Forward remember a project's page, and skip one that's gone.
- Not here yet: a project page's own panels on the rail
  (`panel.projectpage`: the page hides the thread's pane for now), decisions
  (`projects.decisions`), and the card's grow on hover.
- The rules are `desktop/core/src/projects.rs`, with the Swift tests ported
  (sections and parts, cards, search, decisions, folds, the page's words,
  what a page borrows from its threads); both apps' dumps say what the views
  show (`project`, `projectCards`, the Projects view's sections).
- Files: `desktop/core/src/projects.rs`, `desktop/gpui/src/projects.rs`,
  `projects_list.rs`, `projects_page.rs` and `project_page.rs` (new); the
  lines that hook them into `app.rs`, `shell.rs`, `sidebar.rs`, `header.rs`,
  `highlights_project.rs`, `automation.rs`, `settings.rs`, `icons.rs` and
  `main.rs`; the `image` crate in `desktop/gpui/Cargo.toml` (already in the
  build through GPUI); `features/projects-views.feature` and its steps.
- Verification: projects-views.feature's 9 scenarios on Tinker Desk against
  the GPUI app (9 of 9) and the Swift app (8 of 8; the Highlights one is
  `@swift-pending`, as the ⓘ's popover is a window of its own that the
  Swift script's click doesn't look in); projects.feature's two parts
  scenarios on both apps, no longer pending; navigation, header,
  sidebar-actions, sidebar-rows, rails, new-thread, highlights and librarian
  features on the GPUI app (99 of 99 with the above); core's tests, the
  integration ones too. The Projects page and Threadbear's and Walkies'
  pages side by side with the Swift app's in dark and measured: the headers
  to the pixel, the text within a point.

### 2026-11-11 The Usage panel and the honey pot (`usage.spend`, `usage.pools`, `rail.honeypot`)

- Usage in the activity bar is the honey pot: filled to the room on the
  login the next new thread would use (the tighter of its 5-hour and weekly
  windows; empty at its limit or unknown), draining in a short step as it's
  spent and rising with a drip down the outside when that login's window
  resets, its tooltip each pool's best room. It opens the panel under the
  threads, as the placeholder did.
- The panel, as the Swift one: Today, Last hour and 7 days, each with its
  short line where it fits and the sentence in its tip; tokens by the hour
  today or by the day for the week, stacked by login in the logins' own
  colours, with the dashed line at now, the hovered bar's breakdown and the
  key (Today or 7 days is remembered as `usage.chartRange`). Then a card per
  pool (each provider named in a mixed pool, the logins in no pool last, a
  card per provider without pools): each login's dot, name, note and today's
  tokens, its 5-hour bar and weekly charts (the rolling week with last
  week's readings, broken at the reset, this pace dashed on to the reset, an
  orange dot where it runs out), a model's own week (Weekly Fable), what to
  do about each window, and the lines under it (old numbers, credits, Token
  Stats' notices, the shared part said once at the foot).
- The Settings card main has had since 368d99d8: Show Weekly Fable, on by
  default and remembered as `usage.showModelWeeks`, there only while a login
  has a model's own week.
- The numbers and words are `desktop/core/src/usage.rs` (UsageStatsMath,
  LimitPace, RollingWeek, UsageGroups, HoneyLevel, the parts of
  AccountUsageText the panel reads, LoginColours), with the Swift tests
  ported; the panel and the dump both read them.
- The panel is a view of its own drawn `cached`, as the Threads panel is.
  Drawn with the shell, every spinner frame laid it out and painted its
  charts again, and open wide a debug build spent so long on frames that it
  stopped answering. Charts scrolled out of sight aren't painted.
- Both apps' dumps say what the panel draws (`usage`: the pot's level, login
  and tooltip, the tiles, chart, cards, notes, `settings` and footer), and
  both apps' `usage` command takes `range` and `showModelWeeks`.
- Not the same yet: the switch is this app's small one (28 by 16, as on the
  Librarian page and in Settings) where macOS 27's is 44 by 20 with a pill
  knob; the row is kept 20 tall so the card lines up. Where scroll bars
  always show, the Swift panel keeps room for one and GPUI's scroll views
  don't, so a tile's line can fit in one app and not the other. The CLIs at
  the end of the Swift `usage` text are the Updates panel's here.
- Files: `desktop/core/src/usage.rs` and `desktop/gpui/src/usage.rs` (new),
  a few lines in `app.rs`, `rails.rs`, `shell.rs`, `settings.rs`,
  `automation.rs` and `main.rs`, core's `logins.rs` (a login's plan) and
  `new_thread.rs` (`day_name` shared); `features/usage.feature` and its
  steps; the Swift dump (`AppDump.swift`) and `usage` command
  (`Automation+Commands.swift`), `app/README.md`.
- Verification: usage.feature's 4 scenarios on Tinker Desk against both apps
  (4 of 4 each) and rails.feature on the GPUI app; core's usage tests; the
  panel side by side in both apps, docked and wide, by the hour and the day,
  with the Settings card on and off, in dark and light, overlaid and
  measured (the pot, the card edges, the headers, the switch row), and both
  dumps compared scene by scene (the same, but for one "runs out" time a
  minute on as the clock moved between runs). The drip, in a build with the
  pot slowed tenfold, as the debug build's pictures come too slowly to catch
  it.
### 2026-11-11 Editing a message (`transcript.edit`, `composer.editing`)

- Hovering one of Mike's messages that can be edited (his own, from this
  app, in a thread whose agent can go back: Claude's and Codex's) shows a
  pencil left of the bubble, level with its last line. It puts the message
  in the box, pictures as chips where their markers were (core's
  `edit_draft`, from InlinePictures.draft), with the banner "Editing a
  message. Sending cuts the thread back to it and undoes the agent's file
  changes since." and Cancel (or Esc), which puts back what was in the box.
  The open card waits while an edit is under way, and the box says "Your
  message…".
- Sending it calls `threads.edit`; the banner says "Cutting the thread back to
  this message…" with a spinner until the new turn has started, and an error
  leaves it there to try again. When the thread has background work running
  (-32050), an alert asks first: "Stop the background work?", Stop It and Go
  Back or Cancel (`alert.rs`, an NSAlert as the Swift app's).
- Both apps' dumps give the seq being edited (`composer.editing`), and both
  apps take `edit-message` with a row, as a click on its pencil.
- Files: `desktop/core/src/composer.rs`, `desktop/gpui/src/{composer,rows,
  thread,app,automation,alert,main}.rs`,
  `app/Sources/Threadbear/Automation.swift`, `features/editing.feature` and
  its steps.
- Verification: on Tinker Desk, `@transcript.edit` passes on both apps (2 of
  2: the message in the box and Esc leaving it; an edit sent replaces the
  message and the thread carries on from it), with `@composer.send` and
  `@transcript.user` on GPUI; core's test for the draft an edit puts back.
  Pictures of the box mid-edit in both apps, dark: the banner's words, Cancel
  and the box under it match.

### 2026-11-11 A sub-agent's own transcript (`header.subagent`, `sidebar.subagents`, `highlights.subagents`)

- A click on a sub-agent, under its thread in the sidebar or in Highlights,
  opens its own transcript in place of the thread's (`threads.subagent`),
  under a bar with Back to the thread, its title, and what it is ("Explore ·
  Done · 4 tool calls · from Why is the build slow on CI?"). It has no
  composer, as it takes its brief from the agent that started it. While it
  runs it's read again every 3 s and as the thread's summary says it did
  something, with the working line until it finishes. The open one is marked
  in the sidebar. "Couldn't load this sub-agent." with Try again when the read
  fails, "There's no record of what this sub-agent did." when the runner has
  none (-32006), and "Nothing yet." for one that hasn't done anything.
- Opening a thread, or the new-thread page, puts the thread back.
- A setting changed in the moment before a quit is saved as the app quits
  (the Usage helper found it waited on the throttled save and was lost).
- Both apps' dumps say which sub-agent is open (`subagent`), and the GPUI app
  takes the Swift app's `subagent` command. The mock can drop a sub-agent's
  record (`mock.forgetSubagent`).
- Files: `desktop/gpui/src/{thread,app,header,shell,sidebar,highlights,
  automation}.rs`, `app/Tools/mock-runner/server.mjs`,
  `features/subagents.feature` and its steps.
- Verification: on Tinker Desk, `@header.subagent` passes on both apps (2 of
  2), and `@sidebar.subagents`, `@highlights.subagents` and `@foundation.rpc`
  on GPUI. Pictures of a sub-agent open in both apps, dark: the bar's title,
  meta line and Back, and the transcript under it, sit where Swift's do.

### 2026-11-11 An agent's show_panel, and pictures in replies (`panel.requests`, `rich.localpicture`)

- A panel an agent asks to show (`show_panel`, and `pin` with `show`) comes up
  once Mike is looking at its thread: straight away if he is, else when he
  next opens it or comes back to the app. Highlights, Patches and the
  thread's own panels become its chosen section and open beside the other
  pane sections (or as a tab, on a side or in a window of their own, as asked);
  Usage, Updates and Connectors open under the threads list; one already open
  comes to the front. A pin opens in the pane (fetched first if it isn't in
  Highlights yet), never in its own app, and one unpinned since still opens
  Highlights. The app then says so (`threads.panelShown`), which clears it;
  each request is shown once (core's `panel_requests.rs`, from PanelRequests
  and PanelRequestGate). One the thread hasn't got is left for the runner to
  clear with Mike's next message.
- `![alt](path)` in a reply draws the picture when it's a file on this Mac
  that's there; a web address, another Mac's file or one that's gone stays a
  link with its alt text, "[alt]", as the Swift app's does. A web picture is
  never fetched.
- Not here: `reload` (web panels aren't in this app yet), the Browser pane,
  and the Swift app's 420 by 320 cap on a reply's picture (the kit draws it
  the column's width).
- Files: `desktop/core/src/{panel_requests,inbox,lib}.rs`,
  `desktop/gpui/src/{app,rows}.rs`, `desktop/gpui/src/rows/markdown.rs`,
  `features/panel-requests.feature` and its steps.
- Verification: on Tinker Desk, `@panel.requests` passes on both apps (2 of
  2: the Updates panel comes up and the mock hears it was shown; a panel the
  thread hasn't got stays waiting), with the reply scenarios on GPUI; core's
  tests (the gate's and the request's) and the markdown tests (a web picture,
  a missing file, code left as written, a file on this Mac, another Mac's
  thread).

### 2026-11-11 Queued messages, kept output, footer tips and row menus (`transcript.pending`, `transcript.heldback`, `transcript.turnfooter`, `transcript.rowmenu`, `transcript.lightup`, `menu.thread`)

- A message the runner takes but holds behind a running turn says so under
  its bubble ("(queued)" in the dump, the clock in the app) 0.6 s after the
  runner answers, until its own turn shows it; one still queued after the
  thread has sat idle for 10 s is taken for lost and goes, as the Swift app's
  `checkQueued` has it.
- Opening another runner's tool whose output was kept on that runner asks
  for it (`threads.heldBody`), which fills the row in when that runner has
  full sync on.
- The turn footer's words have the Swift tip: the tokens out and in, the
  cache reads, writes and uncached ones broken out, and the cost at API
  prices.
- A row's menu says Collapse on an open tool or thinking row, not only a
  group, and Copy debug info says what Swift's does: the events behind the row
  (`source_seqs`, as ranges), the runner's version, the turn's tokens and
  error kind on a footer, and a `tb events … --from … --to …` line ready to
  run (with the right `tb` for a dev runner, or the fixture's file on the
  mock).
- A work link's step stays lit for 2.5 s and then fades over 0.8 s, behind the
  row's own drawing, rather than going at once. It still scrolls the step to
  the top of the view rather than a third of the way down.
- The Thread menu's Hibernate has "Until the runner restarts" while a restart
  is waiting.
- Not here: a message held in the app while a restarting runner is away (the
  Swift app's `waitingForRunner`), and View > Collapse Threads (⌘/), which
  needs the dock to hide a panel while keeping its place.
- Files: `desktop/core/src/reducer/{row_debug,format,event,mod}.rs`,
  `desktop/gpui/src/{thread,rows,shell,sidebar}.rs`,
  `desktop/gpui/src/rows/export.rs`, `features/sending.feature` and its steps.
- Verification: on Tinker Desk, `@transcript.pending` passes on both apps
  (the mock's `slow` with `queueMs` holds the turn), and
  `@transcript.rowmenu`, `@transcript.turnfooter`, `@menu.thread` and
  `@composer.send` on GPUI (6 of 6); core's tests, the footer tip's and the
  seq ranges' among them.

### 2026-11-11 Getting unstuck, search, and schedules from where they show (`banner.connection`, `system.startrunner`, `banner.signin`, `banner.error`, `thread.loadfailure`, `sidebar.search`, `header.schedulechip`, `highlights.scheduled`)

- The connection bar tells the Swift app's cases apart: a dropped connection
  ("Lost the connection to the runner. Reconnecting…"), a runner that can't be
  reached ("Can't reach the runner.", the reason in its tip) and a refused
  token (the runner closes with 4401: "The runner didn't accept the token in
  secrets/runner-token."). While it tries again it keeps saying which, rather
  than "Connecting…" on each try; Try again says "Connecting…" until that try
  has an answer.
- Can't reach the runner offers Start runner (`runner_launcher.rs`):
  `launchctl kickstart` of the installed runner's LaunchAgent, loading its
  plist first if need be, then a fresh try. Only for an app on the installed
  runner's own data folder, so an app on the mock or a dev runner never
  starts the live one; it spins and can't be pressed twice while it works.
- Sign in… opens Terminal on the CLI's own sign-in for the login that was
  refused, as the runner starts that CLI (claude-main with no
  CLAUDE_CONFIG_DIR, other Claude logins with their folder, Codex with
  CODEX_HOME), through a `.command` script written into a new file and renamed
  over the old one, with the Swift app's quoting rules (core's `sign_in.rs`,
  with the Swift tests ported, the zsh run among them). Another Mac's
  signed-out thread says to sign in on that Mac, with no Sign in….
- Investigate works in an installed build: its thread starts in the
  checkout the runner follows main in (`runner.status`'s `update.checkout`),
  or a fork's release checkout, and is handed what the app knew (the error,
  when, the open thread, the versions; core's `investigate.rs`, from
  InvestigateText). It clears only the error it was about.
- A thread whose events can't be read says "Couldn't load this thread.",
  why, and Try again, in place of the transcript; a reload over a transcript
  already shown keeps it. The dump's `transcript.state` is `failed: …`, as the
  Swift app's.
- Search matches the title, the preview and the librarian's search words
  (`keywords`), case and accents aside ("cafe" finds "Café").
- The bar's Scheduled chip says when its schedule runs, or that it's paused or
  gone, and opens it in the editor; Highlights' Scheduled rows open it too,
  with Edit and Show all schedules on their menu.
- The buttons on the bar and the load view are drawn as Swift's regular push
  button (`push_button.rs`): 24 tall, a fill of white at 8% in the dark look
  and black at 8% in the light one, no edge, measured against Swift's pictures
  (#2e2e2e on #1c1c1c, #ebebeb on white, and tinted on the yellow bar).
- The mock gains `mock.goAway` (a drop with no restart notice),
  `mock.failEvents` and `mock.keywords`; both apps' `highlight` command can
  click a Scheduled row and list its menu.
- Codex reviewed the sign-in script, Start runner and the rest (static, at
  high effort): it found the refused token taken for an unreachable runner,
  Investigate clearing a newer error, and the open gate's source check failing
  on the new launchctl call and on an ask link's `file://` pasted from a path
  (from the ask links slice). All four are fixed; the check now names
  `runner_launcher.rs` as allowed, and the ask link is made with
  `open_policy::file_url`.
- Not here: the CLI-update line on the bar (it comes with the Updates panel's
  CLI state), and BrowserBear's `threadbear://` links (`nav.deeplink`).
- Files: `desktop/client/src/runner.rs`, `desktop/core/src/{sign_in,
  investigate,inbox,schedules,logins}.rs`, `desktop/core/tests/
  {sign_in_script,open_gate_source}.rs`, `desktop/gpui/src/{header,app,thread,
  sidebar,highlights,open_gate,push_button,runner_launcher,composer_card}.rs`,
  `app/Sources/Threadbear/Automation+Commands.swift`,
  `app/Tools/mock-runner/server.mjs`, `features/{connection,inbox,header,
  highlights}.feature` and their steps.
- Verification: on Tinker Desk, `@banner.connection`, `@thread.loadfailure`
  and `@foundation.rpc` pass on both apps (4 of 4 each), and
  `@header.schedulechip`, `@highlights.scheduled`, `@sidebar.search` and
  `@card.needsyou` pass on both (8 of 8 each). Pictures of the load view and
  the bar in both apps and both looks, measured with pixel-overlay: the
  heading, the reason and the button sit on Swift's rows to the point, and the
  fills match Swift's colours. `cargo test -p threadbear-core` (all of it,
  the open gate's source check included) and the client's tests pass.

### 2026-11-11 Notifications, the Dock badge, the finished ding and Ask before quitting (`system.notifications`, `system.badge`, `settings.general`)

- A thread that comes to need Mike while he isn't looking at it posts a
  notification (`notify.rs`, from Notifier): its title, and the first thing it
  asks or its preview. One per thread and point in it, so a later one doesn't
  replace an unread one. A click brings the app forward on that thread, after
  the list loads if the click is what started the app. Never for another
  runner's thread. macOS asks for permission the first time, for "Threadbear
  GPUI" as its own app.
- The Dock badge counts the threads waiting on Mike, other runners' included
  (`inbox::needs_you_count`, from ThreadSections.needsYouCount), set only when
  it changes, and kept up as a hibernating thread wakes.
- Settings > General has the Swift app's Quitting and Notifications sections:
  ⌘Q and Quit ask first, with Don't ask again (an NSAlert, as gpui's own prompt
  has no suppression box), and the switch turns asking back on; the finished
  ding (Chime, Bell or Droplet, made as WAV data in core the same way as the
  Swift app's `CompletionSound`, played with NSSound) when a thread stops
  working, at most once in 1.5 s, with its volume and Play.
- A scripted app (the automation build) does none of it, as the Swift app's
  dev and screenshot runs don't.
- Not here: notifications and the badge on Windows and Linux.
- Files: `desktop/core/src/{inbox,completion_sound}.rs`,
  `desktop/gpui/src/{notify,ding,quit_prompt,settings,settings_window,app,
  shell,sidebar,remote}.rs`.
- Verification: core's tests for when to notify, the tag, the badge's count
  and the dings' WAV data; both builds of the app compile on Tinker Desk.
  Notifications themselves need the installed app and macOS's permission, so
  they're for Mike to see on his Mac.

### 2026-11-11 Waking schedules, folds that slide, logins as they change, reply buttons, drafts, Quit and ask links

- An idle thread a schedule is due to message waits under Hibernating, soonest
  to wake first, rather than sitting in the Inbox (`inbox::wakes`); its row
  says when (`sidebar.sections`).
- Inbox, Running, Hibernating and Archived fold and unfold with Swift's
  0.2 s slide, their rows measured and clipped as they go.
- Logins and pools follow the runner's `accounts.updated` and `pools.updated`;
  Settings > Pools edits the pools (`settings.pools`).
- Reply buttons in a reply are pills that press, with Swift's colours for
  start, send, pressed, pressing and failed (`rich.replybuttons`).
- What's typed and not sent survives a quit (`desktop-drafts.json`, written
  as the app quits and a moment after typing).
- Quit, Hide, Hide Others, Show All and Minimize are in the menus and on their
  keys.
- An ask's links (↗ and review links) are under it on the card
  (`card.needsyou`).
- Verification: the scenarios for each (`@sidebar.sections`, `@reply-buttons`,
  `@composer.textbox`, `@menu.app`, `@card.needsyou`) on Tinker Desk, as each
  slice landed.
### 2026-11-11 A reply's shell block into the Terminal (`terminal.putcommand`)

- A closed ```sh, ```bash, ```zsh or ```shell block in a reply has the Swift
  app's two icons at its top right, where ShellBlockIcons puts them: play
  (Put in Terminal) and copy, each ticking green for a moment when pressed.
  Play shows only where the Terminal is the thread's own shell in its own
  folder on this Mac (the thread in the main panel, on this Mac, not
  archived, its folder there); elsewhere a block offers copy alone. A block
  still streaming in has none, and one with control or invisible characters
  isn't offered (threadbear_core's `ShellBlock`, ported with its tests).
- Play brings the Terminal up, starts the thread's shell if it has none,
  waits for its first prompt, and pastes the command in brackets with the
  keyboard on it, never pressing Return; the panel says "Return runs it".
  A second block's command takes the place of one still unrun (^U, checked
  gone off the screen first); the same one again does nothing; a shell
  that's running something, or has someone's own words on its prompt, gets
  nothing typed: the command is copied, and a note says "Terminal's in use,
  so it's copied". What's on the prompt is followed from the keys, as the
  Swift app does (`TerminalPut`), and whether the shell has the terminal from
  the pty's foreground group.
- The cursor now blinks as SwiftTerm's does, steady while you type and when
  a program asks for a steady one.
- Both dumps give a reply row's `shellBlocks` ("[put, copy] npm test" or
  "[copy] npm test"); both apps' `terminal` `put` takes the play icon's own
  path, a busy shell's copy included (the scripted Swift app's block icons
  now copy to its own pasteboard, as its other copies do). The mock's
  `mock.terminal` takes `block` for a reply holding one.
- Not here: the Swift app's other reason for no play icon, a thread open in
  a panel of its own, as this app doesn't open threads in panels of their
  own yet (its scenario stays `@manual @gpui-pending`).
- Files: `desktop/gpui/src/shell_blocks.rs`, the hooks in `rows.rs`
  (`assistant_row`), `thread.rs` (the dump), `main.rs`, `terminal.rs`
  (`put` through the icon's path, the blink), `terminal_element.rs`,
  `app/Sources/Threadbear/{AppDump,Automation}.swift`,
  `features/terminal.feature` and its steps, `features/reading-a-thread.feature`.
- Verification: on Tinker Desk, terminal.feature's 15 scenarios (four new:
  a block's command at the prompt and only Return running it, a second
  command in place of the first, a busy shell's copy, copy alone where the
  folder isn't here), GPUI 15 of 15 and Swift 15 of 15, and
  reading-a-thread.feature on GPUI. Pictures of both apps from `compare.ts`,
  dark and light, of a reply's block and the Terminal after play: the icons
  sit where Swift's do (the kit filled the corner it puts actions in, which
  showed as a box behind them; they're out of its flow now), and the pasted
  command, highlighted by zsh, and "Return runs it" match.

### 2026-11-11 The Terminal: a real shell per thread (`terminal.panel`, `terminal.lifecycle`)

- ⌘J (and View > Show Terminal) brings the Terminal up under the thread with
  a third of its height, as the Swift app does, starts the open thread's own
  shell in its folder and gives it the keyboard; ⌘J again puts it away and
  the shell keeps running. A thread on another machine, an archived one or
  one whose folder has gone gets the home shell, with the Swift app's line
  saying why over it. Nothing starts until it's asked for: the empty panel
  says "Start a shell" (its tip names the folder), and New Shell starts
  another once one has exited.
- The emulator is `alacritty_terminal` 0.26.0 (Apache-2.0): its screen,
  scrollback (10,000 lines), selection and escape codes, and its pty with a
  reader thread per shell. The shell is your login shell, `$SHELL -l`,
  started through `/usr/bin/env -u` so it never has a thread's id (the Swift
  app's `TerminalSession.environment`), with `TERM=xterm-256color`,
  `COLORTERM=truecolor` and `TERM_PROGRAM=Threadbear`. Each thread's shell
  stops (SIGHUP, as closing a Terminal window does) once the runner says the
  thread's archived or deleted, or its full list comes back without it, and
  every shell stops when the app quits.
- Drawn as an element of its own (`terminal_element.rs`): the grid in the
  Mac's monospaced system font (SF Mono, `.AppleSystemUIFontMonospaced`, the
  Swift app's `monospacedSystemFont`, one point over the transcript's code)
  with SwiftTerm's cell size, the panel's own background and text colours in
  each look, Terminal.app's 16 colours and SwiftTerm's 256 worked out from
  them in Lab, bold in the bright colours as SwiftTerm draws it, the cursor as
  an accent block (an outline when the terminal hasn't the keyboard, a bar or
  underline when a program asks), the selection in the system's colour, and
  an input method's text being composed at the cursor.
- Keys as a terminal sends them (`threadbear_core::terminal::encode_key`,
  with tests): arrows and Home/End in normal and application mode, xterm's
  modifier forms, F1 to F12, Control's C0 codes, ⌥ as Meta (⌥B is ESC b,
  ⌥← and ⌥→ go by words, as SwiftTerm has it), Page Up/Down scrolling back at
  a prompt and going to a full-screen program otherwise. A plain character
  goes through the input method, so accents, other languages and dictation
  work. ⌘C copies the selection, ⌘V pastes (in brackets when the program
  asked, with any ESC taken out so a paste can't close them), ⌘A selects
  everything; drag, double-click (a word) and triple-click (a line) select,
  Shift-click extends. The wheel scrolls back, or sends the arrow keys or the
  wheel itself to a full-screen program that asked; clicks go to a program
  that asked for the mouse (Shift to select instead). The grid follows the
  panel's size and the pty tells the program.
- Not here yet: a link the terminal shows (an OSC 8 hyperlink or a path)
  can't be clicked to open it; the cursor doesn't blink. The Swift spec for
  links stays `@manual @gpui-pending` in reading-a-thread.feature.
- Both apps' dumps have `terminal` (whether it's up, the shell, its screen's
  lines, its size, whether it has the keyboard, every shell), and both take
  the `terminal` command with `show`, `keys`/`press`, `scroll`, `select`,
  `copy` and `paste` as well as `toggle`, `type` and `put`; the GPUI app's
  `keys` and `press` go through its real key handling and input method. The
  mock's `mock.terminal` gives a thread a real folder (a temp one the spec
  makes, as the demo threads' are Mike's).
- Files: `desktop/core/src/terminal.rs`, `desktop/gpui/src/{terminal,
  terminal_element}.rs`, the hooks in `app.rs`, `shell.rs`, `automation.rs`
  and `main.rs`, `desktop/core/tests/open_gate_source.rs` (the terminal's
  file is allowed to start a program; `tty::new(` now counts as starting
  one anywhere else), `app/Sources/Threadbear/{AppDump,Automation}.swift`,
  `Views/WindowDock.swift`, `app/Tools/mock-runner/server.mjs`,
  `features/terminal.feature` and its steps, `features/tools/compare.ts`
  (`$thread` and `{"sleep": ms}` in a scene, and short file names),
  `docs/on-your-mac.md`.
- Verification: on Tinker Desk, `@terminal.panel` and `@terminal.lifecycle`
  (11 scenarios: ⌘J and the folder, `pwd`, colour output, scrolling back 300
  lines of `seq`, select all and copy, paste, the grid and `tput cols`
  following a wider window, the home shell's note, Start a shell, a shell per
  thread, archive stopping it, New Shell), GPUI 11 of 11 and Swift 11 of 11;
  core's tests (28 new). Pictures of both apps from `compare.ts` with the
  windows in front (SwiftTerm draws nothing behind other windows), dark and
  light, of a prompt, colours, bold and `ls -G`: the overlay is grey over the
  whole grid, so the font, the cell size and the colours line up point for
  point; the one difference it showed, bold cyan drawn as plain cyan, was
  fixed to bold-as-bright.
### 2026-11-11 The Runners, Updates and Connectors panels, with their buttons and dots (`runners.panel`, `updates.panel`, `connectors.panel`, `rail.activity`, `rail.dots`)

- Connectors, Runners and Updates are at the foot of the activity bar, above
  Usage, as in the Swift app: Connectors once the runner says it watches a
  gateway, Runners with more than one runner, Updates always. Each opens its
  panel under the threads list (as a tab beside Usage or another of them if
  one's open) and closes it again. The accent dot is something only Mike can
  do (a service signed out; a launcher to reinstall, a release of Mike's to
  bring in, a machine not following GitHub's main), the grey one something
  that sorts itself out (the gateway out of reach, a service connecting or in
  error; a runner offline or updating; a runner build or a CLI's new version
  on its way). Their tooltips say what's going on, as the Swift icons' do.
- Runners: how many are up, refresh (spinning until the runner answers) and
  the ⓘ; then a card per runner in Mike's order, with its icon, name, a star
  where new threads start, the commit it runs, its state ("Online, up 3h",
  "Updating", "Offline, last seen 2d ago") with its dot, and the threads
  running there, which open on a click. A click on its name opens its CPU,
  memory and disk bars and what the machine is, kept across launches
  (`runnersPanelOpen`). The figures are asked for every 10 s only while some
  runner's details are open and the window can be seen, and the list every
  30 s while the panel's on show. The cards drag into a new order, shown at
  once and saved with `runners.setOrder`.
- Updates: App, Runner (its build from main, threads finishing on the
  previous version, which open, the launcher's command with Copy, and
  machines not following GitHub's main), From Mike on a fork (the release's
  notes, then Bring Them In and Let's Discuss, which start their thread and
  open it, spinning meanwhile), and the CLIs (each one's version, Update or
  Update anyway, Go back, what a held one lacks, its update policy as a menu
  that shows the new choice at once and puts the old one back if the runner
  refuses, and Check now).
- Connectors: the gateway's machine with its admin page, refresh and the ⓘ;
  a row per service with its state, its message, and Connect (its sign-in
  page, through the open gate) or a quiet arrow to its page on the gateway;
  and why there's nothing when the gateway can't be reached or isn't set up.
- The words, figures, order and dots are in desktop/core (`runners.rs`,
  `updates.rs`, `connectors.rs`), ported from RunnersText, RunnerStats,
  UpdatesText, CliText and ConnectorsText with their tests. Swift's small
  push buttons and borderless menu are drawn to its sizes
  (`activity_panels.rs`).
- Not here: the app's own update (the Swift app's AppUpdater section and the
  icon's right-click popover), as this app doesn't build and swap itself in
  yet, so its App section says how a new build goes in; Setup's button, which
  comes with `tour.setup`; the tour's intros for these panels; and "Not paired
  with this Mac", as this app reaches other Macs' threads only through Convex
  so far.
- Both apps' dumps give each panel's lines and whether it's on show
  (`runners`, `updates`, `connectors`) and the rail buttons' `quietDot`. The
  GPUI app takes the Swift app's `runners`, `updates` and `connectors`
  commands, and `runners` with `drag` and `onto` drags one card onto another
  through real pointer events and hit testing.
- Files: `desktop/core/src/{runners,updates,connectors,remote}.rs`,
  `desktop/gpui/src/{activity_panels,runners_panel,updates_panel,
  connectors_panel,rails,shell,app,automation,remote,settings,icons}.rs`,
  `app/Sources/Threadbear/AppDump.swift`,
  `features/{runners,updates,connectors,rails}.feature` and their steps.
- Verification: on Tinker Desk, rebased on main: the new scenarios with
  the rails' and the dock's (`@runners.panel`, `@updates.panel`,
  `@connectors.panel`, `@rail.activity`, `@rail.dots`, `@rail.right`,
  `@dock.layout`, `@dock.reset`), GPUI 20 of 20 and Swift 19 of 19 (the card
  drag is `@swift-pending`: a scripted drag in the Swift app needs the real
  pointer); core's tests and the pairing check. Pictures of both apps' three
  panels side by side in dark, and Updates in light, measured with
  pixel-overlay and the pixels' colours: the lines sit where Swift's do, and
  the push buttons (Swift's are a flat 7.5% fill with no edge, the same
  widths), the menu's words and chevron and the default runner's star were
  brought in to match. Choosing from the update policy menu by script shows
  the new choice at once. Left as they are, as they're the whole app's: every
  panel sits 2 points higher under the dock's bar, bruce's Mac mini is the
  icon set's server, and GPUI's text draws a little lighter.
### 2026-11-11 The review window's clips, pages and Open live (`review.media`, `review.openlive`)

- GPUI can't play a video or show a page yet, so a shot that's a clip or a
  page shows its still on the stage, dimmed, with Play the clip or Open the
  page over it. Each goes through the open gate (`open_file`), which opens
  the clip in the app this computer plays videos in and the page in the
  browser; nothing else is opened from the window. The Swift window plays
  the clip on the stage with its sound (muted side by side) and shows the
  page in place with its scripts off, so `review.media` stays pending here.
  In the grid, a clip's or a page's still has a play or globe mark on it.
  Zoom, the minimap and the title bar's zoom button are for pictures only,
  as in the Swift window.
- Open live, in the caption of an option that has a `url`, opens it in the
  browser through the gate's `open_web`: only an http or https page, as the
  url was already checked when the review was read.
- Both apps' `review` command says what the stage draws (`stage`: picture,
  player, page, or still here) and what Open live would open (`live`), so
  the specs check them without opening anything on the machine running
  them.
- Files: `desktop/gpui/src/review_window/stage.rs` (the still and its
  button, the grid's mark, the caption buttons' own symbol sizes),
  `desktop/gpui/src/review_window.rs`,
  `app/Sources/Threadbear/Automation+Waiting.swift`, `features/review.feature`,
  `features/steps/review.steps.ts`.
- Verification: review.feature on Tinker Desk: GPUI 16 of 16 (Open live
  included; the clip-plays scenario fails as it should, `@gpui-pending`),
  Swift 14 of 14 (media included). review-shot.ts's `shots-page`,
  `shots-video` and `shots-all` in both apps: the Swift window plays the
  clip and shows the page; the GPUI one shows each still with its button.

### 2026-11-11 The review window's stage: every shot, side by side, zoom, the minimap and today's version (`review.nav`, `review.zoom`, `review.minimap`, `review.holdspace`)

- Back and next step through every shot of every option, then all of them
  side by side to choose from (a grid of as many columns as make them
  biggest), and stop at either end; the up and down arrows jump between
  options, Return goes in and out, Esc out of a zoom then out to all of
  them. Under the stage, the option's shots in a strip and a mark for every
  shot of every option (a reference's hollow), each a click away, the row
  kept on the one on show; the rail has an option's shots too.
- Zoom: the title bar's button (full size from the top, or fit), ⌘= ⌘- ⌘0
  on a shot, the wheel and a pinch zoom round the pointer, a trackpad swipe
  zooms from fit and moves round once zoomed (decided as it starts, so it
  doesn't turn from one to the other), dragging moves round, a double click
  fits. The minimap shows the whole shot with the part on show while some
  of it's off the stage, and goes where it's pressed or dragged.
- Holding Space, or the caption's button, shows today's version (the first
  reference option) over the shot on show: its shot of the same name, at
  the same zoom and place; moving anywhere lets go.
- ⌘W closes the window, from the reply box too, and the draft's kept.
- The mock runner's `mock.review` puts a review on a thread as show_options
  would, with several shots an option, a page, a clip and a live url
  (`features/support/review-demo.ts`), for the specs and review-shot.ts's
  new states.
- Left: the stage has no scroll bars when zoomed (the minimap says where),
  and two-finger double tap doesn't zoom (GPUI has no smart magnify).
  Holding Space can't be scripted (a scripted key goes down and up at once),
  so its spec presses the caption's button. The window opens with the keys
  on the stage, so the arrows step at once, as the Swift code means them
  to; the Swift window's reply box takes the keys as it opens.
- Files: `desktop/gpui/src/review_window.rs` (keys, ⌘W, a link followed
  from inside the app's own update), `review_window/stage.rs` (the strip's
  titles on SwiftUI's line, the peek button's 140), `features/review.feature`,
  `features/steps/review.steps.ts`, `features/support/review-demo.ts`,
  `features/tools/review-shot.ts`, `app/Tools/mock-runner/server.mjs`.
- Verification: review.feature on Tinker Desk, GPUI 15 of 15 and Swift 12
  of 12 (keys and the bar's list can't be scripted in the Swift window).
  review-shot.ts's `shots` and `shots-all` in both apps, overlaid: the
  strip, the marks with several dots, the rail's shots and the grid land on
  the Swift window's points.

### 2026-11-11 The design review window: answering an agent's options (`review.window`, `review.picks`, `transcript.review`, `header.pendingreviews`)

- An agent's `show_options` can be answered here now. The review card's
  button, a `review:` link in a reply or on an ask, and the bar over a
  thread with reviews waiting ("Review design options (N)", its list from
  `reviews.list`, older reviews too) open the review in a window of its own,
  90% of the screen and centred, as the Swift one: a waiting review opens on
  its first option one at a time, one that's over on all of them.
- Down the right, the agent's lead and each option with its reasons, a tick
  for as many as Mike wants (a reference has none), the agent's pick starred
  with "Recommended", one reply box, None and Send. Send says what will go
  ("Send B + C", "Send none of them"), spins and takes no second press while
  it goes, and closes the window once it's sent. The draft is kept on this
  computer at each change (`desktop-review-drafts.json` in the runner's
  folder, as the Swift app's defaults) and saved to the runner half a second
  after the last one; closing the window keeps it. A save or send against a
  stale revision reloads, keeps what's typed and says so, with Save my draft;
  calls ride out a runner that's restarting for 20 seconds. Answered,
  cancelled and superseded reviews are read-only, with what was sent and
  Done. The title bar's ... has Cancel review, asked first.
- The stage, its arrows, the marks for every shot of every option, the grid
  of all of them and the shot's zoom are drawn too; their specs come with a
  later entry.
- The rules are `threadbear_core::review` (the session and its save races,
  the walkthrough, the grid's columns, the zoom and minimap), with the Swift
  tests ported. The Swift minimap is bottom right, whatever its comment says,
  so this one is too.
- The rail draws the agent's notes as SwiftUI's inline markdown does (bold,
  italic, code in the mono face with no box, links through the open gate),
  on the rail's own line heights (`review::inline_markdown`). GPUI snaps a
  line height to whole device pixels, rounding a half down, so on a 1x
  screen a reason's 16.5 lines are 16 apart (a point or two a row); on a
  Retina screen they're SwiftUI's. Bold text is a little wider than
  SwiftUI's here, so a note can wrap a word sooner.
- Left: an unknown review in a `review:` link says so in the error bar
  rather than beeping.
- Both apps' automation: `review` answers `focused`, `sendLabel`, `editable`
  and `minimap` too, takes `browse` and `open: false`; `click` takes
  `"review": id` for a control in that window, and the header dump has
  `pendingReviews`. `features/tools/review-shot.ts` takes both apps' review
  window in eight states and lays each pair over each other.
- Files: `desktop/core/src/review.rs`, `desktop/gpui/src/review_window.rs`
  and `review_window/{stage,rail}.rs`, the hooks in `rows/cards.rs`,
  `rows/markdown.rs`, `header.rs`, `app.rs` and `automation.rs`;
  `app/Sources/Threadbear/{Automation,Automation+Waiting,AppDump}.swift`;
  `features/review.feature`, `features/steps/review.steps.ts`,
  `features/tools/review-shot.ts`.
- Verification: `features/review.feature` on Tinker Desk, GPUI 8 of 8, Swift
  7 of 7 (the bar's list is a Swift popover the scripts can't reach). The
  core's 42 review tests. review-shot.ts in dark, both apps, overlaid: the
  stage, the grid, the row under it, the title bar and the footer land on
  the Swift window's points; the zoomed shot is 7 off only because Tinker
  Desk's Swift scroll view shows a legacy scroll bar.

### 2026-11-11 Point for point with the Swift app: the composer, its card, and what's round them

- Mike: "can you do like a pixel by pixel comparison ... this decision panel
  that I'm typing into right now looks slightly larger on the GPUI version".
  `features/tools/compare.ts` now lays each pair over each other (Swift in
  red, GPUI in cyan, grey where they agree) and can run commands before a
  shot (a card's second page). `features/tools/pixel-overlay.swift`'s `rows`
  and `cols` give each line's top and bottom in both pictures, paired, which
  is where every number below came from.
- The composer box: the kit's text box pads itself 10 and 8, so the GPUI box
  was 18 taller than Swift's with its text 11 further in. It's unpadded now
  (`composer::bare_textarea`, the Something else box too), and the box's edge
  is drawn over it as Swift's strokeBorder is (0.5, or 1 in the accent),
  rather than taking room in it. The empty box now matches to the point.
- The card (`RowRenderer.asksText`): Swift's paged heading is a 38 pt line
  (a work line plus 6 otherwise), each ask 12 under what's above, its number
  where a list's goes (8 in, text at 30), a where line at 11 pt on 19, and
  buttons on 38 pt lines 6 under that, 29.3 tall, three no-break spaces
  either side and four between, with a soft shadow. The title is 14 pt
  medium beside SF's filled hand (drawn here, as Lucide has only the
  outline), the picked button is the accent with white words, the agent's
  pick has an accent edge on the grey, the pager's live arrow is a button,
  and the chevron is Swift's 22 pt square at 5 pt corners. A paged card is
  as tall as its tallest page, all pages in one grid cell with only the shown
  one drawn. The separator under it is half a point. The done card has SF's
  tall tick, rows 8 over and under with their circle at 8 and words at 34,
  and the small labels at 11 pt.
- Elsewhere: the user bubble is as wide as its widest line (GPUI's wrapped
  text takes all the room it's offered); list markers end 11.2 short of the
  text, which puts a bullet and "1." where Swift's are; a reply that opens
  with a heading starts without the heading's 6 (TextKit leaves out a first
  paragraph's space before); the Highlights rows take SwiftUI's own line
  heights (14 at 11 pt, 16 at 13, 19 at 15: `style::natural_line`, measured
  with NSHostingView), where GPUI's are 1.618 times the size; the sidebar's
  search field is Swift's 29 tall; the thread's bar title sits where Swift's
  does; Full access has SF's open lock and "+" SF's size; and the needs-you
  orange is systemOrange.
- A thread is read only while one of the app's windows is in front (the
  main one, Settings or a panel's window of its own, each saying so from its
  own activation), as Swift's `isLooking` has it: GPUI read one as soon as it
  opened, behind other windows too, which moved it down the Inbox in the
  scripted pictures. A click or key in a window counts as well, for an X
  server that never says a window's in front, but nothing a script does
  does, and the scripts' `focus` now means "in front" in both apps. GPUI's
  own "active window" isn't used: on the Mac it's NSApp.mainWindow, which
  stays set behind another app.
- Left: gpui-kit puts list items 3.75 apart (a quarter of the text size,
  fixed in the kit) where Swift's are 6, and indents a nested list 21 where
  Swift's is 30, so a card with a list in its why is a few points shorter;
  that needs the kit to take an item gap. Reply buttons inside list items
  (pills in Swift) are still plain links here.
- The reading styles (the transcript's, a file preview's, the card's) are
  kept per zoom as well as per appearance, as their code sizes are zoomed
  when they're built.
- Files: `desktop/gpui/src/{composer_card,composer,composer_chips,icons,
  style,rows,rows/markdown,highlights,sidebar,header,thread,app,shell,
  settings_window,automation,open_gate,files}.rs`,
  `desktop/dock/src/{content,windows}.rs` (`window_activation`),
  `features/tools/{compare.ts,pixel-overlay.swift}`.
- Verification: compare.ts on Tinker Desk, every demo thread in dark, with
  the rows tool after each change (the empty composer, the card on both
  pages, the bubble, the Highlights rows): the sidebar went from 10.8% of
  its pixels apart to 2.1%, and the Highlights and main panel came down in
  every thread. Codex's review, three rounds (read tracking, zoom, a "2."
  ask's escape, floating windows, no window manager, every scripted input),
  each finding fixed. The GPUI specs on Linux and the Mac.

### 2026-11-11 The Settings window, with General and Models (`settings.window`, `settings.general`, `settings.models`)

- ⌘, and the rail's gear open Settings in a window of its own, with the
  Swift app's sidebar of sections (General, Models, Pools, Runners, Phone &
  browser, Keyboard) and a grouped form whose sections say, behind their ⓘ,
  what each is and what to ask your agent. A second ⌘, brings it forward.
- General: Appearance (this app's own, kept in desktop-settings.json and
  followed by every window), the default folder with Choose…, standing
  instructions (saved a second after typing stops and when the window
  closes; a file changed outside Settings is shown as it was when reading
  started, with Show Newer Version, Clear and Use These, which sends the
  words shown), Code review, Sleep, Updates from Mike (on a fork),
  Experiments, and the runner's version. A change shows at once, and one
  the runner turns down puts back what it had (unless something newer came)
  with its reason under the form.
- Models: Small jobs' pool, with None and a set pool that has no Claude
  login, and the private folders, with a minus on each and Add Folder….
- Not here yet: the ding (GPUI has no sound to play it with), the activity
  log (the GPUI app keeps none) and Quitting's question (it doesn't ask),
  and the Pools, Runners, Phone & browser and Keyboard sections, which say
  so.
- The GPUI app's scripts reach the window as the Swift app's do (`settings`
  with `tab` or `close`, `"settings": true` on `click` and `screenshot`), and
  both dumps say what it shows (`settingsWindow`).
- Verification: settings.feature on Linux and on Tinker Desk; the Swift app
  passes the window scenario, and its two click scenarios are GPUI-only, as
  a scripted click in the Swift app's AppKit form starts a tracking loop that
  never answers; both apps' General and Models side by side.

### 2026-11-11 The Librarian view and its page (`librarian.log`, `librarian.page`, `librarian.picture`)

- Librarian in the activity bar (with its count of looks under way and
  waiting) opens the view in the sidebar and its page beside it, as in the
  Swift app. The view: what it's doing, with its mark in the rows' gutter (a
  spinner while it works), the look under way and when it started, today's
  looks, skips and tokens, Pause or Resume (shown at once, put back if the
  runner says no) and Look at everything (spins until the looks are
  queued); then Next, each look's thread (which opens), why and when; and
  Recent, what each look did, to which thread and projects, coloured by how
  it went, with why it filed the thread in its tooltip.
- The page: the bear at its card catalogue, the title with its ⓘ, what
  it's doing, the six tiles (LibrarianStats, ported to desktop/core with the
  status model and its words), then its settings: Look at threads, and
  Runs on, the pool whose Claude logins it uses (the runner's small-jobs
  setting, `settings.update { models: { pool } }`), and More in Settings.
  The app now keeps the runner's settings (`settings.get`, then
  `settings.updated`) for that, and for the Settings window to come.
- The Scheduled and Librarian pages share one page (desktop/gpui/src/
  home_page.rs): the picture, what it says when poked, the title and ⓘ, the
  status line and the tiles.
- A project's name in a Recent row is plain words for now, as a project's
  page isn't in this app yet.
- Both apps' dumps give the page's status and tiles (`homePage`) and the
  librarian's state in words (`librarian`).
- Verification: librarian.feature's 3 scenarios with scheduled.feature and
  header.feature on Linux and on Tinker Desk against both apps (24 of 24
  each; both apps' tiles agree, Projects it made and Threads it filed
  included); the view and page side by side in both apps.

### 2026-11-11 The limit box on the composer (`banner.limit`)

- The Swift app moved a thread at its limit from a bar across the top of
  the thread into a box at the top of its message field (ae80b80b, Mike's
  call on 10 Nov), after the last full run, so its `banner.limit` scenario
  failed there while the GPUI app still drew the bar. The GPUI app has the
  box now: what happened, and when it carries on by itself ("Carries on by
  itself at 11:06 am."), with a clock while it waits and the red gauge
  otherwise; Switch to (a pool thread's Pin to) the login with the most
  room; and Carry on by itself, Yes or No (`threads.update
  carryOnAtReset`), for a turn that stopped at the limit on this Mac.
- A pinned thread waiting for its own login (`waitingForRoom` without a
  pool, also new on 10 Nov) gets the box too.
- Both dumps say what the box offers (`limit.carryOn`), and the scenario
  checks it.
- Verification: header.feature on Linux and on Tinker Desk against both
  apps (12 of 12 each); the thread side by side in both apps (compare.ts).

### 2026-11-11 The Scheduled view, its page and the editor (`schedule.list`, `schedule.page`, `schedule.editor`)

- Scheduled in the activity bar opens the list in the sidebar and its page in
  the main panel, as in the Swift app. The list has Active, Paused and Done
  headings that fold (Done starts folded; saved as
  `collapsedScheduleSections`). A row says when it runs next (or Paused,
  Done), when it runs, the other Mac it runs on, and where it sends: a link to
  that thread, or to the last one it started. Under that, a last run that
  didn't send, or another Mac's Run now still waiting.
- On hover: Run now (a spinner until the runner answers), Pause or Resume
  (shown at once, put back if the runner says no) and Delete (asks first,
  gone at once, back if refused). The right-click menu has the same, with
  Edit.
- The page: the bear winding its alarm clock (built into the app), the title
  with its ⓘ, what's next and the six tiles (`ScheduledStats`); the picture
  says something when it's poked. Its wobble isn't here yet.
- The editor (`ScheduleEditor`): the message, a title, Once (a date and time)
  or Repeat (presets that are only cron underneath, or a cron of its own),
  and under it what the runner makes of that as it changes
  (`schedules.preview`, a quarter of a second after the last change). Then
  Sends to, Runs on (another Mac's folder is typed as a path there), Folder,
  and Model and Effort from the login it runs on (`ScheduleModels`, ported
  to desktop/core). Delete asks over the sheet and closes both.
- Picking a thread (a schedule's target) puts the thread back in place of
  the page and leaves the list as it is; Back and Forward remember the page.
- Both apps' dumps say what a schedule's row shows, the page in place of the
  thread (`homePage`) and the editor as it was opened (`scheduleEditor`). The
  GPUI app takes the Swift app's `schedule` and `close-schedule` commands,
  and its `select` now picks a thread without leaving the view, as Swift's
  does.
- Schedule… in a composer's + menu opens the editor on the box's words and
  its thread (the new-thread box's: a new thread each time on its folder,
  login and model), and they leave the box once it's saved. Pictures are
  chips in the GPUI box's text, so a box with any is left as it is, and the
  editor says so.
- Codex's review found five things, all fixed: an editor opened before this
  Mac's runner was known saved another Mac's schedule with this Mac's
  folder and models (it's now taken to be the other Mac's until known); a
  Run now or Pause answered after the schedule was deleted brought its row
  back (fixed in the Swift app too: only a new schedule's reply adds a
  row); Back to a page recorded the thread behind it as a step, which cut
  off Forward (a scenario now covers it, on both apps); Save's answer could
  close Delete's question instead of the editor (Delete is off while
  saving, and only the editor that saved closes); and a page's place in
  the history needed the thread behind it to still exist. Its second look
  found Back to such a page then recorded it as a new place, trapping Back
  there; a place put back now settles into its own entry, keeping Forward.
- Remaining: Schedule… from Send's right-click menu
  (`composer.schedulefromsend`), the header's schedule chip, the picture's
  wobble, Return in the message box saving.
- Verification: scheduled.feature's 9 scenarios on Linux, and on Tinker Desk
  against both apps; pictures of both apps' list, page and editor side by
  side (the spacing was brought in to match after the first pair).

### 2026-11-10 One gate for opening links and files (Daybreak 58, 59, 63, 64, 67)

- Every link and file the app hands to the system goes through
  `desktop/gpui/src/open_gate.rs`, on the rules of the Swift app's OpenGate,
  written in `desktop/core/src/open_policy.rs`: a web page opens, any other
  scheme asks by name, a file is judged and opened as its real target after
  symlinks, a known document opens and anything else (a program, a bundle, a
  script, an unknown kind, a document whose app runs it, which macOS's Launch
  Services is asked) asks first, and another runner's file never opens here.
  File URLs are built from the path with `#` and `?` escaped, never pasted.
  `desktop/core/tests/open_gate_source.rs` fails on an opener anywhere else.
- The slash palette's Open file previews the skill in the pane; a file link in
  another runner's thread previews there too, read through that runner.
- Every rich text view (replies, plan and ask text, Markdown previews) is made
  by the gate's `text_view` or `markdown_view`, as the kit's own TextView opens
  a link with the system's opener when it has no handler; a link in a
  previewed Markdown file to another file previews in the pane.
- Threadbear's own approval cards show the runner's words under the title
  (`ApprovalText::runner_reason`), in both apps. File cards show every line
  of the diff, any argument it doesn't draw (`replace_all`, a notebook cell)
  and a line break at an edit's end in words, in both apps (and the runner
  sends an edit's card whole, refusing one too long to show), and Return on a stacked card allows only the ticked
  calls, as its Allow button does.
- Pasted pictures are staged in a fresh 0700 folder (`XDG_RUNTIME_DIR` when
  there is one), written 0600, and removed once the runner has its copy.
- Remaining: the Swift app's forwarding page for a local `.html` link's
  `?query` and `#fragment`, which the gate drops; previewing transcript file
  links in the pane on this runner's threads (they open in their app).
- Verification: `cargo test --workspace` on Tinker Desk (macOS, with the Launch
  Services code); the open policy's own tests and the source scan on Linux;
  the whole spec suite on Linux (see the report for the run).
### 2026-11-10 Looking like the Swift app, and the pool limit line

- Found by putting both apps' pictures of every demo thread side by side
  (`features/tools/compare.ts`, which Mike's "a number of things that are
  off" asked for).
- Dark mode was near-black: gpui-kit's own dark theme is #0a0a0a, where the
  Swift app draws on the system's window colour (#1e1e1e on macOS 26 and 27).
  The GPUI themes now take NSColor.windowBackgroundColor from AppKit on the
  Mac, and those values elsewhere (`style::install_window_colours`). Menus
  and popovers sit a little lighter, as macOS's do.
- Scroll bars show only while scrolling, as the Swift transcript sets them,
  whatever the system's setting. The side pane's second "Highlights" title
  under the dock's is gone. Rows name their Mac ("• m5") once there's
  another, as Swift's have since 9 Nov, and a long status gives way before
  the folder does. The thread's bar shows the project's own icon, not its
  letter.
- A thread waiting on a permission card counts as busy, as Swift's
  `isBusy` has it, so the composer offers Stop.
- `banner.limit`: a pool thread out of room says "Every login in Evenings is
  out of room. It carries on by itself at …" with a quiet clock and "Pin to
  mike (78% left)"; an account at its limit names its short login, when it
  resets, and the login with most room. The model chip says "Limit reached"
  (or "· 12%" running low) when every login in its pool is, and the account
  chip for its login. Ported from Swift's `limitAdvice`, `Pools.chipBadge`,
  `AccountUsageText.switchTarget` and `switchTitle` into `desktop/core`.
- Files: `desktop/gpui/src/{style,app,header,highlights,sidebar,remote,
  composer_chips,automation}.rs`, `desktop/core/src/{inbox,new_thread,
  logins,sidebar}.rs`, `AppDump.swift` (`limit`, `composer.chipBadge`),
  `features/header.feature`.
- Remaining from the pictures: reply buttons drawn as pills, the thread chips
  bar ("Review design options"), the rail's Connectors, Runners, Updates and
  Setup buttons and the Librarian's count, and the providers' logos on the
  model chip.
- Codex (gpt-6.1-sol) tried to break the drag, the caching, the sort, `busy()`
  and the limit port. It found no stale views, no lease panic and no change of
  order, and two real problems, both fixed: a torn-out window got its last
  move from a press that ended without a mouse-up, so it could jump to where
  the pointer had gone since (`pointer_up(released, ..)` in
  `desktop/dock/src/windows.rs`); and the limit line lacked Swift's case for a
  turn that failed on its limit while the account still says it has room
  (`failedOnAccount`, now `note_failed_account` in `header.rs`). That case
  looks for the newest turn footer, as Swift's does, and the demo thread that
  failed on its limit has none for the failed turn, so neither app shows a
  line there; there's no scenario for it yet.
- Idle frames: an open thread draws nothing once it has settled (four seconds,
  no frames); the readout's 30 fps was the second after opening one.
- Verification: `features/tools/compare.ts` pictures of the changed threads
  beside the Swift app's; specs 193 of 193 on Tinker Desk's Mac and on Linux
  (`cloud`), the Swift app 191 of 191 there, the new `banner.limit` scenario
  passing on both apps; `cargo test -p threadbear-core` 163 of 163.

### 2026-11-10 Dragging the window, and smooth scrolling at 120 Hz

- Mike couldn't move the installed app's window by its top. The strips that
  start the OS's window drag on the press worked; the panels' bars and the
  thread's title didn't, as they only know it's a drag once the pointer moves,
  and AppKit won't start a window drag from a move event. On the Mac those now
  move the window by hand, as Swift's dock does (`platform::grab_window`), and
  a drag from the title no longer starts a rename (`DockView::press_moved`).
- Scrolling: every frame rebuilt the whole window, thread list included. The
  Threads panel and the side pane are cached views now, the Inbox sorts by
  keys worked out once, and the menu bar and the Inbox badge are worked out
  only when the app changes. Tinker Desk (M2, 60 Hz), 6,000 rows and 60
  threads, real flings: work per frame 7.8 ms in the middle, 13.4 at 90% and
  21.8 at 99% before; 6.3, 7.9 and 9.7 after (28 late frames to 1).
- View > Show Frame Rate in the installed build (`frame-stats`, GPUI's frame
  profiler) shows frames a second, late frames and the slowest frame.
- Files: `desktop/dock/src/{platform,view,windows}.rs`, `desktop/gpui/src/
  {frame_stats,sidebar,shell,app,rails,header,composer_chips}.rs`,
  `desktop/core/src/inbox.rs`, `features/tools/{window-drag,scroll-perf,
  real-screen}.ts`.
- Remaining: the root, the dock's bars and the thread pane still draw again on
  every frame of a scroll (the dock would have to hand out its bars as views).
- Verification: `features/tools/window-drag.ts` on Tinker Desk's desktop,
  every part of the top moves the window and a click on the title renames;
  `features/tools/scroll-perf.ts` for the numbers above; specs 192 of 192 on
  Tinker Desk's Mac and 191 of 192 on Linux (`cloud`). The one was a step
  that took an older reply for the streaming one when the stream was slow to
  start; it now tells them apart, and passes 4 of 4 on Linux and on both apps
  on the Mac.

### 2026-11-10 Every spec on real Windows

- Windows paths: the runner's `C:\\` paths work in folder names, relative paths
  and Highlights (`forward_slashes` in `desktop/core`), and the mock accepts them.
- Remaining: the folder browser still joins with `/`, which Windows tolerates
  but shows as mixed separators.
- Verification: `Behaviour specs` in the Desktop app workflow, 192 of 192 on
  Windows, Mac and Linux (run 42648513140).

### 2026-11-09 The first app that can be used all day

- Merged: the sidebar, the transcript's real rows, the composer and its cards,
  the window on BearDock with its rails, header, banners, menus and shortcuts,
  Highlights, previews, Patches, the Files panel, the new-thread page, the slash
  palette and other runners' threads.
- Fixed in the merge: dropping a file on selectable text now works, as a drop
  layer goes over the thread while a file's being dragged.
- Remaining: Scheduled, Projects, Librarian, Usage, Updates and Settings pages,
  the bears and flights, web panels, the terminal.
- Verification: 192 scenarios pass on Linux and on a real Mac; the Swift app passes
  the 188 of them that aren't marked GPUI-only.

### 2026-11-09 Against a real runner

- `features/tools/real-runner.ts`: the GPUI app on a dev runner, not the mock,
  sending a message on claude-haiku-5-5 and getting the reply back.
- Verification: on `cloud`, "reply after 4480 ms" with rows `user group assistant
  footer`, the same row kinds as the Swift app.
### 2026-11-09 The window shell: BearDock, rails, the thread's bar, banners, notes, menus and keys

- The app's window is BearDock now: `desktop/gpui/src/shell.rs` hosts the dock
  (threads, thread, Highlights as panels, default layout and View > Reset, the
  layout saved and restored), with no title bar and the traffic lights in the
  top-left panel's bar, the bars moving the window. `rails.rs` is the activity
  bar and the right rail (`ActivityBar`, `PanelRail`, with the Inbox count, lit
  buttons and the Highlights dot). `header.rs` is the thread's title line in the
  panel's bar (inline rename, project and schedule chips), back and forward with
  tips that name where they go, share with its popover, the ... menu, the banners
  (connection, error with Investigate, limit, sign-in, the refusal offer) and the
  floating notes (Archived with Undo, end of inbox). `settings.rs` is one JSON file
  in the runner's data folder (`desktop-settings.json`, keyed by the Swift
  UserDefaults names): window frame, zoom, folds, the Inbox's sort, the panel
  layout. Zoom scales everything: every `px` in the window goes through
  `style::px` (the Swift `zoomed`), which each drawing file gets with one
  `use crate::style::px;` that shadows GPUI's, and gpui-kit's own sizes follow
  through the theme's rem size. Shortcuts are `threadbear-core`'s `shortcuts`
  (the Swift tests ported), bound from shortcuts.json with a poll that picks up
  edits. Menus are the same model for the Mac's menu bar and the `menus` command.
- Files: `desktop/gpui/src/{shell,rails,header,settings}.rs` (new), `app.rs`
  (state and the render), `main.rs` (window options, frame memory), `style.rs`
  (zoom), small additive edits to `sidebar.rs`, `thread.rs`, `composer_card.rs`,
  `rows/*`, `automation.rs` (the Swift names: `dock-move`, `panel`, `float`,
  `mode`, `zoom`, `rename`, `undo-archive`, `open-archived`, `error`,
  `mouse-button`, `copy-thread-id`, `menus`, `menu-item`, `app-menu`, `data-dir`,
  `share`, `signin`, `refusal`, `appearance`, `resize`), `desktop/core`
  (`navigation`, `shortcuts`, `zoom`), `desktop/client` (`retry_now`),
  `desktop/dock/src/view.rs` (`set_options`, so the bars follow the zoom). Specs:
  `features/{window,rails,header,navigation}.feature`. Swift: `AppDump` has
  `rails`, `header`, `zoom`, `inboxEndNote`; new debug commands `menus`,
  `menu-item`, `data-dir`; `dev-app.sh rebuild --no-build`.
- Remaining: the Scheduled, Projects and Librarian views and Usage/Settings pages
  are placeholders (later stages). Highlights' content comes from
  `desktop/gpui/src/highlights.rs` (other branch): `render_highlights_panel` in
  `shell.rs` is the one place to call it. Files is a placeholder panel. The
  Browser rail button, panel origin badges and a thread's own patch panels'
  content aren't there yet. Menus aren't drawn on Windows and Linux yet
  (`AppMenuBar`, `gpui-kit`'s `title-bar` for the window buttons there).
  Sign in… says the CLI command rather than opening Terminal. The refusal offer
  sits with the banners, not above the composer. Hold-⌘ hints, swipes and the
  honey pot are other rows. A scripted Swift app's menu items keep their greyed
  state from the last time the menu was opened, so three key-press scenarios are
  `@swift-pending` (`⌘[`, `⌘0`, `shortcuts.json` pressing the new key); the same
  state shows they pass through the menu on GPUI. The picture's traffic lights
  aren't in the GPUI shot (it draws in-process); their position was set from the
  Swift shot and not checked against a real window capture.
- Written but with no runtime check yet: window frame memory (scripted runs never
  save a frame, so only `zoom::on_screen` and the settings file have tests); the
  limit, sign-in and refusal banners and the project and schedule chips (no spec
  or fixture reaches them); a real mouse's side buttons (`mouse-button` calls the
  same handler, not a real event). On GPUI the `@menu.*` scenarios read the menu
  model, not the system menu bar. Simplified from Swift: the limit banner offers
  the first login not at its limit rather than the one with most room, and a
  waiting restart's banner has one fixed sentence.
- Verification: `TB_APP=swift` on Tinker Desk: 37 scenarios (37 passed); `TB_APP=gpui`
  on `cloud` (Xvfb): 40 scenarios (40 passed); `TB_APP=gpui` on Tinker Desk: 40
  scenarios (40 passed), the `@dock.layout or @dock.reset or @foundation.zoom or
  @rail.* or @header.* or @banner.* or @toast.* or @nav.* or @menu.* or
  @sidebar.sections or @sidebar.sort` set. `cargo test -p threadbear-core` (the new
  `navigation`, `shortcuts` and `zoom` modules) and `settings` tests. Side-by-side
  picture of the window on Tinker Desk: the Swift app and the GPUI app with the
  same thread open, 1180 by 800, dark.
### 2026-11-09 Move to another runner (`sidebar.move`)

- The row menu's Move to ▸ (behind `moveToRunner`): it goes straight away when
  the other runner has a clean answer, else the sheet comes up with its guess at
  the folder and login, what's in the way and what won't come, and asks again as
  they change. Cancel Move while it's going, and "Moving to bruce" on the row.
- Files: `desktop/gpui/src/move_sheet.rs`, `desktop/core/src/remote.rs`,
  `desktop/gpui/src/sidebar.rs` (the menu lines), `features/remote-threads.feature`,
  `app/Sources/Threadbear/Views/MoveSheet.swift` and `AppDump.swift` (the dump's
  `moveSheet`, debug only).
- Remaining: the login picker is a popover where Swift's is a pop-up menu; a
  summary landing while a move is being asked for takes the row's guess off
  (`keepingAskedMove`), which only shows as a flicker.
- Verification: `@sidebar.move`, 5 of 5 on `TB_APP=swift` and on `TB_APP=gpui` on
  Tinker Desk, and in the whole-suite runs under the new-thread page's entry
  below; a picture of the sheet on Tinker Desk.

### 2026-11-09 Other runners' threads and the runner chip (`sidebar.row.remote`, `composer.runner`)

- Rows from `remoteThreads.list`/`remoteThreads.updated` sit beside this Mac's,
  with the machine's icon in the tile and "On bruce" (faded when offline); opening
  one reads its events and sends through this Mac's runner, the app's default way
  in (the direct link's own connections aren't ported, `directRemoteLink`).
  The box on such a thread leads with its machine's chip; a thread's logins and
  folders are that runner's once they've come (`remoteThreads.logins`), as is the
  draft's. The new-thread box has the Runs on chip: this Mac, each runner, the star
  on the default and a Default submenu that sets it, and on another runner its
  folders. `composer-menu` with `runner` reads and clicks it, `setDefault` sets
  the star.
- Files: `desktop/gpui/src/remote.rs`, `desktop/gpui/src/composer_chips.rs`,
  `desktop/core/src/remote.rs`, `features/remote-threads.feature`,
  `app/Tools/mock-runner/server.mjs` (the mock now answers another runner's
  threads over its own connection, as the real runner does through the backend,
  and `remoteThreads.logins`).
- Remaining: the direct link (Settings > Runners), `files.read` of another
  runner's paths (their pictures show their own placeholder), the sparse mode is
  on for the relay but a real relay hasn't been run from this app.
- Verification: `@sidebar.row.remote` and `@composer.runner`, 7 of 7 on both
  apps on Tinker Desk, and in the whole-suite runs under the new-thread page's
  entry below.

### 2026-11-09 The slash command palette (`composer.slash`, `composer.slash.keys`)

- Floating over the box in the same window (a deferred layer, as wide as the box,
  half the window tall): categories with counts and Recent, the matches, the picked
  command in full with Run, Add details and Open file. The keys come from the box's
  own actions while it's open (arrows move the pick, Tab fills it in, Return runs
  it, Esc closes it until the words change), so the cursor stays in the box.
  Recent is kept in `gpui-prefs.json`. The rules are in `desktop/core/src/slash.rs`
  with tests; the dump's `palette` is on both apps.
- Files: `desktop/gpui/src/slash_palette.rs`, `desktop/gpui/src/composer.rs`,
  `desktop/core/src/slash.rs`, `features/slash-palette.feature`,
  `app/Sources/Threadbear/AppDump.swift` and `ComposerView.swift` (debug only).
- Remaining: `composer.slash.resize` (dragging its edges and columns);
  descriptions read as plain text rather than inline markdown. (Open file now
  previews the skill in the pane, as the Swift app does: see the 10 Nov entry.)
- Verification: `@composer.slash` and `@composer.slash.keys`, 9 of 9 on both apps
  on Tinker Desk, and in the whole-suite runs under the new-thread page's entry
  below; `cargo test -p threadbear-core` (the palette's own rules, 18 tests); a
  picture of `/co` on Tinker Desk read against SlashPaletteView.

### 2026-11-09 The new-thread page (`thread.newpage`, `composer.starting`)

- ⌘N and the compose button open a page with the box about 40% of the way down,
  the heading above it, "Starts in misc on the Claude pool" under it and the
  logins' room cards (a click pins the draft to that login); a project's New
  Thread adds its "In …" chip and folder. Sending goes through `threads.create` on
  the draft's pool or login, the words greyed and locked with a spinner for Send
  until the thread exists, and starts the next one from the defaults again. `new`
  (with `project`), `options`, and `composer-menu` work on the page as in the
  Swift app. The picks that stick (model, effort, last login, default runner, recent
  commands) live in `<data dir>/gpui-prefs.json`; `docs/on-your-mac.md` should
  name it when this app ships.
- Files: `desktop/gpui/src/new_thread.rs`, `desktop/gpui/src/prefs.rs`,
  `desktop/core/src/new_thread.rs`, `features/new-thread.feature`, a page branch
  in `desktop/gpui/src/app.rs`.
- Remaining: the page doesn't clamp the heading and note for very short windows
  as ComposerPlacement does; the bear over the heading.
- Verification: `TB_APP=gpui` whole suite, 125 of 125 on Tinker Desk (real Mac
  desktop) and 125 of 125 on Linux (`cloud`) at the final commit; `TB_APP=swift`
  whole suite, 125 of 125 on Tinker Desk; `cargo test -p threadbear-core`
  (92 lib tests); side-by-side of the page at
  `/tmp/gpui-new-thread-side-by-side.png`.
### 2026-11-09 Pictures in the transcript: an in-app viewer

- A click on a picture tile opens an overlay on that picture, with arrows through
  its message's other pictures and Escape to close, in place of the file's default
  app. Only a real, decodable picture opens (named as one, not a code runner, then
  decoded from its bytes): `viewer.rs`, `looks_like_picture` and `runs_code` in core,
  the `quickLooks` rule carried over.
- `tile` and `viewer` join the automation commands on both apps (Swift's `viewer`
  says what Quick Look would have shown, as the real panel comes to the front of a
  desktop); the arrows are GPUI's own, so that scenario is `@swift-pending`.
- Files: `desktop/gpui/src/viewer.rs`, `rows/images.rs`, `features/picture-viewer.feature`.
- Remaining: zoom and pan, video, PDFs, drag out (`file.quicklook`, `drag.fileout`).
- Verification: `TB_APP=swift` 1 of 1 and `TB_APP=gpui` 2 of 2 on Tinker Desk (real Mac) and on Linux (`cloud`), the arrows scenario `@swift-pending`; picture of the overlay on the GPUI app. Whole suite with the pane's 19 new scenarios: `TB_APP=swift` 124 of 124 on Tinker Desk, `TB_APP=gpui` 125 of 125 on Tinker Desk and 124 of 125 on Linux (one streaming scenario was too slow on a loaded box and passed on its own).

### 2026-11-09 Files panel and the folder browser

- The Files panel and folder previews: a list with fold-out rows, sortable columns
  (Date Modified and Size go when the pane is narrow, as the Finder's do), a path bar
  under the list, double-click to go in with Back to come out, and the right-click menu
  (Open, Open in its app, Reveal, Copy path, Sort by, Show hidden files). Rows come
  from the runner's `files.list`, so a thread on another Mac browses the same.
- Files: `desktop/core/src/folders.rs` (FolderSort, crumbs, sizes, `files.read`
  decoding), `desktop/gpui/src/folders.rs`, `features/folders.feature`.
- Remaining: the icon grid (`folder.icons`, the menu item is there, greyed), Finder keys
  and Quick Look on Space (`folder.keys`), type-to-select, the folder watch.
- Verification: `TB_APP=swift` 6 of 6 on Tinker Desk; `TB_APP=gpui` 6 of 6 on Tinker Desk (real Mac) and on Linux (`cloud`); side-by-side pictures against the Swift pane.

### 2026-11-09 Patches: the list, chips and native panels

- The Patches section (Trust, waiting lines, what each gives, grouped under its
  project), chips over the composer (`actions.run`, spinner while it runs), list panels
  (headings, tones, Show in pane rows), image panels (a refreshed picture through
  `files.read`, so another Mac's works) and the web-panel placeholder with Open in Browser.
- Files: `desktop/gpui/src/patches.rs`, `features/patches.feature`, core `Panel`,
  `Action`, `PatchInfo`, `PanelRender`.
- Remaining: chip look (the Swift app's grouped bar with its +N overflow; these are
  simple pills), patch and `.app` picture icons (`patches.icons`), web panels
  (`panel.web`).
- Verification: `TB_APP=swift` 5 of 5 on Tinker Desk; `TB_APP=gpui` 5 of 5 on Tinker Desk (real Mac) and on Linux (`cloud`); side-by-side pictures against the Swift pane.

### 2026-11-09 File previews in the pane

- `PanePreview` and its Back stack, the preview host, a picture view with Copy Image
  (the file's own bytes), code in a read-only editor coloured by the kit's syntect
  highlighter (the `syntect` feature on `gpui-kit`; the transcript's code blocks stay
  plain, as the Swift app's are), Markdown drawn as a page, and web pages and `.html`
  files handed to the system browser for now.
- Files come from the runner (`files.read`), with `runnerId` for another machine's
  thread, so a preview never touches the Mac's own disk.
- Files: `desktop/gpui/src/files.rs`, `features/file-previews.feature`.
- Remaining: the in-app web view (`file.htmlpreview`, later work), video and PDF
  (`file.video`, `file.quicklook`), editing and find (`file.editor`, `file.find`),
  the Markdown Preview/Raw menu, drag out of a picture.
- Verification: `TB_APP=swift` 5 of 5 on Tinker Desk and `TB_APP=gpui` 5 of 5 on Tinker Desk (real Mac); side-by-side pictures against the Swift pane.

### 2026-11-09 The pane beside the thread: Highlights

- `desktop/core/src/highlights.rs` ports the logic and protocol types of
  HighlightSections, HighlightFold, HighlightProjects, PreviewRules, FileIcon,
  HighlightText, PanelIcon, ScheduleRows, StartedThreadRows and PatchGroups,
  with the Swift tests. `desktop/gpui/src/highlights.rs` is the pane: one
  self-contained `SidePane` entity beside the thread in a simple split (`new`
  and `render`; the shell will host it as a dock panel), drawing the brief,
  Needs you ticks (the same `threads.tickAsk`, optimistic), project blocks with
  the ⓘ popover (Keep, Take out), Scheduled, Sub-agents, Threads it started,
  Pinned, Files, Links and Changes, with Show all, the folds remembered across
  launches, and the row menus.
- `dump`'s new `panels.pane` says what's drawn on both apps (AppDump.swift,
  `highlights.rs` `dump`); `highlight` joins the automation commands
  (app/README.md).
- Files: `desktop/core/src/highlights.rs`, `desktop/gpui/src/highlights*.rs`,
  `features/highlights.feature`, `features/support/pane.ts`,
  `app/Sources/Threadbear/AppDump.swift`, `Automation+Commands.swift`.
- Remaining: the Notion board (`highlights.board`), the project page link from
  the ⓘ, middle truncation of file names, drag-out, Open in Panel's page view.
- Verification: `TB_APP=swift` 14 of 14 and `TB_APP=gpui` 14 of 14 on Linux
  (`cloud`) and on Tinker Desk; side-by-side picture of the pane against Swift's.

### 2026-11-09 BearDock for GPUI

- New crates: `desktop/dock/layout` (the tree, line for line from Swift, no GPUI)
  and `desktop/dock` (rendering, dragging, drop targets, torn-out windows), with
  a demo.
- Not in the app yet. GPUI can't scale or tilt a live subtree, so the ghost is a
  second copy of the panel. Glides between layouts aren't there. Moving a window
  by hand is Mac only; Windows and X11 use the OS drag, Wayland a menu command.
- Remaining: integrating it into the app, then the glides.
- Verification: `cargo test -p threadbear-dock-layout`, 32 tests; `press` tests
  headless, 4; real CGEvent drags on Tinker Desk through `pointer-test.sh`.

### 2026-11-09 Composer: the open card in the box

- The open card (Needs you or Done) is drawn at the top of the composer from the
  reducer's `open_card()`, and its row in the list is zero height, as the Swift
  one is. Done has Archive and its follow-ups with tick and "?"; Needs you has its
  asks with answer buttons or rows, Something else boxes, Return going on to the
  next unanswered ask, collapse, and one ask at a time when it's tall. Picks stay
  with the composer while another thread is open, and `decide` and `ask` match
  the Swift automation (`dump` has `composer.card` and `decisions` on both apps,
  the Swift one added to AppDump.swift).
- Files: `desktop/gpui/src/composer_card.rs`, `desktop/gpui/src/rows.rs` (one
  line), `features/composer-cards.feature`, `app/Sources/Threadbear/AppDump.swift`.
- Remaining: pictures in the Something else boxes, the page slide, the card
  keeping its tallest page's height, link buttons on asks, the beep when Return
  can't go on, the picks' chips when collapsed, the new-thread page and the
  slash palette.
- Verification: whole suite 44 of 44 on `TB_APP=gpui` on Linux (`cloud`) and a
  real Mac (Tinker Desk), and 44 of 44 on `TB_APP=swift` on Tinker Desk;
  side-by-side screenshot of the paged card against Swift's.

### 2026-11-09 Composer: the chips under the box

- The row under the words: "+" (Attach images…, folder), the model chip with its
  picker (search, rail of pools or providers, rows with default and selection,
  effort), the permission chip, the account chip when there are no pools, and the
  context ring with its popover. Each click updates at once through the
  `update_open` guess. `composer-menu` (permission, model, context) matches the
  Swift text; `desktop/core/src/logins.rs` ports the picker logic with tests.
- Files: `desktop/gpui/src/composer_chips.rs`, `desktop/core/src/logins.rs`,
  `features/composer-chips.feature`.
- Remaining: provider logos on the model rows and chip, usage badges on the
  account and model chips, Older rail details, Schedule… in "+".
- Verification: `TB_APP=swift` on Tinker Desk 4 of 4 for
  `@composer.permission @composer.modelpicker`; `TB_APP=gpui` on Linux (`cloud`)
  4 of 4 and on Tinker Desk 10 of 10 together with the composer's text scenarios;
  screenshot of the chip row against Swift's.

### 2026-11-09 Composer: pictures as inline chips

- Paste, drop or the automation's `paste-image` puts a chip with a thumbnail in the
  words at the cursor, each picture copied in through `media.import`, dimmed
  until it's in; the strip of thumbnails over the words has × on hover; a chip's
  hover shows the picture after a beat; `[Picture N]` markers go with the words.
  These are real gpui-kit inline tokens (`InlineToken` on `TextareaState`, 0.9.0),
  so no fork: atomic in the editing, in undo, with the Swift draft's U+FFFC mark.
  The whole thread is a drop target (`on_drop` and `drag_over` on the pane).
- Files: `desktop/gpui/src/composer.rs`, `features/sending.feature`,
  `features/support/sample.png`, `features/tools/shot.ts` (optional commands).
- Remaining: `composer.chipcopy`, a whole-insert undo step (each chip and its
  spaces are separate undo steps today).
- Verification: `TB_APP=gpui` on Linux (`cloud`) and a real Mac (Tinker Desk), 4 of
  4 for `@composer.picturechips @composer.filedrop @composer.chiphover`;
  `TB_APP=swift` on Tinker Desk 4 of 4; screenshot of two chips against Swift's.

### 2026-11-09 Composer: its own file, the rounded box, list continuation, paste

- The composer is out of `thread.rs` into `desktop/gpui/src/composer.rs`, one
  `Composer` entity per thread on the app (`Threadbear.composers`) so what's typed
  and its undo stay with the thread. Rounded card, 15/23 text, two to ten lines,
  Return sends, Shift-Return makes a new line and carries markdown lists on
  through one edit (`ListContinuation` is in `desktop/core/src/composer.rs` with
  its tests), the send circle and Stop. `paste` joins the automation commands.
- Files: `desktop/gpui/src/composer.rs`, `desktop/core/src/composer.rs`,
  `features/sending.feature`.
- Remaining: pictures, the chips under the box, the cards, the new-thread page.
- Verification: `TB_APP=gpui` on Linux (`cloud`) and a real Mac (Tinker Desk) 7 of
  7 for `@composer.textbox @composer.return @composer.send @composer.listcontinue`;
  `TB_APP=swift` on Tinker Desk 4 of 4 for the new ones; side-by-side screenshot of
  the box against the Swift one.
### 2026-11-09 The transcript's rows, drawn like the Swift app

- What changed: the reading column, Mike's bubble, assistant markdown in
  gpui-kit's TextView styled from Theme.swift's values (`rows/markdown.rs`),
  work lines opening to their detail (`rows/work.rs`), diff rows and the Edited
  N files card (`rows/diff.rs`), every card with its buttons, spinner and
  tooltips (`rows/cards.rs`), images (`rows/images.rs`), notes, gaps, errors,
  footers, the row's right-click menu with Copy message, Copy transcript and
  Copy debug info (`rows/export.rs`), work links scrolling to their step, and
  the list itself: GPUI's `list` with TranscriptController's behaviour
  (`rows/list.rs`): tail following, paging older events in near the top,
  remembered scroll per thread (across a thread being reopened, paging until
  its row is back), not moving when rows open or are added above.
- Files: `desktop/gpui/src/rows.rs`, `desktop/gpui/src/rows/`, the list parts
  of `desktop/gpui/src/thread.rs`, additions to `desktop/gpui/src/style.rs`,
  `automation.rs` (`scroll`, `link`, `menu`, `clipboard`, `expand` with `file`
  and `all`); `features/reading-a-thread.feature`, `features/cards.feature`,
  `features/scrolling.feature` and their steps; `features/tools/rows.ts`.
- Remaining: the bear in the working line (a paw for now; foundation.bear),
  file-path links' search of the files a thread's tools touched and their
  right-click menus (paths in inline code link and open from the thread's
  folder), reply buttons, thread and picture chips, the selection bar, the
  edit pencil and send lift, the table's natural widths (the kit stretches
  them to the column), held-back tool output for other runners' threads, the
  lit-up step's fade, opening a review.
- Verification: `TB_APP=swift cucumber-js` on Tinker Desk's desktop, 41 of 41
  scenarios; `TB_APP=gpui cucumber-js` on Tinker Desk's desktop, 41 of 41, and
  on `cloud` under Xvfb, 41 of 41. `features/tools/long-thread.ts` on Tinker
  Desk (M2), release build, real rows: first page open in 23 ms, 10,000 rows
  loaded and folded in 123 ms, a scrolling frame 0.55 ms median, 1.2 ms worst
  (0.62 and 2.5 ms for fast scrolling). Side-by-side screenshots of every card
  and row kind from both apps.
### 2026-11-09 The sidebar: rows, headings, actions, Hibernate

- Rules ported to `desktop/core/src/sidebar.rs` and `inbox.rs`: `ThreadSections`
  (the Inbox order, and the Mac's Inbox/Running split where the first port had
  followed the website; `InboxUrgency` now checked against its shared cases),
  `ThreadRowLook`, `RowMode`, `ThreadByline`, `ProjectMark`, `RelativeTime` and
  the trailing words, `Snooze`, `WakeWords`, `BackgroundWork`,
  `BusyThreadAction`, `SubagentRows`, `ArchiveRows`.
- `desktop/gpui/src/sidebar.rs` draws them: tile (letter, symbol or picture),
  bold title, place line with the origin mark, blue dot or tick, status and
  time with the ticking timer, headings with counts and folds, the Inbox sort,
  hover buttons that change the row at once, the right-click menu, Hibernate
  presets, Pick a time, Wake when, Rename and Delete, search, New thread, the
  sub-agent tree, archived paging, the loading rows. `icons.rs` maps SF Symbols
  to Lucide.
- `sidebar.archived.foot` dropped: the Swift app undid it on 9 Nov (cfbf3cb3).
- Specs: four new files, 32 scenarios (`sidebar-rows`, `-sections`, `-actions`,
  `-loading`); the Swift `dump` says what a row draws; `desktop/scripts/spec.sh`
  builds and runs them through `remote.sh` with a private copy of the binary.
- Remaining: other runners' rows and Move to, swipe and drag, row motion; Open
  in New Panel is greyed until BearDock; a sub-agent's click opens its thread
  (no sub-agent transcript yet); the open row doesn't hold its place when read;
  folds and sort aren't kept across launches; Pick a time has no script path on
  either app (`@manual`).
- Verification: `TB_APP=swift cucumber-js` on Tinker Desk, 56 of 56 scenarios;
  `TB_APP=gpui cucumber-js` on Tinker Desk 56 of 56 and on `cloud` 56 of 56;
  `cargo test -p threadbear-core`; side-by-side pictures from Tinker Desk.

### 2026-11-09 Risk checks: Chromium in a slot, accessibility, Windows

- Spike (`gpui-web-spike` branch): CEF 156 in a GPUI layout slot on macOS, with a
  gpui-kit menu in its own window over the page.
- Accessibility: Voice Type works; the AX tree below the window doesn't yet.
- Windows: CI ran all 24 scenarios on a real `windows-latest` machine.
- Remaining: Windows and Linux for the web engine, the accessibility tree.
- Verification: spike screenshots from Tinker Desk; `ax-dump` on Tinker Desk;
  `Behaviour specs` in the Desktop app workflow, 24 of 24 on Windows.

### 2026-11-09 The transcript's logic in Rust

- Ported `ThreadbearCore/Transcript` to `desktop/core/src/reducer`, and the
  GPUI thread pane now draws its rows from it (a first pass at how each kind
  looks).
- Golden files: `GoldenTranscriptTests.swift` writes the rows for all 18
  fixtures, closed and open, and both languages check them.
- Remaining: every row's real look, composer card, paging older rows in.
- Verification: `cargo test -p threadbear-core`, 197 tests; specs 24 of 24 on
  Linux (`cloud`) and on a real Mac desktop (Tinker Desk).

### 2026-11-09 Long-thread speed check

- `features/tools/long-thread.ts` on Tinker Desk (M2), release build: a thread
  of 12,500 rows loads and folds in 94 ms, and a frame takes 0.6 ms at the
  median and 2.5 ms at worst while scrolling. A debug build is about ten times
  slower and still fast enough.
- Remaining: paging, and the same check after real row layouts land.
- Verification: the numbers above, run on the machine.

### 2026-11-09 The spec harness

- Added `features/` (Gherkin, Cucumber.js), driving the Swift app through
  `dev-app.sh` and the GPUI app through a debug-only automation socket.
- Files: `features/**`, `desktop/gpui/src/automation.rs`,
  `desktop/scripts/remote.sh`.
- Remaining: the rest of the daily driver.
- Verification: `TB_APP=swift cucumber-js` on Tinker Desk, 24 of 24 scenarios;
  `TB_APP=gpui cucumber-js` on `cloud` under Xvfb, 17 of 24 at this point.
