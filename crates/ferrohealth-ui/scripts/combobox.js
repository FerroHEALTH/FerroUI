// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The combobox. With no script it is the native select the server drew;
// this replaces it with the WAI-ARIA 1.2 combobox pattern over a listbox
// (WAI-ARIA Authoring Practices, the editable combobox with list
// autocomplete): the input filters the options as the reader types; Down
// and Up open the list and move through it, Home and End go to its ends,
// Enter chooses, Escape closes it. It draws at most ROWS options and says
// when more match. Clearing the words clears the value. A hidden input
// carries the select's name, so the form posts the same value either way.
// Over a console's search (data-fui-remote) the typed words post under the
// remote's name and the options the page arrived with show unfiltered.
// A fui:choose event on the root puts it on the listed option of its
// value, or on none, and says no change (a pick-or-type field's back).
import { element } from "./shared.js";

export const ROWS = 100;

export function matches(option, needle) {
  return (
    option.label.toLowerCase().includes(needle) ||
    option.value.toLowerCase().includes(needle) ||
    (option.detail || "").toLowerCase().includes(needle)
  );
}

// The places of the options the typed words keep, at most ROWS of them,
// and whether more match.
export function shown(options, query, typing, remote) {
  const needle = query.trim().toLowerCase();
  const filtering = !remote && typing && needle !== "";
  const all = [];
  options.forEach((option, place) => {
    if (!filtering || matches(option, needle)) all.push(place);
  });
  return { places: all.slice(0, ROWS), more: all.length > ROWS };
}

const ARROWS = new Set(["ArrowDown", "ArrowUp"]);
const MOVES = new Map([
  ["ArrowDown", (active, count) => (active === null ? 0 : Math.min(active + 1, count - 1))],
  ["ArrowUp", (active) => (active === null ? 0 : Math.max(active - 1, 0))],
  ["Home", () => 0],
  ["End", (active, count) => count - 1],
]);

// The row the arrows, Home and End move the highlight to from `active`
// over `count` rows, null with no row, or undefined for any other key.
export function stepped(key, active, count) {
  const move = MOVES.get(key);
  if (!move) return undefined;
  return count === 0 ? null : move(active, count);
}

