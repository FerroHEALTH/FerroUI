# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""FerroEHR's shared parts: the state mark, plurals, the planned mark, the
JSON and XML highlighters, the library's B1, B3, B4 and B10 helpers, M1's
tree rows and M2's coding, and the new library components E1 (subject
line), E2 (version history), E3 (clinical reading), E4 (folder tree), E5
(path browser), E6 (condition groups), E7 (results pane) and E8 (scope
grant)."""
import re

from common import (badge, button, copy_button, e, health_dot, icon, icon_button, identifier, input_, field, link_button,
                    menu, MENU_SEP, menu_item, table, td, th)

WORD = {"ok": "up", "warn": "degraded", "bad": "unreachable", "neutral": "not checked", "muted": "off"}


def state(tone, word=None, detail=None):
    """A state as the dot's shape and colour and the word, never colour alone (F12)."""
    d = f'<span class="fui-status-detail">{e(detail)}</span>' if detail else ""
    w = word or WORD[tone]
    w = w[:1].upper() + w[1:]
    return (f'<span class="fui-state fui-tone-{"neutral" if tone == "muted" else tone}{" is-muted" if tone == "muted" else ""}">'
            f'{health_dot(tone, w)}{e(w)}{d}</span>')


def plural(n, word, many=None):
    """'1 composition', '3 compositions': every count goes through here."""
    w = word if n == 1 else (many or word + "s")
    return f"{n:,} {w}"


def quiet(line):
    """An empty pane or answer: one quiet line, no icon."""
    return f'<p class="fui-transform-empty">{line}</p>'


def empty_line(line, action=""):
    """A one-line empty state with no icon, its one action beside it."""
    a = f" {action}" if action else ""
    return f'<p class="fui-transform-empty mk-empty-line">{line}{a}</p>'


def planned(issue, text):
    return f'<p class="fui-proposed">{badge("neutral", f"Planned #{issue}")}<span>{text}</span></p>'


def wb(text):
    """An id or path that may break after each slash or colon pair, never inside a segment."""
    return e(text).replace("/", "/<wbr>").replace("::", "::<wbr>")


def mono(text):
    return f"<code>{wb(text)}</code>"


def arch(aid, cls=""):
    """An archetype id (or an ADL 2 HRID) in mono, which breaks only after a
    dot, never inside a segment."""
    segs = e(aid).split(".")
    inner = ".<wbr>".join(f"<span>{x}</span>" for x in segs)
    return f'<code class="fui-archetype{(" " + cls) if cls else ""}" title="{e(aid)}">{inner}</code>'


def path(p):
    """An archetype path in mono, which breaks only after a slash."""
    import re as _re
    parts = _re.split(r"(?<=[/.\[])", p)
    return f'<span class="fui-path">{"<wbr>".join(f"<span>{e(x)}</span>" for x in parts if x)}</span>'


def short(uid):
    """A uuid shortened to its first and last groups, the whole in the tooltip."""
    head, tail = uid[:8], uid[-4:]
    return f'<code title="{e(uid)}" class="mk-nowrap">{head}…{tail}</code>'


# ---- M2 Coding ----

def coding(display, code, system=None, inline=False, href=None):
    d = e(display) if display else '<span class="fui-muted">no text</span>'
    if href:
        d = f'<a class="fui-row-link" href="{href}">{d}</a>'
    sysw = f'<span class="fui-coding-system">{e(system)}</span>' if system else ""
    cls = "fui-coding" + (" fui-coding-inline" if inline else "")
    return (f'<span class="{cls}"><span class="fui-coding-display">{d}</span>'
            f'<span class="fui-coding-meta"><code class="fui-coding-code">{e(code)}</code>{sysw}</span></span>')


# ---- E1 Subject line ----

def masked(tail, ns=None, static=False, revealed=None, left=None, state_=None, show_ns=True):
    """R13: a subject id, masked, ending in `tail`; its namespace as the tag."""
    return identifier(ns or "Subject id", tail, revealed=revealed, left=left, static=static, reveal_id="reveal",
                      show_domain=bool(ns) and show_ns, state=state_, total=30)


def subject_line(subject=True, queryable=True, modifiable=True, revealed=False, state_=None, expired=False):
    if subject:
        if revealed:
            who = masked("37", "demo.hospital.mrn", revealed="P-20481-537", left=24)
        else:
            who = masked("537", "demo.hospital.mrn", state_=("expired" if expired else state_))
        who = f'<span class="fui-subject-label">Subject</span>{who}'
    else:
        who = '<span class="fui-subject-label">Subject</span><span class="fui-subject-none">No external subject: the EHR is bound to no outside identity</span>'
    q = state("ok", "Queryable") if queryable else state("muted", "Not queryable")
    m = state("ok", "Modifiable") if modifiable else state("muted", "Not modifiable")
    return f'<div class="fui-subject"><span class="fui-subject-who">{who}</span><span class="fui-subject-states">{q}{m}</span></div>'


