// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The highlighted editor and the code editor around it (B2). The server
// draws the text area over its highlighted, numbered layer, so a form posts
// the text and the reader sees it highlighted with no script. Here each
// edit is highlighted again with the library's tokenizers written for the
// browser (highlight.js), and the layer redraws only the lines whose
// highlighting or mark changed. In a code editor the toolbar's line count,
// the cursor's place, the changed state and the current marked line follow
// the caret, and the diagnostics list beside it (ul[data-fui-for]) marks
// the row of the line the cursor is on; following a row's line link moves
// the caret there and gives the editor the focus.
import { linesHtml } from "./highlight.js";

const OPEN = "";
const SHUT = "";
const text = (word) => (word && typeof word === "object" ? word.en : word || "");

export function fill(phrase, values) {
  let out = text(phrase);
  for (const [name, value] of Object.entries(values)) out = out.split(OPEN + name + SHUT).join(String(value));
  return out;
}

export function grouped(number, group) {
  return String(number).replace(/\B(?=(\d{3})+(?!\d))/g, group);
}

// The caret's line and column (in characters) at an offset in UTF-16 code
// units, as the crate's caret_at reads it.
export function caretAt(value, offset) {
  let line = 1;
  let column = 1;
  let units = 0;
  for (const c of value) {
    if (units >= offset) break;
    units += c.length;
    if (c === "\n") {
      line++;
      column = 1;
    } else column++;
  }
  return { line, column };
}

// The offset of a line and column, as the crate's offset_of reads it.
export function offsetOf(value, at) {
  let line = 1;
  let column = 1;
  let units = 0;
  for (const c of value) {
    if (line > at.line || (line === at.line && column >= at.column)) break;
    if (c === "\n" && line === at.line) break;
    units += c.length;
    if (c === "\n") {
      line++;
      column = 1;
    } else column++;
  }
  return units;
}

// The grammar of a text, as the crate's Language::detect reads it.
export function detect(value) {
  const first = value.trimStart().charAt(0);
  return first === "{" || first === "[" ? "json" : first === "<" ? "xml" : "plain";
}

export function lineCount(value) {
  const body = value.endsWith("\n") ? value.slice(0, -1) : value;
  return body.split("\n").length;
}

function kinds(marks) {
  const out = new Map();
  for (const mark of marks) if (out.get(mark.line) !== "Error") out.set(mark.line, mark.kind);
  return out;
}

export function lineClass(kind, line, current) {
  if (!kind) return "";
  const cls = kind === "Error" ? "fui-mark-error" : "fui-mark-warning";
  return current === line ? cls + " is-current" : cls;
}

// How a layer goes from its old lines to its new ones: the first `head`
// stay, `removed` old lines go and `inserted` new lines take their place,
// and the last `tail` keep their elements.
export function patch(old, next) {
  let head = 0;
  while (head < old.length && head < next.length && old[head] === next[head]) head++;
  const room = Math.min(old.length, next.length) - head;
  let tail = 0;
  while (tail < room && old[old.length - 1 - tail] === next[next.length - 1 - tail]) tail++;
  return { head, removed: old.length - head - tail, inserted: next.length - head - tail, tail };
}

