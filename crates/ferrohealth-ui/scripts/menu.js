// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The menu and the listbox (F13, R29): a <details> whose summary is the
// trigger. The browser opens and closes it; this adds the WAI-ARIA Authoring
// Practices menu button and listbox patterns: the focus moves to the first
// item (or the chosen option) as it opens, the arrows, Home, End and a letter
// walk the items, Escape closes it and returns the focus, a focus or a press
// outside closes it, choosing an item closes it, the popup flips where it has
// no room, the sheet's Cancel shows, and a copy item copies its text.
const ITEMS = "[role='menuitem'], [role='menuitemradio'], [role='option']";

export function moved(key, current, count) {
  if (count === 0) return null;
  const last = count - 1;
  if (key === "ArrowDown") return current === null || current >= last ? 0 : current + 1;
  if (key === "ArrowUp") return current === null || current <= 0 ? last : current - 1;
  if (key === "Home") return 0;
  if (key === "End") return last;
  return null;
}

export function typed(letter, current, words) {
  letter = letter.toLowerCase();
  if ([...letter].length !== 1 || !letter.trim()) return null;
  const start = current === null ? 0 : current + 1;
  for (let step = 0; step < words.length; step++) {
    const at = (start + step) % words.length;
    if (words[at].trimStart().toLowerCase().startsWith(letter)) return at;
  }
  return null;
}

export function flips(anchor, popup, viewport) {
  const up = anchor.bottom + popup.height > viewport.height && anchor.top - popup.height >= 0;
  const side = popup.left < 0 || popup.right > viewport.width;
  return { up, side };
}

function focus(element) {
  if (element) element.focus();
}

export function enhance(details) {
  const summary = details.querySelector("summary");
  const list = details.querySelector("[role='menu'], [role='listbox']");
  const cancel = details.querySelector(".fui-menu-cancel");
  if (!summary || !list) return;
  const listbox = list.getAttribute("role") === "listbox";
  const items = () => [...list.querySelectorAll(ITEMS)];
  for (const item of items()) {
    item.tabIndex = -1;
    if (item.hasAttribute("data-fui-copy")) item.hidden = false;
  }
  if (cancel) cancel.hidden = false;
  const focused = () => items().indexOf(document.activeElement);
  const expanded = () => summary.setAttribute("aria-expanded", details.open ? "true" : "false");
  expanded();

  function place() {
    const popup = list.parentElement;
    const [up, side] = listbox ? ["is-up", "is-end"] : ["fui-menu-up", "fui-menu-start"];
    popup.classList.remove(up, side);
    if (window.matchMedia("(width < 40rem)").matches) return;
    const f = flips(details.getBoundingClientRect(), popup.getBoundingClientRect(), {
      width: window.innerWidth,
      height: window.innerHeight,
    });
    if (f.up) popup.classList.add(up);
    if (f.side) popup.classList.add(side);
  }

  function close() {
    details.open = false;
    focus(summary);
  }

  details.addEventListener("toggle", () => {
    expanded();
    if (!details.open) return;
    place();
    const all = items();
    if (all.includes(document.activeElement)) return;
    const chosen = all.findIndex((item) => item.getAttribute("aria-selected") === "true");
    focus(all[chosen >= 0 ? chosen : 0]);
  });
  details.addEventListener("keydown", (event) => {
    const key = event.key;
    const onTrigger = event.target === summary;
    if (key === "Escape" && details.open) {
      event.preventDefault();
      close();
      return;
    }
    if (onTrigger) {
      if (!details.open && (key === "ArrowDown" || key === "ArrowUp")) {
        event.preventDefault();
        details.open = true;
        const all = items();
        focus(key === "ArrowUp" ? all[all.length - 1] : all[0]);
      }
      return;
    }
    if (key === "Tab") {
      details.open = false;
      return;
    }
    const all = items();
    const current = focused();
    const at = current < 0 ? null : current;
    if (key === " " && at !== null && all[at].tagName === "A") {
      event.preventDefault();
      all[at].click();
      return;
    }
    let next = moved(key, at, all.length);
    if (next === null) next = typed(key, at, all.map((item) => item.innerText));
    if (next !== null) {
      event.preventDefault();
      focus(all[next]);
    }
  });
  details.addEventListener("focusout", (event) => {
    if (details.open && !details.contains(event.relatedTarget)) details.open = false;
  });
  details.addEventListener("click", (event) => {
    const item = event.target.closest(ITEMS);
    if (!item) return;
    if (item.getAttribute("aria-disabled") === "true") {
      event.preventDefault();
      return;
    }
    const text = item.getAttribute("data-fui-copy");
    if (text !== null) copy(text, details);
    close();
  });
  if (cancel) cancel.addEventListener("click", close);
}

function copy(text, details) {
  const words = {
    copied: details.getAttribute("data-copied") || "Copied",
    refused: details.getAttribute("data-copy-refused") || "The browser refused to copy",
  };
  const say = (ok) =>
    document.dispatchEvent(
      new CustomEvent("fui:toast", { detail: { tone: ok ? "ok" : "bad", title: ok ? words.copied : words.refused } }),
    );
  if (!navigator.clipboard) return say(false);
  navigator.clipboard.writeText(text).then(() => say(true), () => say(false));
}
