// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The keys a control announces in aria-keyshortcuts (#526, #530): a chord
// (Control or Command with a key) presses its control from a field too, a
// single key presses it, or focuses a field, outside a field alone, never
// below 40 rem (family rule 6). A work-list row's control answers only while
// the row holds the focus, which its dialog returns to. Within a modal only
// its controls answer, and a chord leaves the focus in its field. A go-to
// pair of the legend (data-fui-go, a first key then a letter) follows its
// link. The legend's ? and the search's key are theirs.
import { typing } from "./tabs.js";

const ACTS = "a[href], button, input, select, textarea, summary";
const OWN = "[data-fui-shortcuts], form[data-fui-shortcut] *";
const phone = () => window.matchMedia("(width < 40rem)").matches;

export function parse(value) {
  return (value || "").trim().split(/\s+/).filter(Boolean).map((token) => {
    const at = token.lastIndexOf("+", token.length - 2);
    const held = at < 0 ? [] : token.slice(0, at).split("+");
    return { key: token.slice(at + 1), held: new Set(held) };
  });
}

export function matches(shortcut, event) {
  const key = event.key === " " ? "Space" : event.key;
  const letter = /^\p{L}$/u.test(shortcut.key);
  if (letter ? key.toUpperCase() !== shortcut.key.toUpperCase() : key !== shortcut.key) return false;
  const control = shortcut.held.has("Control") || shortcut.held.has("Meta");
  if (control !== (event.ctrlKey || event.metaKey)) return false;
  // A sign such as [ takes Shift or Alt on some layouts, so only a letter
  // or a sign announced with them holds them to it.
  const alt = shortcut.held.has("Alt");
  const shift = shortcut.held.has("Shift");
  if (!letter && !alt && !shift) return true;
  return alt === event.altKey && shift === event.shiftKey;
}

function field(element) {
  const tag = element.tagName.toLowerCase();
  return tag === "textarea" || tag === "select" || (tag === "input" && typing({ target: element }));
}

function reach(event) {
  const active = document.activeElement;
  const scope = document.querySelector("dialog:modal") || document;
  const found = [];
  for (const element of scope.querySelectorAll("[aria-keyshortcuts]")) {
    if (!element.matches(ACTS) || element.matches(OWN) || element.disabled) continue;
    if (!element.checkVisibility() || !parse(element.getAttribute("aria-keyshortcuts")).some((s) => matches(s, event))) continue;
    const row = element.closest("[data-fui~='worklist'] tr[data-key]");
    if (row && !row.contains(active)) continue;
    const near = row ? 0 : element.form && active && element.form === active.form ? 1 : 2;
    found.push({ element, row, near });
  }
  found.sort((a, b) => a.near - b.near);
  return found[0];
}

function lead(key) {
  return document.querySelector("[data-fui-go='" + CSS.escape(key) + "']");
}

let pending = null;
let bound = false;

function pressed(event) {
  if (event.defaultPrevented || event.isComposing || event.key === "?" || phone()) return;
  const chord = event.ctrlKey || event.metaKey;
  const free = !chord && !typing(event);
  const group = pending;
  pending = null;
  if (group && free) {
    const link = group.querySelector("a[data-fui-go-key='" + CSS.escape(event.key.toLowerCase()) + "']");
    if (link) {
      event.preventDefault();
      link.click();
      return;
    }
  }
  if (!chord && !free) return;
  const hit = reach(event);
  if (hit) {
    event.preventDefault();
    if (event.repeat) return;
    if (field(hit.element)) hit.element.focus();
    else {
      // A dialog returns the focus to what held it as it opened.
      if (!hit.row && hit.element.matches("[commandfor], [aria-haspopup]")) hit.element.focus();
      hit.element.click();
    }
    return;
  }
  if (free && !document.querySelector("dialog:modal")) {
    pending = lead(event.key.toLowerCase());
    if (pending) event.preventDefault();
  }
}

export function enhance() {
  if (bound) return;
  bound = true;
  window.addEventListener("keydown", pressed);
}
