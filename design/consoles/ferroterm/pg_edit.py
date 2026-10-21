# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The three editors, each a T2 page in edit reached from its resource (or
from New on its list): the code system editor, the value set composer and
the concept map editor. A save states If-Match; the docked bar holds
Discard and Save. A visitor composes and previews, and is offered the
sign-in where a writer has Save."""
from common import (
    MENU_SEP, busy_button, button, check, dock, e, empty_quiet, field, icon, icon_button, input_, link_button, notice, page_heading,
    refusal, select, shell, table, td, th, toast, toasts, when,
)
from data import FINDINGS, LEGACY, LEGACY_MAPPINGS, REL_WORD, SF
from term import canon, clause, coding, load_report, quiet, relation, section, sections

STATUS_OPTS = [("draft", "Draft"), ("active", "Active"), ("retired", "Retired")]


def edit_dock(msg="", state="ready", save="Save", back_href="code-system.html", disabled=None, visitor=False):
    m = f'<p class="fui-dock-message">{msg}</p>' if msg else '<span></span>'
    if visitor:
        acts = link_button("secondary", "Discard", back_href, "x") + link_button("primary", "Sign in to save", "sign-in.html", "log-in")
    elif state == "saving":
        acts = button("secondary", "Discard", attrs="disabled") + busy_button("primary", "Saving")
    elif disabled:
        acts = link_button("secondary", "Discard", back_href, "x") + f'<button type="button" class="fui-button fui-button-primary" disabled title="{e(disabled)}">{icon("save", "16")}{save}</button>'
    else:
        acts = link_button("secondary", "Discard", back_href, "x") + button("primary", save, "save", keys="Ctrl s")
    return dock(m, acts, "")


def msg(glyph, text):
    return f'{icon(glyph, "16")}<span>{text}</span>'


def edit_shell(title, current, crumbs, sub, body, dock_html, toast_html="", overlay=None, visitor=False):
    """An editor: the trail ends with the resource, the title says Edit or New."""
    h = page_heading(title, subtitle=sub, crumbs=crumbs, here=False)
    doc = f"{title} · {crumbs[-1][0]}" if title == "Edit" else title
    return shell(doc, current, h + f'<div class="fui-stack">{body}</div>', dock_html=dock_html, toast_html=toast_html, overlay=overlay, visitor=visitor)


# ---- the code system editor ----

def cs_meta(new=False, error=False):
    url = "" if new else SF
    err = "A code system needs a canonical before it can be saved." if error else None
    return section("Metadata", '<div class="fui-form-grid">'
                   + field("cs-url", "Canonical URL", input_("cs-url", url, "https://terminology.example.org/fhir/CodeSystem/…", extra='class="fui-input fui-mono"' + (' aria-invalid="true"' if error else "")), error=err, cls="fui-span-2")
                   + field("cs-ver", "Business version", input_("cs-ver", "" if new else "3"))
                   + field("cs-name", "Name", input_("cs-name", "" if new else "SyntheticClinicalFindings", extra='class="fui-input fui-mono"'))
                   + field("cs-title", "Title", input_("cs-title", "" if new else "Synthetic clinical findings"), cls="fui-span-2")
                   + field("cs-status", "Publication status", select("cs-status", STATUS_OPTS, "draft" if new else "active"))
                   + field("cs-content", "Content mode", select("cs-content", [("complete", "All concepts"), ("fragment", "Some concepts"), ("example", "Examples only")], "complete"))
                   + '</div><div class="fui-add-row">' + check("cs-case", "Codes are case sensitive", True) + "</div>")


def concept_rows(rows, focus=None):
    out = []
    for code, display, parent, inactive, nl in rows:
        life = select(f"life-{code}", [("active", "Active"), ("deprecated", "Deprecated"), ("retired", "Retired")], "retired" if inactive else "active", cls="fui-select fui-select-small")
        out.append(f'<tr><th scope="row" data-role="title"><code>{e(code)}</code></th>'
                   + td("Display", input_(f"d-{code}", display, extra=f'aria-label="Display of {e(code)}"'), role="sub")
                   + td("Parent", f"<code>{e(parent)}</code>" if parent else '<span class="mk-dash">root</span>', role="meta-q")
                   + td("Lifecycle", life, role="meta-2") + "</tr>")
    return table("Concepts", th("Code") + th("Display") + th("Parent") + th("Lifecycle"), out)


def cs_editor_body(new=False, error=False):
    props = section("Declared properties", '<p class="fui-small fui-muted fui-section-intro">The lifecycle properties are declared by the editor on every save, so they are not listed here.</p>'
                    + '<div class="fui-repeater"><div class="fui-repeater-head"><span class="fui-label">Property code</span><span class="fui-label">Type</span><span></span></div>'
                    + ("" if new else f'<div class="fui-repeater-row">{input_("p-1", "parent", extra="aria-label=" + chr(34) + "Property code" + chr(34) + " class=" + chr(34) + "fui-input fui-mono" + chr(34))}{select("p-1t", [("code", "code"), ("string", "string"), ("boolean", "boolean")], "code")}{icon_button("x", "Remove the property parent")}</div>')
                    + f'</div><div class="fui-add-row">{button("ghost", "Declare a property", "plus", small=True)}</div>', count=0 if new else 1)
    if new:
        cbody = quiet("No concept yet.")
    else:
        cbody = ('<p class="fui-small fui-muted fui-section-intro">A concept is retired rather than removed, so a code that has been published keeps meaning what it meant.</p>'
                 + f'{concept_rows(FINDINGS[:1] + FINDINGS[9:13])}'
                 + '<p class="fui-small fui-muted">5 of 14 concepts shown, the general findings.</p>')
    concepts = section("Concepts", cbody + f'<div class="fui-add-row">{button("ghost", "Add a concept", "plus", small=True)}</div>', count=0 if new else 14)
    return sections(cs_meta(new, error), props, concepts, card=True)


CS_CRUMBS = [("Code systems", "code-systems.html"), ("Synthetic clinical findings", "code-system.html")]
CS_SUB = f'Version 3, opened {when("2026-10-21T09:36")}.'


def cs_edit(body=None, dock_html=None, top="", toast_html=""):
    return edit_shell("Edit", "systems", CS_CRUMBS, CS_SUB, top + (body or cs_editor_body()),
                      dock_html if dock_html is not None else edit_dock(msg("pencil", "1 change: SF-1390 Malaise, old code is retired."), save="Save this code system"), toast_html)


def page_code_system_edit():
    return cs_edit()


def page_code_system_saving():
    return cs_edit(dock_html=edit_dock(msg("pencil", "1 change: SF-1390 Malaise, old code is retired."), "saving", "Save this code system"))


def page_code_system_saved():
    top = load_report("ok", f'Saved {when("2026-10-21T09:41")}. The server now holds version 4.', actions=link_button("secondary", "Open the code system", "code-system.html", "arrow-right", small=True))
    check_ = section("What the server says about the retired concept", '<p class="fui-small fui-muted fui-section-intro">CodeSystem/$validate-code, run against the version that was just saved.</p>'
                     + f'<p>{coding("Malaise, old code", "SF-1390", inactive=True, inline=True)}</p><p class="fui-small">The server reports <code>SF-1390</code> as inactive.</p>')
    return cs_edit(body=sections(check_, card=True), top=top, dock_html="")


def page_code_system_conflict():
    rf = refusal("The save was refused: someone else changed this code system since it was opened.", request="PUT /r4/CodeSystem/synthetic-findings · If-Match: W/\"3\"",
                 issues=[("bad", "error", "conflict", "The server holds version 4 now. Reload it and apply the change again; your edits stay in the form until you do.")])
    rf += f'<div class="fui-cluster">{button("primary", "Reload the current version", "refresh-cw")}</div>'
    return cs_edit(top=rf, dock_html=edit_dock(msg("circle-x", "Not saved. Reload the current version first."), save="Save this code system", disabled="Reload the current version first"))


def page_code_system_refused():
    rf = refusal("The server refused the change, in its own words below.", request="PUT /r4/CodeSystem/synthetic-findings",
                 issues=[("bad", "error", "forbidden", "The access token carries no user/CodeSystem.cud scope. This account does not carry the permission for this change.")])
    return cs_edit(top=rf, dock_html=edit_dock(msg("circle-x", "Not saved: this account cannot change a code system."), save="Save this code system", disabled="This account cannot change a code system"))


def page_code_system_expired():
    rf = refusal("Your session ended, so the save was not sent.", issues=[("warn", "warning", "login", "Sign in again to make this change. The form keeps your edits in this page while you do.")])
    rf += f'<div class="fui-cluster">{link_button("primary", "Sign in again", "sign-in.html", "log-in")}</div>'
    return cs_edit(top=rf, dock_html=edit_dock(msg("circle-alert", "Not saved: sign in again first."), save="Save this code system", disabled="Sign in again first"))


def page_code_system_new(error=False):
    body = cs_editor_body(new=True, error=error)
    m = msg("circle-alert", "A code system needs a canonical, a status, and a content mode before it can be saved.") if error else ""
    d = edit_dock(m, save="Create this code system", back_href="code-systems.html", disabled="Name a canonical first" if error else None)
    return edit_shell("New code system", "systems", [("Code systems", "code-systems.html")], "Written through the FHIR API, like any client.", body, d)


def page_code_system_new_invalid():
    return page_code_system_new(error=True)


# ---- the value set composer ----

def compose_meta(new=False):
    return section("Metadata", '<div class="fui-form-grid">'
                   + field("vs-url", "Canonical URL", input_("vs-url", "" if new else "https://terminology.example.org/fhir/ValueSet/synthetic-respiratory", "https://terminology.example.org/fhir/ValueSet/…", extra='class="fui-input fui-mono"'), cls="fui-span-2")
                   + field("vs-ver", "Business version", input_("vs-ver", "" if new else "2"))
                   + field("vs-status", "Publication status", select("vs-status", STATUS_OPTS, "draft" if new else "active"))
                   + field("vs-title", "Title", input_("vs-title", "" if new else "Synthetic respiratory findings"), cls="fui-span-2")
                   + field("vs-inactive", "Inactive codes", select("vs-inactive", [("server", "Leave it to the server"), ("include", "Include them"), ("leave", "Leave them out")], "server"), cls="fui-span-2")
                   + "</div>")


def filter_row(prop="concept", op="is-a", value="SF-1200", invalid=False, error=None):
    inv = ' aria-invalid="true"' if invalid else ""
    err = f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    return (f'<div class="fui-clause-filter">{field("f-prop", "Property", select("f-prop", [("concept", "concept"), ("parent", "parent"), ("inactive", "inactive")], prop))}'
            f'{field("f-op", "Operator", select("f-op", [("is-a", "is-a"), ("descendent-of", "descendent-of"), ("is-not-a", "is-not-a"), ("=", "="), ("in", "in")], op))}'
            f'{field("f-val", "Value", input_("f-val", value, extra="class=" + chr(34) + "fui-input fui-mono" + chr(34) + inv))}'
            f'{icon_button("x", "Remove this filter")}</div>{err}')


def compose_clauses(invalid=False, filter_error=None):
    src = field("c1-sys", "Code system", select("c1-sys", [("sf", "Synthetic clinical findings"), ("care", "Synthetic care settings")], "sf"))
    ver = field("c1-ver", "Version", select("c1-ver", [("3", "3 (default)")], "3"))
    fbody = (f'<div class="fui-clause-grid">{src}{ver}</div><p class="fui-clause-sub">Filters</p>'
             + filter_row(value="SF-12OO" if filter_error else "SF-1200", invalid=bool(filter_error), error=filter_error)
             + f'<div class="fui-add-row">{button("ghost", "Add filter", "plus", small=True)}</div>')
    issue = None
    if invalid:
        fbody += ('<p class="fui-clause-sub">Codes</p><ul class="fui-clause-codes">'
                  f'<li>{coding("Fever", "SF-1310", inline=True)}{icon_button("x", "Remove SF-1310")}</li></ul>')
        issue = ("vsd-3", "This clause lists codes and filters at once. Split it into two clauses: they union.")
    rm = icon_button("trash-2", "Remove this clause")
    inc = clause("include", "Synthetic clinical findings, version 3", "Includes <code>SF-1200</code> Finding of the chest and everything below it.", fbody, rm, issue)
    ebody = (f'<div class="fui-clause-grid">{field("c2-sys", "Code system", select("c2-sys", [("sf", "Synthetic clinical findings")], "sf"))}{field("c2-ver", "Version", select("c2-ver", [("3", "3 (default)")], "3"))}</div>'
             '<p class="fui-clause-sub">Codes</p><ul class="fui-clause-codes">'
             f'<li>{coding("Wheeze", "SF-1220", inline=True)}{icon_button("x", "Remove SF-1220")}</li></ul>'
             f'<div class="fui-add-row">{button("ghost", "Add code", "search", small=True)}</div>')
    exc = clause("exclude", "Synthetic clinical findings, version 3", "Excludes 1 code: Wheeze.", ebody, icon_button("trash-2", "Remove this clause"))
    adds = f'<div class="fui-cluster fui-add-row">{button("secondary", "Add an include", "plus", small=True)}{button("secondary", "Add an exclude", "plus", small=True)}{button("ghost", "Add a published value set", "layers", small=True)}</div>'
    return section("Includes and excludes", '<p class="fui-small fui-muted fui-section-intro">Includes add codes; excludes take them back out.</p>'
                   + f'<div class="fui-clauses">{inc}{exc}</div>' + adds, count="2 clauses")


def preview_card(state="idle"):
    run = busy_button("secondary", "Expanding", small=True) if state == "running" else button("secondary", "Run the preview", "play", small=True, keys="Ctrl Enter")
    head = f'<div class="fui-panel-head"><h2 class="fui-card-title"><span>The preview</span></h2><div class="fui-actions">{run}</div></div>'
    if state == "idle":
        body = quiet("Run the preview to see what this selects.")
    elif state == "running":
        body = '<div class="fui-panel-body"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Expanding the definition</span></div>'
    elif state == "refused":
        body = ('<div class="fui-panel-body">' + refusal("The server refused the preview.", request="POST /r4/ValueSet/$expand",
                issues=[("bad", "error", "invalid", "The filter value <code>SF-12<mark>O</mark>O</code> is not a code of Synthetic clinical findings, at character 6.")]) + "</div>")
    else:
        rows = [("SF-1200", "Finding of the chest"), ("SF-1210", "Cough"), ("SF-1211", "Dry cough"), ("SF-1212", "Productive cough")]
        t = table("The preview", th("Concept"), [f'<tr><th scope="row" data-role="title">{coding(d, c, inline=True)}</th></tr>' for c, d in rows])
        body = (f'<div class="fui-panel-body"><p class="fui-small" role="status">This definition selects 4 concepts.</p>'
                + field("pv-filter", "Filter the preview", input_("pv-filter", "", "Text in a display")) + "</div>"
                + f'{t}')
    return f'<section class="fui-card fui-panel" aria-label="The preview">{head}{body}</section>'


VS_CRUMBS = [("Value sets", "value-sets.html"), ("Synthetic respiratory findings", "value-set.html")]
VS_SUB = f'Version 2, opened {when("2026-10-21T09:38")}.'


def composer(preview="idle", invalid=False, filter_error=None, dock_html=None, top="", visitor=False, new=False, toast_html=""):
    form = sections(compose_meta(new), compose_clauses(invalid, filter_error), card=True)
    body = top + f'<div class="fui-editor-split">{form}{preview_card(preview)}</div>'
    if dock_html is None:
        if visitor:
            dock_html = edit_dock(msg("info", "Sign in to save. Composing and previewing work without one."), back_href="value-set.html", visitor=True)
        elif invalid:
            dock_html = edit_dock(msg("circle-x", "The include breaks vsd-3: fix it to save."), save="Save this value set", back_href="value-set.html", disabled="The include breaks vsd-3")
        else:
            dock_html = edit_dock(msg("pencil", "1 change: the exclude takes SF-1220 Wheeze back out."), save="Save this value set", back_href="value-set.html")
    if new:
        return edit_shell("New value set", "valuesets", [("Value sets", "value-sets.html")], "Build a value set and preview what it selects.", body, dock_html, toast_html, visitor=visitor)
    return edit_shell("Edit", "valuesets", VS_CRUMBS, VS_SUB, body, dock_html, toast_html, visitor=visitor)


def page_value_set_edit():
    return composer()


def page_value_set_preview():
    return composer("answered")


def page_value_set_preview_running():
    return composer("running")


def page_value_set_filter_refused():
    return composer("refused", filter_error="SF-12OO is not a code of this code system: the server points at character 6, a letter O where a zero belongs.")


def page_value_set_invalid():
    return composer(invalid=True)


def page_value_set_visitor():
    return composer("answered", visitor=True)


def page_value_set_new():
    return composer(new=True, dock_html=edit_dock(msg("circle-alert", "A canonical and a clause that selects something are what a save needs."), save="Create this value set", back_href="value-sets.html", disabled="Name a canonical and a clause first"))


def page_value_set_saved():
    top = load_report("ok", f'Saved {when("2026-10-21T09:45")}. The server now holds version 3.', actions=link_button("secondary", "Open the value set", "value-set.html", "arrow-right", small=True))
    return composer("answered", top=top, dock_html="")


# ---- the concept map editor ----

def cm_meta():
    return section("Metadata", '<div class="fui-form-grid">'
                   + field("cm-url", "Canonical URL", input_("cm-url", "https://terminology.example.org/fhir/ConceptMap/legacy-findings", extra='class="fui-input fui-mono"'), cls="fui-span-2")
                   + field("cm-ver", "Business version", input_("cm-ver", "2"))
                   + field("cm-status", "Publication status", select("cm-status", STATUS_OPTS, "active"))
                   + "</div>")


REL_OPTS = [("", "Choose one"), ("equivalent", "Equivalent"), ("source-is-broader-than-target", "Broader than the target"),
            ("source-is-narrower-than-target", "Narrower than the target"), ("related-to", "Related to"), ("not-related-to", "Not related to")]


def element_block(src, sd, rel, tgt, tdp, cm, invalid=False):
    err = None
    if invalid:
        err = f"The target <code>{tgt}</code> of <code>{src}</code> states no relationship to its source, which every served version makes mandatory."
    if rel is None and tgt is None:
        targets = check(f"nomap-{src}", "This code maps to nothing (noMap)", True)
    else:
        targets = ('<div class="fui-clause-filter">'
                   + field(f"t-{src}", "Target code", input_(f"t-{src}", tgt, extra='class="fui-input fui-mono"'))
                   + field(f"r-{src}", "Relationship", select(f"r-{src}", REL_OPTS, "" if invalid else rel, extra=' aria-invalid="true"' if invalid else ""), error=err)
                   + field(f"c-{src}", "Comment", input_(f"c-{src}", cm or "", "Optional"))
                   + icon_button("x", f"Remove this target of {src}") + "</div>")
    acts = button("ghost", "Preview through $translate", "corner-down-right", small=True) + icon_button("trash-2", f"Remove the code {src}")
    return (f'<li class="fui-map-element"><div class="fui-map-element-head">{coding(sd, src, inline=True)}<div class="fui-actions">{acts}</div></div>{targets}</li>')


def cm_editor(invalid=False, preview=None, top="", dock_html=None):
    rows = LEGACY_MAPPINGS[:4] + LEGACY_MAPPINGS[6:]
    els = "".join(element_block(*r, invalid=(invalid and r[0] == "LF-04")) for r in rows)
    grp = (f'<div class="fui-clause-grid">{field("g-src", "Source system", select("g-src", [("legacy", "Legacy findings")], "legacy"))}'
           f'{field("g-tgt", "Target system", select("g-tgt", [("sf", "Synthetic clinical findings")], "sf"))}</div>'
           f'<p class="fui-clause-sub">Codes · 5 of 7 shown</p><ul class="fui-map-elements">{els}</ul>'
           f'<div class="fui-add-row">{button("ghost", "Add a code", "plus", small=True)}</div>')
    group = section("Group 1: Legacy findings to Synthetic clinical findings", grp, actions=icon_button("trash-2", "Remove this group"))
    adds = section("", f'<div class="fui-cluster">{button("secondary", "Add a group", "plus", small=True)}</div><p class="fui-small fui-muted">A group maps codes of one system to codes of another, and carries at least one code.</p>')
    body = top + sections(cm_meta(), group, adds, card=True)
    if preview:
        body += preview
    if dock_html is None:
        dock_html = (edit_dock(msg("circle-x", "LF-04 has a target with no relationship: fix it to save."), save="Save this concept map", back_href="concept-map.html", disabled="LF-04 has a target with no relationship")
                     if invalid else edit_dock(msg("pencil", "1 change: LF-04 now maps to SF-1220 Wheeze."), save="Save this concept map", back_href="concept-map.html"))
    return edit_shell("Edit", "conceptmaps", [("Concept maps", "concept-maps.html"), ("Legacy findings to synthetic findings", "concept-map.html")],
                      f'Version 2, opened {when("2026-10-21T09:39")}.', body, dock_html)


def page_concept_map_edit():
    return cm_editor()


def page_concept_map_invalid():
    return cm_editor(invalid=True)


def page_concept_map_preview():
    r = load_report("ok", "Translated <code>LF-04</code> Chest complaint through the map on this screen: 1 match.", glyph="corner-down-right")
    t = table("The matches", th("Target") + th("Relationship"), [f'<tr><th scope="row" data-role="title">{coding("Wheeze", "SF-1220", "Synthetic clinical findings")}</th>{td("Relationship", relation("broader than the target"), role="sub")}</tr>'])
    pv = f'<section class="fui-card fui-panel" aria-label="The preview"><div class="fui-panel-head"><h2 class="fui-card-title"><span>The preview</span></h2></div><div class="fui-panel-body">{r}</div>{t}</section>'
    return cm_editor(preview=pv)


def page_concept_map_new():
    grp = section("Groups", quiet("No group yet.") + f'<div class="fui-add-row">{button("secondary", "Add a group", "plus", small=True)}</div>')
    meta = section("Metadata", '<div class="fui-form-grid">' + field("cm-url", "Canonical URL", input_("cm-url", "", "https://terminology.example.org/fhir/ConceptMap/…", extra='class="fui-input fui-mono"'), cls="fui-span-2")
                   + field("cm-ver", "Business version", input_("cm-ver", "")) + field("cm-status", "Publication status", select("cm-status", STATUS_OPTS, "draft")) + "</div>")
    d = edit_dock(msg("circle-alert", "A concept map needs a canonical and a group with at least one code before it can be saved."), save="Create this concept map", back_href="concept-maps.html", disabled="Name a canonical and add a group first")
    return edit_shell("New concept map", "conceptmaps", [("Concept maps", "concept-maps.html")], "Map the codes of one system to another.", sections(meta, grp, card=True), d)
