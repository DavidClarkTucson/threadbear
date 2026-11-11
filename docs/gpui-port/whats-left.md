# What's left before the GPUI app can replace the Swift one

The GPUI app's gaps against the Swift app, in plain words, for going through
one by one. It comes from a read of both apps' code on 11 Nov 2026 (the
catalogue's rows are in `features.md`, with the ids in brackets here), less
what's landed since; `PARITY.md` has what's been done, slice by slice, with
how each was checked. Tick a line off, or strike it if it doesn't matter, as
you go.

## Being built now

- [ ] The design review window: answering an agent's design options, stepping
  through every shot, zoom, minimap, the header's "Review design options (N)"
  and `review:` links (`review.*`, `transcript.review`, `header.pendingreviews`)
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
- [ ] Mermaid diagrams and ```html blocks in replies show as code: there's no
  web runtime yet (`rich.mermaid`, `rich.html`, `rich.chrome`, `rich.fullsize`)
- [ ] Web panels (the Convex dashboard and the like) show a placeholder with
  Open in browser (`panel.web`, `panel.convex`, `panel.bar`)
- [ ] No Browser pane, so BrowserBear's handoffs don't show in the app
  (`browser.*`, `rich.browserbear`, `rail.right`)
- [ ] A reply's links all go to the system browser: no "in a panel", Chrome
  profiles or Copy Link on right-click, no hover peek, ⌘-click isn't read
  (`rich.webmenu`, `rich.filemenu`, `rich.linkpeek`, `rich.cmdclick`)

### Reading and copying
- [ ] Selecting text in a reply has no Explain, Quote or Copy bar, and no
  Explain pane (`rich.selectionbar`, `rich.explain`)
- [ ] ⌘C copies plain text only, without the links (`rich.copy`)
- [ ] Tables stretch to the column's width rather than their natural widths,
  and list items sit a little closer than Swift's (`rich.table`, `rich.blocks`)
- [ ] File names in replies don't find the files the thread's tools touched,
  and this Mac's documents and pictures open in their app, not the pane
  (`rich.filerefs`, `file.host`)

### Files and folders in the pane
- [x] No Preview/Markdown switch on a Markdown file (`file.markdown`)
- [x] No find bar (`file.find`)
- [ ] Files are read-only: no editing, autosave or ⌘S (`file.editor`,
  `file.conflict`)
- [ ] PDFs, videos and .html files open in their own app rather than the pane
  (`file.quicklook`, `file.video`, `file.htmlpreview`)
- [ ] A folder doesn't update while it's open, takes one row at a time, has no
  type-to-select, no Space, ⌘↑ or ⌘⇧. keys, no icon view, and the path bar's
  "…" isn't a menu (`folder.*`)

### The composer and the card
- [ ] Copying or cutting picture chips loses the pictures; a pasted thread id
  stays plain text; no card mentions (`composer.chipcopy`, `composer.threadid`,
  `composer.cardmention`)
- [ ] The login picker is one flat list: no room left per login, no provider
  groups (`composer.account`)
- [ ] No "Pinned to … · Unpin" line in the model picker, and no Claude
  breakdown in the context ring (`composer.modelpicker`,
  `composer.contextring`)
- [ ] No right-click on Send to schedule the message
  (`composer.schedulefromsend`)
- [ ] The slash palette can't be resized (`composer.slash.resize`)
- [ ] The card: no lab-review findings on it, no slide between pages, picks
  don't show as chips while it's collapsed, no pictures in a Something else
  box, no beep when Return can't go on (`card.*`)

### Threads and the sidebar
- [ ] No trackpad swipe or drag-left to archive a row, and rows jump between
  sections instead of gliding (`sidebar.swipe`, `sidebar.mousedrag`,
  `sidebar.rowmotion`)
- [ ] No "Open in New Panel" (a thread in a panel of its own), or panels that
  close with their thread (`dock.threadpanel`, `dock.openelsewhere`,
  `dock.contexttabs`)
- [ ] No View > Collapse Threads (⌘/) (`menu.view`)
- [ ] No trackpad swipe back and forward, and no key hints while ⌘ is held
  (`nav.swipe`, `nav.hints`)
- [ ] `threadbear://` links (BrowserBear's pill on an agent's tab) don't open
  this app (`nav.deeplink`)
- [ ] Threads, files and pictures can't be dragged out of the app, and the
  card's Something else box takes no drops (`drag.*`, `highlights.dragout`,
  `file.picture`, `transcript.images`)
- [ ] Another Mac's pictures in a transcript stay blank (`transcript.images`)

### Smaller things
- [ ] The offer to continue a refused turn sits at the top, not just above the
  composer (`banner.refusal`)
- [ ] No milestone toast (`toast.milestone`)
- [ ] Highlights: no Notion board, and a link's menu has no Chrome profiles or
  Default (`highlights.board`, `highlights.rowmenus`)
- [ ] An image panel has no Open Simulator button; `.app`, `.icns` and PDF
  icons fall back to a symbol (`panel.image`, `patches.icons`,
  `rail.originbadge`)
- [ ] Settings > Runners, Devices and Keyboard are placeholders (the Keyboard
  file is picked up live if edited by hand) (`settings.runners`,
  `settings.devices`, `settings.keyboard`)
- [ ] Help's two items, About's window, the tour and Finish Setting Up are
  stubs (`menu.help`, `tour.*`)
- [ ] The bears and the little animations: the working and waking bears, the
  archive and hibernate flights, inbox zero, the message rising from the box,
  the Librarian's wobble (`juice.*`, `foundation.bear`, `transcript.working`,
  `transcript.woke`, `transcript.sendlift`, `librarian.picture`)
- [ ] The system accent colour isn't followed (always blue), and Reduce Motion
  isn't read (`foundation.theme`, `foundation.reducemotion`)
- [ ] The switches are smaller than macOS 27's (the Usage helper's note: one
  fix for Settings, the Librarian page and Usage)

## Before other people move over

- [ ] The app updating itself, and a seamless restart onto a new build
  (`system.selfupdate`, `system.handover`)
- [ ] Crash reports to Sentry, and the hang watch (`system.crash`,
  `system.hangwatch`)
- [ ] The activity log (`system.activitylog`, `foundation.tooltips`)
- [ ] Bringing the Swift app's settings and drafts across on first launch
- [ ] Windows and Linux: notifications, the badge, a menu bar and window
  buttons (`dock.titlebar`, `menu.*`)
- [ ] Accessibility below the window (`foundation.accessibility`)
- [ ] Idle redraws: the sidebar's spinners keep the main thread busy in debug
  builds, and panels drawn inline with the shell redo their layout every
  frame (the Usage helper's note)
