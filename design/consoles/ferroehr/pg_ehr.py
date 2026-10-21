# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""EHRs: the list, New EHR, and one EHR with its tabs (Compositions,
Directory, Status, Contributions, Tags), the editors of its status and
directory, the contribution builder and the contribution page."""
import tpl
from common import ago
from common import (
    MENU_SEP, action_group, badge, busy_button, button, check, dock, e, field, icon, icon_button, input_, link_button, menu, menu_item,
    notice, page_heading, refusal, select, shell, table, tabbed, tabs, td, th, toast, toasts, when,
)
from data import COMPS, CONTRIBS, E1, E2, EHRS, SYSTEM_ID, TEMPLATES
from ehr import (
    arch, path,
    at_time, codeview, confirm_delete, coding, diag_list, editor, empty_line, facts, intro, load_report, masked, plural, quiet,
    reveal_dialog, section, sections, short, state, subject_line, tree, tree_item, version_cell,
)
from parts import date_input, line_chart, skeleton_lines

# ---- the EHRs list ----


def ehrs_heading():
    return page_heading("EHRs", actions=link_button("primary", "New EHR", "ehr-new.html", "plus"),
                        subtitle="The newest EHRs the CDR holds, read with AQL.")


def ehr_rows(items=EHRS, hover=None):
    rows = []
    for i, (eid, tail, created, n) in enumerate(items):
        href = "ehr.html" if eid == E1 else ("ehr-no-subject.html" if tail is None else "ehr.html")
        subj = masked(tail, static=True) if tail else '<span class="fui-muted">No external subject</span>'
        cls = ' class="mk-hover"' if hover == i else ""
        rows.append(f'<tr{cls}><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{short(eid)}</a></th>'
                    + td("Subject", subj, role="sub") + td("Created", ago(created), role="meta-q")
                    + td("Compositions", str(n), "fui-num", role="meta-2") + "</tr>")
    return table("EHRs", th("EHR") + th("Subject") + th("Created", key="c", sort="descending") + th("Compositions", num=True), rows)


def window_pager(first=1, last=10, more=True):
    """R12: an AQL window has no total, so the pager says the rows on screen and offers Next."""
    prev = '<button type="button" class="fui-button fui-button-secondary fui-button-small" disabled>' + icon("chevron-left", "14") + '<span class="fui-step-words">Previous</span><span class="fui-sr-only"> (unavailable)</span></button>'
    nxt = '<button type="button" class="fui-button fui-button-secondary fui-button-small"><span class="fui-step-words">Next</span>' + icon("chevron-right", "14") + "</button>"
    return (f'<nav class="fui-pager" aria-label="EHR pages"><p role="status">Rows {first}–{last}, newest first</p>'
            f'<div class="fui-pager-steps">{prev}{nxt if more else ""}</div><div class="fui-pager-sizes"><span class="fui-muted">The CDR reports no total for an AQL window</span></div></nav>')


def ehrs_page(body=None, value="", chips=(), foot=None):
    tb = tpl.toolbar("EHR id or subject id", value)
    c = tpl.card(body if body is not None else ehr_rows(), foot=window_pager() if foot is None else foot, label="EHRs", title="EHRs")
    return shell("EHRs", "ehrs", ehrs_heading() + tpl.list_body("", tb, tpl.active(chips), c))


def page_ehrs():
    return ehrs_page()


def page_ehrs_hover():
    return ehrs_page(ehr_rows(hover=1))


def page_ehrs_found():
    tb = tpl.toolbar("EHR id or subject id")
    chips = (f'<div class="fui-filter-chips"><span class="fui-small fui-muted">Showing</span><span class="fui-filter-chip">Subject id {masked("537", static=True)}'
             f'<button type="button" aria-label="Remove the filter subject id">{icon("x", "12")}</button></span>{button("ghost", "Clear filters", "x", small=True)}</div>')
    c = tpl.card(ehr_rows(EHRS[-1:]), label="EHRs", title="EHRs", count="1")
    return shell("EHRs", "ehrs", ehrs_heading() + tpl.list_body("", tb, chips, c))


def page_ehrs_no_match():
    tb = tpl.toolbar("EHR id or subject id", "0199ffff-0000-7000-8000-000000000000")
    c = tpl.card(empty_line("No EHR has that EHR id or subject id."), label="EHRs", title="EHRs", count="0")
    return shell("EHRs", "ehrs", ehrs_heading() + tpl.list_body("", tb, "", c))


def page_ehrs_loading():
    rows = "".join(f'<tr aria-hidden="true"><th scope="row">{skeleton_lines([60])}</th><td>{skeleton_lines([50])}</td><td>{skeleton_lines([40])}</td><td>{skeleton_lines([30])}</td></tr>' for _ in range(6))
    t = table("EHRs", th("EHR") + th("Subject") + th("Created") + th("Compositions", num=True), [rows])
    c = tpl.card(t, label="EHRs", title="EHRs")
    return shell("EHRs", "ehrs", ehrs_heading() + '<p class="fui-sr-only" role="status">Reading the newest EHRs</p>' + tpl.list_body("", tpl.toolbar("EHR id or subject id"), "", c))


def page_ehrs_empty():
    c = tpl.card(empty_line("The CDR holds no EHR yet."), label="EHRs", title="EHRs", count="0")
    return shell("EHRs", "ehrs", ehrs_heading() + tpl.list_body("", "", "", c))


# ---- New EHR ----

def edit_shell(title, current, crumbs, sub, body, dock_html, toast_html="", overlay=None):
    """An editor: the trail ends with the object, the title says Edit or New."""
    h = page_heading(title, subtitle=sub, crumbs=crumbs, here=False)
    doc = f"{title} · {crumbs[-1][0]}" if title == "Edit" else title
    return shell(doc, current, h + f'<div class="fui-stack">{body}</div>', dock_html=dock_html, toast_html=toast_html, overlay=overlay)


def msg(glyph, text):
    return f'{icon(glyph, "16")}<span>{text}</span>'


def edit_dock(message="", save="Save", back_href="#", disabled=None, state_="ready"):
    m = f'<p class="fui-dock-message">{message}</p>' if message else "<span></span>"
    if state_ == "saving":
        acts = button("secondary", "Discard", attrs="disabled") + busy_button("primary", "Saving")
    elif disabled:
        acts = link_button("secondary", "Discard", back_href, "x") + f'<button type="button" class="fui-button fui-button-primary" disabled title="{e(disabled)}">{icon("save", "16")}{save}</button>'
    else:
        acts = link_button("secondary", "Discard", back_href, "x") + button("primary", save, "save", keys="Ctrl s")
    return dock(m, acts, "")


def ehr_new(top="", ehr_id="", invalid=False):
    subj = section("Subject", intro("Bind the EHR to an outside identity, or leave both empty for an EHR with no external subject.")
                   + '<div class="fui-form-grid">'
                   + field("s-id", "Subject id", '<div class="fui-input-group">' + input_("s-id", "P-20517-284", type_="password", extra='class="fui-input fui-mono" spellcheck="false"') + icon_button("eye", "Show the subject id", cls="fui-input-toggle", attrs='aria-pressed="false"') + "</div>", help_="Masked as you type, as everywhere in the viewer.", cls="fui-span-2")
                   + field("s-ns", "Namespace", input_("s-ns", "demo.hospital.mrn", extra='class="fui-input fui-mono" spellcheck="false"'), cls="fui-span-2")
                   + "</div>")
    err = "Not a UUID. Leave it empty and the CDR mints one." if invalid else None
    eid = section("", '<div class="fui-form-grid">' + field("e-id", "EHR id", input_("e-id", ehr_id, "Leave empty for a new id", extra='class="fui-input fui-mono" spellcheck="false"' + (' aria-invalid="true"' if invalid else "")),
                                                                    help_=None if invalid else "Optional. A UUID; the CDR refuses an id already in use.", error=err, cls="fui-span-2") + "</div>")
    if invalid:
        d = edit_dock(msg("circle-x", "Fix the EHR id before creating."), "Create this EHR", "ehrs.html", disabled="Fix the EHR id first")
    else:
        d = edit_dock("", "Create this EHR", "ehrs.html")
    return edit_shell("New EHR", "ehrs", [("EHRs", "ehrs.html")], "An EHR with a subject, or with none.", top + sections(subj, eid, card=True), d)


def page_ehr_new():
    return ehr_new()


def page_ehr_new_invalid():
    return ehr_new(ehr_id="0199c2a4-7e1b", invalid=True)


def page_ehr_new_conflict():
    rf = refusal("The CDR refused the EHR, in its own words below.", request=f"PUT /ferroehr/rest/openehr/v1/ehr/{E1}",
                 issues=[("bad", "error", "409 Conflict", f"An EHR with id {E1} already exists.")])
    return ehr_new(top=rf, ehr_id=E1).replace('<p class="fui-dock-message"></p>', "")


# ---- one EHR ----

EHR_TABS = [("compositions", "Compositions", "ehr.html", 8), ("directory", "Directory", "ehr-directory.html"), ("status", "Status", "ehr-status.html"),
            ("contributions", "Contributions", "ehr-contributions.html"), ("tags", "Tags", "ehr-tags.html")]
EHR_TABS_NODIR = [t for t in EHR_TABS if t[0] != "directory"]


def ehr_more(admin=True, open_=False):
    items = [menu_item("Commit several changes", "git-commit", href="contribution-new.html"),
             menu_item("Edit status", "pencil", href="ehr-status-edit.html"),
             menu_item("Copy EHR id", "copy")]
    if admin:
        items += [MENU_SEP, menu_item("Delete EHR…", "trash-2", danger=True, href="ehr-delete.html")]
    return items


def ehr_header(more_open=False, admin=True, subject=True, revealed=False, queryable=True, reveal_state=None, eid=E1, created="2026-03-02T10:14"):
    ag = action_group(link_button("primary", "New composition", "composition-new.html", "plus"), ehr_more(admin), open_=more_open, label="More actions for the EHR")
    return page_heading(f"EHR {eid[:8]}…{eid[-4:]}", actions=ag, subtitle=f"Created {when(created)} on <code>{SYSTEM_ID}</code>.", crumbs=[("EHRs", "ehrs.html")],
                        after=subject_line(subject, queryable, True, revealed, reveal_state))


def ehr_page(tab="compositions", panel=None, header=None, tabs_list=EHR_TABS, toast_html="", overlay=None, reader=False, dock_html=""):
    h = header or ehr_header(admin=not reader)
    p = panel if panel is not None else comps_panel()
    return shell(f"EHR {E1[:8]}", "ehrs", h + f'<div class="fui-stack">{tabbed(tabs("Views of this EHR", tabs_list, tab), p)}</div>',
                 toast_html=toast_html, overlay=overlay, reader=reader, dock_html=dock_html)


def comp_rows(items=COMPS):
    rows = []
    for key, name, tmpl, start, composer, versions, uid in items:
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="composition.html">{e(name)}</a><span class="fui-row-sub">{e(tmpl)}</span></th>'
                    + td("Start time", ago(start), role="sub") + td("Composer", e(composer), role="meta-q")
                    + td("Versions", str(versions), "fui-num", role="meta-2") + "</tr>")
    return table("Compositions", th("Composition") + th("Start time", key="s", sort="descending") + th("Composer") + th("Versions", num=True), rows)


def comps_toolbar(template=None, start=None, composer=""):
    return tpl.toolbar("Composer", composer, [tpl.filter_button("Template", template), tpl.filter_button("Start time", start)])


def comps_panel(rows=None, toolbar=None, chips="", note=""):
    tb = toolbar if toolbar is not None else comps_toolbar()
    return f'<div class="fui-panel"><div class="fui-panel-body fui-stack mk-routes-filters">{tb}{chips}</div>{rows if rows is not None else comp_rows()}{note}</div>'


def page_ehr(toast_html=""):
    return ehr_page(toast_html=toast_html)


def page_ehr_more():
    return ehr_page(header=ehr_header(more_open=True))


def page_ehr_filtered():
    tb = comps_toolbar("Vital signs encounter.v1", "1 Sep to 21 Oct")
    chips = tpl.active(["Template: Vital signs encounter.v1", "Start time: 2026-09-01 to 2026-10-21"])
    vs = [c for c in COMPS if c[2] == "Vital signs encounter.v1"]
    return ehr_page(panel=comps_panel(comp_rows(vs), tb, chips), tabs_list=[("compositions", "Compositions", "ehr.html", 3)] + EHR_TABS[1:])


def page_ehr_reveal():
    return ehr_page(overlay=reveal_dialog())


def page_ehr_revealed():
    return ehr_page(header=ehr_header(revealed=True))


def page_ehr_no_subject():
    eid = EHRS[4][0]
    note = f'<section class="fui-panel-section"><p class="fui-small fui-muted">This EHR has no directory. {link_button("ghost", "Create the directory", "ehr-directory.html", "folder-plus", small=True)}</p></section>'
    one = [("c", "Vital signs", "Vital signs encounter.v1", "2026-10-19T14:20", "Demo Nurse", 1, "0199c26a-9e10-7a42-8b3d-1c5f7e9a0b28")]
    h = ehr_header(subject=False, eid=eid, created="2026-10-19T14:05")
    return shell("EHR", "ehrs", h + f'<div class="fui-stack">{tabbed(tabs("Views of this EHR", [("compositions", "Compositions", "ehr-no-subject.html", 1)] + EHR_TABS_NODIR[1:], "compositions"), comps_panel(comp_rows(one), note=note))}</div>')


def page_ehr_not_found():
    h = page_heading("EHR 0199c2a4…1a99", crumbs=[("EHRs", "ehrs.html")])
    body = (refusal("The CDR has no EHR with this id (404).", request="GET /ferroehr/rest/openehr/v1/ehr/0199c2a4-7e1b-7c3e-9a51-4f2d8b6e1a99")
            + f'<div class="fui-cluster">{link_button("primary", "Back to EHRs", "ehrs.html", "arrow-left")}</div>')
    return shell("EHR not found", "ehrs", h + f'<div class="fui-stack">{body}</div>')


def page_ehr_delete():
    d = confirm_delete("del", f"Delete EHR {E1[:8]}…?", "Removes the EHR and all its data for good: 8 compositions, 8 contributions, its status, its directory and their audit details. Nothing stays readable, and it cannot be undone.",
                       True, "Delete the EHR for good",
                       extra=field("del-confirm", "Type the first 8 characters of the EHR id to confirm", input_("del-confirm", "0199c2a4", extra='class="fui-input fui-mono"')))
    return ehr_page(overlay=d)


def page_ehr_reader():
    return ehr_page(header=ehr_header(more_open=True, admin=False), reader=True)


# ---- Directory (E4) ----

def folder_tree(editing=False, rename=False):
    def acts(name, ref=False):
        if not editing:
            return ""
        if ref:
            return icon_button("x", f"Remove the reference {name}")
        return icon_button("folder-plus", f"Add a folder in {name}") + icon_button("file-text", f"Add a reference in {name}") + icon_button("pencil", f"Rename {name}") + icon_button("trash-2", f"Remove {name}")
    r1 = tree_item("Vital signs", "0199f1d0…7c11", leaf=True, glyph="file-text", ref=True, href="composition.html", actions=acts("Vital signs", True))
    r2 = tree_item("Laboratory result report", "0199e4b7…8a37", leaf=True, glyph="file-text", ref=True, href="composition.html", actions=acts("Laboratory result report", True))
    r3 = tree_item("Discharge summary", "01995f22…2b53", leaf=True, glyph="file-text", ref=True, href="composition.html", actions=acts("Discharge summary", True))
    f1 = tree_item("2026-10 Hypertension follow-up", children=[r1, r2], expanded=True, glyph="folder", count="2 items", actions=acts("2026-10 Hypertension follow-up"),
                   rename="2026-10 Hypertension follow-up, week 43" if rename else None)
    f2 = tree_item("2026-08 Admission", children=[r3], expanded=True, glyph="folder", count="1 item", actions=acts("2026-08 Admission"))
    ep = tree_item("Episodes", children=[f1, f2], expanded=True, glyph="folder", count="2 folders", actions=acts("Episodes"))
    co = tree_item("Correspondence", leaf=True, glyph="folder", count="empty", actions=acts("Correspondence"))
    root = tree_item("root", children=[ep, co], expanded=True, glyph="folder", selected=not editing, actions=acts("root"))
    return tree([root], "The directory of this EHR", "fui-folder-tree" + (" is-editing" if editing else ""))


DIR_VERSIONS = [("3", "2026-10-21T08:20", "Demo Nurse", "Modification"), ("2", "2026-10-14T10:45", "Demo Clinician", "Modification"), ("1", "2026-03-02T10:20", "Demo Operator", "Creation")]


def dir_versions_table(restore_focus=False):
    rows = []
    for i, (v, w, who, how) in enumerate(DIR_VERSIONS):
        act = "" if i == 0 else link_button("ghost", "Restore…", "ehr-directory-restore.html", "rotate-ccw", small=True, attrs=f'aria-label="Restore version {v}"')
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="ehr-directory.html">{version_cell(v, current=i == 0)}</a></th>'
                    + td("Change", how, role="sub") + td("Committed by", who, role="meta-q") + td("When", ago(w), role="meta-2")
                    + td("", act, "fui-row-menu", role="menu") + "</tr>")
    return table("Directory versions", th("Version") + th("Change") + th("Committed by") + th("When") + th('<span class="fui-sr-only">Restore</span>'), rows)


def dir_panel(menu_open=False):
    m = menu([menu_item("Copy the directory id", "copy"), MENU_SEP, menu_item("Delete the directory…", "trash-2", danger=True, href="ehr-directory-delete.html")], label="Directory actions", sheet=True) if menu_open else ""
    trig = (f'<span class="fui-anchor"><a href="ehr-directory-more.html" class="fui-icon-button" role="button" aria-haspopup="menu" aria-expanded="{"true" if menu_open else "false"}" '
            f'aria-label="More actions for the directory" title="More actions">{icon("ellipsis", "16")}</a>{m}</span>')
    tools = link_button("secondary", "Edit the directory", "ehr-directory-edit.html", "pencil", small=True) + trig
    path = (f'<div class="fui-at-time">{field("d-path", "Show one folder by path", input_("d-path", "", "/Episodes/2026-10 Hypertension follow-up", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
            f'{button("secondary", "Show", "folder")}</div>')
    tree_s = section("Folders", folder_tree() + path, actions=tools)
    vers = section("Versions", intro("The newest three; an older one opens read-only and can be restored.") + dir_versions_table()
                   + f'<div class="fui-add-row">{button("ghost", "Load older versions", "history", small=True)}</div>'
                   + at_time("dir-at", what="the directory as it stood then"), count=3)
    return sections(tree_s, vers)


def page_ehr_directory(overlay=None):
    return ehr_page("directory", dir_panel(), overlay=overlay)


def page_ehr_directory_more():
    return ehr_page("directory", dir_panel(menu_open=True))


def page_ehr_directory_restore():
    d = (f'<div role="dialog" aria-modal="true" aria-labelledby="restore-title" class="fui-dialog fui-modal mk-open" open>'
         f'<h2 id="restore-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Restore version 2 of the directory?</h2>'
         '<p>The CDR commits version 2\'s whole tree as a new version 4. Version 3 stays in the history. If the directory changed since you opened it, the CDR refuses (412) and nothing is written.</p>'
         f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Restore as version 4", "rotate-ccw")}</div></div>')
    return page_ehr_directory(overlay=d)


def page_ehr_directory_delete():
    d = confirm_delete("del", "Delete the directory?", "Marks the directory deleted with a new version; its three versions stay readable, and the compositions it points at are not touched. A new directory can be created afterwards.",
                       False, "Delete the directory")
    return page_ehr_directory(overlay=d)


# ---- Status (E2) ----

STATUS_JSON = """{
  "_type": "EHR_STATUS",
  "archetype_node_id": "openEHR-EHR-EHR_STATUS.generic.v1",
  "name": { "_type": "DV_TEXT", "value": "EHR Status" },
  "uid": {
    "_type": "OBJECT_VERSION_ID",
    "value": "0199c2a4-7e2c-7d10-8f3a-6b1e9c4d2a55::cdr.example.org::2"
  },
  "subject": {
    "_type": "PARTY_SELF",
    "external_ref": {
      "_type": "PARTY_REF",
      "id": { "_type": "GENERIC_ID", "value": "••••••537", "scheme": "demo.hospital.mrn" },
      "namespace": "demo.hospital.mrn",
      "type": "PERSON"
    }
  },
  "is_queryable": true,
  "is_modifiable": true
}"""

STATUS_VERSIONS = [("2", "2026-10-14T10:41", "Demo Clinician", "Modification"), ("1", "2026-03-02T10:14", "Demo Operator", "Creation")]


def status_versions_table():
    rows = []
    for i, (v, w, who, how) in enumerate(STATUS_VERSIONS):
        href = "ehr-status.html" if i == 0 else "ehr-status-version.html"
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{version_cell(v, current=i == 0)}</a></th>'
                    + td("Change", how, role="sub") + td("Committed by", who, role="meta-q") + td("When", ago(w), role="meta-2") + "</tr>")
    return table("Status versions", th("Version") + th("Change") + th("Committed by") + th("When"), rows)


def status_panel(version="2", queryable=True, top=""):
    f = facts([("Version", f'<code>0199c2a4…2a55::{SYSTEM_ID}::{version}</code>'), ("Committed", when("2026-10-14T10:41") if version == "2" else when("2026-03-02T10:14")),
               ("Contribution", '<a href="contribution.html">0199d6a3…0a73</a>'), ("Signed", "No"), ("Archetype", arch("openEHR-EHR-EHR_STATUS.generic.v1"))])
    text = STATUS_JSON if queryable else STATUS_JSON.replace('"is_queryable": true', '"is_queryable": false')
    if version == "1":
        text = text.replace("::2", "::1")
    doc = section("Document", intro("The subject id is masked here as in the header; Reveal shows it.") + codeview(f"EHR_STATUS · version {version}", text, what="the EHR status"))
    vers = section("Versions", status_versions_table() + at_time("st-at", what="the status as it stood then"), count=2)
    return top + sections(section("This version", f, actions=link_button("secondary", "Edit status", "ehr-status-edit.html", "pencil", small=True) if version == "2" else ""), doc, vers)


def page_ehr_status(toast_html=""):
    return ehr_page("status", status_panel(), toast_html=toast_html)


def page_ehr_status_version():
    top = notice("info", "history", f'You are reading version 1 of 2, committed {when("2026-03-02T10:14")}. <a href="ehr-status.html">Read the latest</a>')
    return ehr_page("status", f'<div class="fui-stack">{top}{status_panel("1")}</div>')


def page_ehr_status_not_queryable():
    top = notice("warn", "triangle-alert", "This EHR is not queryable: AQL over the population leaves it out.")
    h = ehr_header(queryable=False)
    return ehr_page("status", f'<div class="fui-stack">{top}{status_panel(queryable=False)}</div>', header=h)


# ---- Contributions ----

WRITES = [0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 2]


def contrib_rows(items=CONTRIBS):
    rows = []
    for uid, w, who, how, n in items:
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="contribution.html">{short(uid)}</a></th>'
                    + td("Committed", ago(w), role="sub") + td("Committer", e(who), role="meta-q") + td("Change", how, role="meta-2")
                    + td("Versions", str(n), "fui-num", role="wide") + "</tr>")
    return table("Contributions", th("Contribution") + th("Committed", key="c", sort="descending") + th("Committer") + th("Change") + th("Versions", num=True), rows)


def page_ehr_contributions():
    chart = line_chart("writes", "Writes per day to this EHR", "Writes per day, 22 September to 21 October 2026: 7 writes, at most 2 on one day.",
                       [("Writes", WRITES, 0)], [(0, "22 Sep"), (9, "1 Oct"), (19, "11 Oct"), (29, "21 Oct")], height=140, legend=False)
    act = section("Writes per day", chart)
    lst = section("Contributions", contrib_rows(), count=8)
    return ehr_page("contributions", sections(act, lst))


# ---- Tags ----

EHR_TAGS = [
    ("Vital signs", "composition.html", "versioned composition", [("follow-up", "2026-11-04", "/content[openEHR-EHR-OBSERVATION.blood_pressure.v2]"), ("reviewed", "Demo Clinician", None)]),
    ("Problem list", "composition.html", "version 4", [("migrated", "legacy-ehr-2025", None)]),
    ("EHR status", "ehr-status.html", "versioned EHR status", [("research-consent", "withdrawn", None)]),
]


def tag_rows(groups=EHR_TAGS):
    body = ""
    for name, href, form, tags_ in groups:
        body += (f'<tr class="fui-group-row"><th scope="rowgroup" colspan="3"><a href="{href}">{e(name)}</a> <span class="fui-small fui-muted">· {form}</span></th></tr>')
        for k, v, p in tags_:
            body += (f'<tr><th scope="row" data-role="title"><code>{e(k)}</code></th>' + td("Value", e(v), role="sub")
                     + td("Target path", path(p) if p else '<span class="mk-dash">the whole object</span>', role="meta-q") + "</tr>")
    return table("Tags in this EHR", th("Key") + th("Value") + th("Target path"), [body])


def page_ehr_tags():
    tb = tpl.toolbar("Key, value or target path")
    p = f'<div class="fui-panel"><div class="fui-panel-body fui-stack mk-routes-filters">{tb}<p class="fui-small fui-muted">A tag is set on its object, on that object\'s Tags tab; this list reads them all.</p></div>{tag_rows()}</div>'
    return ehr_page("tags", p)


def page_ehr_tags_filtered():
    tb = tpl.toolbar("Key, value or target path", "follow-up")
    p = f'<div class="fui-panel"><div class="fui-panel-body fui-stack mk-routes-filters">{tb}{tpl.active(["Key: follow-up"])}</div>{tag_rows([EHR_TAGS[0][:3] + ([EHR_TAGS[0][3][0]],)])}</div>'
    return ehr_page("tags", p)


# ---- editors: status, directory ----

def status_edit(top="", d=None):
    flags = section("Capabilities", check("st-q", "Queryable", True, "Population queries (AQL) include this EHR.")
                    + check("st-m", "Modifiable", True, "New content may be committed to this EHR."))
    other = editor('{\n  "_type": "ITEM_TREE",\n  "archetype_node_id": "at0001",\n  "name": { "_type": "DV_TEXT", "value": "Details" },\n  "items": []\n}', label="other_details",
                   lang_word="Canonical JSON · ITEM_STRUCTURE", pos="Ln 5, Col 13")
    od = section("Other details", intro("An ITEM_STRUCTURE in canonical JSON, empty to remove it; every other attribute goes back as the CDR served it.") + other)
    dk = d or edit_dock(msg("pencil", "1 change: Other details."), "Save as version 3", "ehr-status.html")
    return edit_shell("Edit", "ehrs", [("EHRs", "ehrs.html"), (f"EHR {E1[:8]}…", "ehr-status.html")], "Status version 2, opened 09:38.", top + sections(flags, od, card=True), dk)


def page_ehr_status_edit():
    return status_edit()


def page_ehr_status_conflict():
    rf = refusal("Another client committed a new status since you opened it, so the CDR refused the save (412).", request=f"PUT /ferroehr/rest/openehr/v1/ehr/{E1}/ehr_status",
                 issues=[("bad", "error", "412 Precondition Failed", "If-Match names version 2; the current version is 3.")])
    rf += f'<div class="fui-cluster">{link_button("primary", "Reload version 3", "ehr-status.html", "refresh-cw")}<span class="fui-small fui-muted">Your edit stays in this page until you leave it.</span></div>'
    return status_edit(rf, edit_dock(msg("circle-x", "Not saved. Reload the current version first."), "Save as version 3", "ehr-status.html", disabled="Reload the current version first"))


def dir_edit(top="", d=None):
    body = section("Folders", intro("Changes stay in this page until you save them as one new version.") + folder_tree(editing=True, rename=True)
                   + f'<div class="fui-add-row">{button("ghost", "Add a folder at the root", "folder-plus", small=True)}</div>')
    adv = section("", f'<p class="fui-small fui-muted">The tree is the canonical FOLDER; {link_button("ghost", "Edit it as JSON", "#", "braces", small=True, attrs="rel=" + chr(34) + "nofollow" + chr(34))}</p>')
    dk = d or edit_dock(msg("pencil", "2 changes: a folder renamed, a reference removed."), "Save as version 4", "ehr-directory.html")
    return edit_shell("Edit", "ehrs", [("EHRs", "ehrs.html"), (f"EHR {E1[:8]}…", "ehr-directory.html")], "Directory version 3, opened 09:38.", top + sections(body, card=True), dk)


def page_ehr_directory_edit():
    return dir_edit()


def page_ehr_directory_conflict():
    rf = refusal("Another client changed the directory since you opened it, so the CDR refused the save (412).", request=f"PUT /ferroehr/rest/openehr/v1/ehr/{E1}/directory",
                 issues=[("bad", "error", "412 Precondition Failed", "If-Match names version 3; the current version is 4.")])
    rf += f'<div class="fui-cluster">{link_button("primary", "Reload version 4", "ehr-directory.html", "refresh-cw")}{button("secondary", "Overwrite with my tree…", "save")}</div>'
    return dir_edit(rf, edit_dock(msg("circle-x", "Not saved. Reload, or overwrite with your tree."), "Save as version 4", "ehr-directory.html", disabled="Reload or overwrite first"))


# ---- the contribution builder ----

VITALS_JSON = """{
  "_type": "COMPOSITION",
  "archetype_node_id": "openEHR-EHR-COMPOSITION.encounter.v1",
  "name": { "_type": "DV_TEXT", "value": "Vital signs" },
  "archetype_details": {
    "archetype_id": { "value": "openEHR-EHR-COMPOSITION.encounter.v1" },
    "template_id": { "value": "Vital signs encounter.v1" },
    "rm_version": "1.2.0"
  },
  "language": { "terminology_id": { "value": "ISO_639-1" }, "code_string": "en" },
  "territory": { "terminology_id": { "value": "ISO_3166-1" }, "code_string": "NL" },
  "category": { "value": "event", "defining_code": { "code_string": "433" } },
  "composer": { "_type": "PARTY_IDENTIFIED", "name": "Demo Nurse" },
  "context": { "start_time": { "value": "2026-10-21T08:14:00+02:00" }, "setting": { "value": "other care", "defining_code": { "code_string": "238" } } },
  "content": [ "… 2 observations …" ]
}"""


def staged_card(n, title, kind, change, body, issue=None):
    acts = icon_button("x", f"Remove change {n}")
    iss = f'<p class="fui-clause-issue" role="note">{icon("circle-x", "14")}<span>{issue}</span></p>' if issue else ""
    return (f'<div class="fui-repeater-card"><div class="fui-repeater-card-head"><h3>{n}. {title}</h3>{acts}</div>'
            f'<div class="fui-form-grid">{field(f"k{n}", "Change", select(f"k{n}", [("cc", "New composition"), ("ca", "Amended composition"), ("sm", "EHR status")], kind), cls="fui-span-2")}'
            f'{field(f"t{n}", "Change type", select(f"t{n}", [("249", "Creation"), ("251", "Amendment"), ("250", "Modification")], change), cls="fui-span-2")}</div>{body}{iss}</div>')


def contribution_new(top="", refused=False, d=None):
    c1 = staged_card(1, "New composition: Vital signs", "cc", "249",
                     field("tm1", "Template", select("tm1", [(t[0], t[0]) for t in TEMPLATES[:5]], "Vital signs encounter.v1"), cls="fui-span-2") + editor(VITALS_JSON, label="The composition of change 1", pos="Ln 16, Col 3", dirty=True,
                                                                                                                                  diags={14: ("bad", "VALIDATION", "/context/setting: the code is missing.")} if refused else None),
                     issue=None)
    c2 = staged_card(2, "EHR status: version 2 to 3", "sm", "250", '<p class="fui-small fui-muted">Loaded from version 2, which it supersedes. Change: Queryable cleared.</p>')
    staged = section("Staged changes", intro("Every change commits together, or none of them does; leaving this page discards them.") + f'<div class="fui-repeater">{c1}{c2}</div>'
                     + f'<div class="fui-add-row">{button("ghost", "Stage a change", "plus", small=True)}</div>', count=2)
    audit = section("Contribution audit", '<div class="fui-form-grid">' + field("au-d", "Description", input_("au-d", "Follow-up visit and research opt-out"), cls="fui-span-2")
                    + field("au-c", "Committer", input_("au-c", "", "Leave empty to commit as Demo Operator"), cls="fui-span-2") + "</div>")
    if refused:
        dk = edit_dock(msg("circle-x", "Nothing was committed."), "Commit 2 changes", "ehr-contributions.html")
    else:
        dk = d or edit_dock(msg("git-commit", "As one contribution: all of them or none."), "Commit 2 changes", "ehr-contributions.html")
    return edit_shell("New contribution", "ehrs", [("EHRs", "ehrs.html"), (f"EHR {E1[:8]}…", "ehr.html")], "Several changes, committed as one atomic contribution.",
                      top + sections(staged, audit, card=True), dk)


def page_contribution_new():
    return contribution_new()


def page_contribution_committed():
    top = load_report("ok", f'Committed {when("2026-10-21T09:41")} as contribution 0199f1e2…4d07.', counts=(("2", "versions"),),
                      actions=link_button("secondary", "Open the contribution", "contribution.html", "arrow-right", small=True))
    vers = section("Versions it made", table("Versions it made", th("Object") + th("Version") + th("Change"), [
        f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="composition.html">Vital signs</a></th>{td("Version", "Version 1", role="sub")}{td("Change", "Creation", role="meta-q")}</tr>'
        f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="ehr-status.html">EHR status</a></th>{td("Version", "Version 3", role="sub")}{td("Change", "Modification", role="meta-q")}</tr>']), count=2)
    h = page_heading("New contribution", subtitle="Several changes, committed as one atomic contribution.", crumbs=[("EHRs", "ehrs.html"), (f"EHR {E1[:8]}…", "ehr.html")], here=False)
    return shell("New contribution", "ehrs", h + f'<div class="fui-stack">{top}{sections(vers, card=True)}</div>')


def page_contribution_refused():
    rf = refusal("The CDR refused the contribution, so nothing was committed.", request=f"POST /ferroehr/rest/openehr/v1/ehr/{E1}/contribution",
                 issues=[("bad", "error", "422 Unprocessable", "versions[0].data /context/setting: DV_CODED_TEXT needs defining_code.")])
    return contribution_new(rf, refused=True)


def contribution_body(demo=False):
    f = facts([("Committed", when("2026-10-14T10:40")), ("Committer", "Demo Clinician"), ("Change type", "Creation"), ("Description", "Problems and medication reviewed at the outpatient visit"),
               ("System", f"<code>{SYSTEM_ID}</code>")])
    if demo:
        f = facts([("Committed", when("2026-10-20T15:02")), ("Committer", "Demo Operator"), ("Change type", "Modification"), ("Description", "Identity corrected"), ("System", f"<code>{SYSTEM_ID}</code>")])
        rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="party.html">Demo Person 0417</a></th>{td("Kind", "Person", role="sub")}{td("Version", "Version 2", role="meta-q")}</tr>']
        t = table("Versions it touched", th("Object") + th("Kind") + th("Version"), rows)
    else:
        rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="composition.html">Problem list</a></th>{td("Kind", "Composition", role="sub")}{td("Version", "Version 1", role="meta-q")}</tr>',
                f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="ehr-status.html">EHR status</a></th>{td("Kind", "EHR status", role="sub")}{td("Version", "Version 2", role="meta-q")}</tr>']
        t = table("Versions it touched", th("Object") + th("Kind") + th("Version"), rows)
    uid, who, at, ch, code, n = ("0199eb10-6c2f-7a51-9d04-3e8b1f7d7d42", "Demo Operator", "2026-10-20T15:02:40+02:00", "modification", "251", "1 OBJECT_REF") if demo else \
        ("0199d6a3-31b8-7f04-8c5a-2e9d6b1f0a73", "Demo Clinician", "2026-10-14T10:40:12+02:00", "creation", "249", "2 OBJECT_REFs")
    rec = codeview("CONTRIBUTION", '{\n  "_type": "CONTRIBUTION",\n  "uid": { "value": "' + uid + '" },\n  "audit": {\n    "system_id": "cdr.example.org",\n    "committer": { "_type": "PARTY_IDENTIFIED", "name": "' + who + '" },\n    "time_committed": { "value": "' + at + '" },\n    "change_type": { "value": "' + ch + '", "defining_code": { "code_string": "' + code + '" } }\n  },\n  "versions": [ "… ' + n + ' …" ]\n}', what="the contribution")
    return sections(section("Audit", f), section("Versions it touched", t, count=len(rows)), section("Record", rec))


def page_contribution():
    h = page_heading("Contribution 0199d6a3…0a73", subtitle="One change set, committed together.", crumbs=[("EHRs", "ehrs.html"), (f"EHR {E1[:8]}…", "ehr-contributions.html")])
    return shell("Contribution", "ehrs", h + f'<div class="fui-stack">{contribution_body()}</div>')
