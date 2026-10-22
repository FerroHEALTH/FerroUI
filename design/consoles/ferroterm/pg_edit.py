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


def edit_dock(msg="", state="ready", save="Save", back_href="code-system.html", disabled=None, visitor=False, save_variant="primary", discard=True):
    m = f'<p class="fui-dock-message">{msg}</p>' if msg else '<span></span>'
    d = link_button("secondary", "Discard", back_href, "x") if discard else ""
    if visitor:
        acts = d + link_button("primary", "Sign in to save", "sign-in.html", "log-in")
    elif state == "saving":
        acts = button("secondary", "Discard", attrs="disabled") + busy_button("primary", "Saving")
    elif disabled:
        acts = d + f'<button type="button" class="fui-button fui-button-{save_variant}" disabled title="{e(disabled)}">{icon("save", "16")}{save}</button>'
    else:
        acts = d + button("primary", save, "save", keys="Ctrl S")
    return dock(m, acts, "")


def msg(glyph, text):
    return f'{icon(glyph, "16")}<span>{text}</span>'


def edit_shell(title, current, crumbs, sub, body, dock_html, toast_html="", overlay=None, visitor=False):
    """An editor: the trail ends with the resource, the title says Edit or New."""
    h = page_heading(title, subtitle=sub, crumbs=crumbs, here=False)
    doc = f"{title} · {crumbs[-1][0]}" if title in ("Edit", "Saved") else title
    return shell(doc, current, h + f'<div class="fui-stack">{body}</div>', dock_html=dock_html, toast_html=toast_html, overlay=overlay, visitor=visitor)


def read_only(html):
    """The form while a save is in flight: every field read-only, every
    select and button held."""
    return (html.replace("<input ", "<input readonly ").replace("<select ", "<select disabled ")
            .replace('<button type="button" class="fui-button fui-button-ghost', '<button type="button" disabled class="fui-button fui-button-ghost')
            .replace('<button type="button" class="fui-icon-button"', '<button type="button" disabled class="fui-icon-button"'))


# ---- the code system editor ----

def cs_meta(new=False, error=False):
    url = "" if new else SF
    err = "FerroTERM needs a canonical URL to create a code system." if error else None
    return section("Metadata", '<div class="fui-form-grid">'
                   + field("cs-url", "Canonical URL", input_("cs-url", url, "https://…/CodeSystem/…", extra='class="fui-input fui-mono"' + (' aria-invalid="true"' if error else "")), error=err, cls="fui-span-row")
                   + field("cs-title", "Title", input_("cs-title", "" if new else "Synthetic clinical findings"), cls="fui-span-2")
                   + field("cs-name", "Name", input_("cs-name", "" if new else "SyntheticClinicalFindings", extra='class="fui-input fui-mono"'), cls="fui-span-2")
                   + field("cs-ver", "Business version", input_("cs-ver", "" if new else "3"))
                   + field("cs-status", "Publication status", select("cs-status", STATUS_OPTS, "draft" if new else "active"))
                   + field("cs-content", "Content mode", select("cs-content", [("complete", "All concepts"), ("fragment", "Some concepts"), ("example", "Examples only")], "complete"))
                   + '</div><div class="fui-add-row">' + check("cs-case", "Codes are case sensitive", True) + "</div>")


RETIRED = "SF-1320"


def concept_rows(rows, retire=None):
    out = []
    for code, display, parent, inactive, nl in rows:
        life = select(f"life-{code}", [("active", "Active"), ("deprecated", "Deprecated"), ("retired", "Retired")], "retired" if (inactive or code == retire) else "active", cls="fui-select fui-select-small")
        out.append(f'<tr><th scope="row" data-role="title"><code>{e(code)}</code></th>'
                   + td("Display", input_(f"d-{code}", display, extra=f'aria-label="Display of {e(code)}"'), role="sub")
                   + td("Parent", f"<code>{e(parent)}</code>" if parent else '<span class="mk-dash">top level</span>', role="meta-q")
                   + td("Lifecycle", life, role="meta-2") + "</tr>")
    return table("Concepts", th("Code") + th("Display") + th("Parent") + th("Lifecycle"), out)


