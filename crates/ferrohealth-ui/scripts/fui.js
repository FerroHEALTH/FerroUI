// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The loader: the one script a page loads. It marks the document, finds
// every element that asks for a module (data-fui="<job>"), imports the
// module from this file's own directory and hands the element to its
// enhance(element), and does the same for an element added later. Every
// page works without it; a module adds behaviour on top (FerroUI #510).
const HOOK = "data-fui";
const DONE = "data-fui-done";
const base = new URL(".", import.meta.url);
const loaded = new Map();

function moduleOf(job) {
  if (!loaded.has(job)) {
    loaded.set(job, import(new URL(job + ".js", base).href));
  }
  return loaded.get(job);
}

function enhance(element) {
  if (element.hasAttribute(DONE)) return;
  element.setAttribute(DONE, "");
  const jobs = element.getAttribute(HOOK).split(/\s+/).filter(Boolean);
  for (const job of jobs) {
    moduleOf(job)
      .then((module) => module.enhance(element))
      .catch((error) => console.error("fui: " + job + " refused", error));
  }
}

function sweep(root) {
  if (root.nodeType !== 1) return;
  if (root.hasAttribute(HOOK)) enhance(root);
  for (const element of root.querySelectorAll("[" + HOOK + "]")) enhance(element);
}

document.documentElement.setAttribute("data-fui-script", "");
sweep(document.documentElement);
new MutationObserver((records) => {
  for (const record of records) {
    for (const node of record.addedNodes) sweep(node);
  }
}).observe(document.documentElement, { childList: true, subtree: true });
