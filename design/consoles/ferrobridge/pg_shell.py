# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The shell and global pages: the dashboard and its states, the top-bar
search, the sidebar, the user menu, toasts, the status pages, About, the
focus sheet, the menu and action-group sheets and the shortcut legend."""
from common import (
    MENU_SEP, badge, button, card, centred, copy_button, definitions, disclosure, e, empty_state, health_dot, icon, icon_button,
    input_, link_button, menu, menu_item, notice, page_heading, pill, refusal, search_control, select, shell, shortcut_legend,
    stat_card, table, tabbed, tabs, td, th, titled_panel, toast, toasts, when,
)
from bridge import json_html, plan_badge, quiet, state

SEARCH_SCOPE = "The search finds contexts, mapping files and templates by name, path, id or resource type, and never takes a patient identifier."
from data import BRIDGE, COMMIT, PINS, VERSION
from parts import skeleton_lines
from pg_flow import flow_card

# ---- dashboard ----


SUBTITLES = {
    "up": "the bridge at <code>bridge.demo-regio.example.org</code>, as it answered at 09:42.",
    "loading": "asking the bridge at <code>bridge.demo-regio.example.org</code>…",
    "down": "the bridge at <code>bridge.demo-regio.example.org</code> last answered at 09:40.",
}


def dashboard_heading(state_="up"):
    return page_heading("Dashboard", subtitle=f"Wednesday 21 October 2026 · {SUBTITLES[state_]}")


def stats(term="ok", errors=0, operator=True):
    if operator:
        s1 = stat_card("layers", "9 of 9", "Contexts loaded", badge("neutral", "4 OMOCL"), href="mappings.html")
        s2 = (stat_card("circle-check", "0 errors", "Running load", badge("warn", "3 warnings"), href="mappings-diagnostics.html") if not errors
              else stat_card("circle-x", f"{errors} errors", "Running load", badge("bad", "set not reloaded"), href="mappings-diagnostics.html"))
    else:
        s1 = stat_card("layers", '<span class="mk-dash">–</span>', "Contexts loaded", badge("neutral", "operator API off"), href="mappings-forbidden.html")
        s2 = stat_card("circle-check", '<span class="mk-dash">–</span>', "Running load", badge("neutral", "operator API off"), href="mappings-forbidden.html")
    s3 = stat_card("plug", "3 of 3", "Upstreams reachable", pill("ok", "All up") if term == "ok" else badge("warn", "1 degraded"), href="connections.html" if term == "ok" else "connections-attention.html")
    return f'<div class="fui-grid fui-grid-stats">{s1}{s2}{s3}</div>'


def attention_card():
    rows = [
        f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="connection-terminology.html">Terminology server</a></th>'
        + td("Kind", "Connection", role="sub") + td("State", state("warn", "degraded"), role="meta-2") + td("Why", "Answers 503 since 09:31", role="meta-q") + "</tr>",
    ]
    return ('<section class="fui-card fui-panel fui-worklist"><div class="fui-panel-head"><h2 class="fui-card-title"><span>Needs attention <span class="mk-count">· 1</span></span></h2></div>'
            + table("What needs attention", th("What") + th("Kind") + th("State") + th("Why"), rows) + "</section>")


def page_dashboard(search_state=None, search_value="", results=None, menu_open=False, menu_theme="system", menu_focus=None,
                   rail=False, rail_tip=None, drawer=False, toast_html="", banner=False, attention=False, hover=None, pulse=False, operator=True):
    term = "warn" if attention else "ok"
    hub = {} if operator else {"hub_line": "– · needs the operator API", "hub_tone": "neutral"}
    flow = flow_card({"term": term}, hover=hover, pulse_frame=pulse, dev=banner,
                     foot=("3 faces on, and the ETL runs as a batch · 3 of 3 upstreams answer; the terminology server answers 503." if attention else None), **hub)
    parts = stats(term, operator=operator)
    if not operator:
        parts += notice("info", "info", "<strong>The operator API is off on this bridge</strong>, so the mapping counts are not shown. The flow and the connections read the health endpoints, which need no operator scope. "
                        '<span class="fui-small fui-muted"><code>[operator] enabled = false</code></span>')
    if attention:
        parts += attention_card()
    content = dashboard_heading() + f'<div class="fui-stack fui-gap-section">{parts}{flow}</div>'
    return shell("Dashboard", "dashboard", content, search_state=search_state, search_value=search_value, results=results,
                 menu_open=menu_open, menu_theme=menu_theme, menu_focus=menu_focus, rail=rail, rail_tip=rail_tip, drawer=drawer,
                 toast_html=toast_html, banner=banner, incidents=0 if operator else None, unknown="off")


def page_dashboard_attention():
    return page_dashboard(attention=True)


def page_dashboard_operator_off():
    return page_dashboard(operator=False)


def page_flow_hover():
    return page_dashboard(hover="cdr")


def page_flow_hover_pulse():
    return page_dashboard(hover="cdr", pulse=True)


def page_flow_help():
    from pg_flow import help_dialog
    html = page_dashboard()
    return (html.replace('</div></body></html>', f'<div class="mk-scrim" aria-hidden="true"></div>{help_dialog()}</div></body></html>')
            .replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1))


def page_dashboard_loading():
    st = '<div class="fui-grid fui-grid-stats">' + "".join(
        f'<div class="fui-card fui-stat" aria-hidden="true"><span class="fui-stat-glyph mk-sk-glyph"></span><div class="fui-stat-text mk-grow">{skeleton_lines([40, 80])}</div></div>'
        for _ in range(3)) + "</div>"
    fl = f'<div class="fui-card mk-sk-card" aria-hidden="true">{skeleton_lines([20, 100, 90, 100, 80, 60])}</div>'
    content = dashboard_heading("loading") + '<p class="fui-sr-only" role="status">Loading the dashboard</p>' + f'<div class="fui-stack fui-gap-section" aria-busy="true">{st}{fl}</div>'
    return shell("Dashboard", "dashboard", content, incidents=None, unknown="loading")


def page_dashboard_empty():
    body = quiet('The bridge runs no mapping yet: its mapping directories hold no file, so the facade serves no resource type. <a href="edit-new.html">Write the first mapping file</a>')
    steps = ('<ol class="mk-setup-steps"><li><strong>A context file</strong><span>a FHIR profile on an openEHR template</span></li>'
             '<li><strong>Its model files</strong><span>what each element maps to</span></li>'
             '<li><strong>An OMOCL file</strong><span>for the ETL: an archetype onto an OMOP CDM table</span></li>'
             '<li><strong>A check</strong><span>the bridge compiles the set before it runs</span></li>'
             '<li><strong>A first transform</strong><span>in the Playground, with a synthetic composition</span></li></ol>')
    content = dashboard_heading() + f'<div class="fui-stack">{body}{card("Setup order", steps)}</div>'
    return shell("Dashboard", "dashboard", content)


def tech_detail(failure="Connection refused at the bridge's address", rid="01JAF3K7Q4M2"):
    return disclosure("Technical detail", definitions([
        ("Failure", failure), ("Bridge", f"<code>{BRIDGE}/</code>"),
        ("Request id", f"<code>{rid}</code>"), ("Time", when("2026-10-21T09:42:11", seconds=True)), ("Attempts", "3 of 3, the last 4 seconds ago"),
    ]))


def unreachable_block(what):
    return (
        refusal(f"The bridge refused the connection, so {what} cannot be shown.")
        + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried {when("2026-10-21T09:42:11", seconds=True)} · request id <code>01JAF3K7Q4M2</code></span></div>'
    )


def page_bridge_unreachable():
    content = (
        dashboard_heading("down") + '<div class="fui-stack">' + unreachable_block("the dashboard")
        + card("While the bridge is unreachable", '<p class="fui-muted">The console keeps no copy of the bridge, so no view can be drawn until the bridge answers. Your session is kept: you do not need to sign in again. The bridge may still serve its clients; only this console cannot reach it.</p>')
        + tech_detail() + "</div>"
    )
    return shell("Dashboard", "dashboard", content, status="down", incidents=None)


def page_dashboard_dev():
    return page_dashboard(banner=True)


# ---- the top-bar search ----

def search_results_list(kind):
    if kind == "none":
        body = quiet(f'Nothing found for “Immunization”. {SEARCH_SCOPE} <a href="mappings-files.html">Every mapping file</a>')
        return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'
    groups = [
        ("Contexts", [("synthetic_body_weight.context", "Observation · synthetic.vital_signs.v1 · 2 warnings", "warn", "file-program.html", True),
                      ("synthetic_vital_signs.context", "Observation · synthetic.vital_signs.v1", "ok", "file-provisioned.html", False)]),
        ("Mapping files", [("fhirconnect/models/body_weight.model.yml", "model · console · 1 warning", None, "file-model.html", False)]),
        ("Templates", [("synthetic.vital_signs.v1", "template directory · 2 contexts", None, "template.html", False)]),
    ]
    body = '<p class="mk-results-head">4 matches for “weight”</p>'
    for g, items in groups:
        body += f'<p class="mk-results-group">{g}</p><ul role="listbox" aria-label="{g}">' + "".join(
            f'<li role="option"{" aria-selected=\"true\"" if sel else ""}><a href="{href}">{health_dot(t, "compiles with warnings") if t == "warn" else (health_dot("ok", "compiles") if t else icon("file-code", "14"))}<span class="mk-results-name">{e(n)}</span>'
            f'<span class="fui-small fui-muted">{e(line)}</span></a></li>' for n, line, t, href, sel in items) + "</ul>"
    return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'


def topbar_state_page(state_=None, value="", results=None):
    return page_dashboard(search_state=state_, search_value=value, results=results)


def page_search_states():
    states = [
        ("Default", "the icon, the placeholder and the quiet / hint", "", None),
        ("Hover", "the edge takes the ink", "", "hover"),
        ("Focused, empty", "one ring round the whole group; the hint is gone", "", "focus"),
        ("Focused and filled", "the ring, the typed value, the hint gone", "weight", "focus-filled"),
        ("Filled, not focused", "the hint stays hidden while the field holds text", "weight", None),
        ("Busy", "the icon becomes the spinner while the search runs", "weight", "busy"),
        ("Disabled", "the sunken ground, no pointer; the shortcut does nothing (the bridge is unreachable)", "", "disabled"),
    ]
    cards = ""
    for i, (title, line, value, st) in enumerate(states):
        cards += (f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                  f'<div class="mk-search-demo">{search_control(value, st, id_=f"find-{i}")}</div></div>')
    keys = card("What it finds", definitions([
        ("Contexts", "by name or resource type; a match opens the context file's Program tab"),
        ("Mapping files", "by path"),
        ("Templates", "by id"),
        ("Never", "a patient identifier or a value inside a composition"),
        ("Keys", "/ focuses it at wide widths; ↑ and ↓ move through the matches, Enter opens one, Esc closes"),
    ]))
    content = page_heading("Search control, states", subtitle=SEARCH_SCOPE) + f'<div class="fui-stack mk-search-sheet">{cards}{keys}</div>'
    return shell("Search states", "", content)


# ---- shell states ----

def page_sidebar_rail():
    return page_dashboard(rail=True, rail_tip="mappings")


def page_sidebar_drawer():
    import pg_maps
    return pg_maps.page_file().replace('class="fui-sidebar"', 'class="fui-sidebar mk-drawer-open"', 1).replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1).replace('<main id="main"', '<div class="mk-scrim mk-scrim-drawer" aria-hidden="true"></div><main id="main"', 1)


def page_user_menu():
    return page_dashboard(menu_open=True)


def page_user_menu_dark():
    html = page_dashboard(menu_open=True, menu_theme="dark")
    return html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)


def page_user_menu_signout():
    return page_dashboard(menu_open=True, menu_focus="signout")


def page_toast(kind):
    import pg_maps
    # NOTE: a reload's outcome lives in the B4 load report alone, so no toast repeats it.
    t = {
        "ok": toast("ok", "circle-check", "Path copied", "The file's path is on your clipboard."),
        "warn": toast("warn", "triangle-alert", "The terminology server answers with errors", "It answers 503, so a coded value the bridge must look up is refused until it recovers.",
                      action=link_button("secondary", "Open the connection", "connection-terminology.html", small=True)),
        "bad": toast("bad", "circle-alert", "The download failed", "The bridge did not answer, so synthetic_body_weight.context.yml was not downloaded.",
                     action=button("secondary", "Try again", small=True)),
    }[kind]
    return pg_maps.page_file(toast_html=toasts(t))


def status_page(code, glyph, title, body, actions, current="dashboard", status_="up"):
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon(glyph, "24")}</span><p class="fui-status-page-code">{code}</p>'
               f'<h1>{title}</h1><p>{body}</p><div class="fui-actions">{actions}</div></div>')
    return shell(title, current, content, status=status_)


def page_forbidden():
    return status_page("403 · Not allowed", "ban", "This view needs the operator scope",
                       "You are signed in as Demo Operator, but your access token carries no operator scope, so the bridge's operator API refused it. Ask your identity provider's administrator for the scope, then sign in again. "
                       '<span class="fui-small fui-muted">The scope is <code>__ferrobridge.operator</code>, set as <code>[auth] product_scope</code>.</span>',
                       button("primary", "Sign in again", "log-in") + link_button("secondary", "Dashboard", "dashboard.html"), current="mappings")


def page_not_found():
    return status_page("404 · Not found", "compass", "No mapping file at this path",
                       "The bridge reads no file <code>fhirconnect/<wbr>synthetic_immunization.context.yml</code>. It may be mistyped, or the file was removed and the set reloaded. Every file is listed on the Mapping set.",
                       link_button("primary", "Every mapping file", "mappings-files.html", "files") + link_button("secondary", "Dashboard", "dashboard.html"), current="mappings")


def page_unreadable():
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon("server-crash", "24")}</span><p class="fui-status-page-code">500 · Fault</p>'
               f'<h1>The console cannot read its sessions</h1><p>It cannot tell whether you are signed in, so it shows nothing. Ask whoever runs ferrobridge-console to restart it; the bridge keeps serving its clients meanwhile.</p>'
               f'<div class="fui-actions">{link_button("primary", "Try again", "dashboard.html", "refresh-cw")}</div></div>')
    return centred("Fault", content)


# ---- about ----

INFO_DOC = """{
  "product": "FerroBRIDGE",
  "version": "0.0.12",
  "build": {
    "commit": "6af12584e817c0d4b2a9f1e3c5d7b9a1f2e4c6d8",
    "commit_short": "6af1258",
    "built_at": "2026-10-19T22:31:22Z",
    "rustc": "rustc 1.99.1"
  },
  "pins": [
    { "name": "FHIRconnect", "version": "v1.0.0" },
    { "name": "FHIR", "version": "R4 (4.0.1)" },
    { "name": "OMOCL", "version": "v1.0.0" },
    { "name": "OMOP CDM", "version": "v5.4" },
    { "name": "openEHR ITS-REST", "version": "1.1.0" },
    { "name": "openehr-* crates", "version": "0.0.87" },
    { "name": "fhir-types", "version": "0.1.127" }
  ],
  "lanes": [
    { "name": "facade", "enabled": true },
    { "name": "hl7v2", "enabled": true, "listen": "0.0.0.0:2575", "up": true },
    { "name": "operations", "enabled": true },
    { "name": "etl", "enabled": true },
    { "name": "terminology", "enabled": true }
  ]
}"""


def console_rows():
    from settings import srow
    return (srow("Bridge", "The bridge this console reads, with your own token.", "<code>https://bridge.demo-regio.example.org</code>", "bridge.base_url")
            + srow("Identity provider", "Where operators sign in.", "<code>https://idp.demo-regio.example.org/realms/bridge</code>", "oidc.issuer")
            + srow("Client", "The console's client at the identity provider.", "<code>ferrobridge-console</code>", "oidc.client_id")
            + srow("Sessions", "Ended after 30 minutes without a request and after 12 hours at the latest; at most 10,000 at once.", "30 min · 12 h · 10,000",
                   "session.idle_timeout_s", keys=("session.idle_timeout_s", "session.absolute_timeout_s", "session.max_sessions")))


def page_about():
    import tpl
    def facts(rows):
        return '<dl class="fui-facts">' + "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rows) + "</dl>"
    pins = ", ".join(f"{n} {v}" for n, v in PINS[:5])
    vers = f'<section class="fui-card"><h2 class="fui-card-title"><span>Versions</span></h2>{facts([("Console", f"ferrobridge-console {VERSION}"), ("Bridge", f"FerroBRIDGE {VERSION} · commit <code>{COMMIT}</code>"), ("Built", "2026-10-19 22:31 UTC · rustc 1.99.1"), ("Specifications", pins), ("FerroUI", "ferrohealth-ui 0.3.0")])}</section>'
    maker = f'<section class="fui-card"><h2 class="fui-card-title"><span>Manufacturer</span></h2>{facts([("Made by", "Cadasto B.V."), ("Licence", "Business Source License 1.1"), ("Support", '<a href="mailto:info@cadasto.com">info@cadasto.com</a>'), ("Documentation", '<a href="https://ferrobridge.eu/docs/" rel="external noopener">The FerroBRIDGE book</a>')])}</section>'
    acts = copy_button("Copy", "the info document") + button("secondary", "Download", "download", small=True)
    doc = (f'<section class="fui-card fui-panel mk-options-card"><div class="fui-panel-head"><h2 class="fui-card-title"><span>The bridge\'s info document</span></h2><div class="fui-actions">{acts}</div></div>'
           f'<div class="fui-panel-body fui-stack"><p class="fui-small fui-muted">What the bridge answers at <code>GET /health/info</code>: its version, build, pins and lanes.</p>'
           f'<div class="fui-code-panel mk-options-code" role="region" aria-label="The info document" tabindex="0"><pre>{json_html(INFO_DOC)}</pre></div></div></section>')
    console = (f'<section class="fui-card fui-panel"><div class="fui-panel-head"><h2 class="fui-card-title"><span>This console</span></h2></div>'
               + tpl.settings_panel(console_rows(), 'Set in <code>ferrobridge-console.toml</code> · read at start') + "</section>")
    content = (page_heading("About", subtitle="The console, the bridge it reads, and the manufacturer.")
               + f'<div class="fui-stack"><div class="fui-grid">{vers}{maker}</div>{console}{doc}</div>')
    return shell("About", "about", content)


# ---- keyboard and focus ----

def page_shortcuts():
    import pg_edit
    legend = shortcut_legend([
        ("Everywhere", [("/", "Find a context, a mapping file or a template"), ("?", "This legend"), ("Esc", "Close a dialog, a menu or a panel"),
                        ("g d", "Go to the dashboard"), ("g m", "Go to the Mapping set"), ("g p", "Go to the Playground"), ("g c", "Go to the configuration")]),
        ("Lists", [("j | down", "Next row"), ("k | up", "Previous row"), ("Enter", "Open the row"), ("left | right", "Previous and next page")]),
        ("Editor", [("Ctrl Enter", "Check the draft against the set"), ("Ctrl S", "Go to the save's reason"), ("F8", "Next diagnostic")]),
        ("Playground", [("Ctrl Enter", "Run the transform")]),
    ])
    return pg_edit.page_edit(overlay=legend)


def page_focus_rings():
    def row(title, html):
        return f'<div class="mk-focus-row"><span class="mk-focus-name">{title}</span><div class="mk-focus-demo">{html}</div></div>'
    rows = (
        row("Primary button", button("primary", "Run $tofhir", "play", cls="mk-focus", keys="Ctrl Enter"))
        + row("Secondary button", button("secondary", "Check", "list-checks", cls="mk-focus"))
        + row("Icon button", icon_button("ellipsis", "More actions", cls="mk-focus"))
        + row("Link", '<a href="file.html" class="mk-focus-ring mk-inline-link">synthetic_body_weight.context.yml</a>')
        + row("Text field", input_("f1", "fhirconnect/allergy.context.yml", cls="fui-input mk-focus"))
        + row("Select", select("f2", [("all", "Every language")], "all", cls="fui-select mk-focus"))
        + row("Switch", '<div class="fui-check"><input type="checkbox" id="f4" class="fui-switch mk-focus" checked /><label for="f4" class="fui-label">Display fallback</label></div>')
        + row("Segmented control", '<div role="group" class="fui-segmented"><button class="fui-segment" aria-pressed="true">Canonical</button><button class="fui-segment mk-focus">FLAT</button></div>')
        + row("Tab", tabbed(tabs("Example", [("source", "Source", "file.html"), ("program", "Program", "file-program.html")], "source", focus="program"), '<span class="fui-small fui-muted">Panel</span>'))
        + row("Search control", search_control("", "focus", id_="find-f"))
        + row("Editor", '<div class="fui-editor mk-focus-within"><div class="fui-editor-stack mk-short-stack"><pre class="fui-editor-layer" aria-hidden="true"><span class="fui-code-line"><span class="fui-tok-key">grammar</span>: FHIRConnect/v1.0.0</span></pre><textarea class="fui-editor-input" aria-label="Mapping file">grammar: FHIRConnect/v1.0.0</textarea></div></div>')
        + row("Diagnostic link", '<a class="fui-diag-where mk-focus-ring" href="file-line.html#L17">line 17, column 7</a>')
        + row("Menu item", menu([menu_item("Copy path", "copy", focus=True), menu_item("Download", "download")], anchored=False))
    )
    content = page_heading("Focus, on every control", subtitle="One ring, in the focus colour at 3:1 contrast, on every control type.") + f'<div class="fui-card mk-focus-sheet">{rows}</div>'
    return shell("Focus rings", "", content)


def _frame(title, line, html, cls=""):
    return f'<div class="fui-card mk-state-frame {cls}"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div class="mk-menu-stage">{html}</div></div>'


def file_menu_items(hover=None, focus=None, desc=False, disabled=False):
    return [menu_item("Copy path", "copy", hover=hover == "copy", focus=focus == "copy", desc="fhirconnect/synthetic_body_weight.context.yml" if desc else None),
            menu_item("Download", "download", desc="The bytes the bridge serves" if desc else None),
            menu_item("Check against the set", "list-checks", disabled="The bridge is reloading: try again in a moment" if disabled else None,
                      desc="Compiles the set with this file; writes nothing" if desc else ("The bridge is reloading: try again in a moment" if disabled else None))]


def trig(open_=True):
    return (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="More actions for synthetic_body_weight.context.yml" title="More actions">{icon("ellipsis", "16")}</button>')


def page_menu_states():
    def demo(items, up=False, open_=True):
        m = menu(items, label="Actions for the file", up=up) if open_ else ""
        return f'<div class="mk-menu-demo{" mk-menu-demo-up" if up else ""}"><span class="fui-anchor">{trig(open_)}{m}</span></div>'
    frames = (
        _frame("Rest", "4 px under its trigger, aligned to its end edge; ink text, muted icons", demo(file_menu_items()))
        + _frame("Hover", "the neutral tint, the icon to ink", demo(file_menu_items(hover="copy")))
        + _frame("Keyboard focus", "the same tint, plus the focus ring; ↑ ↓ Home End and a letter move, Enter activates, Esc returns to the trigger", demo(file_menu_items(focus="copy")))
        + _frame("Disabled, with its reason", "muted and skipped by the keyboard; its reason is the line under the label", demo(file_menu_items(disabled=True)))
        + _frame("With descriptions", "one muted line under a label when the action needs it", demo(file_menu_items(desc=True)))
        + _frame("Flipped up", "no room below: it opens above its trigger and stays in the viewport", demo(file_menu_items(), up=True))
    )
    sheet = '<div class="mk-sheet-demo">' + menu(file_menu_items(), label="Actions for the file", sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>"
    frames += _frame("Phone: the bottom sheet", "below 40 rem a row menu or More is a sheet: its title, 48 px rows, Cancel", sheet)
    keys = card("Rules", definitions([
        ("Roles", 'role="menu" and role="menuitem"; aria-haspopup and aria-expanded on the trigger'),
        ("Destructive", "FerroBRIDGE's menus hold none today: the operator API deletes no file. When one comes, it goes last, after a separator, in the danger tone"),
        ("Never clipped", "it lives in the top layer, above every row and panel"),
    ]))
    content = page_heading("Menu, states", subtitle="F13 in FerroBRIDGE: the one anchored menu behind every ⋯, every More and the user menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}{keys}</div>'
    return shell("Menu states", "", content)


def inline(html):
    """A specimen's open menu stays in its card at every width: no phone sheet."""
    import re
    html = re.sub(r'<p class="fui-menu-sheet-title">.*?</p>|<button type="button" class="fui-menu-cancel">Cancel</button>', "", html)
    return html.replace("fui-menu fui-menu-compact fui-menu-anchored fui-menu-sheet", "fui-menu fui-menu-compact fui-menu-anchored")


