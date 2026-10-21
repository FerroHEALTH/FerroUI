#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Renders a design set's built pages to PNG in headless Chrome: every page
at 1440 px light, 1440 px dark and 390 px light, full page, at 2x, into
design/out/<set>/renders/ as <number>-<slug>-<width>-<theme>.png.

Usage: render.py <set> [slug ...]   (run build.py first)"""
import base64
import os
import sys
import time

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chrome as browser  # noqa: E402
import design_set  # noqa: E402

SHOTS = (
    ("1440-light", 1440, 900, 2, False, "light"),
    ("1440-dark", 1440, 900, 2, False, "dark"),
    ("390-light", 390, 844, 2, True, "light"),
)


def capture(chrome, session, url, width, height, scale, mobile, theme, out):
    chrome.send(
        "Emulation.setDeviceMetricsOverride",
        {"width": width, "height": height, "deviceScaleFactor": scale, "mobile": mobile},
        session,
    )
    chrome.send("Emulation.setEmulatedMedia", {"features": [{"name": "prefers-color-scheme", "value": theme}]}, session)
    # Date fields in the console's one date order, year first.
    try:
        chrome.send("Emulation.setLocaleOverride", {"locale": "en-CA"}, session)
    except RuntimeError:
        pass
    chrome.send("Page.navigate", {"url": url}, session)
    chrome.wait_event("Page.loadEventFired", session)
    # Pin the theme the way the console does: data-theme on <html>.
    chrome.send(
        "Runtime.evaluate",
        {"expression": f"(document.documentElement.dataset.lockTheme ? document.documentElement.setAttribute('data-theme', document.documentElement.dataset.lockTheme) : document.documentElement.setAttribute('data-theme', '{theme}')); document.fonts.ready.then(()=>1)", "awaitPromise": True},
        session,
    )
    time.sleep(0.15)
    metrics = chrome.send("Page.getLayoutMetrics", {}, session)
    size = metrics["cssContentSize"]
    full_h = int(size["height"] + 0.999)
    # Lay the page out at its full height so sticky parts sit where the
    # whole page puts them, then shoot the clip.
    chrome.send(
        "Emulation.setDeviceMetricsOverride",
        {"width": width, "height": max(height, full_h), "deviceScaleFactor": scale, "mobile": mobile},
        session,
    )
    time.sleep(0.1)
    shot = chrome.send(
        "Page.captureScreenshot",
        {
            "format": "png",
            "clip": {"x": 0, "y": 0, "width": width, "height": max(height, full_h), "scale": 1},
            "captureBeyondViewport": True,
        },
        session,
    )
    with open(out, "wb") as f:
        f.write(base64.b64decode(shot["data"]))
    return width, max(height, full_h)


def worker(site, chrome, session, slugs):
    number = {e[3]: e[2] for e in site.pages.flat()}
    for slug in slugs:
        for suffix, width, height, scale, mobile, theme in SHOTS:
            path = os.path.join(site.renders_dir, f"{number[slug]}-{slug}-{suffix}.png")
            capture(chrome, session, site.page_url(slug), width, height, scale, mobile, theme, path)
        print(f"{site.name}/{slug}", flush=True)
    return []


def main():
    site, only = design_set.site_from_argv(__doc__)
    os.makedirs(site.renders_dir, exist_ok=True)
    slugs = [s for s in site.slugs() if not only or s in only]
    missing = [s for s in slugs if not os.path.exists(os.path.join(site.pages_dir, s + ".html"))]
    if missing:
        raise SystemExit(f"{site.name}: {len(missing)} pages are not built; run build.py {site.name} first")
    browser.run(site, "render", slugs, worker)
    print(f"{site.name}: {len(slugs) * len(SHOTS)} renders in {os.path.relpath(site.renders_dir, design_set.REPO)}")


if __name__ == "__main__":
    main()
