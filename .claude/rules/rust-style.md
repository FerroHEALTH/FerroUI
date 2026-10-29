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

A hand-written Rust file is at most 1000 lines, and a file over 750 lines is
split into a module folder before it grows further. The shape of a split is
a folder `<name>/mod.rs` holding the module doc, the `mod` declarations and
the items every child shares, with one child file per concern. A split never
creates a test file under `src/`, adds no `pub use`, and moves code without
changing behaviour.

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
