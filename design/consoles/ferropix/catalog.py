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
    textarea, th, timeline, titled_panel, toast, toasts, well, when, settings_tabs, settings_panel, health_dot,
)

# ---- synthetic data: one source, data.py ----

import data as D_
from data import bsn_valid, PERSONS, PAIRS, PAIR, DEFERRED_PAIR

BSN_A = "999999123"
PERSON_A = PERSONS["mohr"]["id"]
PERSON_A_SHORT = PERSONS["mohr"]["short"]
PERSON_B = PERSONS["maiden"]["id"]
PERSON_B_SHORT = PERSONS["maiden"]["short"]
PERSON_C_SHORT = PERSONS["mohr-a"]["short"]


def masked_phone(tail):
    return f'<span class="fui-identifier-mask" aria-label="masked, ends in {tail}"><span class="fui-identifier-dots" aria-hidden="true">••••••</span>{tail}</span>'


MASKED_PHONE = masked_phone("4421")


def person_href(key):
    p = PERSONS.get(key)
    return (p["page"] + ".html") if p and p.get("page") else None


def patient_link(key_or_name):
    """A patient's name, linked to its own page when it has one."""
    p = PERSONS.get(key_or_name)
    if not p:
        return e(key_or_name)
    h = person_href(key_or_name)
    return f'<a href="{h}">{e(p["name"])}</a>' if h else e(p["name"])


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


# ---- the queue: grade, then score, then the longest wait (architecture
# 5.3). A certain pair is linked without review (FHIR R4 match-grade), and
# nothing under possible_at 0.60 is queued, so the queue holds probable and
# possible pairs only ----

GRADE_TONE = {"probable": "info", "possible": "warn", "certain": "ok"}
MOHR_I = D_.position("mohr") - 1
SORT_WORDS = "Sorted by grade, then score, then the longest wait"
FIND_WORDS = "Identifier, name or birth date"


def pair_href(p):
    return (p["slug"] + ".html") if p.get("slug") else None


def row_menu(p, open_=False, focus=None):
    a, b = p["names"]
    trigger = (
        f'<button type="button" class="fui-icon-button{" is-pressed" if open_ else ""}" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
        f'aria-label="Actions for {e(a)} and {e(b)}" title="More actions">{icon("ellipsis", "16")}</button>'
    )
    m = ""
    if open_:
        m = menu([
            menu_item("Open", "arrow-right", keys="Enter", href=pair_href(p)),
            menu_item("Open in a side panel", "panel-right", keys="o", href="queue-side-panel.html", focus=focus == "panel"),
            menu_item("Defer…", "clock", keys="d", href="compare-defer.html" if p["key"] == "mohr" else None),
        ], label=f"Actions for {a} and {b}")
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def queue_rows(focused=None, hover=None, kb_focus=None, selected=(), menu_open=None, rows=None, checks=True, selecting=False):
    out = []
    for i, p in enumerate(rows if rows is not None else D_.PAGE1):
        a, b = p["names"]
        attrs = ""
        cls = []
        deferred = p.get("back")
        if deferred:
            cls.append("is-excluded")
        if i == focused:
            attrs += ' aria-current="true"'
        if i == hover:
            cls.append("mk-hover")
        if i == kb_focus:
            cls.append("mk-focus")
        if i in selected:
            attrs += ' aria-selected="true"'
        c = f' class="{" ".join(cls)}"' if cls else ""
        tone = "neutral" if deferred else GRADE_TONE[p["grade"]]
        gp = '<span class="fui-muted">Deferred</span>' if deferred else pill(GRADE_TONE[p["grade"]], p["grade"])
        box = (
            f'<td class="fui-row-check" data-role="wide"><input type="checkbox" aria-label="Select {e(a)} and {e(b)}"{" checked" if i in selected else ""} /></td>'
            if checks else ""
        )
        href = pair_href(p)
        title_a = f'<a class="fui-row-link" href="{href}">{e(a)}</a>' if href else e(a)
        if selecting:
            title_a = (f'<input type="checkbox" class="fui-narrow-only" aria-label="Select {e(a)} and {e(b)}"{" checked" if i in selected else ""} /> ' + title_a)
        wait = (f'<span class="fui-time" title="Deferred: back in the queue on {p["back"]}">back {p["back"]}</span>' if deferred
                else f'<span class="fui-time">{p["wait"]}</span>')
        out.append(
            f'<tr data-key="cand-{p["key"]}"{attrs}{c}>' + box
            + f'<th scope="row" data-role="title">{title_a} <span class="fui-muted">{p["a"]["values"].get("born", "1966")[:4] if p["a"]["values"] else "1966"}</span></th>'
            + f'<td data-label="and" data-role="title-2" title="{e(b)}"><span>{e(b)} <span class="fui-muted">{p["b"]["values"].get("born", "1966")[:4] if p["b"]["values"] else "1966"}</span></span></td>'
            + td("Domains", f'{tag(p["a"]["domain"])} {tag(p["b"]["domain"])}', role="wide")
            + td("Score", score_cell(p["score"], tone), role="meta-1")
            + td("Grade", gp, role="meta-2")
            + td("Waiting", wait, "fui-num", role="meta-back" if deferred else "meta-3")
            + td("", row_menu(p, open_=(i == menu_open)), "fui-row-menu", role="menu")
            + "</tr>"
        )
    return out


def queue_head(checks=True, some_checked=False):
    mixed = ' aria-checked="mixed" data-indeterminate' if some_checked else ""
    box = f'<th scope="col" class="fui-row-check"><input type="checkbox" aria-label="Select every candidate on this page"{mixed} /></th>' if checks else ""
    return (
        box + th("Record A") + th("Record B") + th("Domains") + th("Score", num=False)
        + th("Grade", "grade", "descending", title=SORT_WORDS)
        + th("Waiting", num=True) + th('<span class="fui-sr-only">Actions</span>')
    )


INDETERMINATE = "document.querySelectorAll('[data-indeterminate]').forEach(function (c) { c.indeterminate = true; });"


def grade_items(grade="all"):
    return [(w, GRADE_TONE[w], D_.GRADE_COUNTS[w], grade in ("all", w)) for w in ("probable", "possible")]


def queue_tools(grade="all", domain="all", age="any", text="", deferred=False, open_=None):
    gw = None if grade == "all" else grade
    aw = {"any": None, "7": "longer than 7 days", "30": "longer than 30 days"}[age]
    dw = None if domain == "all" else domain
    toggle = f'<button type="button" class="fui-chip-check" aria-pressed="{"true" if deferred else "false"}">{icon("clock", "14")}Include deferred</button>'
    return tpl.toolbar(FIND_WORDS, text,
                       [tpl.filter_button("Grade", gw, open_ == "grade", grade_items(grade)), tpl.filter_button("Domain", dw), tpl.filter_button("Waiting", aw), toggle])


def filter_chips(items):
    return tpl.active(items)


def queue_count(total=D_.QUEUE_TOTAL):
    n = len(D_.LONG_WAITS)
    return f"{total} candidates · {n} waiting longer than 7 days"


def worklist(rows, head, foot_total=D_.QUEUE_TOTAL, selection=None, sort_note=True, count=None):
    hint = shortcut_hint([("j k | up down", "move"), ("Enter", "open"), ("o", "side panel"), ("d", "defer"), ("?", "all shortcuts")])
    pg = pager("Candidates", foot_total, 10) if foot_total else ""
    note = f'<p class="fui-sort-note">{SORT_WORDS}</p>' if sort_note else ""
    foot = f'<div class="fui-worklist-foot">{hint}{pg}</div>'
    sel = selection or f'<div class="fui-worklist-head"><p role="status">{count or queue_count()}</p>{note}</div>'
    return (
        '<div class="fui-card fui-panel fui-worklist" role="grid" aria-label="Candidates">'
        + sel
        + table(f"Candidates: {SORT_WORDS.lower()}", head, rows, scroll_label="Candidates")
        + foot
        + "</div>"
    ), note


QUEUE_SUB = "Pairs of records the matcher could not decide. A row is one pair; open it to compare and decide."


def queue_page(rows_html=None, tools=None, chips="", selection=None, overlay=None, toast_html="", total=D_.QUEUE_TOTAL,
               queue=D_.QUEUE_TOTAL, body=None, side=None, count=None, head=None, script=""):
    heading = page_heading("Review queue", subtitle=QUEUE_SUB)
    tools = tools if tools is not None else queue_tools()
    if body is None:
        wl, note = worklist(rows_html if rows_html is not None else queue_rows(), head or queue_head(), total, selection, count=count)
        body = wl
    content = heading + tpl.list_body("", tools, chips, body)
    if side:
        overlay = side
    return shell("Review queue", "queue", content, overlay=overlay, toast_html=toast_html, queue=queue, script=script)


def page_queue():
    return queue_page()


def page_queue_hover():
    return queue_page(queue_rows(hover=MOHR_I))


def page_queue_focus():
    return queue_page(queue_rows(focused=MOHR_I, kb_focus=MOHR_I))


SELECTED = (3, 4)


def selection_bar(open_=False):
    names = [PAIRS[i]["names"] for i in SELECTED]
    pop = defer_popover(len(SELECTED)) if open_ else ""
    trigger = button("secondary", "Defer…", "clock", small=True, attrs=f'aria-haspopup="dialog" aria-expanded="{"true" if open_ else "false"}"')
    return (
        f'<div class="fui-selection-bar" role="status"><span>{len(SELECTED)} candidates selected</span>'
        f'<div class="fui-actions"><span class="fui-anchor">{trigger}{pop}</span>{button("ghost", "Clear the selection", "x", small=True)}</div></div>'
    )


def page_queue_selected():
    return queue_page(queue_rows(selected=SELECTED, selecting=True), selection=selection_bar(), head=queue_head(some_checked=True), script=INDETERMINATE)


def page_queue_menu():
    return queue_page(queue_rows(menu_open=MOHR_I, hover=MOHR_I))


def page_queue_side_panel():
    p = PAIR["mohr"]
    a, b = p["a"]["values"], p["b"]["values"]
    panel = (
        '<aside class="fui-side-panel mk-panel-open" aria-labelledby="side-title">'
        f'<div class="fui-side-panel-head"><h2 id="side-title" class="fui-card-title">{e(p["names"][0])} and {e(p["names"][1])}</h2>'
        f'{icon_button("x", "Close the panel")}</div>'
        '<div class="fui-side-panel-body">'
        + f'<p class="fui-cluster">{score_cell(p["score"], "info")} {pill("info", p["grade"])} <span class="fui-muted">waiting {p["wait"]}</span></p>'
        + definitions([
            ("Family name", f'{a["family"]} <span class="fui-muted">and</span> {b["family"]} {mark_differs()}'),
            ("Given names", f'{a["given"]}, both'),
            ("Birth date", f'{a["born"]}, both'),
            ("Postal code", f'{a["postal"]}, both'),
            ("Street", f'{a["street"]} <span class="fui-muted">and</span> {b["street"].split()[-1]} {mark_differs()}'),
            ("Domains", f'{tag(p["a"]["domain"])} {tag(p["b"]["domain"])}'),
        ])
        + f'<p class="fui-small fui-muted">Compared because both share the {p["paired_by"]}, the blocking key.</p>'
        + f'<div class="fui-actions">{link_button("primary", "Open to decide", pair_href(p), "arrow-right", keys="Enter")}{link_button("secondary", "Defer…", "compare-defer.html", "clock", keys="d")}</div>'
        + "</div></aside>"
    )
    return queue_page(queue_rows(focused=MOHR_I), side=panel)


def mark_differs():
    return f'<span class="fui-compare-mark fui-compare-differ">{icon("equal-not", "12")}<span>differs</span></span>'


def page_queue_filtered():
    rows = [p for p in PAIRS if p["grade"] == "probable" and p["found_days"] > 7]
    tools = queue_tools(grade="probable", age="7")
    chips = filter_chips(["Grade · probable", "Waiting · longer than 7 days"])
    return queue_page(queue_rows(rows=rows), tools=tools, chips=chips, total=len(rows), count=f"{len(rows)} of {D_.QUEUE_TOTAL} candidates")


def page_queue_deferred():
    d = DEFERRED_PAIR
    rows = [p for p in PAIRS if (D_.GRADE_RANK[p["grade"]], -p["score"]) <= (D_.GRADE_RANK[d["grade"]], -d["score"])]
    rows = rows + [d] + [p for p in D_.PAGE1 if p not in rows]
    rows = rows[:10]
    tools = queue_tools(deferred=True)
    return queue_page(queue_rows(rows=rows), tools=tools, total=D_.QUEUE_TOTAL + 1,
                      count=f"{D_.QUEUE_TOTAL + 1} candidates · 1 deferred, shown in its place by grade and score, back {d['back']}")


def page_queue_empty():
    body = empty_quiet("square-check", "The queue is empty",
                       f'Every candidate is decided; the last decision was today 09:41. <a href="dashboard.html">Go to the dashboard</a>')
    return queue_page(body=body, queue=0, tools="")


def page_queue_no_match():
    body = empty_quiet("search-x", "No candidate matches these filters",
                       f'{D_.QUEUE_TOTAL} candidates wait, but no possible pair waits longer than 30 days. <a href="queue-filtered.html">Show the probable pairs that wait longer than 7 days</a>')
    tools = queue_tools(grade="possible", age="30")
    chips = filter_chips(["Grade · possible", "Waiting · longer than 30 days"])
    return queue_page(tools=tools, chips=chips, body=body)


def page_queue_loading():
    rows = "".join(
        '<tr aria-hidden="true"><td class="fui-row-check" data-role="wide"><span class="fui-skeleton mk-w100 mk-sk-box"></span></td>'
        f'<th data-role="title">{skeleton_lines([70])}</th><td data-role="title-2">{skeleton_lines([60])}</td>'
        f'<td data-role="wide">{skeleton_lines([80])}</td><td data-role="meta-1">{skeleton_lines([70])}</td>'
        f'<td data-role="meta-2">{skeleton_lines([50])}</td><td data-role="meta-back">{skeleton_lines([40])}</td><td></td></tr>'
        for _ in range(8)
    )
    body = (
        '<div class="fui-card fui-panel fui-worklist" aria-busy="true" aria-label="Candidates, loading">'
        '<div class="fui-worklist-head"><p role="status"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Loading the candidates</span></p></div>'
        + table("Candidates, loading", queue_head(), [rows], scroll_label="Candidates")
        + "</div>"
    )
    return queue_page(body=body, queue=D_.QUEUE_TOTAL)


def unreachable_refusal(what):
    return (
        refusal(f"The FerroPIX server did not answer, so {what} cannot be shown.")
        + '<p class="mk-refusal-line">The connection was refused at 09:42:11 today. Your session is kept; nothing you entered is lost.</p>'
    )


def tech_detail():
    return disclosure(
        "Technical detail",
        definitions([
            ("Failure", "The server refused the connection"),
            ("Request id", f"<code>{D_.UNREACHABLE_RID}</code>"),
            ("Time", when("2026-10-21T09:42:11", seconds=True)),
            ("Attempts", "3 of 3, the last 4 seconds ago"),
        ]),
    )


def try_again_line():
    return f'<div class="fui-cluster">{button("secondary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried {when("2026-10-21T09:42:11", seconds=True)}</span></div>'


def page_queue_unreachable():
    body = (
        '<div class="fui-stack">'
        + refusal("The queue cannot be shown: the FerroPIX server refused the connection.")
        + try_again_line()
        + tech_detail()
        + "</div>"
    )
    return shell("Review queue", "queue", page_heading("Review queue", subtitle=QUEUE_SUB) + f'<div class="fui-stack">{body}</div>',
                 status=("bad", "Unreachable"), queue=None)


# ---- compare and decide: one page per pair, from data.py ----

FIELDS = [
    ("family", "Family name"), ("given", "Given names"), ("born", "Birth date"), ("gender", "Gender"),
    ("postal", "Postal code"), ("street", "Street"), ("phone", "Telephone"), ("identifier", "Identifier"),
]
VERDICT_WORDS = {"agree": "agree", "differ": "differ", "absent": "absent"}


def pair_record(p, side, revealed=False):
    r = p[side]
    letter = side.upper()
    vals = dict(r["values"])
    if vals.get("phone"):
        vals["phone"] = masked_phone(vals["phone"])
    if revealed and side == "b":
        vals["identifier"] = identifier(r["domain"], r["tail"], revealed=f"LAB-0077-{r['tail']}", left=24)
    else:
        vals["identifier"] = identifier(r["domain"], r["tail"], reveal_id="reveal")
    return {"key": side, "title": r["name"], "short": f"{letter} · {r['name'].split(',')[0]}", "letter": letter,
            "sub": f"{r['domain']} record · last updated " + when(r["updated"]), "values": vals}


def pair_verdict(p):
    def v(f, i):
        if f == "identifier":
            return None
        return p["fields"][f][0]
    return v


def pair_note(p):
    marks = [p["fields"][k][0] for k in D_.FIELD_ORDER]
    d, a, x = marks.count("differ"), marks.count("agree"), marks.count("absent")
    absent_b = [k for k in D_.FIELD_ORDER if p["fields"][k][0] == "absent" and p["fields"][k][1] == "absent in B"]
    parts = [f"{d} field{'s' if d != 1 else ''} differ{'s' if d == 1 else ''}", f"{a} agree"]
    if x:
        both = x - len(absent_b)
        if absent_b:
            parts.append(f"{len(absent_b)} {'is' if len(absent_b) == 1 else 'are'} absent in B")
        if both:
            parts.append(f"{both} {'is' if both == 1 else 'are'} absent in both")
    return ", ".join(parts[:-1]) + (" and " if len(parts) > 1 else "") + parts[-1] + ". The differing fields come first."


def pair_header(p):
    def rec(letter, side):
        r = p[side]
        person = PERSONS.get(r.get("person"))
        if person:
            others = [i for i in person["identifiers"] if i[0] != r["domain"]]
            oth = " ".join(identifier(d, t, static=True) for d, t, *_ in others) or '<span class="fui-muted">none</span>'
            pid = person["short"]
            name = patient_link(r["person"])
        else:
            oth = '<span class="fui-muted">none</span>'
            pid = f"0199{abs(hash(r['name'])) % 0xffff:04x}⁠…⁠{r['tail']}0"
            name = e(r["name"])
        return (
            f'<div class="mk-pair-record"><span class="mk-pair-name"><span class="mk-pair-letter" aria-hidden="true">{letter}</span>{name}'
            f'<span class="fui-muted mk-pair-domain">{r["domain"]} record</span></span>'
            f'<span class="mk-pair-facts"><span class="fui-small fui-muted">Person</span>{chip(pid)}<span class="fui-small fui-muted">Other identifiers</span>{oth}</span></div>'
        )
    return '<div class="mk-pair">' + rec("A", "a") + rec("B", "b") + "</div>"


