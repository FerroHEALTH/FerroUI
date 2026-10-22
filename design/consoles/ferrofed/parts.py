# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Parts every FerroFED mockup page shares, copied from the FerroPIX
generator unchanged where the family pattern is the same (charts, the date
field and picker, the change marks, the revision strip, the folding page
actions, the mode badge, the settings rows and cards, the listbox)."""
import calendar as _cal
import datetime as _dt
import math

from common import (badge, button, e, icon, icon_button, input_, kbd, menu, menu_item, td, th, table, field)

def bsn_valid(digits):
    weights = [9, 8, 7, 6, 5, 4, 3, 2, -1]
    return sum(w * int(d) for w, d in zip(weights, digits)) % 11 == 0


def nice_range(low, high):
    raw = (high - low) / 3.0
    power = 10 ** math.floor(math.log10(raw)) if raw > 0 else 1
    frac = raw / power
    nice = 1 if frac < 1.5 else 2 if frac < 3 else 5 if frac < 7 else 10
    step = nice * power
    lo = math.floor(low / step) * step
    hi = math.ceil(high / step) * step
    if hi - lo < step:
        hi = lo + step
    return lo, hi, step


def line_chart(id_, label, summary, series, x_ticks, height=220, legend=True):
    """series: [(name, [y...], palette_index)]; x evenly spaced."""
    ys = [y for _, pts, _ in series for y in pts]
    lo, hi, step = nice_range(0, max(ys))
    n = max(len(pts) for _, pts, _ in series)
    top, bottom = 10.0, 26.0
    W = 1000.0

    def ypx(v):
        return (height - bottom) - (v - lo) / (hi - lo) * (height - bottom - top)

    ticks = []
    v = lo
    while v <= hi + 1e-9:
        ticks.append(v)
        v += step
    ylabels = "".join(f"<span>{int(t) if t == int(t) else t}</span>" for t in reversed(ticks))
    grid = "".join(
        f'<line class="{"fui-chart-axis" if i == 0 else "fui-chart-grid"}" x1="0" x2="100%" y1="{ypx(t):.1f}" y2="{ypx(t):.1f}"/>'
        for i, t in enumerate(ticks)
    )
    paths = ""
    for name, pts, pal in series:
        d = " ".join(f'{"M" if i == 0 else "L"}{i / (n - 1) * W:.1f} {ypx(y):.1f}' for i, y in enumerate(pts))
        paths += f'<path class="fui-chart-line fui-series-{pal}" d="{d}"/>'
    labels = ""
    for i, text in x_ticks:
        share = i / (n - 1)
        anchor = "start" if share <= 0.01 else "end" if share >= 0.99 else "middle"
        labels += f'<text class="fui-chart-label" x="{share * 100:.2f}%" y="{height - 6:.1f}" text-anchor="{anchor}">{e(text)}</text>'
    svg = (
        f'<div class="fui-chart-frame"><div class="fui-chart-y" aria-hidden="true">{ylabels}</div>'
        f'<svg class="fui-chart-svg" width="100%" height="{height}" role="img" aria-labelledby="{id_}-title {id_}-desc">'
        f'<title id="{id_}-title">{e(label)}</title><desc id="{id_}-desc">{e(summary)}</desc>{grid}'
        f'<svg class="fui-chart-plot-lines" width="100%" height="{height}" viewBox="0 0 {W:.0f} {height}" preserveAspectRatio="none">{paths}</svg>'
        f"{labels}</svg></div>"
    )
    chips = ""
    if legend and len(series) > 1:
        chips = '<figcaption class="fui-legend" aria-label="Series">' + "".join(
            f'<button type="button" class="fui-legend-chip" aria-pressed="true"><span class="fui-swatch fui-series-{pal}" aria-hidden="true"></span>{e(name)}</button>'
            for name, _, pal in series
        ) + "</figcaption>"
    return f'<figure class="fui-chart"><div class="fui-chart-plot">{svg}</div>{chips}</figure>'


def date_input(id_, value):
    """The console's date field: year first, as every date it shows, with
    the calendar behind a button (R23)."""
    return ('<div class="fui-input-group fui-date">' + input_(id_, value, "YYYY-MM-DD", extra='inputmode="numeric" pattern="\\d{4}-\\d{2}-\\d{2}" spellcheck="false"')
            + icon_button("calendar", "Choose a date", cls="fui-input-toggle") + "</div>")


def skeleton_lines(widths):
    return "".join(f'<span class="fui-skeleton mk-w{w}" aria-hidden="true"></span>' for w in widths)


def filter_chips(items):
    chips = "".join(
        f'<span class="fui-filter-chip">{w}<button type="button" aria-label="Remove the filter {e(w)}">{icon("x", "12")}</button></span>'
        for w in items
    )
    return f'<div class="fui-filter-chips"><span class="fui-small fui-muted">Filtered by</span>{chips}{button("ghost", "Clear filters", "x", small=True)}</div>'


def change_del(v):
    return f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-sr-only">was </span><span class="fui-change-text">{v}</span></del>'


def change_ins(v):
    return f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span><span class="fui-sr-only">now </span>{v}</ins>'


def change_plain(before, after):
    return (f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-sr-only">was </span><span class="fui-change-text">{e(before)}</span></del> '
            f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span><span class="fui-sr-only">now </span>{e(after)}</ins>')


def effect(kind):
    if kind == "live":
        return f'<span class="fui-effect fui-effect-live">{icon("circle-check", "12")}Live at once</span>'
    return f'<span class="fui-effect fui-effect-restart">{icon("refresh-cw", "12")}At the next start</span>'


def revision_strip(state, title, line, actions=""):
    tone = {"running": "", "draft": " fui-revision-draft", "ok": " fui-tone-ok", "warn": " fui-tone-warn", "bad": " fui-tone-bad", "refused": " fui-tone-warn"}[state]
    glyph = {"running": "file-text", "draft": "pencil", "ok": "circle-check", "warn": "refresh-cw", "bad": "triangle-alert", "refused": "triangle-alert"}[state]
    role = ' role="status"' if state in ("ok", "warn", "bad", "refused") else ""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    return f'<div class="fui-revision{tone}"{role}>{icon(glyph, "18")}<div class="fui-revision-text"><strong>{title}</strong><span>{line}</span></div>{a}</div>'


def mode_badge(mode):
    """Whether the deployment manages its configuration as revisions, quiet
    in the page header; the key on hover or focus."""
    word = "Revisions managed" if mode == "managed" else "Revisions off"
    glyph = "history" if mode == "managed" else "file-text"
    return (f'<span class="fui-mode-badge" tabindex="0" aria-describedby="mode-tip">{icon(glyph, "14")}{word}'
            f'<span id="mode-tip" role="tooltip" class="fui-tooltip fui-tooltip-below fui-mode-tip"><code>config.ui = "{mode}"</code></span></span>')


def fold_actions(items, menu_items, open_=False):
    """Page actions in the title row; below 40 rem one Actions menu."""
    m = menu(menu_items, label="Page actions") if open_ else ""
    trigger = (f'<span class="fui-anchor fui-actions-fold-menu"><button type="button" class="fui-button fui-button-secondary" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}">'
               f'Actions{icon("chevron-down", "16")}</button>{m}</span>')
    return f'<div class="fui-actions-fold">{"".join(items)}{trigger}</div>'


def setting_row(label, desc, control, id_=None, saved=False):
    s = f'<span class="fui-setting-saved" role="status">{icon("check", "14")}Saved</span>' if saved else ""
    lab = f'<label for="{id_}" class="fui-setting-label">{label}</label>' if id_ else f'<span class="fui-setting-label">{label}</span>'
    return (f'<div class="fui-setting-row{" is-saved" if saved else ""}"><div class="fui-setting-text">{lab}<p class="fui-setting-desc">{desc}</p></div>'
            f'<div class="fui-setting-control">{s}{control}</div></div>')


def setting_card(id_, title, rows, desc=None):
    d = f'<p class="fui-setting-section-desc">{desc}</p>' if desc else ""
    return f'<section class="fui-card fui-setting-card" id="{id_}" aria-labelledby="{id_}-title"><h2 class="fui-card-title" id="{id_}-title">{title}</h2>{d}<div class="fui-setting-rows">{rows}</div></section>'


def theme_preview(kind):
    if kind == "system":
        return '<span class="fui-preview fui-preview-split" aria-hidden="true"><span class="fui-preview fui-preview-light">' + PREVIEW_INNER + '</span><span class="fui-preview fui-preview-dark">' + PREVIEW_INNER + "</span></span>"
    return f'<span class="fui-preview fui-preview-{kind}" aria-hidden="true">{PREVIEW_INNER}</span>'


PREVIEW_INNER = ('<span class="pv-top"></span><span class="pv-body"><span class="pv-side"><i></i><i></i><i></i></span>'
                 '<span class="pv-main"><i class="pv-title"></i><i></i><i></i><i class="pv-short"></i><b></b></span></span>')


def density_preview(kind):
    rows = "".join("<i></i>" for _ in range(5 if kind == "compact" else 3))
    return f'<span class="fui-preview fui-preview-table fui-preview-{kind}" aria-hidden="true"><span class="pv-head"></span>{rows}</span>'


def choice_cards(name, legend, options, checked, previews, focus=None):
    cards = ""
    for v, w, sub in options:
        on = v == checked
        cards += (f'<label class="fui-choice-card{" mk-focus-ring" if v == focus else ""}"><input type="radio" class="fui-sr-only" name="{name}" value="{v}"{" checked" if on else ""} />'
                  f'{previews(v)}<span class="fui-choice-card-text"><span class="fui-choice-card-title">{w}</span><span class="fui-choice-card-sub">{sub}</span></span>'
                  f'<span class="fui-choice-card-check" aria-hidden="true">{icon("check", "12")}</span></label>')
    return f'<fieldset class="fui-choice-cards"><legend class="fui-sr-only">{legend}</legend>{cards}</fieldset>'


def listbox(options, current, id_="lang-list", label="Language"):
    items = ""
    for v, w, note in options:
        sel = v == current
        n = f'<span class="fui-listbox-note">{note}</span>' if note else ""
        items += (f'<li role="option" class="fui-listbox-option{" mk-hover" if v == "nl" else ""}" aria-selected="{"true" if sel else "false"}"><span class="fui-listbox-text"><span>{w}</span>{n}</span>'
                  f'{icon("check", "14") if sel else ""}</li>')
        if v == "browser":
            items += '<li role="separator" class="fui-listbox-sep"></li>'
    return f'<ul id="{id_}" role="listbox" aria-label="{e(label)}" class="fui-listbox">{items}</ul>'


def select_button(id_, value, open_=False, menu_html="", wide=False):
    return (f'<span class="fui-select-anchor{" fui-select-anchor-wide" if wide else ""}"><button type="button" id="{id_}" class="fui-select fui-select-button" aria-haspopup="listbox" aria-expanded="{"true" if open_ else "false"}">'
            f'<span>{value}</span>{icon("chevron-down", "16")}</button>{menu_html if open_ else ""}</span>')


MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]


WEEKDAYS = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]


WEEKDAY_NAMES = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]


PICK_TODAY = _dt.date(2026, 10, 21)


def month_grid(year, month, selected=None, rng=None, tentative=None, min_d=None, max_d=None, hover=None, focus=None, title_btn=True, nav="both"):
    """One month, Monday first (the nl and en-GB week), as an ARIA grid."""
    weeks = _cal.Calendar(firstweekday=0).monthdatescalendar(year, month)
    head = "".join(f'<th scope="col" abbr="{WEEKDAY_NAMES[i]}">{d}</th>' for i, d in enumerate(WEEKDAYS))
    rows = ""
    lo, hi = (rng or (None, None))
    if tentative and lo and not hi:
        hi = tentative
    for week in weeks:
        cells = ""
        for d in week:
            cls = ["fui-day"]
            attrs = ""
            out = d.month != month
            if out:
                cls.append("fui-day-out")
            disabled = (min_d and d < min_d) or (max_d and d > max_d)
            if d == PICK_TODAY:
                attrs += ' aria-current="date"'
            sel = d == selected or (lo and d == lo) or (rng and rng[1] and d == rng[1])
            if sel:
                attrs += ' aria-selected="true"'
            if lo and hi and lo < d < hi:
                cls.append("fui-day-between" + (" fui-day-tentative" if tentative and not (rng and rng[1]) else ""))
            if lo and hi and d == lo:
                cls.append("fui-day-start")
            if lo and hi and d == hi:
                cls.append("fui-day-end")
                if tentative and not (rng and rng[1]):
                    cls.append("fui-day-tentative-end")
            if d == hover:
                cls.append("mk-hover")
            if d == focus:
                cls.append("mk-focus")
            dis = " disabled" if disabled else ""
            label = f"{WEEKDAY_NAMES[d.weekday()]} {d.day} {MONTHS[d.month - 1]} {d.year}"
            tab = ' tabindex="0"' if d == (focus or selected or lo) else ' tabindex="-1"'
            cells += f'<td role="gridcell"{attrs}><button type="button" class="{" ".join(cls)}" aria-label="{label}"{tab}{dis}>{d.day}</button></td>'
        rows += f"<tr>{cells}</tr>"
    title = f"{MONTHS[month - 1]} {year}"
    t = (f'<button type="button" class="fui-datepicker-title" aria-label="Choose the month and year, now {title}">{title}{icon("chevron-down", "14")}</button>'
         if title_btn else f'<span class="fui-datepicker-title fui-datepicker-title-static">{title}</span>')
    prev = icon_button("chevron-left", "Previous month") if nav in ("both", "prev") else '<span class="fui-datepicker-navgap"></span>'
    nxt = icon_button("chevron-right", "Next month") if nav in ("both", "next") else '<span class="fui-datepicker-navgap"></span>'
    return (f'<div class="fui-datepicker-month"><div class="fui-datepicker-head">{prev}{t}{nxt}</div>'
            f'<table role="grid" class="fui-datepicker-grid" aria-label="{title}"><thead><tr>{head}</tr></thead><tbody>{rows}</tbody></table></div>')


def month_chooser(year, month):
    cells = "".join(
        f'<button type="button" class="fui-datepicker-cell"{" aria-selected=\"true\"" if i + 1 == month else ""}{" aria-current=\"date\"" if (i + 1 == PICK_TODAY.month and year == PICK_TODAY.year) else ""}>{m[:3]}</button>'
        for i, m in enumerate(MONTHS))
    return (f'<div class="fui-datepicker-month"><div class="fui-datepicker-head">{icon_button("chevron-left", "Previous year")}'
            f'<button type="button" class="fui-datepicker-title" aria-label="Back to the days">{year}{icon("chevron-up", "14")}</button>{icon_button("chevron-right", "Next year")}</div>'
            f'<div class="fui-datepicker-months" role="grid" aria-label="Months of {year}">{cells}</div></div>')


def picker(label, body, foot=True, quick=None, presets=None, time=None, sheet=False, extra_cls="", status=""):
    q = ""
    if quick:
        q = '<div class="fui-datepicker-quick" role="group" aria-label="Quick choices">' + "".join(
            f'<button type="button" class="fui-chip-check fui-datepicker-chip"{" aria-pressed=\"true\"" if on else " aria-pressed=\"false\""}>{w}</button>' for w, on in quick) + "</div>"
    pre = ""
    if presets:
        pre = '<div class="fui-datepicker-presets" role="group" aria-label="Ranges">' + "".join(
            f'<button type="button" class="fui-datepicker-preset"{" aria-pressed=\"true\"" if on else " aria-pressed=\"false\""}>{w}</button>' for w, on in presets) + "</div>"
    tm = ""
    if time:
        h, m = time
        tm = ('<div class="fui-datepicker-time"><span class="fui-label" id="tm-label">Time <span class="fui-muted">(24 h)</span></span>'
              f'<div class="fui-datepicker-time-fields" role="group" aria-labelledby="tm-label">'
              f'<input class="fui-input" value="{h}" inputmode="numeric" aria-label="Hours" maxlength="2" /><span aria-hidden="true">:</span>'
              f'<input class="fui-input" value="{m}" inputmode="numeric" aria-label="Minutes" maxlength="2" /></div></div>')
    f = ""
    if foot:
        f = (f'<div class="fui-datepicker-foot"><div class="fui-actions">{button("ghost", "Today", small=True)}{button("ghost", "Clear", small=True)}</div>'
             f'<div class="fui-actions">{button("secondary", "Cancel", small=True, keys="Esc")}{button("primary", "Apply", small=True)}</div></div>')
    handle = '<span class="fui-datepicker-handle" aria-hidden="true"></span>' if sheet else ""
    cls = "fui-datepicker" + (" fui-datepicker-range" if presets else "") + (f" {extra_cls}" if extra_cls else "")
    st = f'<p class="fui-datepicker-status" role="status">{status}</p>' if status else ""
    return (f'<div class="{cls}" role="dialog" aria-label="{e(label)}">{handle}<p class="fui-datepicker-label">{label}</p>'
            f'<div class="fui-datepicker-body">{pre}<div class="fui-datepicker-main">{q}{body}{tm}{st}</div></div>{f}</div>')


def date_field_open(id_, value, label, pop, invalid=None):
    inv = ' aria-invalid="true"' if invalid else ""
    er = f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {invalid}</p>' if invalid else ""
    return (f'<div class="fui-field fui-date-field"><label for="{id_}" class="fui-label">{label}</label><div class="fui-input-group fui-date">'
            + input_(id_, value, "YYYY-MM-DD", extra=f'inputmode="numeric" spellcheck="false" aria-haspopup="dialog" aria-expanded="true"{inv}')
            + icon_button("calendar", "Choose a date", cls="fui-input-toggle", attrs='aria-pressed="true"') + f"</div>{er}{pop}</div>")


D = _dt.date
