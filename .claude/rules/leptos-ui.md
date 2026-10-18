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
so a workspace `--all-features` build never enables both.

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
last page "301-312 of 312"), a step that cannot be taken stays visible and
says "(unavailable)" in text, a sortable header is a link with `aria-sort`,
numbers align right, and the table shows every row of its page and scrolls
sideways in its card, never the page.

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
  Tailwind slate, radii 0.5 rem and 0.375 rem.
- A brand's theme file sets `--accent`, `--accent-hover`, `--accent-ink`,
  `--accent-tint` and `--focus` in three blocks: `:root`,
  `:root[data-theme="dark"]`, and the system's dark under
  `prefers-color-scheme`. Nothing else.
- The theme is `data-theme` (`light`, `dark`, `system`) on `<html>`, written
  by the console's server, so the first paint is right.
- Six steps of type (`--text-display` to `--text-micro`) and four of space
  (`--space-tight` to `--space-section`), the compact density under
  `data-density="compact"`; the system font stack, no web font.

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

## 11. Bundle

A library component reaches every console's bundle, so a new browser-side
dependency or a heavy generic is justified against the bytes it adds. The
gallery, which carries every component, is held to a budget in CI:
`scripts/checks/gallery-bundle.sh` fails when its brotli-compressed
WebAssembly is over the number below, which the script reads from this table.

| Measure | Budget |
|---|---|
| WebAssembly, brotli-compressed | 409600 bytes |

The budget is 400 KiB, 26% over the 325387 bytes the first release measured
(2026-10-18, cargo-leptos 0.3.11, wasm-bindgen 0.2.129, Rust 1.98.1). A change
that needs more raises it here, in its own pull request, with the measured
size and the reason.
