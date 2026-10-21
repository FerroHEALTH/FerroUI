# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The manifest of the library's reference templates, in the shape the
render, overflow and hover tools read: (number, slug, title, route, caption)."""

GROUPS = [
    ("templates", "Page templates", [
        ("T1", "list", "T1 List page", "/things", "Title and intro; one card with the table edge to edge and a header row. The summary strip and the toolbar appear only past 10 items or when the list has filters."),
        ("T1.1", "list-many", "T1 List page, many items", "/things", "Past 10 items: the summary chips filter, the toolbar finds, the pager closes the card."),
        ("T1.2", "list-tabs", "T1 List page with binder tabs", "/things", "Two real views of one list: the binder tabs carry title and count, the card is their panel."),
        ("T2", "detail", "T2 Detail page", "/things/{id}", "Breadcrumbs, the title with its state, one primary action plus More; binder tabs over one panel."),
        ("T3", "settings", "T3 Settings and reference page", "/settings/general", "Title, binder tabs (one section each, each a route), one full-width panel: a one-line intro with that tab's Edit, then setting rows with a readable label column and the value from 40 % of the panel."),
        ("T3.1", "settings-edit", "T3 Settings page, a tab in edit", "/settings/general/edit", "Edit works per tab: the tab carries a draft badge, controls replace values in place, the docked bar spans the content area."),
        ("T4", "about", "About (its own simple page)", "/about", "Two cards of facts and, where the product has one, a full-width card with the raw self-description and Copy and Download. No tabs, no sub-navigation."),
    ]),
]


def flat():
    for gid, gtitle, entries in GROUPS:
        for entry in entries:
            yield (gid, gtitle) + entry
