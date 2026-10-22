#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Checks the forms and the page shape of every page at 1440 and 390 px,
light:

- top-line: fields drawn side by side in one row whose labels (or, with a
  hidden label, whose controls) do not share a top line; a check group has
  no field label and is left out;
- clips: a text input whose placeholder, or whose value when the field is
  narrower than its row, is wider than the room inside the input, measured
  with the input's font; a value longer than a field that already spans
  its row scrolls, as every text input does;
- drawer: an open phone drawer drawn as a native popover that is not docked
  at the left and top edges, not full height, or keeps a margin or the UA
  border;
- primary: two visible primary actions with the same label outside a
  dialog, menu, popover or specimen card.

Usage: forms.py <set> [slug ...]   (run build.py first)"""
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chrome as browser  # noqa: E402

PROBE = r"""(() => {
const out = [];
const vis = (el) => { const r = el.getBoundingClientRect(); if (r.width < 2 || r.height < 2) return false;
  if (el.closest('.fui-sr-only, [hidden]')) return false; const cs = getComputedStyle(el); return cs.visibility !== 'hidden' && cs.display !== 'none'; };
const short = (s) => (s || '').replace(/\s+/g, ' ').trim().slice(0, 40);
// top-line
const CONTROL = 'input:not([type=checkbox]):not([type=radio]), select, textarea, .fui-segmented, .fui-input-group';
for (const box of document.querySelectorAll('.fui-form-grid, .fui-transform-options, .fui-repeater-row, .fui-cond, .fui-field-row')) {
  if (!vis(box)) continue;
  // NOTE: a check group (fui-field-checks) has no field label; its labels sit on the inputs' line by design.
  const fields = [...box.children].filter(c => c.matches('.fui-field') && !c.matches('.fui-field-checks') && vis(c));
  const rows = [];
  for (const f of fields) { const r = f.getBoundingClientRect(); const row = rows.find(g => g.some(o => { const q = o.getBoundingClientRect(); return Math.min(r.bottom, q.bottom) - Math.max(r.top, q.top) > 0.5 * Math.min(r.height, q.height); })); if (row) row.push(f); else rows.push([f]); }
  for (const row of rows) {
    if (row.length < 2) continue;
    const labels = row.map(f => [...f.children].find(c => c.matches('.fui-label, label, legend') && vis(c)));
    const useLabel = labels.every(Boolean);
    const tops = row.map((f, i) => { const el = useLabel ? labels[i] : f.querySelector(CONTROL); return el && vis(el) ? el.getBoundingClientRect().top : null; });
    if (tops.some(t => t === null)) continue;
    const d = Math.max(...tops) - Math.min(...tops);
    if (d > 2) out.push(['top-line', (box.className || '').toString().split(' ')[0], row.map(f => short((f.querySelector('.fui-label, label') || {}).textContent || '?')).join(' | '), Math.round(d) + 'px']);
  }
}
// clips
const cv = document.createElement('canvas').getContext('2d');
for (const el of document.querySelectorAll('input:not([type]), input[type=text], input[type=search], input[type=url], input[type=email], input[type=number], input[type=tel], input[type=password]')) {
  if (!vis(el)) continue;
  const cs = getComputedStyle(el);
  const text = el.value || el.placeholder || ''; if (!text) continue;
  cv.font = `${cs.fontStyle} ${cs.fontWeight} ${cs.fontSize} ${cs.fontFamily}`;
  const ls = parseFloat(cs.letterSpacing) || 0;
  const need = cv.measureText(el.type === 'password' && el.value ? '•'.repeat(el.value.length) : text).width + ls * text.length;
  const room = el.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
  // A value longer than a field that already spans its row scrolls inside it, as any text input does; only a
  // field narrower than its row, or a placeholder (design text), is a defect.
  const unit = el.closest('.fui-input-group, .fui-secret-field, form.fui-search') || el;
  let field = unit.closest('.fui-field, .fui-dock-field') || unit.parentElement;
  while (getComputedStyle(field).display === 'contents') field = field.parentElement;
  let row = field.parentElement; while (getComputedStyle(row).display === 'contents') row = row.parentElement;
  const rcs = getComputedStyle(row);
  const rowRoom = row.clientWidth - parseFloat(rcs.paddingLeft) - parseFloat(rcs.paddingRight);
  const fcs = getComputedStyle(field);
  const fieldRoom = field.clientWidth - parseFloat(fcs.paddingLeft) - parseFloat(fcs.paddingRight);
  const spans = unit.getBoundingClientRect().width >= fieldRoom - 2 && field.getBoundingClientRect().width >= rowRoom - 2;
  if (need > room + 1 && !(el.value && spans)) out.push(['clips', el.id || el.name || '?', el.value ? 'value' : 'placeholder', short(text), Math.round(need) + '>' + Math.round(room)]);
}
// drawer
const vw = document.documentElement.clientWidth, vh = document.documentElement.clientHeight;
for (const d of document.querySelectorAll('.fui-sidebar.mk-drawer-open, .fui-sidebar[open], .fui-drawer.mk-drawer-open')) {
  if (!vis(d) || vw >= 640) continue;
  const r = d.getBoundingClientRect(), cs = getComputedStyle(d);
  const why = [];
  if (r.left > 1) why.push('left ' + Math.round(r.left));
  if (r.top > 1) why.push('top ' + Math.round(r.top));
  if (r.height < vh - 2) why.push('height ' + Math.round(r.height) + ' of ' + vh);
  for (const k of ['marginTop', 'marginLeft', 'marginBottom']) if (Math.abs(parseFloat(cs[k]) || 0) > 0.5) why.push(k + ' ' + cs[k]);
  if ((parseFloat(cs.borderTopWidth) || 0) >= 1 && (parseFloat(cs.borderLeftWidth) || 0) >= 1) why.push('border on every side ' + cs.borderTopWidth);
  if (why.length) out.push(['drawer', (d.id || 'sidebar'), why.join(', ')]);
}
// primary
const seen = new Map();
for (const b of document.querySelectorAll('.fui-button-primary')) {
  if (!vis(b)) continue;
  if (b.closest('dialog, [role=dialog], [role=alertdialog], .fui-dialog, .fui-popover, .fui-menu, .fui-toast, [class*="-demo"], .mk-state-card, .mk-specimen, .fui-sheet')) continue;
  const c = b.cloneNode(true); c.querySelectorAll('kbd, .fui-kbd, .fui-keys, .fui-sr-only, svg').forEach(x => x.remove());
  const label = short(c.textContent); if (!label) continue;
  seen.set(label, (seen.get(label) || 0) + 1);
}
for (const [label, n] of seen) if (n > 1) out.push(['primary', label, n + ' visible']);
return out; })()"""


LOADS = tuple({"width": width, "theme": "light", "still": True} for width in (1440, 390))


def measure(chrome, session, slug, load):
    return [(slug, load["width"]) + tuple(hit) for hit in browser.evaluate(chrome, session, PROBE) or []]


def main():
    return browser.main_one("forms", __doc__)


if __name__ == "__main__":
    sys.exit(main())