export function enhance(root) {
  const area = root.querySelector("textarea");
  const layer = root.querySelector(".fui-editor-layer");
  const stack = root.querySelector(".fui-editor-stack");
  if (!area || !layer) return;
  let language = root.getAttribute("data-language") || "plain";
  const follows = root.hasAttribute("data-fui-detect");
  const marks = JSON.parse(root.getAttribute("data-marks") || "[]");
  const marked = kinds(marks);
  const prefix = area.id + "-L";
  const panel = root.closest(".fui-mapping-editor");
  const words = panel ? JSON.parse(panel.getAttribute("data-fui-words") || "{}") : null;
  const opened = panel ? (panel.hasAttribute("data-fui-opened") ? panel.getAttribute("data-fui-opened") : area.defaultValue) : null;
  const isNew = panel ? panel.hasAttribute("data-fui-new") : false;
  const list = area.id ? document.querySelector("ul[data-fui-for='" + CSS.escape(area.id) + "']") : null;
  let bodies = linesHtml(language, area.value);
  let current = root.hasAttribute("data-current") ? Number(root.getAttribute("data-current")) : null;
  if (stack) stack.classList.remove("is-static");

  function element(line, body) {
    const span = document.createElement("span");
    const cls = lineClass(marked.get(line), line, current);
    span.className = "fui-code-line" + (cls ? " " + cls : "");
    span.id = prefix + line;
    span.innerHTML = body;
    return span;
  }
  function relabel(from) {
    const all = layer.children;
    for (let at = from; at < all.length; at++) {
      const line = at + 1;
      const cls = lineClass(marked.get(line), line, current);
      all[at].id = prefix + line;
      all[at].className = "fui-code-line" + (cls ? " " + cls : "");
    }
  }
  function redraw() {
    if (follows && detect(area.value) !== language) {
      language = detect(area.value);
      root.setAttribute("data-language", language);
      layer.textContent = "";
      bodies = [];
    }
    const next = linesHtml(language, area.value);
    const p = patch(bodies, next);
    const all = layer.children;
    for (let n = 0; n < p.removed; n++) if (all[p.head]) all[p.head].remove();
    const after = layer.children[p.head] || null;
    for (let n = 0; n < p.inserted; n++) layer.insertBefore(element(p.head + n + 1, next[p.head + n]), after);
    bodies = next;
    if (p.removed !== p.inserted) relabel(p.head + p.inserted);
  }
  function said(line) {
    const kind = marked.get(line);
    if (!kind || !words) return "";
    return fill(kind === "Error" ? words.error_on : words.warning_on, { line: grouped(line, words.grouping.group) });
  }
  function follow() {
    const at = caretAt(area.value, area.selectionStart || 0);
    const now = marked.has(at.line) ? at.line : null;
    if (now !== current) {
      const before = current;
      current = now;
      for (const line of [before, now]) {
        if (line === null) continue;
        const span = layer.children[line - 1];
        if (span) {
          const cls = lineClass(marked.get(line), line, current);
          span.className = "fui-code-line" + (cls ? " " + cls : "");
        }
      }
      if (list) {
        for (const row of list.querySelectorAll("li[data-line]")) {
          const on = current !== null && Number(row.getAttribute("data-line")) === current;
          row.classList.toggle("is-current", on);
          if (on) row.setAttribute("aria-current", "true");
          else row.removeAttribute("aria-current");
        }
      }
      if (panel) {
        const status = panel.querySelector(".fui-editor-status [role='status']");
        if (status) status.textContent = current === null ? "" : said(current);
      }
    }
    if (panel && words) {
      const position = panel.querySelector("[data-fui-position]");
      if (position) position.textContent = fill(words.position, { line: grouped(at.line, words.grouping.group), column: grouped(at.column, words.grouping.group) });
    }
  }
  function measure() {
    if (!panel || !words) return;
    const count = lineCount(area.value);
    const lines = count === 1 ? text(words.one_line) : fill(words.lines, { count: grouped(count, words.grouping.group) });
    const bar = panel.querySelector("[data-fui-lead]");
    if (bar) bar.textContent = bar.getAttribute("data-fui-lead") + " · " + lines;
    const dirty = !isNew && area.value !== opened;
    const state = panel.querySelector("[data-fui-state]");
    if (state) {
      state.classList.toggle("fui-editor-dirty", dirty);
      state.textContent = text(isNew ? words.new_file : dirty ? words.changed : words.unchanged);
    }
    panel.classList.toggle("is-empty", area.value.trim() === "");
  }
  area.addEventListener("input", () => {
    redraw();
    measure();
    follow();
  });
  for (const kind of ["keyup", "click", "select"]) area.addEventListener(kind, follow);
  if (list) {
    list.addEventListener("click", (event) => {
      const link = event.target.closest("a.fui-diag-where");
      const row = link && link.closest("li[data-line]");
      if (!row) return;
      const at = { line: Number(row.getAttribute("data-line")), column: Number(link.getAttribute("data-column") || "1") };
      const offset = offsetOf(area.value, at);
      setTimeout(() => {
        area.setSelectionRange(offset, offset);
        area.focus({ preventScroll: true });
        follow();
      }, 0);
    });
  }
}
