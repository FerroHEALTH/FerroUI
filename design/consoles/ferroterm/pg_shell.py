# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The shell and global pages: the dashboard and its states, the top-bar
search and the find page, the sidebar, the user menu and the version
select, toasts, the status pages, About, the focus sheet, the menu and
action-group sheets and the shortcut legend."""
import tpl
from common import (
    MENU_SEP, action_group, badge, button, card, centred, copy_button, definitions, disclosure, e, empty_state, health_dot, icon, icon_button,
    input_, link_button, menu, menu_item, notice, page_heading, refusal, search_control, select, shell, shortcut_legend,
    stat_card, table, tabbed, tabs, td, th, toast, toasts, when,
)
from data import BASE, CODE_SYSTEMS, FHIR_BASE, VERSION, WRITES
from parts import skeleton_lines
from term import canon, facts, json_html, quiet, source_tag

# ---- dashboard ----


def dashboard_heading(as_of=", as it answered at 09:42", root="R4"):
    return page_heading("Dashboard", subtitle=f"What <code>terminology.example.org</code> serves on FHIR {root}{as_of}.")


def stats():
    s1 = stat_card("book-open", "15", "Code systems", badge("neutral", "3 stored on this server"), href="code-systems.html")
    s2 = stat_card("layers", "13", "Value sets", badge("neutral", "8 stored on this server"), href="value-sets.html")
    s3 = stat_card("arrow-left-right", "4", "Concept maps", badge("neutral", "3 stored on this server"), href="concept-maps.html")
    s4 = stat_card("history", "7", "Recent writes", '<span class="fui-small fui-muted" title="7 writes from 2026-10-14 to 2026-10-21">since 14 Oct</span>', href="#recent-changes")
    return f'<div class="fui-grid fui-grid-stats">{s1}{s2}{s3}{s4}</div>'


def write_when(w):
    if "T" in w:
        d, hm = w.split("T")
        day = {"2026-10-20": "yesterday", "2026-10-19": "2 days ago", "2026-10-16": "5 days ago", "2026-10-14": "7 days ago"}[d]
        return f'<time class="fui-time" datetime="{w}" title="{d} {hm} CEST">{day}</time>'
    return tpl.rel(w)


TYPE_WORD = {"CodeSystem": "Code system", "ValueSet": "Value set", "ConceptMap": "Concept map"}


def recent_card(rows_n=7):
    rows = []
    for typ, title, ver, how, w, href in WRITES[:rows_n]:
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{e(title)}</a><span class="fui-row-sub">{TYPE_WORD[typ]} · version {ver}</span></th>'
                    + td("Change", "Created" if how == "create" else "Updated", role="meta-q")
                    + td("When", write_when(w), role="meta-2") + "</tr>")
    t = table("Recent changes", th("Resource") + th("Change") + th("When"), rows)
    return tpl.card(t, label="Recent changes", title="Recent changes", count=len(rows), id_="recent-changes")


def systems_card():
    from pg_res import versions_word
    rows = []
    for key, name, url, vers, src, content, hier, subs, status, n, href in CODE_SYSTEMS[:6]:
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{e(name)}</a></th>'
                    + td("Source", source_tag(src), role="meta-q") + td("Versions", versions_word(vers), role="meta-2") + "</tr>")
    t = table("Code systems", th("Code system") + th("Source") + th("Versions"), rows)
    return tpl.card(t, label="Code systems", title="Code systems", count="6 of 15", actions=link_button("ghost", "All code systems", "code-systems.html", glyph_end="arrow-right", small=True))


def page_dashboard(search_state=None, search_value="", results=None, menu_open=False, menu_theme="system", menu_focus=None,
                   rail=False, rail_tip=None, drawer=False, toast_html="", banner=False, visitor=False, version_open=False, overlay=None, reader=False,
                   as_of=", as it answered at 09:42", status="up"):
    content = dashboard_heading(as_of) + f'<div class="fui-stack fui-gap-section">{stats()}<div class="fui-grid fui-grid-start">{recent_card()}{systems_card()}</div></div>'
    return shell("Dashboard", "dashboard", content, search_state=search_state, search_value=search_value, results=results,
                 menu_open=menu_open, menu_theme=menu_theme, menu_focus=menu_focus, rail=rail, rail_tip=rail_tip, drawer=drawer,
                 toast_html=toast_html, banner=banner, visitor=visitor, version_open=version_open, overlay=overlay, reader=reader,
                 dev=banner, status=status)


def page_dashboard_loading():
    st = '<div class="fui-grid fui-grid-stats">' + "".join(
        f'<div class="fui-card fui-stat" aria-hidden="true"><span class="fui-stat-glyph mk-sk-glyph"></span><div class="fui-stat-text mk-grow">{skeleton_lines([40, 80])}</div></div>'
        for _ in range(4)) + "</div>"
    sk = f'<div class="fui-card mk-sk-card" aria-hidden="true">{skeleton_lines([30, 100, 90, 100, 80, 95])}</div>'
    content = page_heading("Dashboard", subtitle="Reading what <code>terminology.example.org</code> serves on FHIR R4.") + '<p class="fui-sr-only" role="status">Reading what this server serves</p>' + f'<div class="fui-stack fui-gap-section" aria-busy="true">{st}<div class="fui-grid fui-grid-start">{sk}{sk}</div></div>'
    return shell("Dashboard", "dashboard", content)


def page_dashboard_empty():
    body = quiet('FHIR R6 serves no code systems, value sets or concept maps on this server. <a href="dashboard.html">Read FHIR R4 instead</a>')
    content = dashboard_heading(root="R6") + f'<div class="fui-stack">{body}</div>'
    return shell("Dashboard", "dashboard", content, version="R6")


def tech_detail(failure="Connection refused at the server's address", rid="01JAH2V6Q8R4"):
    return disclosure("Technical detail", definitions([
        ("Failure", failure), ("Internal address", '<code>http://ferroterm:8080/</code> <span class="fui-small fui-muted">base_url in console.toml</span>'),
        ("Request id", f"<code>{rid}</code>"), ("Time", when("2026-10-21T09:42:11", seconds=True)), ("Attempts", "3 of 3, the last 4 seconds ago"),
    ]))


def unreachable_block(what):
    return (
        refusal(f"The terminology server refused the connection, so {what} cannot be shown.")
        + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried {when("2026-10-21T09:42:11", seconds=True)} · request id <code>01JAH2V6Q8R4</code></span></div>'
    )


def page_server_unreachable():
    content = (
        dashboard_heading("") + '<div class="fui-stack">' + unreachable_block("the dashboard")
        + '<p class="fui-small fui-muted">The console keeps no copy of the server, so nothing shows until it answers; your session is kept.</p>'
        + tech_detail() + "</div>"
    )
    return shell("Dashboard", "dashboard", content, status="down")


def page_dashboard_visitor():
    return page_dashboard(visitor=True)


def page_dashboard_dev():
    return page_dashboard(banner=True)


# ---- the top-bar search and the find page ----

def results_list(kind):
    if kind == "none":
        body = (
            f'<div class="mk-results-empty">{icon("search-x", "16")}<div><strong>No code system, value set or concept map on FHIR R4 has this canonical.</strong>'
            '<span>Another FHIR version may hold it: switch the FHIR version beside the search, and search again.</span></div></div>'
        )
        return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'
    groups = [
        ("Value sets", [("Synthetic respiratory findings", "Value set · stored on this server · version 2", "value-set.html", True, "layers")]),
        ("With what you typed", [("Search the designations for “respir”", "In the concept browser, in a code system you choose", "browse-results.html", False, "search"),
                                 ("Every place it can go", "The find page", "find.html", False, "arrow-right")]),
    ]
    body = '<p class="mk-results-head">1 match for “respir”</p>'
    for g, items in groups:
        body += f'<p class="mk-results-group">{g}</p><ul role="listbox" aria-label="{g}">' + "".join(
            f'<li role="option"{" aria-selected=\"true\"" if sel else ""}><a href="{href}">{icon(gl, "14")}<span class="mk-results-name">{e(n)}</span>'
            f'<span class="fui-small fui-muted">{e(line)}</span></a></li>' for n, line, href, sel, gl in items) + "</ul>"
    return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'


def topbar_state_page(state_=None, value="", results=None):
    return page_dashboard(search_state=state_, search_value=value, results=results)


def page_find():
    rows = [
        ("Look up SF-1211", "Opens Look up with the code filled in. Name the system it belongs to.", "lookup-answered.html", "search"),
        ("Check SF-1211", "Opens Check with the code filled in, ready for a code system or a value set.", "check-valid.html", "shield-check"),
        ("Translate SF-1211", "Opens Translate with the code filled in. Name the system it belongs to.", "translate.html", "corner-down-right"),
        ("Search the designations for SF-1211", "Searches the concept browser for it, in the code system you pick there.", "browse-results.html", "search"),
    ]
    items = "".join(
        f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{h}">{e(t)}</a><span class="fui-row-sub">{e(line)}</span></th></tr>'
        for t, line, h, g in rows)
    t = table("Where it can go", th('<span class="fui-sr-only">Screen</span>'), items)
    head = page_heading("Find", subtitle="What you typed, read by its shape, and the screens that take it.")
    read = notice("info", "info", "<strong><code>SF-1211</code></strong> reads as a code from its shape alone. No code system, value set or concept map on FHIR R4 has it as a name or a canonical.")
    content = head + f'<div class="fui-list-page">{read}{tpl.card(t, label="Where it can go", title="Where it can go", count=4)}</div>'
    return shell("Find", "", content, search_value="SF-1211")


def page_search_states():
    states = [
        ("Default", "the icon, the placeholder and the quiet / hint", "", None),
        ("Hover", "the edge takes the ink", "", "hover"),
        ("Focused, empty", "one ring round the whole group; the hint is gone", "", "focus"),
        ("Focused and filled", "the ring, the typed value, the hint gone", "respir", "focus-filled"),
        ("Filled, not focused", "the hint stays hidden while the field holds text", "respir", None),
        ("Busy", "the icon becomes the spinner while the search runs", "respir", "busy"),
        ("Disabled", "the sunken ground, no pointer; the server is unreachable", "", "disabled"),
    ]
    cards = ""
    for i, (title, line, value, st) in enumerate(states):
        cards += (f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                  f'<div class="mk-search-demo">{search_control(value, st, id_=f"find-{i}")}</div></div>')
    keys = card("What it finds", definitions([
        ("Code systems, value sets, concept maps", "by name, title or canonical, on the FHIR version in view"),
        ("A code", "read by its shape: offers Look up, Check and Translate with it filled in"),
        ("A phrase", "offers a search of the designations in the concept browser"),
        ("Keys", "/ focuses it at wide widths; ↑ and ↓ move through the matches, Enter opens one, Esc closes"),
    ]))
    content = page_heading("Search control, states", subtitle="One joined control that finds a resource, or says where a code or a phrase can go.") + f'<div class="fui-stack mk-search-sheet">{cards}{keys}</div>'
    return shell("Search states", "", content)


# ---- shell states ----

def page_sidebar_rail():
    return page_dashboard(rail=True, rail_tip="systems")


def page_sidebar_drawer():
    import pg_res
    return pg_res.page_code_system().replace('class="fui-sidebar"', 'class="fui-sidebar mk-drawer-open"', 1).replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1).replace('<main id="main"', '<div class="mk-scrim mk-scrim-drawer" aria-hidden="true"></div><main id="main"', 1)


def page_user_menu():
    return page_dashboard(menu_open=True)


def page_user_menu_dark():
    html = page_dashboard(menu_open=True, menu_theme="dark")
    return html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)


def page_user_menu_signout():
    return page_dashboard(menu_open=True, menu_focus="signout")


def page_visitor_menu():
    return page_dashboard(visitor=True, menu_open=True)


def page_version_menu():
    return page_dashboard(version_open=True)


def page_toast(kind):
    import pg_res
    if kind == "ok":
        return pg_res.vs_page(header=pg_res.vs_header(line="Version 3 · stored on this server"), panel=pg_res.vs_definition(changed="2026-10-21T09:45"),
                              toast_html=toasts(toast("ok", "circle-check", "Saved", "Synthetic respiratory findings · the server now holds version&nbsp;3.")))
    if kind == "warn":
        return page_dashboard(as_of=", as it last answered at 09:41", status="down",
                              toast_html=toasts(toast("warn", "triangle-alert", "The server stopped answering", "The read at 09:42 failed. The screen shows what the server answered at 09:41; a run or a save waits until it answers.",
                                                      action=button("secondary", "Try again", "refresh-cw", small=True))))
    return pg_res.cs_page_detail("history", pg_res.history_panel(),
                                 toast_html=toasts(toast("bad", "circle-alert", "The restore was refused", "Someone changed the code system since you opened it. Reload to read version&nbsp;4.",
                                                         action=link_button("secondary", "Reload", "code-system-history.html", small=True))))


def status_page(code, glyph, title, body, actions, current="dashboard", status_="up", reader=False):
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon(glyph, "24")}</span><p class="fui-status-page-code">{code}</p>'
               f'<h1>{title}</h1><p>{body}</p><div class="fui-actions">{actions}</div></div>')
    return shell(title, current, content, status=status_, reader=reader)


def page_forbidden():
    return status_page("Not allowed · 403", "ban", "This account cannot edit value sets",
                       "You are signed in as Demo Reader, and this account may not edit value sets. You can still read the value set and preview an expansion.</p>"
                       '<p class="fui-small fui-muted">Needs the scope <code>user/ValueSet.cud</code>.',
                       link_button("primary", "Back to the value set", "value-set.html", "arrow-left") + button("secondary", "Sign in again", "log-in"), current="valuesets", reader=True)


def page_not_found():
    return status_page("Not found · 404", "compass", "No such screen",
                       "The console has no page at <code>/valueset/<wbr>synthetic-respiratory</code>. Value sets live under <code>/valuesets</code>.",
                       link_button("primary", "Value sets", "value-sets.html", "layers") + link_button("secondary", "Dashboard", "dashboard.html"), current="")


def page_unreadable():
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon("server-crash", "24")}</span><p class="fui-status-page-code">Fault · 500</p>'
               f'<h1>The console cannot read its sessions</h1><p>It cannot tell whether you are signed in, so it shows nothing. Ask whoever runs the console to restart it; the terminology server keeps answering its clients meanwhile.</p>'
               f'<div class="fui-actions">{link_button("primary", "Try again", "dashboard.html", "refresh-cw")}</div></div>')
    return centred("Fault", content)


# ---- about ----

CAPABILITY = """{
  "resourceType": "CapabilityStatement",
  "status": "active",
  "date": "2026-10-19",
  "kind": "instance",
  "software": { "name": "FerroTERM", "version": "0.1.8" },
  "implementation": {
    "description": "FerroTERM terminology server",
    "url": "https://terminology.example.org/r4"
  },
  "fhirVersion": "4.0.1",
  "format": [ "application/fhir+json", "application/fhir+xml" ],
  "instantiates": [
    "http://hl7.org/fhir/CapabilityStatement/terminology-server"
  ],
  "rest": [
    {
      "mode": "server",
      "resource": [
        { "type": "CodeSystem", "interaction": [
          { "code": "read" }, { "code": "vread" }, { "code": "update" }, { "code": "delete" },
          { "code": "history-instance" }, { "code": "history-type" }, { "code": "create" }, { "code": "search-type" } ] },
        { "type": "ValueSet", "interaction": [
          { "code": "read" }, { "code": "vread" }, { "code": "update" }, { "code": "delete" },
          { "code": "history-instance" }, { "code": "history-type" }, { "code": "create" }, { "code": "search-type" } ] },
        { "type": "ConceptMap", "interaction": [
          { "code": "read" }, { "code": "vread" }, { "code": "update" }, { "code": "delete" },
          { "code": "history-instance" }, { "code": "history-type" }, { "code": "create" }, { "code": "search-type" } ] }
      ],
      "operation": [
        { "name": "lookup", "definition": "http://hl7.org/fhir/OperationDefinition/CodeSystem-lookup" },
        { "name": "validate-code", "definition": "http://hl7.org/fhir/OperationDefinition/CodeSystem-validate-code" },
        { "name": "subsumes", "definition": "http://hl7.org/fhir/OperationDefinition/CodeSystem-subsumes" },
        { "name": "expand", "definition": "http://hl7.org/fhir/OperationDefinition/ValueSet-expand" },
        { "name": "translate", "definition": "http://hl7.org/fhir/OperationDefinition/ConceptMap-translate" }
      ]
    }
  ]
}"""


def page_about():
    roots = ", ".join(f'<span class="fui-nowrap">{r} ({v})</span>' for r, v in (("R4", "4.0.1"), ("R4B", "4.3.0"), ("R5", "5.0.0"), ("R6", "6.0.0-ballot5")))
    vers = card("Versions", facts([("Console", f"FerroTERM Console {VERSION}"), ("Server", f"FerroTERM {VERSION}"),
                                   ("FHIR roots", roots), ("FerroUI", "ferrohealth-ui 0.3.0")]))
    maker = card("Manufacturer", facts([("Made by", "Cadasto B.V."), ("Licence", "Business Source License 1.1"), ("Support", '<a href="mailto:info@cadasto.com">info@cadasto.com</a>'),
                                        ("Documentation", '<a href="https://ferroterm.eu/docs/" rel="external">The FerroTERM book</a>')]))
    acts = copy_button("Copy", "the CapabilityStatement") + button("secondary", "Download", "download", small=True)
    doc = (f'<section class="fui-card fui-panel mk-options-card"><div class="fui-panel-head"><h2 class="fui-card-title"><span>The CapabilityStatement of FHIR R4</span></h2><div class="fui-actions">{acts}</div></div>'
           f'<div class="fui-panel-body fui-stack"><p class="fui-small fui-muted">What the R4 root declares at <code>GET /r4/metadata</code>; <a href="server.html">Server</a> compares the four roots.</p>'
           f'<div class="fui-code-panel fui-self-description fui-scroll-cue" role="region" aria-label="The CapabilityStatement" tabindex="0"><pre>{json_html(CAPABILITY)}</pre></div></div></section>')
    content = (page_heading("About", subtitle="The console, the server it reads, and the manufacturer.")
               + f'<div class="fui-stack"><div class="fui-grid fui-grid-start">{vers}{maker}</div>{doc}</div>')
    return shell("About", "about", content)


# ---- keyboard and focus ----

def page_shortcuts():
    import pg_ops
    legend = shortcut_legend([
        ("Everywhere", [("/", "Find a resource, a code or a phrase"), ("?", "This legend"), ("Esc", "Close a dialog, a menu or a panel"),
                        ("g d", "Go to the dashboard"), ("g c", "Go to code systems"), ("g v", "Go to value sets"), ("g m", "Go to concept maps"), ("g b", "Go to the concept browser"),
                        ("g o", "Go to operations"), ("g s", "Go to the server")]),
        ("Lists", [("j | down", "Next row"), ("k | up", "Previous row"), ("Enter", "Open the row"), ("left | right", "Previous and next page")]),
        ("Concept tree", [("up | down", "Previous and next concept"), ("right", "Open a level, or go to its first child"), ("left", "Close a level, or go to the parent"), ("Enter", "Read the concept")]),
        ("Operations and editors", [("Ctrl Enter", "Run, or run the preview"), ("Ctrl s", "Save")]),
    ])
    return pg_ops.page_lookup_answered(overlay=legend)


def page_focus_rings():
    from term import concept_tree, tree_item

    def row(title, html):
        return f'<div class="mk-focus-row"><span class="mk-focus-name">{title}</span><div class="mk-focus-demo">{html}</div></div>'
    tree = concept_tree([tree_item("SF-1210", "Cough", expanded=False, focus=True)], "Example tree")
    rows = (
        row("Primary button", button("primary", "Look up", "search", cls="mk-focus", keys="Ctrl Enter"))
        + row("Secondary button", button("secondary", "Run the preview", "play", cls="mk-focus"))
        + row("Icon button", icon_button("ellipsis", "More actions", cls="mk-focus"))
        + row("Link", '<a href="value-set.html" class="mk-focus-ring mk-inline-link">Synthetic respiratory findings</a>')
        + row("Text field", input_("f1", "SF-1211", cls="fui-input mk-focus"))
        + row("Select", select("f2", [("sf", "Synthetic clinical findings")], "sf", cls="fui-select mk-focus"))
        + row("Version select", '<button type="button" class="fui-version-select mk-focus mk-focus-ring" aria-haspopup="menu" aria-expanded="false" title="FHIR version: R4 (FHIR 4.0.1)">R4' + icon("chevron-down", "14") + "</button>")
        + row("Check box", '<div class="fui-check"><input type="checkbox" id="f4" class="fui-checkbox mk-focus" checked /><label for="f4" class="fui-label">Active concepts only</label></div>')
        + row("Segmented control", '<div role="group" class="fui-segmented"><button class="fui-segment" aria-pressed="true">Membership</button><button class="fui-segment mk-focus">Relation</button></div>')
        + row("Tab", tabbed(tabs("Example", [("def", "Definition", "value-set.html"), ("exp", "Expansion", "value-set-expansion.html"), ("hist", "History", "value-set-history.html")], "def", focus="exp"), '<span class="fui-small fui-muted">Panel</span>'))
        + row("Search control", search_control("", "focus", id_="find-f"))
        + row("Concept tree", tree)
        + row("Menu item", menu([menu_item("Copy canonical", "copy", focus=True), menu_item("Download JSON", "download")], anchored=False))
    )
    content = page_heading("Focus, on every control", subtitle="One focus ring, at 3:1 contrast against its ground, on every control type.") + f'<div class="fui-card mk-focus-sheet">{rows}</div>'
    return shell("Focus rings", "", content)


def _frame(title, line, html, cls=""):
    return f'<div class="fui-card mk-state-frame {cls}"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div class="mk-menu-stage">{html}</div></div>'


def vs_menu_items(hover=None, focus=None, desc=False, disabled=False):
    return [menu_item("Expand in Operations", "list-tree", href="expand-answered.html", hover=hover == "exp", focus=focus == "exp", desc="Every page of the expansion" if desc else None),
            menu_item("Check a code in it", "shield-check", href="check.html", desc="ValueSet/$validate-code with this canonical" if desc else None),
            menu_item("Copy canonical", "copy", desc="The value set's canonical, to the clipboard" if desc else None,
                      disabled="The clipboard is not available in this browser" if disabled else None),
            menu_item("Download JSON", "download")]


def trig(open_=True):
    return (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="More actions for Synthetic respiratory findings" title="More actions">{icon("ellipsis", "16")}</button>')


def page_menu_states():
    def demo(items, up=False, open_=True):
        m = menu(items, label="More actions for the value set", up=up) if open_ else ""
        return f'<div class="mk-menu-demo{" mk-menu-demo-up" if up else ""}"><span class="fui-anchor">{trig(open_)}{m}</span></div>'
    frames = (
        _frame("Rest", "4 px under its trigger, aligned to its end edge; ink text, muted icons", demo(vs_menu_items()))
        + _frame("Hover", "the neutral tint, the icon to ink", demo(vs_menu_items(hover="exp")))
        + _frame("Keyboard focus", "the same tint, plus the focus ring; ↑ ↓ Home End and a letter move, Enter activates, Esc returns to the trigger", demo(vs_menu_items(focus="exp")))
        + _frame("Disabled, with its reason", "muted and skipped by the keyboard; the reason is its tooltip", demo(vs_menu_items(disabled=True)))
        + _frame("With descriptions", "one muted line under a label when the action needs it", demo(vs_menu_items(desc=True)))
        + _frame("Flipped up", "no room below: it opens above its trigger and stays in the viewport", demo(vs_menu_items(), up=True))
    )
    sheet = '<div class="mk-sheet-demo">' + menu(vs_menu_items(), label="More actions for the value set", sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>"
    frames += _frame("Phone: the bottom sheet", "below 40 rem a row menu or More is a sheet: its title, 48 px rows, Cancel", sheet)
    keys = card("Rules", definitions([
        ("Roles", 'role="menu" and role="menuitem"; aria-haspopup and aria-expanded on the trigger'),
        ("Destructive", "The console deletes nothing today. When it does, Delete goes last, after a separator, in the danger tone, with an ellipsis"),
        ("Never clipped", "it lives in the top layer, above every row and panel"),
    ]))
    content = page_heading("Menu, states", subtitle="F13 in FerroTERM: the one anchored menu behind every ⋯, every More and the user menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}{keys}</div>'
    return shell("Menu states", "", content)


def page_action_group_states():
    edit = link_button("primary", "Edit", "value-set-edit.html", "pencil")
    items = vs_menu_items()
    frames = (
        _frame("A value set stored on this server", "Edit, with the operations in More", f'<div class="mk-menu-demo">{action_group(edit, items, label="More actions for the value set")}</div>')
        + _frame("More open", "the same anchored menu", f'<div class="mk-menu-demo">{action_group(edit, items, open_=True, label="More actions for the value set", sheet=False)}</div>')
        + _frame("A value set the console cannot edit", "the primary becomes Check a code; More holds the rest", f'<div class="mk-menu-demo">{action_group(link_button("primary", "Check a code in it", "check.html", "shield-check"), items[2:], label="More actions for the value set")}</div>')
        + _frame("A code system stored on this server", "Edit, and More with Browse concepts and the operations",
                 f'<div class="mk-menu-demo">{action_group(link_button("primary", "Edit", "code-system-edit.html", "pencil"), [menu_item("Browse concepts", "network", href="browse.html"), menu_item("Look up a code in it", "search", href="lookup.html")], open_=True, label="More actions for the code system", sheet=False)}</div>')
    )
    sheet = '<div class="mk-sheet-demo">' + menu(items, label="More actions for the value set", sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>"
    frames += _frame("Phone", "the primary stays; More opens the bottom sheet", sheet)
    content = page_heading("Action group, states", subtitle="F14 in FerroTERM: the title row's one primary action and its More menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}</div>'
    return shell("Action group states", "", content)
