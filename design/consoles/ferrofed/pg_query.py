# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The query console and the stored queries: the editor with its parameters
and federation options, every answer the gateway can give, and the stored
queries with their drift repair."""
import re

from common import (
    badge, busy_button, button, card, check, chip, definitions, disclosure, e, empty_quiet, empty_state, field,
    icon, icon_button, input_, kbd, link_button, menu, menu_item, notice, page_heading, pager, pill, refusal, select,
    shell, status, table, tabbed, tabs, tag, td, textarea, th, titled_panel, toast, toasts, when, copy_button,
)
from data import AQL_VITALS, EEHRXF, MEMBERS, PATIENT_NS, PATIENT_PARAM, STORED
from parts import filter_chips, fold_actions, skeleton_lines
from pg_shell import replica_line

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
        out += (f'<div class="fui-repeater-row"><div class="fui-field"><label class="fui-sr-only" for="pn{i}">Name of parameter {i + 1}</label>{input_(f"pn{i}", n, extra="spellcheck=\"false\" class=\"fui-input fui-mono\"" + inv)}</div>'
                f'<div class="fui-field"><label class="fui-sr-only" for="pv{i}">Value of parameter {i + 1}</label>{input_(f"pv{i}", v, extra="spellcheck=\"false\" class=\"fui-input fui-mono\"")}</div>'
                f'{icon_button("x", f"Remove parameter {i + 1}")}</div>')
        if er:
            out += f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {er}</p>'
    return f'<div class="fui-repeater">{out}</div>'


def options_block(open_=True, chosen=(), dedup="default", best_effort=False):
    eps = "".join(f'<label class="fui-chip-check"><input type="checkbox"{" checked" if m[6][0][0] in chosen else ""} />{m[6][0][0]}</label>' for m in MEMBERS if m[7] == "active")
    orgs = "".join(f'<label class="fui-chip-check"><input type="checkbox" />{m[1]}</label>' for m in MEMBERS if m[7] == "active")
    body = (
        f'<div class="fui-field"><span class="fui-label" id="o-eps">Endpoints</span><div class="fui-chip-checks" role="group" aria-labelledby="o-eps">{eps}</div>'
        '<p class="fui-help">Leave all clear to let the gateway choose. Sent as <code>openEHR-federation-endpoint</code>.</p></div>'
        f'<div class="fui-field"><span class="fui-label" id="o-orgs">Organisations</span><div class="fui-chip-checks" role="group" aria-labelledby="o-orgs">{orgs}</div></div>'
        + field("o-dedup", "Deduplication", select("o-dedup", [("default", "The gateway's default (none)"), ("none", "none"), ("version-identity", "version-identity")], dedup))
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
            '<div class="fui-stack">'
            + '<div class="fui-columns">'
            + field("sq-name", "Stored query", select("sq-name", [(n, n) for n, _, _ in STORED], "org.demo-regio::vital_signs"))
            + field("sq-version", "Version <span class=\"fui-muted\">(optional)</span>", select("sq-version", [("", "The newest, 1.1.0"), ("1.1.0", "1.1.0"), ("1.0.0", "1.0.0")], ""))
            + "</div>"
            + f'<div class="fui-field"><span class="fui-label">Its AQL, version 1.1.0 <span class="fui-muted">· read-only</span></span>{editor(AQL_VITALS, "sq-aql", readonly=True)}</div>'
            + "</div>"
        )
    right = (
        '<div class="fui-stack">'
        f'<div class="fui-field"><span class="fui-label">Parameters</span>{param_rows(params, errors)}'
        f'<div>{button("secondary", "Add parameter", "plus", small=True)}</div>'
        '<p class="fui-help">Each value is sent as a string, in the request body: never in a URL.</p></div>'
        '<div class="fui-columns">'
        + field("offset", "Offset", input_("offset", offset, extra='inputmode="numeric"'), help_="Rows to skip.")
        + field("fetch", "Fetch", input_("fetch", fetch, extra='inputmode="numeric"'), help_="Rows to return.")
        + "</div>" + options_block(options_open, chosen, best_effort=best_effort) + "</div>"
    )
    run = busy_button("primary", "Running") if running else button("primary", "Run query", "play", keys="Ctrl Enter", type_="submit")
    form = (f'<form class="fui-form" method="post" action="#"><div class="mk-ff-split">{left}{right}</div>'
            f'<div class="fui-run-row">{run}<p>Runs as you, through the gateway\'s own query surface, exactly as a client sends it.</p></div></form>')
    return tabbed(t, form)


# ---- the answer ----

ROWS = [
    ("2026-10-18T09:12", "128", "node-a-pub"), ("2026-10-11T14:40", "131", "node-c-pub"), ("2026-10-04T08:55", "126", "node-a-pub"),
    ("2026-09-27T10:05", "135", "node-d-pub"), ("2026-09-20T09:30", "129", "node-a-pub"), ("2026-09-13T11:15", "133", "node-c-pub"),
]


def fanout(outcomes, summary, line):
    items = ""
    for ep, st, rows, ms, err in outcomes:
        tone = {"active": "ok", "node-error": "bad", "timeout": "bad", "excluded": "neutral", "not-resolved": "warn"}[st]
        cls = " is-excluded" if st == "excluded" else ""
        meta = (f'{rows} row' if rows == 1 else f'{rows} rows') + f' · {ms:,} ms' if rows is not None else (f"no rows · {ms:,} ms" if ms else "not asked")
        er = f'<span class="fui-fanout-error">{e(err)}</span>' if err else ""
        items += (f'<li class="fui-fanout-item fui-tone-{tone}{cls}"><span class="fui-fanout-line"><code>{ep}</code>{pill(tone, st)}</span>'
                  f'<span class="fui-fanout-meta">{meta}</span>{er}</li>')
    return f'<div class="fui-fanout"><p class="fui-fanout-summary">{summary}<span class="fui-muted">{line}</span></p><ul class="fui-fanout-list" aria-label="What each endpoint did">{items}</ul></div>'


def rows_table(rows):
    body = [f'<tr><th scope="row">{when(t)}</th>' + td("systolic", v, "fui-num") + td("From", f"<code>{ep}</code>") + "</tr>" for t, v, ep in rows]
    return table("The rows", th("recorded") + th("systolic", num=True) + th("From", title="The endpoint each row came from, from meta.federation"), body)


def results_card(body, view="table"):
    seg = (f'<div role="group" aria-label="Show the answer as" class="fui-segmented">'
           f'<button type="button" class="fui-segment" aria-pressed="{"true" if view == "table" else "false"}">Table</button>'
           f'<button type="button" class="fui-segment" aria-pressed="{"true" if view == "raw" else "false"}">Raw</button></div>')
    return titled_panel("Answer", seg, f'<div class="fui-panel-body fui-stack">{body}</div>')


OUT_ALL = [("node-a-pub", "active", 3, 41, None), ("node-b-pub", "active", 0, 388, None), ("node-c-pub", "active", 2, 66, None), ("node-d-pub", "active", 1, 118, None)]
OUT_PARTIAL = [("node-a-pub", "active", 3, 41, None), ("node-b-pub", "node-error", None, 2410, "The node answered 503 Service Unavailable."), ("node-c-pub", "active", 2, 66, None), ("node-d-pub", "active", 1, 118, None)]


def answer_complete():
    f = fanout(OUT_ALL, f'<strong>6 rows</strong>{pill("ok", "complete")}', "4 of 4 nodes answered · 412 ms · deduplication none")
    return f + rows_table(ROWS)


def answer_partial():
    f = fanout(OUT_PARTIAL, f'<strong>6 rows</strong>{pill("warn", "incomplete")}', "3 of 4 nodes answered · 2,412 ms · best effort")
    warn = notice("warn", "triangle-alert", "<strong>Incomplete answer.</strong> node-b-pub did not answer, so these rows are not the whole answer (<code>meta.federation.complete</code> is false). Do not read a missing row as absent.")
    return warn + f + rows_table(ROWS)


RAW = """{
  "meta": {
    "federation": {
      "complete": true,
      "endpoints": [
        { "id": "node-a-pub", "status": "active", "rows": 3, "latency_ms": 41 },
        { "id": "node-b-pub", "status": "active", "rows": 0, "latency_ms": 388 },
        { "id": "node-c-pub", "status": "active", "rows": 2, "latency_ms": 66 },
        { "id": "node-d-pub", "status": "active", "rows": 1, "latency_ms": 118 }
      ]
    }
  },
  "columns": [ { "name": "recorded" }, { "name": "systolic" } ],
  "rows": [
    [ "2026-10-18T09:12:00+02:00", 128 ],
    [ "2026-10-11T14:40:00+02:00", 131 ],
    …
  ]
}"""


def query_page(form, answer, overlay=None, toast_html=""):
    content = (page_heading("Query console", subtitle="Run AQL or a stored query across the federation, as yourself. The rows are shown here and kept by neither the console nor the browser.")
               + f'<div class="fui-stack">{form}{answer}</div>')
    return shell("Query console", "query", content, overlay=overlay, toast_html=toast_html)


def page_query(overlay=None):
    ans = results_card(empty_quiet("square-terminal", "No query has run yet", "Write AQL or choose a stored query, name the patient as a parameter, then run it."))
    return query_page(query_form(), ans, overlay=overlay)


def page_query_stored():
    ans = results_card(empty_quiet("square-terminal", "No query has run yet", "Choose a stored query and its version, then run it."))
    return query_page(query_form("stored"), ans)


def page_query_options():
    ans = results_card(empty_quiet("square-terminal", "No query has run yet", "Write AQL or choose a stored query, name the patient as a parameter, then run it."))
    return query_page(query_form(options_open=True, chosen=("node-a-pub", "node-c-pub"), best_effort=True), ans)


def page_query_invalid():
    errs = {1: "Parameter 2 has no name of letters, digits and underscores: “name space” holds a space."}
    form = query_form(params=[("patient", PATIENT_PARAM), ("name space", PATIENT_NS)], errors=errs)
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 field needs a change before the query runs</p>'
               '<ul><li><a href="#pn1">Parameter 2: a name of letters, digits and underscores</a></li></ul></div>')
    ans = results_card(empty_quiet("square-terminal", "Nothing was sent", "The gateway was not asked: fix the parameter, then run the query again."))
    return query_page(summary + form, ans)


def page_query_running():
    sk = "".join(f'<tr aria-hidden="true"><th>{skeleton_lines([60])}</th><td>{skeleton_lines([40])}</td><td>{skeleton_lines([60])}</td></tr>' for _ in range(4))
    body = ('<p class="fui-fanout-summary" role="status"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Asking 4 nodes</span>'
            '<span class="fui-muted">within the overall budget of 4,000 ms</span></p>'
            + table("The rows, loading", th("recorded") + th("systolic", num=True) + th("From"), [sk]))
    return query_page(query_form(running=True), results_card(body))


def page_query_complete():
    return query_page(query_form(), results_card(answer_complete()))


def page_query_partial():
    return query_page(query_form(best_effort=True, chosen=()), results_card(answer_partial()))


def page_query_failed():
    f = fanout(OUT_PARTIAL, f'<strong>No rows</strong>{pill("bad", "504 · failed")}', "the gateway answers all or nothing, and one node did not answer")
    body = refusal("The gateway failed the query: 504. A node in scope did not answer, so it returned no rows.") + f + (
        f'<div class="fui-cluster">{button("secondary", "Run again with best effort", "play", small=True)}<span class="fui-small fui-muted">Best effort answers with the rows that came back, and says the answer is incomplete.</span></div>')
    return query_page(query_form(), results_card(body))


def page_query_refused():
    bad = "SELECT c/uid/value FROM COMPOSITION c WHERE\n  c/name/value = 'Blood pressure' AND"
    form = query_form(text=bad, params=[], marks={2: "error"}, aql_error="Line 2: the condition after AND is missing.")
    body = refusal("The gateway refused the query: 400 (not-aql).", issues=[("bad", "error", "not-aql", "line 2, column 40: expected a condition after AND")]) + '<p class="mk-refusal-line">No node was asked. Fix the AQL and run it again.</p>'
    return query_page(form, results_card(body))


def page_query_raw():
    body = (f'<div class="fui-code-panel" role="region" aria-label="The federated result set as JSON" tabindex="0"><pre>{e(RAW)}</pre></div>'
            f'<div class="fui-actions">{copy_button("Copy", "the result set")}</div>')
    return query_page(query_form(), results_card(body, view="raw"))


def page_query_no_rows():
    body = fanout(OUT_ALL[:1] + [(ep, "active", 0, ms, None) for ep, _, _, ms, _ in OUT_ALL[1:]], f'<strong>No rows</strong>{pill("ok", "complete")}', "4 of 4 nodes answered · 398 ms") + empty_state(
        "search-x", "Every node answered, and none holds a matching row",
        "The answer is complete, so no node holds a blood pressure for this patient in this namespace. Check the namespace parameter if you expected one.",
        button("secondary", "Edit the parameters", "pencil"), filtered=True)
    body = body.replace("<strong>No rows</strong>", "<strong>No rows</strong>", 1)
    return query_page(query_form(), results_card(body.replace("(\"node-a-pub\", \"active\", 3", "")))


# ---- stored queries ----

def stored_rows(items, group=None, locked_word=None):
    import tpl
    out = []
    if group:
        out.append(f'<tr class="mk-group-row"><th scope="rowgroup" colspan="3"><span class="mk-group-name"><code>{group}</code></span> {locked(locked_word)}</th></tr>')
    for name, versions, at in items:
        ns, short = name.split("::")
        out.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="stored-query.html"><code>{e(short)}</code></a></th>'
                   + td("Versions", '<span class="mk-ff-versions">' + "".join(f'<span class="fui-tag">{v}</span>' for v in versions) + "</span>", role="sub")
                   + td("Newest stored", when(at), role="meta-q") + "</tr>")
    return out


def stored_head():
    return th("Name", "name", "ascending") + th("Versions", title="Newest first") + th("Newest stored")


def locked(word):
    return f'<span class="fui-tag fui-tag-locked">{icon("lock", "12")}{e(word)}</span>'


def stored_table(items=None, eu=True):
    rows = stored_rows(items if items is not None else STORED, "org.demo-regio", "Store · immutable versions")
    if eu:
        rows += stored_rows([(f"eu.ferrofed.eehrxf::{n}", ["1.0.0"], "2026-10-01T00:00") for n in EEHRXF[:4]], "eu.ferrofed.eehrxf", "Gateway · reserved")
    return table("Stored queries", stored_head(), rows, scroll_label="Stored queries")


def stored_page(body=None, tools=None, chips="", overlay=None, top="", toast_html="", summ=None, foot=None):
    import tpl
    cardh = body if body is not None and "fui-empty" in body else tpl.card(body if body is not None else stored_table(),
                                                                          foot if foot is not None else '<span class="fui-small fui-muted">Showing 4 of the 11 reserved queries · <a href="#">show all</a></span>', label="Stored queries", count="17")
    summary_html = summ if summ is not None else tpl.summary([("17 stored queries", None, True), ("6 in org.demo-regio", None, False), ("11 reserved", None, False)])
    content = (page_heading("Stored queries", actions=link_button("primary", "Store a new version", "stored-new.html", "plus"), subtitle="The queries the gateway holds; a stored version never changes.")
               + top + tpl.list_body(summary_html, tools if tools is not None else stored_tools(), chips, cardh))
    return shell("Stored queries", "stored", content, overlay=overlay, toast_html=toast_html)


def stored_tools(text="", ns=None):
    import tpl
    return tpl.toolbar("A namespace or a name", text, [tpl.filter_button("Namespace", ns)])


def page_stored():
    return stored_page()


def page_stored_filtered():
    import tpl
    items = [s_ for s_ in STORED if "lab" in s_[0] or "medication" in s_[0]]
    return stored_page(body=stored_table(items, eu=False), tools=stored_tools("me"), chips=tpl.active(["Name: me"]), foot="")


def page_stored_no_match():
    import tpl
    body = empty_state("search-x", "No stored query matches “vaccin”", f"The gateway holds {len(STORED) + len(EEHRXF)} stored queries, but no name contains “vaccin”. The patient summary's immunisations are under eu.ferrofed.eehrxf.",
                       button("primary", "Clear the filter", "x"), filtered=True)
    return stored_page(body=body, tools=stored_tools("vaccin"), chips=tpl.active(["Name: vaccin"]))


def page_stored_loading():
    rows = "".join(f'<tr aria-hidden="true"><th data-role="title">{skeleton_lines([60])}</th><td data-role="sub">{skeleton_lines([50])}</td><td data-role="meta-q">{skeleton_lines([60])}</td></tr>' for _ in range(6))
    return stored_page(body=table("Stored queries, loading", stored_head(), [rows]), foot="")


def repair_dialog(busy=False):
    members = "".join(f'<label class="fui-chip-check"><input type="checkbox"{" checked" if m[0] in ("node-b",) else ""} />{m[6][0][0]}</label>' for m in MEMBERS if m[7] == "active")
    go = busy_button("primary", "Sending") if busy else button("primary", "Send the version again", "send")
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="rp-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="rp-title" class="fui-dialog-title">{icon("send", "18")} Repair stored-query drift</h2>'
        '<p>Sends the registry\'s held copy of one version to the endpoints you name, for a node that lost it or holds a different text. The registry\'s copy stays as it is.</p>'
        '<div class="fui-modal-body fui-stack"><div class="fui-columns">'
        + field("rp-name", "Stored query", select("rp-name", [("v", "org.demo-regio::vital_signs")], "v", extra="disabled"))
        + field("rp-version", "Version", select("rp-version", [("1.1.0", "1.1.0"), ("1.0.0", "1.0.0")], "1.1.0"))
        + "</div>"
        f'<div class="fui-field"><span class="fui-label" id="rp-m">Endpoints</span><div class="fui-chip-checks" role="group" aria-labelledby="rp-m">'
        f'<label class="fui-chip-check"><input type="checkbox" />Every endpoint in service</label>{members}</div>'
        '<p class="fui-help">Sent as <code>openEHR-federation-endpoint</code>. It goes to replica gateway-0, the first one configured: the replicas share one stored-query store.</p></div>'
        + field("rp-reason", "Reason", textarea("rp-reason", "node-b restored from backup without its stored queries"), help_="Required. Recorded with your name.")
        + f'</div><div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{go}</div></div>'
    )


def page_stored_repair():
    return stored_query_page(overlay=repair_dialog())


def page_stored_repaired():
    top = (f'<div class="fui-notice fui-tone-warn" role="status" tabindex="-1">{icon("triangle-alert", "16")}<div><strong>Replica gateway-0 sent vital_signs 1.1.0 again: not every endpoint took it.</strong>'
           '<ul class="fui-bullets fui-small"><li><code>node-b-pub</code>: 503 Service Unavailable; the node is failing. Send it again once node-b is up.</li></ul>'
           '<p class="fui-small fui-muted">The registry\'s copy: held, unchanged. Recorded with your reason.</p>'
           f'<div class="fui-actions">{button("secondary", "Send again to node-b-pub…", "send", small=True)}</div></div></div>')
    return stored_query_page(top=top)


def stored_query_page(version="1.1.0", missing=None, overlay=None, menu_open=False, top=""):
    from common import action_group
    ag = action_group(link_button("primary", "Open in query console", "query-stored.html", "square-terminal"),
                      [menu_item("Send again…", "send", href="stored-query-repair.html", desc="To the endpoints that lost it"), menu_item("Store the next version", "plus", href="stored-new.html")],
                      open_=menu_open, label="More actions for vital_signs")
    head = page_heading("vital_signs", actions=ag, mono=True,
                        subtitle=f"<code>org.demo-regio::vital_signs</code> · version {version}, stored {when('2026-10-14T10:20')} · a held version never changes.",
                        crumbs=[("Stored queries", "stored-queries.html")])
    note = notice("info", "info", f"The gateway holds no version {missing} of this query. The newest, 1.1.0, is shown.") if missing else ""
    aql = card("AQL", code_panel(AQL_VITALS), actions=copy_button("Copy", "the AQL"))
    vlist = ('<ul class="mk-ff-version-list">'
             f'<li><a href="#" aria-current="true">1.1.0<span class="fui-muted">{when("2026-10-14T10:20")}</span></a></li>'
             f'<li><a href="#">1.0.0<span class="fui-muted">{when("2026-09-02T15:30")}</span></a></li></ul>')
    aside = card("Versions <span class=\"mk-count\">· newest first</span>", vlist) + card("Details", definitions([
        ("Namespace", "<code>org.demo-regio</code>"), ("Name", "<code>vital_signs</code>"), ("Parameters", "<code>$patient</code>, <code>$namespace</code>"),
        ("Held in", locked("Store · immutable")), ("Endpoints holding it", "6 of 6 in service · node-b-pub checked " + when("2026-10-21T08:00"))]))
    t = tabs("vital_signs", [("aql", "AQL", "stored-query.html"), ("versions", "Versions", "stored-query.html", 2), ("endpoints", "Endpoints holding it", "stored-query.html", 6)], "aql")
    content = head + f'<div class="fui-stack">{top}{note}' + tabbed(t, f'<div class="fui-main-aside"><div class="fui-main-part">{aql}</div><div class="fui-aside-part">{aside}</div></div>') + "</div>"
    return shell("vital_signs", "stored", content, overlay=overlay)


def page_stored_query():
    return stored_query_page()


def page_stored_query_missing():
    return stored_query_page(missing="1.2.0")
