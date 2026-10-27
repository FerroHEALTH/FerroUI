<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Building a console on FerroUI

FerroUI is the one source for the user interface of every FerroHEALTH
console. This page says what the library decides for you, what your console
decides for itself, which FerroUI document governs each part, and how to ask
for a change. Every console repository points its agents here.

No specification governs the split between the library and a console: our
own design (owner, 2026-10-22).

## Who this is for

You work in a console repository that builds on
[`ferrohealth-ui`](https://crates.io/crates/ferrohealth-ui): FerroBRIDGE,
FerroFED, FerroTERM, FerroEHR or FerroPIX, as a person or as an agent. Read
this page before you write or review UI code there.

## What the library owns

The library decides these, and a console takes them as they are:

- **The components:** every view function, ready component and widget the
  design names, with its markup, roles and keyboard paths. The ready
  components are
  listed in [`crates/ferrohealth-ui/CLAUDE.md`](../crates/ferrohealth-ui/CLAUDE.md).
- **The stylesheet and the tokens:** `ferrohealth_ui::style::STYLESHEET`
  with every `fui-` class and the family rules as CSS, the role layer, the
  type and space steps, and one theme file per brand.
- **The interaction contract and the mode:** native elements first,
  behaviour in Rust compiled to WebAssembly, the whole page hydrated and
  split by route so no interaction reloads the document, every page
  working without the WebAssembly as the fallback, and no hand-written
  JavaScript.
- **The words machinery and the library's words:** the Fluent catalogs a
  component draws from, the `*Words` structs each widget takes as props,
  `i18n::Locale` and its negotiation, and the stray-literal check.
- **The accessibility of each part:** WCAG 2.2 AA contrast for every role
  pairing in every brand, light and dark, the focus ring, the Authoring
  Practices patterns, and reflow at 320 px.
- **The bundle rules:** which dependencies may reach the browser, what an
  route's code loads, and the gallery's own budget.

## What the console owns

Your console decides these, in its own repository:

- its routes and pages, built from the library's templates and components;
- its server boundary: server functions, actions, the session and the
  backend client;
- its configuration and its security headers, the strict CSP included;
- its own bundle budget, measured over its own WebAssembly and kept in its
  own `leptos-ui.md`;
- its browser journeys and screenshots;
- the words of its own pages, in its own Fluent catalog.

A console may write a component of its own where a library widget shares state
with the console's code, such as an editor and the check that runs as you
type (`.claude/rules/leptos-ui.md` §1). It still draws with the library's
widgets and classes.

## How a console builds

A console hydrates the whole page and draws every page after the first in
the browser, so a link or a form never reloads the document (FerroUI #410;
`.claude/rules/leptos-ui.md` §15):

1. **No `islands` feature.** Turn on `hydrate` in the browser half and
   `ssr` in the server half, chosen by target, and never the `leptos`
   feature `islands`.
2. **Pages in a few lazy routes.** Group your pages into a few
   `leptos_router` `LazyRoute`s (`#[lazy_route]`), one for each part of your
   navigation read together, each drawing the page its path names, as the
   gallery's `app/ferrohealth-ui-gallery/src/routes.rs` does: one lazy route
   a page splits the code into hundreds of chunks, more than a browser
   fetches at once. Hydrate with
   `leptos::mount::hydrate_lazy(App)`, write `<HydrationScripts options/>`,
   and build with `cargo leptos build --split`. A build with lazy code and
   no `--split` imports `__wasm_split_placeholder__`, which no browser
   resolves, and nothing hydrates; check your glue for it, as the gallery's
   `scripts/release/gallery-site.sh` does.
3. **Prefetch.** Call `widget::prefetch::routes` once from your root
   component with every route's path and its `LazyRoute::preload`, marking
   your main navigation's routes `idle`.
4. **State the server read reaches the browser.** No component reads a
   cookie, the session or the request on the server alone: carry it with a
   `SharedValue` or a resource while the page hydrates, and read it again
   with a server function after a navigation (the gallery's
   `app/ferrohealth-ui-gallery/src/cookies.rs`).
5. **Actions are server functions.** A mutation is a server function behind
   `<ActionForm>`, which runs in the page and posts with no WebAssembly as
   the fallback; a link is a plain `<a>`, which the router follows. Your
   `no-referrer` policy withholds the `Referer` the fallback returns to, so
   draw `widget::action::return_field()` in the form, take an argument named
   `widget::action::RETURN`, and on a plain post call `leptos_axum::redirect`
   with `widget::action::plain_return`, or `with_error` when the function
   fails, so the page drawn again shows the error in the action's value (the
   gallery's `app/ferrohealth-ui-gallery/src/acknowledge.rs`).
6. **The page announced.** Draw `island::PageAnnouncer` once at your root,
   beside the toast region: after a navigation in the page it moves the
   focus to the new page's heading and says its title (WCAG 2.2 SC 2.4.3,
   SC 4.1.3).
