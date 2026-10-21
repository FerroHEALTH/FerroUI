# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Organisations, their nodes, and each node's endpoints: the one home of
each, in FerroFED's own model ([[organisation]], [[node]], [[endpoint]]).
An organisation is created on the Organisations page; its nodes are added,
edited, excluded and removed inside it; a node's endpoints, and each
endpoint's credentials, live on the node."""
from common import (
    action_group, MENU_SEP, badge, busy_button, button, card, check, choice_group, definitions, dock, dock_field, e, empty_state,
    facts_row, field, health_dot, icon, icon_button, input_, link_button, menu, menu_item, notice, page_heading, pager,
    pill, select, shell, table, tabbed, tabs, tag, td, textarea, th, titled_panel, toast, toasts, when,
)
from data import ORGS, STATE_WORD, large_orgs, worst, ROUTES
from parts import filter_chips, fold_actions, revision_strip, skeleton_lines
from pg_shell import replica_line, tech_detail, unreachable_block

KIND_WORD = {"query": "Query API", "fhir": "FHIR"}
ORG_HREF = {"zvs": "organisation.html", "tko": "organisation-degraded.html", "zgp": "organisation.html", "rdw": "organisation.html", "htm": "organisation.html", "zbt": "organisation-empty.html"}
ZBT = {"slug": "zbt", "id": "org-zbt", "name": "Zorgcentrum Beta-Testerveen", "standing": "active", "nodes": []}


def state(tone, detail=None):
    """A state as the dot's shape and colour and the word, never the colour alone."""
    d = f'<span class="fui-status-detail">{e(detail)}</span>' if detail else ""
    return f'<span class="fui-state fui-tone-{"neutral" if tone == "muted" else tone}{" is-muted" if tone == "muted" else ""}">{health_dot(tone)}{STATE_WORD[tone]}{d}</span>'


def node_href(n):
    return "node-degraded.html" if n["tone"] == "warn" else "node.html"


def ms(v):
    return f"{v:,} ms" if v is not None else '<span class="mk-dash">–</span>'


def at(t):
    return when("2026-10-21T" + t, seconds=True) if t else '<span class="mk-dash">never</span>'


def node_probe(n):
    return max((x["probe"] for x in n["endpoints"] if x.get("probe")), default=None)


def org_find(slug):
    return next((x for x in ORGS if x["slug"] == slug), ZBT)


# ---- the Organisations page: Organisations and All nodes (list template) ----

import tpl