def score_card(p):
    rows = "".join(score_bar_row(label, reading, w) for _, label, reading, w in sorted(p["rows"], key=lambda r: -r[3]))
    sign = "+" if p["total"] >= 0 else "−"
    total = (
        f'<div class="fui-score-total"><span>Total weight <span class="fui-muted">{sign}{abs(p["total"]):.1f}</span></span>'
        f'<span>Score <span class="fui-score-figure">{p["score"]:.2f}</span> ' + pill(GRADE_TONE[p["grade"]], p["grade"]) + "</span></div>"
    )
    words = ", ".join(f"{label.lower()} {'+' if w > 0 else ('−' if w < 0 else '')}{abs(w):g}" for _, label, _, w in p["rows"])
    scale = score_scale(p["score"], f"Score {p['score']:.2f}, {p['grade']}, on the scale: possible from 0.60, probable from 0.80, certain from 0.95. {words}.")
    return card("Score breakdown", f'<div class="fui-score">{rows}</div>{total}{scale}'
                f'<p class="fui-small fui-muted">Compared because both share the {p["paired_by"]}, the blocking key. '
                'Weights as set in <a href="configuration-weights.html">Configuration</a>, revision 7.</p>')


DECISIONS = [("primary", "Link", "link", "l"), ("secondary", "Not a match", "circle-x", "n"), ("secondary", "Defer", "clock", "d")]
MOHR_REASON = "Maiden name; same telephone and address history"


def next_after(p, removed=True):
    i = D_.position(p["key"])
    return PAIRS[i] if i < len(PAIRS) else None


def decision_dock(state="idle", reason=MOHR_REASON, id_="reason", message=""):
    msg = message
    err = None
    acts = ""
    if state == "missing":
        reason = ""
        err = "Write why you decided this; the reason goes into the audit trail with your name."
    for variant, label, glyph, key in DECISIONS:
        if state == "submitting":
            acts += busy_button(variant, "Linking") if label == "Link" else button(variant, label, glyph, attrs="disabled", keys=key)
        elif state in ("done", "stale"):
            acts += button(variant, label, glyph, attrs="disabled", keys=key)
        else:
            acts += button(variant, label, glyph, keys=key)
    disabled_field = state in ("submitting", "done", "stale")
    nxt = next_after(PAIR["mohr"])
    pos = D_.position("mohr")
    if state == "done":
        msg = (
            '<p class="fui-dock-message fui-dock-done" role="status">'
            f'{icon("circle-check", "16")}Linked: Maiden, Alice is now part of Mohr, Alice. '
            f'<span class="fui-spinner fui-small"><span class="fui-spinner-ring" aria-hidden="true"></span>Opening the next candidate, {pos}&nbsp;of&nbsp;{D_.QUEUE_TOTAL - 1}</span></p>'
        )
    if state == "stale":
        msg = (
            '<div class="fui-dock-message">'
            + notice("warn", "triangle-alert", "Someone decided first: Demo Reviewer linked this pair at 09:41 today, so the pair has left the queue. Your decision was not sent.",
                     action=" " + link_button("secondary", "Open the next candidate", pair_href(nxt), "arrow-right", small=True)
                     + " " + link_button("ghost", "Open the linked patient", "patient-identifiers.html", small=True))
            + "</div>"
        )
    fld = dock_field(id_, "Reason", reason, "Why you decided this", keys="r", error=err, disabled=disabled_field)
    return dock(fld, acts, msg)


def compare_body(p, revealed=False):
    cmp_ = compare("The two records, field by field, the differing fields first",
                   FIELDS, pair_record(p, "a"), [pair_record(p, "b", revealed)], pair_verdict(p), note=pair_note(p))
    main = card("Fields", cmp_)
    return f'<div class="fui-stack">{pair_header(p)}<div class="fui-main-aside"><div class="fui-main-part">{main}</div><div class="fui-aside-part">{score_card(p)}</div></div></div>'


def compare_heading(p, total=D_.QUEUE_TOTAL, pos=None, steps=True):
    i = D_.position(p["key"])
    pos = pos or i
    prev = PAIRS[i - 2] if i > 1 else None
    nxt = PAIRS[i] if i < len(PAIRS) else None
    days = p["found_days"]
    found = f"Found {int(days)} days ago" if days >= 2 else ("Found yesterday" if days >= 1 else "Found today")
    return page_heading(
        "These two records look like the same patient",
        actions=stepper(f"Candidate {pos} of {total}", pair_href(prev) if prev else "#", pair_href(nxt) if nxt else "#",
                        prev_ok=bool(prev) and steps, next_ok=bool(nxt) and steps),
        subtitle=f"{found}. Your decision is recorded with your name and reason.",
        crumbs=[("Review queue", "queue.html")],
    )


def page_compare(state="idle", overlay=None, revealed=False, toast_html="", key="mohr", queue=D_.QUEUE_TOTAL):
    p = PAIR[key]
    content = compare_heading(p, steps=state not in ("submitting", "done", "stale")) + compare_body(p, revealed)
    reason = MOHR_REASON if key == "mohr" else ""
    return shell("Compare and decide", "queue", content, overlay=overlay,
                 dock_html=decision_dock(state, reason), toast_html=toast_html,
                 queue=queue if state not in ("done",) else D_.QUEUE_TOTAL - 1)


def page_compare_done():
    return page_compare("done")


def page_compare_not_a_match():
    # The next candidate has loaded; the last decision is confirmed in the dock.
    p = next_after(PAIR["mohr"])
    pos = D_.position("mohr")
    content = compare_heading(p, total=D_.QUEUE_TOTAL - 1, pos=pos) + compare_body(p)
    msg = (f'<p class="fui-dock-message fui-dock-done" role="status">{icon("circle-check", "16")}'
           'Not a match: Mohr, Alice and Maiden, Alice. The pair is not queued again until either record changes.</p>')
    return shell("Compare and decide", "queue", content, dock_html=decision_dock("idle", "", message=msg), queue=D_.QUEUE_TOTAL - 1)


def page_shortcuts():
    legend = shortcut_legend([
        ("Everywhere", [("/", "Find a patient"), ("?", "Keyboard shortcuts"), ("Esc", "Close a dialog, a menu or a panel"), ("g d", "Go to the dashboard"), ("g q", "Go to the review queue"), ("g p", "Go to your preferences")]),
        ("Lists", [("j | down", "Next row"), ("k | up", "Previous row"), ("Enter", "Open the row"), ("o", "Open in a side panel")]),
        ("Compare and decide", [("l", "Link the records"), ("n", "Not a match"), ("d", "Defer the candidate"), ("r", "Go to the reason"),
                                ("right", "Next candidate"), ("left", "Previous candidate")]),
        ("Patient", [("m", "Merge into…")]),
    ])
    return page_dashboard(overlay=legend)


def page_decision_states():
    """The states sheet of the docked decision bar."""
    def frame(title, line, html):
        return (f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                f'<div class="mk-dock-demo">{html}</div></div>')
    states = [
        ("Idle", "The reason at the start, the decisions at the end; Link is the primary.", decision_dock("idle", id_="reason-1")),
        ("Idle, empty", "Nothing typed yet; the decisions stay enabled.", decision_dock("idle", "", id_="reason-2")),
        ("Reason missing", "A decision without a reason focuses the field with its error.", decision_dock("missing", id_="reason-3")),
        ("Submitting", "The chosen decision is busy; the others and the field wait.", decision_dock("submitting", id_="reason-4")),
        ("Done", "The bar confirms in words, and the next candidate loads.", decision_dock("done", id_="reason-5")),
        ("Stale, 409", "Someone decided first: nothing is sent, and the bar offers the next candidate.", decision_dock("stale", id_="reason-6")),
    ]
    sheet = "".join(frame(*s) for s in states)
    keys = card("Keys and focus", extra_cls="fui-wide-only", body=definitions([
        ("L, N, D", "Decide; each acts only when no field has the focus, and shows as a quiet cap after its label at wide widths."),
        ("R", "Moves the focus to the reason."),
        ("Enter in the reason", "Does nothing on its own: a decision is always one deliberate key or click."),
        ("← and →", "The previous and next candidate, in the page header beside the position."),
        ("Phone", "The reason on its own row, the three decisions sharing the row under it; no key hints."),
    ]))
    content = page_heading("Decision bar, states", subtitle="Docked to the foot of the main area, or the window's foot on a phone.") + f'<div class="fui-stack mk-dock-sheet">{sheet}{keys}</div>'
    return shell("Decision bar states", "", content)


# ---- dashboard ----

def oldest_rows():
    rows = []
    for p in D_.OLDEST:
        a, b = p["names"]
        pair = f"{a} and {b}"
        rows.append(
            f'<tr><th scope="row"><a class="fui-truncate mk-pair-line" href="{pair_href(p)}" title="{e(pair)}">{e(a)} <span class="fui-muted">and</span> {e(b)}</a></th>'
            + td("Grade", pill(GRADE_TONE[p["grade"]], p["grade"])) + td("Waiting", p["wait"], "fui-num") + "</tr>"
        )
    return rows


def dashboard_heading():
    return page_heading("Dashboard", subtitle="Wednesday 21 October 2026 · the day's state of the index.")


def up_state(word, detail=None, tone="ok"):
    d = f'<span class="fui-status-detail">{e(detail)}</span>' if detail else ""
    return f'<span class="fui-status fui-tone-{tone}">{health_dot(tone)}{e(word)}{d}</span>'


def page_dashboard(search_state=None, search_value="", results=None, menu_open=False, menu_theme="light",
                   rail=False, rail_tip=None, drawer=False, toast_html="", queue=D_.QUEUE_TOTAL, overlay=None, backlog=0, status=("ok", "Server up")):
    pct = round(D_.AUTO_LINKED / D_.FED_TODAY * 100)
    stats = (
        '<div class="fui-grid fui-grid-stats">'
        + stat_card("inbox", f"{D_.QUEUE_TOTAL}", "Waiting in the queue", badge("warn", f"{len(D_.LONG_WAITS)} longer than 7 days"), href="queue.html")
        + stat_card("link", f"{pct} %", "Linked without review, today", badge("neutral", f"{D_.AUTO_LINKED:,} of {D_.FED_TODAY:,} records"))
        + stat_card("activity", f"{D_.FED_TODAY:,}", "Records fed today", badge("neutral", f"FHIR {D_.FED_FHIR} · HL7 v2 {D_.FED_V2}"))
        + (stat_card("bell", "1", "Subscriptions behind", badge("warn", f"{backlog} notifications wait")) if backlog else stat_card("bell", "0", "Subscriptions in error", pill("ok", "7 delivering")))
        + "</div>"
    )
    peak_f = max(D_.FEEDS_FHIR)
    peak_v = max(D_.FEEDS_V2)
    charts = (
        '<div class="fui-grid">'
        + card("Queue depth, 30 days", line_chart("depth", "Queue depth over 30 days", f"Between {min(D_.QUEUE_DEPTH)} and {max(D_.QUEUE_DEPTH)} candidates waiting; {D_.QUEUE_TOTAL} today.", [("Waiting", D_.QUEUE_DEPTH, 0)], [(0, "22 Sep"), (15, "7 Oct"), (29, "Today")]))
        + card("Feeds per hour, today", line_chart("feeds", "Feeds per hour by channel", f"FHIR peaks at {peak_f} at 08:00 and HL7 v2 at {peak_v}; {D_.FED_FHIR} and {D_.FED_V2} so far today.", [("FHIR", D_.FEEDS_FHIR, 0), ("HL7 v2", D_.FEEDS_V2, 1)], [(0, "00:00"), (3, "03:00"), (6, "06:00"), (9, "09:00")]))
        + "</div>"
    )

    def listener(name, st, lines):
        return f'<div class="mk-listener"><div class="mk-listener-line"><strong>{name}</strong>{st}</div><div class="fui-small fui-muted">{lines}</div></div>'

    sender = (up_state("Behind", f"{backlog} queued", "warn") if backlog else up_state("0 queued", "last sent today 09:41"))
    sender_line = (f"7 subscriptions · ehr-research is not answering; retrying every 30 s" if backlog
                   else "7 subscriptions · ITI-93 notifications · 0 retries today")
    listeners = card(
        "Listeners",
        '<div class="mk-listeners">'
        + listener("HTTP face", up_state("Up", "2 ms p50 · 11 ms p99"), "FHIR R4 PIXm, PDQm and PMIR · 1,312 requests this hour")
        + listener("MLLP listener", up_state("Up", "last message today 09:40"), "HL7 v2 ADT · 2 senders today: his-prod, lab-lis")
        + listener("Feed sender", sender, sender_line)
        + listener("Database", up_state("Up", "pool 4 of 16"), "PostgreSQL 18 · 412,318 persons · last vacuum today 03:00")
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
                 toast_html=toast_html, queue=queue, overlay=overlay, status=status)


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
        + try_again_line()
        + card("While the server is away", '<p class="fui-muted">This console keeps no patient data, so no page can be drawn until the server answers. Your session is kept: you do not need to sign in again.</p>')
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
        people = ["mohr", "alissa"]
        items = ""
        for i, k in enumerate(people):
            p = PERSONS[k]
            d, t = p["identifiers"][0][:2]
            items += (f'<li role="option"{" aria-selected=\"true\"" if i == 0 else ""}><a href="{person_href(k)}">'
                      f'<span class="mk-results-main"><span class="mk-results-name">{e(p["name"])}</span><br />'
                      f'<span class="fui-small fui-muted">Born {p["born"]}, {p["gender"]}</span></span>{identifier(d, t, static=True)}</a></li>')
        body = (f'<p class="mk-results-head">2 patients · name and birth year</p><ul role="listbox" aria-label="Patients found">{items}</ul>'
                f'<a href="search.html" class="mk-results-more">All results in Search{icon("arrow-right", "14")}</a>')
    return f'<div id="find-results" class="mk-results" role="region" aria-label="Search answer">{body}</div>'


def topbar_state_page(state=None, value="", results=None):
    if state == "disabled":
        return page_dashboard(search_state=state, search_value=value, results=results, status=("warn", "Index rebuilding"))
    return page_dashboard(search_state=state, search_value=value, results=results)


def page_search_states():
    from common import search_control as sc
    states = [
        ("Default", "The icon, the placeholder and the quiet / hint.", "", None),
        ("Hover", "The edge takes the ink.", "", "hover"),
        ("Focused, empty", "One ring round the whole group; the hint is gone.", "", "focus"),
        ("Focused and filled", "The ring, the typed value, the hint gone.", "Mohr 1958", "focus-filled"),
        ("Filled, not focused", "The hint stays hidden while the field holds text.", "Mohr 1958", None),
        ("Busy", "The icon becomes the spinner and Find waits while the search runs.", "Mohr 1958", "busy"),
        ("Disabled", "The sunken ground and its reason, while the index is rebuilding; the shortcut does nothing.", "", "disabled"),
    ]
    cards = ""
    for i, (title, line, value, state) in enumerate(states):
        cards += f'<div class="fui-card"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>{sc(value, state, id_=f"find-{i}")}</div>'
    legend = card("Keys", extra_cls="fui-wide-only", body=definitions([
        (kbd("/"), "Focuses the field from anywhere outside an editable element; announced through <code class='mk-nowrap'>aria-keyshortcuts</code> on the input."),
        (kbd("Esc"), "Clears the field and returns the focus to the page."),
        (kbd("Enter"), "Posts the search; the console reads the shape of what was typed."),
        (kbd("Tab"), "One stop for the field, one for the submit."),
    ]))
    content = page_heading("Search control, states", subtitle="One joined control: the icon, the field, the / hint and the submit; below 40 rem the submit keeps its icon alone.") + f'<div class="fui-stack"><div class="mk-states">{cards}</div>{legend}</div>'
    return shell("Search control states", "", content)


# ---- search ----

SEARCH_PEOPLE = {"mohr": (0.97, "certain"), "maiden": (0.91, "probable"), "alissa": (0.74, "possible")}


def search_result_rows(with_score=False):
    keys = ["mohr", "maiden", "alissa"] if with_score else ["mohr", "alissa", "maiden"]
    rows = []
    for k in keys:
        p = PERSONS[k]
        score, grade = SEARCH_PEOPLE[k]
        idents = '<span class="mk-idents">' + "".join(identifier(d, t, static=True) for d, t, *_ in p["identifiers"]) + "</span>"
        fed = max(i[2] for i in p["identifiers"])
        rows.append(
            f'<tr><th scope="row"><a href="{person_href(k)}">{e(p["name"])}</a></th>'
            + td("Born", p["born"]) + td("Gender", p["gender"] or '<span class="mk-dash" title="absent">–</span>') + td("Identifiers", idents)
            + (td("Score", score_cell(score, GRADE_TONE[grade])) + td("Grade", pill(GRADE_TONE[grade], grade)) if with_score else "")
            + td("Last fed", rel_time(fed)) + "</tr>"
        )
    return rows


SEARCH_SUB = "Find a patient by one identifier or by demographics; results stay in this page and never reach a URL."


