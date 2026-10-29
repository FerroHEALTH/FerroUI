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

function make(tag, props, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props || {})) {
    if (value === null || value === undefined || value === false) continue;
    if (key === "class") node.className = value;
    else if (key === "text") node.textContent = value;
    else node.setAttribute(key, value === true ? "" : value);
  }
  for (const child of children) if (child) node.append(child);
  return node;
}

export function enhance(root) {
  const select = root.querySelector("select");
  if (!select) return;
  const id = select.id;
  const name = root.getAttribute("data-fui-name") || select.name;
  const remote = root.getAttribute("data-fui-remote");
  const options = [...select.options]
    .filter((option) => option.value !== "")
    .map((option) => ({ value: option.value, label: option.textContent, detail: option.getAttribute("data-detail") }));
  const chosenAt = options.findIndex((option) => option.value === select.value);
  const check = root.querySelector("template[data-fui-icons]");
  const state = {
    selected: chosenAt >= 0 ? options[chosenAt].value : null,
    chosenLabel: chosenAt >= 0 ? options[chosenAt].label : null,
    open: false,
    typing: false,
    query: remote ? root.getAttribute("data-fui-query") || "" : "",
    active: null,
  };
  const listId = id + "-list";
  const input = make("input", {
    id,
    type: "text",
    role: "combobox",
    class: "fui-input fui-combobox-input",
    autocomplete: "off",
    "aria-autocomplete": "list",
    "aria-expanded": "false",
    "aria-controls": listId,
    "aria-describedby": select.getAttribute("aria-describedby"),
    placeholder: root.getAttribute("data-fui-placeholder") || "",
    name: remote,
  });
  const hidden = make("input", { type: "hidden", name });
  const list = make("ul", { id: listId, role: "listbox", "aria-label": root.querySelector("label").textContent, class: "fui-combobox-list", hidden: true });
  const box = make("div", { class: "fui-combobox-box" }, input, hidden, list);
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
      const item = make("li", {
        id: id + "-option-" + place,
        role: "option",
        class: "fui-combobox-option",
        "aria-selected": String(chosen),
        "data-active": state.active === row ? "true" : null,
      },
      make("span", { class: "fui-combobox-text" }, make("span", { class: "fui-combobox-label", text: option.label }), option.detail ? make("span", { class: "fui-combobox-detail", text: option.detail }) : null),
      make("span", { class: "fui-combobox-check", "aria-hidden": "true" }));
      if (chosen && check) {
        const svg = check.content.querySelector("svg");
        if (svg) item.lastChild.append(svg.cloneNode(true));
      }
      item.addEventListener("mousedown", (event) => event.preventDefault());
      item.addEventListener("mousemove", () => {
        if (state.active !== row) {
          state.active = row;
          draw();
        }
      });
      item.addEventListener("click", () => choose(place));
      list.append(item);
    });
    const empty = (text) => make("li", { class: "fui-combobox-empty", role: "option", "aria-disabled": "true", "aria-selected": "false", text });
    if (places.length === 0) list.append(empty(root.getAttribute("data-fui-no-match") || ""));
    if (more) list.append(empty(root.getAttribute("data-fui-more") || ""));
    const active = state.open && state.active !== null ? places[state.active] : undefined;
    if (active === undefined) input.removeAttribute("aria-activedescendant");
    else {
      input.setAttribute("aria-activedescendant", id + "-option-" + active);
      const row = list.children[state.active];
      if (row && row.scrollIntoView) row.scrollIntoView({ block: "nearest" });
    }
  }
  function choose(place) {
    const option = options[place];
    if (option) {
      state.selected = option.value;
      state.chosenLabel = option.label;
    }
    close();
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
    state.active = at >= 0 ? at : places.length ? 0 : null;
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
  input.addEventListener("keydown", (event) => {
    const { places } = view();
    const key = event.key;
    if (key === "ArrowDown" || key === "ArrowUp") {
      event.preventDefault();
      if (!state.open) return openAt(chosenPlace());
      if (places.length === 0) state.active = null;
      else if (state.active === null) state.active = 0;
      else state.active = key === "ArrowDown" ? Math.min(state.active + 1, places.length - 1) : Math.max(state.active - 1, 0);
      return draw();
    }
    if (!state.open) return;
    if (key === "Home") {
      event.preventDefault();
      state.active = places.length ? 0 : null;
      draw();
    } else if (key === "End") {
      event.preventDefault();
      state.active = places.length ? places.length - 1 : null;
      draw();
    } else if (key === "Enter") {
      const place = state.active === null ? undefined : places[state.active];
      if (place === undefined && remote) return;
      event.preventDefault();
      if (place !== undefined) choose(place);
    } else if (key === "Escape") {
      event.preventDefault();
      close();
    }
  });
  input.addEventListener("blur", close);
  draw();
}
