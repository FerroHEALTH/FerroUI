// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The library's highlighters written again for the editor: YAML, JSON, XML,
// AQL and HL7 v2, each one linear pass over the characters of its input,
// the texts of its tokens joined being the input exactly. Each follows the
// Rust tokenizer of the same name in the crate's highlight module token for
// token; the gallery's parity tests hold the two to the same tokens and
// markup over a corpus. A text past MAX_BYTES (UTF-8) is one plain token.
export const MAX_BYTES = 512 * 1024;

const CLASS = {
  key: "fui-tok-key",
  string: "fui-tok-string",
  number: "fui-tok-number",
  keyword: "fui-tok-keyword",
  comment: "fui-tok-comment",
  tag: "fui-tok-tag",
  attribute: "fui-tok-attribute",
  path: "fui-tok-path",
  parameter: "fui-tok-parameter",
  segment: "fui-hl7-segid",
  separator: "fui-hl7-sep",
};

const space = (c) => /\p{White_Space}/u.test(c);
const alpha = (c) => /\p{Alphabetic}/u.test(c);
const alnum = (c) => /[\p{Alphabetic}\p{N}]/u.test(c);
const asciiAlpha = (c) => /^[A-Za-z]$/.test(c);
const digit = (c) => /^[0-9]$/.test(c);

export function push(out, kind, text) {
  if (!text) return;
  const last = out[out.length - 1];
  if (last && last.kind === kind) last.text += text;
  else out.push({ kind, text });
}

// A cursor over the code points of a text from a UTF-16 offset, keeping
// the end of the furthest unit it has read (`seen`): what a token read
// past its own end decides where an edit has to resume.
export class Chars {
  constructor(text, from = 0) {
    this.t = text;
    this.i = from;
    this.seen = from;
  }
  see(end) {
    if (end > this.seen) this.seen = end;
  }
  point(at) {
    const high = this.t.charCodeAt(at);
    if (high >= 0xd800 && high <= 0xdbff) {
      const low = this.t.charCodeAt(at + 1);
      if (low >= 0xdc00 && low <= 0xdfff) return this.t.slice(at, at + 2);
    }
    return this.t[at];
  }
  peek() {
    if (this.done()) return undefined;
    const c = this.point(this.i);
    this.see(this.i + c.length);
    return c;
  }
  next() {
    if (this.i >= this.t.length) return undefined;
    const c = this.point(this.i);
    this.i += c.length;
    this.see(this.i);
    return c;
  }
  done() {
    this.see(this.i + 1);
    return this.i >= this.t.length;
  }
  take(accept) {
    const from = this.i;
    while (!this.done() && accept(this.peek())) this.next();
    return this.t.slice(from, this.i);
  }
  quoted() {
    const from = this.i;
    const quote = this.next();
    let escaped = false;
    while (!this.done()) {
      const c = this.next();
      if (escaped) escaped = false;
      else if (c === "\\") escaped = true;
      else if (c === quote) break;
    }
    return this.t.slice(from, this.i);
  }
  // Through the first `end`, or to the end of the text.
  through(end) {
    const from = this.i;
    while (!this.done()) {
      this.next();
      if (this.i - from >= end.length && this.t.startsWith(end, this.i - end.length)) break;
    }
    return this.t.slice(from, this.i);
  }
  startsWith(prefix) {
    this.see(this.i + prefix.length);
    return this.t.startsWith(prefix, this.i);
  }
  // The first code point past a run of those `accept` takes, read and
  // left in place.
  past(accept) {
    let unit = this.i;
    while (unit < this.t.length) {
      const c = this.point(unit);
      if (!accept(c)) {
        this.see(unit + c.length);
        return c;
      }
      unit += c.length;
    }
    this.see(unit + 1);
    return undefined;
  }
}

function number(chars) {
  let out = "";
  if (chars.peek() === "-") out += chars.next();
  let previous = null;
  while (!chars.done()) {
    const c = chars.peek();
    if (digit(c) || c === "." || c === "e" || c === "E" || ((c === "+" || c === "-") && (previous === "e" || previous === "E"))) {
      out += chars.next();
      previous = c;
    } else break;
  }
  return out;
}

// JSON and XML run as steps, one token or tag part each, from a state
// that holds all a step needs of the text before it, so an edit can
// resume where a step began. JSON keeps no state between its tokens.
function jsonStep(chars, state, out) {
  const c = chars.peek();
  if (c === '"') {
    const value = chars.quoted();
    push(out, chars.past(space) === ":" ? "key" : "string", value);
  } else if (c === "-" || digit(c)) push(out, "number", number(chars));
  else if (asciiAlpha(c)) {
    const word = chars.take(asciiAlpha);
    push(out, ["true", "false", "null"].includes(word) ? "keyword" : "plain", word);
  } else if (space(c)) push(out, "plain", chars.take(space));
  else if ("{}[],:".includes(c)) push(out, "punctuation", chars.next());
  else push(out, "plain", chars.next());
  return state;
}

