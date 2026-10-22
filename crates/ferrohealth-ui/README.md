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
ferrohealth-ui = "=0.1.7"
```

Pin the version exactly: a console moves its pin after looking at its own
screens.

## Islands

The crate has no features of its own. It turns on the `leptos` feature
`islands` and compiles under `csr`, `ssr` and `hydrate`, whichever your own
`leptos` dependency turns on. A console renders in Leptos islands mode: the
server draws every page whole, and the browser hydrates only its islands.
Set the `islands` feature, write `<HydrationScripts options islands=true/>`
in your shell, and hydrate from your browser entry, which names this crate
so its islands are linked into your WebAssembly:

```text
#[wasm_bindgen]
pub fn hydrate() {
    ferrohealth_ui::i18n::provide_page_locale(your_locale(
        &ferrohealth_ui::i18n::document_lang().unwrap_or_default(),
    ));
    leptos::mount::hydrate_islands();
}
```

Every page works with no WebAssembly: the HTML is complete, and every
important action is a link or a form.

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

A theme file sets seven properties. Five (`--accent`, `--accent-hover`,
`--accent-ink`, `--accent-tint`, `--focus`) are written in three blocks:
`:root`, `:root[data-theme="dark"]`, and
`@media (prefers-color-scheme: dark) { :root:not([data-theme="light"], [data-theme="dark"]) }`.
Two (`--preview-accent-light`, `--preview-accent-dark`) are written once, in
a closing `:root` block: the accents the theme previews in Your preferences
draw, whatever theme the page itself is in.
The theme choice is `data-theme` (`light`, `dark` or `system`) on `<html>`,
which your server writes from its own cookie or setting (`theme::Theme`).

## Components

- `view`: `button`, `status` (pill, badge, banner), `surface` (page header,
  card, stat, definitions, disclosure), `form` (field, check), `tabs` (link
  tabs), `empty`, `refusal` (an error with its issues), `code` (highlighted
  panel and numbered lines), `theme_switch`, `menu`, `dialog`
  (a popover confirmation), `shell` (skip link, top bar, footer, centred
  page) and `sign_in` (a basic form, a provider button, or both).
- `island`: the ready islands, `ToastRegion`, `NavDrawer` and
  `KeyboardTabs`, each taking its words as props.
- `widget`: the interactive components as functions you call inside an
  island: `nav` (the sidebar, plain links with the page marked, inside its
  drawer island), `tabs` (the panels for `KeyboardTabs`), `toast` (`raise`
  from any island), `editor` (the highlighted overlay editor with line
  marks), `filter` (an in-place filter over rows that are already island
  data) and the rest.
- `table`: the paging and sorting model (`Paging`, `Window`, `Sort`,
  `StateSource`), the data table with its cells and list card (`view`), the
  link and `POST` pagers (`pager`) and the find field (`filter`), each a
  link or a form that works with no script.
- `highlight::html(language, text, lines, marked)`, `icon::icon(glyph, size)`,
  `tone::Tone`, `theme::Theme`.

## The top bar

The top bar keeps its start (the drawer button and the lockup) and its end
on one row at every width. When the row is short of room, the lockup's
accent word folds away first (below 30rem). A status pill in the end keeps
its full words and never shortens to its dot: show it from the width where
the row has room (`fui-roomy-only`, from 64rem) and put it in the operator
menu's `extra` below that if a narrow screen needs it. The end's buttons (the theme toggle, the operator menu) never fold. A
search box between them takes a row of its own below 48rem. Put anything
else a console offers, such as a three-way theme choice or a scope list, in
the operator menu's `extra`.

No component writes a `style` attribute or a `<style>` element, defines a
server function, fetches, or touches browser storage.

## Locales

Every word a component draws itself (a pager step, a dialog's Cancel, the
theme toggle's name, an empty listing's range) comes from a Fluent catalog
under `locales/`, one directory per language. English is the one catalog
that ships, and every lookup falls back to it; a language joins as one
catalog file, with no code change. Choose the locale once per render, at the
root of the page tree on the server, and write it as `<html lang>`; an
island takes the words it draws as props the server resolved:

```rust
use ferrohealth_ui::i18n::{Locale, locale, provide_locale};
use leptos::prelude::*;

let lang = Owner::new().with(|| {
    // From the configuration, or `Locale::negotiate` over `Accept-Language`.
    provide_locale(Locale::parse("en-GB").unwrap_or_default());
    locale().tag()
});
assert_eq!(lang, "en-GB");
```

A page that provides no locale draws English. Catalog the console's own
strings the same way: a `fluent_templates::static_loader!` over the
console's own `locales/`, looked up with `Locale::lookup`, so the console's
words follow the same choice. `view::theme_switch::locale_switch` is the
language switch, each choice named in its own language.

## Licence

Business Source License 1.1, Licensor Cadasto B.V., converting to Apache 2.0
four years after each version (`LICENSE`, `NOTICE` in the repository). The
Lucide icons under `vendor/lucide/` are under the ISC licence.
