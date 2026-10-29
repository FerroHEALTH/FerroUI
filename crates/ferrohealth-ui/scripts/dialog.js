// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// A dialog, a popover form or a side panel: the browser opens it as a modal
// through the command attribute, keeps the page behind inert, closes it on
// Escape and returns the focus. This moves the focus where the dialog says
// as it opens: the control data-fui-focus selects, else its first field or
// control (WAI-ARIA Authoring Practices, the dialog (modal) pattern).
const CONTROLS = "[autofocus], input:not([type='hidden']), textarea, select, button, a[href]";

export function enhance(dialog) {
  dialog.addEventListener("toggle", () => {
    if (!dialog.open) return;
    const wanted = dialog.getAttribute("data-fui-focus");
    const first = (wanted && dialog.querySelector(wanted)) || dialog.querySelector(CONTROLS);
    if (first) first.focus();
  });
}
