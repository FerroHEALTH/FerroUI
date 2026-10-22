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
                    + td("Root archetype", arch(root), role="sub") + td("Compositions", f"{n:,}", "fui-num", role="meta-q", unit="composition" if n == 1 else "compositions")
                    + td("Uploaded", ago(up), role="meta-2") + td("", tmpl_row_menu(open_row == i, tid), "fui-row-menu", role="menu") + "</tr>")
    return table("ADL 1.4 templates", th("Template") + th("Root archetype") + th("Compositions", num=True) + th("Uploaded") + th('<span class="fui-sr-only">Actions</span>'), rows)


def adl2_rows():
    rows = []
    for hrid, concept, root, n, up in ADL2:
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="template-adl2.html">{arch(hrid)}</a><span class="fui-row-sub">{e(concept)}</span></th>'
                    + td("Root archetype", arch(root), role="sub") + td("Compositions", str(n), "fui-num", role="meta-q", unit="composition" if n == 1 else "compositions")
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
    return tmpl_page(body=tmpl_rows(lst), value="list", chips=["Contains: list"], n14=len(lst), n2=0)


def page_templates_row_menu():
    return tmpl_page(body=tmpl_rows(open_row=1))


def page_templates_empty():
    return tmpl_page(body=empty_line("No template is uploaded yet."), n14=0, n2=0)


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


ADL2_REFUSED = """template (adl_version=2.4.0; rm_release=1.1.0)
    openEHR-EHR-COMPOSITION.lab_order.v0.5.0

specialise
    openEHR-EHR-COMPOSITION.request.v1

language
    original_language = <[ISO_639-1::en]>

definition
    OBSERVATION[id1.1] matches {
        category matches {[at17]}
    }

terminology
    term_definitions = <
        ["en"] = <
            ["id1.1"] = <text = <"Laboratory order"> description = <"A request for laboratory tests.">>
        >
    >"""

ADL2_CLEAN = ADL2_REFUSED.replace("OBSERVATION[id1.1]", "COMPOSITION[id1.1]").replace("category matches {[at17]}", "category matches {[at1]}").replace(
    '            ["id1.1"] = <text = <"Laboratory order"> description = <"A request for laboratory tests.">>',
    '            ["id1.1"] = <text = <"Laboratory order"> description = <"A request for laboratory tests.">>\n            ["at1"] = <text = <"event"> description = <"The composition records an event.">>')
VARDT_COL = ADL2_REFUSED.split("\n")[10].index("OBSERVATION") + 1
VATDF_COL = ADL2_REFUSED.split("\n")[11].index("at17") + 1


OPT_FULL = OPT.replace("""    <archetype_id><value>openEHR-EHR-COMPOSITION.encounter.v1</value></archetype_id>
  </definition>""", """    <archetype_id><value>openEHR-EHR-COMPOSITION.encounter.v1</value></archetype_id>
    <attributes xsi:type="C_MULTIPLE_ATTRIBUTE" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <rm_attribute_name>content</rm_attribute_name>
      <children xsi:type="C_ARCHETYPE_ROOT"><rm_type_name>OBSERVATION</rm_type_name><archetype_id><value>openEHR-EHR-OBSERVATION.blood_pressure.v2</value></archetype_id></children>
      <children xsi:type="C_ARCHETYPE_ROOT"><rm_type_name>OBSERVATION</rm_type_name><archetype_id><value>openEHR-EHR-OBSERVATION.pulse.v2</value></archetype_id></children>
      <children xsi:type="C_ARCHETYPE_ROOT"><rm_type_name>OBSERVATION</rm_type_name><archetype_id><value>openEHR-EHR-OBSERVATION.body_temperature.v2</value></archetype_id></children>
      <children xsi:type="C_ARCHETYPE_ROOT"><rm_type_name>OBSERVATION</rm_type_name><archetype_id><value>openEHR-EHR-OBSERVATION.pulse_oximetry.v1</value></archetype_id></children>
    </attributes>
    <!-- abridged: each archetype root's own constraints are left out here -->
  </definition>""")


