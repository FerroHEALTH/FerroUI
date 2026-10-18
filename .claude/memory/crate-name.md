---
name: crate-name
description: "The library crate is ferrohealth-ui (Rust path ferrohealth_ui) and the gallery ferrohealth-ui-gallery; ferroui on crates.io belongs to an unrelated project"
metadata:
  type: project
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

The crate is `ferrohealth-ui`, the gallery `ferrohealth-ui-gallery`
(`publish = false`). The repository is `FerroHEALTH/FerroUI`, and FerroUI is
the name in prose.

**Why:** `ferroui` on crates.io is taken by an unrelated project (owner,
2026-10-18). The owner reserved `ferrohealth-ui` at `0.0.0` so Trusted
Publishing could take over from `0.1.0`.

**How to apply:** never name a crate, a package or a feature `ferroui`, and
never run `cargo publish` against that name. Consoles depend on
`ferrohealth-ui = "=X.Y.Z"`.
