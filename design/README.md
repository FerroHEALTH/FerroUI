<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# The console design library

This directory holds the approved design of the FerroHEALTH consoles: the
component specification, the capture scaffolding, the brand themes, the
reference templates and every page of the five console mockups (FerroPIX,
FerroFED, FerroBRIDGE, FerroTERM and the FerroEHR viewer). It is the
source of truth for FerroUI 0.3.0: the library's components are built to
match it.

| Path | What it is |
|---|---|
| `COMPONENTS.md` | The specification: the family rules, the templates T1 to T3 and About, the table, and every component (R1 to R29, F1 to F14, B1 to B10, M1 to M6, E1 to E8 and the additions). |
| `css/ferroui-console.css` | The capture scaffolding the mockups draw with, over the crate's `crates/ferrohealth-ui/css/ferrohealth-ui.css`: an open dialog, menu or picker drawn in the page, and a hover or focus held still. Every rule is prefixed `mk-` and is never component API. |
| `themes/` | One brand file per console: its accents and the two preview accents Your preferences draws. |
| `templates/` | The reference templates' page table (`catalog.py`), manifest and the library's helpers (`common.py`, `data.py`). |
| `consoles/<console>/` | One console's page modules, its manifest (`pages.py`), its page table (`catalog.py`), the words of its contact sheet (`console.py`), its brand icons (`assets/`) and its `baseline.tsv`. |
| `tools/` | The shared tools: the build with its markup checks, the renderer, the browser checks (overflow, hover, contrast, selects, popups, forms), the contact sheet, and the sync. |

The mockups draw with the crate's own stylesheet, copied beside the pages
at build time, so a mockup shows what a console ships. It holds every
component rule and the family rules as CSS; the library's own copy of
`ferrohealth-ui.css` is never kept here.

`COMPONENTS.md` keeps the words of the design folder it came from, so its
"What is in this folder" table and its commands name that folder's layout.
This file and the table above name the layout here.

## Render

You need Python 3.14 and Google Chrome. `FERROUI_CHROME` names the browser
binary when it is not the macOS install or on `PATH`.

```sh
design/tools/render.sh ferrobridge     # one console
design/tools/render.sh templates       # the reference templates
design/tools/render.sh all             # every set
```

Each set lands in `design/out/<set>/`, which git ignores: `pages/` holds
the HTML with its stylesheets, `renders/` every page at 1440 px light and
dark and at 390 px light, full page at 2x, and `index.html` the contact
sheet. Name slugs after the set to render only those pages.
`FERROUI_DESIGN_JOBS` sets how many browsers run side by side (4 by
default). The renders are never committed; the five consoles come to about
2,400 images.

## Check

```sh
design/tools/check.sh ferrobridge      # or all
```

The checks are the ones CI runs, one job per set:

- **Build** (`build.py`): every internal link resolves to a page; no
  nine-digit run passes the BSN eleven-proof; no button-looking link leads
  nowhere; a count of one takes the singular; a page's and a section's
  intro is one sentence and a settings panel's at most two; no live page
  links to a planned screen. A set's route-only pages (FerroFED's `r-…`
  pages, one per tab of every object the captured screens do not show)
  are built and held to the same markup checks, but no browser check,
  render or contact sheet reads them.
- **Overflow** (`overflow.py`): at 1440 and 390 px, a box narrower than its
  content, anything past the viewport, text on text, text past an SVG
  card, a word broken inside itself, a short label forced over two lines.
- **Hover** (`hover.py`): hovering or focusing a control moves nothing
  outside it (family rule 3).
- **Contrast** (`contrast.py`): every visible text run, light and dark,
  against WCAG 2.2 AA.
- **Selects** (`selects.py`): every select and R29 listbox button has an
  accessible name, shows its chosen value unclipped, is at least 24 px, and
  keeps `aria-expanded` in step with its listbox.
- **Popups** (`popups.py`): every open listbox, menu, popover, search
  results panel and date picker, at 1440 light and dark and 390 light, is
  inside the screen, shows its items unclipped, is solid (ground, border
  and shadow) and stands out from what it opens over.
- **Forms** (`forms.py`): at 1440 and 390 px light, fields side by side
  share a top line, no text input's placeholder (or value, in a field
  narrower than its row) is clipped, an open phone drawer is docked full
  height at the left edge, and no two visible primary actions carry the
  same label outside a dialog, menu or specimen.

Each set's `baseline.tsv` lists the findings it carried when its design was
approved. A check fails on a finding the baseline does not hold, and on a
baseline entry that no longer occurs, so the list only shrinks. After an
approved change to a set, rewrite it with
`FERROUI_DESIGN_BASELINE=write design/tools/check.sh <set>` and review the
diff. The browser checks measure the system font, so CI runs them on macOS,
where the approved renders were drawn.

## Change a component

A component change lands in this order:

1. `COMPONENTS.md`, the specification;
2. the crate's `crates/ferrohealth-ui/css/ferrohealth-ui.css`, and a theme
   file when a brand changes (`themes/` here and the crate's `themes/`);
3. every console that draws the component, re-rendered with
   `design/tools/render.sh all`, each changed image looked at, and checked
   with `design/tools/check.sh all`.

The Rust component in `crates/ferrohealth-ui` follows the design, never the
other way round.

## Sync a revised design

The design is reviewed in its own folders, one `FerroUI-ontwerp/` for the
library and one `<Console>-ontwerp/` per console. A revised design replaces
this directory's copy in one command:

```sh
python3 design/tools/sync.py ~/Downloads
```

The sync copies the specification, the themes and every console's page
modules; it copies no stylesheet, and says so, since a revised layer is
merged by hand into the crate's stylesheet (its component rules) and
`css/ferroui-console.css` (its `mk-` rules); drops a module that is identical to the shared one
it would shadow; turns each console's `build.py` into `catalog.py` and its
`index.py` into `console.py`, keeping a `route_pages()` beside the page
table; and adds the SPDX lines. It keeps each
`baseline.tsv` and never touches the shared tools. Run the checks after a
sync and update the baselines in the same pull request.
