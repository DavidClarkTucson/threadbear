# Threadbear for Linux, Windows and Mac (spike)

A cross-platform Threadbear app, built with [Tauri](https://tauri.app): a
Rust shell with the interface written in React. Like the Mac app (`app/`),
it holds no agent state of its own. It talks to a runner (`runner/`) over the
protocol in [`../docs/protocol.md`](../docs/protocol.md).

This is a spike. It signs in to the runner, shows the inbox, opens a thread
(live, as it streams), answers a card's asks and approvals, sends messages
and stops a turn. It doesn't do anything else yet. "Threadbear Rust port" in
Threadbear's projects has the plan and what's left.

## How it's put together

- **`src-tauri/`, the Rust side.** It owns the runner connection
  (`runner.rs`): the challenge sign-in (`auth.rs`, the same HMAC as
  `runner/src/rpc-auth.ts`), hello, requests and replies, and reconnecting
  when the runner restarts. The interface calls `runner_call(method,
  params)` and listens for `runner-status` and `runner-notification`
  events. The token never reaches the web view.
- **`src/`, the interface.** Plain React, no framework. It shares code
  with the rest of the repo rather than copying it: the protocol's types
  come from `runner/src/protocol.ts`, and the inbox's sections from the
  website's `web/src/sections.ts`, so this app, the Mac app, the phone and
  the website agree on what goes where. `transcript.ts` folds a thread's
  events into what's drawn, following the protocol's replacement and
  rewind rules.
- **Which runner.** The one in `THREADBEAR_HOME` (default `~/Library/Application
  Support/Threadbear`, which the runner uses on Linux too), on the port from
  its `runner.json` (default 8797), as `tb` finds it.

## Run it

You need Node 24 and Rust (`rustup`). On Linux you also need WebKitGTK and
friends:

```bash
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev build-essential
```

On a Mac you need Xcode's command line tools; on Windows, the C++ build tools
and WebView2 (already on Windows 10 and 11).

```bash
npm ci
npm ci --prefix ../app/Tools/mock-runner    # for the tests and the mock
npx tauri dev                               # the app, with the interface hot-reloading
THREADBEAR_HOME=/tmp/tb-mock npx tauri dev  # against a mock runner (below)
npx tauri build                             # an installable app in src-tauri/target/release/bundle
```

To run it against the mock runner instead of your real one:
`node ../app/Tools/mock-runner/server.mjs --data-dir /tmp/tb-mock`. In
`tauri dev`, a change under `src/` shows up straight away, and a change to
the Rust side rebuilds and restarts the app.

## Tests

```bash
npm run typecheck && npm test                        # the interface: inbox sections, transcript folding, the auth vector
cargo test --manifest-path src-tauri/Cargo.toml      # the Rust side, including signing in to the mock runner
```

- `test/auth-vector.test.ts` works out a sign-in proof with the runner's own
  code and checks it against `test/auth-vector-*.txt`, which the Rust tests
  check too. If the runner's auth changes, both sides fail until they agree
  again.
- `.github/workflows/desktop.yml` runs all of it on Linux, Mac and Windows,
  and uploads each platform's app (a .deb, a .dmg and a Windows installer) as
  the run's artifacts.

## Seeing it on a server

`scripts/shot.sh OUT_DIR [--home DIR] [--click X,Y | --type TEXT | --key KEY]...` runs the debug
build on a virtual display (Xvfb) against the mock (or the runner in
`--home`) and saves screenshots: `0.png` before any step, then one after
each. Build first with `npx tauri build --debug --no-bundle`. It works
around two things a desktop never needs: WebKitGTK draws nothing on Xvfb
with its GPU paths on, and GTK waits 25 seconds for a session bus a server
doesn't have. Native tooltips draw as black boxes there; that's Xvfb, not
the app.

## Known gaps

- On Windows, path handling (`folderName` splits on `/`) and the runner's
  data dir are still Mac-shaped. The runner itself doesn't run on Windows
  yet.
- Opening a thread marks it read, and the sidebar then re-sorts with the open
  thread moving. The Mac app keeps it in place.
- Reasoning shows in full, sub-agents, pictures, diffs, Highlights, patches,
  panels, projects, schedules, settings, search and the archive aren't there,
  and nothing is signed or auto-updated.
