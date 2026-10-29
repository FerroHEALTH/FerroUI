<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Cutting a release

No specification governs the release procedure: our own design. A release
is the signed tag `vX.Y.Z` on the merge commit of the release pull request on
`main`; `.github/workflows/release.yml` does the rest
(`.claude/rules/ci-cd.md`, `.claude/rules/crates-publishing.md`).

## Before the release pull request

1. **The milestone is empty.** Every issue and pull request of the milestone
   titled `vX.Y.Z` is closed or moved; `release.yml` refuses the tag
   otherwise (`scripts/release/milestone.sh check vX.Y.Z`).
2. **The screenshots are refreshed.** Feature pull requests commit no image
   under `docs/screenshots/`, so the refresh pull request lands first, on the
   branch `chore/screenshots`. The images come from CI, never from your own
   machine: a capture draws its fonts as the system that runs it does, so
   only captures made on the same runner image can be compared. Push the
   branch, wait for its `gallery (capture <brand>)` jobs, one per brand,
   which capture every page and state, and download the
   `gallery-screenshots` artifact the `gallery-screenshots` job merges
   from them:

   ```sh
   gh run download <run-id> -n gallery-screenshots -D docs/screenshots
   python3 scripts/release/hero.py
   ```

   Commit the whole `docs/screenshots/` tree, the images a removed page or
   state left behind deleted, and the README's hero, `docs/hero.png` and
   `docs/hero-dark.png`, which `hero.py` composes from the new images. CI
   runs the screenshot guard on that branch. Look at every image that
   changed, in every brand, light and dark. The capture runs with
   `prefers-reduced-motion: reduce`, so an unchanged page captures
   pixel-identical and every image that changed is a change to look at.
   On the refresh and the release pull request CI captures every page and
   state again and compares each committed image with its fresh capture by
   its pixels (`every_committed_screenshot_shows_what_a_fresh_capture_draws`),
   so the refresh proves its images show the tree it lands on. Before the
   images are committed that check fails, naming each image that changes:
   that list is what to look at. A release pull request opened before the
   refresh fails there too. Every page's footer names the version, so
   the release pull request commits the artifact of its own capture job
   too, after the bump, and carries the `screenshots` label: pass it to
   `gh pr create --label screenshots`, and CI reads it on the first run.
   A label added later re-runs that run's `detect` job and every job after
   it once the run completes (`pr-guards.yml`).

   A capture on your own machine (`FERROUI_SCREENSHOTS=1
   scripts/chrome-lock.sh cargo nextest run --locked -p
   ferrohealth-ui-gallery -E 'test(every_page_is_captured) |
   test(every_state_is_captured)'`) is for looking at a change before you
   push it; it is never committed.

## The release pull request

On the branch `release/vX.Y.Z`:

1. **Bump the version.** Set the `[workspace.package]` version in
   `Cargo.toml` and run `cargo update -w`.
2. **Assemble the changelog.** Every change since the last release is a
   fragment under `changelog.d/` (`changelog.d/README.md`), and entries
   written before the fragments may still sit under `[Unreleased]`. Run

   ```sh
   scripts/release/changelog.sh --assemble X.Y.Z YYYY-MM-DD
   ```

   It writes a `## [X.Y.Z] - YYYY-MM-DD` section under a fresh empty
   `[Unreleased]`, holding the `[Unreleased]` entries and then the
   fragments in Keep a Changelog order; it moves the `[Unreleased]` link
   reference on, adds the version's, and `git rm`s the fragments. It refuses
   a malformed fragment, a version that already has a section, and a release
   with no entry. Commit `CHANGELOG.md` and the removals together, and read
   the section as the release notes. A Security section names an advisory
   in every entry and, beside other sections, opens with its reason
   (`changelog.d/README.md`, Security entries); reread that reason now.
3. **Say what was looked at.** The pull request body says, per screen and
   per state, what was checked and what was fixed
   (`.claude/rules/console-ui.md` §11).
