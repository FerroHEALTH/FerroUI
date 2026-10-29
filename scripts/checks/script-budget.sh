#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Script budget guard: the plain ES modules under crates/ferrohealth-ui/scripts/
# stay small and keep to what a console's Content-Security-Policy and
# docs/consoles.md allow (no specification governs the check itself: our own
# design, after VernumBOEK's budget).
#
#   S1  a module over 16 KiB;
#   S2  the modules together over 160 KiB;
#   S3  `eval(` or `new Function`, which `script-src 'self'` refuses;
#   S4  an absolute URL (`http://`, `https://` or a protocol-relative `//`
#       in a string): a module loads and asks only on the page's origin;
#   S5  `fetch(` outside the two modules whose wire docs/consoles.md
#       documents, the search box's typeahead and the session watch.
#
# Comment lines are passed over for S3 to S5.
#
# Usage: scripts/checks/script-budget.sh [DIR] | --self-test
#
# Exit 0 = within the budget, 1 = a finding (listed as rule path: text), 2 =
# a usage error.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
module_max=16384
total_max=163840
fetchers="search.js session.js"

check() {
  local dir="$1" file name size total=0
  for file in "$dir"/*.js; do
    [ -f "$file" ] || continue
    name="$(basename "$file")"
    size="$(wc -c <"$file" | tr -d ' ')"
    total=$((total + size))
    [ "$size" -le "$module_max" ] || echo "S1 $file: $size bytes, over $module_max"
    awk -v name="$name" -v fetchers=" $fetchers " '
      /^[ \t]*\/\// { next }
      /(^|[^A-Za-z0-9_.])eval\(|new Function/ { print "S3 " FILENAME ":" FNR ": " $0 }
      /https?:\/\/|["'\''`]\/\/[A-Za-z0-9]/ { print "S4 " FILENAME ":" FNR ": " $0 }
      /(^|[^A-Za-z0-9_.])fetch\(/ && index(fetchers, " " name " ") == 0 { print "S5 " FILENAME ":" FNR ": " $0 }
    ' "$file"
  done
  [ "$total" -le "$total_max" ] || echo "S2 $dir: $total bytes together, over $total_max"
}

self_test() {
  local work out want
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' RETURN
  head -c 17000 /dev/zero | tr '\0' ' ' >"$work/big.js"
  for want in 1 2 3 4 5 6 7 8 9 10; do
    head -c 15000 /dev/zero | tr '\0' ' ' >"$work/pad$want.js"
  done
  cat >"$work/bad.js" <<'EOF'
eval("1");
const f = new Function("return 1");
import("https://example.org/x.js");
const u = "//example.org/x";
fetch("/answer");
EOF
  cat >"$work/search.js" <<'EOF'
// A comment may name https://example.org and eval(.
fetch(url, { headers: { Accept: "application/json" } });
EOF
  out="$(check "$work")"
  for want in S1 S2 S3 S4 S5; do
    printf '%s\n' "$out" | grep -q "^$want " ||
      { echo "self-test: $want was not reported" >&2; printf '%s\n' "$out" >&2; return 1; }
  done
  if printf '%s\n' "$out" | grep -q 'search\.js'; then
    echo "self-test: the typeahead's fetch or a comment was reported" >&2
    printf '%s\n' "$out" >&2
    return 1
  fi
  echo "script-budget: self-test passed"
}

dir="crates/ferrohealth-ui/scripts"
case "${1:-}" in
  "") ;;
  --self-test)
    self_test
    exit $?
    ;;
  -*)
    echo "usage: scripts/checks/script-budget.sh [DIR] | --self-test" >&2
    exit 2
    ;;
  *) dir="$1" ;;
esac

cd "$root"
[ -d "$dir" ] || { echo "script-budget: $dir is no directory" >&2; exit 2; }
found="$(check "$dir")"
if [ -n "$found" ]; then
  printf '%s\n' "$found" >&2
  echo "script-budget: split or shrink the module, or move the work to the server." >&2
  exit 1
fi
echo "script-budget: every module under $dir within the budget"
