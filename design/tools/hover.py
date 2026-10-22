#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Hover stability: hovering or focusing a control never moves anything.
For every page at 1440 and 390 px, measures every element's box at rest,
then hovers and focuses each interactive element in turn and measures
again; any box outside the hovered element's own subtree that moved is a
finding. Paint-only effects (tints, opacity, outlines) pass.

Usage: hover.py <set> [slug ...]   (run build.py first)"""
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chrome as browser  # noqa: E402

SNAP = r"""
(() => {
  const els = [...document.querySelectorAll('body *')];
  window.__els = els;
  window.__base = els.map(e => { const r = e.getBoundingClientRect(); return [r.x, r.y, r.width, r.height]; });
  const ctl = [...document.querySelectorAll('a[href], button, summary, input, select, textarea, [tabindex]:not([tabindex="-1"]), .fui-tnode-link')]
    .filter(e => { const r = e.getBoundingClientRect(); const cs = getComputedStyle(e); return r.width > 0 && r.height > 0 && cs.visibility !== 'hidden' && !e.disabled; });
  window.__ctl = ctl.slice(0, 80);
  return window.__ctl.map(e => { const r = e.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; });
})()
"""

DIFF = r"""
((i, mode) => {
  const c = window.__ctl[i];
  const out = [];
  window.__els.forEach((e, k) => {
    if (c === e || c.contains(e) || e.contains(c)) return;
    if (e.closest('.fui-tooltip, .fui-menu, .fui-popover, [role=tooltip], .fui-skip, .fui-spinner-ring, .fui-topology-traffic, .fui-path-pulse')) return;
    if (e.getAnimations && e.getAnimations().length) return;
    const r = e.getBoundingClientRect(); const b = window.__base[k];
    if (b[2] * b[3] === 0 || r.width * r.height === 0) return;
    const d = Math.max(Math.abs(r.x - b[0]), Math.abs(r.y - b[1]), Math.abs(r.width - b[2]), Math.abs(r.height - b[3]));
    if (d > 0.5) out.push([e.tagName.toLowerCase() + '.' + (e.getAttribute('class') || '').split(' ')[0], Math.round(d)]);
  });
  const name = c.tagName.toLowerCase() + '.' + (c.getAttribute('class') || '').split(' ')[0] + ' ' + (c.innerText || c.getAttribute('aria-label') || '').trim().slice(0, 30);
  return out.length ? [mode, name, out.length, out.slice(0, 3)] : null;
})
"""


LOADS = ({"width": 1440, "still": True}, {"width": 390, "still": True})
# Hovering leaves the pointer on the page, so no other check reads this load.
POINTER = True


def measure(chrome, session, slug, load):
    found = []
    width = load["width"]
    h = browser.evaluate(chrome, session, "document.documentElement.scrollHeight")
    chrome.send("Emulation.setDeviceMetricsOverride", {"width": width, "height": int(h), "deviceScaleFactor": 1, "mobile": width < 600}, session)
    browser.settle(chrome, session)
    pts = browser.evaluate(chrome, session, SNAP)
    for i, (x, y) in enumerate(pts):
        chrome.send("Input.dispatchMouseEvent", {"type": "mouseMoved", "x": x, "y": y}, session)
        r = browser.evaluate(chrome, session, f"{DIFF}({i}, 'hover')")
        if r:
            found.append((slug, width, r[0], r[1], str(r[3])))
        chrome.send("Runtime.evaluate", {"expression": f"window.__ctl[{i}].focus({{preventScroll: true}})"}, session)
        r = browser.evaluate(chrome, session, f"{DIFF}({i}, 'focus')")
        if r:
            found.append((slug, width, r[0], r[1], str(r[3])))
    chrome.send("Input.dispatchMouseEvent", {"type": "mouseMoved", "x": 0, "y": 0}, session)
    return found


def main():
    return browser.main_one("hover", __doc__)


if __name__ == "__main__":
    sys.exit(main())
