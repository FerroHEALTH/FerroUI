# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Headless Chrome over the DevTools protocol on --remote-debugging-pipe
(JSON records on fd 3 and 4), with no dependency beyond the standard
library. Every browser tool drives its pages through this."""
import fcntl
import json
import os
import time

import design_set


class Chrome:
    def __init__(self, profile, log_dir):
        child_read, parent_write = os.pipe()
        parent_read, child_write = os.pipe()
        # A dup2 onto the same number keeps close-on-exec, so the child's ends
        # move above 4 first and are then placed on 3 and 4.
        high = [fcntl.fcntl(fd, fcntl.F_DUPFD_CLOEXEC, 10) for fd in (child_read, child_write)]
        os.close(child_read)
        os.close(child_write)
        child_read, child_write = high
        os.makedirs(log_dir, exist_ok=True)
        log = os.open(os.path.join(log_dir, f"chrome-{os.getpid()}.log"), os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
        args = [
            design_set.chrome_binary(),
            "--headless=new",
            "--remote-debugging-pipe",
            "--no-first-run",
            "--no-default-browser-check",
            "--disable-gpu",
            "--hide-scrollbars",
            "--font-render-hinting=none",
            "--lang=en-CA",
            f"--user-data-dir={profile}",
            *(["--no-sandbox"] if os.environ.get("FERROUI_CHROME_NO_SANDBOX") == "1" else []),
            "about:blank",
        ]
        # The child reads on fd 3 and writes on fd 4 (what --remote-debugging-pipe
        # expects); the remapping happens in the child alone, so no fd of this
        # process moves.
        actions = [
            (os.POSIX_SPAWN_DUP2, child_read, 3),
            (os.POSIX_SPAWN_DUP2, child_write, 4),
            (os.POSIX_SPAWN_OPEN, 0, os.devnull, os.O_RDONLY, 0),
            (os.POSIX_SPAWN_OPEN, 1, os.devnull, os.O_WRONLY, 0),
            (os.POSIX_SPAWN_DUP2, log, 2),
        ]
        self.pid = os.posix_spawn(args[0], args, os.environ, file_actions=actions)
        for fd in (child_read, child_write, log):
            os.close(fd)
        self.w = os.fdopen(parent_write, "wb", buffering=0)
        self.r = os.fdopen(parent_read, "rb", buffering=0)
        self.buf = b""
        self.next_id = 0
        self.events = []

    def send(self, method, params=None, session=None):
        self.next_id += 1
        msg = {"id": self.next_id, "method": method, "params": params or {}}
        if session:
            msg["sessionId"] = session
        self.w.write(json.dumps(msg).encode() + b"\0")
        while True:
            rec = self.recv()
            if rec.get("id") == msg["id"]:
                if "error" in rec:
                    raise RuntimeError(f"{method}: {rec['error']}")
                return rec.get("result", {})
            self.events.append(rec)

    def recv(self):
        while b"\0" not in self.buf:
            chunk = self.r.read(65536)
            if not chunk:
                raise RuntimeError("chrome closed the pipe")
            self.buf += chunk
        rec, _, self.buf = self.buf.partition(b"\0")
        return json.loads(rec)

    def wait_event(self, method, session=None, timeout=20):
        deadline = time.time() + timeout
        for i, ev in enumerate(self.events):
            if ev.get("method") == method and (session is None or ev.get("sessionId") == session):
                return self.events.pop(i)
        while time.time() < deadline:
            rec = self.recv()
            if rec.get("method") == method and (session is None or rec.get("sessionId") == session):
                return rec
            self.events.append(rec)
        raise TimeoutError(method)

    def close(self):
        try:
            self.send("Browser.close")
        except Exception:
            pass
        os.waitpid(self.pid, 0)


def open_page(site, profile):
    """Starts Chrome with its own profile under the set's state directory and
    returns it with one attached page session, Page and Runtime enabled."""
    chrome = Chrome(os.path.join(site.state_dir, profile), site.state_dir)
    target = chrome.send("Target.createTarget", {"url": "about:blank"})["targetId"]
    session = chrome.send("Target.attachToTarget", {"targetId": target, "flatten": True})["sessionId"]
    chrome.send("Page.enable", {}, session)
    chrome.send("Runtime.enable", {}, session)
    return chrome, session


def load(chrome, session, url, width, height=900, theme=None, reduced_motion=False, still=False):
    """Navigates the session to url at a viewport and colour scheme, sets the
    page's data-theme as the console does, and waits for the load event.
    With still, a finite animation is run to its end and an endless one (a
    spinner) is paused at its start, so a page measures the same on every
    run."""
    chrome.send("Emulation.setDeviceMetricsOverride", {"width": width, "height": height, "deviceScaleFactor": 1, "mobile": width < 600}, session)
    features = []
    if theme:
        features.append({"name": "prefers-color-scheme", "value": theme})
    if reduced_motion:
        features.append({"name": "prefers-reduced-motion", "value": "reduce"})
    if features:
        chrome.send("Emulation.setEmulatedMedia", {"features": features}, session)
    chrome.send("Page.navigate", {"url": url}, session)
    chrome.wait_event("Page.loadEventFired", session)
    if theme:
        lock = "document.documentElement.dataset.lockTheme"
        chrome.send("Runtime.evaluate", {"expression": f"document.documentElement.setAttribute('data-theme', {lock} || '{theme}')"}, session)
    if still:
        chrome.send("Runtime.evaluate", {"expression": "document.getAnimations().forEach(a => { if (a.effect && a.effect.getComputedTiming().endTime === Infinity) { a.pause(); a.currentTime = 0; } else { a.finish(); } })"}, session)
    settle(chrome, session)


SETTLE = r"""
new Promise((done) => {
  const shape = () => [...document.querySelectorAll('body *')].reduce((n, e) => { const r = e.getBoundingClientRect(); return n + r.x * 3 + r.y * 7 + r.width * 11 + r.height * 13; }, document.documentElement.scrollHeight);
  let last = null, same = 0, frames = 0;
  const tick = () => {
    const now = shape();
    same = now === last ? same + 1 : 0;
    last = now;
    if (same >= 3 || ++frames > 120) done(frames); else requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
})
"""


def settle(chrome, session):
    """Waits until every box on the page keeps its place for three frames in
    a row (at most 120 frames), so a measure never races a layout still
    running after a load or a viewport change."""
    chrome.send("Runtime.evaluate", {"expression": SETTLE, "awaitPromise": True, "returnByValue": True}, session)


def evaluate(chrome, session, expression):
    """The JSON value of a JavaScript expression in the page."""
    return chrome.send("Runtime.evaluate", {"expression": expression, "returnByValue": True}, session)["result"].get("value")


def _part(args):
    site_name, profile, slugs, worker = args
    site = design_set.Site(site_name)
    chrome, session = open_page(site, profile)
    try:
        return worker(site, chrome, session, slugs)
    finally:
        chrome.close()


def run(site, profile, slugs, worker, jobs=None):
    """Runs worker(site, chrome, session, slugs) over the slugs split across
    jobs browsers (FERROUI_DESIGN_JOBS, else the CPU count up to 4) and
    returns the concatenated lists it returns."""
    import concurrent.futures

    jobs = jobs or int(os.environ.get("FERROUI_DESIGN_JOBS") or min(4, os.cpu_count() or 1))
    parts = [slugs[i::jobs] for i in range(jobs) if slugs[i::jobs]]
    if len(parts) <= 1:
        return _part((site.name, profile + "-0", slugs, worker))
    with concurrent.futures.ProcessPoolExecutor(len(parts)) as pool:
        results = pool.map(_part, [(site.name, f"{profile}-{i}", p, worker) for i, p in enumerate(parts)])
        return [x for r in results for x in r]