def prop_row(n, code, typ, locked=False):
    lock = ' readonly title="The editor declares this property on every save"' if locked else ""
    rm = f'<button type="button" class="fui-icon-button" disabled aria-label="The editor keeps the property {code}" title="The editor keeps the property {code}">{icon("lock", "16")}</button>' if locked else icon_button("x", f"Remove the property {code}")
    return (f'<div class="fui-repeater-row">{field(f"p-{n}", "Property code", input_(f"p-{n}", code, extra="class=" + chr(34) + "fui-input fui-mono" + chr(34) + lock))}'
            f'{field(f"p-{n}t", "Type", select(f"p-{n}t", [("code", "code"), ("string", "string"), ("boolean", "boolean")], typ, extra="disabled" if locked else ""))}{rm}</div>')


def cs_editor_body(new=False, error=False, retire=RETIRED):
    if new:
        pbody = quiet("No properties yet.")
    else:
        pbody = f'<div class="fui-repeater">{prop_row(1, "parent", "code")}{prop_row(2, "inactive", "boolean", locked=True)}</div>'
    props = section("Declared properties", pbody + f'<div class="fui-add-row">{button("ghost", "Declare a property", "plus", small=True)}</div>', count=None if new else 2)
    if new:
        cbody = quiet("No concepts yet.")
    else:
        cbody = ('<p class="fui-small fui-muted fui-section-intro">A concept is retired, never removed, so a published code keeps meaning what it meant.</p>'
                 + f'{concept_rows(FINDINGS[:1] + FINDINGS[9:13], retire)}'
                 + f'<div class="fui-add-row"><span class="fui-small fui-muted">5 of 14 concepts shown, the general findings.</span>{button("ghost", "Show all 14", "chevron-down", small=True)}</div>')
    concepts = section("Concepts", cbody + f'<div class="fui-add-row">{button("ghost", "Add a concept", "plus", small=True)}</div>', count=None if new else 14)
    return sections(cs_meta(new, error), props, concepts, card=True)


CS_CRUMBS = [("Code systems", "code-systems.html"), ("Synthetic clinical findings", "code-system.html")]
CS_SUB = f'Version 3, opened {when("2026-10-21T09:36")}.'
CS_CHANGE = msg("pencil", "1 change: SF-1320 Fatigue is retired.")


def cs_edit(body=None, dock_html=None, top="", toast_html="", title="Edit", sub=CS_SUB, visitor=False):
    return edit_shell(title, "systems", CS_CRUMBS, sub, top + (body or cs_editor_body()),
                      dock_html if dock_html is not None else edit_dock(CS_CHANGE, save="Save this code system"), toast_html, visitor=visitor)


def page_code_system_edit():
    return cs_edit()


def page_code_system_saving():
    return cs_edit(body=read_only(cs_editor_body()), dock_html=edit_dock(CS_CHANGE, "saving", "Save this code system"))


def page_code_system_saved():
    top = load_report("ok", f'Saved {when("2026-10-21T09:41")}. The server now holds version 4.', actions=link_button("secondary", "Open the code system", "code-system.html", "arrow-right", small=True))
    check_ = section("What Look up returns for the retired concept",
                     f'<p>{coding("Fatigue", "SF-1320", inactive=True, inline=True)}</p>'
                     '<p class="fui-small fui-muted">From CodeSystem/$lookup on version 4: the property <code>inactive</code> is <code>true</code>.</p>')
    return cs_edit(body=sections(check_, card=True), top=top, dock_html="", title="Saved", sub=f'Version 4, saved {when("2026-10-21T09:41")}.')


