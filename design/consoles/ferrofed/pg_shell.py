# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The shell and global pages: the dashboard and its states, the top-bar
search, the sidebar, the user menu, toasts, the status pages, About, the
focus sheet and the shortcut legend."""
from common import (
    MENU_SEP, OPERATOR, badge, button, card, chip, definitions, disclosure, e, empty_state, icon, icon_button,
    input_, link_button, menu, menu_item, notice, page_heading, pill, refusal, search_control, select, shell,
    shortcut_legend, stat_card, status, table, tabbed, tabs, tag, td, th, titled_panel, toast, toasts, when,
    copy_button, check,
)
from data import DEPENDENCIES, EHR_A, INCIDENTS_OPEN, MEMBERS, STORED, EEHRXF, VERSION
from parts import skeleton_lines

HEALTH = {
    "up": ("ok", "circle-check", "up"),
    "failing": ("bad", "circle-x", "failing"),
    "down": ("bad", "server-crash", "down"),
    "unknown": ("neutral", "clock", "not yet observed"),
}


def health(h, detail=None):
    tone, glyph, word = HEALTH[h]
    return status(tone, glyph, word, detail)


def membership(m):
    return pill("ok", "active") if m == "active" else pill("neutral", m)


def replica_line(name="gateway-0", when_=None, show=False):
    """Which replica answered: shown only on About and Self-description."""
    if not show:
        return ""
    t = f' · {when(when_, seconds=True)}' if when_ else ""
    return f'<p class="fui-replica-line">{icon("server", "12")}Answered by replica <code>{e(name)}</code>{t}</p>'


def endpoint_link(ep, href="node.html"):
    return f'<a href="{href}"><code>{e(ep)}</code></a>'


# ---- dashboard ----

def dashboard_heading():
    return page_heading("Dashboard", subtitle="Wednesday 21 October 2026 · the federation as replica gateway-0 sees it.")


def dependency_list(states=None):
    items = ""
    for key, what, host, h, line in DEPENDENCIES:
        h = (states or {}).get(key, h)
        st = health(h)
        items += (f'<div class="mk-listener"><div class="mk-listener-line"><strong>{e(what)}</strong>{st}</div>'
                  f'<div class="fui-small fui-muted"><code>{e(host)}</code> · {e(line) if h == "up" else "the gateway asks it and gets a 503; the Mitz pre-filter answers not-asked"}</div></div>')
    return f'<div class="mk-listeners">{items}</div>'


def incident_rows(items):
    rows = []
    for kind, at, ehr, claim, desc in items:
        rows.append(
            f'<tr><th scope="row"><a href="integrity.html">{e(kind)}</a></th>' + td("Members", " ".join(f"<code>{c}</code>" for c in claim))
            + td("Detected", when(at)) + "</tr>"
        )
    return rows


def attention_rows():
    from data import ORGS
    rows = []
    for o in ORGS:
        for n in o["nodes"]:
            if n["tone"] in ("ok",):
                continue
            from pg_orgs import state, node_href
            why = {"warn": "503 since 09:38", "muted": "organisation excluded", "neutral": "never probed", "bad": "no answer"}[n["tone"]]
            rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{node_href(n)}"><code>{n["id"]}</code></a></th>'
                        + td("Organisation", e(o["name"]), role="sub") + td("State", state(n["tone"]), role="meta-2") + td("Why", why, role="meta-q") + "</tr>")
    return rows


def topology_summary():
    """A compact picture of the federation: the gateway, its organisations and
    their nodes as rows of dots, each in its state; it leads to Topology."""
    from data import ORGS
    from common import health_dot
    lines = "".join(f'<li><span class="mk-topo-org">{e(o["name"])}</span><span class="mk-topo-dots">{"".join(health_dot(n["tone"]) for n in o["nodes"])}</span></li>' for o in ORGS)
    svc = "".join(health_dot(t) for t in ["ok", "ok", "ok", "warn", "ok", "ok", "ok", "ok"])
    return card("Topology", f'<div class="mk-topo-mini"><span class="mk-topo-gw">{icon("network", "16")}Gateway</span><ul>{lines}<li><span class="mk-topo-org">Services</span><span class="mk-topo-dots">{svc}</span></li></ul></div>'
                + '<p class="fui-small fui-muted">5 organisations · 6 nodes: 4 reachable, 1 degraded, 1 excluded · 8 services: 1 degraded (Mitz)</p>',
                actions=link_button("secondary", "Topology", "topology.html", glyph_end="chevron-right", small=True))


def page_dashboard(search_state=None, search_value="", results=None, menu_open=False, menu_theme="light", menu_focus=None,
                   rail=False, rail_tip=None, drawer=False, toast_html="", banner=False, org=None):
    stats = (
        '<div class="fui-grid fui-grid-stats">'
        + stat_card("building-2", "5", "Organisations", badge("neutral", "6 nodes, 7 endpoints"), href="organisations.html")
        + stat_card("heart-pulse", "4 of 6", "Nodes reachable", badge("warn", "node-b degraded"), href="organisations-nodes.html")
        + stat_card("shield-alert", "2", "Open incidents", badge("warn", "1 ehr_id held"), href="integrity.html")
        + stat_card("file-text", "Revision 7", "Running configuration", pill("ok", "in step on 2 replicas"), href="configuration.html")
        + "</div>"
    )
    attention = (
        '<section class="fui-card fui-panel fui-worklist"><div class="fui-panel-head"><h2 class="fui-card-title"><span>Nodes that need attention</span></h2>'
        f'<div class="fui-actions">{link_button("secondary", "All nodes", "organisations-nodes.html", glyph_end="chevron-right", small=True)}</div></div>'
        + table("Nodes that are not reachable", th("Node") + th("Organisation") + th("State") + th("Why"), attention_rows())
        + "</section>"
    )
    incidents = titled_panel(
        "Open incidents",
        link_button("secondary", "Integrity", "integrity.html", glyph_end="chevron-right", small=True),
        table("Open incidents, newest first", th("Kind") + th("Nodes") + th("Detected"), incident_rows(INCIDENTS_OPEN)),
    )
    versions = sum(len(v) for _, v, _ in STORED)
    config = card("Configuration", definitions([
        ("Mode", '<span class="fui-mode-badge">' + icon("monitor", "14") + "Console mode</span>"),
        ("Running", f'Revision 7 · applied {when("2026-10-19T14:12")} by Demo Operator'),
        ("Replicas", '<ul class="fui-replicas"><li><code>gateway-0</code>' + pill("ok", "in step") + '</li><li><code>gateway-1</code>' + pill("ok", "in step") + "</li></ul>"),
        ("Stored queries", f'{len(STORED)} queries in {versions} versions · {len(EEHRXF)} reserved'),
    ]), actions=link_button("secondary", "Configuration", "configuration.html", glyph_end="chevron-right", small=True))
    content = dashboard_heading() + (
        f'<div class="fui-stack fui-gap-section">{stats}'
        f'<div class="fui-grid fui-grid-start">{attention}{topology_summary()}</div>'
        f'<div class="fui-grid fui-grid-start">{incidents}{config}</div></div>'
    )
    return shell("Dashboard", "dashboard", content, search_state=search_state, search_value=search_value, results=results,
                 menu_open=menu_open, menu_theme=menu_theme, menu_focus=menu_focus, rail=rail, rail_tip=rail_tip, drawer=drawer,
                 toast_html=toast_html, banner=banner, org=org)


def page_dashboard_loading():
    stats = '<div class="fui-grid fui-grid-stats">' + "".join(
        f'<div class="fui-card fui-stat" aria-hidden="true"><span class="fui-stat-glyph mk-sk-glyph"></span><div class="fui-stat-text mk-grow">{skeleton_lines([40, 80])}</div></div>'
        for _ in range(4)) + "</div>"

    def sk(lines):
        return f'<div class="fui-card mk-sk-card" aria-hidden="true">{skeleton_lines(lines)}</div>'
    rows = '<div class="fui-grid fui-grid-start">' + sk([30, 100, 100, 100, 100, 100]) + sk([30, 90, 70, 90, 60, 90, 70]) + "</div>"
    rows2 = '<div class="fui-grid fui-grid-start">' + sk([30, 100, 100]) + sk([30, 80, 90, 70, 60]) + "</div>"
    content = dashboard_heading() + '<p class="fui-sr-only" role="status">Loading the dashboard</p>' + f'<div class="fui-stack fui-gap-section" aria-busy="true">{stats}{rows}{rows2}</div>'
    return shell("Dashboard", "dashboard", content, incidents=None)


def page_dashboard_empty():
    body = empty_state(
        "building-2", "No organisation takes part yet",
        "The gateway serves, but its registry declares no organisation, so a query reaches no one. Start in Organisations: create the first organisation, then its nodes, then each node's endpoints and their credentials.",
        link_button("primary", "Go to Organisations", "organisations-empty.html", "arrow-right") + link_button("secondary", "How configuration works", "configuration-mode-dialog.html", "info"),
    )
    steps = ('<ol class="mk-setup-steps"><li><strong>An organisation</strong><span>on the Organisations page</span></li>'
             '<li><strong>Its nodes</strong><span>each CDR, inside its organisation</span></li>'
             '<li><strong>Their endpoints and credentials</strong><span>how the gateway reaches each node, on the node</span></li>'
             '<li><strong>A first query</strong><span>in the query console, to see every node answer</span></li></ol>')
    content = dashboard_heading() + f'<div class="fui-stack">{body}{card("Setup order", steps)}</div>'
    return shell("Dashboard", "dashboard", content, incidents=0)


def tech_detail(failure="Connection refused at the gateway's address", rid="01JAF3K7Q4M2"):
    return disclosure("Technical detail", definitions([
        ("Failure", failure), ("Gateway", "<code>https://gateway.demo-regio.example.org/</code>"),
        ("Request id", f"<code>{rid}</code>"), ("Time", when("2026-10-21T09:42:11", seconds=True)), ("Attempts", "3 of 3, the last 4 seconds ago"),
    ]))


def unreachable_block(what):
    return (
        refusal(f"The gateway refused the connection, so {what} cannot be shown.")
        + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried {when("2026-10-21T09:42:11", seconds=True)} · request id <code>01JAF3K7Q4M2</code></span></div>'
    )


def page_gateway_unreachable():
    content = (
        dashboard_heading() + '<div class="fui-stack">' + unreachable_block("the dashboard")
        + card("What still works", '<p class="fui-muted">The console keeps no copy of the federation, so no view can be drawn until the gateway answers. Your session is kept: you do not need to sign in again. The gateway may still serve its clients; only this console cannot reach it.</p>')
        + tech_detail() + "</div>"
    )
    return shell("Dashboard", "dashboard", content, status="down", incidents=None)


def page_dashboard_dev():
    return page_dashboard(banner=True)


# ---- the top-bar search ----

def search_results_list(kind):
    if kind == "system":
        body = ('<p class="mk-results-head">1 match for “legacy-zvs”</p><p class="mk-results-group">Routes</p><ul role="listbox" aria-label="Routes">'
                f'<li role="option" aria-selected="true"><a href="node-routes.html">{icon("route", "14")}<span class="mk-results-name">legacy-zvs.example.org</span>'
                '<span class="fui-small fui-muted">routes to node-a · registered · node-a-pub</span></a></li></ul>')
        return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'
    if kind == "none":
        body = (
            f'<div class="mk-results-empty">{icon("search-x", "16")}<div><strong>Nothing found for “node-x”</strong>'
            '<span>The search reads organisation names, node and endpoint ids, system ids, creating_system_ids, base URLs and stored-query names. It never takes a patient identifier: run a query for that.</span></div></div>'
            f'<a href="query.html" class="mk-results-more">Open the query console{icon("arrow-right", "14")}</a>'
        )
    else:
        from common import health_dot
        groups = [
            ("Organisations", [("Testkliniek Oost", "1 node · degraded", "warn", "organisation-degraded.html", True)]),
            ("Nodes", [("node-b", "Testkliniek Oost · cdr-b.example.org", "warn", "node-degraded.html", False),
                       ("node-a-lab", "Ziekenhuis Voorbeeldstad · cdr-a-lab.example.org", "ok", "node.html", False)]),
            ("Stored queries", [("org.demo-regio::lab_results_recent", "2 versions", None, "stored-query.html", False)]),
        ]
        body = '<p class="mk-results-head">4 matches for “b”</p>'
        for g, items in groups:
            body += f'<p class="mk-results-group">{g}</p><ul role="listbox" aria-label="{g}">' + "".join(
                f'<li role="option"{" aria-selected=\"true\"" if sel else ""}><a href="{href}">{health_dot(t) if t else icon("database", "14")}<span class="mk-results-name">{e(n)}</span>'
                f'<span class="fui-small fui-muted">{e(line)}</span></a></li>' for n, line, t, href, sel in items) + "</ul>"
    return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'


def topbar_state_page(state=None, value="", results=None):
    return page_dashboard(search_state=state, search_value=value, results=results)


def page_search_states():
    states = [
        ("Default", "the icon, the placeholder and the quiet / hint", "", None),
        ("Hover", "the edge takes the ink", "", "hover"),
        ("Focused, empty", "one ring round the whole group; the hint is gone", "", "focus"),
        ("Focused and filled", "the ring, the typed value, the hint gone", "node-b", "focus-filled"),
        ("Filled, not focused", "the hint stays hidden while the field holds text", "node-b", None),
        ("Busy", "the icon becomes the spinner while the search runs", "node-b", "busy"),
        ("Disabled", "the sunken ground, no pointer; the shortcut does nothing (the gateway is unreachable)", "", "disabled"),
    ]
    cards = ""
    for i, (title, line, value, state) in enumerate(states):
        cards += (f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                  f'<div class="mk-search-demo">{search_control(value, state, id_=f"find-{i}")}</div></div>')
    keys = card("What it finds", definitions([
        ("Organisations", "by name or id"),
        ("Nodes", "by node id or system_id, and by an endpoint's id or base URL"),
        ("Stored queries", "by qualified name"),
        ("Routes", "by creating_system_id: the node it routes to and on what ground, opening that node's Routes tab"),
        ("Never", "a patient identifier: the console runs a patient's data only through the query console, as a parameter"),
        ("Keys", "/ focuses it at wide widths; ↑ and ↓ move through the matches, Enter opens one, Esc closes"),
    ]))
    content = page_heading("Search control, states", subtitle="FerroUI #144 in FerroFED: one joined control that finds an organisation, a node or a stored query. Below 40 rem the hint is gone and the submit keeps its icon alone.") + f'<div class="fui-stack mk-search-sheet">{cards}{keys}</div>'
    return shell("Search states", "dashboard", content)


# ---- shell states ----

def page_sidebar_rail():
    return page_dashboard(rail=True, rail_tip="integrity")


def page_sidebar_drawer():
    import pg_orgs
    return pg_orgs.node_page("node-a").replace('class="fui-sidebar"', 'class="fui-sidebar mk-drawer-open"', 1).replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1).replace('<main id="main"', '<div class="mk-scrim mk-scrim-drawer" aria-hidden="true"></div><main id="main"', 1)


def page_user_menu():
    return page_dashboard(menu_open=True)


def page_user_menu_dark():
    html = page_dashboard(menu_open=True, menu_theme="dark")
    return html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)


def page_user_menu_signout():
    return page_dashboard(menu_open=True, menu_focus="signout")


def page_toast(kind):
    import pg_orgs
    t = {
        "ok": toast("ok", "circle-check", "Revision 8 applied", "Every replica follows within 5 seconds. Recorded with your reason."),
        "warn": toast("warn", "triangle-alert", "node-b is degraded", "Its endpoint answered 503 to the last 12 requests. Queries in scope come back incomplete.",
                      action=link_button("secondary", "Open node-b", "node-degraded.html", small=True)),
        "bad": toast("bad", "circle-alert", "The ehr_id was not cleared", "Replica gateway-1 refused it (403 operator-unverified): your token was not verified there. Nothing changed on gateway-1."),
    }[kind]
    return pg_orgs.orgs_page(toast_html=toasts(t))


def status_page(code, glyph, title, body, actions, current="dashboard", status_="up"):
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon(glyph, "24")}</span><p class="fui-status-page-code">{code}</p>'
               f'<h1>{title}</h1><p>{body}</p><div class="fui-actions">{actions}</div></div>')
    return shell(title, current, content, status=status_)


def page_forbidden():
    return status_page("403 · Not allowed", "ban", "This view needs the operator scope",
                       "You are signed in as Demo Operator, but your access token carries no <code>__ferrofed.operator</code> scope, so the gateway's operator surface refused it. Ask your identity provider's administrator for the scope, then sign in again.",
                       button("primary", "Sign in again", "log-in") + link_button("secondary", "Dashboard", "dashboard.html"), current="integrity")


def page_not_found():
    return status_page("404 · Not found", "compass", "No node at this address",
                       "The gateway declares no node <code>node-x</code>. It may be mistyped, or the node was removed in a later revision. Every node is listed under its organisation, and all together in All nodes.",
                       link_button("primary", "All nodes", "organisations-nodes.html", "server") + link_button("secondary", "Dashboard", "dashboard.html"), current="orgs")


def page_unreadable():
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon("server-crash", "24")}</span><p class="fui-status-page-code">500 · Fault</p>'
               f'<h1>The console cannot read its sessions</h1><p>It cannot tell whether you are signed in, so it shows nothing. Ask whoever runs ferrofed-console to restart it; your gateway keeps serving its clients meanwhile.</p>'
               f'<div class="fui-actions">{link_button("primary", "Try again", "dashboard.html", "refresh-cw")}</div></div>')
    from common import centred
    return centred("Fault", content)


def page_no_admin():
    return status_page("Not set up", "settings", "Configuration needs an admin listener",
                       "This console names no gateway replica under <code>[[admin.replicas]]</code> in <code class='fui-break'>ferrofed-console.toml</code>, so it cannot read or change the configuration, and it offers no admin action. Every view still reads the gateway. Add the admin listener of each replica to the console's configuration and restart it.",
                       link_button("primary", "How configuration works", "configuration-mode-dialog.html", "info") + link_button("secondary", "Dashboard", "dashboard.html"), current="configuration")


# ---- about ----

def json_html(text):
    """JSON with the editor's token colours: keys, strings, numbers, literals."""
    import re
    out, pos = [], 0
    for m in re.finditer(r'"(?:[^"\\]|\\.)*"(\s*:)?|-?\d+(?:\.\d+)?|\b(?:true|false|null)\b', text):
        out.append(e(text[pos:m.start()]))
        tok = m.group(0)
        if tok.startswith('"'):
            if m.group(1):
                key = tok[: len(tok) - len(m.group(1))]
                out.append(f'<span class="fui-tok-key">{e(key)}</span>{e(m.group(1))}')
            else:
                out.append(f'<span class="fui-tok-string">{e(tok)}</span>')
        elif tok[0].isdigit() or tok[0] == "-":
            out.append(f'<span class="fui-tok-number">{tok}</span>')
        else:
            out.append(f'<span class="fui-tok-keyword">{tok}</span>')
        pos = m.end()
    out.append(e(text[pos:]))
    return "".join(out)


