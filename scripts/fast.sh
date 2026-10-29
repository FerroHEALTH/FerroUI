#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# The default local gate: what your change reaches, measured from the merge
# base with origin/main to the working tree (untracked files included) and
# classified by scripts/checks/affected.sh, the table the CI `detect` job
# reads. CI stays the full gate: every lane on a push to main, and every
# lane a pull request's change reaches. No specification governs this: our
# own design (.claude/rules/ci-cd.md).
#
# The steps, each only when the change reaches it:
#   1. cargo fmt --all --check
#   2. clippy -D warnings on the reached packages
#   3. cargo nextest run and the doctests of the reached packages
#   4. the journeys of the touched area, against the gallery served
#      in-process
#   5. design/tools/check.sh for each design set the change reaches
# The journeys and the design checks hold the machine-wide Chrome lock
# (scripts/chrome-lock.sh), so parallel worktrees queue. It never runs the
# screenshot capture.
#
# Usage:
#   scripts/fast.sh [--base <rev> | --change <rev> | --all] [--no-browser] [--dry-run]
#   scripts/fast.sh --self-test
#
#   --base <rev>   measure from this revision instead of the merge base
#   --change <rev> run what the one commit <rev> reaches, as CI's detect
#                  job classified it, over the working tree
#   --all          every step over everything, every journey and design set
#   --no-browser   leave out the journeys and the design checks
#   --dry-run      print the plan and run nothing
#
# A compile cache such as sccache (RUSTC_WRAPPER, or build.rustc-wrapper in
# your own ~/.cargo/config.toml) makes a fresh worktree build much faster;
# the repository does not need one.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

die() {
  echo "fast: $*" >&2
  exit 1
}

usage() {
  sed -n '/^# Usage:/,/^# A compile cache/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//' >&2
  exit 2
}

# The paths changed between BASE and the working tree, untracked included.
changed_paths() {
  {
    git diff --name-only "$1" --
    git ls-files --others --exclude-standard
  } | sort -u
}

self_test() {
  local out
  bash scripts/checks/affected.sh --self-test
  out="$(bash "$0" --base HEAD --dry-run 2>&1)" || die "self-test: a dry run failed: $out"
  # A shallow checkout (CI's) holds no parent to replay a commit against.
  if git rev-parse --verify --quiet HEAD^ > /dev/null; then
    out="$(bash "$0" --change HEAD --dry-run 2>&1)" || die "self-test: a dry run of one commit failed: $out"
  fi
  out="$(bash "$0" --all --dry-run --no-browser 2>&1)"
  case "$out" in *"cargo fmt --all --check"*"cargo clippy"*"cargo nextest run"*) ;; *) die "self-test: --all plans no full Rust tier: $out" ;; esac
  case "$out" in *"journeys"*"skipped"*) ;; *) die "self-test: --no-browser still plans the journeys: $out" ;; esac
  out="$(bash "$0" --all --dry-run 2>&1)"
  case "$out" in *"chrome-lock.sh"*"FERROUI_JOURNEYS=1"*"chrome-lock.sh"*"check.sh"*) ;; *) die "self-test: --all plans no locked journeys and design checks: $out" ;; esac
  case "$out" in *FERROUI_SCREENSHOTS*) die "self-test: the plan runs the screenshot capture" ;; esac
  echo "fast: self-test passed"
}

base=""
change=""
all=false
browser=true
dry=false
while [ $# -gt 0 ]; do
  case "$1" in
    --base)
      [ $# -ge 2 ] || usage
      base="$2"
      shift 2
      ;;
    --change)
      [ $# -ge 2 ] || usage
      change="$2"
      shift 2
      ;;
    --all) all=true; shift ;;
    --no-browser) browser=false; shift ;;
    --dry-run) dry=true; shift ;;
    --self-test) self_test; exit ;;
    *) usage ;;
  esac
done

if [ "$all" = true ]; then
  plan="$(bash scripts/checks/affected.sh --all)"
elif [ -n "$change" ]; then
  plan="$(git diff --name-only "$change^" "$change" -- | bash scripts/checks/affected.sh)"
  echo "fast: the paths $(git rev-parse --short "$change") changes"
