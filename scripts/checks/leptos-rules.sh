#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Leptos rules guard: the mechanical part of .claude/rules/leptos-ui.md §14
# and §15, the Leptos book's way as FerroUI holds it (no specification
# governs the check itself: our own design; each rule cites its book chapter
# and Leptos 0.8 source in the rules file).
#
# It reads the hand-written .rs files of the library and the gallery, skips
# comment lines, and reports each line that breaks a rule:
#
#   L1  the library's view or widget tier imports leptos_router: an island
#       has no router in the browser, and the view tier draws plain links
#       (§15.6);
#   L2  an inline event-handler attribute (`oninput="..."`) or a `<script`
#       element: the strict CSP runs scripts by nonce alone (§0, §14.4);
#   L3  the library defines a server function or loads data (`#[server]`,
#       `Resource`, `LocalResource`, `OnceResource`, `ServerAction`,
#       `Action::new`): data arrives as props (§0, §14.6);
#   L4  `.get()` cloned only to read one fact of a collection
#       (`.get().is_empty()`, `.len()`, `.iter()`, `.contains(`, `.first()`,
#       `.last()`); read it with `.read()` or `.with()` (§14.2);
#   L5  `cfg!(target_arch ...)` in code: a value that differs between the
#       halves changes the view and breaks hydration; browser code goes
#       behind `#[cfg(...)]` in a handler or an effect (§14.8);
#   L6  `inner_html` outside the files whose markup the library escapes and
#       tests (§14.3);
#   L7  `Signal::derive(||` with no `move`: a constant, which is
#       `Signal::stored` (§14.1);
#   L8  a `usize` or `isize` in the signature of an `#[island]`: island props
#       cross the 32-bit WebAssembly boundary as JSON (§15.2);
#   L9  a `style:` directive or a `style=(...)` tuple: no inline style (§0);
#       the stylesheet test holds the plain ` style=` attribute;
#   L10 the library depends on leptos_meta: the document head is the
#       console's (§14.9).
#
# A finding the rule reads wrong, or one an open issue removes, is allowed
# by a line of scripts/checks/leptos-rules.allow:
#
#   <rule> <path from the repository root> <ERE matched against the line>   # reason, with its issue
#
# Usage: scripts/checks/leptos-rules.sh [--self-test]
#
# Exit 0 = clean, 1 = a finding (listed as rule path:line: text), 2 = a
# usage error.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
allow_file="$root/scripts/checks/leptos-rules.allow"

library="crates/ferrohealth-ui/src"
gallery="app/ferrohealth-ui-gallery/src"

# Files whose inner_html carries markup the library escaped itself, each
# held by an escaping test (§14.3).
inner_html_files=(
  "crates/ferrohealth-ui/src/icon.rs"
  "crates/ferrohealth-ui/src/view/chart.rs"
  "crates/ferrohealth-ui/src/view/code.rs"
  "crates/ferrohealth-ui/src/widget/chart.rs"
  "crates/ferrohealth-ui/src/widget/document.rs"
  "crates/ferrohealth-ui/src/widget/editor.rs"
)

findings=()

# Whether the finding RULE PATH TEXT is allowed by the allow file.
allowed() {
  local rule="$1" path="$2" text="$3" a_rule a_path a_regex
  [ -f "$allow_file" ] || return 1
  while read -r a_rule a_path a_regex; do
    case "$a_rule" in "" | "#"*) continue ;; esac
    a_regex="${a_regex%%   #*}"
    a_regex="${a_regex%"${a_regex##*[![:space:]]}"}"
    if [ "$a_rule" = "$rule" ] && [ "$a_path" = "$path" ] &&
      printf '%s\n' "$text" | grep -Eq -- "$a_regex"; then
      return 0
    fi
  done <"$allow_file"
  return 1
}

# Records every line of the .rs files under DIRS matching ERE as RULE.
scan() {
  local rule="$1" ere="$2"
  shift 2
  local hit path rest line text
  while IFS= read -r hit; do
    path="${hit%%:*}"
    rest="${hit#*:}"
    line="${rest%%:*}"
    text="${rest#*:}"
    case "$(printf '%s' "$text" | sed 's/^[[:space:]]*//')" in "//"*) continue ;; esac
    if ! allowed "$rule" "$path" "$text"; then
      findings+=("$rule $path:$line: $(printf '%s' "$text" | sed 's/^[[:space:]]*//')")
    fi
  done < <(cd "$root" && grep -rnE --include='*.rs' -- "$ere" "$@" 2>/dev/null || true)
}

