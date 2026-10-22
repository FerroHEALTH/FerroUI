#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Fails when the gallery's site bundle names a directory of the
# host that built it (no specification governs this: our own design).
#
# scripts/release/gallery-site.sh remaps the cargo home, the toolchain and the
# checkout to fixed prefixes for the WebAssembly build, so a bundle names
# none of them. This reads every WebAssembly and JavaScript file of the
# bundle (the entry, its glue, the lazy chunks and their loader) for a
# home directory (/home/, /Users/, /root/, C:\Users\), a CI runner path
# (/runner/, /github/workspace), the cargo registry (.cargo/registry) and a
# rustup toolchain (.rustup/toolchains), and prints each one it finds.
#
# Usage: scripts/checks/gallery-paths.sh [pkg-dir]
#   pkg-dir defaults to target/site/pkg.
#
# Exit 0 = no host path. Exit 1 = a host path, or a bundle file missing.
# Exit 2 = a usage error.
set -Eeuo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
name="ferrohealth-ui-gallery"

case "$#" in
  0) pkg="$root/target/site/pkg" ;;
  1) pkg="$1" ;;
  *)
    echo "usage: scripts/checks/gallery-paths.sh [pkg-dir]" >&2
    exit 2
    ;;
esac

pattern='(/home/|/Users/|/root/|[A-Za-z]:\\Users\\|/runner/|/github/workspace|\.cargo/registry|\.rustup/toolchains)[^"[:cntrl:]]*'

for file in "$name.wasm" "$name.js"; do
  if [ ! -s "$pkg/$file" ]; then
    echo "gallery-paths: $pkg/$file is missing; run scripts/release/gallery-site.sh --release first." >&2
    exit 1
  fi
done

# The entry, its glue, and the lazy islands' chunks and their loader.
found=0
for path in "$pkg"/*.wasm "$pkg"/*.js; do
  file="$(basename "$path")"
  hits="$(LC_ALL=C grep -a -o -E "$pattern" "$pkg/$file" | sort -u || true)"
  if [ -n "$hits" ]; then
    count="$(printf '%s\n' "$hits" | wc -l | tr -d '[:space:]')"
    echo "::error file=scripts/release/gallery-site.sh::pkg/$file names $count path(s) of the build host; remap them in scripts/release/gallery-site.sh." >&2
    printf '%s\n' "$hits" | head -n 20 >&2
    found=1
  fi
done
[ "$found" -eq 0 ] || exit 1
echo "gallery-paths: the bundle names no directory of the build host"