def org_menu(name, open_=False):
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
               f'aria-label="Actions for {e(name)}" title="More actions">{icon("ellipsis", "16")}</button>')
    m = menu([menu_item("Open", "arrow-right", keys="Enter", href="organisation.html"), menu_item("Its nodes", "server", href="organisation-nodes.html", hover=True),
              MENU_SEP, menu_item("Exclude…", "ban", href="organisation-exclude.html", danger=True)], label=f"Actions for {name}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def excluded_badge():
    return ' <span class="fui-badge fui-tone-neutral">excluded</span>'


def org_rows(orgs=None, hover=None, menu_open=None, menus=True):
    rows = []
    for i, o in enumerate(orgs or ORGS):
        tones = [n["tone"] for n in o["nodes"]]
        live = [t for t in tones if t != "muted"]
        w = worst(live) if live else ("muted" if tones else "neutral")
        last = max((node_probe(n) for n in o["nodes"] if node_probe(n)), default=None)
        exc = o["standing"] != "active"
        cls = " ".join(x for x in ["mk-hover" if i == hover else "", "is-excluded" if exc else ""] if x)
        rows.append(
            f'<tr{f" class={chr(34)}{cls}{chr(34)}" if cls else ""}><th scope="row" data-role="title"><a class="fui-row-link" href="{ORG_HREF[o["slug"]]}">{e(o["name"])}</a></th>'
            + td("Id", f'<code>{o["id"]}</code>', role="wide") + td("Nodes", str(len(o["nodes"])), "fui-num", role="meta-q")
            + td("State", state(w), role="meta-2") + td("Last probe", tpl.rel(last), role="wide")
            + td("", org_menu(o["name"], menu_open == i) if menus else "", "fui-row-menu", role="menu") + "</tr>")
    return rows


def org_head():
    return (th("Organisation", "name", "ascending") + th("Id") + th("Nodes", num=True) + th("State", title="The worst state of its nodes")
            + th("Last probe") + th('<span class="fui-sr-only">Actions</span>'))


STATE_ITEMS = [("reachable", "ok", 3, True), ("degraded", "warn", 1, True), ("unreachable", "bad", 0, True), ("not probed", "neutral", 0, True), ("excluded", "muted", 1, True)]


def org_summary(pressed="all", n_orgs=5, deg=1, exc=1):
    return tpl.summary([(f"{n_orgs} organisations", None, pressed == "all"), (f"{deg} degraded", "warn", pressed == "warn"), (f"{exc} excluded", "muted", pressed == "muted")])


def org_toolbar(value="", state_word=None, open_=False):
    return tpl.toolbar("Organisation name or id", value, [tpl.filter_button("State", state_word, open_, STATE_ITEMS)])


COUNT = "5 organisations · 6 nodes"


def orgs_page(tab="orgs", body=None, tools=None, chips="", toast_html="", readonly=False, strip="", org_count=5, node_count=6, summary_html=None, foot=""):
    t = tabs("Organisations", [("orgs", "Organisations", "organisations.html", org_count), ("nodes", "All nodes", "organisations-nodes.html", node_count)], tab)
    table_html = body if body is not None else table("Organisations", org_head(), org_rows(), scroll_label="Organisations")
    cardh = tpl.card(table_html, foot, tabs_html=t, label="Organisations")
    act = "" if readonly else link_button("primary", "New organisation", "organisation-new.html", "plus")
    head = page_heading("Organisations", actions=act, subtitle="The organisations in the federation and their nodes.")
    # NOTE: the summary strip and the toolbar appear only past 10 items; a short list needs neither.
    many = max(org_count, node_count if tab == "nodes" else 0) > 10
    summ = summary_html if summary_html is not None else (org_summary() if many else "")
    tl = tools if tools is not None else (org_toolbar() if many else "")
    content = head + strip + tpl.list_body(summ, tl, chips, cardh)
    return shell("Organisations", "orgs", content, toast_html=toast_html)


def orgs_table(rows):
    return table("Organisations", org_head(), rows, scroll_label="Organisations")


def page_organisations():
    return orgs_page()


def page_organisations_hover():
    return orgs_page(body=orgs_table(org_rows(hover=1)))


def page_organisations_menu():
    return orgs_page(body=orgs_table(org_rows(hover=1, menu_open=1)))


def big_rows(items):
    rows = []
    for o in items:
        exc = o["worst"] == "muted"
        rows.append(f'<tr{" class=" + chr(34) + "is-excluded" + chr(34) if exc else ""}><th scope="row" data-role="title"><a class="fui-row-link" href="organisation.html">{e(o["name"])}</a></th>'
                    + td("Id", f'<code>{o["id"]}</code>', role="wide") + td("Nodes", str(o["n"]), "fui-num", role="meta-q") + td("State", state(o["worst"]), role="meta-2")
                    + td("Last probe", tpl.rel(o["probe"]), role="wide") + td("", org_menu(o["name"]), "fui-row-menu", role="menu") + "</tr>")
    return rows


def big_summary(pressed="all"):
    return tpl.summary([("30 organisations", None, pressed == "all"), ("1 degraded", "warn", pressed == "warn"), ("1 unreachable", "bad", pressed == "bad"), ("1 not probed", "neutral", False), ("1 excluded", "muted", False)])


def page_organisations_filtered():
    big = large_orgs()
    o = [x for x in big if x["worst"] == "warn"]
    items = [(w, t, n, t == "warn") for w, t, n, _ in STATE_ITEMS]
    tools = tpl.toolbar("Organisation name or id", "", [tpl.filter_button("State", "degraded", True, items)])
    return orgs_page(body=orgs_table(big_rows(o)), tools=tools, chips=tpl.active(["State: degraded"]), summary_html=big_summary("warn"),
                     org_count=30, node_count=sum(x["n"] for x in big))


def page_organisations_no_match():
    big = large_orgs()
    body = empty_state("search-x", "No organisation matches these filters", "30 organisations take part, but none has an unreachable node and a name with “kliniek”. Clear a filter to see them.",
                       button("primary", "Clear filters", "x") + button("secondary", "Remove “State: unreachable”"), filtered=True)
    return orgs_page(body=body, tools=org_toolbar("kliniek", "unreachable"), chips=tpl.active(["State: unreachable", "Name: kliniek"]), summary_html=big_summary(),
                     org_count=30, node_count=sum(x["n"] for x in big))


def page_organisations_loading():
    rows = "".join(f'<tr aria-hidden="true"><th data-role="title">{skeleton_lines([80])}</th><td data-role="wide">{skeleton_lines([50])}</td><td data-role="meta-q">{skeleton_lines([20])}</td>'
                   f'<td data-role="meta-2">{skeleton_lines([60])}</td><td data-role="wide">{skeleton_lines([60])}</td><td></td></tr>' for _ in range(5))
    return orgs_page(body=table("Organisations, loading", org_head(), [rows]))


def page_organisations_unreachable():
    head = page_heading("Organisations", subtitle="The organisations in the federation and their nodes.")
    content = head + '<div class="fui-list-page">' + unreachable_block("the organisations") + tech_detail() + "</div>"
    return shell("Organisations", "orgs", content, status="down", incidents=None)


def page_organisations_empty():
    body = empty_state("building-2", "No organisation takes part yet",
                       "Create the first organisation, then add its nodes and each node's endpoints: an organisation comes before its nodes.",
                       link_button("primary", "New organisation", "organisation-new.html", "plus"))
    return orgs_page(body=body, tools="", org_count=0, node_count=0, summary_html="")


def all_node_rows():
    rows = []
    items = [(o, n) for o in ORGS for n in o["nodes"]]
    order = {"bad": 0, "warn": 1, "neutral": 2, "ok": 3, "muted": 4}
    for o, n in sorted(items, key=lambda x: order[x[1]["tone"]]):
        eps = n["endpoints"]
        exc = n["tone"] == "muted"
        rows.append(f'<tr{" class=" + chr(34) + "is-excluded" + chr(34) if exc else ""}><th scope="row" data-role="title"><a class="fui-row-link" href="{node_href(n)}"><code>{n["id"]}</code></a></th>'
                    + td("Organisation", e(o["name"]), role="sub") + td("System id", f'<code>{n["sid"]}</code>', role="wide")
                    + td("Endpoints", str(len(eps)), "fui-num", role="wide") + td("State", state(n["tone"]), role="meta-2")
                    + td("p50", ms(eps[0]["p50"]), "fui-num", role="wide") + td("Last probe", tpl.rel(node_probe(n)), role="meta-q") + "</tr>")
    return rows


def page_organisations_nodes():
    body = table("Every node", th("Node") + th("Organisation") + th("System id") + th("Endpoints", num=True) + th("State", "state", "ascending", title="Worst first") + th("p50", num=True) + th("Last probe"), all_node_rows())
    return orgs_page("nodes", body=body)


def page_organisations_large():
    big = large_orgs()
    n_nodes = sum(o["n"] for o in big)
    return orgs_page(body=orgs_table(big_rows(big[:25])), org_count=30, node_count=n_nodes, summary_html=big_summary(), foot=pager("Organisations", 30, 25, 1, sizes=(25, 50, 100)))


def page_organisations_directory():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="organisation.html">{e(o["name"])}</a></th>' + td("Id", f'<code>{o["id"]}</code>', role="wide")
            + td("Nodes", str(len(o["nodes"])), "fui-num", role="meta-q") + td("From", f'<span class="fui-tag fui-tag-locked">{icon("lock", "12")}Care services directory</span>', role="meta-1") + "</tr>" for o in ORGS]
    strip = revision_strip("running", "Read from the care services directory", "This gateway reads its registry from <code>https://mcsd.demo-regio.example.org/fhir</code> (<code>[registry.mcsd]</code>). Change an organisation, a node or an endpoint there: the console shows them read-only in either mode.")
    return orgs_page(body=table("Organisations", th("Organisation") + th("Id") + th("Nodes", num=True) + th("From"), rows, scroll_label="Organisations"), readonly=True, strip=strip)


def page_organisations_file_mode():
    strip = revision_strip("running", "Set in ferrofed.toml", "This deployment runs in file mode: the organisations, nodes and endpoints are the registry document <code>/etc/ferrofed/registry.json</code>. Change it there and reload; these pages show them read-only.",
                           link_button("ghost", "How configuration works", "configuration-file-mode-dialog.html", "info", small=True))
    return orgs_page(body=orgs_table(org_rows(menus=False)), readonly=True, strip=strip)


# ---- one organisation ----

ORG_TABS = [("overview", "Overview", "organisation.html"), ("nodes", "Nodes", "organisation-nodes.html"), ("requests", "Requests", "organisation-requests.html"),
            ("audit", "Audit", "organisation-audit.html"), ("settings", "Settings", "organisation-settings.html")]


def org_page(slug, tab, panel, overlay=None, menu_open=False, top="", toast_html="", dock_html=""):
    o = org_find(slug)
    ag = action_group(link_button("primary", "Add node", "node-new.html", "plus"),
                      [menu_item("Edit", "pencil", href="organisation-settings-edit.html", keys="e"), MENU_SEP, menu_item("Exclude…", "ban", href="organisation-exclude.html", danger=True)],
                      open_=menu_open, label=f"More actions for {o['name']}")
    standing = pill("ok", "active") if o["standing"] == "active" else pill("neutral", "excluded")
    n = len(o["nodes"])
    head = page_heading(e(o["name"]), actions=ag, badge=standing,
                        subtitle=f'Organisation <code>{o["id"]}</code> · {n} node{"s" if n != 1 else ""}', crumbs=[("Organisations", "organisations.html")])
    t = tabs(o["name"], [(k, w, h, n) if k == "nodes" else (k, w, h) for k, w, h in ORG_TABS], tab)
    content = head + f'<div class="fui-stack">{top}{tabbed(t, panel)}</div>'
    return shell(o["name"], "orgs", content, overlay=overlay, toast_html=toast_html, org=slug if slug in ("zvs", "tko", "zbt") else "zvs", dock_html=dock_html)


