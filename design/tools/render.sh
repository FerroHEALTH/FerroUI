#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Builds a design set's pages and renders them, every page at 1440 px light
# and dark and at 390 px light, with the contact sheet, into the ignored
# design/out/<set>/ (no specification governs this: our own design).
#
# Usage: design/tools/render.sh <set|all> [slug ...]
#   A set is `templates` or a console: ferrobridge, ferroehr, ferrofed,
#   ferropix, ferroterm. FERROUI_CHROME names the browser binary and
#   FERROUI_DESIGN_JOBS the number of browsers run side by side. The renders
#   hold the machine-wide Chrome lock (scripts/chrome-lock.sh).
set -euo pipefail
tools="$(cd "$(dirname "$0")" && pwd)"
if [[ "${FERROUI_CHROME_LOCK_HELD:-}" != 1 && $# -ge 1 ]]; then
  exec bash "$tools/../../scripts/chrome-lock.sh" bash "$0" "$@"
fi

if [[ $# -lt 1 ]]; then
  sed -n '/^# Usage:/,/^set /p' "$0" | sed '$d; s/^# \{0,1\}//' >&2
  exit 2
fi

sets=("$1")
shift
if [[ "${sets[0]}" == all ]]; then
  read -r -a sets <<< "$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import design_set; print(" ".join(design_set.names()))' "$tools")"
fi

for set in "${sets[@]}"; do
  # The build's markup checks report here; rendering goes on so a finding
  # can be looked at.
  python3 "$tools/build.py" "$set" "$@" || echo "render: $set has markup findings; design/tools/check.sh $set lists them" >&2
  python3 "$tools/render.py" "$set" "$@"
  python3 "$tools/index.py" "$set"
done
