#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# The crate version rule (.claude/rules/crates-publishing.md): the workspace
# version is the library's version and the release tag, so it never moves
# backwards, Cargo.lock carries it for ferrohealth-ui, and CHANGELOG.md has
# a section for it. A change that moves the version is a release change and
# must move all three together (no specification governs this: our own
# design).
#
# Each file is read whole into a variable before anything searches it: a
# reader that stops at its first match (`grep -q`, an awk `exit`) at the end
# of a `git show` pipe would kill the writer with SIGPIPE, which pipefail
# reports as a failure once the file outgrows the pipe buffer (#203).
#
# Usage: scripts/checks/crate-version-guard.sh <base-ref> [head-ref]
#        scripts/checks/crate-version-guard.sh --self-test
set -euo pipefail
cd "$(dirname "$0")/../.."

# Builds a repository whose changelog and lock are each over 64 KB, with the
# matched line near the top, and checks the guard passes on it every time and
# fails when the changelog lacks the section.
self_test() {
  local work script status out i
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' RETURN
  script="$PWD/scripts/checks/crate-version-guard.sh"
  mkdir -p "$work/scripts/checks"
  cp "$script" "$work/scripts/checks/"
  git -C "$work" init -q
  printf '[workspace.package]\nversion = "1.2.3"\n' > "$work/Cargo.toml"
  {
    printf '[[package]]\nname = "ferrohealth-ui"\nversion = "1.2.3"\n'
    for i in $(seq 3000); do printf '\n[[package]]\nname = "padding-%s"\n' "$i"; done
  } > "$work/Cargo.lock"
  {
    printf '# Changelog\n\n## [1.2.3]\n\n'
    for i in $(seq 3000); do printf -- '- An entry that pads the changelog, %s.\n' "$i"; done
  } > "$work/CHANGELOG.md"
  git -C "$work" add -A
  git -C "$work" -c user.name=self-test -c user.email=self-test@example.invalid \
    -c commit.gpgsign=false commit -q -m base
  for i in $(seq 20); do
    if ! out="$(bash "$work/scripts/checks/crate-version-guard.sh" HEAD HEAD 2>&1)"; then
      echo "self-test: run $i failed on a changelog that holds the section: $out" >&2
      return 1
    fi
  done
  sed -i.bak 's/^## \[1\.2\.3\]$/## [1.2.2]/' "$work/CHANGELOG.md"
  rm "$work/CHANGELOG.md.bak"
  git -C "$work" -c user.name=self-test -c user.email=self-test@example.invalid \
    -c commit.gpgsign=false commit -q -am 'no section'
  status=0
  out="$(bash "$work/scripts/checks/crate-version-guard.sh" HEAD~1 HEAD 2>&1)" || status=$?
  if [[ "$status" -ne 1 || "$out" != *"CHANGELOG.md has no section for 1.2.3"* ]]; then
    echo "self-test: a changelog without the section passed (exit $status): $out" >&2
    return 1
  fi
  echo "crate-version-guard: self-test passed"
}

if [[ "${1:-}" == "--self-test" ]]; then
  self_test
  exit $?
fi

base="${1:?usage: crate-version-guard.sh <base-ref> [head-ref]}"
head="${2:-HEAD}"

# The `version` key of `[workspace.package]` in a Cargo.toml read on stdin.
workspace_version() {
  awk '/^\[workspace\.package\]/{p=1; next} /^\[/{p=0}
       p && /^version[[:space:]]*=/{gsub(/.*=[[:space:]]*"|".*/, ""); print; exit}'
}

# The version of the ferrohealth-ui package in a Cargo.lock read on stdin.
locked_version() {
  awk '/^name = "ferrohealth-ui"$/{p=1; next}
       p && /^version = /{gsub(/version = "|"/, ""); print; exit}'
}

manifest="$(git show "$head:Cargo.toml")"
# The base has no manifest before the workspace's first change.
base_manifest="$(git show "$base:Cargo.toml" 2> /dev/null || true)"
lock="$(git show "$head:Cargo.lock")"
changelog="$(git show "$head:CHANGELOG.md")"

want="$(workspace_version <<< "$manifest")"
was="$(workspace_version <<< "$base_manifest")"
locked="$(locked_version <<< "$lock")"

failed=0
if [[ -z "$want" ]]; then
  echo "crate-version-guard: Cargo.toml declares no [workspace.package] version" >&2
  exit 1
fi

if [[ -n "$was" && "$was" != "$want" ]]; then
  newest="$(printf '%s\n%s\n' "$was" "$want" | sort -V | tail -1)"
  if [[ "$newest" != "$want" ]]; then
    echo "crate-version-guard: the version moves backwards, $was to $want" >&2
    failed=1
  fi
fi

if [[ "$locked" != "$want" ]]; then
  echo "crate-version-guard: Cargo.lock holds ferrohealth-ui ${locked:-<none>}, the workspace $want; run cargo update -w" >&2
  failed=1
fi

if [[ "$changelog" != *"## [$want]"* ]]; then
  echo "crate-version-guard: CHANGELOG.md has no section for $want" >&2
  failed=1
fi

[[ "$failed" -eq 0 ]] || exit 1
echo "crate-version-guard: version $want, the lock and the changelog agree"
