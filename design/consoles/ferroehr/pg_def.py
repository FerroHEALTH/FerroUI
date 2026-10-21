# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Definitions: Templates (the list with its two families, Upload, one
template with its tabs and its delete) and the Terminology workbench."""
import tpl
from common import ago
from common import (
    MENU_SEP, action_group, badge, button, busy_button, e, field, icon, icon_button, input_, link_button, menu, menu_item, notice, page_heading,
    refusal, select, shell, table, tabbed, tabs, td, th, when,
)
from data import ADL2, TEMPLATES
from ehr import (
    dialog,
    arch, path,
    codeview, coding, confirm_delete, diag_list, editor, empty_line, facts, intro, load_report, quiet, rm_type, section, sections, state, tree, tree_item,
)
from pg_ehr import edit_dock, edit_shell, msg

# ---- Templates ----


def templates_heading():
    return page_heading("Templates", actions=link_button("primary", "Upload a template", "template-upload.html", "upload"),
                        subtitle="The operational templates the CDR validates compositions against.")


def tmpl_row_menu(open_=False, name=""):
    items = [menu_item("Build a query", "square-terminal", href="query.html"), MENU_SEP, menu_item("Delete…", "trash-2", danger=True, href="template-delete.html")]
    m = menu(items, label=f"Actions for {name}", sheet=True) if open_ else ""
    return (f'<span class="fui-anchor"><button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
            f'aria-label="Actions for {e(name)}" title="Actions">{icon("ellipsis", "16")}</button>{m}</span>')


def tmpl_rows(items=TEMPLATES, open_row=None):
    rows = []
    for i, (tid, concept, root, n, up) in enumerate(items):
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="template.html">{e(tid)}</a><span class="fui-row-sub">{e(concept)}</span></th>'
                    + td("Root archetype", arch(root), role="sub") + td("Compositions", f"{n:,}", "fui-num", role="meta-q")
                    + td("Uploaded", ago(up), role="meta-2") + td("", tmpl_row_menu(open_row == i, tid), "fui-row-menu", role="menu") + "</tr>")
    return table("ADL 1.4 templates", th("Template") + th("Root archetype") + th("Compositions", num=True) + th("Uploaded") + th('<span class="fui-sr-only">Actions</span>'), rows)


def adl2_rows():
    rows = []
    for hrid, concept, root, n, up in ADL2:
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="template-adl2.html">{e(concept)}</a><span class="fui-row-sub"><code>{e(hrid)}</code></span></th>'
                    + td("Root archetype", arch(root), role="sub") + td("Compositions", str(n), "fui-num", role="meta-q")
                    + td("Uploaded", ago(up), role="meta-2") + td("", tmpl_row_menu(False, concept), "fui-row-menu", role="menu") + "</tr>")
    return table("ADL 2 templates", th("Template") + th("Root archetype") + th("Compositions", num=True) + th("Uploaded") + th('<span class="fui-sr-only">More actions</span>'), rows)


def tmpl_tabs(cur, n14=12, n2=3):
    # n2 is the ADL 2 count after any filter.
    return tabs("Template families", [("adl14", "ADL 1.4", "templates.html", n14), ("adl2", "ADL 2", "templates-adl2.html", n2)], cur)


def tmpl_page(cur="adl14", body=None, value="", chips=(), n14=12, n2=3):
    tb = tpl.toolbar("Id, concept or archetype", value)
    b = body if body is not None else (tmpl_rows() if cur == "adl14" else adl2_rows())
    c = tpl.card(b, tabs_html=tmpl_tabs(cur, n14, n2), label="Templates")
    return shell("Templates", "templates", templates_heading() + tpl.list_body("", tb, tpl.active(chips), c))


def page_templates():
    return tmpl_page()


def page_templates_adl2():
    return tmpl_page("adl2")


def page_templates_filtered():
    lst = [t for t in TEMPLATES if "list" in t[0].lower()]
    return tmpl_page(body=tmpl_rows(lst), value="", chips=["Contains: list"], n14=len(lst), n2=0)


def page_templates_row_menu():
    return tmpl_page(body=tmpl_rows(open_row=1))


def page_templates_empty():
    c = tpl.card(empty_line("No template is uploaded yet."), label="Templates", title="Templates", count="0")
    return shell("Templates", "templates", templates_heading() + tpl.list_body("", "", "", c))


OPT = """<?xml version="1.0" encoding="UTF-8"?>
<template xmlns="http://schemas.openehr.org/v1">
  <language>
    <terminology_id><value>ISO_639-1</value></terminology_id>
    <code_string>en</code_string>
  </language>
  <description>
    <original_author id="name">Demo Modeller</original_author>
    <lifecycle_state>published</lifecycle_state>
  </description>
  <uid><value>5f2c8e41-0b7d-4a93-9e16-3d8a7c0f2b54</value></uid>
  <template_id><value>Vital signs encounter.v1</value></template_id>
  <concept>Vital signs encounter</concept>
  <definition>
    <rm_type_name>COMPOSITION</rm_type_name>
    <node_id/>
    <archetype_id><value>openEHR-EHR-COMPOSITION.encounter.v1</value></archetype_id>
  </definition>
