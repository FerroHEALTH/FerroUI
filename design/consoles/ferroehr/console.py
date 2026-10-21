# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""What the shared tools need to know about the FerroEHR viewer mockups."""

# The theme file under design/themes/ and the console whose assets/ holds
# the brand icons.
BRAND = "ferroehr"
ASSETS = "ferroehr"

# The contact sheet: its title, its heading, its accent in light and dark,
# and its introduction ({total} is the page count).
TITLE = "FerroEHR viewer mockups"
HEADING = "FerroEHR viewer mockups"
ACCENT = ("#0f766e", "#2dd4bf")
INTRO = """<p>The redesigned FerroEHR operator UI, <code>ferroehr-viewer</code> (image <code>ghcr.io/ferrohealth/ferroehr-viewer</code>, environment prefix <code>FERROEHR_VIEWER__</code>), step 5 of the family console redesign after FerroPIX, FerroFED, FerroBRIDGE and FerroTERM. It is drawn only with the FerroUI library in the FerroEHR teal theme: every resting screen and every interactive state, {total} pages, each at 1440 px in light and dark and at 390 px in light, full page, at 2x. Grouped by screen: the resting screen first, its states after it.</p>
<p>Every page shows what the viewer has today, laid out anew. The two planned screens (<em>Planned</em>, FerroEHR #3661 and #3382) are a separate group at the end, and no live page links to them. Configuration is file mode only (family rule 13): the CDR's PostgreSQL holds clinical data and never configuration, so Configuration is a read-only view of <code>ferroehr.toml</code>, as the CDR serves it redacted, and of the viewer's own <code>ferroehr-viewer.toml</code>.</p>
<p>All data is synthetic or public. Subjects are synthetic hospital numbers (<code>P-nnnnn-nnn</code>) in the namespace <code>demo.hospital.mrn</code>, masked everywhere with the audited reveal; the people are Demo Operator, Demo Clinician, Demo Nurse and Demo Person 0417; archetype ids are public CKM ids; hosts are under <code>example.org</code>. Every nine-digit number fails the BSN eleven-proof, and the generator checks every page.</p>"""
