# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Queries: the stored queries grouped by namespace, one stored query with
Definition, Run and Versions, and the Query workbench (Build | AQL) with
its shared results pane and Save to the CDR."""
import tpl
from common import ago
from common import (
    MENU_SEP, action_group, badge, busy_button, button, check, e, field, icon, icon_button, input_, link_button, menu, menu_item, notice,
    page_heading, refusal, select, shell, table, tabbed, tabs, td, th, when,
)
from data import BP_AQL, E1, QUERIES, TEMPLATES
from ehr import (
    UNIT_WORD, aql_html, codeview, confirm_delete, diag_list, editor, empty_line, facts, intro, load_report, quiet, results_head, section, sections, short, state,
    version_cell,
)
from parts import line_chart

# ---- Stored queries ----


def sq_heading():
    return page_heading("Stored queries", subtitle="The CDR's stored queries, grouped by the namespace of their name.")


def sq_menu(name, open_=False):
    items = [menu_item("Run", "play", href="stored-query-run.html"), menu_item("Open in Query", "square-terminal", href="query-aql.html"), MENU_SEP,
             menu_item("Delete this version…", "trash-2", danger=True, href="stored-query-delete.html")]
    m = menu(items, label=f"Actions for {name}", sheet=True) if open_ else ""
    return (f'<span class="fui-anchor"><button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
            f'aria-label="More actions for {e(name)}" title="More actions">{icon("ellipsis", "16")}</button>{m}</span>')


def sq_rows(items=QUERIES, open_row=None, only=None):
    body = ""
    groups = []
    for q in items:
        if q[0] not in groups:
            groups.append(q[0])
    i = 0
    for g in groups:
        if only and g != only:
            continue
        qs = [q for q in items if q[0] == g]
        names = len({q[1] for q in qs})
        label = f"<code>{g}</code>" if g else "Unqualified"
        vers = f" · {len(qs)} versions" if len(qs) != names else ""
        body += f'<tr class="fui-group-row"><th scope="rowgroup" colspan="4">{label} <span class="fui-small fui-muted">· {names} {"query" if names == 1 else "queries"}{vers}</span></th></tr>'
        for ns, name, ver, params in qs:
            p = ", ".join(f"<code>{x}</code>" for x in params) if params else '<span class="mk-dash">none</span>'
            older = any(q[1] == name and q[0] == ns and q[2] > ver for q in items)
            sub = '<span class="fui-row-sub">older version</span>' if older else ""
            href = "stored-query-version.html" if older else "stored-query.html"
            body += (f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{e(name)}</a>{sub}</th>'
                     + td("Version", ver, role="sub") + td("Parameters", p, role="meta-q") + td("", sq_menu(name, open_row == i), "fui-row-menu", role="menu") + "</tr>")
            i += 1
    return table("Stored queries", th("Name") + th("Version") + th("Parameters") + th('<span class="fui-sr-only">Actions</span>'), [body])


def sq_page(body=None, chips=(), value="", count="7 queries, 8 versions"):
    tb = ""
    c = tpl.card(body if body is not None else sq_rows(), label="Stored queries", title="Stored queries", count=count,
                 actions=link_button("ghost", "Write one in Query", "query-aql-new.html", "square-terminal", small=True))
    return shell("Stored queries", "stored", sq_heading() + tpl.list_body("", tb, tpl.active(chips), c))


def page_stored_queries():
    return sq_page()


def page_stored_queries_row_menu():
    return sq_page(sq_rows(open_row=0))


def page_stored_queries_namespace():
    return sq_page(sq_rows(only="org.example.cardio"), chips=["Namespace org.example.cardio, from the dashboard"], count="2 queries, 3 versions")


def page_stored_queries_empty():
    c = tpl.card(empty_line("No stored query yet."), label="Stored queries", title="Stored queries", count="0",
                 actions=link_button("ghost", "Write one in Query", "query-aql-new.html", "square-terminal", small=True))
    return shell("Stored queries", "stored", sq_heading() + tpl.list_body("", "", "", c))


SQ_TABS = [("definition", "Definition", "stored-query.html"), ("run", "Run", "stored-query-run.html"), ("versions", "Versions", "stored-query-versions.html", 2)]
QNAME = "org.example.cardio::bp_latest"


def sq_header(version="1.1.0", more_open=False):
    more = [menu_item("Copy name", "copy"), MENU_SEP, menu_item(f"Delete version {version}…", "trash-2", danger=True, href="stored-query-delete.html")]
    ag = action_group(link_button("primary", "Open in Query", "query-aql.html", "square-terminal"), more, open_=more_open, label="More actions for the stored query")
    return page_heading("bp_latest", actions=ag, crumbs=[("Stored queries", "stored-queries.html")], subtitle=f"<code>{QNAME}</code> · version {version}")


def sq_detail(tab, panel, overlay=None, version="1.1.0", more_open=False, tabs_list=SQ_TABS):
    return shell("bp_latest", "stored", sq_header(version, more_open) + f'<div class="fui-stack">{tabbed(tabs("Views of this stored query", tabs_list, tab), panel)}</div>', overlay=overlay)


def page_stored_query(overlay=None, more_open=False):
    f = facts([("Type", "AQL"), ("Saved", when("2026-10-08T14:22", absolute=True)),
               ("Parameters", "<code>ehr_id</code>")])
    return sq_detail("definition", sections(section("Facts", f), section("AQL", codeview("bp_latest 1.1.0", BP_AQL, lang="aql", what="the AQL"))), overlay, more_open=more_open)


def page_stored_query_more():
    return page_stored_query(more_open=True)


BP_AQL_100 = """SELECT
  c/context/start_time/value AS time,
  o/data[at0001]/events[at0006]/data[at0003]/items[at0004]/value/magnitude AS systolic
