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
   branch `chore/screenshots`:

   ```sh
   scripts/release/gallery-site.sh --release
   FERROUI_SCREENSHOTS=1 cargo nextest run --locked -p ferrohealth-ui-gallery \
     -E 'test(every_page_is_captured) | test(every_state_is_captured)'
   FERROUI_SCREENSHOT_GUARD=1 cargo nextest run --locked -p ferrohealth-ui-gallery \
     -E 'test(every_shot_has_its_light_and_dark_image)'
   python3 scripts/release/hero.py
   ```

   Commit the whole `docs/screenshots/` tree, the images a removed page or
   state left behind deleted, and the README's hero, `docs/hero.png` and
   `docs/hero-dark.png`, which `hero.py` composes from the new images. CI
   runs the screenshot guard on that branch. Look at every image that
   changed, in every brand, light and dark.

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
   the section as the release notes.
3. **Say what was looked at.** The pull request body says, per screen and
   per state, what was checked and what was fixed
   (`.claude/rules/leptos-ui.md` §11).
4. **The gates pass.** CI runs the screenshot guard on every `release/`
   branch, so a release pull request cannot merge while a listed page or
   state lacks its images. The crate-version guard checks the version, the
   lock and the changelog section agree.

## The tag

After the release pull request merges, tag its merge commit signed and push
the tag:

```sh
git tag -s vX.Y.Z -m vX.Y.Z <merge-commit>
git push origin vX.Y.Z
```

`release.yml` validates the tag against the workspace version, takes the
notes from the `[X.Y.Z]` section, publishes the release, and its `crates`
leg waits on the `crates-io` environment's reviewer. Its last job closes the
milestone.

## Checklist

- [ ] The milestone `vX.Y.Z` holds no open item.
- [ ] The refresh pull request (`chore/screenshots`) merged after the last
      feature pull request of the milestone, and its images were looked at.
- [ ] `release/vX.Y.Z` bumps the version and the lock.
- [ ] `scripts/release/changelog.sh --assemble X.Y.Z YYYY-MM-DD` ran;
      `changelog.d/` holds only its README.
- [ ] The pull request body says per screen and state what was checked.
- [ ] CI is green on the release pull request, the screenshot guard included.
- [ ] The tag is signed, on the merge commit.
