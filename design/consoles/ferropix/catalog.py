# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The FerroPIX console mockups' page table: each slug of pages.py and the
function that draws it, read by design/tools/build.py."""
import math

import tpl
from common import (
    MENU_SEP, THRESHOLDS, action_group, badge, busy_button, button, card, centred, check, chip, choice_group, compare,
    copy_button, definitions, disclosure, dock, dock_field, e, empty_quiet, empty_state, facts_row, field,
    icon, icon_button, identifier, input_, kbd, link_button, lockup, menu, menu_item, notice, page_heading,
    pager, pill, refusal, score_bar_row, score_cell, score_scale, search_control, segmented_form, select,
    shell, shortcut_hint, shortcut_legend, stat_card, status, stepper, table, tabbed, tabs, tag, td,
    textarea, th, timeline, titled_panel, toast, toasts, well, when, settings_tabs, settings_panel,
)

# ---- synthetic data (every 9-digit number fails the BSN eleven-proof;
# every address is a Voorbeeld- or Teststraat in the 9999 range; no real
# person) ----

def bsn_valid(digits):
    weights = [9, 8, 7, 6, 5, 4, 3, 2, -1]
    return sum(w * int(d) for w, d in zip(weights, digits)) % 11 == 0


BSN_A = "999999123"
for b in (BSN_A, "999999550", "999999777"):
    assert not bsn_valid(b), b

PERSON_A = "0199c2f1-5a8e-7c3e-9b41-00000000a001"
PERSON_A_SHORT = "0199c2f1\u2060…\u2060a001"
PERSON_B = "0199c2f1-5a8e-79a1-9b41-00000000b002"
PERSON_B_SHORT = "0199c2f1\u2060…\u2060b002"
PERSON_C_SHORT = "0199b7d0\u2060…\u2060c003"
MASKED_PHONE = '<span class="fui-identifier-mask" aria-label="masked, ends in 4421"><span class="fui-identifier-dots" aria-hidden="true">••••••</span>4421</span>'


def person_chip(short, full):
    return f'<span class="fui-cluster">{chip(short, full)}{copy_button()}</span>'


# ---- charts, in the frame FerroUI draws ----

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


# ---- the queue: grade, then score, then age (architecture 5.3); nothing
# under possible_at 0.60 is queued ----

CANDIDATES = [
    # a, a_born, a_dom, b, b_born, b_dom, score, grade, waiting, waiting_days
    ("Okafor, Chidi", "1985-11-19", "BSN", "Okafor, Chidi N.", "1985-11-19", "Radiology RIS", 0.97, "certain", "2 d", 2),
    ("Petrova, Irina", "1963-02-17", "BSN", "Petrov, Irina", "1963-02-17", "Laboratory", 0.96, "certain", "22 h", 0.9),
    ("Mohr, Alice", "1958-01-30", "BSN", "Maiden, Alice", "1958-01-30", "Laboratory", 0.91, "probable", "12 d", 12),
    ("Jansen, Piet", "1971-06-02", "Hospital MRN", "Janssen, P.", "1971-06-02", "Hospital MRN", 0.88, "probable", "9 d", 9),
    ("Demir, Ayşe", "1990-03-08", "Hospital MRN", "Demir, Ayse", "1990-03-08", "Laboratory", 0.86, "probable", "8 d", 8),
    ("Testman, Bram", "1949-09-23", "Hospital MRN", "Testman, Abraham", "1949-09-23", "Radiology RIS", 0.84, "probable", "2 d", 2),
    ("Smit, Daan", "2015-08-09", "Hospital MRN", "Smit, D.", "2015-08-09", "Radiology RIS", 0.82, "probable", "6 h", 0.25),
    ("De Vries, Anna", "1990-07-14", "Hospital MRN", "Vries, Anna", "1990-07-14", "Laboratory", 0.79, "possible", "14 d", 14),
    ("Voorbeeld, Carla", "1977-04-30", "Laboratory", "Voorbeeld, C.", "1977-04-03", "Hospital MRN", 0.71, "possible", "7 d", 7),
    ("Nguyen, Linh", "2002-12-01", "BSN", "Nguyen, L.", "2002-12-10", "Hospital MRN", 0.64, "possible", "3 d", 3),
]
DEFERRED = [
    ("Bakker, Joost", "1966-05-12", "Hospital MRN", "Bakker, J.", "1966-05-12", "Laboratory", 0.87, "probable", "back 2026-10-28", 0),
]
GRADE_RANK = {"certain": 0, "probable": 1, "possible": 2}
for c in CANDIDATES:
    assert c[6] >= THRESHOLDS[0][1], c
assert CANDIDATES == sorted(CANDIDATES, key=lambda c: (GRADE_RANK[c[7]], -c[6], -c[9]))

GRADE_TONE = {"probable": "info", "possible": "warn", "certain": "ok", "deferred": "neutral"}


def row_menu(i, a, b, open_=False, focus=None):
    trigger = (
        f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
        f'aria-label="Actions for {e(a)} and {e(b)}" title="More actions">{icon("ellipsis", "16")}</button>'
    )
    m = ""
    if open_:
        m = menu([
            menu_item("Open", "arrow-right", keys="Enter", href="compare.html"),
            menu_item("Open in a side panel", "panel-right", keys="o", focus=focus == "panel"),
            menu_item("Defer…", "clock", keys="d"),
        ], label=f"Actions for {a} and {b}")
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def queue_rows(focused=None, hover=None, kb_focus=None, selected=(), menu_open=None, rows=None, checks=True):
    out = []
    for i, (a, ab, ad, b, bb, bd, score, grade, age, _) in enumerate(rows or CANDIDATES):
        attrs = ""
        cls = []
        if i == focused:
            attrs += ' aria-current="true"'
        if i == hover:
            cls.append("mk-hover")
        if i == kb_focus:
            cls.append("mk-focus")
        if i in selected:
            attrs += ' aria-selected="true"'
        c = f' class="{" ".join(cls)}"' if cls else ""
        deferred = grade == "deferred" or age.startswith("back")
        tone = "neutral" if deferred else GRADE_TONE[grade]
        gp = pill("neutral", "deferred") if deferred else pill(GRADE_TONE[grade], grade)
        box = (
            f'<td class="fui-row-check" data-role="wide"><input type="checkbox" aria-label="Select {e(a)} and {e(b)}"{" checked" if i in selected else ""} /></td>'
            if checks else ""
        )
        out.append(
            f'<tr data-key="cand-{i}"{attrs}{c}>' + box
            + f'<th scope="row" data-role="title"><a class="fui-row-link" href="compare.html">{e(a)}</a> <span class="fui-muted">{ab[:4]}</span></th>'
            + td("and", f'{e(b)} <span class="fui-muted">{bb[:4]}</span>', role="title-2")
            + td("Domains", f"{tag(ad)} {tag(bd)}", role="wide")
            + td("Score", score_cell(score, tone), role="meta-1")
            + td("Grade", gp, role="meta-2")
            + td("Waiting", f'<span class="fui-time">{age}</span>', "fui-num", role="meta-3")
            + td("", row_menu(i, a, b, open_=(i == menu_open)), "fui-row-menu", role="menu")
            + "</tr>"
        )
    return out


def queue_head(checks=True, all_checked=False):
    box = f'<th scope="col" class="fui-row-check"><input type="checkbox" aria-label="Select every candidate on this page"{" checked" if all_checked else ""} /></th>' if checks else ""
    return (
        box + th("Record A") + th("Record B") + th("Domains") + th("Score", num=False)
        + th("Grade", "grade", "descending", title="Sorted by grade, then score, then age")
        + th("Waiting", num=True) + th('<span class="fui-sr-only">Actions</span>')
    )


GRADE_ITEMS = [("certain", "ok", 4, True), ("probable", "warn", 21, True), ("possible", "neutral", 13, True)]


def queue_tools(grade="all", domain="all", age="any", text="", deferred=False, open_=None):
    gw = None if grade == "all" else grade
    aw = {"any": None, "7": "longer than 7 days", "30": "longer than 30 days"}[age]
    dw = None if domain == "all" else domain
    toggle = f'<button type="button" class="fui-chip-check" aria-pressed="{"true" if deferred else "false"}">{icon("clock", "14")}Include deferred</button>'
    items = [(w, t, n, (grade in ("all", w))) for w, t, n, _ in GRADE_ITEMS]
    return tpl.toolbar("A name, birth date or identifier", text,
                       [tpl.filter_button("Grade", gw, open_ == "grade", items), tpl.filter_button("Domain", dw), tpl.filter_button("Waiting", aw), toggle])


def filter_chips(items):
    return tpl.active(items)


def worklist(rows, head, foot_total=38, selection=None, sort_note=True, count="38 candidates · 5 waiting longer than 7 days"):
    hint = shortcut_hint([("j k | up down", "move"), ("Enter", "open"), ("o", "side panel"), ("d", "defer"), ("?", "all shortcuts")])
    pg = pager("Candidates", foot_total, 10) if foot_total else ""
    note = '<p class="fui-sort-note">Sorted by grade, then score, then the longest wait</p>' if sort_note else ""
    foot = f'<div class="fui-worklist-foot">{hint}{pg}</div>'
    sel = selection or f'<div class="fui-worklist-head"><p role="status">{count}</p>{note}</div>'
    return (
        '<div class="fui-card fui-panel fui-worklist" role="grid" aria-label="Candidates">'
        + sel
        + table("Candidates: grade, then score, then the longest wait", head, rows, scroll_label="Candidates")
        + foot
        + "</div>"
    ), note


def queue_page(rows_html=None, tools=None, chips="", selection=None, overlay=None, toast_html="", total=38,
               queue=38, body=None, side=None, count="38 candidates · 5 waiting longer than 7 days"):
    heading = page_heading(
        "Review queue",
        subtitle="Pairs of records the matcher could not decide. A row is one pair; open it to compare and decide.",
    )
    tools = tools if tools is not None else queue_tools()
    if body is None:
        wl, note = worklist(rows_html if rows_html is not None else queue_rows(), queue_head(), total, selection, count=count)
        body = wl
    else:
        note = ""
    content = heading + tpl.list_body("", tools, chips, body)
    if side:
        overlay = side
    return shell("Review queue", "queue", content, overlay=overlay, toast_html=toast_html, queue=queue)


def page_queue():
    return queue_page()


def page_queue_hover():
    return queue_page(queue_rows(hover=2))


def page_queue_focus():
    return queue_page(queue_rows(focused=2, kb_focus=2))


def page_queue_selected():
    sel = (
        '<div class="fui-selection-bar" role="status"><span>2 candidates selected</span>'
        f'<div class="fui-actions">{button("secondary", "Defer…", "clock", small=True)}{button("ghost", "Clear the selection", "x", small=True)}</div></div>'
    )
    return queue_page(queue_rows(selected=(3, 4)), selection=sel)


def page_queue_menu():
    return queue_page(queue_rows(menu_open=2, hover=2))


def page_queue_side_panel():
    a = CANDIDATES[2]
    panel = (
        '<aside class="fui-side-panel mk-panel-open" aria-labelledby="side-title">'
        f'<div class="fui-side-panel-head"><h2 id="side-title" class="fui-card-title">Mohr, Alice and Maiden, Alice</h2>'
        f'{icon_button("x", "Close the panel")}</div>'
        '<div class="fui-side-panel-body">'
        + f'<p class="fui-cluster">{score_cell(0.91, "info")} {pill("info", "probable")} <span class="fui-muted">waiting 12 d</span></p>'
        + definitions([
            ("Family name", f'Mohr <span class="fui-muted">and</span> Maiden {mark_differs()}'),
            ("Given names", "Alice, both"),
            ("Birth date", "1958-01-30, both"),
            ("Postal code", "9999 AB, both"),
            ("Street", f'Voorbeeldstraat 12 <span class="fui-muted">and</span> 21 {mark_differs()}'),
            ("Domains", f'{tag("BSN")} {tag("Laboratory")}'),
        ])
        + '<p class="fui-small fui-muted">Paired by the blocking key postal code and birth date.</p>'
        + f'<div class="fui-actions">{link_button("primary", "Open to decide", "compare.html", "arrow-right", keys="Enter")}{button("secondary", "Defer…", "clock", keys="d")}</div>'
        + "</div></aside>"
    )
    return queue_page(queue_rows(focused=2), side=panel)


def mark_differs():
    return f'<span class="fui-compare-mark fui-compare-differ">{icon("equal-not", "12")}<span>differs</span></span>'


def page_queue_filtered():
    rows = [c for c in CANDIDATES if c[7] == "probable" and c[9] > 7]
    tools = queue_tools(grade="probable", age="7")
    chips = filter_chips(["Grade: probable", "Waiting: longer than 7 days"])
    return queue_page(queue_rows(rows=rows), tools=tools, chips=chips, total=2, count="2 of 38 candidates")


def page_queue_deferred():
    rows = CANDIDATES[:9] + DEFERRED
    rows_html = queue_rows(rows=rows)
    tools = queue_tools(deferred=True)
    return queue_page(rows_html, tools=tools, total=39, count="39 candidates · 1 deferred, back 2026-10-28")


def page_queue_empty():
    body = empty_state(
        "square-check",
        "The queue is empty",
        "Every candidate has been decided. The last decision was at 09:42 today, by Demo Steward. New candidates appear here as the feeds bring records the matcher cannot decide.",
        link_button("secondary", "Go to the dashboard", "dashboard.html", "layout-dashboard"),
    )
    return queue_page(body=body, queue=0)


def page_queue_no_match():
    body = empty_state(
        "search-x",
        "No candidate matches these filters",
        "38 candidates are waiting, but none is certain and waits longer than 30 days. Clear a filter to see them.",
        button("primary", "Clear filters", "x") + button("secondary", "Remove “Waiting: longer than 30 days”"),
        filtered=True,
    )
    tools = queue_tools(grade="certain", age="30")
    chips = filter_chips(["Grade: certain", "Waiting: longer than 30 days"])
    return queue_page(tools=tools, chips=chips, body=body)


def page_queue_loading():
    rows = "".join(
        '<tr aria-hidden="true"><td class="fui-row-check" data-role="wide"><span class="fui-skeleton mk-w100 mk-sk-box"></span></td>'
        f'<th data-role="title">{skeleton_lines([70])}</th><td data-role="title-2">{skeleton_lines([60])}</td>'
        f'<td data-role="wide">{skeleton_lines([80])}</td><td data-role="meta-1">{skeleton_lines([70])}</td>'
        f'<td data-role="meta-2">{skeleton_lines([50])}</td><td data-role="meta-3">{skeleton_lines([40])}</td><td></td></tr>'
        for _ in range(8)
    )
    body = (
        '<div class="fui-card fui-panel fui-worklist" aria-busy="true" aria-label="Candidates, loading">'
        '<div class="fui-worklist-head"><p role="status"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Loading the candidates</span></p></div>'
        + table("Candidates, loading", queue_head(), [rows], scroll_label="Candidates")
        + "</div>"
    )
    return queue_page(body=body, queue=38)


def unreachable_refusal(what):
    return (
        refusal(f"The FerroPIX server did not answer, so {what} cannot be shown.")
        + '<p class="mk-refusal-line">The connection was refused at 09:42:11 today. Your session is kept; nothing you entered is lost.</p>'
    )


def tech_detail():
    return disclosure(
        "Technical detail",
        definitions([
            ("Failure", "Connection refused by the server's address"),
            ("Request id", "<code>01HZ8X3K7Q4M</code>"),
            ("Time", when("2026-10-21T09:42:11", seconds=True)),
            ("Attempts", "3 of 3, the last 4 seconds ago"),
        ]),
    )


def page_queue_unreachable():
    body = (
        '<div class="fui-stack">'
        + refusal("The queue cannot be shown: the FerroPIX server refused the connection.")
        + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried {when("2026-10-21T09:42:11", seconds=True)} · request id <code>01HZ8X3K7Q4M</code></span></div>'
        + tech_detail()
        + "</div>"
    )
    return shell("Review queue", "queue", page_heading("Review queue", subtitle="Pairs of records the matcher could not decide.") + f'<div class="fui-stack">{body}</div>',
                 status=("bad", "Unreachable"), queue=None)


# ---- compare and decide ----

FIELDS = [
    ("family", "Family name"), ("given", "Given names"), ("born", "Birth date"), ("gender", "Gender"),
    ("postal", "Postal code"), ("street", "Street"), ("phone", "Telephone"), ("identifier", "Identifier"),
]


def rec_a(revealed=False):
    return {
        "key": "a", "title": "Mohr, Alice", "short": "A · Mohr", "letter": "A", "sub": "BSN record · fed " + when("2026-10-20T09:41"),
        "values": {
            "family": "Mohr", "given": "Alice", "born": "1958-01-30", "gender": "female", "postal": "9999 AB",
            "street": "Voorbeeldstraat 12", "phone": MASKED_PHONE,
            "identifier": identifier("BSN", "123", reveal_id="reveal"),
        },
    }


def rec_b(revealed=False):
    ident = identifier("Laboratory", "550", revealed="LAB-0077-550", left=24) if revealed else identifier("Laboratory", "550", reveal_id="reveal")
    return {
        "key": "b", "title": "Maiden, Alice", "short": "B · Maiden", "sub": "Laboratory record · fed " + when("2026-09-02T08:15"),
        "values": {
            "family": "Maiden", "given": "Alice", "born": "1958-01-30", "gender": None, "postal": "9999 AB",
            "street": "Voorbeeldstraat 21", "phone": MASKED_PHONE, "identifier": ident,
        },
    }


VERDICT_AB = {"family": "differ", "given": "agree", "born": "agree", "gender": "absent", "postal": "agree", "street": "differ", "phone": "agree", "identifier": None}


def pair_header():
    def rec(letter, name, domain, person, others):
        return (
            f'<div class="mk-pair-record"><span class="mk-pair-name"><span class="mk-pair-letter" aria-hidden="true">{letter}</span>{e(name)}'
            f'<span class="fui-muted mk-pair-domain">{domain}</span></span>'
            f'<span class="mk-pair-facts"><span class="fui-small fui-muted">Person</span>{chip(person)}<span class="fui-small fui-muted">Other identifiers</span>{others}</span></div>'
        )
    return (
        '<div class="mk-pair">'
        + rec("A", "Mohr, Alice", "BSN record", PERSON_A_SHORT, identifier("Hospital MRN", "994", static=True))
        + rec("B", "Maiden, Alice", "Laboratory record", PERSON_B_SHORT, '<span class="fui-muted">none</span>')
        + "</div>"
    )


def score_card():
    rows = (
        score_bar_row("Birth date", "exact", 6.0)
        + score_bar_row("Given names", "exact", 4.2)
        + score_bar_row("Telephone", "exact, normalised", 3.9)
        + score_bar_row("Postal code", "exact", 2.6)
        + score_bar_row("Gender", "absent in B", 0.0)
        + score_bar_row("Street", "token overlap 0.5", -0.4)
        + score_bar_row("Family name", "Jaro-Winkler 0.71", -3.1)
    )
    total = (
        '<div class="fui-score-total"><span>Total weight <span class="fui-muted">+13.2</span></span>'
        '<span>Score <span class="fui-score-figure">0.91</span> ' + pill("info", "probable") + "</span></div>"
    )
    scale = score_scale(0.91, "Score 0.91, probable, on the scale: possible from 0.60, probable from 0.80, certain from 0.95. Birth date +6.0, given names +4.2, telephone +3.9, postal code +2.6, gender 0, street −0.4, family name −3.1.")
    return card("Score breakdown", f'<div class="fui-score">{rows}</div>{total}{scale}'
                '<p class="fui-small fui-muted">Paired by the blocking key postal code and birth date. Weights as set in <a href="configuration.html">Configuration</a>, revision 7.</p>')


DECISIONS = [("primary", "Link", "link", "l"), ("secondary", "Not a match", "circle-x", "n"), ("secondary", "Defer", "clock", "d")]


def decision_dock(state="idle", reason="Maiden name; same telephone and address history"):
    msg = ""
    err = None
    disabled_field = False
    acts = ""
    if state == "missing":
        reason = ""
        err = "Write why you decided this; the reason goes into the audit trail with your name."
    for variant, label, glyph, key in DECISIONS:
        if state == "submitting":
            if label == "Link":
                acts += busy_button(variant, "Linking")
            else:
                acts += button(variant, label, glyph, attrs="disabled", keys=key)
        elif state in ("done", "stale"):
            acts += button(variant, label, glyph, attrs="disabled", keys=key)
        else:
            acts += button(variant, label, glyph, keys=key)
    if state == "submitting":
        disabled_field = True
    if state == "done":
        disabled_field = True
        msg = (
            '<p class="fui-dock-message fui-dock-done" role="status">'
            f'{icon("circle-check", "16")}Linked: Maiden, Alice is now part of Mohr, Alice. '
            '<span class="fui-spinner fui-small"><span class="fui-spinner-ring" aria-hidden="true"></span>Opening the next candidate, 3 of 37</span></p>'
        )
    if state == "stale":
        msg = (
            '<div class="fui-dock-message">'
            + notice("warn", "triangle-alert", "Someone decided first: Demo Reviewer linked this pair at 09:41 today. Your decision was not sent; your reason is kept for the reloaded candidate.",
                     action=" " + button("secondary", "Reload the candidate", "refresh-cw", small=True))
            + "</div>"
        )
    fld = dock_field("reason", "Reason", reason, "Why you decided this", keys="r", error=err, disabled=disabled_field)
    return dock(fld, acts, msg)


def compare_body(revealed=False, skeleton=False):
    cmp_ = compare(
        "The two records, field by field, the differing fields first",
        FIELDS, rec_a(), [rec_b(revealed)], lambda f, i: VERDICT_AB[f],
        note="2 fields differ, 4 agree, 1 is absent in B. The differing fields come first.",
    )
    main = card("Fields", cmp_)
    return f'<div class="fui-stack">{pair_header()}<div class="fui-main-aside"><div class="fui-main-part">{main}</div><div class="fui-aside-part">{score_card()}</div></div></div>'


def compare_heading(position="Candidate 3 of 38", title="These two records look like the same patient", found="found 12 days ago"):
    return page_heading(
        title,
        actions=stepper(position, "compare.html", "compare.html"),
        subtitle=f"{found} · your decision is recorded with your name and reason.",
        crumbs=[("Review queue", "queue.html")],
    )


def page_compare(state="idle", overlay=None, revealed=False, toast_html=""):
    content = compare_heading() + compare_body(revealed)
    return shell("Compare and decide", "queue", content, overlay=overlay,
                 dock_html=decision_dock(state), toast_html=toast_html)


def page_compare_done():
    return page_compare("done")


def page_compare_not_a_match():
    # The next candidate has loaded; the last decision is confirmed in a toast.
    nxt = (
        compare_heading("Candidate 3 of 37", found="found 9 days ago")
        + '<div class="fui-stack">'
        + '<div class="mk-pair">'
        + f'<div class="mk-pair-record"><span class="mk-pair-name"><span class="mk-pair-letter" aria-hidden="true">A</span>Jansen, Piet<span class="fui-muted mk-pair-domain">Hospital MRN record</span></span>'
        + f'<span class="mk-pair-facts"><span class="fui-small fui-muted">Person</span>{chip("0199a4c2…d004")}<span class="fui-small fui-muted">Other identifiers</span><span class="fui-muted">none</span></span></div>'
        + f'<div class="mk-pair-record"><span class="mk-pair-name"><span class="mk-pair-letter" aria-hidden="true">B</span>Janssen, P.<span class="fui-muted mk-pair-domain">Hospital MRN record</span></span>'
        + f'<span class="mk-pair-facts"><span class="fui-small fui-muted">Person</span>{chip("0199a4c2…e005")}<span class="fui-small fui-muted">Other identifiers</span><span class="fui-muted">none</span></span></div>'
        + "</div>"
        + '<div class="fui-main-aside"><div class="fui-main-part">'
        + card("Fields", '<p class="fui-small">1 field differs, 5 agree, 1 is absent in B.</p>' + compare(
            "The two records", [("family", "Family name"), ("given", "Given names"), ("born", "Birth date"), ("postal", "Postal code"), ("street", "Street"), ("gender", "Gender")],
            {"key": "a", "title": "Jansen, Piet", "short": "A · Jansen", "sub": "Hospital MRN record · fed " + when("2026-10-12T10:20"),
             "values": {"family": "Jansen", "given": "Piet", "born": "1971-06-02", "postal": "9997 CD", "street": "Teststraat 8", "gender": "male"}},
            [{"key": "b", "title": "Janssen, P.", "short": "B · Janssen", "sub": "Hospital MRN record · fed " + when("2026-10-11T15:02"),
              "values": {"family": "Janssen", "given": "P.", "born": "1971-06-02", "postal": "9997 CD", "street": "Teststraat 8", "gender": None}}],
            lambda f, i: {"family": "differ", "given": "agree", "born": "agree", "postal": "agree", "street": "agree", "gender": "absent"}[f]))
        + '</div><div class="fui-aside-part">'
        + card("Score breakdown", '<div class="fui-score">' + score_bar_row("Birth date", "exact", 6.0) + score_bar_row("Postal code", "exact", 2.6)
               + score_bar_row("Street", "token overlap 1.0", 2.2) + score_bar_row("Given names", "initial agrees: half", 2.1) + score_bar_row("Gender", "absent in B", 0)
               + score_bar_row("Family name", "Jaro-Winkler 0.96", 5.1) + '</div><div class="fui-score-total"><span>Total weight <span class="fui-muted">+18.0</span></span><span>Score <span class="fui-score-figure">0.88</span> '
               + pill("info", "probable") + "</span></div>" + score_scale(0.88, "Score 0.88, probable"))
        + "</div></div></div>"
    )
    t = toasts(toast("ok", "circle-check", "Not a match: Mohr, Alice and Maiden, Alice",
                     "The pair is not queued again until either record changes. Recorded with your reason."))
    return shell("Compare and decide", "queue", nxt, dock_html=decision_dock("idle", ""), toast_html=t, queue=37)


def page_shortcuts():
    legend = shortcut_legend([
        ("Compare and decide", [("l", "Link the records"), ("n", "Not a match"), ("d", "Defer the candidate"), ("r", "Go to the reason"),
                                ("right", "Next candidate"), ("left", "Previous candidate")]),
        ("Lists", [("j | down", "Next row"), ("k | up", "Previous row"), ("Enter", "Open the row"), ("o", "Open in a side panel")]),
        ("Everywhere", [("/", "Find a patient"), ("?", "This legend"), ("Esc", "Close a dialog, a menu or a panel"), ("g d", "Go to the dashboard"), ("g q", "Go to the review queue"), ("g p", "Go to your preferences")]),
        ("Patient", [("m", "Merge into…")]),
    ])
    return page_compare(overlay=legend)


def page_decision_states():
    """The states sheet of the docked decision bar (FerroUI #145)."""
    def frame(title, line, html):
        return (f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                f'<div class="mk-dock-demo">{html}</div></div>')
    states = [
        ("Idle", "the reason at the start, the decisions at the end; Link is the primary", decision_dock("idle")),
        ("Idle, empty", "nothing typed yet; the decisions stay enabled", decision_dock("idle", "")),
        ("Reason missing", "a decision without a reason focuses the field with its error", decision_dock("missing")),
        ("Submitting", "the chosen decision is busy, the others and the field wait", decision_dock("submitting")),
        ("Done", "the bar confirms in words, and the next candidate loads", decision_dock("done")),
        ("Stale, 409", "someone decided first: nothing is sent, the reason is kept, Reload", decision_dock("stale")),
    ]
    sheet = "".join(frame(*s) for s in states)
    keys = card("Keys and focus", definitions([
        ("L, N, D", "decide; each acts only when no field has the focus, and shows as a quiet cap after its label at wide widths"),
        ("R", "moves the focus to the reason"),
        ("Enter in the reason", "does nothing on its own: a decision is always one deliberate key or click"),
        ("← and →", "the previous and next candidate, in the page header beside the position"),
        ("Phone", "the reason on its own row, the three decisions sharing the row under it; no key hints"),
    ]))
    content = page_heading("Decision bar, states", subtitle="FerroUI #145: docked to the foot of the main area (the window's foot on a phone), one top rule, no shadow and no margin. Previous and next live in the page header.") + f'<div class="fui-stack mk-dock-sheet">{sheet}{keys}</div>'
    return shell("Decision bar states", "queue", content)


# ---- dashboard ----

QUEUE_DEPTH = [52, 49, 47, 51, 55, 58, 54, 50, 46, 44, 43, 47, 49, 52, 48, 45, 41, 39, 42, 44, 40, 38, 37, 41, 43, 40, 39, 36, 38, 38]
FEEDS_FHIR = [120, 95, 80, 70, 60, 90, 180, 260, 310, 290]
FEEDS_V2 = [40, 35, 30, 28, 30, 50, 120, 170, 190, 175]
FEEDS_PMIR = [5, 3, 2, 2, 3, 8, 14, 20, 22, 18]


def oldest_rows():
    rows = []
    for a, ab, ad, b, bb, bd, score, grade, age, days in sorted(CANDIDATES, key=lambda c: -c[9])[:5]:
        pair = f"{a} and {b}"
        rows.append(
            f'<tr><th scope="row"><a class="fui-truncate mk-pair-line" href="compare.html" title="{e(pair)}">{e(a)} <span class="fui-muted">and</span> {e(b)}</a></th>'
            + td("Grade", pill(GRADE_TONE[grade], grade)) + td("Waiting", age, "fui-num") + "</tr>"
        )
    return rows


def dashboard_heading():
    return page_heading("Dashboard", subtitle="Wednesday 21 October 2026 · the day's state of the index.")


def page_dashboard(search_state=None, search_value="", results=None, menu_open=False, menu_theme="light",
                   rail=False, rail_tip=None, drawer=False, toast_html="", queue=38):
    stats = (
        '<div class="fui-grid fui-grid-stats">'
        + stat_card("inbox", "38", "Waiting in the queue", badge("warn", "5 longer than 7 days"), href="queue.html")
        + stat_card("link", "92 %", "Auto-linked, last 24 hours", badge("neutral", "1,106 of 1,204 records"))
        + stat_card("activity", "1,204", "Records fed today", badge("neutral", "FHIR 812 · v2 371 · PMIR 21"))
        + stat_card("bell", "0", "Subscriptions in error", pill("ok", "7 delivering"))
        + "</div>"
    )
    charts = (
        '<div class="fui-grid">'
        + card("Queue depth, 30 days", line_chart("depth", "Queue depth over 30 days", "Between 36 and 58 candidates waiting; 38 today.", [("Waiting", QUEUE_DEPTH, 0)], [(0, "21 Sep"), (15, "6 Oct"), (29, "Today")]))
        + card("Feeds per hour, today", line_chart("feeds", "Feeds per hour by channel", "FHIR peaks at 310 at 08:00, v2 at 190, PMIR at 22.", [("FHIR", FEEDS_FHIR, 0), ("v2", FEEDS_V2, 1), ("PMIR", FEEDS_PMIR, 2)], [(0, "00:00"), (3, "03:00"), (6, "06:00"), (9, "09:00")]))
        + "</div>"
    )

    def listener(name, st, lines):
        return f'<div class="mk-listener"><div class="mk-listener-line"><strong>{name}</strong>{st}</div><div class="fui-small fui-muted">{lines}</div></div>'

    listeners = card(
        "Listeners",
        '<div class="mk-listeners">'
        + listener("HTTP face", status("ok", "circle-check", "up", "2 ms p50 · 11 ms p99"), "FHIR R4 PIXm, PDQm and PMIR · 1,312 requests this hour")
        + listener("MLLP listener", status("ok", "circle-check", "up", "last message today 09:41"), "HL7 v2 ADT · 3 senders today: his-prod, lab-lis, ris-prod")
        + listener("Feed sender", status("ok", "circle-check", "0 queued", "last sent today 09:41"), "7 subscriptions · ITI-93 notifications · 0 retries today")
        + listener("Database", status("ok", "circle-check", "up", "pool 4 of 16"), "PostgreSQL 18 · 412,318 persons · last vacuum today 03:00")
        + "</div>",
    )
    oldest = titled_panel(
        "Oldest in the queue",
        link_button("secondary", "All candidates", "queue.html", glyph_end="chevron-right", small=True),
        table("The five longest-waiting candidates", th("Pair") + th("Grade") + th("Waiting", num=True), oldest_rows()),
    )
    content = dashboard_heading() + f'<div class="fui-stack fui-gap-section">{stats}{charts}<div class="fui-grid fui-grid-start">{listeners}{oldest}</div></div>'
    return shell("Dashboard", "dashboard", content, search_state=search_state, search_value=search_value, results=results,
                 menu_open=menu_open, menu_theme=menu_theme, rail=rail, rail_tip=rail_tip, drawer=drawer,
                 toast_html=toast_html, queue=queue)


def page_dashboard_loading():
    def sk_card(lines):
        return f'<div class="fui-card mk-sk-card" aria-hidden="true">{skeleton_lines(lines)}</div>'
    stats = '<div class="fui-grid fui-grid-stats">' + "".join(
        f'<div class="fui-card fui-stat" aria-hidden="true"><span class="fui-stat-glyph mk-sk-glyph"></span><div class="fui-stat-text mk-grow">{skeleton_lines([40, 80])}</div></div>'
        for _ in range(4)) + "</div>"
    charts = '<div class="fui-grid">' + "".join(
        f'<div class="fui-card" aria-hidden="true">{skeleton_lines([40])}<span class="fui-skeleton mk-sk-chart"></span></div>' for _ in range(2)) + "</div>"
    lower = '<div class="fui-grid fui-grid-start">' + sk_card([30, 90, 70, 90, 60, 90, 70]) + sk_card([40, 100, 100, 100, 100, 100]) + "</div>"
    content = dashboard_heading() + '<p class="fui-sr-only" role="status">Loading the dashboard</p>' + f'<div class="fui-stack fui-gap-section" aria-busy="true">{stats}{charts}{lower}</div>'
    return shell("Dashboard", "dashboard", content, queue=None)


def page_unreachable():
    content = (
        dashboard_heading()
        + '<div class="fui-stack">'
        + refusal("The FerroPIX server refused the connection, so the dashboard cannot be shown.")
        + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried {when("2026-10-21T09:42:11", seconds=True)} · request id <code>01HZ8X3K7Q4M</code></span></div>'
        + card("What still works", '<p class="fui-muted">Nothing patient-related is kept in this console, so no page can be drawn until the server answers. Your session is kept: you do not need to sign in again.</p>')
        + tech_detail()
        + "</div>"
    )
    return shell("Dashboard", "dashboard", content, status=("bad", "Unreachable"), queue=None)


# ---- the topbar search, one page per state ----

def search_results_list(kind):
    if kind == "none":
        body = (
            f'<div class="mk-results-empty">{icon("search-x", "16")}<div><strong>No patient found for “Mohr 1985”</strong>'
            '<span>Check the spelling or the year, or search by demographics for near spellings.</span></div></div>'
            f'<a href="search-demographics.html" class="mk-results-more">Search by demographics{icon("arrow-right", "14")}</a>'
        )
    else:
        people = [("Mohr, Alice", "1958-01-30", "female", "BSN", "123", "0.97"), ("Mohr, Alissa", "1958-01-30", "female", "Hospital MRN", "m94", "0.74")]
        body = '<p class="mk-results-head">2 patients · name and year</p><ul role="listbox" aria-label="Patients found">' + "".join(
            f'<li role="option"{" aria-selected=\"true\"" if i == 0 else ""}><a href="patient-identifiers.html"><span class="mk-results-name">{e(n)}</span>'
            f'<span class="fui-small fui-muted">{b} · {g}</span>{identifier(d, t, static=True)}</a></li>'
            for i, (n, b, g, d, t, s) in enumerate(people)
        ) + f'</ul><a href="search.html" class="mk-results-more">All results in Search{icon("arrow-right", "14")}</a>'
    return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'


def topbar_state_page(state=None, value="", results=None):
    return page_dashboard(search_state=state, search_value=value, results=results)


def page_search_states():
    from common import search_control as sc
    states = [
        ("Default", "the icon, the placeholder and the quiet / hint", "", None),
        ("Hover", "the edge takes the ink", "", "hover"),
        ("Focused, empty", "one ring round the whole group; the hint is gone", "", "focus"),
        ("Focused and filled", "the ring, the typed value, the hint gone", "Mohr 1958", "focus-filled"),
        ("Filled, not focused", "the hint stays hidden while the field holds text", "Mohr 1958", None),
        ("Busy", "the icon becomes the spinner while the search runs", "Mohr 1958", "busy"),
        ("Disabled", "the sunken ground, no pointer; the shortcut does nothing", "", "disabled"),
    ]
    cards = ""
    for i, (title, line, value, state) in enumerate(states):
        cards += f'<div class="fui-card"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>{sc(value, state, id_=f"find-{i}")}</div>'
    legend = card("Keys", definitions([
        (kbd("/"), "focuses the field from anywhere outside an editable element; announced through <code class='mk-nowrap'>aria-keyshortcuts</code> on the input"),
        (kbd("Esc"), "clears the field and returns the focus to the page"),
        (kbd("Enter"), "posts the search; the console reads the shape of what was typed"),
        (kbd("Tab"), "one stop for the field, one for the submit"),
    ]))
    content = page_heading("Search control, states", subtitle="FerroUI #144: one joined control with the icon, the field, the / hint and the submit. Below 40 rem the hint is gone and the submit keeps its icon alone, named for a reader.") + f'<div class="fui-stack"><div class="mk-states">{cards}</div>{legend}</div>'
    return shell("Search control states", "search", content)


# ---- search ----

def search_result_rows(with_score=False):
    people = [
        ("Mohr, Alice", "1958-01-30", "female", [("BSN", "123"), ("Hospital MRN", "994")], "2026-10-20T09:41", 0.97, "certain"),
        ("Maiden, Alice", "1958-01-30", "", [("Laboratory", "550")], "2026-09-02T08:15", 0.91, "probable"),
        ("Mohr, Alissa", "1958-01-30", "female", [("Hospital MRN", "m94")], "2026-07-11T16:20", 0.74, "possible"),
    ]
    if with_score:
        people.sort(key=lambda p: -p[5])
    else:
        people = [people[0], people[2], people[1]]
    rows = []
    for name, born, gender, ids, fed, score, grade in people:
        idents = '<span class="mk-idents">' + "".join(identifier(d, t, static=True) for d, t in ids) + "</span>"
        rows.append(
            f'<tr><th scope="row"><a href="patient-identifiers.html">{e(name)}</a></th>'
            + td("Born", born) + td("Gender", gender or '<span class="mk-dash" title="absent">–</span>') + td("Identifiers", idents)
            + (td("Score", score_cell(score, GRADE_TONE[grade])) + td("Grade", pill(GRADE_TONE[grade], grade)) if with_score else "")
            + td("Last fed", rel_time(fed)) + "</tr>"
        )
    return rows


def page_search(demographics=False, shown=False):
    seg = segmented_form("Search by", [("identifier", "By identifier"), ("demographics", "By demographics")], "demographics" if demographics else "identifier")
    if not demographics:
        eye = (f'<button type="button" class="fui-icon-button fui-input-toggle" aria-pressed="{"true" if shown else "false"}" aria-label="Show the identifier while held" '
               f'title="Show while held">{icon("eye-off" if shown else "eye", "16")}</button>')
        form = (
            '<form method="post" action="#" class="fui-form mk-search-form"><div class="fui-form-grid mk-id-grid">'
            + field("domain", "Domain", '<div class="fui-combobox"><div class="fui-combobox-box">' + input_("domain", "BSN", "Choose a domain", extra='role="combobox" aria-expanded="false" aria-autocomplete="list"', cls="fui-input fui-combobox-input") + "</div></div>")
            + field("identifier", "Identifier", '<div class="fui-input-group">' + input_("identifier", "999999123", "", "text" if shown else "password", extra='autocapitalize="off" spellcheck="false" aria-describedby="identifier-help"') + eye + "</div>", cls="fui-span-2")
            + f'<div class="mk-grid-button">{button("primary", "Search", "search", type_="submit")}</div>'
            + "</div>"
            + '<p id="identifier-help" class="fui-help">Masked as you type and sent in the request body, never in the address bar. Hold the eye to check what you typed.</p></form>'
        )
        results_head = th("Name", "name", "ascending") + th("Born") + th("Gender") + th("Identifiers") + th("Last fed")
        count = "1 patient holds this identifier; 2 more share the name and birth date"
        rows = search_result_rows()
    else:
        form = (
            '<form method="post" action="#" class="fui-form mk-search-form"><div class="fui-form-grid">'
            + field("family", "Family name", input_("family", "Mohr"))
            + field("given", "Given names", input_("given", "Alice"))
            + field("born", "Birth date", date_input("born", "1958-01-30"))
            + field("gender", "Gender", select("gender", [("", "Any"), ("female", "Female"), ("male", "Male"), ("other", "Other"), ("unknown", "Unknown")], ""))
            + field("postal", "Postal code", input_("postal", "", "9999 AB"))
            + "</div>"
            + '<div class="fui-form-foot">' + check("exact", "Exact names only", False, "Off: the scorer accepts a near spelling, such as Jansen for Janssen.")
            + button("primary", "Search", "search", type_="submit") + "</div></form>"
        )
        results_head = th("Name") + th("Born") + th("Gender") + th("Identifiers") + th("Score", "score", "descending") + th("Grade") + th("Last fed")
        count = "3 patients scored against what you typed, the likeliest first"
        rows = search_result_rows(with_score=True)
    results = titled_panel(
        f'Results <span class="mk-count">· {count}</span>', "",
        table("Search results", results_head, rows),
    )
    content = (
        page_heading("Search", subtitle="Find a patient by one identifier, or by the demographics the matcher scores. Results live in this page and never in a URL.")
        + f'<div class="fui-stack">{card("Find a patient", seg + form)}{results}</div>'
    )
    return shell("Search", "search", content)


# ---- patient ----

PATIENT_TABS = [("identifiers", "Identifiers", "patient-identifiers.html", 4), ("records", "Records", "patient-records.html", 3),
                ("history", "History", "patient-history.html", 8), ("pseudonyms", "Pseudonyms", "patient-pseudonyms.html", 2)]


def patient_actions(menu_open=False):
    return action_group(link_button("primary", "Merge into…", "merge-preview.html", "merge", keys="m"),
                        [menu_item("Resolve a pseudonym", "fingerprint"), menu_item("Copy the person id", "copy"), MENU_SEP,
                         menu_item("Unmerge the merge of 2026-06-30…", "undo", href="patient-unmerge.html", danger=True)],
                        open_=menu_open, label="More actions for Mohr, Alice")


def patient_heading(menu_open=False):
    sub = (
        '<span class="fui-cluster">Born 1958-01-30 · female · 3 active records in 3 domains · '
        + person_chip(PERSON_A_SHORT, PERSON_A) + "</span>"
    )
    return page_heading("Mohr, Alice", actions=patient_actions(menu_open), subtitle=sub, crumbs=[("Search", "search.html")])


def patient_page(tab, panel, overlay=None, menu_open=False, focus=None, hover=None, scrolled=False, toast_html="", title="Mohr, Alice", script=""):
    content = patient_heading(menu_open) + tabbed(tabs("Patient", PATIENT_TABS, tab, focus=focus, hover=hover, scrolled=scrolled), panel)
    return shell(title, "search", content, overlay=overlay, toast_html=toast_html, script=script)


def identifier_rows(bsn="masked"):
    if bsn == "revealed":
        b = identifier("BSN", "123", revealed="999 999 123", left=27, show_domain=False)
    elif bsn == "copied":
        b = identifier("BSN", "123", revealed="999 999 123", left=21, show_domain=False, copied=True)
    elif bsn == "busy":
        b = identifier("BSN", "123", state="busy", show_domain=False)
    elif bsn == "expired":
        b = identifier("BSN", "123", reveal_id="reveal", state="expired", show_domain=False)
    else:
        b = identifier("BSN", "123", reveal_id="reveal", show_domain=False)
    rows = [
        ("BSN", b, pill("ok", "active"), "2026-10-20T09:41", "feed <span class=\"fui-muted\">brp-feed · minted this person</span>"),
        ("Hospital MRN", identifier("Hospital MRN", "994", reveal_id="reveal", show_domain=False), pill("ok", "active"), "2026-10-19T14:03",
         "deterministic <span class=\"fui-muted\">the BSN agreed</span>"),
        ("Laboratory", identifier("Laboratory", "550", reveal_id="reveal", show_domain=False), pill("ok", "active"), "2026-09-02T08:15",
         f"steward <span class=\"fui-muted\">Demo Steward · {when('2026-10-21T09:42')}</span>"),
        ("Radiology RIS", identifier("Radiology RIS", "077", reveal_id="reveal", show_domain=False), pill("neutral", "subsumed"), "2026-06-30T11:20",
         f"merge <span class=\"fui-muted\">from {PERSON_C_SHORT}</span>"),
    ]
    return [
        f'<tr><th scope="row">{e(d)}</th>' + td("Identifier", ident) + td("State", st) + td("Last fed", when(fed)) + td("Linked by", by) + "</tr>"
        for d, ident, st, fed, by in rows
    ]


def identifiers_panel(bsn="masked"):
    live = {"revealed": '<p class="fui-sr-only" role="status">BSN revealed for 30 seconds.</p>',
            "expired": '<p class="fui-sr-only" role="status">The BSN is hidden again.</p>'}.get(bsn, "")
    return titled_panel(
        "Identifiers <span class=\"mk-count\">· one per domain</span>",
        '<span class="fui-small fui-muted">A reveal shows the value for 30 seconds and is recorded.</span>',
        table("Identifiers of Mohr, Alice", th("Domain") + th("Identifier") + th("State") + th("Last fed") + th("Linked by"), identifier_rows(bsn)),
    ) + live


def page_patient_identifiers(bsn="masked", overlay=None, menu_open=False, toast_html=""):
    return patient_page("identifiers", identifiers_panel(bsn), overlay=overlay, menu_open=menu_open, toast_html=toast_html)


def reveal_dialog():
    return (
        '<div id="reveal" role="dialog" aria-labelledby="reveal-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="reveal-title" class="fui-dialog-title">{icon("eye", "18")} Reveal the BSN of Mohr, Alice?</h2>'
        '<p>The value is fetched from the server, shown for 30 seconds, and the reveal is recorded in the audit trail with your name and this reason.</p>'
        '<div class="fui-modal-body">'
        + field("reveal-reason", "Reason", textarea("reveal-reason", "Phone call with the laboratory about sample LAB-…550"), help_="Required. A colleague reading the trail should understand it.")
        + "</div>"
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", "x")}{button("primary", "Reveal for 30 s", "eye")}</div></div>'
    )


def page_reveal():
    return page_patient_identifiers(overlay=reveal_dialog())


def page_merge_picker():
    dialog = (
        '<div id="merge-pick" role="dialog" aria-labelledby="merge-pick-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="merge-pick-title" class="fui-dialog-title">{icon("merge", "18")} Merge which patient into Mohr, Alice?</h2>'
        '<p class="fui-muted">Find the other patient. Nothing is written here: the next step shows what the merge would change.</p>'
        '<div class="fui-modal-body">'
        + field("merge-find", "Patient", '<div class="fui-input-group">' + input_("merge-find", "Maiden", extra='role="combobox" aria-expanded="true" aria-controls="merge-options"') + "</div>", help_="A name, a birth date, or a person id. An identifier is searched masked, as in Search.")
        + '<ul id="merge-options" role="listbox" class="mk-pick-list" aria-label="Patients found">'
        + f'<li role="option" aria-selected="true"><span class="mk-pick-main"><strong>Maiden, Alice</strong><span class="fui-small fui-muted">1958-01-30 · 1 record</span></span>{identifier("Laboratory", "550", static=True)}{chip(PERSON_B_SHORT)}</li>'
        + f'<li role="option"><span class="mk-pick-main"><strong>Maiden, Alicia</strong><span class="fui-small fui-muted">1961-08-14 · 2 records</span></span>{identifier("Hospital MRN", "312", static=True)}{chip("0199b0e3…f006")}</li>'
        + "</ul></div>"
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", "x")}{link_button("primary", "Preview the merge", "merge-preview.html", "arrow-right")}</div></div>'
    )
    return page_patient_identifiers(overlay=dialog)


def page_resolve_dialog():
    dialog = (
        '<div id="resolve" role="dialog" aria-labelledby="resolve-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="resolve-title" class="fui-dialog-title">{icon("fingerprint", "18")} Resolve a pseudonym</h2>'
        '<p class="fui-muted">Answers the patient a pseudonym belongs to. Recorded as a disclosure with your name and reason.</p>'
        '<div class="fui-modal-body">'
        + field("res-ns", "Pseudonymisation domain", select("res-ns", [("research-2026-q3", "research-2026-q3 · cohort study"), ("ferroehr-nl", "ferroehr-nl · research export")], "research-2026-q3"))
        + field("res-p", "Pseudonym", input_("res-p", "c41e-8kd", extra='spellcheck="false" autocapitalize="off"'), help_="Shown as typed: a pseudonym names no one without this step, and you need to check what you pasted. It is posted, never put in a URL, and the trail keeps its last three characters.")
        + field("res-r", "Reason", textarea("res-r", "Research ticket RES-204: withdrawal of consent"))
        + "</div>"
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", "x")}{button("primary", "Resolve", "fingerprint")}</div></div>'
    )
    return page_patient_identifiers(overlay=dialog)


def page_patient_loading():
    rows = "".join(
        f'<tr aria-hidden="true"><th>{skeleton_lines([70])}</th><td>{skeleton_lines([60])}</td><td>{skeleton_lines([40])}</td><td>{skeleton_lines([70])}</td><td>{skeleton_lines([80])}</td></tr>'
        for _ in range(4)
    )
    panel = ('<p class="fui-sr-only" role="status">Loading the identifiers</p>'
             + titled_panel("Identifiers", "", table("Identifiers, loading", th("Domain") + th("Identifier") + th("State") + th("Last fed") + th("Linked by"), [rows])))
    return patient_page("identifiers", panel)


def page_patient_unreachable():
    panel = (
        '<div class="fui-stack">'
        + refusal("The identifiers cannot be shown: the FerroPIX server refused the connection.")
        + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried {when("2026-10-21T09:42:11", seconds=True)} · request id <code>01HZ8X3K7Q4M</code></span></div>'
        + tech_detail() + "</div>"
    )
    content = patient_heading() + tabbed(tabs("Patient", PATIENT_TABS, "identifiers"), panel)
    return shell("Mohr, Alice", "search", content, status=("bad", "Unreachable"), queue=None)


def rec(key, title, sub, ident, family="Mohr", gender="female", street="Voorbeeldstraat 12"):
    return {"key": key, "title": title, "short": title, "sub": sub,
            "values": {"identifier": ident, "family": family, "given": "Alice", "born": "1958-01-30", "gender": gender,
                       "postal": "9999 AB", "street": street, "phone": MASKED_PHONE}}


PATIENT_FIELDS = [("identifier", "Identifier")] + [f for f in FIELDS if f[0] != "identifier"]


def patient_verdict(f, i):
    if f == "identifier":
        return None
    if i == 0:
        return "agree"
    return VERDICT_AB[f]


def page_patient_records():
    lead = rec("bsn", "BSN", "fed " + when("2026-10-20T09:41") + " · brp-feed", identifier("BSN", "123", reveal_id="reveal", show_domain=False))
    mrn = rec("mrn", "Hospital MRN", "fed " + when("2026-10-19T14:03") + " · deterministic", identifier("Hospital MRN", "994", reveal_id="reveal", show_domain=False))
    lab = rec("lab", "Laboratory", "fed " + when("2026-09-02T08:15") + " · steward link", identifier("Laboratory", "550", reveal_id="reveal", show_domain=False),
              family="Maiden", gender=None, street="Voorbeeldstraat 21")
    cmp_ = compare(
        "The source records side by side, each field marked against the BSN record",
        PATIENT_FIELDS, lead, [mrn, lab], patient_verdict, differ_first=False, shown=1,
        note="Marked against the BSN record, the leading record. The Hospital MRN record agrees in every field; the Laboratory record differs in 2 and lacks 1.",
    )
    return patient_page("records", card("Records, side by side", cmp_))


def change_del(v):
    return f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-sr-only">was </span><span class="fui-change-text">{v}</span></del>'


def change_ins(v):
    return f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span><span class="fui-sr-only">now </span>{v}</ins>'


def change_plain(before, after):
    return (f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-sr-only">was </span><span class="fui-change-text">{e(before)}</span></del> '
            f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span><span class="fui-sr-only">now </span>{e(after)}</ins>')


HISTORY_KINDS = [("link", 1), ("reveal", 1), ("view", 1), ("version", 3), ("auto-link", 1), ("merge", 1)]


def history_events():
    version_body = definitions([
        ("Postal code", change_plain("9999 AC", "9999 AB")),
        ("Street", change_plain("Voorbeeldstraat 10", "Voorbeeldstraat 12")),
        ("Source", "brp-feed · FHIR Patient update (ITI-104)"),
        ("Request id", "<code>01HZ7M3P1Q2R</code>"),
    ])
    return [
        {"at": "2026-10-21T09:42:10", "kind": "link", "tone": "ok", "actor": "Demo Steward",
         "title": "Linked the Laboratory record to this patient", "reason": "Maiden name; same telephone and address history",
         "body": disclosure("Candidate evidence", definitions([("Score", "0.91, probable"), ("Paired by", "postal code and birth date"), ("Candidate", "<code>cand-…3f9a</code>, opened 3 times")]))},
        {"at": "2026-10-21T09:41:55", "kind": "reveal", "tone": "warn", "actor": "Demo Steward",
         "title": "Revealed the BSN for 30 seconds", "reason": "Phone call with the laboratory about sample LAB-…550"},
        {"at": "2026-10-21T09:40:30", "kind": "view", "tone": "neutral", "actor": "Demo Steward", "title": "Opened this patient from the queue"},
        {"at": "2026-10-20T09:41", "kind": "version", "tone": "info", "actor": "feed · brp-feed",
         "title": "BSN record updated: the address changed", "body": disclosure("The version's fields", version_body, open_=True)},
        {"at": "2026-10-19T14:03", "kind": "auto-link", "tone": "info", "actor": "system",
         "title": "Hospital MRN record linked deterministically: the BSN agreed"},
        {"at": "2026-10-19T14:03", "kind": "version", "tone": "info", "actor": "feed · his-prod", "title": "Hospital MRN record created (HL7 v2 ADT A28)"},
        {"at": "2026-06-30T11:20", "kind": "merge", "tone": "warn", "actor": "Demo Steward",
         "title": f"Merged {PERSON_C_SHORT} into this patient: 1 identifier moved (Radiology RIS)", "reason": "Duplicate registration confirmed by the radiology desk",
         "body": '<p class="fui-small">Reversible: this is the latest merge. 2 subscriptions were told (ITI-93 merge).</p>'},
        {"at": "2025-11-05T07:58", "kind": "version", "tone": "info", "actor": "feed · brp-feed", "title": "BSN record created; this person minted"},
    ]


def history_filter(checked):
    allk = [k for k, _ in HISTORY_KINDS]
    word = "every kind but views" if set(checked) == {k for k in allk if k != "view"} else ", ".join(k for k in allk if k in checked)
    since = button("secondary", "Since 2025-11-05", "calendar", attrs='aria-haspopup="dialog" aria-expanded="false"')
    return f'<div class="fui-list-toolbar mk-history-toolbar"><div class="fui-list-toolbar-start">{tpl.filter_button("Kind", word)}{since}</div></div>'


def page_patient_history(filtered=False):
    events = history_events()
    checked = {"link", "merge", "auto-link"} if filtered else {k for k, _ in HISTORY_KINDS if k != "view"}
    shown = [ev for ev in events if ev["kind"] in checked]
    hidden = len(events) - len(shown)
    note = f'<p class="fui-small fui-muted">{len(shown)} of {len(events)} events, newest first · {hidden} hidden by the filter; views are off by default.</p>'
    chips = tpl.active(["Kind: link, merge, auto-link"]) if filtered else ""
    body = f'<div class="fui-stack">{history_filter(checked)}{chips}{note}{timeline("History of Mohr, Alice", shown)}</div>'
    return patient_page("history", body)


def page_patient_pseudonyms():
    rows = [
        ("ferroehr-nl", "Research export to FerroEHR", identifier("ferroehr-nl", "q7v", reveal_id="reveal", dots=10, show_domain=False), pill("ok", "present"), "2026-09-02"),
        ("research-2026-q3", "Cohort study, pseudonymised", identifier("research-2026-q3", "8kd", reveal_id="reveal", dots=10, show_domain=False), pill("ok", "present"), "2026-07-01"),
        ("dashboard-kpi", "Hospital KPI dashboard", '<span class="fui-muted">no pseudonym</span>', pill("neutral", "none"), None),
    ]
    body = [
        f'<tr><th scope="row">{e(n)}</th>' + td("Purpose", p) + td("Pseudonym", v) + td("State", s) + td("Minted", when(m) if m else '<span class="mk-dash">–</span>') + "</tr>" for n, p, v, s, m in rows
    ]
    t = titled_panel("Pseudonyms <span class=\"mk-count\">· one per domain</span>", button("secondary", "Resolve a pseudonym", "fingerprint", small=True),
                     table("Pseudonyms of Mohr, Alice", th("Domain") + th("Purpose") + th("Pseudonym") + th("State") + th("Minted"), body))
    return patient_page("pseudonyms", t)


def page_tabs_hover_focus():
    return patient_page("identifiers", identifiers_panel(), focus="history", hover="records")


def page_tabs_scrolled():
    script = "document.querySelector('.fui-tabs').scrollLeft = 400;"
    return patient_page("pseudonyms", page_patient_pseudonyms_panel(), scrolled=True, script=script)


def page_patient_pseudonyms_panel():
    html = page_patient_pseudonyms()
    a = html.index('<div class="fui-tab-panel">') + len('<div class="fui-tab-panel">')
    b = html.index("</div></div></main>")
    return html[a:b]


def page_tabs_states():
    def sheet(title, line, cur, focus=None, hover=None, counts=True, phone=False):
        items = PATIENT_TABS if counts else [t[:3] for t in PATIENT_TABS]
        body = '<p class="fui-small fui-muted mk-tab-demo-body">The panel\'s top edge runs under the inactive tabs and opens under the active one.</p>'
        cls = " mk-tab-phone" if phone else ""
        return (f'<div class="fui-card mk-state-frame{cls}"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                + tabbed(tabs("Example", items, cur, focus=focus, hover=hover, scrolled=phone), body) + "</div>")
    frames = (
        sheet("Rest", "the active tab raised and joined to its panel; the others behind on the sunken ground", "identifiers")
        + sheet("Another tab active", "History active; Identifiers steps back", "history")
        + sheet("Hover", "an inactive tab lifts halfway towards the panel's ground", "identifiers", hover="records")
        + sheet("Keyboard focus", "the ring inside the tab, never clipped by its neighbours", "identifiers", focus="history")
        + sheet("Without counts", "a tab shows a real count only; none is a shortcut", "records", counts=False)
        + sheet("Narrow, scrolled", "below 40 rem the row scrolls; the fades say there is more on either side", "pseudonyms", phone=True)
    )
    keys = card("Behaviour", definitions([
        ("Markup", "a <code>nav</code> of links with <code>aria-current=\"page\"</code> when each tab is a route, or <code>role=\"tablist\"</code> with roving focus when the tabs switch in place"),
        ("Keys", "Tab reaches the active tab; ← and → move between tabs in a tablist; Enter or Space selects"),
        ("Counts", "a quiet number for a real count; a shortcut is never drawn on a tab"),
        ("Dark", "the same three surfaces of the theme: the page, the sunken tabs, the raised sheet"),
    ]))
    content = page_heading("Tabs, states", subtitle="Binder tabs joined to their panel, for every tabbed page in every console: the patient, a domain, a configuration.") + f'<div class="fui-stack">{frames}{keys}</div>'
    return shell("Tabs states", "search", content)


def page_unmerge():
    rows = [
        (f"Radiology RIS {identifier('Radiology RIS', '077', static=True, show_domain=False)}", change_plain(f"Mohr, Alice · {PERSON_A_SHORT}", f"Mohr, A. · {PERSON_C_SHORT}")),
        (f"Person {chip(PERSON_C_SHORT)}", change_plain("subsumed, resolves to " + PERSON_A_SHORT, "active again")),
        ("Subscriptions told", '<span class="fui-muted">2 · ITI-93 unmerge notification</span>'),
        ("Candidates", '<span class="fui-muted">the pair is queued again for review</span>'),
    ]
    body = [f'<tr><th scope="row">{k}</th>' + td("Change", v) + "</tr>" for k, v in rows]
    merge_facts = definitions([
        ("Merged", f"{when('2026-06-30T11:20')} by Demo Steward"),
        ("Reason given", "Duplicate registration confirmed by the radiology desk"),
        ("Moved", "1 identifier: Radiology RIS"),
    ])
    content = (
        page_heading("Unmerge Mohr, A. from Mohr, Alice", subtitle="This is the latest merge of this patient, so it can be reverted. Nothing is written until you confirm.",
                     crumbs=[("Search", "search.html"), ("Mohr, Alice", "patient-identifiers.html")])
        + '<div class="fui-stack">' + card("The merge to revert", merge_facts)
        + card("What the unmerge changes", table("What the unmerge changes", th("What") + th("Change"), body)) + "</div>"
    )
    dk = dock(dock_field("unmerge-reason", "Reason", "Radiology desk: two different patients after all", "Why the merge is reverted"),
              link_button("secondary", "Cancel", "patient-identifiers.html") + button("danger", "Unmerge…", "undo"))
    return shell("Unmerge", "search", content, dock_html=dk)


# ---- merge: one confirmation (the page with its docked bar, then one short
# final dialog that does not repeat the table) ----

def merge_content():
    survivor = choice_group("survivor", "Who survives", [("a", "Mohr, Alice · older · 3 identifiers"), ("b", "Maiden, Alice · 1 identifier")], "a")

    def removed(v):
        return f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-sr-only">was </span><span class="fui-change-text">{v}</span></del>'

    def added(v):
        return f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span><span class="fui-sr-only">now </span>{v}</ins>'

    unset = '<span class="fui-muted">not set</span>'
    rows = [
        (f"Laboratory {identifier('Laboratory', '550', static=True, show_domain=False)}", removed(f"Maiden, Alice · {chip(PERSON_B_SHORT)}"), added(f"Mohr, Alice · {chip(PERSON_A_SHORT)}")),
        (f"Person {chip(PERSON_B_SHORT)}", removed("active"), added("subsumed, resolves to Mohr, Alice")),
        ("Pseudonym in ferroehr-nl", removed("resolves to Maiden, Alice"), added("resolves to Mohr, Alice")),
        ("Subscriptions told", unset, added("2 · ITI-93 merge notification")),
        ("Candidates closed", unset, added("1 · this pair, as linked")),
        ("Reversible until", unset, added("the next merge of either patient")),
    ]
    body = [f'<tr><th scope="row">{k}</th>' + td("Before", b) + td("After", a) + "</tr>" for k, b, a in rows]
    change = table("What the merge changes", th("What") + th("Before") + th("After"), body)
    return (
        page_heading("Merge Maiden, Alice into Mohr, Alice", subtitle="Nothing is written until you confirm. The table is the server's answer for this preview.",
                     crumbs=[("Search", "search.html"), ("Mohr, Alice", "patient-identifiers.html")])
        + '<div class="fui-stack">' + card("", survivor + '<p class="fui-help">The older patient keeps its id; every identifier of the other moves to it, and the other becomes subsumed.</p>')
        + card("What changes", change) + "</div>"
    )


def merge_dock(state="idle"):
    reason = "Maiden name; same telephone and address history"
    msg = ""
    acts = link_button("secondary", "Cancel", "patient-identifiers.html")
    if state == "submitting":
        acts = button("secondary", "Cancel", attrs="disabled") + busy_button("destructive", "Merging")
    elif state == "failed":
        msg = ('<div class="fui-dock-message">' + notice(
            "bad", "circle-alert",
            "Nothing was merged: Maiden, Alice changed at 09:44 today (a new Laboratory version), so this preview is out of date.",
            action=" " + button("secondary", "Preview again", "refresh-cw", small=True)) + "</div>")
        acts += button("danger", "Merge 1 identifier…", "merge", attrs="disabled")
    else:
        acts += button("danger", "Merge 1 identifier…", "merge")
    return dock(dock_field("merge-reason", "Reason", reason, "Why these are one patient", help_=None, disabled=state == "submitting"), acts, msg)


def page_merge():
    return shell("Merge", "search", merge_content(), dock_html=merge_dock())


def page_merge_confirm():
    dialog = (
        '<div id="merge-confirm" role="dialog" aria-labelledby="merge-confirm-title" class="fui-dialog fui-dialog-danger fui-modal mk-open mk-open-low" open>'
        f'<h2 id="merge-confirm-title" class="fui-dialog-title">{icon("triangle-alert", "18")} Merge Maiden, Alice into Mohr, Alice?</h2>'
        '<p>The merge is written at once and told to 2 subscribers. It can be reverted only until the next merge of either patient.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Go back")}{button("danger", "Merge", "merge")}</div></div>'
    )
    return shell("Merge", "search", merge_content(), dock_html=merge_dock(), overlay=dialog)


def page_merge_success():
    t = toasts(toast("ok", "circle-check", "Merged: Maiden, Alice is part of Mohr, Alice", "1 identifier moved; 2 subscribers told. Revert from History until the next merge."))
    return page_patient_identifiers_after_merge(t)


def page_patient_identifiers_after_merge(t):
    html = page_patient_identifiers(toast_html=t)
    return html


def page_merge_failure():
    return shell("Merge", "search", merge_content(), dock_html=merge_dock("failed"))


# ---- registry ----

DOMAINS = [
    ("BSN", "http://fhir.nl/fhir/NamingSystem/bsn", "authority", True, "brp-feed", "412,318", "2026-10-21T09:41", ("ok", "active")),
    ("Hospital MRN", "urn:oid:1.3.6.1.4.1.21367.2017.2.5.99", "local", False, "his-prod", "388,104", "2026-10-21T09:40", ("ok", "active")),
    ("Laboratory", "urn:oid:1.3.6.1.4.1.21367.2017.2.5.100", "local", False, "lab-lis", "96,220", "2026-10-21T08:15", ("ok", "active")),
    ("Radiology RIS", "urn:oid:1.3.6.1.4.1.21367.2017.2.5.101", "local", False, "ris-prod", "41,007", "2026-10-18T17:02", ("ok", "active")),
    ("Legacy ZIS", "urn:oid:1.3.6.1.4.1.21367.2017.2.5.42", "local", False, None, "12,330", "2024-02-01T00:00", ("neutral", "retired")),
]


def rel_time(iso):
    """A time in a list: relative today and within a month, the full timestamp as its tooltip."""
    if not iso:
        return '<span class="mk-dash">never</span>'
    import datetime as _dt
    d, t = iso.split("T")
    if d == "2026-10-21":
        return tpl.rel(t + (":00" if len(t) == 5 else ""))
    days = (_dt.date(2026, 10, 21) - _dt.date.fromisoformat(d)).days
    word = f"{days} days ago" if days <= 30 else d
    return f'<time class="fui-time" datetime="{iso}" title="{d} {t} CEST">{word}</time>'


def domain_rows(retired_ris=False):
    rows = []
    for n, s, k, sens, src, recs, fed, st in DOMAINS:
        if retired_ris and n == "Radiology RIS":
            st = ("neutral", "retired")
        rows.append(
            f'<tr><th scope="row"><a href="domain.html">{e(n)}</a></th>' + td("System", f"<code class=\"fui-break\">{e(s)}</code>")
            + td("Kind", badge("accent" if k == "authority" else "neutral", k)) + td("Sensitive", (icon("shield-check", "14") + " yes") if sens else "no")
            + td("Source", f'<a href="source.html"><code>{src}</code></a>' if src else '<span class="mk-dash">–</span>') + td("Records", recs, "fui-num") + td("Last fed", rel_time(fed)) + td("State", pill(*st)) + "</tr>"
        )
    return rows


def page_domains():
    t = titled_panel(
        "Domains <span class=\"mk-count\">· 5, 4 active</span>", "",
        table("Identifier domains", th("Name", "name", "ascending") + th("System") + th("Kind") + th("Sensitive") + th("Source") + th("Records", num=True) + th("Last fed") + th("State"), domain_rows()),
    )
    content = page_heading("Identifier domains", actions=link_button("primary", "New domain", "domain-new.html", "plus"),
                           subtitle="The assigning authorities, each with its one Source and its feed activity. A sensitive domain is masked everywhere and revealed with a reason.") + f'<div class="fui-stack">{t}</div>'
    return shell("Identifier domains", "domains", content)


def domain_feeds():
    feeds = [
        ("09:41:58", "FHIR", "ITI-104 update", ("ok", "accepted"), "01HZ7M3P1Q2R"), ("09:41:12", "FHIR", "ITI-104 add", ("ok", "accepted"), "01HZ7M2K9W1A"),
        ("09:40:47", "FHIR", "ITI-104 update", ("ok", "accepted"), "01HZ7M1Q2E5T"), ("09:39:30", "FHIR", "ITI-104 add", ("warn", "queued for review"), "01HZ7LZ77N3B"),
        ("09:38:05", "FHIR", "ITI-104 update", ("ok", "accepted"), "01HZ7LY4C8DK"), ("09:36:51", "FHIR", "ITI-104 add", ("bad", "refused: no birth date"), "01HZ7LX0B6FJ"),
    ]
    rows = [
        f'<tr><th scope="row">{when("2026-10-21T" + t, seconds=True)}</th>' + td("Channel", tag(ch)) + td("Message", m) + td("Outcome", status(o[0], "circle-check" if o[0] == "ok" else ("clock" if o[0] == "warn" else "circle-x"), o[1]))
        + td("Request id", f"<code>{r}</code>") + "</tr>"
        for t, ch, m, o, r in feeds
    ]
    return titled_panel("Last feeds <span class=\"mk-count\">· never a value</span>", "", table("The last feeds", th("Time") + th("Channel") + th("Message") + th("Outcome") + th("Request id"), rows))


def page_domain(name="BSN", retired=False, overlay=None, toast_html=""):
    activity = [310, 280, 240, 260, 290, 320, 300, 270, 250, 230, 260, 290, 310, 330, 300, 280, 260, 240, 250, 270, 300, 320, 310, 290, 270, 250, 260, 280, 300, 311]
    if name == "BSN":
        facts = facts_row([
            ("System", "<code>http://fhir.nl/fhir/NamingSystem/bsn</code>"), ("Kind", badge("accent", "authority")), ("Sensitive", "yes, masked everywhere"),
            ("Source", "<code>brp-feed</code>"), ("Records", "412,318"), ("State", pill("ok", "active")), ("Created", "2025-11-01"),
        ])
        sub = "Identifier domain · the Dutch citizen service number, fed by the BRP."
        actions = action_group(link_button("primary", "Edit", "domain-edit.html", "pencil"), [menu_item("Retire…", "archive", href="domain-retire.html", danger=True)], label=f"More actions for {name}")
    else:
        st = pill("neutral", "retired") if retired else pill("ok", "active")
        facts = facts_row([
            ("System", "<code>urn:oid:1.3.6.1.4.1.21367.2017.2.5.101</code>"), ("Kind", badge("neutral", "local")), ("Sensitive", "no"),
            ("Source", "<span class=\"mk-dash\">–</span> (none while retired)" if retired else "<code>ris-prod</code>"), ("Records", "41,007"), ("State", st), ("Created", "2025-11-01"),
        ])
        sub = "Identifier domain · the radiology information system's patient numbers."
        actions = (action_group(button("primary", "Reinstate…", "rotate-ccw"), []) if retired else
                   action_group(link_button("primary", "Edit", "domain-edit.html", "pencil"), [menu_item("Retire…", "archive", href="domain-retire.html", danger=True)], label=f"More actions for {name}"))
    notice_html = ""
    if retired:
        notice_html = notice("neutral", "archive", f"Retired {when('2026-10-21T09:47')} by Demo Steward: “RIS replaced by the new PACS numbering.” Feeds to this domain are refused; its 41,007 records stay and keep answering lookups.")
    content = (
        page_heading(name, actions=actions, subtitle=sub, crumbs=[("Identifier domains", "domains.html")])
        + '<div class="fui-stack">' + notice_html + card("Definition", facts)
        + '<div class="fui-grid fui-grid-start">' + card("Feeds per day, 30 days", line_chart("feed-activity", "Feeds per day", "Between 230 and 330 feeds a day.", [("Feeds", activity, 0)], [(0, "21 Sep"), (15, "6 Oct"), (29, "Today")], height=160, legend=False))
        + card("Outcomes, today", definitions([("Accepted", "1,106"), ("Queued for review", "38"), ("Refused", "3 <span class=\"fui-muted\">· 2 without a birth date, 1 unknown system</span>"), ("p99 latency", "11 ms")]))
        + "</div>" + domain_feeds() + "</div>"
    )
    return shell(name, "domains", content, overlay=overlay, toast_html=toast_html)


def page_domain_retire():
    dialog = (
        '<div role="dialog" aria-labelledby="retire-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="retire-title" class="fui-dialog-title">{icon("archive", "18")} Retire Radiology RIS?</h2>'
        '<p>From then on a feed to this domain is refused, and no new record is accepted in it. Its 41,007 records stay, keep answering lookups and keep their links. A retired domain can be reinstated.</p>'
        + field("retire-reason", "Reason", textarea("retire-reason", "RIS replaced by the new PACS numbering"), help_="Required. Recorded in the audit trail.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel")}{button("danger", "Retire the domain", "archive")}</div></div>'
    )
    return page_domain("Radiology RIS", overlay=dialog)


def page_domain_retired():
    t = toasts(toast("ok", "circle-check", "Radiology RIS is retired", "Recorded in the audit trail with your reason."))
    return page_domain("Radiology RIS", retired=True, toast_html=t)


def domain_form(new=True, invalid=False):
    if new:
        name = field("d-name", "Name", input_("d-name", "Dental clinic"))
        system = field("d-system", "System", input_("d-system", "urn:oid:1.3.6.1.4.1.21367.2017.2.5.102", extra='spellcheck="false"'),
                       help_="The URI every feed and lookup names. It cannot change once a record exists.")
        kind = choice_group("d-kind", "Kind", [("local", "Local"), ("authority", "Authority")], "local")
        kind += '<p class="fui-help">An authority\'s identifier links records across domains without review; a local one never does.</p>'
        src = field("d-src", "Source <span class=\"fui-muted\">(required)</span>", select("d-src", [("dental-ehr", "dental-ehr · HL7 v2"), ("brp-feed-2", "brp-feed-2 · FHIR")], "dental-ehr", extra="required"),
                    help_='The one source that may feed this domain, picked from <a href="sources.html">Sources</a>. Set the source up first.')
    else:
        name = field("d-name", "Name", input_("d-name", "BSN"))
        system = field("d-system", "System", input_("d-system", "http://fhir.nl/fhir/NamingSystem/bsn", extra="disabled"),
                       help_=f"{icon('lock', '12')} Fixed: 412,318 records carry it.")
        kind = field("d-kind", "Kind", input_("d-kind", "authority", extra="disabled"), help_=f"{icon('lock', '12')} Fixed while records exist.")
        src = field("d-src", "Source", select("d-src", [("brp-feed", "brp-feed"), ("brp-feed-2", "brp-feed-2 · client credentials")], "brp-feed-2"),
                    help_='The one source that may feed this domain, picked from <a href="sources.html">Sources</a>. Set the source up first.')
    sens = check("d-sens", "Sensitive", not new or False, "Masked everywhere and revealed only with a reason.", extra='class="fui-switch"')
    return f'<div class="fui-form mk-edit-form"><div class="fui-columns">{name}{system}</div><div class="fui-columns">{kind}{src}</div>{sens}</div>'


def page_domain_new_no_sources():
    empty = empty_state("radio-tower", "Add the source that feeds this domain first",
                        "Every identifier domain names the one source that feeds it, and no source is set up yet.",
                        link_button("primary", "New source", "source-new.html", "plus"))
    name = field("d-name", "Name", input_("d-name", "Dental clinic"))
    system = field("d-system", "System", input_("d-system", "urn:oid:1.3.6.1.4.1.21367.2017.2.5.102", extra='spellcheck="false"'))
    form = f'<div class="fui-form mk-edit-form"><div class="fui-columns">{name}{system}</div><div class="fui-field"><span class="fui-label">Source <span class="fui-muted">(required)</span></span>{empty}</div></div>'
    content = (
        page_heading("New identifier domain", subtitle="A domain is a database record: created here, recorded in the audit trail, and live at once.", crumbs=[("Identifier domains", "domains.html")])
        + '<div class="fui-stack">' + card("Definition", form) + "</div>"
    )
    dk = dock(dock_field("d-reason", "Reason", "", "Why this domain is created"),
              link_button("secondary", "Cancel", "domains.html") + button("primary", "Create the domain", "plus", attrs="disabled"))
    return shell("New identifier domain", "domains", content, dock_html=dk)


def page_dashboard_empty():
    body = empty_state("radio-tower", "Nothing is fed yet",
                       "Set the index up in this order: add a source, then the identifier domain it feeds, then check the matching thresholds. Records arrive with the source's first feed.",
                       link_button("primary", "New source", "source-new.html", "plus") + link_button("secondary", "Configuration", "configuration.html", "sliders-horizontal"))
    steps = ('<ol class="mk-setup-steps"><li><strong>Sources</strong><span>who may feed, and how each connects</span></li>'
             '<li><strong>Identifier domains</strong><span>each names its one source</span></li>'
             '<li><strong>Matching</strong><span>thresholds and weights</span></li></ol>')
    content = dashboard_heading() + f'<div class="fui-stack">{body}{card("Setup order", steps)}</div>'
    return shell("Dashboard", "dashboard", content, queue=0)


def page_domain_new():
    content = (
        page_heading("New identifier domain", subtitle="A domain is a database record: created here, recorded in the audit trail, and live at once. No restart.", crumbs=[("Identifier domains", "domains.html")])
        + '<div class="fui-stack">' + card("Definition", domain_form(True)) + "</div>"
    )
    dk = dock(dock_field("d-reason", "Reason", "Dental clinic joins the index in November", "Why this domain is created"),
              link_button("secondary", "Cancel", "domains.html") + button("primary", "Create the domain", "plus"))
    return shell("New identifier domain", "domains", content, dock_html=dk)


def page_domain_edit():
    diff = table("What changes", th("Setting") + th("Running") + th("After the save"),
                 [f'<tr><th scope="row">Source</th>' + td("Running", change_del("<code>brp-feed</code>")) + td("After the save", change_ins("<code>brp-feed-2</code>")) + "</tr>"])
    content = (
        page_heading("Edit BSN", subtitle="Your changes are a draft until you save them with a reason. A domain change is live at once.", crumbs=[("Identifier domains", "domains.html"), ("BSN", "domain.html")])
        + '<div class="fui-stack">' + revision_strip("draft", "Draft of BSN · 1 change", "Based on what runs now. Nothing changes until you save.")
        + card("Definition", domain_form(False))
        + card("What changes <span class=\"mk-count\">· 1</span>", diff + '<p class="fui-small fui-muted">From the save on, a feed from brp-feed is refused for this domain and brp-feed-2 is admitted. <span class="fui-effect fui-effect-live">' + icon("circle-check", "12") + "Live at once</span></p>")
        + "</div>"
    )
    dk = dock(dock_field("d-reason", "Reason", "BRP connection moves to the new client registration", "Why this changes"),
              link_button("secondary", "Discard the draft", "domain.html") + button("primary", "Save the change", "check"))
    return shell("Edit BSN", "domains", content, dock_html=dk)


def page_pseudonym_domains():
    rows_data = [
        ("ferroehr-nl", "Research export to FerroEHR", "keyed hash (HMAC-SHA-256)", "231,440", "2025-11-12", ("ok", "active")),
        ("research-2026-q3", "Cohort study, pseudonymised", "keyed hash (HMAC-SHA-256)", "18,902", "2026-07-01", ("ok", "active")),
        ("dashboard-kpi", "Hospital KPI dashboard", "random, stored", "402,110", "2025-12-03", ("ok", "active")),
        ("pilot-2025", "Pilot export, closed", "random, stored", "4,018", "2025-06-10", ("neutral", "retired")),
    ]
    rows = [
        f'<tr><th scope="row"><a href="pseudonym-domain-edit.html">{e(n)}</a></th>' + td("Purpose", p) + td("Method", m) + td("Pseudonyms", c, "fui-num") + td("Created", when(cr)) + td("State", pill(*st)) + "</tr>"
        for n, p, m, c, cr, st in rows_data
    ]
    resolve = card(
        "Resolve a pseudonym",
        '<form method="post" action="#" class="fui-form"><div class="fui-form-grid">'
        + field("ns", "Domain", select("ns", [("ferroehr-nl", "ferroehr-nl"), ("research-2026-q3", "research-2026-q3"), ("dashboard-kpi", "dashboard-kpi")], "ferroehr-nl"))
        + field("pseudonym", "Pseudonym", input_("pseudonym", "", extra='spellcheck="false" autocapitalize="off"'))
        + field("resolve-reason", "Reason", input_("resolve-reason", ""), cls="fui-span-2")
        + "</div>"
        + f'<div class="fui-form-foot"><p class="fui-help">The pseudonym is shown as you type it: on its own it names no one, and you need to check what you pasted. The answer is the disclosure, recorded with your reason.</p>{button("primary", "Resolve", "fingerprint", type_="submit")}</div></form>',
    )
    t = titled_panel("Domains <span class=\"mk-count\">· 4, 3 active</span>", "",
                     table("Pseudonymisation domains", th("Namespace", "ns", "ascending") + th("Purpose") + th("Method") + th("Pseudonyms", num=True) + th("Created") + th("State"), rows))
    content = page_heading("Pseudonymisation domains", actions=link_button("primary", "New domain", "pseudonym-domain-new.html", "plus"), subtitle="The namespaces a pseudonym is minted in, and their counts. A resolution is a disclosure.") + f'<div class="fui-stack">{t}{resolve}</div>'
    return shell("Pseudonymisation domains", "pseudonym-domains", content)


def page_pseudonym_domain_edit():
    form = (
        '<div class="fui-form mk-edit-form"><div class="fui-columns">'
        + field("p-ns", "Namespace", input_("p-ns", "research-2026-q3", extra="disabled"), help_=f"{icon('lock', '12')} Fixed: 18,902 pseudonyms are minted in it.")
        + field("p-method", "Method", input_("p-method", "keyed hash (HMAC-SHA-256)", extra="disabled"), help_=f"{icon('lock', '12')} Fixed: a new method is a new domain.")
        + "</div>"
        + field("p-purpose", "Purpose", input_("p-purpose", "Cohort study, pseudonymised; closes 2027-06-30"))
        + choice_group("p-state", "State", [("active", "Active"), ("retired", "Retired")], "active")
        + "</div>"
    )
    diff = table("What changes", th("Setting") + th("Running") + th("After the save"),
                 [f'<tr><th scope="row">Purpose</th>' + td("Running", change_del("Cohort study, pseudonymised")) + td("After the save", change_ins("Cohort study, pseudonymised; closes 2027-06-30")) + "</tr>"])
    content = (
        page_heading("Edit research-2026-q3", subtitle="Pseudonymisation domain · a database record, live at once when saved.", crumbs=[("Pseudonymisation domains", "pseudonym-domains.html")])
        + '<div class="fui-stack">' + revision_strip("draft", "Draft of research-2026-q3 · 1 change", "Nothing changes until you save.")
        + card("Definition", form) + card("What changes <span class=\"mk-count\">· 1</span>", diff) + "</div>"
    )
    dk = dock(dock_field("p-reason", "Reason", "Study end date agreed with the DPO", "Why this changes"),
              link_button("secondary", "Discard the draft", "pseudonym-domains.html") + button("primary", "Save the change", "check"))
    return shell("Edit research-2026-q3", "pseudonym-domains", content, dock_html=dk)


# ---- configuration: from the file and from the console (review item 6) ----
# The core always stays in ferropix.toml. Everything else lives in one
# place per deployment, chosen by [config] mode: "file" (the console reads
# it, read-only) or "console" (each change a draft, checked, diffed against
# what runs, applied with a reason as a revision, live where it can be, the
# restart named where not, 409 when stale). The two never mix.

def effect(kind):
    if kind == "live":
        return f'<span class="fui-effect fui-effect-live">{icon("circle-check", "12")}Live at once</span>'
    return f'<span class="fui-effect fui-effect-restart">{icon("refresh-cw", "12")}At the next start</span>'


def revision_strip(state, title, line, actions=""):
    tone = {"running": "", "draft": " fui-revision-draft", "ok": " fui-tone-ok", "warn": " fui-tone-warn", "bad": " fui-tone-bad"}[state]
    glyph = {"running": "file-text", "draft": "pencil", "ok": "circle-check", "warn": "refresh-cw", "bad": "triangle-alert"}[state]
    role = ' role="status"' if state in ("ok", "warn", "bad") else ""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    return f'<div class="fui-revision{tone}"{role}>{icon(glyph, "18")}<div class="fui-revision-text"><strong>{title}</strong><span>{line}</span></div>{a}</div>'


THRESH = [("possible_at", "Possible from", "0.60", "a candidate, below the probable ones"),
          ("review_at", "Probable from", "0.80", "a candidate, the likeliest first"),
          ("link_at", "Certain from", "0.95", "a candidate at the top; linked alone only with auto_link")]

WEIGHTS = [
    ("family", "Family name", "Jaro-Winkler, whole from 0.92, half from 0.85", "+5.1", "−3.1"),
    ("given", "Given names", "Jaro-Winkler; initials agree: half", "+4.2", "−2.4"),
    ("born", "Birth date", "exact; transposed or one digit off: half", "+6.0", "−5.0"),
    ("gender", "Gender", "exact; absent on either side: no weight", "+0.9", "−2.0"),
    ("postal", "Postal code", "exact; the first four characters: half", "+2.6", "−0.8"),
    ("street", "Address line", "token overlap", "+2.2", "−0.4"),
    ("phone", "Telephone", "exact after normalisation (E.164)", "+3.9", "−1.2"),
]


def value_cell(v, mode, key, draft=None, error=None, sign=""):
    if mode == "view":
        return v
    nv = draft.get(key, v) if draft else v
    changed = nv != v
    inv = ' aria-invalid="true"' if error and key in error else ""
    cls = "fui-input" + (" fui-config-changed" if changed else "")
    was = f'<span class="fui-config-was">was {v}</span>' if changed else ""
    er = f'<span class="fui-config-error" id="{key}-error">{error[key]}</span>' if error and key in error else ""
    return f'<span class="fui-config-input"><input class="{cls}" id="cfg-{key}" value="{e(nv)}" inputmode="decimal" aria-label="{e(key)}"{inv} />{was}{er}</span>'


def thresholds_card(mode="view", draft=None, error=None):
    rows = ""
    for key, label, v, does in THRESH:
        rows += (f'<tr><th scope="row">{label}<span class="fui-small fui-muted mk-key"><code>{key}</code></span></th>'
                 + td("Value", value_cell(v, mode, key, draft, error), "fui-num") + td("Means", does) + "</tr>")
    rows += (f'<tr><th scope="row">Link alone<span class="fui-small fui-muted mk-key"><code>auto_link</code></span></th>'
             + td("Value", "off", "fui-num") + td("Means", "a certain pair still waits for a steward") + "</tr>")
    vals = {k: float((draft or {}).get(k, v)) for k, _, v, _ in THRESH}
    try:
        sc = score_scale(None, "The thresholds on the scale", [("possible", vals["possible_at"]), ("probable", vals["review_at"]), ("certain", vals["link_at"])]) if not error else ""
    except Exception:
        sc = ""
    t = table("Thresholds", th("Grade") + th("Score", num=True) + th("Means"), [rows], stack=True)
    head_actions = effect("live")
    return card("Thresholds", t + sc + '<p class="fui-small fui-muted">A change re-grades the open queue at once; a pair under possible_at leaves the queue and comes back when either record changes.</p>', actions=head_actions)


def weights_card(mode="view", draft=None):
    rows = []
    for key, label, comp, agree, dis in WEIGHTS:
        a = value_cell(agree, mode, key + ".agree", draft)
        d = value_cell(dis, mode, key + ".disagree", draft)
        if mode == "view":
            a = f'<span class="fui-score-weight fui-score-add">{agree}</span>'
            d = f'<span class="fui-score-weight fui-score-remove">{dis}</span>'
        rows.append(f'<tr><th scope="row">{label}</th>' + td("Comparator", comp) + td("Agree", a, "fui-num") + td("Disagree", d, "fui-num") + "</tr>")
    t = table("Field weights", th("Field") + th("Comparator") + th("Agree", num=True) + th("Disagree", num=True), rows)
    return card("Field weights <span class=\"mk-count\">· log2(m / u) and log2((1 − m) / (1 − u))</span>", t, actions=effect("live"))


def blocking_card(mode="view", added=False):
    keys = [
        ("Birth year and the family name's phonetic key", True),
        ("Postal code and birth date", True),
        ("An identifier value shared across domains", True),
    ]
    items = ""
    for k, on in keys:
        if mode == "view":
            items += f"<li>{k}</li>"
        else:
            items += f'<li>{check("bk-" + str(len(items)), k, on)}</li>'
    if added:
        items += '<li>Given name\'s phonetic key and birth date <span class="fui-badge fui-tone-warn">added · waits for the restart</span></li>'
    lst = f'<ul class="{"fui-bullets" if mode == "view" else "mk-plain-list"}">{items}</ul>'
    return card("Blocking keys", lst + '<p class="fui-small fui-muted">A pair is scored only when it shares a key. A change re-indexes the key columns, so it takes the next start.</p>',
                actions=effect("restart"))


def rules_card():
    return card("Deterministic rules", '<ul class="fui-bullets"><li>A record whose <strong>BSN</strong> agrees with an active record links to it without review, unless the birth year or a stated gender conflicts.</li>'
                '<li>A local identifier never links across domains: two hospitals that both number a patient 12345 say nothing.</li>'
                '<li>A feed\'s own replaced-by is a link by definition.</li></ul>', actions='<span class="fui-small fui-muted">Built in, not configurable</span>')


CFG_TABS = [("thresholds", "Thresholds", "configuration.html"), ("weights", "Field weights", "configuration-weights.html"),
            ("blocking", "Blocking keys", "configuration-blocking.html"), ("rules", "Rules", "configuration-rules.html"),
            ("history", "History", "configuration-history.html")]
CFG_INTRO = {
    "thresholds": "Where a pair's score puts it in the queue. Changes apply at once and re-grade the open queue.",
    "weights": "What each field adds when it agrees and takes away when it disagrees, as log2(m / u) and log2((1 − m) / (1 − u)). Changes apply at once.",
    "blocking": "Which records are compared at all. Changes take the next start, after the key columns are re-indexed.",
    "rules": "The links the server makes without scoring. Built in, not configurable.",
}


def matching_heading(actions, mode="console"):
    return configuration_heading(mode)


def matching_view_strip():
    return revision_strip("running", "Revision 7 runs", f"Applied {when('2026-10-14T11:05')} by Demo Steward: “Raise review_at after the September audit.”")


def compact_strip(n="7", at="2026-10-14", by="Demo Steward", reason="Raise review_at after the September audit"):
    return (f'<div class="fui-revision mk-revision-compact">{icon("file-text", "16")}<p><strong>Revision {n}</strong> · applied {at} by {by} · '
            f'<span class="fui-pill fui-tone-ok" title="{e("Running since " + at + ": " + reason)}">in step</span></p></div>')


def key_hint(k, show=False):
    return f'<code class="mk-key-hint{" is-shown" if show else ""}" title="Copy the key">{e(k)}{icon("copy", "12")}</code>'


def srow(label, desc, value, key, restart=False, control=None, edit=False, error=None, changed=None):
    nxt = f'<span class="fui-effect fui-effect-restart" title="Takes effect at the next start">{icon("refresh-cw", "12")}Next start</span>' if restart else ""
    ctl = control if edit and control else f'<span class="fui-setting-value">{value}</span>'
    er = f'<p class="fui-help fui-error mk-row-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    was = f'<span class="fui-config-was">was {e(changed)}</span>' if changed else ""
    return (f'<div class="fui-setting-row{" is-changed" if changed else ""}{" is-invalid" if error else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}{key_hint(key)}</span>'
            f'<p class="fui-setting-desc">{desc}</p>{er}</div><div class="fui-setting-control">{ctl}{nxt}{was}</div></div>')


def num_input(key, v, changed=False, invalid=False):
    cls = "fui-input" + (" fui-config-changed" if changed else "")
    inv = f' aria-invalid="true" aria-describedby="{key}-error"' if invalid else ""
    return f'<input class="{cls} mk-num-input" id="cfg-{key}" value="{e(v)}" inputmode="decimal" aria-label="{e(key)}"{inv} />'


THRESH_DESC = {"possible_at": "The lowest score that makes a pair a candidate, below the probable ones.",
               "review_at": "From here a candidate is probable; the likeliest come first.",
               "link_at": "From here a candidate is certain; it is linked alone only with Link alone on."}


def config_rows(tab, mode="view", draft=None, error=None, blocking_added=False):
    edit = mode == "edit"
    draft = draft or {}
    error = error or {}
    if tab == "thresholds":
        out = ""
        for key, label, v, _ in THRESH:
            nv = draft.get(key, v)
            ch = nv != v
            out += srow(label, THRESH_DESC[key], v, "matching." + key, control=num_input(key, nv, ch, key in error), edit=edit,
                        error=error.get(key), changed=v if edit and ch else None)
        out += srow("Link alone", "Link a certain pair without a steward.", "off", "matching.auto_link")
        vals = {k: float(draft.get(k, v)) for k, _, v, _ in THRESH}
        scale = "" if error else score_scale(None, "The thresholds on the scale", [("possible", vals["possible_at"]), ("probable", vals["review_at"]), ("certain", vals["link_at"])])
        return out, f'<div class="fui-panel-body">{scale}</div>' if scale else ""
    if tab == "weights":
        out = ""
        for key, label, comp, agree, dis in WEIGHTS:
            val = f'<span class="fui-score-weight fui-score-add">{agree}</span> agree · <span class="fui-score-weight fui-score-remove">{dis}</span> disagree'
            out += srow(label, comp[0].upper() + comp[1:] + ".", val, f"matching.fields.{key}")
        return out, ""
    if tab == "blocking":
        keys = [("birth_year_family", "Birth year and the family name's phonetic key"), ("postal_birth", "Postal code and birth date"), ("shared_identifier", "An identifier value shared across domains")]
        out = "".join(srow(w, "A pair is scored only when it shares this key.", "on", f"matching.blocking[{i}]", restart=True,
                           control=f'<input type="checkbox" class="fui-switch" checked aria-label="{e(w)}" />', edit=edit) for i, (_, w) in enumerate(keys))
        if blocking_added:
            out += srow("Given name's phonetic key and birth date", "Added in revision 8; the key columns are re-indexed at the next start.", '<span class="fui-badge fui-tone-warn">added, waits</span>', "matching.blocking[3]", restart=True)
        return out, ""
    rules = [("BSN agrees", "A record whose BSN agrees with an active record links to it without review, unless the birth year or a stated gender conflicts."),
             ("Local identifiers", "A local identifier never links across domains: two hospitals that both number a patient 12345 say nothing."),
             ("Replaced by", "A feed's own replaced-by is a link by definition.")]
    return "".join(srow(w, d, "built in", f"matching.rules.{w.split()[0].lower()}") for w, d in rules), ""


def config_tab_panel(tab, mode="view", draft=None, error=None, blocking_added=False, config_mode="console", top=""):
    rows, below = config_rows(tab, mode, draft, error, blocking_added)
    intro = CFG_INTRO[tab]
    if mode == "edit":
        n = len([k for k in (draft or {}) if (draft or {}).get(k) != dict((x[0], x[2]) for x in THRESH).get(k)])
        intro = f"A draft of this tab, based on revision 7: {n} changes. Nothing runs until you apply it."
        act = '<span class="fui-pill fui-tone-accent">editing</span>'
    elif config_mode == "file" or tab == "rules":
        act = ""
    else:
        act = link_button("secondary", "Edit", "matching-edit.html" if tab == "thresholds" else "#", "pencil", small=True)
    return (
        f'<div class="fui-settings-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{intro}</p><div class="fui-actions">{act}</div></div>'
        f'{top}<div class="fui-setting-rows">{rows}</div>{below}</div>')


def config_page(tab, strip, panel, dock_html="", overlay=None, toast_html="", extra_top="", below="", config_mode="console", drafts=(), running="7"):
    if ("fui-revision-draft" in strip or 'class="fui-revision"' in strip) and "mk-revision-compact" not in strip:
        strip = (compact_strip(running) if running == "7" else compact_strip(running, "2026-10-21", "Demo Steward", "Fewer weak candidates after the October review")) if config_mode == "console" else compact_strip(running, "2026-10-21", "a reading of ferropix.toml", "Read after a change to the file")
    content = (configuration_heading(config_mode, running) + f'<div class="fui-stack">{strip}{extra_top}'
               + settings_tabs("Configuration sections", CFG_TABS, tab, panel, drafts=drafts) + below + "</div>")
    return shell("Configuration", "configuration", content, dock_html=dock_html, overlay=overlay, toast_html=toast_html)


def matching_page(strip, mode="view", draft=None, error=None, dock_html="", overlay=None, actions=None, toast_html="", extra_top="", blocking_added=False, below="", config_mode="console", tab=None):
    tab = tab or ("blocking" if blocking_added else "thresholds")
    panel = config_tab_panel(tab, mode, draft, error, blocking_added, config_mode, top=extra_top)
    return config_page(tab, strip, panel, dock_html, overlay, toast_html, below, "", config_mode, drafts=("thresholds",) if mode == "edit" else ())


def review_panel(intro, body):
    return (f'<div class="fui-settings-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{intro}</p></div>'
            f'<div class="fui-panel-body fui-stack">{body}</div></div>')


def page_configuration_weights():
    return matching_page(matching_view_strip(), tab="weights")


def page_configuration_blocking():
    return matching_page(matching_view_strip(), tab="blocking")


def page_configuration_rules():
    return matching_page(matching_view_strip(), tab="rules")


def page_matching():
    return matching_page(matching_view_strip())


DRAFT = {"review_at": "0.82", "possible_at": "0.65"}


def edit_dock(primary="Review the changes", reason="", reason_label="Reason", disabled=False, msg=""):
    fld = dock_field("cfg-reason", reason_label, reason, "Why this changes; kept with the revision")
    acts = button("secondary", "Discard the draft", "x") + button("primary", primary, "git-compare", attrs="disabled" if disabled else "")
    return dock(fld, acts, msg)


def page_matching_edit():
    strip = revision_strip("draft", "Draft based on revision 7 · 2 changes", "Nothing runs until you apply it.")
    return matching_page(strip, "edit", DRAFT, dock_html=edit_dock(reason="Fewer weak candidates after the October review"), actions="")


def page_matching_invalid():
    draft = {"review_at": "0.82", "possible_at": "0.85"}
    err = {"possible_at": "Must be below Probable from (0.82)."}
    strip = revision_strip("draft", "Draft based on revision 7 · 2 changes", "Nothing runs until you apply it.")
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 value needs a change before you can review</p>'
               '<ul><li><a href="#cfg-possible_at">Possible from: must be below Probable from (0.82)</a></li></ul></div>')
    return matching_page(strip, "edit", draft, err, dock_html=edit_dock(reason="Fewer weak candidates after the October review", disabled=True), actions="", extra_top=summary)


def diff_table(rows, right="Draft"):
    body = [
        f'<tr><th scope="row"><code>{k}</code></th>' + td("Revision 7, running", f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-change-text">{a}</span></del>')
        + td(right, f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span>{b}</ins>') + td("Takes effect", effect(fx)) + "</tr>"
        for k, a, b, fx in rows
    ]
    return table("What the draft changes", th("Setting") + th("Revision 7, running") + th(right) + th("Takes effect"), body)


DIFF_ROWS = [("matching.review_at", "0.80", "0.82", "live"), ("matching.possible_at", "0.60", "0.65", "live")]


def diff_card(rows=DIFF_ROWS, right="Draft"):
    effect_text = ('<p class="fui-small">What applying does to the open queue, as the server computed it: '
                   '<strong>3 candidates</strong> move from probable to possible, and <strong>2</strong> fall under 0.65 and leave the queue. They are kept, and return when either record changes or the threshold drops.</p>')
    return card(f"What the draft changes <span class=\"mk-count\">· {len(rows)} settings</span>", diff_table(rows, right) + effect_text)


def diff_body(rows=None):
    rows = rows or DIFF_ROWS
    return (diff_table(rows) + '<p class="fui-small">What applying does to the open queue, as the server computed it: '
            '<strong>3 candidates</strong> move from probable to possible, and <strong>2</strong> fall under 0.65 and leave the queue. They are kept, and return when either record changes or the threshold drops.</p>')


def review_dk():
    return dock(dock_field("cfg-reason", "Reason", "Fewer weak candidates after the October review", "Why this changes; kept with the revision"),
                link_button("secondary", "Back to editing", "matching-edit.html", "pencil") + button("primary", "Apply as revision 8…", "check"))


def page_matching_diff():
    panel = review_panel("Your draft against revision 7, checked the way the server checks its file: no problems.", diff_body())
    return config_page("thresholds", compact_strip(), panel, dock_html=review_dk(), drafts=("thresholds",))


def page_matching_apply():
    panel = review_panel("Your draft against revision 7, checked the way the server checks its file: no problems.", diff_body())
    dialog = (
        '<div role="dialog" aria-labelledby="apply-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="apply-title" class="fui-dialog-title">{icon("check", "18")} Apply revision 8?</h2>'
        '<p>Both changes are live at once: new scores use them, and the open queue is re-graded (3 candidates move to possible, 2 leave the queue).</p>'
        + well('<p class="fui-small"><strong>Reason:</strong> Fewer weak candidates after the October review</p><p class="fui-small fui-muted">Recorded in the audit trail with your name. Revision 7 stays in the history and can be restored in one step.</p>')
        + f'<div class="fui-dialog-actions">{button("secondary", "Go back")}{button("primary", "Apply revision 8", "check")}</div></div>'
    )
    return config_page("thresholds", compact_strip(), panel, dock_html=review_dk(), overlay=dialog, drafts=("thresholds",))


def page_matching_applied():
    strip = revision_strip("ok", "Revision 8 runs", f"Applied {when('2026-10-21T09:58')} by you: “Fewer weak candidates after the October review.” The queue was re-graded: 36 candidates wait.",
                           link_button("ghost", "Revision 8 in History", "configuration-history.html", "history", small=True))
    t = toasts(toast("ok", "circle-check", "Revision 8 applied", "Live at once; recorded in the audit trail."))
    draft = {"review_at": "0.82", "possible_at": "0.65"}
    global THRESH
    saved = THRESH
    THRESH = [("possible_at", "Possible from", "0.65", saved[0][3]), ("review_at", "Probable from", "0.82", saved[1][3]), saved[2]]
    try:
        html = matching_page(strip, toast_html=t)
    finally:
        THRESH = saved
    return html.replace('aria-label="38 waiting">38<', 'aria-label="36 waiting">36<')


def page_matching_restart():
    strip = revision_strip("warn", "Revision 8 is applied, and part of it waits for a restart",
                           "The blocking key you added takes effect at the next start, after the key columns are re-indexed. Until then the scorer pairs records by the three keys of revision 7.",
                           f'<span class="fui-pill fui-tone-warn">pending restart</span>')
    rows = [("matching.blocking[3]", "not set", "given name's phonetic key, birth date", "restart")]
    below = card("Waiting for the restart", diff_table(rows, "Revision 8") + '<p class="fui-small fui-muted">Restart with <code>ferropix serve</code> as usual; the re-index runs before the server listens and is reported on the dashboard. Nothing else is pending.</p>')
    return matching_page(strip, blocking_added=True, below=below)


def page_matching_conflict():
    strip = revision_strip("bad", "Someone applied revision 8 while you edited",
                           f"Demo Reviewer applied it {when('2026-10-21T09:51')}: “Pilot a stricter certain grade.” Your draft is kept and was not applied. Base it on revision 8, or discard it.",
                           button("primary", "Rebase my draft on revision 8", "git-compare", small=True) + button("secondary", "Discard my draft", "x", small=True))
    body = [
        f'<tr><th scope="row"><code>{k}</code></th>' + td("Revision 7, your base", a) + td("Revision 8, now running", b) + td("Your draft", c) + td("", s) + "</tr>"
        for k, a, b, c, s in [
            ("matching.review_at", "0.80", "0.80", "0.82", '<span class="fui-badge fui-tone-accent">yours alone</span>'),
            ("matching.possible_at", "0.60", "0.60", "0.65", '<span class="fui-badge fui-tone-accent">yours alone</span>'),
            ("matching.link_at", "0.95", "0.97", "0.95", f'<span class="fui-badge fui-tone-warn">theirs alone</span>'),
        ]
    ]
    t = card("Three revisions side by side", table("Your draft against revision 8", th("Setting") + th("Revision 7, your base") + th("Revision 8, now running") + th("Your draft") + th('<span class="fui-sr-only">Whose</span>'), body)
             + '<p class="fui-small fui-muted">Rebasing keeps your two changes and takes link_at 0.97 from revision 8. You review the result before anything is applied.</p>')
    return config_page("thresholds", strip, config_tab_panel("thresholds", "edit", {"review_at": "0.82", "possible_at": "0.65"}), extra_top=t, drafts=("thresholds",), running="8")


CORE = ('<ul class="fui-bullets fui-small"><li><code>profile</code>, <code>[server]</code>, <code>[telemetry]</code></li><li><code>[db]</code> and the sealing key</li>'
        '<li><code>[auth]</code> and the two product scopes</li><li><code>[operator]</code>, <code>[audit]</code>, <code>[usage_report]</code>, <code>[dev]</code></li>'
        '<li><code>[hl7v2]</code> listener: address, frame limit, timeouts</li><li><code>[config] mode</code> itself</li></ul>')
REST = ('<ul class="fui-bullets fui-small"><li>Matching: thresholds, field weights, blocking keys, <code>auto_link</code></li><li>Pseudonymisation domains</li>'
        '<li>Sources, then the identifier domains each feeds</li><li>The PIXm, PDQm and PMIR options</li><li>The longest reveal a steward may choose</li></ul>')


def mode_dialog(mode):
    """How configuration works: the core beside the one place everything
    else lives, and the other mode. A modal with the focus trapped in it
    and returned to the button that opened it."""
    if mode == "console":
        here = '<h3>' + icon("monitor", "16") + ' Everything else: here, in the console</h3><p class="fui-small fui-muted">Saved as revisions with a reason, applied live where it can be, restored in one step.</p>'
        other = ('<p class="fui-small fui-muted mk-mode-other">The other mode, <strong>file</strong>, keeps everything in ferropix.toml for version control, and this console then shows it read-only. '
                 'A deployment uses one mode. To switch, export the running revision as TOML, set <code>[config] mode = "file"</code> beside it, and restart.</p>')
    else:
        here = '<h3>' + icon("file-text", "16") + ' Everything else: also in ferropix.toml</h3><p class="fui-small fui-muted">Changed in the file, reviewed in git, applied by a reload; shown here read-only, with each reload in the history.</p>'
        other = ('<p class="fui-small fui-muted mk-mode-other">The other mode, <strong>console</strong>, keeps only the core in the file and edits everything else here. '
                 'A deployment uses one mode. To switch, set <code>[config] mode = "console"</code> and restart: the file\'s settings become revision 1.</p>')
    lead = ("This deployment runs in <strong>console mode</strong> (<code>[config] mode = \"console\"</code>)." if mode == "console"
            else "This deployment runs in <strong>file mode</strong> (<code>[config] mode = \"file\"</code>).")
    return (
        '<div id="mode-dialog" role="dialog" aria-modal="true" aria-labelledby="mode-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="mode-title" class="fui-dialog-title">{icon("info", "18")} How configuration works</h2>'
        f'<p>{lead} The core always stays in ferropix.toml; everything else lives in one place, never in both.</p>'
        '<div class="fui-modal-body"><div class="fui-ownership">'
        '<div><h3>' + icon("lock", "16") + ' The core: always in ferropix.toml</h3><p class="fui-small fui-muted">Never in a revision, so no screen can lock you out or lower the security floor.</p>' + CORE + '</div>'
        '<div>' + here + REST + '</div></div>' + other + '</div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc", attrs="autofocus")}</div></div>'
    )


def mode_badge(mode):
    """The deployment's mode, quiet in the page header; the TOML line on
    hover or focus."""
    word = "Console mode" if mode == "console" else "File mode"
    glyph = "monitor" if mode == "console" else "file-text"
    return (f'<span class="fui-mode-badge" tabindex="0" aria-describedby="mode-tip">{icon(glyph, "14")}{word}'
            f'<span id="mode-tip" role="tooltip" class="fui-tooltip fui-tooltip-below fui-mode-tip"><code>[config] mode = "{mode}"</code></span></span>')


def mode_help_button(mode):
    href = "configuration-mode-dialog.html" if mode == "console" else "configuration-file-mode-dialog.html"
    return link_button("ghost", "How configuration works", href, "info", attrs='aria-haspopup="dialog"')


def fold_actions(items, menu_items, open_=False):
    """Page actions in the title row; below 40 rem one Actions menu."""
    m = menu(menu_items, label="Page actions") if open_ else ""
    trigger = (f'<span class="fui-anchor fui-actions-fold-menu"><button type="button" class="fui-button fui-button-secondary" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}">'
               f'Actions{icon("chevron-down", "16")}</button>{m}</span>')
    return f'<div class="fui-actions-fold">{"".join(items)}{trigger}</div>'


def configuration_heading(mode, running="7"):
    if mode == "console":
        items = [mode_help_button("console"),
                 button("secondary", f"Export revision {running} as TOML", "download"),
                 link_button("secondary", "Import TOML…", "configuration-import.html", "upload")]
        menu_items = [menu_item("How configuration works", "info", href="configuration-mode-dialog.html"),
                      menu_item(f"Export revision {running} as TOML", "download"), menu_item("Import TOML…", "upload", href="configuration-import.html")]
        sub = "Where this deployment's settings live, the revision that runs, and every change before it."
    else:
        items = [mode_help_button("file"), button("secondary", "Export the running settings as TOML", "download")]
        menu_items = [menu_item("How configuration works", "info", href="configuration-file-mode-dialog.html"),
                      menu_item("Export the running settings as TOML", "download")]
        sub = "Where this deployment's settings live, what runs, and every reading of the file."
    return page_heading("Configuration", actions=action_group(items[0], [m for m in menu_items[1:]], label="More configuration actions"), subtitle=sub, badge=mode_badge(mode))


def page_configuration(overlay=None, after_apply=False):
    strip = revision_strip("running", "Revision 7 runs", f"Applied {when('2026-10-14T11:05')} by Demo Steward: “Raise review_at after the September audit.”")
    running = "7"
    hist = []
    if after_apply:
        strip = revision_strip("running", "Revision 8 runs", f"Applied {when('2026-10-21T09:58')} by Demo Steward: “Fewer weak candidates after the October review.”")
        running = "8"
        hist = [("8", "console", "Demo Steward", "2026-10-21T09:58", "Fewer weak candidates after the October review", pill("ok", "running"))]
    hist += [
        ("7", "console", "Demo Steward", "2026-10-14T11:05", "Raise review_at after the September audit", "" if after_apply else pill("ok", "running")),
        ("6", "console", "Demo Reviewer", "2026-10-13T10:30", "Shorter reveal limit, DPO request", ""),
        ("5", "restore", "Demo Steward", "2026-09-30T15:40", "Restore revision 3: too many weak pairs", ""),
        ("4", "console", "Demo Reviewer", "2026-09-30T15:12", "Weight telephone higher", ""),
    ]
    rows = [
        f'<tr><th scope="row" class="mk-nowrap">Revision {n}</th>' + td("How", badge("neutral", src)) + td("By", by, "mk-nowrap") + td("When", when(at)) + td("Reason", r) + td("State", st)
        + td("", (f'<a class="fui-icon-button" href="configuration-compare.html" aria-label="Compare revision {n} with the running one" title="Compare with running">{icon("git-compare", "16")}</a>'
                  + icon_button("rotate-ccw", f"Restore revision {n}…")) if n != running else "", "fui-table-actions") + "</tr>"
        for n, src, by, at, r, st in hist
    ]
    t = ('<div class="fui-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">Every revision, newest first. Compare one with what runs, or restore it as a new revision.</p></div>'
         + table("Revisions", th("Revision") + th("How") + th("By") + th("When") + th("Reason") + th("State") + th('<span class="fui-sr-only">Actions</span>'), rows) + "</div>")
    return config_page("history", strip, t, overlay=overlay, running=running)


def page_configuration_file_mode(overlay=None):
    strip = revision_strip("running", "ferropix.toml, read at the last reload", f"Read {when('2026-10-21T06:00')} after a change to the file (config repository commit 4f2a9c1). Change the file to change any setting below the core.")
    hist = [
        ("4", "reload", "ferropix.toml", "2026-10-21T06:00", "Read after a change to the file", pill("ok", "running")),
        ("3", "reload", "ferropix.toml", "2026-10-14T06:00", "Read after a change to the file", ""),
        ("2", "start", "ferropix.toml", "2026-10-13T06:00", "Read at the start", ""),
    ]
    rows = [
        f'<tr><th scope="row">Reading {n}</th>' + td("How", badge("neutral", src)) + td("From", by) + td("When", when(at)) + td("Note", r) + td("State", st)
        + td("", f'<a class="fui-icon-button" href="configuration-compare.html" aria-label="Compare reading {n} with the running one" title="Compare with running">{icon("git-compare", "16")}</a>' if n != "4" else "", "fui-table-actions") + "</tr>"
        for n, src, by, at, r, st in hist
    ]
    t = ('<div class="fui-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">Each reading of the file, newest first. To go back, revert the file and reload.</p></div>'
         + table("Readings", th("Reading") + th("How") + th("From") + th("When") + th("Note") + th("State") + th('<span class="fui-sr-only">Actions</span>'), rows) + "</div>")
    return config_page("history", strip, t, overlay=overlay, config_mode="file")


def page_configuration_mode_dialog():
    return matching_page(matching_view_strip(), overlay=mode_dialog("console"))


def page_configuration_file_mode_dialog():
    return page_configuration_file_mode(overlay=mode_dialog("file"))


def page_matching_file_mode():
    strip = revision_strip("running", "Configured in ferropix.toml", f"Read {when('2026-10-21T06:00')} at the last reload. This deployment keeps its settings in the file, so this page is read-only: change <code>[matching]</code> in the file and reload.",
                           mode_help_button("file"))
    return matching_page(strip, actions="", config_mode="file")


def page_domains_file_mode():
    t = titled_panel(
        "Domains <span class=\"mk-count\">· 5, 4 active</span>", "",
        table("Identifier domains", th("Name", "name", "ascending") + th("System") + th("Kind") + th("Sensitive") + th("Source") + th("Records", num=True) + th("Last fed") + th("State"), domain_rows()),
    )
    strip = revision_strip("running", "Configured in ferropix.toml", "The domains are the file's <code>[[domain]]</code> entries: add, change or retire one there and reload. The counts and the feed activity are live.",
                           mode_help_button("file"))
    content = page_heading("Identifier domains", subtitle="The assigning authorities, each with its one Source and its feed activity.") + f'<div class="fui-stack">{strip}{t}</div>'
    return shell("Identifier domains", "domains", content)


# ---- audit ----

ACTS = [
    ("09:42:10", "Demo Steward", ("ok", "link"), "Mohr, Alice", "Maiden name; same telephone and address history", "01HZ8K7Q4A1B"),
    ("09:41:55", "Demo Steward", ("warn", "reveal"), "Mohr, Alice", "Phone call with the laboratory about sample LAB-…550", "01HZ8K6M2C3D"),
    ("09:40:30", "Demo Steward", ("neutral", "view"), "Mohr, Alice", None, "01HZ8K5A8E5F"),
    ("09:40:02", "system", ("info", "auto-link"), "Jansen, Piet", "BSN agreed", "01HZ8K4ZZG7H"),
    ("09:31:44", "Demo Steward", ("neutral", "defer"), "Okafor, Chidi", "Awaiting the radiology desk's reply", "01HZ8JX01J9K"),
    ("09:28:09", "Demo Steward", ("bad", "not-a-match"), "Nguyen, Linh", "Different birth date confirmed with the registration desk", "01HZ8JT4RL2M"),
    ("09:12:51", "Demo Steward", ("warn", "resolve-pseudonym"), "Petrova, Irina", "Research query, ticket RES-204", "01HZ8J8Q0N4P"),
    ("08:55:17", "Demo Steward", ("neutral", "export"), None, "Weekly audit extract for the DPO", "01HZ8HV2WQ6R"),
    ("08:30:00", "Demo Steward", ("neutral", "sign-in"), None, None, "01HZ8H0A1S8T"),
]
REVEAL_DETAIL = definitions([("Revealed", "BSN, for 30 s, copied once"), ("Shown until", when("2026-10-21T09:42:25", seconds=True)), ("Client", "console · session …8f1e")])


def audit_filters(kind="all", actor="all", live=True, range_html=None, range_word="Today"):
    kw = {"all": None, "reveal": "reveal and resolve", "link": "link", "merge": "merge", "view": "view"}[kind]
    aw = {"all": None, "demo": "Demo Steward", "system": "system", "feed": "feeds"}[actor]
    if range_html:
        rng = (f'<span class="fui-anchor">{button("secondary", range_word, "calendar", attrs="aria-haspopup=" + chr(34) + "dialog" + chr(34) + " aria-expanded=" + chr(34) + "true" + chr(34))}'
               f'<span class="mk-range-pop">{range_html}</span></span>')
    else:
        rng = button("secondary", range_word, "calendar", attrs='aria-haspopup="dialog" aria-expanded="false"')
    return tpl.toolbar("A name or a person id", "", [tpl.filter_button("Actor", aw), tpl.filter_button("Kind", kw), rng])


def audit_rows(acts, expanded=None):
    rows = []
    for i, (t, actor, kind, patient, reason, rid) in enumerate(acts):
        p = f'<a href="patient-identifiers.html">{e(patient)}</a>' if patient else '<span class="mk-dash">–</span>'
        r = e(reason) if reason else '<span class="mk-dash">–</span>'
        is_open = i == expanded
        toggle = button("ghost", f'<span class="fui-sr-only">Detail of the action at {t}</span>', "chevron-down" if is_open else "chevron-right", small=True,
                        attrs=f'aria-expanded="{"true" if is_open else "false"}" title="Detail"')
        rows.append(
            f'<tr><th scope="row">{when("2026-10-21T" + t, seconds=True)}</th>' + td("Actor", e(actor), "mk-nowrap") + td("Kind", badge(*kind)) + td("Patient", p, "mk-nowrap")
            + td("Reason", r) + td("Request id", f"<code>{rid}</code>") + td("", toggle, "fui-table-actions") + "</tr>"
        )
        if is_open:
            rows.append(f'<tr class="mk-detail-row"><td colspan="7">{well(REVEAL_DETAIL)}</td></tr>')
    return rows


def audit_page(acts=ACTS, expanded=None, filters=None, chips="", export="idle", toast_html="", count=None, loading=False):
    if export == "busy":
        exp = busy_button("secondary", "Exporting", small=True)
    else:
        exp = button("secondary", "Export CSV", "download", small=True)
    n = count or f"{len(acts)} today"
    head = th("Time", "time", "descending") + th("Actor") + th("Kind") + th("Patient") + th("Reason") + th("Request id") + th('<span class="fui-sr-only">Detail</span>')
    if loading:
        rows = ["".join(f'<tr aria-hidden="true"><th>{skeleton_lines([70])}</th><td>{skeleton_lines([80])}</td><td>{skeleton_lines([50])}</td><td>{skeleton_lines([70])}</td><td>{skeleton_lines([90])}</td><td>{skeleton_lines([80])}</td><td></td></tr>' for _ in range(8))]
        n = '<span class="fui-spinner fui-small"><span class="fui-spinner-ring" aria-hidden="true"></span>Loading</span>'
    else:
        rows = audit_rows(acts, expanded)
    extra = ""
    if export == "busy":
        extra = notice("info", "clock", "Exporting 9 actions for 2026-10-21 as CSV. The export is itself recorded in the trail.")
    t = titled_panel(f"Actions <span class=\"mk-count\">· {n}</span>", exp, table("Steward actions", head, rows))
    content = page_heading("Audit trail", subtitle="Every steward action: who made it, when and why. An export is itself an action.") + tpl.list_body("", filters or audit_filters(), chips, extra + t)
    return shell("Audit trail", "audit", content, toast_html=toast_html)


def page_audit():
    return audit_page()


def page_audit_detail():
    return audit_page(expanded=1)


def page_audit_filtered():
    acts = [a for a in ACTS if a[2][1] in ("reveal", "resolve-pseudonym")]
    chips = filter_chips(["Kind: reveal and resolve"])
    return audit_page(acts, filters=audit_filters(kind="reveal"), chips=chips, count="2 of 9 today")


def page_audit_export_busy():
    return audit_page(export="busy")


def page_audit_export_done():
    t = toasts(toast("ok", "download", "Export ready", "ferropix-audit-2026-10-21.csv · 9 actions, 3 KB. The export is recorded in the trail.",
                     action=button("secondary", "Download", "download", small=True)))
    acts = [("09:43:02", "Demo Steward", ("neutral", "export"), None, "Daily extract for the DPO", "01HZ8K8T5U2V")] + ACTS
    return audit_page(acts, toast_html=t, count="10 today")


def page_audit_loading():
    return audit_page(loading=True)


# ---- settings: every choice applies at once (review item 33) ----

# ---- settings: a personal page of setting rows (review round 1, owner) ----

SETTINGS_SECTIONS = [("appearance", "Appearance", "sun"), ("language", "Language and region", "globe"),
                     ("privacy", "Privacy", "eye-off"), ("keyboard", "Keyboard", "keyboard"), ("account", "Account", "circle-user")]

LANGUAGES = [("browser", "Follow the browser", "English, from your browser"), ("en", "English", None), ("nl", "Nederlands", None)]


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


def settings_page(theme="light", saved=None, lang_open=False, nav_open=False, overlay=None, lock_dark=False, tab="appearance"):
    sel = lambda id_, opts, cur: select(id_, [(v, w) for v, w in opts], cur, cls="fui-select fui-setting-select")
    appearance = setting_card("appearance", "Appearance",
        setting_row("Theme", "Light, dark, or the one your system uses. Applies at once.",
                    choice_cards("theme", "Theme", [("light", "Light", "Plum on white"), ("dark", "Dark", "Plum on ink"), ("system", "System", "Follows your device")], theme, theme_preview), saved=saved == "theme")
        + setting_row("Density", "Compact fits more rows in the queue and every list; comfortable, the default, gives rows and forms room.",
                      choice_cards("density", "Density", [("comfortable", "Comfortable", "More room"), ("compact", "Compact", "More rows")], "comfortable", density_preview), saved=saved == "density")
        + setting_row("Reduce motion", "Fewer animations in menus, panels and the countdown ring.", sel("motion", [("system", "Follow the system"), ("always", "Always reduce")], "system"), id_="motion", saved=saved == "motion"))
    lang_ctrl = select_button("lang", "English", lang_open, listbox(LANGUAGES, "en"))
    language = setting_card("language", "Language and region",
        setting_row("Language", "The console's words. Each language is listed in its own name; more follow.", lang_ctrl, id_="lang", saved=saved == "lang")
        + setting_row("Dates", "Written year first everywhere, with today said in words. Hover any time for the full timestamp.", '<span class="fui-setting-value"><time class="fui-time">2026-10-21</time> · today 09:42</span>')
        + setting_row("First day of the week", "Follows the language; change it if your team counts weeks differently.", sel("week", [("auto", "Monday, from the language"), ("mon", "Monday"), ("sun", "Sunday")], "auto"), id_="week")
        + setting_row("Time zone", "Times are shown in the server's zone; your browser's is the same.", '<span class="fui-setting-value">Europe/Amsterdam · CEST, UTC+2</span>'))
    privacy = setting_card("privacy", "Privacy",
        setting_row("Reveal duration", "How long a revealed identifier stays before it is masked again. The server caps every reveal at its maximum (30 s here), whatever you choose.", sel("reveal", [("10", "10 seconds"), ("15", "15 seconds"), ("30", "30 seconds")], "15"), id_="reveal", saved=saved == "reveal")
        + setting_row("Hide when I leave the page", "Mask every revealed identifier again as soon as you open another page or tab.",
                      '<input type="checkbox" class="fui-switch" id="hide-leave" checked aria-label="Hide when I leave the page" />', saved=saved == "hide")
        + setting_row("Your reveals", "Every reveal is recorded with your name, the reason and the duration.", link_button("secondary", "Your reveals in the audit trail", "audit-filtered.html", "file-clock", small=True)))
    keyboard = setting_card("keyboard", "Keyboard",
        setting_row("Shortcut hints", "The quiet key caps after a label. Phones never show them.", sel("hints", [("wide", "Wide screens only (default)"), ("always", "Always"), ("never", "Never")], "wide"), id_="hints")
        + setting_row("All shortcuts", "Every shortcut of the screen you are on, in one list.", link_button("secondary", "Open the shortcut legend", "shortcuts.html", "keyboard", small=True, keys="?")))
    account = setting_card("account", "Account",
        setting_row("Signed in as", "Your name as the trail records it.", '<span class="fui-setting-value fui-setting-strong">Demo Steward</span>')
        + setting_row("Identity provider", "Where you signed in.", '<span class="fui-setting-value">FerroHEALTH ID</span>')
        + setting_row("Access", "The scopes your token holds.", f'<span class="fui-setting-value mk-scopes">{badge("accent", "steward")}{badge("accent", "administrator")}</span>')
        + setting_row("Session", "Started at sign-in; ends after 30 minutes without activity, at the latest at the time shown.", f'<span class="fui-setting-value">since {when("2026-10-21T08:30")} · until today 16:30</span>')
        + setting_row("Sign out", "This browser only, or every session you have open.", button("secondary", "Sign out", "log-out", small=True) + button("danger", "Sign out everywhere", "log-out", small=True))
        + setting_row("Privacy notice", "How the console treats your own data.", link_button("ghost", "Read the privacy notice", "#", "file-text", small=True)))
    reset = f'<div class="fui-settings-foot"><p class="fui-small fui-muted">Kept in this browser as cookies, read when each page is drawn. Nothing here names a patient.</p>{button("ghost", "Reset to defaults…", "rotate-ccw", small=True)}</div>'
    import re as _re
    def rows_of(card_html):
        m = _re.search(r'<div class="fui-setting-rows">(.*)</div></section>$', card_html, _re.S)
        return m.group(1) if m else card_html
    panels = {"appearance": appearance, "language": language, "privacy": privacy, "keyboard": keyboard, "account": account}
    inner = settings_panel(rows_of(panels[tab]))
    content = (page_heading("Your preferences", subtitle='Only for you, in this browser. Deployment settings are in <a href="configuration.html">Configuration</a>.')
               + f'<div class="fui-stack">{settings_tabs("Preference sections", PREF_TABS, tab, inner)}{reset}</div>')
    html = shell("Your preferences", "", content, overlay=overlay)
    if lock_dark:
        html = html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)
    return html


PREF_TABS = [("appearance", "Appearance", "preferences.html"), ("language", "Language and region", "preferences-language-open.html"),
             ("privacy", "Privacy", "preferences-saved.html"), ("keyboard", "Keyboard", "preferences-keyboard.html"), ("account", "Account", "preferences-account.html")]


def page_settings():
    return settings_page()


def page_settings_language_open():
    return settings_page(lang_open=True, tab="language")


def page_settings_saved():
    return settings_page(saved="reveal", tab="privacy")


def page_settings_keyboard():
    return settings_page(tab="keyboard")


def page_settings_reset():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="reset-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="reset-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Reset your preferences?</h2>'
        '<p>Theme, density, motion, language, week, reveal duration, hiding and hints go back to their defaults: their cookies in this browser are cleared. Your account, the configuration of the deployment and the audit trail are not touched.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Reset to defaults", "rotate-ccw")}</div></div>'
    )
    return settings_page(overlay=dialog)


def page_settings_account():
    return settings_page(tab="account")


def page_settings_dark():
    return settings_page(theme="dark", saved="theme", lock_dark=True)


# ---- sign-in ----

def sign_in_card(notice_html="", username="", password_error=None, locked=False, redirecting=False):
    if redirecting:
        provider = f'<button type="button" class="fui-button fui-button-primary" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>Taking you to FerroHEALTH ID…</button>'
    else:
        provider = f'<a href="#" rel="external" class="fui-button fui-button-primary">{icon("log-in", "16")}Sign in with FerroHEALTH ID</a>'
    dis = " disabled" if (locked or redirecting) else ""
    inv = ' aria-invalid="true" aria-describedby="sign-in-password-error"' if password_error else ""
    er = f'<p id="sign-in-password-error" class="fui-help fui-error">{icon("circle-alert", "14")} {password_error}</p>' if password_error else ""
    return (
        f'<div class="fui-card fui-sign-in"><div class="fui-sign-in-lockup">{lockup()}</div>'
        + notice_html + provider
        + '<div class="fui-divider">or, on this development deployment</div>'
        '<form method="post" action="#" class="fui-form">'
        + f'<div class="fui-field"><label for="sign-in-username" class="fui-label">Username</label><input id="sign-in-username" name="username" class="fui-input" value="{e(username)}" autocomplete="username" autocapitalize="off" spellcheck="false"{dis} /></div>'
        + f'<div class="fui-field"><label for="sign-in-password" class="fui-label">Password</label><input id="sign-in-password" name="password" type="password" class="fui-input"{" value=\"xxxxxxxx\"" if password_error else ""} autocomplete="current-password"{inv}{dis} />{er}</div>'
        + f'<button type="submit" class="fui-button fui-button-secondary"{dis}>{icon("log-in", "16")}Sign in</button></form>'
        '<div class="fui-sign-in-links"><a href="#" rel="external noopener">Documentation</a><a href="#" rel="external noopener">Privacy notice</a></div></div>'
        '<p class="fui-sign-in-caption">The console for the data stewards of the patient index. No patient data is shown before you sign in.</p>'
    )


def sign_in_page(**kw):
    return centred("Sign in", f'<div class="fui-stack mk-sign-in-stack">{sign_in_card(**kw)}</div>')


def page_sign_in():
    return sign_in_page()


def page_sign_in_redirecting():
    return sign_in_page(redirecting=True, notice_html='<div role="status" class="fui-sign-in-notice fui-tone-info">Opening the FerroHEALTH ID sign-in. You come back here when you are signed in.</div>')


def page_sign_in_wrong():
    return sign_in_page(username="demo.steward", password_error="4 tries are left before a 15-minute pause.",
                        notice_html='<div role="alert" class="fui-sign-in-notice fui-tone-bad">Not signed in: the username or the password is wrong.</div>')


def page_sign_in_locked():
    return sign_in_page(username="demo.steward", locked=True,
                        notice_html='<div role="alert" class="fui-sign-in-notice fui-tone-warn">Sign-in paused for this username after 5 failed tries. Try again after 09:57, or sign in with FerroHEALTH ID.</div>')


def page_sign_in_expired():
    return sign_in_page(notice_html='<div role="status" class="fui-sign-in-notice fui-tone-warn">Your session ended after 30 minutes without activity. Sign in again to go back to the review queue.</div>')


# ---- global: shell states, toasts, status pages, focus ----

def page_sidebar_rail():
    return page_dashboard(rail=True, rail_tip="queue")


def page_sidebar_drawer():
    return page_dashboard(drawer=True)


def page_user_menu():
    return page_dashboard(menu_open=True)


def page_user_menu_dark():
    html = page_dashboard(menu_open=True, menu_theme="dark")
    return html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)


