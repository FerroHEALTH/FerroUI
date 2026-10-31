// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The highlighted editor and the code editor around it (B2). The server
// draws the text area over its highlighted, numbered layer, so a form posts
// the text and the reader sees it highlighted with no script. Here each
// edit is highlighted again with the library's tokenizers written for the
// browser (highlight.js): line by line for a grammar whose tokens keep to
// their line, from the first changed line for JSON and XML (rehighlight.js),
// and the layer redraws only the lines whose highlighting or
// mark changed. In a code editor the toolbar's line count,
// the cursor's place, the changed state and the current marked line follow
// the caret, and the diagnostics list beside it (ul[data-fui-for]) marks
// the row of the line the cursor is on; following a row's line link moves
// the caret there and gives the editor the focus.
import { fits, linesHtml } from "./highlight.js";
import { Highlight, RESUMABLE, patch } from "./rehighlight.js";
import { fill, grouped, text, trim, trimStart } from "./shared.js";

// The grammars whose tokens never run past a line, which an edit
// highlights again line by line: a key costs the lines it touches, not the
// document.
const LINE_BY_LINE = new Set(["yaml", "plain"]);

// The caret's line and column (in characters) at an offset in UTF-16 code
// units, as the crate's caret_at reads it.
export function caretAt(value, offset) {
  let line = 1;
  let start = 0;
  for (let at = value.indexOf("\n"); at !== -1 && at < offset; at = value.indexOf("\n", start)) {
    line++;
    start = at + 1;
  }
  return { line, column: [...value.slice(start, offset)].length + 1 };
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
  const first = trimStart(value).charAt(0);
  return first === "{" || first === "[" ? "json" : first === "<" ? "xml" : "plain";
}

// Whether the editor holds no text but white space, its panel then
// drawn empty, as the crate's editor_panel draws it.
export const blank = (value) => trim(value) === "";

export function lineCount(value) {
  const body = value.endsWith("\n") ? value.slice(0, -1) : value;
  return body.split("\n").length;
}

export function kinds(marks) {
  const out = new Map();
  for (const mark of marks) if (out.get(mark.line) !== "Error") out.set(mark.line, mark.kind);
  return out;
}

export function lineClass(kind, line, current) {
  if (!kind) return "";
  const cls = kind === "Error" ? "fui-mark-error" : "fui-mark-warning";
  return current === line ? cls + " is-current" : cls;
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
  let model = null;
  const highlighted = (value) => {
    if (RESUMABLE.has(language) && fits(value)) {
      if (model && model.language === language) return model.edit(value);
      model = new Highlight(language, value);
      return model.html;
    }
    model = null;
    return linesHtml(language, value);
  };
  let bodies = highlighted(area.value);
  let raw = area.value.split("\n");
  let current = root.hasAttribute("data-current") ? Number(root.getAttribute("data-current")) : null;
  if (stack) stack.classList.remove("is-static");

  function named(line) {
    const cls = lineClass(marked.get(line), line, current);
    return "fui-code-line" + (cls ? " " + cls : "");
  }
  function element(line, body) {
    const span = document.createElement("span");
    span.className = named(line);
    span.id = prefix + line;
    span.innerHTML = body;
    return span;
  }
  function relabel(from) {
    const all = layer.children;
    for (let at = from; at < all.length; at++) {
      all[at].id = prefix + (at + 1);
      all[at].className = named(at + 1);
    }
  }
  function redraw() {
    if (follows && detect(area.value) !== language) {
      language = detect(area.value);
      root.setAttribute("data-language", language);
      layer.textContent = "";
      bodies = [];
      raw = [];
    }
    const lines = area.value.split("\n");
    let next;
    if (LINE_BY_LINE.has(language) && raw.length === bodies.length && fits(area.value)) {
      const changed = patch(raw, lines);
      next = bodies
        .slice(0, changed.head)
        .concat(lines.slice(changed.head, changed.head + changed.inserted).map((line) => linesHtml(language, line)[0]))
        .concat(bodies.slice(bodies.length - changed.tail));
    } else next = highlighted(area.value);
    raw = lines;
    const p = patch(bodies, next);
    const all = layer.children;
    // A line kept in place takes its new text: no numbered line comes or
    // goes, so the browser counts no line number again.
    const kept = Math.min(p.removed, p.inserted);
    for (let n = 0; n < kept; n++) if (all[p.head + n]) all[p.head + n].innerHTML = next[p.head + n];
    for (let n = kept; n < p.removed; n++) if (all[p.head + kept]) all[p.head + kept].remove();
    const after = layer.children[p.head + kept] || null;
    for (let n = kept; n < p.inserted; n++) layer.insertBefore(element(p.head + n + 1, next[p.head + n]), after);
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
        if (span) span.className = named(line);
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
    panel.classList.toggle("is-empty", blank(area.value));
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
