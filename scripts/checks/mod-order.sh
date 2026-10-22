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
# The gallery's screenshot driver, `tests/it/states.rs`, is held the same way
# in two more lists: its `AREAS` entries (`area::STATES,`) and the capture
# calls (`area::function(browser, …)`), each in ascending order.
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

# The gallery's screenshot driver, whose area lists check_driver also reads.
driver="app/ferrohealth-ui-gallery/tests/it/states.rs"

# check_driver FILE: prints `FILE:LINE: …` for each `AREAS` entry and each
# capture call that sorts before the one above it.
check_driver() {
  awk -v file="$1" '
    match($0, /^ +[a-z_][a-z0-9_]*::STATES,$/) {
      name = $0
      sub(/^ +/, "", name)
      sub(/,$/, "", name)
      if (areas_seen && name <= areas_last) {
        printf "%s:%d: %s comes after %s; keep the AREAS entries in alphabetical order\n", file, NR, name, areas_last
      }
      areas_last = name
      areas_seen = 1
    }
    match($0, /^ +[a-z_][a-z0-9_]*::[a-z_][a-z0-9_]*\(browser,/) {
      name = $0
      sub(/^ +/, "", name)
      sub(/\(.*$/, "", name)
      if (calls_seen && name <= calls_last) {
        printf "%s:%d: %s comes after %s; keep the capture calls in alphabetical order\n", file, NR, name, calls_last
      }
      calls_last = name
      calls_seen = 1
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
  printf '%s\n' 'const AREAS: &[&[&str]] = &[' '    charts::STATES,' '    tables::STATES,' '];' \
    '    charts::charts(browser, &gallery, brand, theme).await?;' \
    '    tables::compact(browser, &gallery, brand, theme).await?;' \
    '    tables::tables_and_editor(browser, &gallery, brand, theme).await?;' > "$work/driver.rs"
  [[ -z "$(check_driver "$work/driver.rs")" ]] || {
    echo "mod-order: self-test failed: an ordered driver was refused:" >&2
    check_driver "$work/driver.rs" >&2
    exit 1
  }
  printf '%s\n' '    preferences::preferences(browser, &gallery, brand, theme).await?;' \
    '    records::records(browser, &gallery, brand, theme).await?;' \
    '    charts::charts(browser, &gallery, brand, theme).await?;' > "$work/calls.rs"
  [[ "$(check_driver "$work/calls.rs")" == *"calls.rs:3: charts::charts comes after records::records"* ]] || {
    echo "mod-order: self-test failed: a capture call appended out of order passed." >&2
    exit 1
  }
  printf '%s\n' '    records::STATES,' '    charts::STATES,' > "$work/areas.rs"
  [[ "$(check_driver "$work/areas.rs")" == *"areas.rs:2: charts::STATES comes after records::STATES"* ]] || {
    echo "mod-order: self-test failed: an AREAS entry out of order passed." >&2
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
  if [[ "$file" == "$driver" ]]; then
    out+="$(check_driver "$file")"$'\n'
  fi
done
out="$(sed '/^$/d' <<<"$out")"
if [[ -n "$out" ]]; then
  printf '%s\n' "$out" >&2
  exit 1
fi
echo "mod-order: the module declarations of ${#files[@]} file(s) are in alphabetical order."
