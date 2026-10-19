---
paths: ["crates/**", "app/**"]
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Leptos component rules (FerroUI, and every FerroHEALTH console)

This is the one component rule set of the FerroHEALTH consoles. Each
console's own `leptos-ui.md` keeps only what is particular to it (its server
functions, its session, its bundle budget) and points here. The oracle for
every Leptos question is the official Leptos book
(<https://github.com/leptos-rs/book>, `main`, Leptos 0.8), never memory; the
oracles for roles, keyboard paths and contrast are WCAG 2.2, WAI-ARIA 1.2 and
the Authoring Practices (`spec-adherence.md`).

## 0. The mandates (absolute)

- **Rust only, zero hand-written JavaScript.** No authored `.js`, no inline
  `<script>` bodies, no `onxxx="..."` attributes. JS-wrapping crates are
  banned; a chart is drawn in Rust as SVG.
- **No inline style.** No component writes a `style` attribute or injects a
  `<style>` element, and none relies on a CSS-in-Rust runtime, so every
  console runs `style-src 'self'` with scripts by nonce
  (`crates/ferrohealth-ui/tests/it/stylesheet.rs` holds the library to it).
- **The library defines no server function**, fetches nothing, and touches no
  browser storage. Data arrives as props; theme persistence, URL policy and
  the router base are each console's.
- **No patient identifier in a URL.** A listing that can show patient data
  keeps its page, size and sort in the page (`StateSource::Session`,
  `widget::pager::session_pager`).
- **No user-visible string outside the catalogs** (owner, 2026-10-19). Every
  word a component draws itself is a message of the library's Fluent
  catalog, every word a console draws is a message of the console's, and a
  new language is a catalog file with no code change (§13).

## 1. Two tiers

- **`view`: static view functions.** Plain functions returning a typed view,
  never a `#[component]` (a component writes a props type and a builder the
  bundle carries for each) and never taking a reactive prop. They render the
  same on the server, in a hydrated page, under client-side rendering, and
  through `to_html()` into a string a read-only screen draws with
  `inner_html`. What they need to behave they get from the browser: a
  `popover` opens and closes with no script, a `<details>` folds, a form
  posts.
- **`widget`: the interactive components**, each where interaction needs
  browser code (the drawer's focus, keyboard tabs, toasts, the session pager,
  the table filter, the highlighted editor). Browser-only calls run inside an
  event handler or an `Effect`, behind `#[cfg(target_arch = "wasm32")]`, so
  the first paint is the server's and hydration matches it.
- **A console's screens are erased sections** (`.into_any()`): a monolithic
  `view!` over nested views blows rustc's layout recursion. The library's
  `shell` and `centred` take and return `AnyView` for that reason.

## 2. Render-mode agnostic

The library compiles under `csr`, `ssr` and `hydrate` alike, with
`leptos = { default-features = false }`; the console's own Leptos
dependency turns the mode on and features unify. A console chooses its
halves by compilation target, never by a Cargo `ssr`/`hydrate` feature pair,
so a workspace `--all-features` build never enables both. The browser half
asks for the lowest `web-sys` and `wasm-bindgen-futures` the library's
Leptos resolves with and for no `wasm-bindgen` of its own, so a console's
lock picks its wasm-bindgen and the CLI that must equal it
(`scripts/checks/browser-floor.sh` builds the library at those floors).

## 3. Reactivity and props

- Component functions are setup functions and run once
  (`reactivity/interlude_functions`).
- `.get()`/`.set()` for cheap values; `.with()`/`.read()` for collections.
  Never write one signal from an `Effect` that reads another
  (`working_with_signals` §4).
- A widget takes changing values as `Signal<T>` and reports changes through a
  callback (`code_editor(Editor { text, marks, editable, .. }, on_input)`).

## 4. Views

- Dynamic lists use `<For each key>` with a stable, data-derived key, never an
  index.
- **An error never renders as nothing**: a refusal renders its status and the
  issues it carried (`view::refusal`), an information issue with no text is
  counted, never listed blank.
- Valid HTML only: an explicit `<tbody>`, no block inside `<p>`.
- No non-determinism in a render (random ids, timestamps).
- An anchor to a console's server route carries `rel="external"` (the
  sign-in), so the client router leaves it alone.

## 5. Forms

- Every control carries an explicit, stable `id` and `name`; a label names it
  with `for`; help is named by `aria-describedby` (`view::form::help_id`).
- A `<textarea>` takes its text through `prop:value` alone, never a text
  child: the server writes a child as it stands, so `</textarea>` in the text
  would end the element.
- The sign-in form is a plain `POST` with plain, uncontrolled inputs and the
  `username` / `current-password` autocomplete tokens; it works before the
  page hydrates and hydration never wipes what was typed.
- A form whose field can carry clinical content is never a `GET` form.

## 6. Tables and paging (one model)

