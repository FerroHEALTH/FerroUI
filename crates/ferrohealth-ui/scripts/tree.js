// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The tree view (WAI-ARIA Authoring Practices 1.2, tree view). The server
// draws the open items with their children and a closed level it read
// hidden; this folds a drawn level in place, walks the keys over the items
// in sight with one tab stop, keeps the open fields of a form in step and
// keeps the page's folds for the session. Opening a level not read yet
// follows the toggle's link, and by the keyboard the focus returns to its
// item on the page the link draws. The walk is the crate's in_sight and
// moved.
const KEY = "fui-tree:";
const FOCUS = "fui-tree-focus:";

const focusOn = (other) => (other ? { focus: other.key } : null);

// Right opens a closed item, and on an open one moves to its first child.
function inward(item, next) {
  if (item.expandable && !item.open) return { open: item.key };
  if (!item.open) return null;
  return next && next.parent === item.key ? { focus: next.key } : null;
}

// Left closes an open item, and on another moves to its parent.
function outward(item) {
  if (item.open) return { close: item.key };
  return item.parent === null ? null : { focus: item.parent };
}

// Enter and Space follow an item's link, or fold it.
function activated(item) {
  if (item.link) return { follow: item.key };
  if (item.open) return { close: item.key };
  return item.expandable ? { open: item.key } : null;
}

const MOVES = new Map([
  ["ArrowDown", (item, at, shown) => focusOn(shown[at + 1])],
  ["ArrowUp", (item, at, shown) => (at > 0 ? focusOn(shown[at - 1]) : null)],
  ["Home", (item, at, shown) => focusOn(shown[0])],
  ["End", (item, at, shown) => focusOn(shown[shown.length - 1])],
  ["ArrowRight", (item, at, shown) => inward(item, shown[at + 1])],
  ["ArrowLeft", outward],
  ["Enter", activated],
  [" ", activated],
]);

// The move a key makes from the item at index `at` of the items `shown`
// ({key, parent, expandable, open, link}), as the crate's moved.
export function moved(key, at, shown) {
  const item = shown[at];
  const move = MOVES.get(key);
  return item && move ? move(item, at, shown) : null;
}

// The items of the tree `list` in sight, in document order, as the
// crate's in_sight walks them.
export function inSight(list) {
  const own = (item) => item.closest("[role='tree']") === list;
  return [...list.querySelectorAll("li[role='treeitem']")]
    .filter((item) => own(item) && !item.parentElement.closest("ul[role='group'][hidden]"))
    .map((item) => {
      const parent = item.parentElement.closest("li[role='treeitem']");
      return {
        key: item.dataset.key,
        parent: parent && own(parent) ? parent.dataset.key : null,
        expandable: item.hasAttribute("aria-expanded"),
        open: item.getAttribute("aria-expanded") === "true",
        link: Boolean(item.querySelector(":scope > .fui-tree-row > a.fui-tree-label")),
      };
    });
}