def page_code_system_conflict():
    rf = refusal("Someone changed this code system since you opened it, so the save was refused.", request="PUT /r4/CodeSystem/synthetic-findings · If-Match: W/\"3\"",
                 issues=[("bad", "Error", "conflict", "The server holds version 4 now. Reload it and apply the change again; your edits stay in the form until you do.")])
    rf += f'<div class="fui-cluster">{button("primary", "Reload the current version", "refresh-cw")}</div>'
    return cs_edit(top=rf, dock_html=edit_dock(msg("circle-x", "Not saved. Reload the current version first."), save="Save this code system", disabled="Reload the current version first", save_variant="secondary"))


def page_code_system_refused():
    rf = refusal("The server refused the change: this account may not edit code systems.", request="PUT /r4/CodeSystem/synthetic-findings",
                 issues=[("bad", "Error", "forbidden", "The access token carries no user/CodeSystem.cud scope.")])
    rf += (f'<div class="fui-cluster">{link_button("primary", "Back to the code system", "code-system.html", "arrow-left")}'
           f'{link_button("secondary", "Sign in with another account", "sign-in.html", "log-in")}</div>')
    return cs_edit(top=rf, dock_html=edit_dock(msg("circle-x", "Not saved: this account cannot edit code systems."), save="Save this code system", disabled="This account cannot edit code systems", save_variant="secondary"))


def page_code_system_expired():
    rf = notice("warn", "triangle-alert", "Your session ended, so the save was not sent. Sign in again; the form keeps your edits in this page while you do.")
    rf += f'<div class="fui-cluster">{link_button("primary", "Sign in again", "sign-in.html", "log-in")}</div>'
    return cs_edit(top=rf, visitor=True, dock_html=edit_dock(msg("triangle-alert", "Not saved: sign in again first."), save="Save this code system", disabled="Sign in again first", save_variant="secondary"))


def page_code_system_new(error=False):
    body = cs_editor_body(new=True, error=error)
    d = edit_dock("", save="Create this code system", back_href="code-systems.html", disabled="Add a canonical URL first" if error else None)
    return edit_shell("New code system", "systems", [("Code systems", "code-systems.html")], "Written through the FHIR API, like any client.", body, d)


def page_code_system_new_invalid():
    return page_code_system_new(error=True)


# ---- the value set composer ----

def compose_meta(new=False):
    return section("Metadata", '<div class="fui-form-grid">'
                   + field("vs-url", "Canonical URL", input_("vs-url", "" if new else "https://terminology.example.org/fhir/ValueSet/synthetic-respiratory", "https://…/ValueSet/…", extra='class="fui-input fui-mono"'), cls="fui-span-row")
                   + field("vs-title", "Title", input_("vs-title", "" if new else "Synthetic respiratory findings"), cls="fui-span-2")
                   + field("vs-inactive", "Inactive codes", select("vs-inactive", [("server", "Leave it to the server"), ("include", "Include them"), ("leave", "Leave them out")], "server"), cls="fui-span-2")
                   + field("vs-ver", "Business version", input_("vs-ver", "" if new else "2"))
                   + field("vs-status", "Publication status", select("vs-status", STATUS_OPTS, "draft" if new else "active"))
                   + "</div>")


PROP_OPTS = [("concept", "Concept"), ("parent", "Parent"), ("inactive", "Inactive")]
OP_OPTS = [("is-a", "Is a (is-a)"), ("descendent-of", "Below (descendent-of)"), ("is-not-a", "Is not a (is-not-a)"), ("=", "Equals (=)"), ("in", "One of (in)")]


def filter_row(prop="concept", op="is-a", value="SF-1200", invalid=False, error=None):
    inv = ' aria-invalid="true"' if invalid else ""
    err = f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    return (f'<div class="fui-clause-filter">{field("f-prop", "Property", select("f-prop", PROP_OPTS, prop))}'
            f'{field("f-op", "Operator", select("f-op", OP_OPTS, op))}'
            f'{field("f-val", "Value", input_("f-val", value, extra="class=" + chr(34) + "fui-input fui-mono" + chr(34) + inv))}'
            f'{icon_button("x", "Remove this filter")}</div>{err}')


