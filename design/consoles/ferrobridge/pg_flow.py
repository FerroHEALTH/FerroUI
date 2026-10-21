# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""B9 Flow: the F9 topology drawn as a pipeline with no hub at the root.
Callers on the left, the bridge's faces, the mapping set in the middle, the
upstreams on the right. The same cards, Bézier edges with horizontal
tangents, state by line style plus one marker at the child end, labels only
on problem edges, the hover path with its pulse, and a card click that opens
the thing's page. Copied from FerroFED's pg_topo (tnode, curve) and laid out
for FerroBRIDGE."""
from common import GLYPHS, e, link_button

H, ROW = 56, 66
W_CALL, W_FACE, W_HUB, W_UP = 178, 206, 196, 214
GAP = 74
X_CALL = 0
X_FACE = X_CALL + W_CALL + GAP
X_HUB = X_FACE + W_FACE + GAP
X_UP = X_HUB + W_HUB + GAP + 36
TOP = 34
GLYPH_OF = {"caller": "users", "face": "workflow", "hub": "layers", "up": "database",
            "facade": "flame", "ops": "flask-conical", "hl7": "message-square", "etl": "table-2", "cdr": "database",
            "term": "book-open", "cdm": "table-2", "clients": "users", "senders": "send", "sched": "clock"}
WORD = {"ok": "Up", "warn": "Degraded", "bad": "Unreachable", "neutral": "Not checked", "muted": "Off"}


def glyph(name, x, y, size=18):
    return f'<svg x="{x}" y="{y}" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{GLYPHS[name]}</svg>'


def fit(t, n):
    return t if len(t) <= n else t[:n - 1] + "…"


def dot_svg(tone, x, y):
    if tone == "warn":
        return f'<rect class="fui-tdot fui-tone-warn" x="{x - 3.5}" y="{y - 3.5}" width="7" height="7" transform="rotate(45 {x} {y})"/>'
    if tone == "bad":
        return f'<path class="fui-tdot fui-tone-bad" d="M{x} {y - 4.5} L{x + 4.5} {y + 3.5} L{x - 4.5} {y + 3.5} Z"/>'
    if tone in ("neutral", "muted", "planned"):
        return f'<circle class="fui-tdot fui-tdot-ring" cx="{x}" cy="{y}" r="3.5"/>'
    return f'<circle class="fui-tdot fui-tone-ok" cx="{x}" cy="{y}" r="4"/>'


def tnode(kind, g, x, y, name, line, tone, label, w, h=H, cls="", href="#", mono=False):
    t = "neutral" if tone in ("muted", "planned") else tone
    extra = (" is-muted" if tone == "muted" else "") + (f" {cls}" if cls else "")
    dot = dot_svg(tone, 47, h / 2 + 10) if kind != "gateway" else ""
    lx = 56 if dot else 42
    m = " fui-tnode-mono" if mono else ""
    body = (f'<title>{e(label)}</title><rect class="fui-tnode-box" width="{w}" height="{h}" rx="8"/><rect class="fui-tnode-bar" width="4" height="{h - 16}" x="0" y="8" rx="2"/>'
            f'<g class="fui-tnode-glyph">{glyph(g, 14, h / 2 - 9)}</g>'
            f'<text class="fui-tnode-name{m}" x="42" y="{h / 2 - 4}">{e(fit(name, int((w - 50) / 6.4)))}</text>'
            f'{dot}<text class="fui-tnode-line" x="{lx}" y="{h / 2 + 14}">{e(fit(line, int((w - lx - 8) / 5.5)))}</text>')
    return (f'<g class="fui-tnode fui-tnode-{kind} fui-tone-{t}{extra}" transform="translate({x} {y})">'
            f'<a class="fui-tnode-link" href="{href}" aria-label="{e(label)}">{body}</a></g>')


def curve_d(x1, y1, x2, y2):
    c = (x2 - x1) * 0.48
    return f"M{x1} {y1} C{x1 + c:.1f} {y1}, {x2 - c:.1f} {y2}, {x2} {y2}"


def curve(x1, y1, x2, y2, tone, tip, path=False, dim=False, label=None):
    cls = f"fui-edge fui-edge-{tone}" + (" is-path" if path else "") + (" is-dim" if dim else "")
    out = f'<path class="{cls}" d="{curve_d(x1, y1, x2, y2)}"><title>{e(tip)}</title></path>'
    dimc = " is-dim" if dim else ""
    mx = x2 - 11
    if tone == "bad":
        out += f'<g class="fui-edge-mark fui-edge-mark-bad{dimc}" transform="translate({mx} {y2})"><circle r="7"/><path d="M-3 -3 L3 3 M3 -3 L-3 3"/></g>'
    elif tone == "warn":
        out += f'<g class="fui-edge-mark fui-edge-mark-warn{dimc}" transform="translate({mx} {y2})"><circle r="7"/><path d="M0 -3.5 V1 M0 3.4 V3.5"/></g>'
    if label:
        wl = 8 + len(label) * 5.7
        cx = x2 - (22 + wl / 2) if tone in ("bad", "warn") else x2 - (6 + wl / 2)
        out += (f'<g class="fui-edge-label{dimc}" transform="translate({cx - wl / 2:.1f} {y2 - 9})"><rect width="{wl:.1f}" height="18" rx="9"/>'
                f'<text x="{wl / 2:.1f}" y="12.5" text-anchor="middle">{e(label)}</text></g>')
    return out


# The bridge as the dashboard draws it. (id, column, row y, kind glyph, name, line, tone, href)
def nodes(states):
    s = {"facade": "ok", "ops": "ok", "hl7": "ok", "etl": "ok", "cdr": "ok", "term": "ok", "cdm": "ok"}
    s.update(states or {})
    line = {
        "facade": {"ok": "On · /fhir", "muted": "Off"},
        "ops": {"ok": "On · $tofhir, $toopenehr", "muted": "Off"},
        "hl7": {"ok": "On · MLLP :2575", "muted": "Off", "bad": "Listener down"},
        "etl": {"ok": "Batch · its own process", "neutral": "No run yet"},
        "cdr": {"ok": "FerroEHR · Up", "bad": "No answer · 3 attempts", "warn": "Slow · 2,940 ms"},
        "term": {"ok": "FerroTERM · Up", "bad": "Unreachable · 503", "neutral": "Not configured"},
        "cdm": {"ok": "PostgreSQL · Up", "bad": "Unreachable"},
    }
    def ln(k):
        return line[k].get(s[k], WORD[s[k]])
    return s, [
        ("clients", "call", 1.0, "clients", "FHIR clients", "SMART App Launch", "ok", "facade-security.html"),
        ("senders", "call", 2.5, "senders", "HL7 v2 senders", "2 accepted senders", "ok" if s["hl7"] != "muted" else "muted", "hl7v2-settings.html"),
        ("sched", "call", 3.5, "sched", "ETL run", "by your scheduler", "ok", "omop.html"),
        ("facade", "face", 0.5, "facade", "FHIR facade", ln("facade"), s["facade"], "facade.html"),
        ("ops", "face", 1.5, "ops", "Operations", ln("ops"), s["ops"], "facade-capability.html"),
        ("hl7", "face", 2.5, "hl7", "HL7 v2", ln("hl7"), s["hl7"], "hl7v2.html"),
        ("etl", "face", 3.5, "etl", "OMOP ETL", ln("etl"), s["etl"], "omop.html"),
        ("cdr", "up", 1.0, "cdr", "openEHR CDR", ln("cdr"), s["cdr"], "connection.html"),
        ("term", "up", 2.0, "term", "Terminology server", ln("term"), s["term"], "connection-terminology.html"),
        ("cdm", "up", 3.5, "cdm", "OMOP CDM database", ln("cdm"), s["cdm"], "connection-cdm.html"),
    ]


EDGES = [("clients", "facade"), ("clients", "ops"), ("senders", "hl7"), ("sched", "etl"),
         ("facade", "hub"), ("ops", "hub"), ("hl7", "hub"), ("etl", "hub"),
         ("hub", "cdr"), ("hub", "term"), ("hub", "cdm")]


def worst(ts):
    for t in ("bad", "warn", "neutral", "ok"):
        if t in ts:
            return t
    return "muted"


def flow_svg(states=None, hover=None, pulse_frame=False, hub_line="9 contexts · 4 OMOCL", hub_tone="ok"):
    s, ns = nodes(states)
    col = {"call": (X_CALL, W_CALL), "face": (X_FACE, W_FACE), "up": (X_UP, W_UP)}
    pos = {}
    N, E = [], []
    for nid, c, r, g, name, line, tone, href in ns:
        x, w = col[c]
        y = TOP + (r - 0.5) * ROW
        pos[nid] = (x, y, w, tone)
    hy = TOP + H / 2 + 1.5 * ROW - 36
    pos["hub"] = (X_HUB, hy, W_HUB, hub_tone)
    # The path a hovered card stands on, from its caller through the hub: the
    # cards and edges on it stay lit, the pulse runs along it, as in F9.
    CHAINS = {"cdr": [("clients", "facade"), ("facade", "hub"), ("hub", "cdr")], "facade": [("clients", "facade"), ("facade", "hub"), ("hub", "cdr")],
              "ops": [("clients", "ops"), ("ops", "hub"), ("hub", "cdr")], "hl7": [("senders", "hl7"), ("hl7", "hub"), ("hub", "cdr")],
              "etl": [("sched", "etl"), ("etl", "hub"), ("hub", "cdm")], "term": [("clients", "facade"), ("facade", "hub"), ("hub", "term")],
              "cdm": [("sched", "etl"), ("etl", "hub"), ("hub", "cdm")]}
    chain = CHAINS.get(hover, []) if hover else []
    on = {n for edge in chain for n in edge}
    pulse = ""
    for a, b in EDGES:
        xa, ya, wa, ta = pos[a]
        xb, yb, wb, tb = pos[b]
        ha = 72 if a == "hub" else H
        hb = 72 if b == "hub" else H
        x1, y1, x2, y2 = xa + wa, ya + ha / 2, xb, yb + hb / 2
        tone = tb if b != "hub" else ta
        if b == "hub" and ta == "muted":
            tone = "muted"
        path = (a, b) in chain
        dim = bool(hover) and not path
        label = None
        if tone == "bad":
            label = "no answer" if b == "cdr" else ("503" if b == "term" else "down")
        elif tone == "warn":
            label = "2,940 ms"
        tip = f"{a} to {b}: {WORD[tone]}"
        E.append(curve(x1, y1, x2, y2, tone, tip, path=path, dim=dim, label=label))
    if chain:
        segs = []
        for a, b in chain:
            xa, ya, wa, _ = pos[a]
            xb, yb, wb, _ = pos[b]
            ha = 72 if a == "hub" else H
            hb = 72 if b == "hub" else H
            segs.append(curve_d(xa + wa, ya + ha / 2, xb, yb + hb / 2))
        if segs:
            d = " ".join(segs)
            pulse = f'<path class="fui-path-glow" d="{d}"/><path class="fui-path-pulse{" is-frame" if pulse_frame else ""}" d="{d}" pathLength="100"/>'
    for nid, c, r, g, name, line, tone, href in ns:
        x, y, w, _ = pos[nid]
        cls = ("is-path " if hover and nid in on else "") + ("is-dim" if hover and nid not in on else "")
        lab = f"{name}, {line}: {WORD.get(tone, tone)}. Opens its page."
        kind = {"call": "svc", "face": "node", "up": "svc"}[c]
        N.append(tnode(kind, GLYPH_OF[g], x, y, name, line, tone, lab, w, cls=cls.strip(), href=href))
    hcls = "is-path" if hover else ""
    N.append(tnode("gateway", "layers", X_HUB, hy, "FerroBRIDGE", hub_line, hub_tone, f"FerroBRIDGE, the running mapping set: {hub_line}. Opens the Mapping set.", W_HUB, h=72, cls=hcls, href="mappings.html"))
    lanes = "".join(f'<text class="fui-topology-lane" x="{x}" y="12">{t}</text>' for x, t in
                    ((X_CALL, "Callers"), (X_FACE, "Faces"), (X_HUB, "Mapping set"), (X_UP, "Upstreams")))
    width = X_UP + W_UP + 16
    height = TOP + 4 * ROW
    inner = (f'<g class="fui-topology-bands">{lanes}</g><g class="fui-topology-edges">{"".join(E)}</g>'
             f'<g class="fui-topology-pulse" aria-hidden="true">{pulse}</g><g class="fui-topology-nodes">{"".join(N)}</g>')
    return (f'<div class="fui-topology-canvas"><svg class="fui-topology-svg" viewBox="-8 -4 {width} {height}" role="group" '
            f'aria-label="The bridge as a flow: callers, faces, the mapping set and the upstreams; each card opens its page">{inner}</svg></div>')


def flow_list(states=None, hub_line="9 contexts · 4 OMOCL", hub_tone="ok"):
    """The phone twin: the same cards as rows, lane by lane, each opening its page."""
    from common import health_dot
    _, ns = nodes(states)
    lanes = [("Callers", "call"), ("Faces", "face"), ("Mapping set", "hub"), ("Upstreams", "up")]
    out = ""
    for title, col in lanes:
        if col == "hub":
            items = [("FerroBRIDGE", hub_line, hub_tone, "mappings.html")]
        else:
            items = [(n, l, t, h) for _, c, _, _, n, l, t, h in ns if c == col]
        rows = "".join(f'<li><a class="fui-row-link" href="{h}">{health_dot(t, WORD.get(t, t))}<span class="fui-pipeline-name">{e(n)}</span><span class="fui-pipeline-line">{e(l)}</span></a></li>' for n, l, t, h in items)
        out += f'<li><p class="fui-pipeline-lane">{title}</p><ul>{rows}</ul></li>'
    return f'<ul class="fui-pipeline-list" aria-label="The bridge as a flow, lane by lane">{out}</ul>'


RULE = [("ok", "Up", "solid", "it answers"), ("warn", "Degraded", "dashed, marked !", "it answers slowly or with an error"),
        ("bad", "Unreachable", "dotted, marked ✕", "it does not answer"), ("muted", "Off", "thin, faded", "the face is not configured to run")]


def help_dialog():
    from common import button, icon
    items = "".join(f'<li><svg class="fui-line-sample" width="26" height="10" aria-hidden="true"><path class="fui-edge fui-edge-{t}" d="M1 5 H25"/></svg>'
                    f'<span><strong>{w}</strong> · {st}: {rule}</span></li>' for t, w, st, rule in RULE)
    return ('<div role="dialog" aria-modal="true" aria-labelledby="fh-title" class="fui-dialog fui-modal mk-open" open>'
            f'<h2 id="fh-title" class="fui-dialog-title">{icon("info", "18")} How the flow is drawn</h2>'
            f'<ul class="mk-plain-list fui-topology-rule">{items}</ul>'
            '<p class="fui-small fui-muted">Colour and line both say the state, and only a degraded or unreachable edge carries a label. A face shows its lane; an upstream shows the bridge\'s readiness indicator for it. A card opens its page.</p>'
            f'<div class="fui-dialog-actions">{button("secondary", "Close", keys="Esc")}</div></div>')


def flow_card(states=None, hover=None, pulse_frame=False, foot=None, hub_line="9 contexts · 4 OMOCL", hub_tone="ok"):
    from common import icon
    f = foot or "3 faces on, the ETL a batch · 3 of 3 upstreams up."
    help_ = f'<a href="flow-help.html" class="fui-button fui-button-ghost fui-button-small" aria-haspopup="dialog">{icon("info", "16")}How it is drawn</a>'
    head = (f'<div class="fui-panel-head"><h2 class="fui-card-title"><span>Flow</span></h2>'
            f'<div class="fui-actions">{help_}{link_button("secondary", "Connections", "connections.html", glyph_end="chevron-right", small=True)}</div></div>')
    return (f'<section class="fui-card fui-panel fui-pipeline" aria-label="Flow">{head}{flow_svg(states, hover, pulse_frame, hub_line, hub_tone)}{flow_list(states, hub_line, hub_tone)}'
            f'<p class="fui-pipeline-foot">{f}</p></section>')
