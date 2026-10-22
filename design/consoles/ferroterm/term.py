# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""FerroTERM's shared parts: the state mark, the source tag, the planned and
proposed marks, the JSON highlighter, the library's B1, B4, B5 and B10
helpers, and the new library components M1 (concept tree), M2 (coding),
M4 (clause), M5 (root matrix) and M6 (mapping row). M3 (the version
select) is in common, in the top bar."""
import re

from common import ROOTS, badge, copy_button, e, health_dot, icon, icon_button
from data import SOURCE_TIP, SOURCE_WORD

WORD = {"ok": "up", "warn": "degraded", "bad": "unreachable", "neutral": "not checked", "muted": "off"}


def state(tone, word=None, detail=None):
    """A state as the dot's shape and colour and the word, never colour alone (F12)."""
    d = f'<span class="fui-status-detail">{e(detail)}</span>' if detail else ""
    w = word or WORD[tone]
    w = w[:1].upper() + w[1:]
    return (f'<span class="fui-state fui-tone-{"neutral" if tone == "muted" else tone}{" is-muted" if tone == "muted" else ""}">'
            f'{health_dot(tone, w)}{e(w)}{d}</span>')


STATUS_TONE = {"active": "ok", "draft": "neutral", "retired": "muted"}


def status_mark(status):
    """A publication status as the F12 state: Active, Draft, Retired."""
    return state(STATUS_TONE[status], status)


def source_tag(source):
    word = SOURCE_WORD[source]
    if source == "written":
        return f'<span class="fui-tag" title="{e(SOURCE_TIP[source])}">{word}</span>'
    return f'<span class="fui-tag fui-tag-locked" title="{e(SOURCE_TIP[source])}">{icon("lock", "12")}{word}</span>'


def plural(n, word, many=None):
    """'1 concept', '3 concepts': every count on every page goes through here."""
    w = word if n == 1 else (many or word + "s")
    return f"{n:,} {w}"


def quiet(line):
    """An empty pane or answer: one quiet line, no icon (as FerroBRIDGE)."""
    return f'<p class="fui-transform-empty">{line}</p>'


def planned(issue, text):
    return f'<p class="fui-proposed">{badge("neutral", f"Planned #{issue}")}<span>{text}</span></p>'


def proposed(text):
    return f'<p class="fui-proposed">{badge("neutral", "Proposed")}<span>{text}</span></p>'


def wb(text):
    """A canonical that may break after each slash, never inside a segment."""
    return e(text).replace("/", "/<wbr>")


def canon(url):
    return f'<code class="fui-canonical">{wb(url)}</code>'


# ---- M2 Coding ----

def inactive_mark():
    return '<span class="fui-inactive-mark">Inactive</span>'


def coding(display, code, system=None, inactive=False, inline=False, href=None):
    d = e(display) if display else '<span class="fui-muted">no display</span>'
    if href:
        d = f'<a class="fui-row-link" href="{href}">{d}</a>'
    sysw = f'<span class="fui-coding-system">{e(system)}</span>' if system else ""
    mark = inactive_mark() if inactive else ""
    cls = "fui-coding" + (" fui-coding-inline" if inline else "") + (" is-inactive" if inactive else "")
    return (f'<span class="{cls}"><span class="fui-coding-display">{d}</span>'
            f'<span class="fui-coding-meta"><code class="fui-coding-code">{e(code)}</code>{sysw}{mark}</span></span>')


# ---- M1 Concept tree ----

def tree_item(code, display, children=None, expanded=False, selected=False, on_path=False, leaf=False, inactive=False, focus=False, href="browse.html", more=None, loading=False):
    if leaf:
        tog = '<span class="fui-tree-spacer" aria-hidden="true"></span>'
    else:
        tog = (f'<button type="button" class="fui-tree-toggle" tabindex="-1" aria-label="{"Collapse" if expanded else "Expand"} {e(display)}">'
               f'{icon("chevron-down" if expanded else "chevron-right", "14")}</button>')
    cls = "fui-tree-item" + (" is-on-path" if on_path else "") + (" is-inactive" if inactive else "")
    attrs = f' aria-selected="{"true" if selected else "false"}"' + (f' aria-expanded="{"true" if expanded else "false"}"' if not leaf else "")
    mark = inactive_mark() if inactive else ""
    row = (f'<div class="fui-tree-row{" mk-focus-ring" if focus else ""}">{tog}<a class="fui-tree-label" href="{href}" tabindex="-1">{e(display)}</a>'
           f'<span class="fui-tree-code">{e(code)}</span>{mark}</div>')
    kids = ""
    if expanded:
        inner = "".join(children or [])
        if loading:
            inner += '<li class="fui-tree-loading" role="none"><span class="fui-spinner-ring" aria-hidden="true"></span>Reading the children</li>'
        if more:
            inner += f'<li class="fui-tree-more" role="none">{more}</li>'
        kids = f'<ul class="fui-tree-group" role="group">{inner}</ul>'
    return f'<li role="treeitem" class="{cls}"{attrs} tabindex="{0 if selected else -1}">{row}{kids}</li>'