def org_overview(slug="zvs"):
    o = org_find(slug)
    tones = [n["tone"] for n in o["nodes"]]
    facts = facts_row([("Organisation id", f'<code>{o["id"]}</code>'), ("Standing", pill("ok", "active")), ("Nodes", str(len(o["nodes"]))),
                       ("Endpoints", str(sum(len(n["endpoints"]) for n in o["nodes"]))), ("Worst node", state(worst(tones))),
                       ("Added", f'{when("2026-10-16T10:05")} · <a href="configuration-history.html">revision 6</a>')])
    counts = {}
    for t in tones:
        counts[t] = counts.get(t, 0) + 1
    summary = " · ".join(f"{c} {STATE_WORD[t]}" for t, c in counts.items())
    glance = card("Nodes at a glance", f'<p class="fui-cluster">{" ".join(health_dot(t) for t in tones)}<span>{summary}</span></p>'
                  + '<p class="fui-small fui-muted">Each node with its endpoints and actions is on the Nodes tab.</p>',
                  actions=link_button("ghost", "Nodes", "organisation-nodes.html", glyph_end="chevron-right", small=True))
    act = card("Last 24 hours", definitions([("Requests", "1,284 to its endpoints · 3 refused"), ("Routed reads", "212, by creating_system_id"), ("Open incidents", '<a href="integrity.html">1</a> names it')]),
               actions=link_button("ghost", "Requests", "organisation-requests.html", glyph_end="chevron-right", small=True))
    topo = card("In the topology", '<p class="fui-small">The gateway, this organisation and its nodes, with each edge\'s last probe.</p>',
                actions=link_button("ghost", "Topology", "topology-selected.html", glyph_end="chevron-right", small=True))
    return f'<div class="fui-stack">{card("Definition", facts)}<div class="fui-grid fui-grid-start">{glance}{act}</div>{topo}</div>'


def page_organisation(menu_open=False, overlay=None):
    return org_page("zvs", "overview", org_overview("zvs"), menu_open=menu_open, overlay=overlay)


def page_organisation_actions():
    return page_organisation(menu_open=True)


def page_organisation_degraded():
    top = notice("warn", "triangle-alert", '<strong>1 node is degraded.</strong> <code>node-b</code> answers 503 since 09:38 today, so the organisation shows degraded: the worst of its nodes.',
                 action=" " + link_button("secondary", "Open node-b", "node-degraded.html", small=True))
    return org_page("tko", "overview", org_overview("tko"), top=top)


def node_menu(nid, open_=False):
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="Actions for {nid}" title="More actions">{icon("ellipsis", "16")}</button>')
    m = menu([menu_item("Open", "arrow-right", href="node.html"), menu_item("Probe now", "activity", href="node-probing.html"), menu_item("Edit", "pencil", href="node-edit.html", keys="e"),
              menu_item("Exclude…", "ban", href="node-exclude.html", hover=True), MENU_SEP, menu_item("Remove…", "trash-2", danger=True, href="node-remove.html")], label=f"Actions for {nid}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def ep_tags(eps, most=2):
    tags = "".join(f'<a class="fui-tag" href="node-endpoints.html">{x["id"]}</a>' for x in eps[:most])
    more = f'<a class="fui-tag" href="node-endpoints.html" title="{e(", ".join(x["id"] for x in eps[most:]))}">+{len(eps) - most}</a>' if len(eps) > most else ""
    return f'<span class="mk-ep-tags">{tags}{more}</span>'


def page_organisation_nodes():
    o = org_find("zvs")
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{node_href(n)}"><code>{n["id"]}</code></a></th>' + td("System id", f'<code>{n["sid"]}</code>', role="sub")
            + td("CDR", e(n["product"]), role="wide") + td("Endpoints", ep_tags(n["endpoints"]), role="wide")
            + td("State", state(n["tone"]), role="meta-2") + td("Last probe", tpl.rel(node_probe(n)), role="meta-q") + td("", node_menu(n["id"], n["id"] == "node-a-lab"), "fui-row-menu", role="menu") + "</tr>" for n in o["nodes"]]
    panel = ('<div class="fui-worklist" aria-label="Nodes">'
             + table("Nodes of this organisation", th("Node", "node", "ascending") + th("System id") + th("CDR") + th("Endpoints") + th("State") + th("Last probe") + th('<span class="fui-sr-only">Actions</span>'), rows)
             + "</div>")
    return org_page("zvs", "nodes", panel)


REQUESTS = [
    ("09:42:07", "node-a-pub", "AQL query", "200", "41 ms", "3 rows", "01JAF3M2C9QK"),
    ("09:41:55", "node-a-pub", "stored query vital_signs 1.1.0", "200", "38 ms", "6 rows", "01JAF3M1Z7PD"),
    ("09:41:20", "node-a-fhir", "Patient/$summary", "200", "212 ms", "1 bundle", "01JAF3KZW2HT"),
    ("09:40:48", "node-a-lab-pub", "routed read, composition", "200", "29 ms", "1", "01JAF3KX0D5A"),
    ("09:39:02", "node-a-pub", "AQL query", "424", "2,500 ms", "time-out", "01JAF3KQ8B1N"),
    ("09:37:44", "node-a-lab-pub", "AQL query", "200", "74 ms", "0 rows", "01JAF3KJ4M7E"),
]


def requests_table(rows):
    body = [f'<tr><th scope="row">{when("2026-10-21T" + t, seconds=True)}</th>' + td("Endpoint", f"<code>{s}</code>") + td("Request", e(r))
            + td("Status", pill("ok" if st.startswith("2") else "bad", st)) + td("Latency", lat, "fui-num") + td("Answer", e(a)) + td("Request id", f"<code>{rid}</code>") + "</tr>"
            for t, s, r, st, lat, a, rid in rows]
    return table("Recent requests", th("Time", "t", "descending") + th("Endpoint") + th("Request") + th("Status") + th("Latency", num=True) + th("Answer") + th("Request id"), body)


def page_organisation_requests():
    tools = tpl.toolbar("A request id", "", [tpl.filter_button("Node"), tpl.filter_button("Status")])
    panel = (f'<div class="fui-stack">{tools}<p class="fui-small fui-muted">What the gateway sent this organisation\'s endpoints, as its access record keeps it: never a query\'s text, a parameter or a value.</p>'
             + requests_table(REQUESTS) + "</div>")
    return org_page("zvs", "requests", panel)


