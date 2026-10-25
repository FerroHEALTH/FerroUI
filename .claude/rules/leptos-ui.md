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
(<https://github.com/leptos-rs/book>, `main`, Leptos 0.8), never memory, and
where the book lags behind 0.8, the Leptos 0.8 source the consoles resolve
and the leptos repository's examples at the matching tag (§14); the
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
  the router base are each console's. The one fetch is the top bar's
  typeahead (owner, 2026-10-25): it asks the answer route the console names
  (`SearchBox::suggest`) on the wire `docs/consoles.md` documents, and the
  console's route reads its own data.
- **No patient identifier in a URL.** A listing that can show patient data
  keeps its page, size and sort out of the URL: its forms post them
  (`StateSource::Session`, `table::pager::post_pager`).
- **No user-visible string outside the catalogs** (owner, 2026-10-19). Every
  word a component draws itself is a message of the library's Fluent
  catalog, every word a console draws is a message of the console's, and a
  new language is a catalog directory with no code change (§13).

## 1. The interaction contract and the three parts

Every component follows one interaction contract (owner, 2026-10-22,
#196; the mode rewritten by the owner's ruling on #410, 2026-10-25), so the
consoles behave alike and no interaction reloads the page:

1. **Native elements first:** `<dialog>`, the `popover` attribute,
   `<details>`, plain links, and GET and POST forms.
2. **Behaviour in Rust compiled to WebAssembly** (§15): menus and sheets,
   keyboard tabs, the date picker, toasts with actions, the reveal
   countdown, live polling, topology pan and zoom, work-list focus, the
   search panel, and the YAML and AQL editor.
3. **Interactions stay in the page, with no document reload:** the
   console's router follows a link and submits a form in the page (§15.2).
   A plain link or post that loads a new document is the fallback while the
   WebAssembly has not loaded, never the normal path, so complete
   server-rendered HTML and a link or a form stay behind every important
   action. The gallery's no-WebAssembly journey (`tests/it/no_wasm.rs`)
   loads every page with the bundle refused and holds its content, links,
   forms and popovers to it.
4. **No hand-written JavaScript and no npm:** only the `wasm-bindgen` glue
   and the loader Leptos writes, no JavaScript library, and the editor is a
   light Rust editor. The strict CSP and the budget (§12) stay.

**The mode is the whole page hydrated, split by route** (book
`ssr/22_life_cycle`, `deployment/binary_size` "Code Splitting"; decided on
#410). Islands mode (#196) was set aside by the owner's ruling that a reload
on every click is not acceptable; the Leptos 0.8.22 islands router never
diffs an island's children or props and has no fallback for a failed
navigation (`crates/ferrohealth-ui/CLAUDE.md`). A console turns on
`hydrate` in its browser half, calls `leptos::mount::hydrate_lazy(App)`
from its browser entry, writes `<HydrationScripts options/>` in its shell,
draws every page as a `LazyRoute` and builds with `cargo leptos build
--split` (§15.1). The library has three parts:

- **`view`: view functions.** Plain functions returning a typed view, never
  a `#[component]` (a component writes a props type and a builder the
  bundle carries for each) and never taking a reactive prop. The server
  draws them, and the browser draws them again as the page hydrates or the
  router draws the next page. What they need to behave they get from the
  browser: a `popover` opens and closes with no script, a `<details>`
  folds, a form posts.
- **`island`: the ready components**, each a small `#[component]` over its
  widget function that owns its state and takes its words as props: the
  toast region, the navigation drawer, the keyboard tabs, the search box.
  The module keeps its name from islands mode.
- **`widget`: the interactive components as functions** a page calls: one
  of the library's own, or a console's own component where the widget
  shares state with the console's code (an editor and the check that runs
  as you type, a filter and the list it filters). Browser-only calls run
  inside an event handler or an `Effect`, behind
  `#[cfg(target_arch = "wasm32")]`, so the first paint is the server's and
  hydration matches it.
- **A console's screens are erased sections** (`.into_any()`): a monolithic
  `view!` over nested views blows rustc's layout recursion. The library's
  `shell` and `centred` take and return `AnyView` for that reason.

Every widget takes the words it draws as props (§15.4): a `*Words` struct
whose `of_page()` resolves them in the page's locale, and an `i18n::Phrase`
for a sentence around a value the browser learns. A word is an
`i18n::Word`, drawn inside `<span lang="en">` when it fell back to English
on a page in another language (WCAG 2.2 SC 3.1.2); a word in an attribute
takes the page's language. A count a widget learns is written with the
page's `i18n::number::Grouping`, carried in its words.

## 2. Render-mode agnostic

The library compiles under `csr`, `ssr` and `hydrate` alike, with
`leptos = { default-features = false }`; the console's own Leptos
dependency turns the mode on and features unify. A console never turns on
`islands`: a client render builds no island children (tachys 0.2
`src/html/islands.rs`), so a page the router draws would lose them. A console chooses its
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
- A `<textarea>` carries its text in the server HTML and takes the live
  text through `prop:value` (book `view/05_forms`, "`<textarea>`"). The
  server writes a textarea's children unescaped (tachys 0.2.19
  `src/html/element/elements.rs`, `textarea ... false`), so the initial text
  is `inner_html` of text escaped with `highlight::escape`, never a raw
  child; the browser decodes it as RCDATA, so `</textarea>` in the text
  stays text. A field with no server text is empty for a page whose
  WebAssembly has not loaded (#205).
- The sign-in form is a plain `POST` with plain, uncontrolled inputs and the
  `username` / `current-password` autocomplete tokens; it works before the
  page hydrates and hydration never wipes what was typed.
- A form whose field can carry clinical content is never a `GET` form.

## 6. Tables and paging (one model)

`table::model` is the arithmetic, `table::view` the data table, its cells and
the list card, `table::pager` the link pager and the `POST` pager, and
`table::filter` the find field: every sort, filter and step is a link or a
form, so a listing works with no WebAssembly too. Page and size are read
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
  (family rule 5): the user menu (`widget::menu::user_menu`) holds the
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

- `cargo fmt` clean; `cargo clippy --workspace --all-targets -- -D warnings`
  on the host and `cargo clippy -p ferrohealth-ui --lib --target
  wasm32-unknown-unknown --features leptos/hydrate -- -D warnings` (and
  `leptos/csr`); `cargo nextest run --workspace`.
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
  and the keyboard tabs; opens every overlay (dialogs, the modal a signal
  opens, the side panel, both menus, the drawer); raises the toasts; and
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

## 12. Bundle

A library component reaches every console's bundle, so a new browser-side
dependency or a heavy generic is justified against the bytes it adds. The
gallery, which carries every component, is held to a budget in CI:
`scripts/checks/gallery-bundle.sh` fails when its brotli-compressed entry
WebAssembly is over the number below, which the script reads from this
table. The entry is what every page loads before its route's code; each
page is a lazy route (§15.1), cut out of the entry by `cargo leptos build
--split` and loaded from its own chunks, ahead of the visit by
`widget::prefetch`. The script prints all the chunks together beside the
entry, so a chunk that grows is seen.

| Measure | Budget |
|---|---|
| Entry WebAssembly, brotli-compressed | 491520 bytes |

The budget is 480 KiB, 34% over the 365376 bytes the gallery measured once
every island took its words as props and the Fluent loader and catalogs
left the browser (#219, 2026-10-22, cargo-leptos 0.3.11, wasm-bindgen
0.2.129, wasm-opt version_133, Rust 1.99.1; 402593 before on the same
tree). The v0.3.1 islands took the gallery to 588808 bytes in one piece
(#262). The masked identifier and the editor's diagnostics list looked
their words up in the browser, which linked the Fluent loader and the whole
catalog again (49840 bytes). A build without each family showed what it
cost: the date pickers 72105, the four mapping editors 51074, the masked
identifiers 33665 and the concept tree 22285. With the words back as props
and those eight islands lazy, the entry is 363730 bytes and the chunks
294040, with the pair input of #259 in the entry. The forms page, which
draws seven of them, loads 257104 bytes of chunks beside the entry, the
layouts page the concept tree's 41313, and every other page none. The combobox, the pick-or-type field and the upload
field stay in the entry: lazy, they cut 43 KB from every page and added
46 KB to the forms page, in chunks that compress worse alone. The budget
stayed where it was.
With the whole page hydrated (#410) every page is a lazy route: the entry is
465338 bytes, what the islands entry measured with the search panel
(465420), and each page's code, 1846251 bytes of chunks for all of them,
loads with its route, ahead of the visit by `widget::prefetch`. Hydrated
whole with no split the gallery was 1557224 bytes. The budget stayed where
it was.
The gallery's measures in bytes, the first three rows the spike of #196
(the same gallery both ways):

| Build | WebAssembly raw | gzip -9 | brotli -q 11 | JS glue raw / brotli |
|---|---:|---:|---:|---:|
| Whole page hydrated (0.2.2) | 2455510 | 943813 | 685903 | 25423 / 6276 |
| Islands, spike (whole sections) | 1527677 | 613353 | 460744 | 25391 / 6325 |
| Islands, as landed | 1389712 | 564473 | 427449 | 22469 / 5817 |
| Islands, words as props (#219) | 1229704 | 486326 | 365376 | 23304 / 5993 |
| v0.3.0 (c429eea) | 1668471 | 648775 | 474239 | 26531 / 6729 |
| v0.3.1 islands, one piece (b742acc) | 2071614 | 809209 | 588808 | 28335 / 7123 |
| Words as props again (#262) | 1934809 | 744438 | 538968 | 28337 / 7090 |
| Entry with eight lazy islands (#262) | 1259810 | 486607 | 363730 | 29138 / 7285 |
| Islands with the search panel (v0.4.0) | 1604276 | 629196 | 465420 | 34813 / 8434 |
| Whole page, no split (#410 spike) | 5795592 | 2211988 | 1557224 | 34461 / 8189 |
| Whole page, every page a lazy route (#410) | 1495980 | 599923 | 465338 | 36651 / 8352 |

Time to interactive on the forms page, Chrome for Testing with the CPU
slowed 4x (the mid-tier phone rate) and the bundle served locally, the
median of seven loads: 315 ms hydrated, 277 ms in islands. On a 1.6 Mbps
link the brotli download alone is about 3.4 s hydrated and 2.3 s in
islands, which is where a phone feels the difference. The budget before
islands was 700 KiB over the 673607 bytes the 0.1.7 gallery measured,
the Fluent catalogs of 0.1.7 included; 0.1.5 measured 606243, 0.1.1
518566 and 0.1.0 325387.

**A console keeps its own budget**, in its own `leptos-ui.md`, over its own
bundle, which carries only the islands it draws: FerroBRIDGE's is 458752
bytes of brotli-compressed WebAssembly. A change that needs more raises
the budget here, in its own pull request, with the measured size and the
reason.

## 13. Strings and locales

The owner's ruling (2026-10-19): English is the one language that ships,
and the family is ready for more, so adding a language later is a catalog
file and no code change. No specification governs the catalog layout or
the lookup order: our own design; the message syntax is Project Fluent's.

- **One crate, one format.** `fluent-templates` (pinned in the workspace
  table) embeds every catalog under a crate's `locales/` directory at build
  time with `static_loader!`, one directory per BCP 47 tag
  (`locales/en/<area>.ftl`), on both halves, with no script and no
  runtime file reader. No hand-rolled catalog format, no second crate.
- **`en` is the reference.** It is complete, every lookup falls back to it,
  and a test holds every catalog under `locales/` to the full message set
  (`i18n::words::tests`), so a new catalog is checked with no code change.
  Only `en` ships; readiness is shown by a throwaway catalog in the tests
  (`tests/it/locales/qaa/`), never by a shipped translation.
- **The locale is chosen once per render.** The console builds an
  `i18n::Locale` from its configuration or the request
  (`Locale::negotiate` reads `Accept-Language`), hands it to
  `i18n::provide_locale` at the root of its page tree on the server, and
  writes `Locale::tag` as `<html lang>` (WCAG 2.2 SC 3.1.1). A page that
  provides none draws English. The browser draws the page too, so the
  console provides the same locale in its browser half: a `SharedValue` of
  the tag at the root carries the server's choice (§15.4).
- **A component reads its words while the page tree is in reach.** A view
  function calls the crate's lookup as it builds its view; a widget reads
  every word it may draw at setup and moves the strings into its handlers
  and tasks, where no owner is in reach.
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
  `scripts/checks/stray-literals.sh` reads the components for literals that
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

## 14. Leptos the book's way

Every rule below says what to do, why, where the book and the Leptos 0.8
source say so, and what holds it: **[check Ln]** is a finding of
`scripts/checks/leptos-rules.sh` (CI job `leptos-rules`; an exception is a
line of `scripts/checks/leptos-rules.allow` with its reason and issue),
**[compiler]** is a build or `-D warnings` failure, **[test]** names the
test, and **[review]** is a reviewer's call, which `/leptos-check` walks.
Book paths are files under the book's `src/`; source paths are files of
the crate at the version the workspace pins (leptos 0.8.22, leptos_macro
0.8.19, leptos_server 0.8.8, tachys 0.2.19, reactive_graph 0.2.16,
server_fn 0.8.13). Where they disagree, the source decides and the
difference is named.

### 14.1 Components and props

- **A changing input is a `Signal<T>` taken with `#[prop(into)]`, and a
  constant is `Signal::stored(v)`.** `Signal` holds a plain value, any
  readable signal and `Signal::derive(f)`, so one prop type serves every
  caller (book `view/03_components`, "`into` Props"); `MaybeSignal` is
  deprecated for it (reactive_graph `src/wrappers.rs`). A
  `Signal::derive(|| constant)` builds a closure for a value that never
  changes. [check L7; compiler for `MaybeSignal`]
- **Optional props say their default.** `#[prop(optional)]` is
  `Default::default()`, `#[prop(default = x)]` names it; an optional closure
  is `Option<Callback<T>>`, never an optional generic, which does not
  compile (book `view/03_components`). [compiler]
- **A generic used by no prop is `#[prop(marker)] PhantomData<T>`**, which
  leaves the builder and the docs (book `view/03_components`, "`marker`
  Props"). [review]
- **Every component and every prop has a doc comment**: the macro writes them
  onto the props builder (book `view/03_components`, "Documenting
  Components"). [compiler: `missing_docs`]
- **Attributes a caller adds land where the component means them.** By
  default they spread onto every top-level element of the view; a component
  that wants them on an inner element uses `AttributeInterceptor` (book
  `view/03_components`, "Spreading Attributes"; leptos
  `src/attribute_interceptor.rs`). [review]
- **Child to parent is a callback or an `on:` listener; context is a
  newtype.** A `WriteSignal` handed down lets any child mutate the parent's
  state; context is found by type at run time, so a bare `bool` or
  `String` context collides (book `view/08_parent_child`). `<Provider>`
  provides without shadowing a same-type context of the caller (leptos
  `src/provider.rs`). [review]
- **Children:** `Children` once, `ChildrenFn` when rendered more than once,
  `ChildrenFragment` to wrap each child, `#[slot]` for typed children, and
  `StoredValue` to project `ChildrenFn` through nested components (book
  `view/09_component_children`, `interlude_projecting_children`). [compiler]

### 14.2 Reactivity

- **A component body runs once.** Anything that changes is a signal or a
  closure that reads one; `{x.get()}` in a view is a value drawn once (book
  `view/01_basic_component`, `reactivity/interlude_functions`). [review]
- **Read a collection by reference.** `.get()` clones the whole value, so
  `rows.get().len()` copies every row to count them; use `.read()` or
  `.with()`, and `.write()` or `.update()` to change one in place (book
  `reactivity/working_with_signals`). [check L4]
- **Never hold a read guard across a write**: it panics at run time (book
  `reactivity/working_with_signals`, "Getting"). [review]
- **A value that follows another is derived, never written by an effect.** B
  from A is a closure or a `Memo`; two values changed together are two
  `set`s in the handler. An `Effect` that reads A and writes B is "officially
  discouraged": two passes through the graph and a route to loops (book
  `reactivity/working_with_signals`, "If you really must...",
  `reactivity/14_create_effect`). [review]
- **A `Memo` is for an expensive computation or the step before an expensive
  effect.** It costs a `PartialEq`, a node and a traversal, more than a cheap
  closure (book `appendix_reactive_graph`, "Memos vs. Derived Signals").
  [review]
- **An `Effect` synchronizes with the world outside the graph** (the DOM, a
  focus, a browser API) and runs in the browser only, a tick after mount;
  `Effect::watch(deps, handler, immediate)` when the handler needs the
  previous value or must stop (book `reactivity/14_create_effect`). [review]
- **`Send + Sync` signals; the `_local` forms only for a `!Send` browser
  value** (book `reactivity/working_with_signals`, "Thread Safety"). [review]
- **Signals live as long as their owner.** A signal made in a branch and
  stored above it is read after disposal (a panic); a signal made high and
  kept in a removed row leaks. Make it where it lives, or use `ArcRwSignal`
  for a collection of signals; `on_cleanup` is the unmount hook (book
  `appendix_life_cycle`). [review]
- **State escalates in this order:** local, the URL, a signal in context, a
  `Store` in context (book `15_global_state`; reactive_stores 0.4). [review]

### 14.3 Views

- **A list that changes is `<For each key children>` with a stable key from
  the data, never an index**; an index is `<ForEnumerate>`; a row's field
  that changes in place is a nested signal, a per-row `Memo` under
  `<ForEnumerate>` or a keyed `Store` field (book `view/04_iteration`,
  `view/04b_iteration`). [review]
- **An expensive branch is behind `<Show when fallback>`**, which memoizes
  the condition; `<ShowLet>` for an `Option` (leptos `src/show_let.rs`);
  divergent branches are `Either`/`EitherOf3` or `.into_any()` (book
  `view/06_control_flow`). [review]
- **`inner_html` only for markup the library escaped itself**, in the files
  the check lists, each with an escaping test; it does not escape (book
  `view/02_dynamic_attributes`, "Injecting Raw HTML"). [check L6; test:
  the escaping tests of `highlight` and `view::chart`]
- **A dynamic class is `class:name=` or `class=("name", f)`**; a dynamic
  style is forbidden (§0). [check L9; test:
  `stylesheet::the_layer_order_comes_first_and_no_style_is_inline`]
- **Valid HTML only** (§4): hydration walks the DOM, so a block in a `<p>`
  or a `<table>` without `<tbody>` breaks it (book `ssr/24_hydration_bugs`).
  [review]

### 14.4 Forms and inputs

- **A controlled input is `prop:value` (or `prop:checked`) with
  `on:input:target`, or `bind:value`, `bind:checked`, `bind:group`**; the
  `value` attribute is the initial value only (book `view/05_forms`).
  [review]
- **A `<select>` is controlled by `prop:value` on the select**, never by a
  reactive `selected` on its options, which sets the initial choice only
  (book `view/05_forms`, "`<select>`"; #206). [review]
- **A `<textarea>` carries its escaped text in the server HTML** (§5, #205).
  [review]
- **Native submission is the floor.** A form posts with no WebAssembly;
  an inline handler such as the book's `oninput="this.form.requestSubmit()"`
  (book `router/20_form`) is JavaScript the strict CSP refuses. [check L2]
- **Parse input yourself**: `type="number"` is not validation (book
  `view/07_errors`). [review]

### 14.5 Errors and async data (a console's side)

The library loads nothing (§0) [check L3]; these bind a console and the
gallery.

- **A `Resource` is the default, and its fetcher reads no signal**: the
  source is tracked and memoized, the value serialized once (book
  `async/10_resources`). A `LocalResource` where a `Resource` works is a
  deoptimization (book `ssr/23_ssr_modes`). [review]
- **Await resources in `move || Suspend::new(async move { .. })` under
  `<Suspense>`, and under `<Transition>` once the page is up**, so a reload
  keeps the old view (book `async/11_suspense`, `async/12_transition`).
  Never create a resource inside a `Suspend`, and never set a signal from a
  resource's fetcher: the fetcher does not run during hydration, so the two
  halves differ (book `server/28_async_quick_reference`). [review]
- **An error renders where it arrives** (§4); an `<ErrorBoundary>` is the
  net for an `Err` drawn in a view (book `view/07_errors`), and a server
  function's error is a typed `FromServerFnError` enum (book
  `server/25_server_functions`, "Using Custom Errors"). [review]
- **A primary content that must read with no script is
  `Resource::new_blocking` on a `SsrMode::PartiallyBlocked` route, or
  `SsrMode::Async`; out-of-order streaming only for secondary parts.**
  Out-of-order fragments arrive in a `<template>` an inline script swaps in
  (book `ssr/23_ssr_modes`, `progressive_enhancement/README`). [review]
- **The streaming and hydration scripts carry the nonce**: `leptos` with
  `nonce` on the server half, and the request's nonce provided to the render
  (leptos `src/nonce.rs`; the gallery's `server::provide_request_nonce`).
  [test: the gallery's CSP tests]

### 14.6 Server functions and actions (a console's side)

- **A server function is a public endpoint** and checks the session
  itself (book `server/25_server_functions`, "An Important Note about
  Security"). [review]
- **No `usize` or `isize` in a server function's arguments or result, or in
  any serialized type** (book `server/25_server_functions`, "Quirks").
  [review; check L8]
- **A mutation is a `ServerAction`, and the default form is `<ActionForm>`**
  (§15.6): it runs in the page, and it posts with no WebAssembly, returns
  through the form redirect, and shows an error from `__err` (book
  `progressive_enhancement/action_form`; server_fn 0.8.13 `src/lib.rs`,
  `form-redirects`). Its fields are named after the function's arguments,
  nested as `arg[field]`, with the default `PostUrl` encoding only (leptos
  `src/form.rs`). [review]
- **Client validation runs in `on:submit:capture`** with `F::from_event`
  (book `progressive_enhancement/action_form`). [review]
- **Request data comes through extractors and context**:
  `leptos_axum::extract`, `leptos_routes_with_context`, `ResponseOptions`,
  `leptos_axum::redirect` (book `server/26_extractors`, `server/27_response`).
  [review]

### 14.7 Router and navigation

- **One `<Router>` and `<Routes fallback>` on the server**, a real 404 that
  sets its status through `ResponseOptions`, and nested routes as layout with
  `<Outlet/>` (book `router/16_routes`, `router/17_nested_routing`). The
  router also supplies the `__err` of a no-script form (§15.6). [review]
- **Navigation is `<a>` and `<form>`, which the router follows in the
  page.** A click handler that navigates is an accessibility defect (book
  `router/19_a`). The current page is `aria-current="page"`, drawn from the
  router's location so it moves with a navigation in the page. [review]
- **Typed params and queries** with `#[derive(Params)]` and
  `use_params::<T>()`, handling `Err` and `None` as user input (book
  `router/18_params_and_queries`). [review]
- **Filter, search and paging state is in the URL**, except what can
  identify a patient (§0), which posts (book `15_global_state`, "URL as
  Global State"). [review]

### 14.8 Hydration correctness

- **Both halves draw the same tree.** No `cfg!(target_arch)` value in view
  code; browser calls go behind `#[cfg(target_arch = "wasm32")]` inside a
  handler or an effect (book `ssr/24_hydration_bugs`). [check L5]
- **No non-determinism in the first render**: a random id or a timestamp
  that differs between server and browser (book `ssr/24_hydration_bugs`).
  [review]

### 14.9 The document head

- **The head is the console's**: the library draws no `leptos_meta` tag
  [check L10]. A console calls `provide_meta_context()` at the root, writes
  `<MetaTags/>` in its shell, and sets `<Title>`, `<Meta>`, `<Link>`,
  `<Stylesheet>`, `<Html {..} lang>` from component bodies, never by editing
  the shell's head (book `metadata`; leptos_meta `src/`). [review]
- **One title format**: a `<Title formatter>` at the root and a `<Title
  text>` per page (book `metadata`). Head content that depends on data needs
  a blocking resource or `SsrMode::Async` (book `ssr/23_ssr_modes`). [review]

### 14.10 Browser code

- **`leptos-use`-style wrappers or `web_sys` with its features, never a
  JavaScript library** (§0; book `web_sys`). An element is reached through
  a typed `NodeRef`, filled when it mounts (book `web_sys`, "Accessing raw
  `HtmlElement`s"). [review]
- **A behaviour shared by several elements is a directive**: `use:f` runs in
  the browser when the element is created or hydrated, never on the server,
  and cleans up with `on_cleanup` (tachys `src/html/directive.rs`; leptos
  `examples/directives` at `v0.8.22`; the book does not cover it; #214).
  [review]

### 14.11 The bundle

- **The browser bundle builds under `wasm-release`** (`opt-level = "z"`,
  `lto = true`, `codegen-units = 1`) and cargo-leptos runs `wasm-opt -Oz` on
  it (book `deployment/binary_size`; cargo-leptos 0.3.11
  `src/compile/front.rs`). A console serves it compressed. [review; the
  budget of §12 in CI]
- **Size sources are checked before they land**: `regex` (about 500 KB),
  a generic with a large body used with several types (keep one concrete
  inner function), `serde` where `serde-lite` serves (book
  `deployment/binary_size`). [review]
- **Every page is a lazy route**: `#[lazy_route]` on a `LazyRoute` and
  `cargo leptos build --split` (book `deployment/binary_size`, "Code
  Splitting"; leptos_router_macro `src/lib.rs`), hydrated with
  `hydrate_lazy`, never `hydrate_body`, which panics on a lazy route. A
  build with lazy code and no `--split` is broken, never merely unsplit:
  its glue imports `__wasm_split_placeholder__`, no browser resolves it,
  and nothing hydrates (#331). [review; the glue check of
  `scripts/release/gallery-site.sh`]

### 14.12 Testing

- **Logic lives in plain functions with unit tests**, so a component stays
  thin (book `testing`): `widget::tabs::moved`, `widget::filter::keeps`.
  [review]
- **Behaviour is proven in a real browser**, by the gallery's journeys over
  thirtyfour (book `testing`, end-to-end; the book's Playwright is
  JavaScript, §0). [test: the gallery journeys]

## 15. Hydration and navigation: the rules a component obeys

The library and the consoles hydrate the whole page and split it by route
(owner ruling on #410, 2026-10-25, which replaced the islands mode of
#196). The authority is the book (`ssr/22_life_cycle`,
`ssr/24_hydration_bugs`, `router/`, `progressive_enhancement/`,
`deployment/binary_size`) with the 0.8 source it pins.

### 15.1 Routes and their code

- **Every page is a `LazyRoute`** whose view draws the page component, and
  the browser entry calls `hydrate_lazy` (leptos `src/mount.rs`). Code
  every page needs, the shell and the widgets it draws, stays in the entry;
  a page's own code loads from its chunks. [review; the budget of §12]
- **A route's code loads before the visit**: the console names its routes
  and their `LazyRoute::preload` once with `widget::prefetch::routes`, so a
  link's hover or focus loads its route, and the main navigation's routes
  load once the first page is idle. [review; `widget::prefetch` tests]

### 15.2 Navigation in the page

- **The router follows every link and form in the page**: `leptos_router`
  takes a same-origin `<a>` click, and a form submits through
  `leptos_router::components::Form` or `<ActionForm>`, so the document,
  the shell and every component's state survive (book `router/19_a`,
  `router/20_form`). A link to a server route that answers with no page
  (the sign-in, a download) carries `rel="external"` (§4). [review; the
  gallery's journeys assert a link keeps its document]
- **A navigation that fails falls back to a full load**: a server function
  or a form the router cannot complete loads the address as a document, so
  the server draws the page or its error, never a blank or stale page.
  [review]
- **The new page is announced and takes the focus** (WCAG 2.2 SC 2.4.3,
  SC 4.1.3): after a navigation in the page the focus moves to the page's
  heading and a polite region says its title. [review]

### 15.3 State both halves read alike

- **No component reads state on the server alone.** The theme, the
  preferences, a listing's position in its cookie and the session reach
  the browser as the server read them: a `SharedValue` or a resource read
  while the page hydrates, and a server function after a navigation in the
  page (book `ssr/22_life_cycle`; leptos_server `src/shared.rs`). The
  gallery's `cookies` module is the shape. [review; a hydration panic fails
  every journey through the console check]
- **No non-determinism in a render** (§4, §14.8): an id, a time or a count
  that differs between the halves breaks hydration. [review]
- **The document head and the root element are drawn once**: an attribute
  of `<html>` the server set from a cookie (`data-theme`, `lang`) is set
  again in the browser when the state that names it changes. [review]

### 15.4 Props and words

- **Props are page data the browser draws itself**: nothing serializes them
  into the page, so a component may take any type; a value a server read is
  carried as §15.3 says. [review]
- **Words are props**, resolved in the page's locale by `*Words::of_page()`
  in both halves, so a console provides the same locale on the server and
  in the browser (a `SharedValue` of its tag), and the browser carries the
  library's catalog. [review; the locale tests]
- **No `usize` or `isize`** in a server function or any serialized type
  (§14.6). [check L8 while it remains]

### 15.5 Context and toasts

- **Context reaches every component under the root**, the router's
  included. [review]
- **Toasts go through one `toast_region` in the shell**, fed by a
  browser-only queue in the crate that any component pushes onto
  (`widget::toast::raise`). With no WebAssembly the server renders the
  flash toasts of a post-redirect-get into the region. [review]

### 15.6 Server functions and actions

- **The library defines no server function** (§0): a library form view
  takes the console's `ActionForm` as children, or renders fields named
  after the function's arguments (`arg[field]`). [check L3]
- **A console's action is a server function behind `<ActionForm>`**: it
  runs in the page with `pending()` and `value()`, and its plain post is
  the fallback that works with no WebAssembly through server_fn's
  `form-redirects` (server_fn 0.8.13 `src/lib.rs`; leptos `src/form.rs`).
  [review]

### 15.7 The no-WebAssembly fallback

- **Every page still works with the WebAssembly withheld**: its server
  HTML shows the content, and a link or a form does the important action
  as a full load. It is the fallback, never the design target. A control
  that can only work with script is absent or disabled before hydration,
  and says why. [test: the gallery's no-WebAssembly journey]
