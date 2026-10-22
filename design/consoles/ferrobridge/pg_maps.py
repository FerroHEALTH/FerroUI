# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The Mapping set (T1 with binder tabs) and its objects: a mapping file
(T2, the one home of a file and of the context it defines) and a template
(T2 without tabs). The context row opens its file's Program tab."""
import tpl
from common import (
    MENU_SEP, action_group, badge, button, card, definitions, dock, dock_field, e, empty_quiet, empty_state, facts_row, field, icon,
    link_button, menu, menu_item, notice, page_heading, pager, pill, refusal, select, shell, table, tabbed, tabs, td, textarea, th, toast, toasts, when,
)
from bridge import (wb, quiet, section, sections, RUNNING_COUNTS, codeview, diags_list, file_href, lang_tag, load_report, locked, owner_tag, plan_badge, planned, short, state)
from data import (CONTEXTS, CONTEXT_YAML, DIAGNOSTICS, DRAFT_ERRORS, EXTENSION_FILES, MODEL_FILES, MODEL_YAML, OMOCL, PROFILE_BASE, SEARCH_FILES, TEMPLATES)
from parts import skeleton_lines
from pg_shell import set_menu_items, tech_detail, unreachable_block

SUB = "What the bridge runs, and the files it runs from."


def set_heading(more_open=False, hover=None):
    ag = action_group(link_button("primary", "New file", "edit-new.html", "file-plus"), set_menu_items(hover), open_=more_open, label="More actions for the mapping set")
    return page_heading("Mapping set", actions=ag, subtitle=SUB)


def running_report(tone="neutral", title=None, counts=RUNNING_COUNTS, actions=""):
    t = title or f'The running set compiles · loaded at the start, {when("2026-10-21T06:00")}'
    return load_report(tone if tone != "neutral" else "ok", t, counts, actions)


SET_TABS = [("contexts", "Contexts", "mappings.html", 9), ("omocl", "OMOCL", "mappings-omocl.html", 4), ("templates", "Templates", "mappings-templates.html", 7),
            ("files", "Files", "mappings-files.html", 27), ("diagnostics", "Diagnostics", "mappings-diagnostics.html", 3)]


def row_menu(name, open_=False, items=None):
    m = menu(items or [], label=f"Actions for {name}", sheet=True) if open_ else ""
    return (f'<span class="fui-anchor"><button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="Actions for {e(name)}" title="More actions">'
            f'{icon("ellipsis", "16")}</button>{m}</span>')


def ctx_name(path):
    return short(path).replace(".context.yml", "")


def ctx_state(warnings):
    return state("ok", "compiles") if not warnings else state("warn", f"{warnings} warning{'s' if warnings != 1 else ''}")


def ctx_items(console):
    items = [menu_item("Open its program", "layers", href="file-program.html"), menu_item("Try in the Playground", "flask-conical", href="playground.html")]
    items += [menu_item("Edit", "pencil", href="edit.html")] if console else [menu_item("Check a draft", "list-checks", href="edit-provisioned.html")]
    return items


def ctx_rows(hover=None, menu_open=None):
    rows = []
    for i, (key, path, rt, prof, tid, models, search, owner, warn) in enumerate(CONTEXTS):
        href = "file-program.html" if key == "body_weight" else "file-provisioned.html"
        cls = ' class="mk-hover"' if i == hover else ""
        rows.append(f'<tr{cls}><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{e(ctx_name(path))}</a></th>'
                    + td("Resource type", e(rt), role="sub") + td("Template", f"<code>{e(tid)}</code>", role="wide")
                    + td("Models", f"{len(models)} model{'s' if len(models) != 1 else ''}", "fui-num", role="meta-q") + td("Search", "declared" if search else '<span class="mk-dash">–</span>', role="wide")
                    + td("Owner", owner_tag(owner), role="wide") + td("State", ctx_state(warn), role="meta-2")
                    + td("", row_menu(ctx_name(path), menu_open == i, ctx_items(owner == "console")), "fui-row-menu", role="menu") + "</tr>")
    return table("Contexts", th("Context", "n", "ascending") + th("Resource type") + th("Template") + th("Models", num=True) + th("Search") + th("Owner")
                 + th("State") + th('<span class="fui-sr-only">Actions</span>'), rows, scroll_label="Contexts")


