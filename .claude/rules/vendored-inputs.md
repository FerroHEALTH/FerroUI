<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Vendored inputs: fetched by a committed script, verbatim, with provenance

FerroUI vendors one external corpus: the Lucide icon outlines it draws, under
`crates/ferrohealth-ui/vendor/lucide/`.

## Rules

- **Every vendored file is fetched by a committed script**
  (`scripts/vendor/lucide.sh`) from a pinned release, never copied by hand.
  The script's `--check` mode fails when a listed icon is missing or an
  unlisted one is present.
- **Vendored files are verbatim**: never edited. The glyph table of
  `src/icon.rs` carries each outline's inner markup, and its test holds every
  entry to its vendored file, so a glyph never drifts from its source.
- **Every vendored tree carries a `PROVENANCE.md`**: the source, the pinned
  release, the date fetched, the licence, and how to refresh it; and the
  upstream licence file beside it (`LICENSE`, ISC for Lucide).
- **A new icon** is a name added to the script's list and an entry added to
  the glyph table in the same change, then a run of the script.
- **Bumping the pin** is a deliberate change: rerun the script, review every
  changed outline in the gallery's icon card, and record the new release in
  `PROVENANCE.md`.