def page_about():
    def facts(rows):
        return '<dl class="fui-facts">' + "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rows) + "</dl>"
    vers = f'<section class="fui-card"><h2 class="fui-card-title"><span>Versions</span></h2>{facts([("Console", f"FerroFED Console {VERSION}"), ("Gateway", f"FerroFED {VERSION}"), ("Specification", "Federation Tier with AQL 1.0.0"), ("openEHR", "ITS-REST 1.1.0"), ("FerroUI", "ferrohealth-ui 0.3.0")])}</section>'
    maker = f'<section class="fui-card"><h2 class="fui-card-title"><span>Manufacturer</span></h2>{facts([("Made by", "Cadasto B.V."), ("Licence", "Business Source License 1.1"), ("Support", '<a href="#">info@cadasto.com</a>'), ("Documentation", '<a href="#">The FerroFED book</a>')])}</section>'
    from pg_fed import OPTIONS_DOC
    acts = copy_button("Copy", "the OPTIONS document") + button("secondary", "Download", "download", small=True)
    doc = (f'<section class="fui-card fui-panel mk-options-card"><div class="fui-panel-head"><h2 class="fui-card-title"><span>The OPTIONS document</span></h2><div class="fui-actions">{acts}</div></div>'
           f'<div class="fui-panel-body fui-stack"><p class="fui-small fui-muted">What the gateway declares in <code>OPTIONS {{base}}/</code>, as every client reads it. Self-description shows the same, in words.</p>'
           f'<div class="fui-code-panel mk-options-code" role="region" aria-label="The OPTIONS document" tabindex="0"><pre>{json_html(OPTIONS_DOC)}</pre></div></div></section>')
    content = (page_heading("About", subtitle="The console, the gateway it reads, and the manufacturer.")
               + f'<div class="fui-stack"><div class="fui-grid fui-grid-start">{vers}{maker}</div>{doc}</div>')
    return shell("About", "about", content)