def template_upload(top="", adl2=False, refused=False):
    fam = (f'<div role="group" aria-label="Family" class="fui-segmented"><a href="template-upload.html" class="fui-segment"{"" if adl2 else " aria-current=" + chr(34) + "true" + chr(34)}>ADL 1.4</a>'
           f'<a href="template-upload-adl2.html" class="fui-segment"{" aria-current=" + chr(34) + "true" + chr(34) if adl2 else ""}>ADL 2</a></div>')
    tools = fam + button("ghost", "Choose a file", "upload", small=True)
    if adl2:
        text = ADL2_REFUSED if refused else ADL2_CLEAN
        diags = {11: ("bad", "VARDT", "The definition's type, OBSERVATION, does not match COMPOSITION in the template id."),
                 12: ("bad", "VATDF", "at17 is used but not defined in the terminology.")} if refused else None
        ed = editor(text, lang="text", lang_word="ADL 2", label="The ADL 2 source", diags=diags, dirty=refused, pos=f"Ln 11, Col {VARDT_COL}" if refused else "Ln 1, Col 1", tools=tools)
        rows = [("bad", "VARDT", 11, VARDT_COL, "The definition's outer type, OBSERVATION, must match the type in the template id, COMPOSITION."),
                ("bad", "VATDF", 12, VATDF_COL, "The code at17 is used in the definition but not defined in the terminology section.")]
    else:
        diags = {11: ("bad", "OPT-UID", "uid is missing its value.")} if refused else None
        ed = editor(OPT.replace("<uid><value>5f2c8e41-0b7d-4a93-9e16-3d8a7c0f2b54</value></uid>", "<uid></uid>") if refused else OPT, lang="xml", lang_word="OPT XML",
                    label="The operational template", diags=diags, dirty=refused, pos="Ln 11, Col 8" if refused else "Ln 1, Col 1", tools=tools)
        rows = [("bad", "OPT-UID", 11, 8, "The template's uid has no value.")]
    if refused:
        body = f'<div class="fui-editor-split">{ed}<section class="fui-card fui-panel" aria-label="Diagnostics"><div class="fui-panel-head"><h2 class="fui-card-title"><span>What the CDR refused <span class="mk-count">· {len(rows)}</span></span></h2></div>{diag_list(rows, current=rows[0][2])}</section></div>'
        d = edit_dock(msg("circle-x", "Not uploaded."), "Upload this template", "templates.html")
    else:
        body = ed
        d = edit_dock(msg("upload", "Not uploaded yet."), "Upload this template", "templates.html")
    return edit_shell("Upload a template", "templates", [("Templates", "templates.html")], "Choose a file or paste the source; the CDR validates it on upload.", top + body, d)


def page_template_upload():
    return template_upload()


def page_template_upload_refused():
    rf = refusal("The CDR refused the template (400 Bad Request), so nothing was uploaded.", request="POST /ferroehr/rest/openehr/v1/definition/template/adl1.4")
    return template_upload(rf, refused=True)


def page_template_upload_adl2():
    return template_upload(adl2=True)


def page_template_upload_adl2_refused():
    rf = refusal("The CDR refused the template (400 Bad Request), so nothing was uploaded.", request="POST /ferroehr/rest/openehr/v1/definition/template/adl2")
    return template_upload(rf, adl2=True, refused=True)


TMPL_TABS = [("overview", "Overview", "template.html"), ("paths", "Paths", "template-paths.html"), ("source", "Source", "template-source.html"), ("example", "Example", "template-example.html")]


def version_select(open_=False):
    """R29: the release version as a select button, its listbox under it."""
    from parts import listbox, select_button
    lb = listbox([("1.1.0", "1.1.0", "Latest"), ("1.0.0", "1.0.0", None)], "1.1.0", id_="ver-list", label="Version")
    lb = lb.replace('<span class="fui-listbox-text"><span>1.0.0</span>', '<a class="fui-listbox-text" href="template-adl2-v1-0-0.html"><span>1.0.0</span>', 1).replace('</span></li></ul>', '</a></li></ul>', 1)
    btn = select_button("ver", "1.1.0 (latest of 2)", open_, lb).replace('id="ver" class=', 'id="ver" aria-label="Version: 1.1.0, the latest of 2" title="The release versions the CDR holds for this HRID" class=', 1)
    if not open_:
        btn = btn.replace('<button type="button" id="ver"', '<a href="template-adl2-versions.html" role="button" id="ver"', 1).replace("</button></span>", "</a></span>", 1)
    return f'<div class="fui-cluster fui-title-select"><label class="fui-label" for="ver">Version</label>{btn}</div>'