def omocl_rows():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="file-omocl.html">{e(short(f))}</a></th>'
            + td("CDM table", f"<code>{t}</code>", role="sub") + td("Archetype", f'<code>{e(a)}</code>', role="wide")
            + td("Owner", owner_tag(o), role="wide") + td("State", state("ok", "compiles"), role="meta-2") + "</tr>" for f, t, a, o in OMOCL]
    return table("OMOCL files", th("File", "n", "ascending") + th("CDM table") + th("Archetype") + th("Owner") + th("State"), rows, scroll_label="OMOCL files")


def tpl_rows():
    rows = []
    for tid, src, f, arch in TEMPLATES:
        n = sum(1 for c in CONTEXTS if c[4] == tid)
        source = "the template directory" if src == "directory" else "the CDR"
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="template.html"><code>{e(tid)}</code></a></th>'
                    + td("Source", source, role="sub") + td("Contexts", str(n), "fui-num", role="meta-q", unit="contexts" if n != 1 else "context") + td("Archetypes", str(arch), "fui-num", role="meta-2", unit="archetypes") + "</tr>")
    return table("Templates", th("Template", "n", "ascending") + th("Source") + th("Contexts", num=True, title="The contexts that name it") + th("Archetypes", num=True), rows, scroll_label="Templates")


def all_files():
    out = []
    for c in CONTEXTS:
        out.append((c[1], "fhirconnect", "context", c[7], c[8]))
    for m in MODEL_FILES:
        w = 1 if m in ("lab_analyte", "body_weight") else 0
        out.append((f"fhirconnect/models/{m}.model.yml", "fhirconnect", "model", "console" if m in ("body_weight", "adverse_reaction") else "provisioned", w))
    for x, _ in EXTENSION_FILES:
        out.append((f"fhirconnect/extensions/{x}.model.yml", "fhirconnect", "extension", "provisioned", 0))
    for s_ in SEARCH_FILES:
        out.append((s_, "fhirconnect", "search", "provisioned", 0))
    for f, t, a, o in OMOCL:
        out.append((f, "omocl", "mapping", o, 0))
    for i, x in enumerate(out):
        if x[0].endswith("synthetic_body_weight.context.yml"):
            out[i] = (x[0], x[1], x[2], x[3], 1)
    return sorted(out)


def file_rows(items=None, limit=25):
    rows = []
    for path, lang, kind, owner, n in (items if items is not None else all_files())[:limit]:
        st = state("ok", "no diagnostic") if not n else state("warn", f"{n} warning{'s' if n != 1 else ''}")
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{file_href(path)}"><code>{wb(path)}</code></a></th>'
                    + td("Kind", kind, role="sub") + td("Owner", owner_tag(owner), role="wide")
                    + td("Diagnostics", st, role="meta-2") + "</tr>")
    # NOTE: the path's first folder already names the language, so the list carries no Language column.
    return table("Mapping files", th("Path", "p", "ascending") + th("Kind") + th("Owner") + th("Diagnostics"), rows, scroll_label="Mapping files")


LANG_ITEMS = [("FHIRconnect", None, 23, True), ("OMOCL", None, 4, True)]
OWNER_ITEMS = [("provisioned", None, 22, False), ("console", None, 5, True)]


def files_toolbar(value="", owner=None, open_=False):
    return tpl.toolbar("Path or file name", value, [tpl.filter_button("Language", None), tpl.filter_button("Owner", owner, open_, OWNER_ITEMS)])


