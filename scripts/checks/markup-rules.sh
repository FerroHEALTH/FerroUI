#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Markup rules guard: the markup the library and the gallery write keeps a
# console's Content-Security-Policy at `script-src 'self'; style-src 'self'`
# (no specification governs the check itself: our own design; CSP Level 3
# refuses inline script and style under that policy).
#
# It reads the Askama templates (.html) under each ROOT and the hand-written
# .rs and .js files there, a .rs file up to its `#[cfg(test)]` module, past
# comment lines, and reports each line that breaks a rule:
#
#   M1  a `style` attribute;
#   M2  a `<style>` element;
#   M3  a `<script>` element other than an empty `<script type="module"
#       src="...">`, the loader the shell writes;
#   M4  an inline event-handler attribute (`onclick="..."`);
#   M5  a `javascript:` URL;
#   M6  a `separator`, `region` or `status` role, which `hr`, a named
#       `section` and `output` carry (ARIA in HTML § 4), unless a `NOTE:`
#       on the line or in the four lines above it says why the element
#       keeps the role.
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
  done < <(find "$@" -type f \( -name '*.html' -o -name '*.rs' -o -name '*.js' \) | LC_ALL=C sort)
  [[ "${#files[@]}" -gt 0 ]] || return 0
  awk '
    FNR == 1 { off = 0; rust = FILENAME ~ /\.rs$/; code = FILENAME ~ /\.(rs|js)$/; noted = -10 }
    off { next }
    rust && /^#\[cfg\(test\)\]/ { off = 1; next }
    /NOTE:/ { noted = FNR }
    code && /^[ \t]*\/\// { next }
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
      if (line ~ /role(=|: |", )"(separator|region|status)"/ && FNR - noted > 4) print "M6 " FILENAME ":" FNR ": " $0
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
<div role="status">x</div>
EOF
  cat >"$work/bad.js" <<'EOF'
const said = element("p", { role: "status" });
EOF
  cat >"$work/bad.rs" <<'EOF'
fn bad() -> String {
    format!("<span style=\"width: 1px\"></span>")
}
EOF
  cat >"$work/fine.html" <<'EOF'
<script type="module" src="{{ loader }}" nonce="{{ nonce }}"></script>
{# NOTE: the region holds blocks, so it keeps the role. #}<div role="status"><div></div></div>
<div data-fui="tree" class="fui-tree" hidden>{{ items }}</div>
EOF
  cat >"$work/fine.rs" <<'EOF'
// A comment may name a style="" attribute.
fn fine() -> &'static str {
    "<span class=\"fui-x\" data-style=\"1\"></span>"
}
// NOTE: the region is the `pre` itself, so it keeps the role.
fn noted() -> &'static str {
    "<pre role=\"region\" aria-label=\"x\"></pre>"
}
#[cfg(test)]
mod tests {
    const BAD: &str = "javascript:alert(1)";
}
EOF
  status=0
  out="$(check "$work")" || status=$?
  [[ "$status" = 0 ]] || { echo "self-test: the check failed to run" >&2; return 1; }
  for want in M1 M2 M3 M4 M5 M6; do
    printf '%s\n' "$out" | grep -q "^$want .*bad.html" ||
      { echo "self-test: $want was not reported" >&2; printf '%s\n' "$out" >&2; return 1; }
  done
  printf '%s\n' "$out" | grep -q "^M1 .*bad.rs" ||
    { echo "self-test: an escaped style attribute in Rust was not reported" >&2; return 1; }
  printf '%s\n' "$out" | grep -q "^M6 .*bad.js" ||
    { echo "self-test: a role a module writes was not reported" >&2; return 1; }
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
  *) ;;
esac

cd "$root"
for dir in "$@"; do
  [[ -d "$dir" ]] || { echo "markup-rules: $dir is no directory" >&2; exit 2; }
done
found="$(check "$@")"
if [[ -n "$found" ]]; then
  printf '%s\n' "$found" >&2
  echo "markup-rules: move the style to the stylesheet and the behaviour to a script module (crates/ferrohealth-ui/scripts/), and for M6 draw hr, a named section or output, or say in a NOTE why the element keeps its role." >&2
  exit 1
fi
echo "markup-rules: no finding under $*"
