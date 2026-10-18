#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Vendors the Lucide icon outlines FerroUI draws, verbatim, from the pinned
# Lucide release, with the release's licence, into
# crates/ferrohealth-ui/vendor/lucide/ (no specification governs this: our own
# design; .claude/rules/vendored-inputs.md).
#
# Each icon is the release's own `icons/<name>.svg`, unchanged. The glyph
# table of crates/ferrohealth-ui/src/icon.rs carries each outline's inner markup, and
# its test holds every entry to the vendored file, so a glyph never drifts
# from its source. Add a name to ICONS and to the glyph table together.
#
# Usage:
#   scripts/vendor/lucide.sh          fetch every icon and the licence
#   scripts/vendor/lucide.sh --check  fail unless every icon of ICONS is
#                                     vendored and no other file is
set -Eeuo pipefail

version="1.51.0"
base="https://raw.githubusercontent.com/lucide-icons/lucide/${version}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
dir="$root/crates/ferrohealth-ui/vendor/lucide"

ICONS=(
  arrow-down arrow-left arrow-right arrow-up arrow-up-down bell check
  chevron-down chevron-left chevron-right chevrons-left chevrons-right
  circle-alert circle-check circle-user circle-x compass copy database
  download file-plus files flask-conical heart-pulse inbox info layers
  layout-dashboard list-checks log-in log-out menu monitor moon pencil play
  plus refresh-cw save search server server-crash settings shield-check
  square-terminal sun trash triangle-alert users x
)

die() {
  echo "lucide: $*" >&2
  exit 1
}

check() {
  local name missing=0
  for name in "${ICONS[@]}"; do
    if [ ! -s "$dir/icons/$name.svg" ]; then
      echo "lucide: $name.svg is not vendored" >&2
      missing=1
    fi
  done
  local file
  for file in "$dir"/icons/*.svg; do
    name="$(basename "$file" .svg)"
    if ! printf '%s\n' "${ICONS[@]}" | grep -qx -- "$name"; then
      echo "lucide: $name.svg is vendored but not listed" >&2
      missing=1
    fi
  done
  [ -s "$dir/LICENSE" ] || {
    echo "lucide: the licence is not vendored" >&2
    missing=1
  }
  [ "$missing" -eq 0 ] || exit 1
  echo "lucide: ${#ICONS[@]} icons of Lucide ${version} are vendored"
}

case "${1:-}" in
  --check)
    check
    exit 0
    ;;
  "") ;;
  *)
    echo "usage: scripts/vendor/lucide.sh [--check]" >&2
    exit 2
    ;;
esac

command -v curl > /dev/null || die "curl is not installed"
mkdir -p "$dir/icons"
curl --fail --silent --show-error --location --retry 3 \
  --output "$dir/LICENSE" "$base/LICENSE"
for name in "${ICONS[@]}"; do
  curl --fail --silent --show-error --location --retry 3 \
    --output "$dir/icons/$name.svg" "$base/icons/$name.svg"
done
check