def set_page(tab="contexts", body=None, tools="", chips="", toast_html="", report=None, more_open=False, overlay=None, foot="", incidents=0, status="up", loading=False):
    if loading:
        sk = '<span class="fui-tab-count fui-skeleton mk-w20" aria-hidden="true"></span>'
        t = tabs("Mapping set", [(k, w + sk, h) for k, w, h, _ in SET_TABS], tab)
    else:
        t = tabs("Mapping set", SET_TABS, tab)
    panel = body if body is not None else {"contexts": ctx_rows, "omocl": omocl_rows, "templates": tpl_rows}[tab]()
    # NOTE: the toolbar and its chips sit inside the tab panel, so the binder tabs stay put from tab to tab.
    lead = f'<div class="fui-panel-body fui-stack">{tools}{chips}</div>' if tools or chips else ""
    cardh = tpl.card(lead + panel, foot, tabs_html=t, label="Mapping set")
    rep = running_report() if report is None else report
    content = set_heading(more_open) + f'<div class="fui-list-page">{rep}{cardh}</div>'
    return shell("Mapping set", "mappings", content, toast_html=toast_html, overlay=overlay, incidents=incidents, status=status, unknown="loading" if loading else "down")


def page_mappings(toast_html=""):
    return set_page(toast_html=toast_html)


def page_mappings_hover():
    return set_page(body=ctx_rows(hover=1))


def page_mappings_menu():
    return set_page(body=ctx_rows(hover=1, menu_open=1))


def page_mappings_omocl():
    return set_page("omocl")


def page_mappings_templates():
    return set_page("templates")


def page_mappings_files():
    return set_page("files", body=file_rows(), tools=files_toolbar(), foot=pager("Mapping files", 27, 25, 1, sizes=(25, 50, 100)))


def page_mappings_files_filtered():
    items = [x for x in all_files() if x[3] == "console"]
    return set_page("files", body=file_rows(items), tools=files_toolbar(owner="console", open_=True), chips=tpl.active(["Owner: console"]))


def page_mappings_files_no_match():
    body = quiet('No OMOCL file the console saved has “weight” in its path. <a href="mappings-files.html">Clear filters</a>')
    return set_page("files", body=body, tools=tpl.toolbar("Path or file name", "weight", [tpl.filter_button("Language", "OMOCL"), tpl.filter_button("Owner", "console")]),
                    chips=tpl.active(["Language: OMOCL", "Owner: console", "Path: weight"]))


def page_mappings_diagnostics():
    return set_page("diagnostics", body=diags_list(DIAGNOSTICS, group=True))


def page_mappings_more():
    return set_page(more_open=True)


def reload_dialog():
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="rl-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="rl-title" class="fui-dialog-title">{icon("refresh-cw", "18")} Reload the set?</h2>'
        '<p>The bridge compiles every file on disk and swaps in the new set only when it compiles. A request in flight finishes on the old set; an error changes nothing.</p>'
        '<p class="fui-small fui-muted">This reloads the replica behind <code>bridge.demo-regio.example.org</code> only.</p>'
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Reload the set", "refresh-cw")}</div></div>'
    )


def page_mappings_reload():
    return set_page(overlay=reload_dialog())


def page_mappings_reloaded():
    rep = load_report("ok", f'Reloaded {when("2026-10-21T09:44")}: the set compiled and replaced the running one', RUNNING_COUNTS)
    return set_page(report=rep)