const xmlName = (c) => alnum(c) || ":_-.".includes(c);

// XML is either in text or inside a tag, past its name.
function xmlStep(chars, state, out) {
  if (state === "tag") {
    const t = chars.peek();
    if (t === ">") {
      push(out, "punctuation", chars.next());
      return "text";
    }
    if (space(t)) push(out, "plain", chars.take(space));
    else if (t === '"' || t === "'") push(out, "string", chars.quoted());
    else if (xmlName(t)) push(out, "attribute", chars.take(xmlName));
    else push(out, "punctuation", chars.next());
    return "tag";
  }
  if (chars.peek() !== "<") push(out, "plain", chars.take((x) => x !== "<"));
  else if (chars.startsWith("<!--")) push(out, "comment", chars.through("-->"));
  else if (chars.startsWith("<![CDATA[")) push(out, "comment", chars.through("]]>"));
  else if (chars.startsWith("<!")) push(out, "comment", chars.through(">"));
  else {
    push(out, "punctuation", chars.take((x) => x === "<" || x === "/" || x === "?"));
    push(out, "tag", chars.take(xmlName));
    return "tag";
  }
  return "text";
}

// The grammars that run as steps, each with its state at the start of a
// text.
export const STEPS = { json: [jsonStep, null], xml: [xmlStep, "text"] };

function stepped(language) {
  return (text) => {
    const [step, first] = STEPS[language];
    const out = [];
    const chars = new Chars(text);
    let state = first;
    while (!chars.done()) state = step(chars, state, out);
    return out;
  };
}

const AQL_KEYWORDS = "AND AS ASC ASCENDING BY CONTAINS DESC DESCENDING DISTINCT EXISTS FALSE FETCH FROM LIKE LIMIT MATCHES NOT NULL OFFSET OR ORDER SELECT TOP TRUE WHERE XOR".split(" ");

function aql(text) {
  const out = [];
  const chars = new Chars(text);
  const word = (c) => alnum(c) || c === "_";
  const predicate = () => {
    let value = chars.next();
    let depth = 1;
    while (!chars.done()) {
      const c = chars.peek();
      if (c === "'" || c === '"') value += chars.quoted();
      else {
        value += chars.next();
        if (c === "[") depth++;
        else if (c === "]" && --depth === 0) break;
      }
    }
    return value;
  };
  while (!chars.done()) {
    const c = chars.peek();
    if (c === "'" || c === '"') push(out, "string", chars.quoted());
    else if (c === "$") push(out, "parameter", chars.next() + chars.take(word));
    else if (c === "[") push(out, "path", predicate());
    else if (c === "/") {
      let value = "";
      while (chars.peek() === "/") {
        value += chars.next() + chars.take(word);
        while (chars.peek() === "[") value += predicate();
      }
      push(out, value === "/" ? "punctuation" : "path", value);
    } else if (c === "-") {
      const dash = chars.next();
      if (chars.peek() !== "-") push(out, "punctuation", dash);
      else push(out, "comment", dash + chars.take((x) => x !== "\n" && x !== "\r"));
    } else if (digit(c)) push(out, "number", number(chars));
    else if (alpha(c) || c === "_") {
      const w = chars.take(word);
      push(out, AQL_KEYWORDS.includes(w.toUpperCase()) && /^[\x00-\x7f]*$/.test(w) ? "keyword" : "identifier", w);
    } else if (space(c)) push(out, "plain", chars.take(space));
    else if ("(),;=<>!*.{}:+".includes(c)) push(out, "punctuation", chars.next());
    else push(out, "plain", chars.next());
  }
  return out;
}

function hl7v2(text) {
  const separators = ["|", "^", "~", "\\", "&"];
  const all = [...text];
  const isSeparatorChar = (c) => !alnum(c) && !space(c) && !/\p{Cc}/u.test(c);
  if (["MSH", "BHS", "FHS"].includes(all.slice(0, 3).join("")) && all[3] !== undefined && isSeparatorChar(all[3])) {
    separators[0] = all[3];
    for (let at = 4, slot = 1; slot < 5 && at < all.length && all[at] !== all[3] && isSeparatorChar(all[at]); at++, slot++) separators[slot] = all[at];
  }
  const out = [];
  const chars = new Chars(text);
  const broken = (c) => c === "\r" || c === "\n";
  let start = true;
  while (!chars.done()) {
    if (start) {
      start = false;
      const id = chars.take((c) => !broken(c) && c !== separators[0]);
      push(out, /^[A-Z][A-Z0-9]{2}$/.test(id) ? "segment" : "plain", id);
      continue;
    }
    const c = chars.next();
    if (broken(c)) {
      start = true;
      push(out, "plain", c);
    } else push(out, separators.includes(c) ? "separator" : "plain", c);
  }
  return out;
}

