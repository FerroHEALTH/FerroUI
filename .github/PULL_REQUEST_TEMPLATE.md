<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

## What changed and why

<!-- Describe the change and the reason for it. Name what a console must change if an API breaks. -->

Closes #NNN

## Checklist

- [ ] Accessibility decisions cite WCAG 2.2, WAI-ARIA 1.2 or the Authoring Practices, not memory and not another library.
- [ ] Rust gates pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace --locked`, `cargo doc` with `RUSTDOCFLAGS=-D warnings`, and `cargo deny check`.
- [ ] I looked at every gallery screenshot a visible change moves (the `gallery-screenshots` artifact), and committed none: the refresh pull request does (`.claude/rules/parallel-prs.md`). New states are named below.
- [ ] A new or changed component was compared with every console's version (`docs/qa/`), and the best one on its merits was restyled to the library's tokens; the winner and the reason are stated below.
- [ ] Its interactive states (pointer, focus, open, each tone) are captured by their area's file under `tests/it/states/` in every brand, light and dark.
- [ ] A release pull request says, per screen and per state, what was checked and what was fixed.
- [ ] Shell and workflow files are clean: `shellcheck --severity=style`, `actionlint`, `zizmor --min-severity=low .github/`.
- [ ] A fragment under `changelog.d/` records the change, if a console sees it (`changelog.d/README.md`); `CHANGELOG.md` is untouched.
- [ ] No patient data in a fixture, test, gallery row or example.
- [ ] Every commit is signed.
- [ ] No AI or assistant attribution anywhere in the commits or this PR.