def page_mappings_reload_refused():
    errs = [("error", "missing-key", "fhirconnect/models/specimen.model.yml", 3, 1, "metadata.version is required (FHIRconnect model-mapping schema).", None),
            ("error", "unknown-archetype", "fhirconnect/synthetic_lab_result.context.yml", 16, 7, "The context lists CLUSTER.specimen.v2, which the template synthetic.lab_report.v1 does not hold.", None)] + DIAGNOSTICS[:1]
    rep = load_report("bad", f'Not reloaded {when("2026-10-21T09:44")}: the files on disk do not compile, so the running set is unchanged', [("2", "errors"), ("1", "warning"), ("27", "files read")],
                      link_button("secondary", "Reload again", "mappings-reload.html", "refresh-cw", small=True))
    body = diags_list(errs, group=True)
    t = tabs("Mapping set", [("diagnostics", "Diagnostics of the reload", "mappings-reload-refused.html", 3), ("contexts", "Running contexts", "mappings.html", 9)], "diagnostics")
    cardh = tpl.card(body, tabs_html=t, label="Reload diagnostics")
    content = set_heading() + f'<div class="fui-list-page">{rep}{cardh}</div>'
    # NOTE: the nav counts the running load's errors, and the refused reload left it at 0.
    return shell("Mapping set", "mappings", content, incidents=0)


def page_mappings_loading():
    rows = "".join(f'<tr aria-hidden="true"><th data-role="title">{skeleton_lines([80])}</th><td data-role="sub">{skeleton_lines([60])}</td><td data-role="wide">{skeleton_lines([70])}</td>'
                   f'<td data-role="meta-q">{skeleton_lines([20])}</td><td data-role="wide">{skeleton_lines([40])}</td><td data-role="wide">{skeleton_lines([50])}</td><td data-role="meta-2">{skeleton_lines([60])}</td><td></td></tr>' for _ in range(6))
    body = table("Contexts, loading", th("Context") + th("Resource type") + th("Template") + th("Models", num=True) + th("Search") + th("Owner") + th("State") + th('<span class="fui-sr-only">Actions</span>'), [rows])
    rep = '<div class="fui-load-report" aria-hidden="true"><span class="fui-skeleton mk-w40"></span></div>'
    return set_page(body=body, report=rep, incidents=None, loading=True)


def page_mappings_unreachable():
    content = page_heading("Mapping set", subtitle=SUB) + '<div class="fui-list-page">' + unreachable_block("the mapping set") + tech_detail() + "</div>"
    return shell("Mapping set", "mappings", content, status="down", incidents=None)


def page_mappings_forbidden():
    body = (quiet('The operator API is off on this bridge, so the mapping set is not shown; the Playground, the FHIR facade and the connections still work. '
                  '<a href="configuration-core.html">Where it is set</a>')
            + '<p class="fui-small fui-muted"><code>[operator] enabled = false</code> · the bridge answers 404 for <code>GET /operator/state</code>. '
            'With it on, a token needs the operator scope: <code>[auth] product_scope</code>, by default <code>__ferrobridge.operator</code>.</p>')
    content = page_heading("Mapping set", subtitle=SUB) + f'<div class="fui-list-page">{body}</div>'
    return shell("Mapping set", "mappings", content, incidents=None, unknown="off")


def page_mappings_empty():
    body = quiet('No mapping file yet: <code>/etc/ferrobridge/mappings</code> and <code>/var/lib/ferrobridge/console</code> are both empty, so the facade serves no resource type. '
                 '<a href="edit-new.html">Write the first file</a>')
    rep = load_report("neutral", "The running set is empty", [("0", "files")])
    content = set_heading() + f'<div class="fui-list-page">{rep}{body}</div>'
    return shell("Mapping set", "mappings", content)


