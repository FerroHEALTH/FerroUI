# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Topology: a live map of the federation as the gateway sees it, drawn as
FerroUI's `topology` component (F9): a left-to-right layered graph with
the gateway at its root, Organisations, then Nodes, beside the services the
gateway asks; edges coloured and dashed by the last probe; a legend that
says the rule; zoom, pan, fit and a minimap; a side-panel slot; and a List
twin with the same data and actions. Only what FerroFED really connects is
drawn; the national contact point's connector is marked planned."""
from common import GLYPHS, MENU_SEP, menu, menu_item, button, card, check, definitions, e, empty_state, health_dot, icon, icon_button, input_, link_button, page_heading, select, shell, toast, toasts, when
from data import ORGS, STATE_WORD, worst, word, status_counts
import data
import routes
from parts import skeleton_lines
from pg_orgs import sparkline, BAD_PROBES, BAD_FAIL
from common import state as state_html
from routes import node_href as _nh, org_href as _oh


def node_href(n):
    return _nh(n["id"])
from pg_shell import refusal, tech_detail

H_NODE, ROW, GAP = 56, 66, 14
# The hub: services on the left (mirrored), the gateway in the centre with its
# callers above, the organisations and their nodes on the right.
W_SVC, W_SG, W_GW, W_ORG, W_NODE = 222, 236, 176, 272, 182
GAP_OUT, GAP_HUB = 92, 84
X_SVC = 0
X_SG = X_SVC + W_SVC + GAP_OUT
X_GW = X_SG + W_SG + GAP_HUB
X_ORG = X_GW + W_GW + GAP_HUB
X_NODE = X_ORG + W_ORG + GAP_OUT
W_NCP = 204
LANE_TOP = 118

# The services as the Services page groups them; only what is set is drawn.
SERVICE_GROUPS = [(g, [(s[1], s[2], s[3], s[6], s[7]) for s in data.SET_UP if s[0] == g]) for g in data.SERVICE_GROUPS]
SERVICE_GROUPS = [(g, xs) for g, xs in SERVICE_GROUPS if xs]
SERVICES = [x for _, xs in SERVICE_GROUPS for x in xs]

GLYPH_OF = {"gateway": "network", "org": "building-2", "node": "server", "svc": "plug", "group": "layers", "ncp": "globe"}
SLOW = 1250


def glyph(name, x, y, size=18):
    return f'<svg x="{x}" y="{y}" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{GLYPHS[name]}</svg>'


def fit_text(t, n):
    return t if len(t) <= n else t[:n - 1] + "…"


def dot_svg(tone, x, y):
    if tone == "warn":
        return f'<rect class="fui-tdot fui-tone-warn" x="{x - 3.5}" y="{y - 3.5}" width="7" height="7" transform="rotate(45 {x} {y})"/>'
    if tone == "bad":
        return f'<path class="fui-tdot fui-tone-bad" d="M{x} {y - 4.5} L{x + 4.5} {y + 3.5} L{x - 4.5} {y + 3.5} Z"/>'
    if tone == "muted":
        return f'<path class="fui-tdot fui-tdot-excluded" d="M{x - 4} {y} H{x + 4}" stroke="currentColor" stroke-width="1.5"/>'
    if tone in ("neutral", "planned"):
        return f'<circle class="fui-tdot fui-tdot-ring" cx="{x}" cy="{y}" r="3.5"/>'
    return f'<circle class="fui-tdot fui-tone-ok" cx="{x}" cy="{y}" r="4"/>'


def tnode(kind, x, y, name, line, tone, label, w, h=H_NODE, cls="", collapsible=None, href=None, tip=None, chars=None, mirror=False):
    """A card is a link to its thing's own page; the chevron (collapse) and the
    Details button (the quick look) sit beside the link, never inside it."""
    t = "neutral" if tone in ("muted", "planned") else tone
    extra = (" is-muted" if tone == "muted" else "") + (" is-planned" if tone == "planned" else "") + (f" {cls}" if cls else "")
    toggle = ""
    if collapsible is not None:
        g = "chevron-right" if collapsible else "chevron-down"
        toggle = f'<g class="fui-tnode-toggle" role="button" tabindex="-1" aria-label="{"Expand" if collapsible else "Collapse"} {e(name)}">{glyph(g, w - 26, 9, 16)}</g>'
    info = ""
    if kind in ("node", "svc", "org"):
        ix = w - (50 if collapsible is not None else 26)
        info = (f'<g class="fui-tnode-info" role="button" tabindex="-1" aria-label="Details of {e(name)}" transform="translate({ix} 8)">'
                f'<title>Details: a quick look</title><rect width="18" height="18" rx="5"/>{glyph("info", 3, 3, 12)}</g>')
    mono = " fui-tnode-mono" if kind == "node" else ""
    room = (w - 46 - (28 if collapsible is not None else 10))
    n_chars = chars or int(room / (7.4 if kind == "node" else 6.7))
    dot = dot_svg(tone, 47, h / 2 + 10) if kind not in ("gateway", "ncp") else ""
    lx = 56 if dot else 42
    body = (f'<title>{e((tip or label) + (" · " + line if line and line not in (tip or label) else ""))}</title><rect class="fui-tnode-box" width="{w}" height="{h}" rx="8"/><rect class="fui-tnode-bar" width="4" height="{h - 16}" x="0" y="8" rx="2"/>'
            f'<g class="fui-tnode-glyph">{glyph(GLYPH_OF[kind], 14, h / 2 - 9)}</g>'
            f'<text class="fui-tnode-name{mono}" x="42" y="{h / 2 - 4}">{e(fit_text(name, n_chars))}</text>'
            f'{dot}<text class="fui-tnode-line" x="{lx}" y="{h / 2 + 14}">{e(fit_text(line, int((w - lx - 10) / 6.2)))}</text>')
    link = f'<a class="fui-tnode-link" href="{href or "#"}" aria-label="{e(label)}">{body}</a>'
    return f'<g class="fui-tnode fui-tnode-{kind} fui-tone-{t}{extra}" transform="translate({x} {y})">{link}{info}{toggle}</g>'


