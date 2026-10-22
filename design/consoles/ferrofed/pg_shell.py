# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The shell and global pages: the dashboard and its states, the top-bar
search, the sidebar, the user menu, toasts, the status pages, About, the
focus sheet and the shortcut legend."""
from common import (
    MENU_SEP, OPERATOR, badge, button, card, definitions, disclosure, e, health_dot, hint, icon, icon_button,
    input_, link_button, menu, menu_item, notice, page_heading, pill, quiet_line, refusal, search_control, select,
    shell, shortcut_legend, stat_card, state, table, tabbed, tabs, td, th, titled_panel, toast, toasts, when,
    copy_button,
)
import data
from data import (BOOK_URL, EEHRXF, INCIDENT_WORD, INCIDENTS_OPEN, N_ENDPOINTS, N_NODES, N_ORGS, ORGS, STORED,
                  SUPPORT_MAIL, VERSION, node_counts, service_counts, word)
from parts import skeleton_lines
import routes


def replica_line(name="gateway-0", when_=None, show=False):
    """Which replica answered: shown only on About and Self-description."""
    if not show:
        return ""
    t = f' · {when(when_, seconds=True)}' if when_ else ""
    return f'<p class="fui-replica-line">{icon("server", "12")}Answered by replica <code>{e(name)}</code>{t}</p>'


def incident_kind(kind):
    return f'{e(INCIDENT_WORD[kind])}{hint(kind)}'


def nodes_list(ids):
    return ", ".join(f'<a href="{routes.node_href(i)}"><code>{i}</code></a>' for i in ids)


# ---- dashboard ----

def dashboard_heading(sub=True):
    return page_heading("Dashboard", subtitle="Wednesday 21 October 2026 · the federation as replica gateway-0 sees it." if sub else None)


def incident_rows(items):
    rows = []
    for kind, at, ehr, claim, desc, _ in items:
        rows.append(
            f'<tr><th scope="row"><a href="integrity.html">{incident_kind(kind)}</a></th>' + td("Nodes", nodes_list(claim))
            + td("Detected", __import__("common").rel(at)) + "</tr>"
        )
    return rows


def attention_rows():
    from common import rel
    rows = []
    for o in ORGS:
        for n in o["nodes"]:
            if n["tone"] == "ok":
                continue
            why = {"warn": f"answers 503 since {n.get('since', '')}", "muted": "its organisation is excluded", "neutral": "never probed", "bad": "no answer"}[n["tone"]]
            cls = ' class="is-excluded"' if n["tone"] == "muted" else ""
            rows.append(f'<tr{cls}><th scope="row" data-role="title"><a class="fui-row-link" href="{routes.node_href(n["id"])}"><code>{n["id"]}</code></a></th>'
                        + td("Organisation", e(o["name"]), role="sub") + td("State", state(n["tone"]), role="meta-2") + td("Why", why, role="meta-q") + "</tr>")
    return rows


def topology_summary():
    """A compact picture of the federation, each line leading to its page;
    the card leads to Topology."""
    lines = "".join(f'<li><a class="mk-topo-org" href="{routes.org_href(o["slug"])}">{e(o["name"])}</a><span class="mk-topo-dots">{"".join(health_dot(n["tone"]) for n in o["nodes"])}</span></li>' for o in ORGS)
    svc = "".join(health_dot(s[6]) for s in data.SET_UP)
    nc, sc = node_counts(), service_counts()
    line = (f'{N_ORGS} organisations · {N_NODES} nodes: {nc.get("ok", 0)} reachable, {nc.get("warn", 0)} degraded, {nc.get("muted", 0)} excluded · '
            f'{len(data.SET_UP)} services: {sc.get("ok", 0)} reachable, {sc.get("warn", 0)} degraded (Mitz), {sc.get("neutral", 0)} not probed')
    return card("Topology", f'<div class="mk-topo-mini"><span class="mk-topo-gw">{icon("network", "16")}Gateway</span><ul>'
                f'<li><span class="mk-topo-org">Calls it: the national contact point</span><span class="mk-topo-dots"></span></li>{lines}'
                f'<li><a class="mk-topo-org" href="services.html">Services</a><span class="mk-topo-dots">{svc}</span></li></ul></div>'
                + f'<p class="fui-small fui-muted">{line}</p>',
                actions=link_button("secondary", "Topology", "topology.html", glyph_end="chevron-right", small=True))


def page_dashboard(search_state=None, search_value="", results=None, menu_open=False, menu_theme="system", menu_focus=None,
                   rail=False, rail_tip=None, drawer=False, toast_html="", banner=False, org=None):
    from common import rel
    nc = node_counts()
    live = N_NODES - nc.get("muted", 0)
    stats = (
        '<div class="fui-grid fui-grid-stats">'
        + stat_card("building-2", str(N_ORGS), "Organisations", badge("neutral", f"{N_NODES} nodes, {N_ENDPOINTS} endpoints"), href="organisations.html")
        + stat_card("heart-pulse", f"{nc.get('ok', 0)} of {live}", "Nodes reachable", badge("warn", "node-b · Degraded"), href="organisations-nodes.html")
        + stat_card("shield-alert", str(len(INCIDENTS_OPEN)), "Open incidents", badge("warn", "1 EHR id held"), href="integrity.html")
        + stat_card("file-text", f"Revision {data.RUNNING}", "Running configuration", pill("ok", "In step on 2 replicas"), href="configuration.html")
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
        table("Open incidents, newest first", th("What happened") + th("Nodes") + th("Detected"),
              [f'<tr><th scope="row"><a href="integrity.html">{incident_kind(k)}</a></th>' + td("Nodes", nodes_list(c)) + td("Detected", rel(at)) + "</tr>" for k, at, _, c, _, _ in INCIDENTS_OPEN]),
    )
    versions = sum(len(v) for _, v, _ in STORED)
    config = card("Configuration", definitions([
        ("Revisions", '<span class="fui-mode-badge">' + icon("monitor", "14") + 'Managed</span>' + hint('config.ui = "managed"')),
        ("Running", f'Revision {data.RUNNING} · applied {when(data.RUNNING_AT)} by {data.RUNNING_BY}'),
        ("Replicas", '<ul class="fui-replicas"><li><code>gateway-0</code>' + pill("ok", "In step") + '</li><li><code>gateway-1</code>' + pill("ok", "In step") + "</li></ul>"),
        ("Stored queries", f'{len(STORED)} queries in {versions} versions, and the gateway\'s {len(EEHRXF)} patient summary queries'),
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
    line = quiet_line("No organisation takes part yet, so a query reaches no one.", "Create the first one in Organisations", "organisations-empty.html", boxed=False)
    steps = ('<ol class="mk-setup-steps"><li><strong>An organisation, with its first node and endpoint</strong><span>saved together, on the Organisations page</span></li>'
             '<li><strong>The endpoint\'s credentials</strong><span>how the gateway signs in to the node, on the node</span></li>'
             '<li><strong>A first query</strong><span>in the query console, to see every node answer</span></li></ol>')
    content = dashboard_heading() + f'<div class="fui-stack">{line}{card("Setup order", steps)}</div>'
    return shell("Dashboard", "dashboard", content, incidents=0)


def tech_detail(failure="Connection refused at the gateway's address", rid="01JAF3K7Q4M2", show_rid=True):
    return disclosure("Technical detail", definitions([
        ("Failure", failure), ("Gateway", f"<code>{data.GATEWAY_URL}/</code>"),
        ("Request id", f"<code>{rid}</code>"), ("Time", when("2026-10-21T09:42:11", seconds=True)), ("Attempts", "3 of 3, the last 4 seconds ago"),
    ]))


def unreachable_block(what):
    return (
        refusal(f"The gateway refused the connection, so {what} cannot be shown.")
        + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried 4 seconds ago</span></div>'
    )


def page_gateway_unreachable():
    content = (
        dashboard_heading(sub=False) + '<div class="fui-stack">' + unreachable_block("the dashboard")
        + card("While the gateway does not answer", '<p class="fui-muted">The console keeps no copy of the federation, so it draws nothing until the gateway answers. Your session is kept. The gateway may still serve its clients; only this console cannot reach it.</p>')
        + tech_detail() + "</div>"
    )
    return shell("Dashboard", "dashboard", content, status="down", incidents=None)


def page_dashboard_dev():
    return page_dashboard(banner=True)


# ---- the top-bar search ----

def mark_q(text, q):
    i = text.lower().find(q.lower())
    if i < 0:
        return e(text)
    return e(text[:i]) + f"<mark>{e(text[i:i + len(q)])}</mark>" + e(text[i + len(q):])


def search_results_list(kind):
    if kind == "system":
        body = ('<p class="mk-results-head">1 match for “legacy-zvs”</p><p class="mk-results-group">Routes</p><ul role="listbox" aria-label="Routes">'
                f'<li role="option" aria-selected="true"><a href="node-routes.html">{icon("route", "14")}<span class="mk-results-name">{mark_q("legacy-zvs.example.org", "legacy-zvs")}</span>'
                '<span class="fui-small fui-muted">A creating system: follow-ups of its records go to node-a, through node-a-pub (a route registered in the registry)</span></a></li></ul>')
        return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'
    if kind == "none":
        body = (f'<div class="mk-results-empty">{icon("search-x", "16")}<div><strong>Nothing found for “node-x”.</strong>'
                f'<span>The search never takes a patient identifier. <a href="query.html">Open the query console</a></span></div></div>')
        return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'
    q = "b"
    zvs, nb, nl = data.org("zvs"), data.node("node-b"), data.node("node-a-lab")
    groups = [
        ("Organisations", [(zvs["name"], f"2 nodes · {word(data.org_tone(zvs))}", data.org_tone(zvs), routes.org_href("zvs"), True)]),
        ("Nodes", [("node-b", f"Testkliniek Oost · {word(nb['tone'])}", nb["tone"], routes.node_href("node-b"), False),
                   ("node-a-lab", f"Ziekenhuis Voorbeeldstad · {word(nl['tone'])}", nl["tone"], routes.node_href("node-a-lab"), False)]),
        ("Stored queries", [("lab_results_recent", "2 versions", None, routes.sq_href("lab_results_recent"), False),
                            ("problem_list", "3 versions", None, routes.sq_href("problem_list"), False)]),
    ]
    n = sum(len(i) for _, i in groups)
    body = f'<p class="mk-results-head">{n} matches for “{q}”</p>'
    for g, items in groups:
        body += f'<p class="mk-results-group">{g}</p><ul role="listbox" aria-label="{g}">' + "".join(
            f'<li role="option"{" aria-selected=\"true\"" if sel else ""}><a href="{href}">{health_dot(t) if t else icon("database", "14")}<span class="mk-results-name">{mark_q(nm, q)}</span>'
            f'<span class="fui-small fui-muted">{e(line)}</span></a></li>' for nm, line, t, href, sel in items) + "</ul>"
    return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'


def topbar_state_page(state_=None, value="", results=None):
    if state_ == "disabled":
        return page_gateway_unreachable()
    return page_dashboard(search_state=state_, search_value=value, results=results)


def page_search_states():
    states = [
        ("Default", "The icon, the placeholder and the quiet / hint", "", None),
        ("Hover", "The edge takes the ink", "", "hover"),
        ("Focused, empty", "One ring round the whole group; the hint is gone", "", "focus"),
        ("Focused and filled", "The ring and the typed value", "node-b", "focus-filled"),
        ("Filled, not focused", "The hint stays hidden while the field holds text", "node-b", None),
        ("Busy", "The icon becomes the spinner while the search runs", "node-b", "busy"),
        ("Disabled", "Only while the gateway is unreachable: there is nothing to search, and the field says so", "", "disabled"),
    ]
    cards = ""
    for i, (title, line, value, st) in enumerate(states):
        cards += (f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                  f'<div class="mk-search-demo">{search_control(value, st, id_=f"find-{i}")}</div></div>')
    keys = card("What it finds", definitions([
        ("Organisations", "by name or id"),
        ("Nodes", "by node id or system id, and by an endpoint's id or base URL"),
        ("Stored queries", "by name"),
        ("Routes", "by creating system: the node its records' follow-ups go to, opening that node's Routes tab"),
        ("Never", "a patient identifier: a patient's data is read only in the query console, as a parameter"),
    ]))
    content = page_heading("Search control, states", subtitle="One joined control that finds an organisation, a node or a stored query. On a phone the hint is gone and the submit keeps its icon alone.") + f'<div class="fui-stack mk-search-sheet">{cards}{keys}</div>'
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
        "ok": toast("ok", "circle-check", "Revision 8 applied", "Zorgcentrum Beta-Testerveen takes part. Every replica follows within 5 seconds."),
        "warn": toast("warn", "triangle-alert", "node-b is degraded", "Its endpoint answers 503 since 09:38. A query that asks it fails, or comes back incomplete under best effort.",
                      action=link_button("secondary", "Open node-b", routes.node_href("node-b"), small=True)),
        "bad": toast("bad", "circle-alert", "The EHR id hold was not cleared at gateway-1", "gateway-0 cleared it. gateway-1 did not accept your access token (401), so it still holds the collision.",
                     action=button("secondary", "Sign in again", "log-in", small=True)),
    }[kind]
    return pg_orgs.orgs_page(toast_html=toasts(t))


def status_page(code, glyph, title, body, actions, current="dashboard", status_="up"):
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon(glyph, "24")}</span><p class="fui-status-page-code">{code}</p>'
               f'<h1>{title}</h1><p>{body}</p><div class="fui-actions">{actions}</div></div>')
    return shell(title, current, content, status=status_)


def page_forbidden():
    return status_page("Not allowed · 403", "ban", "This view needs the operator scope",
                       f"You are signed in as Demo Operator, but your access token does not carry the operator scope{hint('__ferrofed.operator')}, so the gateway refused it. Ask your identity provider's administrator for it, then sign in again.",
                       button("primary", "Sign in again", "log-in") + link_button("secondary", "Dashboard", "dashboard.html"), current="integrity")


def page_not_found():
    return status_page("Not found · 404", "compass", "No node at this address",
                       "The gateway declares no node <code>node-x</code>. It may be mistyped, or the node was removed in a later revision. Every node is listed under its organisation, and all together in All nodes.",
                       link_button("primary", "All nodes", "organisations-nodes.html", "server") + link_button("secondary", "Dashboard", "dashboard.html"), current="orgs")


def page_unreadable():
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon("server-crash", "24")}</span><p class="fui-status-page-code">Fault · 500</p>'
               f'<h1>The console cannot read its sessions</h1><p>It cannot tell whether you are signed in, so it shows nothing. Ask whoever runs <code class="mk-nowrap">ferrofed-console</code> to restart it; your gateway keeps serving its clients meanwhile.</p>'
               f'<div class="fui-actions">{link_button("primary", "Try again", "dashboard.html", "refresh-cw")}</div></div>')
    from common import centred
    return centred("Fault", content)


def page_no_admin():
    return status_page("Not set up", "settings", "Configuration needs an admin listener",
                       "This console knows no gateway replica to ask, so it cannot read or change the configuration and offers no admin action. Every other view still reads the gateway. "
                       f"Add each replica's admin listener to the console's configuration{hint('[[admin.replicas]]')} and restart it.",
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


def code_card(title, text, what, intro=""):
    """The About code panel pattern: Copy and Download in the card header,
    the JSON in the editor's token colours, its own scroll."""
    acts = copy_button("Copy", what) + button("secondary", "Download", "download", small=True)
    i = f'<p class="fui-small fui-muted">{intro}</p>' if intro else ""
    return (f'<section class="fui-card fui-panel mk-options-card"><div class="fui-panel-head"><h2 class="fui-card-title"><span>{title}</span></h2><div class="fui-actions">{acts}</div></div>'
            f'<div class="fui-panel-body fui-stack">{i}<div class="fui-code-panel fui-self-description" role="region" aria-label="{e(title)}" tabindex="0"><pre>{json_html(text)}</pre></div></div></section>')


