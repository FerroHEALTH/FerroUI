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

- **The components:** every view function the design names, with its
  markup, roles and keyboard paths, each over an Askama template compiled
  into the crate and returning `html::Html`.
- **The script modules:** one plain ES module per behaviour under
  `scripts/`, the loader `fui.js` that imports them for each element marked
  `data-fui="<job>"`, and the asset router that serves them
  (`serve::axum_router::router()` at `serve::PREFIX`).
- **The stylesheet and the tokens:** `ferrohealth_ui::style::STYLESHEET`
  with every `fui-` class and the family rules as CSS, the role layer, the
  type and space steps, and one theme file per brand.
- **The interaction contract and the mode:** native elements first,
  every page complete with no script, behaviour in small modules over
  what the server drew, and a navigation as one document
  ([`crates/ferrohealth-ui/CLAUDE.md`](../crates/ferrohealth-ui/CLAUDE.md)).
- **The words machinery and the library's words:** the Fluent catalogs a
  component draws from, `i18n::Locale` and its negotiation, and the
  stray-literal check.
- **The accessibility of each part:** WCAG 2.2 AA contrast for every role
  pairing in every brand, light and dark, the focus ring, the Authoring
  Practices patterns, and reflow at 320 px.
- **The script budget:** 16 KiB a module, 144 KiB for the modules one
  page loads, and 72 KiB for all of them compressed
  (`scripts/checks/script-budget.sh`).

## What the console owns

Your console decides these, in its own repository:

- its routes, its handlers and its pages, built from the library's
  templates and components in its own Askama templates;
- its post routes, its session and its backend client;
- its configuration and its security headers, the strict CSP included;
- where a flash toast travels across a redirect (a query parameter or a
  short-lived cookie of its own);
- its browser journeys and screenshots;
- the words of its own pages, in its own Fluent catalog.

A console writes no script and no component of its own. A behaviour the
library lacks is a FerroUI issue (§ Asking for a component).

## How a console builds

Every page is one document your server draws whole (FerroUI #510;
`.claude/rules/console-ui.md` §1). Why, and what would reopen that
decision, is [`docs/rendering-model.md`](rendering-model.md). Moving from
0.5 is [`docs/migration-0.6.md`](migration-0.6.md).

1. **The assets.** Mount `serve::axum_router::router()` at `serve::PREFIX`.
   It serves the stylesheet, the brand themes and the modules under
   `/ferrohealth-ui/<version>/` with a one-year immutable cache, so a new pin
   is a new path. Without axum, serve `style::STYLESHEET`,
   `style::Brand::theme` and `script::MODULES` at the paths
   `serve::Assets` names.
2. **The document.** Every handler returns `view::shell::document`, which
   writes `<html lang>`, the preference attributes (`data-theme`,
   `data-density`, `data-motion`, `data-hints`), the stylesheet, the theme
   and the loader. A signed-in page puts `view::shell::shell` in its body,
   with the top bar, the sidebar, the page and the toast region; a sign-in
   or error page puts `view::shell::centred` there.
3. **The request's state.** Read the locale (`Locale::negotiate` over
   `Accept-Language`, or your configuration) and the preferences
   (`theme::Preferences::read` over the `Cookie` header) once per request,
   and pass the `&Locale` to every component.
4. **The page's state is in the HTML.** Draw what the request says: the
   open tree levels, the selected tab, the listing's page and sort. A
   module reads what the server drew and keeps nothing a reload would lose
   that the server could have drawn.
5. **Actions are posts.** A mutation is a `<form method="post">` to a route
   of yours. A route whose outcome is the next page redirects with
   `303 See Other` and names the outcome, and the next page draws it as a
   `view::toast::Flash` in `view::toast::toast_region`. A danger
   confirmation with fields is `view::dialog::danger_form_dialog(locale,
   confirmation, ConfirmForm { action, fields }, confirm)`, a `<dialog>`
   whose form posts to `action`.
6. **The policy.** `default-src 'self'; script-src 'self'; style-src
   'self'`. The loader takes a nonce (`Document::nonce`) only when your own
   policy needs one.

## An action that answers on its page

A check, a run or a load report is worth only its answer, and a redirect
draws the page fresh, without the answer and without what the user typed.
Such a form posts to a route that draws the page itself, with the answer and
the posted fields, and answers `200` (FerroUI #485, kept as a plain post in
0.6). No specification governs this: our own design.

- Write each posted field back into the form, escaped by the template, so
  the user sees what they sent beside the answer.
- Never write back a secret field (a password, a key,
  `view::form::masked_input`, `view::secret::secret_field`): draw it empty.
- Draw the answer in a live region (`aria-live="polite"`) or as the page's
  first heading after the form, so a screen reader hears it.
