# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""What the shared tools need to know about the FerroTERM console mockups."""

# The theme file under design/themes/ and the console whose assets/ holds
# the brand icons.
BRAND = "ferroterm"
ASSETS = "ferroterm"

# The contact sheet: its title, its heading, its accent in light and dark,
# and its introduction ({total} is the page count).
TITLE = "FerroTERM console mockups"
HEADING = "FerroTERM console mockups"
ACCENT = ("#0f766e", "#2dd4bf")
INTRO = """<p>The redesigned FerroTERM operator console, <code>ferroterm-console</code> (FerroTERM #800; image <code>ghcr.io/ferrohealth/ferroterm-console</code>, environment prefix <code>FERROTERM_CONSOLE__</code>), step 4 of the family console redesign after FerroPIX, FerroFED and FerroBRIDGE. It is drawn only with the FerroUI library in the FerroTERM teal theme: every resting screen and every interactive state, {total} pages, each at 1440 px in light and dark and at 390 px in light, full page, at 2x. Grouped by screen: the resting screen first, its states after it.</p>
<p>Every page shows what the console has today, laid out anew. Screens for this redesign's proposals (<em>Proposed</em>) are a separate group at the end, and no live page links to them. Configuration is file mode only (family rule 13): FerroTERM has no configuration store, so Configuration is a read-only view of <code>console.toml</code>.</p>
<p>All data is synthetic or public. The concepts shown come from a synthetic code system (codes <code>SF-nnnn</code>, <code>NF-nnn</code>, <code>LF-nn</code>) and from public ones (ISO 3166-1, HL7 terminology); SNOMED CT and LOINC appear only as listed rows, never with content. Hosts are under <code>example.org</code>, and the people are Demo Terminologist and Demo Reader. The conformance and benchmark figures are the ones the FerroTERM repository commits. Every nine-digit number fails the BSN eleven-proof, and the generator checks every page.</p>"""