def page_search(demographics=False, shown=False):
    seg = segmented_form("Search by", [("identifier", "By identifier"), ("demographics", "By demographics")], "demographics" if demographics else "identifier")
    if not demographics:
        eye = (f'<button type="button" class="fui-icon-button fui-input-toggle" aria-pressed="{"true" if shown else "false"}" aria-label="Show the identifier while held" '
               f'title="Show while held">{icon("eye-off" if shown else "eye", "16")}</button>')
        form = (
            '<form method="post" action="#" class="fui-form mk-search-form"><div class="fui-form-grid mk-id-grid">'
            + field("domain", "Domain", '<div class="fui-combobox"><div class="fui-combobox-box">' + input_("domain", "BSN", "Choose a domain", extra='role="combobox" aria-expanded="false" aria-autocomplete="list"', cls="fui-input fui-combobox-input") + "</div></div>")
            + field("identifier", "Identifier", '<div class="fui-input-group">' + input_("identifier", BSN_A, "", "text" if shown else "password", extra='autocapitalize="off" spellcheck="false" aria-describedby="identifier-help"') + eye + "</div>", cls="fui-span-2")
            + f'<div class="mk-grid-button">{button("primary", "Search", "search", type_="submit")}</div>'
            + "</div>"
            + '<p id="identifier-help" class="fui-help">Masked as you type and sent in the request body, never in the address bar.</p></form>'
        )
        results_head = th("Name", "name", "ascending") + th("Born") + th("Gender") + th("Identifiers") + th("Last fed")
        count = "1 patient holds this identifier · 2 more share its birth date"
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
    results = titled_panel(f'Results <span class="mk-count">· {count}</span>', "", table("Search results", results_head, rows))
    content = page_heading("Search", subtitle=SEARCH_SUB) + f'<div class="fui-stack">{card("Find a patient", seg + form)}{results}</div>'
    return shell("Search", "search", content)


# ---- patient: Mohr, Alice in three states, from data.py ----
# rest: BSN, Hospital MRN and Radiology RIS (merged in from Mohr, A. on
# 2026-06-30, the latest merge). merged: Maiden, Alice merged in today, her
# Laboratory record with it. unmerged: the merge of Mohr, A. reverted.

MERGE_AT = "2026-10-21T09:42:05"
UNMERGE_AT = "2026-10-21T09:42:07"


def mohr_identifiers(state="rest"):
    ids = list(PERSONS["mohr"]["identifiers"])
    if state == "merged":
        ids.append(("Laboratory", "550", "2026-09-02T08:15", "merge", "from Maiden, Alice, " + when(MERGE_AT)))
    if state == "unmerged":
        ids = [i for i in ids if i[0] != "Radiology RIS"]
    return ids


def patient_tabs(state="rest", counts=True):
    n_id = len(mohr_identifiers(state))
    n_hist = len(history_events(state))
    items = [("identifiers", "Identifiers", "patient-identifiers.html", n_id), ("records", "Records", "patient-records.html", n_id),
             ("history", "History", "patient-history-merged.html" if state == "merged" else "patient-history.html", n_hist), ("pseudonyms", "Pseudonyms", "patient-pseudonyms.html", len(D_.MOHR_PSEUDONYMS))]
    return items if counts else [t[:3] for t in items]



def patient_actions(menu_open=False, state="rest", offline=False):
    items = []
    if state == "rest":
        items = [menu_item("Unmerge Mohr, A.…", "undo", href="patient-unmerge.html", danger=True)]
    merge = (button("primary", "Merge into…", "merge", attrs='disabled title="The server did not answer"', keys="m") if offline
             else link_button("primary", "Merge into…", "patient-merge-picker.html", "merge", keys="m"))
    return action_group(merge, items, open_=menu_open, label="More actions for Mohr, Alice")


def patient_subtitle(key="mohr", state="rest"):
    p = PERSONS[key]
    n = len(mohr_identifiers(state)) if key == "mohr" else len(p["identifiers"])
    born = f"Born {p['born']}" + (f", {p['gender']}" if p["gender"] else "")
    recs = f"{n} active record{'s' if n != 1 else ''} in {n} domain{'s' if n != 1 else ''}"
    return f'<span class="fui-cluster"><span>{born}</span><span>{recs}</span>{person_chip(p["short"], p["id"])}</span>'


def patient_heading(menu_open=False, state="rest", offline=False):
    return page_heading("Mohr, Alice", actions=patient_actions(menu_open, state, offline), subtitle=patient_subtitle("mohr", state), crumbs=[("Search", "search.html")])


def patient_page(tab, panel, overlay=None, menu_open=False, focus=None, hover=None, scrolled=False, toast_html="", title="Mohr, Alice", script="",
                 state="rest", queue=D_.QUEUE_TOTAL, counts=True):
    content = patient_heading(menu_open, state) + tabbed(tabs("Patient", patient_tabs(state, counts), tab, focus=focus, hover=hover, scrolled=scrolled), panel)
    return shell(title, "search", content, overlay=overlay, toast_html=toast_html, script=script, queue=queue)


def ident_cell(d, t, mode="masked", first=False):
    if not first:
        return identifier(d, t, reveal_id="reveal", show_domain=False)
    if mode == "revealed":
        return identifier(d, t, revealed="999 999 123", left=27, show_domain=False)
    if mode == "copied":
        return identifier(d, t, revealed="999 999 123", left=21, show_domain=False, copied=True)
    if mode == "busy":
        return identifier(d, t, state="busy", show_domain=False)
    if mode == "expired":
        return identifier(d, t, reveal_id="reveal", state="expired", show_domain=False)
    if mode == "refused":
        return (identifier(d, t, reveal_id="reveal", show_domain=False)
                + f'<span class="fui-help fui-error mk-row-error" role="status">{icon("circle-alert", "14")} Not revealed: the hourly limit is reached; try again at 10:05.</span>')
    return identifier(d, t, reveal_id="reveal", show_domain=False)


def identifier_rows_of(ids, mode="masked"):
    rows = []
    for i, (d, t, fed, by, note) in enumerate(ids):
        rows.append(f'<tr><th scope="row">{e(d)}</th>' + td("Identifier", ident_cell(d, t, mode, i == 0)) + td("State", pill("ok", "active"))
                    + td("Last fed", rel_time(fed)) + td("Linked by", f'{by} <span class="fui-muted">{note}</span>') + "</tr>")
    return rows


def identifier_rows(bsn="masked", state="rest"):
    return identifier_rows_of(mohr_identifiers(state), bsn)


def identifiers_panel(bsn="masked", state="rest", name="Mohr, Alice", ids=None):
    live = {"revealed": '<p class="fui-sr-only" role="status">BSN revealed for 30 seconds.</p>',
            "expired": '<p class="fui-sr-only" role="status">The BSN is hidden again.</p>'}.get(bsn, "")
    rows = identifier_rows_of(ids, bsn) if ids is not None else identifier_rows(bsn, state)
    return titled_panel(
        "Identifiers <span class=\"mk-count\">· one per domain</span>",
        '<span class="fui-small fui-muted">A reveal shows the value for 30 seconds and is recorded.</span>',
        table(f"Identifiers of {name}", th("Domain") + th("Identifier") + th("State") + th("Last fed") + th("Linked by"), rows),
    ) + live


def page_patient_identifiers(bsn="masked", overlay=None, menu_open=False, toast_html="", state="rest", queue=D_.QUEUE_TOTAL):
    return patient_page("identifiers", identifiers_panel(bsn, state), overlay=overlay, menu_open=menu_open, toast_html=toast_html, state=state, queue=queue)


def page_person(key):
    """A patient with one or two records: no tabs, the identifiers only."""
    p = PERSONS[key]
    content = page_heading(p["name"], subtitle=patient_subtitle(key), crumbs=[("Search", "search.html")]) + identifiers_panel(name=p["name"], ids=p["identifiers"])
    return shell(p["name"], "search", content)


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
        '<p class="fui-muted">Find the other patient; the next step previews the merge and writes nothing.</p>'
        '<div class="fui-modal-body">'
        + field("merge-find", "Patient", '<div class="fui-input-group">' + input_("merge-find", "Maiden", extra='role="combobox" aria-expanded="true" aria-controls="merge-options"') + "</div>", help_="A name, a birth date or a person id.")
        + '<ul id="merge-options" role="listbox" class="mk-pick-list" aria-label="Patients found">'
        + f'<li role="option" aria-selected="true"><span class="mk-pick-main"><strong>Maiden, Alice</strong><span class="fui-small fui-muted">1958-01-30 · 1 record</span></span>{identifier("Laboratory", "550", static=True)}{chip(PERSON_B_SHORT, PERSON_B)}</li>'
        + f'<li role="option"><span class="mk-pick-main"><strong>Maiden, Alicia</strong><span class="fui-small fui-muted">1961-08-14 · 2 records</span></span>{identifier("Hospital MRN", "312", static=True)}{chip("0199b0e3" + chr(0x2060) + "…" + chr(0x2060) + "f006")}</li>'
        + "</ul></div>"
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", "x")}{link_button("primary", "Preview the merge", "merge-preview.html", "arrow-right")}</div></div>'
    )
    return page_patient_identifiers(overlay=dialog)


def page_patient_loading():
    rows = "".join(
        f'<tr aria-hidden="true"><th>{skeleton_lines([70])}</th><td>{skeleton_lines([60])}</td><td>{skeleton_lines([40])}</td><td>{skeleton_lines([70])}</td><td>{skeleton_lines([80])}</td></tr>'
        for _ in range(3)
    )
    panel = ('<p class="fui-sr-only" role="status">Loading the identifiers</p>'
             + titled_panel("Identifiers", "", table("Identifiers, loading", th("Domain") + th("Identifier") + th("State") + th("Last fed") + th("Linked by"), [rows])))
    return patient_page("identifiers", panel, counts=False)


def page_patient_unreachable():
    panel = (
        '<div class="fui-stack">'
        + refusal("The identifiers cannot be shown: the FerroPIX server refused the connection.")
        + try_again_line()
        + tech_detail() + "</div>"
    )
    content = patient_heading(offline=True) + tabbed(tabs("Patient", patient_tabs(counts=False), "identifiers"), panel)
    return shell("Mohr, Alice", "search", content, status=("bad", "Unreachable"), queue=None)


def rec(key, title, sub, ident, family="Mohr", given="Alice", gender="female", street="Voorbeeldstraat 12"):
    return {"key": key, "title": title, "short": title, "letter": title, "sub": sub,
            "values": {"identifier": ident, "family": family, "given": given, "born": "1958-01-30", "gender": gender,
                       "postal": "9999 AB", "street": street, "phone": MASKED_PHONE}}


PATIENT_FIELDS = [("identifier", "Identifier")] + [f for f in FIELDS if f[0] != "identifier"]
RIS_VERDICT = {"given": "differ"}


def page_patient_records():
    lead = rec("bsn", "BSN", "last updated " + when("2026-10-20T09:41") + " · brp-feed", identifier("BSN", "123", reveal_id="reveal", show_domain=False))
    mrn = rec("mrn", "Hospital MRN", "last updated " + when("2026-10-19T14:03") + " · his-prod", identifier("Hospital MRN", "994", reveal_id="reveal", show_domain=False))
    ris = rec("ris", "Radiology RIS", "last updated " + when("2026-06-30T11:20") + " · merged from Mohr, A.", identifier("Radiology RIS", "077", reveal_id="reveal", show_domain=False), given="A.")

    def verdict(f, i):
        if f == "identifier":
            return None
        return "agree" if i == 0 else RIS_VERDICT.get(f, "agree")
    cmp_ = compare(
        "The source records side by side, each field marked against the BSN record",
        PATIENT_FIELDS, lead, [mrn, ris], verdict, differ_first=False, shown=1,
        note="Marked against the BSN record, the leading record. The Hospital MRN record agrees in every field; the Radiology RIS record has only the initial of the given name.",
    )
    return patient_page("records", card("Records, side by side", cmp_))


def change_del(v):
    return f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-sr-only">was </span><span class="fui-change-text">{v}</span></del>'


def change_ins(v):
    return f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span><span class="fui-sr-only">now </span>{v}</ins>'


