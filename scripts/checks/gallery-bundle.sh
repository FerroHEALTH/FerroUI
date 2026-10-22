#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Measures the gallery's release site bundle (app/ferrohealth-ui-gallery)
# and fails when its brotli-compressed WebAssembly is over its byte budget
# (no specification governs this: our own design).
#
# It reads the bundle scripts/release/gallery-site.sh --release writes, and
# refuses one whose digests differ from the stamp that build leaves in
# target/gallery-site.sha256 on success, so a failed or a debug build never
# measures a stale bundle as within the budget. The budget holds the entry
# WebAssembly, which every page loads before any island hydrates. It prints
# the entry and its JavaScript glue raw, gzip-compressed (gzip -9) and
# brotli-compressed (brotli -q 11), then what each lazy island's chunks add
# on a page that draws it (the files the split manifest names for it,
# brotli-compressed) and all the chunks together, as Markdown tables, and
# appends them to $GITHUB_STEP_SUMMARY when that is set. The budget is read
# from the budget table of the Leptos rule file, beside its reasoning, so the
# number lives in one place.
#
# Usage: scripts/checks/gallery-bundle.sh [pkg-dir]
#   pkg-dir defaults to target/site/pkg.
#
# Exit 0 = within the budget. Exit 1 = over the budget, a bundle file missing,
# a bundle the last release build did not write, a compressor or jq missing,
# or the budget unreadable. Exit 2 = a usage error.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
rule="$root/.claude/rules/leptos-ui.md"
name="ferrohealth-ui-gallery"

case "$#" in
  0) pkg="$root/target/site/pkg" ;;
  1) pkg="$1" ;;
  *)
    echo "usage: scripts/checks/gallery-bundle.sh [pkg-dir]" >&2
    exit 2
    ;;
esac

for tool in gzip brotli jq; do
  if ! command -v "$tool" > /dev/null; then
    echo "gallery-bundle: $tool is not on PATH; the bundle cannot be measured." >&2
    exit 1
  fi
done

for file in "$name.wasm" "$name.js" __wasm_split_manifest.json; do
  if [ ! -s "$pkg/$file" ]; then
    echo "gallery-bundle: $pkg/$file is missing; run scripts/release/gallery-site.sh --release first." >&2
    exit 1
  fi
done

stamp="$root/target/gallery-site.sha256"
if [ ! -s "$stamp" ]; then
  echo "gallery-bundle: $stamp is missing: the last site build failed or was not a release build; run scripts/release/gallery-site.sh --release first." >&2
  exit 1
fi
if command -v sha256sum > /dev/null; then sum=(sha256sum); else sum=(shasum -a 256); fi
if ! (cd "$pkg" && "${sum[@]}" -c --status "$stamp"); then
  echo "gallery-bundle: $pkg is not the bundle the last release build wrote ($stamp); run scripts/release/gallery-site.sh --release first." >&2
  exit 1
fi

# The one row of the budget table whose first cell names the gating measure.
budgets="$(awk -F'|' '
  $2 ~ /^[[:space:]]*Entry WebAssembly, brotli-compressed[[:space:]]*$/ {
    value = $3
    sub(/^[[:space:]]*/, "", value)
    sub(/[[:space:]]*bytes.*$/, "", value)
    print value
  }
' "$rule")"
if ! [[ "$budgets" =~ ^[0-9]+$ ]]; then
  echo "gallery-bundle: $rule has no single \"| Entry WebAssembly, brotli-compressed | N bytes |\" budget row." >&2
  exit 1
fi
budget="$budgets"

raw() { wc -c < "$1" | tr -d '[:space:]'; }
gz() { gzip -9 -c "$1" | wc -c | tr -d '[:space:]'; }
br() { brotli -q 11 -c "$1" | wc -c | tr -d '[:space:]'; }

wasm="$pkg/$name.wasm"
js="$pkg/$name.js"
manifest="$pkg/__wasm_split_manifest.json"
wasm_br="$(br "$wasm")"
percent=$((wasm_br * 100 / budget))

# What a lazy island adds: the brotli bytes of every chunk its manifest entry
# names, the chunks it shares with another island counted for each.
lazy_rows=""
while IFS=$'\t' read -r island files; do
  bytes=0
  for chunk in $files; do
    if [ ! -s "$pkg/$chunk.wasm" ]; then
      echo "gallery-bundle: $manifest names $chunk, which the bundle lacks." >&2
      exit 1
    fi
    bytes=$((bytes + $(br "$pkg/$chunk.wasm")))
  done
  lazy_rows+="| \`${island%_loader_*}\` | $(wc -w <<< "$files" | tr -d '[:space:]') | $bytes |"$'\n'
done < <(jq -r 'to_entries | sort_by(.key)[] | "\(.key)\t\(.value | join(" "))"' "$manifest")
chunks=0
for chunk in "$pkg"/chunk_*.wasm "$pkg"/split_*.wasm; do
  [ -e "$chunk" ] || continue
  chunks=$((chunks + $(br "$chunk")))
done
loader="$(find "$pkg" -maxdepth 1 -name '__wasm_split.*.js' -print -quit)"

table="$(
  echo "### Gallery bundle"
  echo
  echo "| File | Raw | gzip -9 | brotli -q 11 |"
  echo "|---|---:|---:|---:|"
  echo "| \`$name.wasm\` | $(raw "$wasm") | $(gz "$wasm") | $wasm_br |"
  echo "| \`$name.js\` | $(raw "$js") | $(gz "$js") | $(br "$js") |"
  if [ -n "$loader" ]; then
    echo "| \`$(basename "$loader")\` | $(raw "$loader") | $(gz "$loader") | $(br "$loader") |"
  fi
  echo
  echo "| Lazy island | Chunks | brotli -q 11 |"
  echo "|---|---:|---:|"
  printf '%s' "$lazy_rows"
  echo "| every chunk, each once | | $chunks |"
  echo
  echo "Sizes in bytes. Budget: $budget bytes of brotli-compressed entry WebAssembly; this build is $wasm_br bytes, $percent% of it."
)"

echo "$table"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  echo "$table" >> "$GITHUB_STEP_SUMMARY"
fi

if [ "$wasm_br" -gt "$budget" ]; then
  echo "::error file=.claude/rules/leptos-ui.md::The brotli-compressed entry WebAssembly is $wasm_br bytes, over the $budget-byte budget of .claude/rules/leptos-ui.md §12. Shrink the bundle, make a large island lazy, or raise the budget there with the reason." >&2
  exit 1
fi
echo "gallery-bundle: $wasm_br of $budget bytes, within the budget"
