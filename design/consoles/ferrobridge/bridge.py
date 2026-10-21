# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""FerroBRIDGE's shared parts: the state mark, the owner and language tags,
the planned and proposed marks, the YAML highlighter, and the new library
components B1 (code viewer), B3 (diagnostics list), B4 (load report) and
B5 (transform pair). B2 (the editor) is in pg_edit, B6 to B8 in pg_tools
and pg_faces, B9 (the flow) in pg_flow."""
import re

from common import badge, button, copy_button, e, health_dot, icon, icon_button, link_button, pill

WORD = {"ok": "up", "warn": "degraded", "bad": "unreachable", "neutral": "not checked", "muted": "off"}


def state(tone, word=None, detail=None):
    """A state as the dot's shape and colour and the word, never colour alone (F12)."""
    d = f'<span class="fui-status-detail">{e(detail)}</span>' if detail else ""
    w = word or WORD[tone]
    w = w[:1].upper() + w[1:]
    return (f'<span class="fui-state fui-tone-{"neutral" if tone == "muted" else tone}{" is-muted" if tone == "muted" else ""}">'
            f'{health_dot(tone, w)}{e(w)}{d}</span>')


def locked(word, title=None):
    t = f' title="{e(title)}"' if title else ""
    return f'<span class="fui-tag fui-tag-locked"{t}>{icon("lock", "12")}{e(word)}</span>'


def owner_tag(owner):
    if owner == "provisioned":
        return locked("provisioned", "Read from the provisioned mapping directory; the console checks a draft of it but never saves over it")
    return f'<span class="fui-tag" title="Saved by the console under [operator] mapping_dir">console</span>'


def lang_tag(lang):
    return f'<span class="fui-tag">{"FHIRconnect" if lang == "fhirconnect" else "OMOCL"}</span>'


def planned(issue, text):
    return f'<p class="fui-proposed">{badge("neutral", f"Planned #{issue}")}<span>{text}</span></p>'


def proposed(text):
    return f'<p class="fui-proposed">{badge("neutral", "Proposed")}<span>{text}</span></p>'


def plan_badge(issue=None):
    w = f"Planned #{issue}" if issue else "Proposed"
    return f'<span class="fui-badge fui-tone-neutral" title="{"Tracked as issue #" + str(issue) if issue else "Not tracked yet: proposed by this redesign"}">{w}</span>'


def wb(text):
    """A path that may break after each slash, and never inside a name."""
    return e(text).replace("/", "/<wbr>")


def short(path):
    return path.rsplit("/", 1)[-1]


def file_href(path):
    """Every mapping file opens its own page; the mockups draw four of them."""
    if path.endswith("synthetic_body_weight.context.yml"):
        return "file.html"
    if path.startswith("omocl/"):
        return "file-omocl.html"
    if "/models/" in path or "/extensions/" in path:
        return "file-model.html"
    return "file-provisioned.html"


# ---- YAML, as the editor's tokens colour it ----

def hl_yaml(line):
    m = re.match(r"^(\s*)(- )?([A-Za-z_][\w.-]*)(:)(.*)$", line)
    if m:
        ind, dash, key, colon, rest = m.groups()
        return e(ind) + (e(dash) if dash else "") + f'<span class="fui-tok-key">{e(key)}</span>{colon}' + hl_value(rest)
    m = re.match(r"^(\s*)(- )(.*)$", line)
    if m:
        return e(m.group(1)) + e(m.group(2)) + hl_value(" " + m.group(3))[1:]
    if line.strip().startswith("#"):
        return f'<span class="fui-tok-comment">{e(line)}</span>'
    return e(line)


def hl_value(rest):
    v = rest.strip()
    if not v:
        return e(rest)
    lead = rest[: len(rest) - len(rest.lstrip())]
    if v.startswith('"'):
        inner = v
        inner_html = e(inner)
        inner_html = re.sub(r"(\$[a-zA-Z]+)", r'<span class="fui-tok-parameter">\1</span>', inner_html)
        return e(lead) + f'<span class="fui-tok-string">{inner_html}</span>'
    if re.fullmatch(r"[\d.]+", v):
        return e(lead) + f'<span class="fui-tok-number">{e(v)}</span>'
    if v in ("true", "false", "null"):
        return e(lead) + f'<span class="fui-tok-keyword">{e(v)}</span>'
    return e(lead) + e(v)


SEV = {"error": ("bad", "circle-x", "Error"), "warning": ("warn", "triangle-alert", "Warning")}


def code_lines(text, diags=(), current=None, inline=True):
    """Numbered lines, each line a diagnostic names marked in its tone; with
    `inline`, the diagnostic's words drawn under the line (B1)."""
    by_line = {}
    for d in diags:
        by_line.setdefault(d[3], []).append(d)
    out = ""
    for i, line in enumerate(text.rstrip("\n").split("\n"), start=1):
        ds = by_line.get(i, [])
        mark = ""
        if ds:
            mark = " fui-mark-error" if any(d[0] == "error" for d in ds) else " fui-mark-warning"
        cur = " is-current" if current == i else ""
        out += f'<span class="fui-code-line{mark}{cur}" id="L{i}">{hl_yaml(line) or " "}</span>'
        if inline:
            for d in ds:
                tone, g, w = SEV[d[0]]
                out += (f'<span class="fui-code-diag fui-tone-{tone}" role="note">{icon(g, "14")}<span class="fui-code-diag-text">'
                        f'<span class="fui-code-diag-code">{e(d[1])}</span>{e(d[5])}</span></span>')
    return out


