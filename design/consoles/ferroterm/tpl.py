# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The three page templates every console page is built on (NEW, FerroFED,
for every console): the list page, the detail page and the settings and
reference page. This module holds the list page's parts; the detail page is
page_heading + action_group + binder tabs, and the settings page is the
sub-navigation and setting-row column of R26 and R27."""
from common import MENU_SEP, e, health_dot, icon, input_, menu, tabbed

NOW = (9, 42, 11)


def rel(hms, date="2026-10-21"):
    """A time relative to now (09:42:11 today), the full timestamp as its tooltip."""
    if not hms:
        return '<span class="mk-dash">never</span>'
    h, m, s = (int(x) for x in hms.split(":"))
    d = (NOW[0] * 3600 + NOW[1] * 60 + NOW[2]) - (h * 3600 + m * 60 + s)
    if d < 60:
        word = f"{d} s ago"
    elif d < 3600:
        word = f"{d // 60} min ago"
    else:
        word = f"{d // 3600} h ago"
    return f'<time class="fui-time" datetime="{date}T{hms}" title="{date} {hms} CEST">{word}</time>'


def summary(chips):
    """chips: (label, tone or None, pressed). The same component as Services."""
    out = "".join(f'<button type="button" class="fui-chip-check" aria-pressed="{"true" if on else "false"}">{health_dot(t) if t else ""}{e(w)}</button>' for w, t, on in chips)
    return f'<div class="fui-chip-checks fui-summary-chips" role="group" aria-label="Show">{out}</div>'


def filter_button(label, word=None, open_=False, items=None, count=None):
    shown = f"{label} · {word}" if word else label
    m = ""
    if open_ and items:
        rows = "".join(
            f'<label class="fui-menu-item fui-menu-check" role="menuitemcheckbox" aria-checked="{"true" if on else "false"}"><input type="checkbox"{" checked" if on else ""} tabindex="-1" />'
            f'{health_dot(t) if t else ""}<span class="fui-menu-item-text"><span class="fui-menu-item-label">{e(w)}</span></span><span class="fui-menu-count">{n}</span></label>'
            for w, t, n, on in items)
        m = f'<div role="menu" class="fui-menu fui-menu-compact fui-menu-anchored fui-menu-start" aria-label="{e(label)}"><div class="fui-menu-items">{rows}</div></div>'
    return (f'<span class="fui-anchor"><button type="button" class="fui-button fui-button-secondary{" is-filtered" if word else ""}" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}">'
            f'{icon("list-filter", "16")}{e(shown)}{icon("chevron-down", "16")}</button>{m}</span>')


def toolbar(placeholder, value="", filters=(), end=""):
    find = (f'<div class="fui-list-find"><span class="fui-search-glyph">{icon("search", "16")}</span>'
            f'{input_("l-find", value, placeholder, "search", extra=" aria-label=" + chr(34) + e(placeholder) + chr(34))}</div>')
    return f'<div class="fui-list-toolbar"><div class="fui-list-toolbar-start">{find}{"".join(filters)}</div><div class="fui-list-toolbar-end">{end}</div></div>'


def active(chips):
    if not chips:
        return ""
    c = "".join(f'<span class="fui-filter-chip">{e(w)}<button type="button" aria-label="Remove the filter {e(w)}">{icon("x", "12")}</button></span>' for w in chips)
    return f'<div class="fui-filter-chips"><span class="fui-small fui-muted">Showing</span>{c}<button type="button" class="fui-button fui-button-ghost fui-button-small">{icon("x", "16")}Clear filters</button></div>'


def card(table_html, foot="", tabs_html=None, label="List", title=None, count=None, actions="", id_=None):
    """The table edge to edge in one card. Without tabs, a card header row
    (title, count, actions) sits above it; with tabs, the binder tabs carry
    the title and the count and the card is their panel."""
    inner = table_html + (f'<div class="fui-list-foot">{foot}</div>' if foot else "")
    if tabs_html:
        return tabbed(tabs_html, f'<div class="fui-worklist fui-list-panel" aria-label="{e(label)}">{inner}</div>')
    t = title or label
    c = f' <span class="fui-list-count">· {count}</span>' if count else ""
    head = f'<div class="fui-panel-head fui-list-head"><h2 class="fui-card-title"><span>{t}{c}</span></h2><div class="fui-actions">{actions}</div></div>'
    i = f' id="{id_}"' if id_ else ""
    return f'<section{i} class="fui-card fui-panel fui-worklist fui-list-card" aria-label="{e(label)}">{head}{inner}</section>'


def list_body(summary_html, toolbar_html, active_html, card_html, extra=""):
    return f'<div class="fui-list-page">{summary_html}{toolbar_html}{active_html}{card_html}{extra}</div>'


# ---- T3. The settings and reference page: title, binder tabs, one tab panel ----

def settings_tabs(label, items, current, panel_html, drafts=(), many=False, focus=None):
    """items: (key, words, href). Each tab is its own route; the tab panel shows
    only that section. drafts: tab keys that hold a pending draft. many: more
    tabs than a phone row shows, so at 390 px a select replaces the row."""
    links = ""
    for item in items:
        key, words, href = item[:3]
        extra = item[3] if len(item) > 3 else ""
        cur = ' aria-current="page"' if key == current else ""
        badge = '<span class="fui-tab-draft" title="Holds changes in a draft">draft</span>' if key in drafts else ""
        cls = "fui-tab" + (" mk-focus" if key == focus else "") + (f" {extra}" if extra else "")
        links += f'<a href="{href}" class="{cls}"{cur} data-tab="{key}">{e(words)}{badge}</a>'
    nav = f'<nav class="fui-tabs" aria-label="{e(label)}" data-keys="arrows">{links}</nav>'
    sel = ""
    if many:
        opts = "".join(f'<option value="{it[2]}"{" selected" if it[0] == current else ""}>{e(it[1])}{" (draft)" if it[0] in drafts else ""}</option>' for it in items)
        sel = f'<div class="fui-tabs-select"><label class="fui-sr-only" for="tabs-sel">{e(label)}</label><select id="tabs-sel" class="fui-select">{opts}</select></div>'
    cls = "fui-tabbed fui-settings-tabbed" + (" fui-tabbed-many" if many else "")
    return f'<div class="{cls}">{sel}{nav}<div class="fui-tab-panel">{panel_html}</div></div>'


def settings_panel(rows_html, intro="", actions="", top=""):
    """One tab's section: an optional one-line intro with the tab's actions,
    then the setting rows across the panel's width."""
    head = ""
    if intro or actions:
        head = f'<div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{intro}</p><div class="fui-actions">{actions}</div></div>'
    return f'<div class="fui-settings-panel">{head}{top}<div class="fui-setting-rows">{rows_html}</div></div>'
