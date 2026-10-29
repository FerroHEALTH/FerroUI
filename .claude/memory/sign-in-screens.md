---
name: sign-in-screens
description: "The sign-in card has a basic username and password form, an OAuth2 provider button, or both; FerroEHR's sign-in card is the reference design"
metadata:
  type: feedback
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

`view::sign_in` draws one card in three shapes: the basic form alone, the
provider button alone (an OAuth2 or OpenID Connect sign-in), and both with an
"or" divider. The gallery shows all three for every brand.

**Why:** the owner, 2026-10-18: two sign-in screens are needed, a basic login
form and a login button alone for OAuth2, and FerroEHR's existing sign-in is
the design to start from; the screens can always be improved.

**How to apply:** a console picks the shape from its configuration; the form
stays a plain `POST` with uncontrolled inputs and the standard autocomplete
tokens.

**Like every form** ([[reactive-console-ui]], owner 2026-10-29), sign-in
is a plain `POST` that loads a new document; it also starts an OAuth
redirect, which is a full navigation by nature.
