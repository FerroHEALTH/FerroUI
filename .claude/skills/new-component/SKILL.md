---
name: new-component
description: Walks a FerroUI component from its COMPONENTS.md id to a merged change, in the design's order. Covers the design pages, the template and its script module, the API, the CSS layer, the render, stylesheet and contrast tests, the gallery entry in every brand light and dark, the no-script journey, the design check, the screenshots and the changelog fragment. Use when an issue asks for a new or rebuilt component.
allowed-tools: Read, Grep, Glob, Bash, Edit, Write
argument-hint: "<issue number>"
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# /new-component

One component, end to end. The order is the design library's
(`.claude/rules/console-design.md`): the design first, then the Rust. The
component rules are `.claude/rules/console-ui.md`, and every step below
names the section it obeys.

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
   then run `design/tools/render.sh <set>` for each console that draws the
   component and `scripts/fast.sh`, which checks every design set your
   change reaches, before any Rust.
3. **Choose the markup and the module** (§1, §3). Ask, in order:
   - Can HTML, CSS and a native element do it (`<dialog>` with `command`,
     `popover`, `<details>`, a link, a `GET` or `POST` form, a checkbox with
     `:has()`)? Then it is a component in `crates/ferrohealth-ui/src/view/`
     over a template under `templates/<area>/`, and it needs no module.
   - Does it need browser code? Then its root asks for a module with
     `data-fui="<job>"`, and the module under `crates/ferrohealth-ui/scripts/`
     enhances what the template drew, within the script budget (§12).
4. **Write the API.**
   - A component takes plain values and `&Locale` where it draws words, and
     its children as `Html` (§1).
   - Words come from the catalogs on the server: the library's own from
     `i18n`, a console's as `String` from its catalog (§13). A module's words
     ride in data attributes or a `<template>`. A new library word is a
     `Text` declared in `src/i18n/words/<area>.rs` with its English in
     `locales/en/<area>.ftl`, each in the order of the ids.
   - A form names its fields for the console's handler; the library holds
     no route (§0).
   - Document the component and every argument.
5. **Write the no-script floor** (§1). The server HTML shows the content,
   and a link or a form does the important action. A control that only
   script can work is drawn `hidden` or `disabled`, and its module shows or
   enables it.
6. **Write the CSS in the right layer.** Classes are `fui-`, in the
   `components` layer of `crates/ferrohealth-ui/css/ferrohealth-ui.css`,
   naming role tokens only (§8): no palette entry, no inline style (§0). A
   new role joins `tests/it/contrast.rs` in the same change (§7).
7. **Write the tests.**
   - Logic in plain functions with unit tests beside them; a module's
     logic that the Rust decides too answers the same.
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
     `app/ferrohealth-ui-gallery/src/pages/`, with its words declared in
     that page's `src/locale/<area>.rs` and written in
     `locales/en/<area>.ftl`, each in the order of the ids (`shell` for a
     word more than one page draws). The brand routes show it in every
     brand, light and dark.
   - Add its interactive states to its area's file under
     `tests/it/states/`: the names to `STATES`, the steps to the area's
     capture function (§11). A new area is a new file, a `mod` line and an
     `AREAS` entry in `states.rs`, and a call in the capture driver.
   - Put every new `mod` line (library, gallery, `tests/it/main.rs`) in
     alphabetical position; `scripts/checks/mod-order.sh` checks it.
   - Add a journey for each behaviour to `tests/it/journeys.rs` or
     `tests/it/widgets.rs`.
   - Add an assertion to the no-script journey (`tests/it/no_script.rs`):
     the content and the key action work with the modules refused.
9. **Run the gates.**
   - `scripts/fast.sh`: rustfmt, clippy,
     nextest and the doctests, the journeys of the area you touched, and
     `design/tools/check.sh` for each design set your change reaches, with
     no baseline entry added. The browser steps wait on the machine-wide
     Chrome lock, so several worktrees can run it at once. CI runs the
     full gate.
   - The guards: `bash scripts/checks/mod-order.sh`,
     `bash scripts/release/changelog.sh --check`,
     `bash scripts/checks/markup-rules.sh crates/ferrohealth-ui/src crates/ferrohealth-ui/templates app/ferrohealth-ui-gallery/src`, `bash scripts/checks/script-budget.sh`, `bash scripts/checks/stray-literals.sh crates/ferrohealth-ui/src crates/ferrohealth-ui/templates app/ferrohealth-ui-gallery/src` and `bash scripts/checks/comment-style.sh --all`.
10. **Look at it.** Never run the full screenshot capture. Capture only the
    pages you touched:
    `scripts/chrome-lock.sh env FERROUI_SCREENSHOTS=1 FERROUI_SCREENSHOT_PAGES=<page,...> cargo nextest run --locked -p ferrohealth-ui-gallery -E 'test(every_page_is_captured)'`.
    Look at every image that changed, in every brand, light and dark, at
    1440 and 390 px; the `gallery-screenshots` artifact of CI holds the
    whole capture. Then restore `docs/screenshots/` (`git checkout
    origin/main -- docs/screenshots`, and delete any image you added): a
    feature pull request commits no screenshot, the coordinator's refresh
    pull request does (`.claude/rules/parallel-prs.md`). Name each new state
    in the pull request body. Refresh the `docs/qa/` pair (§11).
11. **Check the script budget.** Run `bash scripts/checks/script-budget.sh`
    after a module changes; a module that grows past its share splits
    (§12).
12. **Review and record.**
    - Run `/console-check` over the diff and fix what it finds.
    - Add the changelog fragment
      `changelog.d/<issue>-<kebab-slug>.<section>.md` (one per section),
      naming any API break and what a console changes. Never edit
      `CHANGELOG.md` (`changelog.d/README.md`).
    - Open the pull request from the template with `Closes #<n>`, and arm
      auto-merge (`gh pr merge <n> --auto`).
