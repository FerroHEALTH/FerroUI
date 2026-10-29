#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Markup rules guard: the markup the library and the gallery write keeps a
# console's Content-Security-Policy at `script-src 'self'; style-src 'self'`
# (no specification governs the check itself: our own design; CSP Level 3
# refuses inline script and style under that policy).
#
# It reads the Askama templates (.html) under each ROOT and the hand-written
# .rs files there, up to the file's `#[cfg(test)]` module and past comment
# lines, and reports each line that breaks a rule:
#
#   M1  a `style` attribute;
#   M2  a `<style>` element;
#   M3  a `<script>` element other than an empty `<script type="module"
#       src="...">`, the loader the shell writes;
#   M4  an inline event-handler attribute (`onclick="..."`);
#   M5  a `javascript:` URL.
#
# The words a reader sees are the stray-literal check's
# (scripts/checks/stray-literals.sh), which reads the templates too.
#
# Usage: scripts/checks/markup-rules.sh ROOT... | --self-test
#
# Exit 0 = clean, 1 = a finding (listed as rule path:line: text), 2 = a
# usage error.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

check() {
  local files=()
  local file
  while IFS= read -r file; do
    files+=("$file")
  done < <(find "$@" -type f \( -name '*.html' -o -name '*.rs' \) | LC_ALL=C sort)
  [ "${#files[@]}" -gt 0 ] || return 0
  awk '
    FNR == 1 { off = 0; rust = FILENAME ~ /\.rs$/ }
    off { next }
    rust && /^#\[cfg\(test\)\]/ { off = 1; next }
    rust && /^[ \t]*\/\// { next }
    {
      line = $0
      gsub(/\\"/, "\"", line)
      if (line ~ /[ \t"]style="/) print "M1 " FILENAME ":" FNR ": " $0
      if (line ~ /<style[ >]/) print "M2 " FILENAME ":" FNR ": " $0
      rest = line
      while ((at = index(rest, "<script")) > 0) {
        rest = substr(rest, at)
        if (rest !~ /^<script type="module" src="[^"]*"[^>]*><\/script>/) {
          print "M3 " FILENAME ":" FNR ": " $0
          break
        }
        rest = substr(rest, 8)
      }
      if (line ~ /[ \t"]on[a-z]+="/) print "M4 " FILENAME ":" FNR ": " $0
      if (line ~ /javascript:/) print "M5 " FILENAME ":" FNR ": " $0
    }
  ' "${files[@]}"
}

self_test() {
  local work out status want
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' RETURN
  cat >"$work/bad.html" <<'EOF'
<div style="color: red">x</div>
<style>p {}</style>
<script>alert(1)</script>
<button onclick="go()">x</button>
<a href="javascript:void(0)">x</a>
EOF
  cat >"$work/bad.rs" <<'EOF'
fn bad() -> String {
    format!("<span style=\"width: 1px\"></span>")
}
EOF
  cat >"$work/fine.html" <<'EOF'
<script type="module" src="{{ loader }}" nonce="{{ nonce }}"></script>
<div data-fui="tree" class="fui-tree" hidden>{{ items }}</div>
EOF
  cat >"$work/fine.rs" <<'EOF'
// A comment may name a style="" attribute.
fn fine() -> &'static str {
    "<span class=\"fui-x\" data-style=\"1\"></span>"
}
#[cfg(test)]
mod tests {
    const BAD: &str = "javascript:alert(1)";
}
EOF
  status=0
  out="$(check "$work")" || status=$?
  [ "$status" = 0 ] || { echo "self-test: the check failed to run" >&2; return 1; }
  for want in M1 M2 M3 M4 M5; do
    printf '%s\n' "$out" | grep -q "^$want .*bad.html" ||
      { echo "self-test: $want was not reported" >&2; printf '%s\n' "$out" >&2; return 1; }
  done
  printf '%s\n' "$out" | grep -q "^M1 .*bad.rs" ||
    { echo "self-test: an escaped style attribute in Rust was not reported" >&2; return 1; }
  if printf '%s\n' "$out" | grep -q 'fine\.'; then
    echo "self-test: a clean file was reported" >&2
    printf '%s\n' "$out" >&2
    return 1
  fi
  echo "markup-rules: self-test passed"
}

case "${1:-}" in
  "")
    echo "usage: scripts/checks/markup-rules.sh ROOT... | --self-test" >&2
    exit 2
    ;;
  --self-test)
    self_test
    exit $?
    ;;
  -*)
    echo "usage: scripts/checks/markup-rules.sh ROOT... | --self-test" >&2
    exit 2
    ;;
esac

cd "$root"
for dir in "$@"; do
  [ -d "$dir" ] || { echo "markup-rules: $dir is no directory" >&2; exit 2; }
done
found="$(check "$@")"
if [ -n "$found" ]; then
  printf '%s\n' "$found" >&2
  echo "markup-rules: move the style to the stylesheet and the behaviour to a script module (crates/ferrohealth-ui/scripts/)." >&2
  exit 1
fi
echo "markup-rules: no finding under $*"
