#!/usr/bin/env python3
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Replaces the design library in design/ with the design folders it came
from, so an updated design lands as one mechanical sync.

The source root holds FerroUI-ontwerp/ (COMPONENTS.md, css/, themes/,
tools/) and one <Console>-ontwerp/ per console (src/tools/*.py and the
brand icons in src/). The sync:

- copies COMPONENTS.md, css/ferroui-console.css and themes/*.css; the base
  stylesheet stays the crate's own copy, so the library's is never copied;
- writes the library's helpers tpl.py, parts.py and pg_about.py into
  design/tools/ and its common.py, data.py and pages.py into
  design/templates/, with templates.py turned into templates/catalog.py;
- per console, copies src/tools/*.py except the tools the shared ones in
  design/tools/ replace (build, index, render, overflow, hover, contrast,
  selects, popups, forms) and except any module identical to the shared
  copy it would shadow; turns build.py into catalog.py (its page table and
  any route_pages(), without the checks) and index.py into console.py (the
  contact sheet's words); copies the icons to assets/;
- adds the SPDX lines to every file that lacks them.

Each console directory is replaced whole except its baseline.tsv. The
repository's own tools (build, checks, chrome, design_set, render, the
checks, index, sync) and templates/console.py are never touched.

Usage: sync.py <source root>   (for example ~/Downloads)"""
import filecmp
import os
import re
import shutil
import sys

TOOLS = os.path.dirname(os.path.abspath(__file__))
DESIGN = os.path.dirname(TOOLS)

CONSOLES = {
    "FerroPIX": ("ferropix", "FerroPIX console"),
    "FerroFED": ("ferrofed", "FerroFED console"),
    "FerroBRIDGE": ("ferrobridge", "FerroBRIDGE console"),
    "FerroTERM": ("ferroterm", "FerroTERM console"),
    "FerroEHR": ("ferroehr", "FerroEHR viewer"),
}
REPLACED = {"build.py", "index.py", "render.py", "overflow.py", "hover.py", "contrast.py", "selects.py", "popups.py", "forms.py"}
SHARED_TOOLS = ("tpl.py", "parts.py", "pg_about.py")
SHARED_TEMPLATES = ("common.py", "data.py", "pages.py")
KEEP = {"baseline.tsv"}

PY = "# SPDX-FileCopyrightText: Cadasto B.V.\n# SPDX-License-Identifier: BUSL-1.1\n"
MD = "<!-- SPDX-FileCopyrightText: Cadasto B.V. -->\n<!-- SPDX-License-Identifier: BUSL-1.1 -->\n"
CSS = "/* SPDX-FileCopyrightText: Cadasto B.V. */\n/* SPDX-License-Identifier: BUSL-1.1 */\n"


def licensed(text, header):
    """The text with the SPDX lines after any shebang, unless it has them."""
    if "SPDX-License-Identifier" in "\n".join(text.splitlines()[:6]):
        return text
    if text.startswith("#!"):
        first, _, rest = text.partition("\n")
        return first + "\n" + header + rest
    return header + text


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text)


def read(path):
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def copy_text(src, dst, header):
    write(dst, licensed(read(src), header))


def catalog(build_src, title):
    """A console's build.py as its page table: the imports, the slug table
    or page_fn and any route_pages(), without the checks and main()."""
    full = read(build_src)
    s = full[: full.index("\ndef check_synthetic")].rstrip() + "\n"
    # A console with route-only pages (FerroFED's r-…) names them in
    # route_pages(), which the page table keeps.
    extra = re.search(r"^def route_pages\(\):\n.*?(?=^\S)", full, re.S | re.M)
    if extra:
        s += "\n\n" + extra.group(0).rstrip() + "\n"
    s = re.sub(r'\A#!/usr/bin/env python3\n"""(.*?)"""\n', "", s, flags=re.S)
    s = s.replace("from parts import bsn_valid  # noqa: E402\n", "")
    s = re.sub(r"^sys\.path\.insert\(0, os\.path\.dirname\(os\.path\.abspath\(__file__\)\)\)\n", "", s, flags=re.M)
    s = re.sub(r"^import (os|re|sys)\n", "", s, flags=re.M)
    s = s.replace("  # noqa: E402", "").lstrip("\n")
    doc = f'"""The {title} mockups\' page table: each slug of pages.py and the\nfunction that draws it, read by design/tools/build.py."""\n'
    return PY + doc + s