`table::model` is the arithmetic, `table::view` the data table and the link
pager, `widget::pager` the session pager. Page and size are read
defensively (`?page=` 1-based, `?size=` with choices 25, 50, 100, clamped to
200), the range label names the rows on the page ("1-25 of 312", a short
last page "301-312 of 312"), a step that cannot be taken stays visible as a
disabled control and says "(unavailable)" in visually hidden text, one
pager row holds the range, the steps and the sizes, the range names the listing's noun and says
when the server counted more than it sent, a listing with no stated total
pages with Previous and Next alone (`OpenWindow`), the header is the small
step in sentence case, a sortable header is a link with `aria-sort`,
numbers align right, and the table shows every row of its page, never in a
box capped in height. From 64rem up the header sticks to the viewport just
below the sticky top bar, inside the scroll padding (SC 2.4.11); below
64rem, where a table can be wider than its card, it scrolls sideways inside
the card, never the page (SC 1.4.10), and gives up the sticky header, which
an overflow box would hold to itself. Pure CSS, no script.

## 7. Accessibility

The bar is **WCAG 2.2 Level AA**, in the light and the dark theme of every
brand: 4.5:1 for text and 3:1 for a control edge and the focus ring, held by
`tests/it/contrast.rs` over the role tokens and by the gallery's painted
journey over what the browser draws. No meaning rides on colour alone (a pill
is a word, the pressed theme is checked, the current page is bold with a
bar). Every control is keyboard operable with a visible focus; the tabs
follow the Authoring Practices pattern; popovers close on Escape and return
the focus; motion respects `prefers-reduced-motion`; a page reflows at
320 px with no sideways scroll; the sticky top bar never covers the focus
(SC 2.4.11). A component names a role of the stylesheet, never a palette
entry, and a new role joins the contrast test in the change that adds it.

## 8. Tokens, themes, type

- One role layer (`--surface`, `--surface-raised`, `--surface-sunken`,
  `--edge`, `--edge-control`, `--ink`, `--muted`, `--accent`, `--accent-ink`,
  `--accent-tint`, `--focus`, the status families `--ok`, `--warn`, `--bad`,
  `--info`, `--neutral` with their grounds and dots, `--syntax-*`), neutrals
  Tailwind slate, radii 0.5 rem and 0.375 rem, a card shadow and an overlay
  shadow. A chart series takes a role tone (`chart::ACCENT` to `NEUTRAL`),
  so its colour means what the tone means elsewhere: refusals are `BAD`.
- One hover and focus treatment: an interactive surface takes the sunken
  ground under the pointer (a navigation entry, a ghost or secondary button, a
  tab, a segment, a menu item, a table row at 60%), a filled control its
  hover shade, a card that is a link its accent edge; every control shows the
  2 px focus ring in `--focus`, 2 px out.
- A brand's theme file sets `--accent`, `--accent-hover`, `--accent-ink`,
  `--accent-tint` and `--focus` in three blocks: `:root`,
  `:root[data-theme="dark"]`, and the system's dark under
  `prefers-color-scheme`. Nothing else.
- The theme is `data-theme` (`light`, `dark`, `system`) on `<html>`, written
  by the console's server from its cookie, so the first paint is right; with
  no cookie it follows the system. Every top bar carries the one theme
  toggle (`theme_toggle`), a moon in light and a sun in dark, which posts
  with no script and, hydrated, swaps the theme in place while the console
  persists it in the same cookie; never browser storage. The three-way
  choice belongs in a settings form.
- Six steps of type (`--text-display` to `--text-micro`), each written
  with its leading (`--leading-display` to `--leading-micro`), and four of
  space (`--space-tight` to `--space-section`), the compact density under
  `data-density="compact"`; the system font stack, no web font. The
  Tailwind theme is a `reference` block, so it never writes a variable that
  names itself.

## 9. Icons

Lucide alone, vendored verbatim (`vendored-inputs.md`) and drawn by
`icon(glyph, size)` with `aria-hidden="true"`; no icon crate. An icon never
carries meaning alone: it sits beside its word.

## 10. Testing and gates

- `cargo fmt` clean; `cargo clippy --workspace --all-targets -- -D warnings`
  on the host and `cargo clippy -p ferrohealth-ui --lib --target
  wasm32-unknown-unknown --features leptos/hydrate -- -D warnings` (and
  `leptos/csr`); `cargo nextest run --workspace`.
- The gallery's journeys (`FERROUI_JOURNEYS=1`) and screenshots
  (`FERROUI_SCREENSHOTS=1`); a new component joins a gallery page, a journey
  where it is interactive, and the refreshed screenshots, which are looked at
  before the change merges.

## 11. Quality control (owner rule, 2026-10-18)

The library is held to the best console's quality, not to "it renders".

- **Best of four, one language.** A component is compared across the four
  consoles (FerroEHR, FerroFED, FerroTERM, FerroBRIDGE) and the best version
  on its merits wins: looks, density, hover and focus states, accessibility,
  keyboard path. It is then restyled to the library's one type scale, spacing
  scale, radius and shadow set, hover and focus treatment, and role tokens,
  so a piece taken from one console never looks like another app's.
