#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Writes a design set's contact sheet, design/out/<set>/index.html: every
page of the manifest by group, each with its three renders and a link to
its page, under the set's introduction from its console.py.

Usage: index.py <set>"""
import html
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import design_set  # noqa: E402

SHOTS = (("1440-light", "1440 px, light"), ("1440-dark", "1440 px, dark"), ("390-light", "390 px, light"))


def main():
    site, _ = design_set.site_from_argv(__doc__)
    c = site.console
    sections = ""
    toc = ""
    for gid, gtitle, entries in site.pages.GROUPS:
        cards = ""
        for n, slug, title, route, caption in entries:
            state = "." in n
            imgs = ""
            for suffix, label in SHOTS:
                f = f"renders/{n}-{slug}-{suffix}.png"
                imgs += (f'<figure class="shot shot-{suffix}"><a href="{f}"><img src="{f}" alt="{html.escape(title)}, {label}" loading="lazy" /></a>'
                         f'<figcaption>{label} · <code>{os.path.basename(f)}</code></figcaption></figure>')
            kind = '<span class="kind">state</span>' if state else ""
            cards += (
                f'<section class="screen{" is-state" if state else ""}" id="s{n}"><header><h3><span class="num">{n}</span> {html.escape(title)} {kind}</h3>'
                f'<p><code>{html.escape(route)}</code> · <a href="pages/{slug}.html">page</a></p><p class="caption">{html.escape(caption)}</p></header>'
                f'<div class="shots">{imgs}</div></section>'
            )
        sections += f'<section class="group" id="g-{gid}"><h2>{html.escape(gtitle)} <span class="count">{len(entries)} pages</span></h2>{cards}</section>'
        items = "".join(f'<li><a href="#s{n}">{n} {html.escape(t)}</a></li>' for n, _, t, _, _ in entries)
        toc += f'<li class="toc-group"><a href="#g-{gid}"><strong>{html.escape(gtitle)}</strong></a><ol>{items}</ol></li>'
    total = sum(len(e) for _, _, e in site.pages.GROUPS)
    light, dark = c.ACCENT
    intro = c.INTRO.replace("{total}", str(total))
    page = f"""<!doctype html><html lang="en"><head><meta charset="utf-8" /><meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{html.escape(c.TITLE)}</title><link rel="icon" href="pages/favicon.svg" type="image/svg+xml" />
<style>
:root {{ color-scheme: light dark; --ink: #0f172a; --muted: #526073; --edge: #e2e8f0; --surface: #f8fafc; --raised: #fff; --accent: {light}; }}
@media (prefers-color-scheme: dark) {{ :root {{ --ink: #f1f5f9; --muted: #94a3b8; --edge: #1e293b; --surface: #0b1020; --raised: #0f172a; --accent: {dark}; }} }}
* {{ box-sizing: border-box; }}
body {{ margin: 0; background: var(--surface); color: var(--ink); font: 15px/1.5 ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif; }}
a {{ color: var(--accent); }}
header.top {{ padding: 2rem 1.5rem 1rem; max-width: 100rem; margin: 0 auto; }}
header.top h1 {{ margin: 0 0 .25rem; font-size: 1.5rem; display: flex; align-items: center; gap: .75rem; }}
header.top h1 img {{ width: 2rem; height: 2rem; }}
header.top p {{ margin: .25rem 0; color: var(--muted); max-width: 70rem; }}
.toc {{ columns: 3; column-gap: 2rem; padding-left: 0; margin: 1rem 0 0; font-size: .8125rem; list-style: none; }}
.toc-group {{ break-inside: avoid; margin-bottom: .75rem; }}
.toc-group ol {{ padding-left: 1rem; margin: .25rem 0 0; list-style: none; }}
.group > h2 {{ margin: 2rem 0 0; padding-top: 1.5rem; border-top: 2px solid var(--ink); font-size: 1.25rem; }}
.count {{ color: var(--muted); font-weight: 400; font-size: .875rem; }}
.kind {{ display: inline-block; margin-left: .25rem; padding: 0 .375rem; border-radius: .25rem; background: var(--edge); color: var(--muted); font-size: .75rem; font-weight: 500; vertical-align: middle; }}
.is-state h3 {{ font-weight: 500; }}
@media (width < 48rem) {{ .toc {{ columns: 1; }} }}
main {{ max-width: 100rem; margin: 0 auto; padding: 0 1.5rem 3rem; }}
.screen {{ border-top: 1px solid var(--edge); padding: 1.5rem 0 2rem; }}
.screen h3 {{ margin: 0; font-size: 1.0625rem; }}
.num {{ display: inline-block; min-width: 2.75rem; color: var(--muted); font-variant-numeric: tabular-nums; }}
.screen header p {{ margin: .25rem 0; color: var(--muted); }}
.caption {{ max-width: 70rem; }}
.shots {{ display: grid; grid-template-columns: minmax(0, 3fr) minmax(0, 3fr) minmax(0, 1fr); gap: 1rem; margin-top: 1rem; align-items: start; }}
@media (width < 64rem) {{ .shots {{ grid-template-columns: minmax(0, 1fr); }} }}
.shot {{ margin: 0; }}
.shot img {{ display: block; width: 100%; height: auto; border: 1px solid var(--edge); border-radius: .5rem; background: var(--raised); }}
.shot-390-light a {{ display: block; max-height: 40rem; overflow: hidden; border-radius: .5rem; }}
.shot-390-light figcaption::after {{ content: ' · cropped here, full page behind the link'; }}
.shot figcaption {{ margin-top: .375rem; font-size: .8125rem; color: var(--muted); }}
code {{ font: .875em ui-monospace, SFMono-Regular, Menlo, monospace; }}
</style></head><body>
<header class="top"><h1><img src="pages/{c.BRAND}-icon.svg" alt="" /> {c.HEADING}</h1>
{intro}
<p>Built from <code>design/</code> in FerroUI: <code>design/tools/render.sh {site.name}</code> writes this sheet, the pages and the renders.</p>
<ol class="toc">{toc}</ol></header>
<main>{sections}</main></body></html>"""
    os.makedirs(site.out, exist_ok=True)
    with open(os.path.join(site.out, "index.html"), "w", encoding="utf-8") as f:
        f.write(page)
    print(f"{site.name}: {os.path.relpath(os.path.join(site.out, 'index.html'), design_set.REPO)}")


if __name__ == "__main__":
    main()