7. **Forms in the page.** Draw `island::FormNavigator` once inside your
   `Router`: a `GET` form navigates in the page, and a `POST` form marked
   `widget::submit::NAVIGATES` whose route redirects is posted in the
   background and the router draws the page it names. Mark a form or a
   submit that must load a new document (an export, a download)
   `widget::submit::RELOADS`; with no WebAssembly every form submits as the
   browser does. A post your own server function draws in the page, at an
   address that does not change, calls `widget::submit::drawn()` once its
   answer is drawn, so what reads `widget::submit::posts()` reads again and
   the search box leaves its busy state.
8. **The locale both halves.** Provide the same `i18n::Locale` at your root
   on the server and in the browser (a `SharedValue` of its tag); the
   browser carries the library's catalog.
9. **Nothing changes the page before it hydrates.** Call
   `widget::hydration::mark()` once from your root component and
   `widget::hydration::explain()` first in your browser entry. The root
   element carries `data-ferroui-hydrated` once the page has hydrated; no
   code, script or test of yours adds, removes or rewrites a node the server
   drew before then, or the hydration walk stops on a node it did not expect
   with an internal error. In a debug build, `explain` says so beside such a
   panic. Code that must touch the DOM runs in an `Effect`, which runs after
   hydration; a test waits for the mark.
10. **The shell drawn once.** Draw your shell in one component inside your
    `Router` that calls `widget::nav::follow_path()` first, and draw it
    again only for what changes the whole shell (another product, another
    signed-in operator), never for the path: the navigation marks the page
    in view and the rail's and the theme choice's `back` name it by
    themselves, so the search's words, the navigation's scroll and every
    widget in the shell survive a navigation in the page. A part of the top
    bar that names the page in view (a version select) follows the router
    in a closure of its own.

## The top bar's search

The search box (`view::search::search_box`) is a form that submits to your
find page, and once the page has hydrated it is a typeahead: the panel
anchored under the field asks your answer route as the words are typed.
Your console provides three parts, and the library draws everything else.

1. **The answer:** one function that turns the words into a
   `view::search::SearchAnswer`: the words answered (`query`), the hits
   grouped by kind (`groups`, every hit, which the panel cuts to five of a
   kind), an `unfinished` sentence when one of your sources did not answer,
   and the optional `head`, `none` and `more`.
2. **The answer route:** name it in `SearchBox::suggest`. It answers the
   panel with that answer as JSON.
3. **The find page:** the form's `action`. It reads the same words and
   draws `view::search::find_results(answer)` under its page heading, every
   hit of each kind in a list card of its own, in the page. It is what a
   search shows with no WebAssembly, and where "See all results" leads.
   When the submit runs another search than the panel's, such as a
   patient search the panel never runs because each one is audited, name
   that option in `SearchBox::all` ("Search patients for “{query}”").

The wire of the answer route:

