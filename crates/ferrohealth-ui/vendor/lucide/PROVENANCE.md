<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Lucide icon outlines

- **Source:** the Lucide icon set, <https://github.com/lucide-icons/lucide>,
  each file the release's own `icons/<name>.svg`, and the release's
  `LICENSE`.
- **Release:** `1.51.0` (published 2026-10-03).
- **Fetched:** 2026-10-18, by `scripts/vendor/lucide.sh`; `equal-not`, `eye-off` and
  `minus` on 2026-10-22 from the same release.
- **Licence:** ISC License, Copyright (c) 2026 Lucide Icons and Contributors,
  in `LICENSE` beside this file. The outlines are redistributed unchanged.
- **Use:** `crates/ferrohealth-ui/src/icon.rs` carries each outline's inner
  markup as a `Glyph`; its test holds every glyph to its file here.
- **Refresh:** change the pin in `scripts/vendor/lucide.sh`, run it, review
  every changed outline in the gallery's icon card, and record the release
  here. `scripts/vendor/lucide.sh --check` fails when a listed icon is missing
  or an unlisted one is present.
