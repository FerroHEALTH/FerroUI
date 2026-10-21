# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""What the shared tools need to know about the FerroFED console mockups."""

# The theme file under design/themes/ and the console whose assets/ holds
# the brand icons.
BRAND = "ferrofed"
ASSETS = "ferrofed"

# The contact sheet: its title, its heading, its accent in light and dark,
# and its introduction ({total} is the page count).
TITLE = "FerroFED console mockups"
HEADING = "FerroFED console mockups"
ACCENT = ("#0369a1", "#38bdf8")
INTRO = """<p>The redesigned FerroFED operator console, <code>ferrofed-console</code> (FerroHEALTH/FerroFED#1218), drawn with the FerroUI stylesheet in the FerroFED azure theme on the FerroPIX design language: every resting screen and every interactive state, {total} pages, each at 1440 px in light and dark and at 390 px in light, full page, at 2x. Grouped by screen: the resting screen first, its states after it.</p>
<p>All data is synthetic: no real person, every nine-digit number fails the BSN eleven-proof (the generator checks every page), every organisation, node, endpoint, host and certificate is invented (hosts under <code>example.org</code>), and the operator is Demo Operator.</p>"""