def concept_tree(items, label="The hierarchy of Synthetic clinical findings"):
    return f'<div class="fui-concept-tree"><ul class="fui-tree" role="tree" aria-label="{e(label)}">{"".join(items)}</ul></div>'


# ---- M4 Clause ----

def clause(kind, title, sentence, body="", actions="", issue=None):
    mark = badge("accent", "Include") if kind == "include" else badge("warn", "Exclude")
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    iss = ""
    if issue:
        code, text = issue
        iss = f'<p class="fui-clause-issue" role="note">{icon("circle-x", "14")}<span>{text} <code class="fui-muted">{e(code)}</code></span></p>'
    plain = re.sub(r"<[^>]+>", "", title)
    return (f'<section class="fui-clause{" is-exclude" if kind == "exclude" else ""}" aria-label="{"Include" if kind == "include" else "Exclude"}: {e(plain)}">'
            f'<div class="fui-clause-head">{mark}<h3 class="fui-clause-title">{title}</h3>{a}</div>'
            f'<p class="fui-clause-sentence">{sentence}</p>{body}{iss}</section>')


# ---- M5 Root matrix ----

def root_head(root, answered=True):
    line = dict(ROOTS)[root]
    st = state("ok", "answered") if answered else state("bad", "no answer")
    return f'<span class="fui-root-head"><strong>{root}</strong><span class="fui-muted">{line}</span>{st}</span>'


# ---- M6 Mapping row ----

def relation(rel_word, none=False):
    if none:
        return f'<span class="fui-relation is-none">{icon("ban", "14")}{e(rel_word or "maps to nothing")}</span>'
    return f'<span class="fui-relation">{icon("arrow-right", "14")}{e(rel_word)}</span>'


# ---- B4, B5, B10, B1 from the library ----

def load_report(tone, title, counts=(), actions="", glyph=None):
    """B4: one outcome in one line, with its counts."""
    g = glyph or {"ok": "circle-check", "warn": "triangle-alert", "bad": "circle-x", "neutral": "info"}[tone]
    c = "".join(f"<li><strong>{n}</strong> {w}</li>" for n, w in counts)
    cl = f'<ul class="fui-load-counts">{c}</ul>' if counts else ""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    role = ' role="status"' if tone in ("ok", "bad") else ""
    return (f'<div class="fui-load-report fui-tone-{tone}"{role}>{icon(g, "18")}<span class="fui-load-report-title">{title}</span>{cl}{a}</div>')


def section(title, body, count=None, actions="", cls=""):
    """B10: one part of a tab panel, a heading over it, a hairline above it."""
    c = f' <span class="mk-count">· {count}</span>' if count is not None else ""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    h = f'<div class="fui-panel-section-head"><h3 class="fui-panel-section-title">{title}{c}</h3>{a}</div>' if title else ""
    return f'<section class="fui-panel-section{(" " + cls) if cls else ""}">{h}{body}</section>'


def sections(*parts, card=False):
    return f'<div class="{"fui-card fui-panel" if card else "fui-panel"}">{"".join(parts)}</div>'


def json_html(text):
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


def json_lines(text):
    return "".join(f'<span class="fui-code-line">{json_html(l) or " "}</span>' for l in text.rstrip("\n").split("\n"))


def codeview(title, text, meta="", label=None):
    """B1: a document as the server sent it, numbered, never wrapped."""
    n = len(text.rstrip("\n").split("\n"))
    head = (f'<div class="fui-codeview-head"><span class="fui-codeview-path">{title}</span>'
            f'<span class="fui-codeview-meta">{meta}<span>{n} lines</span>{copy_button("Copy", "the JSON")}'
            f'{icon_button("download", "Download the JSON")}</span></div>')
    return (f'<div class="fui-codeview">{head}<div class="fui-code-panel fui-code-panel-numbered fui-scroll-cue" role="region" aria-label="{e(label or title)}, one numbered line per line" tabindex="0">'
            f'<pre>{json_lines(text)}</pre></div></div>')


def facts(rows):
    return '<dl class="fui-facts">' + "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rows) + "</dl>"