def org_line(tones):
    n = len(tones)
    live = [t for t in tones if t != "muted"]
    if not live:
        return "Excluded"
    bad = [t for t in live if t != "ok"]
    if not bad:
        return f"{n} node{'s' if n != 1 else ''} · all reachable" if n > 1 else "1 node · Reachable"
    w = worst(bad)
    return f"{len([t for t in tones if t == w])} of {n} {word(w)}" if n > 1 else STATE_WORD[w]


def svc_line(items):
    n = len(items)
    bad = [x[3] for x in items if x[3] != "ok"]
    if not bad:
        return f"{n} services · all reachable" if n > 1 else "1 service · reachable"
    w = worst(bad)
    return f"{len(bad)} of {n} {word(w)}" if n > 1 else STATE_WORD[w]


def latency_label(tone, p50):
    if tone == "bad":
        return "no answer"
    if tone == "warn":
        return "503"
    if p50 and p50 >= SLOW:
        return f"{p50:,} ms"
    return None


def curve_d(x1, y1, x2, y2):
    """A cubic Bézier with horizontal tangents at both ends: it leaves and
    enters each card sideways and flows in an S."""
    c = (x2 - x1) * 0.48
    return f"M{x1} {y1} C{x1 + c:.1f} {y1}, {x2 - c:.1f} {y2}, {x2} {y2}"


def curve(x1, y1, x2, y2, tone, eid, tip, path=False, dim=False, label=None, mark=True):
    """One edge: the state's colour and line along the curve; the latency
    and the state marker at the child end, on pills that mask the line."""
    cls = f"fui-edge fui-edge-{tone}" + (" is-path" if path else "") + (" is-dim" if dim else "")
    out = f'<path id="{eid}" class="{cls}" d="{curve_d(x1, y1, x2, y2)}"><title>{e(tip)}</title></path>'
    d = 1 if x2 > x1 else -1
    dimc = " is-dim" if dim else ""
    pc = " is-path" if path else ""
    mx = x2 - d * 11
    if not mark:
        pass
    elif tone == "bad":
        out += f'<g class="fui-edge-mark fui-edge-mark-bad{dimc}" transform="translate({mx} {y2})"><circle r="7"/><path d="M-3 -3 L3 3 M3 -3 L-3 3"/></g>'
    elif tone == "warn":
        out += f'<g class="fui-edge-mark fui-edge-mark-warn{dimc}" transform="translate({mx} {y2})"><circle r="7"/><path d="M0 -3.5 V1 M0 3.4 V3.5"/></g>'
    if label:
        wl = 8 + len(label) * 5.7
        cx = x2 - d * (22 + wl / 2) if tone in ("bad", "warn") else x2 - d * (6 + wl / 2)
        out += (f'<g class="fui-edge-label{dimc}{pc}" transform="translate({cx - wl / 2:.1f} {y2 - 9})"><rect width="{wl:.1f}" height="18" rx="9"/>'
                f'<text x="{wl / 2:.1f}" y="12.5" text-anchor="middle">{e(label)}</text></g>')
    return out


