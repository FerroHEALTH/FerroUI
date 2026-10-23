---
paths:
  - ".github/**"
  - "scripts/**"
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# CI/CD and supply-chain discipline

No specification governs this: our own design, grounded in the OWASP GitHub
Actions Security Cheat Sheet, SLSA v1.0, OpenSSF Scorecard, and Sigstore.

## What runs

- `.github/workflows/ci.yml`: the gate on every pull request, merge-group run
  and push to `main`. The workflow tier (zizmor, actionlint, shellcheck, the
  comment-style guard, the Lucide vendor check), the Rust tier (`cargo fmt
  --all --check`; `cargo clippy --workspace --all-targets --locked -- -D
  warnings`; the two wasm32 clippy runs of `leptos-ui.md` §10 and the
  browser-floor build of `scripts/checks/browser-floor.sh`; `cargo nextest
  run --workspace --locked` plus `cargo test --doc --locked`; `cargo doc` with
  `RUSTDOCFLAGS=-D warnings`; `cargo deny check`; `cargo publish -p
  ferrohealth-ui --dry-run --locked`; the crate-version guard and
  `dependency-review-action` on pull requests), and the gallery tier (two
  jobs side by side, each over its own release bundle: the path and size
  checks of `scripts/checks/gallery-bundle.sh` with the journeys, and the
  screenshot capture in Chrome for Testing, the screenshots uploaded as an
  artifact). The design tier builds the pages of `design/` and runs
  `design/tools/check.sh` per design set on macOS with Chrome at the
  gallery's release, every browser check reading one render of each page
  (`design/tools/sweep.py`); when a change reaches at most two sets, each
  console runs as two jobs (`FERROUI_DESIGN_PART`), since GitHub runs at
  most five macOS jobs at once; `design-licence` holds every design file to the
  SPDX lines (`scripts/checks/spdx.sh`). The contrast test runs inside
  nextest. The `conclusion` job is the single required status check on
  `main`: it passes when every job it needs passed or was skipped.
  The `guards` job runs on every event: the changelog fragments well
  formed (`scripts/release/changelog.sh --check`) and every module list
  alphabetical (`scripts/checks/mod-order.sh`), and on a pull request the
  changelog guard and the screenshot-path guard (`parallel-prs.md`), with
  the labels read when it runs. The `test` job runs the screenshot guard on
  the refresh pull request, a `release/` branch and a manual run.
- **What a change reaches** is `scripts/checks/affected.sh`'s answer over
  the paths it touches, read by the `detect` job: a pull request's merge
  commit against its base, a merge group against the queue's base, and
  everything on a push to `main` or a manual run. The gallery job runs when
  the crates, the gallery, the workspace manifests or the gallery scripts
  change; a design set runs when its own sources, its brand's theme, the
  shared design tools, templates and stylesheets, or the crate's stylesheet
  change. Markdown reaches neither. The CI workflow, its actions and the
  classifier itself reach everything, so a change to the gate runs the
  whole gate. Each rule of the table has a case in its `--self-test`.
- **The merge queue reruns what a merge can break**: clippy, nextest and
  the doctests, rustdoc, cargo-deny, the publish dry run, the gallery job
  and the design sets the group reaches. It leaves out the per-file lints
  (zizmor, actionlint, shellcheck, comment-style, todo-issues, rustfmt,
  lucide, design-licence): a pull request joins the queue only with
  `conclusion` green on its head, those lints read each file alone, and the
  push to `main` runs every one of them again over the merged tree.
- **The Rust build cache** is `Swatinem/rust-cache` inside
  `actions-rust-lang/setup-rust-toolchain`, both pinned by commit, through
  `.github/actions/setup-rust` in every job that compiles; only `main`
  saves it, and rustfmt, cargo-deny and the publish dry run restore none.

## The local gate and the CI gate

`scripts/fast.sh` is the local gate: rustfmt, clippy on the reached
packages (wasm32 only when browser code changed), nextest and the doctests,
the journeys of the touched area against the gallery built with the fast
`wasm-dev` profile, and `design/tools/check.sh` for each reached design set.
It reads the same table as `detect`, from the merge base with `origin/main`
to the working tree. It never runs the screenshot capture; capture only the
pages a change touches with `FERROUI_SCREENSHOT_PAGES`. CI is the full
gate: the release bundle and its budget, every journey, the capture, and
every reached design set.

