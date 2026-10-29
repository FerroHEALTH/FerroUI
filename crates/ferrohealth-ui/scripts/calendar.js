// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The calendar the date field and the date range share: days as numbers
// counted from 1970-01-01, a typed text read as the library's
// calendar::Moment reads it, the refusal DateWords::reason says, and the
// grid of a month with its keyboard path (WAI-ARIA Authoring Practices,
// the date picker dialog pattern). No document is touched here but the
// nodes it is asked to make.
const OPEN = "";
const SHUT = "";

export function fill(phrase, values) {
  let out = phrase;
  for (const [name, value] of Object.entries(values)) out = out.split(OPEN + name + SHUT).join(String(value));
  return out;
}

// Days counted from 1970-01-01 (Howard Hinnant's days_from_civil), so a day
// is one integer and stepping is addition.
export function toNumber(y, m, d) {
  const year = y - (m <= 2 ? 1 : 0);
  const era = Math.floor(year / 400);
  const yoe = year - era * 400;
  const doy = Math.floor((153 * (m + (m > 2 ? -3 : 9)) + 2) / 5) + d - 1;
  const doe = yoe * 365 + Math.floor(yoe / 4) - Math.floor(yoe / 100) + doy;
  return era * 146097 + doe - 719468;
}

export function fromNumber(n) {
  const days = n + 719468;
  const era = Math.floor(days / 146097);
  const doe = days - era * 146097;
  const yoe = Math.floor((doe - Math.floor(doe / 1460) + Math.floor(doe / 36524) - Math.floor(doe / 146096)) / 365);
  const doy = doe - (365 * yoe + Math.floor(yoe / 4) - Math.floor(yoe / 100));
  const mp = Math.floor((5 * doy + 2) / 153);
  const d = doy - Math.floor((153 * mp + 2) / 5) + 1;
  const m = mp < 10 ? mp + 3 : mp - 9;
  return { y: yoe + era * 400 + (m <= 2 ? 1 : 0), m, d };
}

