// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The keyboard shortcut legend (R1): a dialog the page opens on "?" pressed
// outside a field, and from a control marked data-fui-shortcuts such as the
// user menu's Keyboard shortcuts, never below 40 rem, where a phone has no
// keyboard (family rule 6). With no script the control is a link to the
// legend's page.
import { typing } from "./tabs.js";

export function enhance(dialog) {
  const phone = () => window.matchMedia("(width < 40rem)").matches;
  window.addEventListener("keydown", (event) => {
    if (event.key !== "?" || event.defaultPrevented || event.ctrlKey || event.metaKey || event.altKey) return;
    if (typing(event) || phone() || dialog.open) return;
    event.preventDefault();
    dialog.showModal();
  });
  document.addEventListener("click", (event) => {
    const marked = event.target.closest("[data-fui-shortcuts]");
    if (!marked || phone()) return;
    event.preventDefault();
    dialog.showModal();
  });
}
