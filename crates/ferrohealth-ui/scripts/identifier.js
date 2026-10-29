// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The masked identifier's reveal (R13). The server draws a revealed value
// with its seconds (data-seconds); this counts them down, the ring and the
// words following, says the time left in the polite live region at the
// announce point and at zero, and drops the value from the page when the
// seconds run out or Hide is pressed, handing the focus to Reveal when it
// was on the value's controls (WCAG 2.2 SC 2.4.3).
const OPEN = "";
const SHUT = "";
const text = (word) => (word && typeof word === "object" ? word.en : word || "");
const fill = (phrase, values) => {
  let out = text(phrase);
  for (const [name, value] of Object.entries(values)) out = out.split(OPEN + name + SHUT).join(String(value));
  return out;
};

export function ticked(seconds) {
  return seconds > 1 ? seconds - 1 : 0;
}

export function enhance(root) {
  const total = Number(root.getAttribute("data-seconds") || "0");
  if (!total) return;
  const words = JSON.parse(root.getAttribute("data-words") || "{}");
  const [masked, shown] = root.querySelectorAll(".fui-identifier-face");
  const ring = root.querySelector(".fui-identifier-countdown-left");
  const count = root.querySelector("[data-fui-count]");
  const live = root.querySelector(":scope > [role='status']");
  const value = root.querySelector(".fui-identifier-value");
  const hide = root.querySelector("[data-fui-hide]");
  const grouped = (n) => String(n).replace(/\B(?=(\d{3})+(?!\d))/g, words.group || ",");
  let left = total;
  function draw() {
    if (ring) ring.setAttribute("stroke-dashoffset", (34.56 * (1 - Math.min(left / total, 1))).toFixed(2));
    if (count) count.textContent = fill(words.countdown, { count: grouped(left) });
  }
  function drop() {
    clearInterval(timer);
    const held = root.contains(document.activeElement);
    if (value) value.textContent = "•".repeat(6);
    const copy = shown && shown.querySelector("[data-fui-copy-text]");
    if (copy) copy.removeAttribute("data-fui-copy-text");
    shown.classList.add("is-away");
    shown.inert = true;
    masked.classList.remove("is-away");
    masked.inert = false;
    const expired = masked.querySelector(".fui-identifier-expired");
    if (expired) expired.classList.remove("is-away");
    if (live) live.textContent = text(words.hidden_said);
    const reveal = masked.querySelector(".fui-identifier-reveal");
    if (held && reveal) reveal.focus();
  }
  const timer = setInterval(() => {
    left = ticked(left);
    if (left === 0) return drop();
    draw();
    if (left === words.announce_at && live) live.textContent = text(words.left_said);
  }, 1000);
  if (hide) hide.addEventListener("click", drop);
  draw();
}