def change_plain(before, after):
    return (f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-sr-only">was </span><span class="fui-change-text">{e(before)}</span></del> '
            f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span><span class="fui-sr-only">now </span>{e(after)}</ins>')


HISTORY_KINDS = ["link", "reveal", "view", "version", "auto-link", "merge", "unmerge"]


def history_events(state="rest"):
    version_body = definitions([
        ("Postal code", change_plain("9999 AC", "9999 AB")),
        ("Street", change_plain("Voorbeeldstraat 10", "Voorbeeldstraat 12")),
        ("Source", "brp-feed · ITI-104 Add or revise"),
        ("Request id", f"<code>{D_.RID_HISTORY_VERSION}</code>"),
    ])
    latest = state == "rest"
    old_merge = ('<p class="fui-small">Reversible: this is the latest merge. 2 subscribers told (ITI-93 merge).</p>' if latest
                 else '<p class="fui-small">No longer reversible: a later merge followed. 2 subscribers told (ITI-93 merge).</p>' if state == "merged"
                 else '<p class="fui-small">Reverted today 09:42.</p>')
    ev = [
        {"at": "2026-10-21T09:41:55", "kind": "reveal", "tone": "warn", "actor": "Demo Steward",
         "title": "Revealed the BSN for 30 seconds", "reason": "Phone call with the laboratory about sample LAB-…550"},
        {"at": "2026-10-21T09:40:30", "kind": "view", "tone": "neutral", "actor": "Demo Steward", "title": "Opened this patient from the queue"},
        {"at": "2026-10-20T09:41", "kind": "version", "tone": "info", "actor": "feed · brp-feed",
         "title": "BSN record updated: the address changed", "body": disclosure("The version's fields", version_body, open_=True)},
        {"at": "2026-10-19T14:03", "kind": "auto-link", "tone": "info", "actor": "system",
         "title": "Hospital MRN record linked without review: the BSN agreed"},
        {"at": "2026-10-19T14:03", "kind": "version", "tone": "info", "actor": "feed · his-prod", "title": "Hospital MRN record created (HL7 v2 ADT A28)"},
        {"at": "2026-06-30T11:20", "kind": "merge", "tone": "warn", "actor": "Demo Steward",
         "title": "Merged Mohr, A. into this patient: 1 identifier moved (Radiology RIS)", "reason": "Duplicate registration confirmed by the radiology desk",
         "body": old_merge},
        {"at": "2025-11-05T07:58", "kind": "version", "tone": "info", "actor": "feed · brp-feed", "title": "BSN record created; this person created with it"},
    ]
    if state == "merged":
        ev.insert(0, {"at": MERGE_AT, "kind": "merge", "tone": "warn", "actor": "Demo Steward",
                      "title": "Merged Maiden, Alice into this patient: 1 identifier moved (Laboratory)", "reason": MOHR_REASON,
                      "body": '<p class="fui-small">Reversible: this is the latest merge. 2 subscribers told (ITI-93 merge).</p>'})
    if state == "unmerged":
        ev.insert(0, {"at": UNMERGE_AT, "kind": "unmerge", "tone": "warn", "actor": "Demo Steward",
                      "title": "Unmerged Mohr, A.: the Radiology RIS identifier moved back", "reason": "Radiology desk: two different patients after all",
                      "body": '<p class="fui-small">2 subscribers told by FerroPIX\'s own unmerge notification; the pair is queued again.</p>'})
    return ev


PATIENT_TABS = patient_tabs()


def history_filter(checked):
    present = [k for k in HISTORY_KINDS if k in {e_["kind"] for e_ in history_events()}]
    word = "every kind but views" if set(checked) == {k for k in present if k != "view"} else ", ".join(k for k in HISTORY_KINDS if k in checked)
    since = button("secondary", "Since 2025-11-05", "calendar", attrs='aria-haspopup="dialog" aria-expanded="false"')
    return f'<div class="fui-list-toolbar mk-history-toolbar"><div class="fui-list-toolbar-start">{tpl.filter_button("Kind", word)}{since}</div></div>'


def page_patient_history(filtered=False, state="rest"):
    events = history_events(state)
    present = {ev["kind"] for ev in events}
    checked = [k for k in HISTORY_KINDS if k in ("link", "auto-link", "merge")] if filtered else [k for k in HISTORY_KINDS if k in present and k != "view"]
    shown = [ev for ev in events if ev["kind"] in checked]
    hidden = len(events) - len(shown)
    note = f'<p class="fui-small fui-muted">{len(shown)} of {len(events)} events, newest first · {hidden} hidden by the filter{"" if filtered else "; views are off by default"}.</p>'
    chips = tpl.active(["Kind · " + ", ".join(checked)]) if filtered else ""
    body = f'<div class="fui-stack">{history_filter(checked)}{chips}{note}{timeline("History of Mohr, Alice", shown)}</div>'
    return patient_page("history", body, state=state, queue=D_.QUEUE_TOTAL - 1 if state == "merged" else D_.QUEUE_TOTAL)


def pseudonyms_panel():
    rows = []
    for n, purpose, _, _, _, st in D_.PSEUDO_DOMAINS:
        if st[1] != "active":
            continue
        ps = D_.MOHR_PSEUDONYMS.get(n)
        if ps:
            rows.append(f'<tr><th scope="row">{e(n)}</th>' + td("Purpose", purpose) + td("Pseudonym", identifier(n, ps[0], reveal_id="reveal", dots=10, show_domain=False))
                        + td("Created", when(ps[1])) + "</tr>")
        else:
            rows.append(f'<tr class="is-excluded"><th scope="row">{e(n)}</th>' + td("Purpose", purpose) + td("Pseudonym", "none in this domain")
                        + td("Created", '<span class="mk-dash">–</span>') + "</tr>")
    return titled_panel("Pseudonyms <span class=\"mk-count\">· one per domain</span>",
                        link_button("secondary", "Resolve a pseudonym", "pseudonym-domains.html#resolve", "fingerprint", small=True),
                        table("Pseudonyms of Mohr, Alice", th("Domain") + th("Purpose") + th("Pseudonym") + th("Created"), rows))


def page_patient_pseudonyms():
    return patient_page("pseudonyms", pseudonyms_panel())


def page_tabs_hover_focus():
    return patient_page("identifiers", identifiers_panel(), focus="history", hover="records")


def page_tabs_scrolled():
    script = "document.querySelector('.fui-tabs').scrollLeft = 400;"
    return patient_page("pseudonyms", pseudonyms_panel(), scrolled=True, script=script)


def page_tabs_states():
    def sheet(title, line, cur, focus=None, hover=None, counts=True, phone=False):
        items = PATIENT_TABS if counts else [t[:3] for t in PATIENT_TABS]
        body = '<p class="fui-small fui-muted mk-tab-demo-body">The panel\'s top edge runs under the inactive tabs and opens under the active one.</p>'
        cls = " mk-tab-phone" if phone else ""
        return (f'<div class="fui-card mk-state-frame{cls}"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                + tabbed(tabs("Example", items, cur, focus=focus, hover=hover, scrolled=phone), body) + "</div>")
    frames = (
        sheet("Rest", "The active tab raised and joined to its panel; the others behind on the sunken ground.", "identifiers")
        + sheet("Another tab active", "History active; Identifiers steps back.", "history")
        + sheet("Hover", "An inactive tab lifts halfway towards the panel's ground.", "identifiers", hover="records")
        + sheet("Keyboard focus", "The ring inside the tab, never clipped by its neighbours.", "identifiers", focus="history")
        + sheet("Without counts", "A tab shows a real count only; none is a shortcut.", "records", counts=False)
        + sheet("Narrow, scrolled", "Below 40 rem the row scrolls; the fades say there is more on either side.", "pseudonyms", phone=True)
    )
    keys = card("Behaviour", definitions([
        ("Markup", "A <code>nav</code> of links with <code>aria-current=\"page\"</code> when each tab is a route, or <code>role=\"tablist\"</code> with roving focus when the tabs switch in place."),
        ("Keys", "Tab reaches the active tab; ← and → move between tabs in a tablist; Enter or Space selects."),
        ("Counts", "A quiet number for a real count; a shortcut is never drawn on a tab."),
        ("Dark", "The same three surfaces of the theme: the page, the sunken tabs, the raised sheet."),
    ]))
    content = page_heading("Tabs, states", subtitle="Binder tabs joined to their panel, on every tabbed page.") + f'<div class="fui-stack">{frames}{keys}</div>'
    return shell("Tabs states", "", content)


def unmerge_rows():
    return [
        ("Radiology RIS", f"Radiology RIS {identifier('Radiology RIS', '077', static=True, show_domain=False)}", change_plain("Mohr, Alice", "Mohr, A.")),
        ("Mohr, A.", f"Person {chip(PERSON_C_SHORT, PERSONS['mohr-a']['id'])}", change_plain("merged into Mohr, Alice", "a patient of its own again")),
        ("Subscribers told", "Subscribers told", '<span class="fui-muted">2, by FerroPIX\'s own unmerge notification</span>'),
        ("Review queue", "Review queue", '<span class="fui-muted">the pair is queued again for review</span>'),
    ]


UNMERGE_REASON = "Radiology desk: two different patients after all"


def page_unmerge():
    body = [f'<tr><th scope="row">{k}</th>' + td(lab, v) + "</tr>" for lab, k, v in unmerge_rows()]
    merge_facts = definitions([
        ("Merged", f"{when('2026-06-30T11:20')} by Demo Steward"),
        ("Reason given", "Duplicate registration confirmed by the radiology desk"),
        ("Moved", "1 identifier: Radiology RIS"),
    ])
    content = (
        page_heading("Unmerge Mohr, A. from Mohr, Alice", subtitle="The latest merge of this patient can be reverted; nothing is written until you confirm.",
                     crumbs=[("Search", "search.html"), ("Mohr, Alice", "patient-identifiers.html")])
        + '<div class="fui-stack">' + card("The merge to revert", merge_facts)
        + card("What the unmerge changes", table("What the unmerge changes", th("What") + th("Change"), body)) + "</div>"
    )
    dk = dock(dock_field("unmerge-reason", "Reason", UNMERGE_REASON, "Why the merge is reverted"),
              link_button("secondary", "Cancel", "patient-identifiers.html") + button("danger", "Unmerge…", "undo"))
    return shell("Unmerge", "search", content, dock_html=dk)


# ---- merge: one confirmation (the page with its docked bar, then one short
# final dialog that does not repeat the table) ----

def merge_content(stale=False):
    def removed(v):
        return f'<del class="fui-change fui-change-removed fui-change-plain"><span class="fui-change-sign" aria-hidden="true">−</span><span class="fui-sr-only">was </span><span class="fui-change-text">{v}</span></del>'

    def added(v):
        return f'<ins class="fui-change fui-change-added fui-change-plain"><span class="fui-change-sign" aria-hidden="true">+</span><span class="fui-sr-only">now </span>{v}</ins>'

    none = '<span class="fui-muted">none</span>'
    rows = [
        (f"Laboratory {identifier('Laboratory', '550', static=True, show_domain=False)}", removed("Maiden, Alice"), added("Mohr, Alice")),
        (f"Person {chip(PERSON_B_SHORT, PERSON_B)}", removed("active"), added("merged into Mohr, Alice")),
        ("Pseudonym in ferroehr-nl", removed("points to Maiden, Alice"), added("now points to Mohr, Alice")),
        ("Subscribers told", none, added("2 · ITI-93 merge notification")),
        ("Candidates closed", none, added("1 · this pair, as linked")),
        ("Reversible until", none, added("the next merge of either patient")),
    ]
    rc = ' class="is-excluded"' if stale else ""
    body = [f'<tr{rc}><th scope="row">{k}</th>' + td("Before", b) + td("After", a) + "</tr>" for k, b, a in rows]
    title = "What changes" + (' <span class="mk-count">· out of date until you preview again</span>' if stale else "")
    change = table("What the merge changes", th("What") + th("Before") + th("After"), body)
    fact = definitions([("Survives", "Mohr, Alice, the older patient, keeps its id; every identifier of Maiden, Alice moves to it.")])
    return (
        page_heading("Merge Maiden, Alice into Mohr, Alice", subtitle="Nothing is written until you confirm; the table is the server's answer for this preview.",
                     crumbs=[("Search", "search.html"), ("Mohr, Alice", "patient-identifiers.html")])
        + '<div class="fui-stack">' + card("", fact) + card(title, change) + "</div>"
    )


def merge_dock(state="idle"):
    msg = ""
    acts = link_button("secondary", "Cancel", "patient-identifiers.html")
    if state == "failed":
        msg = ('<div class="fui-dock-message">' + notice(
            "bad", "circle-alert",
            "Nothing was merged: Maiden, Alice changed at 09:41 today (a new Laboratory version), so this preview is out of date.",
            action=" " + link_button("secondary", "Preview again", "merge-preview.html", "refresh-cw", small=True)) + "</div>")
        acts += button("danger", "Merge 1 identifier…", "merge", attrs="disabled")
    else:
        acts += button("danger", "Merge 1 identifier…", "merge")
    return dock(dock_field("merge-reason", "Reason", MOHR_REASON, "Why these are one patient"), acts, msg)


def page_merge():
    return shell("Merge", "search", merge_content(), dock_html=merge_dock())


def page_merge_confirm():
    dialog = (
        '<div id="merge-confirm" role="dialog" aria-labelledby="merge-confirm-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="merge-confirm-title" class="fui-dialog-title">{icon("triangle-alert", "18")} Merge Maiden, Alice into Mohr, Alice?</h2>'
        '<p>The merge is written at once and told to 2 subscribers. It can be reverted only until the next merge of either patient.</p>'
        + well(f'<p class="fui-small"><strong>Reason:</strong> {MOHR_REASON}</p>')
        + f'<div class="fui-dialog-actions">{button("secondary", "Go back")}{button("danger", "Merge", "merge")}</div></div>'
    )
    return shell("Merge", "search", merge_content(), dock_html=merge_dock(), overlay=dialog)


def page_merge_success():
    t = toasts(toast("ok", "circle-check", "Merged: Maiden, Alice is part of Mohr, Alice", "1 identifier moved; 2 subscribers told. You can revert it from History until the next merge.",
                     action=link_button("secondary", "Open History", "patient-history-merged.html", small=True)))
    return page_patient_identifiers(toast_html=t, state="merged", queue=D_.QUEUE_TOTAL - 1)


def page_merge_failure():
    return shell("Merge", "search", merge_content(stale=True), dock_html=merge_dock("failed"))


# ---- registry ----

def tz(date):
    """CEST from the last Sunday of March to the last Sunday of October."""
    import datetime as _dt
    d = _dt.date.fromisoformat(date)
    return "CEST" if _dt.date(d.year, 3, 25) <= d < _dt.date(d.year, 10, 25) else "CET"


def rel_time(iso):
    """A time in a list: relative today and within a month, the full timestamp as its tooltip."""
    if not iso:
        return '<span class="mk-dash">never</span>'
    import datetime as _dt
    d, t = iso.split("T")
    if d == "2026-10-21":
        return tpl.rel(t + (":00" if len(t) == 5 else ""))
    days = (_dt.date(2026, 10, 21) - _dt.date.fromisoformat(d)).days
    word = ("1 day ago" if days == 1 else f"{days} days ago") if days <= 30 else d
    return f'<time class="fui-time" datetime="{iso}" title="{d} {t} {tz(d)}">{word}</time>'


def uri(text):
    """A URI that breaks only after a slash or a colon."""
    return '<code class="fui-path">' + e(text).replace("/", "/<wbr>").replace(":", ":<wbr>") + "</code>"


def domain_href(name):
    return D_.DOMAIN[name]["slug"] + ".html"


def source_cell(name):
    return f'<a href="{"source.html" if name == "lab-lis" else "sources.html"}"><code>{e(name)}</code></a>'


def domain_rows(retired_ris=False):
    rows = []
    for d in D_.DOMAINS:
        st = ("neutral", "retired") if (retired_ris and d["name"] == "Radiology RIS") else d["state"]
        retired = st[1] == "retired"
        rows.append(
            f'<tr{" class=\"is-excluded\"" if retired else ""}><th scope="row"><a href="{domain_href(d["name"])}">{e(d["name"])}</a></th>' + td("System", uri(d["system"]))
            + td("Kind", badge("accent" if d["kind"] == "authority" else "neutral", d["kind"])) + td("Sensitive", (icon("shield-check", "14") + " yes") if d["sensitive"] else "no")
            + td("Source", source_cell(d["source"]) if d["source"] else '<span class="mk-dash">–</span>') + td("Records", d["records"], "fui-num") + td("Last fed", rel_time(d["last"]))
            + td("State", "Retired" if retired else pill(*st)) + "</tr>"
        )
    return rows


def domains_table():
    n = len(D_.DOMAINS)
    act = sum(1 for d in D_.DOMAINS if d["state"][1] == "active")
    return titled_panel(
        f"Domains <span class=\"mk-count\">· {n}, {act} active</span>", "",
        table("Identifier domains", th("Name", "name", "ascending") + th("System") + th("Kind") + th("Sensitive") + th("Source") + th("Records", num=True) + th("Last fed") + th("State"), domain_rows()),
    )


DOMAINS_SUB = "The assigning authorities, each with the one source that feeds it; a sensitive domain is masked everywhere."


def page_domains():
    content = page_heading("Identifier domains", actions=link_button("primary", "New domain", "domain-new.html", "plus"), subtitle=DOMAINS_SUB) + f'<div class="fui-stack">{domains_table()}</div>'
    return shell("Identifier domains", "domains", content)


def domain_feeds(name):
    feeds = D_.DOMAIN_FEEDS.get(name, [])
    glyph = {"ok": "circle-check", "warn": "clock", "bad": "circle-x"}
    rows = [
        f'<tr><th scope="row">{rel_time(at) if not at.startswith("2026-10-21") else when(at, seconds=True)}</th>' + td("Channel", tag(ch)) + td("Message", m) + td("Outcome", status(o[0], glyph[o[0]], o[1]))
        + td("Request id", f"<code>{r}</code>") + "</tr>"
        for at, ch, m, o, r in feeds
    ]
    if not rows:
        return titled_panel("Last feeds", "", '<p class="fui-panel-body fui-small fui-muted">No feed since the domain was retired.</p>')
    return titled_panel("Last feeds <span class=\"mk-count\">· identifier values are never shown</span>", "", table("The last feeds", th("Time") + th("Channel") + th("Message") + th("Outcome") + th("Request id"), rows))


def page_domain(name="BSN", retired=False, overlay=None, toast_html=""):
    d = D_.DOMAIN[name]
    is_retired = retired or d["state"][1] == "retired"
    st = pill("neutral", "retired") if is_retired else pill(*d["state"])
    src = "none while retired" if is_retired else source_cell(d["source"])
    facts = facts_row([
        ("System", uri(d["system"])), ("Kind", badge("accent" if d["kind"] == "authority" else "neutral", d["kind"])),
        ("Sensitive", "yes, masked everywhere" if d["sensitive"] else "no"), ("Source", src), ("Records", d["records"]), ("State", st), ("Created", d["created"]),
    ])
    if is_retired:
        actions = action_group(link_button("primary", "Reinstate…", "domain-reinstate.html", "rotate-ccw"), [])
    else:
        actions = action_group(link_button("primary", "Edit", "domain-edit.html", "pencil"),
                               [menu_item("Retire…", "archive", href="domain-retire.html", danger=True)], label=f"More actions for {name}")
    notice_html = ""
    if retired:
        notice_html = notice("neutral", "archive", f"Retired {when('2026-10-21T09:42')} by Demo Steward: “RIS replaced by the new PACS numbering.” Feeds to this domain are refused; its {d['records']} records stay and keep answering lookups.")
    per_day = d["per_day"]
    quiet = [v for v in per_day if v] or [0]
    chart = line_chart("feed-activity", "Feeds per day", f"Between {min(quiet)} and {max(quiet)} feeds a day" + (", none in the last 2 days." if per_day[-1] == 0 and per_day[0] else "."),
                       [("Feeds", per_day, 0)], [(0, "21 Sep"), (15, "6 Oct"), (29, "Yesterday")], height=160, legend=False)
    outs = d["outcomes"]
    if outs:
        rows = [(k, f"{v:,}" + (f' <span class="fui-muted">· {d["refused_note"]}</span>' if k == "Refused" and v and d["refused_note"] else "")) for k, v in outs]
        today = definitions(rows + [("p99 latency", "11 ms")])
    else:
        today = '<p class="fui-small fui-muted">No feed today; the last was ' + rel_time(d["last"]) + ".</p>"
    content = (
        page_heading(name, actions=actions, subtitle=f"Identifier domain · {d['about']}", crumbs=[("Identifier domains", "domains.html")])
        + '<div class="fui-stack">' + notice_html + card("Definition", facts)
        + '<div class="fui-grid fui-grid-start">' + card("Feeds per day, the last 30 days", chart) + card("Outcomes, today", today) + "</div>"
        + domain_feeds(name) + "</div>"
    )
    return shell(name, "domains", content, overlay=overlay, toast_html=toast_html)


def page_domain_retire():
    d = D_.DOMAIN["Radiology RIS"]
    dialog = (
        '<div role="dialog" aria-labelledby="retire-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="retire-title" class="fui-dialog-title">{icon("archive", "18")} Retire Radiology RIS?</h2>'
        f'<p>Feeds to this domain are refused from then on. Its {d["records"]} records stay, keep answering lookups and keep their links; you can reinstate it later.</p>'
        + field("retire-reason", "Reason", textarea("retire-reason", "RIS replaced by the new PACS numbering"), help_="Required. Recorded in the audit trail.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel")}{button("danger", "Retire the domain", "archive")}</div></div>'
    )
    return page_domain("Radiology RIS", overlay=dialog)


def page_domain_retired():
    t = toasts(toast("ok", "circle-check", "Radiology RIS is retired", "Recorded in the audit trail with your reason."))
    return page_domain("Radiology RIS", retired=True, toast_html=t)


def page_domain_reinstate():
    dialog = (
        '<div role="dialog" aria-labelledby="re-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="re-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Reinstate Radiology RIS?</h2>'
        '<p>Feeds to this domain are accepted again from the source you choose; its records were kept.</p>'
        + field("re-src", "Source", select("re-src", [("ris-prod", "ris-prod · FHIR"), ("brp-feed-2", "brp-feed-2 · FHIR")], "ris-prod"))
        + field("re-reason", "Reason", textarea("re-reason", "PACS numbering postponed; RIS feeds resume"), help_="Required. Recorded in the audit trail.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel")}{button("primary", "Reinstate", "rotate-ccw")}</div></div>'
    )
    return page_domain("Radiology RIS", retired=True, overlay=dialog)


def domain_form(new=True, no_sources=False):
    if new:
        name = field("d-name", "Name", input_("d-name", "Dental clinic"))
        system = field("d-system", "System", input_("d-system", "urn:oid:1.3.6.1.4.1.21367.2017.2.5.102", extra='spellcheck="false"'),
                       help_="The URI every feed and lookup names; it cannot change once a record exists.", cls="fui-span-row")
        kind = choice_group("d-kind", "Kind", [("local", "Local"), ("authority", "Authority")], "local") + \
            '<p class="fui-help">An authority\'s identifier links records across domains without review; a local one never does.</p>'
        if no_sources:
            src = ('<div class="fui-field"><span class="fui-label">Source <span class="fui-muted">(required)</span></span>'
                   + empty_quiet("radio-tower", "No source to pick", 'Every source already feeds a domain; add a new source first. <a href="source-new.html">New source</a>') + "</div>")
        else:
            src = field("d-src", "Source <span class=\"fui-muted\">(required)</span>", select("d-src", [("dental-ehr", "dental-ehr · HL7 v2"), ("brp-feed-2", "brp-feed-2 · FHIR")], "dental-ehr", extra="required"),
                        help_='The one source that may feed this domain, from <a href="sources.html">Sources</a>.')
    else:
        name = field("d-name", "Name", input_("d-name", "BSN"))
        system = field("d-system", "System", input_("d-system", "http://fhir.nl/fhir/NamingSystem/bsn", extra="disabled"),
                       help_=f"{icon('lock', '12')} Fixed: 412,318 records carry it.", cls="fui-span-row")
        kind = field("d-kind", "Kind", input_("d-kind", "authority", extra="disabled"), help_=f"{icon('lock', '12')} Fixed while records exist.")
        src = field("d-src", "Source", select("d-src", [("brp-feed", "brp-feed · FHIR"), ("brp-feed-2", "brp-feed-2 · FHIR")], "brp-feed-2"),
                    help_='The one source that may feed this domain, from <a href="sources.html">Sources</a>.')
    sens = check("d-sens", "Sensitive", not new, "Masked everywhere and revealed only with a reason.", extra='class="fui-switch"')
    return f'<div class="fui-form mk-edit-form">{name}{system}<div class="fui-columns"><div>{kind}</div>{src}</div>{sens}</div>'


NEW_DOMAIN_SUB = "A database record: created here with a reason, recorded in the audit trail and live at once."


