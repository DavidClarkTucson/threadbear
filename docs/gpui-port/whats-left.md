# What's left before the GPUI app can replace the Swift one

The GPUI app's gaps against the Swift app, in plain words, for going through
one by one. It comes from a read of both apps' code on 11 Nov 2026 (the
catalogue's rows are in `features.md`, with the ids in brackets here), less
what's landed since; `PARITY.md` has what's been done, slice by slice, with
how each was checked. Tick a line off, or strike it if it doesn't matter, as
you go.

## Being built now

- [x] The design review window: answering an agent's design options, stepping
  through every shot, zoom, minimap, the header's "Review design options (N)"
  and `review:` links (`review.*`, `transcript.review`, `header.pendingreviews`)
- [ ] The review window plays clips and shows pages in place, as the Swift one
  does, rather than stills that open in their apps (`review.media`)
- [x] Projects: the Projects page and sidebar view, a project's page, its
  decisions, Projects ▸ in the row and ... menus, the header's project tiles
  opening the page, New Thread in a project (`projects.*`, `header.projectchip`)
- [x] The Terminal: ⌘J with a shell per thread, Run on an ask with a command,
  the play icon on a reply's shell block (`terminal.*`, `cards.run`)
- [x] Slack, email and draft boxes in replies with Copy, and chips in your
  bubbles: pictures, thread ids and card mentions (`rich.draft`,
  `rich.*chips`, `transcript.user`, `transcript.agentmessage`)

## You'd notice these in a day of use

### Web pages in the app
- [x] Mermaid diagrams and ```html blocks in replies show as code: there's no
  web runtime yet (`rich.mermaid`, `rich.html`, `rich.chrome`, `rich.fullsize`;
  drawn through the web runtime on a Mac since 11 Nov)
- [x] Web panels (the Convex dashboard and the like) show a placeholder with
  Open in browser (`panel.web`, `panel.convex`, `panel.bar`; pages through the
  web runtime on a Mac since 11 Nov, with Open in browser until it's installed)
- [ ] No Browser pane, so BrowserBear's handoffs don't show in the app
  (`browser.*`, `rich.browserbear`, `rail.right`)
- [x] A reply's links all go to the system browser: no "in a panel", Chrome
  profiles or Copy Link on right-click, no hover peek, ⌘-click isn't read
  (`rich.webmenu`, `rich.filemenu`, `rich.linkpeek`, `rich.cmdclick`).
  A page "in the panel" still opens in the browser, until the pane draws
  pages through the web runtime.

### Reading and copying
- [x] Selecting text in a reply has no Explain, Quote or Copy bar, and no
  Explain pane (`rich.selectionbar`, `rich.explain`)
- [x] ⌘C copies plain text only, without the links (`rich.copy`)
- [ ] Tables stretch to the column's width rather than their natural widths,
  and list items sit a little closer than Swift's (`rich.table`, `rich.blocks`)
- [x] File names in replies don't find the files the thread's tools touched,
  and this Mac's documents and pictures open in their app, not the pane
  (`rich.filerefs`, `file.host`)

### Files and folders in the pane
- [x] No Preview/Markdown switch on a Markdown file (`file.markdown`)
- [x] No find bar (`file.find`)
- [x] Files are read-only: no editing, autosave or ⌘S (`file.editor`,
  `file.conflict`)
- [ ] PDFs, videos and .html files open in their own app rather than the pane
  (`file.quicklook`, `file.video`, `file.htmlpreview`)
- [x] A folder doesn't update while it's open, takes one row at a time, has no
  type-to-select, no Space, ⌘↑ or ⌘⇧. keys, no icon view, and the path bar's
  "…" isn't a menu (`folder.*`)

### The composer and the card
- [x] Copying or cutting picture chips loses the pictures; a pasted thread id
  stays plain text; no card mentions (`composer.chipcopy`, `composer.threadid`,
  `composer.cardmention`). In since 11 Nov, but for a click on a card on the
  Highlights' board, which waits for the board, and a thread dragged from the
  sidebar, whose drop the box takes once the rows start one (`drag.threadout`)
- [x] The login picker is one flat list: no room left per login, no provider
  groups (`composer.account`). In since 11 Nov
- [x] No "Pinned to … · Unpin" line in the model picker, and no Claude
  breakdown in the context ring (`composer.modelpicker`,
  `composer.contextring`). In since 11 Nov
- [x] No right-click on Send to schedule the message
  (`composer.schedulefromsend`). In since 11 Nov
- [x] The slash palette can't be resized (`composer.slash.resize`). In since
  11 Nov
- [ ] In a narrow panel (a thread in a panel of its own, a narrow window) the
  chips under the box run past its edge, where the Swift app's shrink their
  labels to fit (`composer.modelpicker`, `composer.permission`)
- [x] The card: no lab-review findings on it, no slide between pages, picks
  don't show as chips while it's collapsed, no pictures in a Something else
  box, no beep when Return can't go on (`card.*`). In since 11 Nov; Reduce
  Motion's crossfade in place of the slide waits for the app to read it
  (`foundation.reducemotion`), and the beep is the Mac's only

### Threads and the sidebar
- [ ] No trackpad swipe or drag-left to archive a row (`sidebar.swipe`,
  `sidebar.mousedrag`). Rows glide between sections and counts roll now
  (`sidebar.rowmotion`)
- [x] No "Open in New Panel" (a thread in a panel of its own), or panels that
  close with their thread (`dock.threadpanel`, `dock.openelsewhere`,
  `dock.contexttabs`)
- [x] No View > Collapse Threads (⌘/) (`menu.view`)
- [ ] No trackpad swipe back and forward (`nav.swipe`). The keys show while
  ⌘ is held now (`nav.hints`)
- [x] `threadbear://` links (BrowserBear's pill on an agent's tab) don't open
  this app (`nav.deeplink`). They open the GPUI app once install-app.sh has
  run; `desktop/scripts/thread-links.sh swift` gives them back