def page_organisation_audit():
    rows = [
        ("2026-10-19T14:12", "Demo Operator", "revision 7", "changed the per-node timeout node-a's queries get", "Raise the per-node timeout for node-b's slower link"),
        ("2026-10-18T11:30", "Demo Operator", "probe", "probed node-a-fhir by hand: 200 in 24 ms", None),
        ("2026-10-17T09:20", "Demo Reviewer", "revision 6", "added node-a-lab with endpoint node-a-lab-pub", "Laboratory CDR joins"),
        ("2026-10-16T10:05", "Demo Reviewer", "revision 6", "added the organisation with node-a and its two endpoints", "Add Ziekenhuis Voorbeeldstad and Revalidatiecentrum Demo-West"),
    ]
    body = [f'<tr><th scope="row">{when(t)}</th>' + td("By", e(by), "mk-nowrap") + td("Kind", badge("neutral", k)) + td("What", e(w)) + td("Reason", e(r) if r else '<span class="mk-dash">–</span>') + "</tr>" for t, by, k, w, r in rows]
    panel = ('<div class="fui-stack"><p class="fui-small fui-muted">Every operator action that touched this organisation, its nodes or their endpoints: revisions, probes by hand, exclusions and clears.</p>'
             + table("Audit", th("When", "t", "descending") + th("By") + th("Kind") + th("What") + th("Reason"), body) + "</div>")
    return org_page("zvs", "audit", panel)


def org_settings_body(edit=False):
    roll = table("Its nodes' identity and consent entries", th("Node") + th("PIXm domain") + th("Mitz care provider") + th("NVI custodian"),
                 [f'<tr><th scope="row"><a href="node-settings.html"><code>{n}</code></a></th>' + td("PIXm domain", p) + td("Mitz", m) + td("NVI", v) + "</tr>"
                  for n, p, m, v in [("node-a", "<code>urn:oid:2.999.21</code>", "V6 · URA <code>90000101</code>", "URA <code>90000101</code>"),
                                     ("node-a-lab", "<code>urn:oid:2.999.27</code>", '<span class="fui-state fui-tone-warn">' + health_dot("warn") + 'not mapped</span>', "URA <code>90000101</code>")]])
    rollup = card("Set on each node", '<p class="fui-small fui-muted">FerroFED keys these by node, so each is set on its node\'s Settings tab; this is a read-only roll-up.</p>' + roll)
    if not edit:
        d = definitions([("Name", "Ziekenhuis Voorbeeldstad"), ("Organisation id", "<code>org-zvs</code> · fixed: it names the organisation in every revision"),
                         ("Nodes", '2 · <a href="organisation-nodes.html">Nodes tab</a>'), ("Standing", pill("ok", "active") + ' <span class="fui-small fui-muted">not excluded · Exclude… in the title row</span>')])
        return '<div class="fui-stack">' + card("Organisation", d) + rollup + "</div>"
    form = ('<div class="fui-form"><div class="fui-columns">'
            + field("os-name", "Name", input_("os-name", "Ziekenhuis Voorbeeldstad Noord", extra='class="fui-input fui-config-changed"'), help_="was Ziekenhuis Voorbeeldstad")
            + field("os-id", "Organisation id", input_("os-id", "org-zvs", extra='class="fui-input fui-mono" disabled'), help_=f"{icon('lock', '12')} Fixed: it names the organisation in every revision.")
            + "</div></div>")
    return ('<div class="fui-stack">' + revision_strip("draft", "Draft of Ziekenhuis Voorbeeldstad · 1 change", "Based on revision 7. Review it, then save and apply it as any revision.")
            + card("Organisation", form) + rollup + "</div>")


def page_organisation_settings():
    return org_page("zvs", "settings", org_settings_body())


def page_organisation_settings_edit():
    d = dock(dock_field("os-reason", "Reason", "The hospital merged with its northern site and renamed", "Why this changes", keys="r"),
             link_button("secondary", "Discard the draft", "organisation-settings.html", "x") + link_button("primary", "Review the changes", "config-review.html", "git-compare"))
    return org_page("zvs", "settings", org_settings_body(edit=True), dock_html=d)


def page_organisation_empty():
    body = empty_state("server", "Add the first node",
                       "Zorgcentrum Beta-Testerveen takes part, but has no node yet, so no query reaches it. Add its CDR as a node; you add the node's endpoints on the node.",
                       link_button("primary", "Add the first node", "node-new.html", "plus"))
    return org_page("zbt", "overview", body, toast_html=toasts(toast("ok", "circle-check", "Revision 8 applied", "Zorgcentrum Beta-Testerveen takes part. Add its first node.")))


def page_organisation_exclude():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="ex-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="ex-title" class="fui-dialog-title">{icon("ban", "18")} Exclude Ziekenhuis Voorbeeldstad?</h2>'
        '<p>Its 2 nodes stay configured but are asked nothing: a query names their endpoints excluded, and a routed read for its EHRs answers 404 until you include it again. Saved as a revision you review and apply.</p>'
        + field("ex-reason", "Reason", textarea("ex-reason", "CDR maintenance window, 21 to 23 October"), help_="Required. Kept with the revision and in the organisation's audit.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("primary", "Exclude and review", "config-review.html", "ban")}</div></div>'
    )
    return page_organisation(overlay=dialog)


# ---- one node ----

def sparkline(points, failing=(), budget=1250, w=320, h=56):
    far = max(points) < budget / 3
    mx = (max(points) * 1.5) if far else max(max(points), budget) * 1.1
    xs = [i * w / (len(points) - 1) for i in range(len(points))]
    path = " ".join(f'{"M" if i == 0 else "L"}{x:.1f} {h - p / mx * h:.1f}' for i, (x, p) in enumerate(zip(xs, points)))
    by = -10 if far else h - budget / mx * h
    marks = "".join(f'<circle class="fui-spark-fail" cx="{xs[i]:.1f}" cy="{h - points[i] / mx * h:.1f}" r="2.5"/>' for i in failing)
    return (f'<svg class="fui-spark" viewBox="0 0 {w} {h}" preserveAspectRatio="none" role="img" aria-label="The last {len(points)} probes">'
            f'<line class="fui-spark-budget" x1="0" x2="{w}" y1="{by:.1f}" y2="{by:.1f}"/><path class="fui-spark-line" d="{path}"/>{marks}</svg>')


OK_PROBES = [41, 38, 44, 39, 37, 42, 40, 36, 45, 39, 38, 41, 43, 37, 39, 40, 38, 42, 44, 39, 41, 38, 37, 40, 38, 39, 41, 42, 38, 38]
BAD_PROBES = [380, 410, 395, 420, 400, 388, 415, 402, 398, 430, 412, 405, 420, 2410, 2380, 2450, 2400, 2420, 2390, 2410, 2440, 2400, 2410, 2380, 2400, 2420, 2410, 2405, 2400, 2410]


def health_card(tone, probing=False):
    probes, failing = (OK_PROBES, ()) if tone == "ok" else (BAD_PROBES, tuple(range(13, 30)))
    line = ("Every probe of the last 30 minutes answered; p50 38 ms, p95 45 ms." if tone == "ok" else
            "node-b-pub answers 503 since 09:38: 17 of the last 30 probes failed.")
    return card("Health", f'<p class="fui-cluster">{state(tone, "503" if tone == "warn" else None)}<span class="fui-small fui-muted">the worst of its endpoints · last probe {when("2026-10-21T09:41:58", seconds=True)}, every 60 s</span></p>'
                + sparkline(probes, failing) + f'<p class="fui-spark-legend"><span>30 min ago</span><span>{"dashed: half the per-node budget" if tone != "ok" else "well under the 1,250 ms threshold"}</span><span>now</span></p>'
                + f'<p class="fui-small">{line} Rule: an answer under 500 within half the per-node budget is reachable; a 5xx or a slower answer is degraded; no answer within the budget is unreachable.</p>')


