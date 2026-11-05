// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// A JSON or XML text highlighted again from the first line an edit
// changed (#531), into the lines linesHtml writes for the whole text.
import { Chars, STEPS, push, span } from "./highlight.js";

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

// The grammars an edit highlights again from its first changed line.
export const RESUMABLE = new Set(Object.keys(STEPS));

// The part of a line's runs before its unit `column`, copied.
function before(runs, column) {
  const out = [];
  let left = column;
  for (const { kind, text } of runs) {
    if (left <= 0) break;
    out.push({ kind, text: text.slice(0, left) });
    left -= text.length;
  }
  return out;
}

const runsHtml = (runs) => runs.map((run) => span(run.kind, run.text)).join("");

// For each line from `walk.mark` whose start the step that began at
// `from` in the state `was` read past: how far before its start the step
// began, and in which state.
function marked(walk, lines, seen, from, was) {
  while (walk.mark < lines.length && seen > walk.markStart) {
    walk.back.push(walk.markStart - from);
    walk.states.push(was);
    walk.markStart += lines[walk.mark].length + 1;
    walk.mark++;
  }
}

// Closes the current line and opens the next one; at a line `at.stop`
// accepts, the scan's answer, its runs before that line complete.
function broken(walk, lines, at) {
  walk.runs.push(walk.current);
  walk.current = [];
  walk.start += lines[walk.line].length + 1;
  walk.line++;
  const k = walk.line - at.mark;
  if (!at.stop || k < 0 || walk.line >= walk.mark || !at.stop(walk.line, walk.back[k], walk.states[k], walk.start)) return null;
  return { runs: walk.runs, back: walk.back.slice(0, k), states: walk.states.slice(0, k), end: walk.line };
}

// Puts the tokens `out` of one step on their lines, or answers where the
// scan stops.
function placed(walk, out, lines, at) {
  for (const token of out) {
    const parts = token.text.split("\n");
    for (let n = 0; n < parts.length; n++) {
      const stopped = n > 0 ? broken(walk, lines, at) : null;
      if (stopped) return stopped;
      push(walk.current, token.kind, parts[n]);
    }
  }
  return null;
}

// The steps of `text` from the unit `at.from` in `at.state`, split into
// `lines`: the runs of each line from `at.line` (its runs before `from`
// being `at.prefix`), and for each line from `at.mark` the step that first
// read past its start. It stops at a line `at.stop` accepts, the runs
// before it complete, or at the end of the text.
function scan(step, text, lines, at) {
  const chars = new Chars(text, at.from);
  const walk = { runs: [], back: [], states: [], current: at.prefix, line: at.line, start: at.start, mark: at.mark, markStart: at.markStart };
  let state = at.state;
  while (!chars.done()) {
    const from = chars.i;
    const was = state;
    const out = [];
    state = step(chars, state, out);
    marked(walk, lines, chars.seen, from, was);
    const stopped = placed(walk, out, lines, at);
    if (stopped) return stopped;
  }
  for (; walk.mark < lines.length; walk.mark++) {
    walk.back.push(walk.markStart - text.length);
    walk.states.push(state);
    walk.markStart += lines[walk.mark].length + 1;
  }
  walk.runs.push(walk.current);
  return { runs: walk.runs, back: walk.back, states: walk.states, end: lines.length };
}

// The highlighted lines of a JSON or XML text, kept across its edits, as
// linesHtml writes them. Each line keeps its runs of one kind and the
// step that first read past its start: how many units before the start
// it began and in which state. An edit resumes at that step of its first
// changed line, and stops at the first line after the change whose step
// began at the same place and in the same state as in the pass before,
// for from there on every step reads the same text from the same state.
export class Highlight {
  constructor(language, text) {
    this.language = language;
    this.step = STEPS[language][0];
    this.text = text;
    this.raw = text.split("\n");
    const all = scan(this.step, text, this.raw, { from: 0, state: STEPS[language][1], prefix: [], line: 0, start: 0, mark: 0, markStart: 0 });
    this.runs = all.runs;
    this.back = all.back;
    this.states = all.states;
    this.html = this.runs.map(runsHtml);
  }

  // The lines of `text`, an edit of the text before, highlighted.
  edit(text) {
    if (text === this.text) return this.html;
    const lines = text.split("\n");
    const old = this.raw;
    const p = patch(old, lines);
    const head = Math.min(p.head, old.length - 1, lines.length - 1);
    const inserted = lines.length - head - p.tail;
    const shift = old.length - lines.length;
    let start = 0;
    for (let n = 0; n < head; n++) start += lines[n].length + 1;
    const from = start - this.back[head];
    let line = head;
    let lineStart = start;
    while (lineStart > from) {
      line--;
      lineStart -= lines[line].length + 1;
    }
    let tailStart = start;
    for (let n = head; n < head + inserted; n++) tailStart += lines[n].length + 1;
    const stop = (n, back, state, at) => n >= head + inserted && at - back >= tailStart && this.back[n + shift] === back && this.states[n + shift] === state;
    const done = scan(this.step, text, lines, {
      from,
      state: this.states[head],
      prefix: before(this.runs[line], from - lineStart),
      line,
      start: lineStart,
      mark: head,
      markStart: start,
      stop,
    });
    const kept = done.end + shift;
    this.runs = this.runs.slice(0, line).concat(done.runs, this.runs.slice(kept));
    this.html = this.html.slice(0, line).concat(done.runs.map(runsHtml), this.html.slice(kept));
    this.back = this.back.slice(0, head).concat(done.back, this.back.slice(kept));
    this.states = this.states.slice(0, head).concat(done.states, this.states.slice(kept));
    this.text = text;
    this.raw = lines;
    return this.html;
  }
}