run_checks() {
  scan L1 'leptos_router' "$library/view" "$library/widget"
  scan L2 '\bon[a-z]+="|<script' "$library" "$gallery"
  scan L3 '#\[server|\bResource::|\bLocalResource\b|\bOnceResource\b|\bServerAction\b|\bAction::new' "$library"
  scan L4 '\.get\(\)\.(is_empty|len|iter|contains|first|last)\(' "$library" "$gallery"
  scan L5 'cfg!\(target_arch' "$library" "$gallery"
  scan L7 'Signal::derive\(\|\|' "$library" "$gallery"
  scan L9 '\bstyle:[a-z-]+=|\bstyle=\(' "$library" "$gallery"

  local hit path
  while IFS= read -r hit; do
    path="${hit%%:*}"
    local listed=0 each
    for each in "${inner_html_files[@]}"; do
      [ "$each" = "$path" ] && listed=1
    done
    if [ "$listed" = 0 ]; then
      local rest="${hit#*:}" text
      text="${rest#*:}"
      case "$(printf '%s' "$text" | sed 's/^[[:space:]]*//')" in "//"*) continue ;; esac
      allowed L6 "$path" "$text" ||
        findings+=("L6 $path:${rest%%:*}: $(printf '%s' "$text" | sed 's/^[[:space:]]*//')")
    fi
  done < <(cd "$root" && grep -rnE --include='*.rs' -- 'inner_html' "$library" "$gallery" 2>/dev/null || true)

  # L8: the signature of each #[island], from the attribute to its body.
  local file
  while IFS= read -r file; do
    awk -v path="$file" '
      /#\[island/ { inside = 1; start = FNR; sig = ""; next }
      inside {
        sig = sig " " $0
        if ($0 ~ /\{[[:space:]]*$/ || $0 ~ /->/) {
          if (sig ~ /(^|[^A-Za-z0-9_])(usize|isize)([^A-Za-z0-9_]|$)/) {
            printf "L8 %s:%d: an island signature names usize or isize\n", path, start
          }
          inside = 0
        }
      }
    ' "$root/$file"
  done < <(cd "$root" && grep -rlE --include='*.rs' -- '#\[island' "$library" "$gallery" 2>/dev/null || true) >"${island_out:?}"
  local finding
  while IFS= read -r finding; do
    findings+=("$finding")
  done <"$island_out"

  if grep -q 'leptos_meta' "$root/crates/ferrohealth-ui/Cargo.toml" ||
    (cd "$root" && grep -rqE --include='*.rs' -- 'leptos_meta' "$library"); then
    findings+=("L10 crates/ferrohealth-ui: the library depends on leptos_meta")
  fi
}

self_test() {
  local work
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' RETURN
  mkdir -p "$work/scripts/checks" "$work/crates/ferrohealth-ui/src/widget" \
    "$work/crates/ferrohealth-ui/src/view" "$work/app/ferrohealth-ui-gallery/src"
  cp "${BASH_SOURCE[0]}" "$work/scripts/checks/leptos-rules.sh"
  printf '[package]\nname = "ferrohealth-ui"\n' >"$work/crates/ferrohealth-ui/Cargo.toml"
  cat >"$work/crates/ferrohealth-ui/src/widget/bad.rs" <<'RS'
use leptos_router::components::A;
// use leptos_router in a comment is fine
fn a() { let n = rows.get().len(); }
fn b() { let s = Signal::derive(|| 1); }
fn c() { if cfg!(target_arch = "wasm32") {} }
#[island]
fn Counter(start: usize) -> impl IntoView {}
RS
  cat >"$work/crates/ferrohealth-ui/src/view/fine.rs" <<'RS'
fn d() { let n = rows.read().len(); let s = Signal::stored(1); }
#[island]
fn Good(start: u32) -> impl IntoView {}
RS
  cat >"$work/app/ferrohealth-ui-gallery/src/page.rs" <<'RS'
fn e() { view! { <input oninput="this.form.requestSubmit()"/> <p inner_html=x></p> } }
RS
  local out status=0
  out="$(bash "$work/scripts/checks/leptos-rules.sh" 2>&1)" || status=$?
  [ "$status" = 1 ] || { echo "self-test: expected exit 1, got $status" >&2; return 1; }
  local want
  for want in L1 L2 L4 L5 L6 L7 L8; do
    printf '%s\n' "$out" | grep -q "^  $want " ||
      { echo "self-test: $want was not reported" >&2; printf '%s\n' "$out" >&2; return 1; }
  done
  if printf '%s\n' "$out" | grep -q 'fine.rs'; then
    echo "self-test: a clean file was reported" >&2
    return 1
  fi
  printf 'L1 crates/ferrohealth-ui/src/widget/bad.rs leptos_router::components   # self-test\n' \
    >"$work/scripts/checks/leptos-rules.allow"
  out="$(bash "$work/scripts/checks/leptos-rules.sh" 2>&1)" || true
  if printf '%s\n' "$out" | grep -q '^  L1 '; then
    echo "self-test: an allowed finding was reported" >&2
    return 1
  fi
  echo "leptos-rules: self-test passed"
}

case "${1:-}" in
  "") ;;
  --self-test)
    self_test
    exit $?
    ;;
  *)
    echo "usage: scripts/checks/leptos-rules.sh [--self-test]" >&2
    exit 2
    ;;
esac

island_out="$(mktemp)"
trap 'rm -f "$island_out"' EXIT
run_checks

if [ "${#findings[@]}" -gt 0 ]; then
  echo "leptos-rules: ${#findings[@]} finding(s) against .claude/rules/leptos-ui.md §14 and §15:" >&2
  printf '  %s\n' "${findings[@]}" >&2
  exit 1
fi
echo "leptos-rules: clean"
