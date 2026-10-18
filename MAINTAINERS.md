<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Maintainers and access continuity

## Roster

| Person        | GitHub                                           | Role              | Since      |
|---------------|--------------------------------------------------|-------------------|------------|
| Ruben Talstra | [@rubentalstra](https://github.com/rubentalstra) | Maintainer (sole) | 2026-10-18 |

**The bus factor of this project is one.** One person can merge, release and
publish. The Licensor and copyright holder is Cadasto B.V. (`LICENSE`), which
handles the business side; the maintainer handles the technical side.

## Publishing identities

| Identity                                     | What it publishes                                    | Held by                                     |
|----------------------------------------------|------------------------------------------------------|---------------------------------------------|
| The GitHub organization `FerroHEALTH`        | the repository, releases, issues and settings        | the organization's owners                   |
| The crates.io crate `ferrohealth-ui`         | the library, through Trusted Publishing from `release.yml` and `publish-crates.yml` behind the `crates-io` environment | the maintainer's crates.io account, the crate's owner |
| The commit- and tag-signing key              | the verified signature on every commit and tag       | the maintainer, on his own hardware         |
| `GITHUB_TOKEN` (ephemeral, per workflow run) | the release asset and its Sigstore provenance        | GitHub, minted per run                      |

## If the maintainer is unavailable

Nothing published disappears: a published release is immutable, its tag
cannot be moved, and a crates.io version stays downloadable. Nothing new
ships. The licence converts to Apache 2.0 four years after each version and
every gate is a committed script, so a fork is a complete continuation. A
console that depends on FerroUI pins an exact version and can keep a fork it
builds.

## Adding a maintainer

The route is in [GOVERNANCE.md](GOVERNANCE.md).