FROM EHR e[ehr_id/value=$ehr_id]
  CONTAINS COMPOSITION c
    CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.blood_pressure.v2]
ORDER BY c/context/start_time/value DESC"""


def page_stored_query_version():
    top = notice("info", "history", f'You are reading version 1.0.0, saved {when("2026-09-02T10:15", absolute=True)}; Latest runs 1.1.0. <a href="stored-query.html">Read 1.1.0</a>')
    f = facts([("Type", "AQL"), ("Saved", when("2026-09-02T10:15", absolute=True)), ("Parameters", "<code>ehr_id</code>")])
    tl = [("definition", "Definition", "stored-query-version.html"), ("versions", "Versions", "stored-query-versions.html", 2)]
    return sq_detail("definition", sections(section("", top), section("Facts", f), section("AQL", codeview("bp_latest 1.0.0", BP_AQL_100, lang="aql", what="the AQL"))), version="1.0.0", tabs_list=tl)


RES_ROWS = [("2026-10-21T07:50:00+02:00", "128", "82"), ("2026-10-07T09:05:00+02:00", "141", "88"), ("2026-09-23T08:51:00+02:00", "136", "85"),
            ("2026-09-09T10:12:00+02:00", "144", "91"), ("2026-08-26T09:40:00+02:00", "139", "87"), ("2026-08-11T16:30:00+02:00", "151", "94")]


BUILT_ROWS = [r for r in RES_ROWS if int(r[1]) > 140]


def results_table(rows=RES_ROWS):
    out = [f'<tr><th scope="row" data-role="title"><code>{t}</code></th>{td("systolic", s, "fui-num", role="sub")}{td("diastolic", d, "fui-num", role="meta-q")}</tr>' for t, s, d in rows]
    return table("Results", th("time") + th("systolic", num=True) + th("diastolic", num=True), out)


def results_chart(rows=RES_ROWS):
    sys_ = [int(r[1]) for r in reversed(rows)]
    dia = [int(r[2]) for r in reversed(rows)]
    months = {"08": "Aug", "09": "Sep", "10": "Oct"}
    lab = [f"{int(r[0][8:10])} {months[r[0][5:7]]}" for r in reversed(rows)]
    n = len(rows) - 1
    ticks = [(0, lab[0]), (n // 2, lab[n // 2]), (n, lab[n])]
    return line_chart("res", "systolic and diastolic over time", f"Systolic from {sys_[0]} to {sys_[-1]} and diastolic from {dia[0]} to {dia[-1]}, {lab[0]} to {lab[-1]} 2026.",
                      [("systolic", sys_, 0), ("diastolic", dia, 1)], ticks, height=220, y_from=min(dia) // 20 * 20)


def results_section(view="table", window="Rows 1 to 6 of 6. No LIMIT in the query, so the CDR returns at most 1,000 rows; times are as the CDR returns them.", rows=RES_ROWS):
    body = results_table(rows) if view == "table" else f'<div class="fui-panel-body">{results_chart(rows)}<p class="fui-small fui-muted">The x axis is the time column, at its true spacing.</p></div>'
    return section("Results", f'<p class="fui-results-window">{window}</p>' + body, count=f"{len(rows)} rows", actions=results_head("", "", view))


def run_form(resolution="latest", version="", invalid=False):
    segs = "".join(f'<label class="fui-segment"><input type="radio" class="fui-sr-only" name="res" value="{k}"{" checked" if k == resolution else ""} />{w}</label>'
                   for k, w in [("latest", "Latest"), ("prefix", "A prefix"), ("exact", "Exact")])
    req = {"latest": f"POST /query/{QNAME}", "prefix": f"POST /query/{QNAME}/{version or '1'}", "exact": f"POST /query/{QNAME}/{version or '1.1.0'}"}[resolution]
    err = "1.1.0 is a full version, so it cannot be a prefix. Choose Exact, or type 1 or 1.1." if invalid else None
    ver = field("rv", "Version", input_("rv", version, "1 or 1.1", extra='class="fui-input fui-mono"' + (' aria-invalid="true"' if invalid else "")), error=err, cls="is-short") if resolution != "latest" else ""
    return (f'<fieldset class="fui-fieldset fui-choice"><legend class="fui-sr-only">Which version runs</legend><div class="fui-segmented">{segs}</div></fieldset>'
            f'{ver}<p class="fui-small fui-muted">Sends <code>{e(req).replace("/", "/<wbr>")}</code></p>')


def run_params():
    return ('<div class="fui-form-grid">' + field("p-ehr", "ehr_id", input_("p-ehr", E1, extra='class="fui-input fui-mono" spellcheck="false"'),
                                                  help_="Numbers and true or false are sent as they are; quote a number to send it as text.", cls="fui-span-2") + "</div>")


def page_stored_query_run():
    body = sections(section("Version", run_form()), section("Parameters", run_params() + f'<div class="fui-transform-run mk-run-row">{button("primary", "Run", "play", keys="Ctrl Enter")}</div>', count=1),
                    section("Results", quiet("Run the query to see its rows here.")))
    return sq_detail("run", body)


def page_stored_query_run_answered():
    body = sections(section("Version", run_form()), section("Parameters", run_params() + f'<div class="fui-transform-run mk-run-row">{button("primary", "Run", "play", keys="Ctrl Enter")}</div>', count=1),
                    results_section())
    return sq_detail("run", body)


def page_stored_query_run_invalid():
    body = sections(section("Version", run_form("prefix", "1.1.0", True)), section("Parameters", run_params() + f'<div class="fui-transform-run mk-run-row">{button("primary", "Run", "play", keys="Ctrl Enter")}</div>', count=1),
                    section("Results", quiet("Nothing was sent: fix the version first.")))
    return sq_detail("run", body)


def page_stored_query_versions():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="stored-query.html">{version_cell("1.1.0", current=True).replace("Version ", "")}</a></th>{td("Saved", ago("2026-10-08T14:22"), role="sub")}{td("Parameters", "<code>ehr_id</code>", role="meta-q")}</tr>',
            f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="stored-query-version.html">{version_cell("1.0.0").replace("Version ", "")}</a></th>{td("Saved", ago("2026-09-02T10:15"), role="sub")}{td("Parameters", "<code>ehr_id</code>", role="meta-q")}</tr>']
    t = table("Versions", th("Version") + th("Saved") + th("Parameters"), rows)
    return sq_detail("versions", sections(section("Versions", intro("Each version is its own immutable definition; Latest runs the highest.") + t, count=2)))


def page_stored_query_delete():
    d = confirm_delete("del", "Delete bp_latest version 1.1.0?", "Removes this version from the CDR's stored queries for good, for every client; version 1.0.0 stays.", True, "Delete version 1.1.0")
    return page_stored_query(overlay=d)


# ---- the Query workbench ----

Q_TABS = [("build", "Build", "query.html"), ("aql", "AQL", "query-aql.html")]

BUILT_AQL = """SELECT
  c/context/start_time/value AS time,
  o/data[at0001]/events[at0006]/data[at0003]/items[at0004]/value/magnitude AS systolic,
  o/data[at0001]/events[at0006]/data[at0003]/items[at0005]/value/magnitude AS diastolic
