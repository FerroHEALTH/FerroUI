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
ferrohealth-ui = "=0.3.2"
```

Pin the version exactly: a console moves its pin after looking at its own
screens. The [console guide](https://github.com/FerroHEALTH/FerroUI/blob/main/docs/consoles.md)
says what the library owns, what a console owns, and which document governs
each part.

## Islands

The crate has no features of its own. It turns on the `leptos` feature
`islands` and compiles under `csr`, `ssr` and `hydrate`, whichever your own
`leptos` dependency turns on. A console renders in Leptos islands mode: the
server draws every page whole, and the browser hydrates only its islands.
Set the `islands` feature, write `<HydrationScripts options islands=true/>`
in your shell, and hydrate from your browser entry, which names this crate
so its islands are linked into your WebAssembly:

```text
extern crate ferrohealth_ui;

#[wasm_bindgen]
pub fn hydrate() {
    leptos::mount::hydrate_islands();
}
```

Every island takes the words it draws as props your server resolves (each
widget's `*Words::of_page()`), so the browser builds no locale and carries
no catalog.

`DatePicker` and `DateRangePicker` are lazy islands (`#[island(lazy)]`):
build your site with `cargo leptos build --split` and their code loads from
chunks of its own on a page that draws them, out of the WebAssembly every
page loads. Without `--split` they hydrate from your one WebAssembly as
every other island does.

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

- `view`: `button`, `status` (pill, badge, banner, the top bar's status
  pill), `surface` (page header, card, stat, definitions, disclosure),
  `form` (field, check), `tabs` (binder tabs
  joined to their panel), `popover` (the anchored popover form), `empty`, `refusal` (an error
  with its issues), `code` (highlighted panel and numbered lines),
  `preferences` (Your preferences: its three tabs, each preference a plain
  form post), `dialog` (a popover confirmation), `shell` (skip link,
  top bar, footer, centred page) and `sign_in` (the R19 card in every
  state, with the provider's action and an optional basic form).
- `island`: the ready islands, such as `ToastRegion`, `NavDrawer` and
  `KeyboardTabs` (the module lists them all), each taking its words as props. Every
  widget takes the words it draws as a `*Words` struct its `of_page()`
  resolves on the server, so no island carries a catalog.
- `widget`: the interactive components as functions you call inside an
  island: `nav` (the sidebar, plain links with the page marked, its counts
  and its rail, inside its drawer island), `menu` (the user menu), `tabs`
  (the panels for `KeyboardTabs`), `toast` (`raise` a title, a line and an action from any island),
  `editor` (the highlighted overlay editor with line marks), `filter` (an
  in-place filter over rows that are already island data) and the rest.
- `table`: the paging and sorting model (`Paging`, `Window`, `Sort`,
  `StateSource`), the data table with its cells and list card (`view`), the
  link and `POST` pagers (`pager`) and the find field (`filter`), each a
  link or a form that works with no script.
- `highlight::html(language, text, lines, marked)`, `icon::icon(glyph, size)`,
  `tone::Tone`, `theme::Theme`.

## The top bar

`view::shell::topbar` draws the drawer button, the lockup, the search box
and, at the end, the status pill (`view::status::status_pill`) and the
user menu (`widget::menu::user_menu`), on one row at every width; the
search box takes a row of its own below 48rem. The top bar carries no
theme control: the theme is set in the user menu and in Your preferences.
The status pill keeps its shape at every width and shows its words from
64rem; below that the words stay its accessible name and its tooltip. The
user menu opens as an anchored menu, and below 40rem as a bottom sheet
with Cancel; a control the top bar shows from 40rem up, such as a version
choice, moves into the menu below it (`Choices`).

No component writes a `style` attribute or a `<style>` element, defines a
server function, fetches, or touches browser storage.

## Locales

Every word a component draws itself (a pager step, a dialog's Cancel, the
user menu's Sign out, an empty listing's range) comes from a Fluent catalog
under `locales/`, one directory per language. English is the one catalog
that ships, and every lookup falls back to it; a language joins as one
catalog directory, with no code change. Choose the locale once per render, at the
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
words follow the same choice. The language preference on Your preferences
(`view::preferences::language_select`) names each choice in its own
language.

## Your preferences

`view::preferences::preferences_page` draws the page behind the operator's
name: Appearance, Language and region (Reading in FerroEHR) and Keyboard,
each a route of yours. Every preference is its own cookie, named by its
`theme::Preference::COOKIE`, and posts as a plain form to the route you
name, so the page works with no WebAssembly. You keep the cookie
(`SameSite=Lax`, `Secure`, one year), return to the tab saying which one
you kept (`saved`), read them all with `theme::Preferences::read` as you
render, and write `data-theme`, `data-density`, `data-motion` and
`data-hints` on `<html>`. The library writes no cookie. Your own
preferences go inside the three tabs (`own`), built with
`preference_row`, `select_preference` and `switch_preference`.

## Licence

Business Source License 1.1, Licensor Cadasto B.V., converting to Apache 2.0
four years after each version (`LICENSE`, `NOTICE` in the repository). The
Lucide icons under `vendor/lucide/` are under the ISC licence.
