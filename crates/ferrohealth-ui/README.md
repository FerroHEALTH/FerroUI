<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# ferrohealth-ui

The shared UI components of the FerroHEALTH consoles (FerroBRIDGE, FerroFED,
FerroEHR and FerroTERM), for Leptos 0.8: static view functions, a few
interactive widgets, one table model, highlighters for YAML, JSON, XML and
AQL, Lucide icons, and one plain-CSS stylesheet with a theme file per brand.
Every pairing of every theme is held to WCAG 2.2 AA by the crate's tests.

```toml
[dependencies]
ferrohealth-ui = "=0.1.1"
```

Pin the version exactly: a console moves its pin after looking at its own
screens.

## Render modes

The crate has no features. It compiles under `csr`, `ssr` and `hydrate`,
whichever your own `leptos` dependency turns on.

## The stylesheet and a theme

Serve two stylesheets from your own origin, the theme after the stylesheet,
so `style-src 'self'` holds:

```rust
use ferrohealth_ui::style::{Brand, STYLESHEET};

let stylesheet: &str = STYLESHEET;          // serve as /ferrohealth-ui.css
let theme: &str = Brand::Bridge.theme();    // serve as /theme.css
```

The same files are `css/ferrohealth-ui.css` and `themes/<brand>.css` in the
crate, for a build that imports them into its Tailwind entry;
`css/tailwind-theme.css` (`style::TAILWIND_THEME`) maps the roles to Tailwind
utilities (`bg-surface-raised`, `text-muted`, `border-edge`). The stylesheet
declares `@layer theme, base, components, utilities;`, so your utilities win.

A theme file sets five properties (`--accent`, `--accent-hover`,
`--accent-ink`, `--accent-tint`, `--focus`) in three blocks: `:root`,
`:root[data-theme="dark"]`, and
`@media (prefers-color-scheme: dark) { :root:not([data-theme="light"], [data-theme="dark"]) }`.
The theme choice is `data-theme` (`light`, `dark` or `system`) on `<html>`,
which your server writes from its own cookie or setting (`theme::Theme`).

## Components

- `view`: `button`, `status` (pill, badge, banner), `surface` (page header,
  card, stat, definitions, disclosure), `form` (field, check), `tabs` (link
  tabs), `empty`, `refusal` (an error with its issues), `code` (highlighted
  panel and numbered lines), `theme_switch`, `menu`, `dialog`
  (a popover confirmation), `shell` (skip link, top bar, footer, centred
  page) and `sign_in` (a basic form, a provider button, or both).
- `widget`: `nav` (the sidebar and drawer), `tabs` (keyboard tabs), `toast`,
  `editor` (the highlighted overlay editor with line marks), `pager` (the
  session pager) and `filter`.
- `table`: the paging and sorting model (`Paging`, `Window`, `Sort`,
  `StateSource`) and the data table with its link pager.
- `highlight::html(language, text, lines, marked)`, `icon::icon(glyph, size)`,
  `tone::Tone`, `theme::Theme`.

No component writes a `style` attribute or a `<style>` element, defines a
server function, fetches, or touches browser storage.

## Licence

Business Source License 1.1, Licensor Cadasto B.V., converting to Apache 2.0
four years after each version (`LICENSE`, `NOTICE` in the repository). The
Lucide icons under `vendor/lucide/` are under the ISC licence.