| | |
|---|---|
| Request, `SearchBox::post` off | `GET {suggest}?{name}={words}`, the words form-encoded, `Accept: application/json` |
| Request, `SearchBox::post` on | `POST {suggest}`, `Content-Type: application/x-www-form-urlencoded`, body `{name}={words}`, so a search that can carry clinical content keeps its words out of the URL |
| Credentials | Same origin: the page's own cookies go with it |
| Answer | `200` with `application/json`, the `SearchAnswer` serialized by serde, as `serde_json` writes it |
| `401` with `SearchBox::signed_out` set | The page loads that address, your signed-out page, as any other request of the page would on a session that is over |
| Any other status | The panel shows "The search did not answer (status)" as a row |
| A body that is no `SearchAnswer` | The panel shows that it could not read the answer |

The panel asks from `SearchBox::suggest_from` characters (two by default),
once the words rest for 200 ms, and drops an answer to words that have since
changed. Your Content-Security-Policy keeps `connect-src 'self'`, which
`default-src 'self'` gives. Check the session on the route as on any other,
and log the words no more than your find page does. The answer travels as
JSON:

```json
{
  "query": "vital",
  "head": null,
  "groups": [
    {
      "kind": "Templates",
      "hits": [
        {
          "href": "/templates/vital_signs.v1",
          "name": "vital_signs.v1",
          "line": "Vital signs encounter",
          "mark": { "Glyph": "file-code" },
          "tag": null
        }
      ]
    }
  ],
  "none": null,
  "unfinished": null,
  "more": null
}
```

A hit's `mark` is `{ "Glyph": "<lucide name>" }` or `{ "Dot": "Ok" }`
(a `tone::Tone`); every field but `query` may be left out.

## The session's end

A signed-in page leaves for your sign-in page the moment its session ends,
warns before an idle timeout, and ends with every other tab of your console
(FerroUI #444; WCAG 2.2 SC 2.2.1 and SC 2.2.5; the OWASP Session Management
Cheat Sheet). The library draws and times everything; your console provides
four parts.

1. **The watch:** draw `island::SessionWatch` once in the shell of a
   signed-in page, with `widget::session::Watch::new(status, sign_in,
   sign_out, left)`: your session route, your sign-in page, your sign-out
   route, and what the session had left as your server drew the page,
   carried to the browser in a `SharedValue` (How a console builds, step 4). Never draw it
   on the sign-in or signed-out page.
2. **The session route:** the wire below. Check the session on it as on any
   other route.
3. **The signed-out page:** draw `island::SessionEnded` on it, which tells
   every other tab that the session has ended.
4. **A `401` your own code meets:** call `widget::session::gone()`, as a
   server function whose session is gone does, and every tab leaves.

| | |
|---|---|
| Read | `GET {status}`, `Accept: application/json`, the page's own cookies. A read never extends the session: answer it without touching the idle timeout. |
| Extend | `POST {status}` with no body. It is the user's own activity (Stay signed in, or their input once a minute): start the idle timeout again, never past the absolute lifetime. |
| Answer | `200` with `application/json`: `{"idle": 840, "absolute": 27000}`, the whole seconds left until the idle timeout and until the absolute lifetime ends (`widget::session::Left`) |
| Gone | `401`: the page loads `{sign_in}?return={path and query}`, and every tab leaves |
| Any other status, or no answer | The page keeps its own count, and leaves once that has run out |

When it asks:

- when the warning is due, `Watch::warn_at` seconds before the idle
  timeout (120 by default, never fewer than 20), and when the session is
  due to end;
- when the tab is shown again or regains the focus, and when the browser
  restores the page from its back-forward cache;
- after the user's own key or pointer input, at most once a minute, to
  extend.

Your background polling (a health check, a status refresh) must not extend
the session on your server, or an open tab never idles out: only the
user's requests count as activity. Your sign-in page reads `return`, keeps
it to a path of your own, and leads back to it after sign-in. Keeping a
draft the user was typing through sign-in is yours, where your console has
drafts. With no WebAssembly nothing changes: your server redirects the next
request to sign-in, as before.

## Which document governs what

