// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The toast region (R17). The server draws the flash toasts; this times them
// out after a dwell, counted while neither the pointer nor the focus is on
// the region, keeps a toast with an action until it is dismissed, shows at
// most three (the oldest that is no refusal and has no action leaving
// first), and raises a toast any module asks for with a "fui:toast" event
// carrying { tone, title, line }. The region's <template> is the toast's
// shape, its words the server's.
export const DWELL_MS = 6000;
export const SHOWN_MAX = 3;

export function trim(toasts) {
  const leaving = [];
  while (toasts.length - leaving.length > SHOWN_MAX) {
    const oldest = toasts.find(
      (toast) => !leaving.includes(toast) && !toast.classList.contains("fui-tone-bad") && !toast.querySelector(".fui-toast-action"),
    );
    if (!oldest) break;
    leaving.push(oldest);
  }
  return leaving;
}

export function enhance(region) {
  let epoch = 0;
  let hovered = false;
  let focused = false;
  const paused = () => hovered || focused;
  const toasts = () => [...region.querySelectorAll(".fui-toast")];

  function settle(toast) {
    const details = toast.querySelector("details.fui-toast-close");
    const button = toast.querySelector("button.fui-toast-dismiss");
    if (details) details.hidden = true;
    if (button) {
      button.hidden = false;
      button.addEventListener("click", () => toast.remove());
    }
    if (!toast.querySelector(".fui-toast-action")) later(toast);
  }

  function later(toast) {
    const mine = epoch;
    setTimeout(() => {
      if (paused() || epoch !== mine) return;
      toast.remove();
    }, DWELL_MS);
  }

  function held(on, which) {
    const before = paused();
    if (which === "pointer") hovered = on;
    else focused = on;
    if (before === paused()) return;
    epoch++;
    if (!paused()) for (const toast of toasts()) if (!toast.querySelector(".fui-toast-action")) later(toast);
  }

  region.addEventListener("pointerenter", () => held(true, "pointer"));
  region.addEventListener("pointerleave", () => held(false, "pointer"));
  region.addEventListener("focusin", () => held(true, "focus"));
  region.addEventListener("focusout", () => held(false, "focus"));
  for (const toast of toasts()) settle(toast);

  document.addEventListener("fui:toast", (event) => {
    const template = region.querySelector("template");
    if (!template) return;
    const { tone = "info", title = "", line = "" } = event.detail || {};
    const toast = template.content.firstElementChild.cloneNode(true);
    toast.classList.add("fui-tone-" + tone);
    if (tone === "bad") toast.setAttribute("role", "alert");
    const glyph = template.content.querySelector("[data-tone='" + tone + "']") || template.content.querySelector("[data-tone]");
    const slot = toast.querySelector(".fui-toast-glyph");
    if (glyph && slot) slot.replaceChildren(glyph.cloneNode(true).firstElementChild);
    toast.querySelector("strong").textContent = title;
    const lineSlot = toast.querySelector(".fui-toast-line");
    if (lineSlot) {
      if (line) lineSlot.textContent = line;
      else lineSlot.remove();
    }
    region.appendChild(toast);
    for (const leaving of trim(toasts())) leaving.remove();
    settle(toast);
  });
}
