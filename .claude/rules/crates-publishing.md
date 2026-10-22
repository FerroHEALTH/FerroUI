<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Published crate discipline (crates.io)

`crates/ferrohealth-ui` is published on crates.io as `ferrohealth-ui` so the
four FerroHEALTH consoles can depend on it. The gallery
(`app/ferrohealth-ui-gallery`) is never published. The crate is BUSL-1.1.
Published versions are immutable, so version hygiene is a hard rule.

The name `ferroui` on crates.io belongs to an unrelated project: never name a
crate `ferroui` and never publish under it.

## One version line

The workspace `version` in the root `Cargo.toml` is the library's version and
the release tag `vX.Y.Z`. The library inherits it (`version.workspace =
true`), so the tag, the crate and the changelog section always agree.

## The bump rule

- **A release bumps the workspace version, the changelog section and the
  lock together** (`cargo update -w`), in the release pull request, whose
  changelog section `scripts/release/changelog.sh --assemble` writes from the
  fragments under `changelog.d/` (`docs/release.md`).
- Consumers pin the crate exactly (`ferrohealth-ui = "=0.1.0"`): Cargo reads a
  `0.x` caret requirement as compatible across patches
  (<https://doc.rust-lang.org/cargo/reference/semver.html>), and a console
  moves its pin deliberately, after looking at its own screens.
- A pull request that breaks a public API says so in its body and its
  changelog fragment, naming what a console changes; while the line is `0.x`,
  such a release moves the minor version.

## The publish lane is resumable and verified

`scripts/release/publish-crates.sh` (`publish` / `verify` / `version`)
uploads the crate, counts "already exists" as done, and reads the registry
back before reporting success. Two lanes call it: the `crates` leg of
`release.yml` on a `v*` tag (the primary path, paused by the `crates-io`
environment's required reviewer) and `publish-crates.yml` on a manual
dispatch (a dry run by default; `publish = true` is the recovery path). Both
authenticate with crates.io Trusted Publishing (OIDC through
`rust-lang/crates-io-auth-action`); no long-lived crates.io token exists in
the repository. Neither lane restores a build cache (`ci-cd.md`).

`cargo publish -p ferrohealth-ui --dry-run --locked` runs on every pull
request as the `publish-dry-run` job.

## Owner steps (done once)

1. The name was reserved at `0.0.0` by the owner's crates.io login, since a
   first version cannot use Trusted Publishing.
2. On crates.io, the crate's Trusted Publishing carries two GitHub entries:
   repository owner `FerroHEALTH`, repository `FerroUI`, workflows
   `release.yml` and `publish-crates.yml`, environment `crates-io`.
3. The `crates-io` GitHub environment carries the owner as required reviewer
   and admits `main` and `v*` tags.

## Official documentation (durable citations)

- Trusted Publishing on crates.io: <https://crates.io/docs/trusted-publishing>
- `cargo publish`: <https://doc.rust-lang.org/cargo/commands/cargo-publish.html>
