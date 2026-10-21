# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The shell and global pages: the dashboard and its states, the top-bar
search and the find page, the sidebar, the user menu, toasts, the status
pages, About, the focus sheet, the menu and action-group sheets and the
shortcut legend."""
import tpl
from common import (
    MENU_SEP, action_group, badge, button, card, centred, copy_button, definitions, disclosure, e, icon, icon_button,
    input_, link_button, menu, menu_item, notice, page_heading, refusal, search_control, select, shell, shortcut_legend,
    stat_card, table, tabbed, tabs, td, th, toast, toasts, when,
)
from data import COMP_COUNT, EHR_COUNT, E1, NAMESPACES, QUERIES, TEMPLATES, VERSION
from ehr import codeview, empty_line, json_html, masked, plural, quiet, short, state
from parts import line_chart, skeleton_lines

# ---- dashboard ----


def dashboard_heading(line="What <code>cdr.example.org</code> holds, as it answered at 09:42."):
    return page_heading("Dashboard", subtitle=line)


def stats(zero=False):
    s1 = stat_card("database", "0" if zero else f"{EHR_COUNT:,}", "EHRs", href="ehrs.html")
    s2 = stat_card("file-text", "0" if zero else f"{COMP_COUNT:,}", "Compositions", href="query.html")
    s3 = stat_card("file-code", "0" if zero else str(len(TEMPLATES)), "Templates", href="templates.html")
    s4 = stat_card("scroll-text", "0" if zero else str(len(QUERIES)), "Stored queries", href="stored-queries.html")
    return f'<div class="fui-grid fui-grid-stats">{s1}{s2}{s3}{s4}</div>'


DAYS = [3, 5, 2, 6, 4, 8, 7, 5, 9, 6, 4, 11, 8, 7, 12, 9, 10, 6, 8, 13, 11, 9, 14, 12, 10, 15, 11, 13, 16, 12]


def activity_card():
    chart = line_chart("trend", "Compositions per day", "Compositions per day by their start time, 22 September to 21 October 2026: from 2 to 16 a day, rising.",
                       [("Compositions", DAYS, 0)], [(0, "22 Sep"), (9, "1 Oct"), (19, "11 Oct"), (29, "21 Oct")], height=200, legend=False)
    body = f'<div class="fui-panel-body">{chart}<p class="fui-small fui-muted">By start time, over the newest 500 compositions; read with one AQL query.</p></div>'
    head = '<div class="fui-panel-head fui-list-head"><h2 class="fui-card-title"><span>Compositions per day</span></h2></div>'
    return f'<section class="fui-card fui-panel">{head}{body}</section>'


def namespaces_card(failed=False):
    rows = []
    for ns, n, m in NAMESPACES:
        q = f"{n} {'query' if n == 1 else 'queries'}"
        if failed and ns == "org.example.quality":
            val = state("warn", "No total")
            q += ' · <a href="stored-query.html">hba1c_overdue 1.0.0</a> did not run'
        else:
            val = f"{m:,} matches"
        label = f"<code>{ns}</code>" if ns != "unqualified" else "Unqualified"
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="stored-queries-namespace.html">{label}</a><span class="fui-row-sub">{q}</span></th>'
                    + td("Matches", val, "fui-num", role="meta-q") + "</tr>")
    t = table("Matches by namespace", th("Namespace") + th("Matches", num=True), rows)
    return tpl.card(t, label="Matches by namespace", title="Matches by namespace", count=len(rows))


def page_dashboard(search_state=None, search_value="", results=None, menu_open=False, menu_theme="light", menu_focus=None,
                   rail=False, rail_tip=None, drawer=False, toast_html="", sandbox=False, overlay=None, failed=False, probes=True, status="up"):
    content = dashboard_heading() + f'<div class="fui-stack fui-gap-section">{stats()}<div class="fui-grid fui-grid-start">{activity_card()}{namespaces_card(failed)}</div></div>'
    return shell("Dashboard", "dashboard", content, search_state=search_state, search_value=search_value, results=results,
                 menu_open=menu_open, menu_theme=menu_theme, menu_focus=menu_focus, rail=rail, rail_tip=rail_tip, drawer=drawer,
                 toast_html=toast_html, sandbox=sandbox, overlay=overlay, probes=probes, status=status)


def page_dashboard_loading():
    st = '<div class="fui-grid fui-grid-stats">' + "".join(
        f'<div class="fui-card fui-stat" aria-hidden="true"><span class="fui-stat-glyph mk-sk-glyph"></span><div class="fui-stat-text mk-grow">{skeleton_lines([40, 80])}</div></div>'
        for _ in range(4)) + "</div>"
    sk = f'<div class="fui-card mk-sk-card" aria-hidden="true">{skeleton_lines([30, 100, 90, 100, 80, 95])}</div>'
    content = dashboard_heading("Reading what <code>cdr.example.org</code> holds…") + f'<div class="fui-stack fui-gap-section" aria-busy="true">{st}<div class="fui-grid fui-grid-start">{sk}{sk}</div></div>'
    return shell("Dashboard", "dashboard", content)


def page_dashboard_empty():
    act = '<section class="fui-card fui-panel"><div class="fui-panel-head fui-list-head"><h2 class="fui-card-title"><span>Compositions per day</span></h2></div>' + empty_line("No composition is committed yet.") + "</section>"
    ns = '<section class="fui-card fui-panel"><div class="fui-panel-head fui-list-head"><h2 class="fui-card-title"><span>Matches by namespace</span></h2></div>' + empty_line("No stored query yet.") + "</section>"
    steps = f'<div class="fui-cluster">{link_button("primary", "Upload a template", "template-upload.html", "upload")}{link_button("secondary", "New EHR", "ehr-new.html", "plus")}</div>'
    content = dashboard_heading() + f'<div class="fui-stack fui-gap-section">{stats(zero=True)}{steps}<div class="fui-grid fui-grid-start">{act}{ns}</div></div>'
    return shell("Dashboard", "dashboard", content)


def tech_detail():
    return disclosure("Technical detail", definitions([
        ("Failure", "Connection refused at the CDR's address"), ("CDR", "<code>http://ferroehr:8080/<wbr>ferroehr/<wbr>rest/<wbr>openehr/<wbr>v1</code>, the viewer's <code>cdr.base_url</code> and <code>cdr.base_path</code>"),
        ("Request id", "<code>01JAH2V6Q8R4</code>"), ("Time", when("2026-10-21T09:42:11", seconds=True)),
    ]))


def page_cdr_unreachable():
    content = (
        dashboard_heading("The CDR last answered at 09:38.") + '<div class="fui-stack">'
        + refusal("The CDR refused the connection, so the dashboard cannot be shown. Your session is kept.")
        + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried {when("2026-10-21T09:42:11", seconds=True)} · request id <code>01JAH2V6Q8R4</code></span></div>'
        + tech_detail() + "</div>"
    )
    return shell("Dashboard", "dashboard", content, status="down", search_state="disabled")


def page_dashboard_sandbox():
    return page_dashboard(sandbox=True)


def page_dashboard_namespace_failed():
    return page_dashboard(failed=True)


# ---- the top-bar search and the find page ----

def results_list(kind):
    if kind == "none":
        body = (
            f'<div class="mk-results-empty">{icon("search-x", "16")}<div><strong>The CDR holds nothing with that id</strong>'
            '<span>The search reads EHR ids, subject ids, composition uids, template ids, stored query names and party ids.</span></div></div>'
            f'<a href="find.html" class="mk-results-more">Where it can go{icon("arrow-right", "14")}</a>'
        )
        return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'
    groups = [
        ("Templates", [("Vital signs encounter.v1", "ADL 1.4 · 1,284 compositions", "template.html", True, "file-code")]),
        ("Stored queries", [("recent_vitals", "Unqualified · version 1.0.0", "stored-query.html", False, "scroll-text")]),
        ("Compositions", [("Vital signs", "today 07:50 · EHR 0199c2a4…1a01", "composition.html", False, "file-text")]),
    ]
    body = '<p class="mk-results-head">3 matches for “vital”</p>'
    for g, items in groups:
        body += f'<p class="mk-results-group">{g}</p><ul role="listbox" aria-label="{g}">' + "".join(
            f'<li role="option"{" aria-selected=\"true\"" if sel else ""}><a href="{href}">{icon(gl, "14")}<span class="mk-results-name">{e(n)}</span>'
            f'<span class="fui-small fui-muted">{e(line)}</span></a></li>' for n, line, href, sel, gl in items) + "</ul>"
    return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'


def subject_results():
    body = ('<p class="mk-results-head">1 EHR for that subject id</p><p class="mk-results-group">EHRs</p><ul role="listbox" aria-label="EHRs">'
            f'<li role="option" aria-selected="true"><a href="ehr.html">{icon("database", "14")}<span class="mk-results-name">EHR 0199c2a4…1a01</span>'
            f'<span class="fui-small fui-muted">Subject {masked("537", static=True)} · 8 compositions</span></a></li></ul>'
            f'<a href="ehrs-found.html" class="mk-results-more">Show it in EHRs{icon("arrow-right", "14")}</a>')
    return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'


def page_topbar_subject():
    html = topbar_state_page("focus-filled", "P-20481-537", subject_results())
    return html.replace('type="search" class="fui-input" placeholder="EHR, composition, template or query" value="P-20481-537"',
                        'type="password" class="fui-input" placeholder="EHR, composition, template or query" value="P-20481-537"', 1)


def topbar_state_page(state_=None, value="", results=None):
    return page_dashboard(search_state=state_, search_value=value, results=results)


def page_topbar_results():
    return topbar_state_page("focus-filled", "vital", results_list("some"))


def page_topbar_no_result():
    return topbar_state_page("focus-filled", "0199ffff-0000-7000-8000-000000000000", results_list("none"))


def page_find():
    rows = [
        ("Open EHR 0199c2a4…1a01", "The EHR with this id.", "ehr.html"),
        ("Find the EHRs with this subject id", "Read as a subject id instead, in the EHRs list.", "ehrs-found.html"),
        ("Open a party with this id", "In Parties, for a person, an organisation, a group, an agent or a role.", "parties.html"),
    ]
    t = '<ul class="fui-match-list">' + "".join(f'<li><a class="fui-row-link" href="{h}">{e(t_)}</a><div class="fui-small fui-muted">{e(line)}</div></li>' for t_, line, h in rows) + "</ul>"
    head = page_heading("Find", subtitle="What you typed, read by its shape, and the screens that take it.")
    read = notice("info", "info", f"<strong>{short(E1)}</strong> reads as an EHR id from its shape alone.")
    content = head + f'<div class="fui-list-page">{read}{tpl.card(t, label="Where it can go", title="Where it can go", count=3)}</div>'
    return shell("Find", "", content, search_value=E1)


def page_search_states():
    states = [
        ("Default", "the icon, the placeholder and the quiet / hint", "", None),
        ("Hover", "the edge takes the ink", "", "hover"),
        ("Focused, empty", "one ring round the whole group; the hint is gone", "", "focus"),
        ("Focused and filled", "the ring, the typed value, the hint gone", "vital", "focus-filled"),
        ("Filled, not focused", "the hint stays hidden while the field holds text", "vital", None),
        ("Busy", "the icon becomes the spinner while the search runs", "vital", "busy"),
        ("Disabled", "the sunken ground, no pointer; the CDR is unreachable", "", "disabled"),
    ]
    cards = ""
    for i, (title, line, value, st) in enumerate(states):
        cards += (f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                  f'<div class="mk-search-demo">{search_control(value, st, id_=f"find-{i}")}</div></div>')
    keys = card("What it finds", definitions([
        ("EHRs", "by EHR id, or by subject id; a subject id is shown masked in every answer"),
        ("Compositions", "by uid, with or without its version"),
        ("Templates and stored queries", "by id or name, in part"),
        ("Parties", "by id"),
        ("Keys", "/ focuses it at wide widths; ↑ and ↓ move through the matches, Enter opens one, Esc closes"),
    ]))
    content = page_heading("Search control, states", subtitle="One joined control that finds what the CDR holds, or says where an id can go.") + f'<div class="fui-stack mk-search-sheet">{cards}{keys}</div>'
    return shell("Search states", "dashboard", content)


# ---- shell states ----

def page_sidebar_rail():
    return page_dashboard(rail=True, rail_tip="ehrs")


def page_sidebar_drawer():
    import pg_ehr
    return pg_ehr.page_ehrs().replace('class="fui-sidebar"', 'class="fui-sidebar mk-drawer-open"', 1).replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1).replace('<main id="main"', '<div class="mk-scrim mk-scrim-drawer" aria-hidden="true"></div><main id="main"', 1)


def page_sidebar_no_subscriptions():
    return page_dashboard(probes=False)


def page_user_menu():
    return page_dashboard(menu_open=True)


def page_user_menu_dark():
    html = page_dashboard(menu_open=True, menu_theme="dark")
    return html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)


def page_user_menu_signout():
    return page_dashboard(menu_open=True, menu_focus="signout")


def page_toast_success():
    import pg_comp
    return pg_comp.page_composition(toast_html=toasts(toast("ok", "circle-check", "Committed", "Vital signs · the CDR now holds version 3.")))


def page_toast_warning():
    t = toast("warn", "triangle-alert", "The CDR stopped answering", "The screen shows what it read at 09:42; a run or a save waits until it answers.",
              action=button("secondary", "Try again", "refresh-cw", small=True))
    return page_dashboard(toast_html=toasts(t), status="unknown")


def page_toast_error():
    import pg_ehr
    return pg_ehr.page_ehr_status(toast_html=toasts(toast("bad", "circle-alert", "The status was not saved", "Another client committed a new EHR status since you opened it (412). Reload to see version 3.",
                                                          action=link_button("secondary", "Reload", "ehr-status.html", small=True))))


def status_page(code, glyph, title, body, actions, current="dashboard", reader=False):
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon(glyph, "24")}</span><p class="fui-status-page-code">{code}</p>'
               f'<h1>{title}</h1><p>{body}</p><div class="fui-actions">{actions}</div></div>')
    return shell(title, current, content, reader=reader)


def page_forbidden():
    return status_page("403 · Not allowed", "ban", "The audit log needs the admin role",
                       "You are signed in as Demo Clinician with the user role, and the CDR shows the audit trail only to the admin role.",
                       link_button("primary", "Dashboard", "dashboard.html", "arrow-left") + link_button("secondary", "Your access", "access.html", "key-round"), current="audit", reader=True)


def page_not_found():
    return status_page("404 · Not found", "compass", "No such screen",
                       "The viewer has no page at <code>/ehr/<wbr>0199c2a4</code>. EHRs live under <code>/ehrs</code>.",
                       link_button("primary", "EHRs", "ehrs.html", "database") + link_button("secondary", "Dashboard", "dashboard.html"), current="")


# ---- about ----

MANIFEST = """{
  "solution": "FerroEHR",
  "solution_version": "4.3.10",
  "vendor": "Cadasto B.V.",
  "restapi_specs_version": "1.1.0",
  "conformance_profile": "STANDARD",
  "endpoints": [
    "/ehr",
    "/query",
    "/definition",
    "/demographic",
    "/admin"
  ]
}"""


def page_about():
    vers = card("Versions", definitions([("Viewer", f"FerroEHR Viewer {VERSION}"), ("CDR", f"FerroEHR {VERSION}"),
                                         ("openEHR REST", "ITS-REST 1.1.0"), ("Specifications", "RM 1.2.0, BASE 1.3.0, TERM 3.1.0, AM 1.4.0 and 2.4.0"), ("FerroUI", "ferrohealth-ui 0.3.0")]))
    maker = card("Manufacturer", definitions([("Made by", "Cadasto B.V."), ("Licence", "Business Source License 1.1"), ("Support", '<a href="#" rel="external">info@cadasto.com</a>'), ("Documentation", '<a href="#" rel="external">The FerroEHR book</a>')]))
    acts = copy_button("Copy", "the conformance manifest") + button("secondary", "Download", "download", small=True)
    doc = (f'<section class="fui-card fui-panel mk-options-card"><div class="fui-panel-head"><h2 class="fui-card-title"><span>The conformance manifest</span></h2><div class="fui-actions">{acts}</div></div>'
           f'<div class="fui-panel-body fui-stack"><p class="fui-small fui-muted">What the CDR answers at <code>OPTIONS /ferroehr/<wbr>rest/<wbr>openehr/<wbr>v1</code>; <code>/demographic</code> is the ITS-REST demographic API, still in development.</p>'
           f'<div class="fui-code-panel mk-options-code" role="region" aria-label="The conformance manifest" tabindex="0"><pre>{json_html(MANIFEST)}</pre></div></div></section>')
    content = (page_heading("About", subtitle="The viewer, the CDR it reads, and the manufacturer.")
               + f'<div class="fui-stack"><div class="fui-grid fui-grid-start">{vers}{maker}</div>{doc}</div>')
    return shell("About", "about", content)


# ---- keyboard and focus ----

def page_shortcuts():
    legend = shortcut_legend([
        ("Everywhere", [("/", "Find anything"), ("?", "This legend"), ("Esc", "Close a dialog, a menu or a panel"),
                        ("g d", "Go to the dashboard"), ("g e", "Go to EHRs"), ("g t", "Go to Templates"), ("g q", "Go to Query")]),
        ("Lists", [("j | down", "Next row"), ("k | up", "Previous row"), ("Enter", "Open the row"), ("left | right", "Previous and next page")]),
        ("Trees", [("up | down", "Previous and next node"), ("right", "Open a level"), ("left", "Close a level, or go to the parent"), ("Enter", "Read the node")]),
        ("Query and editors", [("Ctrl Enter", "Run"), ("Ctrl s", "Save")]),
    ])
    import pg_query
    return pg_query.page_query_aql(overlay=legend)


def page_focus_rings():
    from ehr import tree, tree_item

    def row(title, html):
        return f'<div class="mk-focus-row"><span class="mk-focus-name">{title}</span><div class="mk-focus-demo">{html}</div></div>'
    from ehr import rm_type
    tr = tree([tree_item("Blood pressure", extra=rm_type("OBSERVATION"), focus=True)], "Example tree", "fui-path-tree")
    rows = (
        row("Primary button", button("primary", "Run", "play", cls="mk-focus", keys="Ctrl Enter"))
        + row("Secondary button", button("secondary", "Open", "history", cls="mk-focus"))
        + row("Icon button", icon_button("ellipsis", "More actions", cls="mk-focus"))
        + row("Link", '<a href="#" class="mk-focus-ring mk-inline-link">Vital signs encounter.v1</a>')
        + row("Text field", input_("f1", "P-20481", cls="fui-input mk-focus"))
        + row("Select", select("f2", [("json", "Canonical JSON")], "json", cls="fui-select mk-focus"))
        + row("Check box", '<div class="fui-check"><input type="checkbox" id="f4" class="fui-checkbox mk-focus" checked /><label for="f4" class="fui-label">Queryable</label></div>')
        + row("Segmented control", '<div role="group" class="fui-segmented"><button class="fui-segment" aria-pressed="true">Table</button><button class="fui-segment mk-focus">Chart</button></div>')
        + row("Tab", tabbed(tabs("Example", [("c", "Compositions", "#"), ("d", "Directory", "#"), ("s", "Status", "#")], "c", focus="d"), '<span class="fui-small fui-muted">Panel</span>'))
        + row("Search control", search_control("", "focus", id_="find-f"))
        + row("Masked identifier", masked("537", "demo.hospital.mrn").replace("fui-identifier-reveal", "fui-identifier-reveal mk-focus"))
        + row("Tree", tr)
        + row("Menu item", menu([menu_item("Copy EHR id", "copy", focus=True), menu_item("Edit status", "pencil")], anchored=False))
    )
    content = page_heading("Focus, on every control", subtitle="One focus ring on every control, at 3:1 contrast.") + f'<div class="fui-card mk-focus-sheet">{rows}</div>'
    return shell("Focus rings", "", content)


def _frame(title, line, html, cls=""):
    return f'<div class="fui-card mk-state-frame {cls}"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div class="mk-menu-stage">{html}</div></div>'


def ehr_menu_items(hover=None, focus=None, desc=False, disabled=False, admin=True):
    items = [menu_item("Commit several changes", "git-commit", href="contribution-new.html", hover=hover == "c", focus=focus == "c", desc="Several versions as one contribution" if desc else None),
             menu_item("Copy EHR id", "copy", desc="The EHR id, to the clipboard" if desc else None, disabled="The clipboard is not available in this browser" if disabled else None)]
    if admin:
        items += [MENU_SEP, menu_item("Delete EHR…", "trash-2", danger=True, href="ehr-delete.html", desc="Removes the EHR and all its data for good" if desc else None)]
    return items


def trig(open_=True):
    return (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="More actions for the composition" title="More actions">{icon("ellipsis", "16")}</button>')


def page_menu_states():
    def demo(items, up=False, open_=True):
        m = menu(items, label="Actions for the EHR", up=up) if open_ else ""
        return f'<div class="mk-menu-demo{" mk-menu-demo-up" if up else ""}"><span class="fui-anchor">{trig(open_)}{m}</span></div>'
    frames = (
        _frame("Rest", "4 px under its trigger, at its end edge; Delete last, in the danger tone", demo(ehr_menu_items()))
        + _frame("Hover", "the neutral tint, the icon to ink", demo(ehr_menu_items(hover="c")))
        + _frame("Keyboard focus", "the same tint, plus the focus ring; ↑ ↓ Home End and a letter move, Enter activates, Esc returns to the trigger", demo(ehr_menu_items(focus="c")))
        + _frame("Disabled, with its reason", "muted and skipped by the keyboard; the reason is its tooltip", demo(ehr_menu_items(disabled=True)))
        + _frame("With descriptions", "one muted line under a label when the action needs it", demo(ehr_menu_items(desc=True)))
        + _frame("Without the admin role", "Delete EHR is left out, never disabled", demo(ehr_menu_items(admin=False)))
        + _frame("Flipped up", "no room below: it opens above its trigger and stays in the viewport", demo(ehr_menu_items(), up=True))
    )
    sheet = '<div class="mk-sheet-demo">' + menu(ehr_menu_items(), label="Actions for the EHR", sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>"
    frames += _frame("Phone: the bottom sheet", "below 40 rem a row menu or More is a sheet: its title, 48 px rows, Cancel", sheet)
    keys = card("Rules", definitions([
        ("Roles", 'role="menu" and role="menuitem"; aria-haspopup and aria-expanded on the trigger'),
        ("Destructive", "Last, after a separator, in the danger tone, with an ellipsis; the confirmation says whether the delete is physical or logical"),
        ("Admin only", "Left out when the CDR's admin API is off or your role lacks it, never shown disabled"),
    ]))
    content = page_heading("Menu, states", subtitle="F13 in FerroEHR: the one anchored menu behind every ⋯, every More and the user menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}{keys}</div>'
    return shell("Menu states", "ehrs", content)


def page_action_group_states():
    newc = link_button("primary", "New composition", "composition-new.html", "plus")
    frames = (
        _frame("An EHR", "New composition, and More with the rest", f'<div class="mk-menu-demo">{action_group(newc, ehr_menu_items(), label="More actions for the EHR")}</div>')
        + _frame("More open", "the same anchored menu", f'<div class="mk-menu-demo">{action_group(newc, ehr_menu_items(), open_=True, label="More actions for the EHR")}</div>')
        + _frame("A composition", "Edit as a new version; Delete composition… last",
                 f'<div class="mk-menu-demo">{action_group(link_button("primary", "Edit as a new version", "composition-edit.html", "pencil"), [menu_item("Download", "download"), menu_item("Copy uid", "copy"), MENU_SEP, menu_item("Delete composition…", "trash-2", danger=True, href="composition-delete.html")], label="More actions for the composition")}</div>')
        + _frame("A stored query", "Open in Query; Delete this version… in More",
                 f'<div class="mk-menu-demo">{action_group(link_button("primary", "Open in Query", "query-aql.html", "square-terminal"), [menu_item("Copy name", "copy"), MENU_SEP, menu_item("Delete this version…", "trash-2", danger=True, href="stored-query-delete.html")], label="More actions for the stored query")}</div>')
    )
    sheet = '<div class="mk-sheet-demo">' + menu(ehr_menu_items(), label="More actions for the EHR", sheet=True, anchored=False).replace('fui-menu fui-menu-compact fui-menu-sheet', 'fui-menu fui-menu-compact fui-menu-sheet mk-sheet-static') + "</div>"
    frames += _frame("Phone", "the primary stays; More opens the bottom sheet", sheet)
    content = page_heading("Action group, states", subtitle="F14 in FerroEHR: the title row's one primary action and its More menu.") + f'<div class="fui-stack mk-menu-sheet">{frames}</div>'
    return shell("Action group states", "ehrs", content)
