# Threadbear for Mac, Windows and Linux (GPUI)

One native desktop app, drawn with [GPUI](https://gpui.rs) (Zed's UI
framework) and [gpui-kit](https://gpui-kit.com)'s components. It's growing
to replace the Swift Mac app in `app/`; `docs/gpui-port.md` has the plan and
what's left. The iPhone app stays Swift.

Like the Mac app, it holds no agent state of its own. It talks to a runner
(`runner/`) over the protocol in [`../docs/protocol.md`](../docs/protocol.md).

So far it signs in to the runner, shows the inbox, opens a thread (live, as
it streams), answers a card's asks and approvals, sends messages and stops a
turn.

## How it's put together

- **`client/`, the runner connection** (`runner.rs`): the challenge sign-in
  (`auth.rs`, the same HMAC as `runner/src/rpc-auth.ts`), hello, requests and
  replies, and reconnecting when the runner restarts. `tests/mock_runner.rs`
  signs in to the mock runner.
- **`gpui/`, the app.** GPUI has no tokio, so the connection runs on its own
  runtime and its events cross on a channel (`main.rs`). The inbox rules
  (`inbox.rs`) and transcript folding (`transcript.rs`) are Rust ports of the
  TypeScript ones, with the same test cases, so the two must change together.
- **Which runner.** The one in `THREADBEAR_HOME` (default `~/Library/Application
  Support/Threadbear`, which the runner uses on Linux too), on the port from
  its `runner.json` (default 8797), as `tb` finds it.

## Run it

You need Rust (`rustup`) and Node 24. On a Mac you also need Xcode's command
line tools; on Windows, the C++ build tools. On Linux:

```bash
sudo apt install libasound2-dev libfontconfig-dev libwayland-dev libx11-xcb-dev libxkbcommon-x11-dev libzstd-dev libvulkan-dev
```

On a flaky connection, crates.io downloads need
`CARGO_HTTP_MULTIPLEXING=false CARGO_NET_RETRY=10`.

```bash
npm ci --prefix ../app/Tools/mock-runner   # for the tests and the mock
cargo run -p threadbear-gpui
```

To run it against the mock runner instead of your real one, start
`node ../app/Tools/mock-runner/server.mjs --data-dir /tmp/tb-mock` and run
the app with `THREADBEAR_HOME=/tmp/tb-mock`.

## Tests

```bash
cargo test    # the connection (with the mock) and the app's rules
```

`desktop/test/auth-vector-*.txt` are fixed sign-in proofs. The Rust tests
check them, and so does `runner/test/unit/desktop-auth-vector.test.ts` with
the runner's own code, so if the runner's auth changes both sides fail until
they agree again.

`.github/workflows/desktop.yml` runs it all on Linux, Mac and Windows and
uploads each platform's binary as the run's artifacts.

## Seeing it on a server

`scripts/shot.sh OUT_DIR [--home DIR] [--click X,Y | --type TEXT | --key KEY]...`
runs the debug build (`cargo build -p threadbear-gpui`) on a virtual display
(Xvfb) with Mesa's software Vulkan, against the mock (or the runner in
`--home`), and saves screenshots: `0.png` before any step, then one after
each.

## Known gaps

- On Windows, path handling (`folderName` splits on `/`) and the runner's
  data dir are still Mac-shaped. The runner itself doesn't run on Windows
  yet.
- Opening a thread marks it read, and the sidebar then re-sorts with the open
  thread moving. The Mac app keeps it in place.
