# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Organisations, their nodes, and each node's endpoints: the one home of
each, in FerroFED's own model ([[organisation]], [[node]], [[endpoint]]).
An organisation is created on the Organisations page with its first node
and that node's first endpoint (an organisation manages at least one
endpoint, registry §8.1); further nodes are added, edited, excluded and
removed inside it; a node's endpoints, and each endpoint's credentials,
live on the node. Every object's tabs lead to its own routes (routes.py)."""
from common import (
    action_group, MENU_SEP, badge, busy_button, button, card, conn, definitions, dock, dock_field, e, empty_state,
    field, health_dot, hint, icon, input_, link_button, menu, menu_item, notice, page_heading, pager, pill,
    panel_head, quiet_line, rel, section, sections, select, shell, state, table, tabbed, tabs, tag, td, textarea, th, toast,
    toasts, url_code, when,
)
import data
from data import (CONN_WORD, EXCLUDED_WHY, N_ENDPOINTS, N_NODES, N_ORGS, ORGS, STATE_WORD, ZBT, large_counts,
                  large_orgs, node_counts, org_counts, org_tone, routes_of, word, worst)
import routes
from routes import node_href, org_href
from pg_shell import tech_detail, unreachable_block
import tpl

SIDEBAR_ORGS = ("zvs", "tko", "zbt", "zgp", "rdw", "htm")


def ms(v):
    return f"{v:,} ms" if v is not None else '<span class="mk-dash">–</span>'


def org_find(slug):
    return data.org(slug)


def n_nodes(k):
    return f"{k} node{'s' if k != 1 else ''}"


def org_state_line(o):
    c = {}
    for n in o["nodes"]:
        c[n["tone"]] = c.get(n["tone"], 0) + 1
    return " · ".join(f"{k} {word(t)}" for t, k in sorted(c.items(), key=lambda x: data.STATE_ORDER[x[0]]))


# ---- the Organisations page: Organisations and All nodes (T1) ----