# ---- E2 Version history ----

def version_cell(n, current=False, deleted=False):
    mark = badge("accent", "Current") if current else ""
    if deleted:
        mark = state("muted", "Deleted")
    return f'<span class="fui-version-cell">Version {n}{mark}</span>'


def at_time(id_="at", value="2026-10-14 12:00", what="the version current then"):
    from parts import date_input
    return (f'<div class="fui-at-time">{field(id_, "At a time", input_(id_, value, "YYYY-MM-DD HH:MM", extra="inputmode=" + chr(34) + "numeric" + chr(34) + " spellcheck=" + chr(34) + "false" + chr(34)))}'
            f'{button("secondary", "Open", "history")}<span class="fui-small fui-muted">Opens {what}.</span></div>')


# ---- B4, B10, B1, B3 from the library ----

def load_report(tone, title, counts=(), actions="", glyph=None):
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


def intro(text):
    return f'<p class="fui-small fui-muted fui-section-intro">{text}</p>'


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


def xml_html(text):
    out, pos = [], 0
    for m in re.finditer(r'</?[A-Za-z_][\w:.-]*|/?>|"[^"]*"|[\w:.-]+(?==)', text):
        out.append(e(text[pos:m.start()]))
        tok = m.group(0)
        if tok.startswith("<"):
            out.append(f'<span class="fui-tok-key">{e(tok)}</span>')
        elif tok in (">", "/>"):
            out.append(f'<span class="fui-tok-key">{e(tok)}</span>')
        elif tok.startswith('"'):
            out.append(f'<span class="fui-tok-string">{e(tok)}</span>')
        else:
            out.append(f'<span class="fui-tok-keyword">{e(tok)}</span>')
        pos = m.end()
    out.append(e(text[pos:]))
    return "".join(out)


def aql_html(text):
    kws = r"\b(SELECT|FROM|CONTAINS|WHERE|AND|OR|NOT|ORDER BY|DESC|ASC|LIMIT|OFFSET|COUNT|DISTINCT|EHR|COMPOSITION|OBSERVATION|EVALUATION|INSTRUCTION|ACTION|CLUSTER|ELEMENT|VERSION|TOP|LIKE|matches|exists)\b"
    out, pos = [], 0
    for m in re.finditer(r"'[^']*'|\$[A-Za-z_]\w*|(?<![\w.\-\[])" + kws + r"(?![\w\-])|(?<![\w.\-])-?\d+(?:\.\d+)?(?![\w])", text):
        out.append(e(text[pos:m.start()]))
        tok = m.group(0)
        if tok.startswith("'"):
            out.append(f'<span class="fui-tok-string">{e(tok)}</span>')
        elif tok.startswith("$"):
            out.append(f'<span class="fui-tok-number">{e(tok)}</span>')
        elif tok[0].isdigit() or tok[0] == "-":
            out.append(f'<span class="fui-tok-number">{tok}</span>')
        else:
            out.append(f'<span class="fui-tok-keyword">{e(tok)}</span>')
        pos = m.end()
    out.append(e(text[pos:]))
    return "".join(out)


HL = {"json": json_html, "xml": xml_html, "aql": aql_html, "text": e}


def code_lines(text, lang="json", current=None, diags=None, inline=True):
    """Numbered lines; a line a diagnostic names is marked in its tone, with
    `inline` the diagnostic's words under it (B1). diags: {line: (tone, code, text)}."""
    diags = diags or {}
    out = ""
    for i, l in enumerate(text.rstrip("\n").split("\n"), 1):
        mark = ""
        if i in diags:
            mark = " fui-mark-error" if diags[i][0] == "bad" else " fui-mark-warning"
        cls = "fui-code-line" + mark + (" is-current" if i == current else "")
        out += f'<span class="{cls}" id="L{i}">{HL[lang](l) or " "}</span>'
        if inline and i in diags:
            tone, code, msg = diags[i]
            out += (f'<span class="fui-code-diag fui-tone-{tone}" role="note">{icon("circle-x" if tone == "bad" else "triangle-alert", "14")}<span class="fui-code-diag-text">'
                    f'<span class="fui-code-diag-code">{e(code)}</span>{msg}</span></span>')
    return out