- **Every interactive state is captured.** The gallery's state capture
  (`tests/it/states.rs`) moves the pointer onto a navigation entry, a table
  row, each button variant, a linked card and a tab; gives the focus to a
  control, a navigation entry and the keyboard tabs; opens every overlay
  (dialogs, the modal a signal opens, the side panel, both menus, the
  drawer); raises the toasts; and drives the upload field through empty, a
  file over it, a file read and a file refused; turns the help switch on,
  opens the picker on its input, walks the tree by the keyboard, focuses the
  search box, reveals a choice's fields and draws the tables in the compact
  density, in every brand, light and dark, with reduced motion so each
  state is drawn at rest. A new interactive component adds its states
  there.
- **Side by side with the reference.** `docs/qa/` pairs the reference
  consoles' screenshots with the gallery's, screen by screen; a component that
  changes refreshes its pair.
- **Looked at before a release.** The release pull request says, per screen
  and per state, what was checked and what was fixed, and the owner reviews
  the screenshots before the tag.
- **One owner of the library.** Anything a console needs that the library
  lacks is added here, generic, documented, in the gallery for every brand
  light and dark, tested and contrast-checked; never a single-use component
  inside a console.

## 12. Bundle

A library component reaches every console's bundle, so a new browser-side
dependency or a heavy generic is justified against the bytes it adds. The
gallery, which carries every component, is held to a budget in CI:
`scripts/checks/gallery-bundle.sh` fails when its brotli-compressed
WebAssembly is over the number below, which the script reads from this table.

| Measure | Budget |
|---|---|
| WebAssembly, brotli-compressed | 655360 bytes |

The budget is 640 KiB, 6% over the 620282 bytes the 0.1.6 gallery
measured (2026-10-19, cargo-leptos 0.3.11, wasm-bindgen 0.2.129,
Rust 1.98.1), which was 14039 bytes over 0.1.5's 606243 and 5882 over the
600 KiB budget before it. 0.1.6 adds the field shapes every console needs
so none builds a field by hand (a label made at run time, a hidden label,
an error line, an action with its error line, a field that switches
between a picker and a typed value), and the gallery draws each of them.
0.1.1 measured 518566 bytes and 0.1.0 325387; a console's own bundle
carries only the components it uses. A change
that needs more raises it here, in its own pull request, with the measured
size and the reason.

## 13. Strings and locales

The owner's ruling (2026-10-19): English is the one language that ships,
and the family is ready for more, so adding a language later is a catalog
file and no code change. No specification governs the catalog layout or
the lookup order: our own design; the message syntax is Project Fluent's.

- **One crate, one format.** `fluent-templates` (pinned in the workspace
  table) embeds every catalog under a crate's `locales/` directory at build
  time with `static_loader!`, one directory per BCP 47 tag
  (`locales/en/ferrohealth-ui.ftl`), on both halves, with no script and no
  runtime file reader. No hand-rolled catalog format, no second crate.
- **`en` is the reference.** It is complete, every lookup falls back to it,
  and a test holds every catalog under `locales/` to the full message set
  (`i18n::tests`), so a new catalog file is checked with no code change.
  Only `en` ships; readiness is shown by a throwaway catalog in the tests
  (`tests/it/locales/qaa/`), never by a shipped translation.
- **The locale is chosen once per render.** The console builds an
  `i18n::Locale` from its configuration or the request
  (`Locale::negotiate` reads `Accept-Language`), hands it to
  `i18n::provide_locale` at the root of its page tree on the server and in
  the hydrating browser alike (a value the server decided reaches the
  browser serialized, `SharedValue`), and writes `Locale::tag` as
  `<html lang>` (WCAG 2.2 SC 3.1.1). A page that provides none draws
  English.
- **A component reads its words while the page tree is in reach.** A view
  function calls the crate's lookup as it builds its view; a widget reads
  every word it may draw at setup and moves the strings into its handlers
  and tasks, where no owner is in reach.
- **A console catalogs its own strings the same way**: its own
  `static_loader!` over its own `locales/`, looked up through
  `Locale::lookup` so its words follow the same choice, and the language
  switch is `view::theme_switch::locale_switch`.
- **The stray-literal check fails a string outside the catalogs.**
  `scripts/checks/stray-literals.sh` reads the components for literals that
  read as words (an attribute a reader sees or hears, a text node, any
  literal holding words) and fails on each; a literal no reader sees as text
  (a key name, a product name, SVG markup) is allowed with its reason in
  `scripts/checks/stray-literals.allow`. A console runs the same script over
  its own crate with a baseline that only shrinks while its strings move.
- **The layout leaves room for longer words.** No text sits in a fixed
  width; the gallery's pseudo-locale (`qps`) draws every library string
  accented and two fifths longer, and a change that moves a component's
  layout is captured in it at 1440 and 390 px, light and dark.
