#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Script budget guard: the plain ES modules under crates/ferrohealth-ui/scripts/
# stay small and keep to what a console's Content-Security-Policy and
# docs/consoles.md allow (no specification governs the check itself: our own
# design, after VernumBOEK's budget; the caps and their reasons are
# .claude/rules/console-ui.md §12).
#
#   S1  a module over 16 KiB;
#   S2  the modules one page loads over 144 KiB: the loader, every job the
#       page's line in the page list names, and every module those import,
#       statically or by `import("./x.js")`;
#   S3  `eval(` or `new Function`, which `script-src 'self'` refuses;
#   S4  an absolute URL (`http://`, `https://` or a protocol-relative `//`
#       in a string): a module loads and asks only on the page's origin;
#   S5  `fetch(` outside the two modules whose wire docs/consoles.md
#       documents, the search box's typeahead and the session watch;
#   S6  every module, each compressed by `gzip -9`, over 72 KiB together;
#   S7  a job in the page list with no module, or a list with no page.
#
# The page list (scripts/checks/script-pages.list) holds one line per
# gallery page, its path and its jobs; the gallery's script_pages test fails
# when the list is no longer what the gallery draws. Comment lines are
# passed over for S3 to S5.
#
# Usage: scripts/checks/script-budget.sh [--report] [DIR [LIST]] | --self-test
#
# --report prints what each page loads and the compressed total. Exit 0 =
# within the budget, 1 = a finding (listed as rule path: text), 2 = a usage
# error.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
module_max=16384
page_max=147456
packed_max=73728
fetchers="search.js session.js"

# loads DIR JOB...: the modules a page with these jobs loads, one name per
# line, the loader first; a job with no module is printed as `!job`.
loads() {
  local dir="$1" name seen=" " found
  shift
  local todo=(fui "$@")
  while [ "${#todo[@]}" -gt 0 ]; do
    name="${todo[0]}"
    todo=("${todo[@]:1}")
    case "$seen" in *" $name "*) continue ;; esac
    seen="$seen$name "
    if [ ! -f "$dir/$name.js" ]; then
      echo "!$name"
      continue
    fi
    echo "$name"
    found="$(grep -oE '(from|import\(|import) *"\./[A-Za-z0-9_]+\.js"' "$dir/$name.js" |
      sed -E 's/.*"\.\/([A-Za-z0-9_]+)\.js"/\1/' || true)"
    for name in $found; do todo+=("$name"); done
  done
}

# check DIR LIST [REPORT]: every finding, one per line; with REPORT true,
# also what each page loads and the compressed total.
check() {
  local dir="$1" list="$2" report="${3:-false}" file name size packed=0 page jobs total pages=0
  for file in "$dir"/*.js; do
    [ -f "$file" ] || continue
    name="$(basename "$file")"
    size="$(wc -c <"$file" | tr -d ' ')"
    packed=$((packed + $(gzip -9 -n -c "$file" | wc -c | tr -d ' ')))
    [ "$size" -le "$module_max" ] || echo "S1 $file: $size bytes, over $module_max"
    awk -v name="$name" -v fetchers=" $fetchers " '
      /^[ \t]*\/\// { next }
      /(^|[^A-Za-z0-9_.])eval\(|new Function/ { print "S3 " FILENAME ":" FNR ": " $0 }
      /https?:\/\/|["'\''`]\/\/[A-Za-z0-9]/ { print "S4 " FILENAME ":" FNR ": " $0 }
      /(^|[^A-Za-z0-9_.])fetch\(/ && index(fetchers, " " name " ") == 0 { print "S5 " FILENAME ":" FNR ": " $0 }
    ' "$file"
  done
  [ "$packed" -le "$packed_max" ] || echo "S6 $dir: $packed bytes compressed together, over $packed_max"
  [ "$report" = false ] || echo "compressed: $packed of $packed_max"
  while read -r page jobs; do
    case "$page" in "" | "#"*) continue ;; esac
    pages=$((pages + 1))
    total=0
    # shellcheck disable=SC2086 # every job is one word
    for name in $(loads "$dir" $jobs); do
      case "$name" in
        !*) echo "S7 $list: $page names ${name#!}, which has no module" ;;
        *) total=$((total + $(wc -c <"$dir/$name.js" | tr -d ' '))) ;;
      esac
    done
    [ "$total" -le "$page_max" ] || echo "S2 $list: $page loads $total bytes, over $page_max"
    [ "$report" = false ] || echo "page: $total of $page_max $page"
  done <"$list"
  [ "$pages" -gt 0 ] || echo "S7 $list: no page"
}

self_test() {
  local work out want
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' RETURN
  head -c 17000 /dev/zero | tr '\0' ' ' >"$work/big.js"
  # Random text, which gzip cannot shrink under S6's cap.
  for want in 1 2 3 4 5 6 7 8 9 10; do
    head -c 12000 /dev/urandom | base64 | tr -d '\n' | head -c 15000 >"$work/pad$want.js"
  done
  : >"$work/fui.js"
  # A page whose one job is small and carries it over S2 by its imports.
  {
    for want in 1 2 3 4 5 6 7 8; do printf 'import "./pad%s.js";\n' "$want"; done
    printf 'import { a } from "./pad9.js";\nconst b = await import("./pad10.js");\n'
  } >"$work/wide.js"
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
  printf '# A comment line.\n/light search\n/heavy wide\n/missing search nowhere\n' >"$work/pages.list"
  out="$(check "$work" "$work/pages.list")"
  for want in S1 'S2 [^ ]*: /heavy ' S3 S4 S5 S6 'S7 [^ ]*: /missing names nowhere'; do
    printf '%s\n' "$out" | grep -q "^$want" ||
      { echo "self-test: $want was not reported" >&2; printf '%s\n' "$out" >&2; return 1; }
  done
  if printf '%s\n' "$out" | grep -qE 'search\.js|/light '; then
    echo "self-test: the typeahead's fetch, a comment or a light page was reported" >&2
    printf '%s\n' "$out" >&2
    return 1
  fi
  : >"$work/empty.list"
  check "$work" "$work/empty.list" | grep -q "^S7 [^ ]*: no page" ||
    { echo "self-test: an empty page list was passed" >&2; return 1; }
  echo "script-budget: self-test passed"
}

dir="crates/ferrohealth-ui/scripts"
list="scripts/checks/script-pages.list"
report=false
case "${1:-}" in
  --self-test)
    self_test
    exit $?
    ;;
  --report)
    report=true
    shift
    ;;
esac
case "${1:-}" in
  "") ;;
  -*)
    echo "usage: scripts/checks/script-budget.sh [--report] [DIR [LIST]] | --self-test" >&2
    exit 2
    ;;
  *)
    dir="$1"
    list="${2:-$list}"
    ;;
esac

cd "$root"
[ -d "$dir" ] || { echo "script-budget: $dir is no directory" >&2; exit 2; }
[ -f "$list" ] || { echo "script-budget: $list is no file" >&2; exit 2; }
found="$(check "$dir" "$list" "$report")"
if printf '%s\n' "$found" | grep -q '^S[0-9] '; then
  printf '%s\n' "$found" | grep '^S[0-9] ' >&2
  echo "script-budget: split or shrink the module, or move the work to the server." >&2
  exit 1
fi
[ "$report" = false ] || printf '%s\n' "$found"
echo "script-budget: every module under $dir within the budget"
