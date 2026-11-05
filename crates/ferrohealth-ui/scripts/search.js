// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The top bar's search: its shortcut, its busy state and, where the console
// names an answer route, its typeahead (WAI-ARIA Authoring Practices 1.2,
// the combobox with a listbox popup). The rules are the crate's
// is_shortcut, wants, targets, step, escaped and placed; a hit is drawn as
// the crate's hit_body draws it.
import { typing } from "./tabs.js";
import { fill, text, trim } from "./shared.js";

export const SETTLE = 200;
export const PER_GROUP = 5;

export function wants(words, from) {
  const held = [...trim(words)].length;
  return held > 0 && held >= from;
}

export function escaped(open, empty) {
  if (open) return "close";
  return empty ? "pass" : "clear";
}

export function step(active, count, down) {
  if (count === 0) return null;
  const last = count - 1;
  if (!down) return active === null || active === 0 ? last : Math.min(active, count) - 1;
  if (active === null || active >= last) return 0;
  return active + 1;
}

export function placed(left, right, wide, view) {
  if (left + wide <= view - 16) return "start";
  if (right - wide >= 16) return "end";
  return "narrow";
}

// The options of an answer: at most PER_GROUP hits of each kind, then
// every result, then the way past.
export function targets(answer) {
  const out = [];
  for (const group of answer.groups || []) for (const hit of group.hits.slice(0, PER_GROUP)) out.push({ hit: hit.href });
  if ((answer.groups || []).some((group) => group.hits.length)) out.push({ all: true });
  if (answer.more) out.push({ way: answer.more.href });
  return out;
}

// Where a match of the lower-cased `needle` that begins at the character
// `at` of `chars` ends, or -1 when none begins there.
function matchEnd(chars, at, needle) {
  let matched = 0;
  let next = at;
  while (matched < needle.length) {
    if (next >= chars.length) return -1;
    for (const lower of chars[next].toLowerCase()) {
      if (needle[matched] !== lower) return -1;
      matched++;
    }
    next++;
  }
  return next;
}

// The runs of `value`, each marked when it matches `query` without regard
// to case, as the crate's runs.
export function runs(value, query) {
  const needle = [...trim(query)].flatMap((c) => [...c.toLowerCase()]);
  if (!needle.length) return [[value, false]];
  const chars = [...value];
  const out = [];
  let plain = "";
  let at = 0;
  while (at < chars.length) {
    const next = matchEnd(chars, at, needle);
    if (next < 0) {
      plain += chars[at++];
      continue;
    }
    if (plain) out.push([plain, false]);
    plain = "";
    out.push([chars.slice(at, next).join(""), true]);
    at = next;
  }
  if (plain) out.push([plain, false]);
  return out;
}

// What the search's route answers for `query` from the form `form`: its
// answer, its status when it refused, or why it could not be read.
export async function ask(route, form, name, query) {
  const body = new URLSearchParams([[name, query]]);
  const post = form.method.toLowerCase() === "post";
  try {
    const joint = route.includes("?") ? "&" : "?";
    const response = await fetch(post ? route : route + joint + body, {
      method: post ? "POST" : "GET",
      credentials: "same-origin",
      headers: post ? { Accept: "application/json", "Content-Type": "application/x-www-form-urlencoded" } : { Accept: "application/json" },
      body: post ? body.toString() : undefined,
    });
    if (!response.ok) return { status: response.status };
    return await answered(response);
  } catch (error) {
    console.warn("fui: the browser refused the search's request", error);
    return { failure: "unreachable" };
  }
}

async function answered(response) {
  try {
    return { answer: await response.json() };
  } catch (error) {
    console.warn("fui: the search's answer is unreadable", error);
    return { failure: "unreadable" };
  }
}

function el(tag, cls, ...children) {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  node.append(...children.filter((child) => child !== null && child !== undefined));
  return node;
}

function emphasised(value, query) {
  return runs(value, query).map(([run, marked]) => (marked ? el("mark", "fui-search-match", run) : document.createTextNode(run)));
}