def page_about():
    def facts(rows):
        return '<dl class="fui-facts">' + "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rows) + "</dl>"
    vers = f'<section class="fui-card"><h2 class="fui-card-title"><span>Versions</span></h2>{facts([("Console", f"FerroFED Console {VERSION}"), ("Gateway", f"FerroFED {VERSION}"), ("Specification", "Federation Tier with AQL 1.0.0"), ("openEHR", "ITS-REST 1.1.0"), ("FerroUI", "ferrohealth-ui 0.3.0")])}</section>'
    maker = (f'<section class="fui-card"><h2 class="fui-card-title"><span>Manufacturer</span></h2>{facts([("Made by", "Cadasto B.V."), ("Licence", "Business Source License 1.1"), ("Support", f"<a href=\"mailto:{SUPPORT_MAIL}\">{SUPPORT_MAIL}</a>"), ("Documentation", f"<a href=\"{BOOK_URL}\" rel=\"external\">The FerroFED book</a>")])}</section>')
    from pg_fed import OPTIONS_DOC
    doc = code_card("The OPTIONS document", OPTIONS_DOC, "the OPTIONS document", "What the gateway declares at <code>OPTIONS {base}/</code>, as every client reads it.")
    content = (page_heading("About", subtitle="The console, the gateway it reads, and the manufacturer.")
               + f'<div class="fui-stack"><div class="fui-grid fui-grid-start">{vers}{maker}</div>{doc}</div>')
    return shell("About", "about", content)


