// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The date range of a history filter (R22, R25). With no script its From
// and To fields show; this hides them, shows the button naming the range
// and draws the picker from the days and the words the server hands it
// (data-fui-range, JSON): the presets beside two months. Apply writes both
// fields and submits the form.

import { allows, arrow, element, fill, fromNumber, grid, iso, keyTarget, monthTitle, openingDay, parseIso, picker, reaches, stepMonth, text, config as read } from "./calendar.js";

export function enhance(root) {
  const cfg = read(root, "data-fui-range");
  if (cfg) rangeField(root, cfg, cfg.words, root.querySelector(":scope > button"));
}

function rangeField(root, config, words, button) {
  const fields = root.querySelector(".fui-date-range-fields");
  const [from, to] = fields.querySelectorAll("input");
  const summary = button.querySelector("[data-fui-summary]");
  let saved = [from.value, to.value];
  let hovered = null;
  let open = false;
  let panel = null;
  const ends = () => [parseIso(from.value), parseIso(to.value)];
  const summarize = (a, b) => {
    const [x, y] = [parseIso(a), parseIso(b)];
    return x !== null && y !== null ? fill(text(words.range_words), { from: iso(x), to: iso(y) }) : text(words.any);
  };
  const state = {
    range: true,
    focused: openingDay(ends()[1] ?? ends()[0], config.today, config.bounds),
    shown: null,
    chosen(n) {
      const [a, b] = ends();
      return a === n || b === n;
    },
    classes(n, out) {
      let cls = "fui-day" + (out ? " fui-day-out" : "");
      const [first, last] = ends();
      if (first === null) return cls;
      const tentative = last === null;
      const end = last !== null ? last : hovered !== null && hovered > first ? hovered : null;
      if (end === null) return cls;
      if (first < n && n < end) cls += tentative ? " fui-day-between fui-day-tentative" : " fui-day-between";
      if (first < end && n === first) cls += " fui-day-start";
      if (first < end && n === end) cls += tentative ? " fui-day-end fui-day-tentative-end" : " fui-day-end";
      return cls;
    },
    hover(n) {
      if (hovered === n) return;
      hovered = n;
      if (open) draw(false);
    },
    choose(n) {
      const [first, last] = ends();
      if (first !== null && last === null && n < first) {
        from.value = iso(n);
        to.value = iso(first);
      } else if (first !== null && last === null) to.value = iso(n);
      else {
        from.value = iso(n);
        to.value = "";
      }
      hovered = null;
      this.focused = n;
      reveal(n);
      draw(true);
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
    close,
  };
  function reveal(n) {
    const { y, m } = fromNumber(n);
    const shown = state.shown;
    const earlier = shown ? stepMonth(shown.y, shown.m, -1) : null;
    const key = (v) => v.y * 12 + v.m;
    if (!shown) state.shown = { y, m };
    else if (y * 12 + m > key(shown)) state.shown = { y, m };
    else if (earlier && y * 12 + m < key(earlier)) state.shown = stepMonth(y, m, 1) || { y, m };
  }
  function status() {
    const [first, last] = ends();
    if (first !== null && last !== null) {
      const count = Math.abs(last - first) + 1;
      const days = count <= words.day_counts.length ? text(words.day_counts[count - 1]) : null;
      return days ? fill(text(words.range_full), { from: iso(first), to: iso(last), days }) : fill(text(words.range_words), { from: iso(first), to: iso(last) });
    }
    if (first !== null) return fill(text(words.range_half), { from: iso(first) });
    return "";
  }
  function month(y, m, back, next) {
    const earlier = stepMonth(y, m, -1);
    const later = stepMonth(y, m, 1);
    const head = element("div", { class: "fui-datepicker-head" });
    if (back) {
      head.append(arrow(root, "chevron-left", text(words.previous_month), back, !earlier || !reaches(config.bounds, earlier.y, earlier.m), () => {
        const shown = stepMonth(state.shown.y, state.shown.m, -1);
        if (shown) state.shown = shown;
        draw(false);
      }));
    } else head.append(element("span", { class: "fui-datepicker-navgap" }));
    head.append(element("span", { class: "fui-datepicker-title", text: monthTitle(words, y, m) }));
    if (next) {
      head.append(arrow(root, "chevron-right", text(words.next_month), "fui-icon-button", !later || !reaches(config.bounds, later.y, later.m), () => {
        const shown = stepMonth(state.shown.y, state.shown.m, 1);
        if (shown) state.shown = shown;
        draw(false);
      }));
    } else head.append(element("span", { class: "fui-datepicker-navgap" }));
    return element("div", { class: "fui-datepicker-month" }, head, grid(config, state, y, m));
  }
  // The frame, the status line and the buttons are made once as the panel
  // opens, so a control the reader holds stays the same element; a draw
  // redraws the presets and the two months.
  let frame = null;
  function build() {
    const presets = element("div", { class: "fui-datepicker-presets", role: "group", "aria-label": text(words.ranges) });
    const pair = element("div", { class: "fui-datepicker-pair" });
    const said = element("p", { class: "fui-datepicker-status", role: "status", hidden: true });
    const apply = element("button", { type: "button", class: "fui-button fui-button-primary fui-button-small", text: text(words.apply), onclick: () => {
      close(false);
      if (from.form) from.form.requestSubmit();
    } });
    panel.append(element("span", { class: "fui-datepicker-handle", "aria-hidden": "true" }), element("p", { class: "fui-datepicker-label", text: config.label }));
    panel.append(element("div", { class: "fui-datepicker-body" }, presets, element("div", { class: "fui-datepicker-main" }, pair, said)));
    panel.append(element("div", { class: "fui-datepicker-foot" },
      element("div", { class: "fui-actions" }, element("button", { type: "button", class: "fui-button fui-button-ghost fui-button-small", text: text(words.clear), onclick: () => {
        from.value = "";
        to.value = "";
        hovered = null;
        draw(false);
      } })),
      element("div", { class: "fui-actions" },
        element("button", { type: "button", class: "fui-button fui-button-secondary fui-button-small", text: text(words.cancel), onclick: () => close(true) }),
        apply,
      ),
    ));
    frame = { presets, pair, said, apply };
  }
  function draw(focus) {
    if (!panel) return;
    if (!frame) build();
    panel.classList.toggle("is-start", Boolean(state.flip));
    frame.presets.replaceChildren(...config.presets.map(([span, label]) => {
      const pair = span ? [parseIso(span[0]), parseIso(span[1])] : null;
      const [a, b] = ends();
      return element("button", { type: "button", class: "fui-datepicker-preset", "aria-pressed": String(pair !== null && a === pair[0] && b === pair[1]), disabled: pair === null, text: label, onclick: () => {
        if (!pair) return;
        from.value = iso(pair[0]);
        to.value = iso(pair[1]);
        hovered = null;
        state.focused = pair[1];
        reveal(pair[1]);
        draw(false);
      } });
    }));
    const later = state.shown;
    const earlier = stepMonth(later.y, later.m, -1) || later;
    frame.pair.replaceChildren(month(earlier.y, earlier.m, "fui-icon-button", false), month(later.y, later.m, "fui-icon-button fui-datepicker-phone-prev", true));
    const said = status();
    frame.said.textContent = said;
    frame.said.hidden = said === "";
    const [a, b] = ends();
    frame.apply.disabled = (a !== null) !== (b !== null);
    if (focus) {
      const target = panel.querySelector("[data-day='" + iso(state.focused) + "']");
      if (target) requestAnimationFrame(() => target.focus());
    }
  }
  function close(restore) {
    if (restore) [from.value, to.value] = saved;
    hovered = null;
    open = false;
    if (panel) panel.remove();
    panel = null;
    frame = null;
    button.setAttribute("aria-expanded", "false");
    summary.textContent = summarize(from.value, to.value);
  }
  fields.hidden = true;
  button.hidden = false;
  button.addEventListener("click", () => {
    if (open) return close(true);
    saved = [from.value, to.value];
    const [first, last] = ends();
    state.focused = openingDay(last ?? first, config.today, config.bounds);
    state.shown = null;
    reveal(state.focused);
    open = true;
    panel = picker(root, config, state);
    root.append(panel);
    button.setAttribute("aria-expanded", "true");
    draw(true);
    const field = root.getBoundingClientRect();
    state.flip = field.right - panel.getBoundingClientRect().width < 16;
    panel.classList.toggle("is-start", state.flip);
  });
  document.addEventListener("pointerdown", (event) => {
    if (open && !root.contains(event.target)) close(true);
  });
}
