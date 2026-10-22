#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# The changelog guard (no specification governs it: our own design). A pull
# request records its effect as a fragment under changelog.d/ and never edits
# the [Unreleased] section of CHANGELOG.md, so two pull requests never touch
# the same lines. It passes when the change adds a fragment (any file there
# but the README) and leaves [Unreleased] as it found it. The release cut,
# which adds a version section to CHANGELOG.md, passes as it is.
# scripts/release/changelog.sh --check judges a fragment's form; this guard
# asks only whether there is one.
#
#   changelog-guard.sh [--no-changelog] <base-ref> [head-ref]
#   changelog-guard.sh --self-test
#
# --no-changelog is passed when the pull request carries the `no-changelog`
# label: no fragment is asked for, and [Unreleased] is still held. The change
# is what the head added since it forked from the base, read from the diff
# against the merge base of the two, so a fragment that reached the base
# after the fork is never counted as the change's own.
#
# Exit 0 when the change passes, 1 when it does not, and 2 on a usage error
# or a base and head with no merge base.
set -euo pipefail

# unreleased REV: the body of the [Unreleased] section of CHANGELOG.md at REV,
# from its heading to the next version heading or link reference.
unreleased() {
  git show "$1:CHANGELOG.md" 2> /dev/null | awk '
    /^## \[Unreleased\][[:space:]]*$/ { on = 1; next }
    on && (/^## \[/ || /^\[[^]]+\]: /) { exit }
    on { print }
  '
}

# versions REV: the version headings of CHANGELOG.md at REV, one per line.
versions() {
  git show "$1:CHANGELOG.md" 2> /dev/null | grep -E '^## \[[0-9]' | sort || true
}

# The self-test builds a stub repository in a temporary directory, with a copy
# of this script in it, and runs the guard there as CI does: the base is
# main's tip and the head is the branch.
self_test() {
  local script status
  # Global, so the EXIT trap still sees it after the function returns.
  work="$(mktemp -d)"
  # git writes its objects read-only, so the tree is made writable before it
  # is removed.
  trap 'chmod -R u+w "$work" && rm -r -- "$work"' EXIT
  script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  mkdir -p "$work/scripts/checks" "$work/changelog.d"
  cp "$script" "$work/scripts/checks/changelog-guard.sh"

  # A repository of its own, so no global hook, signing key or identity of the
  # caller's reaches it.
  stub_git() {
    git -C "$work" -c user.name=self-test -c user.email=self-test@example.org \
      -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@"
  }
  commit() {
    local message="$1"
    stub_git add -A
    stub_git commit -q -m "$message"
  }
  branch() {
    local name="$1"
    stub_git checkout -q main
    stub_git checkout -q -b "$name"
  }
  # expect NAME WANT BRANCH [FLAG]: the guard over BRANCH against main exits WANT.
  expect() {
    local name=$1 want=$2 branch=$3
    shift 3
    status=0
    (cd "$work" && bash scripts/checks/changelog-guard.sh "$@" main "$branch") > "$work/out" 2>&1 || status=$?
    if [[ "$status" -ne "$want" ]]; then
      echo "changelog-guard: self-test failed: $name exited $status, wanted $want." >&2
      cat "$work/out" >&2
      exit 1
    fi
  }
  says() {
    local name=$1 text=$2
    grep -q -- "$text" "$work/out" || {
      echo "changelog-guard: self-test failed: $name did not say '$text'." >&2
      cat "$work/out" >&2
      exit 1
    }
  }

  stub_git init -q -b main
  printf '%s\n' '# Changelog' '' '## [Unreleased]' '' '### Added' '' '- An entry from before the fragments.' '' \
    '## [0.0.1] - 2026-10-01' '' '### Added' '' '- The first release.' '' \
    '[Unreleased]: https://example.org/r/compare/v0.0.1...HEAD' > "$work/CHANGELOG.md"
  printf 'The fragment format.\n' > "$work/changelog.d/README.md"
  printf -- '- An entry already released.\n' > "$work/changelog.d/1-old.added.md"
  echo 'code' > "$work/code.txt"
  commit "the fork point"

  branch fragment
  echo 'more code' >> "$work/code.txt"
  printf -- '- An entry.\n' > "$work/changelog.d/2-new.added.md"
  commit "a change with a fragment"
  branch edited
  echo 'more code' >> "$work/code.txt"
  sed -i.bak 's/^- An entry from before the fragments\.$/&\n- An entry written in place./' "$work/CHANGELOG.md"
  rm "$work/CHANGELOG.md.bak"
  commit "a change that edits [Unreleased]"
  branch both
  printf -- '- An entry.\n' > "$work/changelog.d/2-new.added.md"
  sed -i.bak 's/^- An entry from before the fragments\.$/- An entry, reworded./' "$work/CHANGELOG.md"
  rm "$work/CHANGELOG.md.bak"
  commit "a change with a fragment that also edits [Unreleased]"
  branch released
  printf -- '- A fragment.\n' > "$work/changelog.d/3-typo.fixed.md"
  sed -i.bak 's/^- The first release\.$/- The first release, spelled right./' "$work/CHANGELOG.md"
  rm "$work/CHANGELOG.md.bak"
  commit "a change that fixes a released section"
  branch cut
  printf '%s\n' '# Changelog' '' '## [Unreleased]' '' '## [0.0.2] - 2026-10-10' '' '### Added' '' \
    '- An entry from before the fragments.' '' '## [0.0.1] - 2026-10-01' '' '### Added' '' '- The first release.' '' \
    '[Unreleased]: https://example.org/r/compare/v0.0.2...HEAD' > "$work/CHANGELOG.md"
  stub_git rm -q -- changelog.d/1-old.added.md
  commit "the release cut"
  branch none
  echo 'more code' >> "$work/code.txt"
  commit "a change with no entry"
  branch readme
  echo 'A clearer note.' >> "$work/changelog.d/README.md"
  commit "a change to the README alone"
  branch reworded
  printf -- '- An entry, reworded.\n' > "$work/changelog.d/1-old.added.md"
  commit "a change that rewords a fragment it did not add"
  branch removed
  stub_git rm -q -- changelog.d/1-old.added.md
  commit "a change that removes a fragment"

  stub_git checkout -q main
  printf -- '- An entry from main.\n' > "$work/changelog.d/4-main.fixed.md"
  commit "main gains a fragment after every branch forked"

  expect "a change that adds a fragment" 0 fragment
  expect "a change that edits [Unreleased]" 1 edited
  says "a change that edits [Unreleased]" "edits the \[Unreleased\] section"
  expect "a change that edits [Unreleased] under no-changelog" 1 edited --no-changelog
  expect "a change with a fragment that also edits [Unreleased]" 1 both
  expect "a change that fixes a released section beside its fragment" 0 released
  expect "the release cut" 0 cut
  expect "a change with no entry, behind a main that gained a fragment" 1 none
  says "a change with no entry" "no changelog fragment"
  expect "a change with no entry under no-changelog" 0 none --no-changelog
  expect "a change to the README alone" 1 readme
  expect "a change that rewords a fragment it did not add" 1 reworded
  expect "a change that removes a fragment" 1 removed
  echo "changelog-guard: self-test passed."
}

if [[ "${1:-}" = "--self-test" && $# -eq 1 ]]; then
  self_test
  exit 0
fi

cd "$(dirname "$0")/../.."

need_fragment=1
if [[ "${1:-}" = "--no-changelog" ]]; then
  need_fragment=0
  shift
fi
if [[ $# -lt 1 || $# -gt 2 || -z "${1:-}" ]]; then
  echo "usage: changelog-guard.sh [--no-changelog] <base-ref> [head-ref] | --self-test" >&2
  exit 2
fi
base="$1"
head="${2:-HEAD}"

if ! fork="$(git merge-base "$base" "$head")"; then
  echo "changelog-guard: $base and $head have no merge base; check out the full history (fetch-depth: 0)." >&2
  exit 2
fi

fail=0
if [[ "$(versions "$fork")" != "$(versions "$head")" ]]; then
  echo "changelog-guard: the change adds a version section to CHANGELOG.md: the release cut assembles the fragments and empties [Unreleased]."
  exit 0
fi
if [[ "$(unreleased "$fork")" != "$(unreleased "$head")" ]]; then
  echo "::error::the change edits the [Unreleased] section of CHANGELOG.md: write the entry as a fragment, changelog.d/<issue>-<kebab-slug>.<section>.md, as changelog.d/README.md describes, and leave CHANGELOG.md to the release cut." >&2
  fail=1
fi

if [[ "$need_fragment" -eq 1 ]]; then
  # A rename is read as a removal and an addition, so a fragment renamed into
  # place counts as added and one renamed away does not.
  added="$(git diff --no-renames --name-only --diff-filter=A "$fork" "$head" -- 'changelog.d/*' ':!changelog.d/README.md')"
  if [[ -n "$added" ]]; then
    echo "changelog-guard: the change adds the fragment(s):"
    while IFS= read -r path; do echo "  $path"; done <<<"$added"
  else
    echo "::error::no changelog fragment: add changelog.d/<issue>-<kebab-slug>.<section>.md, as changelog.d/README.md describes, or apply the 'no-changelog' label when no console sees the change." >&2
    fail=1
  fi
fi
exit "$fail"
