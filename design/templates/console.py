# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""What the shared tools need to know about the reference templates."""

# The templates draw in the FerroFED theme with FerroFED's icons.
BRAND = "ferrofed"
ASSETS = "ferrofed"

TITLE = "FerroUI reference templates"
HEADING = "FerroUI reference templates"
ACCENT = ("#0369a1", "#38bdf8")
INTRO = """<p>The page templates of the shared console library, T1 list in three states, T2 detail, T3 settings resting and in edit, and About: {total} pages, each at 1440 px in light and dark and at 390 px in light, full page, at 2x. They use example content and draw in the FerroFED theme. <code>design/COMPONENTS.md</code> says when each part of a template appears.</p>"""

# Example content: a link to a page outside the templates points at #.
EXEMPT_CHECKS = ("dead",)
