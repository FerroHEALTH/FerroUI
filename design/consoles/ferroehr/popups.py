#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""popups.py <console src> [slug...]: every open popup (listbox, menu, popover,
search results, date picker) at 1440 light, 1440 dark and 390 light. Flags:
clipped (an option's text wider than its box or past the popup's edge),
off-screen, not solid (transparent ground, no border or no shadow), and
indistinct (its ground within 1.08:1 of what it opens over and a border that
does not stand out by 1.3:1)."""
import os, sys, glob, importlib.util
HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("render", os.path.join(HERE, "render.py"))
sys.path.insert(0, HERE)
render = importlib.util.module_from_spec(spec); spec.loader.exec_module(render)
src = sys.argv[1]; only = set(sys.argv[2:])
JS = r"""(() => {
const out = [];
const rgb = (s) => { const m = s.match(/[\d.]+/g); if (!m) return [0,0,0,0]; return [+m[0], +m[1], +m[2], m.length > 3 ? +m[3] : 1]; };
const lum = (c) => { const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126*f(c[0]) + 0.7152*f(c[1]) + 0.0722*f(c[2]); };
const ratio = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x,y) + 0.05) / (Math.min(x,y) + 0.05); };
const groundUnder = (pop) => { const r = pop.getBoundingClientRect(); const pts = [[r.left + 4, r.top + r.height/2], [r.right - 4, r.top + r.height/2]];
  for (const [x, y] of pts) { for (const el of document.elementsFromPoint(x, y)) { if (pop.contains(el) || el.closest('.mk-scrim')) continue; const c = rgb(getComputedStyle(el).backgroundColor); if (c[3] >= 0.99) return c; } }
  return rgb(getComputedStyle(document.body).backgroundColor); };
const vw = document.documentElement.clientWidth;
for (const pop of document.querySelectorAll('.fui-listbox, .fui-menu, .fui-popover, .mk-results, .fui-datepicker')) {
  const r = pop.getBoundingClientRect(); if (r.width < 2 || r.height < 2) continue;
  const cs = getComputedStyle(pop); if (cs.visibility === 'hidden' || cs.display === 'none') continue;
  const demo = !!pop.closest('.mk-picker-demo, .mk-picker-frame, .mk-sheet-demo');
  const name = (pop.className || '').toString().split(' ')[0];
  if (r.left < -1 || r.right > vw + 1) out.push([name, 'off-screen', Math.round(r.left), Math.round(r.right)]);
  for (const el of pop.querySelectorAll('*')) {
    if (el.closest('.fui-sr-only')) continue;
    if (!el.childNodes.length || ![...el.childNodes].some(n => n.nodeType === 3 && n.textContent.trim())) continue;
    const er = el.getBoundingClientRect(); if (er.width < 1) continue;
    const ecs = getComputedStyle(el); if (ecs.textOverflow === 'ellipsis') continue;
    if (el.scrollWidth > el.clientWidth + 1 && ecs.overflowX !== 'visible' && ecs.overflowX !== 'auto') out.push([name, 'clipped', el.textContent.trim().slice(0, 40)]);
    if (er.right > r.right + 1 || er.left < r.left - 1) out.push([name, 'past-edge', el.textContent.trim().slice(0, 40), Math.round(er.right - r.right)]);
  }
  const bg = rgb(cs.backgroundColor), bw = parseFloat(cs.borderTopWidth) || 0;
  if (pop.matches('.mk-results') ) { /* scaffolding results panel */ }
  if (bg[3] < 0.99 || bw < 1 || cs.boxShadow === 'none') { if (!demo && !pop.matches('.mk-sheet-static, .fui-datepicker-inline')) out.push([name, 'not-solid', bg[3], bw, cs.boxShadow === 'none' ? 'no shadow' : 'shadow']); }
  const under = groundUnder(pop); const bc = rgb(cs.borderTopColor);
  if (ratio(bg, under) < 1.08 && ratio(bc, under) < 1.3) out.push([name, 'indistinct', ratio(bg, under).toFixed(2), ratio(bc, under).toFixed(2)]);
}
return out; })()"""
ch = render.Chrome(os.path.join(render.STATE_DIR, "chrome-pop"))
n = 0
try:
    t = ch.send("Target.createTarget", {"url": "about:blank"})["targetId"]
    s = ch.send("Target.attachToTarget", {"targetId": t, "flatten": True})["sessionId"]
    ch.send("Page.enable", {}, s); ch.send("Runtime.enable", {}, s)
    for f in sorted(glob.glob(os.path.join(src, "*.html"))):
        slug = os.path.basename(f)[:-5]
        if only and slug not in only: continue
        for w, theme in ((1440, "light"), (1440, "dark"), (390, "light")):
            ch.send("Emulation.setDeviceMetricsOverride", {"width": w, "height": 900, "deviceScaleFactor": 1, "mobile": w < 600}, s)
            ch.send("Emulation.setEmulatedMedia", {"features": [{"name": "prefers-color-scheme", "value": theme}]}, s)
            ch.send("Page.navigate", {"url": "file://" + os.path.abspath(f)}, s); ch.wait_event("Page.loadEventFired", s)
            ch.send("Runtime.evaluate", {"expression": f"document.documentElement.setAttribute('data-theme', document.documentElement.dataset.lockTheme || '{theme}')"}, s)
            for x in ch.send("Runtime.evaluate", {"expression": JS, "returnByValue": True}, s)["result"].get("value") or []:
                n += 1; print(f"{slug:34} {w} {theme:5} {x}", flush=True)
    print(n, "findings")
finally:
    ch.close()
