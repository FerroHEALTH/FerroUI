<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# ferrohealth-ui

The shared UI components of the FerroHEALTH consoles (FerroBRIDGE, FerroFED,
FerroEHR, FerroTERM and FerroPIX): components rendered on the server from
Askama templates, small plain ES modules for the behaviour on top, one table
model, highlighters for YAML, JSON, XML and AQL, Lucide icons, and one
plain-CSS stylesheet with a theme file per brand. Every pairing of every
theme is held to WCAG 2.2 AA by the crate's tests.

```toml
[dependencies]
ferrohealth-ui = "=0.6.3"
```

Pin the version exactly: a console moves its pin after looking at its own
screens. The [console guide](https://github.com/FerroHEALTH/FerroUI/blob/main/docs/consoles.md)
says what the library owns, what a console owns, and which document governs
each part. A console on 0.5 moves with the
[migration guide](https://github.com/FerroHEALTH/FerroUI/blob/main/docs/migration-0.6.md).

## A page

Every component is a function that returns `html::Html`, markup your own
Askama template writes with `{{ part }}` (it implements `HtmlSafe`, so it is
never escaped twice). A handler draws its page and returns the whole
document:

```rust
use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Locale;
use ferrohealth_ui::serve::{Assets, PREFIX, axum_router};
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::theme::Preferences;
use ferrohealth_ui::view::shell::{Document, document};

let app: axum::Router = axum::Router::new()
    // .route("/templates", get(templates)) and the rest of your pages
    .nest(PREFIX, axum_router::router());

let page = document(Document {
    locale: Locale::english(),
    preferences: Preferences::default(),
    title: String::from("Templates · FerroBRIDGE"),
    assets: Assets::of(Brand::Bridge),
    nonce: None,
    head: Html::empty(),
    body: Html::text("the page"),
});
```

`serve::axum_router::router()`, under the default `axum` feature, serves the
stylesheet, the brand themes and the script modules under
`/ferrohealth-ui/<version>/` with a one-year immutable cache. Without axum,
serve `style::STYLESHEET`, `style::Brand::theme` and `script::MODULES` at the
paths `serve::Assets` names.

The document loads one script, the loader `fui.js`, which imports the module
each element asks for with `data-fui="<job>"`. Every page works with the
modules refused: the HTML is complete, and every important action is a link
or a form. Your policy is `script-src 'self'; style-src 'self'`; a nonce is
needed only when your own policy keeps one (`Document::nonce`).

## The stylesheet and a theme

The document links two stylesheets from your own origin, the theme after
the stylesheet, so `style-src 'self'` holds. The asset router serves both;
without it, serve the text yourself:

```rust
use ferrohealth_ui::style::{Brand, STYLESHEET};

let stylesheet: &str = STYLESHEET;          // at Assets::stylesheet()
let theme: &str = Brand::Bridge.theme();    // at Assets::theme()
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

- `view`: every component, each a function over a template taking the
  page's `&Locale` where it draws words. Among them: `button`, `status`
  (pill, badge, banner, the top bar's status pill), `surface` (page header,
  card, stat, definitions, disclosure), `form` (field, check), `tabs`
  (binder tabs joined to their panel, keyboard tabs), `popover`, `empty`,
  `refusal` (an error with its issues), `code` and `editor` (the
  highlighted panel and the overlay editor with line marks), `dialog`,
  `nav` (the sidebar, plain links with the page marked, its counts and its
  rail, inside its drawer), `menu` (the user menu), `action_menu`, `tree`,
  `date`, `combobox`, `search`, `toast` (the region, and a `Flash` the
  server raises as the page arrives), `session` (the watch over the
  session's end), `preferences` (Your preferences), `shell` (the document,
  skip link, top bar, footer, centred page) and `sign_in` (the R19 card in
  every state, with the provider's action and an optional basic form).
- `script`: the modules as text, one per job, and the loader.
- `table`: the paging and sorting model (`Paging`, `Window`, `Sort`,
  `StateSource`), the data table with its cells and list card (`view`), the
  link and `POST` pagers (`pager`) and the find field (`filter`), each a
  link or a form that works with no script.
- `highlight::html(language, text, lines, marked)`, `icon::icon(glyph, size)`,
  `tone::Tone`, `theme::Theme`.

## The top bar

`view::shell::topbar` draws the drawer button, the lockup, the search box
and, at the end, the status pill (`view::status::status_pill`) and the
user menu (`view::menu::user_menu`), on one row at every width; the
search box takes a row of its own below 48rem. The top bar carries no
theme control: the theme is set in the user menu and in Your preferences.
The status pill keeps its shape at every width and shows its words from
64rem; below that the words stay its accessible name and its tooltip. The
user menu opens as an anchored menu, and below 40rem as a bottom sheet
with Cancel; a control the top bar shows from 40rem up, such as a version
choice, moves into the menu below it (`Choices`).

No component writes a `style` attribute, a `<style>` element or an inline
script, defines a route, or touches browser storage beyond what the tree
and the scroll keep for the session. Three modules fetch, from routes you
name: the search box's typeahead (`search`), the session watch
(`session`) and a form answered in place (`answer`, `view::answer`), each
on the wire `docs/consoles.md` documents.

## Locales

Every word a component draws itself (a pager step, a dialog's Cancel, the
user menu's Sign out, an empty listing's range) comes from a Fluent catalog
under `locales/`, one directory per language. English is the one catalog
that ships, and every lookup falls back to it; a language joins as one
catalog directory, with no code change. Choose the locale once per request
and pass it to every component; the document writes it as `<html lang>`:

```rust
use ferrohealth_ui::i18n::Locale;

// From the configuration, or over `Accept-Language`.
let locale = Locale::negotiate("nl-NL, en-GB;q=0.8");
assert!(!locale.tag().is_empty());
```

Catalog the console's own strings the same way: a
`fluent_templates::static_loader!` over the console's own `locales/`,
looked up with `Locale::lookup`, so the console's words follow the same
choice. A module carries no catalog: the words it needs are resolved on the
server and drawn into the page. The language preference on Your preferences
(`view::preferences::language_select`) names each choice in its own
language.

## Your preferences

`view::preferences::preferences_page` draws the page behind the operator's
name: Appearance, Language and region (Reading in FerroEHR) and Keyboard,
each a route of yours. Every preference is its own cookie, named by its
`theme::Preference::COOKIE`, and posts as a plain form to the route you
name, so the page works with no script. You keep the cookie
(`SameSite=Lax`, `Secure`, one year), return to the tab saying which one
you kept (`saved`), read them all with `theme::Preferences::read` as you
render, and write `data-theme`, `data-density`, `data-motion` and
`data-hints` on `<html>`. The library writes no cookie. Your own
preferences go inside the three tabs (`own`), built with
`preference_row`, `select_preference`, `switch_preference` and, for a
value the person types, `typed_preference` (a text or a number field,
`Typed::Text` or `Typed::Number` with its bounds), which posts in the same
form when the field changes and gives its field the focus back once kept.

## Licence

Business Source License 1.1, Licensor Cadasto B.V., converting to Apache 2.0
four years after each version (`LICENSE`, `NOTICE` in the repository). The
Lucide icons under `vendor/lucide/` are under the ISC licence.
