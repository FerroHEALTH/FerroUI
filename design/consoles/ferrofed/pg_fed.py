# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The federation views: the routing table, integrity
with its admin action, and the self-description."""
from common import (
    MENU_SEP, badge, button, busy_button, card, chip, definitions, disclosure, e, empty_state, facts_row, field,
    icon, icon_button, input_, link_button, menu, menu_item, notice, page_heading, pager, pill, refusal, select,
    shell, status, table, tabbed, tabs, tag, td, textarea, th, titled_panel, toast, toasts, well, when, copy_button,
)
from data import (DEPENDENCIES, EHR_A, EHR_B, EHR_C, INCIDENT_KINDS, INCIDENTS_OPEN, INCIDENTS_RESOLVED, MEMBERS,
                  ROUTES, FEDERATION, SPEC, VERSION)
from parts import filter_chips, fold_actions, skeleton_lines
from pg_shell import endpoint_link, health, membership, replica_line, tech_detail, unreachable_block

KIND_WORD = {"query": "Query API", "fhir": "FHIR"}


# ---- routing ----

def all_routes():
    rows = list(ROUTES)
    wards = [("ward-%02d.zvs.example.org" % i, "learned", "node-a", "node-a-pub") for i in range(1, 16)]
    clinics = [("clinic-%s.zgp.example.org" % c, "learned", "node-c", "node-c-pub") for c in "abcdefg"]
    return sorted(rows + wards + clinics, key=lambda r: r[0])


SOURCE_LINE = {"node": "the node's own system_id", "registered": "registered in the registry document", "learned": "learned from a node's answer"}


def routing_rows(rows):
    return [f'<tr><th scope="row"><code>{e(c)}</code></th>' + td("Ground", tag(s)) + td("Node", f'<a href="node.html"><code>{n}</code></a>')
            + td("Endpoint", f"<code>{ep}</code>") + "</tr>" for c, s, n, ep in rows]


def routing_tools(src=None, text=""):
    import tpl
    return tpl.toolbar("A creating_system_id", text, [tpl.filter_button("Ground", src), tpl.filter_button("Node")])


def routing_page(rows, total, tools=None, chips="", body=None, page=1, pressed="all"):
    import tpl
    head = th("creating_system_id", "cs", "ascending") + th("Ground", title="Node: the node's own system_id · registered: in the registry document · learned: from a node's answer") + th("Node") + th("Endpoint")
    if body is None:
        shown = rows[(page - 1) * 25: page * 25]
        cardh = tpl.card(table("The creating_system_id routing table", head, routing_rows(shown)), pager("Routing table", total, 25, page, sizes=(25, 50, 100)), label="Routing table", count=f"{total} creating_system_ids")
    else:
        cardh = body
    summ = tpl.summary([("32 routes", None, pressed == "all"), ("4 node", None, False), ("2 registered", None, pressed == "registered"), ("26 learned", None, False)])
    content = (page_heading("Routing", subtitle="Where each creating_system_id routes a follow-up read or write.")
               + tpl.list_body(summ, tools or routing_tools(), chips, cardh))
    return shell("Routing", "routing", content)


def page_routing():
    r = all_routes()
    return routing_page(r, len(r))


def page_routing_page2():
    r = all_routes()
    return routing_page(r, len(r), page=2)


def page_routing_filtered():
    import tpl
    r = [x for x in all_routes() if x[1] == "registered"]
    return routing_page(r, len(r), tools=routing_tools("registered"), chips=tpl.active(["Ground: registered"]), pressed="registered")


def page_routing_no_match():
    import tpl
    body = empty_state("search-x", "No route matches “pacs”", "32 creating_system_ids are routed, but none contains “pacs”. A system no route names is asked of every node that may hold the EHR.",
                       button("primary", "Clear the filter", "x"), filtered=True)
    return routing_page([], 0, tools=routing_tools(text="pacs"), chips=tpl.active(["Name: pacs"]), body=body)


# ---- integrity ----

def incident_menu(i, open_=False):
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
               f'aria-label="Actions for the incident" title="More actions">{icon("ellipsis", "16")}</button>')
    m = menu([menu_item("Clear the held ehr_id…", "eraser", focus=True), menu_item("Open node-b", "arrow-right", href="node-degraded.html")], label="Actions for the incident", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def incidents_table(items, resolved=False, menu_open=None):
    rows = []
    for i, (kind, at, ehr, claim, desc) in enumerate(items):
        ehr_html = f'<code class="fui-break">{ehr}</code>' if ehr else '<span class="mk-dash">–</span>'
        act = "" if resolved or kind != "EhrIdCollision" else incident_menu(i, menu_open == i)
        rows.append(f'<tr><th scope="row">{e(kind)}<span class="fui-small fui-muted mk-incident-line">{e(desc)}</span></th>' + td("ehr_id", ehr_html)
                    + td("Nodes", " ".join(f'<code>{c}</code>' for c in claim)) + td("Detected", when(at)) + td("", act, "fui-table-actions") + "</tr>")
    return table("Incidents", th("Kind") + th("ehr_id") + th("Nodes") + th("Detected") + th('<span class="fui-sr-only">Actions</span>'), rows)


def integrity_page(tab="open", menu_open=None, overlay=None, top="", open_items=None, resolved_items=None, toast_html="", incidents=2):
    import tpl
    open_items = INCIDENTS_OPEN if open_items is None else open_items
    resolved_items = INCIDENTS_RESOLVED if resolved_items is None else resolved_items
    t = tabs("Incidents by state", [("open", "Open", "integrity.html", len(open_items)), ("resolved", "Resolved", "integrity-resolved.html", len(resolved_items))], tab)
    if tab == "open":
        panel = incidents_table(open_items, menu_open=menu_open) if open_items else empty_state("shield-check", "No open incidents", "None of the incidents this replica keeps is open. A collision cleared at every replica moves to Resolved.")
    else:
        panel = incidents_table(resolved_items, resolved=True)
    head = page_heading("Integrity", actions=button("primary", "Clear an ehr_id…", "eraser"), subtitle="The incidents this replica keeps, and the held ehr_ids an operator cleared.")
    content = head + top + tpl.list_body("", "", "", tpl.card(panel, tabs_html=t, label="Incidents"))
    return shell("Integrity", "integrity", content, overlay=overlay, toast_html=toast_html, incidents=incidents)


def page_integrity():
    return integrity_page()


def page_integrity_resolved():
    return integrity_page("resolved")


def page_integrity_empty():
    return integrity_page(open_items=[], incidents=0)


def page_integrity_menu():
    return integrity_page(menu_open=0)


def clear_dialog(ehr=EHR_A, invalid=False, busy=False, reason="node-c removed its duplicate EHR, ticket OPS-118"):
    inv = ' aria-invalid="true" aria-describedby="cl-ehr-error"' if invalid else ""
    er = '<p id="cl-ehr-error" class="fui-help fui-error">' + icon("circle-alert", "14") + ' “7c1e4a2b-0d3f” is not a bare UUID: give all 36 characters, as the incident names it.</p>' if invalid else ""
    ehr_field = (f'<div class="fui-field"><label for="cl-ehr" class="fui-label">ehr_id</label>{input_("cl-ehr", "7c1e4a2b-0d3f" if invalid else ehr, extra="spellcheck=\"false\" class=\"fui-input fui-mono\"" + inv)}'
                 f'{er}<p class="fui-help">The incidents above offer the ehr_ids they name.</p></div>')
    go = busy_button("primary", "Clearing at gateway-0") if busy else button("primary", "Clear the ehr_id", "eraser")
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="cl-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="cl-title" class="fui-dialog-title">{icon("eraser", "18")} Clear a held ehr_id</h2>'
        '<p>The replica forgets the remedied collision, so the next request finds the one owner again. Clear it only once the node at fault has fixed its data.</p>'
        + ehr_field
        + field("cl-replica", "Replica", select("cl-replica", [("all", "Every replica, one after another"), ("gateway-0", "gateway-0"), ("gateway-1", "gateway-1")], "all"),
                help_="Each replica keeps its own routing state.")
        + field("cl-reason", "Reason", textarea("cl-reason", reason), help_="Required. Recorded with your name under ferrofed::security.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{go}</div></div>'
    )


def page_integrity_clear():
    return integrity_page(menu_open=None, overlay=clear_dialog())


def page_integrity_clear_invalid():
    return integrity_page(overlay=clear_dialog(invalid=True))


def page_integrity_clear_busy():
    return integrity_page(overlay=clear_dialog(busy=True))


def page_integrity_cleared():
    outcome = (
        f'<div class="fui-notice fui-tone-ok" role="status" tabindex="-1">{icon("circle-check", "16")}<div><strong>Cleared {EHR_A[:8]}… at both replicas.</strong>'
        '<ul class="fui-bullets fui-small"><li><code>gateway-0</code>: it held a collision between node-b and node-c. It forgot an ehr_id index entry and the bindings of 3 sessions.</li>'
        '<li><code>gateway-1</code>: it held nothing for it, which is expected: no request for it reached this replica.</li></ul></div></div>'
    )
    open_items = INCIDENTS_OPEN[1:]
    resolved = [(INCIDENTS_OPEN[0][0], INCIDENTS_OPEN[0][1], EHR_A, INCIDENTS_OPEN[0][3], "Cleared at gateway-0 and gateway-1 by Demo Operator today 09:44.")] + INCIDENTS_RESOLVED
    t = toasts(toast("ok", "circle-check", "The ehr_id is cleared", "Recorded with your reason. The incident moved to Resolved."))
    return integrity_page(top=outcome, open_items=open_items, resolved_items=resolved, toast_html=t, incidents=1)


def page_integrity_clear_refused():
    outcome = (
        f'<div class="fui-notice fui-tone-warn" role="alert" tabindex="-1">{icon("triangle-alert", "16")}<div><strong>Cleared at gateway-0; gateway-1 refused it.</strong>'
        '<ul class="fui-bullets fui-small"><li><code>gateway-0</code>: it held a collision between node-b and node-c, and forgot it.</li>'
        '<li><code>gateway-1</code>: refused, 403 <code>operator-unverified</code>. Your token was not verified there, so nothing changed on that replica. Sign in again, then clear it at gateway-1 alone.</li></ul>'
        f'<div class="fui-actions">{button("secondary", "Clear at gateway-1…", "eraser", small=True)}</div></div></div>'
    )
    return integrity_page(top=outcome)


# ---- self-description ----

OPTIONS_DOC = """{
  "federation": {
    "id": "demo-regio-noord",
    "specification_version": "1.0.0",
    "endpoints": [
      { "id": "node-a-pub", "organisation": "org-zvs", "status": "active" },
      { "id": "node-b-pub", "organisation": "org-tko", "status": "active" },
      { "id": "node-c-pub", "organisation": "org-zgp", "status": "active" },
      { "id": "node-d-pub", "organisation": "org-rdw", "status": "active" },
      { "id": "node-e-pub", "organisation": "org-htm", "status": "suspended" }
    ],
    "dedup": { "header": "openEHR-federation-dedup", "modes": ["none", "version-identity"] },
    "completeness": { "partial": "offered" },
    "budgets": { "per_node_ms": 2500, "overall_ms": 4000, "timeout_policy": "abandon-and-mark" },
    "consent": { "disclose": true }
  }
}"""


def ref_nav(items, current):
    links = "".join(f'<li><a href="#{k}" class="fui-nav-entry"{" aria-current=" + chr(34) + "true" + chr(34) if k == current else ""}>{icon(g, "16")}{w}</a></li>' for k, w, g in items)
    opts = "".join(f'<option{" selected" if k == current else ""}>{w}</option>' for k, w, g in items)
    return (f'<nav class="fui-settings-nav" aria-label="Sections"><ul class="fui-nav-list">{links}</ul></nav>'
            f'<div class="fui-settings-nav-select"><label for="ref-sec" class="fui-label">Section</label><select id="ref-sec" class="fui-select">{opts}</select></div>')


def ro_row(label, desc, value, key=None):
    k = f'<code class="mk-term" title="The term clients see in OPTIONS">{e(key)}</code>' if key else ""
    return (f'<div class="fui-setting-row"><div class="fui-setting-text"><span class="fui-setting-label">{label}</span><p class="fui-setting-desc">{desc}</p>{k}</div>'
            f'<div class="fui-setting-control"><span class="fui-setting-value">{value}</span></div></div>')


def offered(word="Offered", tone="ok"):
    return f'<span class="fui-badge fui-tone-{tone}">{word}</span>'


def sd_card(sid, title, rows, intro="", actions=""):
    head = f'<div class="fui-panel-head mk-card-head"><h2 class="fui-card-title" id="{sid}-t">{title}</h2>{actions}</div>'
    return f'<section class="fui-card fui-setting-card" aria-labelledby="{sid}-t">{head}{f"<p class=fui-setting-section-desc>{intro}</p>" if intro else ""}<div class="fui-setting-rows">{rows}</div></section>'


SD_TABS = [("fed", "Federation", "self-description.html"), ("caps", "Capabilities", "self-description-capabilities.html"),
           ("areas", "ITS-REST areas", "self-description-areas.html"), ("limits", "Limits", "self-description-limits.html"),
           ("eps", "Endpoints", "self-description-endpoints.html")]


def sd_sections():
    jwk = f'<span class="mk-nowrap" title="https://gateway.demo-regio.example.org/.well-known/jwks.json"><code>gateway.demo-regio.example.org</code></span>{icon_button("copy", "Copy the JWK Set URL")}'
    fed = (ro_row("Federation", "The name clients see in every answer.", f'<code>{FEDERATION}</code>{icon_button("copy", "Copy the federation name")}', "federation.id")
           + ro_row("Specification", "The text this gateway implements.", '<a href="#">Federation Tier with AQL 1.0.0</a>')
           + ro_row("JWK Set", "The keys a node checks the gateway's signatures with.", jwk)
           + ro_row("Answered by", "The replica this page read; each replica answers for itself.", "<code>gateway-0</code>"))
    caps = (ro_row("AQL fan-out", "A query goes to every node in scope.", offered())
            + ro_row("Deduplication", "Rows the same version gives twice are counted once, when a client asks.", offered("Offered · by version identity"), "openEHR-federation-dedup")
            + ro_row("Best effort", "A client may ask for the rows that came back when a node does not answer.", offered(), "openEHR-federation-completeness: partial")
            + ro_row("Stored-query registry", "Stored queries are kept and served by the gateway.", offered())
            + ro_row("Stored-query fan-out", "A stored version is not pushed to every node.", offered("Not offered", "neutral"))
            + ro_row("Template upload fan-out", "A template goes to one chosen node.", offered("Not offered", "neutral"))
            + ro_row("Consent exclusions", "An excluded node is named consent-denied.", offered("Disclosed", "info")))

    def mode(w, tone="info"):
        return f'<span class="fui-badge fui-tone-{tone}">{w}</span>'
    areas = (ro_row("Query", "AQL and stored queries fan out to the nodes in scope.", mode("federated", "accent"))
             + ro_row("EHR", "Each request under an ehr_id goes to the one node that owns it.", mode("routed"))
             + ro_row("Definition", "Templates go to one chosen node; stored queries are the gateway's.", mode("routed · registry"))
             + ro_row("Demographic", "Not served: answers 501.", mode("not served", "neutral")))
    aggs = "".join(f'<span class="fui-tag">{a}</span>' for a in ["COUNT", "SUM", "MIN", "MAX", "AVG"])
    limits = (ro_row("Per-node budget", "How long one node may take.", "2,500 ms") + ro_row("Overall budget", "How long a whole query may take.", "4,000 ms")
              + ro_row("Timeout policy", "A late node is named time-out and left out.", "abandon and mark") + ro_row("Offset strategy", "An offset is applied per node within a bound.", "bounded")
              + ro_row("Decomposable aggregates", "Aggregates the gateway can combine across nodes.", f'<span class="mk-ep-tags">{aggs}</span>'))
    from data import ORGS
    eprows = []
    for o in ORGS:
        for n in o["nodes"]:
            for ep in n["endpoints"]:
                ex = o["standing"] != "active" if isinstance(o.get("standing"), str) else False
                eprows.append(f'<tr{" class=is-excluded" if ex else ""}><th scope="row"><code>{e(ep["id"])}</code></th>' + td("Node", f'<a href="node.html">{e(n["id"])}</a>')
                              + td("Organisation", e(o["name"])) + td("Type", f'<code>{e(ep["conn"])}</code>') + td("Declared", "excluded" if ex else "yes") + "</tr>")
    eps = table("Declared endpoints", th("Endpoint") + th("Node") + th("Organisation") + th("Type") + th("Declared"), eprows)
    return {
        "fed": ("", fed),
        "caps": ("What OPTIONS tells a client this gateway does.", caps),
        "areas": ("How each area of the openEHR REST API is served.", areas),
        "limits": ('As set in <a href="configuration.html">Configuration</a>, the way clients read them.', limits),
        "eps": ("What OPTIONS lists for clients to target.", eps),
    }


def page_self_description(open_doc=False, tab="fed"):
    import tpl
    intro, rows = sd_sections()[tab]
    if tab == "eps":
        inner = f'<div class="fui-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{intro}</p></div>{rows}</div>'
    else:
        inner = tpl.settings_panel(rows, intro)
    content = (page_heading("Self-description", subtitle='What clients see when they ask the gateway what it is. The raw OPTIONS document is on <a href="about.html">About</a>.')
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