def build_graph(states=None, collapsed=(), dim=None, selected=None, hover=None, traffic=False, large=False, services_collapsed=False, focus=None,
                pulse_frame=False, collapse_healthy=False, show=None, p50s=None, no_orgs=False):
    """The hub: the callers above the gateway, the services it asks on its left
    (mirrored), the organisations and their nodes on its right. Each edge is a
    curve with horizontal tangents; a parent's children fan out from one
    anchor in their own order, so no two curves cross."""
    states = states or {}
    orgs = []
    if large:
        from data import large_orgs
        counts = [3, 2, 3, 2, 4, 2, 3, 2, 3, 2, 2, 2]
        for i, (o, n) in enumerate(zip(large_orgs(), counts)):
            tones = ["ok"] * n
            if i == 4:
                tones[1] = "warn"
            if i == 9:
                tones[0] = "bad"
            if i == 7:
                tones = ["neutral"] * n
            nodes = [{"id": f"node-{i + 1:02d}{chr(97 + k)}", "tone": t, "p50": None if t in ("bad", "neutral") else (2180 if t == "warn" else 30 + (i * 17 + k * 11) % 90)} for k, t in enumerate(tones)]
            orgs.append({"slug": "big", "name": o["name"], "nodes": nodes})
    elif no_orgs:
        pass
    else:
        for o in ORGS:
            nodes = []
            for n in o["nodes"]:
                t = states.get(n["id"], n["tone"])
                p = (p50s or {}).get(n["id"], n["endpoints"][0]["p50"])
                if show is not None and t not in show:
                    continue
                nodes.append({"id": n["id"], "tone": t, "p50": None if t in ("bad", "neutral", "muted") else p, "href": node_href(n)})
            if nodes:
                orgs.append({"slug": o["slug"], "name": o["name"], "nodes": nodes})

    def matches(name):
        return dim is None or dim.lower() in name.lower()

    hover_org = next((o["name"] for o in orgs if hover and (o["name"] == hover or any(n["id"] == hover for n in o["nodes"]))), None)
    E, N, LANES, TR, MINI = [], [], [], [], []
    hover_on = bool(hover)

    # right side: organisations and their nodes
    y = LANE_TOP
    right = []
    for o in orgs:
        tones = [n["tone"] for n in o["nodes"]]
        is_col = o["name"] in collapsed or (collapse_healthy and all(t == "ok" for t in tones))
        ys = [y] if is_col else [y + k * ROW for k in range(len(o["nodes"]))]
        y = ys[-1] + ROW + GAP
        right.append((o, (ys[0] + ys[-1]) / 2, ys, is_col))
    right_h = y - GAP
    # left side: the service groups and their services
    y = LANE_TOP
    left = []
    for gname, items in SERVICE_GROUPS:
        items = [x for x in items if show is None or x[3] in show]
        if not items:
            continue
        ys = [y] if services_collapsed else [y + k * ROW for k in range(len(items))]
        y = ys[-1] + ROW + GAP
        left.append((gname, items, (ys[0] + ys[-1]) / 2, ys))
    left_h = y - GAP
    bottom = max(right_h, left_h)
    gcy = (LANE_TOP + bottom - (ROW - H_NODE)) / 2
    gy = gcy - 36
    gx_l, gx_r = X_GW, X_GW + W_GW

    LANES.append(f'<text class="fui-topology-lane" x="{X_GW + W_GW / 2}" y="12" text-anchor="middle">Calls the gateway</text>')
    LANES.append(f'<text class="fui-topology-lane" x="{X_SVC}" y="{LANE_TOP - 16}">Services the gateway asks</text>')
    LANES.append(f'<text class="fui-topology-lane" x="{X_ORG}" y="{LANE_TOP - 16}">Organisations and nodes</text>')
    LANES.append(f'<line class="fui-topology-hairline" x1="{X_SVC}" x2="{X_NODE + W_NODE}" y1="{LANE_TOP - 34}" y2="{LANE_TOP - 34}"/>')

    # callers: the national contact point, planned, above the gateway
    ncx = X_GW + W_GW / 2 - W_NCP / 2
    N.append(tnode("ncp", ncx, 22, "National contact point", "MyHealth@EU · calls the FHIR face", "neutral",
                   "National contact point: MyHealth@EU calls the gateway's FHIR face", W_NCP, chars=24, href="self-description.html"))
    E.append(f'<path class="fui-edge fui-edge-neutral{" is-dim" if hover_on else ""}" d="M{X_GW + W_GW / 2} {22 + H_NODE} C{X_GW + W_GW / 2} {gy - 10}, {X_GW + W_GW / 2} {gy - 10}, {X_GW + W_GW / 2} {gy}"><title>The national contact point calls the FHIR face</title></path>')
    N.append(tnode("gateway", X_GW, gy, "FerroFED gateway", "2 replicas · in step", "ok", "FerroFED gateway, the hub: 2 replicas in step", W_GW, h=72, href="self-description.html",
                   cls=("is-focused " if focus == "gateway" else "") + ("is-path" if hover_on else "")))
    MINI.append((X_GW, gy, W_GW, 72, "ok"))
    MINI.append((ncx, 22, W_NCP, H_NODE, "neutral"))

    pulse = ""
    for o, oy, ys, is_col in right:
        tones = [n["tone"] for n in o["nodes"]]
        live = [t for t in tones if t != "muted"]
        w = worst(live) if live else "muted"
        on = hover_org == o["name"]
        dimo = (not matches(o["name"]) and not any(matches(n["id"]) for n in o["nodes"])) or (hover_on and not on)
        E.append(curve(gx_r, gcy, X_ORG, oy + H_NODE / 2, w, f"e-o-{o['name'].lower().replace(' ', '-')}", f"Gateway to {o['name']}: the worst of its nodes, {STATE_WORD[w]}", path=on, dim=dimo, mark=is_col))
        N.append(tnode("org", X_ORG, oy, o["name"], org_line(tones), w, f'{o["name"]}, organisation, {org_line(tones)}', W_ORG, collapsible=is_col,
                       cls=("is-dim " if dimo else "") + ("is-path" if on else ""), href=_oh(o["slug"]) if o["slug"] != "big" else "organisation.html"))
        MINI.append((X_ORG, oy, W_ORG, H_NODE, w))
        if is_col:
            continue
        for n, ny in zip(o["nodes"], ys):
            onn = on and (hover == o["name"] or hover == n["id"])
            dimn = (not matches(n["id"]) and not matches(o["name"])) or (hover_on and not onn)
            tip = f'{o["name"]} to {n["id"]}: {STATE_WORD[n["tone"]]}' + (", answers 503 since 09:38" if n["tone"] == "warn" else "") + (f', median {n["p50"]:,} ms' if n["p50"] else "") + ("" if n["tone"] in ("muted", "neutral") else ", last probe 09:41:58")
            E.append(curve(X_ORG + W_ORG, oy + H_NODE / 2, X_NODE, ny + H_NODE / 2, n["tone"], f"e-{n['id']}", tip, path=onn, dim=dimn, label=latency_label(n["tone"], n["p50"])))
            nline = STATE_WORD[n["tone"]] + (" · 503" if n["tone"] == "warn" else "") + (" · no answer" if n["tone"] == "bad" else "")
            cls = ("is-selected " if selected == n["id"] else "") + ("is-dim " if dimn else "") + ("is-path " if onn else "") + ("is-focused" if focus == n["id"] else "")
            reason = " (its organisation is excluded while its CDR moves)" if n["tone"] == "muted" else ""
            N.append(tnode("node", X_NODE, ny, n["id"], nline, n["tone"], f'{n["id"]}, node of {o["name"]}, {STATE_WORD[n["tone"]]}{reason}', W_NODE,
                           cls=cls.strip(), href="node.html" if large else n["href"], tip=tip + reason))
            MINI.append((X_NODE, ny, W_NODE, H_NODE, n["tone"]))
            flow = curve_d(gx_r, gcy, X_ORG, oy + H_NODE / 2) + " " + curve_d(X_ORG + W_ORG, oy + H_NODE / 2, X_NODE, ny + H_NODE / 2)
            if onn:
                pulse = f'<path class="fui-path-glow" d="{flow}"/><path class="fui-path-pulse{" is-frame" if pulse_frame else ""}" d="{flow}" pathLength="100"/>'
            if traffic and n["tone"] == "ok":
                TR.append(f'<path id="f-{n["id"]}" class="fui-flow" d="{flow}"/>')
                for k in range(2):
                    TR.append(f'<circle class="fui-traffic" r="3"><animateMotion dur="2.2s" begin="-{k * 1.1 + len(n["id"]) * 0.13:.1f}s" repeatCount="indefinite"><mpath href="#f-{n["id"]}"/></animateMotion></circle>')
    for gname, items, gyy, ys in left:
        gw_ = worst([x[3] for x in items])
        dg = bool(dim) or hover_on
        E.append(curve(gx_l, gcy, X_SG + W_SG, gyy + H_NODE / 2, gw_, f"e-g-{gname[:5]}", f"Gateway to the {gname} services: {svc_line(items)}", dim=dg, mark=services_collapsed))
        N.append(tnode("group", X_SG, gyy, gname, svc_line(items), gw_, f"{gname} services, {svc_line(items)}", W_SG, collapsible=services_collapsed, cls="is-dim" if dg else "", href="services.html#" + gname.split()[0].lower()))
        MINI.append((X_SG, gyy, W_SG, H_NODE, gw_))
        if services_collapsed:
            continue
        for (k, nm, role, t, p), yy in zip(items, ys):
            tip = f"{nm}: {STATE_WORD[t]}" + (", answers 503 since 09:31" if t == "warn" else (f", median {p:,} ms" if p else "")) + ("" if t == "neutral" else ", last probe 09:41:52")
            E.append(curve(X_SG, gyy + H_NODE / 2, X_SVC + W_SVC, yy + H_NODE / 2, t, f"e-{k}", tip, dim=dg, label="503" if t == "warn" else latency_label(t, p)))
            N.append(tnode("svc", X_SVC, yy, nm, STATE_WORD[t] + (" · 503" if t == "warn" else ""), t, f"{nm}, {gname} service, {STATE_WORD[t]}", W_SVC, cls="is-dim" if dg else "", href=routes.svc_href(k), tip=tip))
            MINI.append((X_SVC, yy, W_SVC, H_NODE, t))
            if traffic and t == "ok" and k in ("pix", "nvi", "arr"):
                flow = curve_d(gx_l, gcy, X_SG + W_SG, gyy + H_NODE / 2) + " " + curve_d(X_SG, gyy + H_NODE / 2, X_SVC + W_SVC, yy + H_NODE / 2)
                TR.append(f'<path id="f-{k}" class="fui-flow" d="{flow}"/><circle class="fui-traffic" r="3"><animateMotion dur="2.4s" begin="-0.8s" repeatCount="indefinite"><mpath href="#f-{k}"/></animateMotion></circle>')
    width = X_NODE + W_NODE + 16
    height = bottom + 16
    svg_inner = (f'<g class="fui-topology-bands">{"".join(LANES)}</g><g class="fui-topology-edges">{"".join(E)}</g>'
                 f'<g class="fui-topology-pulse" aria-hidden="true">{pulse}</g><g class="fui-topology-traffic" aria-hidden="true">{"".join(TR)}</g><g class="fui-topology-nodes">{"".join(N)}</g>')
    mini = [("x", x, yy, w, h, t) for x, yy, w, h, t in MINI]
    return svg_inner, width, height, mini


