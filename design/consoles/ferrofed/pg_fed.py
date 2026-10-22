# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The federation views: integrity with its admin action, and the
self-description."""
import json

from common import (
    MENU_SEP, busy_button, button, e, field, hint, icon, icon_button, input_,
    link_button, menu, menu_item, page_heading, quiet_line, rel, section, select, shell, url_code,
    table, tabs, td, th,
)
import data
from data import (EHR_A, FEDERATION, INCIDENT_KINDS, INCIDENT_WORD, INCIDENTS_OPEN, INCIDENTS_RESOLVED, JWKS_URI,
                  SPEC_URL)
import routes
import tpl


# ---- integrity ----

def incident_menu(kind, claim, open_=False):
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
               f'aria-label="Actions for the incident" title="More actions">{icon("ellipsis", "16")}</button>')
    items = []
    if kind == "EhrIdCollision":
        items = [menu_item("Clear the held EHR id…", "eraser", href="integrity-clear.html", focus=True), MENU_SEP]
    items += [menu_item(f"Open {c}", "arrow-right", href=routes.node_href(c)) for c in claim]
    m = menu(items, label="Actions for the incident", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def nodes_cell(claim):
    return ", ".join(f'<code>{c}</code>' for c in claim)


def incidents_table(items, resolved=False, menu_open=None):
    rows = []
    for i, (kind, at, ehr, claim, desc, res) in enumerate(items):
        ehr_html = f'<code class="mk-nowrap">{ehr}</code>' if ehr else '<span class="mk-dash">–</span>'
        act = "" if resolved else incident_menu(kind, claim, menu_open == i)
        note = f'<span class="fui-small fui-muted mk-incident-line">{e(desc)}{(" " + e(res) + ".") if res else ""}</span>'
        rows.append(f'<tr><th scope="row" data-role="title">{e(INCIDENT_WORD[kind])}{hint(kind)}{note}</th>' + td("EHR id", ehr_html, role="wide")
                    + td("Nodes", nodes_cell(claim), role="meta-q") + td("Detected", rel(at), role="meta-q") + td("", act, "fui-row-menu", role="menu") + "</tr>")
    return '<div class="fui-worklist">' + table("Incidents", th("What happened") + th("EHR id") + th("Nodes") + th("Detected") + th('<span class="fui-sr-only">Actions</span>'), rows) + "</div>"


def kind_counts():
    items = "".join(f'<li><span>{e(INCIDENT_WORD[k])}{hint(k)}</span><strong class="fui-num">{n}</strong></li>' for k, n in INCIDENT_KINDS)
    return section("Since gateway-0 started", f'<ul class="mk-plain-list mk-kind-counts">{items}</ul>')


def integrity_page(tab="open", menu_open=None, overlay=None, top="", open_items=None, resolved_items=None, toast_html="", incidents=2):
    open_items = INCIDENTS_OPEN if open_items is None else open_items
    resolved_items = INCIDENTS_RESOLVED if resolved_items is None else resolved_items
    t = tabs("Incidents by state", [("open", "Open", "integrity.html", len(open_items)), ("resolved", "Resolved", "integrity-resolved.html", len(resolved_items))], tab)
    if tab == "open":
        panel = incidents_table(open_items, menu_open=menu_open) if open_items else quiet_line("No incident is open. A collision cleared at every replica moves to Resolved.", "Resolved incidents", "integrity-resolved.html")
    else:
        panel = incidents_table(resolved_items, resolved=True)
    head = page_heading("Integrity", actions=link_button("primary", "Clear an EHR id…", "integrity-clear.html", "eraser"), subtitle="The incidents this replica keeps, and the held EHR ids an operator cleared.")
    content = head + top + tpl.list_body("", "", "", tpl.card(panel, tabs_html=t, label="Incidents")) + f'<div class="fui-card fui-panel">{kind_counts()}</div>'
    return shell("Integrity", "integrity", content, overlay=overlay, toast_html=toast_html, incidents=incidents)


def page_integrity():
    return integrity_page()


def page_integrity_resolved():
    return integrity_page("resolved")


def page_integrity_empty():
    return integrity_page(open_items=[], incidents=0)


def page_integrity_menu():
    return integrity_page(menu_open=0)


def clear_dialog(invalid=False, busy=False):
    inv = ' aria-invalid="true" aria-describedby="cl-ehr-error"' if invalid else ""
    ro = " readonly" if busy else ""
    er = ('<p id="cl-ehr-error" class="fui-help fui-error">' + icon("circle-alert", "14") + ' “7c1e4a2b-0d3f” is shortened: give the whole EHR id, 36 characters, as the incident names it.</p>') if invalid else ""
    ehr_field = (f'<div class="fui-field"><label for="cl-ehr" class="fui-label">EHR id</label>{input_("cl-ehr", "7c1e4a2b-0d3f" if invalid else EHR_A, extra="spellcheck=" + chr(34) + "false" + chr(34) + " class=" + chr(34) + "fui-input fui-mono" + chr(34) + inv + ro)}'
                 f'{er}<p class="fui-help">Filled from the incident.</p></div>')
    rep = select("cl-replica", [("all", "Every replica, one after another"), ("gateway-0", "gateway-0"), ("gateway-1", "gateway-1")], "all", extra="disabled" if busy else "")
    go = busy_button("primary", "Clearing at gateway-0") if busy else button("primary", "Clear the EHR id", "eraser", attrs="disabled" if invalid else "")
    cancel_help = '<p class="fui-small fui-muted">Cancel stops before gateway-1; gateway-0 keeps what it already cleared.</p>' if busy else ""
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="cl-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="cl-title" class="fui-dialog-title">{icon("eraser", "18")} Clear a held EHR id</h2>'
        '<p>The replica forgets the collision, so the next request finds the one owner again. Clear it only once the node at fault has fixed its data.</p>'
        + ehr_field
        + field("cl-replica", "Replica", rep, help_="Each replica keeps its own routing state.")
        + f'<p class="fui-small fui-muted">Recorded with your name in the integrity log{hint("ferrofed::integrity")}.</p>{cancel_help}'
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{go}</div></div>'
    )


def page_integrity_clear():
    return integrity_page(overlay=clear_dialog())


def page_integrity_clear_invalid():
    return integrity_page(overlay=clear_dialog(invalid=True))


def page_integrity_clear_busy():
    return integrity_page(overlay=clear_dialog(busy=True))


def outcome(tone, title, items, action=""):
    lis = "".join(f'<li><code>{r}</code>: {t}</li>' for r, t in items)
    a = f'<div class="fui-actions">{action}</div>' if action else ""
    role = "status" if tone == "ok" else "alert"
    return (f'<div class="fui-stack mk-outcome"><div class="fui-notice fui-tone-{tone}" role="{role}" tabindex="-1">{icon("circle-check" if tone == "ok" else "triangle-alert", "16")}<div><strong>{title}</strong>'
            f'<ul class="fui-bullets fui-small">{lis}</ul>{a}</div></div></div>')


def page_integrity_cleared():
    top = outcome("ok", f'Cleared <span title="{EHR_A}">{EHR_A[:8]}…</span> at both replicas. The incident moved to Resolved.',
                  [("gateway-0", "it held a collision between node-b and node-c, and forgot its index entry and the bindings of 3 sessions."),
                   ("gateway-1", "it held nothing for it, as expected: no request for it reached this replica.")])
    o = INCIDENTS_OPEN[0]
    resolved = [(o[0], o[1], EHR_A, o[3], "Cleared at gateway-0 and gateway-1.", "Demo Operator, today 09:44")] + INCIDENTS_RESOLVED
    return integrity_page(top=top, open_items=INCIDENTS_OPEN[1:], resolved_items=resolved, incidents=1)


def page_integrity_clear_refused():
    top = outcome("warn", "Cleared at gateway-0; gateway-1 did not clear it.",
                  [("gateway-0", "it held a collision between node-b and node-c, and forgot it."),
                   ("gateway-1", "refused, 401: it did not accept your access token, so nothing changed there.")],
                  button("secondary", "Sign in again", "log-in", small=True))
    return integrity_page(top=top)


# ---- self-description: the OPTIONS document, built from the one federation ----

def options_doc():
    eps = [{"id": ep["id"], "organisation": ep["node"]["org"]["id"], "status": "suspended" if ep["tone"] == "muted" else "active",
            "node_id": ep["node"]["id"], "system_id": ep["node"]["sid"], "product": ep["node"]["product"].split()[0], "version": ep["node"]["product"].split()[1]}
           for ep in sorted(data.QUERY_ENDPOINTS, key=lambda x: x["id"])]
    doc = {
        "federation": {
            "id": FEDERATION,
            "spec_version": "1.0",
            "aql": {"fan_out": True},
            "dedup": {"default": "none", "modes": ["none", "version-identity"], "request_header": "openEHR-federation-dedup"},
            "timeout": {"per_node_ms": 2500, "overall_ms": 4000, "policy": "abandon-and-mark", "localization_ms": 1500, "demographics_ms": 800, "consent_ms": 1000},
            "completeness": {"default": "all-or-nothing", "best_effort": True, "opt_in": {"header": "openEHR-federation-completeness", "value": "partial"}},
            "paging": {"offset_strategy": "bounded", "max_window": 1000, "max_node_rows": 9999},
            "aggregates": {"decomposable": ["COUNT", "SUM", "MIN", "MAX", "AVG"]},
            "definition": {"fan_out_template_upload": False, "stored_query_registry": True, "stored_query_fan_out": False},
            "localization": {"on_failure": "closed", "mode": "nl-gf-nvi"},
            "auth": {"jwks_uri": JWKS_URI},
            "its_rest": {
                "query": "federated: GET and POST {base}/v1/query/aql and GET and POST {base}/v1/query/{name}[/{version}] fan out, each stored query served from the gateway registry",
                "ehr": "routed: every request under {base}/v1/ehr/{ehr_id} goes to the one node that owns the ehr_id",
                "definition": "routed-single-node: a template request under {base}/v1/definition/template/ goes to the one endpoint the targeting headers name, never merged; stored queries at the gateway registry",
                "demographic": "unsupported: 501",
            },
            "consent": {"prefilter": "nl-gf-mitz", "on_unavailable": "pass-to-node", "disclose": True, "emergency": "apply"},
        },
        "endpoints": eps,
    }
    return json.dumps(doc, indent=2, ensure_ascii=False)


OPTIONS_DOC = options_doc()


def ro_row(label, desc, value, key=None):
    k = f'<code class="mk-term" title="The term clients see in OPTIONS">{e(key)}</code>' if key else ""
    return (f'<div class="fui-setting-row"><div class="fui-setting-text"><span class="fui-setting-label">{label}</span><p class="fui-setting-desc">{desc}</p>{k}</div>'
            f'<div class="fui-setting-control"><span class="fui-setting-value">{value}</span></div></div>')


def offered(w="Offered", tone="ok"):
    return f'<span class="fui-badge fui-tone-{tone}">{w}</span>'


SD_TABS = [("fed", "Federation", "self-description.html"), ("caps", "Capabilities", "self-description-capabilities.html"),
           ("areas", "ITS-REST areas", "self-description-areas.html"), ("limits", "Limits", "self-description-limits.html"),
           ("eps", "Endpoints", "self-description-endpoints.html")]


def sd_sections():
    def copy(what):
        return icon_button("copy", f"Copy {what}")
    fed = (ro_row("Federation", "The identifier OPTIONS declares for this federation.", f'<code>{FEDERATION}</code>{copy("the federation id")}', "federation.id")
           + ro_row("Specification", "The text this gateway implements.", f'<a href="{SPEC_URL}" rel="external">Federation Tier with AQL 1.0</a>', "federation.spec_version")
           + ro_row("JWK Set", "The keys a node checks the gateway's signatures with.", f'{url_code(JWKS_URI)}{copy("the JWK Set URL")}', "federation.auth.jwks_uri")
           + ro_row("Answered by", "The replica this page read; each replica answers for itself.", "<code>gateway-0</code>"))
    caps = (ro_row("AQL fan-out", "A query goes to every node in scope.", offered(), "aql.fan_out")
            + ro_row("Deduplication", "Rows one version gives twice are counted once, when a client asks; none by default.", offered("Offered · by version identity"), "dedup")
            + ro_row("Best effort", "A client may ask for the rows that came back when a node does not answer; all or nothing by default.", offered(), "completeness.best_effort")
            + ro_row("When the localizer fails", "No node is asked, and the answer says why.", "Ask no node", "localization.on_failure: closed")
            + ro_row("Stored-query registry", "Stored queries are kept and served by the gateway.", offered(), "definition.stored_query_registry")
            + ro_row("Stored-query fan-out", "A stored version is not sent on to the nodes.", offered("Not offered", "neutral"), "definition.stored_query_fan_out")
            + ro_row("Template upload fan-out", "A template goes to one chosen node.", offered("Not offered", "neutral"), "definition.fan_out_template_upload")
            + ro_row("Consent exclusions", "A node the consent pre-filter excludes is named consent-denied.", offered("Disclosed", "info"), "consent.disclose"))

    def mode(w, tone="info"):
        return f'<span class="fui-badge fui-tone-{tone}">{w}</span>'
    areas = (ro_row("Query", "AQL and stored queries fan out to the nodes in scope.", mode("Federated", "accent"))
             + ro_row("EHR", "Each request under an EHR id goes to the one node that owns it.", mode("Routed"))
             + ro_row("Definition", "Templates go to one chosen node; stored queries are the gateway's.", mode("Routed, registry at the gateway"))
             + ro_row("Demographic", "Answers 501.", mode("Not served", "neutral")))
    aggs = "".join(f'<span class="fui-tag">{a}</span>' for a in ["COUNT", "SUM", "MIN", "MAX", "AVG"])
    limits = (ro_row("Per-node budget", "How long one node may take.", "2,500 ms", "timeout.per_node_ms") + ro_row("Overall budget", "How long a whole query may take.", "4,000 ms", "timeout.overall_ms")
              + ro_row("Localizer budget", "A part of the overall budget.", "1,500 ms", "timeout.localization_ms")
              + ro_row("A late node", "Named time-out and left out; this is fixed.", "Abandon and mark", "timeout.policy")
              + ro_row("Paging", "Each node is asked for offset plus limit rows; the gateway merges, sorts and cuts the page, within 1,000 rows.", "1,000 rows", "paging.max_window")
              + ro_row("Rows per node", "The most rows read of one node for a query without LIMIT.", "9,999 rows", "paging.max_node_rows")
              + ro_row("Decomposable aggregates", "Aggregates the gateway can combine across nodes.", f'<span class="mk-ep-tags">{aggs}</span>', "aggregates.decomposable"))
    eprows = []
    for ep in sorted(data.QUERY_ENDPOINTS, key=lambda x: x["id"]):
        n = ep["node"]
        ex = ep["tone"] == "muted"
        eprows.append(f'<tr{" class=is-excluded" if ex else ""}><th scope="row"><code>{e(ep["id"])}</code></th>' + td("Node", f'<a href="{routes.node_href(n["id"])}">{e(n["id"])}</a>')
                      + td("Organisation", e(n["org"]["name"])) + td("Status", ("Suspended" if ex else "Active") + hint("suspended" if ex else "active")) + "</tr>")
    eps = table("Declared endpoints", th("Endpoint") + th("Node") + th("Organisation") + th("Status"), eprows)
    return {
        "fed": ("", fed),
        "caps": ("What OPTIONS tells a client this gateway does.", caps),
        "areas": ("How each area of the openEHR REST API is served.", areas),
        "limits": ('The limits clients read; they are set in <a href="configuration.html">Configuration</a>.', limits),
        "eps": ("The openEHR Query API endpoints OPTIONS lists for clients to target. A FHIR endpoint is never one, so it is not listed.", eps),
    }


def page_self_description(tab="fed"):
    intro, rows = sd_sections()[tab]
    if tab == "eps":
        inner = f'<div class="fui-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{intro}</p></div>{rows}</div>'
    else:
        inner = tpl.settings_panel(rows, intro)
    content = (page_heading("Self-description", subtitle='What clients see when they ask the gateway what it is. The OPTIONS document itself is on <a href="about.html">About</a>.')
               + tpl.settings_tabs("Self-description sections", SD_TABS, tab, inner))
    return shell("Self-description", "federation", content)


def page_self_description_capabilities():
    return page_self_description(tab="caps")


def page_self_description_areas():
    return page_self_description(tab="areas")


def page_self_description_limits():
    return page_self_description(tab="limits")


def page_self_description_endpoints():
    return page_self_description(tab="eps")