def page_domain_new_no_sources():
    content = (
        page_heading("New identifier domain", subtitle=NEW_DOMAIN_SUB, crumbs=[("Identifier domains", "domains.html")])
        + '<div class="fui-stack">' + card("Definition", domain_form(True, no_sources=True)) + "</div>"
    )
    dk = dock(dock_field("d-reason", "Reason", "", "Why this domain is created"),
              link_button("secondary", "Cancel", "domains.html") + button("primary", "Create the domain", "plus", attrs='disabled title="Pick a source first"'))
    return shell("New identifier domain", "domains", content, dock_html=dk)


def page_domain_new():
    content = (
        page_heading("New identifier domain", subtitle=NEW_DOMAIN_SUB, crumbs=[("Identifier domains", "domains.html")])
        + '<div class="fui-stack">' + card("Definition", domain_form(True)) + "</div>"
    )
    dk = dock(dock_field("d-reason", "Reason", "Dental clinic joins the index in November", "Why this domain is created"),
              link_button("secondary", "Cancel", "domains.html") + button("primary", "Create the domain", "plus"))
    return shell("New identifier domain", "domains", content, dock_html=dk)


def page_domain_edit():
    diff = table("What changes", th("Setting") + th("Running") + th("After the save"),
                 [f'<tr><th scope="row">Source</th>' + td("Running", change_del("<code>brp-feed</code>")) + td("After the save", change_ins("<code>brp-feed-2</code>")) + "</tr>"])
    content = (
        page_heading("Edit", subtitle="A draft of BSN with 1 change; nothing changes until you save it with a reason.", crumbs=[("Identifier domains", "domains.html"), ("BSN", "domain.html")])
        + '<div class="fui-stack">'
        + card("Definition", domain_form(False))
        + card("What changes <span class=\"mk-count\">· 1</span>", diff + '<p class="fui-small fui-muted">From the save on, a feed from brp-feed is refused for this domain and brp-feed-2 is admitted. <span class="fui-effect fui-effect-live">' + icon("circle-check", "12") + "Live at once</span></p>")
        + "</div>"
    )
    dk = dock(dock_field("d-reason", "Reason", "BRP connection moves to the new client registration", "Why this changes"),
              link_button("secondary", "Discard the draft", "domain.html") + button("primary", "Save the change", "check"))
    return shell("Edit BSN", "domains", content, dock_html=dk)


def pseudo_rows():
    rows = []
    for n, p, m, c, cr, st in sorted(D_.PSEUDO_DOMAINS):
        retired = st[1] == "retired"
        rows.append(f'<tr{" class=\"is-excluded\"" if retired else ""}><th scope="row"><a href="pseudonym-domain-edit.html">{e(n)}</a></th>' + td("Purpose", p) + td("Method", m)
                    + td("Pseudonyms", c, "fui-num") + td("Created", when(cr)) + td("State", "Retired" if retired else pill(*st)) + "</tr>")
    return rows


def resolve_card(state="empty"):
    r = D_.RESOLVE
    filled = state != "empty"
    ns = [(n, n) for n, *_rest, st in sorted(D_.PSEUDO_DOMAINS) if st[1] == "active"]
    answer = ""
    if state == "answer":
        p = PERSONS[r["person"]]
        answer = ('<div class="fui-well" role="status">'
                  + f'<p><strong>The pseudonym belongs to {patient_link(r["person"])}</strong>, born {p["born"]} · {chip(p["short"], p["id"])}</p>'
                  + f'<p class="fui-small fui-muted">Recorded as a disclosure {when("2026-10-21T09:42:11", seconds=True)} with your reason · request id <code>{D_.RID_RESOLVE}</code></p></div>')
    btn = busy_button("secondary", "Resolving") if state == "busy" else button("secondary", "Resolve", "fingerprint", type_="submit")
    return (
        '<section class="fui-card" id="resolve"><h2 class="fui-card-title"><span>Resolve a pseudonym</span></h2>'
        '<form method="post" action="#" class="fui-form"><div class="fui-form-grid">'
        + field("ns", "Domain", select("ns", ns, r["domain"] if filled else "ferroehr-nl"))
        + field("pseudonym", "Pseudonym", input_("pseudonym", r["pseudonym"] if filled else "", extra='spellcheck="false" autocapitalize="off"'))
        + field("resolve-reason", "Reason", input_("resolve-reason", r["reason"] if filled else ""), cls="fui-span-2")
        + "</div>"
        + f'<div class="fui-form-foot"><p class="fui-help">Shown as typed, since a pseudonym names no one by itself; the answer is a disclosure, recorded with your reason.</p>{btn}</div></form>'
        + answer + "</section>"
    )


def page_pseudonym_domains(resolve="empty"):
    n = len(D_.PSEUDO_DOMAINS)
    act = sum(1 for x in D_.PSEUDO_DOMAINS if x[5][1] == "active")
    t = titled_panel(f"Domains <span class=\"mk-count\">· {n}, {act} active</span>", "",
                     table("Pseudonymisation domains", th("Namespace", "ns", "ascending") + th("Purpose") + th("Method") + th("Pseudonyms", num=True) + th("Created") + th("State"), pseudo_rows()))
    content = page_heading("Pseudonymisation domains", actions=link_button("primary", "New domain", "pseudonym-domain-new.html", "plus"),
                           subtitle="The namespaces pseudonyms are created in, and the one place a pseudonym is resolved.") + f'<div class="fui-stack">{t}{resolve_card(resolve)}</div>'
    return shell("Pseudonymisation domains", "pseudonym-domains", content)


def page_pseudonym_resolve():
    return page_pseudonym_domains("filled")


def page_pseudonym_resolve_answer():
    return page_pseudonym_domains("answer")


def page_pseudonym_domain_edit():
    form = (
        '<div class="fui-form mk-edit-form"><div class="fui-columns">'
        + field("p-ns", "Namespace", input_("p-ns", "research-2026-q3", extra="disabled"), help_=f"{icon('lock', '12')} Fixed: 18,902 pseudonyms are created in it.")
        + field("p-method", "Method", input_("p-method", "Keyed hash (HMAC-SHA-256)", extra="disabled"), help_=f"{icon('lock', '12')} Fixed: a new method is a new domain.")
        + "</div>"
        + field("p-purpose", "Purpose", input_("p-purpose", "Cohort study, pseudonymised; closes 2027-06-30"))
        + "</div>"
    )
    diff = table("What changes", th("Setting") + th("Running") + th("After the save"),
                 [f'<tr><th scope="row">Purpose</th>' + td("Running", change_del("Cohort study, pseudonymised")) + td("After the save", change_ins("Cohort study, pseudonymised; closes 2027-06-30")) + "</tr>"])
    actions = action_group("", [menu_item("Retire…", "archive", danger=True)], label="More actions for research-2026-q3")
    content = (
        page_heading("Edit", actions=actions, subtitle="A draft of research-2026-q3 with 1 change; nothing changes until you save it with a reason.",
                     crumbs=[("Pseudonymisation domains", "pseudonym-domains.html"), ("research-2026-q3", "pseudonym-domains.html")])
        + '<div class="fui-stack">' + card("Definition", form) + card("What changes <span class=\"mk-count\">· 1</span>", diff) + "</div>"
    )
    dk = dock(dock_field("p-reason", "Reason", "Study end date agreed with the DPO", "Why this changes"),
              link_button("secondary", "Discard the draft", "pseudonym-domains.html") + button("primary", "Save the change", "check"))
    return shell("Edit research-2026-q3", "pseudonym-domains", content, dock_html=dk)


def page_pseudonym_domain_new():
    form = (
        '<div class="fui-form mk-edit-form"><div class="fui-columns">'
        + field("p-ns", "Namespace", input_("p-ns", "research-2027-q1", extra='spellcheck="false"'), help_="Lower case, digits and hyphens; it cannot change once a pseudonym is created in it.")
        + field("p-purpose", "Purpose", input_("p-purpose", "Follow-up cohort, pseudonymised"))
        + "</div>"
        + choice_group("p-method", "Method", [("keyed", "Keyed hash (HMAC-SHA-256)"), ("random", "Random, stored")], "keyed")
        + '<p class="fui-help">A keyed hash gives a patient the same pseudonym every time and needs no table; a random one is stored and can be withdrawn one by one.</p>'
        + "</div>"
    )
    content = (
        page_heading("New pseudonymisation domain", subtitle=NEW_DOMAIN_SUB, crumbs=[("Pseudonymisation domains", "pseudonym-domains.html")])
        + '<div class="fui-stack">' + card("Definition", form) + "</div>"
    )
    dk = dock(dock_field("p-reason", "Reason", "Follow-up study approved by the ethics board, ticket RES-311", "Why this domain is created"),
              link_button("secondary", "Cancel", "pseudonym-domains.html") + button("primary", "Create the domain", "plus"))
    return shell("New pseudonymisation domain", "pseudonym-domains", content, dock_html=dk)


# ---- configuration: console mode (FerroPIX keeps its own store) and file
# mode. FerroPIX has no server configuration module yet, so no page shows a
# configuration key; labels only. ----

def effect(kind):
    if kind == "live":
        return f'<span class="fui-effect fui-effect-live">{icon("circle-check", "12")}Live at once</span>'
    return f'<span class="fui-effect fui-effect-restart">{icon("refresh-cw", "12")}At the next start</span>'


def revision_strip(state, title, line, actions=""):
    tone = {"running": "", "ok": " fui-tone-ok", "warn": " fui-tone-warn", "bad": " fui-tone-bad"}[state]
    glyph = {"running": "file-text", "ok": "circle-check", "warn": "refresh-cw", "bad": "triangle-alert"}[state]
    role = ' role="status"' if state in ("ok", "warn", "bad") else ""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    return f'<div class="fui-revision{tone}"{role}>{icon(glyph, "18")}<div class="fui-revision-text"><strong>{title}</strong><span>{line}</span></div>{a}</div>'


THRESH = list(D_.THRESH)
THRESH_DESC = {"possible_at": "The lowest score that makes a pair a candidate.",
               "review_at": "From here a candidate is probable; the likeliest come first.",
               "link_at": "From here a pair is certain and is linked without review."}
WEIGHTS = D_.WEIGHTS
BLOCKING = [("Birth year and the family name's sound", "Finds a family name spelled two ways, such as Jansen and Janssen."),
            ("Postal code and birth date", "Finds a patient registered twice at one address."),
            ("An identifier value shared across domains", "Finds the same number fed by two sources.")]
ADDED_KEY = "The given name's sound and birth date"
RULES = [("BSN agrees", "A record whose BSN agrees with an active record links to it without review, unless the birth year or a stated gender conflicts."),
         ("Local identifiers", "A local identifier never links across domains: two hospitals that both number a patient 12345 say nothing."),
         ("Replaced by", "A feed's own replaced-by is a link by definition.")]

CFG_TABS = [("thresholds", "Thresholds", "configuration.html"), ("weights", "Field weights", "configuration-weights.html"),
            ("blocking", "Blocking keys", "configuration-blocking.html"), ("rules", "Rules", "configuration-rules.html"),
            ("history", "History", "configuration-history.html")]
FILE_TABS = [("thresholds", "Thresholds", "matching-file-mode.html"), ("weights", "Field weights", "configuration-file-mode.html"),
             ("blocking", "Blocking keys", "matching-file-mode.html"), ("rules", "Rules", "matching-file-mode.html")]
WEIGHT_FORMULA = "Agree adds log2(m / u), disagree adds log2((1 − m) / (1 − u)), from each field's m and u probabilities."
CFG_INTRO = {
    "thresholds": "Where a pair's score puts it; changes apply at once and re-grade the open queue.",
    "weights": f'What each field adds when it agrees and takes away when it disagrees <span class="fui-muted" title="{e(WEIGHT_FORMULA)}" tabindex="0">(how)</span>; changes apply at once.',
    "blocking": "Which records are compared at all; a change takes the next start, after the key columns are re-indexed.",
    "rules": "The links the server makes without scoring; built in and read-only.",
}


def compact_strip(n="7", at="2026-10-14", by="Demo Steward", reason=D_.REVISIONS[0]["reason"]):
    return (f'<div class="fui-revision mk-revision-compact">{icon("file-text", "16")}<p><strong>Revision {n}</strong> · applied {at} by {by} · '
            f'<span class="fui-pill fui-tone-ok" title="{e("Running since " + at + ": " + reason)}">In step</span></p></div>')


def file_line():
    return ('<p class="fui-small fui-muted mk-file-line">Set in ferropix.toml · read at start · '
            '<a href="configuration-file-mode-dialog.html">How configuration works</a></p>')


def key_hint(k, show=False):
    # NOTE: the library's copyable key hint; unused until FerroPIX's configuration module names its keys.
    return f'<button type="button" class="mk-key-hint{" is-shown" if show else ""}" aria-label="Copy the key {e(k)}" title="Copy the key"><code>{e(k)}</code>{icon("copy", "12")}</button>'


def srow(label, desc, value, restart=False, control=None, edit=False, error=None, changed=None):
    nxt = f'<span class="fui-effect fui-effect-restart" title="Takes effect at the next start">{icon("refresh-cw", "12")}Waits for the restart</span>' if restart else ""
    ctl = control if edit and control else f'<span class="fui-setting-value">{value}</span>'
    er = f'<p class="fui-help fui-error mk-row-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    was = f'<span class="fui-config-was">was {e(changed)}</span>' if changed else ""
    return (f'<div class="fui-setting-row{" is-changed" if changed else ""}{" is-invalid" if error else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}</span>'
            f'<p class="fui-setting-desc">{desc}</p>{er}</div><div class="fui-setting-control">{ctl}{nxt}{was}</div></div>')


def num_input(key, v, changed=False, invalid=False, label=""):
    cls = "fui-input" + (" fui-config-changed" if changed else "")
    inv = f' aria-invalid="true" aria-describedby="{key}-error"' if invalid else ""
    return f'<input class="{cls} mk-num-input" id="cfg-{key}" value="{e(v)}" inputmode="decimal" aria-label="{e(label or key)}"{inv} />'


def config_rows(tab, mode="view", draft=None, error=None, blocking_added=False, thresh=None):
    edit = mode == "edit"
    draft = draft or {}
    error = error or {}
    thresh = thresh or THRESH
    if tab == "thresholds":
        out = ""
        for key, label, v in thresh:
            nv = draft.get(key, v)
            ch = nv != v
            out += srow(label, THRESH_DESC[key], v, control=num_input(key, nv, ch, key in error, label), edit=edit,
                        error=error.get(key), changed=v if edit and ch else None)
        out += srow("Link certain pairs without review", "A certain pair is linked at once (FHIR match grade certain); off sends it to the queue.", "on",
                    control='<input type="checkbox" class="fui-switch" checked aria-label="Link certain pairs without review" />', edit=edit)
        vals = {k: float(draft.get(k, v)) for k, _, v in thresh}
        scale = "" if error else score_scale(None, "The thresholds on the scale", [("possible", vals["possible_at"]), ("probable", vals["review_at"]), ("certain", vals["link_at"])])
        return out, f'<div class="fui-panel-body">{scale}</div>' if scale else ""
    if tab == "weights":
        out = ""
        for key, label, comp, agree, dis in WEIGHTS:
            if edit:
                ctl = (f'<span class="mk-nowrap">{num_input(key + "-agree", f"+{agree}", label=label + ", agree")} agree</span> '
                       f'<span class="mk-nowrap">{num_input(key + "-disagree", f"−{abs(dis)}", label=label + ", disagree")} disagree</span>')
            else:
                ctl = None
            val = f'<span class="fui-score-weight fui-score-add">+{agree}</span> agree · <span class="fui-score-weight fui-score-remove">−{abs(dis)}</span> disagree'
            out += srow(label, comp + ".", val, control=ctl, edit=edit)
        return out, ""
    if tab == "blocking":
        out = "".join(srow(w, desc, "on", control=f'<input type="checkbox" class="fui-switch" checked aria-label="{e(w)}" />', edit=edit) for w, desc in BLOCKING)
        if edit:
            out += srow(ADDED_KEY, "Pairs records that share the given name's sound and the birth date.", "off",
                        control=f'<input type="checkbox" class="fui-switch fui-config-changed" checked aria-label="{e(ADDED_KEY)}" />', edit=True, changed="off")
        elif blocking_added:
            out += srow(ADDED_KEY, "Added in revision 9; the key columns are re-indexed at the next start.", "on", restart=True)
        return out, ""
    rules = "".join(f"<li><strong>{w}.</strong> {d}</li>" for w, d in RULES)
    return f'<ul class="fui-bullets mk-rules">{rules}</ul>', ""


def edit_href(tab):
    return {"thresholds": "matching-edit.html", "weights": "configuration-weights-edit.html", "blocking": "configuration-blocking-edit.html"}.get(tab)


def config_tab_panel(tab, mode="view", draft=None, error=None, blocking_added=False, config_mode="console", top="", intro=None, thresh=None, base="7", edit_off=None):
    rows, below = config_rows(tab, mode, draft, error, blocking_added, thresh)
    intro = intro or CFG_INTRO[tab]
    act = ""
    if mode == "edit":
        n = len(draft or {}) or 1
        intro = f"A draft of this tab, based on revision {base}: {n} change{'s' if n != 1 else ''}."
    elif config_mode == "console" and tab != "rules":
        if edit_off:
            act = button("secondary", "Edit", "pencil", small=True, attrs=f'disabled title="{e(edit_off)}"')
        else:
            act = link_button("secondary", "Edit", edit_href(tab), "pencil", small=True)
    body = rows if tab == "rules" else f'<div class="fui-setting-rows">{rows}</div>'
    return (f'<div class="fui-settings-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{intro}</p><div class="fui-actions">{act}</div></div>'
            f'{top}{body}{below}</div>')


def config_page(tab, strip, panel, dock_html="", overlay=None, toast_html="", config_mode="console", drafts=(), running="7", queue=D_.QUEUE_TOTAL):
    tabs_ = CFG_TABS if config_mode == "console" else FILE_TABS
    content = (configuration_heading(config_mode, running) + f'<div class="fui-stack">{strip}'
               + settings_tabs("Configuration sections", tabs_, tab, panel, drafts=drafts) + "</div>")
    return shell("Configuration", "configuration", content, dock_html=dock_html, overlay=overlay, toast_html=toast_html, queue=queue)


def matching_page(tab="thresholds", overlay=None):
    return config_page(tab, compact_strip(), config_tab_panel(tab), overlay=overlay)


def page_matching():
    return matching_page()