def page_toast(kind):
    t = {
        "ok": toast("ok", "circle-check", "Linked: Maiden, Alice is part of Mohr, Alice", "Recorded with your reason. 2 subscribers told."),
        "warn": toast("warn", "triangle-alert", "The feed sender is behind", "41 notifications wait for subscriber ehr-research; retrying every 30 s.",
                      action=link_button("secondary", "Dashboard", "dashboard.html", small=True)),
        "bad": toast("bad", "circle-alert", "The reveal failed", "The server refused it (403): your token lacks the steward scope. Nothing was shown or recorded."),
    }[kind]
    return queue_page(toast_html=toasts(t))


def status_page(code, glyph, title, body, actions, current="dashboard", tone_status=("ok", "Server up")):
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon(glyph, "24")}</span><p class="fui-status-page-code">{code}</p>'
               f'<h1>{title}</h1><p>{body}</p><div class="fui-actions">{actions}</div></div>')
    return shell(title, current, content, status=tone_status)


def page_forbidden():
    return status_page("403 · Not allowed", "ban", "This page needs the administrator scope",
                       "You are signed in as Demo Steward with the steward scope. Changing the matching configuration needs the administrator scope (__ferropix.admin). Ask an administrator, or read the values on the Matching page.",
                       link_button("primary", "Back to Configuration", "configuration.html", "arrow-left") + link_button("secondary", "Dashboard", "dashboard.html"), current="configuration")


