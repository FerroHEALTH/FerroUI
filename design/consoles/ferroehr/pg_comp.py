# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Compositions: one composition with its tabs (Reading, Document,
Versions, Tags), its delete, and the editors: Edit as a new version and
New composition."""
from common import ago
from common import (
    toast, toasts,
    MENU_SEP, action_group, badge, button, e, field, icon, icon_button, input_, link_button, menu_item, notice, page_heading, refusal,
    select, shell, table, tabbed, tabs, td, th, when,
)
from data import C1, E1, SYSTEM_ID, TEMPLATES
from ehr import (
    tag_editor, contrib_ref,
    path,
    at_time, codeview, coding, confirm_delete, diag_list, editor, facts, intro, nullf, qty, quiet, reading, reading_section, section,
    sections, short, state, version_cell,
)
from pg_ehr import VITALS_JSON, edit_dock, edit_shell, msg

CRUMBS = [("EHRs", "ehrs.html"), (f"EHR {E1[:8]}…", "ehr.html")]
COMP_TABS = [("reading", "Reading", "composition.html"), ("document", "Document", "composition-document.html"),
             ("versions", "Versions", "composition-versions.html", 3), ("tags", "Tags", "composition-tags.html", 2)]


def comp_more(open_=False):
    return [menu_item("Copy the id", "copy"), MENU_SEP, menu_item("Delete composition…", "trash-2", danger=True, href="composition-delete.html")]


def comp_header(more_open=False, deleted=False, version="3"):
    prim = "Edit as a new version" if version == "3" else "Edit the latest"
    if deleted:
        ag = action_group(link_button("primary", "Read version 3", "composition-deleted-version.html", "history"), [menu_item("Copy the id", "copy")], label="More actions for the composition")
        return page_heading("Vital signs", actions=ag, badge=state("muted", "Deleted"), crumbs=CRUMBS,
                            subtitle=f"Vital signs encounter.v1 · deleted {when('2026-10-21T09:12')} by Demo Clinician as version 4.")
    ag = action_group(link_button("primary", prim, "composition-edit.html", "pencil"), comp_more(), open_=more_open, label="More actions for the composition")
    line = f"version {version} of 3 · " if version == "3" else ""
    return page_heading("Vital signs", actions=ag, crumbs=CRUMBS,
                        subtitle=f"Vital signs encounter.v1 · {line}started {when('2026-10-21T07:50')} by Demo Nurse.")


def comp_page(tab="reading", panel=None, header=None, toast_html="", overlay=None, tabs_list=COMP_TABS):
    h = header or comp_header()
    p = panel if panel is not None else reading_panel()
    return shell("Vital signs", "ehrs", h + f'<div class="fui-stack">{tabbed(tabs("Views of this composition", tabs_list, tab), p)}</div>', toast_html=toast_html, overlay=overlay)


def reading_panel(older=False, top=""):
    sys_ = 128 if not older else 141
    bp = reading_section("Blood pressure", "openEHR-EHR-OBSERVATION.blood_pressure.v2", [
        ("Systolic", qty(str(sys_), "mm[Hg]")), ("Diastolic", qty("82" if not older else "88", "mm[Hg]")),
        ("Position", coding("Sitting", "at1001", inline=True)), ("Cuff size", nullf("No information: unknown"))],
        reading_section("Device", "openEHR-EHR-CLUSTER.device.v1", [("Device name", "Automatic upper-arm cuff")]))
    pulse = reading_section("Pulse", "openEHR-EHR-OBSERVATION.pulse.v2", [("Rate", qty("72", "/min")), ("Regularity", coding("Regular", "at0006", inline=True))])
    temp = reading_section("Body temperature", "openEHR-EHR-OBSERVATION.body_temperature.v2", [("Temperature", qty("37.1", "Cel"))])
    spo2 = reading_section("Pulse oximetry", "openEHR-EHR-OBSERVATION.pulse_oximetry.v1", [("SpO₂", qty("97", "%"))])
    ctx = reading_section("Context", None, [("Start time", f'{when("2026-10-21T07:50", absolute=True)}'), ("Setting", coding("other care", "238", "openehr", inline=True)), ("Composer", "Demo Nurse")])
    note = "" if older else f'<section class="fui-panel-section"><p class="fui-small">{link_button("ghost", "Show the full document", "composition-document.html", "braces", small=True)}</p></section>'
    t = f'<section class="fui-panel-section">{top}</section>' if top else ""
    return f'<div class="fui-panel">{t}{reading(ctx, bp, pulse, temp, spo2)}{note}</div>'


def page_composition(toast_html=""):
    return comp_page(toast_html=toast_html)


FORMATS = [("json", "JSON", "composition-document.html", "Canonical JSON"), ("xml", "XML", "composition-document.html", "Canonical XML"),
           ("flat", "Flat", "composition-flat.html", "FLAT simplified format"), ("structured", "Structured", "composition-document.html", "STRUCTURED simplified format")]

FLAT = """{
  "vital_signs_encounter/_uid": "0199f1d0-33aa-7b20-8c4e-2a9e51b07c11::cdr.example.org::3",
  "vital_signs_encounter/language|code": "en",
  "vital_signs_encounter/territory|code": "NL",
  "vital_signs_encounter/context/start_time": "2026-10-21T07:50:00+02:00",
  "vital_signs_encounter/context/setting|code": "238",
  "vital_signs_encounter/composer|name": "Demo Nurse",
  "vital_signs_encounter/blood_pressure/any_event:0/systolic|magnitude": 128,
  "vital_signs_encounter/blood_pressure/any_event:0/systolic|unit": "mm[Hg]",
  "vital_signs_encounter/blood_pressure/any_event:0/diastolic|magnitude": 82,
  "vital_signs_encounter/blood_pressure/any_event:0/diastolic|unit": "mm[Hg]",
  "vital_signs_encounter/blood_pressure/any_event:0/position|code": "at1001",
  "vital_signs_encounter/pulse/any_event:0/rate|magnitude": 72,
  "vital_signs_encounter/pulse/any_event:0/rate|unit": "/min",
  "vital_signs_encounter/body_temperature/any_event:0/temperature|magnitude": 37.1,
  "vital_signs_encounter/body_temperature/any_event:0/temperature|unit": "Cel",
  "vital_signs_encounter/blood_pressure/any_event:0/cuff_size/null_flavour|code": "253",
  "vital_signs_encounter/blood_pressure/device/device_name": "Automatic upper-arm cuff",
  "vital_signs_encounter/pulse/any_event:0/regularity|code": "at0006",
  "vital_signs_encounter/pulse_oximetry/any_event:0/spo2|numerator": 97.0,
  "vital_signs_encounter/pulse_oximetry/any_event:0/spo2|denominator": 100.0,
  "vital_signs_encounter/pulse_oximetry/any_event:0/spo2|type": 2
}"""


def format_switch(current):
    segs = "".join(f'<a href="{h}" class="fui-segment" title="{tip}"{" aria-current=" + chr(34) + "true" + chr(34) if k == current else ""}>{w}</a>' for k, w, h, tip in FORMATS)
    return f'<div role="group" aria-label="Format" class="fui-segmented">{segs}</div>'


def document_panel(fmt="json"):
    text = VITALS_JSON if fmt == "json" else FLAT
    title = {"json": "COMPOSITION · canonical JSON", "flat": "COMPOSITION · FLAT"}[fmt]
    top = section("", f'<div class="fui-cluster">{format_switch(fmt)}<span class="fui-small fui-muted">The CDR converts; the viewer shows its answer as sent.</span></div>')
    return sections(top, section("", codeview(title, text, what="the composition")))


def page_composition_document():
    return comp_page("document", document_panel())


def page_composition_flat():
    return comp_page("document", document_panel("flat"))


VERS = [("3", "2026-10-21T08:14", "Demo Nurse", "Modification", "0199f1d0…4f16"), ("2", "2026-10-21T08:02", "Demo Nurse", "Modification", "0199f1c4…9e30"),
        ("1", "2026-10-21T07:55", "Demo Nurse", "Creation", "0199f1b2…8a29")]


def versions_panel(selected="3"):
    rows = []
    for i, (v, w, who, how, cu) in enumerate(VERS):
        href = "composition-versions.html" if i == 0 else "composition-old-version.html"
        sel = ' aria-current="true"' if v == selected and i != 0 else ""
        rows.append(f'<tr{sel}><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{version_cell(v, current=i == 0)}</a></th>'
                    + td("Change", how, role="sub") + td("Committed by", who, role="meta-q") + td("When", ago(w), role="meta-2")
                    + td("Contribution", contrib_ref(cu), role="wide") + "</tr>")
    t = table("Versions", th("Version") + th("Change") + th("Committed by") + th("When") + th("Contribution"), rows)
    env = facts([("Lifecycle state", "Complete"), ("Preceding version", f"<code>{C1[:8]}…::{SYSTEM_ID}::2</code>"), ("Contribution", contrib_ref("0199f1d0…4f16")),
                 ("Signed", "No"), ("Holds content", "Yes")])
    obj = facts([("Versioned object", f'<span title="{C1}">{short(C1)}</span>'), ("EHR", f'<a href="ehr.html" title="{E1}">{short(E1)}</a>'), ("First committed", when("2026-10-21T07:55", absolute=True))])
    return sections(section("Versions", t + at_time("c-at", "2026-10-21 08:05", "the version current then"), count=3), section(f"Version {selected}", env), section("The versioned composition", obj))


def page_composition_versions():
    return comp_page("versions", versions_panel())


def page_composition_old_version():
    top = notice("info", "history", f'You are reading version 2 of 3, committed {when("2026-10-21T08:02")}. <a href="composition.html">Read the latest</a>')
    tl = [("reading", "Reading", "composition-old-version.html"), ("versions", "Versions", "composition-versions.html", 3)]
    return comp_page("reading", reading_panel(older=True, top=top), header=comp_header(version="2"), tabs_list=tl)


TAGS = [("follow-up", "2026-11-04", "/content[openEHR-EHR-OBSERVATION.blood_pressure.v2]"), ("reviewed", "Demo Clinician", None)]


def page_composition_tags(toast_html="", rows=TAGS, conflict=False):
    tl = COMP_TABS[:3] + [("tags", "Tags", "composition-tags.html", len(rows))]
    return comp_page("tags", tag_editor(rows, "composition", conflict), toast_html=toast_html, tabs_list=tl)


def page_composition_tag_removed():
    t = toast("ok", "circle-check", "Tag removed", "follow-up is gone from Vital signs.", action=button("secondary", "Undo", "undo", small=True))
    return page_composition_tags(toasts(t), TAGS[1:])


def page_composition_tags_conflict():
    return page_composition_tags(conflict=True)


def page_composition_more():
    return comp_page(header=comp_header(more_open=True))


def page_composition_delete():
    d = confirm_delete("del", "Delete the composition Vital signs?", "Marks the composition deleted with a new version 4; its three versions and their audit stay readable, and it leaves the EHR's composition list.",
                       False, "Delete the composition")
    return comp_page(overlay=d)


def page_composition_deleted():
    rows = [f'<tr><th scope="row" data-role="title">{version_cell("4", deleted=True)}</th>{td("Change", "Deletion", role="sub")}{td("Committed by", "Demo Clinician", role="meta-q")}{td("When", ago("2026-10-21T09:12"), role="meta-2")}{td("Contribution", contrib_ref("0199f22a…61c4"), role="wide")}</tr>']
    for i, (v, w, who, how, cu) in enumerate(VERS):
        href = "composition-deleted-version.html" if v == "3" else "composition-old-version.html"
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{version_cell(v)}</a></th>'
                    + td("Change", how, role="sub") + td("Committed by", who, role="meta-q") + td("When", ago(w), role="meta-2")
                    + td("Contribution", contrib_ref(cu), role="wide") + "</tr>")
    p = sections(section("Versions", table("Versions", th("Version") + th("Change") + th("Committed by") + th("When") + th("Contribution"), rows), count=4))
    return comp_page("versions", p, header=comp_header(deleted=True), tabs_list=DELETED_TABS)


DELETED_TABS = [("reading", "Version 3", "composition-deleted-version.html"), ("versions", "Versions", "composition-deleted.html", 4)]


def page_composition_deleted_version():
    top = notice("info", "history", f'You are reading version 3, the last before the deletion on {when("2026-10-21T09:12", absolute=True)}; it is read-only.')
    return comp_page("reading", reading_panel(older=True, top=top), header=comp_header(deleted=True), tabs_list=DELETED_TABS)


# ---- editors ----

def comp_edit(top="", diags=None, d=None):
    text = VITALS_JSON.replace('"code_string": "NL"', '"code_string": "NLD"').replace(', "defining_code": { "terminology_id": { "value": "openehr" }, "code_string": "238" }', "") if diags else VITALS_JSON
    text = text.replace("2026-10-21T07:50:00+02:00", "2026-10-21T07:45:00+02:00")
    ed = editor(text, label="The composition, canonical JSON", diags=diags, pos="Ln 14, Col 42", current=None if diags else 14,
                tools=button("ghost", "Revert", "rotate-ccw", small=True))
    if diags:
        cols = {11: 78, 14: 72}
        rows = [("bad", m[1], ln, cols.get(ln, 1), m[2]) for ln, m in diags.items()]
        body = f'<div class="fui-editor-split">{ed}<section class="fui-card fui-panel" aria-label="Diagnostics"><div class="fui-panel-head"><h2 class="fui-card-title"><span>What the CDR refused <span class="mk-count">· {len(rows)}</span></span></h2></div>{diag_list(rows, current=14)}</div>'.replace("</section></div>", "</section></div>")
        body = body + "</section></div>" if not body.endswith("</div>") else body
        body = f'<div class="fui-editor-split">{ed}<section class="fui-card fui-panel" aria-label="Diagnostics"><div class="fui-panel-head"><h2 class="fui-card-title"><span>What the CDR refused <span class="mk-count">· {len(rows)}</span></span></h2></div>{diag_list(rows, current=14)}</section></div>'
    else:
        body = ed
    dk = d or edit_dock(msg("pencil", "1 change: the start time, 07:50 to 07:45."), "Save as version 4", "composition.html")
    return edit_shell("Edit", "ehrs", CRUMBS + [("Vital signs", "composition.html")], "Version 3, opened 09:38.", top + body, dk)


def page_composition_edit():
    return comp_edit()


def page_composition_edit_invalid():
    rf = refusal("The CDR refused the document, so no version was saved.", request=f"PUT /ferroehr/rest/openehr/v1/ehr/{E1}/composition/{C1}",
                 issues=[("bad", "Error", "422 Unprocessable Content", "2 validation errors, listed with the document.")])
    diags = {11: ("bad", "ISO_3166-1", "/territory: NLD is not an ISO 3166-1 alpha-2 code."), 14: ("bad", "DV_CODED_TEXT", "/context/setting needs its defining_code.")}
    return comp_edit(rf, diags, edit_dock(msg("circle-x", "Not saved: 2 errors to fix."), "Save as version 4", "composition.html"))


def page_composition_edit_conflict():
    rf = refusal("Another client saved a new version since you opened it, so the CDR refused the save (412).", request=f"PUT /ferroehr/rest/openehr/v1/ehr/{E1}/composition/{C1}",
                 issues=[("bad", "Error", "412 Precondition Failed", "If-Match names version 3; the latest is version 4.")])
    rf += f'<div class="fui-cluster">{button("secondary", "Reload version 4", "refresh-cw")}<span class="fui-small fui-muted">Reloading puts version 4 in the editor and replaces your edit.</span></div>'
    return comp_edit(rf, d=edit_dock(msg("circle-x", "Not saved."), "Save", "composition.html", disabled="Reload the latest version first"))


def comp_new(flat=False, fmt_key=None):
    fmt_key = fmt_key or ("flat" if flat else "json")
    cur = lambda k: " aria-current=" + chr(34) + "true" + chr(34) if k == fmt_key else ""
    fmt = (f'<div role="group" aria-label="Format" class="fui-segmented"><a href="composition-new.html" class="fui-segment" title="Canonical JSON"{cur("json")}>JSON</a>'
           f'<a href="composition-new-xml.html" class="fui-segment" title="Canonical XML"{cur("xml")}>XML</a><a href="composition-new-flat.html" class="fui-segment" title="FLAT simplified format"{cur("flat")}>Flat</a></div>')
    help_ = "Flat needs the template; the viewer sends it for you." if flat else "The template the document is committed against."
    meta = section("", '<div class="fui-form-grid">' + field("nt", "Template", select("nt", [(t[0], t[0]) for t in TEMPLATES], "Vital signs encounter.v1"), help_=help_, cls="fui-span-2")
                   + f'<div class="fui-field fui-span-2"><span class="fui-label">Format</span>{fmt}</div></div>')
    text = FLAT.replace('  "vital_signs_encounter/_uid": "0199f1d0-33aa-7b20-8c4e-2a9e51b07c11::cdr.example.org::3",\n', "") if flat else (VITALS_XML if fmt_key == "xml" else VITALS_JSON)
    ed = editor(text, lang="xml" if fmt_key == "xml" else "json", label="The new composition", lang_word="", pos="Ln 1, Col 1", dirty=False, tools=button("ghost", "Choose a file", "upload", small=True))
    return edit_shell("New composition", "ehrs", CRUMBS, "A document committed as version 1 under a template.", sections(meta, card=True) + ed,
                      edit_dock(msg("file-plus", "Not committed yet."), "Commit this composition", "ehr.html"))


def page_composition_new():
    return comp_new()


def page_composition_new_flat():
    return comp_new(flat=True)


def page_composition_new_xml():
    return comp_new(fmt_key="xml")


VITALS_XML = """<?xml version="1.0" encoding="UTF-8"?>
<composition xmlns="http://schemas.openehr.org/v1" archetype_node_id="openEHR-EHR-COMPOSITION.encounter.v1">
  <name><value>Vital signs</value></name>
  <archetype_details>
    <archetype_id><value>openEHR-EHR-COMPOSITION.encounter.v1</value></archetype_id>
    <template_id><value>Vital signs encounter.v1</value></template_id>
    <rm_version>1.2.0</rm_version>
  </archetype_details>
  <language><terminology_id><value>ISO_639-1</value></terminology_id><code_string>en</code_string></language>
  <territory><terminology_id><value>ISO_3166-1</value></terminology_id><code_string>NL</code_string></territory>
  <category><value>event</value><defining_code><terminology_id><value>openehr</value></terminology_id><code_string>433</code_string></defining_code></category>
  <composer xsi:type="PARTY_IDENTIFIED" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><name>Demo Nurse</name></composer>
  <context><start_time><value>2026-10-21T07:50:00+02:00</value></start_time><setting><value>other care</value><defining_code><terminology_id><value>openehr</value></terminology_id><code_string>238</code_string></defining_code></setting></context>
</composition>"""
