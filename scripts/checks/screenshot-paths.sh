#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# The screenshot-path guard (no specification governs it: our own design). A
# feature pull request commits no image under docs/screenshots/: CI uploads
# the capture as an artifact for review, and the refresh pull request (branch
# chore/screenshots, or the `screenshots` label) commits the whole set, so
# two feature pull requests never touch the same image. CI skips this guard
# on the refresh pull request, where the screenshot guard runs instead.
#
#   screenshot-paths.sh <base-ref> [head-ref]
#   screenshot-paths.sh --self-test
#
# The change is read from the merge base of the two, so an image the base
# gained after the fork is never counted as the change's own. Exit 0 when the
# change touches no screenshot, 1 naming each one it touches, 2 on a usage
# error or a base and head with no merge base.
set -euo pipefail

self_test() {
  local script status
  # Global, so the EXIT trap still sees it after the function returns.
  work="$(mktemp -d)"
  # git writes its objects read-only, so the tree is made writable before it
  # is removed.
  trap 'chmod -R u+w "$work" && rm -r -- "$work"' EXIT
  script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  mkdir -p "$work/scripts/checks" "$work/docs/screenshots/states"
  cp "$script" "$work/scripts/checks/screenshot-paths.sh"
  stub_git() {
    git -C "$work" -c user.name=self-test -c user.email=self-test@example.org \
      -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@"
  }
  commit() {
    stub_git add -A
    stub_git commit -q -m "$1"
  }
  branch() {
    stub_git checkout -q main
    stub_git checkout -q -b "$1"
  }
  expect() {
    local name=$1 want=$2 branch=$3
    status=0
    (cd "$work" && bash scripts/checks/screenshot-paths.sh main "$branch") > "$work/out" 2>&1 || status=$?
    if [[ "$status" -ne "$want" ]]; then
      echo "screenshot-paths: self-test failed: $name exited $status, wanted $want." >&2
      cat "$work/out" >&2
      exit 1
    fi
  }

  stub_git init -q -b main
  echo 'png' > "$work/docs/screenshots/index.png"
  echo 'code' > "$work/code.txt"
  commit "the fork point"
  branch code
  echo 'more code' >> "$work/code.txt"
  commit "a change to code alone"
  branch added
  mkdir -p "$work/docs/screenshots/states"
  echo 'png' > "$work/docs/screenshots/states/hover-new.png"
  commit "a change that adds an image"
  branch changed
  echo 'png, again' > "$work/docs/screenshots/index.png"
  commit "a change that redraws an image"
  branch removed
  stub_git rm -q -- docs/screenshots/index.png
  commit "a change that removes an image"
  stub_git checkout -q main
  mkdir -p "$work/docs/screenshots/states"
  echo 'png' > "$work/docs/screenshots/states/hover-main.png"
  commit "main gains an image after every branch forked"

  expect "a change to code alone, behind a main that gained an image" 0 code
  expect "a change that adds an image" 1 added
  grep -q 'docs/screenshots/states/hover-new.png' "$work/out" || {
    echo "screenshot-paths: self-test failed: the refusal did not name the image." >&2
    exit 1
  }
  expect "a change that redraws an image" 1 changed
  expect "a change that removes an image" 1 removed
  echo "screenshot-paths: self-test passed."
}

if [[ "${1:-}" = "--self-test" && $# -eq 1 ]]; then
  self_test
  exit 0
fi

cd "$(dirname "$0")/../.."
if [[ $# -lt 1 || $# -gt 2 || -z "${1:-}" ]]; then
  echo "usage: screenshot-paths.sh <base-ref> [head-ref] | --self-test" >&2
  exit 2
fi
base="$1"
head="${2:-HEAD}"
if ! fork="$(git merge-base "$base" "$head")"; then
  echo "screenshot-paths: $base and $head have no merge base; check out the full history (fetch-depth: 0)." >&2
  exit 2
fi
touched="$(git diff --no-renames --name-only "$fork" "$head" -- docs/screenshots)"
if [[ -n "$touched" ]]; then
  echo "::error::the change touches docs/screenshots/, which only the refresh pull request (branch chore/screenshots) commits: restore each image with 'git checkout origin/main -- <file>' (or remove one the change added). CI uploads the capture as the gallery-screenshots artifact for review." >&2
  while IFS= read -r path; do echo "  $path" >&2; done <<<"$touched"
  exit 1
fi
echo "screenshot-paths: the change touches no screenshot."
