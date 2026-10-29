// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The picker and the pick-or-type field. A picker's select submits nothing;
// choosing a listed option writes its value into the text input beside it,
// and choosing the last option ("another", marked data-other) shows the
// input and moves the focus there, unless the reader is stepping through
// the closed select by arrow key (WCAG 2.2 SC 3.2.2). A pick-or-type field
// swaps its picker for its text input when the picker's data-other option
// is chosen (a select's or a combobox's), and back on its control marked
// data-fui-pick-back, which puts the picker on the listed option chosen
// before, or on its first, so choosing data-other again shows the input.
// With no script both of a picker's controls show.
export function enhance(root) {
  const pick = root.querySelector(":scope > [data-side='pick']");
  const type = root.querySelector(":scope > [data-side='type']");
  if (pick && type) {
    const back = root.querySelector("[data-fui-pick-back]");
    const side = (typing) => {
      pick.hidden = typing;
      type.hidden = !typing;
      if (back) back.hidden = !typing;
      const field = (typing ? type : pick).querySelector("input:not([type='hidden']), textarea, select");
      if (field) field.focus();
    };
    const other = (control) => {
      const option = control.selectedOptions ? control.selectedOptions[0] : control;
      return Boolean(option && option.hasAttribute("data-other"));
    };
    // The listed option chosen last: a select's place, a combobox's value.
    let before = { place: 0, value: "" };
    const drawn = pick.querySelector("select");
    if (drawn && !other(drawn)) before = { place: drawn.selectedIndex, value: drawn.value };
    pick.addEventListener("change", (event) => {
      if (other(event.target)) side(true);
      else before = { place: event.target.selectedIndex ?? 0, value: event.target.value };
    });
    const listed = () => {
      const select = pick.querySelector("select");
      if (select) {
        const option = select.options[before.place];
        select.selectedIndex = option && !option.hasAttribute("data-other") ? before.place : 0;
        return;
      }
      const combobox = pick.querySelector("[data-fui='combobox']");
      if (combobox) combobox.dispatchEvent(new CustomEvent("fui:choose", { detail: { value: before.value } }));
    };
    if (back)
      back.addEventListener("click", () => {
        listed();
        side(false);
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
  // A form's reset puts the select and the input back; the input follows.
  if (select.form) select.form.addEventListener("reset", () => setTimeout(show));
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