def page_mappings_import():
    form = field("imp-zip", "ZIP of a mapping set", '<div class="fui-well">' + icon("upload", "16") + ' <strong>demo-region-mappings-2026-10.zip</strong> · 31 files · 48 KB</div>',
                 help_="As an export wrote it.")
    rep = load_report("ok", "Checked: the set compiles with the imported files", [("4", "files changed"), ("1", "added"), ("0", "errors"), ("3", "warnings")])
    diags = section("Diagnostics", diags_list(DIAGNOSTICS, group=True), len(DIAGNOSTICS))
    body = (planned(610, "Export the set as a ZIP, and import one as a checked revision whose diagnostics show before anything runs. "
                         'A revision is applied from the Mapping set: <a href="mappings-reload.html">Reload</a>.')
            + card("The ZIP", form) + rep + f'<section class="fui-card fui-panel">{diags}</section>')
    d = dock(dock_field("imp-reason", "Reason", "Mappings as merged in the regional mapping repository", help_="One line, at most 200 characters"),
             link_button("secondary", "Cancel", "mappings.html", "x") + button("primary", "Save as a revision", "save"))
    content = page_heading("Import a mapping set", crumbs=[("Mapping set", "mappings.html")]) + f'<div class="fui-stack">{body}</div>'
    return shell("Import a mapping set", "mappings", content, dock_html=d)


# ---- one mapping file ----

FILE = "fhirconnect/synthetic_body_weight.context.yml"
FILE_DIAGS = [d for d in DIAGNOSTICS if d[2] == FILE]


def file_menu(open_=False, console=True):
    items = [menu_item("Copy path", "copy"), menu_item("Download", "download"), menu_item("Check against the set", "list-checks", desc="Compiles the set with this file; writes nothing")]
    return items


def file_heading(title=FILE, sub=None, console=True, more_open=False, badge_html=None, primary=None):
    p = primary if primary is not None else (link_button("primary", "Edit", "edit.html", "pencil", keys="e") if console else link_button("primary", "Check a draft", "edit-provisioned.html", "list-checks"))
    ag = action_group(p, file_menu(), open_=more_open, label=f"More actions for {short(title)}")
    s = sub or "Context of <strong>Observation</strong> on <a href=\"template.html\"><code>synthetic.vital_signs.v1</code></a>, saved by the console."
    return page_heading(short(title), actions=ag, subtitle=s, crumbs=[("Mapping set", "mappings-files.html")], badge=badge_html or state("warn", "1 warning"))


def file_tabs(cur, href_prefix="file"):
    items = [("source", "Source", f"{href_prefix}.html"), ("program", "Program", f"{href_prefix}-program.html")]
    return tabs(f"{short(FILE)} sections", items, cur)


def source_panel(text=CONTEXT_YAML, diags=FILE_DIAGS, current=None, path=FILE, owner="console", lang="fhirconnect", sha="5712e3f7…f2e0e3fa", framed=False):
    """B1 beside B3, split by a hairline inside the tab panel; `framed` puts
    them in their own card on a page with no tabs."""
    cv = codeview(path, text, diags, current, lang, owner, sha)
    if diags:
        side = section("Diagnostics", diags_list(diags, current=current, href="file-line.html"), len(diags))
    else:
        side = section("Diagnostics", quiet("No diagnostics: the running load raised none about this file."))
    split = f'<div class="fui-panel fui-source-split">{cv}{side}</div>'
    return f'<section class="fui-card fui-panel">{split}</section>' if framed else split


def page_file(current=None, more_open=False, toast_html=""):
    h = file_heading(more_open=more_open)
    body = tabbed(file_tabs("source"), source_panel(current=current))
    return shell(short(FILE), "mappings", h + f'<div class="fui-stack">{body}</div>', toast_html=toast_html)


def page_file_line():
    return page_file(current=FILE_DIAGS[0][3])


def page_file_more():
    return page_file(more_open=True)