def page_not_found():
    return status_page("404 · Not found", "compass", "No patient with this person id",
                       "The person id 0199c2f1…f0f0 is not known to FerroPIX. It may be mistyped, or it was never minted here. A merged person still opens: its id leads to the survivor.",
                       link_button("primary", "Search for a patient", "search.html", "search") + link_button("secondary", "Dashboard", "dashboard.html"), current="search")


def page_focus_rings():
    def row(title, html):
        return f'<div class="mk-focus-row"><span class="mk-focus-name">{title}</span><div class="mk-focus-demo">{html}</div></div>'
    rows = (
        row("Primary button", button("primary", "Link", "link", cls="mk-focus", keys="l"))
        + row("Secondary button", button("secondary", "Defer", "clock", cls="mk-focus"))
        + row("Danger button", button("danger", "Merge 1 identifier…", "merge", cls="mk-focus"))
        + row("Icon button", icon_button("ellipsis", "More actions", cls="mk-focus"))
        + row("Link", '<a href="#" class="mk-focus-ring mk-inline-link">Mohr, Alice</a>')
        + row("Text field", input_("f1", "Maiden name", cls="fui-input mk-focus"))
        + row("Select", select("f2", [("all", "All grades")], "all", cls="fui-select mk-focus"))
        + row("Checkbox", '<div class="fui-check"><input type="checkbox" id="f3" class="mk-focus" checked /><label for="f3" class="fui-label">Include deferred</label></div>')
        + row("Switch", '<div class="fui-check"><input type="checkbox" id="f4" class="fui-switch mk-focus" checked /><label for="f4" class="fui-label">Sensitive</label></div>')
        + row("Segmented control", '<div role="group" class="fui-segmented"><button class="fui-segment" aria-pressed="true">By identifier</button><button class="fui-segment mk-focus">By demographics</button></div>')
        + row("Filter chip", '<div class="fui-chip-checks"><label class="fui-chip-check mk-focus-ring"><input type="checkbox" checked />link</label></div>')
        + row("Tab", tabbed(tabs("Example", [t[:3] for t in PATIENT_TABS[:3]], "identifiers", focus="records"), '<span class="fui-small fui-muted">Panel</span>'))
        + row("Search control", search_control("", "focus", id_="find-f"))
        + row("Work-list row", '<div class="fui-card fui-panel fui-worklist">' + table("Example", queue_head(checks=False), queue_rows(rows=CANDIDATES[2:3], kb_focus=0, focused=0, checks=False)) + "</div>")
        + row("Menu item", menu([menu_item("Open in a side panel", "panel-right", keys="o", focus=True), menu_item("Defer…", "clock", keys="d")], anchored=False))
    )
    content = page_heading("Focus, on every control", subtitle="The ring is the --focus role, 2 px, held to 3:1 against its ground in both themes; it sits outside a control and inside a row, a tab or a menu item so no neighbour clips it.") + f'<div class="fui-card mk-focus-sheet">{rows}</div>'
    return shell("Focus rings", "", content)


