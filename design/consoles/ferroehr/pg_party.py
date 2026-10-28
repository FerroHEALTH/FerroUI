# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Parties: open a party by id and the demographic tag index, New party,
one party with its tabs, its editor and delete, relationships, and the
demographic contribution."""
import tpl
from common import ago
from common import (
    MENU_SEP, action_group, button, e, field, icon, icon_button, input_, link_button, menu_item, notice, page_heading, refusal, select,
    shell, table, tabbed, tabs, td, th, toast, toasts, when,
)
from data import ORG1, P1, REL1, SYSTEM_ID
from ehr import (
    tag_editor, diag_list,
    arch, path,
    at_time, codeview, coding, confirm_delete, editor, empty_line, facts, intro, reading, reading_section, section, sections, short, state, version_cell,
)
from pg_ehr import contribution_body, edit_dock, edit_shell, msg

KINDS = [("person", "Person"), ("organisation", "Organisation"), ("group", "Group"), ("agent", "Agent"), ("role", "Role")]


def parties_heading():
    return page_heading("Parties", actions=link_button("primary", "New party", "party-new.html", "plus"),
                        subtitle="People, organisations, groups, agents and roles, opened by their id.")


def open_form(kind="person", value="", invalid=None):
    inv = ' aria-invalid="true" aria-describedby="p-id-error"' if invalid else ""
    return (f'<form class="fui-card fui-panel" method="get" action="party.html"><div class="fui-transform-options">'
            f'{field("p-kind", "Kind", select("p-kind", KINDS, kind))}'
            f'{field("p-id", "Party uid", input_("p-id", value, "", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34) + " spellcheck=" + chr(34) + "false" + chr(34) + inv), error=invalid, cls="is-id")}'
            f'<div class="fui-transform-run">{button("secondary", "Open", "arrow-right", type_="submit")}</div></div></form>')


TAG_INDEX = [("follow-up", "2026-11-04", "Demo Person 0417", "party.html", "Person"), ("research-consent", "granted", "Demo Person 0417", "party.html", "Person"),
             ("migrated", "legacy-mpi-2025", "Demo General Hospital", "party-organisation.html", "Organisation"), ("verified", "2026-10-02", "Attending physician", "party-role.html", "Role")]


def tag_index(rows=TAG_INDEX):
    out = []
    for k, v, who, href, kind in rows:
        out.append(f'<tr><th scope="row" data-role="title"><code>{e(k)}</code></th>{td("Value", e(v), role="sub")}'
                   f'{td("On", "<a class=" + chr(34) + "fui-row-link" + chr(34) + " href=" + chr(34) + href + chr(34) + ">" + e(who) + "</a>", role="meta-q")}{td("Kind", kind, role="meta-2")}</tr>')
    return table("The demographic tag index", th("Key") + th("Value") + th("On") + th("Kind"), out)


def parties_page(top="", value="", invalid=None):
    note = notice("info", "info", "Parties follow the released RM 1.1.0 demographic model; the REST API for them is still in development, and FerroEHR adds the relationship routes it does not define yet.")
    c = tpl.card(tag_index(), label="Tags on parties", title="Tags on parties", count=4)
    return shell("Parties", "parties", parties_heading() + f'<div class="fui-list-page">{note}{open_form(value=value, invalid=invalid)}{top}{c}</div>')


def page_parties():
    return parties_page()


def page_parties_not_found():
    msg_ = ('No person has this uid. Try it as <a href="parties-not-found.html">Organisation</a>, <a href="parties-not-found.html">Group</a>, '
            '<a href="parties-not-found.html">Agent</a> or <a href="parties-not-found.html">Role</a>.')
    return parties_page("", "8a3e51c2-6d04-4f7b-9e18-2c5a0b7d3f99", invalid=msg_)


PERSON_JSON = """{
  "_type": "PERSON",
  "archetype_node_id": "openEHR-DEMOGRAPHIC-PERSON.person.v1",
  "name": { "_type": "DV_TEXT", "value": "PERSON" },
  "archetype_details": {
    "archetype_id": { "value": "openEHR-DEMOGRAPHIC-PERSON.person.v1" },
    "rm_version": "1.1.0"
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
    meta = section("", '<div class="fui-form-grid">' + field("pk", "Kind", select("pk", KINDS[:5], "person"), cls="fui-span-2") + "</div>")
    diags = {12: ("bad", "ARCHETYPE_ID", "/identities[0]/archetype_node_id: person_name is not an archetype id.")} if refused else None
    ed = editor(PERSON_JSON.replace("openEHR-DEMOGRAPHIC-PARTY_IDENTITY.person_name.v1", "person_name") if refused else PERSON_JSON,
                label="The new party", diags=diags, pos="Ln 12, Col 30" if refused else "Ln 1, Col 1", dirty=refused, tools=button("ghost", "Choose a file", "upload", small=True))
    if refused:
        ed = f'<div class="fui-editor-split">{ed}<section class="fui-card fui-panel" aria-label="Diagnostics"><div class="fui-panel-head"><h2 class="fui-card-title"><span>What the CDR refused <span class="mk-count">· 1</span></span></h2></div>{diag_list([("bad", "ARCHETYPE_ID", 12, 30, "/identities[0]/archetype_node_id: person_name is not an archetype id.")], current=12)}</section></div>'
    d = edit_dock(msg("circle-x", "Not created.") if refused else msg("file-plus", "Not created yet."), "Create this party", "parties.html")
    return edit_shell("New party", "parties", [("Parties", "parties.html")], "A starting document for a person.", top + sections(meta, card=True) + ed, d)


def page_party_new():
    return party_new()


def page_party_new_refused():
    rf = refusal("The CDR refused the party (422 Unprocessable Content), so nothing was created.", request="POST /ferroehr/rest/openehr/v1/demographic/person")
    return party_new(rf, refused=True)


PARTY_TABS = [("overview", "Overview", "party.html"), ("document", "Document", "party-document.html"), ("versions", "Versions", "party-versions.html", 2),
              ("relationships", "Relationships", "party-relationships.html", 1), ("tags", "Tags", "party-tags.html", 2)]


def party_header():
    more = [menu_item("Relate this party", "link", href="relationship-new.html"), menu_item("Copy the id", "copy"), MENU_SEP,
            menu_item("Delete party…", "trash-2", danger=True, href="party-delete.html")]
    ag = action_group(link_button("primary", "Edit", "party-edit.html", "pencil"), more, label="More actions for the party")
    return page_heading("Demo Person 0417", actions=ag, crumbs=[("Parties", "parties.html")], subtitle=f"Person · version 2 · {short(P1)}")


def party_page(tab="overview", panel=None, overlay=None):
    p = panel if panel is not None else party_overview()
    return shell("Demo Person 0417", "parties", party_header() + f'<div class="fui-stack">{tabbed(tabs("Views of this party", PARTY_TABS, tab), p)}</div>', overlay=overlay)


def party_overview():
    f = facts([("Kind", "Person"), ("Archetype", arch("openEHR-DEMOGRAPHIC-PERSON.person.v1")),
               ("Current version", f"<code>{P1[:8]}…::{SYSTEM_ID}::2</code>"), ("Identities", "1"), ("Relationships from this party", '<a href="party-relationships.html">1</a>')])
    r = reading(reading_section("Legal name", "openEHR-DEMOGRAPHIC-PARTY_IDENTITY.person_name.v1", [("Given name", "Demo"), ("Family name", "Person 0417")]))
    return sections(section("Facts", f), section("Identities", r))


def page_party():
    return party_page()


def page_party_document():
    return party_page("document", sections(section("", codeview("PERSON · version 2", PERSON_JSON, what="the party"))))


PARTY_VERSIONS = [("2", "2026-10-20T15:02", "Modification", "0199eb10…7d42"), ("1", "2026-09-11T11:30", "Creation", "01994a77…1c09")]


def party_versions_panel(reading=None, top=""):
    rows = []
    for i, (v, w, how, cu) in enumerate(PARTY_VERSIONS):
        href = "party-versions.html" if i == 0 else "party-version-1.html"
        cur = ' aria-current="true"' if reading == v else ""
        link = f'<a href="party-contribution.html"><code>{cu}</code></a>' if cu == "0199eb10…7d42" else f"<code>{cu}</code>"
        rows.append(f'<tr{cur}><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{version_cell(v, current=i == 0)}</a></th>{td("Change", how, role="sub")}'
                    f'{td("Committed by", "Demo Operator", role="meta-q")}{td("When", ago(w), role="meta-2")}{td("Contribution", link, role="wide")}</tr>')
    t = table("Versions", th("Version") + th("Change") + th("Committed by") + th("When") + th("Contribution"), rows)
    v = reading or "2"
    env = facts([("Lifecycle state", "Complete"), ("Preceding version", f"<code>{P1[:8]}…::{SYSTEM_ID}::1</code>" if v == "2" else '<span class="mk-dash">none</span>'), ("Signed", "No")])
    parts = [section("", top)] if top else []
    parts += [section("Versions", t + at_time("p-at", what="the party as it stood then", href="party-version-1.html"), count=2), section(f"Version {v}", env)]
    return sections(*parts)


def page_party_versions():
    return party_page("versions", party_versions_panel())


def page_party_version_1():
    top = notice("info", "history", f'You are reading version 1 of 2, committed {when("2026-09-11T11:30")}; it is read-only. <a href="party-versions.html">Read the latest</a>')
    return party_page("versions", party_versions_panel("1", top))


def page_party_relationships():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="relationship.html">Employment</a><span class="fui-row-sub"><code>employment</code></span></th>{td("Target", "<a href=" + chr(34) + "party-organisation.html" + chr(34) + ">Demo General Hospital</a>", role="sub")}{td("Target kind", "Organisation", role="meta-q")}</tr>']
    t = table("Relationships", th("Type") + th("Target") + th("Target kind"), rows)
    body = intro("The relationships this person is the source of; the CDR cannot list the ones that point at it.") + t
    return party_page("relationships", sections(section("Relationships", body, count=1, actions=link_button("secondary", "Relate this party", "relationship-new.html", "link", small=True))))


def page_party_tags():
    return party_page("tags", tag_editor([("follow-up", "2026-11-04", None), ("research-consent", "granted", None)], "party"))


PERSON_EDITED = PERSON_JSON.replace('"value": "Person 0417"', '"value": "Person-Visser 0417"')
FAMILY_LINE = PERSON_EDITED.split("\n").index(next(l for l in PERSON_EDITED.split("\n") if "Person-Visser" in l)) + 1
FAMILY_COL = next(l for l in PERSON_EDITED.split("\n") if "Person-Visser" in l).index("Person-Visser") + 1


def party_edit(top="", d=None):
    ed = editor(PERSON_EDITED, label="The party", pos=f"Ln {FAMILY_LINE}, Col {FAMILY_COL}", current=FAMILY_LINE)
    body = sections(section("Document", intro("The whole party as JSON, as New party takes it; it must keep at least one identity.") + ed), card=True)
    return edit_shell("Edit", "parties", [("Parties", "parties.html"), ("Demo Person 0417", "party.html")], "Version 2, opened 09:38.", top + body,
                      d or edit_dock(msg("pencil", "1 change: the family name."), "Save as version 3", "party.html"))


def page_party_edit():
    return party_edit()


def page_party_edit_conflict():
    rf = refusal("Another client saved a new version since you opened it, so the CDR refused the save (412).", request=f"PUT /ferroehr/rest/openehr/v1/demographic/person/{P1}::{SYSTEM_ID}::2",
                 issues=[("bad", "Error", "412 Precondition Failed", "If-Match names version 2; the latest is version 3.")])
    rf += f'<div class="fui-cluster">{button("secondary", "Reload version 3", "refresh-cw")}<span class="fui-small fui-muted">Reloading puts version 3 in the editor and replaces your edit.</span></div>'
    return party_edit(rf, edit_dock(msg("circle-x", "Not saved."), "Save", "party.html", disabled="Reload the latest version first"))


def page_party_delete():
    d = confirm_delete("del", "Delete Demo Person 0417?", "The party is marked deleted. Versions 1 and 2 stay readable, and its 1 relationship stays.", False, "Delete the party")
    return party_page(overlay=d)


def page_party_deleted():
    from ehr import state as _st
    more = [menu_item("Copy the id", "copy")]
    ag = action_group(link_button("primary", "Read version 2", "party-versions.html", "history"), more, label="More actions for the party")
    h = page_heading("Demo Person 0417", actions=ag, badge=_st("muted", "Deleted"), crumbs=[("Parties", "parties.html")],
                     subtitle=f"Person · deleted {when('2026-10-21T09:20', absolute=True)} by Demo Operator as version 3")
    rows = [f'<tr><th scope="row" data-role="title">{version_cell("3", deleted=True)}</th>{td("Change", "Deletion", role="sub")}{td("Committed by", "Demo Operator", role="meta-q")}{td("When", ago("2026-10-21T09:20"), role="meta-2")}</tr>']
    rows += [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{"party-versions.html" if v == "2" else "party-version-1.html"}">{version_cell(v)}</a></th>{td("Change", how, role="sub")}{td("Committed by", "Demo Operator", role="meta-q")}{td("When", ago(w), role="meta-2")}</tr>'
             for v, w, how, cu in PARTY_VERSIONS]
    t = table("Versions", th("Version") + th("Change") + th("Committed by") + th("When"), rows)
    body = sections(section("Versions", intro("The party is deleted; its versions stay readable, and its relationship stays.") + t, count=3))
    toast_ = toasts(toast("ok", "circle-check", "Party deleted", "Demo Person 0417 is marked deleted as version 3."))
    return shell("Demo Person 0417", "parties", h + f'<div class="fui-stack">{body}</div>', toast_html=toast_)


ENDS = facts([("Source", '<a href="party.html">Demo Person 0417</a> · Person, this party')])


def relationship_form(top="", refused=False, d=None):
    ends = section("From", ENDS)
    tk = field("r-tk", "Target kind", select("r-tk", KINDS, "organisation"), cls="fui-span-2")
    tid = field("r-tid", "Target party uid", input_("r-tid", ORG1, extra='class="fui-input fui-mono"' + (' aria-invalid="true"' if refused else "")),
                help_=None if refused else button("ghost", "Find a party…", "search", small=True, attrs='aria-haspopup="dialog"'),
                error="No organisation has this uid." if refused else None, cls="fui-span-2")
    grid = section("Relationship", '<div class="fui-form-grid">'
                   + field("r-type", "Type", input_("r-type", "employment", extra='class="fui-input fui-mono"'), help_="The type as a key, for example employment.", cls="fui-span-2")
                   + tk + tid
                   + field("r-from", "Valid from", input_("r-from", "2024-02-01", "YYYY-MM-DD", extra='inputmode="numeric"'), cls="fui-span-2")
                   + field("r-to", "Valid to (optional)", input_("r-to", "", "YYYY-MM-DD", extra='inputmode="numeric"'), cls="fui-span-2")
                   + "</div>")
    det = section("Details", intro("Optional extra details, as JSON.") + editor("", label="details", dirty=False))
    dk = d or edit_dock(msg("file-plus", "Not created yet."), "Create this relationship", "party-relationships.html")
    return edit_shell("New relationship", "parties", [("Parties", "parties.html"), ("Demo Person 0417", "party-relationships.html")], "From this party to another.",
                      top + sections(ends, grid, det, card=True), dk)


def page_relationship_new():
    return relationship_form()


def page_relationship_new_refused():
    rf = refusal("The CDR refused the relationship, so nothing was created.", request="POST /ferroehr/rest/extensions/v1/demographic/party_relationship",
                 issues=[("bad", "Error", "422 Unprocessable Content", "/target: no ORGANISATION with this uid.")])
    return relationship_form(rf, refused=True, d=edit_dock(msg("circle-x", "Not created."), "Create this relationship", "party-relationships.html", disabled="Fix the target first"))


REL_FACTS = [("Source", '<a href="party.html">Demo Person 0417</a> · Person'), ("Target", '<a href="party-organisation.html">Demo General Hospital</a> · Organisation'),
             ("Type", 'Employment <code class="fui-small">employment</code>'), ("Details", '<span class="mk-dash">none</span>'), ("Time validity", "From 2024-02-01")]


def page_relationship():
    more = [menu_item("Copy the id", "copy"), MENU_SEP, menu_item("Delete relationship…", "trash-2", danger=True)]
    ag = action_group(link_button("primary", "Edit", "relationship-edit.html", "pencil"), more, label="More actions for the relationship")
    h = page_heading("Employment", actions=ag, crumbs=[("Parties", "parties.html"), ("Demo Person 0417", "party-relationships.html")], subtitle=f"Relationship · version 1 · {short(REL1)}")
    rows = [f'<tr><th scope="row" data-role="title">{version_cell("1", current=True)}</th>{td("Change", "Creation", role="sub")}{td("Committed by", "Demo Operator", role="meta-q")}'
            f'{td("When", ago("2026-09-11T11:34"), role="meta-2")}{td("Contribution", "<code>01994a7c…2e18</code>", role="wide")}</tr>']
    t = table("Versions", th("Version") + th("Change") + th("Committed by") + th("When") + th("Contribution"), rows)
    body = sections(section("Facts", facts(REL_FACTS)), section("Versions", t + at_time("rel-at", what="the relationship as it stood then"), count=1), card=True)
    return shell("Employment", "parties", h + f'<div class="fui-stack">{body}</div>')


def page_party_contribution():
    h = page_heading("Contribution 0199eb10…7d42", subtitle="One change set, committed together.", crumbs=[("Parties", "parties.html"), ("Demo Person 0417", "party-versions.html")])
    return shell("Contribution", "parties", h + f'<div class="fui-stack">{contribution_body(demo=True)}</div>')


def relationship_edit(top="", d=None, changed=False):
    ends = section("Ends", intro("The two ends are fixed; other parties make another relationship.") + facts(REL_FACTS[:2]))
    grid = section("Relationship", '<div class="fui-form-grid">'
                   + field("re-ty", "Type", input_("re-ty", "employment", extra='class="fui-input fui-mono"'), cls="fui-span-2")
                   + field("re-from", "Valid from", input_("re-from", "2024-02-01", "YYYY-MM-DD", extra='inputmode="numeric"'), cls="fui-span-2")
                   + field("re-to", "Valid to (optional)", input_("re-to", "2026-12-31" if changed else "", "YYYY-MM-DD", extra='inputmode="numeric"'), cls="fui-span-2")
                   + "</div>")
    det = section("Details", intro("Optional extra details, as JSON.") + editor("", label="details", dirty=False))
    dk = d or edit_dock(msg("pencil", "No change yet."), "Save as version 2", "relationship.html", disabled="Change something first")
    return edit_shell("Edit", "parties", [("Parties", "parties.html"), ("Demo Person 0417", "party-relationships.html"), ("Employment", "relationship.html")], "Version 1, opened 09:38.",
                      top + sections(ends, grid, det, card=True), dk)


def page_relationship_edit():
    return relationship_edit()


def page_relationship_edit_conflict():
    rf = refusal("Another client saved a new version since you opened it, so the CDR refused the save (412).", request=f"PUT /ferroehr/rest/extensions/v1/demographic/party_relationship/{REL1}::{SYSTEM_ID}::1",
                 issues=[("bad", "Error", "412 Precondition Failed", "If-Match names version 1; the latest is version 2.")])
    rf += f'<div class="fui-cluster">{button("secondary", "Reload version 2", "refresh-cw")}<span class="fui-small fui-muted">Reloading shows version 2 here and replaces your edit.</span></div>'
    return relationship_edit(rf, edit_dock(msg("circle-x", "Not saved."), "Save", "relationship.html", disabled="Reload the latest version first"), changed=True)


def other_party(name, kind, uid, arch_id, ident):
    ag = action_group(link_button("primary", "Edit", "party-edit.html", "pencil"), [menu_item("Copy the id", "copy")], label=f"More actions for {name}")
    h = page_heading(name, actions=ag, crumbs=[("Parties", "parties.html")], subtitle=f"{kind} · version 1 · {short(uid)}")
    f = facts([("Kind", kind), ("Archetype", arch(arch_id)), ("Current version", f"<code>{uid[:8]}…::{SYSTEM_ID}::1</code>"), ("Identities", "1")])
    r = reading(reading_section(ident[0], ident[1], ident[2]))
    return shell(name, "parties", h + f'<div class="fui-stack">{sections(section("Facts", f), section("Identities", r), card=True)}</div>')


def page_party_organisation():
    return other_party("Demo General Hospital", "Organisation", ORG1, "openEHR-DEMOGRAPHIC-ORGANISATION.organisation.v1",
                       ("Name", "openEHR-DEMOGRAPHIC-PARTY_IDENTITY.organisation_name.v1", [("Name", "Demo General Hospital")]))


def page_party_role():
    return other_party("Attending physician", "Role", "5d2a7c91-0e43-4b18-a6f5-3c9e1b8d0f27", "openEHR-DEMOGRAPHIC-ROLE.professional_role.v1",
                       ("Role name", "openEHR-DEMOGRAPHIC-PARTY_IDENTITY.role_name.v1", [("Name", "Attending physician")]))
