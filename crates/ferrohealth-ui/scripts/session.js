// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The end of a console session, in the page. The server draws the warning
// closed with the seconds the session had left; this asks the console's
// session route when the time is up, when the tab is shown again or takes
// the focus, and on the user's own input once a minute (only that input
// extends the session, a POST), opens the warning before the idle timeout,
// leaves for sign-in with this page as the return address once the
// session has ended, and tells every other tab over a BroadcastChannel.
// The plan, the messages and the address are the crate's plan, Message and
// sign_in_address.
import { clock, elapsed } from "./countdown.js";
import { fill } from "./shared.js";

export const WARN_FLOOR = 20;
export const ACTIVITY_GAP = 60000;
export const CHANNEL = "ferroui-session";
let away = false;

export function plan(left, warnAt) {
  const warn = Math.max(warnAt, WARN_FLOOR);
  if (left.absolute < left.idle) return { warnIn: null, endIn: left.absolute };
  return { warnIn: Math.max(left.idle - warn, 0), endIn: left.idle };
}

export function after(left, seconds) {
  return { idle: Math.max(left.idle - seconds, 0), absolute: Math.max(left.absolute - seconds, 0) };
}

export function messageText(message) {
  return message.ended ? "ended" : "extended " + message.left.idle + " " + message.left.absolute;
}

export function parseMessage(text) {
  const words = String(text).split(" ");
  const whole = (word) => (/^\d+$/.test(word || "") ? Number(word) : null);
  if (words[0] === "ended" && words.length === 1) return { ended: true };
  if (words[0] === "extended" && words.length === 3) {
    const idle = whole(words[1]);
    const absolute = whole(words[2]);
    if (idle !== null && absolute !== null) return { left: { idle, absolute } };
  }
  return null;
}

export function signInAddress(signIn, here) {
  const joint = signIn.includes("?") ? "&" : "?";
  let out = signIn + joint + "return=";
  for (const byte of new TextEncoder().encode(here)) {
    const c = String.fromCharCode(byte);
    out += /[A-Za-z0-9\-._~/]/.test(c) ? c : "%" + byte.toString(16).toUpperCase().padStart(2, "0");
  }
  return out;
}

function tell(message) {
  try {
    const channel = new BroadcastChannel(CHANNEL);
    channel.postMessage(messageText(message));
    channel.close();
  } catch (_) {
    // A browser with no channel ends this tab alone.
  }
}

// Says to every tab of the console that the session has ended.
export function gone() {
  tell({ ended: true });
}

// Says so to every other tab and loads `address` in this one, whose own
// watch then stays put.
export function goneTo(address) {
  away = true;
  gone();
  window.location.assign(address);
}

export function enhance(root) {
  if (root.hasAttribute("data-fui-ended")) {
    gone();
    return;
  }
  const status = root.getAttribute("data-fui-status");
  const signIn = root.getAttribute("data-fui-sign-in");
  const warnAt = Number(root.getAttribute("data-fui-warn-at") || "120");
  const endsPhrase = JSON.parse(root.getAttribute("data-fui-ends") || '""');
  const ends = root.querySelector("#" + CSS.escape(root.id) + "-ends");
  let known = { left: { idle: Number(root.getAttribute("data-fui-idle")), absolute: Number(root.getAttribute("data-fui-absolute")) }, at: performance.now(), wall: Date.now() };
  let timers = [];
  let ticker = null;
  let asking = false;
  let leaving = false;

  const now = () => {
    const spent = elapsed(performance.now() - known.at, Date.now() - known.wall);
    return { left: after(known.left, Math.floor(spent / 1000)), spent };
  };
  const clear = () => {
    for (const timer of timers) clearTimeout(timer);
    timers = [];
    if (ticker !== null) clearInterval(ticker);
    ticker = null;
  };
  const say = (seconds) => {
    if (!ends) return;
    const phrase = typeof endsPhrase === "object" && endsPhrase ? endsPhrase.en : endsPhrase;
    ends.textContent = fill(String(phrase), { time: clock(seconds) });
  };
  function leave(tellOthers) {
    if (leaving) return;
    leaving = true;
    clear();
    if (tellOthers) tell({ ended: true });
    window.location.assign(signInAddress(signIn, location.pathname + location.search));
  }
  function learn(left) {
    known = { left, at: performance.now(), wall: Date.now() };
    schedule();
  }
  function warn() {
    const tick = () => say(Math.min(now().left.idle, now().left.absolute));
    tick();
    ticker = setInterval(tick, 250);
    if (!root.open) root.showModal();
  }
  function schedule() {
    if (leaving) return;
    clear();
    const next = plan(now().left, warnAt);
    if (next.endIn === 0) return leave(true);
    if (next.warnIn === 0) warn();
    else {
      if (root.open) root.close();
      if (next.warnIn !== null) timers.push(setTimeout(() => ask(false), next.warnIn * 1000));
    }
    timers.push(setTimeout(() => ask(false), next.endIn * 1000));
  }
  async function ask(extend) {
    if (leaving || (extend && asking)) return;
    if (extend) asking = true;
    let answer = null;
    try {
      const response = await fetch(status, { method: extend ? "POST" : "GET", credentials: "same-origin" });
      if (response.status === 401) answer = "gone";
      else if (response.ok) answer = await response.json();
      else console.warn("fui: the session route answered " + response.status);
    } catch (error) {
      console.warn("fui: the session route did not answer", error);
    }
    if (extend) asking = false;
    if (answer === "gone") leave(true);
    else if (answer && Number.isInteger(answer.idle) && Number.isInteger(answer.absolute)) {
      learn({ idle: answer.idle, absolute: answer.absolute });
      if (extend) tell({ left: known.left });
    } else schedule();
  }

  for (const kind of ["visibilitychange", "focus", "pageshow"]) {
    window.addEventListener(kind, () => {
      if (!document.hidden) ask(false);
    });
  }
  for (const kind of ["keydown", "pointerdown"]) {
    window.addEventListener(kind, () => {
      if (!root.open && now().spent >= ACTIVITY_GAP) ask(true);
    });
  }
  try {
    new BroadcastChannel(CHANNEL).onmessage = (event) => {
      const message = parseMessage(event.data);
      if (!message) return;
      if (message.ended) {
        if (!away) leave(false);
      } else learn(message.left);
    };
  } catch (_) {
    // No channel: this tab watches alone.
  }
  root.addEventListener("cancel", (event) => {
    event.preventDefault();
    ask(true);
  });
  const stay = root.querySelector("[data-fui-stay]");
  if (stay) stay.addEventListener("click", () => ask(true));
  schedule();
}