# ---- the screens every remaining control leads to ----

def page_pseudonym_domain_new():
    form = (
        '<div class="fui-form mk-edit-form"><div class="fui-columns">'
        + field("p-ns", "Namespace", input_("p-ns", "research-2027-q1", extra='spellcheck="false"'), help_="Lower case, digits and hyphens. It names the domain in every export and cannot change once a pseudonym is minted.")
        + field("p-purpose", "Purpose", input_("p-purpose", "Follow-up cohort, pseudonymised"))
        + "</div>"
        + choice_group("p-method", "Method", [("keyed", "Keyed hash (HMAC-SHA-256)"), ("random", "Random, stored")], "keyed")
        + '<p class="fui-help">A keyed hash gives the same pseudonym for the same patient every time and needs no table; a random one is stored and can be withdrawn one by one.</p>'
        + "</div>"
    )
    content = (
        page_heading("New pseudonymisation domain", subtitle="A database record: created here with a reason, recorded in the audit trail, and live at once.", crumbs=[("Pseudonymisation domains", "pseudonym-domains.html")])
        + '<div class="fui-stack">' + card("Definition", form) + "</div>"
    )
    dk = dock(dock_field("p-reason", "Reason", "Follow-up study approved by the ethics board, ticket RES-311", "Why this domain is created"),
              link_button("secondary", "Cancel", "pseudonym-domains.html") + button("primary", "Create the domain", "plus"))
    return shell("New pseudonymisation domain", "pseudonym-domains", content, dock_html=dk)