def program_panel():
    facts = facts_row([("Resource type", "Observation"), ("Profile", f'<code>{wb(PROFILE_BASE + "synthetic-body-weight")}</code>'),
                       ("Template", '<a href="template.html"><code>synthetic.vital_signs.v1</code></a>'), ("Start", "<code>OBSERVATION.body_weight.v2</code>")])
    models = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="file-model.html">body_weight</a></th>' + td("File", f'<code>{wb("fhirconnect/models/body_weight.model.yml")}</code>', role="wide")
              + td("Owner", owner_tag("console"), role="meta-2") + "</tr>"]
    mt = table("Models", th("Model") + th("File") + th("Owner"), models)
    search = quiet("No search file names this context, so it adds no search parameter to Observation.")
    lanes = definitions([("FHIR facade", '<a href="facade-capability.html">Observation</a>: read, vread, create, update, <code>$validate</code>'), ("Operations", "<code>$tofhir</code>, <code>$toopenehr</code>"),
                         ("HL7 v2", "ORU^R01 results with LOINC 29463-7")])
    return sections(section("The compiled program", facts), section("Models", mt, 1), section("Search", search), section("Where it runs", lanes))


def page_file_program():
    h = file_heading()
    return shell(short(FILE), "mappings", h + f'<div class="fui-stack">{tabbed(file_tabs("program"), program_panel())}</div>')


PROV = "fhirconnect/synthetic_vital_signs.context.yml"
PROV_YAML = (CONTEXT_YAML.replace("synthetic_body_weight.context", "synthetic_vital_signs.context").replace("synthetic-body-weight", "synthetic-vital-signs")
             .replace('    - "OBSERVATION.body_weight.v2"\n    - "CLUSTER.device.v1"\n  start: "OBSERVATION.body_weight.v2"',
                      '    - "OBSERVATION.blood_pressure.v2"\n    - "OBSERVATION.body_temperature.v2"\n    - "OBSERVATION.pulse.v2"\n  start: "OBSERVATION.blood_pressure.v2"')
             .replace("  version: 1.2.0", "  version: 2.0.1").replace('    version: "1.2.0"', '    version: "2.0.1"'))


def page_file_provisioned():
    sub = ('Context of <strong>Observation</strong> on <a href="template.html"><code>synthetic.vital_signs.v1</code></a>, provisioned.')
    h = file_heading(PROV, sub, console=False, badge_html=state("ok", "compiles"))
    note = notice("info", "lock", "<strong>Provisioned, so read-only here.</strong> Check a draft against the set, or save it under a path of its own.")
    body = note + tabbed(tabs("File sections", [("source", "Source", "file-provisioned.html"), ("program", "Program", "file-program.html")], "source"),
                         source_panel(PROV_YAML, [], path=PROV, owner="provisioned", sha="b20c9e41…07d1aa35"))
    return shell(short(PROV), "mappings", h + f'<div class="fui-stack">{body}</div>')


MODEL = "fhirconnect/models/body_weight.model.yml"
MODEL_DIAGS = [d for d in DIAGNOSTICS if d[2] == MODEL]


def page_file_model():
    sub = "Model of <code>OBSERVATION.body_weight.v2</code>, saved by the console."
    h = file_heading(MODEL, sub, badge_html=state("warn", "1 warning"))
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="file-program.html">synthetic_body_weight</a></th>' + td("Resource type", "Observation", role="sub") + "</tr>"]
    used = f'<section class="fui-card fui-panel"><div class="fui-panel-head"><h2 class="fui-card-title"><span>Used by <span class="mk-count">· 1 context</span></span></h2></div>' + table("Contexts that use this model", th("Context") + th("Resource type"), rows) + "</section>"
    body = source_panel(MODEL_YAML, MODEL_DIAGS, path=MODEL, sha="0e7a55c2…9b14d6e0", framed=True) + used
    return shell(short(MODEL), "mappings", h + f'<div class="fui-stack">{body}</div>')


OMOCL_FILE = "omocl/synthetic_drug.omocl.yml"
# NOTE: OMOCL v1.0.0, after docs/specs/omocl/medical_data/instruction/Medication_order_v3.yml.
OMOCL_YAML = """grammar: OMOCL/v1.0.0
type: model
metadata:
  name: synthetic_drug
  version: 1.0.0
spec:
  system: OMOP
  version: 5.4
  openEhrConfig:
    archetype: openEHR-EHR-INSTRUCTION.medication_order.v3
mappings:
  - type: "DrugExposure"
    base_path: "activities[at0001]"
    concept_id:
      alternatives:
        - path: "description[at0002]/items[at0070]"
    drug_exposure_start_date:
      alternatives:
        - path: "description[at0002]/items[at0113]/items[at0012]"
    drug_exposure_end_date:
      alternatives:
        - path: "description[at0002]/items[at0113]/items[at0013]"
    route_concept_id:
      optional: true
      alternatives:
        - path: "description[at0002]/items[at0091]"
"""