def add(label):
    return button("ghost", label, "plus", small=True)


def compose_clauses(invalid=False, filter_error=None, new=False):
    adds = f'<div class="fui-cluster fui-add-row">{add("Add an include")}{add("Add an exclude")}{add("Add a published value set")}</div>'
    intro = '<p class="fui-small fui-muted fui-section-intro">Includes add codes; excludes take them back out.</p>'
    if new:
        return section("Includes and excludes", intro + quiet("No clauses yet.") + adds)
    src = field("c1-sys", "Code system", select("c1-sys", [("sf", "Synthetic clinical findings"), ("care", "Synthetic care settings")], "sf"))
    ver = field("c1-ver", "Version", select("c1-ver", [("3", "3 (default)")], "3"))
    fbody = (f'<div class="fui-clause-grid">{src}{ver}</div><p class="fui-clause-sub">Filters</p>'
             + filter_row(value="SF-12OO" if filter_error else "SF-1200", invalid=bool(filter_error), error=filter_error)
             + f'<div class="fui-add-row">{add("Add filter")}</div>')
    issue = None
    if invalid:
        fbody += ('<p class="fui-clause-sub">Codes</p><ul class="fui-clause-codes">'
                  f'<li>{coding("Fever", "SF-1310", inline=True)}{icon_button("x", "Remove SF-1310")}</li></ul>'
                  f'<div class="fui-add-row">{add("Add code")}</div>')
        issue = ("vsd-3", "This clause lists codes and filters at once. Split it into two clauses; the two clauses add together.")
    rm = icon_button("trash-2", "Remove this clause")
    inc = clause("include", "Synthetic clinical findings, version 3", "Includes <code>SF-1200</code> Finding of the chest and everything below it.", fbody, rm, issue)
    ebody = (f'<div class="fui-clause-grid">{field("c2-sys", "Code system", select("c2-sys", [("sf", "Synthetic clinical findings")], "sf"))}{field("c2-ver", "Version", select("c2-ver", [("3", "3 (default)")], "3"))}</div>'
             '<p class="fui-clause-sub">Codes</p><ul class="fui-clause-codes">'
             f'<li>{coding("Wheeze", "SF-1220", inline=True)}{icon_button("x", "Remove SF-1220")}</li></ul>'
             f'<div class="fui-add-row">{add("Add code")}</div>')
    exc = clause("exclude", "Synthetic clinical findings, version 3", f'Excludes 1 code: {coding("Wheeze", "SF-1220", inline=True)}', ebody, icon_button("trash-2", "Remove this clause"))
    return section("Includes and excludes", intro + f'<div class="fui-clauses">{inc}{exc}</div>' + adds, count="2 clauses")


def preview_card(state="idle"):
    run = busy_button("secondary", "Running the preview", small=True) if state == "running" else button("secondary", "Run the preview", "play", small=True, keys="Ctrl Enter")
    head = f'<div class="fui-panel-head"><h2 class="fui-card-title"><span>The preview</span></h2><div class="fui-actions">{run}</div></div>'
    if state == "idle":
        body = quiet("Run the preview to see what this selects.")
    elif state == "running":
        body = '<div class="fui-panel-body"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Expanding the definition</span></div>'
    elif state == "refused":
        body = quiet(f'{icon("circle-x", "14")} The server refused the preview: see the filter value in the include.')
    else:
        rows = [("SF-1200", "Finding of the chest"), ("SF-1210", "Cough"), ("SF-1211", "Dry cough"), ("SF-1212", "Productive cough")]
        t = table("The preview", th("Concept"), [f'<tr><th scope="row" data-role="title">{coding(d, c, inline=True)}</th></tr>' for c, d in rows])
        body = (f'<div class="fui-panel-body fui-stack"><p class="fui-small" role="status">This definition selects 4 concepts.</p>'
                + field("pv-filter", "Filter the preview", input_("pv-filter", "", "Text in a display")) + "</div>"
                + f'{t}')
    return f'<section class="fui-card fui-panel" aria-label="The preview">{head}{body}</section>'