export const leap = (y) => (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0;
export const daysIn = (y, m) => (m === 2 ? (leap(y) ? 29 : 28) : [4, 6, 9, 11].includes(m) ? 30 : 31);
export const pad = (n, w) => String(n).padStart(w, "0");
export const iso = (n) => {
  const { y, m, d } = fromNumber(n);
  return pad(y, 4) + "-" + pad(m, 2) + "-" + pad(d, 2);
};
// Monday 0 to Sunday 6; 1970-01-01 was a Thursday.
export const weekday = (n) => (((n + 3) % 7) + 7) % 7;

function day(y, m, d) {
  if (!(y >= 1 && y <= 9999) || m < 1 || m > 12 || d < 1 || d > daysIn(y, m)) return null;
  return toNumber(y, m, d);
}

const digits = (text) => /^[0-9]+$/.test(text);

// Reads a day as Day::parse_in reads it: YYYY-MM-DD always, and the page's
// order (dmy or mdy) with one- or two-digit parts split by . / or -.
export function parseDay(text, order) {
  const t = text.trim();
  const iso = /^([0-9]{4})-([0-9]{2})-([0-9]{2})$/.exec(t);
  if (iso) {
    const [y, m, d] = [Number(iso[1]), Number(iso[2]), Number(iso[3])];
    if (y < 1) return { error: "format" };
    if (m < 1 || m > 12) return { error: "month", month: m };
    const n = day(y, m, d);
    return n === null ? { error: "day", year: y, month: m } : { day: n };
  }
  if (order === "Ymd") return { error: "format" };
  const parts = t.split(/[./-]/);
  if (parts.length !== 3) return { error: "format" };
  const [first, second, year] = parts;
  const [dd, mm] = order === "Dmy" ? [first, second] : [second, first];
  const short = (p) => (p.length >= 1 && p.length <= 2 && digits(p) ? Number(p) : null);
  const d = short(dd);
  const m = short(mm);
  if (d === null || m === null || year.length !== 4 || !digits(year) || Number(year) < 1) return { error: "format" };
  const y = Number(year);
  if (m < 1 || m > 12) return { error: "month", month: m };
  const n = day(y, m, d);
  return n === null ? { error: "day", year: y, month: m } : { day: n };
}

// Reads a field's text as Moment::parse_in does: a day, and with a time
// "YYYY-MM-DD HH:MM" (or a T for the space).
export function parseMoment(text, withTime, order) {
  const t = text.trim();
  if (!withTime) return parseDay(t, order);
  const at = t.search(/[ T]/);
  if (at < 0) {
    const plain = parseDay(t, order);
    return plain.error ? plain : { error: "format" };
  }
  const parsed = parseDay(t.slice(0, at), order);
  if (parsed.error) return parsed;
  const clock = /^([0-9]{2}):([0-9]{2})$/.exec(t.slice(at + 1).trim());
  if (!clock || Number(clock[1]) > 23 || Number(clock[2]) > 59) return { error: "time" };
  return { day: parsed.day, hour: Number(clock[1]), minute: Number(clock[2]) };
}

export function allows(bounds, n) {
  return (bounds.earliest === null || n >= bounds.earliest) && (bounds.latest === null || n <= bounds.latest);
}

// Why a typed text is no value the field takes, as DateWords::reason says.
export function reason(words, text, withTime, bounds) {
  const t = text.trim();
  if (t === "") return null;
  const parsed = parseMoment(t, withTime, words.order);
  if (parsed.error === "format") return fill(words.not_format, { value: t });
  if (parsed.error === "time") return fill(words.not_a_time, { value: t });
  if (parsed.error === "month") return fill(words.no_month, { value: t, month: parsed.month });
  if (parsed.error === "day") {
    const place = parsed.month === 2 && leap(parsed.year) ? 12 : parsed.month - 1;
    return fill(words.not_a_day, { value: t, why: words.month_lengths[place] || "" });
  }
  if (bounds.earliest !== null && parsed.day < bounds.earliest) return fill(words.before, { value: t, earliest: iso(bounds.earliest) });
  if (bounds.latest !== null && parsed.day > bounds.latest) return fill(words.after, { value: t, latest: iso(bounds.latest) });
  return null;
}

export const text = (word) => (word && typeof word === "object" ? word.en : word || "");
export const parseIso = (value) => (value ? parseDay(value, "Ymd").day ?? null : null);

export function monthTitle(words, y, m) {
  return fill(text(words.month_title), { month: text(words.months[m - 1]), year: y });
}

export function dayName(words, n) {
  const { y, m, d } = fromNumber(n);
  return fill(text(words.day_name), { weekday: text(words.weekdays[weekday(n)]), day: d, month: text(words.months[m - 1]), year: y });
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

// The icon `name` from the field's own <template data-fui-icons>, which the
// server drew, so no outline is written twice.
export function glyph(root, name) {
  const holder = root.querySelector("template[data-fui-icons]");
  const found = holder && holder.content.querySelector("[data-icon='" + name + "'] svg");
  return found ? found.cloneNode(true) : document.createTextNode("");
}

// The weeks of a month, each seven day numbers, from the week holding its
// first day to the week holding its last.
export function weeks(y, m, first) {
  const start = toNumber(y, m, 1);
  const back = (7 + weekday(start) - first) % 7;
  const end = toNumber(y, m, daysIn(y, m));
  const out = [];
  for (let at = start - back; at <= end; at += 7) {
    out.push([0, 1, 2, 3, 4, 5, 6].map((step) => at + step));
  }
  return out;
}

export function stepMonth(y, m, count) {
  const total = y * 12 + (m - 1) + count;
  const year = Math.floor(total / 12);
  if (year < 1 || year > 9999) return null;
  return { y: year, m: (total % 12) + 1 };
}

export function reaches(bounds, y, m) {
  const first = toNumber(y, m, 1);
  const last = toNumber(y, m, daysIn(y, m));
  return (bounds.earliest === null || last >= bounds.earliest) && (bounds.latest === null || first <= bounds.latest);
}

export function openingDay(chosen, today, bounds) {
  if (chosen !== null && chosen !== undefined) return chosen;
  if (bounds.earliest !== null && today < bounds.earliest) return bounds.earliest;
  if (bounds.latest !== null && today > bounds.latest) return bounds.latest;
  return today;
}

export function picker(root, config, state) {
  const { words } = config;
  const panel = element("div", { class: "fui-datepicker" + (state.range ? " fui-datepicker-range" : ""), id: config.id + "-picker", role: "dialog", "aria-label": config.label });
  panel.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      state.close(true);
    }
  });
  if (state.range) panel.addEventListener("mouseleave", () => state.hover(null));
  return panel;
}

