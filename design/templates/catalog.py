# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The library's reference templates (T1 list, T2 detail, T3 settings and
reference, and About), drawn with the family's helpers in the FerroFED
theme with example content, read by design/tools/build.py. A link to a
page that is not a template points nowhere."""
import re

import pages
import tpl
from common import (
    MENU_SEP, action_group, button, card, dock, dock_field, e, facts_row, health_dot, icon, link_button, menu, menu_item,
    page_heading, pager, pill, shell, table, tabbed, tabs, td, th,
)

ITEMS = [("Locatie Noord", "loc-n", 3, "ok", "09:41:58"), ("Locatie Oost", "loc-o", 1, "warn", "09:41:40"), ("Locatie Zuid", "loc-z", 2, "ok", "09:41:31"),
         ("Locatie West", "loc-w", 1, "ok", "09:40:12"), ("Locatie Midden", "loc-m", 1, "muted", None)]
WORD = {"ok": "reachable", "warn": "degraded", "muted": "excluded", "bad": "unreachable"}


def state(t):
    return f'<span class="fui-state fui-tone-{"neutral" if t == "muted" else t}{" is-muted" if t == "muted" else ""}">{health_dot(t)}{WORD[t]}</span>'


def row_menu(name):
    return (f'<span class="fui-anchor"><button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="false" aria-label="Actions for {e(name)}" title="More actions">'
            f'{icon("ellipsis", "16")}</button></span>')


def rows(items):
    out = []
    for name, id_, n, t, probe in items:
        cls = ' class="is-excluded"' if t == "muted" else ""
        out.append(f'<tr{cls}><th scope="row" data-role="title"><a class="fui-row-link" href="detail.html">{e(name)}</a></th>' + td("Id", f"<code>{id_}</code>", role="wide")
                   + td("Units", str(n), "fui-num", role="meta-q") + td("State", state(t), role="meta-2") + td("Last check", tpl.rel(probe), role="wide")
                   + td("", row_menu(name), "fui-row-menu", role="menu") + "</tr>")
    return rows_table(out)


def rows_table(body):
    return table("Locations", th("Location", "n", "ascending") + th("Id") + th("Units", num=True) + th("State") + th("Last check") + th('<span class="fui-sr-only">Actions</span>'), body, scroll_label="Locations")


def head(**kw):
    return page_heading("Locations", actions=link_button("primary", "New location", "#", "plus"), subtitle="Example content: the template's parts, not a product's.", **kw)


def page_list():
    c = tpl.card(rows(ITEMS), label="Locations", count="5")
    return shell("Locations", "orgs", head() + tpl.list_body("", "", "", c))


def page_list_many():
    many = [(f"Locatie {i:02d}", f"loc-{i:02d}", 1 + i % 3, "warn" if i == 4 else "ok", f"09:4{i % 2}:{(i * 7) % 60:02d}") for i in range(1, 26)]
    summ = tpl.summary([("30 locations", None, True), ("1 degraded", "warn", False), ("1 excluded", "muted", False)])
    tools = tpl.toolbar("Location name or id", "", [tpl.filter_button("State")])
    c = tpl.card(rows(many), pager("Locations", 30, 25, 1, sizes=(25, 50, 100)), label="Locations", count="30")
    return shell("Locations", "orgs", head() + tpl.list_body(summ, tools, "", c))


def page_list_tabs():
    t = tabs("Locations", [("all", "Locations", "list-tabs.html", 5), ("units", "All units", "#", 8)], "all")
    c = tpl.card(rows(ITEMS), tabs_html=t, label="Locations")
    return shell("Locations", "orgs", head() + tpl.list_body("", "", "", c))


def page_detail():
    ag = action_group(link_button("primary", "Add unit", "#", "plus"), [menu_item("Edit", "pencil", keys="e"), MENU_SEP, menu_item("Remove…", "trash-2", danger=True)], label="More actions for Locatie Noord")
    h = page_heading("Locatie Noord", actions=ag, badge=state("ok"), subtitle="Location <code>loc-n</code> · 3 units", crumbs=[("Locations", "list.html")])
    t = tabs("Locatie Noord", [("overview", "Overview", "detail.html"), ("units", "Units", "#", 3), ("audit", "Audit", "#"), ("settings", "Settings", "#")], "overview")
    facts = facts_row([("Id", "<code>loc-n</code>"), ("Standing", pill("ok", "active")), ("Units", "3"), ("Added", "2026-10-16")])
    panel = f'<div class="fui-grid fui-grid-start">{card("Location", facts)}{card("Last 24 hours", "<p>1,284 requests · 3 refused</p>")}</div>'
    return shell("Locatie Noord", "orgs", h + f'<div class="fui-stack">{tabbed(t, panel)}</div>')


SET_TABS = [("general", "General", "settings.html"), ("limits", "Limits", "#"), ("access", "Access log", "#"), ("history", "History", "#")]


def srow(label, desc, value, control=None, edit=False, restart=False, changed=None):
    nxt = f'<span class="fui-effect fui-effect-restart">{icon("refresh-cw", "12")}Next start</span>' if restart else ""
    ctl = control if edit and control else f'<span class="fui-setting-value">{value}</span>'
    was = f'<span class="fui-config-was">was {e(changed)}</span>' if changed else ""
    return (f'<div class="fui-setting-row{" is-changed" if changed else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}</span><p class="fui-setting-desc">{desc}</p></div>'
            f'<div class="fui-setting-control">{ctl}{nxt}{was}</div></div>')


def settings_rows(edit=False):
    dur = '<div class="fui-input-group mk-dur"><input class="fui-input fui-config-changed" value="3000" inputmode="numeric" /><span class="fui-input-suffix">ms</span></div>'
    return (srow("Timeout", "How long one call may take, 200 to 10,000 ms.", "2,500 ms", dur, edit, changed="2,500 ms" if edit else None)
            + srow("Retries", "How often a failed call is tried again.", "2")
            + srow("Mode", "Which units a request asks.", "nearest", restart=True)
            + srow("Best effort", "A caller may ask for a partial answer.", "offered"))


def settings_page(edit=False):
    ag = action_group(link_button("ghost", "How settings work", "#", "info"), [menu_item("Export as TOML", "download"), menu_item("Import TOML…", "upload")], label="More settings actions")
    h = page_heading("Settings", actions=ag, subtitle="Example content: deployment-wide settings, one section per tab.")
    act = '<span class="fui-pill fui-tone-accent">editing</span>' if edit else link_button("secondary", "Edit", "settings-edit.html", "pencil", small=True)
    panel = tpl.settings_panel(settings_rows(edit), "How calls behave. Changes apply at once unless a row says Next start.", act)
    body = tpl.settings_tabs("Settings sections", SET_TABS, "general", panel, drafts=("general",) if edit else ())
    d = dock(dock_field("reason", "Reason", "Slow links at two locations", "Why this changes; kept with the revision", keys="r"),
             link_button("secondary", "Discard", "settings.html", "x") + button("primary", "Review changes", "git-compare")) if edit else ""
    return shell("Settings", "configuration", h + f'<div class="fui-stack">{body}</div>', dock_html=d)


DOC = """{
  "name": "example",
  "version": "1.0.0",
  "limits": { "timeout_ms": 2500, "retries": 2 },
  "best_effort": true
}"""


def page_about():
    from pg_about import json_html
    facts = lambda rs: '<dl class="fui-facts">' + "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rs) + "</dl>"
    v = f'<section class="fui-card"><h2 class="fui-card-title"><span>Versions</span></h2>{facts([("Console", "Example Console 0.1.0"), ("Server", "Example 0.1.0"), ("FerroUI", "ferrohealth-ui 0.3.0")])}</section>'
    m = f'<section class="fui-card"><h2 class="fui-card-title"><span>Manufacturer</span></h2>{facts([("Made by", "Cadasto B.V."), ("Licence", "Business Source License 1.1"), ("Support", "<a href=\"#\">info@cadasto.com</a>")])}</section>'
    doc = (f'<section class="fui-card fui-panel mk-options-card"><div class="fui-panel-head"><h2 class="fui-card-title"><span>The self-description</span></h2>'
           f'<div class="fui-actions">{button("secondary", "Copy", "copy", small=True)}{button("secondary", "Download", "download", small=True)}</div></div>'
           f'<div class="fui-panel-body fui-stack"><p class="fui-small fui-muted">What the server declares, as every client reads it.</p>'
           f'<div class="fui-code-panel fui-self-description" role="region" aria-label="The self-description" tabindex="0"><pre>{json_html(DOC)}</pre></div></div></section>')
    return shell("About", "about", page_heading("About", subtitle="The console, the server it reads, and the manufacturer.") + f'<div class="fui-stack"><div class="fui-grid fui-grid-start">{v}{m}</div>{doc}</div>')


_PAGES = {"list": page_list, "list-many": page_list_many, "list-tabs": page_list_tabs, "detail": page_detail,
         "settings": settings_page, "settings-edit": lambda: settings_page(edit=True), "about": page_about}


def _local(fn):
    """Points every link to a page outside the templates at #."""
    slugs = {x[3] for x in pages.flat()}
    return lambda: re.sub(r'href="([a-z0-9-]+)\.html"', lambda m: m.group(0) if m.group(1) in slugs else 'href="#"', fn())


PAGES = {slug: _local(fn) for slug, fn in _PAGES.items()}