def page_action_group_states():
    from common import action_group
    edit = link_button("primary", "Edit", "edit.html", "pencil")
    items = file_menu_items()
    frames = (
        _frame("Rest", "one primary action and one More button; nothing else in the title row", f'<div class="mk-menu-demo">{action_group(edit, items, label="More actions for the file")}</div>')
        + _frame("More open", "the same anchored menu", f'<div class="mk-menu-demo">{inline(action_group(edit, items, open_=True, label="More actions for the file"))}</div>')
        + _frame("A provisioned file", "the primary becomes Check a draft: the console never saves over it", f'<div class="mk-menu-demo">{action_group(link_button("primary", "Check a draft", "edit-provisioned.html", "list-checks"), items, label="More actions for the file")}</div>')
        + _frame("The Mapping set", "New file, and More with the set's two actions",
                 f'<div class="mk-menu-demo">{inline(action_group(link_button("primary", "New file", "edit-new.html", "file-plus"), set_menu_items(), open_=True, label="More actions for the mapping set"))}</div>')
    )
    sheet = '<div class="mk-sheet-demo">' + menu(items, label="More actions for the file", sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>"
    frames += _frame("Phone", "the primary stays; More opens the bottom sheet", sheet)
    content = page_heading("Action group, states", subtitle="F14 in FerroBRIDGE: the title row's one primary action and its More menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}</div>'
    return shell("Action group states", "", content)


def set_menu_items(hover=None):
    return [menu_item("Check the set", "list-checks", desc="Compiles every file on disk; writes nothing"),
            menu_item("Reload the set…", "refresh-cw", href="mappings-reload.html", hover=hover == "reload", desc="Replaces the running set when it compiles")]