FROM EHR e
  CONTAINS COMPOSITION c[openEHR-EHR-COMPOSITION.encounter.v1]
    CONTAINS (OBSERVATION o[openEHR-EHR-OBSERVATION.blood_pressure.v2]
      AND OBSERVATION p[openEHR-EHR-OBSERVATION.pulse.v2]
      AND OBSERVATION t[openEHR-EHR-OBSERVATION.body_temperature.v2])
WHERE o/data[at0001]/events[at0006]/data[at0003]/items[at0004]/value/magnitude > 140
  AND o/data[at0001]/events[at0006]/state[at0007]/items[at0008]/value/defining_code/code_string = 'at1001'
  AND NOT (p/data[at0002]/events[at0003]/data[at0001]/items[at0004]/value/magnitude > 100
    OR t/data[at0002]/events[at0003]/data[at0001]/items[at0004]/value/magnitude > 38)
ORDER BY c/context/start_time/value DESC
LIMIT 50"""


def save_popover(conflict=False):
    err = "Version 1.1.0 of this name already exists, and a saved version cannot change." if conflict else None
    return (f'<div class="fui-popover mk-save-pop" role="dialog" aria-labelledby="save-title"><p id="save-title" class="fui-popover-title">Save to the CDR</p>'
            + field("sv-ns", "Namespace", input_("sv-ns", "org.example.cardio", extra='class="fui-input fui-mono"'))
            + field("sv-n", "Name", input_("sv-n", "bp_latest", extra='class="fui-input fui-mono"'))
            + field("sv-v", "Version", input_("sv-v", "1.1.0" if conflict else "1.2.0", "Empty writes 1.0.0", extra='class="fui-input fui-mono"' + (' aria-invalid="true"' if conflict else "")), error=err)
            + (f'<div class="fui-cluster">{button("secondary", "Use 1.2.0", small=True)}</div>' if conflict else '<p class="fui-small fui-muted">Saved versions: 1.0.0 and 1.1.0.</p>')
            + f'<div class="fui-cluster">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Save", "save", attrs="disabled" if conflict else "")}</div></div>')


def q_header(running=False, save_open=False, conflict=False, aql=False, run_title=None):
    if running:
        more = (f'<span class="fui-anchor"><button type="button" class="fui-button fui-button-secondary fui-more" aria-haspopup="menu" aria-expanded="false" aria-label="More actions for the query">'
                f'More{icon("chevron-down", "16")}</button></span>')
        return page_heading("Query", actions=f'<div class="fui-action-group">{busy_button("primary", "Running")}{button("secondary", "Cancel", "x")}{more}</div>',
                            subtitle="Build AQL from a template, or write it; run it against the CDR.")
    run = button("primary", "Run", "play", keys="Ctrl Enter", attrs=f'title="{e(run_title)}"' if run_title else "")
    save = (f'<span class="fui-anchor">{button("secondary", "Save to the CDR", "save", attrs="aria-haspopup=" + chr(34) + "dialog" + chr(34) + " aria-expanded=" + chr(34) + ("true" if save_open else "false") + chr(34))}'
            f'{save_popover(conflict) if save_open else ""}</span>')
    return page_heading("Query", actions=f'<div class="fui-action-group">{run}{save}</div>', subtitle="Build AQL from a template, or write it; run it against the CDR.")


def q_page(tab, panel, header=None, overlay=None):
    h = header or q_header()
    return shell("Query", "query", h + f'<div class="fui-stack">{tabbed(tabs("Ways to write the query", Q_TABS, tab), panel)}</div>', overlay=overlay)


def cond(n, label, path, op, value, unit=None, ops=(">", "≥", "=", "<", "≤", "≠"), coded=False):
    if coded:
        val = select(f"cv{n}", [("at1000", "Standing · at1000"), ("at1001", "Sitting · at1001"), ("at1002", "Reclining · at1002"), ("at1003", "Lying · at1003"), ("at1014", "Lying with tilt to left · at1014")], value)
    else:
        val = (f'<div class="fui-input-group">{input_(f"cv{n}", value, extra="inputmode=" + chr(34) + "decimal" + chr(34))}<span class="fui-input-suffix" title="UCUM {e(unit)}">{e(UNIT_WORD.get(unit, unit))}</span></div>' if unit
               else input_(f"cv{n}", value))
    return (f'<li class="fui-cond"><div class="fui-field"><label class="fui-label" for="cp{n}">Path</label>'
            f'{select(f"cp{n}", [("p", label)], "p")}<span class="fui-cond-path" title="{e(path)}">{e(path)}</span></div>'
            f'{field(f"co{n}", "Operator", select(f"co{n}", [(o, o) for o in ops], op))}{field(f"cv{n}", "Value", val)}'
            f'{icon_button("x", f"Remove the condition on {label}")}</li>')


def group(kind, conds, nested="", not_=False, n=1, remove=True):
    seg = (f'<div role="group" aria-label="Group {n} matches" class="fui-segmented">'
           f'<button type="button" class="fui-segment" aria-pressed="{"true" if kind == "all" else "false"}">All</button>'
           f'<button type="button" class="fui-segment" aria-pressed="{"true" if kind == "any" else "false"}">Any</button></div>')
    acts = f'<div class="fui-actions">{icon_button("trash-2", f"Remove group {n}")}</div>' if remove else ""
    head = (f'<div class="fui-cond-head"><span class="fui-cond-phrase">Match</span>{seg}<span class="fui-cond-phrase">of these conditions</span>'
            f'{check(f"not{n}", "Match the opposite (NOT)", not_)}{acts}</div>')
    sub = f" to group {n}" if n > 1 else ""
    adds = (f'<div class="fui-add-row">{button("ghost", "Add a condition", "plus", small=True, attrs=f"aria-label={chr(34)}Add a condition{sub}{chr(34)}")}'
            + (button("ghost", "Add a group", "plus", small=True) if n == 1 else "") + '</div>')
    return f'<div class="fui-cond-group{" is-not" if not_ else ""}">{head}<ul class="fui-cond-list">{"".join(conds)}{nested}</ul>{adds}</div>'


SYS_PATH = "…/items[at0004]/value/magnitude"


def build_panel(results=None):
    opts = (f'<div class="fui-transform-options">{field("q-t", "Template", select("q-t", [(t[0], t[0]) for t in TEMPLATES[:6]], "Vital signs encounter.v1"))}'
            f'{field("q-shape", "Returns", select("q-shape", [("data", "Data points"), ("comp", "Whole compositions"), ("count", "A count"), ("ehrs", "EHRs (cohort)")], "data"))}</div>')
    pts = ('<div class="fui-repeater"><div class="fui-repeater-head" aria-hidden="true"><span class="fui-label">Data point</span><span class="fui-label">Column name</span><span></span></div>'
           + "".join(f'<div class="fui-repeater-row"><div class="fui-field"><label class="fui-sr-only" for="dp{i}">Data point {i + 1}</label>{select(f"dp{i}", [(a, w)], a)}</div>'
                     f'<div class="fui-field"><label class="fui-sr-only" for="da{i}">Column name of data point {i + 1}</label>{input_(f"da{i}", a, extra="class=" + chr(34) + "fui-input fui-mono" + chr(34))}</div>'
                     f'{icon_button("x", f"Remove the data point {a}")}</div>'
                     for i, (a, w) in enumerate([("time", "Context › Start time"), ("systolic", "Blood pressure › Systolic"), ("diastolic", "Blood pressure › Diastolic")]))
           + f'</div><div class="fui-add-row">{button("ghost", "Add a data point", "plus", small=True)}</div>')
    inner = group("any", [cond(3, "Pulse › Rate", "…/items[at0004]/value/magnitude", ">", "100", "/min"), cond(4, "Body temperature", "…/items[at0004]/value/magnitude", ">", "38", "Cel")],
                  not_=True, n=2)
    conds = group("all", [cond(1, "Blood pressure › Systolic", SYS_PATH, ">", "140", "mm[Hg]"), cond(2, "Blood pressure › Position", "…/items[at0008]/value", "=", "at1001", ops=("=", "≠"), coded=True)],
                  nested=f"<li>{inner}</li>", n=1, remove=False)
    order = (f'<div class="fui-transform-options">{field("q-o", "Order by", select("q-o", [("t", "Context › Start time")], "t"))}'
             f'{field("q-d", "Direction", select("q-d", [("desc", "Newest first"), ("asc", "Oldest first")], "desc"))}{field("q-l", "Row limit", input_("q-l", "50", extra="inputmode=" + chr(34) + "numeric" + chr(34)))}</div>')
    aql = section("The AQL it builds", codeview("Built AQL", BUILT_AQL, lang="aql", what="the AQL", actions=False).replace(
        '<span>15 lines</span>', '<span>15 lines</span>' + link_button("ghost", "Open in AQL", "query-aql-built.html", "arrow-right", small=True)))
    parts = [section("", opts), section("Data points", pts, count=3), section("Conditions", conds), section("Order and limit", order), aql]
    parts.append(results if results is not None else section("Results", quiet("Run the query to see its rows here.")))
    return sections(*parts)


def page_query():
    return q_page("build", build_panel())


def page_query_build_results():
    return q_page("build", build_panel(results_section(window="Rows 1 to 3 of 3, inside the query's own LIMIT 50.", rows=BUILT_ROWS)))


def page_query_build_chart():
    return q_page("build", build_panel(results_section("chart", window="Rows 1 to 3 of 3, inside the query's own LIMIT 50.", rows=BUILT_ROWS)))


def page_query_running():
    return q_page("build", build_panel(section("Results", '<p class="fui-transform-empty"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Running against the CDR</span> Edits to the query apply to the next run.</p>')),
                  header=q_header(running=True))


def aql_panel(text=BP_AQL, diags=None, results=None, top="", param=("ehr_id", E1), dirty=False, pos="Ln 1, Col 1", build="link"):
    tools = {"link": link_button("ghost", "Open in Build", "query.html", "list-tree", small=True),
             "disabled": '<button type="button" class="fui-button fui-button-ghost fui-button-small" disabled title="Fix the grammar error first">' + icon("list-tree", "16") + "Open in Build</button>",
             "none": ""}[build]
    ed = editor(text, lang="aql", lang_word="AQL", label="The AQL", diags=diags, dirty=dirty, pos=pos, tools=tools)
    if diags:
        rows = [("bad", d[1], ln, 3, d[2]) for ln, d in diags.items()]
        ed = (f'<div class="fui-editor-split">{ed}<section class="fui-card fui-panel" aria-label="Diagnostics"><div class="fui-panel-head"><h2 class="fui-card-title">'
              f'<span>Grammar check <span class="mk-count">· {len(rows)}</span></span></h2></div>{diag_list(rows, current=rows[0][2])}</section></div>')
    pn, pv = param or ("", "")
    params = "" if param is None else ('<div class="fui-repeater"><div class="fui-repeater-head" aria-hidden="true"><span class="fui-label">Name</span><span class="fui-label">Value</span><span></span></div><div class="fui-repeater-row"><div class="fui-field"><label class="fui-sr-only" for="pn0">Name of parameter 1</label>'
              + input_("pn0", pn, extra='class="fui-input fui-mono"') + '</div><div class="fui-field"><label class="fui-sr-only" for="pv0">Value of parameter 1</label>'
              + input_("pv0", pv, extra='class="fui-input fui-mono" spellcheck="false"') + "</div>" + icon_button("x", f"Remove the parameter {pn}") + "</div></div>"
              + f'<div class="fui-add-row">{button("ghost", "Add a parameter", "plus", small=True)}</div>')
    parts = [section("", top + ed)] + ([section("Parameters", params, count=1)] if params else [])
    parts.append(results if results is not None else section("Results", quiet("Run the query to see its rows here.")))
    return sections(*parts)


def page_query_aql(overlay=None):
    return q_page("aql", aql_panel(), overlay=overlay)


def page_query_aql_invalid():
    bad = BP_AQL.replace("CONTAINS OBSERVATION", "CONTAIN OBSERVATION")
    diags = {7: ("bad", "AQL-SYNTAX", "Expected CONTAINS, AND, OR or WHERE; found CONTAIN.")}
    return q_page("aql", aql_panel(bad, diags, section("Results", quiet("Nothing was sent: the grammar check found 1 error.")), dirty=True, pos="Ln 7, Col 3", build="disabled"),
                  header=q_header(run_title="The grammar check found 1 error; fix it before you run"))


def page_query_aql_results():
    return q_page("aql", aql_panel(results=results_section()))


def page_query_save():
    return q_page("aql", aql_panel(), header=q_header(save_open=True))


def page_query_save_conflict():
    return q_page("aql", aql_panel(), header=q_header(save_open=True, conflict=True))


def page_query_cannot_build():
    top = notice("info", "info", "Opened here, not in Build: <code>hypertensive_ehrs</code> 1.0.0 uses <code>$systolic</code>, a parameter Build has no field for.")
    text = "SELECT DISTINCT e/ehr_id/value AS ehr_id\nFROM EHR e\n  CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.blood_pressure.v2]\nWHERE o/data[at0001]/events[at0006]/data[at0003]/items[at0004]/value/magnitude > $systolic"
    return q_page("aql", aql_panel(text, top=f'<div class="mk-short-stack">{top}</div>', param=("systolic", "140"), build="none"))


def page_query_aql_built():
    top = notice("info", "info", "The AQL Build made, as it runs; edits here leave Build behind.")
    return q_page("aql", aql_panel(BUILT_AQL, top=f'<div class="mk-short-stack">{top}</div>', param=None, pos="Ln 1, Col 1"))


def page_query_aql_new():
    return q_page("aql", aql_panel("", param=None))