def tmpl_header(adl2=False, ver_open=False, release="1.1.0"):
    more = [menu_item("Download", "download"), menu_item("Copy id", "copy"), MENU_SEP, menu_item("Delete…", "trash-2", danger=True, href="template-delete.html")]
    ag = action_group(link_button("primary", "Build a query", "query.html", "square-terminal"), more, label="More actions for the template")
    if adl2:
        after = version_select(ver_open) if release == "1.1.0" else notice("info", "history", 'You are reading release 1.0.0. <a href="template-adl2.html">Read 1.1.0, the latest</a>')
        return page_heading("Vital signs encounter", actions=ag, crumbs=[("Templates", "templates-adl2.html")],
                            subtitle=arch(f"openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.{release[2:]}") + " · ADL 2", after=after)
    return page_heading("Vital signs encounter.v1", actions=ag, crumbs=[("Templates", "templates.html")], subtitle="Vital signs encounter · ADL 1.4")


ADL2_TABS = [("overview", "Overview", "template-adl2.html"), ("paths", "Paths", "template-adl2-paths.html"), ("source", "Source", "template-adl2-source.html"),
             ("aom2", "JSON", "template-adl2-aom.html"), ("example", "Example", "template-adl2-example.html")]


def tmpl_detail(tab="overview", panel=None, overlay=None, adl2=False, ver_open=False, release="1.1.0"):
    p = panel if panel is not None else tmpl_overview(adl2, release)
    tl = ADL2_TABS if adl2 else TMPL_TABS
    if adl2 and release != "1.1.0":
        tl = [("overview", "Overview", "template-adl2-v1-0-0.html")]
    return shell("Vital signs encounter", "templates", tmpl_header(adl2, ver_open, release) + f'<div class="fui-stack">{tabbed(tabs("Views of this template", tl, tab), p)}</div>', overlay=overlay)


def tmpl_overview(adl2=False, release="1.1.0"):
    if adl2:
        f = facts([("Concept", "Vital signs encounter"), ("Release", release), ("Default language", "English (en)"), ("Languages", "English (en)"),
                   ("Root archetype", arch("openEHR-EHR-COMPOSITION.encounter.v1")), ("Uploaded", when("2026-10-12T11:20" if release == "1.1.0" else "2026-09-30T10:05", absolute=True))])
        use = facts([("Compositions committed under it", "0")])
        return sections(section("Identity", f), section("Use", use))
    f = facts([("Concept", "Vital signs encounter"), ("Template version", "1"), ("Default language", "English (en)"), ("Languages", "English (en), Dutch (nl)"),
               ("UID", "<code>5f2c8e41-0b7d-4a93-9e16-3d8a7c0f2b54</code>"), ("Root archetype", arch("openEHR-EHR-COMPOSITION.encounter.v1")), ("Uploaded", when("2026-02-11T09:00"))])
    use = facts([("Compositions committed under it", '<a href="query.html">1,284</a>')])
    return sections(section("Identity", f), section("Use", use))


def page_template():
    return tmpl_detail()


