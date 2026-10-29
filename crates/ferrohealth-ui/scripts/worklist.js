// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The work-list (R9). With no script the table is a table. Here it becomes
// a grid with a roving tabindex (WAI-ARIA Authoring Practices 1.2, the grid
// pattern): one tab stop, the focused row's, marked aria-current; j and
// Down move to the next row, k and Up to the one before, Home and End to
// the ends, Enter follows the row's link; the focused row is scrolled clear
// of the sticky header and the foot (WCAG 2.2 SC 2.4.11). The header
// checkbox selects every row and shows the mixed state.
const CONTROLS = "a[href], button, input, select, textarea";

export function moved(key, current, count) {
  if (count === 0) return null;
  const last = count - 1;
  if (key === "j" || key === "ArrowDown") return Math.min(current + 1, last);
  if (key === "k" || key === "ArrowUp") return Math.min(Math.max(current - 1, 0), last);
  if (key === "Home") return 0;
  if (key === "End") return last;
  return null;
}

export function selection(checked, count) {
  if (checked === 0 || count === 0) return "none";
  return checked >= count ? "all" : "some";
}

export function enhance(root) {
  const rows = () => [...root.querySelectorAll("tbody tr[data-key]")];
  const table = root.querySelector("table");
  if (table) table.setAttribute("role", "grid");
  const region = root.querySelector(".fui-table-scroll");
  if (region) region.tabIndex = -1;
  function apply(index) {
    rows().forEach((row, at) => {
      const on = at === index;
      row.tabIndex = on ? 0 : -1;
      if (on) row.setAttribute("aria-current", "true");
      else row.removeAttribute("aria-current");
      for (const control of row.querySelectorAll(CONTROLS)) {
        if (on) control.removeAttribute("tabindex");
        else control.tabIndex = -1;
      }
    });
  }
  const header = root.querySelector("thead .fui-select-all");
  const checks = () => [...root.querySelectorAll("tbody td.fui-row-check input[type=checkbox]")];
  function sync() {
    const all = checks();
    if (!all.length) return;
    for (const input of all) {
      const row = input.closest("tr");
      if (row) row.setAttribute("aria-selected", input.checked ? "true" : "false");
    }
    if (header) {
      const state = selection(all.filter((input) => input.checked).length, all.length);
      header.checked = state === "all";
      header.indeterminate = state === "some";
    }
  }
  const start = rows().findIndex((row) => row.hasAttribute("aria-current"));
  apply(start < 0 ? 0 : start);
  if (header) header.hidden = false;
  sync();
  root.addEventListener("change", (event) => {
    if (event.target === header) for (const input of checks()) input.checked = header.checked;
    sync();
  });
  root.addEventListener("focusin", (event) => {
    const row = event.target.closest("tbody tr[data-key]");
    if (!row || row.hasAttribute("aria-current")) return;
    apply(rows().indexOf(row));
  });
  root.addEventListener("keydown", (event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return;
    const target = event.target;
    if (!(target instanceof HTMLElement) || target.tagName !== "TR") return;
    const all = rows();
    const current = all.indexOf(target);
    if (current < 0) return;
    if (event.key === "Enter") {
      event.preventDefault();
      const link = target.querySelector("a.fui-row-link");
      if (link) link.click();
      return;
    }
    const next = moved(event.key, current, all.length);
    if (next === null) return;
    event.preventDefault();
    apply(next);
    const row = all[next];
    row.focus();
    const rect = row.getBoundingClientRect();
    const head = root.querySelector("thead");
    const top = head ? Math.max(head.getBoundingClientRect().bottom, 0) : 0;
    const card = root.closest(".fui-worklist");
    const foot = card && card.querySelector(".fui-worklist-foot");
    const bottom = foot ? Math.min(foot.getBoundingClientRect().top, innerHeight) : innerHeight;
    if (rect.top < top) scrollBy(0, rect.top - top);
    else if (rect.bottom > bottom) scrollBy(0, rect.bottom - bottom);
  });
}