def editor(text, lang="json", lang_word="Canonical JSON", label="the document", diags=None, current=None, dirty=True, pos="Ln 1, Col 1", tools=""):
    """B2: the library editor with F8's toolbar above and the status line below."""
    n = len(text.rstrip("\n").split("\n"))
    marks = ""
    if diags:
        ne = sum(1 for d in diags.values() if d[0] == "bad")
        nw = len(diags) - ne
        marks = f" · <span>{plural(ne, 'error')}, {plural(nw, 'warning')}</span>"
    inv = ' aria-invalid="true"' if diags else ""
    bar = f'<div class="fui-editor-bar"><span>{lang_word} · {plural(n, "line")}{marks}</span><div class="fui-actions">{tools}</div></div>'
    ed = (f'<div class="fui-editor"{inv}><div class="fui-editor-stack"><pre class="fui-editor-layer" aria-hidden="true">{code_lines(text, lang, current, diags, inline=False)}</pre>'
          f'<textarea id="ed" name="ed" class="fui-editor-input" aria-label="{e(label)}" autocomplete="off" autocapitalize="off" spellcheck="false" wrap="soft">{e(text)}</textarea></div></div>')
    d = '<span class="fui-editor-dirty">Changed since opened</span>' if dirty else '<span>Unchanged</span>'
    status = f'<div class="fui-editor-status"><span>{pos}</span><span>UTF-8 · LF</span>{d}</div>'
    return f'<section class="fui-card fui-panel fui-mapping-editor" aria-label="Editor">{bar}{ed}{status}</section>'


def codeview(title, text, lang="json", meta="", label=None, what="the document", current=None, diags=None, actions=True):
    """B1: a document as the CDR sent it, numbered, never wrapped."""
    n = len(text.rstrip("\n").split("\n"))
    acts = (copy_button("Copy", what) + icon_button("download", f"Download {what}")) if actions else ""
    head = (f'<div class="fui-codeview-head"><span class="fui-codeview-path">{title}</span>'
            f'<span class="fui-codeview-meta">{meta}<span>{plural(n, "line")}</span>{acts}</span></div>')
    return (f'<div class="fui-codeview">{head}<div class="fui-code-panel fui-code-panel-numbered" role="region" aria-label="{e(label or title)}, one numbered line per line" tabindex="0">'
            f'<pre>{code_lines(text, lang, current, diags)}</pre></div></div>')


def diag_list(rows, current=None):
    """B3: one row per finding, errors first. rows: (tone, code, line, col, message)."""
    items = ""
    for tone, code, line, col, msg in rows:
        word = {"bad": "Error", "warn": "Warning"}[tone]
        glyph = {"bad": "circle-x", "warn": "triangle-alert"}[tone]
        where = f'<a class="fui-diag-where" href="#L{line}">line {line}, column {col}</a>' if line else '<span class="fui-diag-where">the whole document</span>'
        items += (f'<li class="fui-diag fui-tone-{tone}{" is-current" if current == line else ""}"><span class="fui-diag-sev">{icon(glyph, "14")}{word}</span>'
                  f'<span class="fui-diag-head"><code class="fui-diag-code">{e(code)}</code>{where}</span><p class="fui-diag-msg">{msg}</p></li>')
    return f'<ul class="fui-diags" aria-label="Diagnostics">{items}</ul>'


def facts(rows):
    return '<dl class="fui-facts">' + "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rows) + "</dl>"


# ---- M1 tree rows, E4 folder tree, E5 path browser ----

def tree_item(label, code="", children=None, expanded=False, selected=False, on_path=False, leaf=False, focus=False,
              href="#", glyph=None, extra="", ref=False, actions="", rename=None, count=None):
    if leaf:
        tog = '<span class="fui-tree-spacer" aria-hidden="true"></span>'
    else:
        tog = (f'<button type="button" class="fui-tree-toggle" tabindex="-1" aria-label="{"Collapse" if expanded else "Expand"} {e(label)}">'
               f'{icon("chevron-down" if expanded else "chevron-right", "14")}</button>')
    cls = "fui-tree-item" + (" is-on-path" if on_path else "") + (" is-ref" if ref else "") + (" mk-focus-row" if focus else "")
    attrs = f' aria-selected="{"true" if selected else "false"}"' + (f' aria-expanded="{"true" if expanded else "false"}"' if not leaf else "")
    g = f'<span class="fui-tree-glyph">{icon(glyph, "14")}</span>' if glyph else ""
    if rename is not None:
        lab = input_("rename", rename, extra=f'aria-label="New name for {e(label)}"')
    else:
        lab = f'<a class="fui-tree-label" href="{href}" tabindex="-1">{e(label)}</a>'
    c = f'<span class="fui-tree-code">{e(code)}</span>' if code else ""
    n = f'<span class="fui-tree-count">{count}</span>' if count else ""
    a = f'<span class="fui-tree-actions">{actions}</span>' if actions else ""
    row = f'<div class="fui-tree-row{" mk-focus" if focus else ""}">{tog}{g}{lab}{c}{extra}{n}{a}</div>'
    kids = ""
    if expanded and children:
        kids = f'<ul class="fui-tree-group" role="group">{"".join(children)}</ul>'
    return f'<li role="treeitem" class="{cls}"{attrs} tabindex="{0 if selected else -1}">{row}{kids}</li>'