def path_tree(selected="systolic", adl2=False):
    pages = {"systolic": "template-adl2-paths.html" if adl2 else "template-paths.html", "position": "template-adl2-paths-node.html" if adl2 else "template-paths-node.html"}

    def leaf(label, t, key):
        return tree_item(label, extra=rm_type(t), leaf=True, selected=key == selected, href=pages.get(key, "#"))
    dev = tree_item("Device", extra=rm_type("CLUSTER"), children=[])
    bp = tree_item("Blood pressure", extra=rm_type("OBSERVATION"), expanded=True, on_path=True, children=[
        leaf("Systolic", "DV_QUANTITY", "systolic"), leaf("Diastolic", "DV_QUANTITY", "diastolic"), leaf("Position", "DV_CODED_TEXT", "position"), leaf("Cuff size", "DV_CODED_TEXT", "cuff"), dev])
    pulse = tree_item("Pulse", extra=rm_type("OBSERVATION"), children=[])
    temp = tree_item("Body temperature", extra=rm_type("OBSERVATION"), children=[])
    spo2 = tree_item("Pulse oximetry", extra=rm_type("OBSERVATION"), children=[])
    ctx = tree_item("Context", extra=rm_type("EVENT_CONTEXT"), children=[])
    root = tree_item("Vital signs encounter", extra=rm_type("COMPOSITION"), expanded=True, on_path=True, children=[ctx, bp, pulse, temp, spo2])
    return tree([root], "The paths of Vital signs encounter.v1", "fui-path-tree")


def node_pane(node="systolic", adl2=False):
    if node == "position":
        p = ("/content[id0.1]/data[id2]/events[id7]/state[id8]/items[id9]/value" if adl2
             else "/content[openEHR-EHR-OBSERVATION.blood_pressure.v2]/data[at0001]/events[at0006]/state[at0007]/items[at0008]/value")
        rows = [("Path", path(p)), ("RM type", rm_type("DV_CODED_TEXT")), ("Occurrences", "0..1")]
        if not adl2:
            rows.append(("FLAT path", path("vital_signs_encounter/blood_pressure/any_event:0/position")))
        f = facts(rows)
        codes = '<ul class="fui-code-list">' + "".join(f"<li>{coding(d, c, 'local', inline=True)}</li>" for d, c in [("Standing", "at1000"), ("Sitting", "at1001"), ("Reclining", "at1002"), ("Lying", "at1003"), ("Lying with tilt to left", "at1014")]) + "</ul>"
        return (f'<div class="fui-browser-head"><h3 class="fui-browser-title">Position</h3><div class="fui-actions">{link_button("secondary", "Build a query on this path", "query.html", "square-terminal", small=True)}</div></div>'
                f'<div class="fui-browser-body fui-stack">{f}<div><p class="fui-label">Value set · 5 codes</p>{codes}</div></div>')
    p = ("/content[id0.1]/data[id2]/events[id7]/data[id4]/items[id5]/value" if adl2
         else "/content[openEHR-EHR-OBSERVATION.blood_pressure.v2]/data[at0001]/events[at0006]/data[at0003]/items[at0004]/value")
    rows = [("Path", path(p)), ("RM type", rm_type("DV_QUANTITY")), ("Occurrences", "0..1"), ("Units", '<span title="UCUM mm[Hg]">mm Hg</span>, 0 to under 1,000')]
    if not adl2:
        rows.append(("FLAT path", path("vital_signs_encounter/blood_pressure/any_event:0/systolic")))
    f = facts(rows)
    return (f'<div class="fui-browser-head"><h3 class="fui-browser-title">Systolic</h3><div class="fui-actions">{link_button("secondary", "Build a query on this path", "query.html", "square-terminal", small=True)}</div></div>'
            f'<div class="fui-browser-body">{f}</div>')


def paths_panel(node="systolic", adl2=False):
    find = (f'<div class="fui-browser-find"><div class="fui-list-find"><span class="fui-search-glyph">{icon("search", "16")}</span>'
            f'{input_("p-find", "", "Node name or path", "search", extra="aria-label=" + chr(34) + "Find a node by name or path" + chr(34))}</div></div>')
    left = (f'<div class="fui-browser-pane"><div class="fui-browser-head"><h3 class="fui-browser-title">Paths</h3></div>'
            f'<div class="fui-browser-body">{find}{path_tree("position" if node == "position" else "systolic", adl2)}</div></div>')
    right = f'<div class="fui-browser-pane">{node_pane(node, adl2)}</div>'
    note = '<section class="fui-panel-section"><p class="fui-small fui-muted">The viewer builds these paths itself: the CDR has no web template for ADL 2.</p></section>' if adl2 else ""
    return f'<div class="fui-panel"><div class="fui-browser is-two">{left}{right}</div>{note}</div>'


