#!/bin/bash
# Checks that remote.sh's copy never leaves cargo on a host thinking a changed
# file is unchanged. Run it by hand after changing how remote.sh copies:
#
#   desktop/scripts/test-remote-freshness.sh <ssh host>
#
# It copies remote.sh into a throwaway checkout holding one tiny crate, so it
# builds in seconds and its folders on the host are its own (and go after a few
# days, like any checkout's). Then it plays what happened on 11 Nov (thread
# 699f31d2): a file's changed here, a build on the host compiles the old copy
# after that change was made (another run still going, or a test script that
# edits the copy and puts it back), and the next run syncs the change. When
# rsync kept this Mac's times, the synced file was older than that build, so
# cargo called the crate fresh and ran the old code: a green run of what you'd
# changed nothing in.
set -euo pipefail
host="${1:?usage: test-remote-freshness.sh <ssh host>}"
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work="$(mktemp -d "${TMPDIR:-/tmp}/remote-freshness.XXXXXX")"
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/desktop/scripts" "$work/runner/scripts" "$work/desktop/src"
cp "$repo/desktop/scripts/remote.sh" "$repo/desktop/scripts/trim-remote-cache.sh" "$work/desktop/scripts/"
cp "$repo/runner/scripts/find-node.sh" "$work/runner/scripts/"
cat >"$work/desktop/Cargo.toml" <<'EOF'
[package]
name = "freshness"
version = "0.1.0"
edition = "2021"
EOF
answer() { printf 'fn main() { println!("answer {}", %s); }\n' "$1" >"$work/desktop/src/main.rs"; }
run() { "$work/desktop/scripts/remote.sh" "$host" -- bash -c "cd desktop && $1"; }
# The copy's folder on the host, as remote.sh names it.
key="$(printf '%s' "$(cd "$work" && pwd)" | shasum | cut -c1-10)"
remote_src=".cache/threadbear-check/gpui/src-$key"

answer 1
run 'cargo run -q' | grep -qx 'answer 1' || { echo "FAIL: the first run didn't build"; exit 1; }

answer 2
sleep 2
# A build on the host after the change was made here, of the copy it still has.
ssh "$host" "cd $remote_src/desktop && touch src/main.rs && PATH=\"\$HOME/.cache/threadbear-check/gpui/cargo/bin:\$HOME/.cargo/bin:\$PATH\" CARGO_TARGET_DIR=\"\$HOME/.cache/threadbear-check/gpui/target-$key\" cargo build -q"

got="$(run 'cargo run -q')"
# And a run with nothing changed shouldn't rebuild anything.
again="$(run 'cargo build 2>&1')"
ssh "$host" "rm -rf .cache/threadbear-check/gpui/src-$key .cache/threadbear-check/gpui/target-$key"

if [ "$got" != "answer 2" ]; then
  echo "FAIL: after the change, the host ran the old build ($got)"
  exit 1
fi
if grep -q Compiling <<<"$again"; then
  echo "FAIL: a run with nothing changed rebuilt:"; echo "$again"
  exit 1
fi
echo "ok: the host built what was synced, and nothing when nothing changed"