| Question | Document |
|---|---|
| The interaction contract, the mode, the ready components, the bundle measurements | [`crates/ferrohealth-ui/CLAUDE.md`](../crates/ferrohealth-ui/CLAUDE.md) |
| The component rules every console follows | [`.claude/rules/leptos-ui.md`](../.claude/rules/leptos-ui.md) |
| The design library, the order of a change and the 13 family rules | [`.claude/rules/console-design.md`](../.claude/rules/console-design.md) |
| How the design is rendered and checked | [`design/README.md`](../design/README.md) |
| Every component, by its ID (B2, R28, F9 and so on), and the templates T1 to T3 | [`design/COMPONENTS.md`](../design/COMPONENTS.md) |
| Your console's pages as designed | [`design/consoles/<console>/`](../design/consoles/) |
| Integration: the stylesheet, a theme, hydration and routes, locales, Your preferences | [`crates/ferrohealth-ui/README.md`](../crates/ferrohealth-ui/README.md) |

The sections of `.claude/rules/leptos-ui.md` a console reads most:

- §0 The mandates: Rust only, no inline style, no server function in the
  library, no patient identifier in a URL, no string outside the catalogs.
- §1 The interaction contract and the three parts (`view`, `island`,
  `widget`).
- §5 Forms and §6 Tables and paging.
- §7 Accessibility.
- §8 Tokens, themes, type.
- §12 Bundle, with the rule that a console keeps its own budget.
- §13 Strings and locales.
- §14 Leptos the book's way, with §14.5 and §14.6 written for a console's
  side.
- §15 Hydration and navigation: routes and their code, navigation in the
  page, state both halves read alike, words, server functions, and the
  no-WebAssembly floor.

The FerroUI repository's `design/` is the only design source. A design
folder anywhere else, in a download or in a console repository, is stale
for a console: a console never builds from it or syncs from it.

## Which version to read

Read FerroUI's documents at the git tag that matches your `ferrohealth-ui`
pin: with `=0.2.0`, read the tag `v0.2.0`; with `=0.4.0`, read `v0.4.0`.
Read `main` only when you are moving your console to the next release. The
rules on `main` describe components your pinned version may not have.

The design library, `console-design.md` and this page first appear in
0.3.0, so a console still on 0.2.0 has none of them at its tag. Start new UI
work after your console moves to 0.3.x.

## Moving to a new release

The consoles move together. FerroUI 0.3.0, 0.3.1 and 0.3.2 rebuild the
library on the reviewed design, and no console moves before 0.3.2 ships.
Then the rollout, [FerroUI #161](https://github.com/FerroHEALTH/FerroUI/issues/161)
in milestone v0.4.0, moves every console in one go. For each console it:

1. bumps the pin and takes the stylesheet and the theme file from the
   release;
2. applies every breaking change named under Changed and Removed in the
   changelogs, and deletes the console's local markup for those parts;
3. refreshes and looks at the console's screenshots and journeys.

Each console's own migration issue is filed in its own repository when the
rollout starts.

## Asking for a component

A missing component, a wrong behaviour or a wrong look is an issue in
[FerroHEALTH/FerroUI](https://github.com/FerroHEALTH/FerroUI/issues), linked
from the console's own issue. The fix lands in FerroUI in the order
`console-design.md` sets (the design, the stylesheet, then the Rust), and the
console takes it with the next release.

Never build the part in the console repository: no local copy of a
component, no shadow component, no stylesheet override and no hand-written
script. A local stand-in drifts from the design, misses the library's
accessibility and contrast tests, and has to be deleted again at the next
move.

## Before you write UI code

1. Find your `ferrohealth-ui` pin, and read the documents above at its tag.
2. Find the page in `design/consoles/<console>/` and the IDs of every
   component it draws in `design/COMPONENTS.md`.
3. Check that the library at your pin has each component. If one is missing
   or wrong, file the FerroUI issue and stop that part.
4. Keep your own work to the console's side: routes, server boundary,
   backend client, configuration and headers, budget, journeys and words.
5. Check the page without WebAssembly: every important action is a link or
   a form.
6. Put every word in your console's catalog and run the stray-literal
   check.
7. Measure your bundle against your own budget.
