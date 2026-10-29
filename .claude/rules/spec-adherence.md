<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Spec adherence (the published specifications are the oracle)

FerroUI's components answer to these sources, never to memory and never to
how another library behaves:

1. **WCAG 2.2** (<https://www.w3.org/TR/WCAG22/>), Level AA: contrast (SC
   1.4.3, 1.4.11), use of colour (1.4.1), reflow (1.4.10), keyboard (2.1.1),
   focus order and visibility (2.4.3, 2.4.7, 2.4.11), names and roles (4.1.2).
2. **WAI-ARIA 1.2** (<https://www.w3.org/TR/wai-aria-1.2/>) and the
   **WAI-ARIA Authoring Practices** (<https://www.w3.org/WAI/ARIA/apg/>):
   the tabs pattern, the dialog pattern, `aria-sort`, live regions.
3. **The HTML Living Standard** (<https://html.spec.whatwg.org/>): the
   `popover` attribute, `<details>`, forms and `autocomplete`, the escaping of
   text and attribute values.
4. **CSS** (<https://www.w3.org/Style/CSS/>): cascade layers, custom
   properties, `prefers-color-scheme`, `prefers-reduced-motion`.
5. **The Askama book** (<https://askama.readthedocs.io/>, Askama 0.16) and
   the docs.rs pages of the pinned crates, for every template question, and
   MDN for a browser API a script module calls.

## Hard rules

- **Read the governing section before implementing or reviewing** a
  component's role, name, keyboard path, focus handling or contrast, and cite
  it (document and section) in the code or the pull request.
- **Cite only durable references**: the five above or official external
  documentation. Never cite an internal markdown file as a design authority.
  Where no specification governs a decision (a class name, a layout), flag it:
  "no specification governs this: our own design".
- **The four FerroHEALTH consoles are prior art**, never the oracle. Where a
  console's component and a specification disagree, the specification wins.
- **Never claim a property no test or citation shows.** A component is
  accessible by evidence: a rendering test, a journey, a contrast row.
