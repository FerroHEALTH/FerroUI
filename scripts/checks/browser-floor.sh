#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Builds the library's browser half at the lowest web-sys and
# wasm-bindgen-futures its manifest asks for, so a console whose lock holds
# an older wasm-bindgen than the gallery's is never moved by the library
# (no specification governs this: our own design).
#
# The library's manifest names each floor as a plain requirement; this
# copies the tracked tree into a scratch directory, narrows the workspace to
# the library, turns each floor into an exact requirement, resolves a fresh
# lock (Cargo backtracks to the leptos release that fits) and checks the
# library for wasm32 under hydrate and under csr. The committed Cargo.lock
# is never touched.
#
# Usage: scripts/checks/browser-floor.sh
#
# Exit 0 = the library builds at its floors. Exit 1 = a floor is unreadable,
# does not resolve, or does not build.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest="crates/ferrohealth-ui/Cargo.toml"

die() {
  echo "browser-floor: $*" >&2
  exit 1
}

# The version requirement of the dependency NAME in the library manifest.
floor_of() {
  awk -v name="$1" '
    $0 ~ "^" name "[[:space:]]*=" {
      if (match($0, /"[0-9][0-9.]*"/)) {
        print substr($0, RSTART + 1, RLENGTH - 2)
        exit
      }
    }
  ' "$root/$manifest"
}

web_sys="$(floor_of web-sys)"
futures="$(floor_of wasm-bindgen-futures)"
[[ "$web_sys" =~ ^0\.3\.[0-9]+$ ]] || die "$manifest names no web-sys floor 0.3.N"
[[ "$futures" =~ ^0\.4\.[0-9]+$ ]] || die "$manifest names no wasm-bindgen-futures floor 0.4.N"

scratch="$(mktemp -d)"
trap 'rm -r -- "$scratch"' EXIT
(cd "$root" && git ls-files -z | tar --null -T - -cf -) | tar -xf - -C "$scratch"

sed -i.orig 's|^members = .*|members = ["crates/ferrohealth-ui"]|' "$scratch/Cargo.toml"
sed -i.orig \
  -e "s|^web-sys = { version = \"$web_sys\"|web-sys = { version = \"=$web_sys\"|" \
  -e "s|^wasm-bindgen-futures = \"$futures\"|wasm-bindgen-futures = \"=$futures\"|" \
  "$scratch/$manifest"
grep -q "^web-sys = { version = \"=$web_sys\"" "$scratch/$manifest" ||
  die "could not pin web-sys in the scratch copy of $manifest"
grep -q "^wasm-bindgen-futures = \"=$futures\"" "$scratch/$manifest" ||
  die "could not pin wasm-bindgen-futures in the scratch copy of $manifest"
rm -f -- "$scratch/Cargo.lock"

cd "$scratch"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target}/browser-floor"
cargo generate-lockfile --quiet ||
  die "web-sys $web_sys and wasm-bindgen-futures $futures do not resolve with the library's leptos"
bindgen="$(awk '/^name = "wasm-bindgen"$/ { p = 1; next } p && /^version = / { gsub(/version = "|"/, ""); print; exit }' Cargo.lock)"
for mode in hydrate csr; do
  cargo check --quiet --locked -p ferrohealth-ui --lib \
    --target wasm32-unknown-unknown --features "leptos/$mode" ||
    die "the library does not build under $mode at web-sys $web_sys"
done
echo "browser-floor: the library builds at web-sys $web_sys, wasm-bindgen-futures $futures, wasm-bindgen $bindgen"
