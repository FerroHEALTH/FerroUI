// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The repeatable rows (F7). With no script Remove and Add post the form and
// the console answers with the list one row shorter or longer. Here both
// act in place: Remove takes its row out and moves the focus to the next
// row's first control (the previous row's at the end, Add when none is
// left); Add inserts a copy of the blank row, its key token replaced by a
// fresh key, and focuses its first control. A list in a row this module
// copied is no list it enhanced, so the list around it takes its presses.
const CONTROLS = "input:not([type=hidden]):not([disabled]),select:not([disabled]),textarea:not([disabled]),button:not([disabled])";
const live = new WeakSet();

export function freshKey(taken) {
  for (let count = 1; ; count++) {
    const key = "new-" + count;
    if (!taken(key)) return key;
  }
}

const listOf = (element) => element.closest(".fui-repeat");

function handlerOf(list) {
  let at = list;
  while (at && !live.has(at)) at = at.parentElement ? listOf(at.parentElement) : null;
  return at;
}

const own = (list, selector) => [...list.querySelectorAll(selector)].filter((element) => listOf(element) === list);

function focusRow(row) {
  const controls = [...row.querySelectorAll(CONTROLS)];
  const first = controls.find((control) => !control.closest(".fui-repeater-card-head")) || controls[0];
  if (first) first.focus();
  return Boolean(first);
}

function markEmpty(list) {
  const empty = own(list, "[data-repeat='empty']")[0];
  if (empty) empty.hidden = own(list, "[data-repeat='rows'] > .fui-repeat-item").length > 0;
}

function remove(list, button) {
  const item = button.closest(".fui-repeat-item");
  if (!item) return;
  let next = item.nextElementSibling;
  while (next && !next.matches(".fui-repeat-item")) next = next.nextElementSibling;
  let previous = item.previousElementSibling;
  while (previous && !previous.matches(".fui-repeat-item")) previous = previous.previousElementSibling;
  item.remove();
  markEmpty(list);
  const row = next || previous;
  if (!(row && focusRow(row))) {
    const add = own(list, "[data-repeat='add']")[0];
    if (add) add.focus();
  }
}

function add(list) {
  const blank = own(list, "fieldset[data-repeat='blank']")[0];
  const rows = own(list, "[data-repeat='rows']")[0];
  if (!blank || !rows) return false;
  const token = blank.getAttribute("data-token") || "__row__";
  const key = freshKey((key) => own(list, "[data-row='" + key + "']").length > 0);
  const copy = document.createDocumentFragment();
  for (const child of blank.cloneNode(true).childNodes) copy.append(child);
  for (const element of copy.querySelectorAll("*")) {
    for (const attribute of [...element.attributes]) {
      if (attribute.value.includes(token)) attribute.value = attribute.value.split(token).join(key);
    }
    element.removeAttribute("data-fui-done");
    if (element.matches(".fui-repeat")) element.removeAttribute("data-fui");
  }
  rows.append(copy);
  markEmpty(list);
  if (rows.lastElementChild) focusRow(rows.lastElementChild);
  return true;
}

export function enhance(root) {
  live.add(root);
  root.addEventListener("click", (event) => {
    const button = event.target.closest("[data-repeat]");
    if (!button) return;
    const list = listOf(button);
    if (!list || handlerOf(list) !== root) return;
    const kind = button.getAttribute("data-repeat");
    if (kind === "remove") {
      event.preventDefault();
      remove(list, button);
    } else if (kind === "add" && add(list)) event.preventDefault();
  });
}