def tree(items, label, cls=""):
    return f'<div class="fui-concept-tree{(" " + cls) if cls else ""}"><ul class="fui-tree" role="tree" aria-label="{e(label)}">{"".join(items)}</ul></div>'


def rm_type(t):
    return f'<span class="fui-rm-type">{e(t)}</span>'


# ---- E3 Clinical reading ----

def reading_section(title, node, rows=(), children=""):
    rs = ""
    for label, value in rows:
        rs += f"<div><dt>{e(label)}</dt><dd>{value}</dd></div>"
    dl = f'<dl class="fui-reading-rows">{rs}</dl>' if rows else ""
    nd = arch(node, "fui-reading-node") if node else ""
    return f'<section class="fui-reading-section"><h3 class="fui-reading-title">{e(title)}{nd}</h3>{dl}{children}</section>'


def qty(v, unit):
    return f'<span class="fui-reading-value">{e(v)}</span> <span class="fui-reading-unit">{e(unit)}</span>'


def nullf(word):
    return f'<span class="fui-reading-null">{e(word)}</span>'


def reading(*secs):
    return f'<div class="fui-reading">{"".join(secs)}</div>'


# ---- E7 Results pane ----

def results_head(count_html, window, view="table", export=True):
    seg = (f'<div role="group" aria-label="Show the results as" class="fui-segmented">'
           f'<a href="#" class="fui-segment"{" aria-current=" + chr(34) + "true" + chr(34) if view == "table" else ""}>{icon("table-2", "14")}Table</a>'
           f'<a href="#" class="fui-segment"{" aria-current=" + chr(34) + "true" + chr(34) if view == "chart" else ""}>{icon("chart-line", "14")}Chart</a></div>')
    ex = (button("secondary", "Export CSV", "download", small=True) + button("secondary", "Export JSON", "download", small=True)) if export else ""
    return seg + ex


# ---- E8 Scope grant ----

OPS = [("c", "Create"), ("r", "Read"), ("u", "Update"), ("d", "Delete"), ("s", "Search")]


def scope(raw, compartment, resource, ops, broad=False, refused=None):
    if refused:
        return (f'<li class="fui-scope is-refused"><code class="fui-scope-raw">{e(raw)}</code>'
                f'<span class="fui-scope-reads"><span class="fui-small">{icon("circle-x", "14")} {refused}</span></span></li>')
    tags = "".join(f'<span class="fui-scope-op is-on">{w}</span>' if k in ops else f'<del class="fui-scope-op" title="{w}: not permitted">{w}</del>' for k, w in OPS)
    b = badge("warn", "Broad access") if broad else ""
    return (f'<li class="fui-scope"><code class="fui-scope-raw">{e(raw)}</code>'
            f'<span class="fui-scope-reads"><span>{compartment} · {resource}</span><span class="fui-scope-ops">{tags}</span>{b}</span></li>')


def scopes(items, label="Scopes"):
    return f'<ul class="fui-scopes" aria-label="{e(label)}">{"".join(items)}</ul>'


# ---- dialogs ----

def dialog(id_, glyph, title, body, actions, wide=False):
    return (f'<div id="{id_}" role="dialog" aria-modal="true" aria-labelledby="{id_}-title" class="fui-dialog{" fui-dialog-wide" if wide else ""} fui-modal mk-open" open>'
            f'<h2 id="{id_}-title" class="fui-dialog-title">{icon(glyph, "18")} {title}</h2>{body}'
            f'<div class="fui-dialog-actions">{actions}</div></div>')


def confirm_delete(id_, title, what, physical, confirm_label, extra=""):
    """A delete confirmation that says in plain words whether the delete is
    physical (gone for good) or logical (a deleted version on top)."""
    tone = ("Physical delete", "bad") if physical else ("Logical delete", "warn")
    body = f'<p>{badge(tone[1], tone[0])} {what}</p>{extra}'
    return dialog(id_, "trash-2", title, body, button("secondary", "Cancel", keys="Esc", attrs="autofocus") + f'<button type="button" class="fui-button fui-button-danger">{icon("trash-2", "16")}{confirm_label}</button>')


def reveal_dialog(what="the subject id of EHR 0199c2a4…1a01"):
    body = ('<p>The value is read from the CDR, shown for 30 seconds, and the reveal is recorded in the audit trail with your name and this reason.</p>'
            '<div class="fui-modal-body">'
            + field("reveal-reason", "Reason", '<textarea id="reveal-reason" class="fui-textarea fui-textarea-prose" rows="2">Checking the record against the referral letter</textarea>',
                    help_="Required. A colleague reading the audit trail should understand it.")
            + "</div>")
    return dialog("reveal", "eye", f"Reveal {what}?", body, button("secondary", "Cancel", "x") + button("primary", "Reveal for 30 s", "eye"))
