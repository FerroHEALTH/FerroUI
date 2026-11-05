// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// What more than one module uses: the words a console sent, read as the
// crate's i18n writes them (a word's English when it is an object, a
// phrase with its named slots filled, a number with its digit groups), and
// a new element with its attributes, listeners and children.
const OPEN = "";
const SHUT = "";

// White space as the crate's str::trim reads it (the Unicode White_Space
// property), so a rule the crate runs too trims the same characters.
const SPACE_START = /^\p{White_Space}+/u;
const SPACE = /\p{White_Space}/u;
export const trimStart = (value) => value.replace(SPACE_START, "");
// One character at a time from the end: a trailing `+$` pattern backtracks
// over a long run of white space.
export function trimEnd(value) {
  let end = value.length;
  while (end > 0 && SPACE.test(value[end - 1])) end--;
  return value.slice(0, end);
}
export const trim = (value) => trimEnd(trimStart(value));

export const text = (word) => (word && typeof word === "object" ? word.en : word || "");

export function fill(phrase, values) {
  let out = text(phrase);
  for (const [name, value] of Object.entries(values)) out = out.split(OPEN + name + SHUT).join(String(value));
  return out;
}

export function grouped(number, group) {
  return String(number).replace(/\d+/g, (run) => {
    let out = run.slice(-3);
    for (let end = run.length - 3; end > 0; end -= 3) out = run.slice(Math.max(0, end - 3), end) + group + out;
    return out;
  });
}

export function element(tag, props, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props || {})) {
    if (value === null || value === undefined || value === false) continue;
    if (key === "class") node.className = value;
    else if (key === "text") node.textContent = value;
    else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
    else node.setAttribute(key, value === true ? "" : value);
  }
  for (const child of children.flat()) if (child) node.append(child);
  return node;
}