def minimap(mini, width, height, view):
    sc = 168 / width
    rects = "".join(f'<rect class="fui-minimap-node fui-tone-{"neutral" if t in ("muted", "planned") else t}" x="{x * sc:.1f}" y="{y * sc:.1f}" width="{w * sc:.1f}" height="{max(h * sc, 2):.1f}" rx="1"/>' for _, x, y, w, h, t in mini)
    vx, vy, vw, vh = view
    return (f'<div class="fui-topology-minimap" aria-label="Minimap: the whole graph, the frame is what the map shows"><svg viewBox="-4 -4 {168 + 8} {height * sc + 8:.0f}" width="176" height="{height * sc + 8:.0f}">'
            f'{rects}<rect class="fui-minimap-view" x="{vx * sc:.1f}" y="{vy * sc:.1f}" width="{vw * sc:.1f}" height="{vh * sc:.1f}" rx="2"/></svg></div>')


RULE = [
    ("ok", "Reachable", "solid", "the last probe got a status below 500, within half the per-node budget (1,250 ms)"),
    ("warn", "Degraded", "dashed, marked !", "it answered with a 5xx, or slower than half the budget"),
    ("bad", "Unreachable", "dotted, marked ✕", "no answer within the per-node budget (2,500 ms)"),
    ("neutral", "Not probed", "dash and dot", "no probe has run yet: a state, never a value"),
    ("muted", "Excluded", "thin, faded, a dash for its mark", "excluded, or its organisation excluded: asked nothing"),
]


def dot_html(t):
    tt = {"planned": "neutral"}.get(t, t)
    return health_dot(tt if tt != "muted" else "muted")


def line_sample(t):
    return f'<svg class="fui-line-sample" width="26" height="10" aria-hidden="true"><path class="fui-edge fui-edge-{t}" d="M1 5 H25"/></svg>'


def help_popover():
    items = "".join(f'<li>{line_sample(t)}<span><strong>{w}</strong> · {st}: {rule}</span></li>' for t, w, st, rule in RULE)
    return ('<div class="fui-popover fui-topology-help-pop" role="dialog" aria-labelledby="th-title"><p id="th-title" class="fui-popover-title">How the edges are drawn</p>'
            f'<ul class="mk-plain-list fui-topology-rule">{items}</ul>'
            '<p class="fui-small fui-muted">An organisation, a service group and the branch into each show the worst of what they hold. The gateway probes every node and service every 60 s with the credentials a query uses. '
            'Only a degraded, unreachable or slow branch carries its latency; every time and p99 is in the tooltip and the side panel. No specification governs this rule: it is FerroFED\'s own design.</p></div>')