The browser steps hold one machine-wide lock (`scripts/chrome-lock.sh`, a
`mkdir` lock under `$TMPDIR` with a stale timeout), so parallel worktrees
queue for Chrome instead of crashing it. `design/tools/check.sh` and
`render.sh` take it themselves; wrap any other browser run:
`scripts/chrome-lock.sh env FERROUI_SCREENSHOTS=1 cargo nextest run …`.

A compile cache makes a fresh worktree's first build much faster. sccache
works (`build.rustc-wrapper` in your own `~/.cargo/config.toml`); the
repository neither needs nor configures one.
- `.github/workflows/pr-guards.yml`: `ci.yml` runs on `opened`, `synchronize`
  and `reopened` only, so a commit gets one run; when `no-changelog` or the
  description changes, this waits for the CI run on the head commit to
  complete and re-runs `guards` or `contribution-licence`, which re-runs
  `conclusion` with it.
- `.github/workflows/codeql.yml`: CodeQL over the `actions` language and Rust.
- `.github/workflows/scorecard.yml`: OpenSSF Scorecard, published to the
  OpenSSF API and to code scanning.
- `.github/workflows/release.yml`: dormant until a `v*` tag is pushed. It
  validates the tag against the workspace version, takes the notes from the
  matching `CHANGELOG.md` section (assembled from the fragments by the
  release pull request, `docs/release.md`), creates the release as a draft, calls
  `release-crate.yml` (a reusable workflow, so the signing identity stays out
  of reach of caller steps) to package the `.crate`, attest it and attach it,
  publishes the release, and then
  its `crates` leg uploads the crate through Trusted Publishing behind the
  `crates-io` environment (`crates-publishing.md`).
  Its `plan` job refuses a tag when no milestone is titled exactly as the tag
  or that milestone still holds an open issue or pull request, naming each
  one, and its last job, `milestone`, closes that milestone once every leg,
  crates included, has succeeded; a failed leg leaves it open and the job log
  names the leg. Both call `scripts/release/milestone.sh`.
- `.github/workflows/publish-crates.yml`: the manual crates.io lane, a dry
  run unless `publish` is set, sharing `scripts/release/publish-crates.sh`
  with the release lane.

## Workflow security (every workflow, no exceptions)

- **Every `uses:` is pinned to a full commit SHA** with a trailing `# vX.Y.Z`
  comment; Dependabot (`github-actions`) bumps them.
- **`permissions: {}` at workflow level**, with the minimum granted per job.
- **`persist-credentials: false`** on every `actions/checkout`.
- **No `${{ }}` context interpolation inside `run:`:** pass context through
  `env:`.
- **A publishing lane restores no build cache.**

Run `zizmor --min-severity=low .github/`, `actionlint` and `shellcheck
--severity=style` by hand before pushing a workflow or script change, and
the `--self-test` of every script you touch. A
finding is fixed, or carries a `# zizmor: ignore[audit]` or `# shellcheck
disable=SCnnnn` with its reason on the line it names; no `.shellcheckrc`
exists.

## Supply chain

- **A release is assembled as a draft and published last.** The fix for a bad
  cut is a new patch version, never a retag: immutable releases freeze a
  published release, and the `release-tags` ruleset stops a tag being moved
  or deleted.
- **Every tag is signed**, on the merge commit on `main`.
- **Always `--locked`**, and `Cargo.lock` is committed.
- **The crate publishes through Trusted Publishing** (OIDC, no long-lived
  token), with a dry run on every pull request.

## Never

- Never unpin a `uses:`, widen a job's permissions without cause, interpolate
  context into `run:`, or restore a cache in a publishing lane.
- Never weaken a gate to go green (`testing.md`); fix the cause.
- **Never add AI or Claude attribution** to any commit, PR, or release text.

## Official documentation (durable citations)

- OWASP GitHub Actions Security Cheat Sheet:
  <https://cheatsheetseries.owasp.org/cheatsheets/GitHub_Actions_Security_Cheat_Sheet.html>
- SLSA v1.0: <https://slsa.dev/spec/v1.0/>
- OpenSSF Scorecard: <https://github.com/ossf/scorecard-action>
- Trusted Publishing on crates.io: <https://crates.io/docs/trusted-publishing>