def page_configuration_weights():
    return matching_page("weights")


def page_configuration_blocking():
    return matching_page("blocking")


def page_configuration_rules():
    return matching_page("rules")


DRAFT = {"review_at": "0.82", "possible_at": "0.65"}
REV8_REASON = D_.REV8["reason"]


def edit_dock(primary="Review the changes", reason="", disabled=False, href="matching-diff.html", discard="configuration.html"):
    fld = dock_field("cfg-reason", "Reason", reason, "Why this changes; kept with the revision")
    go = button("primary", primary, "git-compare", attrs="disabled") if disabled else link_button("primary", primary, href, "git-compare")
    return dock(fld, link_button("secondary", "Discard the draft", discard, "x") + go)


def page_matching_edit():
    return config_page("thresholds", compact_strip(), config_tab_panel("thresholds", "edit", DRAFT), dock_html=edit_dock(reason=REV8_REASON), drafts=("thresholds",))


def page_matching_invalid():
    draft = {"review_at": "0.82", "possible_at": "0.85"}
    err = {"possible_at": "Must be below Probable from (0.82)."}
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 value needs a change before you can review</p>'
               '<ul><li><a href="#cfg-possible_at">Possible from: must be below Probable from (0.82)</a></li></ul></div>')
    return config_page("thresholds", compact_strip(), config_tab_panel("thresholds", "edit", draft, err, top=summary),
                       dock_html=edit_dock(reason=REV8_REASON, disabled=True), drafts=("thresholds",))


def page_configuration_weights_edit():
    return config_page("weights", compact_strip(), config_tab_panel("weights", "edit", {"phone": "+3.9"}), drafts=("weights",),
                       dock_html=edit_dock(reason="", href="configuration-weights.html", discard="configuration-weights.html"))


def page_configuration_blocking_edit():
    return config_page("blocking", compact_strip(), config_tab_panel("blocking", "edit", {"added": "on"}), drafts=("blocking",),
                       dock_html=edit_dock(reason=D_.REV9["reason"], href="matching-restart.html", discard="configuration-blocking.html"))


def diff_table(changes, left="Revision 7, running", right="Draft"):
    body = [
        f'<tr><th scope="row">{k}</th>' + td(left, change_del(a)) + td(right, change_ins(b)) + td("Takes effect", effect(fx)) + "</tr>"
        for k, a, b, fx in changes
    ]
    return table("What changes", th("Setting") + th(left) + th(right) + th("Takes effect"), body)


DIFF_ROWS = [("Possible from", "0.60", "0.65", "live"), ("Probable from", "0.80", "0.82", "live")]
QUEUE_EFFECT = ('<p class="fui-small">What applying does to the open queue, as the server computed it: <strong>3 candidates</strong> move from probable to possible, '
                'and <strong>2</strong> fall under 0.65 and leave the queue until either record changes or the threshold drops.</p>')


def review_panel(intro, body):
    return (f'<div class="fui-settings-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{intro}</p></div>'
            f'<div class="fui-panel-body fui-stack">{body}</div></div>')


def review_dk():
    return dock(dock_field("cfg-reason", "Reason", REV8_REASON, "Why this changes; kept with the revision"),
                link_button("secondary", "Back to editing", "matching-edit.html", "pencil") + link_button("primary", "Apply as revision 8…", "matching-apply.html", "check"))


DIFF_INTRO = "Your draft against revision 7, checked the way the server checks a revision: no problems."


def page_matching_diff():
    return config_page("thresholds", compact_strip(), review_panel(DIFF_INTRO, diff_table(DIFF_ROWS) + QUEUE_EFFECT), dock_html=review_dk(), drafts=("thresholds",))


def page_matching_apply():
    dialog = (
        '<div role="dialog" aria-labelledby="apply-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="apply-title" class="fui-dialog-title">{icon("check", "18")} Apply revision 8?</h2>'
        '<p>Both changes are live at once: new scores use them, and the open queue is re-graded (3 candidates move to possible, 2 leave the queue).</p>'
        + well(f'<p class="fui-small"><strong>Reason:</strong> {REV8_REASON}</p><p class="fui-small fui-muted">Recorded in the audit trail with your name; revision 7 stays in History.</p>')
        + f'<div class="fui-dialog-actions">{button("secondary", "Go back")}{button("primary", "Apply revision 8", "check")}</div></div>'
    )
    return config_page("thresholds", compact_strip(), review_panel(DIFF_INTRO, diff_table(DIFF_ROWS) + QUEUE_EFFECT), dock_html=review_dk(), overlay=dialog, drafts=("thresholds",))


THRESH_8 = [("possible_at", "Possible from", "0.65"), ("review_at", "Probable from", "0.82"), ("link_at", "Certain from", "0.95")]


def page_matching_applied():
    strip = revision_strip("ok", "Revision 8 runs", f"Applied {when(D_.REV8['at'])} by you: “{REV8_REASON}.” The queue was re-graded: {D_.QUEUE_TOTAL - 2} candidates wait.")
    t = toasts(toast("ok", "circle-check", "Revision 8 applied", "Live at once; recorded in the audit trail."))
    return config_page("thresholds", strip, config_tab_panel("thresholds", thresh=THRESH_8), toast_html=t, running="8", queue=D_.QUEUE_TOTAL - 2)


def page_matching_restart():
    strip = revision_strip("warn", "Revision 9 runs; its new blocking key waits for the restart",
                           f"Applied {when(D_.REV9['at'])} by you: “{D_.REV9['reason']}.” Until the next start the scorer pairs records by the three keys it had; restart with <code>ferropix serve</code> as usual, and the re-index runs before the server listens.")
    return config_page("blocking", strip, config_tab_panel("blocking", blocking_added=True, edit_off="A restart is pending; edit after it"), running="9")


def page_matching_conflict():
    r7 = D_.REVISIONS[0]
    strip = revision_strip("bad", "Revision 7 was applied while your draft was open",
                           f"{r7['by']} applied it {when(r7['at'])}: “{r7['reason']}.” Your draft, based on revision 6, is kept and was not applied.",
                           button("primary", "Rebase my draft on revision 7", "git-compare", small=True) + link_button("secondary", "Discard my draft", "configuration.html", "x", small=True))
    body = [
        f'<tr><th scope="row">{k}</th>' + td("Revision 6, your base", a) + td("Revision 7, running", b) + td("Your draft", c) + td("Whose", s) + "</tr>"
        for k, a, b, c, s in [
            ("Possible from", "0.60", "0.60", "0.65", '<span class="fui-badge fui-tone-accent">Yours alone</span>'),
            ("Probable from", "0.78", "0.80", "0.78", '<span class="fui-badge fui-tone-warn">Theirs alone</span>'),
        ]
    ]
    t = card("Three revisions side by side", table("Your draft against revision 7", th("Setting") + th("Revision 6, your base") + th("Revision 7, running") + th("Your draft") + th('<span class="fui-sr-only">Whose</span>'), body)
             + '<p class="fui-small fui-muted">Rebasing keeps your change and takes Probable from 0.80 from revision 7; you review the result before anything is applied.</p>')
    thresh6 = [("possible_at", "Possible from", "0.60"), ("review_at", "Probable from", "0.78"), ("link_at", "Certain from", "0.95")]
    return config_page("thresholds", strip, config_tab_panel("thresholds", "edit", {"possible_at": "0.65"}, top=t, thresh=thresh6, base="6"), drafts=("thresholds",))


CORE = ('<ul class="fui-bullets fui-small"><li>The server: its address, TLS and the deployment profile</li><li>The database and the sealing key</li>'
        '<li>Sign-in: the identity provider and the product scopes</li><li>The audit sink and telemetry</li>'
        '<li>The HL7 v2 listener: address, frame limit and timeouts</li><li>The configuration mode itself</li></ul>')
REST = ('<ul class="fui-bullets fui-small"><li>Matching: thresholds, field weights, blocking keys, and linking certain pairs without review</li>'
        '<li>Sources, then the identifier domains each feeds</li><li>Pseudonymisation domains</li></ul>')


def mode_dialog(mode):
    if mode == "console":
        lead = "This deployment runs in <strong>console mode</strong>: the core stays in ferropix.toml and everything else is changed here."
        here = '<h3>' + icon("monitor", "16") + ' Everything else: here, in the console</h3><p class="fui-small fui-muted">Saved as revisions with a reason, applied live where it can be, restored from History.</p>'
        other = ('<p class="fui-small fui-muted mk-mode-other">The other mode, <strong>file mode</strong>, keeps everything in ferropix.toml for version control and shows it here read-only. '
                 'A deployment uses one mode: to switch, export the running revision as TOML, set file mode in ferropix.toml and restart.</p>')
    else:
        lead = "This deployment runs in <strong>file mode</strong>: everything lives in ferropix.toml; this console shows it read-only."
        here = '<h3>' + icon("file-text", "16") + ' Everything else: also in ferropix.toml</h3><p class="fui-small fui-muted">Changed in the file and reviewed in your repository; read at the next start.</p>'
        other = ('<p class="fui-small fui-muted mk-mode-other">The other mode, <strong>console mode</strong>, keeps only the core in the file and changes everything else here as revisions. '
                 'To switch, set console mode in ferropix.toml and restart: the file\'s settings become revision 1.</p>')
    return (
        '<div id="mode-dialog" role="dialog" aria-modal="true" aria-labelledby="mode-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="mode-title" class="fui-dialog-title">{icon("info", "18")} How configuration works</h2>'
        f'<p>{lead}</p>'
        '<div class="fui-modal-body"><div class="fui-ownership">'
        '<div><h3>' + icon("lock", "16") + ' The core: always in ferropix.toml</h3><p class="fui-small fui-muted">Never in a revision, so no screen can lock you out or lower the security floor.</p>' + CORE + '</div>'
        '<div>' + here + REST + '</div></div>' + other + '</div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc", attrs="autofocus")}</div></div>'
    )


def mode_badge(mode):
    word = "Console mode" if mode == "console" else "File mode"
    glyph = "monitor" if mode == "console" else "file-text"
    tip = "Settings are changed here as revisions" if mode == "console" else "Settings are read from ferropix.toml"
    return (f'<span class="fui-mode-badge" tabindex="0" aria-describedby="mode-tip">{icon(glyph, "14")}{word}'
            f'<span id="mode-tip" role="tooltip" class="fui-tooltip fui-tooltip-below fui-mode-tip">{tip}</span></span>')


def configuration_heading(mode, running="7"):
    if mode == "console":
        primary = link_button("ghost", "How configuration works", "configuration-mode-dialog.html", "info", attrs='aria-haspopup="dialog"')
        items = [menu_item(f"Export revision {running} as TOML", "download"), menu_item("Import TOML…", "upload", href="configuration-import.html")]
        sub = "This deployment's settings and their revisions."
    else:
        primary = link_button("ghost", "How configuration works", "configuration-file-mode-dialog.html", "info", attrs='aria-haspopup="dialog"')
        items = [menu_item("Export the running settings as TOML", "download")]
        sub = "This deployment's settings, read from ferropix.toml."
    return page_heading("Configuration", actions=action_group(primary, items, label="More configuration actions"), subtitle=sub, badge=mode_badge(mode))


def history_rows(revs, running, restore_href="matching-restore.html"):
    rows = []
    for r in revs:
        n = r["n"]
        acts = ""
        if n != running:
            acts = (f'<a class="fui-icon-button" href="configuration-compare.html" aria-label="Compare revision {n} with the running one" title="Compare with running">{icon("git-compare", "16")}</a>'
                    + f'<a class="fui-icon-button" href="{restore_href}#revision-{n}" aria-label="Restore revision {n}…" title="Restore revision {n}…">{icon("rotate-ccw", "16")}</a>')
        rows.append(f'<tr><th scope="row" class="mk-nowrap">Revision {n}</th>' + td("How", r["how"]) + td("By", r["by"], "mk-nowrap") + td("When", when(r["at"]))
                    + td("Reason", r["reason"]) + td("State", pill("ok", "running") if n == running else "") + td("", acts, "fui-table-actions") + "</tr>")
    return rows


def page_configuration(overlay=None):
    revs = D_.REVISIONS
    t = ('<div class="fui-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">Every revision, newest first; compare one with what runs, or restore it as a new revision.</p></div>'
         + table("Revisions", th("Revision") + th("How") + th("By") + th("When") + th("Reason") + th("State") + th('<span class="fui-sr-only">Actions</span>'), history_rows(revs, "7")) + "</div>")
    return config_page("history", compact_strip(), t, overlay=overlay)


def page_matching_restore():
    r6 = D_.REVISIONS[1]
    dialog = (
        '<div role="dialog" aria-labelledby="rs-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="rs-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Restore revision 6?</h2>'
        f'<p>Revision 6 is saved again as the next revision and applied at once: Probable from goes back to 0.78, and the open queue is re-graded.</p>'
        + field("rs-reason", "Reason", textarea("rs-reason", "The September audit change let too many good pairs drop to possible"), help_="Required. Recorded in the audit trail.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel")}{button("primary", "Restore revision 6", "rotate-ccw")}</div></div>'
    )
    return page_configuration(overlay=dialog)


def page_configuration_compare():
    r6, r7 = D_.REVISIONS[1], D_.REVISIONS[0]
    t = diff_table(r7["changes"], left="Revision 6", right="Revision 7, running")
    content = (page_heading("Revision 6 against revision 7", subtitle="Each setting that differs, and how it took effect; every other setting is the same.", crumbs=[("Configuration", "configuration-history.html")])
               + '<div class="fui-stack">' + card(f"{len(r7['changes'])} setting differs", t)
               + card("Revision 7, running", definitions([("From", f"Console · {r7['by']}"), ("When", when(r7["at"])), ("Reason", r7["reason"])]),
                      actions=button("secondary", "Export revision 6 as TOML", "download", small=True) + link_button("ghost", "History", "configuration-history.html", "history", small=True)) + "</div>")
    return shell("Configuration", "configuration", content)


def page_configuration_import():
    toml = '[matching.fields.phone]\nagree = 4.1\ndisagree = -1.2\n'
    form = (field("imp-toml", "TOML", textarea("imp-toml", toml, rows=5, extra='spellcheck="false"', cls="fui-textarea mk-code"),
                  help_="Saved as a dry run; the import itself applies nothing. A setting of the core or a secret written inline is refused, naming it."))
    rows = [("Telephone, agree", "+3.9", "+4.1", "live")]
    content = (page_heading("Import a configuration", subtitle="Paste TOML from version control, read what it changes, then apply it.", crumbs=[("Configuration", "configuration.html")])
               + '<div class="fui-stack">' + card("The TOML", form)
               + card("What the import changes <span class=\"mk-count\">· checked, no problems</span>", diff_table(rows, right="The import")) + "</div>")
    dk = dock(dock_field("imp-reason", "Reason", "The telephone weight from the config repository", "Why this changes"),
              button("secondary", "Discard the import", "x") + button("primary", "Apply the import…", "check"))
    return shell("Configuration", "configuration", content, dock_html=dk)


def page_configuration_file_mode(overlay=None):
    return config_page("weights", file_line(), config_tab_panel("weights", config_mode="file"), overlay=overlay, config_mode="file")


def page_configuration_mode_dialog():
    return matching_page(overlay=mode_dialog("console"))


def page_configuration_file_mode_dialog():
    return page_matching_file_mode(overlay=mode_dialog("file"))


def page_matching_file_mode(overlay=None):
    return config_page("thresholds", file_line(), config_tab_panel("thresholds", config_mode="file"), overlay=overlay, config_mode="file")


def page_domains_file_mode():
    content = page_heading("Identifier domains", subtitle=DOMAINS_SUB) + f'<div class="fui-stack">{file_line()}{domains_table()}</div>'
    return shell("Identifier domains", "domains", content)


# ---- audit ----

import re as _re_audit
REVEAL_DETAIL = definitions([("Revealed", "BSN, for 30 s, copied once"), ("Shown until", when("2026-10-21T09:42:25", seconds=True)), ("Client", "console · session …8f1e")])


def nowrap_ids(text):
    return _re_audit.sub(r"\b([A-Z]{2,}-[\w…]+)", r'<span class="mk-nowrap">\1</span>', text)


def audit_filters(kind="all", actor="all", range_html=None, range_word="Today"):
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
    for i, (t, actor, kind, patient, reason, rid_) in enumerate(acts):
        p = patient_link(patient) if patient else '<span class="mk-dash">–</span>'
        r = nowrap_ids(e(reason)) if reason else '<span class="mk-dash">–</span>'
        is_open = i == expanded
        toggle = button("ghost", f'<span class="fui-sr-only">Detail of the action at {t}</span>', "chevron-down" if is_open else "chevron-right", small=True,
                        attrs=f'aria-expanded="{"true" if is_open else "false"}" title="Detail"')
        rows.append(
            f'<tr><th scope="row">{when("2026-10-21T" + t, seconds=True)}</th>' + td("Actor", e(actor), "mk-nowrap") + td("Kind", badge(*kind)) + td("Patient", p, "mk-nowrap")
            + td("Reason", r) + td("Request id", f"<code>{rid_}</code>") + td("", toggle, "fui-table-actions") + "</tr>"
        )
        if is_open:
            rows.append(f'<tr class="mk-detail-row"><td colspan="7">{well(REVEAL_DETAIL)}</td></tr>')
    return rows


EXPORT_REASON = "Weekly audit extract for the DPO"


def export_popover():
    return (f'<div class="fui-popover" role="dialog" aria-labelledby="exp-title"><p class="fui-popover-title" id="exp-title">Export {len(D_.ACTS)} actions as CSV</p>'
            + field("exp-reason", "Reason", input_("exp-reason", "Daily extract for the DPO"), help_="Required: the export is itself an action in the trail.")
            + f'<div class="fui-actions">{button("secondary", "Cancel", small=True, attrs="disabled")}{busy_button("primary", "Exporting", small=True)}</div></div>')