def page_domain_reinstate():
    dialog = (
        '<div role="dialog" aria-labelledby="re-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="re-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Reinstate Radiology RIS?</h2>'
        '<p>Feeds to this domain are accepted again from the Source you choose. Its records were kept, so nothing is restored by hand.</p>'
        + field("re-src", "Source", select("re-src", [("ris-prod", "ris-prod"), ("pacs-prod", "pacs-prod · client credentials")], "pacs-prod"))
        + field("re-reason", "Reason", textarea("re-reason", "PACS numbering postponed; RIS feeds resume"), help_="Required. Recorded in the audit trail.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel")}{button("primary", "Reinstate", "rotate-ccw")}</div></div>'
    )
    return page_domain("Radiology RIS", retired=True, overlay=dialog)


def page_unmerge_confirm():
    html = page_unmerge()
    dialog = (
        '<div role="dialog" aria-labelledby="um-title" class="fui-dialog fui-dialog-danger fui-modal mk-open mk-open-low" open>'
        f'<h2 id="um-title" class="fui-dialog-title">{icon("triangle-alert", "18")} Unmerge Mohr, A. from Mohr, Alice?</h2>'
        '<p>Mohr, A. becomes a patient of its own again with the Radiology RIS identifier, and 2 subscribers are told. The pair returns to the review queue.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Go back")}{button("danger", "Unmerge", "undo")}</div></div>'
    )
    return html.replace('</div></body></html>', f'<div class="mk-scrim" aria-hidden="true"></div>{dialog}</div></body></html>').replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1)


