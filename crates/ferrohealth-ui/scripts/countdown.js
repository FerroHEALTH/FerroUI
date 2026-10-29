// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

// The commit-confirm countdown (F2). The server draws the seconds left;
// this counts down from there by the time elapsed since the page loaded,
// the larger of the monotonic and the wall clock, so a sleep the monotonic
// clock missed still counts and a clock set back adds nothing, and says the
// time left in the polite live region at each set point and at the end.
export const ANNOUNCE_AT = [60, 30, 10];
const RING = 81.68;
const text = (word) => (word && typeof word === "object" ? word.en : word || "");

export function clock(seconds) {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = String(seconds % 60).padStart(2, "0");
  return hours > 0 ? hours + ":" + String(minutes).padStart(2, "0") + ":" + rest : minutes + ":" + rest;
}

export function ringOffset(left, total) {
  return (RING * (1 - Math.min(left, total) / Math.max(total, 1))).toFixed(2);
}

export function crossed(before, now) {
  for (let at = ANNOUNCE_AT.length - 1; at >= 0; at--) {
    if (now <= ANNOUNCE_AT[at] && ANNOUNCE_AT[at] < before) return at;
  }
  return null;
}

export function elapsed(monotonic, wall) {
  return Math.max(Number.isFinite(monotonic) ? monotonic : 0, Number.isFinite(wall) ? wall : 0);
}

export function enhance(root) {
  const start = Number(root.getAttribute("data-left") || "0");
  const total = Math.max(Number(root.getAttribute("data-total") || "0"), start);
  if (!start) return;
  const words = JSON.parse(root.getAttribute("data-words") || "{}");
  const ring = root.querySelector(".fui-countdown-left");
  const strong = root.querySelector("[role='timer'] strong");
  const line = root.querySelector("[role='timer'] span");
  const said = root.querySelector("[role='status']");
  const began = performance.now();
  const beganWall = Date.now();
  let left = start;
  const timer = setInterval(() => {
    const spent = elapsed(performance.now() - began, Date.now() - beganWall);
    const now = Math.max(0, start - Math.floor(spent / 1000));
    if (now === left) return;
    const before = left;
    left = now;
    if (ring) ring.setAttribute("stroke-dashoffset", ringOffset(left, total));
    if (strong) strong.textContent = clock(left);
    if (left === 0) {
      if (line) line.textContent = text(words.ended);
      if (said) said.textContent = text(words.up);
      clearInterval(timer);
      return;
    }
    const at = crossed(before, left);
    if (at !== null && said && words.said) said.textContent = text(words.said[at]);
  }, 250);
}
