<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Changelog fragments

A pull request with an effect a console sees adds one file here and leaves
`CHANGELOG.md` alone, so two pull requests never touch the same lines.

- **Name:** `<issue>-<kebab-slug>.<section>.md`, for example
  `243-changelog-fragments.changed.md`. `<section>` is a Keep a Changelog
  1.1.0 section in lower case: `added`, `changed`, `deprecated`, `removed`,
  `fixed` or `security`. A change with entries in two sections adds two
  files.
- **Content:** the entry exactly as it reads in `CHANGELOG.md`, one or more
  list items that open with `- ` and continue on lines indented by two
  spaces, with no heading and no blank line inside. A fragment carries no
  SPDX header: its text moves into `CHANGELOG.md`, which carries the header.
- **Issue reference:** the cut ends the first paragraph of each entry with
  `(#<issue>)`, the number the file name starts with, unless the entry already
  names that issue, so the release notes link every entry to its issue
  without the fragment spelling it out.

`scripts/release/changelog.sh --check` validates every fragment, and CI runs
it. `scripts/checks/changelog-guard.sh` refuses a pull request that adds no
fragment (unless it carries the `no-changelog` label) or edits the
`[Unreleased]` section of `CHANGELOG.md` (unless it is the release cut). The
release pull request runs `scripts/release/changelog.sh --assemble <version>
<date>`, which moves the fragments and any entries still under `[Unreleased]`
into `CHANGELOG.md` under the new version and deletes the fragments
(`docs/release.md`). This README is not a fragment.