def page_unmerge_done():
    t = toasts(toast("ok", "circle-check", "Unmerged: Mohr, A. is a patient of its own again", "1 identifier moved back; 2 subscribers told; the pair is in the review queue."))
    return page_patient_identifiers(toast_html=t)


def page_reveal_refused():
    t = toasts(toast("bad", "circle-alert", "The BSN was not revealed", "The server refused the reveal (403): your token lacks the steward scope. Nothing was shown, and the attempt is recorded."))
    return page_patient_identifiers(toast_html=t)


def page_resolve_answer():
    dialog = (
        '<div role="dialog" aria-labelledby="ra-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="ra-title" class="fui-dialog-title">{icon("fingerprint", "18")} The pseudonym belongs to Petrova, Irina</h2>'
        + definitions([("Domain", "research-2026-q3"), ("Pseudonym", identifier("research-2026-q3", "8kd", static=True, dots=10, show_domain=False)),
                       ("Patient", f'<a href="patient-identifiers.html">Petrova, Irina</a> · born 1963-02-17 · {chip("0199a9b1…e007")}'),
                       ("Recorded", f"as a disclosure, {when('2026-10-21T09:12:51')}, with your reason")])
        + f'<div class="fui-dialog-actions">{button("secondary", "Close")}{link_button("primary", "Open the patient", "patient-identifiers.html", "arrow-right")}</div></div>'
    )
    return page_patient_identifiers(overlay=dialog)


