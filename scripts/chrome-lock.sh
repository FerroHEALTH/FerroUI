#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Runs a command while holding the machine-wide Chrome lock, so the browser
# steps of several worktrees (the journeys, the design checks, the
# screenshot capture) queue one behind the other instead of starting more
# Chrome processes than the machine holds, which crashes Chrome ("chrome
# closed the pipe"). No specification governs this: our own design.
#
# The lock is a directory under ${TMPDIR:-/tmp}, created with mkdir, which
# is atomic on every POSIX file system; macOS ships no flock binary. The
# directory holds the owner's pid. A lock whose owner is gone, or older
# than FERROUI_CHROME_LOCK_STALE seconds (an hour by default), is taken
# over. The command runs with FERROUI_CHROME_LOCK_HELD=1, so a nested call
# (fast.sh running design/tools/check.sh) runs at once.
#
# Usage:
#   scripts/chrome-lock.sh <command> [args ...]
#   scripts/chrome-lock.sh --self-test
set -euo pipefail

lock="${FERROUI_CHROME_LOCK:-${TMPDIR:-/tmp}/ferroui-chrome.lock}"
lock="${lock%/}"
stale="${FERROUI_CHROME_LOCK_STALE:-3600}"

# Prints the age in seconds of PATH's last modification.
age_of() {
  local now mtime
  now="$(date +%s)"
  mtime="$(stat -f %m "$1" 2> /dev/null || stat -c %Y "$1" 2> /dev/null || echo "$now")"
  echo $((now - mtime))
}

# Takes the lock, waiting while a live owner holds it; takes over a stale one.
acquire() {
  local said=false owner
  while ! mkdir "$lock" 2> /dev/null; do
    owner="$(cat "$lock/pid" 2> /dev/null || true)"
    if [ -n "$owner" ] && ! kill -0 "$owner" 2> /dev/null; then
      echo "chrome-lock: pid $owner left the lock behind; taking it over" >&2
      rm -rf -- "$lock"
      continue
    fi
    if [ "$(age_of "$lock")" -gt "$stale" ]; then
      echo "chrome-lock: the lock is older than ${stale}s; taking it over" >&2
      rm -rf -- "$lock"
      continue
    fi
    if [ "$said" = false ]; then
      echo "chrome-lock: waiting for pid ${owner:-?} to finish its Chrome steps ($lock)" >&2
      said=true
    fi
    sleep 2
  done
  echo "$$" > "$lock/pid"
}

release() {
  if [ "$(cat "$lock/pid" 2> /dev/null || true)" = "$$" ]; then
    rm -rf -- "$lock"
  fi
}

self_test() {
  local dir out
  dir="$(mktemp -d)"
  trap 'rm -rf -- "$dir"' RETURN
  export FERROUI_CHROME_LOCK="$dir/lock"
  unset FERROUI_CHROME_LOCK_HELD

  out="$(bash "$0" sh -c 'echo "held=$FERROUI_CHROME_LOCK_HELD"; cat "$FERROUI_CHROME_LOCK/pid" > /dev/null && echo locked')"
  [ "$out" = "held=1"$'\n'"locked" ] || { echo "self-test: the command did not run under the lock: $out" >&2; return 1; }
  [ ! -e "$dir/lock" ] || { echo "self-test: the lock outlived its command" >&2; return 1; }

  bash "$0" sh -c 'exit 3' && { echo "self-test: a failing command passed" >&2; return 1; }
  [ ! -e "$dir/lock" ] || { echo "self-test: a failing command left the lock" >&2; return 1; }

  mkdir "$dir/lock"
  sh -c 'exit 0' &
  local gone=$!
  wait "$gone"
  echo "$gone" > "$dir/lock/pid"
  out="$(bash "$0" echo ran 2> /dev/null)"
  [ "$out" = ran ] || { echo "self-test: a lock left by a dead pid was not taken over" >&2; return 1; }

  mkdir "$dir/lock"
  echo "$$" > "$dir/lock/pid"
  out="$(FERROUI_CHROME_LOCK_STALE=-1 bash "$0" echo ran 2> /dev/null)"
  [ "$out" = ran ] || { echo "self-test: a stale lock was not taken over" >&2; return 1; }

  mkdir "$dir/lock"
  echo "$$" > "$dir/lock/pid"
  bash "$0" touch "$dir/second" 2> /dev/null &
  local waiter=$!
  sleep 3
  [ ! -e "$dir/second" ] || { echo "self-test: a second command ran while the lock was held" >&2; return 1; }
  rm -rf -- "$dir/lock"
  wait "$waiter"
  [ -e "$dir/second" ] || { echo "self-test: the waiting command never ran" >&2; return 1; }

  out="$(FERROUI_CHROME_LOCK_HELD=1 bash "$0" echo nested)"
  [ "$out" = nested ] || { echo "self-test: a nested call did not run at once" >&2; return 1; }
  echo "chrome-lock: self-test passed"
}

case "${1:-}" in
  "")
    sed -n '/^# Usage:/,/^set /p' "$0" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
    ;;
  --self-test)
    self_test
    exit
    ;;
esac

if [ "${FERROUI_CHROME_LOCK_HELD:-}" = 1 ]; then
  exec "$@"
fi
acquire
trap release EXIT
trap 'exit 130' INT TERM
FERROUI_CHROME_LOCK_HELD=1 "$@"