function yamlLine(line) {
  const out = [];
  let rest = line;
  const take = (kind, length) => {
    const head = rest.slice(0, length);
    if (head) out.push({ kind, text: head });
    rest = rest.slice(length);
  };
  take("plain", rest.length - rest.replace(/^[ \t]+/, "").length);
  while (rest.startsWith("-") && (rest.length === 1 || rest[1] === " " || rest[1] === "\t")) {
    take("punctuation", 1);
    take("plain", rest.length - rest.replace(/^[ \t]+/, "").length);
  }
  const quotedLength = (text, quote) => {
    for (let at = 1; at < text.length; at++) {
      const c = text[at];
      if (quote === '"' && c === "\\") {
        at++;
        continue;
      }
      if (c === quote) {
        if (quote === "'" && text[at + 1] === "'") {
          at++;
          continue;
        }
        return at + 1;
      }
    }
    return null;
  };
  const keyLength = (text) => {
    if (text.startsWith("#") || text === "") return null;
    let end = null;
    if (text[0] === '"' || text[0] === "'") end = quotedLength(text, text[0]);
    else {
      for (let at = 0; at < text.length; at++) {
        if (text[at] === ":" && (at + 1 === text.length || text[at + 1] === " " || text[at + 1] === "\t")) {
          end = at;
          break;
        }
        if (text[at] === "#" && text.slice(0, at).endsWith(" ")) return null;
      }
    }
    if (end === null || text[end] !== ":") return null;
    const next = text.slice(end + 1);
    return next === "" || next[0] === " " || next[0] === "\t" ? end : null;
  };
  const key = keyLength(rest);
  if (key !== null) {
    take("key", key);
    take("punctuation", 1);
  }
  const isNumber = (s) => {
    const d = s.replace(/^[-+]/, "");
    return d !== "" && /[0-9]/.test(d) && /^[0-9.eE+-]+$/.test(d) && /^[0-9.]/.test(d);
  };
  const classify = (s) => (["true", "false", "null", "~", "True", "False", "Null", "TRUE", "FALSE", "NULL"].includes(s) ? "keyword" : isNumber(s) ? "number" : "plain");
  while (rest !== "") {
    const gap = rest.length - rest.replace(/^[ \t]+/, "").length;
    if (gap > 0) {
      take("plain", gap);
      continue;
    }
    const first = rest[0];
    if (first === "#") take("comment", rest.length);
    else if (first === '"' || first === "'") take("string", quotedLength(rest, first) ?? rest.length);
    else if (first === "&" || first === "*" || first === "!") {
      const end = rest.search(/[ \t]/);
      take("keyword", end < 0 ? rest.length : end);
    } else {
      let length = rest.length;
      for (let at = 1; at < rest.length; at++) {
        if (rest[at] === "#" && (rest[at - 1] === " " || rest[at - 1] === "\t")) {
          length = rest.slice(0, at).replace(/[ \t]+$/, "").length;
          break;
        }
      }
      const scalar = rest.slice(0, length);
      const entry = scalar.endsWith(",") ? scalar.slice(0, -1) : scalar;
      take(classify(entry.replace(/\p{White_Space}+$/u, "")), entry.length);
      if (entry.length < length) take("punctuation", 1);
    }
  }
  return out;
}

function yaml(text) {
  const out = [];
  text.split("\n").forEach((line, at) => {
    if (at > 0) push(out, "plain", "\n");
    for (const token of yamlLine(line)) push(out, token.kind, token.text);
  });
  return out;
}

const TOKENIZERS = { yaml, json: stepped("json"), xml: stepped("xml"), aql, hl7v2 };

// Whether a text is within MAX_BYTES, encoded only when its length leaves
// the answer open: a UTF-16 unit is one to three bytes of UTF-8.
export function fits(text) {
  if (text.length > MAX_BYTES) return false;
  return text.length * 3 <= MAX_BYTES || new TextEncoder().encode(text).length <= MAX_BYTES;
}

export function tokens(language, text) {
  const tokenize = TOKENIZERS[language];
  if (!tokenize || !fits(text)) return text ? [{ kind: "plain", text }] : [];
  return tokenize(text);
}

export function escape(text) {
  return text.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

// A token's text as HTML: escaped, in a span of its kind's class.
export function span(kind, text) {
  const cls = CLASS[kind];
  return cls ? '<span class="' + cls + '">' + escape(text) + "</span>" : escape(text);
}

// The text as one string of HTML per line, as the crate's lines_html
// writes it: each classed token a span of its class, every character
// escaped; a final line break opens one more, empty, line.
export function linesHtml(language, text) {
  const out = [""];
  for (const token of tokens(language, text)) {
    token.text.split("\n").forEach((part, at) => {
      if (at > 0) out.push("");
      if (part) out[out.length - 1] += span(token.kind, part);
    });
  }
  return out;
}
