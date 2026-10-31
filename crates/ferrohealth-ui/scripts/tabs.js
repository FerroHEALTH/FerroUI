// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The tabs (R3). Binder tabs are links to routes; a tab whose panel the
// page already holds (data-fui-panel naming it) switches in place: the panel
// shows, the others hide, the tab is marked current and the address follows
// with replaceState, so no document loads and nothing on the page moves.
// From 40 rem the keys 1 to 9, pressed outside a field, follow the page's
// first binder's tabs. Keyboard tabs (role=tablist) switch their panels with
// the arrows, Home and End (WAI-ARIA Authoring Practices, the tabs pattern,
// automatic activation). The phone's tab select follows the chosen route.
export function tabAt(key, count) {
  if (!/^[1-9]$/.test(key)) return null;
  const at = Number(key) - 1;
  return at < count ? at : null;
}

export function moved(key, current, count) {
  if (count === 0) return null;
  const last = count - 1;
  if (key === "ArrowRight") return current >= last ? 0 : current + 1;
  if (key === "ArrowLeft") return current <= 0 ? last : current - 1;
  if (key === "Home") return 0;
  if (key === "End") return last;
  return null;
}

export function typing(event) {
  const t = event.target;
  if (!(t instanceof HTMLElement)) return false;
  if (t.isContentEditable) return true;
  const tag = t.tagName.toLowerCase();
  if (tag === "textarea" || tag === "select") return true;
  if (tag !== "input") return false;
  const kind = (t.getAttribute("type") || "").toLowerCase();
  return !["checkbox", "radio", "button", "submit", "reset", "range", "color", "file", "image"].includes(kind);
}

export const phone = () => window.matchMedia("(width < 40rem)").matches;
let keysBound = false;

function bindDigits() {
  if (keysBound) return;
  keysBound = true;
  window.addEventListener("keydown", (event) => {
    if (event.defaultPrevented || event.ctrlKey || event.metaKey || event.altKey) return;
    if (typing(event) || phone() || document.querySelector("dialog[open]")) return;
    const nav = document.querySelector("nav[data-fui-tab-keys]");
    if (!nav) return;
    const tabs = [...nav.querySelectorAll("a[href]")];
    const at = tabAt(event.key, tabs.length);
    if (at === null) return;
    event.preventDefault();
    if (tabs[at].getAttribute("aria-current") !== "page") tabs[at].click();
  });
}

function linkTabs(nav) {
  nav.addEventListener("click", (event) => {
    const tab = event.target.closest("a.fui-tab[data-fui-panel]");
    if (!tab || event.ctrlKey || event.metaKey || event.shiftKey || event.button) return;
    const panel = document.getElementById(tab.getAttribute("data-fui-panel"));
    if (!panel) return;
    event.preventDefault();
    for (const other of nav.querySelectorAll("a.fui-tab")) {
      const shown = other === tab;
      if (shown) other.setAttribute("aria-current", "page");
      else other.removeAttribute("aria-current");
      const id = other.getAttribute("data-fui-panel");
      const each = id && document.getElementById(id);
      if (each) each.hidden = !shown;
    }
    history.replaceState(history.state, "", tab.getAttribute("href"));
  });
}

function keyboardTabs(list) {
  const tabs = [...list.querySelectorAll("[role='tab']")];
  const panel = (tab) => document.getElementById(tab.getAttribute("aria-controls"));
  const select = (at) => {
    tabs.forEach((tab, index) => {
      const on = index === at;
      tab.setAttribute("aria-selected", on ? "true" : "false");
      tab.tabIndex = on ? 0 : -1;
      const p = panel(tab);
      if (p) p.hidden = !on;
    });
  };
  tabs.forEach((tab, index) => tab.addEventListener("click", () => select(index)));
  list.addEventListener("keydown", (event) => {
    const current = tabs.indexOf(document.activeElement);
    const next = moved(event.key, current, tabs.length);
    if (next === null) return;
    event.preventDefault();
    select(next);
    tabs[next].focus();
  });
}

function tabSelect(form) {
  const select = form.querySelector("select");
  if (!select) return;
  select.disabled = false;
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    window.location.assign(select.value);
  });
}

export function enhance(element) {
  bindDigits();
  if (element.matches("nav.fui-tabs")) linkTabs(element);
  else if (element.matches("[role='tablist']")) keyboardTabs(element);
  else if (element.matches("form.fui-tabs-select")) tabSelect(element);
}