def page_compare_link_merge():
    dialog = (
        '<div role="dialog" aria-labelledby="lm-title" class="fui-dialog fui-dialog-danger fui-modal mk-open mk-open-low" open>'
        f'<h2 id="lm-title" class="fui-dialog-title">{icon("triangle-alert", "18")} Link these records? The two patients become one</h2>'
        '<p>Maiden, Alice is a patient of its own, so linking merges her into Mohr, Alice: 1 identifier moves, 2 subscribers are told. Reversible until the next merge of either patient.</p>'
        + well('<p class="fui-small"><strong>Reason:</strong> Maiden name; same telephone and address history</p>')
        + f'<div class="fui-dialog-actions">{button("secondary", "Go back")}{button("danger", "Link and merge", "merge")}</div></div>'
    )
    return page_compare(overlay=dialog)


def page_queue_bulk_defer():
    sel = (
        '<div class="fui-selection-bar" role="status"><span>2 candidates selected</span>'
        f'<div class="fui-actions">{button("secondary", "Defer…", "clock", small=True, attrs="aria-haspopup=\"dialog\" aria-expanded=\"true\"")}{button("ghost", "Clear the selection", "x", small=True)}</div></div>'
    )
    return queue_page(queue_rows(selected=(3, 4)), selection=sel, overlay=defer_dialog(count=2, reason="Awaiting the laboratory's batch correction"))


def search_page_body(results_html):
    seg = segmented_form("Search by", [("identifier", "By identifier"), ("demographics", "By demographics")], "demographics")
    form = (
        '<form method="post" action="#" class="fui-form mk-search-form"><div class="fui-form-grid">'
        + field("family", "Family name", input_("family", "Mohr"))
        + field("given", "Given names", input_("given", "Alicia"))
        + field("born", "Birth date", date_input("born", "1985-03-14"))
        + field("gender", "Gender", select("gender", [("", "Any")], ""))
        + field("postal", "Postal code", input_("postal", "9997 CD"))
        + '</div><div class="fui-form-foot">' + check("exact", "Exact names only", True, "On: only exact spellings count.")
        + button("primary", "Search", "search", type_="submit") + "</div></form>"
    )
    content = (page_heading("Search", subtitle="Find a patient by one identifier, or by the demographics the matcher scores. Results live in this page and never in a URL.")
               + f'<div class="fui-stack">{card("Find a patient", seg + form)}{results_html}</div>')
    return content


def page_search_no_match():
    body = empty_state("search-x", "No patient matches", "Nothing scored as a possible match for these five fields with exact names on. Two things to try:",
                       button("primary", "Turn exact names off") + button("secondary", "Search without the postal code"), filtered=True)
    return shell("Search", "search", search_page_body(titled_panel('Results', "", f'<div class="fui-panel-body">{body}</div>')))


def page_search_loading():
    rows = ["".join(f'<tr aria-hidden="true"><th>{skeleton_lines([70])}</th><td>{skeleton_lines([60])}</td><td>{skeleton_lines([40])}</td><td>{skeleton_lines([80])}</td><td>{skeleton_lines([60])}</td><td>{skeleton_lines([40])}</td><td>{skeleton_lines([60])}</td></tr>' for _ in range(3))]
    res = titled_panel('Results <span class="mk-count">· <span class="fui-spinner fui-small"><span class="fui-spinner-ring" aria-hidden="true"></span>Scoring</span></span>', "",
                       table("Search results, loading", th("Name") + th("Born") + th("Gender") + th("Identifiers") + th("Score") + th("Grade") + th("Last fed"), rows))
    return shell("Search", "search", search_page_body(res))


def page_search_refused():
    res = refusal("The search was refused (400): the server could not read the birth date.", issues=[
        ("bad", "error", "invalid", "Patient.birthDate: 1985-03-41 is not a date."),
    ]) + '<p class="mk-refusal-line">Correct the birth date and search again; nothing was searched.</p>'
    return shell("Search", "search", search_page_body(res))


def page_matching_restore():
    dialog = (
        '<div role="dialog" aria-labelledby="rs-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="rs-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Restore revision 7?</h2>'
        '<p>Revision 7 is saved again as revision 9 and applied at once: matching.review_at goes back to 0.80 and matching.possible_at to 0.60, and the open queue is re-graded.</p>'
        + field("rs-reason", "Reason", textarea("rs-reason", "Too many good pairs dropped below possible"), help_="Required. Recorded in the audit trail.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel")}{button("primary", "Restore as revision 9", "rotate-ccw")}</div></div>'
    )
    return page_configuration(overlay=dialog, after_apply=True)


def page_configuration_compare():
    rows = [("matching.review_at", "0.78", "0.80", "live"), ("matching.fields.phone.agree", "+4.4", "+3.9", "live")]
    body = [
        f'<tr><th scope="row"><code>{k}</code></th>' + td("Revision 6", change_del(a)) + td("Revision 7, running", change_ins(b)) + td("Takes effect", effect(fx)) + "</tr>"
        for k, a, b, fx in rows
    ]
    t = table("Revision 6 against revision 7", th("Setting") + th("Revision 6") + th("Revision 7, running") + th("Takes effect"), body)
    content = (page_heading("Revision 6 against revision 7", subtitle="Each setting that differs, with how it took effect. Every other setting is the same.", crumbs=[("Configuration", "configuration.html")])
               + '<div class="fui-stack">' + card("2 settings differ", t) + card("Revision 6", definitions([("From", "the console, Demo Reviewer"), ("When", when("2026-10-13T10:30")), ("Reason", "Shorter reveal limit, DPO request")]),
                                                                                  actions=button("secondary", "Export revision 6 as TOML", "download", small=True) + link_button("ghost", "Revision 6 in History", "configuration.html", "history", small=True)) + "</div>")
    return shell("Configuration", "configuration", content)


def page_configuration_import():
    toml = '[matching]\nreview_at = 0.82\npossible_at = 0.65\n\n[matching.fields.phone]\nagree = 3.9\ndisagree = -1.2\n'
    form = (field("imp-toml", "TOML", textarea("imp-toml", toml, rows=8, extra='spellcheck="false"', cls="fui-textarea mk-code"),
                  help_="Saved as a dry-run revision and never applied by the import. A key of the bootstrap core or a secret written inline is refused, naming the key."))
    rows = [("matching.review_at", "0.80", "0.82", "live"), ("matching.possible_at", "0.60", "0.65", "live")]
    content = (page_heading("Import a configuration", subtitle="From a file kept in version control: paste the TOML, read what it changes, then apply it as any revision.", crumbs=[("Configuration", "configuration.html")])
               + '<div class="fui-stack">' + card("The TOML", form)
               + revision_strip("draft", "Saved as revision 8, a dry run", "Checked: no problems. It changes 2 settings against revision 7; nothing runs until you apply it.")
               + card("What revision 8 changes", diff_table(rows, "Revision 8")) + "</div>")
    dk = dock(dock_field("imp-reason", "Reason", "Thresholds from the October review, as merged in the config repository", "Why this changes"),
              button("secondary", "Discard revision 8", "x") + button("primary", "Apply revision 8…", "check"))
    return shell("Configuration", "configuration", content, dock_html=dk)


def page_audit_no_match():
    body = empty_state("search-x", "No action matches these filters", "9 actions were recorded today, but none by system of the kind merge. Clear a filter to see them.",
                       button("primary", "Clear filters", "x"), filtered=True)
    chips = filter_chips(["Actor: system", "Kind: merge"])
    content = page_heading("Audit trail", subtitle="Every steward action: who made it, when and why. An export is itself an action.") + f'<div class="fui-stack">{audit_filters(kind="merge", actor="system")}{chips}{body}</div>'
    return shell("Audit trail", "audit", content)


def page_about():
    content = (page_heading("About", subtitle="The server this console talks to, the versions, and the manufacturer.")
               + '<div class="fui-stack"><div class="fui-grid fui-grid-start">'
               + card("Versions", definitions([("Console", "FerroPIX Console 0.1.0"), ("Server", "FerroPIX 0.1.0"), ("FHIR", "R4 (4.0.1)"), ("Profiles", "PIXm 3.1.0, PDQm 3.2.0, PMIR 1.6.0"), ("FerroUI", "ferrohealth-ui 0.3.0")]))
               + card("Manufacturer", definitions([("Made by", "Cadasto B.V."), ("Licence", "Business Source License 1.1"), ("Support", '<a href="#">info@cadasto.com</a>'), ("Documentation", '<a href="#">The FerroPIX book</a>')]))
               + "</div>"
               + capability_card()
               + "</div>")
    return shell("About", "about", content)


def capability_card():
    from pg_about import json_html
    acts = copy_button("Copy", "the CapabilityStatement") + button("secondary", "Download", "download", small=True)
    return (f'<section class="fui-card fui-panel mk-options-card"><div class="fui-panel-head"><h2 class="fui-card-title"><span>The CapabilityStatement</span></h2><div class="fui-actions">{acts}</div></div>'
            f'<div class="fui-panel-body fui-stack"><p class="fui-small fui-muted">What the server declares at <code>GET [base]/metadata</code>, as every FHIR client reads it.</p>'
            f'<div class="fui-code-panel mk-options-code" role="region" aria-label="The CapabilityStatement" tabindex="0"><pre>{json_html(CAPABILITY)}</pre></div></div></section>')