def templates_catalog(src):
    s = read(src)
    s = s[s.index("import pages  # noqa: E402"): s.index("\ndef main():")].rstrip() + "\n"
    s = s.replace("  # noqa: E402", "").replace('PAGES = {"list"', '_PAGES = {"list"')
    doc = ('"""The library\'s reference templates (T1 list, T2 detail, T3 settings and\n'
           "reference, and About), drawn with the family's helpers in the FerroFED\n"
           "theme with example content, read by design/tools/build.py. A link to a\n"
           'page that is not a template points nowhere."""\nimport re\n\n')
    tail = ('\n\ndef _local(fn):\n    """Points every link to a page outside the templates at #."""\n'
            "    slugs = {x[3] for x in pages.flat()}\n"
            "    return lambda: re.sub(r'href=\"([a-z0-9-]+)\\.html\"', lambda m: m.group(0) if m.group(1) in slugs else 'href=\"#\"', fn())\n"
            "\n\nPAGES = {slug: _local(fn) for slug, fn in _PAGES.items()}\n")
    return PY + doc + s + tail


def console_py(index_src, brand):
    """The contact sheet's words from a console's index.py."""
    src = read(index_src)
    title = re.search(r"<title>(.*?)</title>", src).group(1)
    light = re.search(r"^:root \{\{.*?--accent: (#[0-9a-f]+);", src, re.M).group(1)
    dark = re.search(r"^@media.*?--accent: (#[0-9a-f]+);", src, re.M).group(1)
    heading = re.search(r'<h1><img src="src/[^"]+" alt="" /> (.*?)</h1>', src).group(1)
    paras = [p for p in re.findall(r"^(<p>.*?</p>)$", src, re.M)
             if not p.startswith("<p>Sources are in") and not p.startswith("<p><code>src/tools/")]
    intro = "\n".join(paras).replace("{{", "{").replace("}}", "}")
    name = heading.split(" mockups")[0]
    return (PY + f'"""What the shared tools need to know about the {name} mockups."""\n\n'
            "# The theme file under design/themes/ and the console whose assets/ holds\n# the brand icons.\n"
            f'BRAND = "{brand}"\nASSETS = "{brand}"\n\n'
            "# The contact sheet: its title, its heading, its accent in light and dark,\n"
            "# and its introduction ({total} is the page count).\n"
            f'TITLE = "{title}"\nHEADING = "{heading}"\nACCENT = ("{light}", "{dark}")\nINTRO = """{intro}"""\n')


def sync_library(lib):
    copy_text(os.path.join(lib, "COMPONENTS.md"), os.path.join(DESIGN, "COMPONENTS.md"), MD)
    copy_text(os.path.join(lib, "css", "ferroui-console.css"), os.path.join(DESIGN, "css", "ferroui-console.css"), CSS)
    themes = os.path.join(DESIGN, "themes")
    shutil.rmtree(themes, ignore_errors=True)
    for f in sorted(os.listdir(os.path.join(lib, "themes"))):
        if f.endswith(".css"):
            copy_text(os.path.join(lib, "themes", f), os.path.join(themes, f), CSS)
    for f in SHARED_TOOLS:
        copy_text(os.path.join(lib, "tools", f), os.path.join(TOOLS, f), PY)
    for f in SHARED_TEMPLATES:
        copy_text(os.path.join(lib, "tools", f), os.path.join(DESIGN, "templates", f), PY)
    write(os.path.join(DESIGN, "templates", "catalog.py"), templates_catalog(os.path.join(lib, "tools", "templates.py")))


def same_as_shared(src, name):
    for shared in (os.path.join(TOOLS, name), os.path.join(DESIGN, "templates", name)):
        if os.path.exists(shared) and read(shared) == licensed(read(src), PY):
            return True
    return False


def sync_console(root, product, brand, title):
    src = os.path.join(root, f"{product}-ontwerp", "src")
    dst = os.path.join(DESIGN, "consoles", brand)
    kept = {f: read(os.path.join(dst, f)) for f in KEEP if os.path.exists(os.path.join(dst, f))}
    shutil.rmtree(dst, ignore_errors=True)
    tools = os.path.join(src, "tools")
    for f in sorted(os.listdir(tools)):
        path = os.path.join(tools, f)
        if not f.endswith(".py") or f in REPLACED or same_as_shared(path, f):
            continue
        copy_text(path, os.path.join(dst, f), PY)
    write(os.path.join(dst, "catalog.py"), catalog(os.path.join(tools, "build.py"), title))
    write(os.path.join(dst, "console.py"), console_py(os.path.join(tools, "index.py"), brand))
    for f in ("favicon.svg", f"{brand}-icon.svg", f"{brand}-icon-dark.svg"):
        os.makedirs(os.path.join(dst, "assets"), exist_ok=True)
        copy_text(os.path.join(src, f), os.path.join(dst, "assets", f), MD)
    for f, text in kept.items():
        write(os.path.join(dst, f), text)


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    root = os.path.abspath(os.path.expanduser(sys.argv[1]))
    sync_library(os.path.join(root, "FerroUI-ontwerp"))
    for product, (brand, title) in CONSOLES.items():
        sync_console(root, product, brand, title)
    print(f"synced {len(CONSOLES)} consoles and the library from {root}")
    for path in ("tools", "templates", "consoles"):
        same = filecmp.dircmp(os.path.join(DESIGN, path), os.path.join(DESIGN, path))
        del same


if __name__ == "__main__":
    main()
