<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# CLAUDE.md: `ferrohealth-ui`

The crate-local discipline of the library. The component rules are
`.claude/rules/leptos-ui.md`; this file holds the interaction contract and
the rendering mode every component is built to, decided on FerroUI#196.

## The interaction contract

Every component follows it (owner, 2026-10-22):

1. **Native elements first:** `<dialog>`, the `popover` attribute,
   `<details>`, plain links, and GET and POST forms.
2. **Behaviour in Rust compiled to WebAssembly**, in islands.
3. **Every page works without the WebAssembly:** complete server-rendered
   HTML, and a link or a form behind every important action. The gallery's
   `tests/it/no_wasm.rs` loads every page with the bundle refused.
4. **No hand-written JavaScript and no npm:** only the `wasm-bindgen` glue
   and the island loader Leptos writes. The strict CSP stays.

## The mode: islands

NOTE: the Leptos book, "Guide: Islands"; FerroUI#196. The library supports
Leptos islands mode alone: the server draws every page whole and the
browser hydrates only the islands. Hydrating the whole page was measured
and rejected:

| Gallery build (bytes) | WebAssembly raw | gzip -9 | brotli -q 11 | JS glue raw / brotli |
|---|---:|---:|---:|---:|
| Whole page hydrated (0.2.2) | 2455510 | 943813 | 685903 | 25423 / 6276 |
| Islands, spike | 1527677 | 613353 | 460744 | 25391 / 6325 |
| Islands, as landed | 1389712 | 564473 | 427449 | 22469 / 5817 |

Measured 2026-10-22 with cargo-leptos 0.3.11, wasm-bindgen 0.2.129,
wasm-opt version_133 and Rust 1.99.1. With the CPU slowed 4x, the forms
page was interactive in 277 ms in islands against 315 ms hydrated (median
of seven); at 1.6 Mbps the brotli download is about 2.3 s against 3.4 s.
Beyond the bytes, data a server component reads is never serialized into
the page a second time as hydration state: only island props reach the
browser, which for clinical data is a privacy gain. The consoles are
listings, details and settings with pockets of interaction, the case the
book names for islands. The spike's islands patch is kept with the #196
pull request as evidence.

**Revisit** the islands router, and with it client-side navigation, when
Leptos ships it stable: in 0.8.22 it is experimental, with gaps in children
diffing, the form submitter, file inputs and the base path.

## How the crate is laid out for it

- `view`: plain functions, drawn on the server alone.
- `island`: the ready islands, each a thin `#[island]` over its widget
  function: `ToastRegion`, `NavDrawer`, `UserMenuButton`, `KeyboardTabs`,
  `KeyHint`, `DocumentViews`, `TabSelect`, `Search`. The module's `#![expect]` lines cover what
  the island macro writes (a forgotten view state, an undocumented props
  struct, a dropped `#[must_use]`).
- `widget`: the interactive components as functions a page calls inside
  an island, the library's own or a console's.

The rules every island keeps are `leptos-ui.md` §15: plain-data props
that are page content and carry the words, server children, context only
from an enclosing island, one toast region fed by a browser-only queue
(`widget::toast::raise`) with flash toasts from the server, plain links and
forms, and the no-WebAssembly floor. The crate keeps no widget on
`leptos_router`, and gives `leptos-island` and `leptos-children`
`display: contents`, with each container's spacing between its children
reaching through them (`tests/it/stylesheet.rs`).

Every widget takes the words it draws as props: a `*Words` struct whose
`of_page()` resolves them on the server, with an `i18n::Phrase` for a
sentence around a value the browser learns (a file's name, a page number).
No island looks a word up, so the browser bundle carries no Fluent loader
and no catalog.

## The bundle budget

The gallery's brotli-compressed WebAssembly is held to 491520 bytes (480
KiB) by `scripts/checks/gallery-bundle.sh`, which reads the number from
`leptos-ui.md` §12. Each console keeps its own budget in its own
repository: FerroBRIDGE's is 458752 bytes.