def audit_page(acts=None, expanded=None, filters=None, chips="", export="idle", toast_html="", count=None, loading=False):
    acts = acts if acts is not None else D_.ACTS
    if export == "busy":
        exp = f'<span class="fui-anchor">{button("secondary", "Export CSV…", "download", small=True, attrs=chr(97) + "ria-expanded=" + chr(34) + "true" + chr(34))}{export_popover()}</span>'
    elif loading:
        exp = button("secondary", "Export CSV…", "download", small=True, attrs='disabled title="Wait until the actions are loaded"')
    else:
        exp = button("secondary", "Export CSV…", "download", small=True, attrs='aria-haspopup="dialog"')
    n = count or f"{len(acts)} today"
    head = th("Time", "time", "descending") + th("Actor") + th("Kind") + th("Patient") + th("Reason") + th("Request id") + th('<span class="fui-sr-only">Detail</span>')
    if loading:
        rows = ["".join(f'<tr aria-hidden="true"><th>{skeleton_lines([70])}</th><td>{skeleton_lines([80])}</td><td>{skeleton_lines([50])}</td><td>{skeleton_lines([70])}</td><td>{skeleton_lines([90])}</td><td>{skeleton_lines([80])}</td><td></td></tr>' for _ in range(8))]
        n = '<span class="fui-spinner fui-small"><span class="fui-spinner-ring" aria-hidden="true"></span>Loading</span>'
    else:
        rows = audit_rows(acts, expanded)
    t = titled_panel(f"Actions <span class=\"mk-count\">· {n}</span>", exp, table("Steward actions", head, rows))
    content = page_heading("Audit trail", subtitle="Every steward action: who made it, when and why.") + tpl.list_body("", filters or audit_filters(), chips, t)
    return shell("Audit trail", "audit", content, toast_html=toast_html)


def page_audit():
    return audit_page()


def page_audit_detail():
    return audit_page(expanded=0)


def page_audit_filtered():
    acts = [a for a in D_.ACTS if a[2][1] in ("reveal", "resolve")]
    chips = filter_chips(["Kind · reveal and resolve"])
    return audit_page(acts, filters=audit_filters(kind="reveal"), chips=chips, count=f"{len(acts)} of {len(D_.ACTS)} today")


def page_audit_export_busy():
    return audit_page(export="busy")


def page_audit_export_done():
    t = toasts(toast("ok", "download", "Export ready", f"ferropix-audit-2026-10-21.csv · {len(D_.ACTS)} actions, 3 KB.",
                     action=button("secondary", "Download", "download", small=True)))
    acts = [("09:42:11", "Demo Steward", ("neutral", "export"), None, "Daily extract for the DPO", D_.RID_EXPORT)] + D_.ACTS
    return audit_page(acts, toast_html=t, count=f"{len(acts)} today")


def page_audit_loading():
    return audit_page(loading=True)


def page_audit_no_match():
    body = empty_quiet("search-x", "No action matches these filters",
                       f'{len(D_.ACTS)} actions were recorded today, but no merge by the system. <a href="audit.html">Show every action of today</a>')
    chips = filter_chips(["Actor · system", "Kind · merge"])
    content = page_heading("Audit trail", subtitle="Every steward action: who made it, when and why.") + f'<div class="fui-stack">{audit_filters(kind="merge", actor="system")}{chips}{body}</div>'
    return shell("Audit trail", "audit", content)


# ---- settings: every choice applies at once (review item 33) ----

# ---- settings: a personal page of setting rows (review round 1, owner) ----

SETTINGS_SECTIONS = [("appearance", "Appearance", "sun"), ("language", "Language and region", "globe"), ("keyboard", "Keyboard", "keyboard")]

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
        + setting_row("Reduce motion", "Fewer animations in menus, panels and the countdown ring.", sel("motion", [("system", "Follow the system"), ("always", "Always reduce")], "system"), id_="motion", saved=saved == "motion")
        + setting_row("Revealed identifiers stay for", "How long a revealed identifier shows before it is masked again; the server caps every reveal at 30 s.", sel("reveal", [("10", "10 seconds"), ("15", "15 seconds"), ("30", "30 seconds")], "15"), id_="reveal", saved=saved == "reveal")
        + setting_row("Hide them when I leave the page", "Masks every revealed identifier as soon as you open another page or tab.",
                      '<input type="checkbox" class="fui-switch" id="hide-leave" checked aria-label="Hide revealed identifiers when I leave the page" />', saved=saved == "hide"))
    lang_ctrl = select_button("lang", "English", lang_open, listbox(LANGUAGES, "en"))
    language = setting_card("language", "Language and region",
        setting_row("Language", "The console's words. Each language is listed in its own name; more follow.", lang_ctrl, id_="lang", saved=saved == "lang")
        + setting_row("Dates", "Written year first everywhere, with today said in words; every time carries its full timestamp as a tooltip.", '<span class="fui-setting-value"><time class="fui-time">2026-10-21</time> · today 09:42</span>')
        + setting_row("First day of the week", "Follows the language; change it if your team counts weeks differently.", sel("week", [("auto", "Monday, from the language"), ("mon", "Monday"), ("sun", "Sunday")], "auto"), id_="week")
        + setting_row("Time zone", "Times are shown in the server's zone; your browser's is the same.", '<span class="fui-setting-value">Europe/Amsterdam · CEST, UTC+2</span>'))
    keyboard = setting_card("keyboard", "Keyboard",
        setting_row("Shortcut hints", "The quiet key caps after a label; phones never show them.", sel("hints", [("wide", "Wide screens only (default)"), ("always", "Always"), ("never", "Never")], "wide"), id_="hints")
        + setting_row("All shortcuts", "Every shortcut of the screen you are on, in one list.", link_button("secondary", "Open Keyboard shortcuts", "shortcuts.html", "keyboard", small=True, keys="?")))
    reset = f'<div class="fui-settings-foot"><p class="fui-small fui-muted">Kept in this browser as cookies, read when each page is drawn. Nothing here names a patient.</p>{button("ghost", "Reset to defaults…", "rotate-ccw", small=True)}</div>'
    import re as _re
    def rows_of(card_html):
        m = _re.search(r'<div class="fui-setting-rows">(.*)</div></section>$', card_html, _re.S)
        return m.group(1) if m else card_html
    panels = {"appearance": appearance, "language": language, "keyboard": keyboard}
    inner = settings_panel(rows_of(panels[tab]))
    content = (page_heading("Your preferences", subtitle='Only for you, in this browser. Deployment settings are in <a href="configuration.html">Configuration</a>.')
               + f'<div class="fui-stack">{settings_tabs("Preference sections", PREF_TABS, tab, inner)}{reset}</div>')
    html = shell("Your preferences", "", content, overlay=overlay)
    if lock_dark:
        html = html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)
    return html


PREF_TABS = [("appearance", "Appearance", "preferences.html"), ("language", "Language and region", "preferences-language-open.html"),
             ("keyboard", "Keyboard", "preferences-keyboard.html")]


def page_settings():
    return settings_page()


def page_settings_language_open():
    return settings_page(lang_open=True, tab="language")


def page_settings_saved():
    return settings_page(saved="reveal")


def page_settings_keyboard():
    return settings_page(tab="keyboard")


def page_settings_reset():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="reset-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="reset-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Reset your preferences?</h2>'
        '<p>Theme, density, motion, reveal duration, hiding, language, week and hints go back to their defaults: their cookies in this browser are cleared. The configuration of the deployment and the audit trail are not touched.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Reset to defaults", "rotate-ccw")}</div></div>'
    )
    return settings_page(overlay=dialog)


def page_settings_dark():
    return settings_page(theme="dark", saved="theme", lock_dark=True)


# ---- sign-in ----

def sign_in_card(notice_html="", username="", password_error=None, locked=False, redirecting=False):
    if redirecting:
        provider = f'<button type="button" class="fui-button fui-button-primary" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>Taking you to FerroHEALTH ID…</button>'
    else:
        provider = f'<a href="sign-in-redirecting.html" class="fui-button fui-button-primary">{icon("log-in", "16")}Sign in with FerroHEALTH ID</a>'
    dis = " disabled" if (locked or redirecting) else ""
    inv = ' aria-invalid="true" aria-describedby="sign-in-password-error"' if password_error else ""
    er = f'<p id="sign-in-password-error" class="fui-help fui-error">{icon("circle-alert", "14")} {password_error}</p>' if password_error else ""
    return (
        f'<div class="fui-card fui-sign-in"><div class="fui-sign-in-lockup">{lockup(link=False)}</div>'
        + notice_html + provider
        + '<div class="fui-divider">or, on this development deployment</div>'
        '<form method="post" action="#" class="fui-form">'
        + f'<div class="fui-field"><label for="sign-in-username" class="fui-label">Username</label><input id="sign-in-username" name="username" class="fui-input" value="{e(username)}" autocomplete="username" autocapitalize="off" spellcheck="false"{dis} /></div>'
        + f'<div class="fui-field"><label for="sign-in-password" class="fui-label">Password</label><input id="sign-in-password" name="password" type="password" class="fui-input" autocomplete="current-password"{inv}{dis}{" autofocus" if password_error else ""} />{er}</div>'
        + f'<button type="submit" class="fui-button fui-button-secondary"{dis}>{icon("log-in", "16")}Sign in</button></form>'
        '<div class="fui-sign-in-links"><a href="https://ferropix.eu/" rel="external noopener">Documentation</a><a href="#" rel="external noopener">Privacy notice</a></div></div>'
        '<p class="fui-sign-in-caption">The console for the data stewards of the patient index. No patient data is shown before you sign in.</p>'
    )


def sign_in_page(**kw):
    return centred("Sign in", f'<div class="fui-stack mk-sign-in-stack">{sign_in_card(**kw)}</div>')


def page_sign_in():
    return sign_in_page()


def page_sign_in_redirecting():
    return sign_in_page(redirecting=True, notice_html='<div role="status" class="fui-sign-in-notice fui-tone-info">You come back here once you are signed in.</div>')


def page_sign_in_wrong():
    return sign_in_page(username="demo.steward", password_error="4 tries are left before a 15-minute pause.",
                        notice_html='<div role="alert" class="fui-sign-in-notice fui-tone-bad">Not signed in: the username or the password is wrong.</div>')


def page_sign_in_locked():
    return sign_in_page(username="demo.steward", locked=True,
                        notice_html='<div role="alert" class="fui-sign-in-notice fui-tone-warn">Sign-in paused for this username after 5 failed tries. Try again after 09:57, or sign in with FerroHEALTH ID.</div>')


def page_sign_in_signed_out():
    return sign_in_page(notice_html='<div role="status" class="fui-sign-in-notice fui-tone-info">You are signed out of this browser.</div>')


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
    if kind == "ok":
        t = toast("ok", "circle-check", "Linked: Maiden, Alice is part of Mohr, Alice", "Recorded with your reason; 2 subscribers told.")
        rows = [p for p in PAIRS if p["key"] != "mohr"][:10]
        return queue_page(queue_rows(rows=rows), toast_html=toasts(t), total=D_.QUEUE_TOTAL - 1, queue=D_.QUEUE_TOTAL - 1, count=queue_count(D_.QUEUE_TOTAL - 1).replace(f"{len(D_.LONG_WAITS)} waiting", f"{len(D_.LONG_WAITS) - 1} waiting"))
    if kind == "warn":
        t = toast("warn", "triangle-alert", "The feed sender is behind", '41 notifications wait for the subscriber <span class="mk-nowrap">ehr-research</span>, which does not answer; FerroPIX retries every 30 s.')
        return page_dashboard(toast_html=toasts(t), backlog=41)
    t = toast("bad", "circle-alert", "The reveal failed", "The server did not answer within 10 s, so nothing was shown or recorded. Try the reveal again.")
    return page_compare(toast_html=toasts(t))


def status_page(code, glyph, title, body, actions, current="dashboard", tone_status=("ok", "Server up")):
    content = (f'<div class="fui-status-page"><span class="fui-empty-glyph">{icon(glyph, "24")}</span><p class="fui-status-page-code">{code}</p>'
               f'<h1>{title}</h1><p>{body}</p><div class="fui-actions">{actions}</div></div>')
    return shell(title, current, content, status=tone_status)


def page_forbidden():
    return status_page("403 · Not allowed", "ban", "Rotating the sealing key needs the key custodian scope",
                       "You are signed in as Demo Steward, steward and administrator; the key custodian scope is held by the security officer. Ask them, or go back to Configuration.",
                       link_button("primary", "Back to Configuration", "configuration.html", "arrow-left") + link_button("secondary", "Dashboard", "dashboard.html"), current="configuration")


def page_not_found():
    return status_page("404 · Not found", "compass", "No patient with this person id",
                       "FerroPIX does not know the person id 0199c2f1" + "⁠…⁠" + "f0f0: it may be mistyped. A merged person still opens and leads to the survivor.",
                       link_button("primary", "Search for a patient", "search.html", "search") + link_button("secondary", "Dashboard", "dashboard.html"), current="search")


def page_focus_rings():
    def row(title, html):
        return f'<div class="mk-focus-row"><span class="mk-focus-name">{title}</span><div class="mk-focus-demo">{html}</div></div>'
    rows = (
        row("Primary button", button("primary", "Link", "link", cls="mk-focus", keys="l"))
        + row("Secondary button", button("secondary", "Defer", "clock", cls="mk-focus"))
        + row("Danger button", button("danger", "Merge 1 identifier…", "merge", cls="mk-focus"))
        + row("Icon button", icon_button("ellipsis", "More actions", cls="mk-focus"))
        + row("Link", '<a href="patient-identifiers.html" class="mk-focus-ring mk-inline-link">Mohr, Alice</a>')
        + row("Text field", input_("f1", "Maiden name", cls="fui-input mk-focus"))
        + row("Select", select("f2", [("all", "All grades")], "all", cls="fui-select mk-focus"))
        + row("Checkbox", '<div class="fui-check"><input type="checkbox" id="f3" class="mk-focus" checked /><label for="f3" class="fui-label">Include deferred</label></div>')
        + row("Switch", '<div class="fui-check"><input type="checkbox" id="f4" class="fui-switch mk-focus" checked /><label for="f4" class="fui-label">Sensitive</label></div>')
        + row("Segmented control", '<div role="group" class="fui-segmented"><button class="fui-segment" aria-pressed="true">By identifier</button><button class="fui-segment mk-focus">By demographics</button></div>')
        + row("Filter chip", '<div class="fui-chip-checks"><label class="fui-chip-check mk-focus-ring"><input type="checkbox" checked />link</label></div>')
        + row("Tab", tabbed(tabs("Example", [t[:3] for t in PATIENT_TABS[:3]], "identifiers", focus="records"), '<span class="fui-small fui-muted">Panel</span>'))
        + row("Search control", search_control("", "focus", id_="find-f"))
        + row("Work-list row", '<div class="fui-card fui-panel fui-worklist">' + table("Example", queue_head(checks=False), queue_rows(rows=[PAIR["mohr"]], kb_focus=0, focused=0, checks=False)) + "</div>")
        + row("Menu item", menu([menu_item("Open in a side panel", "panel-right", keys="o", focus=True), menu_item("Defer…", "clock", keys="d")], anchored=False))
    )
    content = page_heading("Focus, on every control", subtitle="One 2 px ring that stands out in both themes, drawn where no neighbour can clip it.") + f'<div class="fui-card mk-focus-sheet">{rows}</div>'
    return shell("Focus rings", "", content)


# ---- the screens every remaining control leads to ----

def page_dashboard_empty():
    steps = ('<ol class="mk-setup-steps"><li><a href="source-new.html"><strong>New source</strong></a><span>who may feed, and how it connects</span></li>'
             '<li><a href="domain-new.html"><strong>New identifier domain</strong></a><span>each names its one source</span></li>'
             '<li><a href="configuration.html"><strong>Configuration</strong></a><span>the matching thresholds and weights</span></li></ol>')
    body = empty_quiet("radio-tower", "Nothing is fed yet", "Set the index up in this order; records arrive with the source's first feed.")
    content = dashboard_heading() + f'<div class="fui-stack">{body}{steps}</div>'
    return shell("Dashboard", "dashboard", content, queue=0)


def page_unmerge_confirm():
    html = page_unmerge()
    dialog = (
        '<div role="dialog" aria-labelledby="um-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="um-title" class="fui-dialog-title">{icon("triangle-alert", "18")} Unmerge Mohr, A. from Mohr, Alice?</h2>'
        '<p>Mohr, A. becomes a patient of its own again with the Radiology RIS identifier, 2 subscribers are told, and the pair returns to the review queue.</p>'
        + well(f'<p class="fui-small"><strong>Reason:</strong> {UNMERGE_REASON}</p>')
        + f'<div class="fui-dialog-actions">{button("secondary", "Go back")}{button("danger", "Unmerge", "undo")}</div></div>'
    )
    return html.replace('</div></body></html>', f'<div class="mk-scrim" aria-hidden="true"></div>{dialog}</div></body></html>').replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1)


def page_unmerge_done():
    t = toasts(toast("ok", "circle-check", "Unmerged: Mohr, A. is a patient of its own again", "1 identifier moved back; 2 subscribers told; the pair is in the review queue."))
    return page_patient_identifiers(toast_html=t, state="unmerged", queue=D_.QUEUE_TOTAL + 1)


def page_reveal_refused():
    t = toasts(toast("bad", "circle-alert", "The BSN was not revealed", "You reached the limit of 10 reveals an hour; the next is possible at 10:05. Nothing was shown, and the attempt is recorded."))
    return page_patient_identifiers("refused", toast_html=t)


def page_compare_link_merge():
    dialog = (
        '<div role="dialog" aria-labelledby="lm-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="lm-title" class="fui-dialog-title">{icon("triangle-alert", "18")} Link these records? The two patients become one</h2>'
        '<p>Maiden, Alice is a patient of her own, so linking merges her into Mohr, Alice: 1 identifier moves and 2 subscribers are told. Reversible until the next merge of either patient.</p>'
        + well(f'<p class="fui-small"><strong>Reason:</strong> {MOHR_REASON}</p>')
        + f'<div class="fui-dialog-actions">{button("secondary", "Go back")}{button("danger", "Link and merge", "merge")}</div></div>'
    )
    return page_compare(overlay=dialog)


