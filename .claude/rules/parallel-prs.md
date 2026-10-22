<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Parallel pull requests: no shared lines

No specification governs this: our own design (owner decision, 2026-10-22).
Many pull requests are open at once, and each merge used to put every other
one in conflict. The rules below keep two pull requests that work on
different components off each other's lines. Each one has a check.

## The rules

- **A changelog entry is a fragment.** Add
  `changelog.d/<issue>-<kebab-slug>.<section>.md`, `<section>` one of
  `added`, `changed`, `deprecated`, `removed`, `fixed`, `security`, holding
  the entry as it will read in `CHANGELOG.md` (`changelog.d/README.md`).
  Never edit the `[Unreleased]` section of `CHANGELOG.md`; the release pull
  request assembles the fragments (`docs/release.md`). A change no console
  sees carries the `no-changelog` label instead. Check:
  `scripts/checks/changelog-guard.sh` and `scripts/release/changelog.sh
  --check`, in the CI `guards` job.
- **Gallery states live in their area's file.** A state is added to the
  `STATES` list and the capture function of its area under
  `app/ferrohealth-ui-gallery/tests/it/states/` (overview, tables, menus and
  so on). A new area is a new file there, a `mod` line and an `AREAS` entry
  in `states.rs`, and a call in the capture driver. Check:
  `every_state_is_listed_once` and `a_state_states_does_not_list_is_refused`
  over every area's list.
- **Gallery words live in their area's files.** A message is declared in
  `app/ferrohealth-ui-gallery/src/locale/<area>.rs` and its English in
  `locales/en/<area>.ftl`, the area being the gallery page that draws it
  (`shell` for the shell and a word more than one page draws), each file in
  the order of the ids. Check: `the_catalog_holds_every_message_and_no_other`
  over every catalog file; a constant declared twice does not compile.
- **Feature pull requests commit no screenshot.** CI captures the pages and
  uploads them as the `gallery-screenshots` artifact, which the reviewer
  opens; locally you capture and look as before (`testing.md`), then leave
  `docs/screenshots/` as `main` has it. The refresh pull request, branch
  `chore/screenshots`, made by the coordinator after each wave and before
  each release, commits the whole set. Check:
  `scripts/checks/screenshot-paths.sh` refuses an image in any other pull
  request.
- **The screenshot guard runs on the refresh.** The test that every listed
  page and state has both images and that no other image is kept runs behind
  `FERROUI_SCREENSHOT_GUARD=1`: in CI on `chore/screenshots`, on every
  `release/` branch, with the `screenshots` label and on a manual run. A
  feature pull request that adds a state passes without its image, says the
  state's name in its body, and the refresh pull request before the tag
  draws it (`docs/release.md`).
- **Module lists are alphabetical.** Every top-level `mod` declaration of a
  file, `tests/it/main.rs` and the `view`, `widget` and `island` lists
  included, is in ascending order of its name, so two new modules land on
  different lines. Check: `scripts/checks/mod-order.sh`.

## On a conflict anyway

Rebase onto `origin/main`, keep both sides, run the gates, and push with
`--force-with-lease` to your own branch. A conflict in one of the files
above means a rule was broken; say so in the pull request.