4. **The gates pass.** CI runs the screenshot guard on every `release/`
   branch, so a release pull request cannot merge while a listed page or
   state lacks its images. The crate-version guard checks the version, the
   lock and the changelog section agree. A release pull request is merged
   only after its own `gallery (capture)` job, the state capture and the
   pixel check included, has passed, and never directly past the queue.

## Preflight

The `plan` job verifies every release fact at the tagged commit before
anything is built or published. Its checks are one list,
`scripts/release/plan.sh`, and `scripts/release/preflight.sh` runs the same
list on your machine:

```sh
scripts/release/preflight.sh vX.Y.Z
scripts/release/preflight.sh vX.Y.Z --notes /tmp/notes.md   # keep the notes
```

Run it on `release/vX.Y.Z` once the cut is committed, and again on `main`
after the merge, at the commit you are about to tag. It exits 0 when the
`plan` job would pass, names every check that refuses, and prints the path
of the release notes it composed, so you read them before they are
published. You need `gh` logged in, because the milestone check reads the
tracker.

| Check | Script |
|---|---|
| The milestone titled as the tag holds no open item | `scripts/release/milestone.sh check` |
| The `[workspace.package]` version of `Cargo.toml` names the tag's version | `plan.sh` |
| Every section of the changelog holds Keep a Changelog types, each once | `scripts/checks/changelog-structure.sh` |
| The section being cut opens its `### Security` with the reason when it sits beside other changes, and every Security entry names a GHSA id or an existing `security/vex/` path | `scripts/checks/changelog-security.sh --section X.Y.Z` |
| The changelog section becomes the notes, with the support period appended: the month five years after the heading's date (Regulation (EU) 2024/2847 Art 13(19)) | `plan.sh notes` |

`scripts/release/plan.sh prev-tag vX.Y.Z` prints the previous release by
version order from the tags of origin, so a tag pushed whose release never
published still counts. To add a release fact, add a function and a line to
`list()` in `scripts/release/plan.sh`; the `plan` job and the preflight
both pick it up.

## The tag

After the release pull request merges and the preflight passes on `main`,
tag its merge commit signed and push the tag:

```sh
git tag -s vX.Y.Z -m vX.Y.Z <merge-commit>
git push origin vX.Y.Z
```

`release.yml` runs the preflight's list at the tag, takes the notes from
the `[X.Y.Z]` section with the support period appended, publishes the
release, and its `crates` leg waits on the `crates-io` environment's
reviewer. Its last job closes the milestone.

## A security fix

A reported vulnerability gets a private draft GitHub security advisory the
day it is confirmed (`SECURITY.md`, Security advisories). The fix ships in
a patch release of the newest version, alone where that is technically
feasible. Its `security` fragment names the draft's `GHSA-` id; a
dependency finding that does not affect FerroUI names its OpenVEX
statement under `security/vex/` instead, and gets no advisory. When the
release carries other changes too, a `security-reason` fragment says why
(`changelog.d/README.md`). Publish the advisory, with the release version as
its patched version, in the same hour as the release.

An accepted advisory in `deny.toml` needs its statement in
`security/vex/rust-advisories.toml`, and `scripts/security/vex-generate.sh`
regenerates the published document (`security/vex/README.md`).

## Checklist

- [ ] The milestone `vX.Y.Z` holds no open item.
- [ ] The refresh pull request (`chore/screenshots`) merged after the last
      feature pull request of the milestone, and its images were looked at.
- [ ] `release/vX.Y.Z` bumps the version and the lock.
- [ ] `scripts/release/changelog.sh --assemble X.Y.Z YYYY-MM-DD` ran;
      `changelog.d/` holds only its README.
- [ ] `scripts/release/preflight.sh vX.Y.Z` passes, and its notes were read.
- [ ] The pull request body says per screen and state what was checked.
- [ ] CI is green on the release pull request, the screenshot guard included.
- [ ] The tag is signed, on the merge commit.
