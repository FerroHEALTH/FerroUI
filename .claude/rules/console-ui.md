---
paths: ["crates/**", "app/**"]
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Console UI rules (FerroUI, and every FerroHEALTH console)

This is the one component rule set of the FerroHEALTH consoles. Each
console's own `console-ui.md` keeps only what is particular to it (its
routes, its session, its own templates) and points here. The library
renders HTML on the server from Askama templates, and the browser runs a
few plain ES modules on top (FerroUI #510, 0.6.0). The oracles for markup
and behaviour are the HTML Living Standard and CSS, for templates the Askama
book (<https://askama.readthedocs.io/>, the 0.16 the workspace pins), and
for roles, keyboard paths and contrast WCAG 2.2, WAI-ARIA 1.2 and the
Authoring Practices (`spec-adherence.md`), never memory.

## 0. The mandates (absolute)

- **Server-rendered HTML, a little plain JavaScript.** Every component is
  a Rust function or struct over an Askama template compiled into the
  library, returning `html::Html`. Behaviour is a plain ES module under
  `crates/ferrohealth-ui/scripts/`, one per job, loaded by `fui.js` for an
  element that asks with `data-fui="<job>"`. No WebAssembly, no npm, no
  bundler, no JavaScript library, no inline `<script>` body, no `on*=`
  attribute (`scripts/checks/markup-rules.sh`).
- **No inline style.** No component writes a `style` attribute or a
  `<style>` element, so every console runs `style-src 'self'` and
  `script-src 'self'` (`markup-rules.sh`,
  `crates/ferrohealth-ui/tests/it/stylesheet.rs`).
- **The library holds no route and no state.** It defines no handler,
  reads no request and touches no browser storage beyond what the tree and
  the scroll keep for the session. Data arrives as arguments; theme
  persistence, URL policy and the routes are each console's. Two modules
  fetch, each on a wire `docs/consoles.md` documents and from a route the
  console names: the top bar's typeahead (`search`) asks the answer route,
  and the session watch (`session`) asks the session route what the
  session has left, extending it only for the user's own input
  (`scripts/checks/script-budget.sh` refuses any other `fetch`).
- **No patient identifier in a URL.** A listing that can show patient data
  keeps its page, size and sort out of the URL: its forms post them
  (`StateSource::Session`, `table::pager::post_pager`).
- **No user-visible string outside the catalogs** (owner, 2026-10-19). Every
  word a component draws itself is a message of the library's Fluent
  catalog, every word a console draws is a message of the console's, and a
  new language is a catalog directory with no code change (§13).

## 1. The interaction contract

Every component follows one interaction contract (owner's rulings of
2026-10-29 on #510), so the consoles behave alike and nothing flickers:

1. **Native elements first:** `<dialog>` with `command` and `commandfor`,
   the `popover` attribute, `<details>`, plain links, and GET and POST
   forms.
2. **Every page is complete with no script.** The server draws the whole
   page with the state it read (the theme, the listing's position, the open
   tree nodes), and a link or a form is behind every action. The gallery's
   no-script journey (`tests/it/no_script.rs`) loads every page with the
   modules refused and holds its content, links, forms and popovers to it.
3. **A navigation is one fast document.** A link or a form the browser
   follows loads its page whole: no router, no fallback state, no blank
   panel between the click and the page. A post renders the page it was
   posted to, or redirects with a flash toast (post-redirect-get). The
   `keep` module keeps the scroll across a same-path navigation, and the
   `tree` module keeps a tree's folds for the session.
4. **An in-page change is a few lines of script that toggle what the server
   drew:** a tab whose panel the page holds, a tree level the server read,
   a menu, a dialog's focus, a toast's timer. No round trip, no redraw.
   The gallery's navigation journeys hold it: a tab switch loads no
   document and moves no layout, a tree's folds survive a reload, and a
   same-path link keeps the scroll.
5. **Each module stays small:** at most 16 KiB, no `eval`, no absolute
   URL, within the page and compressed caps of §12
   (`scripts/checks/script-budget.sh`).

The library has two parts:

- **`view`: the components.** A function or a struct over a template,
  taking `&Locale` where it draws words (`TreeWords::of(&locale)`) and its
  children as `Html`, so a card takes its body and the shell takes the
  page. A component that behaves writes `data-fui="<job>"` on its root and
  carries the words the module needs as data attributes or a `<template>`.
- **`scripts`: the modules**, each `enhance(element)` over what the server
  drew, written to do with the keyboard what the component's pattern in the
  Authoring Practices asks. A module answers the same as the Rust beside it
  where both decide (the tree's walk, a date's bounds, the highlighters),
  and a journey holds the two to the same answer.

A word is an `i18n::Word`, drawn inside `<span lang="en">` when it fell
back to English on a page in another language (WCAG 2.2 SC 3.1.2); a word
in an attribute takes the page's language. A count a module learns is
written with the page's `i18n::number::Grouping`, carried in its words.

## 2. Templates and markup

- A template lives under `crates/ferrohealth-ui/templates/<area>/`, its
  struct beside the component; `Html` is `HtmlSafe`, so a template writes a
  part with `{{ part }}` and never escapes it twice. Text goes through
  `Html::text` or a template's own escaping; `Html::raw` takes only markup
  the library made.
- A console cannot `{% extends %}` or `{% import %}` a library template:
  composition is by value, a part passed as `Html`.
- No component writes a `style` attribute, a `<style>` element, an inline
  script or handler, or a `javascript:` URL (`markup-rules.sh` reads the
  templates and the Rust).

## 3. Modules

- One job per module, imported by `fui.js` from its own directory; a page
  loads only the modules its components asked for.
- A module enhances what is there: it never draws content the server did
  not, and a control that works only with script is drawn `hidden` or
  `disabled` and shown or enabled by its module.
- A module keeps a control the reader holds the same element as it
  updates (the date picker's status line and buttons, the combobox's rows),
  so focus and a screen reader's place survive.
- State a module keeps for the session is keyed by the page and the
  element's id in `sessionStorage`, never `localStorage`, and never
  clinical content.

## 4. Views

- A list's items carry a stable, data-derived key, never an index.
- **An error never renders as nothing**: a refusal renders its status and the
  issues it carried (`view::refusal`), an information issue with no text is
  counted, never listed blank.
- Valid HTML only: an explicit `<tbody>`, no block inside `<p>`.
- No non-determinism in a render (random ids, timestamps).

## 5. Forms

- Every control carries an explicit, stable `id` and `name`; a label names it
  with `for`; help is named by `aria-describedby` (`view::form::help_id`).
- A `<textarea>` carries its text in the server HTML, escaped, so the
  browser decodes it as RCDATA and `</textarea>` in the text stays text.
- The sign-in form is a plain `POST` with plain inputs and the
  `username` / `current-password` autocomplete tokens.
- A form a post refuses is drawn again with what the reader typed
  (`view::form::Posted`), every field a `secret_mark` names left out
  (#499).
- A form whose field can carry clinical content is never a `GET` form.

## 6. Tables and paging (one model)

`table::model` is the arithmetic, `table::view` the data table, its cells and
the list card, `table::pager` the link pager and the `POST` pager, and
`table::filter` the find field: every sort, filter and step is a link or a
form, so a listing works with no script. Page and size are read
defensively (`?page=` 1-based, `?size=` with choices 25, 50, 100, clamped to
200), the range label names the rows on the page ("1-25 of 312", a short
last page "301-312 of 312"), a step that cannot be taken stays visible as a
disabled control and says "(unavailable)" in visually hidden text, one
pager row holds the range, the steps and the sizes, the range names the listing's noun and says
when the server counted more than it sent, a listing with no stated total
pages with Previous and Next alone (`OpenWindow`), the header is the small
step in sentence case, a sortable header is a submit button in a `GET` form
(a `POST` form for session state) with `aria-sort` on its header,
numbers align right, grouped in the page's language and never broken,
and the table shows every row of its page, never in a box capped in
height. From 64rem up the header sticks to the viewport just below the
sticky top bar, inside the scroll padding (SC 2.4.11); below 64rem, where a
table can be wider than its card, it scrolls sideways inside the card,
never the page (SC 1.4.10), and gives up the sticky header, which an
overflow box would hold to itself. A table no console bounds
(`data_table_wide`, or any table with a code header) keeps its scroll box
at every width and gives up the sticky header, so a query's result set
never widens the page. Pure CSS, no script.

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
- A brand's theme file sets seven properties: `--accent`, `--accent-hover`,
  `--accent-ink`, `--accent-tint` and `--focus` in three blocks (`:root`,
  `:root[data-theme="dark"]`, and the system's dark under
  `prefers-color-scheme`), then `--preview-accent-light` and
  `--preview-accent-dark` once in a closing `:root` block, the accents the
  theme previews in Your preferences draw whatever theme the page is in,
  each the accent of its theme. Nothing else.
- The theme is `data-theme` (`light`, `dark`, `system`) on `<html>`, written
  by the console's server from its cookie, so the first paint is right; with
  no cookie it follows the system. The top bar carries no theme control
  (family rule 5): the user menu (`view::menu::user_menu`) holds the
  three choices as `menuitemradio` submit buttons of one plain form, and
  Your preferences (`view::preferences`) holds the theme as preview cards;
  the console answers the post with the page in the new theme, kept in the
  same cookie, never browser storage. Each family preference is its own
  cookie (`theme::Preference::COOKIE`), and the console writes reduced
  motion and the shortcut hints on `<html>` as `data-motion` and
  `data-hints`, which the stylesheet reads beside
  `prefers-reduced-motion`.
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

- `cargo fmt` clean; `cargo clippy --workspace --all-targets -- -D warnings`;
  `cargo nextest run --workspace`; `markup-rules.sh`, `script-budget.sh`
  and `stray-literals.sh`.
- The gallery's journeys (`FERROUI_JOURNEYS=1`) and screenshots
  (`FERROUI_SCREENSHOTS=1`); a new component joins a gallery page, a journey
  where it is interactive, and the screenshots, which are looked at before
  the change merges and committed by the refresh pull request
  (`parallel-prs.md`).

## 11. Quality control (owner rule, 2026-10-18)

The library is held to the best console's quality, not to "it renders".

- **Best of four, one language.** A component is compared across the four
  consoles (FerroEHR, FerroFED, FerroTERM, FerroBRIDGE) and the best version
  on its merits wins: looks, density, hover and focus states, accessibility,
  keyboard path. It is then restyled to the library's one type scale, spacing
  scale, radius and shadow set, hover and focus treatment, and role tokens,
  so a piece taken from one console never looks like another app's.
- **Every interactive state is captured.** The gallery's state capture
  (`tests/it/states.rs`, one file per area under `tests/it/states/`) moves
  the pointer onto a navigation entry, a table row, each button variant, a
  linked card and a tab; gives the focus to a control, a navigation entry
  and the keyboard tabs; opens every overlay (dialogs, the modal a row's
  Delete opens, the side panel, both menus, the drawer); raises the toasts; and
  drives the upload field through empty, a file over it, a file read and a
  file refused; turns the help switch off and on, opens the picker on its
  input, walks the tree by the keyboard, focuses the search box, reveals a
  choice's fields and draws the tables in the compact density, in every
  brand, light and dark, with reduced motion so each state is drawn at rest.
  A new interactive component adds its states to its area's file there.
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
- **One card-header action.** A card or a panel that previews part of a
  listing leads to the rest through `card_action` ("All members", the
  words then an arrow a tight space apart), never a console's own link or
  outlined button.
- **An editor's header says what it does.** A form that creates titles
  itself "New <thing>" and one that changes "Edit", through
  `editor_heading`, whose trail ends with the list a new thing joins or
  with the thing edited, never with the editor (`design/COMPONENTS.md`
  T2).


## 12. Script budget

The modules are vendored in the crate and served with long cache headers
under `/ferrohealth-ui/<version>/scripts/`. A reader pays for the modules
the page in front of them loads, so the budget is measured per page, not
over every module (FerroUI #547). `scripts/checks/script-budget.sh` holds
three caps. No specification governs them: our own design.

| Cap | Limit | Measured on 2026-10-31 | Why this limit |
|---|---|---|---|
| One module (S1) | 16 KiB | `highlight.js`, 13 798 bytes | A module is one job; past 16 KiB it holds two, and it splits |
| The modules one page loads (S2) | 144 KiB | `/:brand/forms`, 129 814 bytes; then `/:brand/operations`, 77 955; a signed-in page with only the shell, 42 656; a sign-in page, 6 495 | The forms page shows every form part at once, the most any page asks for; the cap leaves it room for one more module of the full 16 KiB |
| Every module, compressed (S6) | 72 KiB | 59 745 bytes by `gzip -9` (163 756 raw) | What a reader fetches once, cached for a year, after visiting every page; it grows with every module, wherever it loads, and leaves room for about two more full modules |

What one page loads is the loader `fui.js`, the module of every job the
page's line in `scripts/checks/script-pages.list` names, and every module
those import, statically or with `import()`. The list holds every gallery
page and its jobs; the gallery's `script_pages` test walks the gallery and
fails when the list is no longer what it draws, and prints the list to
write. `script-budget.sh --report` prints what each page loads. A change
that needs more splits the module, loads it only where it is used, or
moves the work to the server; raising a cap is a decision recorded here
with its new measurement.

## 13. Strings and locales

The owner's ruling (2026-10-19): English is the one language that ships,
and the family is ready for more, so adding a language later is a catalog
file and no code change. No specification governs the catalog layout or
the lookup order: our own design; the message syntax is Project Fluent's.

- **One crate, one format.** `fluent-templates` (pinned in the workspace
  table) embeds every catalog under a crate's `locales/` directory at build
  time with `static_loader!`, one directory per BCP 47 tag
  (`locales/en/<area>.ftl`), with no runtime file reader. No hand-rolled catalog format, no second crate.
- **`en` is the reference.** It is complete, every lookup falls back to it,
  and a test holds every catalog under `locales/` to the full message set
  (`i18n::words::tests`), so a new catalog is checked with no code change.
  Only `en` ships; readiness is shown by a throwaway catalog in the tests
  (`tests/it/locales/qaa/`), never by a shipped translation.
- **The locale is chosen once per request.** The console's handler builds
  an `i18n::Locale` from its configuration or the request
  (`Locale::negotiate` reads `Accept-Language`), passes `&locale` to every
  component that draws words, and the shell writes `Locale::tag` as
  `<html lang>` (WCAG 2.2 SC 3.1.1).
- **A module's words come from the server.** A component writes the words
  its module may draw as data attributes or in a `<template>`, resolved in
  the page's locale; no catalog reaches the browser.
- **A console catalogs its own strings the same way**: its own
  `static_loader!` over its own `locales/`, looked up through
  `Locale::lookup` so its words follow the same choice, and the language
  choice is the language preference of Your preferences
  (`view::preferences::language_select`).
- **A console hands a component its words made at run time.** Every
  label, title, legend, placeholder and noun a component takes accepts a
  `String` from the console's catalog (a `Word` field takes one with
  `.into()`). Words a component must look up as
  it draws are an `i18n::Message` of the console's catalog: a listing's
  noun selects its form by `$count` (CLDR plural categories, so the library
  holds no plural rule) and carries `.empty` and `.opening` as whole
  sentences; an editor's title selects by `$mode` (`new`, `edit`). A
  column, a listing or a navigation group is built where it is drawn,
  never a `const`.
- **The gallery is held to the same rule.** Its page strings are messages
  of its own catalog (`app/ferrohealth-ui-gallery/locales/`), the
  pseudo-locale stretches them with the library's, and the stray-literal
  check covers it.
- **The stray-literal check fails a string outside the catalogs.**
  `scripts/checks/stray-literals.sh` reads the components and the templates for literals that
  read as words (an attribute a reader sees or hears, a text node, any
  literal holding words) and fails on each; a literal no reader sees as text
  (a key name, a product name, SVG markup) is allowed with its reason in
  `scripts/checks/stray-literals.allow`. A console runs the same script over
  its own crate with a baseline that only shrinks while its strings move.
- **A number is written in the page's language.** A number a message takes
  is grouped by `i18n::number::format_number`, the formatter of every
  catalog (a console sets it on its own), and a count drawn outside a
  message by `i18n::number::grouped`; the marks are each catalog's
  `number-group` and `number-decimal`.
- **The layout leaves room for longer words.** No text sits in a fixed
  width; the gallery's pseudo-locale (`qps`) draws every library string
  accented and two fifths longer, and a change that moves a component's
  layout is captured in it at 1440 and 390 px, light and dark.