CAPABILITY = """{
  "resourceType": "CapabilityStatement",
  "status": "active",
  "kind": "instance",
  "software": { "name": "FerroPIX", "version": "0.1.0" },
  "fhirVersion": "4.0.1",
  "format": [ "application/fhir+json" ],
  "instantiates": [
    "https://profiles.ihe.net/ITI/PIXm/CapabilityStatement/IHE.PIXm.Manager",
    "https://profiles.ihe.net/ITI/PDQm/CapabilityStatement/IHE.PDQm.Supplier",
    "https://profiles.ihe.net/ITI/PMIR/CapabilityStatement/IHE.PMIR.Registry"
  ]
}"""


# ---- sources: who may feed which domain ----
# A FHIR Source is a client its identity provider registers; FerroPIX keeps
# only which domains that client id may feed. An HL7 v2 Source is an MLLP
# sender, which FerroPIX itself registers (sending application and
# facility, the addresses it connects from, its character set).

SOURCES = [
    ("brp-feed", "FHIR", "ITI-104 · client credentials at FerroHEALTH ID", ["BSN"], "2026-10-21T09:41", "812", ("ok", "active"), "console"),
    ("his-prod", "HL7 v2", "MLLP · HIS^ZKH from 10.20.0.0/24", ["Hospital MRN"], "2026-10-21T09:40", "371", ("ok", "active"), "file"),
    ("lab-lis", "HL7 v2", "MLLP · LIS^LAB from 10.20.4.12", ["Laboratory"], "2026-10-21T08:15", "96", ("ok", "active"), "console"),
    ("ris-prod", "FHIR", "ITI-104 · client credentials at FerroHEALTH ID", ["Radiology RIS"], "2026-10-18T17:02", "0", ("warn", "quiet 3 days"), "console"),
    ("brp-feed-2", "FHIR", "ITI-104 · client credentials at FerroHEALTH ID", [], None, "0", ("neutral", "registered, feeds nothing"), "console"),
]


def page_sources():
    rows = []
    for name, ch, how, doms, last, today, st, own in SOURCES:
        rows.append(
            f'<tr><th scope="row"><a href="source.html"><code>{e(name)}</code></a></th>' + td("Channel", tag(ch)) + td("Admitted as", how)
            + td("Feeds", " ".join(f'<a href="domain.html">{e(d)}</a>' for d in doms) if doms else '<span class="mk-dash">no domain names it yet</span>')
            + td("Last fed", rel_time(last)) + td("Today", today, "fui-num") + td("State", pill(*st)) + "</tr>"
        )
    t = titled_panel("Sources <span class=\"mk-count\">· 5, 4 feeding</span>", "",
                     table("Sources", th("Source", "name", "ascending") + th("Channel") + th("Admitted as") + th("Feeds", title="Read from the domains that name this source") + th("Last fed") + th("Today", num=True) + th("State"), rows))
    note = card("How a Source is admitted", '<ul class="fui-bullets fui-small"><li><strong>FHIR</strong>: the client is registered at the identity provider, which issues its token. Here you name its client id; a feed to a domain that does not name this source is refused (ITI-104 answers 403).</li>'
                '<li><strong>HL7 v2</strong>: the sender is registered here by its sending application and facility (MSH-3, MSH-4) and the addresses it connects from; an unknown sender is refused with an AR acknowledgment.</li>'
                '<li>Each identifier domain names its one Source; Feeds here is read from the domains.</li></ul>')
    content = (page_heading("Sources", actions=link_button("primary", "New source", "source-new.html", "plus"),
                            subtitle="The systems allowed to feed records: who each is and how it connects.")
               + f'<div class="fui-stack">{t}{note}</div>')
    return shell("Sources", "sources", content)


def page_source():
    facts = facts_row([("Channel", tag("HL7 v2") + " MLLP"), ("Sending application", "<code>LIS</code>"), ("Sending facility", "<code>LAB</code>"),
                       ("Connects from", "<code>10.20.4.12</code>"), ("Character set", "UNICODE UTF-8 (MSH-18)"), ("State", pill("ok", "active"))])
    doms = ('<p class="mk-feeds"><span class="fui-label">Feeds:</span> <a href="domain.html">Laboratory</a> '
            '<span class="fui-small fui-muted">· read from the domains that name lab-lis as their Source; change it on the domain</span></p>'
            + table("Domains that name lab-lis", th("Domain") + th("Records") + th("Last fed"),
                    [f'<tr><th scope="row"><a href="domain.html">Laboratory</a></th>' + td("Records", "96,220") + td("Last fed", when("2026-10-21T08:15")) + "</tr>"]))
    msgs = [("08:15:02", "ADT^A08", ("ok", "accepted"), "AA"), ("08:14:40", "ADT^A28", ("ok", "accepted"), "AA"), ("08:02:11", "ADT^A40", ("warn", "queued for review"), "AA"), ("07:58:30", "ADT^A31", ("bad", "refused: unknown domain"), "AR")]
    mrows = [f'<tr><th scope="row">{when("2026-10-21T" + t, seconds=True)}</th>' + td("Message", f"<code>{m}</code>") + td("Outcome", status(o[0], "circle-check" if o[0] == "ok" else ("clock" if o[0] == "warn" else "circle-x"), o[1])) + td("Answer", f"<code>{a}</code>") + "</tr>" for t, m, o, a in msgs]
    content = (
        page_heading("lab-lis", actions=action_group(link_button("primary", "Edit", "source-new.html", "pencil"), [menu_item("Disable…", "ban", danger=True)], label="More actions for lab-lis"),
                     subtitle="Source · the laboratory information system, over HL7 v2.", crumbs=[("Sources", "sources.html")])
        + '<div class="fui-stack">' + card("Definition", facts) + card("Feeds <span class=\"mk-count\">· read-only</span>", doms)
        + titled_panel("Last messages <span class=\"mk-count\">· never a value</span>", "", table("Last messages", th("Time") + th("Message") + th("Outcome") + th("Answer"), mrows)) + "</div>"
    )
    return shell("lab-lis", "sources", content)


def page_source_new():
    form = (
        '<div class="fui-form mk-edit-form">'
        + choice_group("s-ch", "Channel", [("fhir", "FHIR client"), ("v2", "HL7 v2 sender")], "v2")
        + '<div class="fui-columns">'
        + field("s-name", "Name", input_("s-name", "dental-ehr", extra='spellcheck="false"'), help_="How the console and the trail name this source.")
        + field("s-ip", "Connects from", input_("s-ip", "10.20.6.0/28", extra='spellcheck="false"'), help_="Addresses or ranges; a connection from anywhere else is closed.")
        + field("s-app", "Sending application (MSH-3)", input_("s-app", "DENT", extra='spellcheck="false"'))
        + field("s-fac", "Sending facility (MSH-4)", input_("s-fac", "DENTAL", extra='spellcheck="false"'))
        + field("s-cs", "Character set when MSH-18 is empty", select("s-cs", [("utf8", "UNICODE UTF-8"), ("8859", "8859/1")], "utf8"))
        + "</div></div>"
    )
    content = (page_heading("New source", subtitle="Who the source is and how it connects. Saved with a reason and live at once; a domain names it when it is created.", crumbs=[("Sources", "sources.html")])
               + '<div class="fui-stack">' + card("Definition", form) + "</div>")
    dk = dock(dock_field("s-reason", "Reason", "Dental clinic joins the index in November", "Why this source is added"),
              link_button("secondary", "Cancel", "sources.html") + button("primary", "Add the source", "plus"))
    return shell("New source", "sources", content, dock_html=dk)


# ---- the date field's picker (R25) ----

import calendar as _cal
import datetime as _dt

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


def picker_single(selected=D(2026, 10, 14), hover=None, focus=None, time=None, label="From", status=""):
    return picker(label, month_grid(2026, 10, selected=selected, hover=hover, focus=focus, max_d=PICK_TODAY), time=time, status=status)


def picker_defer(sheet=False):
    return picker("Defer until", month_grid(2026, 10, selected=D(2026, 10, 28), min_d=D(2026, 10, 22)),
                  quick=[("Tomorrow", False), ("In 3 days", False), ("Next Monday", False), ("In a week", True)], sheet=sheet,
                  status="Back in the queue on Wednesday 28 October 2026.")


def picker_range(half=False, hover=None):
    rng = (D(2026, 10, 14), None if half else D(2026, 10, 21))
    months = (month_grid(2026, 9, rng=rng, tentative=hover, max_d=PICK_TODAY, title_btn=False, nav="prev")
              + month_grid(2026, 10, rng=rng, tentative=hover, max_d=PICK_TODAY, hover=hover, title_btn=False, nav="next"))
    status = "From 2026-10-14: now choose the end." if half else "2026-10-14 to 2026-10-21 · 8 days."
    return picker("Actions between", f'<div class="fui-datepicker-pair">{months}</div>',
                  presets=[("Today", False), ("Last 7 days", not half), ("Last 30 days", False), ("This month", False)], status=status)


def page_date_picker_states():
    def frame(title, line, html):
        return f'<div class="fui-card mk-picker-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div class="mk-picker-demo">{html}</div></div>'
    frames = (
        frame("Open, a date chosen", "today ringed, the chosen day filled, days after today disabled for an audit From",
              date_field_open("ps1", "2026-10-14", "From", picker_single()))
        + frame("A day hovered", "the hovered day takes the sunken ground; the chosen one stays filled",
                date_field_open("ps2", "2026-10-14", "From", picker_single(hover=D(2026, 10, 8))))
        + frame("Keyboard focus", "arrows move by day, PageUp and PageDown by month, Home and End to the week's ends, Enter chooses",
                date_field_open("ps3", "2026-10-14", "From", picker_single(focus=D(2026, 10, 16))))
        + frame("Date and time", "a time row, 24 h, for a date-time field",
                date_field_open("ps4", "2026-10-14 08:30", "From", picker_single(time=("08", "30"), label="From, date and time")))
        + frame("Out-of-range days and quick choices", "Defer cannot go before tomorrow; the quick choices fill the grid",
                date_field_open("ps5", "2026-10-28", "Defer until", picker_defer()))
        + frame("Month and year chooser", "the title opens the months of the year; the year steps with the arrows",
                date_field_open("ps6", "2026-10-14", "From", picker("From", month_chooser(2026, 10))))
        + frame("Range, half chosen", "the start is set; the hovered day previews the range before the second click",
                date_field_open("ps7", "2026-10-14", "From", picker_range(half=True, hover=D(2026, 10, 19))))
        + frame("Range, both ends", "the days between tinted, presets at the side; From and To fill together",
                date_field_open("ps8", "2026-10-14", "From", picker_range()))
        + frame("Invalid typed date", "the field keeps what was typed and says why; the grid stays on the last valid month with nothing chosen",
                date_field_open("ps9", "2026-10-41", "From", picker("From", month_grid(2026, 10, max_d=PICK_TODAY), status="Type a date as YYYY-MM-DD."), invalid="2026-10-41 is not a date: October has 31 days."))
    )
    keys = card("Keys and roles", definitions([
        ("Arrows", "a day; PageUp and PageDown a month; Home and End the week's start and end"),
        ("Enter", "chooses the focused day; a range takes two"),
        ("Esc", "closes and returns the focus to the field"),
        ("Typing", "stays possible in the field; the grid follows a valid date"),
        ("Roles", "<code>role=\"dialog\"</code> with a label, a <code>grid</code> with <code class='mk-nowrap'>aria-selected</code> and <code>aria-current=\"date\"</code>"),
        ("Phone", "the same content as a bottom sheet, larger days, no key hints"),
    ]))
    content = page_heading("Date field and picker, states", subtitle="One year-first field, one picker: audit From and To, Defer until, the history filter, every date-time field. A popover under the field on a desktop, a bottom sheet on a phone.") + f'<div class="fui-stack mk-picker-sheet">{frames}{keys}</div>'
    return shell("Date picker states", "audit", content)


def audit_filters_with(picker_html, field_id="from"):
    return audit_filters(range_html=picker_html, range_word="2026-10-14 to 2026-10-21")


def page_audit_range_picker():
    filt = audit_filters_with(picker_range())
    return audit_page(filters=filt)


def page_date_picker_sheet():
    filt = audit_filters_with(picker_single(label="From", status="Wednesday 14 October 2026").replace('class="fui-datepicker"', 'class="fui-datepicker fui-datepicker-sheet-capable"', 1))
    return audit_page(filters=filt)


# ---- Defer: one dialog everywhere (row menu, side panel, selection bar,
# compare dock) ----

def defer_dialog(subject="Mohr, Alice and Maiden, Alice", count=1, reason="Awaiting the laboratory's call back about the maiden name",
                 date=None, quick="In a week", focus=None, hover=None):
    import datetime as _d
    date = date or _d.date(2026, 10, 28)
    title = "Defer this candidate" if count == 1 else f"Defer {count} candidates"
    lead = (f"<strong>{e(subject)}</strong> leave the queue and come back on the date you choose, in their place by grade and score."
            if count == 1 else f"The {count} selected pairs leave the queue and come back on the date you choose; each deferral is its own record.")
    lead = lead.replace("leave the queue and come back", "leaves the queue and comes back").replace("in their place", "in its place") if count == 1 else lead
    quicks = [(w, w == quick) for w in ("Tomorrow", "In 3 days", "Next Monday", "In a week")]
    qhtml = '<div class="fui-datepicker-quick" role="group" aria-label="Quick choices">' + "".join(
        f'<button type="button" class="fui-chip-check fui-datepicker-chip" aria-pressed="{"true" if on else "false"}">{w}</button>' for w, on in quicks) + "</div>"
    grid = month_grid(date.year, date.month, selected=date, min_d=PICK_TODAY + _d.timedelta(days=1), focus=focus, hover=hover)
    words = f"{WEEKDAY_NAMES[date.weekday()]} {date.day} {MONTHS[date.month - 1]} {date.year}"
    pick = (f'<div class="fui-datepicker fui-datepicker-inline" role="group" aria-label="Back in the queue on">{qhtml}{grid}'
            f'<p class="fui-datepicker-status" role="status">Back in the queue on {words}.</p></div>')
    return (
        '<div id="defer" role="dialog" aria-modal="true" aria-labelledby="defer-title" class="fui-dialog fui-modal mk-open fui-defer-dialog" open>'
        f'<h2 id="defer-title" class="fui-dialog-title">{icon("clock", "18")} {title}</h2>'
        f'<p>{lead}</p><div class="fui-modal-body">'
        + f'<div class="fui-field"><span class="fui-label">Until</span>{pick}</div>'
        + field("defer-reason", "Reason", textarea("defer-reason", reason), help_="Required. Recorded with the deferral and shown when the pair returns.")
        + f'</div><div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", f"Defer to {date.isoformat()}", "clock")}</div></div>'
    )


def page_compare_defer():
    return page_compare(overlay=defer_dialog())


def page_compare_defer_picker():
    import datetime as _d
    return page_compare(overlay=defer_dialog(date=_d.date(2026, 11, 2), quick="Next Monday", focus=_d.date(2026, 11, 2)))



PAGES = {
    "dashboard": page_dashboard,
    "dashboard-loading": page_dashboard_loading,
    "server-unreachable": page_unreachable,
    "sidebar-rail": page_sidebar_rail,
    "sidebar-drawer": page_sidebar_drawer,
    "user-menu": page_user_menu,
    "user-menu-dark": page_user_menu_dark,
    "toast-success": lambda: page_toast("ok"),
    "toast-warning": lambda: page_toast("warn"),
    "toast-error": lambda: page_toast("bad"),
    "forbidden": page_forbidden,
    "not-found": page_not_found,
    "focus-rings": page_focus_rings,
    "shortcuts": page_shortcuts,
    "search-states": page_search_states,
    "topbar-default": lambda: topbar_state_page(),
    "topbar-hover": lambda: topbar_state_page("hover"),
    "topbar-focused": lambda: topbar_state_page("focus"),
    "topbar-filled": lambda: topbar_state_page(None, "Mohr 1958"),
    "topbar-busy": lambda: topbar_state_page("busy", "Mohr 1958"),
    "topbar-disabled": lambda: topbar_state_page("disabled"),
    "topbar-results": lambda: topbar_state_page("focus-filled", "Mohr 1958", search_results_list("some")),
    "topbar-no-result": lambda: topbar_state_page("focus-filled", "Mohr 1985", search_results_list("none")),
    "queue": page_queue,
    "queue-row-hover": page_queue_hover,
    "queue-row-focus": page_queue_focus,
    "queue-selected": page_queue_selected,
    "queue-row-menu": page_queue_menu,
    "queue-side-panel": page_queue_side_panel,
    "queue-filtered": page_queue_filtered,
    "queue-deferred": page_queue_deferred,
    "queue-loading": page_queue_loading,
    "queue-unreachable": page_queue_unreachable,
    "queue-empty": page_queue_empty,
    "queue-no-match": page_queue_no_match,
    "compare": lambda: page_compare(),
    "compare-reason-missing": lambda: page_compare("missing"),
    "compare-submitting": lambda: page_compare("submitting"),
    "compare-done": page_compare_done,
    "compare-stale": lambda: page_compare("stale"),
    "compare-revealed": lambda: page_compare(revealed=True),
    "compare-not-a-match": page_compare_not_a_match,
    "compare-defer": page_compare_defer,
    "decision-bar-states": page_decision_states,
    "search": lambda: page_search(),
    "search-demographics": lambda: page_search(demographics=True),
    "patient-identifiers": lambda: page_patient_identifiers(),
    "patient-reveal-busy": lambda: page_patient_identifiers("busy"),
    "patient-revealed": lambda: page_patient_identifiers("revealed"),
    "patient-reveal-expired": lambda: page_patient_identifiers("expired"),
    "patient-copy-done": lambda: page_patient_identifiers("copied"),
    "patient-actions-menu": lambda: page_patient_identifiers(menu_open=True),
    "patient-merge-picker": page_merge_picker,
    "patient-unmerge": page_unmerge,
    "patient-resolve": page_resolve_dialog,
    "patient-loading": page_patient_loading,
    "patient-unreachable": page_patient_unreachable,
    "reveal": page_reveal,
    "patient-records": page_patient_records,
    "patient-history": lambda: page_patient_history(),
    "patient-history-filtered": lambda: page_patient_history(True),
    "patient-pseudonyms": page_patient_pseudonyms,
    "tabs-states": page_tabs_states,
    "tabs-hover-focus": page_tabs_hover_focus,
    "tabs-scrolled": page_tabs_scrolled,
    "merge-preview": page_merge,
    "merge-confirm": page_merge_confirm,
    "merge-success": page_merge_success,
    "merge-failure": page_merge_failure,
    "domains": page_domains,
    "domain-new": page_domain_new,
    "domain-edit": page_domain_edit,
    "domain-retire": page_domain_retire,
    "domain-retired": page_domain_retired,
    "domain": page_domain,
    "pseudonym-domains": page_pseudonym_domains,
    "pseudonym-domain-edit": page_pseudonym_domain_edit,
    "matching-edit": page_matching_edit,
    "matching-invalid": page_matching_invalid,
    "matching-diff": page_matching_diff,
    "matching-apply": page_matching_apply,
    "matching-applied": page_matching_applied,
    "matching-restart": page_matching_restart,
    "matching-conflict": page_matching_conflict,
    "configuration": page_matching,
    "configuration-history": page_configuration,
    "configuration-weights": page_configuration_weights,
    "configuration-blocking": page_configuration_blocking,
    "configuration-rules": page_configuration_rules,
    "audit": page_audit,
    "audit-detail": page_audit_detail,
    "audit-filtered": page_audit_filtered,
    "audit-export-busy": page_audit_export_busy,
    "audit-export-done": page_audit_export_done,
    "audit-loading": page_audit_loading,
    "preferences": page_settings,
    "preferences-language-open": page_settings_language_open,
    "preferences-saved": page_settings_saved,
    "preferences-reset": page_settings_reset,
    "preferences-keyboard": page_settings_keyboard,
    "preferences-account": page_settings_account,
    "preferences-dark": page_settings_dark,
    "sign-in": page_sign_in,
    "sign-in-redirecting": page_sign_in_redirecting,
    "sign-in-wrong": page_sign_in_wrong,
    "sign-in-locked": page_sign_in_locked,
    "sign-in-expired": page_sign_in_expired,
    "pseudonym-domain-new": page_pseudonym_domain_new,
    "domain-reinstate": page_domain_reinstate,
    "patient-unmerge-confirm": page_unmerge_confirm,
    "patient-unmerge-done": page_unmerge_done,
    "patient-reveal-refused": page_reveal_refused,
    "patient-resolve-answer": page_resolve_answer,
    "compare-link-merge": page_compare_link_merge,
    "queue-bulk-defer": page_queue_bulk_defer,
    "search-no-match": page_search_no_match,
    "search-loading": page_search_loading,
    "search-refused": page_search_refused,
    "matching-restore": page_matching_restore,
    "configuration-compare": page_configuration_compare,
    "configuration-import": page_configuration_import,
    "audit-no-match": page_audit_no_match,
    "about": page_about,
    "sources": page_sources,
    "configuration-file-mode": page_configuration_file_mode,
    "configuration-mode-dialog": page_configuration_mode_dialog,
    "domain-new-no-sources": page_domain_new_no_sources,
    "dashboard-empty": page_dashboard_empty,
    "date-picker-states": page_date_picker_states,
    "audit-range-picker": page_audit_range_picker,
    "date-picker-sheet": page_date_picker_sheet,
    "compare-defer-picker": page_compare_defer_picker,
    "configuration-file-mode-dialog": page_configuration_file_mode_dialog,
    "matching-file-mode": page_matching_file_mode,
    "domains-file-mode": page_domains_file_mode,
    "source": page_source,
    "source-new": page_source_new,
}