COUNTS = status_counts()


def status_menu(shown=("ok", "warn", "bad", "neutral", "muted"), open_=False, counts=None):
    counts = counts or COUNTS
    n = len(shown)
    word = "Status" if n == 5 else f"Status · {n} of 5"
    items = "".join(
        f'<label class="fui-menu-item fui-menu-check" role="menuitemcheckbox" aria-checked="{"true" if t in shown else "false"}"><input type="checkbox"{" checked" if t in shown else ""} tabindex="-1" />'
        f'{line_sample(t)}{dot_html(t)}<span class="fui-menu-item-text"><span class="fui-menu-item-label">{STATE_WORD[t]}</span></span><span class="fui-menu-count">{counts[t]}</span></label>'
        for t in ("ok", "warn", "bad", "neutral", "muted"))
    m = ""
    if open_:
        foot = (f'<div class="fui-menu-sep" role="separator"></div><div class="fui-menu-foot"><p class="fui-small fui-muted">Counts nodes and services, each once. Colour and line both say the state. A group and the branch into it show the worst of what they hold; '
                'only a degraded, unreachable or slow branch carries its latency.</p><a href="topology-help.html" class="fui-small">How edges are drawn</a></div>')
        m = f'<div role="menu" class="fui-menu fui-menu-compact fui-menu-anchored fui-menu-start mk-status-menu" aria-label="Show states"><div class="fui-menu-items">{items}</div>{foot}</div>'
    return (f'<span class="fui-anchor"><button type="button" class="fui-button fui-button-secondary{" is-filtered" if n < 5 else ""}" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}">'
            f'{icon("list-filter", "16")}{word}{icon("chevron-down", "16")}</button>{m}</span>')


def refresh_control(open_=False, loading=False):
    m = ""
    if open_:
        m = menu([menu_item("Refresh now", "refresh-cw"), MENU_SEP] + [menu_item(w, "check" if w == "Every 30 s" else None, current=(w == "Every 30 s")) for w in ("Every 30 s", "Every minute", "Every 5 minutes", "Off")], label="Refresh")
    upd = "Reading…" if loading else 'Updated <time class="fui-time" datetime="2026-10-21T09:42:11" title="2026-10-21 09:42:11 CEST">13 s ago</time> · every 30 s'
    return (f'<span class="fui-topology-updated"><span class="fui-small fui-muted" role="status">{upd}</span>'
            f'<span class="fui-anchor"><button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="Refresh now, or choose how often" title="Refresh">{icon("refresh-cw", "16")}</button>{m}</span></span>')


def toolbar(filter_text="", traffic=False, shown=("ok", "warn", "bad", "neutral", "muted"), status_open=False, attention=False, view="auto", refresh_open=False, loading=False):
    att = f'<button type="button" class="fui-chip-check" aria-pressed="{"true" if attention else "false"}">{icon("triangle-alert", "14")}Needs attention</button>'
    left = (f'<div class="fui-topology-find"><span class="fui-search-glyph">{icon("search", "16")}</span>{input_("t-find", filter_text, "Filter by name", "search", extra=chr(32) + "aria-label=" + chr(34) + "Filter the map by name" + chr(34))}</div>'
            + status_menu(shown, status_open) + att)
    return (f'<div class="fui-topology-bar"><div class="fui-topology-bar-start">{left}</div>'
            f'<div class="fui-topology-bar-end">{refresh_control(refresh_open, loading)}{view_switch(view)}</div></div>')


def active_filters(chips, shown=None):
    import tpl
    return tpl.active(chips, shown)


def zoom_controls(level="100 %", traffic=False, minimap_on=False, large=False, tip=None):
    """The one control cluster of every map: zoom, the zoom readout and Fit,
    then collapse, live traffic and the minimap."""
    def tb(key, glyph, label, attrs=""):
        t = f'<span class="fui-tooltip fui-tooltip-above" role="tooltip">{e(label)}</span>' if tip == key else ""
        b = icon_button(glyph, label, attrs=attrs, cls="mk-hover" if tip == key else "")
        return f'<span class="fui-anchor">{b}{t}</span>'
    return (f'<div class="fui-topology-zoom" role="toolbar" aria-label="Map controls">{tb("in", "plus", "Zoom in")}{tb("out", "minus", "Zoom out")}'
            f'<span class="fui-small fui-muted mk-zoom-readout" role="status" aria-label="Zoom {level}">{level}</span>'
            f'<button type="button" class="fui-button fui-button-ghost fui-button-small" title="Fit the whole graph">Fit</button><span class="fui-topology-zoom-sep" aria-hidden="true"></span>'
            f'{tb("collapse", "chevron-up", "Collapse every group", attrs="aria-pressed=" + chr(34) + ("true" if large else "false") + chr(34))}'
            f'{tb("traffic", "activity", "Live traffic", attrs="aria-pressed=" + chr(34) + ("true" if traffic else "false") + chr(34))}'
            f'{tb("minimap", "layers", "Minimap", attrs="aria-pressed=" + chr(34) + ("true" if minimap_on else "false") + chr(34))}</div>')


def page_topology_controls():
    frames = ""
    for key, title, line in [("collapse", "Collapse every group", "Folds the services and the organisations to one card each"),
                             ("traffic", "Live traffic", "Dots move along each edge for the last minute of requests"),
                             ("minimap", "Minimap", "A small overview in the corner, for large graphs")]:
        frames += (f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p>'
                   f'<div class="mk-zoom-demo">{zoom_controls(tip=key)}</div></div>')
    content = page_heading("Map controls, tooltips", subtitle="Each icon in the canvas control cluster names itself on hover and on focus, above the button.") + f'<div class="fui-stack">{frames}</div>'
    return shell("Map controls", "topology", content)


