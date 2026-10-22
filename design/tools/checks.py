# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The checks build.py runs over the markup of every page, before any
browser: each returns a list of findings, empty when the page passes."""
import re

# The nouns a count can name on a page; a count of one never takes the
# plural. One list for the family, so every console is held to the same rule.
PLURALS = (
    "changes", "clauses", "codes", "compositions", "concepts", "contributions", "days", "designations", "EHRs",
    "endpoints", "errors", "filters", "findings", "folders", "items", "lines", "mappings", "maps", "matches",
    "members", "minutes", "nodes", "operations", "organisations", "parameters", "properties", "queries",
    "records", "replicas", "roots", "rows", "scopes", "seconds", "separations", "services", "sets",
    "subscriptions", "systems", "tags", "templates", "versions", "warnings", "writes",
)

INTRO_CLASSES = ("fui-page-subtitle", "fui-settings-panel-intro", "fui-small fui-muted fui-section-intro")


# A path on one machine (a home directory, a temporary or private
# directory) in a page source: a page that reads it builds only there.
LOCAL_PATH = re.compile(r"""["'](?:/Users/|/home/|/private/|/tmp/|/var/folders/|~/|[A-Za-z]:\\\\)""")


def local_paths(text):
    """The line numbers of every quoted machine-local path in a source."""
    return [n for n, line in enumerate(text.splitlines(), 1) if LOCAL_PATH.search(line)]


def bsn_valid(digits):
    """The BSN eleven-proof: the weighted digit sum is a multiple of 11."""
    weights = [9, 8, 7, 6, 5, 4, 3, 2, -1]
    return sum(w * int(d) for w, d in zip(weights, digits)) % 11 == 0


def text_of(html):
    return re.sub(r"<[^>]+>", " ", html)


def synthetic(html):
    """Every run of nine digits, spaced or not, fails the eleven-proof, so no
    page carries a plausible BSN."""
    found = []
    for m in re.finditer(r"(?<![\d.])(\d[\d ]{8,12}\d)(?![\d.])", text_of(html)):
        digits = m.group(1).replace(" ", "")
        if len(digits) == 9 and bsn_valid(digits):
            found.append(f"{m.group(1)} passes the BSN eleven-proof")
    return found


def links(html, slugs):
    """Every control that names a page leads to one of the set's pages."""
    return [f"links to {h}.html, which is not a page" for h in sorted(set(re.findall(r'href="([a-z0-9-]+)\.html', html)) - set(slugs))]


def dead(html):
    """No link that looks like a button leads nowhere: href="#" is allowed
    on one only for an external target named as such (rel=...)."""
    return [f"a button-looking link leads nowhere: {m.group(0)[:120]}"
            for m in re.finditer(r'<a [^>]*href="#"[^>]*>', html)
            if "fui-button" in m.group(0) and "rel=" not in m.group(0)]


def plural(html):
    """A count of one never takes a plural noun."""
    pattern = r"(?<![\d.,])1 (" + "|".join(PLURALS) + r")\b"
    return [f"'{m.group(0)}'" for m in re.finditer(pattern, text_of(html))]


def intro(html):
    """A page's intro and a section's intro are one sentence; a settings
    panel's intro is at most two, what the tab governs and then how a change
    takes effect (T3)."""
    found = []
    pattern = r'<p class="(' + "|".join(re.escape(c) for c in INTRO_CLASSES) + r')">(.*?)</p>'
    for m in re.finditer(pattern, html, re.S):
        text = re.sub(r"<[^>]+>", "", m.group(2)).strip()
        n = len(re.findall(r"[.!?](?:\s|$)", text))
        if n > (2 if m.group(1) == "fui-settings-panel-intro" else 1):
            found.append(f"intro of {n} sentences: {text[:90]}")
    return found


def reachable(entries, html_of, planned_group):
    """No live page links to a planned or proposed screen: those are a group
    of their own, for review, and never a dead end in the console."""
    planned = {e[3] for e in entries if e[0] == planned_group}
    found = []
    for e in entries:
        if e[0] == planned_group:
            continue
        for href in sorted(set(re.findall(r'href="([a-z0-9-]+)\.html', html_of[e[3]])) & planned):
            found.append((e[3], f"links to the planned screen {href}.html"))
    return found