def page_file_omocl():
    sub = "OMOCL mapping onto <code>DRUG_EXPOSURE</code>, read by the next ETL run."
    h = file_heading(OMOCL_FILE, sub, badge_html=state("ok", "compiles"))
    body = source_panel(OMOCL_YAML, [], path=OMOCL_FILE, lang="omocl", sha="44d0b8e1…c2f97a13", framed=True)
    return shell(short(OMOCL_FILE), "mappings", h + f'<div class="fui-stack">{body}</div>')


def page_file_not_utf8():
    path = "fhirconnect/models/legacy_export.model.yml"
    sub = "Provisioned, 2,184 bytes."
    h = file_heading(path, sub, console=False, badge_html=state("bad", "not UTF-8"), primary="")
    body = quiet('The console does not show this file: the bridge reads mapping files as UTF-8, and byte 418 starts no UTF-8 sequence. Convert it where it is provisioned, then reload the set. '
                 '<a href="mappings-files.html">Every mapping file</a>')
    diag = [("error", "invalid-encoding", path, 12, 7, "Byte 418 is not UTF-8, so the loader read no further.", None)]
    split = f'<section class="fui-card fui-panel"><div class="fui-panel fui-source-split">{body}{section("Diagnostics", diags_list(diag, href="file-not-utf8.html"), 1)}</div></section>'
    return shell(short(path), "mappings", h + f'<div class="fui-stack">{split}</div>', incidents=1)


# ---- one template ----

def page_template():
    ag = action_group("", [menu_item("Try it in the Playground", "flask-conical", href="playground.html"), menu_item("Download the OPT", "download"), menu_item("Copy the id", "copy")], label="More actions for synthetic.vital_signs.v1")
    h = page_heading("synthetic.vital_signs.v1", actions=ag, subtitle="An operational template, read from the template directory.", crumbs=[("Mapping set", "mappings-templates.html")])
    facts = facts_row([("Source", "template directory"), ("File", f'<code>{wb("templates/synthetic_vital_signs.opt")}</code>'), ("Archetypes", "5"), ("Contexts", "2")])
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{("file-program.html" if k == "body_weight" else "file-provisioned.html")}">{e(ctx_name(p))}</a></th>'
            + td("Resource type", rt, role="sub") + td("Owner", owner_tag(o), role="wide") + td("State", ctx_state(w), role="meta-2") + "</tr>"
            for k, p, rt, pr, tid, m, s_, o, w in CONTEXTS if tid == "synthetic.vital_signs.v1"]
    t = table("Contexts that name it", th("Context") + th("Resource type") + th("Owner") + th("State"), rows)
    arch = '<ul class="mk-plain-list mk-ff-mono-list">' + "".join(f"<li><code>{a}</code></li>" for a in ("OBSERVATION.blood_pressure.v2", "OBSERVATION.body_temperature.v2", "OBSERVATION.pulse.v2", "OBSERVATION.body_weight.v2", "CLUSTER.device.v1")) + "</ul>"
    body = (f'<div class="fui-stack">{card("Template", facts)}<section class="fui-card fui-panel"><div class="fui-panel-head"><h2 class="fui-card-title"><span>Contexts that name it <span class="mk-count">· 2</span></span></h2></div>{t}</section>'
            f'{card("Archetypes", arch)}</div>')
    return shell("synthetic.vital_signs.v1", "mappings", h + body)