def ep_menu(eid, open_=False):
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="Actions for {eid}" title="More actions">{icon("ellipsis", "16")}</button>')
    m = menu([menu_item("Probe now", "activity", href="node-probing.html"), menu_item("Edit", "pencil", href="endpoint-new.html"), menu_item("Credentials…", "key-round", href="endpoint-credentials.html", focus=True),
              menu_item("Exclude…", "ban", href="node-exclude.html"), MENU_SEP, menu_item("Remove…", "trash-2", danger=True, href="node-remove.html")], label=f"Actions for {eid}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def endpoints_panel(node, menu_open=None):
    rows = []
    for x in node["endpoints"]:
        cred = (f'<span class="fui-secret-now">{icon("lock", "12")}OAuth 2.0 · set</span>' if x["id"] != "node-b-pub" else f'<span class="fui-secret-now">{icon("lock", "12")}Basic · set</span>') if x["kind"] == "query" else '<span class="mk-dash">none</span>'
        rows.append(f'<tr><th scope="row"><code>{x["id"]}</code></th>' + td("Connection", f'<code>{x["conn"]}</code>') + td("Base URL", f'<code class="fui-break">{e(x["url"])}</code>')
                    + td("State", state(x["tone"], "503" if x["tone"] == "warn" else None)) + td("p50", ms(x["p50"]), "fui-num") + td("Last probe", at(x["probe"])) + td("Credentials", cred)
                    + td("", ep_menu(x["id"], menu_open == x["id"]), "fui-table-actions") + "</tr>")
    return titled_panel(f'Endpoints <span class="mk-count">· how the gateway reaches this node</span>', link_button("secondary", "Add endpoint", "endpoint-new.html", "plus", small=True),
                        table("Endpoints of this node", th("Endpoint") + th("Connection") + th("Base URL") + th("State") + th("p50", num=True) + th("Last probe") + th("Credentials") + th('<span class="fui-sr-only">Actions</span>'), rows))


NODE_TABS = [("overview", "Overview", "node.html"), ("endpoints", "Endpoints", "node-endpoints.html"), ("routes", "Routes", "node-routes.html"), ("requests", "Requests", "node-requests.html"),
             ("audit", "Audit", "node-audit.html"), ("settings", "Settings", "node-settings.html")]


def node_page(nid="node-a", overlay=None, menu_open=False, top="", probing=False, toast_html="", ep_menu_open=None, tab="overview", panel=None, dock_html=""):
    o = next(o for o in ORGS for n in o["nodes"] if n["id"] == nid)
    n = next(n for n in o["nodes"] if n["id"] == nid)
    org_href = ORG_HREF[o["slug"]]
    probe_btn = busy_button("primary", "Probing") if probing else link_button("primary", "Probe now", "node-probing.html", "activity")
    ag = action_group(probe_btn, [menu_item("Edit", "pencil", href="node-edit.html", keys="e"), MENU_SEP, menu_item("Exclude…", "ban", href="node-exclude.html", hover=(menu_open == "hover")),
                                  menu_item("Remove…", "trash-2", danger=True, href="node-remove.html", hover=(menu_open == "danger"))],
                      open_=bool(menu_open), label=f"More actions for {nid}")
    head = page_heading(nid, mono=True, actions=ag, badge=state(n["tone"]),
                        subtitle=f'Node of <a href="{org_href}">{e(o["name"])}</a> · {e(n["product"])} · <code>{n["sid"]}</code>', crumbs=[("Organisations", "organisations.html"), (o["name"], org_href)])
    if panel is None:
        facts = facts_row([("System id", f'<code>{n["sid"]}</code>'), ("CDR", e(n["product"])), ("Organisation", f'<code>{o["id"]}</code>'), ("Endpoints", f'{len(n["endpoints"])} · <a href="node-endpoints.html">Endpoints tab</a>'),
                           ("In service", "yes"), ("Configured", '<a href="configuration-history.html">revision 6</a>')])
        routed = node_routes(nid)
        route_line = f'<p class="fui-small fui-muted"><a href="node-routes.html">{len(routed)} routes</a>: the creating_system_ids whose follow-up reads and writes go to this node.</p>'
        panel = f'<div class="fui-stack"><div class="fui-grid fui-grid-start">{card("Node", facts)}{health_card(n["tone"], probing)}</div>{route_line}</div>'
    t = tabs(nid, [(k, w, h, len(n["endpoints"])) if k == "endpoints" else ((k, w, h, len(node_routes(nid))) if k == "routes" else (k, w, h)) for k, w, h in NODE_TABS], tab)
    content = head + f'<div class="fui-stack">{top}{tabbed(t, panel)}</div>'
    return shell(nid, "orgs", content, overlay=overlay, org=o["slug"] if o["slug"] in ("zvs", "tko") else "zvs", node=nid, toast_html=toast_html, dock_html=dock_html)


# ---- a node's routes: the creating_system_ids that route to it ----

LEARNED_SEEN = {"lab-zvs.example.org": "09:12:40"}


def node_routes(nid):
    """(creating_system_id, ground, endpoint, last seen) for one node, from
    the registry: its own system_id, the [[creating_system]] mappings to its
    endpoints, and the learned map."""
    rows = [(c, g, ep) for c, g, n, ep in ROUTES if n == nid]
    if nid == "node-a":
        rows += [("his-legacy.zvs.example.org", "registered", "node-a-fhir")]
        rows += [("ward-%02d.zvs.example.org" % i, "learned", "node-a-pub") for i in range(1, 16)]
    order = {"node": 0, "registered": 1, "learned": 2}
    out = []
    for i, (c, g, ep) in enumerate(sorted(rows, key=lambda r: (order[r[1]], r[0]))):
        if g == "learned":
            ago = 120 + ((i * 677) % 9000)
            t = 9 * 3600 + 42 * 60 + 11 - ago
            seen = LEARNED_SEEN.get(c, "%02d:%02d:%02d" % (t // 3600, t % 3600 // 60, t % 60))
        else:
            seen = None
        out.append((c, g, ep, seen))
    return out


GROUND_LINE = {"node": "its own system_id, set in Settings", "registered": "a [[creating_system]] mapping", "learned": "seen in an answer"}


def route_menu(c, open_=False):
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
               f'aria-label="Actions for {e(c)}" title="More actions">{icon("ellipsis", "16")}</button>')
    m = menu([menu_item("Remove…", "trash-2", danger=True)], label=f"Actions for {c}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def routes_panel(nid="node-a", ground=None, menu_open=None):
    all_rows = node_routes(nid)
    rows = [r for r in all_rows if ground is None or r[1] == ground]
    body = []
    for c, g, ep, seen in rows:
        if g == "node":
            act = f'<a class="fui-icon-button" href="node-settings.html" aria-label="Change it in Settings" title="Set in Settings">{icon("settings", "16")}</a>'
        elif g == "registered":
            act = route_menu(c, menu_open == c)
        else:
            act = ""
        body.append(f'<tr><th scope="row" data-role="title"><code>{e(c)}</code></th>' + td("Ground", f'{tag(g)}<span class="fui-small fui-muted mk-ground-line">{GROUND_LINE[g]}</span>', role="sub")
                    + td("Endpoint", f"<code>{ep}</code>", role="meta-1") + td("Last seen", tpl.rel(seen) if seen else '<span class="mk-dash">–</span>', role="meta-q")
                    + td("", act, "fui-row-menu", role="menu") + "</tr>")
    counts = {k: sum(1 for r in all_rows if r[1] == k) for k in ("node", "registered", "learned")}
    chips = tpl.summary([(f"{len(all_rows)} routes", None, ground is None)] + [(f"{counts[k]} {k}", None, ground == k) for k in ("node", "registered", "learned")])
    tools = tpl.toolbar("A creating_system_id", "", [])
    act_chips = tpl.active([f"Ground: {ground}"]) if ground else ""
    head = (f'<div class="fui-panel-head"><p class="fui-small fui-muted mk-routes-intro">The creating_system_ids whose follow-up reads and writes go to this node. '
            f'Registered ones are changed here, as a revision with a reason; learned ones are read-only.</p>'
            f'<div class="fui-actions">{link_button("secondary", "Add system id", "node-routes-add.html", "plus", small=True)}</div></div>')
    filters = f'<div class="fui-panel-body fui-stack mk-routes-filters">{chips}{tools}{act_chips}</div>'
    tbl = table(f"Routes to {nid}", th("creating_system_id", "c", "ascending") + th("Ground") + th("Endpoint") + th("Last seen", title="Learned routes only") + th('<span class="fui-sr-only">Actions</span>'), body, scroll_label=f"Routes to {nid}")
    return f'<div class="fui-panel">{head}{filters}{tbl}</div>'


def page_node_routes():
    return node_page(tab="routes", panel=routes_panel())


def page_node_routes_filtered():
    return node_page(tab="routes", panel=routes_panel(ground="registered", menu_open="legacy-zvs.example.org"))


def page_node_routes_add():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="ra-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="ra-title" class="fui-dialog-title">{icon("route", "18")} Add a system id to node-a</h2>'
        '<p>Follow-up reads and writes of records this system created go to the endpoint you choose. Saved as a revision of the registry, with your reason.</p>'
        + field("ra-id", "creating_system_id", input_("ra-id", "pacs-zvs.example.org"), help_="As the records spell it; case does not matter. It may not be a node's own system_id or already routed.")
        + field("ra-ep", "Endpoint", select("ra-ep", [("node-a-pub", "node-a-pub · Query API"), ("node-a-fhir", "node-a-fhir · FHIR")], "node-a-pub"))
        + field("ra-reason", "Reason", textarea("ra-reason", "The PACS archive's studies are kept in node-a"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("primary", "Review the change", "node-routes-review.html", "git-compare")}</div></div>'
    )
    return node_page(tab="routes", panel=routes_panel(), overlay=dialog)


def page_node_routes_review():
    from pg_config import diff_table, form_page, review_dock
    from parts import revision_strip
    rows = [("creating_system[pacs-zvs.example.org].endpoint", "", "node-a-pub", "live")]
    eff = ('<p class="fui-small">What applying does: follow-up reads and writes of records that <code>pacs-zvs.example.org</code> created go to node-a through '
           '<code>node-a-pub</code>. No other route changes; the id is not a node\'s own system_id and no learned route names it.</p>')
    body = (revision_strip("draft", "Review your draft against revision 7", "The registry check passed: the endpoint exists, the id is new in any case, and it is no node's system_id. Save it as revision 8, or save and apply it in one step.")
            + card('What the draft adds <span class="mk-count">· 1 route</span>', diff_table(rows) + eff))
    return form_page("Review the new route", [("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a", "node-routes.html")],
                     "One registered mapping for node-a, against the revision that runs. Every replica follows what you apply.",
                     body, review_dock(reason="The PACS archive's studies are kept in node-a"), current="orgs", org="zvs", node="node-a")


def page_node_endpoints(ep_menu_open=None):
    n = ORGS[0]["nodes"][0]
    return node_page(tab="endpoints", panel=endpoints_panel(n, ep_menu_open))


def page_node_endpoint_menu():
    return page_node_endpoints("node-a-pub")


def page_node_requests():
    rows = [r for r in REQUESTS if r[1] in ("node-a-pub", "node-a-fhir")]
    return node_page(tab="requests", panel='<div class="fui-stack"><p class="fui-small fui-muted">What the gateway sent this node\'s endpoints: never a query\'s text, a parameter or a value.</p>' + requests_table(rows) + "</div>")


def page_node_audit():
    rows = [("2026-10-19T14:12", "Demo Operator", "revision 7", "per-node timeout raised for every node, node-a included", "Raise the per-node timeout for node-b's slower link"),
            ("2026-10-18T11:30", "Demo Operator", "probe", "probed node-a-fhir by hand: 200 in 24 ms", None),
            ("2026-10-15T17:30", "Demo Operator", "rollback", "credentials of node-a-pub back to revision 3's", "Roll back revision 4: the NVI rejected the new client key")]
    body = [f'<tr><th scope="row">{when(t)}</th>' + td("By", e(by), "mk-nowrap") + td("Kind", badge("neutral", k)) + td("What", e(w)) + td("Reason", e(r) if r else '<span class="mk-dash">–</span>') + "</tr>" for t, by, k, w, r in rows]
    return node_page(tab="audit", panel=table("Audit", th("When", "t", "descending") + th("By") + th("Kind") + th("What") + th("Reason"), body))


def node_settings_body(nid="node-a-lab", edit=False):
    def svc(name, href="service.html"):
        return f'<a href="{href}">{name}</a>'
    unmapped = nid == "node-a-lab"
    node = definitions([("Node id", f"<code>{nid}</code> · fixed"), ("System id", "<code>cdr-a-lab.example.org</code>" if unmapped else "<code>cdr-a.example.org</code>"),
                        ("CDR", "EHRbase 2.36.0" if unmapped else "FerroEHR 4.3.3"), ("In service", "yes")])
    mitz = ('<span class="fui-state fui-tone-warn is-long">' + health_dot("warn") + "Not mapped in Mitz: consent cannot be asked for this node</span>") if unmapped else "type V6 · URA <code>90000101</code>"
    if edit:
        mitz = (input_("ns-mitz", "90000107", extra='class="fui-input fui-mono fui-config-changed" aria-label="Mitz care provider URA"') + '<span class="fui-small fui-muted"> was not mapped · type V6</span>')
    rows = [
        ("PIXm resolver", "PIXm domain", "<code>urn:oid:2.999.27</code>" if unmapped else "<code>urn:oid:2.999.21</code>", "identity-services"),
        ("Mitz", "Care provider", mitz, "consent"),
        ("NVI", "Custodian URAs", "<code>90000101</code>", "localization"),
        ("XCPD", "Community", '<span class="mk-dash">XCPD is not set up</span>', "localization"),
    ]
    body = [f'<tr><th scope="row">{svc(s_)}</th>' + td("Entry", w) + td("Value", v) + td("Approval", badge("neutral", a)) + "</tr>" for s_, w, v, a in rows]
    ic = card("Identity and consent", '<p class="fui-small fui-muted">This node\'s entry in each service\'s map: the service itself is set on its own page.</p>'
              + table("Identity and consent", th("Service") + th("Entry") + th("Value") + th("Approval"), body))
    fhir = card("FHIR face", definitions([("Serves", "laboratory reports" if unmapped else "patient summaries, electronic prescriptions"), ("Set in", "<code>[[fhir.serve]]</code>, this node's entry")]))
    strip = revision_strip("draft", f"Draft of {nid} · 1 change", "Approved by a second operator (consent), then on trial for 2 minutes after the apply.") if edit else ""
    return f'<div class="fui-stack">{strip}{card("Node", node)}{ic}{fhir}</div>'


def page_node_settings():
    top = notice("warn", "triangle-alert", "<strong>Not mapped in Mitz.</strong> Consent cannot be asked for node-a-lab, so the pre-filter names it not-asked in every query.",
                 action=" " + link_button("secondary", "Map it", "node-settings-edit.html", "pencil", small=True))
    return node_page("node-a-lab", tab="settings", panel=node_settings_body(), top=top)


def page_node_settings_edit():
    d = dock(dock_field("ns-reason", "Reason", "Laboratory CDR registered with Mitz under the hospital's URA", "Why this changes", keys="r"),
             link_button("secondary", "Discard the draft", "node-settings.html", "x") + link_button("primary", "Review the changes", "config-review.html", "git-compare"))
    return node_page("node-a-lab", tab="settings", panel=node_settings_body(edit=True), dock_html=d)


def page_node():
    return node_page()


def page_node_degraded():
    top = notice("warn", "triangle-alert", "<strong>Degraded.</strong> Its endpoint node-b-pub answers 503 Service Unavailable since 09:38. A query in its scope comes back incomplete under best effort, and fails under the gateway's all-or-nothing default.")
    return node_page("node-b", top=top)


def page_node_probing():
    t = toasts(toast("ok", "activity", "Probing node-a's 2 endpoints", "Each answer updates its state and the topology; it takes up to the per-node budget of 2.5 s."))
    return node_page(probing=True, toast_html=t)


def page_node_exclude():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="nx-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="nx-title" class="fui-dialog-title">{icon("ban", "18")} Exclude node-a?</h2>'
        '<p>The gateway stops asking its 2 endpoints: a query names them excluded and their rows are missing, said so in every answer. Its endpoints, credentials and routes are kept. Saved as a revision you review and apply; Include undoes it the same way.</p>'
        + field("nx-reason", "Reason", textarea("nx-reason", "CDR upgrade tonight, 22:00 to 23:30"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("primary", "Exclude and review", "config-review.html", "ban")}</div></div>'
    )
    return node_page(overlay=dialog)


def page_node_remove():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="nr-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="nr-title" class="fui-dialog-title">{icon("trash-2", "18")} Remove node-a?</h2>'
        '<p>Ziekenhuis Voorbeeldstad keeps node-a-lab, but no request reaches node-a any more: its 2 endpoints and their credentials go with it, and 2 creating_system_ids that route here answer 404 until another node claims them. Saved as a revision you review and apply.</p>'
        + field("nr-reason", "Reason", textarea("nr-reason", "The CDR is replaced by node-a2, added in revision 8"), help_="Required.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("danger", "Remove and review", "config-review.html", "trash-2")}</div></div>'
    )
    return node_page(overlay=dialog)


# ---- forms: organisation, node, endpoint ----

def form_page(title, crumbs, sub, body, dock_html, org=None, node=None):
    content = page_heading(title, subtitle=sub, crumbs=crumbs) + f'<div class="fui-stack">{body}</div>'
    return shell(title, "orgs", content, dock_html=dock_html, org=org, node=node)


def rdock(reason, disabled=False, cancel=("Cancel", "organisations.html")):
    return dock(dock_field("f-reason", "Reason", reason, "Why this changes; kept with the revision", keys="r"),
                link_button("secondary", cancel[0], cancel[1], "x") + (button("primary", "Review the changes", "git-compare", attrs="disabled") if disabled else link_button("primary", "Review the changes", "config-review.html", "git-compare")))


def page_organisation_new():
    form = ('<div class="fui-form"><div class="fui-columns">'
            + field("on-id", "Organisation id", input_("on-id", "org-zbt", extra='spellcheck="false" class="fui-input fui-mono"'), help_="1 to 64 letters, digits, ‘.’, ‘-’ or ‘_’. Fixed once saved.")
            + field("on-name", "Name", input_("on-name", "Zorgcentrum Beta-Testerveen"))
            + "</div></div>")
    body = revision_strip("draft", "Draft based on revision 7 · 1 organisation added", "Its nodes come next, inside the organisation, once it runs.") + card("Organisation", form)
    return form_page("New organisation", [("Organisations", "organisations.html")], "An organisation first; then its nodes, then their endpoints.", body,
                     rdock("Zorgcentrum Beta-Testerveen joins the federation in November"))


def node_form(errors=None):
    errors = errors or {}

    def f(id_, label, val, help_=None, mono=True, key=None):
        inv = ' aria-invalid="true"' if errors.get(key) else ""
        return field(id_, label, input_(id_, val, extra=f'spellcheck="false" class="fui-input{" fui-mono" if mono else ""}"{inv}'), help_=help_, error=errors.get(key))
    org_fact = facts_row([("Organisation", "Zorgcentrum Beta-Testerveen"), ("Id", "<code>org-zbt</code>"), ("Nodes so far", "none")])
    node = ('<div class="fui-columns">' + f("nn-id", "Node id", "node-f", "1 to 64 letters, digits, ‘.’, ‘-’ or ‘_’. Fixed once saved.", key="id")
            + f("nn-sid", "System id", "cdr-c.example.org" if errors else "cdr-f.example.org", "The openEHR system_id the node names in every version it writes.", key="sid")
            + f("nn-prod", "CDR product", "FerroEHR", mono=False) + f("nn-ver", "CDR product version", "4.3.3", mono=False) + "</div>")
    ep = ('<p class="fui-small fui-muted">A node needs one endpoint to be asked. Give its first one here; add more on the node.</p><div class="fui-columns">'
          + f("ne-id", "Endpoint id", "node-f-pub", key="eid") + f("ne-url", "Base URL", "cdr-f.example.org/openehr" if errors else "https://cdr-f.example.org/openehr", key="url")
          + field("ne-conn", "Connection type", select("ne-conn", [("q", "openehr-rest-query"), ("f", "hl7-fhir-rest")], "q")) + "</div>"
          + check("ne-svc", "In service", True, "Out of service, the gateway asks it nothing and names it excluded.", extra='class="fui-switch"'))
    after = '<p class="fui-small fui-muted">Credentials are set on the endpoint once the node runs: they take a second operator\'s approval, so they are a revision of their own.</p>'
    return card("Organisation", org_fact) + card("Node", node) + card("First endpoint", ep + after)


def page_node_new():
    body = revision_strip("draft", "Draft based on revision 8 · 1 node added to Zorgcentrum Beta-Testerveen", "Checked as you type; every rule ferrofed config check applies runs again before you can save.") + node_form()
    return form_page("Add a node", [("Organisations", "organisations.html"), ("Zorgcentrum Beta-Testerveen", "organisation-empty.html")],
                     "Added inside its organisation, which is set and shown here, never chosen.", body,
                     rdock("First node of Zorgcentrum Beta-Testerveen", cancel=("Cancel", "organisation-empty.html")), org="zbt")


def page_node_new_invalid():
    errs = {"url": "The base URL is not an absolute http or https URL.", "sid": "node-c already names cdr-c.example.org: each node's system_id is its own."}
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 2 fields need a change before you can review</p>'
               '<ul><li><a href="#nn-sid">System id: node-c already names it</a></li><li><a href="#ne-url">Base URL: not an absolute URL</a></li></ul></div>')
    return form_page("Add a node", [("Organisations", "organisations.html"), ("Zorgcentrum Beta-Testerveen", "organisation-empty.html")],
                     "Added inside its organisation, which is set and shown here, never chosen.", summary + node_form(errs),
                     rdock("First node of Zorgcentrum Beta-Testerveen", disabled=True, cancel=("Cancel", "organisation-empty.html")), org="zbt")


def page_node_edit():
    fields = ('<div class="fui-columns">'
              + field("ed-id", "Node id", input_("ed-id", "node-a", extra='class="fui-input fui-mono" disabled'), help_=f"{icon('lock', '12')} A node's id is its identity: add a new node to change it.")
              + field("ed-sid", "System id", input_("ed-sid", "cdr-a.example.org", extra='class="fui-input fui-mono"'))
              + field("ed-prod", "CDR product", input_("ed-prod", "FerroEHR")) + field("ed-ver", "CDR product version", input_("ed-ver", "4.3.4", extra='class="fui-input fui-config-changed"'), help_="was 4.3.3")
              + "</div>" + check("ed-svc", "In service", True, "Out of service, the gateway asks none of its endpoints.", extra='class="fui-switch"'))
    body = revision_strip("draft", "Draft of node-a · 1 change", "Based on revision 7. Its endpoints are edited on the node, each in its own row.") + card("Node", fields)
    return form_page("Edit node-a", [("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a", "node.html")],
                     "Ziekenhuis Voorbeeldstad · your change is a draft until you review and save it.", body,
                     rdock("node-a upgraded to FerroEHR 4.3.4", cancel=("Cancel", "node.html")), org="zvs", node="node-a")


def page_endpoint_new():
    cats = "".join(f'<label class="fui-chip-check"><input type="checkbox"{" checked" if c in ("Patient summaries", "Laboratory reports") else ""} />{c}</label>'
                   for c in ["Patient summaries", "Electronic prescriptions", "Electronic dispensations", "Laboratory reports", "Medical imaging", "Discharge reports"])
    fields = (facts_row([("Node", "<code>node-a</code>"), ("Organisation", "Ziekenhuis Voorbeeldstad"), ("Endpoints so far", "node-a-pub, node-a-fhir")])
              + '<div class="fui-columns">'
              + field("ae-id", "Endpoint id", input_("ae-id", "node-a-fhir-2", extra='spellcheck="false" class="fui-input fui-mono"'))
              + field("ae-url", "Base URL", input_("ae-url", "https://cdr-a.example.org/fhir/r4", extra='spellcheck="false" class="fui-input fui-mono"'))
              + field("ae-conn", "Connection type", select("ae-conn", [("f", "hl7-fhir-rest"), ("q", "openehr-rest-query")], "f"))
              + field("ae-sys", "Identifier system of the ehr_id", input_("ae-sys", "urn:oid:2.999.2.1", extra='class="fui-input fui-mono"'), help_="The system the endpoint is told the ehr_id in.")
              + "</div>" + f'<div class="fui-field"><span class="fui-label" id="ae-cats">Categories</span><div class="fui-chip-checks" role="group" aria-labelledby="ae-cats">{cats}</div></div>')
    body = revision_strip("draft", "Draft based on revision 7 · 1 endpoint added to node-a", "The managing organisation is the node's own.") + card("Endpoint", fields)
    return form_page("Add an endpoint", [("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a", "node.html")],
                     "How the gateway reaches node-a: added on the node, which is set and shown here.", body,
                     rdock("Second FHIR endpoint for the R4 laboratory reports", cancel=("Cancel", "node.html")), org="zvs", node="node-a")


def switcher():
    from data import large_orgs
    items = [("Ziekenhuis Voorbeeldstad", "2 nodes", "ok", True), ("Zorggroep Proefdorp", "1 node", "ok", False), ("Zorgcentrum Beta-Testerveen", "no node yet", "neutral", False)]
    big = [(o["name"], f'{o["n"]} node{"s" if o["n"] != 1 else ""}', o["worst"], False) for o in large_orgs()[:30] if "Proef" in o["name"] or "Ziekenhuis" in o["name"]][:3]
    def li(n, line, t, sel):
        q = "Proef" if "Proef" in n else ("Zie" if n.startswith("Zie") else "")
        name = e(n).replace(q, f"<mark>{q}</mark>", 1) if q else e(n)
        return (f'<li role="option" aria-selected="{"true" if sel else "false"}">{health_dot(t)}<span class="fui-switcher-text"><strong>{name}</strong><span>{line} · {STATE_WORD[t]}</span></span>'
                f'{icon("arrow-right", "14") if sel else ""}</li>')
    recent = "".join(li(*x) for x in items[:1])
    rest = "".join(li(*x) for x in items[1:] + big)
    return ('<div role="dialog" aria-modal="true" aria-label="Jump to organisation" class="fui-switcher mk-open">'
            f'<div class="fui-switcher-field">{icon("search", "16")}<input class="fui-input" type="search" value="proef" aria-label="Organisation name or id" aria-controls="sw-list" aria-activedescendant="sw-0" autocomplete="off" /><kbd class="fui-kbd">Esc</kbd></div>'
            f'<p class="fui-switcher-head">Current</p><ul class="fui-switcher-list" role="listbox" id="sw-list" aria-label="Organisations">{recent}</ul>'
            f'<p class="fui-switcher-head">Matching “proef” · 4 of 30</p><ul class="fui-switcher-list" role="listbox" aria-label="Matching organisations">{rest}</ul>'
            '<p class="fui-switcher-foot"><span><kbd class="fui-kbd">↑</kbd><kbd class="fui-kbd">↓</kbd> move</span><span><kbd class="fui-kbd">Enter</kbd> open</span><span><kbd class="fui-kbd">G</kbd><kbd class="fui-kbd">O</kbd> from anywhere</span></p></div>')


def page_quick_switcher():
    return page_organisation(overlay=switcher())


def page_node_more():
    return node_page(menu_open="danger")
