<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# CLAUDE.md

**FerroUI** is the shared UI component library of the FerroHEALTH consoles:
FerroBRIDGE, FerroEHR, FerroFED, FerroPIX and FerroTERM. It is one Cargo workspace with
the library crate `ferrohealth-ui` (Rust path `ferrohealth_ui`, published to
crates.io) and the gallery app `ferrohealth-ui-gallery` (`publish = false`),
which shows every component in every product theme, light and dark, and is
the library's visual review surface and its proof in CI.

FerroUI in prose, `ferrohealth-ui` in identifiers. The crates.io name
`ferroui` belongs to an unrelated project: never name a crate `ferroui` and
never publish under it.

## What the library is

- **The whole page hydrated, in three parts** (FerroUI #410, which
  replaced the islands mode of #196). The server draws every page whole,
  the browser hydrates it, and the console's `leptos_router` draws the next
  page in the browser, so no link or form reloads the document; the pages
  load in a few groups, each a lazy route whose code loads from chunks of
  its own, prefetched on a link's hover or focus (`widget::prefetch`). `view` holds view functions
  (plain functions, no reactive props, no `#[component]`), drawn on the
  server and again in the browser. `island` holds the ready components
  (toast region, navigation drawer, keyboard tabs, the search box), whose
  words are props. `widget` holds the interactive components as functions
  a page calls, whose browser code runs only in handlers and effects behind
  `#[cfg(target_arch = "wasm32")]`. A page whose WebAssembly has not loaded
  still works: a link or a form is behind every important action, as the
  fallback.
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
  touches browser storage. The one fetch is the top bar's typeahead, which
  asks the answer route the console names (`SearchBox::suggest`), on the
  wire `docs/consoles.md` documents. Theme persistence, URL policy and the router base
  are the console's.

## How we use Leptos

The library and the consoles use Leptos 0.8 the way the Leptos book teaches
it, the whole page hydrated and split by route (#410): the server draws the
first page, the browser hydrates it and draws every page after it, and no
interaction reloads the document. Where the book lags behind 0.8, the
Leptos 0.8 source the workspace pins decides.

- **A console starts at `docs/consoles.md`:** what the library owns, what
  the console owns, and which document here governs each part.
- **The rules are `.claude/rules/leptos-ui.md`.** §14 states each Leptos rule
  with its book chapter and 0.8 source, and §15 is the hydration rules:
  routes and their chunks, navigation in the page, state both halves read
  alike, forms and actions, locale and toasts, and the no-WebAssembly
  fallback.
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
- `app/ferrohealth-ui-gallery`: the gallery, server rendered and hydrated
  whole, its pages in five lazy routes, its halves chosen by target (never by
  Cargo feature),
  with its browser journeys (the no-WebAssembly journey among them) and
  screenshots under `tests/it/` (`FERROUI_JOURNEYS=1`,
  `FERROUI_SCREENSHOTS=1`), its states one file per area under
  `tests/it/states/`, its words one file per area under `src/locale/` and
  `locales/en/`, and the images under `docs/screenshots/`, which only the
  refresh pull request commits.
- `design/`: the approved console design and the source of truth for
  0.3.0 (`design/README.md`): `COMPONENTS.md` (the family rules, the
  templates and every component spec), the `mk-` capture scaffolding, the five
  brand themes, the reference templates and each console's page sources
  under `consoles/<console>/`, with the shared tools in `tools/` (build,
  render, the overflow, hover, contrast, selects, popups and forms
  checks, and the sync
  from the design folders). Renders go to the ignored `design/out/`.
- `scripts/`: `fast.sh` (the local gate), `chrome-lock.sh` (the
  machine-wide lock the browser steps hold), `vendor/lucide.sh`, `release/`
  (journey tools, the gallery site build, the crate publish, `changelog.sh`
  that checks and assembles the fragments), `checks/` (the committed guards,
  the Leptos rules check `leptos-rules.sh`, the change classifier
  `affected.sh`, and the changelog, screenshot-path and module-order guards
  among them), `gh/` (the tracker helpers).
- `changelog.d/`: the changelog fragments, one file per entry, assembled
  into `CHANGELOG.md` by the release pull request (`docs/release.md`).
- `.github/`: CI with a single required `conclusion` job, CodeQL, Scorecard,
  Dependabot, the release lane with crates.io Trusted Publishing.
- `.claude/`: the working discipline (rules, agents, skills, hooks, memory).

## IMPORTANT hard rules

- **A component follows the design in `design/`** (`.claude/rules/console-design.md`):
  a change lands in `design/COMPONENTS.md` and the stylesheet first, then
  every console is re-rendered and checked, then the Rust follows.
- **Every component follows the interaction contract** (`design/COMPONENTS.md`
  § Interaction, `crates/ferrohealth-ui/CLAUDE.md`): native elements first,
  behaviour in Rust compiled to WebAssembly, interactions in the page with
  no document reload, a plain link or post as the fallback while the
  WebAssembly has not loaded, no hand-written JavaScript.
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
- **`scripts/fast.sh` is the local gate; CI is the full gate**
  (`.claude/rules/ci-cd.md`). Run `fast.sh` before you push: it runs what
  your change reaches, and the browser steps queue on the Chrome lock.
  Never run the full screenshot capture locally; capture the pages you
  touched with `FERROUI_SCREENSHOT_PAGES`.
- **Never weaken, skip, or delete a test** (`.claude/rules/testing.md`).
- **Keep the changelog** (`CHANGELOG.md`, Keep a Changelog 1.1.0) through a
  fragment under `changelog.d/`; never edit its `[Unreleased]` section.
- **Parallel pull requests share no lines** (`.claude/rules/parallel-prs.md`):
  changelog fragments, gallery states and words in their area's own files,
  no screenshots in a feature pull request (the `chore/screenshots` refresh
  commits them, before every release), and every `mod` list alphabetical.
- **Look at every gallery screenshot** a change moves before it merges.

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
  set), `console-design` (the design library and the family rules), and
  `parallel-prs` (no shared lines between open pull requests).
- Agents: `implementer`, `spec-researcher`. Skills: `next-task`,
  `phase-done`, `phase-status`, `new-component`, `leptos-check`.
