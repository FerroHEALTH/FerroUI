---
name: reactive-console-ui
description: The consoles are server-rendered HTML with a little plain JavaScript (owner, 2026-10-29, FerroUI #510), reversing the hydrated-WebAssembly ruling of 2026-10-25; an in-page change is a few lines of script, a navigation one fast document
metadata:
  type: feedback
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

The owner ruled on 2026-10-29 (FerroUI #510) that the FerroHEALTH consoles
move to the VernumBOEK model: server-rendered HTML from Askama templates, a
few plain ES modules, no WebAssembly and no hydration. This reverses
the ruling of 2026-10-25 (FerroUI #410), which made hydrated WebAssembly the
normal path.

**Why:** the hydrated consoles flickered on every click and lost what the
reader had done. A tab or a tree node was a navigation the router redrew
whole, with an empty fallback while it asked the server again, so expansion,
scroll and selection were gone. FerroUI #196 had earlier moved to islands
and made every action a full page load, which was slow too.

**How to apply:** the bar is that nothing flickers and nothing is lost.
- An in-page change (a tab whose panel the page holds, a tree level the
  server read, a menu) is a few lines of script that toggle what the server
  drew, with no round trip.
- A navigation is one fast document the browser paints whole, keeping
  scroll and expansion (`keep`, `tree`).
- Every page works with no script.
- Judge a UI change by the interaction it gives, and hold it with the
  gallery's navigation journeys.
