<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Lucide icon outlines

- **Source:** the Lucide icon set, <https://github.com/lucide-icons/lucide>,
  each file the release's own `icons/<name>.svg`, and the release's
  `LICENSE`.
- **Release:** `1.51.0` (published 2026-10-03).
- **Fetched:** 2026-10-18, by `scripts/vendor/lucide.sh`; `equal-not`, `eye-off` and
  `minus` on 2026-10-22, and `building-complex`, `flame`, `message-square`,
  `plug` and `table-2` on 2026-10-23, and `corner-down-right`,
  `git-branch`, `list-filter` and `list-tree` later that day, and
  `arrow-left-right`, `book-open`, `git-compare`, `send` and
  `user-round-search` on 2026-10-23 too, from the same release. The
  release names `building-2` a deprecated alias of `building-complex`,
  `trash-2` one of `trash` and `history` one of `rotate-ccw-clock`, so
  each file is vendored under its current name.
- **Licence:** ISC License, Copyright (c) 2026 Lucide Icons and Contributors,
  in `LICENSE` beside this file. The outlines are redistributed unchanged.
- **Use:** `crates/ferrohealth-ui/src/icon.rs` carries each outline's inner
  markup as a `Glyph`; its test holds every glyph to its file here.
- **Refresh:** change the pin in `scripts/vendor/lucide.sh`, run it, review
  every changed outline in the gallery's icon card, and record the release
  here. `scripts/vendor/lucide.sh --check` fails when a listed icon is missing
  or an unlisted one is present.
