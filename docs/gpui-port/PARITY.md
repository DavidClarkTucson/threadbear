# GPUI port: parity

What the GPUI app does compared with the Swift app, worked out from which
behaviour specs pass (`features/`). The table is written by
`node features/parity.mjs` from each machine's report. Don't edit it by hand.

A blank cell means no spec covers that yet, not that the feature is missing.
The full list of features is [features.md](features.md).

## The table

<!-- parity:start -->

21 of 325 features have a spec so far, across 24 scenarios.

Scenarios passing: Swift: 24 of 24, Mac: 24 of 24, Windows: 24 of 24, Linux: 24 of 24.


### 0. Foundations everything else needs

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `foundation.rpc` Runner RPC client | pass | pass | pass | pass |

### 3. Sidebar and inbox

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `sidebar.sections` Sections | pass | pass | pass | pass |
| `sidebar.search` Thread search | pass | pass | pass | pass |
| `sidebar.row.marks` Row left marks | pass | pass | pass | pass |
| `sidebar.row.hoverbuttons` Hover buttons | pass | pass | pass | pass |
| `sidebar.hibernate.presets` Hibernate presets | pass | pass | pass | pass |

### 5. Transcript row kinds

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `transcript.user` Your bubble | pass | pass | pass | pass |
| `transcript.reply` Assistant reply | pass | pass | pass | pass |
| `transcript.tool` Tool line | pass | pass | pass | pass |
| `transcript.approval` Approval card | pass | pass | pass | pass |
| `transcript.approval.settled` Settled approval | pass | pass | pass | pass |
| `transcript.question` Question card | pass | pass | pass | pass |
| `transcript.turnfooter` Turn footer | pass | pass | pass | pass |
| `transcript.error` Error row | pass | pass | pass | pass |
| `transcript.workedfor` Worked-for group | pass | pass | pass | pass |
| `transcript.changes` Changes card | pass | pass | pass | pass |

### 7. Composer

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `composer.textbox` Text box | pass | pass | pass | pass |
| `composer.return` Return, Shift-Return | pass | pass | pass | pass |
| `composer.send` Send and Stop | pass | pass | pass | pass |

### 8. Cards in the composer

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `card.needsyou` Needs you card | pass | pass | pass | pass |
| `card.answers` Answer and follow-up state | pass | pass | pass | pass |

<!-- parity:end -->

## The log

One entry a slice, newest first. Every entry has a `Verification:` line: a spec
run or a screenshot from the running app. A slice with no runtime check doesn't
count. (Ghostex's port log is where this shape comes from; its slices 214 to 268
built parsers and gates with no launch of the app, which is the trap.)

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
- Specs: four new files, 31 scenarios (`sidebar-rows`, `-sections`, `-actions`,
  `-loading`); the Swift `dump` says what a row draws; `desktop/scripts/spec.sh`
  builds and runs them through `remote.sh` with a private copy of the binary.
- Remaining: other runners' rows and Move to, swipe and drag, row motion; Open
  in New Panel is greyed until BearDock; a sub-agent's click opens its thread
  (no sub-agent transcript yet); the open row doesn't hold its place when read;
  folds and sort aren't kept across launches; Pick a time has no script path on
  either app (`@manual`).
- Verification: `TB_APP=swift cucumber-js` on Tinker Desk, 55 of 55 scenarios;
  `TB_APP=gpui cucumber-js` on Tinker Desk 55 of 55 and on `cloud` 55 of 55;
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