</template>"""


def template_upload(top="", adl2=False, refused=False):
    fam = (f'<div role="group" aria-label="Family" class="fui-segmented"><a href="template-upload.html" class="fui-segment"{"" if adl2 else " aria-current=" + chr(34) + "true" + chr(34)}>ADL 1.4</a>'
           f'<a href="template-upload-adl2-refused.html" class="fui-segment"{" aria-current=" + chr(34) + "true" + chr(34) if adl2 else ""}>ADL 2</a></div>')
    meta = section("", f'<div class="fui-cluster"><div class="fui-field"><span class="fui-label">Family</span>{fam}</div></div>')
    if adl2:
        text = "archetype (adl_version=2.4.0; rm_release=1.2.0)\n    openEHR-EHR-COMPOSITION.lab_order.v0.5.0\n\nspecialise\n    openEHR-EHR-COMPOSITION.request.v1\n\nlanguage\n    original_language = <[ISO_639-1::en]>\n\ndefinition\n    COMPOSITION[id1.1] matches {\n        category matches {[at17]}\n    }"
        diags = {2: ("bad", "VARDT", "0.5.0 is not a release the CDR holds."), 13: ("bad", "VCORMT", "at17 is not defined in the terminology of this artefact.")} if refused else None
        ed = editor(text, lang="text", lang_word="ADL 2", label="The ADL 2 source", diags=diags, pos="Ln 13, Col 29", tools=button("ghost", "Choose a file", "upload", small=True))
        rows = [("bad", "VARDT", 2, 5, "Version 0.5.0 is newer than the 0.4.0 the CDR holds; a specialisation must name a held parent release."),
                ("bad", "VCORMT", 13, 29, "The code at17 is used but not defined in the terminology section.")]
    else:
        diags = {11: ("bad", "OPT-UID", "uid is missing its value.")} if refused else None
        ed = editor(OPT.replace("<uid><value>5f2c8e41-0b7d-4a93-9e16-3d8a7c0f2b54</value></uid>", "<uid></uid>") if refused else OPT, lang="xml", lang_word="OPT XML",
                    label="The operational template", diags=diags, pos="Ln 11, Col 8", tools=button("ghost", "Choose a file", "upload", small=True))
        rows = [("bad", "OPT-UID", 11, 8, "The template's uid has no value.")]
    if refused:
        body = f'<div class="fui-editor-split">{ed}<section class="fui-card fui-panel" aria-label="Diagnostics"><div class="fui-panel-head"><h2 class="fui-card-title"><span>What the CDR refused <span class="mk-count">· {len(rows)}</span></span></h2></div>{diag_list(rows)}</section></div>'
        d = edit_dock(msg("circle-x", f"Not uploaded: {len(rows)} error{'s' if len(rows) != 1 else ''} to fix."), "Upload this template", "templates.html")
    else:
        body = ed
        d = edit_dock("", "Upload this template", "templates.html")
    return edit_shell("Upload a template", "templates", [("Templates", "templates.html")], "Choose a file or paste the source; the CDR validates it on upload.", top + sections(meta, card=True) + body, d)


def page_template_upload():
    return template_upload()


def page_template_upload_refused():
    rf = refusal("The CDR refused the template, so nothing was uploaded.", request="POST /ferroehr/rest/openehr/v1/definition/template/adl1.4",
                 issues=[("bad", "error", "400 Bad Request", "1 validation error, listed beside the source.")])
    return template_upload(rf, refused=True)


def page_template_upload_adl2_refused():
    rf = refusal("The CDR refused the source, so nothing was uploaded.", request="POST /ferroehr/rest/openehr/v1/definition/template/adl2",
                 issues=[("bad", "error", "400 Bad Request", "2 validation errors, listed beside the source.")])
    return template_upload(rf, adl2=True, refused=True)


TMPL_TABS = [("overview", "Overview", "template.html"), ("paths", "Paths", "template-paths.html"), ("source", "Source", "template-source.html"), ("example", "Example", "template-example.html")]


def version_select(open_=False):
    """R29: the release version as a select button, its listbox under it."""
    from parts import listbox, select_button
    lb = listbox([("1.1.0", "1.1.0", "Latest"), ("1.0.0", "1.0.0", None)], "1.1.0", id_="ver-list", label="Version")
    btn = select_button("ver", "1.1.0 (latest of 2)", open_, lb).replace('id="ver" class=', 'id="ver" aria-label="Version: 1.1.0, the latest of 2" title="The release versions the CDR holds for this HRID" class=', 1)
    if not open_:
        btn = btn.replace('<button type="button" id="ver"', '<a href="template-adl2-versions.html" role="button" id="ver"', 1).replace("</button></span>", "</a></span>", 1)
    return f'<div class="fui-cluster fui-title-select"><label class="fui-label" for="ver">Version</label>{btn}</div>'


def tmpl_header(adl2=False, ver_open=False):
    more = [menu_item("Download", "download"), menu_item("Copy id", "copy"), MENU_SEP, menu_item("Delete…", "trash-2", danger=True, href="template-delete.html")]
    ag = action_group(link_button("primary", "Build a query", "query.html", "square-terminal"), more, label="More actions for the template")
    if adl2:
        return page_heading("Vital signs encounter", actions=ag, crumbs=[("Templates", "templates-adl2.html")],
                            subtitle=arch("openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.1.0") + " · ADL 2", after=version_select(ver_open))
    return page_heading("Vital signs encounter.v1", actions=ag, crumbs=[("Templates", "templates.html")], subtitle="Vital signs encounter · ADL 1.4")


def tmpl_detail(tab="overview", panel=None, overlay=None, adl2=False, ver_open=False):
    p = panel if panel is not None else tmpl_overview(adl2)
    tl = TMPL_TABS
    if adl2:
        tl = [("overview", "Overview", "template-adl2.html"), ("paths", "Paths", "template-adl2-paths.html"), ("source", "Source", "template-adl2-source.html"),
              ("aom2", "Compiled (JSON)", "template-adl2-aom.html"), ("example", "Example", "template-adl2-example.html")]
    return shell("Vital signs encounter", "templates", tmpl_header(adl2, ver_open) + f'<div class="fui-stack">{tabbed(tabs("Views of this template", tl, tab), p)}</div>', overlay=overlay)


def tmpl_overview(adl2=False):
    if adl2:
        f = facts([("Concept", "Vital signs encounter"), ("Root archetype", arch("openEHR-EHR-COMPOSITION.encounter.v1")),
                   ("Original language", "English (en)"), ("Compositions", "0")])
        return sections(section("Identity", f), section("", '<p class="fui-small fui-muted">Paths are built by the viewer; the CDR has no web template for ADL 2.</p>'))
    f = facts([("Concept", "Vital signs encounter"), ("Template version", "1"), ("Default language", "English (en)"), ("Languages", "English (en), Dutch (nl)"),
               ("UID", "<code>5f2c8e41-0b7d-4a93-9e16-3d8a7c0f2b54</code>"), ("Root archetype", arch("openEHR-EHR-COMPOSITION.encounter.v1")), ("Uploaded", when("2026-02-11T09:00"))])
    use = facts([("Compositions committed under it", "1,284")])
    return sections(section("Identity", f), section("Use", use))


def page_template():
    return tmpl_detail()


def path_tree(selected="systolic"):
    def leaf(label, t, key):
        return tree_item(label, extra=rm_type(t), leaf=True, selected=key == selected, href="template-paths-node.html" if key == "position" else "template-paths.html")
    dev = tree_item("Device", extra=rm_type("CLUSTER"), children=[])
    bp = tree_item("Blood pressure", extra=rm_type("OBSERVATION"), expanded=True, on_path=True, children=[
        leaf("Systolic", "DV_QUANTITY", "systolic"), leaf("Diastolic", "DV_QUANTITY", "diastolic"), leaf("Position", "DV_CODED_TEXT", "position"), leaf("Cuff size", "DV_CODED_TEXT", "cuff"), dev])
    pulse = tree_item("Pulse", extra=rm_type("OBSERVATION"), children=[])
    temp = tree_item("Body temperature", extra=rm_type("OBSERVATION"), children=[])
    spo2 = tree_item("Pulse oximetry", extra=rm_type("OBSERVATION"), children=[])
    ctx = tree_item("Context", extra=rm_type("EVENT_CONTEXT"), children=[])
    root = tree_item("Vital signs encounter", extra=rm_type("COMPOSITION"), expanded=True, on_path=True, children=[ctx, bp, pulse, temp, spo2])
    return tree([root], "The paths of Vital signs encounter.v1", "fui-path-tree")


def node_pane(node="systolic"):
    if node == "position":
        f = facts([("Path", path("/content[openEHR-EHR-OBSERVATION.blood_pressure.v2]/data[at0001]/events[at0006]/state[at0007]/items[at1000]/value")),
                   ("RM type", rm_type("DV_CODED_TEXT")), ("Occurrences", "0..1"), ("FLAT path", path("vital_signs_encounter/blood_pressure/any_event:0/position"))])
        codes = '<ul class="fui-code-list">' + "".join(f"<li>{coding(d, c, 'local', inline=True)}</li>" for d, c in [("Standing", "at1000"), ("Sitting", "at1001"), ("Lying", "at1002"), ("Reclining", "at1003")]) + "</ul>"
        return (f'<div class="fui-browser-head"><h3 class="fui-browser-title">Position</h3><div class="fui-actions">{link_button("secondary", "Build a query on this path", "query.html", "square-terminal", small=True)}</div></div>'
                f'<div class="fui-browser-body fui-stack">{f}<div><p class="fui-label">Value set · 4 codes</p>{codes}</div></div>')
    f = facts([("Path", path("/content[openEHR-EHR-OBSERVATION.blood_pressure.v2]/data[at0001]/events[at0006]/data[at0003]/items[at0004]/value")),
               ("RM type", rm_type("DV_QUANTITY")), ("Occurrences", "0..1"), ("Units", "<code>mm[Hg]</code>, from 0 to 1,000"),
               ("FLAT path", path("vital_signs_encounter/blood_pressure/any_event:0/systolic"))])
    return (f'<div class="fui-browser-head"><h3 class="fui-browser-title">Systolic</h3><div class="fui-actions">{link_button("secondary", "Build a query on this path", "query.html", "square-terminal", small=True)}</div></div>'
            f'<div class="fui-browser-body">{f}</div>')


def paths_panel(node="systolic"):
    find = (f'<div class="fui-browser-find"><div class="fui-list-find"><span class="fui-search-glyph">{icon("search", "16")}</span>'
            f'{input_("p-find", "", "Node name or path", "search", extra="aria-label=" + chr(34) + "Find a node by name or path" + chr(34))}</div></div>')
    left = (f'<div class="fui-browser-pane"><div class="fui-browser-head"><h3 class="fui-browser-title">Paths</h3></div>'
            f'<div class="fui-browser-body">{find}{path_tree("position" if node == "position" else "systolic")}</div></div>')
    right = f'<div class="fui-browser-pane">{node_pane(node)}</div>'
    return f'<div class="fui-panel"><div class="fui-browser is-two">{left}{right}</div></div>'


def page_template_paths():
    return tmpl_detail("paths", paths_panel())


def page_template_paths_node():
    return tmpl_detail("paths", paths_panel("position"))


def page_template_source():
    return tmpl_detail("source", sections(section("", codeview("Vital signs encounter.v1 · OPT", OPT, lang="xml", what="the OPT"))))


EXAMPLE = """{
  "_type": "COMPOSITION",
  "archetype_node_id": "openEHR-EHR-COMPOSITION.encounter.v1",
  "name": { "_type": "DV_TEXT", "value": "Vital signs encounter" },
  "content": [
    {
      "_type": "OBSERVATION",
      "archetype_node_id": "openEHR-EHR-OBSERVATION.blood_pressure.v2",
      "name": { "_type": "DV_TEXT", "value": "Blood pressure" },
      "data": { "events": [ { "data": { "items": [
        { "name": { "value": "Systolic" }, "value": { "_type": "DV_QUANTITY", "magnitude": 120.0, "units": "mm[Hg]" } }
      ] } } ] }
    }
  ]
}"""


def example_opts():
    return (f'<div class="fui-transform-options">{field("x-fmt", "Format", select("x-fmt", [("json", "JSON"), ("xml", "XML"), ("flat", "Flat"), ("structured", "Structured")], "json"))}'
            f'{field("x-det", "Annotations", select("x-det", [("minimal", "None"), ("annotated", "With node names and paths")], "minimal"))}'
            f'{field("x-typ", "Which values", select("x-typ", [("minimal", "Required only"), ("max", "Every optional one too")], "minimal"))}</div>')


def page_template_example():
    return tmpl_detail("example", sections(section("", example_opts()), section("", codeview("Example · generated by the CDR", EXAMPLE, what="the example"))))


def page_template_delete():
    d = dialog("del", "trash-2", "Delete the template Vital signs encounter.v1?",
               f'<p>{badge("bad", "Physical delete")} 1,284 compositions use this template, so the CDR will refuse until they are deleted or migrated.</p>'
               f'<p>{link_button("ghost", "Show the 1,284 compositions", "query.html", "square-terminal", small=True)}</p>',
               button("secondary", "Cancel", keys="Esc", attrs="autofocus") + f'<button type="button" class="fui-button fui-button-danger" disabled title="1,284 compositions use it">{icon("trash-2", "16")}Delete the template</button>')
    return tmpl_detail(overlay=d)


def page_template_delete_refused():
    rf = refusal("The CDR refused the delete: compositions still use this template.", request="DELETE /ferroehr/rest/openehr/v1/admin/template/Vital%20signs%20encounter.v1",
                 issues=[("bad", "error", "409 Conflict", "1,284 committed compositions reference this template. Delete or migrate them first.")])
    rf += f'<div class="fui-cluster">{link_button("secondary", "Show the 1,284 compositions", "query.html", "square-terminal")}</div>'
    return tmpl_detail(panel=f'<div class="fui-stack">{rf}{tmpl_overview()}</div>')


def page_template_adl2():
    return tmpl_detail(adl2=True)


AOM2 = """{
  "_type": "OPERATIONAL_TEMPLATE",
  "archetype_id": { "value": "openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.1.0" },
  "adl_version": "2.4.0",
  "rm_release": "1.2.0",
  "definition": {
    "_type": "C_COMPLEX_OBJECT",
    "rm_type_name": "COMPOSITION",
    "node_id": "id1.1",
    "attributes": [ "… 4 C_ATTRIBUTEs …" ]
  }
}"""


def page_template_adl2_aom():
    return tmpl_detail("aom2", sections(section("", codeview("Release 1.1.0", AOM2, what="the AOM2 JSON"))), adl2=True)


# ---- Terminology ----

TERM_TABS = [("terminologies", "Terminologies", "terminology.html"), ("lookup", "Look up", "terminology-lookup.html"),
             ("valuesets", "Value sets", "terminology-valuesets.html"), ("subsumption", "Broader and narrower", "terminology-subsumption.html")]

TERMS = [("openehr", "openEHR terminology", "openEHR", "3.1.0"), ("ISO_639-1", "Languages (ISO 639-1)", "ISO", "2002"), ("ISO_3166-1", "Countries (ISO 3166-1)", "ISO", "2020"),
         ("IANA_character-sets", "Character sets", "IANA", "2024-06"), ("IANA_media-types", "Media types", "IANA", "2025-01"), ("openehr_normal_statuses", "Normal statuses", "openEHR", "3.1.0")]


def term_page(tab, panel):
    h = page_heading("Terminology", subtitle="What the CDR can answer about the code sets it serves.")
    return shell("Terminology", "terminology", h + f'<div class="fui-stack">{tabbed(tabs("Terminology operations", TERM_TABS, tab), panel)}</div>')


def page_terminology():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="terminology-descriptor.html">{e(n)}</a><span class="fui-row-sub"><code>{e(i)}</code></span></th>'
            f'{td("Publisher", p, role="sub")}{td("Version", v, role="meta-q")}</tr>' for i, n, p, v in TERMS]
    t = table("Terminologies", th("Terminology") + th("Publisher") + th("Version"), rows)
    return term_page("terminologies", sections(section("Terminologies the CDR serves", t, count=len(TERMS))))


def page_terminology_descriptor():
    f = facts([("Id", "<code>openehr</code>"), ("Publisher", "openEHR Foundation"), ("Identifying URI", "<code>https://specifications.openehr.org/<wbr>releases/<wbr>TERM/<wbr>Release-3.1.0</code>"),
               ("Versions", "3.1.0"), ("Fields you can ask for", "<code>rubric</code>, <code>group</code>, <code>language</code>")])
    h = page_heading("openEHR terminology", crumbs=[("Terminology", "terminology.html")], subtitle="<code>openehr</code>")
    acts = f'<div class="fui-cluster">{link_button("secondary", "Look up a code in it", "terminology-lookup.html", "search")}{link_button("secondary", "Expand one of its groups", "terminology-valuesets.html", "list-tree")}</div>'
    return shell("openEHR terminology", "terminology", h + f'<div class="fui-stack">{sections(section("Descriptor", f), section("", acts), card=True)}</div>')


def lookup_opts(code="249"):
    return (f'<div class="fui-transform-options">{field("l-t", "Terminology", select("l-t", [(i, n) for i, n, _, _ in TERMS], "openehr"))}'
            f'{field("l-c", "Code", input_("l-c", code, extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
            f'{field("l-d", "Effective date (optional)", input_("l-d", "", "YYYY-MM-DD", extra="inputmode=" + chr(34) + "numeric" + chr(34)))}'
            f'<div class="fui-transform-run">{button("primary", "Look up", "search", keys="Ctrl Enter")}</div></div>')


def page_terminology_lookup():
    ans = load_report("ok", f"Defined: {coding('creation', '249', inline=True)}")
    f = facts([("Language", "English (en)"), ("Preferred term", "Yes"), ("Group", "audit_change_type")])
    return term_page("lookup", sections(section("", lookup_opts()), section("", ans + f)))


def page_terminology_lookup_unknown():
    ans = load_report("neutral", "The <code>openehr</code> terminology does not define <code>999</code>.")
    return term_page("lookup", sections(section("", lookup_opts("999")), section("", ans)))


def page_terminology_valuesets():
    opts = (f'<div class="fui-transform-options">{field("v-t", "Terminology", select("v-t", [(i, n) for i, n, _, _ in TERMS], "openehr"))}'
            f'{field("v-g", "Value set", select("v-g", [(g, g) for g in ("audit_change_type", "composition_category", "setting", "version_lifecycle_state", "null_flavours", "term_mapping_purpose")], "audit_change_type"))}'
            f'<div class="fui-transform-run">{button("primary", "Expand", "list-tree", keys="Ctrl Enter")}</div></div>')
    mem = [("creation", "249"), ("amendment", "250"), ("modification", "251"), ("synthesis", "252"), ("deleted", "523"), ("attestation", "666"), ("restoration", "572"), ("format conversion", "573"), ("unknown", "253")]
    lst = '<ul class="fui-code-list mk-plain-list">' + "".join(f"<li>{coding(d, c, inline=True)}</li>" for d, c in mem) + "</ul>"
    val = (f'<div class="fui-transform-options">{field("v-c", "Code", input_("v-c", "251", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
           f'<div class="fui-transform-run">{button("secondary", "Validate", "shield-check")}</div></div>')
    verdict = load_report("ok", "<code>251</code> modification is a member of <code>audit_change_type</code>.")
    return term_page("valuesets", sections(section("", opts), section("Members", lst, count=len(mem)), section("Validate a code", val + verdict)))


def page_terminology_subsumption():
    opts = (f'<div class="fui-transform-options">{field("s-t", "Terminology", select("s-t", [(i, n) for i, n, _, _ in TERMS], "openehr"))}'
            f'{field("s-a", "Broader code", input_("s-a", "249", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
            f'{field("s-b", "Narrower code", input_("s-b", "251", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
            f'<div class="fui-transform-run">{button("primary", "Test", "git-compare", keys="Ctrl Enter")}</div></div>')
    verdict = load_report("neutral", "<code>249</code> is not broader than <code>251</code>.")
    why = '<p class="fui-small fui-muted">The openEHR vocabulary has no broader or narrower codes, so it always answers no; a terminology server bound to the CDR can answer otherwise.</p>'
    return term_page("subsumption", sections(section("", opts), section("", verdict + why)))


def page_terminology_off():
    h = page_heading("Terminology", subtitle="What the CDR can answer about the code sets it serves.")
    body = (f'<section class="fui-card fui-panel"><div class="fui-panel-body fui-stack"><p><strong>The CDR\'s terminology API is off.</strong></p>'
            f'<p class="fui-small fui-muted">Set in <code>ferroehr.toml</code> · <a href="configuration-integrations.html">Integrations settings</a></p></div></section>')
    return shell("Terminology", "terminology", h + f'<div class="fui-stack">{body}</div>', probes=False)



ADL2_SRC = """archetype (adl_version=2.4.0; rm_release=1.2.0; generated)
    openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.1.0

specialise
    openEHR-EHR-COMPOSITION.encounter.v1

language
    original_language = <[ISO_639-1::en]>

definition
    COMPOSITION[id1.1] matches {
        content matches {
            use_archetype OBSERVATION[id0.1, openEHR-EHR-OBSERVATION.blood_pressure.v2]
            use_archetype OBSERVATION[id0.2, openEHR-EHR-OBSERVATION.pulse.v2]
            use_archetype OBSERVATION[id0.3, openEHR-EHR-OBSERVATION.body_temperature.v2]
            use_archetype OBSERVATION[id0.4, openEHR-EHR-OBSERVATION.pulse_oximetry.v1]
        }
    }"""


def page_template_adl2_paths():
    return tmpl_detail("paths", paths_panel(), adl2=True)


def page_template_adl2_source():
    return tmpl_detail("source", sections(section("", codeview("ADL 2 source · 1.1.0", ADL2_SRC, lang="text", what="the source"))), adl2=True)


def page_template_adl2_example():
    return tmpl_detail("example", sections(section("", example_opts()), section("", codeview("Example · generated by the CDR", EXAMPLE, what="the example"))), adl2=True)


def page_template_adl2_versions():
    return tmpl_detail(adl2=True, ver_open=True)
