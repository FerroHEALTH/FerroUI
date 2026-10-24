#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Finds text that overflows on every page at 1440 and 390 px: a box whose
content is wider than it without an intended ellipsis or scroller, anything
past the viewport's right edge, text on other text, text past an SVG card,
a word broken inside itself, and a short label forced over two lines.

Usage: overflow.py <set> [slug ...]   (run build.py first)"""
import os
import sys
import time

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chrome as browser  # noqa: E402

PROBE = r"""
(() => {
  window.scrollTo(0, 0);
  const vw = document.documentElement.clientWidth;
  // An open popup (menu, listbox, popover, date picker) may be wider than
  // its trigger by design; while it stays inside the viewport, the boxes
  // that hold it do not count as overflowing.
  const popupsInside = (el) => {
    const ps = el.querySelectorAll('.fui-menu, .fui-listbox, .fui-popover, .fui-datepicker, .fui-tooltip');
    if (!ps.length) return false;
    for (const p of ps) { const r = p.getBoundingClientRect(); if (r.width && (r.left < -1 || r.right > vw + 1)) return false; }
    return true;
  };
  const out = [];
  const name = (el) => {
    let s = el.tagName.toLowerCase();
    if (el.id) s += '#' + el.id;
    const c = (el.getAttribute('class') || '').split(/\s+/).filter(Boolean).slice(0, 2).join('.');
    if (c) s += '.' + c;
    return s;
  };
  const intended = (el) => {
    for (let e = el; e && e !== document.body; e = e.parentElement) {
      const cs = getComputedStyle(e);
      if (cs.textOverflow === 'ellipsis') return true;
      if (['auto', 'scroll'].includes(cs.overflowX)) return true;
      if (cs.clipPath && cs.clipPath !== 'none') return true;
      const rr = e.getBoundingClientRect();
      if (rr.width <= 1 || rr.height <= 1) return true;
      if (e.classList && (e.classList.contains('fui-sr-only') || e.classList.contains('fui-nav-group') || e.classList.contains('fui-nav-word'))) return true;
      if (e.tagName === 'CAPTION' || e.tagName === 'svg' || e.tagName === 'SELECT') return true;
      // A spinner measured mid-turn is not a layout fault.
      if (e.classList && (e.classList.contains('fui-spinner-ring') || e.classList.contains('fui-spinner'))) return true;
      if (e.querySelector && e.querySelector(':scope > .fui-spinner-ring')) return true;
      if (cs.animationName && cs.animationName !== 'none' && cs.animationPlayState !== 'paused' && /spin|rotate/i.test(cs.animationName)) return true;
    }
    return false;
  };
  for (const el of document.querySelectorAll('body *')) {
    const cs = getComputedStyle(el);
    if (cs.display === 'none' || cs.visibility === 'hidden') continue;
    const r = el.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) continue;
    if (intended(el)) continue;
    const text = (el.innerText || '').trim().slice(0, 50);
    // A box narrower than its content (clipped or spilling).
    if (el.scrollWidth > el.clientWidth + 2 && el.clientWidth > 0 && !['inline'].includes(cs.display) && !popupsInside(el)
        && !['INPUT', 'TEXTAREA', 'PRE', 'svg', 'path'].includes(el.tagName)) {
      out.push(['wider-than-box', name(el), Math.round(el.scrollWidth - el.clientWidth), text]);
    }
    // Past the viewport's right edge (not inside a positioned overlay that is meant to be there).
    if (r.right > vw + 1 && r.left < vw && !popupsInside(el)) {
      out.push(['past-viewport', name(el), Math.round(r.right - vw), text]);
    }
  }
  // Text that sits on other text: leaf elements with their own words whose
  // boxes intersect, outside dialogs, menus and popovers drawn over a page.
  const overlay = (el) => el.closest('.mk-open, .fui-menu, .fui-popover, .fui-side-panel, .fui-toasts, .fui-search-results, .fui-tooltip, .mk-scrim, .fui-sidebar.mk-drawer-open, .fui-dock, .fui-datepicker, .fui-listbox');
  const leaves = [];
  for (const el of document.querySelectorAll('body *')) {
    if (overlay(el)) continue;
    const own = [...el.childNodes].some(n => n.nodeType === 3 && n.textContent.trim());
    if (!own || !(el.innerText || '').trim()) continue;
    const cs = getComputedStyle(el);
    if (cs.visibility === 'hidden' || cs.display === 'none' || intended(el)) continue;
    // Only the element's own words: a descendant's text is measured as its
    // own leaf, inside whatever clips it (a key hint that scrolls).
    for (const node of el.childNodes) {
      if (node.nodeType !== 3 || !node.textContent.trim()) continue;
      const range = document.createRange();
      range.selectNodeContents(node);
      for (const r of range.getClientRects()) if (r.width > 2 && r.height > 2) leaves.push([el, r]);
    }
  }
  for (let i = 0; i < leaves.length; i++) for (let j = i + 1; j < leaves.length; j++) {
    const [a, ra] = leaves[i], [b, rb] = leaves[j];
    if (a === b || a.contains(b) || b.contains(a)) continue;
    const w = Math.min(ra.right, rb.right) - Math.max(ra.left, rb.left);
    const h = Math.min(ra.bottom, rb.bottom) - Math.max(ra.top, rb.top);
    if (w > 3 && h > 4) out.push(['text-overlap', name(a) + ' / ' + name(b), Math.round(w), (a.innerText || '').trim().slice(0, 24) + ' / ' + (b.innerText || '').trim().slice(0, 24)]);
  }
  // SVG text that runs past its card in a graph (the topology's nodes).
  for (const g of document.querySelectorAll('.fui-tnode')) {
    const box = g.querySelector('.fui-tnode-box');
    if (!box) continue;
    const bw = box.getBBox().width;
    if (!bw || !g.getBoundingClientRect().width) continue;
    for (const t of g.querySelectorAll('text')) {
      const x = parseFloat(t.getAttribute('x') || 0);
      const w = t.getComputedTextLength();
      if (x + w > bw - 6) out.push(['text-past-card', 'g.fui-tnode', Math.round(x + w - bw + 6), (t.textContent || '').slice(0, 40)]);
    }
  }
  // A word broken across lines, and a short phrase forced onto several
  // lines by a narrow column ("2,500 / ms").
  const wordsOf = (node) => {
    const res = [];
    const re = /\S+/g; let m;
    while ((m = re.exec(node.textContent))) res.push([m.index, m.index + m[0].length, m[0]]);
    return res;
  };
  for (const el of document.querySelectorAll('body *')) {
    if (overlay(el) || intended(el)) continue;
    const cs = getComputedStyle(el);
    if (cs.display === 'none' || cs.visibility === 'hidden') continue;
    // An archetype id or a path breaks at its offered breaks by design.
    if (el.closest('svg, pre, .fui-break, .fui-code-panel, textarea, .fui-editor, .fui-archetype, .fui-path')) continue;
    const texts = [...el.childNodes].filter(n => n.nodeType === 3 && n.textContent.trim());
    if (!texts.length) continue;
    let lines = new Set(), total = '';
    for (const tn of texts) {
      total += tn.textContent;
      for (const [a, b, w] of wordsOf(tn)) {
        const r = document.createRange(); r.setStart(tn, a); r.setEnd(tn, b);
        const rects = [...r.getClientRects()].filter(x => x.width > 1);
        if (rects.length > 1 && w.length > 3 && !w.includes('-') && Math.abs(rects[0].top - rects[rects.length - 1].top) > 4) out.push(['word-broken', name(el), rects.length, w.slice(0, 30)]);
        for (const x of rects) lines.add(Math.round(x.top / 4));
      }
    }
    const t = total.trim();
    if (t.length <= 22 && t.split(/\s+/).length <= 3 && lines.size > 1 && !el.closest('.fui-tooltip')) out.push(['forced-wrap', name(el), lines.size, t]);
  }
  // Keep the innermost offenders only.
  const seen = new Set();
  return out.filter(o => { const k = o[0] + o[1] + o[3]; if (seen.has(k)) return false; seen.add(k); return true; }).slice(0, 60);
})()
"""


LOADS = ({"width": 1440, "still": True}, {"width": 390, "still": True})


def measure(chrome, session, slug, load):
    time.sleep(0.05)
    return [(slug, load["width"], kind, el, text) for kind, el, _px, text in browser.evaluate(chrome, session, PROBE) or []]


def main():
    return browser.main_one("overflow", __doc__)


if __name__ == "__main__":
    sys.exit(main())