# ---- keyboard and focus ----

def page_shortcuts():
    import pg_query
    legend = shortcut_legend([
        ("Everywhere", [("/", "Find an organisation, a node or a stored query"), ("?", "This legend"), ("Esc", "Close a dialog, a menu or a panel"),
                        ("g d", "Go to the dashboard"), ("g o", "Go to Organisations, where the list has its own filter"), ("g q", "Go to the query console"), ("g c", "Go to the configuration")]),
        ("Lists", [("j | down", "Next row"), ("k | up", "Previous row"), ("Enter", "Open the row"), ("left | right", "Previous and next page")]),
        ("Query console", [("Ctrl Enter", "Run the query, from the editor too"), ("t", "Table or raw answer")]),
        ("Configuration", [("e", "Edit the tab you are on"), ("r", "Go to the reason")]),
    ])
    return pg_query.page_query(overlay=legend)


def page_focus_rings():
    import pg_fed

    def row(title, html):
        return f'<div class="mk-focus-row"><span class="mk-focus-name">{title}</span><div class="mk-focus-demo">{html}</div></div>'
    rows = (
        row("Primary button", button("primary", "Run query", "play", cls="mk-focus", keys="Ctrl Enter"))
        + row("Secondary button", button("secondary", "Roll back", "rotate-ccw", cls="mk-focus"))
        + row("Danger button", button("danger", "Remove node…", "trash-2", cls="mk-focus"))
        + row("Icon button", icon_button("ellipsis", "More actions", cls="mk-focus"))
        + row("Link", '<a href="#" class="mk-focus-ring mk-inline-link">node-b</a>')
        + row("Text field", input_("f1", "cdr-f.example.org", cls="fui-input mk-focus"))
        + row("Select", select("f2", [("all", "Every health")], "all", cls="fui-select mk-focus"))
        + row("Checkbox", '<div class="fui-check"><input type="checkbox" id="f3" class="mk-focus" checked /><label for="f3" class="fui-label">In service</label></div>')
        + row("Switch", '<div class="fui-check"><input type="checkbox" id="f4" class="fui-switch mk-focus" checked /><label for="f4" class="fui-label">Best effort</label></div>')
        + row("Segmented control", '<div role="group" class="fui-segmented"><button class="fui-segment" aria-pressed="true">Keep</button><button class="fui-segment mk-focus">Set a value</button><button class="fui-segment">Read from a file</button></div>')
        + row("Filter chip", '<div class="fui-chip-checks"><label class="fui-chip-check mk-focus-ring"><input type="checkbox" checked />node-a-pub</label></div>')
        + row("Tab", tabbed(tabs("Example", [("overview", "Overview", "#"), ("nodes", "Nodes", "#"), ("requests", "Requests", "#")], "overview", focus="nodes"), '<span class="fui-small fui-muted">Panel</span>'))
        + row("Search control", search_control("", "focus", id_="find-f"))
        + row("Editor", '<div class="fui-editor mk-focus-within"><div class="fui-editor-stack mk-short-stack"><pre class="fui-editor-layer" aria-hidden="true"><span class="fui-code-line"><span class="fui-tok-keyword">SELECT</span> e/ehr_id/value</span></pre><textarea class="fui-editor-input" aria-label="AQL">SELECT e/ehr_id/value</textarea></div></div>')
        + row("Menu item", menu([menu_item("Edit", "pencil", focus=True), menu_item("Remove…", "trash-2", danger=True)], anchored=False))
    )
    content = page_heading("Focus, on every control", subtitle="The ring is the --focus role, 2 px, held to 3:1 against its ground in both themes; outside a control, inside a row, a tab or a menu item, so no neighbour clips it.") + f'<div class="fui-card mk-focus-sheet">{rows}</div>'
    return shell("Focus rings", "", content)


