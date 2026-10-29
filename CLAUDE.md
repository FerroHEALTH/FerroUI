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

- **Server-rendered HTML with a little plain JavaScript** (FerroUI #510,
  0.6.0, which replaced the hydrated mode of #410). Every component in
  `view` is a function over an Askama template compiled into the crate,
  returning `html::Html`, which a console's own template writes with
  `{{ part }}`. The behaviour is one small ES module per job under
  `scripts/`, which the loader `fui.js` imports for an element marked
  `data-fui="<job>"`; a module enhances what the server drew and changes a
  page in place with no round trip. A link or a form loads one complete
  document, and every page works with the modules refused. No WebAssembly,
  no npm, no bundler.
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
  deterministic; the `highlight` module runs the same grammars in the
  editor as you type.
- **Lucide icons** vendored verbatim with provenance (`vendor/lucide/`), drawn
  by `icon(glyph, size)` beside the word they illustrate; no icon crate.
- **No component** writes a `style` attribute, a `<style>` element or an
  inline script (a console keeps `style-src 'self'` and `script-src
  'self'`), defines a route, or touches browser storage beyond what the
  tree and the scroll keep for the session. Two modules fetch, each on a
  wire `docs/consoles.md` documents: the top bar's typeahead asks the
  answer route the console names (`SearchBox::suggest`), and the session
  watch (`view::session`, #444) asks the console's session route what the
  session has left. Theme persistence, URL policy and the routes are the
  console's.

## How a console builds on it

- **A console starts at `docs/consoles.md`:** what the library owns, what
  the console owns, and which document here governs each part. A console on
  0.5 moves with `docs/migration-0.6.md`.
- **The rules are `.claude/rules/console-ui.md`:** the mandates, the
  interaction contract, templates and markup (§2), modules (§3), and the
  script budget (§12).
- **`scripts/checks/markup-rules.sh` and `scripts/checks/script-budget.sh`
  enforce the mechanical part in CI:** no inline style, script or handler
  in a template or the Rust, and every module within 16 KiB (160 KiB for
  all) with no `eval`, no absolute URL and no `fetch` outside the two
  wires.
- **`/new-component`** walks a component from its COMPONENTS.md id to the
  gallery, the tests, the screenshots and the changelog.
- **`/console-check`** reviews a diff against the console rules before a
  pull request opens.

## Repo map

- `crates/ferrohealth-ui`: the library. `src/` (`view`, `table`, `html`,
  `script`, `serve`, `i18n`, `highlight`, `icon`, `tone`, `theme`,
  `style`), `templates/`, `scripts/`, `css/`, `themes/`, `locales/`,
  `vendor/lucide/`, `tests/it/` (contrast, stylesheet rules, rendering).
- `app/ferrohealth-ui-gallery`: the gallery, an axum server whose handlers
  draw every page,
  with its browser journeys (the no-script journey among them) and
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
  (journey tools, the crate publish, `changelog.sh` that checks and
  assembles the fragments), `checks/` (the committed guards, the markup
  check `markup-rules.sh`, the script budget, the change classifier
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
  every page complete with no script, behaviour in small plain modules over
  what the server drew, a navigation as one document.
- **The component rules are `.claude/rules/console-ui.md`.** Each product's
  own `console-ui.md` keeps only what is particular to it and points here.
- **The authorities are the specifications**: WCAG 2.2, WAI-ARIA 1.2 and the
  WAI-ARIA Authoring Practices, the HTML Living Standard, CSS, and for
  templates the Askama book. Cite them; never resolve a question from memory.
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
  `project-board`, `vendored-inputs`, `console-ui` (the component rule
  set), `console-design` (the design library and the family rules), and
  `parallel-prs` (no shared lines between open pull requests).
- Agents: `implementer`, `spec-researcher`. Skills: `next-task`,
  `phase-done`, `phase-status`, `new-component`, `console-check`.
