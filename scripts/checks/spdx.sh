#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Every tracked first-party file under the given paths carries both SPDX
# lines of the project licence, `SPDX-FileCopyrightText: Cadasto B.V.` and
# `SPDX-License-Identifier: BUSL-1.1`, in its first twelve lines (room for a
# shebang or a front-matter block before them). No specification governs
# this: our own design, after the REUSE convention for the two tags.
#
# Usage: scripts/checks/spdx.sh <path> [path ...]
set -euo pipefail
cd "$(dirname "$0")/../.."

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <path> [path ...]" >&2
  exit 2
fi

failed=0
checked=0
while IFS= read -r -d '' f; do
  case "$f" in
    *.png | *.jpg | *.ico) continue ;;
  esac
  checked=$((checked + 1))
  head="$(head -n 12 -- "$f")"
  if ! grep -qF 'SPDX-FileCopyrightText: Cadasto B.V.' <<< "$head" \
    || ! grep -qF 'SPDX-License-Identifier: BUSL-1.1' <<< "$head"; then
    echo "spdx: $f lacks the SPDX lines in its first twelve lines" >&2
    failed=1
  fi
done < <(git ls-files -z -- "$@")

[[ "$failed" -eq 0 ]] || exit 1
echo "spdx: $checked files carry both SPDX lines"