def canvas(svg_inner, width, height, zoom=None, pan=(0, 0), extra="", height_px=None, cls="", traffic=False, minimap_on=False, large=False):
    if zoom is None:
        svg = f'<svg class="fui-topology-svg" viewBox="-8 -8 {width} {height}" role="group" aria-label="The federation as a graph; arrows move along the edges, Enter opens a node">{svg_inner}</svg>'
        stage = svg
        st = ""
    else:
        svg = f'<svg class="fui-topology-svg" viewBox="-8 -8 {width} {height}" width="{width * zoom:.0f}" height="{height * zoom:.0f}" role="group" aria-label="The federation as a graph; arrows move along the edges, Enter opens a node">{svg_inner}</svg>'
        stage = f'<div class="fui-topology-stage" style="transform: translate({pan[0]}px, {pan[1]}px)">{svg}</div>'
        st = f' style="height: {height_px}px"' if height_px else ""
    return f'<div class="fui-topology-canvas{(" " + cls) if cls else ""}"{st}>{stage}{extra}{zoom_controls("100 %" if zoom is None else f"{int(zoom * 100)} %", traffic, minimap_on, large)}</div>'


def side_panel():
    from common import rel
    n = data.node("node-b")
    ep = n["endpoints"][0]
    eps = (f'<li><div class="fui-cluster"><code>{ep["id"]}</code>{state_html("warn", "503")}</div><span class="fui-small fui-muted">{data.CONN_WORD["query"]} · median {ep["p50"]:,} ms · last probe {rel(ep["probe"])}</span></li>')
    return (
        '<aside class="fui-topology-panel" aria-labelledby="tp-title">'
        f'<div class="fui-side-panel-head"><div><h2 id="tp-title" class="fui-card-title"><code>node-b</code></h2><p class="fui-small fui-muted">Node of <a href="organisation-degraded.html">Testkliniek Oost</a> · {e(n["product"])}</p></div>{icon_button("x", "Close the panel")}</div>'
        f'<div class="fui-side-panel-body fui-stack">{state_html("warn", "503 since 09:38")}'
        + definitions([("System id", f'<code>{n["sid"]}</code>'), ("Path", "Gateway → Testkliniek Oost → node-b")])
        + f'<div><p class="fui-label">Endpoints <span class="fui-muted">· 1</span></p><ul class="mk-plain-list mk-ff-kv">{eps}</ul></div>'
        + f'<div><p class="fui-label">Last 30 probes</p>{sparkline(BAD_PROBES, BAD_FAIL)}<p class="fui-spark-legend"><span>30 min ago</span><span>a dot: a 503</span><span>now</span></p></div>'
        + f'<div><p class="fui-label">Recent errors</p><ul class="mk-plain-list fui-small"><li>{rel("09:41:58")} · 503 Service Unavailable · a probe</li>'
          f'<li>{rel("09:40:12")} · 503 Service Unavailable · an AQL query</li><li>{rel("09:38:04")} · 503 Service Unavailable · the first one</li></ul></div>'
        + f'<div class="fui-actions">{button("primary", "Probe now", "activity", small=True)}{link_button("secondary", "Open page", "node-degraded.html", "arrow-right", small=True)}</div></div></aside>'
    )


def view_switch(view):
    m = "true" if view == "map" else "false"
    l_ = "true" if view == "list" else "false"
    auto = " fui-view-auto" if view == "auto" else ""
    return (f'<div role="group" aria-label="View" class="fui-segmented fui-view-switch{auto}">'
            f'<a href="topology.html" class="fui-segment fui-view-map" aria-pressed="{m if view != "auto" else "true"}">{icon("network", "14")}Map</a>'
            f'<a href="topology-list.html" class="fui-segment fui-view-list" aria-pressed="{l_ if view != "auto" else "false"}">{icon("list-checks", "14")}List</a></div>')


def lane_row(title, line):
    return f'<li role="presentation" class="fui-tree-lane"><span>{title}</span><span class="fui-small fui-muted">{line}</span></li>'


def group_state(items):
    tones = [x[3] for x in items]
    bad = [t for t in tones if t != "ok"]
    if not bad:
        return state_html("ok")
    w = worst(bad)
    k = len([t for t in tones if t == w])
    return state_html(w, f"{k} of {len(tones)}" if len(tones) > 1 else None)


