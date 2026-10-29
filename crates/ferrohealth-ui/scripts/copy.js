// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The copy button and the key hint: the server draws the button hidden and
// the text it copies on the page. This shows the button (and hides a key
// hint's plain text), and on a press writes the text with the asynchronous
// clipboard (W3C Clipboard API and events, writeText), shows "Copied" for
// two seconds, and says either outcome in the polite status beside it. A
// page over plain HTTP has no clipboard, which is a refusal.
export const COPIED_MS = 2000;

export function linesText(holder, finalBreak) {
  const lines = [...holder.querySelectorAll(".fui-code-line")].map((line) => line.textContent);
  return lines.join("\n") + (finalBreak ? "\n" : "");
}

function source(root) {
  if (root.hasAttribute("data-fui-copy-text")) return root.getAttribute("data-fui-copy-text");
  const element = root.getAttribute("data-fui-copy-element");
  if (element !== null) {
    const found = document.getElementById(element);
    return found ? found.textContent : null;
  }
  const lines = root.getAttribute("data-fui-copy-lines");
  if (lines !== null) {
    const holder = document.getElementById(lines);
    return holder ? linesText(holder, root.hasAttribute("data-fui-final-break")) : null;
  }
  return null;
}

export function enhance(root) {
  const button = root.tagName === "BUTTON" ? root : root.querySelector("button");
  if (!button) return;
  const status = root.tagName === "BUTTON" ? root.nextElementSibling : root.querySelector("[role='status']");
  const refused = root.querySelector ? root.querySelector(".fui-copy-refused") : null;
  if (root.tagName === "BUTTON") {
    const text = root.previousElementSibling;
    if (text && text.classList.contains("is-text")) text.hidden = true;
  }
  button.hidden = false;
  const ready = button.querySelector("[data-fui-ready]");
  const done = button.querySelector("[data-fui-done]");
  let timer = null;
  function say(ok) {
    if (status) status.textContent = status.getAttribute(ok ? "data-said-copied" : "data-said-refused") || "";
    if (refused) refused.hidden = ok;
    if (ok && ready && done) {
      ready.hidden = true;
      done.hidden = false;
      clearTimeout(timer);
      timer = setTimeout(() => {
        ready.hidden = false;
        done.hidden = true;
      }, COPIED_MS);
    }
  }
  button.addEventListener("click", () => {
    const text = source(root);
    if (text === null || !navigator.clipboard) return say(false);
    navigator.clipboard.writeText(text).then(() => say(true), () => say(false));
  });
}