def page_template_paths():
    return tmpl_detail("paths", paths_panel())


def page_template_paths_node():
    return tmpl_detail("paths", paths_panel("position"))


def page_template_source():
    return tmpl_detail("source", sections(section("", codeview("Source · OPT, abridged", OPT_FULL, lang="xml", what="the OPT"))))


EXAMPLE = """{
  "_type": "COMPOSITION",
  "archetype_node_id": "openEHR-EHR-COMPOSITION.encounter.v1",
  "name": { "_type": "DV_TEXT", "value": "Vital signs encounter" },
  "archetype_details": {
    "archetype_id": { "value": "openEHR-EHR-COMPOSITION.encounter.v1" },
    "template_id": { "value": "Vital signs encounter.v1" },
    "rm_version": "1.1.0"
  },
  "language": { "terminology_id": { "value": "ISO_639-1" }, "code_string": "en" },
  "territory": { "terminology_id": { "value": "ISO_3166-1" }, "code_string": "NL" },
  "category": { "value": "event", "defining_code": { "terminology_id": { "value": "openehr" }, "code_string": "433" } },
  "composer": { "_type": "PARTY_SELF" },
  "context": {
    "start_time": { "value": "2026-10-21T09:42:11+02:00" },
    "setting": { "value": "other care", "defining_code": { "terminology_id": { "value": "openehr" }, "code_string": "238" } }
  },
  "content": []
}"""

EXAMPLE_ADL2 = EXAMPLE.replace('"archetype_node_id": "openEHR-EHR-COMPOSITION.encounter.v1"', '"archetype_node_id": "openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.1.0"').replace(
    '"template_id": { "value": "Vital signs encounter.v1" }', '"template_id": { "value": "openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.1.0" }')


def example_opts(adl2=False):
    fmts = [("json", "JSON"), ("xml", "XML")] if adl2 else [("json", "JSON"), ("xml", "XML"), ("flat", "Flat"), ("structured", "Structured")]
    return (f'<div class="fui-transform-options">{field("x-fmt", "Format", select("x-fmt", fmts, "json"))}'
            f'{field("x-typ", "Type", select("x-typ", [("input", "Input: ready to commit"), ("output", "Output: as the CDR returns it")], "input"))}'
            f'{field("x-det", "Detail level", select("x-det", [("required", "Required: committable, mandatory parts only"), ("medium", "Medium: realistic, committable"), ("complete", "Complete: every possible part")], "required"))}</div>')


def page_template_example():
    return tmpl_detail("example", sections(section("", example_opts()), section("", codeview("Example · input, required", EXAMPLE, what="the example"))))


def page_template_delete():
    d = dialog("del", "trash-2", "Delete the template Vital signs encounter.v1?",
               f'<p>{badge("bad", "Permanent delete")} 1,284 compositions use this template, so the CDR will refuse until they are deleted or migrated.</p>'
               f'<p>{link_button("ghost", "Show the 1,284 compositions", "query.html", "square-terminal", small=True)}</p>',
               button("secondary", "Cancel", keys="Esc", attrs="autofocus") + f'<button type="button" class="fui-button fui-button-danger" disabled title="1,284 compositions use it">{icon("trash-2", "16")}Delete the template</button>')
    return tmpl_detail(overlay=d)


def page_template_adl2():
    return tmpl_detail(adl2=True)


AOM2 = """{
  "_type": "OPERATIONAL_TEMPLATE",
  "archetype_id": { "value": "openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.1.0" },
  "adl_version": "2.4.0",
  "rm_release": "1.1.0",
  "definition": {
    "_type": "C_COMPLEX_OBJECT",
    "rm_type_name": "COMPOSITION",
    "node_id": "id1.1",
    "attributes": [
      {
        "_type": "C_MULTIPLE_ATTRIBUTE",
        "rm_attribute_name": "content",
        "children": [
          { "_type": "C_ARCHETYPE_ROOT", "rm_type_name": "OBSERVATION", "node_id": "id0.1", "archetype_ref": "openEHR-EHR-OBSERVATION.blood_pressure.v2" },
          { "_type": "C_ARCHETYPE_ROOT", "rm_type_name": "OBSERVATION", "node_id": "id0.2", "archetype_ref": "openEHR-EHR-OBSERVATION.pulse.v2" },
          { "_type": "C_ARCHETYPE_ROOT", "rm_type_name": "OBSERVATION", "node_id": "id0.3", "archetype_ref": "openEHR-EHR-OBSERVATION.body_temperature.v2" },
          { "_type": "C_ARCHETYPE_ROOT", "rm_type_name": "OBSERVATION", "node_id": "id0.4", "archetype_ref": "openEHR-EHR-OBSERVATION.pulse_oximetry.v1" }
        ]
      }
    ]
  }
}"""