- A reload asks to post again, as for any form that answers its own post.

## Your own preferences

Your own rows go inside the three tabs of Your preferences
(`PreferencesPage::own`), each a `view::preferences::preference_row` around
one control, its `control_id` the control's name. The control is a
`select_preference`, a `switch_preference` or, for a value the person
types, a `typed_preference` (FerroUI #527):

```rust
typed_preference(&locale, TypedPreference {
    name: String::from("page-size"),
    kind: Typed::Number { min: Some(10), max: Some(200), step: None },
    value: String::from("25"),
    placeholder: None,
    post: post.clone(),
    saved: saved == Some("page-size"),
})
```

`Typed::Text { min_length, max_length }` draws a text field, such as a
BCP 47 tag. Each is the same plain form as a select: with no script its
Save button posts it, and once the `preference` module runs it posts when
the field changes (on Enter, or when the focus leaves it changed), and the
row you mark `saved` gives the field the focus back. The browser holds the
value to its bounds before it posts; your route checks it again and keeps
it in its cookie.

## The top bar's search

The search box (`view::search::search_box`) is a form that submits to your
find page, and with its `search` module it is a typeahead: the panel
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
   search shows with no script, and where "See all results" leads.
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
| `401` with `SearchBox::signed_out` set | The page loads that address, your signed-out page, as any other request of the page would on a session that is over, and every other tab of your console leaves for sign-in |
| `401` with no `SearchBox::signed_out` | The panel shows "The search did not answer (401)" as a row, and every tab with a session watch, this one included, leaves for sign-in |
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

## Keyboard shortcuts

You write no key listener. Every shortcut is a control already on the
screen that announces its keys in `aria-keyshortcuts` (WAI-ARIA 1.2), and
the `keys` module presses it (FerroUI #526, #530). It runs from 40 rem up
(family rule 6), never on the legend's `?` or the search box's own key.

- **A chord** (`Ctrl Enter`, `Ctrl s`; Command on a Mac) presses its
  control from inside a field too, and leaves the focus there. Set
  `view::transform::Run::shortcut` for Run and `view::dock::Decision::shortcut`
  for a docked decision; when two controls announce the same chord, the
  one in the focused field's form wins.
- **A single key** (`l`, `n`, `]`) presses its control, or moves the focus
  to it when it is a field (`view::dock::DockField::shortcut`, `r`), and
  only outside a field. A control that opens a dialog takes the focus
  first, so the dialog returns it there.
- **A row's key** reaches a control inside a work-list row only while that
  row holds the focus. Draw the row's side panel with
  `view::side_panel::side_panel_with_shortcut(panel, "o")`: `o` opens the
  focused row's panel and the browser returns the focus to the row when
  it closes. List the key in the work-list's hint and in the legend.
- **Go-to pairs** are a first key, then a letter, then a link of your
  navigation. Draw the legend with
  `view::shortcuts::shortcut_legend_with_go_to(locale, groups, GoTo { title,
  lead: "g".into(), pairs })`, each `GoToPair { key, href, does }`; the
  legend lists each pair as its link and the module follows it.
  `GoTo::group` gives the same pairs as a `ShortcutGroup` for your own
  shortcuts page.

The library's controls that announce a key load the module themselves, and
so does the legend. Your own control (a page header's Previous and Next, a
button that opens a dialog) writes `aria-keyshortcuts` through
`view::kbd::aria_keys` and, on a page with no legend, `data-fui="keys"`
(`view::shortcuts::KEYS_JOB`). With the modules refused nothing binds, and
every control still works by the pointer and Tab.

## The session's end

A signed-in page leaves for your sign-in page the moment its session ends,
warns before an idle timeout, and ends with every other tab of your console
(FerroUI #444; WCAG 2.2 SC 2.2.1 and SC 2.2.5; the OWASP Session Management
Cheat Sheet). The library draws the warning and the `session` module times
everything; your console provides four parts.

1. **The watch:** draw `view::session::session_watch(&watch)` once in the
   shell of a signed-in page, with `view::session::Watch::new(locale,
   status, sign_in, sign_out, left)`: your session route, your sign-in
   page, your sign-out route, and what the session had left as your
   handler drew the page. Never draw it on the sign-in or signed-out page.
2. **The session route:** the wire below. Check the session on it as on any
   other route.
3. **The signed-out page:** draw `view::session::session_ended()` on it,
   which tells every other tab that the session has ended.
4. **A `401` your own handler meets:** redirect to your sign-in page with
   `view::session::sign_in_address(sign_in, here)`, so the user comes back
   to the page.

| | |
|---|---|
| Read | `GET {status}`, `Accept: application/json`, the page's own cookies. A read never extends the session: answer it without touching the idle timeout. |
| Extend | `POST {status}` with no body. It is the user's own activity (Stay signed in, or their input once a minute): start the idle timeout again, never past the absolute lifetime. |
| Answer | `200` with `application/json`: `{"idle": 840, "absolute": 27000}`, the whole seconds left until the idle timeout and until the absolute lifetime ends (`view::session::Left`) |
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
drafts. With no script nothing changes: your server redirects the next
request to sign-in.

## The logic written twice

Some rules run on your server and again in the browser: the server draws a
page with the Rust copy, and a script module changes it in place with its
own copy. Since 0.6 the two copies are separate code, so FerroUI holds each
pair to one answer with a parity test (#548): the test feeds the same inputs
to both copies and fails on any difference. The Rust copy runs in the test,
and the module's copy runs in the gallery's browser, imported from the page
the way the loader imports it. The parity tests are journeys in
`app/ferrohealth-ui-gallery/tests/it/parity*.rs`, and each test's name
carries the words of both copies' files, so a change to either copy runs it
in `scripts/fast.sh`. Where the two differed, the module now answers as the
Rust copy does, because your server says the same thing when it refuses a
posted value.

| Rule | Rust copy | Module copy | Parity test |
|---|---|---|---|
| The YAML, JSON, XML, AQL and HL7 v2 tokens, the markup of each line, and the size past which a text is plain | `highlight::tokens`, `lines`, `line_html`, `MAX_HIGHLIGHT_BYTES` | `highlight.js`: `tokens`, `linesHtml`, `MAX_BYTES` | `parity::the_highlight_*`, `parity::a_highlight_*` |
| The caret's line and column, the offset of a line and column, the grammar of a text, the line count, a line's mark, and the empty editor | `view::editor::caret_at`, `offset_of`, `mark_kind`, `MarkKind::class`, `editor_panel`; `highlight::Language::detect`; `view::code::line_count` | `editor.js`: `caretAt`, `offsetOf`, `detect`, `lineCount`, `kinds`, `lineClass`, `blank` | `parity::the_editor_*` |
| A day as a number, its name and weekday, the month lengths, a month's steps and weeks, a typed day and time, the bounds | `calendar::Day`, `Month`, `Day::parse_in`, `Moment::parse_in`, `Bounds`, `leap`, `days_in` | `calendar.js`: `toNumber`, `fromNumber`, `iso`, `weekday`, `leap`, `daysIn`, `stepMonth`, `weeks`, `parseDay`, `parseMoment`, `allows`, `reaches` | `parity_dates::the_calendar_*` |
| Why a typed date is refused, a month's title and a day's name | `view::date::DateWords::reason`, `month_title`, `day_name` | `calendar.js`: `reason`, `monthTitle`, `dayName` | `parity_dates::the_date_field_*`, `parity_dates::the_date_picker_*` |
| A range's summary and status line | `view::date::range_summary`, `range_status` | `date_range.js`: `summary`, `status` | `parity_dates::the_date_range_*` |
| The tree's items in sight and the move each key makes | `view::tree::in_sight`, `moved` | `tree.js`: `inSight`, `moved` | `parity_keys::the_tree_*` |
| The tabs' arrows and digits | `view::tabs::moved`, `view::shortcuts::tab_at` | `tabs.js`: `moved`, `tabAt` | `parity_keys::the_tabs_*` |
| The menu's and the listbox's arrows, a typed letter and the flip | `view::action_menu::moved`, `typed`, `flips`; `view::menu::step` | `menu.js`: `moved`, `typed`, `flips` (`user_menu.js` takes `moved`) | `parity_keys::the_menu_*` |
| The work-list's keys and selection | `view::worklist::moved`, `selection` | `worklist.js`: `moved`, `selection` | `parity_keys::the_worklist_*` |
| The topology's zoom steps, keys and List moves | `view::topology::STEPS`, `stepped`, `asked`, `moved` | `topology.js`: `STEPS`, `stepped`, `asked`, `moved` | `parity_keys::the_topology_*` |
| The keys that hold the masked input's eye | `view::masked::holds` | `masked.js`: `holds` | `parity_keys::the_masked_*` |
| The typeahead's words, matches, steps, Escape, panel side and options, and a hit's markup | `view::search::wants`, `runs`, `step`, `escaped`, `placed`, `targets`, `hit_body` | `search.js`: `wants`, `runs`, `step`, `escaped`, `placed`, `targets`, `hitBody` | `parity_words::the_search_*` |
| The session's plan, its messages between tabs and the sign-in address | `view::session::plan`, `Left::after`, `Message`, `sign_in_address` | `session.js`: `plan`, `after`, `messageText`, `parseMessage`, `signInAddress` | `parity_words::the_session_*` |
| A file's size in words, its refusal and its decoding | `view::upload::SizeWords::size`, `admit`, `Refused::sentence`, `decode` | `upload.js`: `size`, `refusal`, `decode` | `parity_words::the_upload_*` |
| A phrase's open placeables and a count's digit groups | `i18n::Phrase::fill`, `i18n::number::Grouping::write` | `shared.js`: `fill`, `grouped` | `parity_words::the_shared_*` |
| The countdown's clock, ring and set points | `view::countdown::clock`, `ring_offset`, `crossed`, `elapsed` | `countdown.js`: `clock`, `ringOffset`, `crossed`, `elapsed` | `parity_words::the_countdown_*` |
| The reveal's count down | `view::identifier::ticked` | `identifier.js`: `ticked` | `parity_words::the_identifier_*` |
| The options a combobox keeps and the rows a filter keeps | `view::combobox::ComboOption::matches`, `view::filter::keeps` | `combobox.js`: `matches`; `filter.js`: `keeps` | `parity_words::the_combobox_*` |
| A new repeatable row's key | `view::repeat::fresh_key` | `repeat.js`: `freshKey` | `parity_words::the_repeat_*` |

The modules trim white space as Rust's `str::trim` does (`shared.js`:
`trim`, `trimStart`), so a byte order mark or a next line (U+0085) reads
the same on both sides. A rule only a module holds, such as the editor's
re-highlighting from the first changed line or the day a picker opens on,
has no Rust copy and so no parity test; its journey holds it.

## Which document governs what

| Question | Document |
|---|---|
| The interaction contract, the mode, the crate's layout, the session's end | [`crates/ferrohealth-ui/CLAUDE.md`](../crates/ferrohealth-ui/CLAUDE.md) |
| The component rules every console follows | [`.claude/rules/console-ui.md`](../.claude/rules/console-ui.md) |
| The design library, the order of a change and the 13 family rules | [`.claude/rules/console-design.md`](../.claude/rules/console-design.md) |
| How the design is rendered and checked | [`design/README.md`](../design/README.md) |
| Every component, by its ID (B2, R28, F9 and so on), and the templates T1 to T3 | [`design/COMPONENTS.md`](../design/COMPONENTS.md) |
| Your console's pages as designed | [`design/consoles/<console>/`](../design/consoles/) |
| Integration: the assets, the document, a theme, locales, Your preferences | [`crates/ferrohealth-ui/README.md`](../crates/ferrohealth-ui/README.md) |
| Moving a console from 0.5 | [`docs/migration-0.6.md`](migration-0.6.md) |
| Why server-rendered HTML, the candidates weighed, and what would reopen it | [`docs/rendering-model.md`](rendering-model.md) |

The sections of `.claude/rules/console-ui.md` a console reads most:

- §0 The mandates: server-rendered HTML with a little plain JavaScript, no
  inline style, no route or state in the library, no patient identifier in
  a URL, no string outside the catalogs.
- §1 The interaction contract and the two parts (`view` and `scripts`).
- §2 Templates and markup, and §3 Modules.
- §5 Forms and §6 Tables and paging.
- §7 Accessibility.
- §8 Tokens, themes, type.
- §13 Strings and locales.

The FerroUI repository's `design/` is the only design source. A design
folder anywhere else, in a download or in a console repository, is stale
for a console: a console never builds from it or syncs from it.

## Which version to read

Read FerroUI's documents at the git tag that matches your `ferrohealth-ui`
pin: with `=0.5.8`, read the tag `v0.5.8`; with `=0.6.0`, read `v0.6.0`.
Read `main` only when you are moving your console to the next release. The
rules on `main` describe components your pinned version may not have.

The design library, `console-design.md` and this page first appear in
0.3.0, so a console still on 0.2.0 has none of them at its tag. Start new UI
work after your console moves to 0.3.x. The server-rendered mode
first appears in 0.6.0; at an earlier tag the documents describe the
hydrated mode.

## Moving to a new release

The consoles move together, each under a migration issue in its own
repository. For each console a move:

1. bumps the pin and takes the stylesheet, the themes and the modules from
   the release (the asset router serves them at the new version's path);
2. applies every breaking change named under Changed and Removed in the
   changelogs, and deletes the console's local markup for those parts;
3. refreshes and looks at the console's screenshots and journeys.

The move to 0.6.0 replaces the console's whole UI layer, in the order
[`docs/migration-0.6.md`](migration-0.6.md) gives: FerroEHR, FerroBRIDGE,
FerroFED, FerroTERM, FerroPIX.

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
4. Keep your own work to the console's side: routes and handlers, post
   routes, backend client, configuration and headers, journeys and words.
5. Check the page with the scripts refused: every important action is a
   link or a form.
6. Put every word in your console's catalog and run the stray-literal
   check.
