#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""WCAG 2.2 AA contrast of every visible text run on every page, in light and
dark: the text colour against the first opaque background behind it (alpha
grounds composited). 4.5:1 for body text, 3:1 for large text (24 px, or
18.66 px bold). Disabled controls and placeholders are exempt (SC 1.4.3).

Usage: contrast.py <set> [slug ...]   (run build.py first)"""
import os
import sys
import time

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chrome as browser  # noqa: E402

PROBE = r"""
(() => {
  const parse = (c) => { const m = c.match(/rgba?\(([^)]+)\)/); if (!m) return null; const p = m[1].split(/[ ,\/]+/).filter(Boolean).map(Number); return [p[0], p[1], p[2], p.length > 3 ? p[3] : 1]; };
  const lum = ([r, g, b]) => { const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b); };
  const over = (top, bot) => { const a = top[3]; return [top[0] * a + bot[0] * (1 - a), top[1] * a + bot[1] * (1 - a), top[2] * a + bot[2] * (1 - a), 1]; };
  const bgOf = (el) => {
    const layers = [];
    for (let e = el; e; e = e.parentElement) {
      const c = parse(getComputedStyle(e).backgroundColor);
      if (c && c[3] > 0) { layers.push(c); if (c[3] >= 1) break; }
    }
    let base = [255, 255, 255, 1];
    if (!layers.length || layers[layers.length - 1][3] < 1) base = parse(getComputedStyle(document.body).backgroundColor) || base;
    for (let i = layers.length - 1; i >= 0; i--) base = over(layers[i], base);
    return base;
  };
  const out = [];
  for (const el of document.querySelectorAll('body *')) {
    const own = [...el.childNodes].some(n => n.nodeType === 3 && n.textContent.trim());
    if (!own) continue;
    const cs = getComputedStyle(el);
    if (cs.visibility === 'hidden' || cs.display === 'none' || parseFloat(cs.opacity) === 0) continue;
    const r = el.getBoundingClientRect();
    if (r.width < 1 || r.height < 1) continue;
    if (el.closest('[disabled], [aria-disabled="true"], .fui-sr-only, .is-dim, svg, .mk-scrim, [aria-hidden="true"], .fui-editor-input')) continue;
    let op = 1; for (let e = el; e; e = e.parentElement) op *= parseFloat(getComputedStyle(e).opacity);
    if (op < 0.99) continue;
    const fg = parse(cs.color); if (!fg) continue;
    const bg = bgOf(el);
    const f = over(fg, bg);
    const l1 = lum(f), l2 = lum(bg);
    const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
    const size = parseFloat(cs.fontSize), bold = parseInt(cs.fontWeight) >= 700;
    const need = (size >= 24 || (bold && size >= 18.66)) ? 3 : 4.5;
    if (ratio < need) out.push([el.tagName.toLowerCase() + '.' + (el.getAttribute('class') || '').split(' ')[0], ratio.toFixed(2), need, el.innerText.trim().slice(0, 30)]);
  }
  const seen = new Set();
  return out.filter(o => { const k = o[0] + o[3]; if (seen.has(k)) return false; seen.add(k); return true; }).slice(0, 20);
})()
"""


LOADS = tuple({"width": 1440, "theme": theme, "reduced_motion": True, "still": True} for theme in ("light", "dark"))


def measure(chrome, session, slug, load):
    time.sleep(0.3)
    return [(slug, load["theme"], tag, f"{ratio} < {need}", text) for tag, ratio, need, text in browser.evaluate(chrome, session, PROBE) or []]


def main():
    return browser.main_one("contrast", __doc__)


if __name__ == "__main__":
    sys.exit(main())
