#!/bin/bash
# Run by remote.sh on the host, from its gpui folder, before it copies or builds
# anything: gives disk back when the host is short, and refuses the run when it
# still is. remote.sh's age limit doesn't help when several checkouts are all in
# use, and these folders filled the cloud box on 9 Nov, which runs a runner too:
# its saves started failing (THREADBEAR-1R).
#
#   trim-remote-cache.sh <this checkout's key>
#
# Exit 3: too little disk left to build on.
set -u
own="${1:?usage: trim-remote-cache.sh <key>}"

# Room for a build from nothing (a debug target has reached 7 GiB) with some to
# spare. Under this, idle checkouts lose their target folders.
TRIM_GIB=12
# Under this a build isn't started at all: it would take the last of the disk
# from whatever else runs on the host.
REFUSE_GIB=3
# A checkout used this recently may be building now, so its folder stays.
BUSY_MINUTES=120

free_gib() {
  local kb
  kb="$(df -Pk . | awk 'NR==2 {print $4}')"
  # A df we can't read says nothing: don't delete or refuse on it.
  case "$kb" in ""|*[!0123456789]*) echo 999999; return ;; esac
  echo $((kb / 1048576))
}

# Removes a target folder if the host is short and it's idle. $1 is what says
# when it was last used, $2 the folder.
evict() {
  [ "$(free_gib)" -lt "$TRIM_GIB" ] || return 0
  [ -d "$2" ] || return 0
  [ -n "$(find "$1" -maxdepth 0 -mmin +"$BUSY_MINUTES" 2>/dev/null)" ] || return 0
  echo "Short of disk on this host, so removing $2, which nothing has used lately. Its checkout rebuilds next time it runs." >&2
  rm -rf "$2"
}

# The one folder every checkout shared before they each got their own.
evict target target
# Oldest first, by the checkout's copy: remote.sh touches that on every run,
# and a target folder's own time doesn't move while cargo works inside it.
# Names are src-<hex>, so the unquoted list is safe.
for s in $(ls -dtr src-* 2>/dev/null); do
  key="${s#src-}"
  [ "$key" = "$own" ] || evict "$s" "target-$key"
done

free="$(free_gib)"
if [ "$free" -lt "$REFUSE_GIB" ]; then
  echo "This host has only $free GiB free, and every build folder left is in use, so this run won't start. Free some space there and run it again." >&2
  exit 3
fi
