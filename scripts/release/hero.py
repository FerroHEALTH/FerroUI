# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Composes the README's hero, docs/hero.png and docs/hero-dark.png, from
the gallery screenshots under docs/screenshots/: the top of four brands'
pages at 1440 px and one brand's page at 390 px, each light image in the
light hero and each dark image in the dark one (no specification governs
this: our own design). The refresh pull request runs it after the capture
(docs/release.md).

Usage: python3 scripts/release/hero.py
  FERROUI_CHROME names the browser binary. Run it under
  scripts/chrome-lock.sh beside other browser work."""
import base64
import html
import os
import shutil
import sys
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "design", "tools"))

import chrome  # noqa: E402

# The wide tiles, two by two, and the phone tile beside them.
WIDE = ["ferrobridge/code", "ferrofed/federation", "ferroterm/terminology", "ferroehr/clinical"]
NARROW = "ferropix/tables-narrow"

# One hero's backdrop and tile edge per theme.
THEMES = {
    "": {"page": "#eef1f5", "edge": "#d5dbe3"},
    "-dark": {"page": "#0b0e13", "edge": "#2a313c"},
}

TILE_W, TILE_H, GAP, PAD = 600, 375, 16, 32
PHONE_W = 360
PHONE_H = 2 * TILE_H + GAP
WIDTH = PAD * 2 + 2 * TILE_W + GAP + GAP + PHONE_W
HEIGHT = PAD * 2 + PHONE_H


def image(name, suffix):
    path = os.path.join(ROOT, "docs", "screenshots", f"{name}{suffix}.png")
    if not os.path.exists(path):
        raise SystemExit(f"hero: {os.path.relpath(path, ROOT)} is missing; capture the gallery first")
    return "file://" + path


def page(suffix):
    colours = THEMES[suffix]
    wide = "".join(f'<img class="wide" src="{html.escape(image(n, suffix))}" alt="">' for n in WIDE)
    phone = f'<img class="phone" src="{html.escape(image(NARROW, suffix))}" alt="">'
    return f"""<!doctype html><meta charset="utf-8"><style>
html, body {{ margin: 0; background: {colours['page']}; }}
body {{ display: flex; gap: {GAP}px; padding: {PAD}px; width: {WIDTH - 2 * PAD}px; }}
.grid {{ display: grid; grid-template-columns: repeat(2, {TILE_W}px); gap: {GAP}px; }}
img {{ display: block; object-fit: cover; object-position: top; border: 1px solid {colours['edge']};
  border-radius: 10px; box-sizing: border-box; box-shadow: 0 6px 24px rgb(0 0 0 / 0.12); }}
.wide {{ width: {TILE_W}px; height: {TILE_H}px; }}
.phone {{ width: {PHONE_W}px; height: {PHONE_H}px; }}
</style><div class="grid">{wide}</div>{phone}"""


def main():
    work = tempfile.mkdtemp(prefix="ferroui-hero-")
    browser = chrome.Chrome(os.path.join(work, "profile"), work)
    try:
        _, session = chrome.attach(browser)
        for suffix in THEMES:
            source = os.path.join(work, f"hero{suffix}.html")
            with open(source, "w", encoding="utf-8") as f:
                f.write(page(suffix))
            chrome.load(browser, session, "file://" + source, WIDTH, HEIGHT)
            shot = browser.send("Page.captureScreenshot", {"format": "png", "clip": {"x": 0, "y": 0, "width": WIDTH, "height": HEIGHT, "scale": 2}}, session)
            out = os.path.join(ROOT, "docs", f"hero{suffix}.png")
            with open(out, "wb") as f:
                f.write(base64.b64decode(shot["data"]))
            print(f"hero: wrote {os.path.relpath(out, ROOT)}")
    finally:
        browser.close()
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
