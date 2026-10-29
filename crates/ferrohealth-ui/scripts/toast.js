// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The toast region (R17). The server draws the flash toasts; this times them
// out after a dwell, counted while neither the pointer nor the focus is on
// the region, keeps a toast with an action until it is dismissed, shows at
// most three (the oldest that is no refusal and has no action leaving
// first), and raises a toast any module asks for with a "fui:toast" event
// carrying { tone, title, line, action }, and one a link marked
// data-fui-toast asks for in place of loading its page. The region's
// <template> is the toast's shape, its words the server's.
export const DWELL_MS = 6000;
export const SHOWN_MAX = 3;

export function trim(toasts) {
  const leaving = [];
  while (toasts.length - leaving.length > SHOWN_MAX) {
    const oldest = toasts.find(
      (toast) => !leaving.includes(toast) && !toast.classList.contains("fui-tone-bad") && !toast.querySelector("[data-fui-action]"),
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
    const button = toast.querySelector("button[data-fui-dismiss]");
    if (details) details.hidden = true;
    if (button) {
      button.hidden = false;
      button.addEventListener("click", () => toast.remove());
    }
    if (!toast.querySelector("[data-fui-action]")) later(toast);
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
    if (!paused()) for (const toast of toasts()) if (!toast.querySelector("[data-fui-action]")) later(toast);
  }

  region.addEventListener("pointerenter", () => held(true, "pointer"));
  region.addEventListener("pointerleave", () => held(false, "pointer"));
  region.addEventListener("focusin", () => held(true, "focus"));
  region.addEventListener("focusout", () => held(false, "focus"));
  for (const toast of toasts()) settle(toast);

  document.addEventListener("fui:toast", (event) => {
    const template = region.querySelector("template");
    if (!template) return;
    const { tone = "info", title = "", line = "", action = null } = event.detail || {};
    const toast = template.content.firstElementChild.cloneNode(true);
    toast.classList.add("fui-tone-" + tone);
    if (tone === "bad") toast.setAttribute("role", "alert");
    const glyph = template.content.querySelector("[data-tone='" + tone + "']") || template.content.querySelector("[data-tone]");
    const slot = toast.querySelector(":scope > .fui-icon");
    if (glyph && slot) slot.replaceWith(glyph.firstElementChild.cloneNode(true));
    toast.querySelector("strong").textContent = title;
    const lineSlot = toast.querySelector(".fui-toast-text > span");
    if (lineSlot) {
      if (line) lineSlot.textContent = line;
      else lineSlot.remove();
    }
    const link = toast.querySelector("a[data-fui-action]");
    if (link && action && action.href) {
      link.href = action.href;
      link.textContent = action.label || "";
      link.hidden = false;
    } else if (link) link.remove();
    region.appendChild(toast);
    for (const leaving of trim(toasts())) leaving.remove();
    settle(toast);
  });

  // A link to a page that answers with a toast raises that toast in place:
  // data-fui-toast names its tone, and its title, line and action ride in
  // data-fui-toast-title, -line, -action and -href.
  document.addEventListener("click", (event) => {
    const control = event.target.closest("[data-fui-toast]");
    if (!control) return;
    event.preventDefault();
    // A confirm inside a dialog has done the dialog's work.
    const dialog = control.closest("dialog");
    if (dialog && dialog.open) dialog.close();
    const words = control.dataset;
    document.dispatchEvent(
      new CustomEvent("fui:toast", {
        detail: {
          tone: words.fuiToast,
          title: words.fuiToastTitle,
          line: words.fuiToastLine,
          action: words.fuiToastHref ? { label: words.fuiToastAction, href: words.fuiToastHref } : null,
        },
      }),
    );
  });
}