export function enhance(root) {
  const list = root.querySelector("ul[role='tree']");
  if (!list) return;
  const fields = root.querySelector("[data-fui-open-fields]");
  const icons = root.querySelector("template[data-fui-icons]");
  const store = KEY + location.pathname + location.search + "#" + list.id;
  const focusKey = FOCUS + location.pathname + "#" + list.id;
  const own = (item) => item.closest("[role='tree']") === list;
  const items = () => [...list.querySelectorAll("li[role='treeitem']")].filter(own);
  const groupOf = (item) => item.querySelector(":scope > ul[role='group']");
  const toggleOf = (item) => item.querySelector(":scope > .fui-tree-row > .fui-tree-toggle");
  const isOpen = (item) => item.getAttribute("aria-expanded") === "true";
  const byKey = (key) => items().find((item) => item.dataset.key === key);

  function kept() {
    try {
      return JSON.parse(sessionStorage.getItem(store) || "{}");
    } catch (_) {
      return {};
    }
  }
  function keep(key, open) {
    try {
      const folds = kept();
      folds[key] = open;
      sessionStorage.setItem(store, JSON.stringify(folds));
    } catch (_) {
      // A private window may refuse storage; the folds then last the page.
    }
  }
  function sync() {
    if (!fields) return;
    const name = fields.dataset.fuiOpenFields;
    fields.replaceChildren(
      ...items()
        .filter(isOpen)
        .map((item) => {
          const input = document.createElement("input");
          input.type = "hidden";
          input.name = name;
          input.value = item.dataset.key;
          return input;
        }),
    );
  }
  function mark(item, open) {
    item.setAttribute("aria-expanded", String(open));
    const toggle = toggleOf(item);
    if (!toggle) return;
    toggle.setAttribute("aria-label", toggle.getAttribute(open ? "data-fui-collapse" : "data-fui-expand") || "");
    const glyph = icons?.content.querySelector("[data-icon='" + (open ? "chevron-down" : "chevron-right") + "'] svg");
    const old = toggle.querySelector("svg");
    if (glyph && old) old.replaceWith(glyph.cloneNode(true));
  }
  function close(item) {
    const group = groupOf(item);
    if (group) {
      // The tab stop never hides in a closed level: it moves to the item.
      if (group.querySelector("li[tabindex='0']")) for (const each of items()) each.tabIndex = each === item ? 0 : -1;
      group.hidden = true;
    }
    mark(item, false);
    keep(item.dataset.key, false);
    sync();
  }
  // Whether the item opened in place; an item the server did not draw open
  // follows its toggle's link, its level shown loading meanwhile.
  function open(item) {
    const group = groupOf(item);
    if (group) {
      group.hidden = false;
      mark(item, true);
      keep(item.dataset.key, true);
      sync();
      return true;
    }
    const toggle = toggleOf(item);
    if (toggle?.tagName === "A") {
      const loading = document.createElement("ul");
      loading.setAttribute("role", "group");
      loading.className = "fui-tree-group";
      const row = document.createElement("li");
      row.setAttribute("role", "none");
      row.className = "fui-tree-loading";
      const ring = document.createElement("span");
      ring.className = "fui-spinner-ring";
      ring.setAttribute("aria-hidden", "true");
      row.append(ring, root.dataset.fuiReading || "");
      loading.append(row);
      item.append(loading);
      mark(item, true);
      sync();
    }
    return false;
  }
  function focusOn(item) {
    for (const each of items()) each.tabIndex = each === item ? 0 : -1;
    item.focus();
  }

  for (const [key, isOpenNow] of Object.entries(kept())) {
    const item = byKey(key);
    if (!item?.hasAttribute("aria-expanded") || isOpen(item) === isOpenNow) continue;
    if (isOpenNow) {
      if (groupOf(item)) open(item);
    } else close(item);
  }
  for (const toggle of root.querySelectorAll("button.fui-tree-toggle")) toggle.disabled = false;
  let waiting = null;
  try {
    waiting = sessionStorage.getItem(focusKey);
    sessionStorage.removeItem(focusKey);
  } catch {
    // Storage refused (a private window, a blocked site): nothing waits.
    waiting = null;
  }
  const returned = waiting && byKey(waiting);
  if (returned) focusOn(returned);

  list.addEventListener("click", (event) => {
    const toggle = event.target.closest(".fui-tree-toggle");
    if (!toggle || !list.contains(toggle)) return;
    const item = toggle.closest("li[role='treeitem']");
    if (isOpen(item)) {
      event.preventDefault();
      close(item);
    } else if (open(item) || toggle.tagName !== "A") event.preventDefault();
  });
  list.addEventListener("focusin", (event) => {
    const item = event.target.closest("li[role='treeitem']");
    if (item && own(item) && event.target === item) for (const each of items()) each.tabIndex = each === item ? 0 : -1;
  });
  list.addEventListener("keydown", (event) => {
    const item = event.target;
    if (item.getAttribute("role") !== "treeitem" || !own(item)) return;
    const shown = inSight(list);
    const at = shown.findIndex((each) => each.key === item.dataset.key);
    const step = moved(event.key, at, shown);
    if (!step) return;
    event.preventDefault();
    if (step.focus) focusOn(byKey(step.focus));
    else if (step.close) close(byKey(step.close));
    else if (step.open) {
      const target = byKey(step.open);
      const toggle = toggleOf(target);
      if (groupOf(target)) open(target);
      else if (toggle?.tagName === "A") {
        // The level loads with the page its link draws; the focus waits there.
        try {
          sessionStorage.setItem(focusKey, step.open);
        } catch (_) {
          // Storage refused: the page opens with the focus at its top.
        }
        toggle.click();
      }
    } else if (step.follow) {
      const link = byKey(step.follow).querySelector(":scope > .fui-tree-row > a.fui-tree-label");
      if (link) link.click();
    }
  });
}
