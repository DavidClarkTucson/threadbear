#!/usr/bin/env bash
# Gradle for this project, kept small enough for the 8 GB Hetzner box:
# a memory cap on the whole build, no daemon left behind, one build at a time.
#   ./build.sh :kit:test
#   ./build.sh :app:assembleDebug
set -euo pipefail
cd "$(dirname "$0")"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/android-sdk}"
exec 9>"${TMPDIR:-/tmp}/threadbear-android-build.lock"
if ! flock -n 9; then
  echo "Waiting for another Android build to finish..." >&2
  flock 9
fi
if command -v systemd-run >/dev/null && systemctl --user show-environment >/dev/null 2>&1; then
  exec systemd-run --user --scope -q -p MemoryMax=5.5G -- ./gradlew --no-daemon "$@"
fi
exec ./gradlew --no-daemon "$@"