def _frame(title, line, html, cls=""):
    return f'<div class="fui-card mk-state-frame {cls}"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div class="mk-menu-stage">{html}</div></div>'


def node_menu_items(hover=None, focus=None, desc=False, disabled=False):
    from common import MENU_SEP
    return [menu_item("Edit", "pencil", keys="e", hover=hover == "edit", focus=focus == "edit", desc="Its system id, CDR and service entries" if desc else None),
            menu_item("Probe all endpoints", "activity", disabled="node-e is excluded: include it before probing" if disabled else None, desc="2 endpoints, within 2.5 s" if desc else None),
            MENU_SEP,
            menu_item("Exclude…", "ban", hover=hover == "exclude", desc="Asked nothing until included" if desc else None),
            menu_item("Remove…", "trash-2", danger=True, hover=hover == "remove", focus=focus == "remove", desc="Its endpoints and credentials go with it" if desc else None)]


def trig(open_=True):
    return (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="More actions for node-a" title="More actions">{icon("ellipsis", "16")}</button>')


def page_menu_states():
    from common import menu
    def demo(items, up=False, open_=True):
        m = menu(items, label="Actions for node-a", up=up) if open_ else ""
        return f'<div class="mk-menu-demo{" mk-menu-demo-up" if up else ""}"><span class="fui-anchor">{trig(open_)}{m}</span></div>'
    frames = (
        _frame("Rest", "4 px under its trigger, aligned to its end edge; ink text, muted icons, a quiet key cap", demo(node_menu_items()))
        + _frame("Hover", "the neutral tint, the icon to ink", demo(node_menu_items(hover="edit")))
        + _frame("Keyboard focus", "the same tint, plus the focus ring; ↑ ↓ Home End and a letter move, Enter activates, Esc returns to the trigger", demo(node_menu_items(focus="edit")))
        + _frame("Destructive item hovered", "danger ink at rest, the danger ground on hover or focus; always last, after a separator", demo(node_menu_items(hover="remove")))
        + _frame("Disabled, with its reason", "muted and skipped by the keyboard; the reason is its tooltip", demo(node_menu_items(disabled=True)))
        + _frame("With descriptions", "one muted line under a label when the action needs it; the width still fits the longest label, 12 to 20 rem", demo(node_menu_items(desc=True)))
        + _frame("Flipped up", "no room below: it opens above its trigger and stays in the viewport", demo(node_menu_items(), up=True))
    )
    sheet = '<div class="mk-sheet-demo">' + menu(node_menu_items(), label="Actions for node-a", sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>"
    frames += _frame("Phone: the bottom sheet", "below 40 rem a row menu or More is a sheet: its title, 48 px rows, the destructive item last, Cancel", sheet)
    keys = card("Rules", definitions([
        ("Roles", 'role="menu" and role="menuitem"; aria-haspopup and aria-expanded on the trigger, which shows its pressed state while open'),
        ("Keys", "Enter, Space or ↓ opens and focuses the first item; ↑ ↓ move, Home and End jump, a letter jumps, Enter activates, Esc closes and returns the focus"),
        ("Never clipped", "it lives in the top layer, above every row and panel; a parent's scroll never cuts it"),
    ]))
    content = page_heading("Menu, states", subtitle="NEW (FerroFED, for every console), F13: the one anchored menu behind every ⋯, every More and the user menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}{keys}</div>'
    return shell("Menu states", "orgs", content)


def page_action_group_states():
    from common import action_group, menu
    probe = link_button("primary", "Probe now", "node-probing.html", "activity")
    items = [i for i in node_menu_items() if "Probe all" not in i]
    frames = (
        _frame("Rest", "one primary action and one More button; nothing else in the title row", f'<div class="mk-menu-demo">{action_group(probe, items, label="More actions for node-a")}</div>')
        + _frame("More open", "the same anchored menu: Edit, a separator, Exclude… and Remove…, destructive last", f'<div class="mk-menu-demo">{action_group(probe, items, open_=True, label="More actions for node-a")}</div>')
        + _frame("A destructive item hovered", "the danger ground under Remove…", f'<div class="mk-menu-demo">{action_group(probe, [i.replace("fui-menu-item fui-menu-danger", "fui-menu-item fui-menu-danger mk-hover") for i in items], open_=True, label="More actions for node-a")}</div>')
        + _frame("Split button", "when the menu holds variants of the same action: Probe now ▾ with Probe all endpoints", f'<div class="mk-menu-demo">{action_group(probe, items, label="More actions for node-a", split_items=[menu_item("Probe node-a-pub only", "activity"), menu_item("Probe all endpoints", "activity", hover=True)], split_open=True)}</div>')
    )
    sheet = '<div class="mk-sheet-demo">' + menu(items, label="More actions for node-a", sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>"
    frames += _frame("Phone", "the primary stays; More opens the bottom sheet. It replaces the Actions fold at every width", sheet)
    keys = card("Rules", definitions([
        ("One primary", "the most used action as the button; every other action in More"),
        ("Destructive", "only in the menu, last, after a separator; the ellipsis says a confirmation follows"),
        ("Name", "More, with the accessible name “More actions for node-a”; an icon-only ⋯ is allowed where room is short"),
    ]))
    content = page_heading("Action group, states", subtitle="NEW (FerroFED, for every console), F14: the title row's one primary action and its More menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}{keys}</div>'
    return shell("Action group states", "orgs", content)
