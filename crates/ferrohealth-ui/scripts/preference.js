// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The form of one preference (R28): a plain post with a Save button. Once
// this runs, a change posts at once and the button leaves; the row just
// saved (data-fui-saved) gives the focus back to its control, so a keyboard
// carries on where it was.
export function enhance(form) {
  const save = form.querySelector("button[type='submit']");
  if (save) save.hidden = true;
  form.addEventListener("change", () => form.requestSubmit());
  if (form.hasAttribute("data-fui-saved")) {
    const control = form.querySelector("input:checked, select, input[type='checkbox']");
    if (control) control.focus();
  }
}
