// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The date field (R23, R25). The server draws the field, its disabled
// calendar button and its refusal line; this enables the button and draws
// the picker from the days and the words the server hands it
// (data-fui-date, JSON): one month, its arrows, a month and year chooser
// behind the title, the quick choices or a time row. Cancel and Escape put
// back what the field held when the picker opened; Apply keeps it.

import { allows, arrow, dayName, element, fill, fromNumber, glyph, grid, iso, keyTarget, monthTitle, openingDay, pad, parseIso, parseMoment, picker, reaches, reason, stepMonth, text, config as read } from "./calendar.js";

export function enhance(root) {
  const cfg = read(root, "data-fui-date");
  if (cfg) field(root, cfg, cfg.words, root.querySelector(".fui-input-group button"));
}

function field(root, config, words, button) {
  const input = root.querySelector("input");
  const line = root.querySelector("p[aria-live]");
  const withTime = config.with_time;
  let original = input.value;
  let touched = false;
  let open = false;
  let months = false;
  let panel = null;
  let clock = { hour: "00", minute: "00" };
  const initial = parseMoment(input.value, withTime, words.order);
  if (initial.hour !== undefined) clock = { hour: pad(initial.hour, 2), minute: pad(initial.minute, 2) };
  const state = {
    range: false,
    focused: openingDay(initial.day, config.today, config.bounds),
    shown: null,
    chosenDay() {
      const parsed = parseMoment(input.value, withTime, words.order);
      return parsed.error ? null : parsed.day;
    },
    chosen(n) {
      return this.chosenDay() === n;
    },
    classes(n, out) {
      return "fui-day" + (out ? " fui-day-out" : "");
    },
    hover() {},
    choose(n) {
      touched = true;
      input.value = iso(n) + (withTime ? " " + clock.hour.trim().padStart(2, "0") + ":" + clock.minute.trim().padStart(2, "0") : "");
      this.focused = n;
      reveal(n);
      sync(true);
    },
    key(event) {
      const target = keyTarget(event, this.focused, config.first);
      if (target === undefined) return;
      event.preventDefault();
      if (target !== null && allows(config.bounds, target)) {
        this.focused = target;
        reveal(target);
        draw(true);
      }
    },
    hover() {},
    close,
  };
  function reveal(n) {
    const { y, m } = fromNumber(n);
    state.shown = { y, m };
  }
  function sync(focus = false) {
    const why = touched ? reason(words, input.value, withTime, config.bounds) : null;
    if (touched) {
      line.textContent = why || "";
      line.hidden = !why;
      if (why) input.setAttribute("aria-invalid", "true");
      else input.removeAttribute("aria-invalid");
    }
    if (open) draw(focus);
  }
  function status() {
    const why = touched ? reason(words, input.value, withTime, config.bounds) : null;
    if (why) return parseMoment(input.value, withTime, words.order).error ? text(words.type_format) : why;
    const chosen = state.chosenDay();
    if (chosen !== null && config.chosen) return fill(config.chosen, { day: dayName(words, chosen) });
    return "";
  }
  // The panel's frame, its time row, status line and buttons are made once
  // as it opens, so a control the reader holds stays the same element; a
  // draw redraws the quick choices and the month.
  let frame = null;
  function build() {
    const said = element("p", { class: "fui-datepicker-status", role: "status", hidden: true });
    const main = element("div", { class: "fui-datepicker-main" });
    if (config.with_time) {
      const part = (key, label) => element("input", { class: "fui-input", inputmode: "numeric", maxlength: "2", "aria-label": text(label), value: clock[key], oninput: (event) => {
        clock[key] = event.target.value;
        const chosen = state.chosenDay();
        if (chosen !== null) {
          touched = true;
          input.value = iso(chosen) + " " + clock.hour.trim().padStart(2, "0") + ":" + clock.minute.trim().padStart(2, "0");
          sync();
        }
      } });
      main.append(element("div", { class: "fui-datepicker-time" },
        element("span", { class: "fui-label" }, text(words.time) + " ", element("span", { class: "fui-muted", text: text(words.time_hint) })),
        element("div", { class: "fui-datepicker-time-fields", role: "group", "aria-label": text(words.time) }, part("hour", words.hours), element("span", { "aria-hidden": "true", text: ":" }), part("minute", words.minutes)),
      ));
    }
    main.append(said);
    const apply = element("button", { type: "button", class: "fui-button fui-button-primary fui-button-small", text: text(words.apply), onclick: () => close(false) });
    panel.append(element("span", { class: "fui-datepicker-handle", "aria-hidden": "true" }), element("p", { class: "fui-datepicker-label", text: config.label }));
    panel.append(element("div", { class: "fui-datepicker-body" }, main));
    panel.append(element("div", { class: "fui-datepicker-foot" },
      element("div", { class: "fui-actions" },
        element("button", { type: "button", class: "fui-button fui-button-ghost fui-button-small", disabled: !allows(config.bounds, config.today), text: text(words.today), onclick: () => state.choose(config.today) }),
        element("button", { type: "button", class: "fui-button fui-button-ghost fui-button-small", text: text(words.clear), onclick: () => {
          touched = true;
          input.value = "";
          sync();
        } }),
      ),
      element("div", { class: "fui-actions" },
        element("button", { type: "button", class: "fui-button fui-button-secondary fui-button-small", text: text(words.cancel), onclick: () => close(true) }),
        apply,
      ),
    ));
    frame = { main, anchor: main.firstChild, said, apply, drawn: [] };
  }
  function draw(focus) {
    if (!panel) return;
    if (!frame) build();
    panel.classList.toggle("is-end", Boolean(state.flip));
    const parts = [];
    if (config.quick.length) {
      parts.push(element("div", { class: "fui-datepicker-quick", role: "group", "aria-label": text(words.quick) }, config.quick.map(([dayText, label]) => {
        const n = parseIso(dayText);
        return element("button", { type: "button", class: "fui-chip-check fui-datepicker-chip", "aria-pressed": String(n !== null && state.chosenDay() === n), disabled: n === null, text: label, onclick: () => n !== null && state.choose(n) });
      })));
    }
    const { y, m } = state.shown;
    if (months) parts.push(monthChooser(y));
    else {
      const earlier = stepMonth(y, m, -1);
      const later = stepMonth(y, m, 1);
      const title = monthTitle(words, y, m);
      const head = element("div", { class: "fui-datepicker-head" },
        arrow(root, "chevron-left", text(words.previous_month), "fui-icon-button", !earlier || !reaches(config.bounds, earlier.y, earlier.m), () => {
          state.shown = earlier;
          draw(false);
        }),
        element("button", { type: "button", class: "fui-datepicker-title", "aria-label": fill(text(words.choose_month), { title }), text: title, onclick: () => {
          months = true;
          draw(false);
        } }, glyph(root, "chevron-down")),
        arrow(root, "chevron-right", text(words.next_month), "fui-icon-button", !later || !reaches(config.bounds, later.y, later.m), () => {
          state.shown = later;
          draw(false);
        }),
      );
      parts.push(element("div", { class: "fui-datepicker-month" }, head, grid(config, state, y, m)));
    }
    for (const old of frame.drawn) old.remove();
    for (const part of parts) frame.main.insertBefore(part, frame.anchor);
    frame.drawn = parts;
    const said = status();
    frame.said.textContent = said;
    frame.said.hidden = said === "";
    frame.apply.disabled = touched && reason(words, input.value, withTime, config.bounds) !== null;
    if (focus) {
      const target = panel.querySelector("[data-day='" + iso(state.focused) + "']");
      if (target) requestAnimationFrame(() => target.focus());
    }
  }
  function monthChooser(year) {
    let shownYear = year;
    const box = element("div", { class: "fui-datepicker-month" });
    const render = () => {
      box.replaceChildren();
      const reachesYear = (y) => y >= 1 && y <= 9999 && [...Array(12).keys()].some((i) => reaches(config.bounds, y, i + 1));
      box.append(element("div", { class: "fui-datepicker-head" },
        arrow(root, "chevron-left", text(words.previous_year), "fui-icon-button", !reachesYear(shownYear - 1), () => {
          shownYear -= 1;
          render();
        }),
        element("button", { type: "button", class: "fui-datepicker-title", "aria-label": text(words.back_to_days), text: String(shownYear), onclick: () => {
          months = false;
          draw(false);
        } }, glyph(root, "chevron-up")),
        arrow(root, "chevron-right", text(words.next_year), "fui-icon-button", !reachesYear(shownYear + 1), () => {
          shownYear += 1;
          render();
        }),
      ));
      const today = fromNumber(config.today);
      box.append(element("div", { class: "fui-datepicker-months", role: "grid", "aria-label": fill(text(words.months_of), { year: shownYear }) },
        [...Array(12).keys()].map((i) => element("button", {
          type: "button",
          class: "fui-datepicker-cell",
          "aria-label": monthTitle(words, shownYear, i + 1),
          "aria-selected": state.shown.y === shownYear && state.shown.m === i + 1 ? "true" : null,
          "aria-current": today.y === shownYear && today.m === i + 1 ? "date" : null,
          disabled: !reaches(config.bounds, shownYear, i + 1),
          text: [...text(words.months[i])].slice(0, 3).join(""),
          onclick: () => {
            state.shown = { y: shownYear, m: i + 1 };
            months = false;
            draw(false);
          },
        })),
      ));
    };
    render();
    return box;
  }
  function close(restore) {
    if (restore) input.value = original;
    open = false;
    months = false;
    if (panel) panel.remove();
    panel = null;
    frame = null;
    button.setAttribute("aria-expanded", "false");
    if (restore) sync();
    input.focus();
  }
  button.disabled = false;
  button.setAttribute("aria-label", text(words.choose));
  input.addEventListener("input", () => {
    touched = true;
    const parsed = parseMoment(input.value, withTime, words.order);
    if (!parsed.error) {
      if (parsed.hour !== undefined) clock = { hour: pad(parsed.hour, 2), minute: pad(parsed.minute, 2) };
      state.focused = parsed.day;
      reveal(parsed.day);
    }
    sync();
  });
  button.addEventListener("click", () => {
    if (open) return close(true);
    original = input.value;
    state.focused = openingDay(state.chosenDay(), config.today, config.bounds);
    reveal(state.focused);
    months = false;
    open = true;
    panel = picker(root, config, state);
    root.append(panel);
    button.setAttribute("aria-expanded", "true");
    draw(true);
    const field = root.getBoundingClientRect();
    state.flip = field.left + panel.getBoundingClientRect().width > window.innerWidth - 16;
    panel.classList.toggle("is-end", state.flip);
  });
  document.addEventListener("pointerdown", (event) => {
    if (open && !root.contains(event.target)) close(false);
  });
}

