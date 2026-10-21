# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""What the shared tools need to know about the FerroPIX console mockups."""

# The theme file under design/themes/ and the console whose assets/ holds
# the brand icons.
BRAND = "ferropix"
ASSETS = "ferropix"

# The contact sheet: its title, its heading, its accent in light and dark,
# and its introduction ({total} is the page count).
TITLE = "FerroPIX console mockups"
HEADING = "FerroPIX console mockups, on the shared library"
ACCENT = ("#6b21a8", "#c084fc")
INTRO = """<p>The operator console of section 12 of the FerroPIX architecture, drawn with the FerroUI stylesheet in the Plum &amp; Iron brand, after review round 1 and the back-feed from the shared FerroUI console library (binder-tab settings pages, the domains table everywhere, F13 menus, F14 action groups, comfortable density): every resting screen and every interactive state, {total} pages, each at 1440 px in light and dark and at 390 px in light, full page, at 2x. Grouped by screen: the resting screen first, its states after it.</p>
<p>All data is synthetic: no real person, every nine-digit number fails the BSN eleven-proof (the generator checks every page), every address is a Voorbeeldstraat or a Teststraat in the 9999 range, and the steward is Demo Steward.</p>"""