def page_template_adl2_aom():
    return tmpl_detail("aom2", sections(section("", codeview("JSON · release 1.1.0", AOM2, what="the JSON"))), adl2=True)


# ---- Terminology ----

TERM_TABS = [("terminologies", "Terminologies", "terminology.html"), ("lookup", "Look up", "terminology-lookup.html"),
             ("valuesets", "Value sets", "terminology-valuesets.html"), ("subsumption", "Hierarchy", "terminology-subsumption.html")]

TERMS = [("openehr", "openEHR terminology", "openEHR Foundation", "3.1.0"), ("ISO_639-1", "Languages (ISO 639-1)", "ISO", "2002"), ("ISO_3166-1", "Countries (ISO 3166-1)", "ISO", "2020"),
         ("IANA_character-sets", "Character sets", "IANA", "2024-06"), ("IANA_media-types", "Media types", "IANA", "2025-01"), ("openehr_normal_statuses", "Normal statuses", "openEHR Foundation", "3.1.0")]
TERM_URI = {"openehr": "https://specifications.openehr.org/<wbr>releases/<wbr>TERM/<wbr>Release-3.1.0", "ISO_639-1": "https://www.iso.org/<wbr>iso-639-language-code",
            "ISO_3166-1": "https://www.iso.org/<wbr>iso-3166-country-codes.html", "IANA_character-sets": "https://www.iana.org/<wbr>assignments/<wbr>character-sets",
            "IANA_media-types": "https://www.iana.org/<wbr>assignments/<wbr>media-types", "openehr_normal_statuses": "https://specifications.openehr.org/<wbr>releases/<wbr>TERM/<wbr>Release-3.1.0"}


def term_slug(i):
    return "terminology-descriptor.html" if i == "openehr" else f"terminology-descriptor-{i.lower().replace('_', '-')}.html"


def term_page(tab, panel):
    h = page_heading("Terminology", subtitle="What the CDR can answer about the code sets it serves.")
    return shell("Terminology", "terminology", h + f'<div class="fui-stack">{tabbed(tabs("Terminology operations", TERM_TABS, tab), panel)}</div>')


def page_terminology():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{term_slug(i)}">{e(n)}</a><span class="fui-row-sub"><code>{e(i)}</code></span></th>'
            f'{td("Publisher", p, role="sub")}{td("Version", v, role="meta-q")}</tr>' for i, n, p, v in TERMS]
    t = table("Terminologies", th("Terminology") + th("Publisher") + th("Version"), rows)
    return term_page("terminologies", sections(section("Terminologies the CDR serves", t, count=len(TERMS))))


def page_terminology_descriptor(tid="openehr"):
    i, n, p, v = next(t for t in TERMS if t[0] == tid)
    fields = "<code>rubric</code>, <code>group</code>, <code>language</code>" if p == "openEHR Foundation" else "<code>rubric</code>, <code>language</code>"
    f = facts([("Id", f"<code>{e(i)}</code>"), ("Publisher", p), ("Identifying URI", f"<code>{TERM_URI[i]}</code>"),
               ("Version", v), ("Fields you can ask for", fields)])
    h = page_heading(n, crumbs=[("Terminology", "terminology.html")], subtitle=f"<code>{e(i)}</code>")
    acts = link_button("secondary", "Look up a code in it", "terminology-lookup.html", "search")
    if p == "openEHR Foundation":
        acts += link_button("secondary", "Expand one of its groups", "terminology-valuesets.html", "list-tree")
    return shell(n, "terminology", h + f'<div class="fui-stack">{sections(section("Descriptor", f), section("", f"<div class=fui-cluster>{acts}</div>"), card=True)}</div>')


