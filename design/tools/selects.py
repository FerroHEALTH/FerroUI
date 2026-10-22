#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Checks every select on every page at 1440 and 390 px, the native select
and R29's button with its listbox alike:

- it has a name a screen reader announces: a label that names it with
  `for`, `aria-label` or `aria-labelledby` (WCAG 2.2 SC 4.1.2), and a
  native select carries an id and a name;
- the chosen value's words fit inside the control, never clipped;
- the control is at least 24 by 24 CSS px (SC 2.5.8);
- an R29 button's `aria-expanded` matches its listbox: an open one shows a
  listbox with exactly one option marked `aria-selected="true"`, inside the
  viewport, and a closed one shows none.

Usage: selects.py <set> [slug ...]   (run build.py first)"""
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chrome as browser  # noqa: E402

PROBE = r"""
(() => {
  const out = [];
  const visible = (el) => {
    if (!el) return false;
    const cs = getComputedStyle(el), r = el.getBoundingClientRect();
    return cs.display !== 'none' && cs.visibility !== 'hidden' && r.width > 0 && r.height > 0;
  };
  const name = (el) => el.tagName.toLowerCase() + (el.id ? '#' + el.id : '');
  const labelled = (el) => (el.id && document.querySelector('label[for="' + CSS.escape(el.id) + '"]'))
    || el.getAttribute('aria-label') || el.getAttribute('aria-labelledby') || el.closest('label');
  const canvas = document.createElement('canvas').getContext('2d');
  for (const el of document.querySelectorAll('select, button[aria-haspopup="listbox"]')) {
    if (!visible(el)) continue;
    const native = el.tagName === 'SELECT';
    if (native && (!el.id || !el.name)) out.push([name(el), 'no-id-or-name', '']);
    if (native && !labelled(el)) out.push([name(el), 'no-label', '']);
    if (!native && !(el.innerText || '').trim() && !el.getAttribute('aria-label')) out.push([name(el), 'no-name', '']);
    const r = el.getBoundingClientRect();
    if (r.width < 24 || r.height < 24) out.push([name(el), 'target-under-24px', Math.round(r.width) + 'x' + Math.round(r.height)]);
    const cs = getComputedStyle(el);
    if (native) {
      const opt = el.options[el.selectedIndex];
      if (opt) {
        canvas.font = cs.font;
        const need = canvas.measureText(opt.text).width;
        const room = el.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
        if (need > room + 1) out.push([name(el), 'value-clipped', opt.text.slice(0, 40)]);
      }
    } else {
      const words = el.querySelector('span') || el;
      if (words.scrollWidth > words.clientWidth + 1) out.push([name(el), 'value-clipped', (words.innerText || '').trim().slice(0, 40)]);
      const box = el.parentElement.querySelector('[role="listbox"]');
      const open = el.getAttribute('aria-expanded') === 'true';
      if (open !== visible(box)) out.push([name(el), 'expanded-mismatch', open ? 'open with no listbox' : 'closed with a listbox']);
      if (open && visible(box)) {
        const chosen = box.querySelectorAll('[role="option"][aria-selected="true"]').length;
        if (chosen !== 1) out.push([name(el), 'selected-options', String(chosen)]);
        const b = box.getBoundingClientRect();
        if (b.right > document.documentElement.clientWidth + 1 || b.left < -1) out.push([name(el), 'listbox-past-viewport', '']);
      }
    }
  }
  return out;
})()
"""


LOADS = ({"width": 1440, "still": True}, {"width": 390, "still": True})


def measure(chrome, session, slug, load):
    return [(slug, load["width"], kind, el, detail) for el, kind, detail in browser.evaluate(chrome, session, PROBE) or []]


def main():
    return browser.main_one("selects", __doc__)


if __name__ == "__main__":
    sys.exit(main())