def tree_list(states=None, collapsed=(), show=None, match=None, large=False, services_collapsed=False):
    """The List twin of the same graph: the same filter, the same collapse."""
    from common import rel
    states = states or {}
    rows = [f'<li role="treeitem" aria-level="1" aria-expanded="true" class="fui-tree-row fui-tree-root"><span class="fui-tree-name">{icon("network", "16")}<strong>FerroFED gateway</strong></span>'
            f'<span>{state_html("ok")}</span><span class="fui-small fui-muted">2 replicas in step</span><span></span><span></span></li>']
    if not match and show is None:
        rows.append(lane_row("Callers", "who calls the gateway"))
        rows.append('<li role="treeitem" aria-level="2" class="fui-tree-row fui-tree-l2"><span class="fui-tree-name">' + icon("globe", "16")
                    + '<span>National contact point</span></span><span></span><span class="fui-small fui-muted">MyHealth@EU, calls the FHIR face</span><span></span><span></span></li>')
    if not match:
        groups = [(g, [x for x in items if show is None or x[3] in show]) for g, items in SERVICE_GROUPS]
        groups = [(g, items) for g, items in groups if items]
        if groups:
            rows.append(lane_row("Services", "what the gateway asks"))
        for gname, items in groups:
            rows.append(f'<li role="treeitem" aria-level="2" aria-expanded="{"false" if services_collapsed else "true"}" class="fui-tree-row fui-tree-l2"><span class="fui-tree-name">{icon("chevron-right" if services_collapsed else "chevron-down", "14")}{icon("layers", "16")}<a href="services.html">{gname}</a></span>'
                        f'<span>{group_state(items)}</span><span class="fui-small fui-muted">{len(items)} service{"s" if len(items) != 1 else ""}</span><span></span><span></span></li>')
            if not services_collapsed:
                rows.extend(svc_rows(items))
    rows.append(lane_row("Organisations", "and their nodes"))
    if large:
        from data import large_orgs
        counts = [3, 2, 3, 2, 4, 2, 3, 2, 3, 2, 2, 2]
        for i, (o, k) in enumerate(zip(large_orgs(), counts)):
            tones = ["ok"] * k
            if i == 4:
                tones[1] = "warn"
            if i == 9:
                tones[0] = "bad"
            if i == 7:
                tones = ["neutral"] * k
            w = worst(tones)
            col = all(t == "ok" for t in tones)
            rows.append(f'<li role="treeitem" aria-level="2" aria-expanded="{"false" if col else "true"}" class="fui-tree-row fui-tree-l2"><span class="fui-tree-name">{icon("chevron-right" if col else "chevron-down", "14")}{icon("building-2", "16")}<a href="organisation.html">{e(o["name"])}</a></span>'
                        f'<span>{state_html(w)}</span><span class="fui-small fui-muted">{k} nodes</span><span></span><span></span></li>')
            if not col:
                for j, t in enumerate(tones):
                    nid = f"node-{i + 1:02d}{chr(97 + j)}"
                    rows.append(f'<li role="treeitem" aria-level="3" class="fui-tree-row fui-tree-l3"><span class="fui-tree-name">{icon("server", "16")}<a href="node.html"><code>{nid}</code></a></span>'
                                f'<span>{state_html(t)}</span><span class="fui-small fui-muted">{"–" if t in ("bad", "neutral") else "median " + str(30 + (i * 17 + j * 11) % 90) + " ms"}</span><span></span><span></span></li>')
        return f'<ul role="tree" aria-label="The federation as a list" class="fui-topology-list">{"".join(rows)}</ul>'
    for o in ORGS:
        nodes = [(n, states.get(n["id"], n["tone"])) for n in o["nodes"]]
        nodes = [(n, t) for n, t in nodes if (show is None or t in show) and (not match or match in n["id"] or match.lower() in o["name"].lower())]
        if not nodes:
            continue
        tones = [t for _, t in nodes]
        w = worst([t for t in tones if t != "muted"]) if any(t != "muted" for t in tones) else "muted"
        col = o["name"] in collapsed
        rows.append(f'<li role="treeitem" aria-level="2" aria-expanded="{"false" if col else "true"}" class="fui-tree-row fui-tree-l2{" is-excluded" if w == "muted" else ""}"><span class="fui-tree-name">'
                    f'{icon("chevron-right" if col else "chevron-down", "14")}{icon("building-2", "16")}<a href="{_oh(o["slug"])}">{e(o["name"])}</a></span>'
                    f'<span>{state_html(w)}</span><span class="fui-small fui-muted">{len(o["nodes"])} node{"s" if len(o["nodes"]) != 1 else ""}</span><span></span><span></span></li>')
        if col:
            continue
        for n, t in nodes:
            p = n["p50"] if t not in ("bad", "muted") else None
            last = ("probed " + rel(n["probe"])) if t not in ("muted", "neutral") else ("never probed" if t == "neutral" else "asked nothing")
            rows.append(f'<li role="treeitem" aria-level="3" class="fui-tree-row fui-tree-l3{" is-excluded" if t == "muted" else ""}"><span class="fui-tree-name">{icon("server", "16")}<a href="{node_href(n)}"><code>{n["id"]}</code></a></span>'
                        f'<span>{state_html(t, "503" if t == "warn" else None)}</span>'
                        f'<span class="fui-small fui-muted">{f"median {p:,} ms" if p else "–"}</span><span class="fui-small fui-muted">{last}</span>'
                        f'<span class="fui-tree-act">{button("ghost", "Probe now", "activity", small=True) if t != "muted" else ""}</span></li>')
    return f'<ul role="tree" aria-label="The federation as a list" class="fui-topology-list">{"".join(rows)}</ul>'


def svc_rows(items):
    rows = []
    for k, nm, ln, t, p in items:
        rows.append(f'<li role="treeitem" aria-level="3" class="fui-tree-row fui-tree-l3"><span class="fui-tree-name">{icon("plug", "16")}<a href="{routes.svc_href(k)}">{e(nm)}</a></span>'
                    f'<span>{state_html(t, "503" if t == "warn" else None)}</span><span class="fui-small fui-muted">{f"median {p:,} ms" if p and t == "ok" else "–"}</span>'
                    f'<span class="fui-small fui-muted">{e(ln[0].upper() + ln[1:])}</span><span class="fui-tree-act">{button("ghost", "Probe now", "activity", small=True)}</span></li>')
    return rows


def topo_page(view="auto", graph_kw=None, filter_text="", traffic=False, panel=False, zoom=None, pan=(0, 0), height_px=None, mini_view=None,
              list_kw=None, body_override=None, toast_html="", hint=False, status="up", incidents=2, help_open=False,
              shown=("ok", "warn", "bad", "neutral", "muted"), status_open=False, attention=False, chips=(), refresh_open=False, overlay=None, chips_shown=None, empty_line=""):
    graph_kw = graph_kw or {}
    head = page_heading("Topology", subtitle="How the gateway reaches each organisation, node and service, from the last probe.")
    if body_override is not None:
        body = body_override
    else:
        svg_inner, w, h, mini = build_graph(**graph_kw)
        extra = ""
        if mini_view:
            extra += minimap(mini, w, h, mini_view)
        if hint:
            extra += f'<p class="fui-topology-hint" role="note">{icon("hand", "16")}Drag to move, pinch to zoom</p>'
        cv = canvas(svg_inner, w, h, zoom, pan, extra, height_px, traffic=traffic, minimap_on=bool(mini_view), large=bool(graph_kw.get("large")))
        mapblock = f'<div class="fui-topology-map{" has-panel" if panel else ""}">{cv}{side_panel() if panel else ""}</div>'
        listblock = tree_list(**(list_kw or {}))
        body = (f'<div class="fui-topology fui-topology-view-{view}{" has-hover" if graph_kw.get("hover") else ""}{" has-filter" if filter_text else ""}">'
                + toolbar(filter_text, traffic, shown, status_open, attention, view, refresh_open) + active_filters(list(chips), chips_shown) + empty_line
                + f'<div class="fui-topology-mapwrap">{mapblock}</div><div class="fui-topology-listwrap">{listblock}</div></div>')
    content = head + f'<div class="fui-stack">{body}</div>'
    return shell("Topology", "topology", content, toast_html=toast_html, status=status, incidents=incidents, overlay=overlay)


