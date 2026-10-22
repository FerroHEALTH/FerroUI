#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# The module-order guard (no specification governs it: our own design). Every
# top-level `mod name;` declaration of a file, of any visibility and whatever
# blank lines, comments or attributes sit between, is in ascending order of
# its name, so two pull requests that each add a module land on different
# lines. rustfmt sorts only a run of declarations with nothing between them;
# this guard holds the whole file. Inline modules (`mod tests { … }`) are
# not declarations and are not read. Names compare byte by byte (LC_ALL=C).
#
#   mod-order.sh [file.rs ...]    the named files, or every tracked .rs file
#   mod-order.sh --self-test
#
# Exit 0 when every file is in order, 1 naming each file and pair out of
# order.
set -euo pipefail
export LC_ALL=C

# check FILE: prints `FILE:LINE: …` for each declaration that sorts before the
# one above it.
check() {
  awk -v file="$1" '
    match($0, /^(pub(\([a-z:_ ]+\))? )?mod [A-Za-z_][A-Za-z0-9_]*;/) {
      name = $0
      sub(/^(pub(\([a-z:_ ]+\))? )?mod /, "", name)
      sub(/;.*$/, "", name)
      if (seen && name <= last) {
        printf "%s:%d: mod %s comes after mod %s; keep the declarations in alphabetical order\n", file, NR, name, last
      }
      last = name
      seen = 1
    }
  ' "$1"
}

self_test() {
  local work
  work="$(mktemp -d)"
  # Global, so the EXIT trap still sees it after the function returns.
  self_test_work="$work"
  trap 'rm -r -- "$self_test_work"' EXIT
  printf '%s\n' 'pub mod app;' 'mod data;' '' '#[cfg(test)]' 'pub(crate) mod pick_or_type;' \
    '// A comment.' 'mod picker;' 'mod tests {' '    mod zeta;' '    mod alpha;' '}' > "$work/ordered.rs"
  [[ -z "$(check "$work/ordered.rs")" ]] || {
    echo "mod-order: self-test failed: an ordered file was refused:" >&2
    check "$work/ordered.rs" >&2
    exit 1
  }
  printf '%s\n' 'mod widgets;' '' 'mod browser;' > "$work/apart.rs"
  [[ "$(check "$work/apart.rs")" == *"apart.rs:3: mod browser comes after mod widgets"* ]] || {
    echo "mod-order: self-test failed: a declaration below a blank line was not compared." >&2
    exit 1
  }
  printf '%s\n' 'pub mod table;' 'pub mod table;' > "$work/twice.rs"
  [[ -n "$(check "$work/twice.rs")" ]] || {
    echo "mod-order: self-test failed: a module declared twice passed." >&2
    exit 1
  }
  printf '%s\n' 'pub mod view;' 'pub(crate) mod icon;' > "$work/visibility.rs"
  [[ -n "$(check "$work/visibility.rs")" ]] || {
    echo "mod-order: self-test failed: a pub(crate) declaration out of order passed." >&2
    exit 1
  }
  echo "mod-order: self-test passed."
}

if [[ "${1:-}" == "--self-test" ]]; then
  [[ $# -eq 1 ]] || {
    echo "usage: mod-order.sh [file.rs ...] | --self-test" >&2
    exit 2
  }
  self_test
  exit 0
fi

cd "$(dirname "$0")/../.."
files=("$@")
if [[ ${#files[@]} -eq 0 ]]; then
  while IFS= read -r -d '' file; do files+=("$file"); done < <(git ls-files -z '*.rs')
fi
out=""
for file in "${files[@]}"; do
  out+="$(check "$file")"$'\n'
done
out="$(sed '/^$/d' <<<"$out")"
if [[ -n "$out" ]]; then
  printf '%s\n' "$out" >&2
  exit 1
fi
echo "mod-order: the module declarations of ${#files[@]} file(s) are in alphabetical order."
