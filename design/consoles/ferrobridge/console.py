# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""What the shared tools need to know about the FerroBRIDGE console mockups."""

# The theme file under design/themes/ and the console whose assets/ holds
# the brand icons.
BRAND = "ferrobridge"
ASSETS = "ferrobridge"

# The contact sheet: its title, its heading, its accent in light and dark,
# and its introduction ({total} is the page count).
TITLE = "FerroBRIDGE console mockups"
HEADING = "FerroBRIDGE console mockups"
ACCENT = ("#4f46e5", "#a5b4fc")
INTRO = """<p>The redesigned FerroBRIDGE operator console, <code>ferrobridge-console</code> (image <code>ghcr.io/ferrohealth/ferrobridge-console</code>, environment prefix <code>FERROBRIDGE_CONSOLE__</code>), step 3 of the family console redesign after FerroPIX and FerroFED. It is drawn with the FerroUI library in the FerroBRIDGE indigo theme, B1 to B10 now folded into it: every resting screen and every interactive state, {total} pages, each at 1440 px in light and dark and at 390 px in light, full page, at 2x. Grouped by screen: the resting screen first, its states after it.</p>
<p>Every page shows what the console has today. Screens for planned work (<em>Planned #N</em>) and for this redesign's proposals (<em>Proposed</em>) are a separate group at the end, and no live page links to them. Configuration is file mode only (owner decision): every Settings tab is a read-only view of <code>ferrobridge.toml</code>.</p>
<p>All data is synthetic. Templates are <code>synthetic.*</code>, patients are <code>Patient/synthetic-NNNN</code>, hosts are under <code>example.org</code>, and the operators are Demo Operator and Demo Reviewer. Every nine-digit number fails the BSN eleven-proof, and the generator checks every page.</p>"""