def page_topology():
    return topo_page()


def page_topology_degraded():
    st = {"node-d": "bad"}
    return topo_page(graph_kw={"states": st}, list_kw={"states": st})


def page_topology_selected():
    return topo_page("map", graph_kw={"selected": "node-b", "focus": "node-b"}, panel=True, zoom=0.62, pan=(0, 0), height_px=760)


def page_topology_hover():
    return topo_page(graph_kw={"hover": "node-b"})


def page_topology_hover_pulse():
    return topo_page(graph_kw={"hover": "node-b", "pulse_frame": True})


def page_topology_filtered():
    show = ("warn", "bad", "neutral")
    c = status_counts()
    n = sum(c[t] for t in show)
    return topo_page("map", graph_kw={"show": show}, list_kw={"show": show}, shown=show, status_open=True, attention=True,
                     chips=["Needs attention"], chips_shown=(n, sum(c.values())))


def page_topology_search():
    return topo_page(graph_kw={"dim": "node-a"}, list_kw={"match": "node-a"}, filter_text="node-a", chips=["Name: node-a"], chips_shown=(3, 13))


def page_topology_help():
    help_pop = ('<div role="dialog" aria-modal="true" aria-labelledby="th-title" class="fui-dialog fui-modal mk-open" open>'
                f'<h2 id="th-title" class="fui-dialog-title">{icon("info", "18")} How edges are drawn</h2>'
                + help_popover().replace('<div class="fui-popover fui-topology-help-pop" role="dialog" aria-labelledby="th-title"><p id="th-title" class="fui-popover-title">How the edges are drawn</p>', '<div>')
                + f'<div class="fui-dialog-actions">{button("secondary", "Close", keys="Esc")}</div></div>')
    return topo_page(overlay=help_pop)


def page_topology_collapsed():
    c = ("Ziekenhuis Voorbeeldstad",)
    return topo_page(graph_kw={"collapsed": c, "services_collapsed": True}, list_kw={"collapsed": c, "services_collapsed": True})


def page_topology_large():
    return topo_page("map", graph_kw={"large": True, "services_collapsed": True, "collapse_healthy": True}, list_kw={"large": True, "services_collapsed": True}, mini_view=(0, 0, 1460, 900))


def page_topology_list():
    return topo_page("list")


def page_topology_traffic():
    return topo_page(graph_kw={"traffic": True}, traffic=True)


def page_topology_phone_map():
    return topo_page("map", graph_kw={}, zoom=0.5, pan=(-170, 44), height_px=560, hint=True)


def page_topology_loading():
    from parts import skeleton_lines
    nodes = "".join(f'<rect class="fui-topology-skel" x="{x}" y="{y}" width="{w}" height="{H_NODE}" rx="8"/>' for x, w, y in [(X_GW, W_GW, 320), (X_ORG, W_ORG, 140), (X_ORG, W_ORG, 300), (X_ORG, W_ORG, 460), (X_NODE, W_NODE, 120), (X_NODE, W_NODE, 200), (X_NODE, W_NODE, 300), (X_NODE, W_NODE, 460), (X_SG, W_SG, 160), (X_SG, W_SG, 380), (X_SVC, W_SVC, 120), (X_SVC, W_SVC, 200), (X_SVC, W_SVC, 380)])
    svg = f'<svg class="fui-topology-svg" viewBox="-8 -8 1370 600" aria-hidden="true">{nodes}</svg>'
    sk_list = '<ul class="fui-topology-list" aria-hidden="true">' + "".join(f'<li class="fui-tree-row fui-tree-l{2 if i % 3 == 0 else 3}"><span class="fui-tree-name">{skeleton_lines([60])}</span><span>{skeleton_lines([40])}</span><span></span><span></span><span></span></li>' for i in range(8)) + "</ul>"
    body = ('<div class="fui-topology fui-topology-view-auto" aria-busy="true">' + toolbar(loading=True) + f'<div class="fui-topology-mapwrap"><div class="fui-topology-map"><div class="fui-topology-canvas">{svg}'
            '<p class="fui-topology-hint" role="status"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Reading the map…</span></p></div></div></div>'
            f'<div class="fui-topology-listwrap">{sk_list}</div></div>')
    return topo_page(body_override=body, incidents=None)


def page_topology_empty():
    from common import quiet_line
    return topo_page(graph_kw={"no_orgs": True}, list_kw={"show": ()}, empty_line=quiet_line("No organisation takes part yet, so the organisations side is empty.", "Create the first one in Organisations", "organisations-empty.html", boxed=False), incidents=0)


def page_topology_unreachable():
    body = ('<div class="fui-stack">' + refusal("The gateway refused the connection, so the map cannot be drawn.")
            + f'<div class="fui-cluster">{button("primary", "Try again", "refresh-cw")}<span class="fui-small fui-muted">Last tried 4 seconds ago. The last map is not shown: a stale map would claim states nobody knows.</span></div>'
            + tech_detail() + "</div>")
    return topo_page(body_override=body, status="down", incidents=None)