// What one hit shows, as the crate's hit_body draws it, its glyph from
// `icon`.
export function hitBody(hit, query, icon) {
  let mark = null;
  if (hit.mark?.Dot) {
    mark = el("span", "fui-dot fui-tone-" + String(hit.mark.Dot).toLowerCase());
    mark.setAttribute("aria-hidden", "true");
  } else if (hit.mark?.Glyph) mark = icon(hit.mark.Glyph);
  return [mark, el("span", "fui-hit-name", ...emphasised(hit.name, query)), el("span", "fui-small fui-muted fui-hit-line", ...emphasised(hit.line || "", query)), hit.tag ? el("span", "fui-tag", hit.tag) : null];
}

export function enhance(form) {
  const field = form.querySelector("input[type='search']");
  const submit = form.querySelector("button[type='submit']");
  const glyph = form.querySelector(".fui-search-glyph");
  const icons = form.querySelector("template[data-fui-icons]");
  const words = JSON.parse(form.dataset.fuiWords || "{}");
  const shortcut = form.dataset.fuiShortcut;
  const route = form.dataset.fuiSuggest;
  const from = Number(form.dataset.fuiFrom || "2");
  const signedOut = form.dataset.fuiSignedOut;
  const panel = form.querySelector(".fui-search-results");
  const list = panel?.querySelector("[role='listbox']");
  const said = form.querySelector(":scope > [role='status']");
  const id = field.id;
  const icon = (name) => {
    const found = icons?.content.querySelector("[data-icon='" + name + "'] svg");
    return found ? found.cloneNode(true) : null;
  };
  let answer = null;
  let options = [];
  let active = null;
  let pending = null;
  let asked = 0;

  function setBusy(on) {
    if (on) form.setAttribute("aria-busy", "true");
    else form.removeAttribute("aria-busy");
    submit.disabled = on || field.disabled;
    glyph.replaceChildren(on ? el("span", "fui-spinner-ring") : icon("search") || "");
    if (on) glyph.firstChild.setAttribute("aria-hidden", "true");
  }
  form.addEventListener("submit", () => {
    reset();
    setBusy(true);
  });
  window.addEventListener("pageshow", () => setBusy(false));
  if (shortcut && !field.disabled) {
    window.addEventListener("keydown", (event) => {
      if (event.defaultPrevented || event.key !== shortcut || event.ctrlKey || event.metaKey || event.altKey || typing(event)) return;
      event.preventDefault();
      field.focus();
      field.setSelectionRange(field.value.length, field.value.length);
    });
  }
  if (!route || !panel || field.disabled) return;

  field.setAttribute("role", "combobox");
  field.setAttribute("aria-autocomplete", "list");
  field.setAttribute("aria-expanded", "false");
  field.setAttribute("aria-controls", list.id);

  function open(on) {
    panel.hidden = !on;
    field.setAttribute("aria-expanded", String(on));
    if (!on) activate(null);
    else if (!window.matchMedia("(width < 48rem)").matches) {
      panel.classList.remove("is-end", "is-narrow");
      requestAnimationFrame(() => {
        const box = form.getBoundingClientRect();
        const side = placed(box.left, box.right, panel.getBoundingClientRect().width, window.innerWidth);
        if (side !== "start") panel.classList.add("is-" + side);
      });
    }
  }
  function activate(at) {
    active = at;
    options.forEach((option, place) => option.setAttribute("aria-selected", String(place === at)));
    const on = at === null ? null : options[at];
    if (on) {
      field.setAttribute("aria-activedescendant", on.id);
      on.scrollIntoView({ block: "nearest" });
    } else field.removeAttribute("aria-activedescendant");
  }
  function reset() {
    clearTimeout(pending);
    asked++;
    answer = null;
    options = [];
    open(false);
  }
  function option(optionId, cls, target, ...body) {
    const node = el("div", cls, ...body);
    node.id = optionId;
    node.setAttribute("role", "option");
    node.setAttribute("aria-selected", "false");
    node.addEventListener("mousemove", () => {
      if (active !== options.indexOf(node)) activate(options.indexOf(node));
    });
    node.addEventListener("click", () => pick(target));
    options.push(node);
    return node;
  }
  function status(cls, glyphName, words) {
    const row = el("p", "fui-search-results-status" + cls, glyphName ? icon(glyphName) : el("span", "fui-spinner-ring"), words);
    if (!glyphName) row.firstChild.setAttribute("aria-hidden", "true");
    return row;
  }
  function draw(rows, sentence) {
    panel.replaceChildren(...rows, list);
    if (said) said.textContent = sentence;
  }
  function drawAnswer(found) {
    answer = found;
    options = [];
    const query = found.query || "";
    const rows = [];
    if (found.head) rows.push(el("p", "fui-search-results-head", found.head));
    if (found.unfinished) rows.push(status(" fui-tone-warn", "triangle-alert", found.unfinished));
    const hits = (found.groups || []).filter((group) => group.hits.length);
    let sentence;
    if (!hits.length) {
      const [sentenceWords, line] = found.none || [fill(words.nothing, { query }), ""];
      rows.push(el("div", "fui-search-results-empty", icon("search-x"), el("div", null, el("strong", null, sentenceWords), line ? el("span", null, line) : null)));
      sentence = sentenceWords;
    }
    let at = 0;
    const groups = hits.map((group, index) => {
      const kind = el("div", "fui-search-results-group", group.kind);
      kind.id = id + "-kind-" + (index + 1);
      kind.setAttribute("role", "presentation");
      const holder = el("div", null, kind, ...group.hits.slice(0, PER_GROUP).map((hit) => option(id + "-hit-" + ++at, "fui-hit", { hit: hit.href }, ...hitBody(hit, query, icon))));
      holder.setAttribute("role", "group");
      holder.setAttribute("aria-labelledby", kind.id);
      return holder;
    });
    if (hits.length) {
      groups.push(option(id + "-all", "fui-search-results-more", { all: true }, el("span", null, fill(words.all, { query })), icon("arrow-right")));
      sentence = fill(words.counted, { query, count: at });
    }
    if (found.more) groups.push(option(id + "-more", "fui-search-results-more", { way: found.more.href }, el("span", null, found.more.words), icon("arrow-right")));
    list.replaceChildren(...groups);
    draw(rows, sentence);
    active = null;
  }
  function failed(failure) {
    answer = null;
    options = [];
    list.replaceChildren();
    let sentence;
    if (failure === "unreachable") sentence = text(words.unreachable);
    else if (failure === "unreadable") sentence = text(words.unreadable);
    else sentence = fill(words.failed, { status: failure });
    draw([status(" fui-tone-bad", "circle-alert", sentence)], sentence);
  }
  function pick(target) {
    reset();
    if (target.all) form.requestSubmit();
    else {
      setBusy(true);
      window.location.assign(target.hit || target.way);
    }
  }
  async function send(query) {
    const mine = ++asked;
    answer = null;
    options = [];
    list.replaceChildren();
    draw([status("", null, text(words.searching))], text(words.searching));
    open(true);
    const result = await ask(route, form, field.name, query);
    if (mine !== asked) return;
    if (result.status === 401) {
      const session = await import("./session.js");
      if (signedOut) return session.goneTo(signedOut);
      session.gone();
    }
    if (result.answer && Array.isArray(result.answer.groups || [])) drawAnswer(result.answer);
    else failed(result.failure || result.status);
  }

  field.addEventListener("input", () => {
    activate(null);
    clearTimeout(pending);
    const query = field.value;
    if (!wants(query, from)) return reset();
    pending = setTimeout(() => send(query.trim()), SETTLE);
  });
  field.addEventListener("focus", () => {
    if (answer || panel.querySelector(".fui-tone-bad")) open(true);
  });
  field.addEventListener("blur", () => open(false));
  panel.addEventListener("mousedown", (event) => event.preventDefault());
  field.addEventListener("keydown", (event) => {
    const key = event.key;
    if (key === "ArrowDown" || key === "ArrowUp") {
      if (!options.length) return;
      event.preventDefault();
      if (panel.hidden) open(true);
      activate(step(active, options.length, key === "ArrowDown"));
    } else if (key === "Enter") {
      if (!panel.hidden && active !== null && answer) {
        event.preventDefault();
        pick(targets(answer)[active]);
      }
    } else if (key === "Escape") {
      const what = escaped(!panel.hidden, field.value === "");
      if (what === "close") {
        event.preventDefault();
        open(false);
      } else if (what === "clear") {
        event.preventDefault();
        field.value = "";
        reset();
      }
    } else if (key === "Tab") open(false);
  });
}