VS_CRUMBS = [("Value sets", "value-sets.html"), ("Synthetic respiratory findings", "value-set.html")]
VS_SUB = f'Version 2, opened {when("2026-10-21T09:38")}.'
VS_CLEAN = edit_dock(msg("info", "No changes yet."), save="Save this value set", back_href="value-set.html", disabled="Nothing to save yet")


def composer(preview="idle", invalid=False, filter_error=None, dock_html=None, top="", visitor=False, new=False, toast_html=""):
    form = sections(compose_meta(new), compose_clauses(invalid, filter_error, new), card=True)
    body = top + f'<div class="fui-editor-split">{form}{preview_card(preview)}</div>'
    if dock_html is None:
        if visitor:
            dock_html = edit_dock(msg("info", "Sign in to save. You can build and preview without signing in."), back_href="value-set.html", visitor=True)
        elif invalid:
            dock_html = edit_dock(msg("circle-x", "1 change: the include also lists SF-1310 Fever, so it mixes codes and filters. Split it to save."), save="Save this value set",
                                  back_href="value-set.html", disabled="The include mixes codes and filters")
        elif filter_error:
            dock_html = edit_dock(msg("circle-x", "1 change: the include's filter value is now SF-12OO, which the server refused."), save="Save this value set",
                                  back_href="value-set.html", disabled="Fix the filter value first")
        else:
            dock_html = VS_CLEAN
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
    return composer(new=True, dock_html=edit_dock(msg("circle-alert", "Add a canonical URL and a clause to create it."), save="Create this value set", back_href="value-sets.html", disabled="Add a canonical URL and a clause first"))


def page_value_set_saved():
    import pg_res
    top = load_report("ok", f'Saved {when("2026-10-21T09:45")}. The server now holds version 3.')
    return pg_res.vs_page(header=pg_res.vs_header(line="Version 3 · stored on this server"), panel=pg_res.vs_definition(top=f'<div class="fui-stack">{top}</div>', changed="2026-10-21T09:45"))


# ---- the concept map editor ----

def cm_meta(new=False):
    return section("Metadata", '<div class="fui-form-grid">'
                   + field("cm-url", "Canonical URL", input_("cm-url", "" if new else "https://terminology.example.org/fhir/ConceptMap/legacy-findings", "https://…/ConceptMap/…", extra='class="fui-input fui-mono"'), cls="fui-span-row")
                   + field("cm-title", "Title", input_("cm-title", "" if new else "Legacy findings to synthetic findings"), cls="fui-span-2")
                   + field("cm-name", "Name", input_("cm-name", "" if new else "LegacyFindingsToSyntheticFindings", extra='class="fui-input fui-mono"'), cls="fui-span-2")
                   + field("cm-ver", "Business version", input_("cm-ver", "" if new else "2"))
                   + field("cm-status", "Publication status", select("cm-status", STATUS_OPTS, "draft" if new else "active"))
                   + "</div>")


# The R4 ConceptMap equivalence codes, read as the target's relation to the source.
REL_OPTS = [("", "Choose one")] + [(k, (v[:1].upper() + v[1:]) + (" (unmatched)" if k == "unmatched" else "")) for k, v in REL_WORD.items()]