else
  if [ -z "$base" ]; then
    base="$(git merge-base origin/main HEAD)" ||
      die "no merge base with origin/main; fetch it or pass --base <rev>"
  fi
  plan="$(changed_paths "$base" | bash scripts/checks/affected.sh)"
fi
get() { printf '%s\n' "$plan" | sed -n "s/^$1=//p"; }
rust="$(get rust)"
packages="$(get packages)"
gallery="$(get gallery)"
journeys="$(get journeys)"
design="$(get design)"
echo "fast: ${base:+from $(git rev-parse --short "$base"), }packages: ${packages:-none}; journeys: ${journeys:-none}; design: ${design:-none}"

failed=()
step() {
  local name="$1"
  shift
  echo "+ $*"
  if [ "$dry" = true ]; then return 0; fi
  local start=$SECONDS
  if "$@"; then
    echo "fast: $name passed in $((SECONDS - start))s"
  else
    echo "fast: $name FAILED after $((SECONDS - start))s" >&2
    failed+=("$name")
  fi
}
skipped() { echo "fast: $1 skipped: $2"; }

# Chrome for Testing a fresh worktree lacks is taken from the main checkout
# when it has it.
main_checkout="$(cd "$(git rev-parse --path-format=absolute --git-common-dir)/.." && pwd)"
tools_from() {
  if [ -d "$root/target/$1" ]; then
    echo "$root/target/$1"
  elif [ -d "$main_checkout/target/$1" ]; then
    echo "$main_checkout/target/$1"
  fi
}

pkg_args=()
for package in $packages; do pkg_args+=(-p "$package"); done

if [ "$rust" = true ]; then
  step rustfmt cargo fmt --all --check
  step clippy cargo clippy ${pkg_args[@]+"${pkg_args[@]}"} --all-targets --locked -- -D warnings
  step nextest cargo nextest run ${pkg_args[@]+"${pkg_args[@]}"} --locked
  step doctests cargo test --doc ${pkg_args[@]+"${pkg_args[@]}"} --locked
else
  skipped "the Rust tier" "the change reaches no Rust input"
fi

if [ "$browser" = false ]; then
  skipped journeys "--no-browser"
elif [ "$gallery" = true ] && [ -n "$journeys" ]; then
  journey_tools="$(tools_from journey-tools)"
  if [ -z "$journey_tools" ]; then
    step "journey tools" bash scripts/release/journey-tools.sh "$root/target/journey-tools"
    journey_tools="$root/target/journey-tools"
  fi
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64)
      export FERROUI_CHROMEDRIVER="${FERROUI_CHROMEDRIVER:-$journey_tools/chromedriver-mac-arm64/chromedriver}"
      export FERROUI_CHROME="${FERROUI_CHROME:-$journey_tools/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing}"
      ;;
    Linux-x86_64)
      export FERROUI_CHROMEDRIVER="${FERROUI_CHROMEDRIVER:-$journey_tools/chromedriver-linux64/chromedriver}"
      export FERROUI_CHROME="${FERROUI_CHROME:-$journey_tools/chrome-linux64/chrome}"
      ;;
  esac
  filter=()
  if [ "$journeys" != all ]; then filter=(-E "package(ferrohealth-ui-gallery) & $journeys"); fi
  step journeys bash scripts/chrome-lock.sh env FERROUI_JOURNEYS=1 cargo nextest run --locked -p ferrohealth-ui-gallery ${filter[@]+"${filter[@]}"}
else
  skipped journeys "the change reaches no journey"
fi

if [ "$browser" = false ]; then
  skipped "design checks" "--no-browser"
elif [ -n "$design" ]; then
  for set in $design; do
    step "design $set" bash scripts/chrome-lock.sh bash design/tools/check.sh "$set"
  done
else
  skipped "design checks" "the change reaches no design set"
fi

if [ "${#failed[@]}" -gt 0 ]; then
  echo "fast: failed: ${failed[*]}" >&2
  exit 1
fi
echo "fast: every step passed"
