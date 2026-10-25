---
name: brand-palettes
description: "Each product keeps its own accent: BRIDGE indigo, FED azure, EHR its viewer teal, TERM teal, PIX plum; a shade moves only when it fails WCAG 2.2 AA, and the change says so"
metadata:
  type: project
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

Owner decision, 2026-10-18: FerroEHR keeps its viewer palette (teal), not a
new one; FerroBRIDGE keeps indigo, FerroFED its azure accent, FerroTERM its
teal. FerroPIX, the fifth console, takes the plum of its own brand tokens
(`themes/ferropix.css`). The default theme with no brand is the FerroHEALTH
steel.

Two shades moved to pass the contrast test, and the theme files say why:
FerroEHR's light accent is teal 700 (`#0f766e`) because the viewer's teal 600
fails 4.5:1 as link text, and its dark tint is a translucent teal because the
viewer's teal 900 ground fails the muted text.

**How to apply:** a theme file sets the five accent properties in three blocks
and nothing else. A palette change that fails `tests/it/contrast.rs` moves the
smallest shade that passes and names the move in the pull request.
