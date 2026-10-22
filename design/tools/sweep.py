#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Runs every browser check of a design set over one render of its pages:
each page is loaded once per distinct width, colour scheme and motion
setting, and the overflow, selects, contrast, popups and forms probes all
read that load, with the hover pass on a page of its own (hovering leaves
the pointer on it). Each check's findings are held to the baseline as when
it runs alone (overflow.py, hover.py, contrast.py, selects.py, popups.py,
forms.py). FERROUI_DESIGN_PART=k/n checks every nth page from the kth
alone, and holds the baseline to those pages.

Usage: sweep.py <set> [slug ...]   (run build.py first)"""
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chrome as browser  # noqa: E402
import design_set  # noqa: E402

CHECKS = ("overflow", "hover", "contrast", "selects", "popups", "forms")


def main():
    site, only = design_set.site_from_argv(__doc__)
    slugs = design_set.in_part([s for s in site.slugs() if not only or s in only])
    found = browser.check(site, slugs, list(CHECKS))
    status = 0
    for name in CHECKS:
        status |= site.verdict(name, found[name], set(slugs))
    return status


if __name__ == "__main__":
    sys.exit(main())
