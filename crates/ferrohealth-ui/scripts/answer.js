// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// A form that answers on its page, answered in place (#546): the form posts
// with fetch to its own action, whose route draws the whole page with the
// answer in it, and the module moves the children of each answer region
// (the ids in data-fui-answer) from that page into the same region here.
// The region stays the element it was, a polite live region, so a screen
// reader hears the answer and the focus stays where it was. With
// data-fui-settle (milliseconds) the form also posts once a field rests
// from typing. An answer to an older request is dropped. An answer that is
// no 2xx HTML page holding every region, or none at all, and the form posts
// as it does with no script. The wire is in docs/consoles.md.
const KEPT = ["id", "aria-live", "aria-busy", "role"];

// The address the form posts to: its action, or the page's own address.
const address = (form) => new URL(form.getAttribute("action") || document.location.href, document.baseURI);

export function enhance(form) {
  const ids = (form.getAttribute("data-fui-answer") || "").split(/\s+/).filter(Boolean);
  const regions = ids.map((id) => document.getElementById(id));
  if (!ids.length || regions.some((region) => !region || region.contains(form)) || address(form).origin !== location.origin) return;
  for (const region of regions) {
    if (!region.hasAttribute("aria-live") && !/^(status|alert|log)$/.test(region.getAttribute("role") || "")) region.setAttribute("aria-live", "polite");
  }
  const settle = Number(form.getAttribute("data-fui-settle") || "0");
  const get = (form.getAttribute("method") || "get").toLowerCase() === "get";
  const multipart = form.getAttribute("enctype") === "multipart/form-data";
  let asked = 0;
  let pending = null;
  let plain = false;

  function busy(on) {
    for (const region of regions) {
      if (on) region.setAttribute("aria-busy", "true");
      else region.removeAttribute("aria-busy");
    }
  }
  function swap(page) {
    const fresh = ids.map((id) => page.getElementById(id));
    if (fresh.some((region) => !region)) return false;
    const focused = document.activeElement;
    const held = focused && focused.id && regions.some((region) => region.contains(focused)) ? focused.id : null;
    regions.forEach((region, at) => {
      const from = fresh[at];
      for (const { name } of [...region.attributes]) if (!KEPT.includes(name) && !from.hasAttribute(name)) region.removeAttribute(name);
      for (const { name, value } of [...from.attributes]) if (!KEPT.includes(name)) region.setAttribute(name, value);
      region.replaceChildren(...from.childNodes);
    });
    const back = held && document.getElementById(held);
    if (back) back.focus({ preventScroll: true });
    return true;
  }
  // `submitter` is the submit event's (null when none), or undefined for a
  // post while typing, which never falls back to a plain post of its own.
  async function send(submitter) {
    clearTimeout(pending);
    const mine = ++asked;
    const data = new FormData(form, submitter || null);
    const url = address(form);
    if (get) url.search = new URLSearchParams(data);
    busy(true);
    let page = null;
    try {
      const response = await fetch(url, {
        method: get ? "GET" : "POST",
        credentials: "same-origin",
        headers: { Accept: "text/html" },
        body: get ? undefined : multipart ? data : new URLSearchParams(data),
      });
      if (mine !== asked) return;
      if (response.redirected) return window.location.assign(response.url);
      const kind = response.headers.get("Content-Type") || "";
      if (response.ok && kind.startsWith("text/html")) page = new DOMParser().parseFromString(await response.text(), "text/html");
      else console.warn("fui: the answer is no page (" + response.status + " " + kind + ")");
    } catch (error) {
      console.warn("fui: the browser refused the answer's request", error);
    }
    if (mine !== asked) return;
    busy(false);
    if (page && swap(page)) return;
    plain = true;
    // A later task: an answer at once would still be inside the submit event,
    // where the browser ignores a new submission.
    if (submitter !== undefined) setTimeout(() => form.requestSubmit(submitter));
  }

  form.addEventListener("submit", (event) => {
    const by = event.submitter;
    if (plain || event.defaultPrevented || (by && (by.hasAttribute("formaction") || by.hasAttribute("formmethod")))) return;
    event.preventDefault();
    send(by);
  });
  if (settle > 0) {
    form.addEventListener("input", () => {
      if (plain) return;
      clearTimeout(pending);
      pending = setTimeout(() => send(undefined), settle);
    });
  }
}
