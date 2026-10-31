// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The topology (F9): zoom by the cluster and the keys, panning by the
// pointer, the quick look on "i" or Space, and the arrows along the List
// twin. The server draws every card as a link, every fold as a link and
// the Status menu as a form; this enables the zoom cluster, which arrives
// disabled, and writes the zoom as the drawing's width and height
// attributes, never a style. The steps, keys and moves are the crate's
// stepped, asked and moved.
import { fill as filled, text } from "./shared.js";
const english = (word) => Boolean(word && typeof word === "object");
const fill = (phrase, values) => (english(phrase) ? { en: filled(phrase, values) } : filled(phrase, values));

export const STEPS = [0.25, 0.33, 0.5, 0.67, 0.8, 1.0, 1.25, 1.5, 2.0];

export function stepped(scale, inward) {
  const next = inward ? STEPS.find((step) => step > scale + 0.005) : [...STEPS].reverse().find((step) => step < scale - 0.005);
  if (next !== undefined) return next;
  return inward ? STEPS[STEPS.length - 1] : STEPS[0];
}

export function asked(key, onCard) {
  if (key === "+" || key === "=") return "in";
  if (key === "-" || key === "_") return "out";
  if (key === "0") return "fit";
  if ((key === "i" || key === " ") && onCard) return "look";
  return null;
}

export function moved(key, at, count) {
  if (count === 0) return null;
  const last = count - 1;
  if (key === "ArrowDown") return Math.min(at + 1, last);
  if (key === "ArrowUp") return Math.max(at - 1, 0);
  if (key === "Home") return 0;
  if (key === "End") return last;
  return null;
}

export function enhance(root) {
  const words = JSON.parse(root.getAttribute("data-fui-words") || "{}");
  const stage = root.querySelector(".fui-pipeline-stage");
  const scroll = root.querySelector(".fui-topology-scroll");
  let scale = null;
  let drag = null;
  const size = (name) => Number(stage && stage.getAttribute(name)) || 1;
  const drawn = () => (stage ? stage.querySelector(":scope > svg") : null);
  const shownScale = () => {
    const svg = drawn();
    return svg ? Math.max(svg.clientWidth, 1) / size("data-width") : 1;
  };

  function frame() {
    const view = root.querySelector(".fui-minimap-view");
    if (!view || !scroll || !stage) return;
    const mini = Number(view.getAttribute("data-scale")) || 1;
    const shown = shownScale();
    const at = {
      x: scroll.scrollLeft / shown,
      y: scroll.scrollTop / shown,
      width: Math.min(scroll.clientWidth / shown, size("data-width")),
      height: Math.min(scroll.clientHeight / shown, size("data-height")),
    };
    for (const [name, value] of Object.entries(at)) view.setAttribute(name, (value * mini).toFixed(1));
  }

  function zoom(ask) {
    if (!stage) return;
    const width = size("data-width");
    const height = size("data-height");
    const now = scale === null ? shownScale() : scale;
    scale = ask === "in" ? stepped(now, true) : ask === "out" ? stepped(now, false) : null;
    stage.classList.toggle("is-zoomed", scale !== null);
    for (const svg of stage.querySelectorAll(":scope > svg")) {
      const shown = scale === null ? 1 : scale;
      svg.setAttribute("width", (width * shown).toFixed(0));
      svg.setAttribute("height", (height * shown).toFixed(0));
    }
    const percent = ((scale === null ? shownScale() : scale) * 100).toFixed(0);
    const readout = root.querySelector("[data-zoom-readout]");
    if (readout) {
      const said = fill(words.readout, { value: percent });
      readout.textContent = text(said);
      if (english(said)) readout.setAttribute("lang", "en");
      else readout.removeAttribute("lang");
      readout.setAttribute("aria-label", text(fill(words.level, { value: percent })));
    }
    frame();
  }

  function along(target, key) {
    const rows = [...root.querySelectorAll(".fui-topology-list [role='treeitem'] .fui-tree-name > a")];
    const at = rows.indexOf(target);
    if (at < 0) return false;
    let to;
    if (key === "ArrowLeft") {
      const level = (row) => Number(row.closest("[role='treeitem']")?.getAttribute("aria-level") || "1");
      const own = level(rows[at]);
      to = -1;
      for (let back = at - 1; back >= 0; back--) {
        if (level(rows[back]) < own) {
          to = back;
          break;
        }
      }
      if (to < 0) return false;
    } else {
      to = moved(key, at, rows.length);
      if (to === null) return false;
    }
    rows[to].focus();
    return true;
  }

  for (const button of root.querySelectorAll("[data-zoom]")) {
    const title = button.getAttribute("data-title");
    if (title) button.setAttribute("title", title);
    else button.removeAttribute("title");
    button.disabled = false;
  }
  zoom("fit");

  root.addEventListener("click", (event) => {
    const button = event.target.closest("[data-zoom]");
    if (!button || !root.contains(button)) return;
    const ask = button.getAttribute("data-zoom");
    if (ask === "in" || ask === "out" || ask === "fit") zoom(ask);
  });
  root.addEventListener("keydown", (event) => {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    const target = event.target;
    if (target.closest(".fui-topology-list")) {
      if (along(target, event.key)) event.preventDefault();
      return;
    }
    if (!target.closest(".fui-topology-scroll")) return;
    const onCard = target.matches(".fui-tnode-link");
    const ask = asked(event.key.toLowerCase(), onCard);
    if (ask === "look") {
      const link = target.parentElement && target.parentElement.querySelector(".fui-tnode-info a");
      if (link) {
        event.preventDefault();
        link.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
      }
    } else if (ask) {
      event.preventDefault();
      zoom(ask);
    }
  });
  root.addEventListener("mousedown", (event) => {
    if (event.button !== 0 || event.target.closest("a, button")) return;
    const held = event.target.closest(".fui-topology-scroll");
    if (held) drag = { x: event.clientX, y: event.clientY, left: held.scrollLeft, top: held.scrollTop };
  });
  root.addEventListener("mousemove", (event) => {
    if (!drag || !scroll) return;
    event.preventDefault();
    scroll.scrollLeft = drag.left - (event.clientX - drag.x);
    scroll.scrollTop = drag.top - (event.clientY - drag.y);
  });
  root.addEventListener("mouseup", () => (drag = null));
  root.addEventListener("mouseleave", () => (drag = null));
  root.addEventListener("scroll", frame, true);
}
