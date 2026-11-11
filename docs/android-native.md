# Threadbear for Android (native)

A native Android app, Kotlin and Jetpack Compose, ported from the iPhone app
(`ios/`). It lives in `android-native/`, not `android/`: Keith's fork
(platima/threadbear) has a Tauri wrapper in `android/` and merges our main,
so that name is his.

Like the iPhone app it talks only to the Convex mirror (`convex/phone.ts`,
with a phone token), never to a runner, so it works with the Mac asleep.
Same deliberately small feature set: no search, no sort, no settings beyond
what the iPhone has.

- applicationId `com.mikerosoft.threadbear.android` (Keith's wrapper is
  `com.tinkerware.threadbear`), debug-signed for now.
- Convex: `dev.convex:android-convexmobile`, used at its FFI layer
  (`MobileConvexClient`) so answers stay raw JSON, as the Swift app does.
- Built with `android-native/build.sh` (capped memory, one Gradle worker
  pair, no Kotlin daemon), on the Hetzner box or the M5. On a Mac it takes
  the SDK from ~/Library/Android/sdk and its lock through lockf, and goes
  through `scripts/one-build.sh` like any other build there. Installs and
  screenshots go over `adb connect pixel.tail9ef766.ts.net:5555`. A box
  whose adb key the Pixel hasn't allowed keeps putting "Allow USB
  debugging?" over the app until it's disconnected (`adb disconnect`).

## Layout

- `kit/`: plain Kotlin (JVM), the port of `ios/ThreadbearKit`: the rows
  (lenient JSON reading, one bad row is skipped, not the list), the inbox
  sections and urgency, row status words and relative times, turn folding,
  waiting sends, asks and answers, pairing links, the `Backend` interface
  over a `ConvexTransport`, and a fake transport for tests. `./gradlew
  :kit:test` runs its tests on the JVM, no device.
- `app/`: the Compose app: the Convex FFI transport, the token store
  (EncryptedSharedPreferences), screens, and the view model.

## What's ported from ThreadbearKit

| iPhone (ThreadbearKit) | Android (kit) | Slice |
| --- | --- | --- |
| Models.swift (rows, statuses, approvals, ThreadChange) | `Models.kt` | 1 |
| FlexibleDate.swift | `Dates.kt` | 1 |
| Backend.swift, ConvexTransport.swift (protocol) | `Backend.kt`, `Transport.kt` | 1 |
| Inbox.swift, InboxUrgency.swift (sections, RowStatus, RelativeTime, PreviewText) | `Inbox.kt` | 1 |
| Turns.swift, TurnGap.swift (folding, worked lines, dividers) | `Turns.kt` | 1 |
| Pairing.swift (link parsing) | `Pairing.kt` | 1 |
| WaitingSends.swift | `WaitingSends.kt` | 1 |
| MarkdownBlocks.swift | `Markdown.kt` | 1 (basic), 4 |
| AskAnswer/Answers (Turns.swift, Backend.swift) | `Asks.kt` | 2 |
| Secrets.swift, PairingRequest's key pins | `Secrets.kt` | 2 |
| PushKey.swift, PushTaps.swift | `Push.kt` | 3 |
| MessageHistory.swift (older chunks) | `History.kt` | 4 |
| ReplyButtons, ThreadLinks, SlashCommands, PhotoUpload | | 4 |
| Pocket.swift, PocketIcon.swift | `Pocket.kt` | 5 |
| InboxMoves.swift, Tucks.swift | `InboxMoves.kt`, `Tucks.kt` | 5 |
| ThreadSearch (Inbox.swift) | `Search.kt` | 5 |
| NotifyMode, AppModel's runner order and offline line | `Settings.kt` | 5 |
| AppModel's StartingThread (new threads on their way) | `Starting.kt` | 5 |

Not ported: Quick Ask, dictation/Whisper, the bear and pull-peek, the
Pocket's "here" mark (it needs location and a map search), the rows flying
between headings (RowFlight), the demo's sample data beyond what tests need.

## Screens

1. Pair: Scan the code (camera, ML Kit barcode) and Paste a pairing link,
   both asking "Pair with <server>?" before keeping anything, checked with
   `phone:me`. `threadbear://pair?...` links open the app too.
2. Inbox: Starred, Inbox, Running, Hibernating, Archived headings (the last
   three folded at first), rows like the iPhone's: project tile, title,
   place, status word or time, unread dot. Swipe to archive, long-press for
   Star/Hibernate. Offline/connection line at the top.
3. Thread: the folded transcript, live (`phone:thread` + `phone:messages`),
   the working line while it runs, waiting sends with Cancel, the composer
   with Send turning into Stop while a turn runs, then the cards: approval,
   question, secret, asks, done.
4. New thread (FAB on the inbox): text, runner/folder as the iPhone offers.
5. Settings (the bar's menu): paired server, phone, token expiry, runners
   and their order (drag by the handles), Notify me once push is on, the
   self-hosted note, Unpair with a confirm.
6. Pocket (the wallet button, with its count): the deck, a card full screen,
   swipe a card away with Undo, open its thread, "Add to my pocket".

The inbox's search is always showing, and after a 300 ms pause asks Convex
for archived threads past the newest 100 (`phone:search`). Rows swipe left
to archive and right to hibernate (wake, a starred one that's asleep), and
a long press has Star, Hibernate, Archive, Mark as read and Copy Thread ID.
Archive and hibernate hide the row at once and wait for the runner
(ShellModel), as the iPhone's do.

## Slices

1. Pairing, inbox, open a thread and watch it stream, send, stop.
2. Cards: approvals, questions, secrets (sealed), asks, done card with
   follow-ups and archive.
3. Push: FCM through the relay (needs a Firebase app: Keith's project or a
   second key on the relay, Mike decides), tap opens the thread.
4. The rest: hibernate/star/archive menus, new thread, photos, older
   history, reply buttons, links, markdown tables and code.

Each slice: kit tests for what it ports, screenshots from Mike's Pixel in
dark mode, pushed to `cloud/android-native`, landed from the M5.
