<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Rust style: idiomatic library code

Applies to every hand-written `.rs` file in the repository. **FerroUI is
modern idiomatic Rust of our own design**, built on Askama 0.16 and held to the
specifications its components implement: WCAG 2.2, WAI-ARIA 1.2 and the
WAI-ARIA Authoring Practices, the HTML Living Standard, and CSS
(`spec-adherence.md`). The four FerroHEALTH consoles are prior art: read their
components for how a problem was solved, never port one blindly.

## Build idiomatic, compiling, tested code

- **Use proper crates; do not hand-roll** what the pinned set provides. Add a
  dependency through the root `Cargo.toml [workspace.dependencies]` table
  (`dep.workspace = true`), and verify its version against crates.io or
  docs.rs the day it is added, never from memory. The library itself depends
  on `askama`, `serde`, `fluent-templates` and, behind its `axum` feature,
  `axum`; a new library dependency reaches every console, so it is
  justified against what it adds.
- **Every crate you touch compiles, is clippy-clean, and is tested before
  you move on.**
- **Read the governing specification first** for any accessible-name, role,
  keyboard or contrast behaviour (`spec-adherence.md`).

## File and module size

A hand-written Rust file is at most 800 lines, its inline test module
included, and `scripts/checks/file-size.sh` holds the budget (Code is
written once, below). A file near the budget is split into a module folder
before it grows past it. The shape of a split is a folder `<name>/mod.rs`
holding the module doc, the `mod` declarations and the items every child
shares, with one child file per concern. A split never creates a test file
under `src/`, adds no `pub use`, and moves code without changing behaviour.

## Code is written once

No specification governs this: our own design (#536, ported from
VernumBOEK's and FerroEHR's rule of the same name).

- **Share knowledge and decisions, never text that only looks alike.** Two
  copies that change for the same reason (one interaction rule, one markup
  rule, one security rule) move to one owner in the same change. Two pieces
  with different reasons to change stay apart, however alike they read.
- **The owner is the module that owns the concern.** Across crates it is the
  crate lower in the layering (`ferrohealth-ui` below the gallery). Within
  the library it is the module whose component or part of the shell the code
  serves, and for the script modules the module the others import from
  (`calendar.js` for the pickers). It is never a `util`, `utils`, `helpers`
  or `common` module, whose name says nothing about what it holds, and never
  a function that takes a flag to choose between two behaviours.
- **One copy is fine, two are looked at, three are extracted.**
- **The ratchet measures it.** `scripts/checks/duplication.sh` runs jscpd
  (pinned by version and image digest in `docs/VERSIONS.md`, CI tools) over
  the Rust under `app/*/src/`, `crates/*/src/` and `tools/*/src/`, the script
  modules under `crates/*/scripts/` and the templates under `crates/*/templates/` in the
  `duplication` CI job, and holds the total of duplicated lines to the count
  in `scripts/checks/duplication.count`. A rise fails and names both places
  of each new block. A fall passes with a note, so your pull request leaves
  `scripts/checks/duplication.count` and
  `scripts/checks/duplication-baseline.txt` alone; the daily
  `.github/workflows/ratchet-tighten.yml` runs `--update` on `main` and
  pushes the lower count to `chore/ratchet-tighten`, whose pull request a
  maintainer opens. The baseline is the guard's own key of every
  counted block: its two paths and the sha256 of the lines at each, with no
  line number, so a change that leaves a block's lines alone leaves its key
  alone. `--update` never raises the count; a rise that has to stay is a
  hand edit of the file in a change that says why. Locally the guard takes a
  `jscpd` on `PATH` of the pinned version, then the pinned image through
  docker or podman, then the release binary
  `scripts/checks/duplication.sh --install` puts under `target/tools/`.
