// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The user menu (F13): a popover its button opens, which the browser closes
// on Escape and a press outside. This adds the menu button pattern of the
// WAI-ARIA Authoring Practices: the button says whether the menu is open, the
// focus moves to the first item as it opens and back to the button as it
// closes, Up and Down move between the items and wrap, Home and End go to
// the ends, Tab closes it, and choosing an item closes it.
import { moved } from "./menu.js";

const ITEMS = "[role='menuitem'], [role='menuitemradio']";

export function enhance(anchor) {
  const button = anchor.querySelector("button[popovertarget]");
  const menu = anchor.querySelector("[popover]");
  if (!button || !menu) return;
  const items = () => [...menu.querySelectorAll(ITEMS)].filter((item) => item.offsetParent !== null);
  const isOpen = () => menu.matches(":popover-open");
  const expanded = () => button.setAttribute("aria-expanded", isOpen() ? "true" : "false");
  expanded();
  const focusItem = (at) => {
    const all = items();
    if (all.length && all[at]) all[at].focus();
  };
  menu.addEventListener("toggle", () => {
    expanded();
    if (isOpen()) {
      focusItem(0);
    } else if (menu.contains(document.activeElement) || document.activeElement === document.body) {
      button.focus();
    }
  });
  button.addEventListener("keydown", (event) => {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    if (!isOpen()) menu.showPopover();
    focusItem(event.key === "ArrowUp" ? items().length - 1 : 0);
  });
  menu.addEventListener("keydown", (event) => {
    if (event.key === "Tab") {
      if (isOpen()) menu.hidePopover();
      return;
    }
    const all = items();
    const current = all.indexOf(document.activeElement);
    const next = moved(event.key, current < 0 ? null : current, all.length);
    if (next !== null) {
      event.preventDefault();
      focusItem(next);
    }
  });
  menu.addEventListener("click", (event) => {
    if (event.target.closest(ITEMS) && isOpen()) menu.hidePopover();
  });
}
