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

- **The components:** every view function, island and widget the design
  names, with its markup, roles and keyboard paths. The ready islands are
  listed in [`crates/ferrohealth-ui/CLAUDE.md`](../crates/ferrohealth-ui/CLAUDE.md).
- **The stylesheet and the tokens:** `ferrohealth_ui::style::STYLESHEET`
  with every `fui-` class and the family rules as CSS, the role layer, the
  type and space steps, and one theme file per brand.
- **The interaction contract and islands mode:** native elements first,
  behaviour in Rust compiled to WebAssembly in islands, every page working
  without the WebAssembly, and no hand-written JavaScript.
- **The words machinery and the library's words:** the Fluent catalogs a
  component draws from, the `*Words` structs each widget takes as props,
  `i18n::Locale` and its negotiation, and the stray-literal check.
- **The accessibility of each part:** WCAG 2.2 AA contrast for every role
  pairing in every brand, light and dark, the focus ring, the Authoring
  Practices patterns, and reflow at 320 px.
- **The bundle rules:** which dependencies may reach the browser, what an
  island may take as props, and the gallery's own budget.

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

A console may write an island of its own where a library widget shares state
with the console's code, such as an editor and the check that runs as you
type (`.claude/rules/leptos-ui.md` §1). It still draws with the library's
widgets and classes.

## How a console builds

Build the site with `cargo leptos build --split` and turn on the library's
`split` feature (`ferrohealth-ui = { version = "=0.3.7", features =
["split"] }`): the date pickers and the tree then load from chunks of their
own, out of the WebAssembly every page loads. A console that builds
without `--split` leaves the feature off, and those islands hydrate from
its one WebAssembly. Never turn the feature on without `--split`, and never
write a lazy island of your own without it: the glue then imports
`__wasm_split_placeholder__`, which no browser resolves, and no island on
any page hydrates. A build can check its glue for that import, as the
gallery's `scripts/release/gallery-site.sh` does.

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

The wire of the answer route:

| | |
|---|---|
| Request, `SearchBox::post` off | `GET {suggest}?{name}={words}`, the words form-encoded, `Accept: application/json` |
| Request, `SearchBox::post` on | `POST {suggest}`, `Content-Type: application/x-www-form-urlencoded`, body `{name}={words}`, so a search that can carry clinical content keeps its words out of the URL |
| Credentials | Same origin: the page's own cookies go with it |
| Answer | `200` with `application/json`, the `SearchAnswer` serialized by serde, as `serde_json` writes it |
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

## Which document governs what

| Question | Document |
|---|---|
| The interaction contract, islands mode, the ready islands, the bundle measurements | [`crates/ferrohealth-ui/CLAUDE.md`](../crates/ferrohealth-ui/CLAUDE.md) |
| The component rules every console follows | [`.claude/rules/leptos-ui.md`](../.claude/rules/leptos-ui.md) |
| The design library, the order of a change and the 13 family rules | [`.claude/rules/console-design.md`](../.claude/rules/console-design.md) |
| How the design is rendered and checked | [`design/README.md`](../design/README.md) |
| Every component, by its ID (B2, R28, F9 and so on), and the templates T1 to T3 | [`design/COMPONENTS.md`](../design/COMPONENTS.md) |
| Your console's pages as designed | [`design/consoles/<console>/`](../design/consoles/) |
| Integration: the stylesheet, a theme, islands, locales, Your preferences | [`crates/ferrohealth-ui/README.md`](../crates/ferrohealth-ui/README.md) |

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
- §15 Islands: when something is an island, its props, its children,
  context, navigation, server functions from islands, and the
  no-WebAssembly floor.

The FerroUI repository's `design/` is the only design source. A design
folder anywhere else, in a download or in a console repository, is stale
for a console: a console never builds from it or syncs from it.

## Which version to read

Read FerroUI's documents at the git tag that matches your `ferrohealth-ui`
pin: with `=0.2.0`, read the tag `v0.2.0`; with `=0.3.7`, read `v0.3.7`.
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
