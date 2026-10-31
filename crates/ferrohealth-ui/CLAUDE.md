<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# CLAUDE.md: `ferrohealth-ui`

The crate-local discipline of the library. The component rules are
`.claude/rules/console-ui.md`; this file holds the interaction contract and
the rendering mode every component is built to, decided on FerroUI #510.

## The interaction contract

Every component follows it (the owner's rulings of 2026-10-29 on #510):

1. **Native elements first:** `<dialog>` with `command` and `commandfor`,
   the `popover` attribute, `<details>`, plain links, and GET and POST
   forms.
2. **Every page is complete with no script.** The server draws the whole
   page with the state it read, and a link or a form is behind every
   action. The gallery's `tests/it/no_script.rs` loads every page with the
   modules refused.
3. **Behaviour in small plain modules** under `scripts/`, one per job, each
   `enhance(element)` over what the server drew, loaded by `fui.js` for an
   element marked `data-fui="<job>"`. An in-page change toggles what the
   server drew, with no round trip.
4. **A navigation is one fast document.** A link or a form loads its page
   whole; the `keep` module keeps the scroll across a same-path navigation
   and the `tree` module a tree's folds. No WebAssembly, no npm, no bundler:
   the CSP is `script-src 'self'; style-src 'self'`.

## The mode: server-rendered HTML on Askama

NOTE: FerroUI #510; the owner's rulings of 2026-10-29. The library renders
HTML on the server from Askama templates compiled into the crate (Askama
0.16, with no `config` feature: the templates sit in the default
`templates/` with the default whitespace). A component is a function or a
struct over a template that returns [`html::Html`](src/html.rs), a string of
markup that implements `askama::filters::HtmlSafe`, so a console's own
template writes it with `{{ part }}` and never escapes it twice. A console's
handler builds its page from its own templates and the library's parts,
then `view::shell::document` writes the whole document: the stylesheet, the
brand's theme, the loader and the console's own head parts.

The hydrated mode of #410 made every tab and tree node a navigation the
router redrew whole, with an empty fallback while it asked the server
again, so expansion, scroll and selection were lost. Here an in-page change
is a few lines of script, and a navigation is one document the browser
paints whole, with no WebAssembly to load before a click works.

## How the crate is laid out for it

- `view`: the components, each over a template under
  `templates/<area>/`. A component takes `&Locale` where it draws words and
  its children as `Html`.
- `scripts/`: the modules, vendored in the crate as text
  ([`script::MODULES`](src/script.rs)), each within the budget
  `scripts/checks/script-budget.sh` holds (16 KiB, 144 KiB for what one
  page loads, 72 KiB compressed for all).
- `serve`: under the default `axum` feature, the router a console mounts at
  `serve::PREFIX` for the stylesheet, the themes and the modules, with long
  cache headers; `serve::Assets` the paths the shell writes. A console with
  no axum serves the same bytes from `style::STYLESHEET`,
  `style::Brand::theme` and `script::MODULES`.
- `i18n`: the locale, its negotiation, the Fluent catalogs, `Word` and
  `Phrase`. A module's words ride in data attributes or a `<template>`,
  resolved on the server; no catalog reaches the browser.

Each word is an `i18n::Word`, which carries whether it fell back to English
and draws itself inside `<span lang="en">` when it did (WCAG 2.2 SC 3.1.2),
and a component whose module writes a count carries the page's
`i18n::number::Grouping` among its words.

## The session's end (FerroUI #444)

No specification governs the timers and the broadcast: our own design, held
to WCAG 2.2 (SC 2.2.1, SC 2.2.5) and the OWASP Session Management Cheat
Sheet (idle and absolute timeouts, logout across tabs). The decisions:

- **What the browser knows:** the seconds the session has left, idle and
  absolute (`view::session::Left`), never a deadline, so a wrong browser
  clock moves nothing, and never a token. The server writes them on the
  watch (`view::session::session_watch`) and answers its session route with
  the same; the wire is in `docs/consoles.md` § The session's end.
- **Leaving:** when the time is up the `session` module asks the route; a
  `401`, or no answer once its own count has run out, loads the console's
  sign-in page with the page's path and query as `return`.
- **The warning:** `WARN_AT` seconds (120) before the idle timeout, never
  fewer than 20, a modal `<dialog role="alertdialog">` names the time left
  and offers Stay signed in, which takes the focus, and Sign out; Escape
  stays signed in, and the browser returns the focus on close. An absolute
  lifetime that ends first gets no warning, since nothing can extend it.
- **Re-checking:** `visibilitychange`, `focus` and `pageshow` ask the route,
  which covers a laptop that slept while its timers stood still.
- **Idle means the user is idle:** a `GET` of the route never extends the
  session; a `POST` does, sent for Stay signed in and for the user's own
  key or pointer input at most once a minute.
- **Every tab:** a `BroadcastChannel` named `ferroui-session`, bound to the
  console's origin, carries `ended` and `extended <idle> <absolute>`. A
  signed-out page carries `view::session::session_ended`, which tells every
  tab the session is over.
- **Nothing lost:** the return address brings the user back to the page;
  keeping a draft through sign-in is the console's, where it has drafts.
