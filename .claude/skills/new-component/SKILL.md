---
name: new-component
description: Walks a FerroUI component from its COMPONENTS.md id to a merged change, in the design's order. Covers the design pages, the choice between a server view and an island, the API, the CSS layer, the render, stylesheet and contrast tests, the gallery entry in every brand light and dark, the no-WebAssembly journey, the design check, the screenshots and the changelog. Use when an issue asks for a new or rebuilt component.
allowed-tools: Read, Grep, Glob, Bash, Edit, Write
argument-hint: "<issue number>"
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# /new-component

One component, end to end. The order is the design library's
(`.claude/rules/console-design.md`): the design first, then the Rust. The
component rules are `.claude/rules/leptos-ui.md`; §14 is the Leptos way and
§15 the islands rules, and every step below names the section it obeys.

## Steps

1. **Read the contract.** Read the issue with
   `gh issue view <n> --json title,body,comments --jq '.title, .body, (.comments[] | "--- comment ---", .body)'`
   and its edges with `scripts/gh/rel.sh tree <n>`. Note the COMPONENTS.md
   ids it names (R2, F13, B2 and so on).
2. **Read the design.** Open each id's section in `design/COMPONENTS.md` and
   every page that uses it under `design/consoles/<console>/` and
   `design/templates/`. Read the classes it uses in
   `design/css/ferroui-console.css`. If the component must differ from the
   design, change `design/COMPONENTS.md` and the console stylesheet first,
   then run `design/tools/render.sh all` and `design/tools/check.sh all`,
   before any Rust.
3. **Choose a server view or an island** (§15.1). Ask, in order:
   - Can HTML, CSS and a native element do it (`<dialog>`, `popover`,
     `<details>`, a link, a `GET` or `POST` form, a checkbox with `:has()`)?
     Then it is a view function in `crates/ferrohealth-ui/src/view/`, and it
     ships no WebAssembly.
   - Does it need browser code? Then the smallest interactive part is an
     `#[island]` in `src/widget/`, and everything static is a server view
     passed in as its children (§15.3).
   - Does its interactive part share state with another island? Then the
     two nest: the provider is the outer island, and the other is built
     inside its children (§15.4). Siblings share nothing.
4. **Write the API.**
   - A view function takes plain values, never a signal (§1).
   - An island's props are `Serialize + DeserializeOwned`: no closure, no
     `Callback`, no `usize` or `isize`, and no clinical data the page does
     not already show (§15.2).
   - Words are `String` props the console resolved from its catalog. The
     library's own words come from `i18n`, looked up on the server side of
     the island (§13, §15.4).
   - A form view renders fields named after the console's server-function
     arguments, or takes the console's `ActionForm` as children. The library
     defines no server function (§15.7).
   - Document the component and every prop.
5. **Write the no-WebAssembly floor** (§15.8). The server HTML shows the
   content, and a link or a form does the important action. A control that
   only script can work is absent or disabled before hydration, and says
   why.
6. **Write the CSS in the right layer.** Classes are `fui-`, in the
   `components` layer of `crates/ferrohealth-ui/css/ferrohealth-ui.css`,
   naming role tokens only (§8): no palette entry, no inline style (§0). A
   new role joins `tests/it/contrast.rs` in the same change (§7).
7. **Write the tests.**
   - Logic in plain functions with unit tests beside them (§14.12).
   - `crates/ferrohealth-ui/tests/it/render.rs`: the rendered HTML says what
     it must and never carries what it must not, read through
     `markup::tags` by attribute name. Cover the escaping of every text
     the component is given.
   - `tests/it/stylesheet.rs`: every class the component writes is defined
     (the existing test covers it), plus a rule test for any layout
     guarantee.
   - `tests/it/contrast.rs` for a new role pairing.
8. **Add the gallery entry.**
   - Put the component on its page under
     `app/ferrohealth-ui-gallery/src/pages/`, with its words in
     `app/ferrohealth-ui-gallery/locales/en/ferrohealth-ui-gallery.ftl`. The
     brand routes show it in every brand, light and dark.
   - Add its interactive states to `tests/it/states.rs` (§11).
   - Add a journey for each behaviour to `tests/it/journeys.rs` or
     `tests/it/widgets.rs`.
   - Add an assertion to the no-WebAssembly journey (#196): the content and
     the key action work with the WebAssembly withheld.
9. **Run the gates.**
   - `cargo fmt --all --check`.
   - `cargo clippy --workspace --all-targets -- -D warnings`.
   - The two wasm32 clippy runs: `cargo clippy -p ferrohealth-ui --lib --target wasm32-unknown-unknown --features leptos/hydrate -- -D warnings`, and the same with `leptos/csr`.
   - `cargo nextest run --workspace --locked`.
   - The guards: `bash scripts/checks/leptos-rules.sh`, `bash scripts/checks/stray-literals.sh crates/ferrohealth-ui/src app/ferrohealth-ui-gallery/src` and `bash scripts/checks/comment-style.sh --all`.
   - `design/tools/check.sh all`, with no baseline entry added.
10. **Look at it.** Run the journeys (`FERROUI_JOURNEYS=1`), then the
    screenshots and states (`FERROUI_SCREENSHOTS=1`, over the release site
    bundle from `scripts/release/gallery-site.sh --release`). Look at every
    image that changed, in every brand, light and dark, at 1440 and 390 px.
    Refresh the `docs/qa/` pair (§11).
11. **Check the bundle.** Run `bash scripts/checks/gallery-bundle.sh` after
    the release build. A rise past the budget is its own pull request with
    the measured size (§12).
12. **Review and record.**
    - Run `/leptos-check` over the diff and fix what it finds.
    - Add the `[Unreleased]` entry to `CHANGELOG.md`, naming any API break
      and what a console changes.
    - Open the pull request from the template with `Closes #<n>`, and arm
      auto-merge (`gh pr merge <n> --auto`).