def codeview(path, text, diags=(), current=None, lang="fhirconnect", owner="console", sha="5712e3f7…f2e0e3fa", label=None):
    """B1: the file as the bridge serves it, byte for byte, its diagnostics in place."""
    n = len(text.rstrip("\n").split("\n"))
    head = (f'<div class="fui-codeview-head"><span class="fui-codeview-path">{wb(path)}</span>'
            f'<span class="fui-codeview-meta">{lang_tag(lang)}{owner_tag(owner)}<span>{n} lines</span>'
            f'<span class="fui-codeview-sha" title="SHA-256 of the bytes the bridge serves">SHA-256 {e(sha)}</span>'
            f'{copy_button("Copy", "the file")}</span></div>')
    return (f'<div class="fui-codeview">{head}<div class="fui-code-panel fui-code-panel-numbered" role="region" aria-label="{e(label or path)}, one numbered line per line" tabindex="0">'
            f'<pre>{code_lines(text, diags, current)}</pre></div></div>')


def diag_item(d, current=False, href=None, file_in_where=False):
    sev, code, path, line, col, msg, mapping = d
    tone, g, w = SEV[sev]
    where = f"{short(path)}:{line}:{col}" if file_in_where else f"line {line}, column {col}"
    link = href or (file_href(path).replace(".html", "-line.html") if file_href(path) == "file.html" else file_href(path))
    mp = f'<span class="fui-diag-mapping">Mapping {e(mapping)}</span>' if mapping else ""
    return (f'<li class="fui-diag fui-tone-{tone}{" is-current" if current else ""}"><span class="fui-diag-sev">{icon(g, "14")}{w}</span>'
            f'<span class="fui-diag-head"><code class="fui-diag-code">{e(code)}</code><a class="fui-diag-where" href="{link}#L{line}">{e(where)}</a></span>'
            f'<p class="fui-diag-msg">{e(msg)}{mp}</p></li>')


def diags_list(diags, group=False, current=None, href=None):
    """B3: errors first, then warnings; grouped by file where the list spans files."""
    order = sorted(diags, key=lambda d: (0 if d[0] == "error" else 1, d[2], d[3])) if group else sorted(diags, key=lambda d: d[3])
    if not group:
        items = "".join(diag_item(d, current == d[3], href) for d in order)
        return f'<ul class="fui-diags" aria-label="Diagnostics">{items}</ul>'
    files = []
    for d in order:
        if d[2] not in files:
            files.append(d[2])
    out = ""
    for f in files:
        ds = [d for d in order if d[2] == f]
        ne = sum(1 for d in ds if d[0] == "error")
        nw = len(ds) - ne
        cnt = ", ".join(x for x in [f"{ne} error{'s' if ne != 1 else ''}" if ne else "", f"{nw} warning{'s' if nw != 1 else ''}" if nw else ""] if x)
        out += (f'<li><p class="fui-diags-file"><a href="{file_href(f)}" class="fui-row-link"><code>{wb(f)}</code></a><span class="fui-muted">{cnt}</span></p>'
                f'<ul class="fui-diags">{"".join(diag_item(d, href=href) for d in ds)}</ul></li>')
    return f'<ul class="fui-diags" aria-label="Diagnostics, by file">{out}</ul>'


def load_report(tone, title, counts=(), actions=""):
    """B4: the running set's outcome in one line, with its counts."""
    g = {"ok": "circle-check", "warn": "triangle-alert", "bad": "circle-x", "neutral": "layers"}[tone]
    c = "".join(f"<li><strong>{n}</strong> {e(w)}</li>" for n, w in counts)
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    role = ' role="status"' if tone in ("ok", "bad") else ""
    return (f'<div class="fui-load-report fui-tone-{tone}"{role}>{icon(g, "18")}<span class="fui-load-report-title">{title}</span>'
            f'<ul class="fui-load-counts">{c}</ul>{a}</div>')


RUNNING_COUNTS = [("9", "contexts"), ("27", "files"), ("7", "templates"), ("0", "errors"), ("3", "warnings")]


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


def json_panel(text, label):
    return f'<div class="fui-code-panel" role="region" aria-label="{e(label)}" tabindex="0"><pre>{json_html(text)}</pre></div>'


def section(title, body, count=None, actions=""):
    """B10: one part of a tab panel, a heading over it, a hairline above it."""
    c = f' <span class="mk-count">· {count}</span>' if count is not None else ""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    h = f'<div class="fui-panel-section-head"><h3 class="fui-panel-section-title">{title}{c}</h3>{a}</div>' if title else ""
    return f'<section class="fui-panel-section">{h}{body}</section>'


def sections(*parts):
    return f'<div class="fui-panel">{"".join(parts)}</div>'


def pane(title, body, tags="", actions="", status=""):
    """B5: one side of a transform; the format tag after the title, the
    answer's state next, the tools at the end."""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    return f'<section class="fui-transform-pane" aria-label="{e(title)}"><div class="fui-transform-head"><h3 class="fui-transform-title">{title}</h3>{tags}{status}{a}</div>{body}</section>'


def transform(left, right):
    return f'<div class="fui-transform">{left}{right}</div>'


def pane_tools(what):
    return copy_button("Copy", what) + icon_button("download", f"Download {what}")


def input_tools(what, example=True):
    ex = (f'<button type="button" class="fui-button fui-button-ghost fui-button-small" aria-disabled="true" title="Planned #611: an example generated from a template">'
          f'{icon("file-code", "16")}Example</button>') if example else ""
    return ex + button("ghost", "Load file", "upload", small=True) + button("ghost", "Clear", "x", small=True)
