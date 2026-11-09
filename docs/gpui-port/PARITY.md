# GPUI port: parity

What the GPUI app does compared with the Swift app, worked out from which
behaviour specs pass (`features/`). The table is written by
`node features/parity.mjs` from each machine's report. Don't edit it by hand.

A blank cell means no spec covers that yet, not that the feature is missing.
The full list of features is [features.md](features.md).

## The table

<!-- parity:start -->

21 of 325 features have a spec so far, across 24 scenarios.

Scenarios passing: Swift: 24 of 24, Mac: 24 of 24, Windows: not run, Linux: 24 of 24.


### 0. Foundations everything else needs

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `foundation.rpc` Runner RPC client | pass | pass | pending | pass |

### 3. Sidebar and inbox

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `sidebar.sections` Sections | pass | pass | pending | pass |
| `sidebar.search` Thread search | pass | pass | pending | pass |
| `sidebar.row.marks` Row left marks | pass | pass | pending | pass |
| `sidebar.row.hoverbuttons` Hover buttons | pass | pass | pending | pass |
| `sidebar.hibernate.presets` Hibernate presets | pass | pass | pending | pass |

### 5. Transcript row kinds

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `transcript.user` Your bubble | pass | pass | pending | pass |
| `transcript.reply` Assistant reply | pass | pass | pending | pass |
| `transcript.tool` Tool line | pass | pass | pending | pass |
| `transcript.approval` Approval card | pass | pass | pending | pass |
| `transcript.approval.settled` Settled approval | pass | pass | pending | pass |
| `transcript.question` Question card | pass | pass | pending | pass |
| `transcript.turnfooter` Turn footer | pass | pass | pending | pass |
| `transcript.error` Error row | pass | pass | pending | pass |
| `transcript.workedfor` Worked-for group | pass | pass | pending | pass |
| `transcript.changes` Changes card | pass | pass | pending | pass |

### 7. Composer

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `composer.textbox` Text box | pass | pass | pending | pass |
| `composer.return` Return, Shift-Return | pass | pass | pending | pass |
| `composer.send` Send and Stop | pass | pass | pending | pass |

### 8. Cards in the composer

| Feature | Swift | Mac | Windows | Linux |
|---|---|---|---|---|
| `card.needsyou` Needs you card | pass | pass | pending | pass |
| `card.answers` Answer and follow-up state | pass | pass | pending | pass |

<!-- parity:end -->

## The log

One entry a slice, newest first. Every entry has a `Verification:` line: a spec
run or a screenshot from the running app. A slice with no runtime check doesn't
count. (Ghostex's port log is where this shape comes from; its slices 214 to 268
built parsers and gates with no launch of the app, which is the trap.)

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
