#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Builds a design set's pages and runs every check over them: the markup
# checks of build.py (links, the BSN eleven-proof, dead buttons, plurals,
# one-sentence intros, planned screens), then overflow and broken words,
# hover stability, contrast, selects, popups and forms in headless
# Chrome, all read from one render of each page (sweep.py). A finding the set's baseline.tsv does not hold fails, and so does
# a baseline entry that no longer occurs (no specification governs this:
# our own design).
#
# Usage: design/tools/check.sh <set|all> [slug ...]
#   FERROUI_CHROME names the browser binary and FERROUI_DESIGN_JOBS the
#   number of browsers run side by side. FERROUI_DESIGN_PART=k/n runs the
#   browser checks over every nth page from the kth. The checks hold the machine-wide
#   Chrome lock (scripts/chrome-lock.sh), so worktrees run them in turn.
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

failed=0
for set in "${sets[@]}"; do
  python3 "$tools/build.py" "$set" "$@" || failed=1
  python3 "$tools/sweep.py" "$set" "$@" || failed=1
done
exit "$failed"
