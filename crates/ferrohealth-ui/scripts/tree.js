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

// The move a key makes from the item at index `at` of the items `shown`
// ({key, parent, expandable, open, link}), as the crate's moved.
export function moved(key, at, shown) {
  const item = shown[at];
  if (!item) return null;
  const focus = (other) => (other ? { focus: other.key } : null);
  switch (key) {
    case "ArrowDown":
      return focus(shown[at + 1]);
    case "ArrowUp":
      return at > 0 ? focus(shown[at - 1]) : null;
    case "Home":
      return focus(shown[0]);
    case "End":
      return focus(shown[shown.length - 1]);
    case "ArrowRight":
      if (item.expandable && !item.open) return { open: item.key };
      if (item.open) {
        const next = shown[at + 1];
        return next && next.parent === item.key ? { focus: next.key } : null;
      }
      return null;
    case "ArrowLeft":
      if (item.open) return { close: item.key };
      return item.parent === null ? null : { focus: item.parent };
    case "Enter":
    case " ":
      if (item.link) return { follow: item.key };
      if (item.open) return { close: item.key };
      if (item.expandable) return { open: item.key };
      return null;
    default:
      return null;
  }
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
  const parentOf = (item) => item.parentElement.closest("li[role='treeitem']");
  const toggleOf = (item) => item.querySelector(":scope > .fui-tree-row > .fui-tree-toggle");
  const isOpen = (item) => item.getAttribute("aria-expanded") === "true";
  const byKey = (key) => items().find((item) => item.getAttribute("data-key") === key);

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
    const name = fields.getAttribute("data-fui-open-fields");
    fields.replaceChildren(
      ...items()
        .filter(isOpen)
        .map((item) => {
          const input = document.createElement("input");
          input.type = "hidden";
          input.name = name;
          input.value = item.getAttribute("data-key");
          return input;
        }),
    );
  }
  function mark(item, open) {
    item.setAttribute("aria-expanded", String(open));
    const toggle = toggleOf(item);
    if (!toggle) return;
    toggle.setAttribute("aria-label", toggle.getAttribute(open ? "data-fui-collapse" : "data-fui-expand") || "");
    const glyph = icons && icons.content.querySelector("[data-icon='" + (open ? "chevron-down" : "chevron-right") + "'] svg");
    const old = toggle.querySelector("svg");
    if (glyph && old) old.replaceWith(glyph.cloneNode(true));
  }
  function inSight() {
    return items()
      .filter((item) => !item.parentElement.closest("ul[role='group'][hidden]"))
      .map((item) => {
        const parent = parentOf(item);
        return {
          key: item.getAttribute("data-key"),
          parent: parent && own(parent) ? parent.getAttribute("data-key") : null,
          expandable: item.hasAttribute("aria-expanded"),
          open: isOpen(item),
          link: Boolean(item.querySelector(":scope > .fui-tree-row > a.fui-tree-label")),
        };
      });
  }
  function close(item) {
    const group = groupOf(item);
    if (group) {
      // The tab stop never hides in a closed level: it moves to the item.
      if (group.querySelector("li[tabindex='0']")) for (const each of items()) each.tabIndex = each === item ? 0 : -1;
      group.hidden = true;
    }
    mark(item, false);
    keep(item.getAttribute("data-key"), false);
    sync();
  }
  // Whether the item opened in place; an item the server did not draw open
  // follows its toggle's link, its level shown loading meanwhile.
  function open(item) {
    const group = groupOf(item);
    if (group) {
      group.hidden = false;
      mark(item, true);
      keep(item.getAttribute("data-key"), true);
      sync();
      return true;
    }
    const toggle = toggleOf(item);
    if (toggle && toggle.tagName === "A") {
      const loading = document.createElement("ul");
      loading.setAttribute("role", "group");
      loading.className = "fui-tree-group";
      const row = document.createElement("li");
      row.setAttribute("role", "none");
      row.className = "fui-tree-loading";
      const ring = document.createElement("span");
      ring.className = "fui-spinner-ring";
      ring.setAttribute("aria-hidden", "true");
      row.append(ring, root.getAttribute("data-fui-reading") || "");
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
    if (!item || !item.hasAttribute("aria-expanded") || isOpen(item) === isOpenNow) continue;
    if (isOpenNow) {
      if (groupOf(item)) open(item);
    } else close(item);
  }
  for (const toggle of root.querySelectorAll("button.fui-tree-toggle")) toggle.disabled = false;
  let waiting = null;
  try {
    waiting = sessionStorage.getItem(focusKey);
    sessionStorage.removeItem(focusKey);
  } catch (_) {
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
    const shown = inSight();
    const at = shown.findIndex((each) => each.key === item.getAttribute("data-key"));
    const step = moved(event.key, at, shown);
    if (!step) return;
    event.preventDefault();
    if (step.focus) focusOn(byKey(step.focus));
    else if (step.close) close(byKey(step.close));
    else if (step.open) {
      const target = byKey(step.open);
      const toggle = toggleOf(target);
      if (groupOf(target)) open(target);
      else if (toggle && toggle.tagName === "A") {
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
