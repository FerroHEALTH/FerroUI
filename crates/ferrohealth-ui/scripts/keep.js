// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// Scroll kept across a navigation inside one page: a link or a form marked
// data-fui="keep" (a tab, a tree node, a sort header) stores the page's
// scroll position under the address it leads to, in the session alone, and
// the page restores it as it loads. A browser restores scroll on back and
// forward by itself; this covers the link forward (FerroUI #510).
const KEY = "fui-scroll:";

function store(href) {
  try {
    const url = new URL(href, location.href);
    sessionStorage.setItem(KEY + url.pathname + url.search, String(window.scrollY));
  } catch (_) {
    // A private window may refuse storage; the page then opens at the top.
  }
}

export function restore() {
  try {
    const key = KEY + location.pathname + location.search;
    const kept = sessionStorage.getItem(key);
    if (kept === null) return;
    sessionStorage.removeItem(key);
    window.scrollTo(0, Number(kept));
  } catch (_) {
    // Nothing kept, or storage refused.
  }
}

restore();

export function enhance(element) {
  element.addEventListener("click", (event) => {
    const link = event.target.closest("a[href]");
    if (link && element.contains(link)) store(link.href);
  });
  element.addEventListener("submit", (event) => {
    const form = event.target;
    if (form.method.toLowerCase() === "get") {
      const url = new URL(form.action || location.href, location.href);
      url.search = new URLSearchParams(new FormData(form, event.submitter)).toString();
      store(url.href);
    }
  });
}