- [ ] Threads, files and pictures can't be dragged out of the app, and the
  card's Something else box takes no drops (`drag.*`, `highlights.dragout`,
  `file.picture`, `transcript.images`)
- [ ] Another Mac's pictures in a transcript stay blank (`transcript.images`)

### Smaller things
- [ ] The offer to continue a refused turn sits at the top, not just above the
  composer (`banner.refusal`)
- [ ] No milestone toast (`toast.milestone`)
- [ ] Highlights: no Notion board (`highlights.board`)
- [ ] An image panel has no Open Simulator button; `.app`, `.icns` and PDF
  icons fall back to a symbol (`panel.image`, `patches.icons`,
  `rail.originbadge`)
- [x] Settings > Runners, Devices and Keyboard are placeholders (the Keyboard
  file is picked up live if edited by hand) (`settings.runners`,
  `settings.devices`, `settings.keyboard`). In since 11 Nov, but for
  Runners' Connect to other machines directly, as this app has no direct
  link yet
- [ ] Connect to other machines directly: other runners' threads only come
  through Convex here (`settings.runners`, `remote.rs`)
- [ ] Help's two items, About's window, the tour and Finish Setting Up are
  stubs (`menu.help`, `tour.*`)
- [ ] The bears and the little animations: the working and waking bears, the
  archive and hibernate flights, inbox zero, the message rising from the box,
  the Librarian's wobble (`juice.*`, `foundation.bear`, `transcript.working`,
  `transcript.woke`, `transcript.sendlift`, `librarian.picture`)
- [ ] The system accent colour isn't followed (always blue), and Reduce Motion
  isn't read (`foundation.theme`, `foundation.reducemotion`)
- [ ] The switches are smaller than macOS 27's (the Usage helper's note: one
  fix for Settings, the Librarian page and Usage). Settings' are macOS's now
  (`mac_switch.rs`: 36 by 16 with a pill knob, as measured in the Swift
  Settings window); the Librarian page and Usage still have the kit's, and
  want measuring there first (the Usage helper saw 44 by 20)

## Before other people move over

- [x] The app updating itself (`system.selfupdate`)
- [ ] A seamless restart onto a new build (`system.handover`): the window
  goes and comes back for now, so the app only restarts by itself while
  it isn't in front
- [x] Crash reports to Sentry, and the hang watch (`system.crash`,
  `system.hangwatch`)
- [x] The activity log (`system.activitylog`, `foundation.tooltips`)
- [x] Bringing the Swift app's settings and drafts across on first launch
  (`system.swiftimport`)
- [ ] Windows and Linux: notifications, the badge, a menu bar and window
  buttons (`dock.titlebar`, `menu.*`)
- [ ] Accessibility below the window (`foundation.accessibility`)
- [ ] Idle redraws: the sidebar's spinners keep the main thread busy in debug
  builds, and panels drawn inline with the shell redo their layout every
  frame (the Usage helper's note)