# ---- keyboard and focus ----

def page_shortcuts():
    import pg_query
    legend = shortcut_legend([
        ("Everywhere", [("/", "Find an organisation, a node or a stored query"), ("?", "This legend"), ("Esc", "Close a dialog, a menu or a panel"),
                        ("g d", "Go to the dashboard"), ("g o", "Go to Organisations"), ("g q", "Go to the query console"), ("g c", "Go to Configuration")]),
        ("Lists", [("j | down", "Next row"), ("k | up", "Previous row"), ("Enter", "Open the row"), ("left | right", "Previous and next page")]),
        ("Query console", [("Ctrl Enter", "Run the query, from the editor too"), ("t", "Table or raw answer")]),
        ("Configuration", [("e", "Edit the tab you are on"), ("r", "Go to the reason")]),
    ])
    return pg_query.page_query(overlay=legend)


def page_focus_rings():
    def row(title, html):
        return f'<div class="mk-focus-row"><span class="mk-focus-name">{title}</span><div class="mk-focus-demo">{html}</div></div>'
    rows = (
        row("Primary button", button("primary", "Run query", "play", cls="mk-focus", keys="Ctrl Enter"))
        + row("Secondary button", button("secondary", "Roll back", "rotate-ccw", cls="mk-focus"))
        + row("Danger button", button("danger", "Remove node…", "trash-2", cls="mk-focus"))
        + row("Icon button", icon_button("ellipsis", "More actions", cls="mk-focus"))
        + row("Link", '<a href="node-degraded.html" class="mk-focus-ring mk-inline-link">node-b</a>')
        + row("Text field", input_("f1", "cdr-f.example.org", cls="fui-input mk-focus"))
        + row("Select", select("f2", [("all", "Every state")], "all", cls="fui-select mk-focus"))
        + row("Checkbox", '<div class="fui-check"><input type="checkbox" id="f3" class="mk-focus" checked /><label for="f3" class="fui-label">Every endpoint</label></div>')
        + row("Switch", '<div class="fui-check"><input type="checkbox" id="f4" class="fui-switch mk-focus" checked /><label for="f4" class="fui-label">Best effort</label></div>')
        + row("Segmented control", '<div role="group" class="fui-segmented"><button class="fui-segment" aria-pressed="true">Keep</button><button class="fui-segment mk-focus">Set a value</button><button class="fui-segment">Read from a file</button></div>')
        + row("Filter chip", '<div class="fui-chip-checks"><label class="fui-chip-check mk-focus-ring"><input type="checkbox" checked />node-a-pub</label></div>')
        + row("Tab", tabbed(tabs("Example", [("overview", "Overview", "node.html"), ("nodes", "Endpoints", "node-endpoints.html"), ("requests", "Requests", "node-requests.html")], "overview", focus="nodes"), '<span class="fui-small fui-muted">Panel</span>'))
        + row("Search control", search_control("", "focus", id_="find-f"))
        + row("Editor", '<div class="fui-editor mk-focus-within"><div class="fui-editor-stack mk-short-stack"><pre class="fui-editor-layer" aria-hidden="true"><span class="fui-code-line"><span class="fui-tok-keyword">SELECT</span> e/ehr_id/value</span></pre><textarea class="fui-editor-input" aria-label="AQL">SELECT e/ehr_id/value</textarea></div></div>')
        + row("Menu item", menu([menu_item("Edit", "pencil", focus=True), MENU_SEP, menu_item("Remove…", "trash-2", danger=True)], anchored=False))
    )
    content = page_heading("Focus, on every control", subtitle="One 2 px ring on every control, at least 3:1 against its ground in both themes. It sits outside a control and inside a row, a tab or a menu item, so no neighbour clips it.") + f'<div class="fui-card mk-focus-sheet">{rows}</div>'
    return shell("Focus rings", "", content)


