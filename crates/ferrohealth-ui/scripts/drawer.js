// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The navigation drawer (R12): a modal <dialog> the top bar's button opens.
// The browser makes the page inert and closes it on Escape; this moves the
// focus to the first link as it opens, closes it when a link is followed,
// and hides a rail entry's tooltip on Escape until the pointer or the focus
// moves again (WCAG 2.2 SC 1.4.13).
export function enhance(dialog) {
  dialog.addEventListener("toggle", () => {
    if (dialog.open) {
      const first = dialog.querySelector("a");
      if (first) first.focus();
    }
  });
  dialog.addEventListener("click", (event) => {
    if (event.target.closest("a") && dialog.open) dialog.close();
  });
  dialog.addEventListener("keydown", (event) => {
    if (event.key === "Escape") dialog.setAttribute("data-tips", "off");
  });
  const show = () => dialog.removeAttribute("data-tips");
  dialog.addEventListener("pointerover", show);
  dialog.addEventListener("focusin", show);
}
