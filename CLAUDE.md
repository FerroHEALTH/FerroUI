<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# CLAUDE.md

**FerroUI** is the shared UI component library of the FerroHEALTH consoles:
FerroBRIDGE, FerroFED, FerroEHR and FerroTERM. It is one Cargo workspace with
the library crate `ferrohealth-ui` (Rust path `ferrohealth_ui`, published to
crates.io) and the gallery app `ferrohealth-ui-gallery` (`publish = false`),
which shows every component in every product theme, light and dark, and is
the library's visual review surface and its proof in CI.

FerroUI in prose, `ferrohealth-ui` in identifiers. The crates.io name
`ferroui` belongs to an unrelated project: never name a crate `ferroui` and
never publish under it.

## What the library is

- **Islands, in three parts.** A console renders in Leptos islands mode
  (decided on #196, with the measurements in `crates/ferrohealth-ui/CLAUDE.md`):
  the server draws every page whole and the browser hydrates only its
  islands. `view` holds static view functions (plain functions, no reactive
  props, no `#[component]`) drawn on the server alone, and through
  `to_html()` into a string a read-only screen draws with `inner_html`.
  `island` holds the ready islands (toast region, navigation drawer,
  keyboard tabs), whose props, words included, are plain data the server
  resolved. `widget` holds the interactive components as functions a page
  calls inside an island, whose browser code runs only in handlers and
  effects behind `#[cfg(target_arch = "wasm32")]`. Every page works with no
  WebAssembly: a link or a form is behind every important action.
- **One table model** (`table`): page and size read defensively and clamped,
  the row window, the range label, sorting, filtering and paging by links
  and forms that work with no script, and the state source
  `Url` or `Session` (a screen that can show patient data keeps its position
  out of the URL).
- **One stylesheet** (`css/ferrohealth-ui.css`), plain CSS in Tailwind 4's
  layer order, defining every `fui-` class a component writes, so a console
  needs no scan of the crate. A console sets its brand in a theme file
  (`themes/*.css`): seven custom properties, five per theme and the two
  preview accents. `tests/it/contrast.rs`
  holds every role pairing of every brand to WCAG 2.2 AA.
- **Highlighters** for YAML, JSON, XML and AQL (`highlight`), total and
  deterministic, compiled into both halves.
- **Lucide icons** vendored verbatim with provenance (`vendor/lucide/`), drawn
  by `icon(glyph, size)` beside the word they illustrate; no icon crate.
- **No component** writes a `style` attribute or a `<style>` element (a
  console keeps `style-src 'self'`), defines a server function, fetches, or
  touches browser storage. Theme persistence, URL policy and the router base
  are the console's.

## How we use Leptos

The library and the consoles use Leptos 0.8 the way the Leptos book teaches
it, in islands mode (#196): a page is server HTML, and only an island ships
code to the browser. Where the book lags behind 0.8, the Leptos 0.8 source
the workspace pins decides.

- **The rules are `.claude/rules/leptos-ui.md`.** §14 states each Leptos rule
  with its book chapter and 0.8 source, and §15 is the islands rules: when a
  part is an island, its props, its children, context, locale and toasts,
  what reaches the page, forms, and the no-WebAssembly floor.
- **`scripts/checks/leptos-rules.sh` enforces the mechanical part in CI**
  (rules L1 to L10, the `leptos-rules` job). Its allow file lists each
  exception with the issue that removes it.
- **`/new-component`** walks a component from its COMPONENTS.md id to the
  gallery, the tests, the screenshots and the changelog.
- **`/leptos-check`** reviews a diff against §14 and §15 before a pull
  request opens.

## Repo map

- `crates/ferrohealth-ui`: the library. `src/` (`view`, `widget`, `table`,
  `highlight`, `icon`, `tone`, `theme`, `style`), `css/`, `themes/`,
  `vendor/lucide/`, `tests/it/` (contrast, stylesheet rules, rendering).
- `app/ferrohealth-ui-gallery`: the gallery, server rendered with its
  islands hydrated, its halves chosen by target (never by Cargo feature),
  with its browser journeys (the no-WebAssembly journey among them) and
  screenshots under `tests/it/` (`FERROUI_JOURNEYS=1`,
  `FERROUI_SCREENSHOTS=1`) and the images under `docs/screenshots/`.
- `design/`: the approved console design and the source of truth for
  0.3.0 (`design/README.md`): `COMPONENTS.md` (the family rules, the
  templates and every component spec), the `mk-` capture scaffolding, the five
  brand themes, the reference templates and each console's page sources
  under `consoles/<console>/`, with the shared tools in `tools/` (build,
  render, the overflow, hover, contrast, selects, popups and forms
  checks, and the sync
  from the design folders). Renders go to the ignored `design/out/`.
- `scripts/`: `vendor/lucide.sh`, `release/` (journey tools, the gallery
  site build, the crate publish), `checks/` (the committed guards, the
  Leptos rules check `leptos-rules.sh` among them), `gh/` (the tracker
  helpers).
- `.github/`: CI with a single required `conclusion` job, CodeQL, Scorecard,
  Dependabot, the release lane with crates.io Trusted Publishing.
- `.claude/`: the working discipline (rules, agents, skills, hooks, memory).

## IMPORTANT hard rules

- **A component follows the design in `design/`** (`.claude/rules/console-design.md`):
  a change lands in `design/COMPONENTS.md` and the stylesheet first, then
  every console is re-rendered and checked, then the Rust follows.
- **Every component follows the interaction contract** (`design/COMPONENTS.md`
  § Interaction, `crates/ferrohealth-ui/CLAUDE.md`): native elements first,
  behaviour in Rust compiled to WebAssembly in islands, every page working
  without the WebAssembly, no hand-written JavaScript.
- **The component rules are `.claude/rules/leptos-ui.md`.** Each product's
  own `leptos-ui.md` keeps only what is particular to it and points here.
- **The authorities are the specifications**: WCAG 2.2, WAI-ARIA 1.2 and the
  WAI-ARIA Authoring Practices, the HTML Living Standard, CSS, and for Leptos
  the official Leptos book. Cite them; never resolve a question from memory.
  Where none governs a decision, say so: "no specification governs this: our
  own design".
- **Comments follow RFC 505 and RFC 1574 with hard budgets**
  (`.claude/rules/comments.md`); prose follows
  `.claude/rules/writing-style.md`.
- **Branches use conventional types** (`feat/`, `fix/`, `chore/`, `docs/`,
  `refactor/`, `perf/`, `test/`, `ci/`, `build/`, `release/`); every commit is
  signed; `main` takes changes only through pull requests and the merge queue.
- **NEVER add AI or Claude attribution** to a commit, PR, issue, comment, or
  code comment. A `PreToolUse` hook blocks a commit or PR command carrying
  one.
- **Never weaken, skip, or delete a test** (`.claude/rules/testing.md`).
- **Keep the changelog** (`CHANGELOG.md`, Keep a Changelog 1.1.0).
- **Look at every gallery screenshot** a change refreshes before it merges.

## Licence

Business Source License 1.1 (`LICENSE`, `NOTICE`), Licensor Cadasto B.V.,
Change License Apache 2.0 four years after each version; every first-party
file carries `SPDX-FileCopyrightText: Cadasto B.V.` and
`SPDX-License-Identifier: BUSL-1.1`. The vendored Lucide outlines keep their
ISC License, recorded beside them.

## Working discipline (`.claude/`)

- Rules: `writing-style`, `rust-style`, `reliability`, `comments`, `testing`,
  `ci-cd`, `crates-publishing`, `issue-workflow`, `issue-relationships`,
  `project-board`, `vendored-inputs`, `leptos-ui` (the component rule
  set), and `console-design` (the design library and the family rules).
- Agents: `implementer`, `spec-researcher`. Skills: `next-task`,
  `phase-done`, `phase-status`, `new-component`, `leptos-check`.
