#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Writes a design set's pages into design/out/<set>/pages/ with their
stylesheets and icons, and checks the markup of every page on the way:
every internal link resolves, no nine-digit run passes the BSN
eleven-proof, no button-looking link leads nowhere, a count of one takes
the singular, every intro is one sentence (a settings panel's two: what
the tab governs, then how a change takes effect), and no live page links
to a planned screen. A set's route-only pages (the catalog's route_pages(),
FerroFED's r-…) are built and checked the same way, and links may lead to
them, but no browser check or render reads them. Exits 1 on a finding the
set's baseline does not hold.

Usage: build.py <set> [slug ...]   (a set is `templates` or a console name)"""
import os
import sys

# NOTE: no specification governs this: our own design; a fixed string hash keeps every page byte-identical between runs.
if os.environ.get("PYTHONHASHSEED") != "0":
    os.execve(sys.executable, [sys.executable, *sys.argv], {**os.environ, "PYTHONHASHSEED": "0"})
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import checks  # noqa: E402
import design_set  # noqa: E402


def page_function(catalog, slug):
    if hasattr(catalog, "page_fn"):
        return catalog.page_fn(slug)
    return catalog.PAGES[slug]


def main():
    site, only = design_set.site_from_argv(__doc__)
    import catalog  # the set's slug-to-page table, found through the site's path

    entries = list(site.pages.flat())
    slugs = [e[3] for e in entries]
    problems = []
    if len(slugs) != len(set(slugs)):
        problems.append(("manifest", "a slug appears twice"))
    if hasattr(catalog, "PAGES"):
        problems += [("manifest", f"{s} has no page function") for s in slugs if s not in catalog.PAGES]
        problems += [("manifest", f"{s} is not in pages.py") for s in catalog.PAGES if s not in slugs]
    routes = catalog.route_pages() if hasattr(catalog, "route_pages") else {}
    problems += [("manifest", f"{s} is both a page and a route-only page") for s in routes if s in slugs]
    every = slugs + sorted(s for s in routes if s not in slugs)
    site.copy_assets()
    exempt = set(getattr(site.console, "EXEMPT_CHECKS", ()))
    html_of = {}
    for slug in every:
        html = page_function(catalog, slug)() if slug in slugs else routes[slug]()
        html_of[slug] = html
        if only and slug not in only:
            continue
        found = checks.synthetic(html) + checks.links(html, every)
        if "dead" not in exempt:
            found += checks.dead(html)
        found += checks.plural(html) + checks.intro(html)
        problems += [(slug, f) for f in found]
        with open(os.path.join(site.pages_dir, slug + ".html"), "w", encoding="utf-8") as fh:
            fh.write(html)
    planned = getattr(site.pages, "PLANNED", None)
    if planned:
        problems += checks.reachable(entries, html_of, planned)
    ran = set(only) if only else set(every) | {"manifest"}
    return site.verdict("build", problems, ran)


if __name__ == "__main__":
    sys.exit(main())
