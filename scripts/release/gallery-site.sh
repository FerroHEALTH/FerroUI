#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Builds the gallery's site bundle (app/ferrohealth-ui-gallery) with
# cargo-leptos, with the workspace lockfile frozen, and checks the bundle is
# whole (no specification governs this: our own design).
#
# cargo-leptos resolves the workspace through its own `cargo metadata` call,
# which takes no --locked, and cargo has no environment variable for it. So
# the lockfile is checked with `cargo metadata --locked` first, which fails
# loud when Cargo.lock does not satisfy every manifest and leaves the second
# resolution nothing to change, and --locked is passed to the compile itself.
#
# The bundle is target/site: pkg/ferrohealth-ui-gallery.wasm and its
# JavaScript glue. The gallery has no stylesheet of its own: its server serves
# the library's stylesheet and the brand themes from the library's constants,
# as a console does. A release build that succeeds writes the digests of the
# two files to target/gallery-site.sha256, which
# scripts/checks/gallery-bundle.sh reads; the build removes it first, so a
# failed build leaves no stamp and a stale bundle is never measured.
#
# Usage: scripts/release/gallery-site.sh [--release]
#   --release builds the wasm-release profile CI and the budget measure;
#   without it, the fast wasm-dev profile the local journeys use.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
gallery="$root/app/ferrohealth-ui-gallery"

case "${1:-}" in
  "" | --release) ;;
  *)
    echo "usage: scripts/release/gallery-site.sh [--release]" >&2
    exit 2
    ;;
esac

stamp="$root/target/gallery-site.sha256"
rm -f -- "$stamp"

if ! cargo metadata --locked --format-version 1 \
  --manifest-path "$gallery/Cargo.toml" > /dev/null; then
  echo "gallery-site: Cargo.lock does not satisfy the workspace manifests;" >&2
  echo "gallery-site: re-resolve it deliberately and commit the change." >&2
  exit 1
fi

# The bundle names no directory of the build host: the panic locations rustc
# embeds name the registry, the toolchain and the checkout, and each is
# remapped to a fixed prefix for the WebAssembly build alone (the rustc book,
# --remap-path-prefix). Cargo splits the variable on spaces, so a directory
# with one cannot be remapped and is refused.
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
sysroot="$(rustc --print sysroot)"
for dir in "$cargo_home" "$sysroot" "$root"; do
  case "$dir" in
    *" "*)
      echo "gallery-site: $dir holds a space; its path cannot be remapped" >&2
      exit 1
      ;;
  esac
done
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="--remap-path-prefix=$cargo_home=/cargo --remap-path-prefix=$sysroot=/rustc-sysroot --remap-path-prefix=$root=/ferrohealth-ui"

# cargo-leptos runs a wasm-opt it finds on PATH and downloads one unchecked
# when it finds none, so the build refuses to start until PATH holds the
# pinned, verified release (scripts/release/gallery-tools.sh installs it; no
# specification governs this: our own design). The tools installed into the
# workspace's target/gallery-tools go first on PATH when they are there.
tools="$root/target/gallery-tools/bin"
if [ -d "$tools" ]; then
  PATH="$tools:$PATH"
  export PATH
fi
bash "$root/scripts/release/gallery-tools.sh" --check

# cargo-leptos reads its configuration from the crate's own manifest
# directory, so the build runs from there.
cd "$gallery"
cargo leptos build --frontend-only --lib-cargo-args=--locked "$@"

missing=0
for file in pkg/ferrohealth-ui-gallery.wasm pkg/ferrohealth-ui-gallery.js; do
  if [ ! -s "$root/target/site/$file" ]; then
    echo "gallery-site: the bundle has no $file" >&2
    missing=1
  fi
done
[ "$missing" -eq 0 ] || exit 1
if [ "${1:-}" = --release ]; then
  if command -v sha256sum > /dev/null; then sum=(sha256sum); else sum=(shasum -a 256); fi
  (cd "$root/target/site/pkg" && "${sum[@]}" ferrohealth-ui-gallery.wasm ferrohealth-ui-gallery.js) > "$stamp"
fi
echo "gallery-site: the bundle is in target/site"
