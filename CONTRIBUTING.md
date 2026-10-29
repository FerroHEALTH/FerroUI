<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Contributing to FerroUI

FerroUI is the shared server-rendered component library of the FerroHEALTH consoles,
published as `ferrohealth-ui`, with a gallery that shows every component in
every product theme. The working discipline is [`CLAUDE.md`](CLAUDE.md) and
the component rule set is
[`.claude/rules/console-ui.md`](.claude/rules/console-ui.md). Read both before
making a change. The contribution terms every FerroHEALTH product shares are
in the family book at <https://ferrohealth.eu/docs/licensing.html#contributions>.

## Build and test

```
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
bash scripts/checks/markup-rules.sh crates/ferrohealth-ui/src crates/ferrohealth-ui/templates app/ferrohealth-ui-gallery/src
bash scripts/checks/script-budget.sh
```

The gallery runs with `cargo run -p ferrohealth-ui-gallery` and serves on
`127.0.0.1:3000` (`GALLERY_ADDR` moves it).
Its browser journeys and screenshots need Chrome for Testing and its
chromedriver (`scripts/release/journey-tools.sh` installs both):

```
FERROUI_JOURNEYS=1 cargo nextest run -p ferrohealth-ui-gallery --locked
FERROUI_SCREENSHOTS=1 cargo nextest run -p ferrohealth-ui-gallery --locked screenshots
```

A change you can see is captured and looked at, every image it changed,
before you ask for review. Leave `docs/screenshots/` as `main` has it: CI
uploads the capture as an artifact for the reviewer, and the refresh pull
request commits the images (`.claude/rules/parallel-prs.md`).

A change to a workflow or a shell script also needs the linters CI runs:

```
zizmor --min-severity=low .github/
actionlint
shellcheck --severity=style <the shell files you touched>
```

## Standards are the authority

WCAG 2.2 (Level AA), WAI-ARIA 1.2 with the Authoring Practices, the HTML
Living Standard and the CSS specifications decide a component's markup,
keyboard path and colours; the Askama book decides how a template is written. Cite the
section in the pull request. Where none governs a choice, say so: "no
specification governs this: our own design".

## Branches and commits

- Branch names use conventional types: `feat/<slug>`, `fix/<slug>`,
  `chore/<slug>`, `docs/<slug>`, `refactor/<slug>`, `perf/<slug>`,
  `test/<slug>`, `ci/<slug>`, `build/<slug>`, `release/<slug>`.
- Commit messages describe only the change. Do **not** add AI or assistant
  attribution, co-author trailers, or "generated with" lines anywhere.
- **Sign your commits.** `main` refuses an unsigned commit.

## Pull requests

- Declare `Closes #N`, one keyword per issue.
- Never weaken, skip, or delete a test to make a build pass.
- Add a changelog fragment under `changelog.d/` for any change a console
  sees (`changelog.d/README.md`), and name what a console must change when an
  API breaks. Never edit `CHANGELOG.md` itself; the release assembles it.
- Keep every `mod` list alphabetical, and put a gallery state or word in its
  area's own file (`.claude/rules/parallel-prs.md`).
- Write prose to `.claude/rules/writing-style.md`.

## Using AI tools

You may. If a contribution has AI-generated content, say so in the pull
request description: which tool, and what it did. Disclosure lives in the
description, never in a commit trailer. You remain responsible for the
submission in full.

## Security

Report vulnerabilities privately; see [`SECURITY.md`](SECURITY.md). Do not
open a public issue for a security problem.

## Licensing of contributions

FerroUI's own code is licensed under the Business Source License 1.1
([`LICENSE`](LICENSE)). These are the terms the family book states for every
FerroHEALTH product
([Contributions](https://ferrohealth.eu/docs/licensing.html#contributions)).
By submitting a contribution you:

1. certify that you wrote it, or otherwise have the right to submit it under
   these terms;
2. license it under the Business Source License 1.1 as applied to the version
   it lands in, including that version's Change License, so it becomes Apache
   2.0 with the rest of that version; and
3. grant the Licensor named in `LICENSE`, Cadasto B.V., a perpetual,
   irrevocable, worldwide, royalty-free, transferable right to use, reproduce,
   modify, distribute, sublicense and relicense the contribution as part of
   the Licensed Work under any terms, including commercial licences.

You keep your copyright. There is no separate agreement to sign: the pull
request template carries a checkbox recording your acceptance of these terms,
and a pull request from a person does not merge without it (the
`contribution-licence` check, backed by
`scripts/checks/contribution-licence.sh`).
