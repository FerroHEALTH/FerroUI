// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The hold-to-show eye of a masked input (R10). The server draws it
// disabled, named to say the field stays masked; this enables it and names
// it, and while it is held (the primary pointer, touch, or Space or Enter)
// shows the typed text and says it is pressed; released, left or blurred,
// it masks the text again. The typed value never leaves its input.
export function holds(key) {
  return key === " " || key === "Enter";
}

export function enhance(button) {
  const input = document.getElementById(button.getAttribute("data-fui-input") || "");
  if (!input) return;
  const shown = button.querySelector("[data-fui-shown]");
  const masked = button.querySelector("[data-fui-masked]");
  function show(on) {
    input.type = on ? "text" : "password";
    button.setAttribute("aria-pressed", on ? "true" : "false");
    if (shown) shown.hidden = !on;
    if (masked) masked.hidden = on;
  }
  button.disabled = false;
  button.setAttribute("aria-label", button.getAttribute("data-fui-name") || "");
  button.addEventListener("pointerdown", (event) => {
    if (event.button === 0) show(true);
  });
  for (const leave of ["pointerup", "pointerleave", "pointercancel", "blur"]) button.addEventListener(leave, () => show(false));
  button.addEventListener("contextmenu", (event) => event.preventDefault());
  button.addEventListener("keydown", (event) => {
    if (!holds(event.key)) return;
    event.preventDefault();
    show(true);
  });
  button.addEventListener("keyup", (event) => {
    if (!holds(event.key)) return;
    event.preventDefault();
    show(false);
  });
}
