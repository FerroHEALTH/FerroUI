<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# CLAUDE.md: `ferrohealth-ui`

The crate-local discipline of the library. The component rules are
`.claude/rules/leptos-ui.md`; this file holds the interaction contract and
the rendering mode every component is built to, decided on FerroUI#196.

## The interaction contract

Every component follows it (owner, 2026-10-22; the mode rewritten by the
owner's ruling on FerroUI #410, 2026-10-25):

1. **Native elements first:** `<dialog>`, the `popover` attribute,
   `<details>`, plain links, and GET and POST forms.
2. **Behaviour in Rust compiled to WebAssembly**, in components the browser
   hydrates.
3. **Interactions stay in the page, with no document reload.** A link is
   followed by the console's `leptos_router` in the page, and a form
   submits through it; a plain post that loads a new document is the
   fallback for a page whose WebAssembly has not loaded, never the normal
   path. The gallery's `tests/it/no_wasm.rs` still loads every page with
   the bundle refused, so the fallback keeps working.
4. **No hand-written JavaScript and no npm:** only the `wasm-bindgen` glue
   and the loader Leptos writes. The strict CSP stays.

## The mode: the whole page hydrated, split by route

NOTE: the Leptos book, "Hydration" and "Code Splitting"; FerroUI #410. The
server draws every page whole, the browser hydrates the whole page, and the
console's `leptos_router` draws the next page in the browser, so a link or
a form never reloads the document. Each page is a `LazyRoute`, so its code
loads from chunks of its own under `cargo leptos build --split`, and
[`widget::prefetch`](src/widget/prefetch.rs) loads a route's code when a
link to it is hovered or focused, and the main navigation's once the first
page is idle.

FerroUI #196 had chosen islands mode on bundle size and time to interactive.
The owner ruled on #410 that a page that reloads on every click is not
acceptable and that the experience outranks the bundle. The Leptos 0.8.22
islands router was read and set aside: it never diffs an island's children
or props, never removes a stale attribute, has no fallback for a failed
navigation, and drops a form submitter's value and file inputs
(`leptos/src/hydration/islands_routing.js`). The gallery measured the
change (release build, brotli -q 11, Chrome for Testing with the CPU slowed
4x, 2026-10-25):

| Gallery build | Entry WebAssembly, brotli | Page chunks, brotli |
|---|---:|---:|
| Islands, v0.4.0 | 465420 | 387833 (eight lazy islands) |
| Whole page, no split | 1557224 | none |
| Whole page, every page a lazy route | 465338 | 1846251 (all pages) |

| Link click, one session | Islands (a new document) | Whole page (in the page) |
|---|---:|---:|
| tables to forms, first visit | 226 ms | 424 ms |
| forms to tables | 267 ms | 181 ms |
| tables to forms, again | 234 ms | 171 ms |
| find to tables | 267 ms | 134 ms |

A first visit of a heavy page paid for its chunk; [`widget::prefetch`]
loads it before the click.

## How the crate is laid out for it

- `view`: plain functions, drawn on the server and again in the browser as
  the page hydrates or the router draws the next page.
- `island`: the ready components, each a thin `#[component]` over its
  widget function. The module keeps its name from islands mode; it is the
  list of them, so two pull requests that each add one touch different
  lines (`.claude/rules/parallel-prs.md`).
- `widget`: the interactive components as functions a page calls, the
  library's own or a console's; their browser code runs only in handlers
  and effects behind `#[cfg(target_arch = "wasm32")]`, so the first paint is
  the server's and hydration matches it.

Every widget takes the words it draws as props: a `*Words` struct whose
`of_page()` resolves them in the page's locale, with an `i18n::Phrase` for a
sentence around a value the browser learns (a file's name, a page number).
The browser now resolves the words too, so the console provides the same
locale in both halves. Each word is an `i18n::Word`, which carries whether it
fell back to English and draws itself inside `<span lang="en">` when it did
(WCAG 2.2 SC 3.1.2), and a widget that writes a count carries the page's
`i18n::number::Grouping` among its words.

## The bundle budget

The gallery's brotli-compressed entry WebAssembly, what every page loads
before its route's chunks, is held to 491520 bytes (480 KiB) by
`scripts/checks/gallery-bundle.sh`, which reads the number from
`leptos-ui.md` §12. Each console keeps its own budget in its own
repository: FerroBRIDGE's is 458752 bytes.
