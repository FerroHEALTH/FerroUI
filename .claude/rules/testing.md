<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Testing discipline

Test discipline is non-negotiable (a standing hard rule; see `CLAUDE.md`). It
applies to every crate.

## The hard rule

- **Never** silently weaken, skip, or delete an existing test to make a build
  pass.
- **Never** edit a test to route around a bug it exposes. If a test fails and
  the fix is unclear, leave it failing and record a `// TODO(#NNNN):` naming
  its issue.
- An accessibility test asserts the specification: cite the success
  criterion or the ARIA pattern it encodes, and never adjust an expectation to
  match a defect.

## Tooling

- **Runner:** `cargo-nextest` (`cargo nextest run --workspace`).
- **Rendering:** a view is tested by rendering it to HTML on the host with
  `RenderHtml::to_html` under an `Owner`
  (`crates/ferrohealth-ui/tests/it/render.rs`), asserting what it must say,
  what it must never carry (a `style` attribute, unescaped input) and its
  ARIA.
- **Contrast:** `tests/it/contrast.rs` computes every role pairing of every
  brand's three theme blocks (WCAG 2.2 SC 1.4.3 and SC 1.4.11), and the
  gallery's painted-colour journey measures what the browser draws.
- **Browser journeys:** thirtyfour over chromedriver, never Playwright (the
  no-JavaScript mandate), behind `FERROUI_JOURNEYS=1`, failing on any
  `SEVERE` browser log entry and waiting on elements, never on a timer.
- **Screenshots:** behind `FERROUI_SCREENSHOTS=1`, every page of every brand
  light and dark at 1440 px and full height, into `docs/screenshots/`; the
  ordinary suite's guard fails on a shot without both images. Every refreshed
  screenshot is looked at before the change merges.

## Coverage is a mandate, not just pass rate

- Every component has a rendering test, every widget's keyboard path a
  journey, every brand a contrast row.
- Every tokenizer reads its input back exactly (the texts of its tokens joined
  are the input), multi-byte text included.
- **Coverage only ratchets up.** Cases are added, never removed to go green.
- **One behaviour per case.**

## Where tests live

Unit tests live beside the code they test (`#[cfg(test)] mod tests`); a test
that drives the public API lives in the crate's one integration binary,
`tests/it/main.rs` plus one `mod` per topic. Dedicated test files under `src/`
are banned.

## Test shapes

- `Result`-returning tests where plumbing can fail; assertions carry a
  message where the values alone do not say what failed.
- Doctests are copy-paste templates, never `ignore`.

## No patient data, ever

Every row and text the gallery and the tests show is invented for them.
