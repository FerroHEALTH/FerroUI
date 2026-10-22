# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The query console and the stored queries: the editor with its parameters
and federation options, every answer the gateway can give, and the stored
queries with their drift repair and every refusal of a new version."""
import json
import re

from common import (
    MENU_SEP, busy_button, button, card, check, definitions, disclosure, dock, dock_field, e, field, hint,
    icon, icon_button, input_, link_button, menu_item, notice, page_heading, pager, pill, quiet_line, refusal,
    rel, section, sections, select, shell, table, tabbed, tabs, td, textarea, th, when, copy_button,
)
import data
from data import AQL_VITALS, EEHRXF, PATIENT_NS, PATIENT_PARAM, STORED
from parts import skeleton_lines
import routes

KEYWORDS = r"\b(SELECT|FROM|CONTAINS|WHERE|AND|OR|NOT|ORDER BY|DESC|ASC|AS|LIMIT|OFFSET|EHR|COMPOSITION|OBSERVATION|EVALUATION|INSTRUCTION|ACTION)\b"


def hl_line(line):
    out, pos = "", 0
    pat = re.compile(r"(?P<arch>\[openEHR[^\]]*\])|(?P<str>'[^']*')|(?P<param>\$[A-Za-z_]\w*)|(?P<kw>" + KEYWORDS[2:-2] + r")\b|(?P<num>\b\d+(\.\d+)?\b)|(?P<path>[a-z_]+(?:\[[^\]]+\])?(?:/[A-Za-z_]+(?:\[[^\]]+\])?)+)")
    for m in pat.finditer(line):
        out += e(line[pos:m.start()])
        kind = m.lastgroup
        cls = {"arch": "string", "str": "string", "param": "parameter", "kw": "keyword", "num": "number", "path": "path"}[kind]
        out += f'<span class="fui-tok-{cls}">{e(m.group(0))}</span>'
        pos = m.end()
    return out + e(line[pos:])


def code_layer(text, marks=None):
    marks = marks or {}
    return "".join(f'<span class="fui-code-line{" fui-mark-" + marks[i + 1] if i + 1 in marks else ""}">{hl_line(l) or " "}</span>' for i, l in enumerate(text.split("\n")))


def editor(text, id_="aql", readonly=False, marks=None, invalid=False, placeholder="SELECT … FROM EHR e CONTAINS …"):
    ro = " readonly" if readonly else ""
    cls = "fui-editor" + (" is-readonly" if readonly else "")
    inv = ' aria-invalid="true"' if invalid else ""
    return (f'<div class="{cls}"{inv}><div class="fui-editor-stack"><pre class="fui-editor-layer" aria-hidden="true">{code_layer(text, marks)}</pre>'
            f'<textarea id="{id_}" name="{id_}" class="fui-editor-input" autocomplete="off" autocapitalize="off" spellcheck="false" wrap="soft" '
            f'placeholder="{e(placeholder)}"{ro}>{e(text)}</textarea></div></div>')


def code_panel(text):
    return f'<div class="fui-code-panel fui-code-panel-numbered" role="region" aria-label="AQL" tabindex="0"><pre>{code_layer(text)}</pre></div>'


# ---- the editor form ----

def param_rows(rows, errors=None):
    errors = errors or {}
    out = '<div class="fui-repeater-head" aria-hidden="true"><span class="fui-label">Name</span><span class="fui-label">Value</span><span></span></div>'
    for i, (n, v) in enumerate(rows):
        er = errors.get(i)
        inv = ' aria-invalid="true"' if er else ""
        out += (f'<div class="fui-repeater-row"><div class="fui-field"><label class="fui-sr-only" for="pn{i}">Name of parameter {i + 1}</label>{input_(f"pn{i}", n, extra="spellcheck=" + chr(34) + "false" + chr(34) + " class=" + chr(34) + "fui-input fui-mono" + chr(34) + inv)}</div>'
                f'<div class="fui-field"><label class="fui-sr-only" for="pv{i}">Value of parameter {i + 1}</label>{input_(f"pv{i}", v, extra="spellcheck=" + chr(34) + "false" + chr(34) + " class=" + chr(34) + "fui-input fui-mono" + chr(34))}</div>'
                f'{icon_button("x", f"Remove parameter {i + 1}")}</div>')
        if er:
            out += f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {er}</p>'
    return f'<div class="fui-repeater">{out}</div>'


def options_block(open_=True, chosen=(), dedup="default", best_effort=False):
    eps = "".join(f'<label class="fui-chip-check"><input type="checkbox"{" checked" if ep["id"] in chosen else ""} />{ep["id"]}</label>' for ep in data.IN_SERVICE)
    orgs = "".join(f'<label class="fui-chip-check"><input type="checkbox" />{e(o["name"])}</label>' for o in data.ORGS if o["standing"] == "active")
    body = (
        f'<div class="fui-field"><span class="fui-label" id="o-eps">Endpoints</span><div class="fui-chip-checks" role="group" aria-labelledby="o-eps">{eps}</div>'
        f'<p class="fui-help">Leave all clear to let the gateway choose. node-e-pub is excluded, so it is not offered.{hint("openEHR-federation-endpoint")}</p></div>'
        f'<div class="fui-field"><span class="fui-label" id="o-orgs">Organisations</span><div class="fui-chip-checks" role="group" aria-labelledby="o-orgs">{orgs}</div></div>'
        + field("o-dedup", "Deduplication", select("o-dedup", [("default", "None, the gateway's default"), ("version-identity", "By version identity")], dedup))
        + check("o-best", "Best effort", best_effort, "Answer with the rows that came back when a node does not answer. Off, the gateway answers all or nothing.", extra='class="fui-switch"')
    )
    bits = [f"{len(chosen)} endpoints" if chosen else "the gateway chooses the endpoints"] + (["best effort"] if best_effort else [])
    summary = "Federation options" + f' <span class="fui-muted">· {", ".join(bits)}</span>'
    return disclosure(summary, f'<div class="fui-stack">{body}</div>', open_=open_)


def query_form(kind="aql", text=AQL_VITALS, params=None, errors=None, running=False, marks=None, aql_error=None,
               options_open=False, chosen=(), best_effort=False, offset="", fetch="50"):
    params = params if params is not None else [("patient", PATIENT_PARAM), ("namespace", PATIENT_NS)]
    t = tabs("Query kind", [("aql", "AQL", "query.html"), ("stored", "Stored query", "query-stored.html")], kind)
    if kind == "aql":
        lines = len(text.split("\n")) if text else 0
        names = sorted(set(re.findall(r"\$(\w+)", text)))
        bar = (f'<div class="fui-editor-bar"><span>AQL · {lines} lines{" · names " + ", ".join("$" + n for n in names) if names else ""}</span>'
               f'<div class="fui-actions">{button("ghost", "Format", "braces", small=True)}{button("ghost", "Clear", "x", small=True)}</div></div>')
        ae = f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {aql_error}</p>' if aql_error else '<p class="fui-help">Name a patient as a parameter, never in the AQL text.</p>'
        left = f'<div class="fui-field"><label for="aql" class="fui-label">AQL</label>{bar}{editor(text, marks=marks, invalid=bool(aql_error))}{ae}</div>'
    else:
        left = (
            '<div class="fui-stack"><div class="fui-columns">'
            + field("sq-name", "Stored query", select("sq-name", [(n, n.split("::")[1]) for n, _, _ in STORED], "org.demo-regio::vital_signs"))
            + field("sq-version", "Version <span class=\"fui-muted\">(optional)</span>", select("sq-version", [("", "1.1.0, the newest"), ("1.1.0", "1.1.0"), ("1.0.0", "1.0.0")], ""))
            + "</div>"
            + f'<div class="fui-field"><span class="fui-label">Its AQL, version 1.1.0 <span class="fui-muted">· read-only</span></span>{editor(AQL_VITALS, "sq-aql", readonly=True)}</div>'
            + "</div>"
        )
    right = (
        '<div class="fui-stack">'
        f'<div class="fui-field"><span class="fui-label">Parameters</span>{param_rows(params, errors)}'
        f'<div>{button("secondary", "Add parameter", "plus", small=True)}</div>'
        '<p class="fui-help">Each value is sent as a string, in the request body, never in a URL.</p></div>'
        '<div class="fui-columns">'
        + field("offset", "Offset", input_("offset", offset, extra='inputmode="numeric"'), help_="Rows to skip.")
        + field("fetch", "Fetch", input_("fetch", fetch, extra='inputmode="numeric"'), help_="Rows to return.")
        + "</div>" + options_block(options_open, chosen, best_effort=best_effort) + "</div>"
    )
    if running:
        run = busy_button("primary", "Running") + button("secondary", "Stop", "x")
    else:
        run = button("primary", "Run query", "play", keys="Ctrl Enter", type_="submit")
    form = (f'<form class="fui-form" method="post" action="#"><div class="mk-ff-split">{left}{right}</div>'
            f'<div class="fui-run-row">{run}<p>Runs as you, through the gateway\'s own query surface, as a client sends it.</p></div></form>')
    return tabbed(t, form)


# ---- the answer: what each endpoint did, then the rows (E7) ----

ROWS = [
    ("2026-10-18T09:12", "128", "node-a-pub"), ("2026-10-11T14:40", "131", "node-c-pub"), ("2026-10-04T08:55", "126", "node-a-pub"),
    ("2026-09-27T10:05", "135", "node-a-lab-pub"), ("2026-09-20T09:30", "129", "node-a-pub"), ("2026-09-13T11:15", "133", "node-c-pub"),
]
STATUS_WORD = {"active": "Answered", "node-error": "Node error", "time-out": "Timed out", "offline": "Offline", "excluded": "Excluded", "not-localized": "Not asked", "not-resolved": "Not resolved"}


def outcomes(variant="complete"):
    """Every declared endpoint, as meta.federation.endpoints[] names it
    (§11.1): the localizer named four nodes; node-e-pub is excluded."""
    base = [("node-a-pub", "active", 3, 41, None), ("node-a-lab-pub", "active", 1, 74, None), ("node-b-pub", "not-localized", None, None, "The localizer did not name it."),
            ("node-c-pub", "active", 2, 66, None), ("node-d-pub", "active", 0, 118, None), ("node-e-pub", "excluded", None, None, "Excluded by the operator.")]
    if variant in ("partial", "failed"):
        base[2] = ("node-b-pub", "node-error", None, 390, "It answered 503 Service Unavailable.")
    if variant == "empty":
        base = [(ep, st, 0 if st == "active" else r, ms, er) for ep, st, r, ms, er in base]
    return base


def fanout(items, summary, line):
    lis = ""
    for ep, st, rows, ms, err in items:
        tone = {"active": "ok", "node-error": "bad", "time-out": "bad", "offline": "bad", "excluded": "neutral", "not-localized": "neutral", "not-resolved": "warn"}[st]
        cls = " is-excluded" if st in ("excluded", "not-localized") else ""
        if rows is not None:
            meta = f"{rows} row{'s' if rows != 1 else ''} · {ms:,} ms"
        elif ms:
            meta = f"{ms:,} ms"
        else:
            meta = "not asked"
        er = f'<span class="fui-fanout-error">{e(err)}</span>' if err else ""
        lis += (f'<li class="fui-fanout-item fui-tone-{tone}{cls}"><span class="fui-fanout-line"><code>{ep}</code>{pill(tone, STATUS_WORD[st])}{hint(st)}</span>'
                f'<span class="fui-fanout-meta">{meta}</span>{er}</li>')
    return f'<div class="fui-fanout"><p class="fui-fanout-summary">{summary}<span class="fui-muted">{line}</span></p><ul class="fui-fanout-list" aria-label="What each endpoint did">{lis}</ul></div>'


def rows_table(rows):
    body = [f'<tr><th scope="row">{when(t)}</th>' + td("Systolic", v + " mm Hg", "fui-num") + td("From", f"<code>{ep}</code>") + "</tr>" for t, v, ep in rows]
    return table("The rows", th("Recorded") + th("Systolic", num=True) + th("From", title="The endpoint each row came from, from meta.federation"), body)


def results(body, count=None, view="table", answered=True, window=None):
    dis = "" if answered else " disabled"
    seg = (f'<div role="group" aria-label="Show the answer as" class="fui-segmented">'
           f'<button type="button" class="fui-segment" aria-pressed="{"true" if view == "table" and answered else "false"}"{dis}>Table</button>'
           f'<button type="button" class="fui-segment" aria-pressed="{"true" if view == "raw" else "false"}"{dis}>Raw</button></div>')
    exp = (button("secondary", "Export CSV", "download", small=True) + button("secondary", "Export JSON", "download", small=True)) if answered and count else ""
    win = f'<p class="fui-results-window">{window}</p>' if window else ""
    head = f'<div class="fui-panel-section"><div class="fui-panel-section-head"><h2 class="fui-panel-section-title">Answer{f" <span class=mk-count>· {count}</span>" if count is not None else ""}</h2><div class="fui-actions">{seg}{exp}</div></div>{win}</div>'
    return f'<section class="fui-card fui-panel">{head}{body}</section>'


def body_part(html):
    return f'<div class="fui-panel-section">{html}</div>'


WINDOW_6 = "Rows 1 to 6 of 6, inside the fetch of 50."


def answer_complete():
    return body_part(fanout(outcomes(), pill("ok", "Complete"), "4 of 4 asked nodes answered · 412 ms · no deduplication")) + rows_table(ROWS)


def answer_partial():
    warn = notice("warn", "triangle-alert", "<strong>Incomplete answer.</strong> node-b-pub answered with an error (503), so these rows are not the whole answer. Do not read a missing row as absent.")
    return body_part(warn + fanout(outcomes("partial"), pill("warn", "Incomplete"), "4 of 5 asked nodes answered · 412 ms · best effort")) + rows_table(ROWS)


RAW = {
    "meta": {"federation": {"complete": True, "endpoints": [
        {"id": ep, "status": st, **({"row_count": r} if r is not None else {}), **({"latency_ms": ms} if ms else {})} for ep, st, r, ms, _ in outcomes()]}},
    "columns": [{"name": "recorded"}, {"name": "systolic"}],
    "rows": [[t + ":00+02:00", int(v)] for t, v, _ in ROWS],
}


def query_page(form, answer, overlay=None, toast_html=""):
    content = (page_heading("Query console", subtitle="Run AQL or a stored query across the federation as yourself; neither the console nor the browser keeps the rows.")
               + f'<div class="fui-stack">{form}{answer}</div>')
    return shell("Query console", "query", content, overlay=overlay, toast_html=toast_html)


def not_run(text="No query has run yet."):
    return results(quiet_line(text), answered=False)


def page_query(overlay=None):
    return query_page(query_form(), not_run(), overlay=overlay)


def page_query_stored():
    return query_page(query_form("stored"), not_run())


def page_query_options():
    return query_page(query_form(options_open=True, chosen=("node-a-pub", "node-c-pub"), best_effort=True), not_run())


def page_query_invalid():
    errs = {1: "Parameter 2's name holds a space: use letters, digits and underscores, such as name_space."}
    form = query_form(params=[("patient", PATIENT_PARAM), ("name space", PATIENT_NS)], errors=errs)
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 field needs a change before the query runs</p>'
               '<ul><li><a href="#pn1">Parameter 2: its name holds a space</a></li></ul></div>')
    return query_page(summary + form, not_run("Nothing was sent: fix the parameter, then run the query again."))


def page_query_running():
    sk = "".join(f'<tr aria-hidden="true"><th>{skeleton_lines([60])}</th><td>{skeleton_lines([40])}</td><td>{skeleton_lines([60])}</td></tr>' for _ in range(4))
    body = (body_part('<p class="fui-fanout-summary" role="status"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Asking the nodes the localizer names</span>'
                      '<span class="fui-muted">within the overall budget of 4,000 ms</span></p>')
            + table("The rows, loading", th("Recorded") + th("Systolic", num=True) + th("From"), [sk]))
    return query_page(query_form(running=True), results(body, answered=False))


def page_query_complete():
    return query_page(query_form(), results(answer_complete(), "6 rows", window=WINDOW_6))


def page_query_partial():
    return query_page(query_form(best_effort=True), results(answer_partial(), "6 rows", window=WINDOW_6 + " Incomplete: node-b-pub's rows are missing."))


def page_query_failed():
    body = body_part(refusal("The gateway failed the query (424): a node answered with an error, so under all or nothing it returned no rows.")
                     + fanout(outcomes("failed"), pill("bad", "Failed · 424"), "node-b-pub answered 503")
                     + f'<div class="fui-cluster">{button("secondary", "Run again with best effort", "play", small=True)}<span class="fui-small fui-muted">Best effort answers with the rows that came back, and says the answer is incomplete.</span></div>')
    return query_page(query_form(), results(body, "0 rows"))


def page_query_refused():
    bad = "SELECT c/uid/value FROM COMPOSITION c WHERE\n  c/name/value = 'Blood pressure' AND"
    form = query_form(text=bad, params=[], marks={2: "error"}, aql_error="Line 2, column 39: a condition must follow AND.")
    body = body_part(refusal("The gateway refused the query (400): it is not valid AQL. No node was asked.", issues=[("bad", "Error", "not-aql", "line 2, column 39: a condition must follow AND")]))
    return query_page(form, results(body, "0 rows"))


def page_query_raw():
    from pg_shell import json_html
    text = json.dumps(RAW, indent=2)
    body = body_part(f'<div class="fui-panel-section-head"><p class="fui-small fui-muted">The federated result set, as a client receives it.</p><div class="fui-actions">{copy_button("Copy", "the result set")}</div></div>'
                     f'<div class="fui-code-panel fui-self-description" role="region" aria-label="The federated result set as JSON" tabindex="0"><pre>{json_html(text)}</pre></div>')
    return query_page(query_form(), results(body, "6 rows", view="raw", window=WINDOW_6))


def page_query_no_rows():
    body = body_part(fanout(outcomes("empty"), pill("ok", "Complete"), "4 of 4 asked nodes answered · 398 ms")
                     + quiet_line("Every asked node answered, and none holds a blood pressure for this patient in this namespace.", "Check the parameters", "#pn1", boxed=False))
    return query_page(query_form(), results(body, "0 rows", window="0 rows, inside the fetch of 50."))


# ---- stored queries ----

def locked(word):
    return f'<span class="fui-tag fui-tag-locked">{icon("lock", "12")}{e(word)}</span>'


def stored_rows(items, group=None, locked_word=None):
    out = []
    if group:
        out.append(f'<tr class="mk-group-row"><th scope="rowgroup" colspan="3"><span class="mk-group-name"><code>{group}</code></span> {locked(locked_word)}</th></tr>')
    for name, versions, at in items:
        ns, short = name.split("::")
        out.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{routes.sq_href(short)}"><code>{e(short)}</code></a></th>'
                   + td("Versions", '<span class="mk-ff-versions">' + "".join(f'<span class="fui-tag">{v}</span>' for v in versions) + "</span>", role="meta-q")
                   + td("Newest stored", rel(at), role="meta-q") + "</tr>")
    return out


def stored_head():
    return th("Name", "name", "ascending") + th("Versions", title="Newest first") + th("Newest stored")


def eehrxf_items(names=None):
    return [(f"eu.ferrofed.eehrxf::{n}", ["1.0.0"], "2026-10-01T00:00") for n in (names if names is not None else EEHRXF)]


def stored_table(items=None, eu=None):
    rows = stored_rows(items if items is not None else STORED, "org.demo-regio", "The store · versions never change")
    eu = eehrxf_items(EEHRXF[:4]) if eu is None else eu
    if eu:
        rows += stored_rows(eu, "eu.ferrofed.eehrxf", "The gateway's own · read-only")
    return table("Stored queries", stored_head(), rows, scroll_label="Stored queries")


def stored_page(body=None, tools=None, chips="", overlay=None, top="", summ=None, foot=None, count=None):
    import tpl
    total = data.N_STORED
    show_all = f'<span class="fui-small fui-muted">Showing 4 of the {len(EEHRXF)} patient summary queries</span> {button("ghost", f"Show all {len(EEHRXF)}", small=True)}'
    if body is not None and "fui-panel-section" in body and "<table" not in body:
        cardh = tpl.card(body, label="Stored queries", count=count)
    else:
        cardh = tpl.card(body if body is not None else stored_table(), foot if foot is not None else show_all, label="Stored queries", count=str(total) if count is None else count)
    summary_html = summ if summ is not None else tpl.summary([(f"{total} stored queries", None, True), (f"{len(STORED)} in org.demo-regio", None, False), (f"{len(EEHRXF)} the gateway's own", None, False)])
    content = (page_heading("Stored queries", actions=link_button("primary", "New stored query version", "stored-new.html", "plus"), subtitle="The queries the gateway holds; a stored version never changes.")
               + top + tpl.list_body(summary_html, tools if tools is not None else stored_tools(), chips, cardh))
    return shell("Stored queries", "stored", content, overlay=overlay)


def stored_tools(text=""):
    import tpl
    return tpl.toolbar("A namespace or a name", text)


def page_stored():
    return stored_page()


def page_stored_filtered():
    import tpl
    q = "me"
    mine = [s_ for s_ in STORED if q in s_[0].split("::")[1]]
    eu = eehrxf_items([n for n in EEHRXF if q in n])
    k = len(mine) + len(eu)
    return stored_page(body=stored_table(mine, eu), tools=stored_tools(q), chips=tpl.active([f"Name: {q}"], shown=(k, data.N_STORED)), foot="", count=f"showing {k} of {data.N_STORED}")


def page_stored_no_match():
    import tpl
    body = quiet_line("No stored query has “vaccin” in its name. The patient summary's immunisations query is patient-summary-immunisations.", "Clear the filter", "stored-queries.html")
    return stored_page(body=body, tools=stored_tools("vaccin"), chips=tpl.active(["Name: vaccin"], shown=(0, data.N_STORED)), count=f"showing 0 of {data.N_STORED}")


def page_stored_loading():
    rows = "".join(f'<tr aria-hidden="true"><th data-role="title">{skeleton_lines([60])}</th><td data-role="meta-q">{skeleton_lines([50])}</td><td data-role="meta-q">{skeleton_lines([40])}</td></tr>' for _ in range(6))
    import tpl
    sk = '<div class="fui-chip-checks fui-summary-chips" aria-hidden="true">' + "".join(f'<span class="fui-chip-check">{skeleton_lines([30])}</span>' for _ in range(3)) + "</div>"
    return stored_page(body=table("Stored queries, loading", stored_head(), [rows]), foot="", count="", summ=sk)


# ---- one stored query (T2): AQL, Versions and Endpoints holding it ----

SQ_HOLD = [(ep["id"], ep["id"] != "node-b-pub") for ep in data.IN_SERVICE]


def stored_query_page(tab="aql", version="1.1.0", panel=None, overlay=None, top="", short="vital_signs"):
    from common import action_group
    s_ = next((x for x in STORED if x[0].endswith("::" + short)), None) or (f"eu.ferrofed.eehrxf::{short}", ["1.0.0"], "2026-10-01T00:00")
    name, versions, at = s_
    reserved = name.startswith("eu.")
    own = short == "vital_signs"
    items = [] if reserved else [menu_item("Send again…", "send", href="stored-query-repair.html" if own else None, desc="To the endpoints that lost it"),
                                 menu_item("New version", "plus", href="stored-new.html")]
    ag = action_group(link_button("primary", "Open in query console", "query-stored.html", "square-terminal"), items, label=f"More actions for {short}")
    head = page_heading(short, actions=ag, mono=True,
                        subtitle=f"Version {version} of <code>{name}</code>, stored {when(at)}" + (" · the gateway's own, read-only" if reserved else ""),
                        crumbs=[("Stored queries", "stored-queries.html")])
    holding = sum(1 for _, h in SQ_HOLD if h)
    t = tabs(short, [("aql", "AQL", routes.sq_href(short)), ("versions", "Versions", routes.sq_href(short, "versions"), len(versions)),
                     ("endpoints", "Endpoints holding it", routes.sq_href(short, "endpoints"), holding if own else len(SQ_HOLD))], tab)
    if panel is None:
        panel = sq_aql_panel(short, version)
    content = head + f'<div class="fui-stack">{top}' + tabbed(t, panel) + "</div>"
    return shell(short, "stored", content, overlay=overlay)


ARCHETYPE = {"allergies": "EVALUATION.adverse_reaction_risk.v1", "encounters_last_year": "COMPOSITION.encounter.v1", "lab_results_recent": "OBSERVATION.laboratory_test_result.v1",
             "medication_active": "INSTRUCTION.medication_order.v3", "problem_list": "EVALUATION.problem_diagnosis.v1"}


def aql_for(short):
    if short == "vital_signs":
        return AQL_VITALS
    arch = ARCHETYPE.get(short, "COMPOSITION.health_summary.v1")
    kind = arch.split(".")[0]
    alias = "x" if kind != "COMPOSITION" else "c"
    contains = f"  CONTAINS COMPOSITION c\n  CONTAINS {kind} x[openEHR-EHR-{arch}]" if kind != "COMPOSITION" else f"  CONTAINS COMPOSITION c[openEHR-EHR-{arch}]"
    return (f"SELECT\n  c/context/start_time/value AS recorded,\n  {alias}/uid/value AS uid\nFROM EHR e\n{contains}\n"
            "WHERE e/ehr_status/subject/external_ref/id/value = $patient\nORDER BY c/context/start_time/value DESC")


def sq_aql_panel(short, version):
    text = aql_for(short)
    params = ", ".join(f"<code>${n}</code>" for n in sorted(set(re.findall(r"\$(\w+)", text))))
    return sections(section(f"AQL, version {version}", code_panel(text) + f'<p class="fui-small fui-muted">Parameters: {params}</p>', copy_button("Copy", "the AQL")))


def sq_versions_panel(short):
    s_ = next((x for x in STORED if x[0].endswith("::" + short)), None)
    versions = s_[1] if s_ else ["1.0.0"]
    dates = {"1.1.0": "2026-10-14T10:20", "1.0.0": "2026-09-02T15:30"}
    rows = [f'<tr><th scope="row"><a href="{routes.sq_href(short) if i == 0 else routes.href("sq", short + "-v" + v)}"><code>{v}</code></a>{" <span class=\"fui-badge fui-tone-info\">Newest</span>" if i == 0 else ""}</th>'
            + td("Stored", when(dates.get(v, s_[2] if s_ else "2026-10-01T00:00"))) + "</tr>" for i, v in enumerate(versions)]
    return sections(section("Versions", '<p class="fui-small fui-muted">Newest first. A stored version never changes; a correction is a new version.</p>', count=len(versions))
                    + table("Versions", th("Version") + th("Stored"), rows))


def sq_endpoints_panel(short):
    own = short == "vital_signs"
    rows = [f'<tr><th scope="row"><code>{ep}</code></th>' + td("Holds it", ("Yes, the same text" if (h or not own) else '<span class="fui-badge fui-tone-warn">Missing</span>')) + "</tr>" for ep, h in SQ_HOLD]
    k = sum(1 for _, h in SQ_HOLD if h or not own)
    line = f"{k} of {len(SQ_HOLD)} endpoints in service" + (" · node-b-pub is missing it" if own else "") + f", checked {rel('2026-10-21T08:00')}"
    act = link_button("secondary", "Send again…", "stored-query-repair.html", "send", small=True) if own else ""
    return sections(section("Endpoints holding it", f'<p class="fui-small fui-muted">{line}.</p>', act) + table("Endpoints", th("Endpoint") + th("Holds it"), rows))


def page_stored_query():
    return stored_query_page()


def page_stored_query_missing():
    top = notice("warn", "circle-alert", f'<strong>Version 1.2.0 is not held</strong> (404{hint("stored-query-unknown")}). The newest version is 1.1.0.',
                 action=" " + link_button("secondary", "Open 1.1.0", "stored-query.html", small=True))
    return stored_query_page(panel=quiet_line("Nothing to show for 1.2.0.", "Open the newest version, 1.1.0", "stored-query.html"), top=top, version="1.2.0")


def repair_dialog(busy=False):
    members = "".join(f'<label class="fui-chip-check"><input type="checkbox"{" checked" if ep["id"] == "node-b-pub" else ""} />{ep["id"]}</label>' for ep in data.IN_SERVICE)
    go = busy_button("primary", "Sending") if busy else button("primary", "Send the version again", "send")
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="rp-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="rp-title" class="fui-dialog-title">{icon("send", "18")} Send vital_signs again</h2>'
        '<p>Sends the registry\'s copy of one version to the endpoints you choose, for a node that lost it or holds another text. The registry\'s copy stays as it is.</p>'
        '<div class="fui-modal-body fui-stack"><div class="fui-columns">'
        + '<div class="fui-field"><span class="fui-label">Stored query</span><p><code>org.demo-regio::vital_signs</code></p></div>'
        + field("rp-version", "Version", select("rp-version", [("1.1.0", "1.1.0, the newest"), ("1.0.0", "1.0.0")], "1.1.0"))
        + "</div>"
        f'<div class="fui-field"><span class="fui-label" id="rp-m">Endpoints</span><div class="fui-chip-checks" role="group" aria-labelledby="rp-m">'
        f'<label class="fui-chip-check"><input type="checkbox" />Every endpoint in service</label>{members}</div>'
        '<p class="fui-help">It goes to replica gateway-0; the replicas share one stored-query store.</p></div>'
        + field("rp-reason", "Reason", textarea("rp-reason", "node-b restored from backup without its stored queries"), help_="Required. Recorded with your name.")
        + f'</div><div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{go}</div></div>'
    )


def page_stored_repair():
    return stored_query_page(overlay=repair_dialog())


def page_stored_repaired():
    top = (f'<div class="fui-notice fui-tone-warn" role="status" tabindex="-1">{icon("triangle-alert", "16")}<div><strong>gateway-0 sent vital_signs 1.1.0 again; node-b-pub did not take it.</strong>'
           '<ul class="fui-bullets fui-small"><li><code>node-b-pub</code>: 503 Service Unavailable; node-b is degraded. Send it again once node-b answers.</li></ul>'
           '<p class="fui-small fui-muted">The registry\'s copy is unchanged. Recorded with your reason.</p>'
           f'<div class="fui-actions">{button("secondary", "Send again to node-b-pub…", "send", small=True)}</div></div></div>')
    return stored_query_page(top=top)


# ---- a new stored query version ----

BODY_WEIGHT = ("SELECT\n  c/context/start_time/value AS recorded,\n  o/data[at0002]/events[at0003]/data[at0001]/items[at0004]/value/magnitude AS weight\n"
               "FROM EHR e\n  CONTAINS COMPOSITION c\n  CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.body_weight.v2]\nWHERE e/ehr_status/subject/external_ref/id/value = $patient")


def stored_new_page(overlay_=None, name="org.demo-regio::body_weight", version="1.0.0", aql=BODY_WEIGHT, name_err=None, version_err=None, aql_err=None, top="", readonly=False, marks=None):
    def inp(id_, v, err):
        inv = ' aria-invalid="true"' if err else ""
        dis = " disabled" if readonly else ""
        return input_(id_, v, extra='spellcheck="false" class="fui-input fui-mono"' + inv + dis)
    lines = len(aql.split("\n"))
    names = sorted(set(re.findall(r"\$(\w+)", aql)))
    bar = f'<div class="fui-editor-bar"><span>AQL · {lines} lines{" · names " + ", ".join("$" + n for n in names) if names else ""}</span></div>'
    aql_help = f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {aql_err}</p>' if aql_err else '<p class="fui-help">Checked as the gateway parses it. Name a patient as a parameter, never in the text.</p>'
    form = sections(section("The version", '<div class="fui-columns">'
                            + field("sn-name", "Qualified name", inp("sn-name", name, name_err), help_="A namespace and a name, such as org.demo-regio::vital_signs.", error=name_err)
                            + field("sn-version", "Version", inp("sn-version", version, version_err), help_="major.minor.patch.", error=version_err)
                            + "</div>"
                            + f'<div class="fui-field"><label for="sn-aql" class="fui-label">AQL</label>{bar}{editor(aql, "sn-aql", readonly=readonly, marks=marks, invalid=bool(aql_err))}{aql_help}</div>'))
    content = (page_heading("New stored query version", subtitle="Stored at once, for every replica; it is data, not configuration, and can never be changed or removed.",
                            crumbs=[("Stored queries", "stored-queries.html")]) + f'<div class="fui-stack">{top}{form}</div>')
    store = button("primary", "Store the version…", "database", attrs="disabled" if (readonly or name_err or version_err or aql_err) else "")
    dk = dock(dock_field("sn-reason", "Reason", "Body weight for the dietetics pilot, ticket OPS-131", "Required: recorded with the version", disabled=readonly),
              link_button("secondary", "Cancel", "stored-queries.html") + store)
    return shell("New stored query version", "stored", content, dock_html=dk, overlay=overlay_)


def page_stored_new():
    return stored_new_page()


def page_stored_new_confirm():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="sn-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="sn-title" class="fui-dialog-title">{icon("database", "18")} Store body_weight 1.0.0?</h2>'
        '<p>Once stored it stays as it is: a correction is version 1.0.1.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Go back", keys="Esc")}{button("primary", "Store the version", "database")}</div></div>'
    )
    return stored_new_page(dialog)


def page_stored_new_held():
    return stored_new_page(name="org.demo-regio::vital_signs", version="1.1.0", aql=AQL_VITALS,
                           version_err=f"1.1.0 is held already (409{hint('stored-query-held')}): a stored version never changes. Use 1.1.1, the next patch.")


def page_stored_new_reserved():
    return stored_new_page(name="eu.ferrofed.eehrxf::body_weight",
                           name_err=f"The namespace eu.ferrofed.eehrxf is the gateway's own (409{hint('stored-query-reserved')}). Store it under a namespace of your own, such as org.demo-regio.")


def page_stored_new_literal():
    aql = BODY_WEIGHT.replace("= $patient", "= 'ffd-demo-0001'")
    return stored_new_page(aql=aql, marks={7: "error"},
                           aql_err=f"Line 7 names a patient by a literal identifier (400{hint('subject-literal')}). Name it through a parameter, such as $patient; the registry never stores an identifier.")


def page_stored_new_not_aql():
    aql = BODY_WEIGHT.replace("FROM EHR e", "FRM EHR e")
    return stored_new_page(aql=aql, marks={4: "error"}, aql_err=f"Line 4 is not valid AQL (400{hint('not-aql')}): FROM is spelled FRM.")


def page_stored_new_read_only():
    top = notice("info", "lock", f'<strong>This gateway reads its stored queries from files.</strong> It stores no new version from the console (405{hint("stored-query-read-only")}); its operator adds a file and restarts it.')
    return stored_new_page(top=top, readonly=True)


# ---- route-only pages: every stored query's tabs ----

def route_pages():
    out = {}
    shorts = [n.split("::")[1] for n, _, _ in STORED] + list(EEHRXF)
    for sh in shorts:
        if sh != "vital_signs":
            out[routes.slug("sq", sh)] = lambda sh=sh: stored_query_page(short=sh, version=next((x[1][0] for x in STORED if x[0].endswith("::" + sh)), "1.0.0"))
        out[routes.slug("sq", sh, "versions")] = lambda sh=sh: stored_query_page("versions", short=sh, panel=sq_versions_panel(sh))
        out[routes.slug("sq", sh, "endpoints")] = lambda sh=sh: stored_query_page("endpoints", short=sh, panel=sq_endpoints_panel(sh))
    for name, versions, _ in STORED:
        sh = name.split("::")[1]
        for v in versions[1:]:
            out[routes.slug("sq", sh + "-v" + v)] = lambda sh=sh, v=v: stored_query_page(short=sh, version=v)
    return out