def _frame(title, line, html, cls=""):
    return f'<div class="fui-card mk-state-frame {cls}"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div class="mk-menu-stage">{html}</div></div>'


def node_menu_items(hover=None, focus=None, desc=False, disabled=False, keys=True):
    return [menu_item("Edit", "pencil", keys="e" if keys else None, hover=hover == "edit", focus=focus == "edit", desc="Its system id and CDR" if desc else None),
            menu_item("Probe all endpoints", "activity", disabled="Include node-e before probing it" if disabled else None, desc="2 endpoints, within 2.5 s" if desc else None),
            MENU_SEP,
            menu_item("Exclude…", "ban", hover=hover == "exclude", desc="Asked nothing until included" if desc else None),
            menu_item("Remove…", "trash-2", danger=True, hover=hover == "remove", focus=focus == "remove", desc="Its endpoints and credentials go with it" if desc else None)]


def trig(open_=True):
    return (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="More actions for node-a" title="More actions">{icon("ellipsis", "16")}</button>')


def static_sheet(items, label):
    return ('<div class="mk-sheet-demo">' + menu(items, label=label, sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>")


def page_menu_states():
    def demo(items, up=False, open_=True):
        m = menu(items, label="Actions for node-a", up=up) if open_ else ""
        return f'<div class="mk-menu-demo{" mk-menu-demo-up" if up else ""}"><span class="fui-anchor">{trig(open_)}{m}</span></div>'
    frames = (
        _frame("Rest", "4 px under its trigger, aligned to its end edge; ink text, muted icons, a quiet key cap", demo(node_menu_items()))
        + _frame("Hover", "The neutral tint, the icon to ink", demo(node_menu_items(hover="edit")))
        + _frame("Keyboard focus", "The same tint, plus the focus ring; ↑ ↓ Home End and a letter move, Enter activates, Esc returns to the trigger", demo(node_menu_items(focus="edit")))
        + _frame("Destructive item hovered", "Danger ink at rest, the danger ground on hover or focus; always last, after a separator", demo(node_menu_items(hover="remove")))
        + _frame("Disabled, with its reason", "Muted and skipped by the keyboard; the reason is its second line", demo(node_menu_items(disabled=True)))
        + _frame("With descriptions", "One muted line under a label when the action needs it; 12 to 20 rem wide", demo(node_menu_items(desc=True)))
        + _frame("Flipped up", "No room below: it opens above its trigger and stays in the viewport", demo(node_menu_items(), up=True))
    )
    frames += _frame("Phone: the bottom sheet", "On a phone a row menu or More is a sheet: its title, 40 px rows, the destructive item last, Cancel; no key caps", static_sheet(node_menu_items(keys=False), "Actions for node-a"))
    keys = card("Rules", definitions([
        ("Roles", 'role="menu" and role="menuitem"; aria-haspopup and aria-expanded on the trigger, which shows its pressed state while open'),
        ("Keys", "Enter, Space or ↓ opens and focuses the first item; ↑ ↓ move, Home and End jump, a letter jumps, Enter activates, Esc closes and returns the focus"),
        ("Never clipped", "It lives in the top layer, above every row and panel; a parent's scroll never cuts it"),
    ]))
    content = page_heading("Menu, states", subtitle="The one anchored menu behind every ⋯, every More and the user menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}{keys}</div>'
    return shell("Menu states", "orgs", content)


def page_action_group_states():
    from common import action_group
    probe = link_button("primary", "Probe now", "node-probing.html", "activity")
    items = [i for i in node_menu_items() if "Probe all" not in i]
    # The frames draw the menus anchored at every width; the phone sheet is
    # its own static frame below, never a live sheet over the page.
    frames = (
        _frame("Rest", "One primary action and one More button; nothing else in the title row", f'<div class="mk-menu-demo">{action_group(probe, items, label="More actions for node-a")}</div>')
        + _frame("More open", "The anchored menu: Edit, a separator, Exclude… and Remove…, destructive last", f'<div class="mk-menu-demo">{action_group(probe, items, open_=True, label="More actions for node-a", sheet=False)}</div>')
        + _frame("A destructive item hovered", "The danger ground under Remove…", f'<div class="mk-menu-demo">{action_group(probe, [i.replace("fui-menu-item fui-menu-danger", "fui-menu-item fui-menu-danger mk-hover") for i in items], open_=True, label="More actions for node-a", sheet=False)}</div>')
        + _frame("Split button", "When the menu holds variants of the same action: Probe now with Probe all endpoints", f'<div class="mk-menu-demo">{action_group(probe, items, label="More actions for node-a", split_items=[menu_item("Probe node-a-pub only", "activity"), menu_item("Probe all endpoints", "activity", hover=True)], split_open=True, sheet=False)}</div>')
    )
    frames += _frame("Phone", "The primary stays; More opens the bottom sheet", static_sheet([i for i in node_menu_items(keys=False) if "Probe all" not in i], "More actions for node-a"))
    keys = card("Rules", definitions([
        ("One primary", "The most used action as the button; every other action in More"),
        ("Destructive", "Only in the menu, last, after a separator; the ellipsis says a confirmation follows"),
        ("Name", "More, with the accessible name “More actions for node-a”; an icon-only ⋯ where room is short"),
    ]))
    content = page_heading("Action group, states", subtitle="The title row's one primary action and its More menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}{keys}</div>'
    return shell("Action group states", "orgs", content)