def defer_popover(count):
    import datetime as _d
    date = _d.date(2026, 10, 28)
    quicks = [(w, w == "In a week") for w in ("Tomorrow", "In 3 days", "Next Monday", "In a week")]
    qhtml = '<div class="fui-datepicker-quick" role="group" aria-label="Quick choices">' + "".join(
        f'<button type="button" class="fui-chip-check fui-datepicker-chip" aria-pressed="{"true" if on else "false"}">{w}</button>' for w, on in quicks) + "</div>"
    return (
        f'<div class="fui-popover" role="dialog" aria-labelledby="bulk-defer-title"><p class="fui-popover-title" id="bulk-defer-title">Defer {count} candidates</p>'
        + qhtml + field("bulk-until", "Back in the queue on", date_input("bulk-until", date.isoformat()), help_=f"{WEEKDAY_NAMES[date.weekday()]} {date.isoformat()}")
        + field("bulk-reason", "Reason", textarea("bulk-reason", "Awaiting the laboratory's batch correction"), help_="Each deferral is its own record.")
        + f'<div class="fui-actions">{button("secondary", "Cancel", small=True, keys="Esc")}{button("primary", "Defer", "clock", small=True)}</div></div>'
    )


def page_queue_bulk_defer():
    return queue_page(queue_rows(selected=SELECTED, selecting=True), selection=selection_bar(open_=True), head=queue_head(some_checked=True), script=INDETERMINATE)


def search_page_body(results_html, busy=False, born="1985-03-14", born_error=None):
    seg = segmented_form("Search by", [("identifier", "By identifier"), ("demographics", "By demographics")], "demographics")
    inv = ' aria-invalid="true" aria-describedby="born-error" autofocus' if born_error else ""
    born_ctl = ('<div class="fui-input-group fui-date">' + input_("born", born, "YYYY-MM-DD", extra=f'inputmode="numeric" spellcheck="false"{inv}')
                + icon_button("calendar", "Choose a date", cls="fui-input-toggle") + "</div>")
    form = (
        '<form method="post" action="#" class="fui-form mk-search-form"><div class="fui-form-grid">'
        + field("family", "Family name", input_("family", "Mohr"))
        + field("given", "Given names", input_("given", "Alicia"))
        + field("born", "Birth date", born_ctl, error=born_error)
        + field("gender", "Gender", select("gender", [("", "Any")], ""))
        + field("postal", "Postal code", input_("postal", "9997 CD"))
        + '</div><div class="fui-form-foot">' + check("exact", "Exact names only", True, "On: only exact spellings count.")
        + (busy_button("primary", "Searching") if busy else button("primary", "Search", "search", type_="submit")) + "</div></form>"
    )
    return (page_heading("Search", subtitle=SEARCH_SUB)
            + f'<div class="fui-stack">{card("Find a patient", seg + form)}{results_html}</div>')


def page_search_no_match():
    body = empty_quiet("search-x", "No patient matches",
                       'Nothing scored as a possible match for these four fields with exact names on. <a href="search-demographics.html">Turn exact names off</a>')
    return shell("Search", "search", search_page_body(titled_panel('Results', "", f'<div class="fui-panel-body">{body}</div>')))


def page_search_loading():
    rows = ["".join(f'<tr aria-hidden="true"><th>{skeleton_lines([70])}</th><td>{skeleton_lines([60])}</td><td>{skeleton_lines([40])}</td><td>{skeleton_lines([80])}</td><td>{skeleton_lines([60])}</td><td>{skeleton_lines([40])}</td><td>{skeleton_lines([60])}</td></tr>' for _ in range(3))]
    res = titled_panel('Results <span class="mk-count">· <span class="fui-spinner fui-small"><span class="fui-spinner-ring" aria-hidden="true"></span>Scoring</span></span>', "",
                       table("Search results, loading", th("Name") + th("Born") + th("Gender") + th("Identifiers") + th("Score") + th("Grade") + th("Last fed"), rows))
    return shell("Search", "search", search_page_body(res, busy=True))


def page_search_refused():
    res = refusal("The search was refused (400): the server could not read the birth date.", issues=[
        ("bad", "error", "invalid", "Patient.birthDate: 1985-03-41 is not a date."),
    ]) + '<p class="mk-refusal-line">Correct the birth date and search again; nothing was searched.</p>'
    return shell("Search", "search", search_page_body(res, born="1985-03-41", born_error="1985-03-41 is not a date: March has 31 days."))


def page_about():
    content = (page_heading("About", subtitle="The server this console talks to, the versions and the manufacturer.")
               + '<div class="fui-stack"><div class="fui-grid fui-grid-start">'
               + card("Versions", definitions([("Console", "FerroPIX Console 0.1.0"), ("Server", "FerroPIX 0.1.0"), ("FHIR", "R4 (4.0.1)"), ("Profiles", "PIXm 3.1.0, PDQm 3.2.0, PMIR 1.6.0"), ("FerroUI", "ferrohealth-ui 0.3.0")]))
               + card("Manufacturer", definitions([("Made by", "Cadasto B.V."), ("Licence", "Business Source License 1.1"), ("Support", '<a href="mailto:info@cadasto.com">info@cadasto.com</a>'), ("Documentation", '<a href="https://ferropix.eu/" rel="external">The FerroPIX book</a>')]))
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
  "date": "2026-10-21",
  "kind": "instance",
  "software": { "name": "FerroPIX", "version": "0.1.0" },
  "fhirVersion": "4.0.1",
  "format": [ "application/fhir+json" ],
  "instantiates": [
    "https://profiles.ihe.net/ITI/PIXm/CapabilityStatement/IHE.PIXm.Manager",
    "https://profiles.ihe.net/ITI/PDQm/CapabilityStatement/IHE.PDQm.PatientDemographicsSupplier",
    "https://profiles.ihe.net/ITI/PDQm/CapabilityStatement/IHE.PDQm.PatientDemographicsSupplierMatch",
    "https://profiles.ihe.net/ITI/PMIR/CapabilityStatement/IHE.PMIR.PatientIdentityRegistry"
  ]
}"""


def feeds_of(source):
    return [d["name"] for d in D_.DOMAINS if d["source"] == source]


def page_sources():
    rows = []
    for s in D_.SOURCES:
        doms = feeds_of(s["name"])
        name = f'<a href="source.html"><code>{e(s["name"])}</code></a>' if s["name"] == "lab-lis" else f'<code>{e(s["name"])}</code>'
        rows.append(
            f'<tr><th scope="row">{name}</th>' + td("Channel", tag(s["channel"])) + td("Admitted as", s["how"])
            + td("Feeds", " ".join(f'<a href="{domain_href(d)}">{e(d)}</a>' for d in doms) if doms else '<span class="mk-dash">no domain names it yet</span>')
            + td("Last fed", rel_time(s["last"])) + td("Today", f'{s["today"]:,}', "fui-num") + td("State", pill(*s["state"])) + "</tr>"
        )
    feeding = sum(1 for s in D_.SOURCES if s["today"] or s["state"][1] == "quiet 3 days")
    t = titled_panel(f"Sources <span class=\"mk-count\">· {len(D_.SOURCES)}, {feeding} feeding</span>", "",
                     table("Sources", th("Source", "name", "ascending") + th("Channel") + th("Admitted as") + th("Feeds", title="Read from the domains that name this source") + th("Last fed") + th("Today", num=True) + th("State"), rows))
    note = card("How a source is admitted", '<ul class="fui-bullets fui-small"><li><strong>FHIR</strong>: the client is registered at the identity provider, which issues its token; here you name its client id. FerroPIX refuses a feed to a domain that does not name this source with 403, its own choice: ITI-104 names no code for it.</li>'
                '<li><strong>HL7 v2</strong>: the sender is registered here by its sending application and facility (MSH-3, MSH-4) and the addresses it connects from; an unknown sender is refused with an AR acknowledgment.</li>'
                '<li>Each identifier domain names its one source; Feeds here is read from the domains.</li></ul>')
    content = (page_heading("Sources", actions=link_button("primary", "New source", "source-new.html", "plus"),
                            subtitle="The systems allowed to feed records, and how each connects.")
               + f'<div class="fui-stack">{t}{note}</div>')
    return shell("Sources", "sources", content)


def page_source(overlay=None):
    s = D_.SOURCE["lab-lis"]
    facts = facts_row([("Channel", tag("HL7 v2") + " MLLP"), ("Sending application", "<code>LIS</code>"), ("Sending facility", "<code>LAB</code>"),
                       ("Connects from", "<code>10.20.4.12</code>"), ("Character set", "UNICODE UTF-8 (MSH-18)"), ("State", pill(*s["state"]))])
    d = D_.DOMAIN["Laboratory"]
    doms = ('<p class="mk-feeds"><span class="fui-label">Feeds:</span> <a href="domain-lab.html">Laboratory</a> '
            '<span class="fui-small fui-muted">· read from the domains that name lab-lis; change it on the domain</span></p>'
            + table("Domains that name lab-lis", th("Domain") + th("Records") + th("Last fed"),
                    [f'<tr><th scope="row"><a href="domain-lab.html">Laboratory</a></th>' + td("Records", d["records"]) + td("Last fed", rel_time(d["last"])) + "</tr>"]))
    glyph = {"ok": "circle-check", "warn": "clock", "bad": "circle-x"}
    mrows = [f'<tr><th scope="row">{rel_time("2026-10-21T" + t)}</th>' + td("Message", f"<code>{m}</code>") + td("Outcome", status(o[0], glyph[o[0]], o[1])) + td("Answer", f"<code>{a}</code>") + "</tr>" for t, m, o, a in D_.LAB_MESSAGES]
    content = (
        page_heading("lab-lis", actions=action_group(link_button("primary", "Edit", "source-edit.html", "pencil"), [menu_item("Retire…", "archive", href="source-retire.html", danger=True)], label="More actions for lab-lis"),
                     subtitle="Source · the laboratory information system, over HL7 v2.", crumbs=[("Sources", "sources.html")])
        + '<div class="fui-stack">' + card("Definition", facts) + card("Feeds <span class=\"mk-count\">· read-only</span>", doms)
        + titled_panel("Last messages <span class=\"mk-count\">· message contents are never stored</span>", "", table("Last messages", th("Time") + th("Message") + th("Outcome") + th("Answer"), mrows)) + "</div>"
    )
    return shell("lab-lis", "sources", content, overlay=overlay)


def page_source_retire():
    dialog = (
        '<div role="dialog" aria-labelledby="sr-title" class="fui-dialog fui-dialog-danger fui-modal mk-open" open>'
        f'<h2 id="sr-title" class="fui-dialog-title">{icon("archive", "18")} Retire lab-lis?</h2>'
        '<p>The sender is refused from then on with an AR acknowledgment. Laboratory names it as its source, so Laboratory takes no feed until it names another.</p>'
        + field("sr-reason", "Reason", textarea("sr-reason", "The laboratory moves to its new LIS in November"), help_="Required. Recorded in the audit trail.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel")}{button("danger", "Retire the source", "archive")}</div></div>'
    )
    return page_source(overlay=dialog)


def source_form(v):
    return (
        '<div class="fui-form mk-edit-form"><div class="fui-columns">'
        + field("s-name", "Name", input_("s-name", v["name"], extra='spellcheck="false"' + (" disabled" if v.get("fixed") else "")),
                help_=(f"{icon('lock', '12')} Fixed: the trail names the source by it." if v.get("fixed") else "How the console and the trail name this source."))
        + field("s-ch", "Channel", select("s-ch", [("fhir", "FHIR client"), ("v2", "HL7 v2 sender")], "v2", extra=" disabled" if v.get("fixed") else ""))
        + field("s-app", "Sending application (MSH-3)", input_("s-app", v["app"], extra='spellcheck="false"'))
        + field("s-fac", "Sending facility (MSH-4)", input_("s-fac", v["fac"], extra='spellcheck="false"'))
        + field("s-ip", "Connects from", input_("s-ip", v["ip"], extra='spellcheck="false"'), help_="Addresses or ranges; a connection from anywhere else is closed.")
        + field("s-cs", "Character set when MSH-18 is empty", select("s-cs", [("utf8", "UNICODE UTF-8"), ("8859", "8859/1")], "utf8"))
        + "</div></div>"
    )


def page_source_new():
    content = (page_heading("New source", subtitle="Who the source is and how it connects; a domain names it once it exists.", crumbs=[("Sources", "sources.html")])
               + '<div class="fui-stack">' + card("Definition", source_form({"name": "dental-ehr", "app": "DENT", "fac": "DENTAL", "ip": "10.20.6.0/28"})) + "</div>")
    dk = dock(dock_field("s-reason", "Reason", "Dental clinic joins the index in November", "Why this source is added"),
              link_button("secondary", "Cancel", "sources.html") + button("primary", "Add the source", "plus"))
    return shell("New source", "sources", content, dock_html=dk)


def page_source_edit():
    content = (page_heading("Edit", subtitle="A draft of lab-lis with 1 change; nothing changes until you save it with a reason.", crumbs=[("Sources", "sources.html"), ("lab-lis", "source.html")])
               + '<div class="fui-stack">' + card("Definition", source_form({"name": "lab-lis", "app": "LIS", "fac": "LAB", "ip": "10.20.4.12/31", "fixed": True}))
               + card("What changes <span class=\"mk-count\">· 1</span>", table("What changes", th("Setting") + th("Running") + th("After the save"),
                      [f'<tr><th scope="row">Connects from</th>' + td("Running", change_del("<code>10.20.4.12</code>")) + td("After the save", change_ins("<code>10.20.4.12/31</code>")) + "</tr>"]))
               + "</div>")
    dk = dock(dock_field("s-reason", "Reason", "The laboratory adds a second interface engine", "Why this changes"),
              link_button("secondary", "Discard the draft", "source.html") + button("primary", "Save the change", "check"))
    return shell("Edit lab-lis", "sources", content, dock_html=dk)


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
    future_off = bool(max_d) and (year, month) >= (max_d.year, max_d.month)
    nxt = (icon_button("chevron-right", "Next month", attrs="disabled" if future_off else "") if nav in ("both", "next") else '<span class="fui-datepicker-navgap"></span>')
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
    rng = (D(2026, 10, 15), None if half else D(2026, 10, 21))
    months = (month_grid(2026, 9, rng=rng, tentative=hover, max_d=PICK_TODAY, title_btn=False, nav="prev")
              + month_grid(2026, 10, rng=rng, tentative=hover, max_d=PICK_TODAY, hover=hover, title_btn=False, nav="next"))
    status = "From 2026-10-15: now choose the end." if half else "2026-10-15 to 2026-10-21 · 7 days."
    return picker("Actions between", f'<div class="fui-datepicker-pair">{months}</div>',
                  presets=[("Today", False), ("Last 7 days", not half), ("Last 30 days", False), ("This month", False)], status=status)


def page_date_picker_states():
    def frame(title, line, html):
        return f'<div class="fui-card mk-picker-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div class="mk-picker-demo">{html}</div></div>'
    frames = (
        frame("Open, a date chosen", "Today ringed, the chosen day filled, days after today disabled for an audit From.",
              date_field_open("ps1", "2026-10-14", "From", picker_single()))
        + frame("A day hovered", "The hovered day takes the sunken ground; the chosen one stays filled.",
                date_field_open("ps2", "2026-10-14", "From", picker_single(hover=D(2026, 10, 8))))
        + frame("Keyboard focus", "Arrows move by day, PageUp and PageDown by month, Home and End to the week's ends, Enter chooses.",
                date_field_open("ps3", "2026-10-14", "From", picker_single(focus=D(2026, 10, 16))))
        + frame("Date and time", "A time row, 24 h, for a date-time field.",
                date_field_open("ps4", "2026-10-14 08:30", "From", picker_single(time=("08", "30"), label="From, date and time")))
        + frame("Out-of-range days and quick choices", "Defer cannot go before tomorrow; the quick choices fill the grid",
                date_field_open("ps5", "2026-10-28", "Defer until", picker_defer()))
        + frame("Month and year chooser", "The title opens the months of the year; the year steps with the arrows.",
                date_field_open("ps6", "2026-10-14", "From", picker("From", month_chooser(2026, 10))))
        + frame("Range, half chosen", "The start is set; the hovered day previews the range before the second click.",
                date_field_open("ps7", "2026-10-14", "From", picker_range(half=True, hover=D(2026, 10, 19))))
        + frame("Range, both ends", "The days between tinted, presets at the side; From and To fill together.",
                date_field_open("ps8", "2026-10-14", "From", picker_range()))
        + frame("Invalid typed date", "The field keeps what was typed and says why; the grid stays on the last valid month with nothing chosen.",
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
    return shell("Date picker states", "", content)


def audit_filters_with(picker_html, field_id="from"):
    return audit_filters(range_html=picker_html, range_word="2026-10-15 to 2026-10-21")


def page_audit_range_picker():
    filt = audit_filters_with(picker_range())
    return audit_page(filters=filt, count="9 today · the range applies when you choose Apply")


def page_date_picker_sheet():
    filt = audit_filters_with(picker_range().replace('class="fui-datepicker fui-datepicker-range"', 'class="fui-datepicker fui-datepicker-range fui-datepicker-sheet-capable"', 1))
    return audit_page(filters=filt, count="9 today · the range applies when you choose Apply")


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
    return page_compare(overlay=defer_dialog(date=_d.date(2026, 10, 26), quick="Next Monday", focus=_d.date(2026, 10, 26)))



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
    "patient-loading": page_patient_loading,
    "patient-unreachable": page_patient_unreachable,
    "reveal": page_reveal,
    "patient-records": page_patient_records,
    "patient-history": lambda: page_patient_history(),
    "patient-history-filtered": lambda: page_patient_history(True),
    "patient-history-merged": lambda: page_patient_history(state="merged"),
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
    "source-edit": page_source_edit,
    "source-retire": page_source_retire,
    "pseudonym-resolve": page_pseudonym_resolve,
    "pseudonym-resolve-answer": page_pseudonym_resolve_answer,
    "configuration-weights-edit": page_configuration_weights_edit,
    "configuration-blocking-edit": page_configuration_blocking_edit,
    "sign-in-signed-out": page_sign_in_signed_out,
    "domain-ris": lambda: page_domain("Radiology RIS"),
    "domain-mrn": lambda: page_domain("Hospital MRN"),
    "domain-lab": lambda: page_domain("Laboratory"),
    "domain-legacy": lambda: page_domain("Legacy ZIS"),
    "patient-maiden": lambda: page_person("maiden"),
    "patient-alissa": lambda: page_person("alissa"),
    "patient-petrova": lambda: page_person("petrova"),
}
for _p in PAIRS:
    if _p["slug"] != "compare":
        PAGES[_p["slug"]] = (lambda k: (lambda: page_compare(key=k)))(_p["key"])