def org_menu(o, open_=False):
    name = o["name"]
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
               f'aria-label="Actions for {e(name)}" title="More actions">{icon("ellipsis", "16")}</button>')
    m = menu([menu_item("Open", "arrow-right", keys="Enter", href=org_href(o["slug"])), menu_item("Its nodes", "server", href=org_href(o["slug"], "nodes"), hover=True),
              MENU_SEP, menu_item("Exclude…", "ban", href="organisation-exclude.html", danger=True)], label=f"Actions for {name}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def org_row(o, hover=False, menu_open=False, menus=True, cols=("id", "nodes", "state", "probe")):
    t = org_tone(o)
    probe = max((n["probe"] for n in o["nodes"] if n["probe"]), default=None)
    cls = " ".join(x for x in ["mk-hover" if hover else "", "is-excluded" if t == "muted" else ""] if x)
    cells = ""
    if "id" in cols:
        cells += td("Id", f'<code>{o["id"]}</code>', role="sub")
    if "nodes" in cols:
        cells += td("Nodes", n_nodes(len(o["nodes"])), role="meta-q")
    if "state" in cols:
        cells += td("Worst node", state(t), role="meta-2")
    if "probe" in cols:
        cells += td("Last probe", rel(probe), role="meta-q")
    if "from" in cols:
        pass
    return (f'<tr{f" class={chr(34)}{cls}{chr(34)}" if cls else ""}><th scope="row" data-role="title"><a class="fui-row-link" href="{org_href(o["slug"])}">{e(o["name"])}</a></th>'
            + cells + (td("", org_menu(o, menu_open), "fui-row-menu", role="menu") if menus else "") + "</tr>")


def org_head(menus=True):
    return (th("Organisation", "name", "ascending") + th("Id") + th("Nodes") + th("Worst node", title="The worst state of its nodes")
            + th("Last probe") + (th('<span class="fui-sr-only">Actions</span>') if menus else ""))


def org_rows(hover=None, menu_open=None, menus=True):
    return [org_row(o, i == hover, menu_open == i, menus) for i, o in enumerate(ORGS)]


def orgs_table(rows, menus=True):
    return table("Organisations", org_head(menus), rows, scroll_label="Organisations")


def orgs_page(tab="orgs", body=None, tools="", chips="", toast_html="", readonly=False, strip="", org_count=N_ORGS, node_count=N_NODES,
              summary_html="", foot="", counts=True, status="up"):
    oc = org_count if counts else None
    nc = node_count if counts else None
    t = tabs("Organisations", [("orgs", "Organisations", "organisations.html", oc), ("nodes", "All nodes", "organisations-nodes.html", nc)], tab)
    table_html = body if body is not None else orgs_table(org_rows())
    cardh = tpl.card(table_html, foot, tabs_html=t, label="Organisations")
    act = "" if readonly else link_button("primary", "New organisation", "organisation-new.html", "plus")
    head = page_heading("Organisations", actions=act, subtitle="The organisations in the federation and their nodes.")
    content = head + strip + tpl.list_body(summary_html, tools, chips, cardh)
    return shell("Organisations", "orgs", content, toast_html=toast_html, status=status)


def page_organisations():
    return orgs_page()


def page_organisations_hover():
    return orgs_page(body=orgs_table(org_rows(hover=1)))


def page_organisations_menu():
    return orgs_page(body=orgs_table(org_rows(hover=1, menu_open=1)))


# A federation of 30: past ten organisations the list takes its summary
# strip and its find field (T1); 10.3, 10.4 and 10.8 share this one set.
BIG = large_orgs()
BIG_NODES = sum(o["n"] for o in BIG)


def big_rows(items):
    rows = []
    for o in items:
        exc = o["worst"] == "muted"
        rows.append(f'<tr{" class=" + chr(34) + "is-excluded" + chr(34) if exc else ""}><th scope="row" data-role="title"><a class="fui-row-link" href="organisation.html">{e(o["name"])}</a></th>'
                    + td("Id", f'<code>{o["id"]}</code>', role="sub") + td("Nodes", n_nodes(o["n"]), role="meta-q") + td("Worst node", state(o["worst"]), role="meta-2")
                    + td("Last probe", rel(o["probe"]), role="meta-q") + td("", org_menu({"name": o["name"], "slug": "zvs"}), "fui-row-menu", role="menu") + "</tr>")
    return rows


def big_summary(pressed="all"):
    c = large_counts(BIG)
    chips = [(f"{len(BIG)} organisations", None, pressed == "all")]
    for t in ("warn", "bad", "neutral", "muted"):
        if c.get(t):
            chips.append((f"{c[t]} {word(t)}", t if t != "muted" else None, pressed == t))
    return tpl.summary(chips)


def big_toolbar(value=""):
    return tpl.toolbar("Organisation name or id", value)


def page_organisations_filtered():
    o = [x for x in BIG if x["worst"] == "warn"]
    return orgs_page(body=orgs_table(big_rows(o)), tools=big_toolbar(), chips=tpl.active(["Worst node: degraded"], shown=(len(o), len(BIG))),
                     summary_html=big_summary("warn"), org_count=len(BIG), node_count=BIG_NODES)


def page_organisations_no_match():
    body = quiet_line("No unreachable organisation has “kliniek” in its name.", "Clear the filters", "organisations-large.html")
    return orgs_page(body=body, tools=big_toolbar("kliniek"), chips=tpl.active(["Worst node: unreachable", "Name: kliniek"], shown=(0, len(BIG))),
                     summary_html=big_summary("bad"), org_count=len(BIG), node_count=BIG_NODES)


def page_organisations_loading():
    from parts import skeleton_lines
    rows = "".join(f'<tr aria-hidden="true"><th data-role="title">{skeleton_lines([80])}</th><td data-role="sub">{skeleton_lines([50])}</td><td data-role="meta-q">{skeleton_lines([30])}</td>'
                   f'<td data-role="meta-2">{skeleton_lines([60])}</td><td data-role="meta-q">{skeleton_lines([40])}</td><td></td></tr>' for _ in range(5))
    return orgs_page(body=table("Organisations, loading", org_head(), [rows]), counts=False)


def page_organisations_unreachable():
    head = page_heading("Organisations", subtitle="The organisations in the federation and their nodes.")
    content = head + '<div class="fui-list-page">' + unreachable_block("the organisations") + tech_detail(show_rid=False) + "</div>"
    return shell("Organisations", "orgs", content, status="down", incidents=None)


def page_organisations_empty():
    body = quiet_line("No organisation takes part yet.", "Add the first organisation", "organisation-new.html")
    return orgs_page(body=body, org_count=0, node_count=0)


def all_node_rows():
    rows = []
    items = [(o, n) for o in ORGS for n in o["nodes"]]
    for o, n in sorted(items, key=lambda x: data.STATE_ORDER[x[1]["tone"]]):
        exc = n["tone"] == "muted"
        rows.append(f'<tr{" class=" + chr(34) + "is-excluded" + chr(34) if exc else ""}><th scope="row" data-role="title"><a class="fui-row-link" href="{node_href(n["id"])}"><code>{n["id"]}</code></a></th>'
                    + td("Organisation", e(o["name"]), role="sub") + td("System id", f'<code>{n["sid"]}</code>', role="wide")
                    + td("Endpoints", str(len(n["endpoints"])), "fui-num", role="wide") + td("State", state(n["tone"]), role="meta-2")
                    + td("Median response", ms(n["p50"]), "fui-num", role="wide")
                    + td("Last probe", ("probed " + rel(n["probe"])) if n["probe"] else '<span class="mk-dash">never probed</span>', role="meta-q") + "</tr>")
    return rows


def page_organisations_nodes():
    body = table("Every node", th("Node") + th("Organisation") + th("System id") + th("Endpoints", num=True) + th("State", "state", "ascending", title="Worst first")
                 + th("Median response", num=True, title="p50 over the last 30 probes") + th("Last probe"), all_node_rows())
    return orgs_page("nodes", body=body)


def page_organisations_large():
    return orgs_page(body=orgs_table(big_rows(BIG[:25])), org_count=len(BIG), node_count=BIG_NODES, summary_html=big_summary(),
                     tools=big_toolbar(), foot=pager("Organisations", len(BIG), 25, 1, sizes=(25, 50, 100)))


def page_organisations_directory():
    from parts import revision_strip
    strip = revision_strip("running", "Read from the care services directory",
                           f'This gateway reads its registry from {url_code("https://mcsd.demo-regio.example.org/fhir")}{hint("[registry.mcsd]")}. Change an organisation, a node or an endpoint there; the console shows them read-only.')
    rows = [org_row(o, menus=False).replace(f'<a class="fui-row-link" href="{org_href(o["slug"])}">{e(o["name"])}</a>',
                                            f'<a class="fui-row-link" href="{org_href(o["slug"])}">{e(o["name"])}</a> <span class="fui-tag fui-tag-locked">{icon("lock", "12")}Directory</span>') for o in ORGS]
    return orgs_page(body=orgs_table(rows, menus=False), readonly=True, strip=strip)


def page_organisations_file_mode():
    strip = (f'<p class="fui-small fui-muted mk-file-line">{icon("lock", "12")} Set in the registry document <code>/etc/ferrofed/registry.json</code>{hint("[registry] document")} · read at start · '
             f'<a href="configuration-file-mode-dialog.html">How configuration works</a></p>')
    return orgs_page(body=orgs_table(org_rows(menus=False), menus=False), readonly=True, strip=strip)


# ---- one organisation (T2) ----

def org_page(slug, tab, panel, overlay=None, menu_open=False, top="", toast_html="", dock_html="", drafts=(), hide_primary=False):
    o = org_find(slug)
    own = slug == "zvs"
    items = [menu_item("Edit", "pencil", href="organisation-settings-edit.html" if own else None, keys="e"), MENU_SEP,
             menu_item("Exclude…", "ban", href="organisation-exclude.html" if own else None, danger=True), menu_item("Remove…", "trash-2", danger=True)]
    primary = "" if hide_primary else (link_button("primary", "New node", "node-new.html", "plus") if slug == "zgp" else button("primary", "New node", "plus"))
    ag = "" if hide_primary else action_group(primary, items, open_=menu_open, label=f"More actions for {o['name']}")
    head = page_heading(e(o["name"]), actions=ag, badge=state(org_tone(o)),
                        subtitle=f'Organisation{hint(o["id"])} · {n_nodes(len(o["nodes"]))}', crumbs=[("Organisations", "organisations.html")])
    items_ = []
    for k in routes.ORG_TABS:
        words = {"overview": "Overview", "nodes": "Nodes", "requests": "Requests", "audit": "Audit", "settings": "Settings"}[k]
        items_.append((k, words + ('<span class="fui-tab-draft">draft</span>' if k in drafts else ""), org_href(slug, k), len(o["nodes"]) if k == "nodes" else None))
    t = tabs(o["name"], items_, tab)
    content = head + f'<div class="fui-stack">{top}{tabbed(t, panel)}</div>'
    return shell(o["name"], "orgs", content, overlay=overlay, toast_html=toast_html, org=slug, dock_html=dock_html)


ORG_STATS = {
    "zvs": ("1,284 to its endpoints · 3 refused", "212", 0), "tko": ("418 to node-b-pub · 31 answered 503", "57", 1),
    "zgp": ("640 to node-c-pub · none refused", "96", 1), "rdw": ("305 to node-d-pub · 1 refused", "40", 0),
    "htm": ("none: it is excluded", "0", 0), "zbt": ("none yet", "0", 0),
}


def org_overview(slug="zvs"):
    o = org_find(slug)
    added = data.REVISIONS + data.FUTURE
    rev = next(r for r in added if r[0] == o["added"])
    facts = definitions([("State", state(org_tone(o)) + (f' <span class="fui-small fui-muted">{EXCLUDED_WHY}</span>' if o["standing"] != "active" else "")),
                         ("Nodes", f'{" ".join(health_dot(n["tone"]) for n in o["nodes"])} {org_state_line(o)}'),
                         ("Endpoints", str(sum(len(n["endpoints"]) for n in o["nodes"]))),
                         ("Added", f'{when(rev[3])} · <a href="{routes.compare_href(rev[0])}">revision {rev[0]}</a>')])
    req, routed, inc = ORG_STATS[slug]
    inc_line = (f'<a href="integrity.html">1 open incident</a> names this organisation' if inc else "No open incident names this organisation")
    act = definitions([("Requests", req), ("Routed reads", f"{routed}, sent to the node that owns the record"), ("Integrity", inc_line)])
    return sections(section("Organisation", facts), section("Last 24 hours", act, link_button("ghost", "Requests", org_href(slug, "requests"), glyph_end="chevron-right", small=True)))


def page_organisation(menu_open=False, overlay=None):
    return org_page("zvs", "overview", org_overview("zvs"), menu_open=menu_open, overlay=overlay)


def page_organisation_actions():
    return page_organisation(menu_open=True)


def page_organisation_degraded():
    n = data.node("node-b")
    top = notice("warn", "triangle-alert", f'<strong>1 node is degraded.</strong> <code>node-b</code> answers 503 since {n["since"]} today.',
                 action=" " + link_button("secondary", "Open node-b", node_href("node-b"), small=True))
    return org_page("tko", "overview", org_overview("tko"), top=top)


def node_menu(n, open_=False):
    nid = n["id"]
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="Actions for {nid}" title="More actions">{icon("ellipsis", "16")}</button>')
    own = nid == "node-a"
    m = menu([menu_item("Open", "arrow-right", href=node_href(nid)), menu_item("Probe now", "activity", href="node-probing.html" if own else None), menu_item("Edit", "pencil", href="node-edit.html" if own else None, keys="e"),
              MENU_SEP, menu_item("Exclude…", "ban", href="node-exclude.html" if own else None, hover=True), menu_item("Remove…", "trash-2", danger=True, href="node-remove.html" if own else None)], label=f"Actions for {nid}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def ep_tags(n):
    return '<span class="mk-ep-tags">' + "".join(f'<a class="fui-tag" href="{node_href(n["id"], "endpoints")}">{x["id"]}</a>' for x in n["endpoints"]) + "</span>"


def org_nodes_panel(slug, menu_for=None):
    o = org_find(slug)
    rows = [f'<tr{" class=" + chr(34) + "is-excluded" + chr(34) if n["tone"] == "muted" else ""}><th scope="row" data-role="title"><a class="fui-row-link" href="{node_href(n["id"])}"><code>{n["id"]}</code></a></th>'
            + td("System id", f'<code>{n["sid"]}</code>', role="sub") + td("CDR", e(n["product"]), role="meta-q") + td("Endpoints", ep_tags(n), role="meta-q")
            + td("State", state(n["tone"]), role="meta-2") + td("Last probe", rel(n["probe"]), role="meta-q") + td("", node_menu(n, n["id"] == menu_for), "fui-row-menu", role="menu") + "</tr>" for n in o["nodes"]]
    return ('<div class="fui-worklist" aria-label="Nodes">'
            + table("Nodes of this organisation", th("Node", "node", "ascending") + th("System id") + th("CDR") + th("Endpoints") + th("State") + th("Last probe") + th('<span class="fui-sr-only">Actions</span>'), rows)
            + "</div>")


def page_organisation_nodes():
    return org_page("zvs", "nodes", org_nodes_panel("zvs", "node-a-lab"))


# ---- requests: what the gateway sent, never a value ----

REQ_SHAPES = [
    ("09:42:07", "AQL query", "200", 41, "3 rows"), ("09:41:55", "stored query vital_signs 1.1.0", "200", 38, "6 rows"),
    ("09:41:20", "Patient/$summary", "200", 212, "1 bundle"), ("09:40:48", "routed read, composition", "200", 29, "1 composition"),
    ("09:39:02", "AQL query", "504", 2500, "time-out"), ("09:37:44", "AQL query", "200", 74, "0 rows"),
    ("09:36:10", "AQL query", "200", 52, "2 rows"), ("09:35:31", "routed read, EHR status", "200", 18, "1 EHR status"),
    ("09:34:02", "stored query problem_list 2.0.0", "200", 66, "4 rows"), ("09:33:47", "AQL query", "200", 44, "1 row"),
]
RIDS = ["01JAF3M2C9QK", "01JAF3M1Z7PD", "01JAF3KZW2HT", "01JAF3KX0D5A", "01JAF3KQ8B1N", "01JAF3KJ4M7E", "01JAF3KF2W9R", "01JAF3KC7H3T", "01JAF3K9P5LM", "01JAF3K6V1XB"]


def requests_for(eps):
    out = []
    for i, (t, r, st, lat, a) in enumerate(REQ_SHAPES):
        ep = eps[i % len(eps)]
        if ep["kind"] == "fhir" and not r.startswith("Patient"):
            r, a = "Patient/$summary", "1 bundle"
        if ep["kind"] == "query" and r.startswith("Patient"):
            r, a = "AQL query", "2 rows"
        if ep["tone"] == "warn":
            st, lat, a = "503", 388, "node error"
        out.append((t, ep["id"], r, st, lat, a, RIDS[i]))
    return out


def requests_table(rows, who="Endpoint", total=None):
    def tone(st):
        return "ok" if st.startswith("2") else "bad"
    body = [f'<tr><th scope="row">{rel(t)}</th>' + (td(who, f"<code>{s}</code>") if who else "") + td("Request", e(r))
            + td("Status", pill(tone(st), st)) + td("Latency", f"{lat:,} ms", "fui-num") + td("Answer", e(a)) + td("Request id", f"<code>{rid}</code>") + "</tr>"
            for t, s, r, st, lat, a, rid in rows]
    tbl = table("Recent requests", th("Time", "t", "descending") + (th(who) if who else "") + th("Request") + th("Status") + th("Latency", num=True) + th("Answer") + th("Request id"), body)
    return tbl + (f'<div class="fui-list-foot">{pager("Requests", total, 10, 1)}</div>' if total and total > 10 else "")


def requests_panel(eps, total, intro, filters, who="Endpoint"):
    tools = tpl.toolbar("A request id", "", filters)
    return sections(f'<div class="fui-panel-section"><p class="fui-small fui-muted">{intro}</p>{tools}</div>' + requests_table(requests_for(eps), who, total))


REQ_INTRO = "Patient identifiers and values are not shown here; the access record keeps them. Last 24 hours."


def org_requests_panel(slug):
    o = org_find(slug)
    eps = [ep for n in o["nodes"] for ep in n["endpoints"] if ep["tone"] != "muted"]
    if not eps:
        return quiet_line(f"{o['name']} is excluded, so the gateway sent it nothing.", "Its audit", org_href(slug, "audit"))
    total = {"zvs": 1284, "tko": 418, "zgp": 640, "rdw": 305}.get(slug, 0)
    return requests_panel(eps, total, REQ_INTRO, [tpl.filter_button("Node"), tpl.filter_button("Status")])


def page_organisation_requests():
    return org_page("zvs", "requests", org_requests_panel("zvs"))


# ---- audit: every operator action, from the one revision history ----

def audit_events(org_slug=None, nid=None):
    """(at, by, kind, what, reason, revision) newest first."""
    ev = []
    for o in ORGS:
        if org_slug and o["slug"] != org_slug:
            continue
        for n in o["nodes"]:
            if nid and n["id"] != nid:
                continue
            r = next(x for x in data.REVISIONS if x[0] == n["added"])
            eps = ", ".join(x["id"] for x in n["endpoints"])
            ev.append((r[3], r[2], "Change", f"added {n['id']} with {eps}", r[4], r[0]))
        if not nid and o["added"] != "1" or (not nid and o["added"] == "1"):
            pass
    r7 = data.REVISIONS[6]
    if org_slug or nid:
        ev.append((r7[3], r7[2], "Change", "raised the per-node timeout for every node, 2,000 to 2,500 ms", r7[4], "7"))
    if org_slug == "htm" or nid == "node-e":
        r6 = data.REVISIONS[5]
        ev.append((r6[3], r6[2], "Exclusion", "excluded Huisartsenpost Testmeer and its node", "Its CDR moves to a new host", "6"))
    if org_slug == "zvs" or nid == "node-a":
        ev.append(("2026-10-18T11:30", "Demo Operator", "Probe", "probed node-a-fhir by hand: 200 in 24 ms", None, None))
    return sorted(ev, key=lambda x: x[0], reverse=True)


def audit_table(ev):
    body = [f'<tr><th scope="row">{rel(t)}</th>' + td("By", e(by), "mk-nowrap") + td("Kind", e(k))
            + td("What", e(w) + (f' · <a class="mk-nowrap" href="{routes.compare_href(rv)}">revision {rv}</a>' if rv else "")) + td("Reason", e(r) if r else '<span class="mk-dash">–</span>') + "</tr>"
            for t, by, k, w, r, rv in ev]
    return table("Audit", th("When", "t", "descending") + th("By") + th("Kind") + th("What") + th("Reason"), body)


def audit_panel(ev, intro):
    tools = tpl.toolbar("Who, what or a revision number", "", [tpl.filter_button("Kind")])
    return sections(f'<div class="fui-panel-section"><p class="fui-small fui-muted">{intro}</p>{tools}</div>' + audit_table(ev))


def page_organisation_audit():
    return org_page("zvs", "audit", audit_panel(audit_events("zvs"), "Every operator action on this organisation, its nodes and their endpoints, newest first."))


# ---- settings ----

def ic_rows(n, edit=False):
    """This node's entry in each service's map, with the approval it takes."""
    ura = "90000101" if n["org"]["slug"] == "zvs" else {"tko": "90000102", "zgp": "90000103", "rdw": "90000104", "htm": "90000105"}.get(n["org"]["slug"], "90000106")
    unmapped = n["id"] == "node-a-lab"
    mitz = (f'<span class="fui-badge fui-tone-warn">Not mapped</span> <span class="fui-small fui-muted">consent cannot be asked for this node</span>' if unmapped else f"Care provider URA <code>{ura}</code>")
    if edit:
        mitz = (input_("ns-mitz", ura, extra='class="fui-input fui-mono fui-config-changed" aria-label="Mitz care provider URA"') + '<span class="fui-small fui-muted"> was not mapped</span>')
    pix = {"node-a": "21", "node-a-lab": "27", "node-b": "22", "node-c": "23", "node-d": "24", "node-e": "25", "node-f": "26"}[n["id"]]
    ids = "Identity services, localization and consent"
    return [
        ("pixm", "PIXm resolver", "Domain", f"<code>urn:oid:2.999.{pix}</code>", "Identity services", "identity-services"),
        ("mitz", "Mitz", "Care provider", mitz, ids, "identity-services, localization, consent"),
        ("nvi", "NVI", "Custodian", f"URA <code>{ura}</code>", ids, "identity-services, localization, consent"),
        ("xcpd", "XCPD", "Community", '<span class="mk-dash">XCPD is not set up</span>', "Identity services and localization", "identity-services, localization"),
    ]


def ic_table(n, edit=False):
    body = [f'<tr><th scope="row"><a href="{routes.svc_href(k) if k != "xcpd" else "service-new.html"}">{name}</a></th>' + td("Entry", w) + td("Value", v) + td("Approval", e(a) + hint(code)) + "</tr>"
            for k, name, w, v, a, code in ic_rows(n, edit)]
    return table("Identity and consent", th("Service") + th("Entry") + th("Value") + th("Approval"), body)


def org_settings_body(edit=False):
    o = org_find("zvs")
    roll = table("Its nodes' entries", th("Node") + th("PIXm domain") + th("Mitz care provider") + th("NVI custodian"),
                 [f'<tr><th scope="row"><a href="{node_href(n["id"], "settings")}"><code>{n["id"]}</code></a></th>'
                  + "".join(td(lbl, ic_rows(n)[i][3]) for i, lbl in ((0, "PIXm domain"), (1, "Mitz"), (2, "NVI"))) + "</tr>" for n in o["nodes"]])
    rollup = section("Set on each node", '<p class="fui-small fui-muted">A read-only roll-up: each entry is changed on its node\'s Settings tab.</p>' + roll)
    if not edit:
        d = definitions([("Name", "Ziekenhuis Voorbeeldstad"), ("Organisation id", "<code>org-zvs</code> · fixed: every revision names the organisation by it")])
        head = panel_head("Its name and id. A change is saved as a revision you review and apply.",
                                  link_button("secondary", "Edit", "organisation-settings-edit.html", "pencil", small=True, keys="e"))
        return sections(head + section("Organisation", d), rollup)
    form = ('<div class="fui-form"><div class="fui-columns">'
            + field("os-name", "Name", input_("os-name", "Ziekenhuis Voorbeeldstad Noord", extra='class="fui-input fui-config-changed"'), help_="was Ziekenhuis Voorbeeldstad")
            + field("os-id", "Organisation id", input_("os-id", "org-zvs", extra='class="fui-input fui-mono" disabled'), help_=f"{icon('lock', '12')} Fixed: every revision names the organisation by it.")
            + "</div></div>")
    head = panel_head("A draft of this tab, based on revision 7: 1 change.", '')
    return sections(head + section("Organisation", form), rollup)


def page_organisation_settings():
    return org_page("zvs", "settings", org_settings_body())


def draft_dock(reason, discard_href, review_href="config-review.html", disabled=False, primary="Review the changes"):
    p = button("primary", primary, "git-compare", attrs='disabled aria-describedby="dock-why"') if disabled else link_button("primary", primary, review_href, "git-compare")
    msg = '<p id="dock-why" class="fui-small fui-muted">Fix the fields marked above first.</p>' if disabled else ""
    return dock(dock_field("f-reason", "Reason", reason, "Required: kept with the revision", keys="r"),
                link_button("secondary", "Discard", discard_href, "x") + p, msg)


def page_organisation_settings_edit():
    d = draft_dock("The hospital merged with its northern site and renamed", "organisation-settings.html")
    return org_page("zvs", "settings", org_settings_body(edit=True), dock_html=d, drafts=("settings",), hide_primary=True)


def page_organisation_empty():
    """Zorgcentrum Beta-Testerveen, just created with its first node: the
    landing page of revision 8, its node not probed yet."""
    t = toasts(toast("ok", "circle-check", "Revision 8 applied", "Zorgcentrum Beta-Testerveen takes part. node-f is probed within a minute."))
    return org_page("zbt", "overview", org_overview("zbt"), toast_html=t)


def page_organisation_exclude():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="ex-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="ex-title" class="fui-dialog-title">{icon("ban", "18")} Exclude Ziekenhuis Voorbeeldstad?</h2>'
        '<p>The gateway stops asking its 2 nodes. Every answer names their endpoints as excluded, and a read of a record they hold answers 404 until you include it again. Its nodes, endpoints and credentials are kept.</p>'
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
    return (f'<svg class="fui-spark" viewBox="0 0 {w} {h}" preserveAspectRatio="none" role="img" aria-label="The last {len(points)} probes, response time in milliseconds">'
            f'<line class="fui-spark-budget" x1="0" x2="{w}" y1="{by:.1f}" y2="{by:.1f}"/><path class="fui-spark-line" d="{path}"/>{marks}</svg>')


OK_PROBES = [41, 38, 44, 39, 37, 42, 40, 36, 45, 39, 38, 41, 43, 37, 39, 40, 38, 42, 44, 39, 41, 38, 37, 40, 38, 39, 41, 42, 38, 38]
# node-b: answered every probe until 09:38; the 4 probes since (09:38:58 to
# 09:41:58, one a minute) came back 503.
BAD_PROBES = [380, 410, 395, 420, 400, 388, 415, 402, 398, 430, 412, 405, 420, 391, 386, 397, 402, 388, 410, 395, 399, 404, 390, 396, 401, 392, 385, 390, 388, 386]
BAD_FAIL = (26, 27, 28, 29)
RULE_LINE = ("Rule: a successful answer (below status 500) within half the per-node budget, 1,250 ms, is reachable; a 5xx answer or a slower one is degraded; "
             "no answer within the budget is unreachable.")


def health_section(n):
    if n["tone"] == "neutral":
        return section("Health", f'<p>{state("neutral")} <span class="fui-small fui-muted">Added today; the first probe runs within a minute.</span></p>')
    if n["tone"] == "muted":
        return section("Health", f'<p>{state("muted")} <span class="fui-small fui-muted">Asked nothing and not probed while it is excluded: {EXCLUDED_WHY.lower()}.</span></p>')
    bad = n["tone"] == "warn"
    probes, failing = (BAD_PROBES, BAD_FAIL) if bad else (OK_PROBES, ())
    line = (f"{n['endpoints'][0]['id']} answers 503 since {n['since']}: the last 4 probes failed." if bad else "Every probe of the last 30 minutes answered; median 38 ms.")
    return section("Health", f'<p class="fui-cluster">{state(n["tone"], "503" if bad else None)}<span class="fui-small fui-muted">the worst of its endpoints · last probe {rel(n["probe"])}, every 60 s</span></p>'
                   + sparkline(probes, failing) + f'<p class="fui-spark-legend"><span>30 min ago</span><span>{"a dot: a failed probe" if bad else "dashed: 1,250 ms"}</span><span>now</span></p>'
                   + f'<p class="fui-small">{line} {RULE_LINE}</p>')


def ep_menu(ep, open_=False):
    eid = ep["id"]
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="Actions for {eid}" title="More actions">{icon("ellipsis", "16")}</button>')
    own = eid in ("node-a-pub", "node-a-fhir")
    items = [menu_item("Probe now", "activity", href="node-probing.html" if own else None), menu_item("Edit", "pencil", href="endpoint-edit.html" if eid == "node-a-pub" else None)]
    if ep["kind"] == "query":
        items.append(menu_item("Credentials…", "key-round", href="endpoint-credentials.html" if own else None, focus=True))
    items += [MENU_SEP, menu_item("Exclude…", "ban", href="endpoint-exclude.html" if eid == "node-a-pub" else None), menu_item("Remove…", "trash-2", danger=True, href="endpoint-remove.html" if eid == "node-a-fhir" else None)]
    m = menu(items, label=f"Actions for {eid}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def cred_cell(ep):
    if ep["kind"] != "query":
        return '<span class="fui-small fui-muted">the openEHR endpoint\'s</span>'
    if not ep["cred"]:
        return '<span class="mk-dash">none</span>'
    grant, owner = ep["cred"]
    where = "set in ferrofed.toml" if owner == "file" else "set"
    return f'<span class="fui-secret-now">{icon("lock", "12")}{grant} · {where}</span>'


def endpoints_panel(n, menu_open=None):
    rows = []
    for x in n["endpoints"]:
        cls = ' class="is-excluded"' if x["tone"] == "muted" else ""
        rows.append(f'<tr{cls}><th scope="row" data-role="title"><code>{x["id"]}</code></th>' + td("Connection", conn(x["kind"]), role="sub") + td("Base URL", url_code(x["url"]), role="wide")
                    + td("State", state(x["tone"], "503" if x["tone"] == "warn" else None), role="meta-2") + td("Median response", ms(x["p50"]), "fui-num", role="meta-q")
                    + td("Last probe", rel(x["probe"]), role="meta-q") + td("Credentials", cred_cell(x), role="wide")
                    + td("", ep_menu(x, menu_open == x["id"]), "fui-row-menu", role="menu") + "</tr>")
    tbl = table("Endpoints of this node", th("Endpoint") + th("Connection") + th("Base URL") + th("State") + th("Median response", num=True, title="p50 over the last 30 probes")
                + th("Last probe") + th("Credentials") + th('<span class="fui-sr-only">Actions</span>'), rows)
    return sections(section("Endpoints", f'<div class="fui-worklist">{tbl}</div>', link_button("secondary", "New endpoint", "endpoint-new.html", "plus", small=True), count=len(n["endpoints"])))


def node_page(nid="node-a", overlay=None, menu_open=False, top="", probing=False, toast_html="", tab="overview", panel=None, dock_html=""):
    n = data.node(nid)
    o = n["org"]
    own = nid == "node-a"
    probe_btn = busy_button("primary", "Probing") if probing else (link_button("primary", "Probe now", "node-probing.html", "activity") if own else button("primary", "Probe now", "activity", attrs='disabled title="node-e is excluded: include it before probing"' if n["tone"] == "muted" else ""))
    ag = action_group(probe_btn, [menu_item("Edit", "pencil", href="node-edit.html" if own else None, keys="e"), MENU_SEP, menu_item("Exclude…", "ban", href="node-exclude.html" if own else None, hover=(menu_open == "hover")),
                                  menu_item("Remove…", "trash-2", danger=True, href="node-remove.html" if own else None, hover=(menu_open == "danger"))],
                      open_=bool(menu_open), label=f"More actions for {nid}")
    head = page_heading(nid, mono=True, actions=ag, badge=state(n["tone"]),
                        subtitle=f'Node of <a href="{org_href(o["slug"])}">{e(o["name"])}</a> · {e(n["product"])}', crumbs=[("Organisations", "organisations.html"), (o["name"], org_href(o["slug"]))])
    if panel is None:
        panel = node_overview(n)
    words = {"overview": "Overview", "endpoints": "Endpoints", "routes": "Routes", "requests": "Requests", "audit": "Audit", "settings": "Settings"}
    items = []
    for k in routes.NODE_TABS:
        cnt = len(n["endpoints"]) if k == "endpoints" else (len(routes_of(nid)) if k == "routes" else None)
        items.append((k, words[k], node_href(nid, k), cnt))
    t = tabs(nid, items, tab)
    content = head + f'<div class="fui-stack">{top}{tabbed(t, panel)}</div>'
    return shell(nid, "orgs", content, overlay=overlay, org=o["slug"], node=nid, toast_html=toast_html, dock_html=dock_html)


def node_overview(n):
    rev = next(r for r in data.REVISIONS + data.FUTURE if r[0] == n["added"])
    facts = definitions([("System id", f'<code>{n["sid"]}</code>'), ("CDR", e(n["product"])), ("Organisation", f'<a href="{org_href(n["org"]["slug"])}">{e(n["org"]["name"])}</a>'),
                         ("Endpoints", f'<a href="{node_href(n["id"], "endpoints")}">{len(n["endpoints"])}</a>'), ("Added", f'{when(rev[3])} · <a href="{routes.compare_href(rev[0])}">revision {rev[0]}</a>')])
    k = len(routes_of(n["id"]))
    route_line = f'<p class="fui-small fui-muted"><a href="{node_href(n["id"], "routes")}">{k} creating system{"s" if k != 1 else ""}</a> route their follow-up reads and writes to this node.</p>'
    return sections(section("Node", facts + route_line), health_section(n))


# ---- a node's routes ----

GROUND_WORD = {"node": "Its own system id", "registered": "Registered", "learned": "Learned"}
GROUND_LINE = {"node": "set when the node is added or edited", "registered": "a mapping in the registry, changed here", "learned": "seen in an answer, read-only"}
LEARNED_SEEN = {"lab-zvs.example.org": "09:12:40"}


def route_rows(nid):
    out = []
    for i, (c, g, _, ep) in enumerate(routes_of(nid)):
        seen = None
        if g == "learned":
            ago = 120 + ((i * 677) % 9000)
            t = 9 * 3600 + 42 * 60 + 11 - ago
            seen = LEARNED_SEEN.get(c, "%02d:%02d:%02d" % (t // 3600, t % 3600 // 60, t % 60))
        out.append((c, g, ep, seen))
    return out


def route_menu(c, g, open_=False):
    if g == "learned":
        return ""
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
               f'aria-label="Actions for {e(c)}" title="More actions">{icon("ellipsis", "16")}</button>')
    if g == "node":
        items = [menu_item("Change it in Edit node", "pencil", href="node-edit.html")]
    else:
        items = [menu_item("Change the endpoint…", "pencil", href="node-routes-add.html"), MENU_SEP, menu_item("Remove…", "trash-2", danger=True)]
    m = menu(items, label=f"Actions for {c}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def routes_panel(nid="node-a", ground=None, menu_open=None):
    all_rows = route_rows(nid)
    rows = [r for r in all_rows if ground is None or r[1] == ground]
    body = []
    for c, g, ep, seen in rows:
        body.append(f'<tr><th scope="row" data-role="title"><code>{e(c)}</code></th>' + td("Ground", f'{e(GROUND_WORD[g])}<span class="fui-small fui-muted mk-ground-line">{GROUND_LINE[g]}</span>', role="sub")
                    + td("Endpoint", f"<code>{ep}</code>", role="meta-q") + td("Last seen", ("seen " + rel(seen)) if seen else '<span class="mk-dash">–</span>', role="meta-q")
                    + td("", route_menu(c, g, menu_open == c), "fui-row-menu", role="menu") + "</tr>")
    counts = {k: sum(1 for r in all_rows if r[1] == k) for k in ("node", "registered", "learned")}
    chips = tpl.summary([(f"{len(all_rows)} routes", None, ground is None)] + [(f"{counts[k]} {GROUND_WORD[k].lower() if k != 'node' else 'own system id'}", None, ground == k) for k in ("node", "registered", "learned") if counts[k]])
    many = len(all_rows) > 10
    tools = tpl.toolbar("A creating system", "") if many else ""
    intro = ('<p class="fui-small fui-muted">The creating systems whose follow-up reads and writes go to this node. Registered ones are changed here, as a revision with a reason; learned ones are read-only.</p>')
    head = section("Routes", intro + (f'<div class="fui-stack">{chips}{tools}</div>' if many else ""), link_button("secondary", "New route", "node-routes-add.html", "plus", small=True),
                   count=len(all_rows) if ground is None else f"showing {len(rows)} of {len(all_rows)}")
    tbl = table(f"Routes to {nid}", th('Creating system', "c", "ascending", title="creating_system_id") + th("Ground") + th("Endpoint") + th("Last seen", title="Learned routes only") + th('<span class="fui-sr-only">Actions</span>'), body, scroll_label=f"Routes to {nid}")
    return sections(head.replace("</section>", f'<div class="fui-worklist">{tbl}</div></section>'))


def page_node_routes():
    return node_page(tab="routes", panel=routes_panel())


def page_node_routes_filtered():
    return node_page(tab="routes", panel=routes_panel(ground="registered", menu_open="legacy-zvs.example.org"))


def page_node_routes_add():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="ra-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="ra-title" class="fui-dialog-title">{icon("route", "18")} New route to node-a</h2>'
        '<p>Follow-up reads and writes of records this system created go to node-a. Saved as a revision of the registry, with your reason.</p>'
        + field("ra-id", f'Creating system{hint("creating_system_id")}', input_("ra-id", "pacs-zvs.example.org", extra='spellcheck="false" class="fui-input fui-mono"'),
                help_="As the record ids spell it; case does not matter. It may not be a node's own system id, or routed already.")
        + f'<div class="fui-field"><span class="fui-label">Endpoint</span><p><code>node-a-pub</code> · {conn("query")}</p><p class="fui-help">A route goes to an openEHR Query API endpoint, and node-a has one.</p></div>'
        + field("ra-reason", "Reason", textarea("ra-reason", "The PACS archive's studies are kept in node-a"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("primary", "Review the change", "node-routes-review.html", "git-compare")}</div></div>'
    )
    return node_page(tab="routes", panel=routes_panel(), overlay=dialog)


def page_node_routes_review():
    from pg_config import diff_table, review_dock
    rows = [("A new route", "creating_system[pacs-zvs.example.org].endpoint", "", "node-a-pub", "live")]
    eff = ('<p class="fui-small">What applying does: follow-up reads and writes of records that <code>pacs-zvs.example.org</code> created go to node-a through '
           '<code>node-a-pub</code>. No other route changes; the id is no node\'s own system id and no learned route names it.</p>')
    body = sections(section("What the draft adds", diff_table(rows), count="1 route")
                    + section("Checks", '<p class="fui-small">Checked as <code>ferrofed config check</code> would: no problems. The endpoint exists, the id is new in any case, and it is no node\'s system id.</p>' + eff))
    content = page_heading("Review the new route", subtitle="Against revision 7, the one that runs.", crumbs=[("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a", "node-routes.html")]) + body
    return shell("Review the new route", "orgs", content, dock_html=review_dock(reason="The PACS archive's studies are kept in node-a", back="node-routes-add.html", apply_href="config-apply.html"), org="zvs", node="node-a")


def page_node_endpoints(ep_menu_open=None):
    return node_page(tab="endpoints", panel=endpoints_panel(data.node("node-a"), ep_menu_open))


def page_node_endpoint_menu():
    return page_node_endpoints("node-a-pub")


def node_requests_panel(nid):
    n = data.node(nid)
    eps = [ep for ep in n["endpoints"] if ep["tone"] != "muted"]
    if not eps:
        return quiet_line(f"{nid} is excluded, so the gateway sent it nothing.", "Its audit", node_href(nid, "audit"))
    total = {"node-a": 1061, "node-a-lab": 223, "node-b": 418, "node-c": 640, "node-d": 305}.get(nid, 0)
    return requests_panel(eps, total, REQ_INTRO, [tpl.filter_button("Endpoint"), tpl.filter_button("Status")] if len(eps) > 1 else [tpl.filter_button("Status")])


def page_node_requests():
    return node_page(tab="requests", panel=node_requests_panel("node-a"))


def page_node_audit():
    return node_page(tab="audit", panel=audit_panel(audit_events(nid="node-a"), "Every operator action on node-a and its endpoints, newest first."))


def node_settings_body(nid="node-a-lab", edit=False):
    n = data.node(nid)
    intro = ("This node's entry in each service's map; each service itself is set on its own page. The node's id, system id and CDR are on Edit node."
             if not edit else "A draft of this tab, based on revision 7: 1 change.")
    act = '' if edit else (link_button("secondary", "Edit", "node-settings-edit.html", "pencil", small=True, keys="e") if nid == "node-a-lab" else button("secondary", "Edit", "pencil", small=True))
    head = panel_head(intro, act)
    fhir = {"node-a": ("Serves", "Electronic prescriptions", "[[fhir.serve]]"), "node-a-lab": ("Stores received", "Laboratory reports", "[[fhir.receive]]")}.get(nid)
    fh = section("Exchange-format face", definitions([(fhir[0], fhir[1] + hint(fhir[2]))])) if fhir else ""
    return sections(head + section("Identity and consent", ic_table(n, edit)), fh)


def page_node_settings():
    top = notice("warn", "triangle-alert", "<strong>Not mapped in Mitz.</strong> Consent cannot be asked for node-a-lab, so the pre-filter does not ask about it and says so in every answer (not asked).",
                 action=" " + link_button("secondary", "Map it", "node-settings-edit.html", "pencil", small=True))
    return node_page("node-a-lab", tab="settings", panel=node_settings_body(), top=top)


def page_node_settings_edit():
    d = draft_dock("Laboratory CDR registered with Mitz under the hospital's URA", "node-settings.html")
    return node_page("node-a-lab", tab="settings", panel=node_settings_body(edit=True), dock_html=d).replace('data-tab="settings">Settings', 'data-tab="settings">Settings<span class="fui-tab-draft">draft</span>', 1)


def page_node():
    return node_page()


def page_node_degraded():
    top = notice("warn", "triangle-alert", "<strong>Degraded.</strong> Its endpoint node-b-pub answers 503 Service Unavailable since 09:38. A query that asks it fails under the all-or-nothing default (424) and comes back incomplete under best effort.")
    return node_page("node-b", top=top)


def page_node_probing():
    t = toasts(toast("ok", "activity", "Probing node-a's 2 endpoints", "Each answer updates its state and the topology, within the per-node budget of 2.5 s."))
    return node_page(probing=True, toast_html=t)


def page_node_exclude():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="nx-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="nx-title" class="fui-dialog-title">{icon("ban", "18")} Exclude node-a?</h2>'
        '<p>The gateway stops asking its 2 endpoints. Every answer names them as excluded, so a client knows their rows are missing. Its endpoints, credentials and routes are kept, and Include undoes it the same way.</p>'
        + field("nx-reason", "Reason", textarea("nx-reason", "CDR upgrade tonight, 22:00 to 23:30"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("primary", "Exclude and review", "config-review.html", "ban")}</div></div>'
    )
    return node_page(overlay=dialog)


def page_node_remove():
    rs = route_rows("node-a")
    reg = sum(1 for r in rs if r[1] == "registered")
    learned = sum(1 for r in rs if r[1] == "learned")
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="nr-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="nr-title" class="fui-dialog-title">{icon("trash-2", "18")} Remove node-a?</h2>'
        f'<p>Ziekenhuis Voorbeeldstad keeps node-a-lab. node-a\'s 2 endpoints and their credentials go with it. Its {reg} registered routes answer 404 until you map them to another node, and its {learned} learned routes are withdrawn.</p>'
        + field("nr-reason", "Reason", textarea("nr-reason", "The CDR is replaced by node-a2"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("danger", "Remove and review", "config-review.html", "trash-2")}</div></div>'
    )
    return node_page(overlay=dialog)


def page_node_more():
    return node_page(menu_open="danger")


# ---- forms: organisation, node, endpoint ----

def form_page(title, crumbs, sub, body, dock_html, org=None, node=None):
    content = page_heading(title, subtitle=sub, crumbs=crumbs) + f'<div class="fui-stack">{body}</div>'
    return shell(title, "orgs", content, dock_html=dock_html, org=org, node=node)


def f_input(id_, label, val, help_=None, mono=True, error=None):
    inv = ' aria-invalid="true"' if error else ""
    return field(id_, label, input_(id_, val, extra=f'spellcheck="false" class="fui-input{" fui-mono" if mono else ""}"{inv}'), help_=help_, error=error)


def node_fields(prefix, nid, sid, product, version, errors=None):
    errors = errors or {}
    return ('<div class="fui-columns">' + f_input(f"{prefix}-id", "Node id", nid, "Letters, digits, ‘.’, ‘-’ or ‘_’, up to 64. Fixed once saved.")
            + f_input(f"{prefix}-sid", "System id", sid, "The openEHR system id the node writes in every version it creates.", error=errors.get("sid"))
            + f_input(f"{prefix}-prod", "CDR product", product, mono=False) + f_input(f"{prefix}-ver", "CDR product version", version, mono=False) + "</div>")


def first_endpoint_fields(prefix, eid, url, errors=None):
    errors = errors or {}
    return ('<div class="fui-columns">' + f_input(f"{prefix}-eid", "Endpoint id", eid) + f_input(f"{prefix}-url", "Base URL", url, error=errors.get("url"))
            + f'<div class="fui-field"><span class="fui-label">Connection</span><p>{conn("query")}</p><p class="fui-help">A node\'s first endpoint is its openEHR Query API; add a FHIR endpoint on the node.</p></div></div>')


CRED_AFTER = '<p class="fui-small fui-muted">Credentials are set on the endpoint once it runs, from its row on the node.</p>'


def page_organisation_new():
    org_f = ('<div class="fui-columns">' + f_input("on-id", "Organisation id", "org-zbt", "Letters, digits, ‘.’, ‘-’ or ‘_’, up to 64. Fixed once saved.")
             + f_input("on-name", "Name", "Zorgcentrum Beta-Testerveen", mono=False) + "</div>")
    body = sections(
        f'<div class="fui-panel-section"><p class="fui-small fui-muted">A draft based on revision 7. An organisation takes part with at least one node and its endpoint, so all three are saved together.</p></div>'
        + section("Organisation", org_f) + section("Its first node", node_fields("nf", "node-f", "cdr-f.example.org", "FerroEHR", "4.3.3"))
        + section("The node's first endpoint", first_endpoint_fields("nf", "node-f-pub", "https://cdr-f.example.org/openehr") + CRED_AFTER))
    return form_page("New organisation", [("Organisations", "organisations.html")], "Its first node and endpoint come with it.", body,
                     draft_dock(data.REASON_8, "organisations.html"))


def node_new(errors=None):
    zgp = org_find("zgp")
    errors = errors or {}
    summary = ""
    if errors:
        summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 2 fields need a change before you can review</p>'
                   '<ul><li><a href="#nn-sid">System id: node-c already names it</a></li><li><a href="#nn-url">Base URL: not an absolute URL</a></li></ul></div>')
    body = summary + sections(
        f'<div class="fui-panel-section"><p class="fui-small fui-muted">A draft based on revision 7, for {e(zgp["name"])}. Every rule <code>ferrofed config check</code> applies runs again before you can save.</p></div>'
        + section("Node", node_fields("nn", "node-c2", "cdr-c.example.org" if errors else "cdr-c2.example.org", "FerroEHR", "4.3.3", errors))
        + section("Its first endpoint", first_endpoint_fields("nn", "node-c2-pub", "cdr-c2.example.org/openehr" if errors else "https://cdr-c2.example.org/openehr", errors) + CRED_AFTER))
    return form_page("New node", [("Organisations", "organisations.html"), (zgp["name"], org_href("zgp"))],
                     f"Added to {e(zgp['name'])}, which is set here.", body,
                     draft_dock("A second CDR for the group's nursing homes", org_href("zgp"), disabled=bool(errors)), org="zgp")


def page_node_new():
    return node_new()


def page_node_new_invalid():
    return node_new({"url": "The base URL is not an absolute http or https URL.", "sid": "node-c already names cdr-c.example.org: each node's system id is its own."})


def page_node_edit():
    fields = ('<div class="fui-columns">'
              + field("ed-id", "Node id", input_("ed-id", "node-a", extra='class="fui-input fui-mono" disabled'), help_=f"{icon('lock', '12')} A node's id is its identity: add a new node to change it.")
              + field("ed-sid", "System id", input_("ed-sid", "cdr-a.example.org", extra='class="fui-input fui-mono"'), help_="Its own route: follow-ups of records it created go here.")
              + field("ed-prod", "CDR product", input_("ed-prod", "FerroEHR")) + field("ed-ver", "CDR product version", input_("ed-ver", "4.3.4", extra='class="fui-input fui-config-changed"'), help_="was 4.3.3")
              + "</div>")
    body = sections(f'<div class="fui-panel-section"><p class="fui-small fui-muted">A draft based on revision 7: 1 change. To stop asking the node, use Exclude… in its title row.</p></div>' + section("Node", fields))
    return form_page("Edit node-a", [("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a", "node.html")],
                     "Its endpoints are edited on the node, each in its own row.", body,
                     draft_dock("node-a upgraded to FerroEHR 4.3.4", "node.html"), org="zvs", node="node-a")


CATS = ["Patient summaries", "Electronic prescriptions", "Electronic dispensations", "Laboratory reports", "Medical imaging", "Discharge reports"]


def fhir_fields(prefix, eid, url, cats, profile, system, edit_note=None):
    chips = "".join(f'<label class="fui-chip-check"><input type="checkbox"{" checked" if c in cats else ""} />{c}</label>' for c in CATS)
    return ('<div class="fui-columns">'
            + f_input(f"{prefix}-id", "Endpoint id", eid) + f_input(f"{prefix}-url", "Base URL", url)
            + field(f"{prefix}-conn", "Connection", select(f"{prefix}-conn", [("f", "FHIR REST (hl7-fhir-rest)"), ("q", "openEHR Query API (openehr-rest-query)")], "f"))
            + f_input(f"{prefix}-sys", "EHR id system", system, "The system the gateway writes the member's EHR id in.")
            + "</div>"
            + f'<div class="fui-field"><span class="fui-label" id="{prefix}-cats">Categories it serves</span><div class="fui-chip-checks" role="group" aria-labelledby="{prefix}-cats">{chips}</div>'
              '<p class="fui-help">One native endpoint per category on a node; node-a-fhir serves patient summaries.</p></div>'
            + f_input(f"{prefix}-prof", "Profiles", profile, "The profiles its documents follow, one per line; each category's own profile is required."))


def page_endpoint_new():
    body = sections(f'<div class="fui-panel-section"><p class="fui-small fui-muted">A draft based on revision 7, on node-a of Ziekenhuis Voorbeeldstad.</p></div>'
                    + section("Endpoint", fhir_fields("ae", "node-a-lab-reports", "https://cdr-a.example.org/fhir/r4", ["Laboratory reports"],
                                                      "http://hl7.eu/fhir/laboratory/StructureDefinition/Composition-eu-lab", "urn:oid:2.999.2.1")))
    return form_page("New endpoint", [("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a", "node-endpoints.html")],
                     "How the gateway reaches node-a; added on the node.", body,
                     draft_dock("node-a serves its laboratory reports in the exchange format", "node-endpoints.html"), org="zvs", node="node-a")


def page_endpoint_edit():
    fields = ('<div class="fui-columns">'
              + field("ee-id", "Endpoint id", input_("ee-id", "node-a-pub", extra='class="fui-input fui-mono" disabled'), help_=f"{icon('lock', '12')} Fixed: clients name it in every query that targets it.")
              + field("ee-url", "Base URL", input_("ee-url", "https://cdr-a.example.org/openehr/v1", extra='spellcheck="false" class="fui-input fui-mono fui-config-changed"'), help_="was https://cdr-a.example.org/openehr")
              + f'<div class="fui-field"><span class="fui-label">Connection</span><p>{conn("query")}</p></div>'
              + "</div>")
    body = sections(f'<div class="fui-panel-section"><p class="fui-small fui-muted">A draft based on revision 7: 1 change. Its credentials are changed with Credentials… in its row.</p></div>' + section("Endpoint", fields))
    return form_page("Edit node-a-pub", [("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a", "node-endpoints.html")],
                     "An endpoint of node-a.", body, draft_dock("The CDR moved its API under /v1", "node-endpoints.html"), org="zvs", node="node-a")


def page_endpoint_exclude():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="ex-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="ex-title" class="fui-dialog-title">{icon("ban", "18")} Exclude node-a-pub?</h2>'
        '<p>The gateway stops asking this endpoint, and every answer names it as excluded. node-a-fhir keeps serving patient summaries, with the credentials of this endpoint. Its credentials and routes are kept.</p>'
        + field("epx-reason", "Reason", textarea("epx-reason", "Certificate renewal on the CDR, 30 minutes"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("primary", "Exclude and review", "config-review.html", "ban")}</div></div>'
    )
    return page_node_endpoints(None).replace('</div></body></html>', f'<div class="mk-scrim" aria-hidden="true"></div>{dialog}</div></body></html>').replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1)


def page_endpoint_remove():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="epr-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="epr-title" class="fui-dialog-title">{icon("trash-2", "18")} Remove node-a-fhir?</h2>'
        '<p>node-a\'s patient summaries are then mapped from its openEHR records, as every other member\'s are. node-a-pub stays. Its categories and profiles go with it.</p>'
        + field("epr-reason", "Reason", textarea("epr-reason", "The CDR stops serving the exchange format itself"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("danger", "Remove and review", "config-review.html", "trash-2")}</div></div>'
    )
    return page_node_endpoints(None).replace('</div></body></html>', f'<div class="mk-scrim" aria-hidden="true"></div>{dialog}</div></body></html>').replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1)


# ---- route-only pages: every other object's tabs ----

def route_pages():
    """slug → page function, for every tab of every object that has no
    captured screen."""
    out = {}

    def add(kind, key, tab, fn):
        s = routes.slug(kind, key, tab)
        if s.startswith("r-"):
            out[s] = fn
    for o in ORGS + [ZBT]:
        sl = o["slug"]
        add("org", sl, "overview", lambda sl=sl: org_page(sl, "overview", org_overview(sl)))
        add("org", sl, "nodes", lambda sl=sl: org_page(sl, "nodes", org_nodes_panel(sl)))
        add("org", sl, "requests", lambda sl=sl: org_page(sl, "requests", org_requests_panel(sl)))
        add("org", sl, "audit", lambda sl=sl: org_page(sl, "audit", audit_panel(audit_events(sl), "Every operator action on this organisation, its nodes and their endpoints, newest first.")))
        add("org", sl, "settings", lambda sl=sl: org_page(sl, "settings", generic_org_settings(sl)))
        for n in o["nodes"]:
            nid = n["id"]
            add("node", nid, "overview", lambda nid=nid: node_page(nid))
            add("node", nid, "endpoints", lambda nid=nid: node_page(nid, tab="endpoints", panel=endpoints_panel(data.node(nid))))
            add("node", nid, "routes", lambda nid=nid: node_page(nid, tab="routes", panel=routes_panel(nid)))
            add("node", nid, "requests", lambda nid=nid: node_page(nid, tab="requests", panel=node_requests_panel(nid)))
            add("node", nid, "audit", lambda nid=nid: node_page(nid, tab="audit", panel=audit_panel(audit_events(nid=nid), f"Every operator action on {nid} and its endpoints, newest first.")))
            add("node", nid, "settings", lambda nid=nid: node_page(nid, tab="settings", panel=node_settings_body(nid)))
    return out


def generic_org_settings(slug):
    o = org_find(slug)
    d = definitions([("Name", e(o["name"])), ("Organisation id", f'<code>{o["id"]}</code> · fixed: every revision names the organisation by it')])
    head = panel_head("Its name and id. A change is saved as a revision you review and apply.", button("secondary", "Edit", "pencil", small=True))
    return sections(head + section("Organisation", d))
