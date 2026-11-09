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
