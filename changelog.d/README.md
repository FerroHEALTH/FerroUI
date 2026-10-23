<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Changelog fragments

A pull request with an effect a console sees adds one file here and leaves
`CHANGELOG.md` alone, so two pull requests never touch the same lines.

- **Name:** `<issue>-<kebab-slug>.<section>.md`, for example
  `243-changelog-fragments.changed.md`. `<section>` is a Keep a Changelog
  1.1.0 section in lower case: `added`, `changed`, `deprecated`, `removed`,
  `fixed` or `security`. A change with entries in two sections adds two
  files. The one other name, `<issue>-<kebab-slug>.security-reason.md`, is
  the reason paragraph of § Security entries.
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
(`docs/release.md`). `scripts/release/changelog.sh --preview` prints
`CHANGELOG.md` as the cut would write it, with the fragments under
`[Unreleased]`. This README is not a fragment.

## Security entries

Two rules of Regulation (EU) 2024/2847 hold every Security section, and
`scripts/checks/changelog-security.sh` checks both: CI runs it on every pull
request over the preview of `[Unreleased]`
(`scripts/checks/changelog-unreleased.sh`), and the release `plan` job over
the section it cuts (`scripts/release/plan.sh`). A section released before
the rules is never checked again.

- **Every entry names its advisory** (Annex I Part II(4), (8)). A
  `security` fragment carries the `GHSA-` id of the FerroUI security
  advisory drafted for the fix and published with the release
  (`SECURITY.md`, Security advisories), or, for a dependency finding an
  OpenVEX statement shows does not affect FerroUI, the path of that
  statement under `security/vex/`, which must exist. A RUSTSEC id alone is
  not enough: name it beside the advisory.
- **A security fix ships alone, or says why not** (Annex I Part II(2)). A
  release whose Security section sits beside any other section opens that
  section with one paragraph, `Shipped with functional changes because:`
  and the reason on the same line, naming why a security-only patch of the
  newest release was not the way to ship it. The paragraph is a fragment of
  its own, `changelog.d/<issue>-<kebab-slug>.security-reason.md`: plain text
  that opens with that prefix, with no list item and no blank line inside.
  At most one exists at a time; the cut places it first under
  `### Security` and refuses it when no Security entry is pending. Whoever
  adds the first Security fragment beside other pending changes adds it,
  and whoever cuts the release rereads it.
