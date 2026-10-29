// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The picker and the pick-or-type field. A picker's select submits nothing;
// choosing a listed option writes its value into the text input beside it,
// and choosing the last option ("another", marked data-other) shows the
// input and moves the focus there, unless the reader is stepping through
// the closed select by arrow key (WCAG 2.2 SC 3.2.2). A pick-or-type field
// swaps its picker for its text input when the picker's data-other option
// is chosen. With no script both of a picker's controls show.
export function enhance(root) {
  const pick = root.querySelector(":scope > [data-side='pick']");
  const type = root.querySelector(":scope > [data-side='type']");
  if (pick && type) {
    pick.addEventListener("change", (event) => {
      const option = event.target.selectedOptions && event.target.selectedOptions[0];
      if (!option || !option.hasAttribute("data-other")) return;
      pick.hidden = true;
      type.hidden = false;
      const field = type.querySelector("input, textarea, select");
      if (field) field.focus();
    });
    return;
  }
  const select = root.querySelector(":scope > select");
  const box = root.querySelector("[data-fui-picker-input]");
  const input = box && box.querySelector("input");
  if (!select || !box || !input) return;
  let stepping = false;
  const other = () => select.selectedIndex === select.options.length - 1;
  const show = () => box.classList.toggle("fui-picker-hidden", !other());
  show();
  select.addEventListener("pointerdown", () => (stepping = false));
  select.addEventListener("keydown", (event) => {
    const explicit = event.key === "Enter" || event.key === " ";
    stepping = !explicit;
    if (explicit && other()) input.focus();
  });
  select.addEventListener("change", () => {
    if (other()) {
      show();
      if (!stepping) requestAnimationFrame(() => input.focus());
      return;
    }
    input.value = select.value;
    show();
  });
  input.addEventListener("input", () => {
    const at = [...select.options].findIndex((option, index) => index > 0 && !option.hasAttribute("data-other") && option.value === input.value);
    if (at > 0) select.selectedIndex = at;
    else if (input.value === "") select.selectedIndex = 0;
    else select.selectedIndex = select.options.length - 1;
  });
}
