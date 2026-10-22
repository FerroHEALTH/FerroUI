<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# FerroUI

[![crates.io](https://img.shields.io/crates/v/ferrohealth-ui)](https://crates.io/crates/ferrohealth-ui)

The shared UI component library of the FerroHEALTH consoles: FerroBRIDGE,
FerroFED, FerroEHR, FerroTERM and FerroPIX. It is published on crates.io as
[`ferrohealth-ui`](https://crates.io/crates/ferrohealth-ui), for Leptos 0.8,
and every console builds its pages from it and from one reviewed design.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/hero-dark.png">
  <img alt="Gallery pages in four brands at desktop width (the FerroBRIDGE code page, the FerroFED federation page, the FerroTERM terminology page and the FerroEHR clinical record) beside the FerroPIX tables page on a phone" src="docs/hero.png">
</picture>

## What a console gets

- **Islands mode.** The server draws every page whole, and the browser
  hydrates only the interactive parts. `view` holds the static view
  functions, `island` the ready islands (the toast region, the navigation
  drawer, menus, pickers, the tree view and more), and `widget` the
  interactive parts a console's own island calls.
- **One interaction contract** for every component
  ([`design/COMPONENTS.md` § Interaction](design/COMPONENTS.md#interaction)):
  1. native elements first: `<dialog>`, `popover`, `<details>`, links and
     forms;
  2. behaviour in Rust compiled to WebAssembly;
  3. every page works without the WebAssembly, with a link or a form behind
     every important action;
  4. no hand-written JavaScript and no npm, so a strict CSP holds.
- **Five brands, light and dark.** One plain-CSS stylesheet defines every
  `fui-` class, and a theme file per brand sets its accent. The page follows
  the system theme or the user's choice in `data-theme`.
- **One table model:** sorting, filtering and paging by links and forms,
  with the state in the URL or, on a screen that can show patient data, in
  the session.
- **Highlighters** for YAML, JSON, XML and AQL, the Fluent words of every
  component, and the vendored Lucide icons.
- **WCAG 2.2 AA** for every role pairing in every brand, light and dark,
  held by the crate's contrast test and by a gallery journey over the
  colours the browser paints.

## The components

Every component has an ID and a spec in
[`design/COMPONENTS.md`](design/COMPONENTS.md), grouped by the console it
was first designed for and shared by all of them. The
[gallery](app/ferrohealth-ui-gallery) draws each one in every brand, light
and dark, and its screenshots are under [`docs/screenshots/`](docs/screenshots/).

| Family | What it holds | Gallery pages |
|---|---|---|
| [Page templates](design/COMPONENTS.md#page-templates) | T1 list, T2 detail, T3 settings, About, the error page | `/templates/…` |
| [The table](design/COMPONENTS.md#the-table) | the listing with its find field, filters, sort and pager | `/tables` |
| [From FerroPIX](design/COMPONENTS.md#from-ferropix-r1-to-r29) (R1 to R29) | tabs, the work-list, form fields, the date picker, toasts, the select, sign-in, Your preferences | `/forms`, `/overlays`, `/preferences` |
| [From FerroFED](design/COMPONENTS.md#from-ferrofed-f1-to-f14-for-every-console) (F1 to F14) | the revision lifecycle, the commit-confirm countdown, the secret field, the fan-out outcome, the topology, the menu, the state mark | `/configuration`, `/federation` |
| [Shared library pass](design/COMPONENTS.md#added-in-the-shared-library-pass) | the list head, the tab select, the settings panel, search results, the help switch | `/layouts`, `/templates/…` |
| [From FerroBRIDGE](design/COMPONENTS.md#from-ferrobridge-b1-to-b10-for-every-console) (B1 to B10) | the code viewer and editor, diagnostics, the load report, transforms, the ETL run report, the HL7 v2 viewer, the flow | `/code`, `/operations`, `/charts` |
| [From FerroTERM](design/COMPONENTS.md#from-ferroterm-m1-to-m6-for-every-console) (M1 to M6) | the concept tree, the coding, the version select, the value set clause, the root matrix, the mapping row | `/terminology`, `/trees` |
| [From FerroEHR](design/COMPONENTS.md#from-ferroehr-e1-to-e8-for-every-console) (E1 to E8) | the subject line, version history, the clinical reading, the folder tree, the path browser, the query builder, the results pane, the scope grant | `/clinical`, `/trees` |

Each gallery path sits under a brand, such as `/ferrobridge/code`.

## Use it in a console

Pin the exact version that crates.io shows in the badge above. A console
moves its pin after looking at its own screens.

```toml
[dependencies]
ferrohealth-ui = "=X.Y.Z"
```

1. Turn on the Leptos `islands` feature, write
   `<HydrationScripts options islands=true/>` in your shell, and hydrate
   with `leptos::mount::hydrate_islands()`.
2. Serve `ferrohealth_ui::style::STYLESHEET` and your brand's
   `Brand::theme()` as two stylesheets of your own origin, the theme second.
3. Write `data-theme` on `<html>` from the user's cookie.
4. Build each page from the templates and components your console's design
   names under [`design/consoles/`](design/consoles/).

The [crate README](crates/ferrohealth-ui/README.md) is the integration
guide, and [`docs/consoles.md`](docs/consoles.md) says what the library
owns, what your console owns, and how to ask for a component.

## Run the gallery

```
cd app/ferrohealth-ui-gallery
cargo leptos watch
```

Then open <http://127.0.0.1:3000>.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md) and
[SUPPORT.md](SUPPORT.md). A component change starts in the
[design](design/README.md).

## Licence

Business Source License 1.1, Licensor Cadasto B.V., converting to Apache 2.0
four years after each version. See [`LICENSE`](LICENSE) and
[`NOTICE`](NOTICE). The vendored Lucide icons keep their ISC licence.
