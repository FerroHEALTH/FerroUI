---
paths: ["design/**", "crates/**", "app/**"]
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# The console design library and its family rules

`design/` holds the approved console design (owner, 2026-10-21): the
specification `design/COMPONENTS.md`, the capture scaffolding, the five
brand themes, the reference templates and every page of the five console
mockups, with the tools that build, render and check them
(`design/README.md`). It is the source of truth for FerroUI 0.3.0. No
specification governs the design itself: our own design, held to WCAG 2.2
and WAI-ARIA 1.2 like every component (`spec-adherence.md`).

## The order of a change

A component change lands in `design/COMPONENTS.md` and the crate's
stylesheet (`crates/ferrohealth-ui/css/ferrohealth-ui.css`, which holds
every component rule and the family rules as CSS) first, then every
console is re-rendered and checked (`design/tools/render.sh all`,
`design/tools/check.sh all`), then the Rust component follows. A component that differs from the design
is a defect in the component, or a design change made in that order.

## The family rules

`design/COMPONENTS.md` § Family rules is the full text; a page that breaks
one is a defect. In short:

1. One home per thing: each object, setting and document is shown in full
   and changed on one page; others link to it or mirror it read-only.
2. No second sidebar: navigation inside a page is binder tabs, and a page
   with one or two sections has none.
3. No hover layout shift: hover and focus change paint only
   (`design/tools/hover.py`).
4. No link underlines in tables: a name in a table is ink, the row is the
   target.
5. The theme is set only in the user menu and in Your preferences; the top
   bar carries no theme toggle. This replaces the `theme_toggle` of
   `leptos-ui.md` §8 when 0.3.0 removes it.
6. No key hints on phones, below 40 rem; the shortcut legend keeps its
   keys.
7. Destructive actions come last in menus, after a separator, in the danger
   tone, with an ellipsis when a confirmation follows.
8. The simplest design that does the job: ten items or fewer have no
   summary strip and no filter menus, three facts have no tabs, a small
   fact has no side panel.
9. Personal preferences are cookies (`SameSite=Lax`, `Secure`, one year, no
   patient data), read when the server renders; the deployment's settings
   are Configuration.
10. Comfortable is the default density; compact is a preference.
11. Times are relative in lists with the full timestamp in the tooltip, and
    absolute in records and revisions.
12. A state is a shape, a colour and a word together: a filled dot, a
    diamond, a cross, a hollow ring, a muted row.
13. Console configuration mode exists only for a product with its own
    configuration store; the others are file mode only.

## The checks

`design/tools/check.sh` runs the build's markup checks and the overflow,
hover, contrast, selects, popups and forms checks; CI runs it per design set
on macOS (`ci-cd.md`). A set's `baseline.tsv` holds the findings it was approved
with and only shrinks. Never add an entry to make a check pass: fix the
page, or record an approved design change with the baseline rewrite in the
same pull request.
