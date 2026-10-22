# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Where a design set lives and where its output goes, for every tool.

A set is one console under design/consoles/<name>/ or the reference
templates under design/templates/. Its modules are found in this order:
the set's own directory, design/templates/ (the library's helpers) and
design/tools/ (the shared helpers), so a console that keeps its own
common.py draws with it and one that keeps none draws with the library's.

Output goes under design/out/<name>/, which git ignores: pages/ holds the
HTML with its stylesheets and icons, renders/ the PNGs, and index.html the
contact sheet."""
import hashlib
import importlib
import os
import shutil
import sys

TOOLS = os.path.dirname(os.path.abspath(__file__))
DESIGN = os.path.dirname(TOOLS)
REPO = os.path.dirname(DESIGN)
CONSOLES = os.path.join(DESIGN, "consoles")
TEMPLATES = os.path.join(DESIGN, "templates")
OUT = os.path.join(DESIGN, "out")

# The base stylesheet is the crate's own copy, so the mockups draw with
# exactly what a console ships.
BASE_CSS = os.path.join(REPO, "crates", "ferrohealth-ui", "css", "ferrohealth-ui.css")
CONSOLE_CSS = os.path.join(DESIGN, "css", "ferroui-console.css")


BASELINE_HEAD = """# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# The findings this set carried when its design was approved: a check
# fails on any finding not listed here and on a listed one that no longer
# occurs, so the list only shrinks. One finding per line, tab-separated:
# the check, the page, then what the check reports. Rewrite it with
# FERROUI_DESIGN_BASELINE=write design/tools/check.sh <set>.
"""


def names():
    """Every set, the templates first, then the consoles by name."""
    consoles = sorted(d for d in os.listdir(CONSOLES) if os.path.isfile(os.path.join(CONSOLES, d, "pages.py")))
    return ["templates"] + consoles


class Site:
    """One design set: its modules, its manifest and its output paths."""

    def __init__(self, name):
        if name not in names():
            raise SystemExit(f"unknown design set {name!r}; one of: {', '.join(names())}")
        self.name = name
        self.src = TEMPLATES if name == "templates" else os.path.join(CONSOLES, name)
        self.out = os.path.join(OUT, name)
        self.pages_dir = os.path.join(self.out, "pages")
        self.renders_dir = os.path.join(self.out, "renders")
        cache = os.environ.get("XDG_CACHE_HOME") or os.path.join(os.path.expanduser("~"), ".cache")
        # NOTE: no specification governs this: our own design; each checkout keeps its own browser profiles, so two worktrees never share one.
        checkout = hashlib.sha256(REPO.encode()).hexdigest()[:12]
        self.state_dir = os.environ.get("MOCKUP_STATE_DIR") or os.path.join(cache, "ferroui-design", checkout, name)
        for path in (TOOLS, TEMPLATES, self.src):
            if path in sys.path:
                sys.path.remove(path)
            sys.path.insert(0, path)
        self.console = importlib.import_module("console")
        self.pages = importlib.import_module("pages")

    def slugs(self):
        return [entry[3] for entry in self.pages.flat()]

    def page_url(self, slug):
        return "file://" + os.path.join(self.pages_dir, slug + ".html")

    def copy_assets(self):
        """Puts the stylesheets and the brand icons beside the pages."""
        brand = self.console.BRAND
        assets = os.path.join(CONSOLES, self.console.ASSETS, "assets")
        os.makedirs(self.pages_dir, exist_ok=True)
        pairs = [
            (BASE_CSS, "ferrohealth-ui.css"),
            (CONSOLE_CSS, "mockup.css"),
            (os.path.join(DESIGN, "themes", brand + ".css"), brand + ".css"),
        ]
        pairs += [(os.path.join(assets, f), f) for f in sorted(os.listdir(assets))]
        for src, dst in pairs:
            shutil.copyfile(src, os.path.join(self.pages_dir, dst))


    def baseline_path(self):
        return os.path.join(self.src, "baseline.tsv")

    def verdict(self, check, findings, ran):
        """Prints a check's findings against the set's baseline and returns
        the exit status: 1 when a finding is not in the baseline, or when a
        baseline entry for a page that ran no longer occurs (the baseline only
        shrinks). A finding is a tuple of strings whose first item is the
        page's slug; its key is the check's name followed by the tuple."""
        known = set()
        if os.path.exists(self.baseline_path()):
            with open(self.baseline_path(), encoding="utf-8") as fh:
                for line in fh:
                    line = line.rstrip("\n")
                    if line and not line.startswith("#"):
                        known.add(line)
        keys = {"\t".join((check,) + tuple(str(x).replace("\t", " ").replace("\n", " ") for x in f)) for f in findings}
        if os.environ.get("FERROUI_DESIGN_BASELINE") == "write":
            kept = {k for k in known if not (k.split("\t")[0] == check and k.split("\t")[1] in ran)}
            if kept | keys:
                with open(self.baseline_path(), "w", encoding="utf-8") as fh:
                    fh.write(BASELINE_HEAD + "".join(k + "\n" for k in sorted(kept | keys)))
            elif os.path.exists(self.baseline_path()):
                os.remove(self.baseline_path())
            print(f"{self.name} {check}: wrote {len(keys)} findings to {os.path.relpath(self.baseline_path(), REPO)}")
            return 0
        new = sorted(keys - known)
        stale = sorted(k for k in known - keys if k.split("\t")[0] == check and k.split("\t")[1] in ran)
        for k in new:
            print(f"{self.name}: new finding: {k}")
        for k in stale:
            print(f"{self.name}: fixed, remove it from {os.path.relpath(self.baseline_path(), REPO)}: {k}")
        print(f"{self.name} {check}: {len(set(ran) - {'manifest'})} pages, {len(keys)} findings, {len(keys) - len(new)} of them in the baseline")
        return 1 if new or stale else 0


def chrome_binary():
    """The Chrome to drive: FERROUI_CHROME (what CI and the gallery use), else
    the macOS install, else the first Chrome or Chromium on PATH."""
    named = os.environ.get("FERROUI_CHROME")
    if named:
        return named
    mac = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    if os.path.exists(mac):
        return mac
    for cmd in ("google-chrome", "google-chrome-stable", "chromium", "chromium-browser", "chrome"):
        found = shutil.which(cmd)
        if found:
            return found
    raise SystemExit("no Chrome found: set FERROUI_CHROME to the browser binary")


def in_part(slugs):
    """The slugs of the part FERROUI_DESIGN_PART names, `k/n` for every nth
    slug from the kth (CI checks a large set in two jobs), else all."""
    part = os.environ.get("FERROUI_DESIGN_PART")
    if not part:
        return slugs
    try:
        k, n = (int(x) for x in part.split("/"))
    except ValueError:
        raise SystemExit(f"FERROUI_DESIGN_PART={part!r} is not k/n")
    if not 1 <= k <= n:
        raise SystemExit(f"FERROUI_DESIGN_PART={part!r} names no part")
    return slugs[k - 1::n]


def site_from_argv(usage):
    """The set named by argv[1] and the slugs after it (empty means all)."""
    if len(sys.argv) < 2 or sys.argv[1] in ("-h", "--help"):
        raise SystemExit(usage)
    return Site(sys.argv[1]), set(sys.argv[2:])