- **The threshold is 50 tokens over at least 5 lines**, jscpd's own
  defaults, within one format (`rust`, `javascript`, `markup`). The scan
  leaves out the `use` declarations and the modules' `import` statements, a
  file that opens with `// @generated`, a file from its first column-one
  `#[cfg(test)]` line on (the inline test module), and `tests/`. Measured on
  2026-10-31 at `4f897abe` with those exclusions, over 213 files and 60,344
  lines (175 Rust files, 31 script modules and the 7 templates jscpd finds
  tokens in):

  | Threshold | Blocks | Duplicated lines | Blocks across two files | Blocks in the gallery | Blocks in the script modules |
  | --- | --- | --- | --- | --- | --- |
  | 50 tokens, 5 lines | 36 | 307 | 19 | 13 | 10 |
  | 70 tokens, 5 lines | 8 | 87 | 5 | 1 | 4 |
  | 100 tokens, 5 lines | 1 | 21 | 1 | 0 | 0 |

  At 50 tokens the Rust carries 225 of the lines, the script modules 82 and
  the templates none. Raising the threshold drops the short blocks (a tone's
  glyph in the notice and the toast, the date and range pickers' key
  handling, the gallery's specimen rows written side by side) and keeps only
  the flow and topology cards' label text, yet each short block is code
  written twice as well, so 50 it is. Raising the threshold is a change
  to this section and to the guard together, with the table measured again.
- **A hand-written Rust file carries at most 800 lines**, its inline test
  module included. `scripts/checks/file-size.sh` measures every tracked `.rs`
  file under `app/*/src/`, `crates/*/src/` and `tools/*/src/` that does not
  open with `// @generated`, in the `file-size` CI job and per edit through
  `.claude/hooks/rust_fmt_clippy.sh`. The 10 files over the budget when the
  guard landed (3,416 lines over it in all) stand in
  `scripts/checks/file-size.list` with their length; a listed file may shrink
  and never grow. A shrink, a listed file back within the budget and a
  listed file gone pass with a note, and the same daily workflow lowers or
  removes the entry, so the list only tightens. A file over the budget is split by concern. The
  script modules keep their own byte budget (`scripts/checks/script-budget.sh`).

## No function only forwards to another

No specification governs this: our own design (#578, ported from
FerroEHR's rule of the same name, FerroHEALTH/FerroEHR#4663).

- **A function exists because it decides something.** It validates,
  branches, converts, draws, or names a concept its callers need. A
  function whose body is a single call, with its arguments passed through
  or merely borrowed and at most `.await`, `?`, `Ok(..)`, `Some(..)`,
  `.into()` or an error-only `.map_err` around it, is a forwarder. It adds
  a name and a hop and decides nothing, so its callers call the target
  directly, and the library's API may break to get there. A forwarder that
  fixes an argument (`wrap("<div class=\"fui-grid\">", children, "</div>")`)
  is still one: its callers pass the argument, and a flag that a bare
  `bool` would hide becomes a named type.
- **Three kinds are not forwarders.**
  - The methods of a trait implementation (`Display`, `From`, `Default`
    and the rest), which the trait requires. A binary's `main`, which the
    language requires, and a function a specification defines, whose doc
    comment cites the WCAG, WAI-ARIA, HTML Living Standard or CSS section
    that defines it with a `§`, stand with them.
  - A `pub` function in the crate's root API that re-exposes a private
    module's function under the crate's own name, kept only where the
    baseline records why.
  - A constructor (`new`, `with_*`, or an associated function that returns
    `Self`) that fixes an argument.
- **The ratchet measures it.** `tools/forwarders` parses every `.rs` file
  under `app/*/src/`, `crates/*/src/` and `tools/*/src/` with `syn`,
  leaving out a file that opens with `// @generated`, anything under
  `#[cfg(test)]`, and `tests/`. It reports a free function or inherent
  method whose body is one call, to a function or to a method of `self`,
  whose arguments are parameters, literals and constants, or references,
  borrows and clones of those. A call that reads a field of `self`, or
  calls a method of a field or a parameter, reaches what the caller
  cannot, so an accessor is never reported; nor are the `#[cfg]` arms of
  one function. `scripts/checks/forwarders.sh` runs it in the `forwarders`
  CI job and per edit through `.claude/hooks/rust_fmt_clippy.sh`, and
  holds the total to `scripts/checks/forwarders.count`.
  `scripts/checks/forwarders-baseline.txt` keys each counted forwarder by
  its path, owner and target call, with no line number, and a key may
  carry a tab and the reason the forwarder stays. A rise fails and names
  each new forwarder, its target and its place. A fall passes with a note,
  so your pull request leaves both files alone, and the daily
  `.github/workflows/ratchet-tighten.yml` lowers the count on `main`, as it
  does for the duplication ratchet. `--update` never raises the count and
  keeps each reason while its key stands. The 37 forwarders the guard
  found when it landed, measured on 2026-11-03 at `4028d11f`, stand in the
  baseline.

## Comments and documentation

The comment and doc-comment discipline lives in **`comments.md`** (RFC 505
plus RFC 1574): line comments only, `// TODO(#NNNN):` / `// NOTE:` as the only
markers, a NOTE as a citation plus one sentence (at most 3 lines), plain `//`
runs at most 8 lines, the summary-line convention for doc comments.

## Default values live in the struct's `Default` impl (RFC 3681 shape)

One `impl Default` per struct, every default value inline. The per-field
`#[serde(default = "path")]` form, a zero-argument `fn default_x()`, and a
single-reader `const DEFAULT_X` are all banned: each puts a field's default
somewhere other than its own struct. A `const` with more than one consumer
stays a constant (`table::model::DEFAULT_SIZE` is read by the paging, the
pager and the query writer).

## RFC-grounded conventions

| RFC | The rule | Enforcement |
|---|---|---|
| 0505 + 1574 | comment and doc conventions | `comments.md` + `too_long_first_doc_paragraph` |
| 3681 | a field's default lives inline in its `Default` impl | §Default values above |
| 3107 | `#[derive(Default)]` + `#[default]` where the default is a variant | `clippy::derivable_impls` (deny) |
| 0199 / 0344 / 0430 | `as_`/`to_`/`into_` cost conventions, naming | `clippy::wrong_self_convention` |
| 1940 | `#[must_use]` on functions whose result is the point | `clippy::must_use_candidate` (pedantic) |
| 2383 | every suppression carries `reason = "…"` | `clippy::allow_attributes_without_reason` (deny) |
| 1946 | intra-doc links over bare paths | `rustdoc::broken_intra_doc_links` (deny) |

## Type and error conventions

- No `unwrap`/`expect` outside `#[cfg(test)]`; `todo!()` and
  `unimplemented!()` are banned. An error a caller branches on is a type,
  implementing `std::error::Error` (the library has no `thiserror`: its one
  small error, `theme::UnknownTheme`, does not earn the dependency).
- Model closed sets as Rust `enum`s (`Tone`, `Variant`, `Theme`, `Brand`,
  `Language`, `Kind`, `Direction`, `StateSource`).
- **No `use X as Y` import renaming.** Fix the name at its definition, or
  qualify one at the use site.
- `std::sync::LazyLock` for statics. Edition 2024, resolver 3, `cargo fmt`
  clean.
- Suppressions are `#[expect(lint, reason = "…")]` scoped to the smallest
  item (`reliability.md`).

## Documentation (`missing_docs` is enforced workspace-wide)

Every public item carries a doc comment; intra-doc links resolve in the scope
of the module where the item is defined; literal square brackets are escaped.

## Edition-2024 standing guidance

- `if let` scrutinee temporaries drop before `else` (`reliability.md`).
- RPIT captures every in-scope lifetime and type parameter by default: a view
  function that must not capture a borrowed argument says so with
  `impl IntoView + use<>`, and then takes owned or `&str` arguments rather
  than `impl Into<String>`.
- `Future` and `IntoFuture` are in the prelude.

## Naming

A name says what the thing is in its domain. Classes are `fui-` and the
component's name (`fui-code-panel`); roles are the design's names
(`--surface-raised`, `--edge-control`, `--accent-ink`). Modules are named for
their contents.

## No Python, anywhere

The tooling languages are bash and Rust. Python is banned in scripts and
embedded in shell; reach for `jq`, `awk`, `sed`, `grep`, or a Rust tool.

## What not to do

- Do not write a `style` attribute, inject `<style>`, or reach for a
  CSS-in-Rust runtime: the consoles run `style-src 'self'`.
- Do not define a server function, fetch, or touch browser storage in the
  library.
- Do not add a class a component writes without defining it in the
  stylesheet (`tests/it/stylesheet.rs` fails).