def element_block(src, sd, rel, tgt, tdp, cm, invalid=False, preview=""):
    err = ""
    if invalid:
        err = f'<p class="fui-help fui-error">{icon("circle-alert", "14")} Choose a relationship for {e(tgt)}.</p>'
    inv = ' aria-invalid="true"' if invalid else ""
    targets = ('<div class="fui-clause-filter">'
               + field(f"t-{src}", "Target code", input_(f"t-{src}", tgt or "", "No target" if rel == "unmatched" else "", extra='class="fui-input fui-mono"'))
               + field(f"r-{src}", "Relationship", select(f"r-{src}", REL_OPTS, "" if invalid else rel, extra=inv))
               + field(f"c-{src}", "Comment", input_(f"c-{src}", cm or "", "Optional"))
               + icon_button("x", f"Remove this target of {src}") + f"</div>{err}")
    acts = icon_button("corner-down-right", f"Preview {src} through Translate") + icon_button("trash-2", f"Remove the code {src}")
    return (f'<li class="fui-map-element"><div class="fui-map-element-head">{coding(sd, src, inline=True)}<div class="fui-actions">{acts}</div></div>{targets}{preview}</li>')


def cm_editor(invalid=False, preview="", top="", dock_html=None):
    rows = LEGACY_MAPPINGS[:4] + LEGACY_MAPPINGS[6:]
    els = "".join(element_block(*r, invalid=(invalid and r[0] == "LF-04"), preview=(preview if r[0] == "LF-04" else "")) for r in rows)
    grp = (f'<div class="fui-clause-grid">{field("g-src", "Source system", select("g-src", [("legacy", "Legacy findings")], "legacy"))}'
           f'{field("g-tgt", "Target system", select("g-tgt", [("sf", "Synthetic clinical findings")], "sf"))}</div>'
           f'<p class="fui-clause-sub">Codes · 5 of 7 shown</p><ul class="fui-map-elements">{els}</ul>'
           f'<div class="fui-add-row">{button("ghost", "Show all 7", "chevron-down", small=True)}{button("ghost", "Add a code", "plus", small=True)}</div>')
    group = section("Group 1: Legacy findings to Synthetic clinical findings", grp, actions=icon_button("trash-2", "Remove this group"))
    adds = section("", f'<div class="fui-cluster">{button("ghost", "Add a group", "plus", small=True)}</div><p class="fui-small fui-muted">A group maps codes of one system to codes of another, and carries at least one code.</p>')
    body = top + sections(cm_meta(), group, adds, card=True)
    if dock_html is None:
        dock_html = (edit_dock(msg("circle-x", "LF-04 has a target with no relationship: choose one to save."), save="Save this concept map", back_href="concept-map.html", disabled="LF-04 has a target with no relationship")
                     if invalid else edit_dock(msg("pencil", "1 change: LF-04 now maps to SF-1220 Wheeze."), save="Save this concept map", back_href="concept-map.html"))
    return edit_shell("Edit", "conceptmaps", [("Concept maps", "concept-maps.html"), ("Legacy findings to synthetic findings", "concept-map.html")],
                      f'Version 2, opened {when("2026-10-21T09:39")}.', body, dock_html)


def page_concept_map_edit():
    return cm_editor()


def page_concept_map_invalid():
    return cm_editor(invalid=True)


def page_concept_map_preview():
    r = load_report("ok", "Translated <code>LF-04</code> Chest complaint through the map on this screen: 1 match.", glyph="corner-down-right")
    t = table("The matches", th("Target") + th("Relationship"), [f'<tr><th scope="row" data-role="title">{coding("Wheeze", "SF-1220", "Synthetic clinical findings")}</th>{td("Relationship", relation(REL_WORD["narrower"]), role="sub")}</tr>'])
    pv = f'<div class="fui-map-preview" role="region" aria-label="The preview of LF-04">{r}{t}</div>'
    return cm_editor(preview=pv)


def page_concept_map_new():
    grp = section("Groups", quiet("No groups yet.") + f'<div class="fui-add-row">{button("ghost", "Add a group", "plus", small=True)}</div>')
    d = edit_dock(msg("circle-alert", "A concept map needs a canonical URL and a group with at least one code before it can be saved."), save="Create this concept map", back_href="concept-maps.html", disabled="Add a canonical URL and a group first")
    return edit_shell("New concept map", "conceptmaps", [("Concept maps", "concept-maps.html")], "Map the codes of one system to another.", sections(cm_meta(new=True), grp, card=True), d)
