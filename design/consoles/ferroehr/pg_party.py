# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Parties: open a party by id and the demographic tag index, New party,
one party with its tabs, its editor and delete, relationships, and the
demographic contribution."""
import tpl
from common import ago
from common import (
    MENU_SEP, action_group, button, e, field, icon, icon_button, input_, link_button, menu_item, notice, page_heading, refusal, select,
    shell, table, tabbed, tabs, td, th, when,
)
from data import ORG1, P1, REL1, SYSTEM_ID
from ehr import (
    arch, path,
    at_time, codeview, coding, confirm_delete, editor, empty_line, facts, intro, reading, reading_section, section, sections, short, state, version_cell,
)
from pg_ehr import contribution_body, edit_dock, edit_shell, msg

KINDS = [("person", "Person"), ("organisation", "Organisation"), ("group", "Group"), ("agent", "Agent"), ("role", "Role"), ("relationship", "Relationship")]


def parties_heading():
    return page_heading("Parties", actions=link_button("primary", "New party", "party-new.html", "plus"),
                        subtitle="People, organisations, groups, agents and roles, opened by their id.")


def open_form(kind="person", value="", invalid=None):
    inv = ' aria-invalid="true"' if invalid else ""
    return (f'<form class="fui-list-toolbar mk-open-form" method="get" action="party.html"><div class="fui-list-toolbar-start">'
            f'{field("p-kind", "Kind", select("p-kind", KINDS, kind))}'
            f'{field("p-id", "Id", input_("p-id", value, "A versioned object uid or a version uid", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34) + " spellcheck=" + chr(34) + "false" + chr(34) + inv))}'
            f'<div class="fui-field"><span class="fui-label" aria-hidden="true">&nbsp;</span>{button("secondary", "Open", "arrow-right", type_="submit")}</div></div></form>')


TAG_INDEX = [("follow-up", "2026-11-04", "Demo Person 0417", "party.html", "Person"), ("research-consent", "granted", "Demo Person 0417", "party.html", "Person"),
             ("migrated", "legacy-mpi-2025", "Demo General Hospital", "party.html", "Organisation"), ("verified", "2026-10-02", "Attending physician", "party.html", "Role")]


def tag_index(rows=TAG_INDEX):
    out = []
    for k, v, who, href, kind in rows:
        out.append(f'<tr><th scope="row" data-role="title"><code>{e(k)}</code></th>{td("Value", e(v), role="sub")}'
                   f'{td("On", "<a class=" + chr(34) + "fui-row-link" + chr(34) + " href=" + chr(34) + href + chr(34) + ">" + e(who) + "</a>", role="meta-q")}{td("Kind", kind, role="meta-2")}</tr>')
    return table("The demographic tag index", th("Key") + th("Value") + th("On") + th("Kind"), out)


def parties_page(top="", value=""):
    note = notice("info", "info", "Parties follow the released RM 1.1.0 demographic model; the REST API for them is still in development, and FerroEHR adds the relationship routes it does not define yet.")
    c = tpl.card(tag_index(), label="Tagged parties", title="Tagged parties", count=4)
    return shell("Parties", "parties", parties_heading() + f'<div class="fui-list-page">{note}{open_form(value=value)}{top}{c}</div>')


def page_parties():
    return parties_page()


def page_parties_not_found():
    top = f'<p class="fui-small" role="status">{icon("search-x", "14")} No person has the id <code>8a3e51c2-6d04-4f7b-9e18-2c5a0b7d3f99</code>; it may be another kind.</p>'
    return parties_page(top, "8a3e51c2-6d04-4f7b-9e18-2c5a0b7d3f99")


PERSON_JSON = """{
  "_type": "PERSON",
  "archetype_node_id": "openEHR-DEMOGRAPHIC-PERSON.person.v1",
  "name": { "_type": "DV_TEXT", "value": "PERSON" },
  "archetype_details": {
    "archetype_id": { "value": "openEHR-DEMOGRAPHIC-PERSON.person.v1" },
    "rm_version": "1.2.0"
  },
  "identities": [
    {
      "_type": "PARTY_IDENTITY",
      "archetype_node_id": "openEHR-DEMOGRAPHIC-PARTY_IDENTITY.person_name.v1",
      "name": { "_type": "DV_TEXT", "value": "legal name" },
      "details": {
        "_type": "ITEM_TREE",
        "archetype_node_id": "at0001",
        "name": { "_type": "DV_TEXT", "value": "Details" },
        "items": [
          { "_type": "ELEMENT", "archetype_node_id": "at0002", "name": { "value": "Given name" }, "value": { "_type": "DV_TEXT", "value": "Demo" } },
          { "_type": "ELEMENT", "archetype_node_id": "at0003", "name": { "value": "Family name" }, "value": { "_type": "DV_TEXT", "value": "Person 0417" } }
        ]
      }
    }
  ]
}"""


def party_new(top="", refused=False):
    meta = section("", '<div class="fui-form-grid">' + field("pk", "Kind", select("pk", KINDS[:5], "person"), help_="A role also needs a performer, the party that plays it.", cls="fui-span-2") + "</div>")
    diags = {11: ("bad", "VALIDATION", "/identities[0]/archetype_node_id: not an archetype id.")} if refused else None
    ed = editor(PERSON_JSON.replace("openEHR-DEMOGRAPHIC-PARTY_IDENTITY.person_name.v1", "person_name") if refused else PERSON_JSON,
                label="The new party", diags=diags, pos="Ln 11, Col 30", tools=button("ghost", "Choose a file", "upload", small=True))
    d = edit_dock(msg("circle-x", "Not created: 1 error to fix.") if refused else "", "Create this party", "parties.html")
    return edit_shell("New party", "parties", [("Parties", "parties.html")], "The smallest document a person accepts; replace the archetype ids with your own.", top + sections(meta, card=True) + ed, d)


def page_party_new():
    return party_new()


def page_party_new_refused():
    rf = refusal("The CDR refused the party, in its own words below.", request="POST /ferroehr/rest/openehr/v1/demographic/person",
                 issues=[("bad", "error", "422 Unprocessable", "/identities[0]/archetype_node_id: person_name is not an archetype id.")])
    return party_new(rf, refused=True)


PARTY_TABS = [("overview", "Overview", "party.html"), ("document", "Document", "party-document.html"), ("versions", "Versions", "party-versions.html", 2),
              ("relationships", "Relationships", "party-relationships.html", 1), ("tags", "Tags", "party-tags.html", 2)]


def party_header():
    more = [menu_item("Relate this party", "link", href="relationship-new.html"), menu_item("Copy uid", "copy"), MENU_SEP,
            menu_item("Delete party…", "trash-2", danger=True, href="party-delete.html")]
    ag = action_group(link_button("primary", "Edit", "party-edit.html", "pencil"), more, label="More actions for the party")
    return page_heading("Demo Person 0417", actions=ag, crumbs=[("Parties", "parties.html")], subtitle=f"Person · version 2 · {short(P1)}")


def party_page(tab="overview", panel=None, overlay=None):
    p = panel if panel is not None else party_overview()
    return shell("Demo Person 0417", "parties", party_header() + f'<div class="fui-stack">{tabbed(tabs("Views of this party", PARTY_TABS, tab), p)}</div>', overlay=overlay)


def party_overview():
    f = facts([("Kind", "Person"), ("Type", "<code>PERSON</code>, the RM's name for its kind"), ("Archetype", arch("openEHR-DEMOGRAPHIC-PERSON.person.v1")),
               ("Current version", f"<code>{P1[:8]}…::{SYSTEM_ID}::2</code>"), ("Identities", "1"), ("Relationships it is the source of", "1")])
    r = reading(reading_section("Legal name", "openEHR-DEMOGRAPHIC-PARTY_IDENTITY.person_name.v1", [("Given name", "Demo"), ("Family name", "Person 0417")]))
    return sections(section("Facts", f), section("Identities", r))


def page_party():
    return party_page()


def page_party_document():
    return party_page("document", sections(section("", codeview("PERSON · version 2", PERSON_JSON, what="the party"))))


def page_party_versions():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="party-versions.html">{version_cell("2", current=True)}</a></th>{td("Change", "Modification", role="sub")}{td("Committed by", "Demo Operator", role="meta-q")}{td("When", ago("2026-10-20T15:02"), role="meta-2")}{td("Contribution", "<a href=" + chr(34) + "party-contribution.html" + chr(34) + "><code>0199eb10…7d42</code></a>", role="wide")}</tr>',
            f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="party-versions.html">{version_cell("1")}</a></th>{td("Change", "Creation", role="sub")}{td("Committed by", "Demo Operator", role="meta-q")}{td("When", ago("2026-09-11T11:30"), role="meta-2")}{td("Contribution", "<a href=" + chr(34) + "party-contribution.html" + chr(34) + "><code>01994a77…1c09</code></a>", role="wide")}</tr>']
    t = table("Versions", th("Version") + th("Change") + th("Committed by") + th("When") + th("Contribution"), rows)
    env = facts([("Lifecycle state", "Complete"), ("Preceding version", f"<code>{P1[:8]}…::{SYSTEM_ID}::1</code>"), ("Signed", "No")])
    return party_page("versions", sections(section("Versions", t + at_time("p-at", what="the party as it stood then"), count=2), section("Version 2", env)))


def page_party_relationships():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="relationship.html">employment</a></th>{td("Target", "<a href=" + chr(34) + "party.html" + chr(34) + ">Demo General Hospital</a>", role="sub")}{td("Target kind", "Organisation", role="meta-q")}</tr>']
    t = table("Relationships", th("Type") + th("Target") + th("Target kind"), rows)
    body = intro("The relationships this person is the source of; the CDR cannot list the ones that point at it.") + t
    return party_page("relationships", sections(section("Relationships", body, count=1, actions=link_button("secondary", "Relate this party", "relationship-new.html", "link", small=True))))


def page_party_tags():
    rows = [f'<tr><th scope="row" data-role="title"><code>follow-up</code></th>{td("Value", "2026-11-04", role="sub")}{td("Target path", "<span class=" + chr(34) + "mk-dash" + chr(34) + ">the whole party</span>", role="meta-q")}{td("", icon_button("x", "Remove the tag follow-up"), "fui-row-menu", role="menu")}</tr>',
            f'<tr><th scope="row" data-role="title"><code>research-consent</code></th>{td("Value", "granted", role="sub")}{td("Target path", "<span class=" + chr(34) + "mk-dash" + chr(34) + ">the whole party</span>", role="meta-q")}{td("", icon_button("x", "Remove the tag research-consent"), "fui-row-menu", role="menu")}</tr>']
    t = table("Tags", th("Key") + th("Value") + th("Target path") + th('<span class="fui-sr-only">Remove</span>'), rows)
    new = (f'<div class="fui-clause-filter">{field("tk", "Key", input_("tk", "", "flag", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
           f'{field("tv", "Value", input_("tv", "", "Optional"))}{field("tp", "Target path", input_("tp", "", "Optional", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
           f'{button("secondary", "Set the tag", "tags")}</div>')
    return party_page("tags", sections(section("Tags", intro("Tags stay on the party across its versions and are not versioned.") + t, count=2),
                                       section("Set a tag", intro("Saving re-sends the whole collection, so reload first if another client may have tagged it.") + new)))


def page_party_edit():
    ids = editor(PERSON_JSON[PERSON_JSON.index('[\n    {'):PERSON_JSON.rindex("]") + 1], label="identities", lang_word="Canonical JSON · identities", pos="Ln 14, Col 120")
    det = editor("", label="details", lang_word="Canonical JSON · details", dirty=False)
    body = sections(section("Identities", intro("At least one identity; the RM requires it.") + ids),
                    section("Details", intro("Optional, empty to remove it; every other attribute goes back as the CDR served it.") + det), card=True)
    return edit_shell("Edit", "parties", [("Parties", "parties.html"), ("Demo Person 0417", "party.html")], "Version 2, opened 09:38.", body,
                      edit_dock(msg("pencil", "1 change: the family name."), "Save as version 3", "party.html"))


def page_party_delete():
    d = confirm_delete("del", "Delete Demo Person 0417?", "Marks the party deleted with a new version 3; it stops resolving as current, and versions 1 and 2 stay readable in Versions.", False, "Delete the party")
    return party_page(overlay=d)


def page_relationship_new():
    body = sections(section("", '<div class="fui-form-grid">'
                            + field("r-src", "Source", input_("r-src", "Demo Person 0417", extra="readonly"), help_="This party, as the source.", cls="fui-span-2")
                            + field("r-type", "Type", input_("r-type", "employment"), help_="Stored as the relationship's name.", cls="fui-span-2")
                            + field("r-tk", "Target kind", select("r-tk", KINDS[:5], "organisation"), cls="fui-span-2")
                            + field("r-tid", "Target id", input_("r-tid", ORG1, extra='class="fui-input fui-mono"'), cls="fui-span-2")
                            + "</div>"),
                    section("Details", intro("Optional, as an ITEM_STRUCTURE in canonical JSON.") + editor("", label="details", dirty=False)), card=True)
    return edit_shell("New relationship", "parties", [("Parties", "parties.html"), ("Demo Person 0417", "party-relationships.html")], "From this party to another.", body,
                      edit_dock("", "Create this relationship", "party-relationships.html"))


REL_TABS = [("overview", "Overview", "relationship.html"), ("versions", "Versions", "relationship-versions.html", 1)]


def rel_page(tab, panel):
    more = [menu_item("Copy uid", "copy"), MENU_SEP, menu_item("Delete relationship…", "trash-2", danger=True)]
    ag = action_group(link_button("primary", "Edit", "relationship-edit.html", "pencil"), more, label="More actions for the relationship")
    h = page_heading("employment", actions=ag, crumbs=[("Parties", "parties.html"), ("Demo Person 0417", "party-relationships.html")], subtitle=f"Relationship · version 1 · {short(REL1)}")
    return shell("employment", "parties", h + f'<div class="fui-stack">{tabbed(tabs("Views of this relationship", REL_TABS, tab), panel)}</div>')


def page_relationship():
    f = facts([("Source", '<a href="party.html">Demo Person 0417</a> · Person'), ("Target", '<a href="party.html">Demo General Hospital</a> · Organisation'),
               ("Type", "employment"), ("Details", '<span class="mk-dash">none</span>'), ("Time validity", "From 2024-02-01")])
    note = section("", '<p class="fui-small fui-muted">Relationships follow the released RM 1.1.0 demographic model; the REST API does not define their routes yet, so FerroEHR adds them.</p>')
    return rel_page("overview", sections(section("Facts", f), note))


def page_relationship_versions():
    rows = [f'<tr><th scope="row" data-role="title">{version_cell("1", current=True)}</th>{td("Change", "Creation", role="sub")}{td("Committed by", "Demo Operator", role="meta-q")}{td("When", ago("2026-09-11T11:34"), role="meta-2")}</tr>']
    return rel_page("versions", sections(section("Versions", table("Versions", th("Version") + th("Change") + th("Committed by") + th("When"), rows), count=1)))


def page_party_contribution():
    h = page_heading("Contribution 0199eb10…7d42", subtitle="One change set, committed together.", crumbs=[("Parties", "parties.html"), ("Demo Person 0417", "party-versions.html")])
    return shell("Contribution", "parties", h + f'<div class="fui-stack">{contribution_body(demo=True)}</div>')


def page_relationship_edit():
    body = sections(section("", '<div class="fui-form-grid">'
                            + field("re-s", "Source", input_("re-s", "Demo Person 0417", extra="readonly"), cls="fui-span-2")
                            + field("re-t", "Target", input_("re-t", "Demo General Hospital", extra="readonly"), help_="The ends are fixed: other parties make another relationship.", cls="fui-span-2")
                            + field("re-ty", "Type", input_("re-ty", "employment"), cls="fui-span-2") + "</div>"),
                    section("Details", intro("Optional, as an ITEM_STRUCTURE in canonical JSON.") + editor("", label="details", dirty=False)), card=True)
    return edit_shell("Edit", "parties", [("Parties", "parties.html"), ("employment", "relationship.html")], "Version 1, opened 09:38.", body,
                      edit_dock("", "Save as version 2", "relationship.html"))
