<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Governance

How decisions get made in FerroUI, who makes them, and how that changes.

## Current structure: one maintainer

FerroUI has a single maintainer ([MAINTAINERS.md](MAINTAINERS.md)) who holds
final say on every technical decision: what gets built, merged and released.
There is no steering committee and no vote.

Cadasto B.V. is the Licensor and copyright holder (`LICENSE`) and handles the
business side, the commercial licence included: write to
[info@cadasto.com](mailto:info@cadasto.com) or use
<https://www.cadasto.com/contact/>. The maintainer handles the technical side.

## The consoles are the customers

FerroUI exists for the FerroHEALTH consoles. A change that alters how a
console's screens look or behave names that effect in its pull request and
changelog entry, and a console moves its exact version pin deliberately,
after looking at its own screens.

## Where decisions are recorded

| Kind of decision          | Where it lives                                          |
|---------------------------|---------------------------------------------------------|
| What to build, and why    | the issue, with its acceptance criteria                 |
| How a component behaves   | the pull request, citing WCAG, WAI-ARIA or HTML         |
| Component rules           | `.claude/rules/leptos-ui.md`                            |
| A release's contents      | its `CHANGELOG.md` section and its milestone            |

## Becoming a maintainer

Sustained, reviewed contributions over several releases, a demonstrated grasp
of the component rules and the accessibility bar, and the maintainer's
invitation. A new maintainer gains a row in [MAINTAINERS.md](MAINTAINERS.md)
and their handle in [`.github/CODEOWNERS`](.github/CODEOWNERS).

## Changing this document

By pull request, merged by the maintainer.
