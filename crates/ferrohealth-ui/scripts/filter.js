// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The in-place filter: hides each row of the element data-fui-rows names
// whose text does not hold the words typed, ignoring case, and writes the
// count of the rows kept at once and, after the typing pauses, into the
// status region a screen reader hears (WCAG 2.2 SC 4.1.3).
import { fill, trim } from "./shared.js";
export const SETTLE_MS = 600;

export function keeps(words, texts) {
  const wanted = trim(words).toLowerCase();
  return wanted === "" || texts.some((text) => text.toLowerCase().includes(wanted));
}

export function enhance(box) {
  const input = box.querySelector("input[type='search']");
  const shown = box.querySelector("p[aria-hidden='true']");
  const status = box.querySelector("p[role='status']");
  const holder = document.getElementById(box.getAttribute("data-fui-rows") || "");
  const phrase = box.getAttribute("data-fui-phrase") || "";
  if (!input || !holder) return;
  const rows = () => {
    const marked = [...holder.querySelectorAll("[data-fui-filter-text]")];
    return marked.length ? marked : [...holder.children];
  };
  let timer = null;
  function apply() {
    const all = rows();
    let kept = 0;
    for (const row of all) {
      const text = row.getAttribute("data-fui-filter-text") || row.textContent || "";
      const keep = keeps(input.value, [text]);
      row.hidden = !keep;
      if (keep) kept++;
    }
    const sentence = fill(phrase, { kept, total: all.length });
    if (shown) shown.textContent = sentence;
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (status) status.textContent = sentence;
    }, SETTLE_MS);
  }
  input.addEventListener("input", apply);
  if (input.value) apply();
}
