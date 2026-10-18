<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

## What changed and why

<!-- Describe the change and the reason for it. Name what a console must change if an API breaks. -->

Closes #NNN

## Licensing of contributions

- [ ] I accept the terms in [CONTRIBUTING.md § Licensing of contributions](../CONTRIBUTING.md#licensing-of-contributions): I have the right to submit this work, I license it under the project licence of the version it lands in, and I grant the Licensor the relicensing right stated there.

## Checklist

- [ ] Accessibility decisions cite WCAG 2.2, WAI-ARIA 1.2 or the Authoring Practices, not memory and not another library.
- [ ] Rust gates pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, the wasm32 clippy runs, `cargo nextest run --workspace --locked`, `cargo doc` with `RUSTDOCFLAGS=-D warnings`, and `cargo deny check`.
- [ ] A visible change refreshed the gallery screenshots, and I looked at every image it changed.
- [ ] Shell and workflow files are clean: `shellcheck --severity=style`, `actionlint`, `zizmor --min-severity=low .github/`.
- [ ] `CHANGELOG.md` has an `[Unreleased]` entry, if a console sees the change.
- [ ] No patient data in a fixture, test, gallery row or example.
- [ ] Every commit is signed.
- [ ] No AI or assistant attribution anywhere in the commits or this PR.