export function enhance(root) {
  const select = root.querySelector("select");
  if (!select) return;
  const id = select.id;
  const name = root.dataset.fuiName || select.name;
  const remote = root.dataset.fuiRemote;
  const options = [...select.options]
    .filter((option) => option.value !== "")
    .map((option) => ({ value: option.value, label: option.textContent, detail: option.dataset.detail, other: "other" in option.dataset }));
  const chosenAt = options.findIndex((option) => option.value === select.value);
  const check = root.querySelector("template[data-fui-icons]");
  const state = {
    selected: chosenAt >= 0 ? options[chosenAt].value : null,
    chosenLabel: chosenAt >= 0 ? options[chosenAt].label : null,
    open: false,
    typing: false,
    query: remote ? root.dataset.fuiQuery || "" : "",
    active: null,
  };
  const listId = id + "-list";
  const input = element("input", {
    id,
    type: "text",
    role: "combobox",
    class: "fui-input fui-combobox-input",
    autocomplete: "off",
    spellcheck: "false",
    "aria-autocomplete": "list",
    "aria-expanded": "false",
    "aria-controls": listId,
    "aria-describedby": select.getAttribute("aria-describedby"),
    placeholder: root.dataset.fuiPlaceholder || "",
    name: remote,
  });
  const hidden = element("input", { type: "hidden", name });
  const list = element("ul", { id: listId, role: "listbox", "aria-label": root.querySelector("label").textContent, class: "fui-combobox-list", hidden: true });
  const box = element("div", { class: "fui-combobox-box" }, input, hidden, list);
  select.replaceWith(box);

  const view = () => shown(options, state.query, state.typing, Boolean(remote));
  const words = () => {
    if (state.typing) return state.query;
    if (state.selected === null) return remote ? state.query : "";
    const found = options.find((option) => option.value === state.selected);
    return found ? found.label : state.chosenLabel || "";
  };
  function draw() {
    const { places, more } = view();
    input.setAttribute("aria-expanded", String(state.open));
    hidden.value = state.selected || "";
    if (document.activeElement !== input || !state.typing) input.value = words();
    list.hidden = !state.open;
    list.replaceChildren();
    places.forEach((place, row) => {
      const option = options[place];
      const chosen = option.value === state.selected;
      const item = element("li", {
        id: id + "-option-" + place,
        role: "option",
        class: "fui-combobox-option",
        "aria-selected": String(chosen),
        "data-active": state.active === row ? "true" : null,
      },
      element("span", { class: "fui-combobox-text" }, element("span", { class: "fui-combobox-label", text: option.label }), option.detail ? element("span", { class: "fui-combobox-detail", text: option.detail }) : null),
      element("span", { class: "fui-combobox-check", "aria-hidden": "true" }));
      if (chosen && check) {
        const svg = check.content.querySelector("svg");
        if (svg) item.lastChild.append(svg.cloneNode(true));
      }
      item.addEventListener("mousedown", (event) => event.preventDefault());
      item.addEventListener("mousemove", () => {
        if (state.active !== row) {
          state.active = row;
          highlight();
        }
      });
      item.addEventListener("click", () => choose(place));
      list.append(item);
    });
    const empty = (text) => element("li", { class: "fui-combobox-empty", role: "option", "aria-disabled": "true", "aria-selected": "false", text });
    if (places.length === 0) list.append(empty(root.dataset.fuiNoMatch || ""));
    if (more) list.append(empty(root.dataset.fuiMore || ""));
    const active = state.open && state.active !== null ? places[state.active] : undefined;
    if (active === undefined) input.removeAttribute("aria-activedescendant");
    else {
      input.setAttribute("aria-activedescendant", id + "-option-" + active);
      const row = list.children[state.active];
      if (row?.scrollIntoView) row.scrollIntoView({ block: "nearest" });
    }
  }
  // Moves the highlight among the rows drawn, keeping every row.
  function highlight() {
    const { places } = view();
    [...list.children].forEach((item, row) => {
      if (item.getAttribute("role") !== "option" || item.hasAttribute("aria-disabled")) return;
      if (state.active === row) item.dataset.active = "true";
      else delete item.dataset.active;
    });
    const active = state.open && state.active !== null ? places[state.active] : undefined;
    if (active === undefined) input.removeAttribute("aria-activedescendant");
    else {
      input.setAttribute("aria-activedescendant", id + "-option-" + active);
      const row = list.children[state.active];
      if (row?.scrollIntoView) row.scrollIntoView({ block: "nearest" });
    }
  }
  function choose(place) {
    const option = options[place];
    if (option) {
      state.selected = option.value;
      state.chosenLabel = option.label;
    }
    close();
    // The hidden input says what was chosen, as a select's change does.
    if (option) {
      hidden.toggleAttribute("data-other", option.other);
      hidden.dispatchEvent(new Event("change", { bubbles: true }));
    }
  }
  function close() {
    state.open = false;
    state.typing = false;
    if (!remote) state.query = "";
    draw();
  }
  function openAt(place) {
    state.open = true;
    const { places } = view();
    const at = place === null ? -1 : places.indexOf(place);
    const first = places.length ? 0 : null;
    state.active = at >= 0 ? at : first;
    draw();
  }
  function chosenPlace() {
    const at = options.findIndex((option) => option.value === state.selected);
    return at >= 0 ? at : null;
  }
  input.addEventListener("click", () => (state.open ? close() : openAt(chosenPlace())));
  input.addEventListener("input", () => {
    const typed = input.value;
    if (typed.trim() === "") {
      state.selected = null;
      state.chosenLabel = null;
    }
    state.query = typed;
    state.typing = true;
    state.open = true;
    state.active = view().places.length ? 0 : null;
    draw();
  });
  // Enter chooses the highlighted option; over a remote with none it
  // submits the typed words.
  function enter(event, places) {
    const place = state.active === null ? undefined : places[state.active];
    if (place === undefined && remote) return;
    event.preventDefault();
    if (place !== undefined) choose(place);
  }
  input.addEventListener("keydown", (event) => {
    const key = event.key;
    if (!state.open) {
      if (!ARROWS.has(key)) return;
      event.preventDefault();
      openAt(chosenPlace());
      return;
    }
    const { places } = view();
    const next = stepped(key, state.active, places.length);
    if (next !== undefined) {
      event.preventDefault();
      state.active = next;
      highlight();
    } else if (key === "Enter") enter(event, places);
    else if (key === "Escape") {
      event.preventDefault();
      close();
    }
  });
  input.addEventListener("blur", close);
  root.addEventListener("fui:choose", (event) => {
    const value = event.detail ? event.detail.value : "";
    const option = options.find((option) => option.value === value && !option.other);
    state.selected = option ? option.value : null;
    state.chosenLabel = option ? option.label : null;
    delete hidden.dataset.other;
    close();
  });
  draw();
}
