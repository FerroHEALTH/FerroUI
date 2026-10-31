// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The top bar's search: its shortcut, its busy state and, where the console
// names an answer route, its typeahead (WAI-ARIA Authoring Practices 1.2,
// the combobox with a listbox popup). The rules are the crate's
// is_shortcut, wants, targets, step, escaped and placed; a hit is drawn as
// the crate's hit_body draws it.
import { typing } from "./tabs.js";
import { fill, text } from "./shared.js";

export const SETTLE = 200;
export const PER_GROUP = 5;

export function wants(words, from) {
  const held = [...words.trim()].length;
  return held > 0 && held >= from;
}

export function escaped(open, empty) {
  return open ? "close" : empty ? "pass" : "clear";
}

export function step(active, count, down) {
  if (count === 0) return null;
  const last = count - 1;
  if (!down) return active === null || active === 0 ? last : Math.min(active, count) - 1;
  return active === null ? 0 : active >= last ? 0 : active + 1;
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

// The runs of `value`, each marked when it matches `query` without regard
// to case, as the crate's runs.
export function runs(value, query) {
  const needle = [...query.trim()].flatMap((c) => [...c.toLowerCase()]);
  if (!needle.length) return [[value, false]];
  const chars = [...value];
  const out = [];
  let plain = "";
  let at = 0;
  while (at < chars.length) {
    let matched = 0;
    let next = at;
    let ok = true;
    while (matched < needle.length) {
      if (next >= chars.length) {
        ok = false;
        break;
      }
      for (const lower of chars[next].toLowerCase()) {
        if (needle[matched] !== lower) ok = false;
        matched++;
      }
      if (!ok) break;
      next++;
    }
    if (ok) {
      if (plain) out.push([plain, false]);
      plain = "";
      out.push([chars.slice(at, next).join(""), true]);
      at = next;
    } else plain += chars[at++];
  }
  if (plain) out.push([plain, false]);
  return out;
}

function el(tag, cls, ...children) {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  node.append(...children.filter((child) => child !== null && child !== undefined));
  return node;
}

export function enhance(form) {
  const field = form.querySelector("input[type='search']");
  const submit = form.querySelector("button[type='submit']");
  const glyph = form.querySelector(".fui-search-glyph");
  const icons = form.querySelector("template[data-fui-icons]");
  const words = JSON.parse(form.getAttribute("data-fui-words") || "{}");
  const shortcut = form.getAttribute("data-fui-shortcut");
  const route = form.getAttribute("data-fui-suggest");
  const from = Number(form.getAttribute("data-fui-from") || "2");
  const signedOut = form.getAttribute("data-fui-signed-out");
  const panel = form.querySelector(".fui-search-results");
  const list = panel && panel.querySelector("[role='listbox']");
  const said = form.querySelector(":scope > [role='status']");
  const id = field.id;
  const icon = (name) => {
    const found = icons && icons.content.querySelector("[data-icon='" + name + "'] svg");
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
  function emphasised(value, query) {
    return runs(value, query).map(([run, marked]) => (marked ? el("mark", "fui-search-match", run) : document.createTextNode(run)));
  }
  function hitBody(hit, query) {
    let mark = null;
    if (hit.mark && hit.mark.Dot) {
      mark = el("span", "fui-dot fui-tone-" + String(hit.mark.Dot).toLowerCase());
      mark.setAttribute("aria-hidden", "true");
    } else if (hit.mark && hit.mark.Glyph) mark = icon(hit.mark.Glyph);
    return [mark, el("span", "fui-hit-name", ...emphasised(hit.name, query)), el("span", "fui-small fui-muted fui-hit-line", ...emphasised(hit.line || "", query)), hit.tag ? el("span", "fui-tag", hit.tag) : null];
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
      const holder = el("div", null, kind, ...group.hits.slice(0, PER_GROUP).map((hit) => option(id + "-hit-" + ++at, "fui-hit", { hit: hit.href }, ...hitBody(hit, query))));
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
    const sentence = failure === "unreachable" ? text(words.unreachable) : failure === "unreadable" ? text(words.unreadable) : fill(words.failed, { status: failure });
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
    const body = new URLSearchParams([[field.name, query]]);
    const post = form.method.toLowerCase() === "post";
    let result;
    try {
      const response = await fetch(post ? route : route + (route.includes("?") ? "&" : "?") + body, {
        method: post ? "POST" : "GET",
        credentials: "same-origin",
        headers: post ? { Accept: "application/json", "Content-Type": "application/x-www-form-urlencoded" } : { Accept: "application/json" },
        body: post ? body.toString() : undefined,
      });
      if (!response.ok) result = { status: response.status };
      else {
        try {
          result = { answer: await response.json() };
        } catch (error) {
          console.warn("fui: the search's answer is unreadable", error);
          result = { failure: "unreadable" };
        }
      }
    } catch (error) {
      console.warn("fui: the browser refused the search's request", error);
      result = { failure: "unreachable" };
    }
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