export function grid(config, state, y, m) {
  const { words } = config;
  const head = element("tr", null, [0, 1, 2, 3, 4, 5, 6].map((i) => {
    const place = (config.first + i) % 7;
    return element("th", { scope: "col", abbr: text(words.weekdays[place]), text: text(words.weekdays_short[place]) });
  }));
  const rows = weeks(y, m, config.first).map((week) =>
    element("tr", null, week.map((n) => {
      const out = fromNumber(n).m !== m;
      const allowed = allows(config.bounds, n);
      const button = element("button", {
        type: "button",
        class: state.classes(n, out),
        "aria-label": dayName(words, n),
        "data-day": out ? null : iso(n),
        "data-out-day": out ? iso(n) : null,
        tabindex: !out && state.focused === n ? "0" : "-1",
        disabled: !allowed,
        text: String(fromNumber(n).d),
        onmouseenter: () => state.hover(n),
        onclick: () => state.choose(n),
      });
      return element("td", { role: "gridcell", "aria-selected": state.chosen(n) ? "true" : null, "aria-current": n === config.today ? "date" : null }, button);
    })),
  );
  const table = element("table", { role: "grid", "aria-label": monthTitle(words, y, m), class: "fui-datepicker-grid" }, element("thead", null, head), element("tbody", null, rows));
  table.addEventListener("keydown", (event) => state.key(event));
  return table;
}

export function arrow(root, name, label, cls, blocked, press) {
  return element("button", { type: "button", "aria-label": label, disabled: blocked, class: cls, onclick: press }, glyph(root, name));
}


// The day a key moves the grid's focus to from `from`, for a week starting
// on `first`, or undefined for a key the grid leaves alone: the arrows a day
// or a week, Page Up and Page Down a month (with Shift a year), Home and
// End the week's ends.
export function keyTarget(event, from, first) {
  const at = fromNumber(from);
  const span = event.shiftKey ? 12 : 1;
  const start = from - ((7 + weekday(from) - first) % 7);
  switch (event.key) {
    case "ArrowLeft":
      return from - 1;
    case "ArrowRight":
      return from + 1;
    case "ArrowUp":
      return from - 7;
    case "ArrowDown":
      return from + 7;
    case "PageUp":
    case "PageDown": {
      const step = stepMonth(at.y, at.m, event.key === "PageUp" ? -span : span);
      return step ? toNumber(step.y, step.m, Math.min(at.d, daysIn(step.y, step.m))) : null;
    }
    case "Home":
      return start;
    case "End":
      return start + 6;
    default:
      return undefined;
  }
}

export function config(root, attribute) {
  const raw = root.getAttribute(attribute);
  if (!raw) return null;
  const parsed = JSON.parse(raw);
  parsed.today = parseIso(parsed.today);
  parsed.bounds = { earliest: parseIso(parsed.earliest), latest: parseIso(parsed.latest) };
  return parsed;
}