def page_terminology_descriptor_iso_639_1():
    return page_terminology_descriptor("ISO_639-1")


def page_terminology_descriptor_iso_3166_1():
    return page_terminology_descriptor("ISO_3166-1")


def page_terminology_descriptor_iana_character_sets():
    return page_terminology_descriptor("IANA_character-sets")


def page_terminology_descriptor_iana_media_types():
    return page_terminology_descriptor("IANA_media-types")


def page_terminology_descriptor_openehr_normal_statuses():
    return page_terminology_descriptor("openehr_normal_statuses")


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
    ans = load_report("neutral", "The openEHR terminology does not define <code>999</code>.")
    return term_page("lookup", sections(section("", lookup_opts("999")), section("", ans)))


def page_terminology_valuesets():
    opts = (f'<div class="fui-transform-options">{field("v-t", "Terminology", select("v-t", [(i, n) for i, n, _, _ in TERMS], "openehr"))}'
            f'{field("v-g", "Value set", select("v-g", [(g, f"{g.replace('_', ' ').capitalize()} · {g}") for g in ("audit_change_type", "composition_category", "setting", "version_lifecycle_state", "null_flavours", "term_mapping_purpose")], "audit_change_type"))}'
            f'<div class="fui-transform-run">{button("primary", "Expand", "list-tree", keys="Ctrl Enter")}</div></div>')
    mem = [("creation", "249"), ("amendment", "250"), ("modification", "251"), ("synthesis", "252"), ("deleted", "523"), ("attestation", "666"), ("restoration", "816"), ("format conversion", "817"), ("unknown", "253")]
    lst = '<ul class="fui-code-list mk-plain-list">' + "".join(f"<li>{coding(d, c, inline=True)}</li>" for d, c in mem) + "</ul>"
    val = (f'<div class="fui-transform-options">{field("v-c", "Code", input_("v-c", "251", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
           f'<div class="fui-transform-run">{button("secondary", "Validate", "shield-check")}</div></div>')
    verdict = load_report("ok", '<code>251</code> modification is a member of Audit change type <span class="fui-small fui-muted">audit_change_type</span>.')
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
            f'<p class="fui-small fui-muted">Set in <code>ferroehr.toml</code> · read at start · <a href="configuration-dialog.html">How configuration works</a></p></div></section>')
    return shell("Terminology", "terminology", h + f'<div class="fui-stack">{body}</div>')



ADL2_SRC = """template (adl_version=2.4.0; rm_release=1.1.0)
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
    }

terminology
    term_definitions = <
        ["en"] = <
            ["id1.1"] = <text = <"Vital signs encounter"> description = <"The vital signs taken at one encounter.">>
            ["id0.1"] = <text = <"Blood pressure"> description = <"The blood pressure reading.">>
            ["id0.2"] = <text = <"Pulse"> description = <"The pulse reading.">>
            ["id0.3"] = <text = <"Body temperature"> description = <"The body temperature reading.">>
            ["id0.4"] = <text = <"Pulse oximetry"> description = <"The oxygen saturation reading.">>
        >
    >"""


def page_template_adl2_paths():
    return tmpl_detail("paths", paths_panel(adl2=True), adl2=True)


def page_template_adl2_paths_node():
    return tmpl_detail("paths", paths_panel("position", adl2=True), adl2=True)


def page_template_adl2_v1_0_0():
    return tmpl_detail(adl2=True, release="1.0.0")


def page_template_adl2_source():
    return tmpl_detail("source", sections(section("", codeview("Source · ADL 2, release 1.1.0", ADL2_SRC, lang="text", what="the source"))), adl2=True)


def page_template_adl2_example():
    return tmpl_detail("example", sections(section("", example_opts(adl2=True)), section("", codeview("Example · input, required", EXAMPLE_ADL2, what="the example"))), adl2=True)


def page_template_adl2_versions():
    return tmpl_detail(adl2=True, ver_open=True)
