<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - 2026-10-30

### Added

- The combobox answers a `fui:choose` event on its root, which puts it on a
  listed value, or on none, without a change event. A pick-or-type field's
  back control uses it: back puts the picker on the listed option chosen
  before the another option, or on its first, for a select and a combobox
  picker alike (#517).

### Changed

- **Breaking:** the library renders server-side HTML from Askama templates
  and depends on no Leptos crate. Every component is a function or a
  struct that returns `Html`, which a console's own Askama template writes
  unescaped with `{{ part }}`; a component that needs words takes the
  request's `&Locale`. The shell writes the whole document, and the
  interactive behaviour comes from small plain ES modules under
  `/ferrohealth-ui/<version>/`, which the one loader `fui.js` imports for
  each element that asks for one with `data-fui`. Every page works with the
  modules refused. `serve::axum_router::router()` (the default `axum` feature) serves
  the stylesheet, the brand themes and the modules. `docs/migration-0.6.md`
  maps every removed item to its successor (#510).
- The date picker's months view draws an up chevron after its year title,
  as the design draws it, beside the down chevron of the days view's month
  title. Both are hidden from assistive technology (#518).

### Removed

- **Breaking:** the `island` and `widget` modules, whose components now
  live under `view` beside their script modules, and with them the
  hydration layer: `widget::hydration`, `widget::prefetch`,
  `widget::submit`, the answer mode of `widget::action` and the
  `data-ferroui-hydrated` mark. A link is a link and a form posts as the
  browser posts it (#510).

## [0.5.8] - 2026-10-29

### Added

- `view::dialog::danger_action_dialog` draws a danger confirmation with fields
  inside the `<ActionForm>` a console draws for its server action (class
  `view::dialog::DIALOG_FORM`): once the page has hydrated, its confirm runs the
  action in the page and the dialog closes when the console's `done` signal turns
  true; with no WebAssembly the form posts to the server function's address as
  a plain form. `danger_form_dialog` keeps posting to a plain route (#500).

### Changed

- Every menu item that is not destructive or disabled draws a visible bar under
  the pointer and the keyboard focus, in the new `--surface-hover` token: a
  neutral tint a little darker than the panel in light and a little lighter in
  dark (1.37:1 and 1.53:1 against it, the most muted text on it allows at
  4.5:1). Danger items keep `--bad-ground`, a pressed or current item and a
  chosen listbox option keep `--accent-tint` under the pointer, and a disabled
  item draws no bar. The combobox and listbox options and the tree rows hover
  with the same token (#498).

### Fixed

- A form can mark any number of secret fields: `widget::action::secret_mark`
  names each mark after the field it marks (`ferroui_secret_` and the
  name's bytes in hexadecimal), so a hydrated `<ActionForm>` with two marks
  no longer fails on `serde_qs` refusing a name posted twice. The plain post
  still keeps every marked value out of the answered page, and a mark named
  `ferroui_secret` alone, as 0.5.7 drew it, is still read (#503).

## [0.5.7] - 2026-10-29

### Fixed

- The FerroEHR console design names the party-relationship routes under
  `/ferroehr/rest/extensions/v1/`, where FerroEHR 5.6.0 serves the routes
  ITS-REST 1.1.0 does not define, in the refusals of the relationship pages (#493).
- The search box drops the ask still waiting for the words to rest when
  its form is submitted or one of its options is followed, so the panel no
  longer opens again over the page the router draws in place (#494).

### Security

Shipped with functional changes because: FerroTERM and FerroFED wait on the search fix of the same release, and the leak was confirmed while that release was being cut, so one patch carries both.

- A field an answer form marks with `widget::action::secret_mark`, a password
  or a key, goes to the server function alone: the page `draw_answers` draws
  with the answer leaves it out of `Posted`, so its value is no longer written
  into the page's hydration data. `view::secret::secret_field` marks its value
  input itself; a console marks its own secret inputs, `view::form::masked_input`
  among them (GHSA-25wh-2jcm-gvpr) (#499).

## [0.5.6] - 2026-10-28

### Added

- `view::template::settings::ValueSource::CommandLine`, "From the command line" with the keyboard glyph, marks a settings row whose value a `--set key=value` argument set at start, which wins over the environment, the file and a revision. A console built on `ferrohealth-server` maps the command-line layer `ferrohealth-config` reports to it in place of drawing the row unmarked. `ValueSource` is `#[non_exhaustive]`, so no console's code breaks; the message `setting-source-command-line` joins the library's `templates` catalog (#488).

### Fixed

- `widget::action::draw_answers` follows a server function that calls `leptos_axum::redirect` during a plain post of an answer form. Under the forwarded `Accept: */*` that call answers `200` with `Location` and the `serverfnredirect` header, which the wrapper used to draw as the form's page with no answer; it now answers `303 See Other` to that `Location`, with the cookies the function set. A console that answered `303` itself to work around this can call `leptos_axum::redirect` again (#489).

## [0.5.5] - 2026-10-27

### Added

- `widget::action` answers a plain post on its page: a console wraps its
  router in `draw_answers`, draws `answer_field()` in an `<ActionForm>` and
  calls `answered(&action)` where it makes the action, and with no
  WebAssembly the post draws the page the form was on with the function's
  answer, or its error, as the action's value and the posted fields
  (`Posted`) in the form, `Cache-Control: no-store`. The redirect mode of
  `return_field` and `with_error` is unchanged, and with WebAssembly the
  action runs in the page as before. `widget::hydration::hydrate` puts the
  page's own address in the history before it hydrates such a page.
  `docs/consoles.md` § An action that answers on its page documents both
  modes (#485).

## [0.5.4] - 2026-10-27

### Added

- `widget::editor::code_editor_by_edit` and
  `widget::editor::editor_panel_by_edit` draw the editor and the code
  editor panel as `code_editor` and `editor_panel` do, and hand the console
  each edit the reader makes as a `widget::editor::TextEdit` (the replaced
  byte range and its replacement) instead of the whole text, so a console
  keeps its own copy in step with `TextEdit::apply` at the cost of the
  edit. The editor's `text` signal then carries the opening text and
  outside changes only; the console does not write each edited text back
  into it. `code_editor` and `editor_panel` are unchanged (#478).

### Fixed

- The code editor panel (`widget::editor::editor_panel`) keeps its
  toolbar's line count and its status line's changed state by each edit:
  the count comes from the editor's index of the lines, and the changed
  state compares only the bytes between the parts the draft still shares
  with the opened text, so a key no longer counts the lines of the whole
  text or compares the whole draft with the opened one (#477).
- The shell's skip link (`view::shell::skip_link`) now moves the focus to
  the main area on a page that hydrates whole: it carries `target="_self"`,
  which the `leptos_router` link handler leaves to the browser, so the
  browser's fragment navigation moves the sequential focus starting point to
  `#main` and the next Tab reaches the page's first control (WCAG 2.2 SC
  2.4.1). Without hydration it stays a plain fragment link. A console that
  set `target="_self"` on the link after hydration can drop that (#482).

## [0.5.3] - 2026-10-27

### Fixed

- The editor's work per keystroke follows the edit, not the length of the
  text: an edit re-highlights only the lines it touched (a grammar that reads
  across lines, JSON, XML, AQL and HL7 v2, is highlighted whole again once
  typing rests for 250 ms), the caret is read from an index of the lines
  instead of a scan of the text, and a key no longer reads the text area's
  text more than once. Typing into a 64 KB document takes the editor about
  0.4 ms of script per key, down from about 2 ms (#458).
- The search box's panel placement no longer panics when a navigation
  disposes the search between the panel opening and the next animation
  frame: the search cancels the pending frame when it is disposed, and the
  frame reads and writes nothing of a disposed search (#462).
- A search whose answer route answers `401` now says to every other tab of
  the console that the session has ended (`widget::session::gone`), so they
  leave for sign-in with it; with `SearchBox::signed_out` set its own tab
  still loads the signed-out page, and its session watch stays put (#474).

## [0.5.2] - 2026-10-27

### Added

- A signed-in page leaves for sign-in the moment its session ends:
  `island::SessionWatch` (`widget::session`) loads the console's sign-in
  page with the page's address as `return` when the session is gone, warns
  with a modal alert dialog at least 20 seconds (120 by default) before an
  idle timeout with Stay signed in and Sign out (WCAG 2.2 SC 2.2.1), asks
  again when the tab is shown or focused, and ends every tab of the console
  together over a `BroadcastChannel`. Only the user's own input extends the
  session. A console draws the watch in the shell of a signed-in page,
  answers its session route on the wire `docs/consoles.md` § The session's
  end documents (`GET` reads, `POST` extends, `401` when gone), draws
  `island::SessionEnded` on its signed-out page, and calls
  `widget::session::gone()` for a `401` its own code meets (#444).
- `widget::action` carries the page a console's `<ActionForm>` returns to with no WebAssembly in the form: `return_field()` draws the hidden `ferroui_return` field, and the server function redirects a plain post to `plain_return()`, a path on the console's own origin, or to `with_error()`, which adds the error the router reads into the action's value. Before, server_fn sent the post back to the `Referer`, which every console's `no-referrer` policy withholds, so the browser landed on `/` (#449).
- `view::sign_in::State`, `Notice` and `Retry` implement `Serialize` and `Deserialize`, so a console carries a sign-in state its server chose, such as a refused callback, into the hydrated page as a `SharedValue` (#451).
- `widget::submit::drawn()` counts a post a console draws in the page through its own server function, so `posts()` changes and the search box ends its wait (#452).

### Deprecated

- `widget::hydration::mark()`: its effect can run while a lazy route's code
  is still on its way, so the mark could come before the page hydrated.
  Hydrate with `widget::hydration::hydrate(App)`, which writes the mark when
  hydration ends (#448).

### Fixed

- The top bar's search writes its state only while it lives: a blur, a
  submit or a history `pageshow` that arrives after a navigation disposed
  the search no longer reads a disposed value (#445).
- A click or a submit while a lazy route still hydrates no longer panics
  the page. `widget::hydration::hydrate(App)`, the new browser entry in place
  of `leptos::mount::hydrate_lazy`, leaves every click and submit to the
  browser until the page and its lazy route have hydrated, so a link loads
  its page and a form posts as with no WebAssembly. It writes
  `data-ferroui-hydrated` when hydration ends, where `widget::hydration::mark()`
  wrote it from an effect that can run before a lazy route has hydrated;
  `mark()` is deprecated and does nothing once `hydrate` is the entry. A
  console calls `hydrate` from its browser entry and drops `mark()` (#448).
- The sign-in card erases its sections into `AnyView`, so a console's server build renders it under rustc's default recursion limit and needs no `#![recursion_limit]`. The library's tests compile every view at a limit of 80, which keeps 48 levels for a console's own nesting (#450).
- The search box leaves its busy state, and its field is enabled again, once the page its submit leads to is drawn: when the router's path changes or a post is drawn in the page. Before, only `pageshow` ended it, which a navigation in a hydrated page never fires, so the field stayed disabled after the first search (#452).

## [0.5.1] - 2026-10-26

### Added

- `island::ShortcutLegend` opens the keyboard shortcut legend on `?`
  pressed outside a field, and from any control marked
  `widget::shortcuts::OPENS_LEGEND`, which the user menu's Keyboard
  shortcuts item now is; never in the phone layout, and the focus returns
  where it was. The binder's link tabs follow the keys `1` to `9`, pressed
  outside a field, from 40 rem up (`widget::shortcuts::tab_keys`, drawn by
  the binder) (#413).
- `widget::hydration`: `mark()` writes `data-ferroui-hydrated` on the root
  element once the page has hydrated, which a console's code, scripts and
  tests wait for before they change the page, and in a debug build
  `explain()` has a panic before that point say its likely cause, a page
  changed before hydration ended, where Leptos names none. The consumer
  guide gives the rule (`docs/consoles.md`, How a console builds) (#424).
- `SearchBox::all` names the panel's option that submits the search, around
  the open `query`, for a console whose submit runs another search than the
  panel's (a patient search the panel never runs); `None` keeps "See all
  results for “{query}”". `SearchBox::signed_out` is the address the page
  loads when the answer route answers `401`, so a session that ended while
  the panel was open hands the page to the console's sign-in (#431).

### Fixed

- The user menu closes once one of its items is chosen: since the router
  draws the next page in the document, a link or a theme choice in the menu
  left it open over the new page. Its handlers also read nothing the router
  has disposed when a toggle or a focus change arrives after a navigation, as
  the action menu's do (#423).
- The shell can be drawn once and survive a navigation in the page, as
  #410 meant: `widget::nav::follow_path()`, called in the component that
  draws the shell inside the console's `Router`, lets the navigation mark
  the page in view and the rail's and the theme choice's `back` name it as
  the router moves, so a console no longer draws its shell again for each
  path. The search's words and the shell's state now survive a link click
  in the gallery (`docs/consoles.md`, How a console builds, step 10) (#437).

## [0.5.0] - 2026-10-25

### Added

- `island::FormNavigator`, which a console draws once inside its `Router`:
  a `GET` form navigates in the page with its fields in the query, and a
  `POST` form marked `widget::submit::NAVIGATES` (the library's preference,
  theme, rail, segmented and pager forms) is posted in the background and the
  router draws the page its redirect names, falling back to the browser's own
  submit when the post fails or answers with no redirect. A form or submit
  marked `widget::submit::RELOADS`, or with a `target`, submits as the browser
  does, and so does every form with no WebAssembly. `widget::submit::follow`
  follows an address as a link press does, which the tabs, the tree, the
  topology and the search panel now use, and `Act::Download` draws a link the
  router leaves to the browser (`rel="external"`, `download`), which an
  export link now is (#410).
- `island::PageAnnouncer`, which a console draws once at its root: after a
  navigation in the page it moves the focus to the new page's heading and says
  the page's title in a polite live region, as a new document would (WCAG 2.2
  SC 2.4.3, SC 4.1.3) (#410).
- `widget::prefetch::routes` loads a lazy route's code when a link to it is
  hovered or focused, and the main navigation's routes once the first page is
  idle, one after another, so a first visit costs no more than a return. The
  gallery hydrates whole with its 21 pages in five lazy routes: its entry
  WebAssembly is 483968 bytes brotli-compressed with the navigator and the
  announcer, under its 491520-byte budget, and its pages' code is 32 chunks
  (#410).

### Changed

- Breaking: a console hydrates the whole page and draws every page after the
  first in the browser, so no link or form reloads the document; islands mode
  is gone. Every ready component of `island` is a `#[component]`, the crate no
  longer turns on the `leptos` feature `islands`, and its `split` feature is
  gone. A console drops `islands` and `split`, draws its pages in a few
  `leptos_router` `LazyRoute`s, one for each group of pages read together,
  hydrates with `leptos::mount::hydrate_lazy`, writes
  `<HydrationScripts options/>`, builds with `cargo leptos build --split`,
  draws `island::FormNavigator` and `island::PageAnnouncer`, and provides
  the same locale and state in both halves (`docs/consoles.md`, How a console
  builds). A repeatable list's blank row is a hidden, disabled `fieldset` in
  place of a `template`, so it hydrates with the page and never posts (#410).

## [0.4.0] - 2026-10-25

### Added

- A gallery journey tabs through a long page and scrolls each of its controls
  into view at 1440 and 390 px, and fails when one ends under the sticky top
  bar (WCAG 2.2 SC 2.4.11) (#408).
- `view::search::find_results` draws a find page's results in the page under
  its heading, one list card per kind with every hit a link, the warning
  when the answer may not be whole and the empty state when nothing matched:
  what a search shows with no WebAssembly (#409).

### Changed

- The top bar's search answers under the field as you type, and its answer
  is never drawn in a page again. Once the page hydrates, the search box asks
  the console's answer route (`SearchBox::suggest`, from
  `SearchBox::suggest_from` characters) and opens a panel anchored under the
  field: the WAI-ARIA combobox with a listbox popup, its hits grouped by kind
  under headings, at most five of a kind, the words searched for emphasised,
  and a "See all results" option that submits to the find page; ArrowDown and
  ArrowUp walk every kind with the focus kept in the field, rows show while it
  waits, when nothing matched and when the route failed. Breaking:
  `search_box` takes the `SearchBox` alone, `search_answer` is gone (draw the
  find page with `find_results`), the `Search` island takes `SearchWords` in
  place of its children, `SearchAnswer` gains `query` and `unfinished` and
  crosses the wire as JSON, and the hit classes are `fui-hit`,
  `fui-hit-name` and `fui-hit-line`. The route's wire is in
  `docs/consoles.md` (#409).

## [0.3.7] - 2026-10-24

### Added

- T3: a setting row says where its value came from when that is not the
  page's own source, small and muted after the value with its glyph:
  `view::template::settings::ValueSource::Environment`, "From the
  environment", for an environment variable, which wins over the file and
  a revision; `ValueSource::Default`, "Default", when nothing sets it; and
  `ValueSource::File`, "From the file", for a section the file sets in a
  managed configuration. The page's own source is unmarked: the file in
  file mode, the running revision when managed. The gallery's file-mode
  settings page shows an environment and a default value (#396).
- The FerroFED configuration design has an Authentication tab after FHIR
  face, in the managed and the off states, which a console draws with
  `view::template::settings::settings_panel` and `setting_row`: who may
  call, read-only with no Edit in both states, since only `ferrofed.toml`
  sets it. It holds the audience, the operator scope, each token issuer
  with its key set file, and each trusted issuer with its introspection
  secret named by its file and, for a national contact point, its claim
  names, its correlation header and its added countries (#398).

### Changed

- Breaking: `view::template::settings::Setting` gains the public field
  `source: Option<ValueSource>`, so a console sets `source: None` on each
  setting, or the source its running configuration names, in place of its
  own marker. The library's catalog adds `setting-source-default`,
  "Default", `setting-source-environment`, "From the environment", and
  `setting-source-file`, "From the file", so a console that translates the
  library's catalog adds the three (#396).
- Breaking: E5's node inspector draws Build a query on this path only
  where the node has a query: `view::browser::PathNode::query` is an
  `Option<String>`, and a node with `None`, such as one of a demographic
  template (a PERSON's or a ROLE's), where no query can be built, draws no
  button. A console wraps its link in `Some(..)` to keep today's drawing,
  and passes `None` for a template no query can be built on (#399).
- `SECURITY.md` states the fix-time targets every product of the Ferro family
  commits to, counted from the confirmed assessment: a fixed release within
  7 days for a Critical vulnerability, 30 days for High, 90 days for Medium
  and with the next release for Low, beside the acknowledgement within about
  5 business days and the initial assessment within about 10, both best
  effort as before (#403).

### Fixed

- A banner's text, a notice's body and a setting row's description wrap a
  word longer than their box (a URL, an identifier, a long compound in a
  translation), so nothing runs past the screen at 320 px. Before, the
  rest of the line ran past the right edge on a phone. The gallery's
  overview shows a banner holding a long URL (#397).
- A link in a notice keeps its words on one line while they fit, moving
  whole to the next line, and wraps only when it alone is wider than the
  notice. Before, its `white-space: nowrap` ran a long link past the
  screen's edge at 320 px (#397).

## [0.3.6] - 2026-10-24

### Changed

- Breaking: `view::chart::ActivityChart` takes its axis labels as
  `ticks: Vec<(u32, String)>`, `(day index, words)`, in place of `ends`,
  so a console labels the days the FerroEHR mockups label (22 Sep, 1 Oct,
  11 Oct and 21 Oct). A console that set `ends: (first, last)` writes
  `ticks: vec![(0, first), (last_day, last)]` for the same drawing (#387).

### Fixed

- The FerroEHR New contribution mockup draws an Apply submit beside each
  staged change's What changes, as a console has to with no WebAssembly,
  since choosing a kind redraws the fields under it. The select and Apply
  share one row inside the field, `view::layout::field_row(FieldRow::WithActions, ..)`,
  with the help line under both. COMPONENTS.md
  § Interaction states the rule: a select whose choice changes the page
  carries a visible submit beside it and never posts on its change alone
  (WCAG 2.2 SC 3.2.2 On Input), as the phone tab select's Go does. A
  console that draws Apply already needs no change, and can move it into
  that row to match the design (#386).
- A dialog drawn inside a table's actions cell wraps its sentence: every
  dialog (`fui-dialog`) sets `white-space: normal` and `text-align: start`,
  so it no longer inherits the cell's `nowrap` and end alignment, which cut
  its sentence off at 390 px. The gallery's member removals sit in a table,
  each confirmation beside its row menu in the actions cell. A console
  changes nothing (#388).
- A row menu drawn in a table's actions cell (`fui-table-actions`) sits on
  the row's first line, its trigger centred on the line as a button there
  is, so the row keeps its height and the name beside it no longer reads
  above the ⋯. A console changes nothing (#391).

## [0.3.5] - 2026-10-23

### Added

- A danger confirmation that asks for the reader's words, such as the
  required Reason a removal keeps with the revision:
  `view::dialog::danger_form_dialog(confirmation, form, confirm)` draws the
  console's fields (`view::dialog::ConfirmForm::fields`) between the
  sentence and the actions, inside one `POST` form to
  `ConfirmForm::action` that the confirm submits, so a required field holds
  the post with no script. A row menu's Remove… (`Act::Dialog(id)`) opens
  it, one dialog per row, and the focus moves to its first field. The
  stylesheet's dialog rhythm reaches inside its form (`fui-dialog-form`).
  The gallery's members on the overlays page are removed this way (#374).
- A menu item (F13) and a title row's action (F14) can submit the console's
  own form to another address: `widget::action_menu::Act::SubmitTo { action,
  form }` draws a submit button with `formaction`, of the form with the id
  `form` or of the form it sits in, so no nested `<form>` is dropped by the
  parser (#375).
- A menu behind a small labelled button: `widget::action_menu::labelled_menu(glyph,
  words, menu)` and its trigger `widget::action_menu::Trigger::Labelled { glyph,
  words }`, such as E7's Export (#375).
- The Lucide 1.51.0 glyph `Glyph::SCROLL_TEXT` for the FerroEHR design's
  Stored queries, which a console draws in place of its stand-in
  `SEARCH_CODE` (#376).

### Changed

- Breaking: `view::scope::Request` gains the `Profile` variant, so a console
  that matches on `Request` exhaustively adds a `Profile` arm. The library's
  catalog adds `scope-profile`, "Your profile: your name, picture and the
  like", so a console that translates the library's catalog adds it (#373).
- Breaking: E7's results pane draws its exports as one small Export button
  opening a menu of the formats (F13), CSV then JSON, as the FerroEHR
  design draws it; with no row Export is disabled. An `Export::Submit`
  item submits the console's form by `formaction`, so the pane still draws
  no form of its own inside the console's. `view::results::results_pane`
  now holds a menu island, so a console renders it inside a page with the
  server's shared context, as it does every menu. A console's own code
  needs no other change (#375).
- Breaking: the library's catalog adds `results-export`, "Export", and
  rewords `results-export-csv` and `results-export-json` to "CSV" and
  "JSON", the menu's items under Export. A console that translates the
  library's catalog adds `results-export` and rewords the two (#375).
- Breaking: `widget::action_menu::Act` gains the `SubmitTo` variant, so a
  console that matches on `Act` exhaustively adds a `SubmitTo` arm (#375).
- Breaking: `widget::action_menu::Trigger` gains the `Labelled` variant, so
  a console that matches on `Trigger` exhaustively adds a `Labelled` arm (#375).
- Breaking: `Glyph::ALL` is a `[(&str, Glyph); 111]`, up from 110 in 0.3.4
  with the glyph this release vendors, so a console that names the array's
  length writes 111, or reads it as a slice (#376).

### Fixed

- E8 reads the OpenID Connect `profile` scope as the request it is: the new
  `view::scope::Request::Profile` is read from `profile` by
  `Request::parse`, so `ScopeGrant::read` no longer draws it refused, and it
  says "Your profile: your name, picture and the like" (OpenID Connect Core
  1.0, § 5.4) (#373).

## [0.3.4] - 2026-10-23

### Added

- The user menu (F13) takes a console's own links: `UserMenu::links`, each
  a `widget::menu::UserLink` with its words, its address and its glyph,
  drawn between Your preferences and the theme, such as FerroEHR's Your
  access (#347).
- The banner under the top bar ends with a link after its sentence:
  `view::status::banner` takes a fifth argument, `Option<BannerLink>`, its
  words and its address, drawn after a middle dot in the banner's tone,
  such as a sandbox band's Details (#348).
- A relative time for family rule 11: `view::time::relative(raw, now)`
  draws a `<time>` that reads `13 s ago`, `30 min ago`, `2 h ago` the same
  day, `yesterday`, then days, weeks, months and years, against the now the
  console hands it, with the full timestamp as its title and the machine
  form as its `datetime`; `view::time::relative_reading` gives the words
  alone. The words are in the library's catalog, so every console rounds
  and says them alike. A moment after now reads as its full timestamp (#349).
- E8's operations can be built by a console that parses scopes with its own
  grammar: `view::scope::Operations::of(&[Operation])` and
  `Operations: FromIterator<Operation>`, so a `Scope` literal draws the
  console's own parse in `scope_grants` (#350).
- A copy item for the menus (F13): `widget::action_menu::Act::Copy(text)`
  puts the text on the clipboard with the copy button's clipboard code and
  says so in a toast, Copied to the clipboard or the browser's refusal. Only
  script can copy, so the item is hidden until the menu's island runs and
  the text stays where the page shows it. In a title row a copy action is
  the copy button (#351).
- Two Lucide glyphs for the FerroEHR design: `Glyph::GIT_COMMIT` (Lucide
  `git-commit-horizontal`, a contribution) and `Glyph::KEY_ROUND` (Your
  access), vendored from Lucide 1.51.0 (#352).
- The page header's action group (F14) holds a popover form beside its
  primary: `ActionGroup::popover`, set with `ActionGroup::popover(form)`,
  draws an R17 anchored popover form after the primary and before More,
  such as Save to the CDR beside Run (#354).
- E7's results pane rides the console's own form for a run that is a form
  post: `Switch::Form { form }` draws Table and Chart as submits of that
  form posting `shown`, and each export is a `view::results::Export`, a
  link (`Export::Link`) or a submit of that form with `formaction`
  (`Export::Submit`), so no form of the pane's own sits inside the
  console's (#355).
- The line chart's figures table names each point by its own label:
  `LineChart::x_labels`, each an X value and its label, such as a point's
  time on a time axis, which the table under the plot reads before a tick's
  label and the number; the axis draws `x_ticks` alone (#358).
- Three Lucide 1.51.0 glyphs for the FerroBRIDGE flow, the FerroTERM
  sidebar and the FerroPIX navigation: `Glyph::SEND`, `Glyph::BOOK_OPEN`
  and `Glyph::USER_ROUND_SEARCH`. A console replaces its stand-ins
  (`RADIO_TOWER`, `BOOK_A`, `SEARCH`) with them (#360).
- B9's edge rule is public: `view::flow::edge_rule` draws one line per
  `EdgeRule` (the state's line sample, word and line style in the library's
  words, then the console's meaning), and F9 can use it too.
  `view::flow::rule_dialog(FlowRule)` puts it in the short "How the flow is
  drawn" dialog with the console's closing paragraph. `rule_button(id)` is
  the "How it is drawn" button for the flow panel's head, which opens the
  dialog with no script. `FlowState::line` gives each state's line style (#361).
- Two Lucide 1.51.0 glyphs for the FerroTERM design: `Glyph::ARROW_LEFT_RIGHT`
  (Concept maps) and `Glyph::GIT_COMPARE` (Compare). The design's `history`
  is the release's deprecated alias of `rotate-ccw-clock`, so
  `Glyph::ROTATE_CCW_CLOCK` stays the Recent writes glyph (#363).

### Changed

- Breaking: `widget::menu::UserMenu` gains the public field `links`, so a
  console that writes a `UserMenu` literal adds `links: Vec::new()`, or its
  own `UserLink`s (#347).
- Breaking: `view::status::banner` takes a fifth argument, an
  `Option<view::status::BannerLink>`, so every call adds one: `None` for a
  banner with no link (#348).
- Breaking: the library's catalog adds the relative-time messages
  `time-ago-seconds`, `time-ago-minutes`, `time-ago-hours`, `time-yesterday`,
  `time-ago-days`, `time-ago-weeks`, `time-ago-months` and `time-ago-years`
  (each but `time-yesterday` taking `$count`), so a console that translates
  the library's catalog adds the eight (#349).
- Breaking: `widget::action_menu::Act` gains the `Copy` variant, so a
  console that matches on `Act` exhaustively adds a `Copy` arm.
  `widget::action_menu::menu_button` takes a `copy` argument
  (`Option<widget::copy::CopyWords>`) before its children, and
  `island::MenuButton` a `copy` prop: `None` for a menu with no copy item
  (#351).
- Breaking: `Glyph::ALL` is a `[(&str, Glyph); 110]`, up from 103 in 0.3.3
  with the seven glyphs this release vendors (#352, #360, #363), so a
  console that names the array's length writes 110, or reads it as a slice.
- `widget::editor::Editor::id` takes a `Cow<'static, str>`, so a page with a
  number of editors known only at run time gives each its own id and field
  name, such as `staged-3`. Breaking: a static id becomes `"draft".into()` (#353).
- Breaking: `view::action::ActionGroup` gains the public field `popover`,
  so a console that writes an `ActionGroup` literal adds `popover: None`, or
  builds the group with `ActionGroup::popover(form)` (#354).
- Breaking: `view::results::ResultsPane::csv` and `ResultsPane::json` take a
  `view::results::Export`, so an address becomes `Export::Link(address)`,
  and `view::results::Switch` gains the `Form` variant, so a console that
  matches on `Switch` exhaustively adds an arm (#355).
- Breaking: `widget::editor::Editor::editable` says whether the text may be
  edited at all, and an editable text area takes keys from the first paint.
  A console that passed a signal turning true as the page hydrated passes
  `Signal::stored(true)` for an editable text, and false only for a text
  the reader may only read (#356).
- Breaking: `view::dock::Decision` gains the public field `leaves`, so a
  console that writes a `Decision` literal without `..Decision::default()`
  adds `leaves: false`, and sets `leaves: true` on its Discard (#357).
- Breaking: `view::chart::LineChart` gains the public field `x_labels`, so a
  console that writes a `LineChart` literal adds `x_labels: Vec::new()`, or
  each point's own label (#358).
- Breaking: the library's catalog adds the flow rule's messages
  `flow-rule-button`, `flow-rule-title`, `flow-style-up`,
  `flow-style-degraded`, `flow-style-unreachable`, `flow-style-not-checked`
  and `flow-style-off`, so a console that translates the library's catalog
  adds the seven (#361).
- T3's file line can open the informational dialog: `Source::File::how` and
  the second argument of `file_line` are a `view::template::settings::How`.
  `How::Page(url)` keeps the link, and `How::Dialog(id)` draws a button
  styled as the link (`fui-link-button`) that opens `info_dialog` by its id
  with the `show-modal` command, with no script. Breaking: a console wraps
  its URL in `How::Page(…)`, or passes `How::Dialog(…)` and drops a
  title-row button that opened the same dialog (#364).
- Breaking: `widget::repeat::Repeat` gains the public field `token`, so a
  console that writes a `Repeat` literal adds `token: ROW`, and a list
  nested in a `Fields::Card` row names a token of its own there and in its
  blank row's ids and names (#366).

### Removed

- The gallery's `cl-minutes-ago` message: the clinical version history
  reads its times through `view::time::relative`, whose words are the
  library's `time-ago-*` messages. A console that copied the gallery's
  catalog drops it and translates the library's messages instead (#349).
- Breaking: `widget::editor::Editor` no longer implements `Copy`, since its
  id may be made at run time; it is `Clone`. An editor used twice, as by
  `editor_panel` and `editor_diagnostics`, is cloned for the first use
  (#353).
- Breaking: `view::action::ActionGroup` no longer implements `Clone`,
  `PartialEq` or `Eq`, since its popover form holds a view. A console that
  cloned or compared a group builds it again where it needs a second one
  (#354).

### Fixed

- The code editor (B2), the YAML editor and the upload field work with no
  WebAssembly: an editable text area takes keys from the first paint, shows
  its own text in ink over the layer's gutter until its island runs
  (`fui-editor-stack.is-static`), and hands what it holds as the page
  hydrates to `on_input`, so an early key is never lost or written over (#356).
- A decision of the docked bar (R4) that leaves without keeping what the
  form holds, such as Discard, skips the form's checks: `Decision::leaves`
  draws `formnovalidate`, so a required field left empty no longer holds
  the reader on the page (#357).
- F7: Add and Remove on `Fields::Card` rows move the focus to a card's
  first field. Before, they moved it to the Remove in the card's head, so a
  keyboard user's next Enter took the new card back out (#365).
- F7: a list nested in a `Fields::Card` row adds rows in place in a card
  that Add inserted. `Repeat::token` names each list's own key token
  (`ROW` for a top-level list, a token of its own such as `__profile__` for
  a nested one), and the island rekeys the content of nested templates. It
  also takes the presses of a nested list in a card it copied, and finds
  only its own blank row, rows, empty line and Add. Before, a list whose
  drawn cards held a nested blank row copied that nested row when Add was
  pressed (#366).

## [0.3.3] - 2026-10-23

### Added

- Five Lucide glyphs for the FerroBRIDGE and FerroFED navigation:
  `Glyph::FLAME`, `Glyph::MESSAGE_SQUARE`, `Glyph::TABLE_2`, `Glyph::PLUG` and
  `Glyph::BUILDING_COMPLEX`, vendored from Lucide 1.51.0. That release names
  `building-2` a deprecated alias of `building-complex`, so FerroFED's
  Organisations entry takes `Glyph::BUILDING_COMPLEX` and `Glyph::named`
  resolves `building-complex` (#321).
- `view::state::state(mark, word, detail)` draws the F12 state mark a table
  cell, a fact or a title badge shows: the shape of `view::state::Mark` (`Ok`
  a filled circle, `Warn` a diamond, `Bad` a cross, `Neutral` a hollow ring,
  `Off` the hollow ring with the word muted), the word, and an optional quiet
  detail. `Mark` converts from `view::flow::FlowState`. The subject line, the
  version history and the topology's list draw their states through it, and
  the topology's list now hides the shape from assistive technology, since
  the word beside it names the state (#322).
- R10 takes a masked input with a hold-to-show eye: `view::form::masked_input`
  draws the input `type="password"` with no autocomplete, capitals or spelling
  check, and `view::form::Inner::HoldToShow` puts the eye inside its edge
  (`island::HoldToShow` over `widget::masked::hold_to_show`, words in
  `widget::masked::HoldWords`, named by the console with `HoldWords::named`).
  Held by the pointer, by touch or by Space or Enter, the eye shows the typed
  text with `aria-pressed` in step, and released, left or blurred, it masks it
  again. The eye names its input by id, so the typed value is never an island
  prop. Without WebAssembly it is disabled, its name says the field stays
  masked, and the field stays masked (#323).
- `view::surface::Header::subtitle_view` gives a page header a subtitle with
  inline markup, such as a bold word and a code link to another page, in the
  `fui-page-subtitle` style, drawn in place of the text `subtitle` when set.
  The text form works as before (#326).
- F7 repeatable rows take two more layouts: `widget::repeat::Fields::One`, a
  row of one field with its Remove, and `Fields::Card`, a sub-card whose
  heading (the new `RepeatRow::heading`) names its group with Remove beside
  it, then its fields, a list of its own among them when it has one. A press
  inside a nested list adds or removes only that list's rows (#330).
- Four Lucide glyphs for the FerroTERM design: `Glyph::CORNER_DOWN_RIGHT`,
  `Glyph::GIT_BRANCH`, `Glyph::LIST_FILTER` and `Glyph::LIST_TREE`, vendored
  from Lucide 1.51.0. That release names `trash-2` a deprecated alias of
  `trash`, whose outline `Glyph::TRASH` already draws, so the design's
  `trash-2` is `Glyph::TRASH`. `Glyph::ALL` holds 103 glyphs (#336).
- `table::filter::filter_menu`, T1's filter menu: a `<details>` whose summary
  is the button, a checkbox per `FilterChoice` with the number of rows it
  keeps and an optional mark, and Show, sent in a `GET` form of its own
  (`MenuForm::Own`, carrying the listing's other fields) or in the find
  field's form (`MenuForm::Enclosing`), so a listing filters by several
  choices at once with no WebAssembly. While some choice is off the button
  says "Source · 2 of 5" in the accent. F9's Status menu draws through it (#338).
- `view::dialog::info_dialog` with `Explanation`: the informational dialog,
  a page of explanation such as "How configuration works", with the info
  glyph beside its title, the console's paragraphs, and Close as its only
  action, which takes the focus. Its trigger opens it with the `show-modal`
  command, so it works with no WebAssembly (#339).
- The release notes of each release state the month its five years of
  security fixes end (Regulation (EU) 2024/2847 Art 13(19)). The accepted
  `paste` advisory (RUSTSEC-2024-0436) is published as an OpenVEX statement,
  `security/vex/rust-advisories.openvex.json`, for a scanner to read with
  `trivy --vex`. `SECURITY.md` names the `info@cadasto.com` reporting route
  and the advisory each fixed vulnerability is published as (#343).

### Changed

- Breaking: `view::form::Inner` gains the `HoldToShow` variant, so a console
  that matches on `Inner` exhaustively adds an arm. A console that
  translates the library's catalog adds `hold-show` and
  `hold-show-needs-script`, the eye's name with WebAssembly and without (#323).
- Breaking: a refusal with no status shows its summary alone as its title, as
  the design draws it; "No answer:" moves into the title's visually hidden
  lead, so assistive technology still says no answer came.
  `view::refusal::refusal_title` returns the summary alone for such a refusal,
  the new `view::refusal::refusal_lead` returns the hidden lead, and the
  catalog's `refusal-no-answer` is the lead alone and takes no `$summary`. A
  console that draws the title itself draws `refusal_lead` before it, and a
  translated catalog drops `$summary` from `refusal-no-answer` (#324).
- Breaking: `view::surface::Header` gains the public field `subtitle_view`,
  so a console that writes a `Header` literal without `..Header::default()`
  adds `subtitle_view: None` (#326).
- `view::popover::PopoverForm::id` and `view::dialog::Confirmation::id` take a
  `Cow<'static, str>`, so each table row's popover form or confirmation can
  carry an id of its own, such as `remove-node-c`. Breaking: a static id
  becomes `"save-query".into()` (#329).
- Breaking: `widget::repeat::RepeatRow` gains the public field `heading`, so
  a `RepeatRow` literal adds `heading: None`, and `widget::repeat::Fields`
  gains the `One` and `Card` variants, so a console that matches on
  `Fields` exhaustively adds two arms (#330).
- Breaking: `DatePicker`, `DateRangePicker` and `TreeView` are lazy islands
  only with the new `split` feature. A console that builds with
  `cargo leptos build --split` turns on `features = ["split"]` for
  `ferrohealth-ui` to keep their code in chunks of its own; a console that
  builds without `--split` changes nothing and now hydrates them (#331).
- Breaking: `Glyph::ALL` is a `[(&str, Glyph); 103]`, up from 94 in 0.3.2
  with the nine glyphs this release vendors, so a console that names the
  array's length writes 103, or reads it as a slice (#336).
- Breaking: the topology's Status menu takes its words from the filter menu:
  the catalog messages `topology-show` and `topology-status-some` are gone,
  and `filter-menu-show` and `filter-menu-some`
  (`{ $label } · { $shown } of { $total }`) take their place. A console that
  translates the library's catalog renames the two messages (#338).
- A list card's and a panel section's count can say the items shown of a
  larger set: `ListCard::count` and `SectionHead::count` take an
  `Option<view::template::list::HeadingCount>`, drawn as "· 15"
  (`HeadingCount::All`) or "· 6 of 15" (`HeadingCount::Shown`), each number
  with the locale's marks, and `template::list::counted` takes a
  `HeadingCount`. Breaking: `count: Some(15)` becomes
  `count: Some(HeadingCount::All(15))` (or `Some(15.into())` from a `u64`),
  and a console that translates the library's catalog adds `count-shown-of`
  (#340).

### Fixed

- A `view::segmented::choice_group` legend draws as a field label (the body
  size, weight 500, no rule under it), so the choice lines up with the fields
  beside it; the group heading rule of `fui-fieldset` no longer outranks it (#327).
- The user menu's theme segments keep their words on one line whichever theme
  is chosen, at 1440 px and on the 390 px sheet; the menu grows to the row's
  width instead of breaking "System" (#328).
- The library's lazy islands (`DatePicker`, `DateRangePicker`, `TreeView`) are
  lazy only with the new `split` feature, so a console built without
  `cargo leptos build --split` no longer gets a glue that imports
  `__wasm_split_placeholder__` and hydrates no island. `docs/consoles.md` says
  how a console builds, and the gallery's site build fails on a glue that
  imports the placeholder (#331).
- The code editor (B2), the YAML editor and the upload field keep every key
  typed while the console's text comes back late: the text area is the
  source of the live text, a text the editor handed to `on_input` coming
  back is an echo and never written, and only an outside change that differs
  from the field is written into it. Before, a key typed before the echo of
  an earlier one was overwritten and the caret jumped to the end (#333).
- In a stacked table (B10 below 40 rem) a pill, a badge or a state wraps its
  words inside the value column, so a long state such as "saved, not
  applied" never carries the card past a 320 px page (WCAG 2.2 SC 1.4.10) (#334).
- A focused match in M1's match list draws the focus ring on its row (WCAG
  2.2 SC 2.4.7), and a row link (`fui-row-link`, such as `Coding::href`)
  covers its row and gives up its own ring only in a table or a match list.
  Elsewhere it is a plain link with its own ring: a concept's Parent drawn
  with `Coding::href` in `Layout::Inline` no longer covers the pane and takes
  every click over it, and the phone list of B9's flow and a diagnostics
  file head keep a visible ring (#335).
- M5's root matrix keeps its header row at the top of its own scroll box
  from 64 rem up, above its first row. The wide table's sticky header, a top
  bar's height down, outranked it and drew the header over the first rows
  with an empty band above (#337).

## [0.3.2] - 2026-10-23

### Added

- The concept tree and the concept browser (M1), the folder tree of a
  directory (E4) and the path browser of a template (E5).
  `widget::tree` draws the tree as `TreeKind::Concept`, `Folder` or `Path`:
  a level read one at a time (`Children::Unread` shows the loading row
  while the console's server reads it behind the toggle's link), a long
  level ending in the more row (`More::of_page`), the selected concept's
  path in the heavier weight, M2's inactive mark, a folder's glyph and
  count, a reference's short uid, a path's RM type, and in an edit each
  row's named icon actions (`Edit`, submit buttons of the console's form)
  and the inline rename field. The ready island `island::TreeView` takes
  the tree as plain data (`TreeData`) with its words (`TreeWords`), and is
  lazy. `view::browser` lays out the panes (`Panes::Three`, `Flat`, `Two`),
  the pane's find field, the search's matches and the path inspector
  (`path_node`); `view::path` draws an archetype id that breaks only after
  a dot and a path that breaks only after a slash, a dot or an opening
  bracket. The gallery's Trees page shows each in every brand (#170).
- The coding (M2, `view::coding`): a coded value as its display over its
  code in the code face and, where it helps, its system, stacked or inline
  (`Layout`), with a display that leads to the concept's page. An inactive
  coding is muted and carries the inactive mark, the hollow ring and the
  word Inactive (`inactive_mark`); a code never breaks inside itself, and
  `canonical` draws a canonical that breaks only after a slash. A row's
  name over its canonical is `table::view::title_cell` with the canonical
  as its quiet second line (#171).
- The FHIR version select (M3, `widget::version::version_select`): the
  version a console reads through as one compact control in the top bar,
  the version alone with its label and release as its tooltip and
  accessible name, over a menu of `menuitemradio` links that checks the
  root in view and marks the default beside each release. Below 40 rem it
  leaves the top bar, and `VersionSelect::choices` hands the user menu the
  same roots as its segmented group. The menu island's keys walk
  `menuitemradio` items, and its trigger `Trigger::Version` draws the
  control (#172).
- The value set clause (M4, `view::clause`): an include or exclude of a
  value set as its mark, its source as the title, the sentence of what it
  selects, then its filter rows and its codes, each an M2 coding
  (`clause_codes`), with `clause_part` and `clause_fields` for a
  composer's parts and `clauses` to run them edge to edge in their
  section. Include is a solid accent edge and Exclude a dashed warn edge,
  each with its word, and an invariant the clause breaks is named by its
  number inline (`ClauseIssue`) (#173).
- The repeatable rows (F7, `widget::repeat::repeat` and the island
  `Repeater`): a list of the same fields, each row ending in its labelled
  Remove (or a lock that says why the row stays), one Add under the list
  and a quiet line for an empty list. Remove and Add are submit buttons of
  the console's form, so they post with no WebAssembly; once the page
  hydrates, Remove takes its row out and moves the focus to the next row,
  and Add inserts a copy of the blank row with a fresh key (`ROW`) and
  focuses it. On a phone Remove stays on the row's first line. M4's filter
  rows and M6's target rows are F7 rows (`Fields::Three`) (#173).
- The root matrix (M5, `view::root_matrix`): what each FHIR root of a
  server declares, one column per root, each column head the root, its
  release and whether it answered as an F12 state with its word. The first
  column stays while the rest scroll sideways inside the card under the
  scroll cue; a cell a root does not declare says "not declared", and
  every cell of a root that did not answer says "no answer" (#174).
- The mapping row (M6, `view::mapping`): a concept map's mapping as a table
  row of its source coding, the relationship as its words with an arrow,
  the target coding and the comment (`mapping_columns`, `mapping_row`).
  `Relationship` reads the R4 `ConceptMapEquivalence` and the R5
  `ConceptMapRelationship` codes, each in the words its version reads, and
  a mapping to nothing says so with its own glyph and a named dash for its
  target. In an editor each source code is one block (`map_element`) over
  its target rows, F7 rows that name a missing relationship under the row (#175).
- The subject line (E1), `view::subject::subject_line`: the label Subject,
  the subject's masked identifier with its namespace and Reveal or "No
  external subject", and the record's Queryable and Modifiable states as F12
  marks, a cleared one the hollow ring and its negation. It wraps between
  its parts and never inside the identifier (#176).
- The version history and At a time (E2), `view::versions`: the revision
  table with the Current mark or the Deleted state beside each version, the
  change type, the committer, a relative time and the contribution, the
  version being read marked `aria-current`; At a time under it as a plain
  `GET` form of the date and time field and Open with its quiet line; and
  `older_version`, the one notice over an older version with its link to
  the latest (#177).
- The clinical reading of a canonical document (E3),
  `view::reading::reading`: each RM section a heading with its archetype
  node id as a quiet hint that breaks only after a dot, each element a label
  and value row (two columns, one on a phone), a quantity with its unit as
  read and its UCUM code as the tooltip, a coded value as the console's M2
  coding, a null flavour as the hollow ring and its words, nested clusters
  indented with their headings one level deeper, and one foot that says what
  is folded away (#178).
- The query builder's condition groups (E6),
  `view::conditions::condition_groups`: a group's head reads as one phrase,
  "Match [All | Any] of these conditions", with "Match the opposite (NOT)"
  beside it and a dashed edge when negated; each condition an F7 row of the
  path with its archetype path, the operator, a typed value with its unit or
  a coded select, and Remove, which stays on the row's first line on a
  phone. Every control is a field of the console's form, and Remove, Add a
  condition and Add a group are submit buttons, so the builder works with
  no WebAssembly (#181).
- The results pane of a query (E7), `view::results::results_pane`: a B10
  section with the row count in its heading, Table or Chart as links in the
  URL or as a form the console answers from its session, Export CSV and
  Export JSON, the row window in one quiet line, then the table edge to edge
  or the line chart inset (#182).
- The SMART scope grant (E8), `view::scope`: `Scope::parse` reads the scope
  grammar of SMART App Launch 2.2.0 (the compartment, the resource, a subset
  of `cruds` in order or a SMART 1 suffix, and an optional constraint), with
  SMART on openEHR resources such as `composition-*` among them, and
  `scope_grants` draws each scope with its raw text in mono, the compartment
  and resource in words, the five operations ticked or struck through as
  `<del>` that says "not permitted", the warn badge Broad access, and a
  refused scope in the bad tone with what the grammar expected (#183).
- The gallery's Clinical records page shows the six FerroEHR parts in every
  brand (#183).
- `view::lifecycle`, the revision lifecycle and its pending card (F1):
  `lifecycle` draws Saved, Approval, Apply and Confirm as an ordered list,
  each `Step` done, current (`aria-current="step"`), waiting, not needed or
  failed (`StepState`) in a glyph, a colour and a hidden state word, one
  line from 40 rem up and stacked below it; `pending_card` puts a
  revision's words, its actions, its lifecycle and a line under them in one
  card (`PendingCard`), a polite status region when it carries a tone (#188).
- `view::action::small_action_button`, an `Action` as a small button, for a
  card's header row (#188).
- `widget::countdown` and the island `CommitCountdown`, the commit-confirm
  countdown (F2): a ring and the time left with the deadline in words, drawn
  by the server from the seconds left the console supplies (`Countdown`,
  `CountdownWords::of_page`) and counted down in the browser by the page's
  monotonic clock; the time is a `timer` that is not live, a polite region
  says the time left at 60, 30 and 10 seconds (`ANNOUNCE_AT`) and when the
  time is up, and reduced motion stops the ring's sweep.
  `view::lifecycle::trial_actions` draws it with Roll back now and the
  primary Confirm (#189).
- `view::secret`, the write-only secret field (F3): `secret_field` draws
  what is set now by reference (`SecretState`: set from a file, set and not
  shown, not set, or set and unreadable) and one radio group of Keep (Leave
  unset when not set), Set a value, Read from a file and Clear
  (`SecretChoice`), whose chosen control shows with no script; the value is
  a password input with `autocomplete="off"` the page never fills, and
  `SecretField::refuse_value` refuses a value where no secrets directory is
  set. `secret_state` draws the state alone for a read-only settings row (#190).
- The fan-out outcome (F5), `view::fanout::fanout`: the per-node state of
  one request that went to many. When a node failed, "Incomplete answer" comes
  first in words with what is missing, then the verdict (Complete, Incomplete
  or Failed) as a pill with the console's summary line, then one row per node
  with its state as a shape, a colour and a word, its protocol term, what it
  gave and the reason it failed. A node the request left out is drawn dashed
  and stays listed. At a phone's width each node is one line with its reason
  under it (#192).
- The replica line (F6): "Answered by replica gateway-0", one quiet line
  under the title of a reference page, through the new `replica` field of
  `view::template::settings::SettingsPage` and
  `view::template::about::AboutPage` (`view::replica::Replica`). No other
  page can draw it (#193).
- The nested sidebar section (F10): `widget::nav::Nest` and `NestItem` open
  the parent in view and its children under one entry of the navigation,
  such as an organisation and its nodes, each with its health dot as a shape
  named by its word. The entry is marked `aria-current="location"`, the
  child in view `page`. The rail leaves the section out from 48 rem up and
  the drawer keeps it (#194).
- The topology (F9), `view::topology::topology` with the island
  `TopologyKeys`: a federation drawn as a hub with its callers above it and
  its two sides, the start side mirrored, with the flow's (B9) Bézier edges,
  end marks and cards. A problem edge carries its mark and its label, a
  collapsed group one card whose edge carries the mark, and hovering or
  focusing a card raises its path from the hub with the pulse, paint only.
  The toolbar holds the Status menu, which is the legend and a `GET` form of
  the states, and the Map and List switch; the zoom cluster names each icon
  in a tooltip above it, with Collapse every group and a minimap. Zoom works
  by the cluster and by `+`, `-` and `0` on the canvas, the canvas pans by
  the pointer, "i" or Space on a card opens its quick look beside the
  canvas, and reduced motion stops the pulse. The List twin
  (`fui-topology-list`) draws the same graph and states as a tree with the
  same actions, walked by the arrows, at every width and in place of the map
  on a phone. With no WebAssembly every card, fold, quick look and view is a
  link and the zoom says it waits for the script (#195).
- The flow (B9) orders each lane to cross the fewest edges: barycentre
  sweeps over the lanes and swaps of neighbouring cards, deterministic,
  with the console's order breaking a tie, so a flow that crosses nothing
  keeps its order. A lane past `view::flow::FOLD_PAST` (eight) cards folds
  its quiet cards into one summary card with its counts by state, a link
  that unfolds it with or without WebAssembly, and the lane's fold link
  beside its title; a card in a problem state or on a problem edge is
  never folded, and the phone's list folds as the drawing does. Problem
  edges are drawn last, and where several labelled problem edges enter one
  card each label sits after its edge's start. The gallery's charts page
  shows a busy bridge of thirty callers and thirty upstreams, folded and
  unfolded (`?open=callers,upstreams`), and its federation page a
  federation of fifty organisations and two hundred nodes (`?large=50`) (#290).
- `table::view::data_table_wide`: a data table whose box scrolls sideways
  at every width, for a column set no console bounds such as a query's
  result set; its header gives up sticking from 64rem. A table with a code
  header (`Column::mono`) is drawn this way by `data_table` and
  `data_table_unstacked` too, and a code header breaks after a slash, a dot
  or an opening bracket (#293).
- `i18n::number`: `grouped` writes a count in the page's language
  ("312,000"), and `format_number` is the Fluent formatter the library sets
  on its catalogs, which a console sets in the `customise` of its own
  `static_loader!` (`bundle.set_formatter(Some(ferrohealth_ui::i18n::number::format_number))`).
  The marks are the catalog messages `number-group` and `number-decimal`,
  so a language brings its own (#293).
- The diagnostics list windows a long file and says when it is empty. With
  the new `FileFindings::all` link, a file of more than
  `view::diagnostics::WINDOW` (100) findings draws the first 100 and one
  quiet row, "100 of 2000 findings shown. Show all 2000", linking to the
  console's page of them all (`fui-diags-more`); a list or a file with no
  finding says "No errors or warnings." (`fui-diags-empty`) (#295).
- `view::timeline::timeline_window` draws the newest `Window::limit` events
  of a long history and a foot saying how many show of how many, with a
  plain link to the console's page of older events (`Window::older`).
  `view::scope::Request` reads the SMART App Launch 2.2.0 scopes that ask
  for no resource (`openid`, `fhirUser`, `launch`, `launch/<resource>`
  with its `?role=`, `online_access`, `offline_access`) and
  `ScopeGrant::read` reads any raw scope into its grant.
  `view::row_preview::RowValue::Null` draws a column with no value as
  `NULL`, apart from an empty text, which now reads "empty text" (#296).
- `view::coding::MoreCodes` and `more_codes`: the quiet last row of a list
  of codes the console's server pages, "25 of 4000 codes shown. Show the
  next 25", in a clause's codes and a coded node's value set (#297).
- `widget::combobox::searched_combobox` and `Remote`: a combobox over a
  console's own search, for a list too long to send to the browser whole;
  the combobox writes the words typed and shows the options the search
  answers (#301).
- The date field takes a date typed in the order the page's language writes
  one, from the catalog's new `date-order` message (`ymd`, `dmy` or `mdy`;
  English `ymd`), through `calendar::DateOrder`, `Day::parse_in` and
  `Moment::parse_in`, which a console uses to read the posted text (#301).
- `view::refusal::split_issues`, `ISSUES_OPEN` and `ISSUES_DRAWN`, and
  `widget::upload::decode` (#301).
- `i18n::number::Grouping`: the page's digit-group mark, resolved on the
  server and carried in an island's words, with `write` for a count the
  island learns in the browser (#304).
- `i18n::Word`: a library word with whether it fell back to English on a
  page in another language. Drawn as a text node it is wrapped in
  `<span lang="en">` when it fell back (WCAG 2.2 SC 3.1.2); written into an
  attribute it takes the page's language; in an island's props it travels as
  a plain string, or as `{"en": text}` when it fell back. `Word::lang` gives
  the `lang` of an element whose whole text is the word, such as an
  `<option>` (#313).

### Changed

- Breaking: `widget::menu::UserMenu::choices` is a `Vec<Choices>`, so the
  user menu holds one group per control the top bar shows from 40 rem up,
  such as a brand and the FHIR version. A console that passed
  `Some(choices)` passes `vec![choices]`, and `None` becomes `Vec::new()` (#172).
- Breaking: `widget::action_menu::Trigger` has a `Version` variant, the
  top bar's FHIR version control; a console that matches on the trigger
  adds an arm (#172).
- Breaking: `Tag::locked` takes the value's owner (F4), such as
  "Provisioned from mappings/observation.yml", and draws it as the tag's
  tooltip and after its word in its accessible name, so the lock and the
  dashed edge never stand alone. A console that called `.locked()` passes
  the owner it set with `.title(...)` to `.locked(...)` instead (#191).
- Breaking: `SettingsPage` and `AboutPage` take a `replica:
  Option<Replica>` field; pass `None` where the product has no replicas (#193).
- Breaking: `widget::nav::Sidebar` takes a `nest: Option<Nest>` field; pass
  `None` for a navigation with no nested section. `view::flow::FlowState`
  gains the public `tone` and `key` (#194).
- Breaking: the tree keeps its state as data the server can read. Each
  toggle is a link to `TreeNode::toggle_href` when the console gives one,
  so a tree folds with no WebAssembly, and `Tree::open_field` posts the
  open items as hidden fields of the form around it; a console that keeps
  them in its address watches `Tree::expanded` with `Effect::watch`.
  `TreeNode::children` is now `Children` (`None`, `Unread`, `Read`), the
  builder `expandable()` is `unread()`, `Tree::id` and `Tree::label` are
  `String`s, and `Tree` takes `kind`, `editing`, `open_field` and `words`.
  `TreeNode` gains the public fields `count`, `edits`, `inactive`, `mark`,
  `removed`, `rename`, `tag` and `toggle_href`; a console that builds a
  node by struct literal names them, or starts from `TreeNode::new`. The
  tree draws inside its `fui-concept-tree` box (#213).
- Breaking: `view::flow::FlowLane` takes `fold: Option<LaneFold>`, how the lane folds
  past `FOLD_PAST` cards; `None` keeps a lane that never folds. A struct
  literal that builds a lane names the new field (#290).
- `view::results::ResultsPane` takes `numeric`, whether the
  answer has a column the chart can draw. A console sets it from its result
  set. An answer with no row disables both exports and offers no chart, an
  answer with no numeric column offers no chart, and the pane says why in a
  quiet line (#293).
- Every number a message takes, and every count a list card, a panel
  section, a binder tab, a navigation entry or a count cell draws, is
  grouped in threes in the marks of the page's language ("1-25 of 312,000
  rows") and never breaks between its digits. A count past what a float
  holds exactly (`Arg::from(u64)`) is written grouped as text (#293).
- A table header wraps between its words and hyphenates a long word in the
  page's language; a heading and a field label wrap a long word (a German
  or Dutch compound) instead of widening the page (#293).
- Breaking: `view::chart::tick_label` takes the axis step as well,
  `tick_label(value, step)`, and prints the step's decimals; a console that
  printed a figure with it calls the new `view::chart::value_label`, which
  keeps six significant digits. `view::chart::MARKS` names the four point
  marks a series' palette index also picks (#294).
- `view::fanout::Verdict` has a fourth variant, `Unasked`, for a
  fan-out with no node asked; a console that matches on the verdict adds an
  arm. A legend swatch is an SVG of the series' mark (`svg.fui-swatch`), no
  longer a `span` (#294).
- Breaking: `island::CopyButton` takes `source: CopySource` in place of
  `text: String`. `CopySource::Text` carries a short text as before;
  `CopySource::Element` copies the text of an element the page draws, and
  `CopySource::Lines` the code lines of a viewer. The document pane copies
  its raw view and the code viewer its lines, so a large document reaches
  the page once per view and never as island data (a 0.5 MB bundle made a
  4.9 MB page). Line breaks are copied as the page holds them, a carriage
  return as a line feed. `widget::copy::copy_from` draws the button over a
  source; `copy_button` keeps its live text (#295).
- Breaking: `view::diagnostics::FileFindings` takes `all`, the console's
  page of every finding in the file, and `DiagnosticsWords` takes `more`,
  `more_all` and `none`. A console that builds either by struct literal
  names the new fields; `DiagnosticsWords::of_page()` fills the words (#295).
- Breaking: `view::scope::Grant` has a `Request(Request)` variant and
  `view::row_preview::RowValue` a `Null` variant, so a console that matches
  either whole adds the arm. A console that builds its grants with
  `Scope::parse` and refuses every error should move to `ScopeGrant::read`,
  which no longer marks `openid`, `fhirUser` or `launch/patient`
  malformed. The masked identifier's busy Reveal is `aria-disabled` where
  it was `disabled`, so it can hold the focus. `RowEnd::Kept` draws its
  reason as a visible line under the row and its lock is hidden from
  assistive technology, where the reason was the disabled button's name
  and tooltip (#296).
- Breaking: `widget::tree` addresses an item by its path key, the keys from the top
  to it (`path_key`, read back with `path_keys`): `Tree::expanded`,
  `Tree::selected`, `TreeData::open`, `TreeData::selected` and the open
  fields a tree posts hold path keys, and an item's key need only be unique
  among its siblings. The same concept under two parents (a SNOMED CT
  polyhierarchy) is two items, each with its own id, fold, selection and
  place in the keyboard's walk. A top item's path key is its key; a console
  that opens or selects a deeper item builds its path key (#297).
- `view::browser::match_list` takes the rows as `Vec<AnyView>`, and the
  `Match` struct is removed: the match the browser reads marks its link
  `aria-current="true"` (`Coding::current`), and its row takes the tint
  from that mark, where it took `aria-selected` on a plain list item,
  which WAI-ARIA 1.2 does not allow there (#297).
- `view::coding::Coding` has a `current` field (`Coding::current`), which
  puts `aria-current="true"` on its link; a struct literal names it (#297).
- `view::clause::Clause` takes `issues: Vec<ClauseIssue>` in place of
  `issue`: two or more broken invariants draw as a list, one issue each.
  `clause_codes` takes the more row (`Option<MoreCodes>`) after the codes,
  and `view::browser::PathNode` has a `more` field, which also sets the
  count in its value set's heading (#297).
- `widget::filter::table_filter` shows its count at once and says it from
  a status region of its own once the typing pauses (`SETTLE`, 600 ms), so
  assistive technology hears one count for a burst of keys (#297).
- Breaking: the confirmation dialog (`confirm_dialog`, `danger_dialog`),
  the popover form (`popover_form`) and the side panel (`SidePanel`) are
  native modal `<dialog>` elements with `closedby="any"`, opened by their
  trigger's `command="show-modal" commandfor=<id>` and closed by
  `command="close"`, in place of `popover="auto"` and `popovertarget`. The
  page behind is inert while one shows, the focus moves in and returns to
  the trigger, Escape and a press outside still close it, and all of it
  works with no WebAssembly (WAI-ARIA Authoring Practices, the dialog
  (modal) pattern; WCAG 2.2 SC 2.4.3 and SC 2.4.11). A console that closes
  one of these from its own code calls `HTMLDialogElement.close()`, or
  gives its confirming button `command="close" commandfor=<id>`, in place
  of `hidePopover()`. `Act::Dialog`, the Reset to defaults button and every
  trigger the library draws use the command; a trigger a console draws
  itself does the same (#298).
- Breaking: the user menu's popup holds who is signed in above its ARIA
  menu, which is now the inner element with the id `<id>-menu` that the
  button's `aria-controls` names; the popup keeps `<id>` for
  `popovertarget`. `widget::menu::menu_button` no longer draws
  `role="menu"`: its `items` carry the head and the menu, as `user_menu`
  draws them. The button carries no `aria-expanded` until its island runs (#298).
- Breaking: `widget::nav::Entry::href` and `widget::nav::Nest::under` are
  `Cow<'static, str>`, so a console passes a path under a base it reads at
  run time (`format!("{base}/files").into()`) as well as a literal
  (`"/files".into()`). A struct literal that set them to a `&'static str`
  adds `.into()` (#299).
- Breaking: `view::shell::Footer`'s `version` and `about` are
  `Cow<'static, str>`, and `view::shell::lockup` takes its `href` as
  `impl Into<Cow<'static, str>>`, so a console passes a base path or a
  version it learns at run time without leaking a string; a literal takes
  `.into()`. `Footer::release` borrows the footer (#299).
- Breaking: the phone's tab select (`widget::tabs::tab_select`, the
  `TabSelect` island) is named by the label around it and carries no id, so
  a page may hold several many-tab rows; a test or a console that found it
  by `#fui-tabs-select` finds it by `.fui-tabs-select select` (#299).
- The top bar's search answers Escape as the WAI-ARIA Authoring Practices'
  combobox does: an open answer closes and keeps the words, a closed one
  clears them, and the focus stays in the field either way (WCAG 2.2
  SC 2.4.3); it left the field for the page before (#299).
- Breaking: the navigation drawer is a native modal `<dialog>` with
  `closedby="any"` around the `<nav>` landmark, and the top bar's Menu
  button opens it with `command="show-modal" commandfor=<id>` in place of
  `popover="auto"` and `popovertarget`. The page behind is inert while it
  shows, the focus moves in and returns to the button, and Escape and a
  press outside close it, all with no WebAssembly (WAI-ARIA Authoring
  Practices, the dialog (modal) pattern; WCAG 2.2 SC 2.4.3 and SC 2.4.11).
  From 48 rem up the closed dialog is the column beside the page. The id a
  console passes as `Sidebar::id` is now the dialog's; a stylesheet or
  script of a console's own that selects `nav.fui-sidebar` or
  `.fui-sidebar:popover-open` selects `.fui-sidebar` and
  `.fui-sidebar[open]` (#300).
- `widget::nav::Nest` takes an `all` field, the words of the link
  to the parent's page ("All 300 nodes") that the section draws when it
  holds more than `NEST_SHOWN` (8) children (#300).
- A collapsed rail entry carries its words in a drawn tooltip in place of a
  `title` attribute (#300).
- Breaking: `ComboboxWords` carries `more`, `UploadWords` carries
  `read_one` and `read_lossy`, and `DateWords` carries `order`; a console
  that builds one by hand fills them, and `of_page()` does so. A refusal's
  outer `div` no longer carries `role="alert"`: its title does (#301).
- Breaking: `EditorWords`, `SizeWords` and `IdentifierWords` carry
  `grouping`, the page's `i18n::number::Grouping`. A console that builds
  one by struct literal sets it with `Grouping::of_page()`; each type's
  `of_page()` does so (#304).
- Breaking: the library's own words are `i18n::Word` in place of `String`.
  Every `*Words` struct holds `Word` fields; `Phrase::new` takes
  `impl Into<Word>` and `Phrase::fill` returns a `Word`. The methods that
  return a library word return a `Word`: `label` of `Theme`, `Density`,
  `Motion`, `Hints` and `Step`, `Variant::name`, `FlowState::word`,
  `Severity::word`, `Relationship::words`, the scope's `Compartment::words`,
  `Operation::words`, `Request::words` and `ScopeError::expected`,
  `Status::line`, `refusal_title`, `at_time_label`, `Noun::word`, `empty`
  and `opening`, `range_label` and `range_of` of the table windows,
  `partial_note`, `SizeWords::size` and `Refused::sentence` (#313).
- Breaking: the fields a library word can fill are `Word`:
  `Header::title` and `subtitle`, `Tab::label`, `TabChoice::words`,
  `Column::label`, `PreferenceRow::label` and `description`, the words of
  `SelectPreference::choices` and `SegmentedForm::choices`,
  `Confirmation::title` and `message`, `Modal::title` (a `Signal<Word>`),
  `action_menu::Choice::label` and `note`, `Listbox::value`,
  `Trigger::More` and `Trigger::Select`, `ViewChoice::words`, the `words`
  and `link` of `MoreCodes` and the tree's `More`, `RangePreset::words` and
  `Grant::Refused`. `From<String>`, `From<&str>` and
  `From<Cow<'static, str>>` make a console's own words a `Word` in the
  page's language (#313).
- Breaking: these take `impl Into<Word>`: `Tab::new`, `Column::text`,
  `button::label`, `status::badge`, `kbd::shortcut_keys`,
  `surface::definitions`, `surface::page_header`, the form's `Field::new`,
  `help`, `help_if`, `field`, `keyed_field`, `check`, `keyed_check`,
  `check_field` and `keyed_check_field`, and `copy_button` and `copy_from`.
  The islands `CopyButton` (`label`), `NavDrawer` (`close`),
  `UserMenuButton` and `MenuButton` (`cancel`), `TabSelect` (`go`) and
  `PreferenceForm` (`save`) take a `Word`, `ChartLegend`'s `gap` is an
  `Option<Word>`, and so are the matching parameters of `nav::drawer`,
  `chart::legend`, the two `menu_button`s, `tab_select` and
  `preference_form` (#313).

### Removed

- Breaking: `widget::tree::tree_view_collapsing` and the tree's
  `on_expand` and `actions` callbacks. No callback crosses an island: a
  level is read by the server behind the toggle's link, and a row's
  actions are `TreeNode::edits` (#213).

### Fixed

- A line chart's figures table no longer widens the page on a phone: the
  visually hidden box is now a block around the table, since a table ignores
  the 1 px width that hides it (#182).
- A focused segment's ring shows on a pressed segment in every brand, light
  and dark. The ring was the focus colour, which is the accent every pressed
  segment is filled with, so it measured 1:1 on the fill. A pressed segment
  now draws a band of the accent ink inside the focus colour, at least 5.47:1
  against the fill, and the focus colour stays on the segment's edge against
  the surface. A focused segment that is not pressed draws its ring on its
  edge, as a focused radio segment already did (#286).
- The topology (F9) stays one frame tall at the size of a real federation:
  its canvas scrolls inside a frame of at most 70vh at any zoom, where it
  did so only while zoomed, and its minimap keeps inside 168 by 300 px,
  where a tall federation drew it taller than the page (#290).
- A flow's or a topology's card opens its page under the mouse: hovering a
  card faded the edges into a layer above the cards, so the press landed
  on the card and the release on the edges, and the click went nowhere. A
  flow lane's fold link takes the pointer too (#290).
- A 14-column query result no longer scrolls the whole page sideways from
  64rem: its table scrolls inside its card (WCAG 2.2 SC 1.4.10) (#293).
- An open pager whose page is past the end says "Page 9 is past the last
  row." beside the Previous it offers, where it said "No rows" (#293).
- A version history with no version says "No version has been committed."
  in its table, on a phone too, where it drew an empty table (#293).
- The line chart at its edges: a value axis under 0.1 prints the decimals
  of its step (0.0002, 0.0004), never 0.0 on every label; X labels a narrow
  chart has no room for are left out by container queries the server
  chooses from each label's length, so dates never overlap at 320 px or at
  1440 px; every series takes a point mark of its own beside its colour (a
  circle, a square, a diamond or a triangle, 24 distinct pairs), in the plot
  and in its legend swatch, so a seventh series no longer repeats the
  first by colour alone (WCAG 2.2 SC 1.4.1); a long series is drawn with at
  most 60 marks and its line thinned to the lowest and highest point of
  each of 500 columns, so 5,000 points draw no 15,000 circles; and the
  figures table finds its cells by a binary search and prints each value to
  six significant digits (#294).
- The score breakdown (R16) takes any number of thresholds: the first band
  possible, the last certain, every band between probable. Its scale runs
  from a round step under the lowest threshold or score, past 1 when a
  score lies there, threshold labels too close for one row take more rows,
  a field's long word wraps in its column, and a score that is not a number
  reads "no value" with no point drawn (#294).
- The run report (B7) draws a part with any count at least 1.5 % wide with
  a stroke of its own colour, so 3 refusals among 2,000,000 still show (#294).
- The fan-out outcome (F5) says "No node asked" in the neutral tone when
  every node was left out or there is none, never a green Complete, and a
  long node name wraps after its dots inside its row (#294).
- Every mark keeps its meaning under forced colours (family rule 14): the
  state shapes of the health dot and the status pill, the switch's knob and
  its on state, the select's and the combobox's chevron, a pressed segment
  and the theme choice of the user menu, the current page, tab and list
  item, a current row's edge, the worklist row's focus ring, a clause's
  edge, the divider and the timeline's rail, the chart's frame and labels,
  and the swatches of a chart and of the run report are drawn again in
  system colours, in one forced colours block of the stylesheet (#294).
- The code viewer draws every finding: one on line 0 sits above the first
  line and one past the last line under the last, where before neither was
  drawn. Its findings are indexed by line once, so a file of 20000 lines
  with a finding on many of them draws in time linear in both (#295).
- A finding's long code, message, mapping or path breaks at 320 px rather
  than widen the page (WCAG 2.2 SC 1.4.10) (#295).
- The code editor redraws only the lines an edit changes. Each edit
  tokenizes the text once and patches the layer's lines in place, where it
  wrote the whole layer again; the lines after an added or removed line
  only change their ids, and a caret move or a new check keeps the
  highlighting and changes only the marks. A keystroke in a 10000-line
  draft went from about 170 ms to about 75 ms in Chrome, most of which is
  now the browser's own layout of the text area (#295).
- The record parts hold at their edges. Below 64 rem the stacked record
  compare offers every other record: up to five as the radio choice, whose
  long titles now wrap inside their segments, and past five each record as
  a native disclosure (`<details name>`), one open at a time, so no record
  is out of reach on a phone; a `shown` past the end reads the last record.
  A compare with no other record says "no other record to compare" instead
  of "every field agrees". The masked identifier hands the keyboard focus
  to the button of the face that takes the pressed one's place (Reveal to
  its busy twin, then Hide, and Hide, a refusal or the end of the countdown
  back to Reveal), so the focus never drops to the page (WCAG 2.2 SC
  2.4.3), and a long revealed value breaks inside its row. A tag (R2) stays
  on one line while it fits and wraps inside its box when it holds a
  canonical URL or an OID, never widening the page (SC 1.4.10). The
  timeline orders its events by the moment they name, so `10:00+02:00`
  sorts before `09:30Z`. The scope grant shows the search constraint that
  narrows a grant, and its raw scope breaks after a slash, a dot or a query
  mark. The condition builder keeps a chosen value its select does not
  list chosen, said to be not in the list, where it used to post the first
  option, and a condition row stacks its fields in a group too narrow for
  them side by side, where the builder widened the page at 768 px with a
  classic scrollbar (#307). The row preview wraps a long value inside its cell where the
  card's edge cut it off. The clinical reading indents nested clusters at
  most four levels deep and wraps a long label or value; a reader hears a
  heading's title and its node id as two phrases and a quantity's UCUM
  code after its unit. A repeatable row the list keeps says why in a
  visible line under it (#296).
- A long code (more than 24 characters, such as a versioned uid or a UUID)
  breaks after a colon, a dot, a hyphen, a slash or a vertical bar, and a
  canonical breaks inside a long piece between two slashes the same way, so
  a `urn:uuid:` or `urn:oid:` canonical and a versioned uid stay inside a
  320 px page; an ordinary code still never breaks (#297).
- A concept tree keeps the width of a long name beside its code and count,
  where the name ran under them; every tree now scrolls sideways instead (#297).
- An invariant's number in a clause issue (`vsd-3`) never breaks at its
  hyphen, and a long match in a search's list breaks inside its row (#297).
- Every menu, the phone's bottom sheet, the user menu and the listbox stay
  inside the viewport at 400% zoom or with many items: past its height each
  scrolls inside itself, and a sheet keeps its Cancel in sight at its foot
  (WCAG 2.2 SC 1.4.10). A menu item's words, a listbox option and the user
  menu's e-mail wrap past the popup's widest, so nothing is clipped. The
  confirmation dialog scrolls past the viewport's height and breaks a long
  word in its message (#298).
- The keyboard reaches a disabled menu item, which is announced disabled and
  takes no press (WAI-ARIA Authoring Practices, the menu pattern) (#298).
- The user menu's button no longer says it is closed before its island
  runs, when with no WebAssembly the menu can be open (#298).
- The popover form in a browser without CSS anchor positioning is centred
  in the viewport; it opened below the viewport's foot (#298).
- Binder tabs that outrun their sheet from 40 rem scroll inside their row
  under a fade at each edge with more tabs behind it; they widened the page
  and, through `scroll-initial-target`, opened it scrolled sideways and
  down to the current tab (WCAG 2.2 SC 1.4.10). The current tab is now the
  first scroll target only of a row that is its own scroll container (#299).
- A long page title (an archetype id) or crumb (a canonical URL) breaks
  inside the header, the title taking a line of its own before it shrinks;
  More wraps under a long primary action; a stat card's mark wraps inside a
  narrow card. Each widened the page at 320 px or beside the sidebar (the
  gallery's overview measured 1156 px in 1137 at 1152) (#299).
- The search answer ends above the window's foot and scrolls inside
  itself, a long name wrapping in its column; a search squeezed by a long
  operator's name gives up the submit's word, then the `/` cap, keeping
  room for about a dozen characters, where it kept one (#299).
- The docked decision bar stops sticking in a window under 30 rem high
  (under 40 rem on a phone), where it covered most of the page at 400%
  zoom, and keeps its scroll padding from 30 rem high (WCAG 2.2 SC 2.4.11) (#299).
- The toast region shows three toasts at most (`widget::toast::SHOWN_MAX`),
  the oldest that is no refusal and has no action leaving first, stays above the page's
  decision bar, and scrolls past the viewport's height, so toasts never
  cover the decisions (WCAG 2.2 SC 2.4.11). A toast without an action stays
  while the pointer or the focus is on the region (SC 2.2.1), a long file
  name in a toast wraps, and with no WebAssembly a flash toast closes by its
  Dismiss, a native disclosure (#300).
- The nested sidebar section draws eight children at most, the page in view
  always among them, and a link to all; a child's full label is its
  tooltip. A parent with 300 nodes ran about 9000 px and buried the groups
  under it (#300).
- A long word in a sidebar entry wraps; it cut the label and scrolled the
  sidebar sideways (SC 1.4.10) (#300).
- A rail count past 99 shows as "99+" on its badge, which covered the glyph
  and left the window at four digits; the full sidebar still shows the
  number whole (#300).
- The collapsed rail shows an entry's words beside it on keyboard focus as
  well as on hover, until Escape (SC 1.4.13), and keeps scrolling (#300).
- A segmented control's long label wraps inside its segment instead of
  being clipped by the group, and a row that wraps or stacks keeps a
  hairline between every two segments (#301).
- The upload field shows its file input's focus ring on the chooser, takes
  a file of any kind when it names no extension, says when it read only the
  first of several files dropped at once, and warns when a file was not
  UTF-8 text (#301).
- A refusal with thousands of issues shows the 20 gravest, folds the rest up
  to 200 under a native summary and counts any past that, and only its
  title is the live region (#301).
- Clearing the enhanced combobox's words clears its value, as the native
  select's empty choice does, and its list draws at most 100 options and
  says when more match (#301).
- The picker field's select has a visible label, and arrowing through a
  closed select that changes at each key no longer moves the focus to the
  input when it reaches Another (#301).
- The copy button says the browser refused on a page served over plain
  HTTP, where the clipboard is undefined, instead of showing nothing (#301).
- The commit-confirm countdown counts a system sleep the monotonic clock
  missed, by taking the wall clock's elapsed time when it has run further (#301).
- A refusal's own words that fall back to English on a page in another
  language are marked `lang="en"` (#301).
- The code editor's line count and cursor place, the upload field's file
  sizes and the masked identifier's countdown group their digits in the
  page's language in the browser, as the server writes a number:
  `EditorWords`, `SizeWords` and `IdentifierWords` carry the page's
  `i18n::number::Grouping`, which their island writes each count with (#304).
- Every library word that falls back to English on a page in another
  language is marked `lang="en"` where it is drawn, on the server and in
  the islands, and a word the page's catalog holds is drawn as before.
  A word joined into a longer sentence or into a chart's drawn text takes
  the page's language (#313).
- On a phone the topology's List twin keeps a long organisation name whole
  beside its state, which wraps instead of running over the name, and a
  node's note takes a line of its own. A stacked version history marks the
  version being read on its whole row, its bar at the row's edge and clear
  of the text, in forced colours too. The version history's marking no
  longer reaches the worklist's focused row, which drew sunken patches over
  its tint and a second bar beside its checkbox (#317).

## [0.3.1] - 2026-10-22

### Added

- The record compare (`view::compare`): several records side by side, one
  column per record and one row per field, each value marked agree, differ
  or absent against the leading record as a glyph and a word, never colour
  alone. `compare` draws the table at every width with fixed columns, so a
  revealed value moves nothing, scrolling sideways inside its card with the
  field and the leading record held at the start; `compare_stacked` reads one
  record at a time under a radio choice that works with no WebAssembly, every
  mark at the end of its line; `compare_pair` switches between them at 64 rem.
  The differing rows come first when asked, and a caption whose fields all
  agree says so. A cell's value is a `ViewFn`, so it can be an island such as
  a masked identifier, or a record's identifiers in any number and order (#137).
- `view::score::{score_breakdown, score_bar}`, the score breakdown (R16):
  one signed bar per compared field (`Contribution`), its plain words first
  and the comparator as a quiet hint, the total weight, the score with its
  grade, and the thresholds (`Threshold`) on one scale, each label centred
  on its mark and the start kept on a phone. The scale is an image named by
  the score, the grade, the thresholds and every contribution;
  `Score::tone` gives the grade's tone, and `score_bar` is the compact form
  for a table cell (#138).
- The timeline (`view::timeline`): events along a rail, newest first, as a
  labelled ordered list. Each event's dot takes its tone and is hidden from
  assistive technology, its kind is a word with a capital and its raw event
  code a quiet hint, its time is absolute in a `<time>`, its actor, its
  masked identifiers and its reason follow, and its body opens in a native
  `<details>`. The list has no cap in height (#139).
- The masked identifier (`widget::identifier`), for any national or local
  identifier system: the domain is data (`Domain`, a label and the system's
  URI), and `Masked::new` masks on the server with no assumption of length,
  characters, format or check digit, keeping at most `KEEP` trailing
  characters and never more than half, behind a fixed run of dots that never
  tells the length. `identifier` draws it masked with Reveal, which runs the
  console's audited round trip; the console answers in the widget's `Reveal`
  signal, and the widget counts the seconds down, drops the value at zero or
  on Hide, and offers Copy only while it shows. The masked, revealed and busy
  faces share one reserved slot, so nothing around it moves; a refusal is a
  line in words on its row; the countdown speaks through a polite live region
  at 10 and 0 seconds. A revealed value is a `Revealed`, which never
  serializes and whose `Debug` withholds it. Before hydration Reveal is a link
  to the console's own reveal page. `identifier_static` and `identifier_list`
  are the listing forms (#140).
- The Lucide icons `equal-not`, `eye-off` and `minus` (`Glyph::EQUAL_NOT`,
  `Glyph::EYE_OFF`, `Glyph::MINUS`) (#140).
- The work-list (`widget::worklist`, the ready island `island::WorkList`): a
  data table a steward walks with the keyboard. The server draws its rows
  with `worklist::row`, the first focused, and the foot with
  `worklist_foot`, which stays in view below the rows inside the card. Once
  hydrated the table is a grid with one tab stop on the focused row, which
  carries `aria-current`: `j` and Down, `k` and Up, Home and End move it and
  scroll it clear of the sticky header and the foot, and Enter opens it (the
  ready island follows the row's link; `worklist` takes the console's
  `on_open`). One focus outline goes round the whole row, its row menu
  included. With no WebAssembly the table stays a table whose rows open by
  their links (#142).
- `docs/consoles.md`, the guide every console repository points its agents
  to: what the library owns and what a console owns, which FerroUI document
  governs each part, the version rule, the rollout, and where to ask for a
  missing component (#161).
- The code editor (B2), `widget::editor::editor_panel` over a `Panel`: F8's
  toolbar with what the text is, its line count and the console's tools; the
  editor with its gutter marks; and the status line with the cursor's line and
  column, the console's facts and the unsaved dot (`Opened` says what the
  draft is measured against), its words as props (`EditorWords`). While the
  cursor is on a marked line, the line is current and a polite live region
  says so. `editor_split` puts it beside its diagnostics, stacked under 64rem,
  and `editor_diagnostics` draws them in step with the cursor: a row's line
  link moves the caret to its line and column and focuses the editor.
  `caret_at`, `offset_of`, `mark_kind` and `current_line` are the pure parts (#162).
- The diagnostics list (B3), `view::diagnostics`: one row per `Finding` with
  its gravity in shape, colour and word, its code, its line and column as a
  link to the line, its message and its mapping, errors first;
  `diagnostics` for one file and `diagnostics_by_file` grouped by file under
  each file's name and counts, the files with an error first. The row of the
  current line takes `is-current` and `aria-current` (#163) (#162).
- Serde for `view::code::Gravity`, so an island can take findings as props (#162).
- `view::load_report`, the load report (B4): a loaded set's outcome or an
  operation's verdict in one line, its glyph, title, counts and at most one
  action, in the ok, warn and bad outcomes (`Outcome`), the bad one on the
  bad ground. It is a status message when it appears after an action
  (`Arrival::AfterAction`) and plain content when the page loads with it (#164).
- `view::transform`, the transform pair (B5): the options row
  (`transform_options`, plain or `Density::Dense`) with Run at its end
  (`run_button`, ready, running or off with its reason), the input and
  answer panes with their sticky heads and the answer's state
  (`transform_pair`, `TransformPane`, `AnswerState`), the answer before the
  first run, running, answered and refused (`Answer`), the request an
  answer came from with Copy and the raw answer (`request`), and a group of
  check boxes as one field (`field_checks`) (#165).
- `Field::holds_id`, an options row's id field (`is-id`), wide enough for a
  whole uuid or versioned object uid (#165).
- `view::row_preview`, the OMOP row preview (B6): the rows one input becomes,
  table by table, each table's name with its row count, a hairline between
  rows, and an unresolved concept as `0` with the warn mark and its words
  (`RowValue::ConceptZero`). On a phone each column stacks over its value (#166).
- `view::funnel::{funnel, bar_list}`, the ETL run report (B7): one stacked
  bar of where the inputs went (`Funnel`, `FunnelPart`), an SVG image named
  by the caller's sentence, with a legend of big tabular figures, and the
  rows per table as a bar list (`Bar`) with the numbers right-aligned and
  the track under the name on a phone. A part with a count of zero keeps its
  legend entry (#167).
- `view::hl7v2`, the HL7 v2 message viewer (B8): the message raw
  (`hl7v2_raw`), never wrapped and scrolling under the scroll cue, and
  parsed (`hl7v2_segments`), one block per segment with its fields by
  position, name and value, the names dropped on a phone (#168).
- `highlight::Language::Hl7v2`, the HL7 v2 highlighter: each segment id and
  the five separators the message's header declares, total over any input (#168).
- `view::flow::flow`, the flow (B9): F9's cards and edges laid out as a
  pipeline of lanes (`Flow`, `FlowLane`, `FlowCard`, `FlowEdge`), each
  state (`FlowState`) a line style, a colour and on a problem edge one end
  mark and a label, with one quiet foot line. The edges are an SVG image
  whose description names each problem edge; the cards over them are links,
  and hovering or focusing one shows its path with the pulse from the
  stylesheet alone, still under reduced motion. On a phone the list form
  (`fui-pipeline-list`) takes its place (#169).

### Changed

- A data table's checkbox column (`Role::Check`) draws a select-all checkbox
  in its header, hidden until a work-list island shows it; the work-list then
  selects every row or none from it, shows the mixed state while some rows are
  selected, and marks each selected row `aria-selected` (#142).
- The highlighted editor's text area carries `aria-invalid="true"` while an
  error marks one of its lines, and its frame takes the bad tone; a code
  editor inside a panel section runs edge to edge when it is drawn as an
  island too (#162).
- `highlight::Language` gains `Hl7v2` (`Language::ALL` holds six) and
  `highlight::Kind` gains `Segment` and `Separator`, so a console's
  exhaustive match over either needs the new arms (#168).
- `DatePicker` and `DateRangePicker` are lazy islands (`#[island(lazy)]`): a
  console that builds with `cargo leptos build --split` loads their code
  from chunks of its own on a page that draws them, which keeps 72 KB of
  brotli-compressed WebAssembly off every other page. Without `--split`
  nothing changes. The gallery builds with `--split`, makes its mapping
  editors, masked identifiers and concept tree lazy too, and holds its
  entry WebAssembly to the budget, with each lazy island's chunks reported
  beside it (#262).
- Breaking: `widget::editor::editor_diagnostics` takes the list's words as
  a `view::diagnostics::DiagnosticsWords` (`DiagnosticsWords::of_page()`
  on the server), and `widget::identifier::IdentifierWords` gains `masked`,
  the words a mask is heard as (#262).

### Fixed

- An unreachable card in the topology and the flow draws its cross: the dot
  (`fui-tdot-cross`) was filled where its two strokes needed a stroke (#169).
- The editor's text area keeps a draft that opens with a line break whole in
  the server HTML, and the module docs say the server writes the text, so a
  page whose WebAssembly has not loaded posts the draft unchanged; the
  gallery's no-WebAssembly journey posts the code editor's form and reads the
  draft back from the server (#205).
- The masked identifier and the code editor's diagnostics list no longer
  look their words up in the browser, which had linked the Fluent loader and
  the whole catalog into every console's WebAssembly again (50 KB
  brotli-compressed in the gallery) (#262).
- A diagnostics list's file head wraps its counts under the file name when
  both do not fit, so the list no longer pushes a 320-pixel page sideways (#267).
- A diagnostics file head with a link holds the link's cover to itself, so it no longer lies over controls outside the list (#271).
- The HL7 v2 viewer keeps a field position on one line on a phone, where `MSH-10` broke at its hyphen (#272).
- Following a diagnostic's line link keeps the code editor's gutter in view; the editor scrolled sideways by the gutter's width before (#273).
- A work-list row's focus ring is whole on a wide screen, its checkbox and menu cells included; the current row's tint covered the outline there (#275).
- A transform pair's input pane keeps the editor's ground to the pair's foot beside a taller answer, where a white band showed under the editor (#276).

## [0.3.0] - 2026-10-22

### Added

- Binder tabs joined to their panel (R3, #152): `view::tabs::binder(label,
  tabs, current, panel)` draws each `Tab` as a link to its own route, the
  current one `aria-current="page"` and raised into the panel, with an
  optional count whose words name what it counts (`Tab::count`) and the
  draft badge (`Tab::draft`, `fui-tab-draft`). Past five tabs a phone shows
  the sections as a select in a GET form with a Go button, the island
  `island::TabSelect`; choosing an option changes nothing until Go is
  pressed (WCAG 2.2 SC 3.2.2, #239). The select takes the row's place only once
  it works; until then the row of links serves. A
  tab keyed `keyboard` leaves the row below 40 rem. The keyboard tabs of
  `widget::tabs::tab_list` are binder tabs too, their panels the binder
  panel.
- Toasts with a title, a line and an action (R17, #156):
  `widget::toast::Toast` with `ToastAction::Link` or `ToastAction::Run`. A
  toast of the bad tone is `role="alert"`; a toast with an action stays
  until it is dismissed (WCAG 2.2 SC 2.2.1); its Dismiss is absent until
  the page hydrates. The page keeps its foot and footer clear of the region
  while a toast shows.
- The anchored popover form (R17 anchored, #156):
  `view::popover::popover_form`, a native `popover` placed 4 px under its
  trigger and aligned to its end edge by CSS anchor positioning, and a
  bottom sheet over a scrim below 40 rem. The browser closes it on Escape
  and returns the focus to the trigger.
- Every widget's words as props (#219): `CopyWords`, `ModalWords`,
  `HelpWords`, `SidePanelWords`, `ComboboxWords`, `ChartWords`,
  `view::chart::FiguresWords`, `UploadWords`, `SizeWords` and
  `view::time::TimeWords` (with `timestamp_in`), each built on the server
  by its `of_page()`, and `i18n::Phrase` for a sentence around a value the browser
  learns. No island looks a word up, so the browser bundle carries no
  Fluent loader and no catalog: the gallery's brotli-compressed
  WebAssembly falls from 402593 bytes to 365376.
- The interaction contract every component follows, written into
  `design/COMPONENTS.md` § Interaction and the crate's `CLAUDE.md` (#196):
  native elements first, behaviour in Rust compiled to WebAssembly, every
  page working without the WebAssembly, and no hand-written JavaScript.
- Leptos islands mode, the one rendering mode the library supports (#196).
  The server draws every page whole and the browser hydrates only its
  islands. The gallery's brotli-compressed WebAssembly falls from 685903
  bytes hydrated to 427449, and the data its server components read no
  longer reaches the page as hydration state. `ferrohealth_ui::island`
  holds the ready islands: `ToastRegion`, `NavDrawer` and `KeyboardTabs`,
  each taking its words as props. The stylesheet gives `leptos-island` and
  `leptos-children` `display: contents`. A console turns on the `leptos`
  feature `islands`, writes `<HydrationScripts options islands=true/>`, and
  calls `leptos::mount::hydrate_islands()` from a browser entry that names
  this crate.
- `widget::toast::raise(toast)`, which any island calls to show a
  toast, and `widget::toast::Flash` and `RegionWords`: the region draws the
  flash toasts the server hands it, so a form post's outcome shows with no
  WebAssembly.
- `widget::tabs::tab_panels` and `tab_list`: the panels are drawn on the
  server, every one, and stay out of the WebAssembly.
- `widget::nav::marks`, `widget::nav::drawer`, and the `current` path
  `sidebar` takes to mark the page.
- The gallery's no-WebAssembly journey (`tests/it/no_wasm.rs`): every page
  loaded with the bundle refused keeps its content, and its links, forms
  and popovers still work.
- The FerroPIX brand, Plum and Iron: `style::Brand::Pix` and its theme file
  `themes/ferropix.css`, plum `#6b21a8` in light and `#c084fc` in dark with
  dark ink on the accent, and the two preview accents
  (`--preview-accent-light`, `--preview-accent-dark`) the theme previews in
  Your preferences draw (#143). `tests/it/contrast.rs` holds every pairing
  of the brand to WCAG 2.2 AA in both themes, and the gallery shows every
  page in the brand, light and dark. This breaks the public API: a console
  that matches on `Brand` exhaustively adds the `Pix` arm, and
  `Brand::ALL` is a `[Brand; 5]`.
- The Leptos rules every console follows, from a full read of the Leptos
  book checked against the Leptos 0.8 source (#204). `.claude/rules/leptos-ui.md`
  §14 states each rule with its citation and what enforces it, and §15 is
  the islands rules: when a part is an island, its serializable props, its
  server children, context between islands, words as props, the toast
  region, what reaches the page, forms through `ActionForm` in a server
  component, and the no-WebAssembly floor. Two rules change for a console:
  a `<textarea>` now carries its escaped text in the server HTML, and a
  root `SharedValue` locale no longer reaches an island.
  `scripts/checks/leptos-rules.sh` enforces the mechanical part in CI, and
  the skills `/new-component` and `/leptos-check` walk a component and a
  review against the rules.

- The page templates as server views (#154), `view::template`: T1
  `list::list_page` with its card and header row (`list_card`, the count
  muted after a middle dot), the summary strip and the toolbar only past
  ten items (the toolbar also for a list with no stated end), the active
  filter chips with "Showing 7 of 21" and Clear filters as links (R14), and
  `list_empty`, one quiet line with the title row's action as a text link;
  T2 `detail::detail_page` and `detail::stepper`, previous and next of a
  queue as links (R5); T3 `settings::settings_page` over a `Source`: a
  file-only configuration with its one quiet line ("Set in … · read at
  start · How configuration works"), no mode badge and no Next start
  marker, or a managed one with its mode badge, revision line
  (`revision_line`), panel intro and actions; `setting_row`, each
  configuration key a labelled copy button (R15, `island::KeyHint`); and
  `about::about_page`, the Versions and Manufacturer cards and the
  self-description capped at 24 rem with Copy and Download.
- The error page (#187), `view::error_page::error_page` (R20): the status in
  words before its number ("Not allowed · 403"), the title naming the
  error, the body, a quiet hint, the ways on as links, and the time and
  request id in a closed disclosure, in the shell or in the centred layout.
- `view::shell::Footer::release`, the product and its version in the one
  form the footer and the About page both write ("FerroBRIDGE Console
  0.0.12"); `view::code::self_description`; `widget::copy::key_hint` and
  `KeyHintWords`; the Lucide `ban` glyph.
- The classes `fui-key-hint`, `fui-list-count`, `fui-revision-compact` and
  `fui-self-description`, lifted from the mockups' scaffolding into the
  crate's stylesheet and named in `design/COMPONENTS.md`.
- The gallery's Page templates group: every template, the 403 and 404 in
  the shell and the 500 in the centred layout, in every brand.
- The user menu (F13, #151): `widget::menu::user_menu(UserMenu)` draws the
  operator's button and an ARIA menu with who is signed in, Your
  preferences, the theme as three `menuitemradio` choices, Keyboard
  shortcuts with its key (from 40 rem up), and Sign out last after a
  separator. It is a popover the button opens with no script; the theme
  and Sign out are plain form posts, and a group the top bar shows from
  40 rem up (`Choices`, such as a FHIR version) moves into it below that.
  The island `island::UserMenuButton` adds the menu button pattern: the
  focus moves to the first item, Up and Down walk the items, Home and End
  go to the ends, Tab and Escape close it and the focus returns to the
  button. Below 40 rem it opens as a bottom sheet with Cancel.
- The top bar's status pill (F12, #151): `view::status::status_pill` with
  `Reach` (up, degraded, unknown, unreachable, refused, not probed), a
  shape, a colour and a word, the version only in its tooltip. Below
  64 rem it keeps its shape and its words stay its accessible name and its
  tooltip.
- The sidebar's rail (#151): `widget::nav::Sidebar` takes a `Rail`, and the
  navigation folds to its glyphs from 48 rem up, each entry named by its
  tooltip and its accessible name. Collapse and Expand at its foot post
  `sidebar=rail` or `sidebar=full` to the console's route, which keeps the
  choice and draws the next page folded or not. An entry carries an honest
  count (R12, `Count`): a number read as its whole phrase, nothing at zero,
  and a muted dash with the reason when none is known.
- The sign-in page in every state (R19, #186): `view::sign_in::State` is
  ready, redirecting, signed out, expired, full, unavailable, refused,
  rejected, provider unreachable, upstream unreachable and locked, each
  with its notice in its tone as a status or an alert, a muted detail and
  an optional Try again. The provider's action is busy while redirecting
  and disabled while sign-in is unavailable, and the basic form is drawn
  only when the console passes one, with no placeholder.
- `view::shell::lockup_plain`, the lockup as plain text for a page shown
  before sign-in, and `Shell::dock`, the foot of the main area kept for the
  docked bar (R4).
- The Lucide glyphs `keyboard`, `panel-left-close` and `panel-left-open`.

- The table's cell helpers and a stated order (#155): `title_cell`,
  `row_link`, `cell`, `count_cell`, `menu_cell`, `check_cell` and
  `group_row` write each cell's label and its `Role` (`Column::role`), so a
  row takes the design's phone form; `EXCLUDED` mutes an excluded row.
  `Sorting::stated` puts an order of several keys on the leading column's
  header, and `view::template::list::ListCard::order` states it in the
  card's header row beside the count. The Lucide
  `ellipsis` glyph (`Glyph::ELLIPSIS`) draws the row menu's ⋯.
- The find field as a form the server answers (#211):
  `table::filter::find_form` is the toolbar's find field, a `GET` form whose
  words land in the query, or a `POST` form when the words can be clinical.
  It carries the rest of the listing's query (`table::model::fields`) and
  works with no WebAssembly.
- The `POST` pager (#215): `table::pager::post_pager` and `post_open_pager`
  page a listing that keeps its position out of the URL through a form whose
  buttons carry `page` and `size` to the console's route (`PostTo`), so the
  position never reaches a URL or the history and no row is an island prop.
- Panel sections (B10, #159): `view::surface::panel_sections` and
  `panel_section` draw a tab panel's parts, each a heading row (a
  level-three title, a count after a middle dot, actions) over its body,
  with a hairline between parts and no card inside the panel. A table, a
  diagnostics list or a code viewer in a section runs edge to edge.
  `panel_sections_card` holds the sections on a page with no tabs,
  `source_split` puts a file beside its diagnostics (stacked under 64 rem),
  and `section_intro` and `add_row` are the editor parts that sit in a
  section. The gallery's layouts page shows a panel of three sections.
- The code viewer (B1, #160): `view::code::code_viewer`, `code_card` and
  `code_section`, over `Code`, `CodeHead`, `Digest`, `LineDiagnostic` and
  `Gravity`. The head names the path as its title in the prose font,
  breaking only after a slash, with its tags, the line count and the short
  digest; the lines are numbered, a current line takes `is-current`, and
  each diagnostic sits under its line in the bad or warn tone and stays in
  view while the line scrolls. In a panel section the section's heading
  row is the viewer's one head. `highlight::line_html` writes one line's
  tokens. Copy joins the head with the copy island (#220).
- The scroll cue is one class, `fui-scroll-cue`: a thin scrollbar and a
  soft shadow on the clipped edge that goes at the end. The code viewer and
  About's self-description card take it.
- The document pane works without WebAssembly (#212):
  `view::document::document_pane` over `Document` draws the format choice
  as links and every view (highlighted in the code viewer, raw, and the
  console's rendered view) on the server, each reached by its link. The
  island `island::DocumentViews` (props `Serialize + DeserializeOwned`,
  the views as its server children) switches them in place once it has
  hydrated; `widget::document::DocumentView`, `ViewChoice` and
  `view_switch` are its parts.
- `widget::focus::focus_on_open`, the one focus directive
  (`use:focus_on_open=selector`): as a popover or a dialog opens, it moves
  the focus to the first control inside it that the selector names (#214).
  The navigation drawer and the side panel use it.
- The date picker (#157, R25): the island `island::DatePicker` over
  `widget::date::date_field`, a popover under its field from 40 rem and a
  bottom sheet over a scrim below it, at the full width with one month and
  both arrows. It draws one month with a month and year chooser behind its
  title, the field's quick choices (`QuickChoice`, from
  `calendar::Quick`) or a time row, and Today, Clear, Cancel and Apply. It
  follows the WAI-ARIA date picker dialog pattern: every day is named by
  its full date, the arrows move a day or a week, Page Up and Page Down a
  month (with Shift a year), Home and End the week's ends, Escape closes
  and puts the focus back on the field. Days, months and arrows past the
  field's `calendar::Bounds` are disabled, so a field that forbids the
  future offers no later month. A typed text that is no date, or a date
  outside the bounds, marks the field invalid, says why under it, and
  disables Apply. The week starts on the console's `first_day`, from the
  reader's preference cookie (family rule 9). Every word arrives as
  `widget::date::DateWords::of_page`, resolved on the server: a sentence
  that names a chosen day is an `i18n::Phrase`, and a range's day count is
  chosen with its plural on the server.
- The date range of a history filter (#157, R22): the island
  `island::DateRangePicker` over `widget::date::date_range`, one secondary
  button naming the range that opens two months with the presets beside
  them (`RangePreset::of_page` over `calendar::Preset`). "Last 7 days" holds
  today and the six days before it, and the picker's line counts the days.
  Apply writes the From and To fields and submits the filter's form. With
  no WebAssembly the From and To fields show in the button's place and post
  with the form.
- `calendar`, the days a date field counts in: `Day` (crossing to the
  browser as its `YYYY-MM-DD` text), `Month`, `Weekday`, `Clock`, `Moment`,
  `Bounds`, `Quick` and `Preset`, and `Day::parse` and `Moment::parse`,
  which a console's server calls on a posted value to refuse it for the
  same reason the picker does (`calendar::NotADay`).
- The tag (#184, R2): `view::tag::tag` and `view::tag::Tag`, a kind, a
  format, an owner or a namespace as a short word on one line, read as
  text. `Tag::code` adds the raw code as a quiet hint in the code face,
  `Tag::glyph` a glyph hidden from assistive technology, `Tag::mono` the
  code face for a version, and `Tag::locked` F4's lock tag. The gallery's
  overview draws each form.
- A part inside a field's edge (#185, R10): `view::form::Field::leading`
  and `Field::trailing` take an `Inner`: words (`Inner::Text`), a unit
  (`Inner::Unit`, named in the control's description through
  `form::described_with_unit` and `form::unit_id`) or a button
  (`Inner::Button`, with `form::INNER_BUTTON`). The input and its parts
  share one edge and one focus ring.
- The Lucide `calendar`, `chevron-up` and `lock` glyphs (`Glyph::CALENDAR`,
  `Glyph::CHEVRON_UP`, `Glyph::LOCK`).

- The title row's action group (F14), `view::action`: `ActionGroup` with
  one optional primary `Action`, a split primary's `variants` menu, and the
  More menu; `action_group` draws it and `action_button` draws one action
  as a button (#153). An action is a link, a plain form post, a submit of a
  form on the page, or a button that opens a confirmation popover
  (`widget::menu::Act`), so every one works with no WebAssembly.
- The menu (F13) and the select button with its listbox (R29),
  `widget::menu`: `more_menu`, `row_menu`, `variants_menu` and `listbox`,
  over the island `island::MenuButton` (#153). Each is a `<details>` the
  browser opens with no script, its items links or form buttons; the
  island adds the menu button and listbox keyboard paths (the arrows,
  Home, End and a letter, a disabled item skipped), Escape returning the
  focus to the trigger, closing on a focus or a press outside, the flip
  above or to the other edge, and the phone's bottom sheet with its own
  scrim, its title and Cancel. A destructive item comes last after a
  separator in the danger tone, with an ellipsis when a confirmation
  follows, and a disabled item says its reason on a second line.
  `view::theme_switch::locale_select` is the language choice as a select
  button.
- Ready islands for three self-contained widgets (#220):
  `island::Combobox`, `island::PickerField` and `island::ChartLegend`, each
  with serde props and its words as props.
- The Lucide `ellipsis` glyph, `Glyph::ELLIPSIS`, for the row menu's ⋯.
- The top bar's search is one joined control (#144): the search glyph and a
  quiet `/` key cap inside the field, the submit joined on its end edge,
  and one hover edge and focus ring on the group. It is the island
  `island::Search` over `widget::search::search_control`, still a plain
  `GET` form with `role="search"`: `/` from anywhere outside a field
  focuses it (announced through `aria-keyshortcuts`), Escape clears and
  leaves it, a submit turns the glyph into the spinner, and the cap hides
  while the field is filled or focused, while disabled and below 40 rem.
  `SearchBox` takes `submit`, `shortcut` (`Some('/')` by default), `busy`,
  `disabled` (the reason, shown in the field) and `post`, which posts a
  query that can carry clinical content instead of putting it in the URL. The answer under it is
  `view::search::SearchAnswer`, drawn by the server: hits (`Hit`, with a
  `Mark`, a line on its own row and a tag) grouped by kind (`HitGroup`), the
  first one selected for Enter, the way to every result (`Way`), and the
  line when nothing was found. The search result row is the crate's
  `fui-search-results` class, one overlay surface.
- The docked decision bar (R4, #145): `view::dock::decision_bar` over
  `DecisionBar` docks the page's one decision to the foot of the main area,
  with up to two fields (`DockField`: a reason and a path), an editor's
  state line in the field's place when it has none, and its decisions
  (`Decision` with a `DecisionState`: ready, busy, held, or blocked with
  the reason the bar says in words). Every decision is a submit button of
  the bar's own `POST` form or of the page's form it names (`DockForm`), so
  it works with no WebAssembly; a refused reason takes the focus with its
  error. Its lines say a decision is done (`DockLine::Done`) or that
  someone decided first (`DockLine::Stale`). On a phone the fields take
  their own rows and the decisions stack under them, the primary first,
  and the page keeps a focused control clear of the bar.
- Key caps and the keyboard shortcut legend (R1, #141): `view::kbd::kbd`
  draws a key or a chord as printed on the keyboard, `shortcut_keys` the
  alternatives of one shortcut, and `shortcut_hint` the one line at a
  listing's foot, over `Shortcut` and `ShortcutGroup`; `aria_keys` writes
  them for `aria-keyshortcuts`. `widget::shortcuts::shortcut_legend` is
  the legend dialog a console's signal opens over the page, which returns
  the focus when it closes, with its words in `LegendWords`; its dismiss
  button says Close (`LegendWords::close`), as it only informs. The library
  binds no key: `typing`, `chorded` and `phone_layout` are the tests a
  console's listener asks first, and below 40 rem no key cap, hint or
  legend shows.
- Your preferences as library views: `view::preferences::preferences_page`
  draws exactly three tabs, Appearance (theme and density as R28 radio
  preview cards drawing both themes' preview accents, reduce motion),
  Language and region (`reading: true` names it Reading, as FerroEHR does;
  the language as an R29 select with each language in its own name, the
  first day of the week) and Keyboard (shortcut hints and the legend,
  hidden below 40 rem through `data-tab="keyboard"`). A console passes the
  preferences it read, its routes and the cookie name it just kept
  (`saved`), and puts its own preferences inside the three tabs (`own`)
  with `preference_row`, `select_preference` and `switch_preference`.
  Every preference is a plain form post that works with no WebAssembly
  (the new `island::PreferenceForm` posts on change once hydrated and
  returns the focus to the row just saved); Saved has a fixed slot that
  never moves its control; Reset to defaults asks first and posts to the
  console's route. The library writes no cookie (#158).
- `theme::Preference` (each preference's cookie name, cookie value and
  parse, a missing or unknown value read as the default), the new
  preferences `Motion`, `WeekStart`, `Hints` and `Language`,
  `theme::Preferences` with `Preferences::read` over a `Cookie` header, and
  `theme::cookie` (#158).
- The stylesheet reads `data-motion="reduce"` as `prefers-reduced-motion`
  does and `data-hints="always"` or `"never"` for the key hints; write both
  on `<html>` from their cookies. Under reduced motion the Saved mark stays
  instead of fading (#158).
- `island::CopyButton` and `island::SidePanel`, the ready islands of the copy
  button and the side panel, with their words as props (`CopyWords`,
  `SidePanelWords`). The side panel's opener (`widget::side_panel::Opener`)
  is a link to the content's own route until the page hydrates, then the
  button that opens the panel; the panel moves the focus to its Close button
  as it opens and returns it to the opener as it closes (#220).
- Copy in the code viewer's head (`CodeHead::copy`, the id of its button) and
  in the document pane, which copies the document in the format in force (#220).
- `Glyph::name` and `Glyph::named`, and serde for `Glyph` (as its Lucide
  name) and for `view::button::Variant`, so an island can take either as a
  prop (#220).
- The crate docs name the widgets that share state with a console's code
  (editor, filter, pager, dialog, tree, upload, help) as island parts (#220).

### Changed

- `view::surface::page_heading`, `editor_heading` and `page_header` take an
  `ActionGroup` in place of a free view, and the templates' title rows
  follow: `ListPage::actions` replaces `ListPage::primary`,
  `DetailPage::actions` and `SettingsPage::actions` are an `ActionGroup`,
  and `DetailPage::stepper` draws a queue's previous and next (R5) in the
  title row (#153). A destructive action moves into More, last; no
  component draws a danger button beside the primary any more. A console
  passes `ActionGroup::none()` where it passed `()`.
- The combobox's listbox is a keyed `<For>` over the shown options, keyed
  by the option's value, with the empty state outside it, so moving the
  highlight changes only that option's attributes and no row is drawn
  again (#207). `ComboOption` is `Serialize` and `Deserialize`.
- The chart's legend is a static key in the server HTML, each swatch beside
  its series' name, and becomes a toggle per series once its island
  hydrates, so a sighted reader keeps the colour key with no WebAssembly
  (#227). `widget::chart::line_chart(&chart, words)` draws the plot and the table
  on the server and takes no `shown` signal; the legend is
  `island::ChartLegend` (`widget::chart::legend`).
- The picker's "another" choice is its select's last option, marked
  `data-other` and known by its place, so no value a console passes can
  collide with it; the select follows the value through `selectedIndex`,
  and a listed option's value is the console's own (#227). `Picker::id` and
  `Picker::name` are `String`, and `widget::help::help_line` takes a `&str`
  id.
- The help switch is a native checkbox (`role="switch"`) and its help
  lines show and hide by a stylesheet rule, with no signal, no context and
  no script, so a reader shows the help before the page hydrates and with
  no WebAssembly at all (#208). The module moves to `view::help`, since it
  is a static view: `help_switch(id, words)` takes the switch's id, and
  `HelpWords` holds its one `label` in place of `show` and `hide`;
  `help_line`, `explained_field` and `explained_check_field` mark their line
  `fui-explain`, which a form with a switch hides from sight (still read,
  WCAG 2.2 technique C7) while the switch is off. A help line in no form
  with a switch always shows, so the picker's help line follows the switch
  of its form. This breaks the public API: a console replaces
  `widget::help::*` with `view::help::*`, drops `provide_help()` and passes
  `help_switch` an id.
- The tabs, the picker, the tree and the combobox move the focus and reach
  their elements through `NodeRef`s, no longer by an id lookup in the
  document (#214).
- The gallery derives the code-system field's typing from its choice in
  place of an effect that wrote one signal from another, holds its constant
  signals with `Signal::stored`, and takes every page's title from one
  `<Title formatter>` at its root (#216); the titles read as before.
- The document pane is a server view with an island over its views
  (#212): `widget::document::document_pane`, `document_pane_opening` and
  `DocumentPane` give way to `view::document::document_pane` and
  `Document`. A console passes the format links and the text of the format
  in force where it passed signals and an `on_format` callback, and the
  view to open as a `DocumentView`. The pane's Copy is absent until the
  copy island lands (#220).
- `view::code::code_panel` and `code_lines` give way to the code viewer
  (#160): `code_viewer(None, &code)` is the bare box, its lines always
  numbered, and `Code::ids` names the line ids `code_lines` took.
- Inside a tab panel, `view::surface::titled_panel`, `panel_body` and a
  `card` give way to `panel_section` (#159); `titled_panel` stays for a
  card that stands on a page of its own.
- The code viewer's path (`fui-codeview-path`) is the head's title in the
  prose font at the title weight, no longer bold mono, recorded in
  `design/COMPONENTS.md` B1.
- `view::surface::page_heading` draws the trail's back arrow, so below
  40 rem the trail is one back link to the parent (R6), and a state badge
  beside the title from the new `Header::badge` (R24). `Header` is no
  longer `Clone`, `PartialEq` or `Eq`; a console that builds it with every
  field adds `badge: None`. `page_header` draws through it unchanged.
- `view::surface::editor_heading` ends its trail with the list a new thing
  joins or with the thing edited, never with the editor (`fui-crumbs
  is-parent`), and its title is the console's words alone: a console's
  editor message reads `[new] New code system *[edit] Edit` and passes the
  thing edited as the last crumb.
- The footer writes the product and its version without the "v"
  ("FerroBRIDGE Console 0.0.12 · Cadasto B.V. · About"), the form the
  About page shows.
- T3's setting rows are `template::settings::setting_row`;
  `view::layout::field_row` and `keyed_field_row` stay the form grid's row
  of fields, unchanged.
- The design's error pages say the status in words before its number on
  every console.
- The filter chip's remove control may be a link (`.fui-filter-chip > a`).
- `.claude/rules/leptos-ui.md` §11: an editor's title is "New <thing>" or
  "Edit", and its trail ends with the resource.

- A sortable header is a submit button in a `GET` form that carries `sort`,
  `order` and the rest of the listing's query, never a link (#155), so a
  page sorts with no script. This breaks the public API: `DataTable`'s
  `sort` and `sort_href` give way to `sorting: Sorting`, which names the
  sort in force, the form's action and the fields it carries; a listing in
  session state posts the same fields. A console passes
  `Sorting::default()` for a table that does not sort, and reads `sort` and
  `order` from the query or its handler as before.
- The table follows the design's measures (#155): inside a card or a panel
  its rows are hairline lines on a phone, a list card's row is the compact
  work-list line (the checkbox, the name and the ⋯ on the first line, the
  facts on the second, a wide column left out), a count carries its noun,
  and a name is ink with no underline while the whole row is the target. A
  row's checkbox sits on its row's first line on a phone instead of
  leaving the row, and a menu or checkbox column's header is hidden to the
  eye. A console moves its rows onto the cell helpers or writes the same
  `data-label`, `data-role` and `data-unit`.
- The pagers move from `table::view` to `table::pager` (`Step`,
  `step_label`, `Listing`, `pager`, `open_pager`, `link_pager`,
  `link_open_pager`). A console changes its imports.
- `widget::filter::table_filter` is only for rows that are already island
  data; a listing's filter is `table::filter::find_form`.
- The tab API (#152): `view::tabs::link_tabs`, `keyed_tabs` and `KeyedTab`
  are gone; `binder` takes `Tab { key, href, label, count, draft }`
  (`Tab::new(key, href, label)`) and the panel. A console moves each tab
  row and the panel under it into one `binder` call, and the panel loses
  its own card frame.
- The toast API (#156): `widget::toast::raise(tone, text)` is
  `raise(Toast)`, and `Flash` has `title`, `line` and `action` in place of
  `text`. A console that toasts a load result moves it to the load report
  (B4).
- Each widget takes the words it draws (#219): `copy_button` takes
  `CopyWords`, `help_switch` takes `HelpWords`, `line_chart` takes
  `ChartWords`, and `Modal`, `SidePanel`, `Combobox`,
  `KeyedCombobox` and `Upload` gain a `words` field.
  `view::chart::figures` takes `FiguresWords`, and
  `upload::Refused::sentence` takes `UploadWords`. A
  console builds each with `of_page()` on the server and passes it to its
  island as a prop.
- The gallery's bundle budget is 491520 bytes of brotli-compressed
  WebAssembly, down from 552960 (`.claude/rules/leptos-ui.md` §12).
- The console layer of the approved design moves into
  `css/ferrohealth-ui.css` (#150): the class of every console component in
  `design/COMPONENTS.md` and the family rules as CSS, in the `components`
  layer, so a console copies no second stylesheet. `design/css/ferroui-console.css`
  keeps only the mockups' `mk-` capture scaffolding. The sizes of type the
  layer wrote off the scale now name a step, and every step carries its
  leading. A console drops any copy of the design's layer it carries.
- A brand theme file sets seven custom properties instead of five:
  `--preview-accent-light` and `--preview-accent-dark`, each the accent of
  its theme, in a closing `:root` block, read by the theme previews in Your
  preferences. The stylesheet defaults them to the FerroHEALTH steel. A
  console with its own theme file adds the two.
- The family rules change the paint of existing classes: hover and focus
  change paint only and move nothing (rule 3), a link in a table is ink
  with no underline (rule 4), no key hint shows below 40 rem (rule 6),
  comfortable is the default density and compact the preference (rule 10),
  and a state is a shape, a colour and a word together, so a status pill's
  dot is a diamond when degraded, a cross when unreachable and a hollow
  ring when unknown (rule 12). A console looks at its screens.
- Every control shows one focus ring (R18), and every popup (menu,
  popover, listbox, date picker) is one overlay surface: a full border in
  the strong hairline and, in dark, a ground lifted above the raised cards.
- The form grid's fields share their label line (`align-items: start`, R11),
  and a field holding a long token takes the row with `fui-span-row`, which
  moves fields `view::form::columns` draws today. A console looks at its
  forms.
- A stacked table on a phone draws its rows as hairline-separated lines,
  with no gap between bodies, in place of cards.
- `widget::nav::sidebar(id, label, groups, current)` draws plain links with
  the page at `current` marked `aria-current="page"`, inside the
  `NavDrawer` island, and no longer depends on `leptos_router`. A console
  passes the request's path.
- `widget::toast::toast_region(words, flash)` replaces the context-based
  region; a console draws `island::ToastRegion` once in its shell.
- `tone::Tone` is `Serialize` and `Deserialize`, so it crosses to an island.
- The gallery's bundle budget is 552960 bytes of brotli-compressed
  WebAssembly, down from 716800 (`.claude/rules/leptos-ui.md` §12).
- The interactive line chart (`widget::chart::line_chart`) draws its SVG
  once, every series in groups of its own, and a legend chip sets the
  `hidden` attribute of its series' groups alone, so a press no longer
  redraws the SVG as a string the browser parses again (#209). The legend
  chips stay hidden until the page hydrates, so none offers a press that
  does nothing; the figures table reads as before. `view::chart::line_svg`
  draws the same markup as before.
- The date field (#157, R23): a console's own `<input type="date">` and
  `<input type="datetime-local">` give way to `island::DatePicker`, a text
  input written year first (`YYYY-MM-DD`, or `YYYY-MM-DD HH:MM` with
  `with_time`) with its format stated after its label and the calendar
  behind a button inside its edge. The typed text is the field's value, so
  the form posts it with no script; until the WebAssembly runs the button
  is disabled and its name says to type the date. The gallery's date and
  date-time inputs moved to it.
- `.fui-input-group` draws the field's one edge, its ground and its focus
  ring (`:focus-within`), and the input inside it draws none (#185, R10).
  Its prefix (`fui-input-prefix`), suffix (`fui-input-suffix`) and button
  (`fui-input-toggle`) sit beside the typed text, no longer over it, so a
  long unit never covers what was typed. An invalid or disabled input marks
  the whole group. A console that wrote the group by hand keeps its markup.
- `view::surface::chip` is for a short code, such as a path or a scope; a
  kind, a format, an owner or a namespace is a tag (#184).
- The date picker's phone sheet takes the full width for a range too: the
  range's own placement no longer outranks the sheet's below 40 rem, and the
  foot's buttons centre their words (#157).

- The top bar is one struct (#151): `view::shell::topbar(TopBar { sidebar,
  lockup, search, end })` draws the drawer button, the lockup, the search
  box and the end, and carries no theme control (family rule 5). A console
  puts the status pill and the user menu in `end`.
- `widget::nav::sidebar(Sidebar { id, label, groups, current, rail })`
  takes one struct, and `Entry` carries `count: Option<Count>`. A console
  adds `count: None` to an entry with no count, and passes a `Rail` with its
  own sidebar route to offer Collapse.
- `view::sign_in::sign_in(SignIn)` takes the design's set (#186): `title`,
  `state`, `provider` (with its busy words and glyph), `basic` (its divider,
  submit words, username and error), `line`, `note`, `links` and `caption`.
  `Notice` is the words of a state's notice; the tone comes from the
  state. The card and its caption are drawn in one centred column.
- The sign-in page's title, lines and column are crate classes
  (`fui-sign-in-title`, `fui-sign-in-line`, `fui-sign-in-stack`) in place of
  the design's capture scaffolding.

- The design's search answer is the crate's `fui-search-results` class in
  place of the capture scaffolding's `mk-results`, and on a phone the
  docked bar's decisions stack at full width, the primary first, where they
  shared one row (#144, #145).
- `view::search::search_box(search, answer)` takes the answer under the
  search, and `SearchBox`'s fields are `String`s, since the box is an
  island and its props cross to the browser: a console fills the new
  fields or takes `..SearchBox::default()`.
- The copy button and a setting's key hint are hidden until the page
  hydrates, in place of disabled; with no WebAssembly the text they copy
  stays on the page as selectable text, and the key hint shows its key as
  text. A console that built a `CodeHead` with a struct literal adds
  `copy: None` or the id of its Copy button (#220).
- `widget::side_panel::SidePanel` takes `id` and `title` as `String` and an
  `opener: Opener`, and draws its opener itself; its scrolling body's id is
  `id` and `-body`. Below 40 rem the panel is a sheet over the whole layout
  viewport, scrollbar excluded, with its body filling it (#220).

### Removed

- `view::dialog::confirm_trigger`: open a confirmation from an action
  whose act is `widget::menu::Act::Dialog`, a title row's primary or an
  item of More (#153).
- `widget::help::HelpShown` and `widget::help::provide_help`, and the
  catalog message `help-hide` ("Hide help"): the switch keeps one label and
  says on or off as a switch (#208).
- `view::code::code_panel`, `view::code::code_lines`,
  `widget::document::document_pane`, `document_pane_opening` and
  `DocumentPane`, which the code viewer and `view::document::document_pane`
  replace (#160, #212).
- `view::surface::page_heading_live` (#210): a view function takes no
  reactive title. A console calls `page_heading` with the title its server
  knows when it renders.
- `view::surface::table_card`, which the list card replaces:
  `template::list::list_card`.
- `view::shell::About`, `footer_about`, `shell_about` and `centred_about`:
  with no client router an About link is a plain link; call `footer`,
  `shell` and `centred`.
- `i18n::provide_page_locale` and `i18n::document_lang` (#219): a
  console's browser entry builds no locale; it calls `hydrate_islands()`
  alone. `widget::upload::size` is `SizeWords::size`.
- `widget::theme::theme_toggle`, the toggle that swapped the theme in
  place.
- `widget::toast::Toasts`, `provide_toasts` and `use_toasts`: an island
  raises a toast with `raise`.
- `widget::tabs::tabs` and `widget::tabs::Panel`: draw the panels with
  `tab_panels` inside `island::KeyboardTabs`.
- `widget::pager::session_pager` and `session_open_pager` (#215): their
  buttons paged only with the WebAssembly loaded and needed the listing's
  rows as island data. A console pages a session listing with
  `table::pager::post_pager` and keeps the position in its session.
- `view::theme_switch::theme_toggle` and its ids `TO_DARK` and `TO_LIGHT`,
  the theme toggle of the top bar, with its classes (`fui-theme-toggle`,
  `fui-theme-to-dark`, `fui-theme-to-light`) and its catalog messages
  (#151). The theme is set in the user menu and in Your preferences
  (`view::theme_switch::theme_switch`). A console removes the toggle from
  its top bar and passes its theme route to `user_menu`.
- `view::menu::operator_menu` and `OperatorMenu`: use
  `widget::menu::user_menu`. The class `fui-sign-out` goes with them.
- `view::shell::topbar_with` and the two-part `topbar(start, end)`: use
  `topbar(TopBar)`.
- `view::sign_in::REFUSAL_ID` and the separate refusal: a refusal is a
  `State`, and its notice carries the id `NOTICE_ID`.
- The `font-size: 0` that hid a narrow status pill's words; they are
  visually hidden (`fui-roomy-text`) and the type scale test holds every
  size again.
- `view::layout::action_bar` and its `fui-action-bar` class, the actions at
  the foot of a card: the page's decision goes in the docked decision bar,
  `view::dock::decision_bar` (#145).
- `view::theme_switch` and its `theme_switch`, `density_switch`,
  `locale_switch` and `locale_select`: Your preferences
  (`view::preferences`) replaces the segmented switches, and
  `view::preferences::language_select` the language select. A console
  draws `preferences_page` behind the user menu's Your preferences link and
  answers its posts (#158).
- `widget::side_panel::side_panel_trigger` and `SidePanel::body_id`: the
  panel draws its own opener, and its body's id follows from its own (#220).

### Fixed

- The picker's select (`widget::picker::picker_field`) follows its value
  through `prop:value` on the select, so it always shows the value's
  choice, also after the reader has chosen by hand (#206). The server marks
  the initial choice with a static `selected` and writes the value into the
  input, so a page posts it before it hydrates, and the choices are a keyed
  list. The "another" choice no longer starts with U+0000, which the HTML
  parser turns into U+FFFD, so choosing it keeps the value in the input
  where it wrote U+FFFD and "another" before. The gallery's runner form has
  a button that sets the value from the console's side.
- `scripts/checks/crate-version-guard.sh` reads each file into a variable
  before it searches it, so a reader that stops at its first match no
  longer kills `git show` with SIGPIPE, which `pipefail` reported as a
  missing changelog section or a failed lock read once the file outgrew the
  pipe buffer (#203). Its `--self-test`, run in CI, checks a changelog and a
  lock over 64 KB twenty times and a changelog without the section.
- The gallery's state capture lists the card-header action's hover and
  focus (`hover-card-action`, `focus-card-action`), so their images are
  committed for every brand, light and dark, and the screenshot guard no
  longer rejects them after a capture run (#201). The capture now refuses to
  write a state its list does not name and fails when it did not write
  every listed state.
- The editor's text area carries its text in the page the server draws,
  escaped, so a form posts it before or without the WebAssembly; before,
  it arrived empty and only the highlight layer showed the text.
- The top bar's end fits its row with a classic scrollbar, which takes
  15 px from the layout viewport on Linux and Windows (#234). At a 640 px
  window the user menu's button ran 4 to 12 px past the 625 px viewport,
  so the page scrolled sideways (WCAG 2.2 SC 1.4.10). Below 48 rem, where
  the top bar wraps, the operator's name is now visually hidden, as it was
  below 40 rem, and stays the button's accessible name and title; the end
  keeps its parts' width and the lockup's name gives way with an ellipsis.
  The user menu sheet's journey measures the sheet against the layout
  viewport (`clientWidth`, `clientHeight`) instead of the window.
- The About page's Copy works: it is the copy island, where it was a button
  drawn on the server alone that never turned on (#220).
- The native select and the combobox input draw their chevron at the size of the 16 px Lucide chevron a select button draws, 10 by 5 px on screen; it was 12 by 6 (#255).

## [0.2.2] - 2026-10-21

The row depth of a hierarchy table, and two fixes FerroTERM's move onto
FerroUI found. A console pins `=0.2.2` in place of `=0.2.1`; no API breaks.

### Added

- `table::view::depth(level)` and the classes `fui-depth-0` to `fui-depth-8`:
  the indentation of a hierarchy row's leading cell in a table, one loose
  step per level in the wide table and one default step in the stacked one,
  every level past the eighth at the eighth step (#132). The gallery's tables
  page shows a hierarchy in a table.

### Fixed

- A `fui-fieldset` that is a card's own child keeps the card's spacing above
  it: its margin reset no longer outweighs `.fui-card > * + *` (#134).
- A `ul` or `ol` laid out by `fui-grid`, `fui-stack`, `fui-cluster`,
  `fui-split` or `fui-columns` draws no list marker and keeps no indent
  (#135). The gallery's layouts page shows both.

## [0.2.1] - 2026-10-21

A patch release with no API change: a console pins `=0.2.1` in place of
`=0.2.0` and changes nothing else.

### Changed

- The crate declares `rust-version = "1.98"` in place of 1.97, and the
  repository builds with Rust 1.99.1 in place of 1.98.1: the FerroHEALTH
  family builds with the latest stable toolchain and declares the release
  before it as its MSRV. A console on a toolchain older than 1.98 can no
  longer take the crate.

## [0.2.0] - 2026-10-19

Every console can draw every label from its own catalog: each text a
console hands a component accepts a string made at run time, and a
listing's noun is a console message chosen by count. One card-header
action serves every console, and an editor's header says whether it
creates or changes. This breaks the public API: the entries below name
what a console changes.

### Added

- `i18n::Message`: a message of a console's catalog, named by its id, that
  a component looks up as it draws, so the words follow the page's locale
  (FerroHEALTH/FerroUI#114).
- `view::surface::card_action`: the one action at the far end of a card's
  or a panel's header, a link with its words and an arrow after them a
  tight space apart, in the accent with the sunken ground under the pointer
  (FerroHEALTH/FerroUI#122).
- `view::surface::{Mode, editor_title, editor_heading}`: an editor's header
  says "New <thing>" while it creates and "Edit <name>" while it edits, the
  title and the trail's last crumb from one console message that selects
  by `$mode`, in the body face (FerroHEALTH/FerroUI#123).
- A list that runs to a panel's edges keeps its words off them, and a page
  header inside a well leaves no space under it.
- The gallery draws its own page strings from its catalog
  (`app/ferrohealth-ui-gallery/locales/en/ferrohealth-ui-gallery.ftl`), so
  the pseudo-locale stretches every string on a page, and the stray-literal
  check covers the gallery (FerroHEALTH/FerroUI#114).
- The gallery shows the card-header action on the overview, with its hover
  and focus states captured, and both editor headers on the forms page.

### Changed

- Every text a console hands a component accepts a string made at run time,
  so a console draws every label from its catalog (FerroHEALTH/FerroUI#114).
  A console changes:
  - `table::model::Noun` is a console message chosen by count, with no
    English plural rule in the library: `Noun::new(Message::new(catalog,
    "value-sets"))` over a message that selects by `$count` and carries
    `.empty` ("No value sets") and `.opening` ("Value sets"). `Noun::of` and
    the `one`/`many` fields are gone; `Noun::word`, `Noun::empty` and
    `Noun::opening` draw the forms. A noun is still `Copy` and is made where
    the catalog is in reach rather than as a `const`.
  - `table::view::Column::text` takes its header as
    `impl Into<Cow<'static, str>>` and is no longer `const`: keep a table's
    columns in a function.
  - `table::view::Listing` holds its label as a `Cow<'static, str>` and is
    `Clone`, no longer `Copy`; `Listing::new` takes `impl Into<Cow<'static,
    str>>`, and a `String` converts into a listing as a `&'static str` does.
  - `widget::nav::{Entry, Group}` hold their words as `Cow<'static, str>`, a
    group its entries as a `Vec<Entry>`, and `sidebar` takes its label as
    `impl Into<String>` and its groups as a `Vec<Group>`: build the
    navigation where it is drawn.
  - The fields of `view::dialog::Confirmation`, `view::search::SearchBox`,
    `view::menu::OperatorMenu::subtitle`, `view::shell::Footer::product`,
    `view::layout::Pane`, `view::tabs::{Tab, KeyedTab}`,
    `view::segmented::SegmentedForm`, `widget::upload::Upload`,
    `widget::picker::Picker`, `widget::combobox::Combobox`,
    `widget::tree::Tree`, `widget::tabs::Panel`,
    `widget::side_panel::SidePanel` and `widget::document::DocumentPane`
    (whose formats are a `Vec`) that a reader sees are `Cow<'static, str>`;
    a struct that was `Copy` (`Confirmation`, `Footer`, `Tab`, `Upload`,
    `Picker`, `Tree`) is `Clone`. Write `"words".into()` or the catalog's
    `String` with `.into()`.
  - The functions that took a label as `&'static str` take
    `impl Into<String>` (or `impl Into<Oco<'static, str>>` beside
    `form::Field`): `spinner`, `skeleton`, `banner`, `fieldset`, `check`,
    `keyed_check`, `check_field`, `keyed_check_field`, `chip_choices`,
    `choice_group`, `choice_reveal`, `segmented_links`, `link_tabs`,
    `keyed_tabs`, `tile_list`, `change_table`, `confirm_trigger`,
    `link_button`, `lockup`, `help_line`, `explained_field`,
    `explained_check_field`, `side_panel_trigger`, `table_filter`,
    `widget::tabs::tabs` and `widget::segmented::segmented`. The option
    lists of `choice_group`, `choice_reveal` and `segmented` take any
    `AsRef<str>` words, and `definitions` and `facts_row` any
    `Into<String>` label. `form::field` and `keyed_field` take their help
    as `Option<String>`, `divided_list` its label as
    `Option<Cow<'static, str>>`. Those that returned
    `impl IntoView + use<>` return `AnyView`.

- Only the newest release is supported (#119). `SECURITY.md` says that every
  earlier release stops receiving fixes when a new one is published, with no
  maintenance branches, no long-term-support line and no backports, and that
  upgrading is free. Cadasto B.V. commits to keep publishing security fixes
  for at least five years after each release (Regulation (EU) 2024/2847
  Art 13(8)) and ships them only in the newest release (Art 13(10)).
### Fixed

- The footer draws the product's version as data in every locale, never
  through the catalog, and its About link keeps its words on one line
  (FerroHEALTH/FerroUI#116).

## [0.1.7] - 2026-10-19

The family is ready for more languages, with English the one that ships:
every word a component draws itself comes from a Fluent catalog, a new
language is a catalog file with no code change, and the locale is chosen
once per render and reachable by a console for `<html lang>`. The
English words and the markup they draw in are unchanged.

### Added

- `i18n`: the locale and the catalogs. `Locale` (from a BCP 47 tag, or
  negotiated from `Accept-Language` among the languages the library ships
  or, with `negotiate_with`, a console's catalog holds, falling back to the
  console's configured locale),
  `provide_locale` and `locale` to choose it once per render and read it,
  `Locale::tag` for `<html lang>`, `Locale::lookup` for a console's own
  catalog, and `Locale::with_catalog` for a catalog the library does not
  ship. The catalogs are embedded at build time from `locales/` by
  `fluent-templates` 0.15.1, on both halves and with no script; `en` is the
  reference catalog every lookup falls back to (FerroHEALTH/FerroUI#112).
- `view::theme_switch::locale_switch`: the language switch, a plain form
  whose choices are each named in their own language and marked with it in
  `lang` (FerroHEALTH/FerroUI#112).
- `view::layout::keyed_field_row`: a row of fields with an id, for a row a
  console adds, removes or finds, such as one value of a list.
- `scripts/checks/stray-literals.sh`, in CI: fails on a string a reader
  sees in a component outside the catalogs (FerroHEALTH/FerroUI#112).
- The gallery's language switch on the overview's settings, and its
  pseudo-locale, which draws every library string accented and two fifths
  longer to check the layout (FerroHEALTH/FerroUI#112).

### Changed

- Every string a component draws itself is a message of
  `locales/en/ferrohealth-ui.ftl`, looked up in the page's locale. The
  words that were `&'static str` return values are `String` now:
  `theme::Theme::label`, `theme::Density::label`,
  `table::view::Step::label`, `view::refusal::Severity::word` and
  `view::button::Variant::name` (FerroHEALTH/FerroUI#112).
- `view::button::label` and `widget::copy::copy_button` take their label as
  anything that converts into `Oco<'static, str>`, a `&'static str` as
  before or a `String` from a catalog.
- The gallery writes the locale its cookie names as `<html lang>`.

## [0.1.6] - 2026-10-19

The shared form fields every console needs so no field is built by hand: a
label made at run time, a visually hidden label for a list row, a line
after the help for a field's refusal, an action with its error line, a
field that switches between a picker and a typed value, and a checkbox
whose help follows the form's help switch. The additions are a type, four
functions, a class constant and a widget module; `field` and `keyed_field`
take their label as a `&'static str` or a `String` alike, so no call site
changes.

### Added

- `view::form::Field`: a field drawn piece by piece, its label above the
  control or visually hidden (`label_hidden`, for a row of a list of values
  with its remove button), its help, and a trailing slot after the help
  (`after`) for the line a refusal shows in. `field` and `keyed_field` draw
  through it (FerroHEALTH/FerroUI#104, FerroHEALTH/FerroUI#105,
  FerroHEALTH/FerroUI#106).
- `view::form::error_id`, `described_by` and `ERROR`: the id of a field's
  error line, the `aria-describedby` value that names the help and then the
  error line, and the error line's classes, the help's size in the
  refusal's colour (FerroHEALTH/FerroUI#106).
- `view::form::action_field`: a button and the line its refusal shows in,
  one field with no label, such as the button that adds a row to a list
  (FerroHEALTH/FerroUI#108).
- `widget::pick_or_type`: one value picked from a select or a combobox, or
  typed in the input behind its last option, each control with its own
  label hidden and shown with it, the help after both
  (FerroHEALTH/FerroUI#107).
- `widget::help::explained_check_field`: the checkbox row of
  `check_field` with its help line under the form's help switch, hidden
  from sight and still read while the switch is off
  (FerroHEALTH/FerroUI#110).

### Changed

- `view::form::field` and `keyed_field` take their label as anything that
  converts into `Oco<'static, str>`: a `&'static str` as before, or a
  `String` made at run time (FerroHEALTH/FerroUI#104).
- The render tests read a start tag's attributes by name, so none depends
  on the order Leptos writes them in, and a sortable header's `aria-sort` is
  tested beside its scope, class and title (FerroHEALTH/FerroUI#103).
- The screenshot capture waits until the page's fonts have loaded and its
  height holds over two frames before it measures or shoots, so a capture
  no longer catches a page still laying out (FerroHEALTH/FerroUI#101).

### Fixed

- A `FieldRow::WithActions` row keeps its field and its actions on one row
  at every width, where a phone stacked each remove button alone under its
  value.
- A button among a field's parts, such as one that returns from a typed
  value to the list, keeps its own width at the start of the field, where
  it stretched across it.
- A plain data table header writes no `class` attribute, where it wrote an
  empty one (FerroHEALTH/FerroUI#102).

## [0.1.5] - 2026-10-19

A form field whose id is made per row, a title on a data table header, a
gallery build that finds its own tools and a bundle check that never
measures a stale bundle, and a browser half that no longer moves a
console's wasm-bindgen. The additions are five functions and a field of
`table::view::Column`; a console that builds a `Column` with a struct
literal names `title` (none of the four does; they use the constructors).

### Added

- `view::form::keyed_field`, `keyed_check` and `keyed_check_field`: the
  field, the checkbox row and the checkbox with help for a control whose id
  is made at run time, such as one per clause of a list, the label's `for`
  and the help id kept in step with it. `field`, `check` and `check_field`
  draw through them (FerroHEALTH/FerroUI#98).
- `table::view::Column::titled` and the field `title`: a header cell that
  carries a `title`, such as the path a query column reads, so no cell has
  to repeat it (FerroHEALTH/FerroUI#99).

### Changed

- The browser half asks for web-sys 0.3.85 and wasm-bindgen-futures 0.4.58,
  the lowest releases its Leptos resolves with, and no longer depends on the
  wasm-bindgen crate itself, so a console's own lock picks its wasm-bindgen
  and the CLI that must equal it. `scripts/checks/browser-floor.sh` builds
  the library at those floors in CI (FerroHEALTH/FerroUI#44).
- `scripts/release/gallery-site.sh` puts the tools `gallery-tools.sh`
  installed under `target/gallery-tools` on `PATH` itself, and with none
  installed names the command that installs them (FerroHEALTH/FerroUI#97).

### Fixed

- A button or checkbox in a `field_flow` row whose field has a help line
  stays on the controls' line from 30rem up, where it sat on the help's
  line before.
- `scripts/checks/gallery-bundle.sh` refuses a bundle the last release site
  build did not write: the build removes its stamp first and writes the
  bundle's digests on success, so a failed build no longer measures the
  previous bundle as within the budget (FerroHEALTH/FerroUI#97).

## [0.1.4] - 2026-10-19

A shared timestamp, a compact stat tile on a phone, a visible edge on a
disabled field in a well, a field and its button stacked on a phone, and a
data table of several bodies. The additions are a module with two
functions, a class and a variant of `Body`; a console that matches on
`Body` adds an arm for it.

### Added

- `view::time::timestamp` and `view::time::reading`: a date-time drawn for
  reading on one line, an instant moved to UTC and cut to the second
  ("2026-10-18 23:18:37 UTC"), a date-time without an offset as it stands,
  in a `<time>` whose `datetime` is HTML's machine form and whose `title`
  is the value as the server sent it; anything else is drawn verbatim
  (FerroHEALTH/FerroUI#91).
- `fui-time`: a value that never breaks, in a wide table, a stacked card on
  a phone and a definition grid alike.
- `table::view::Body::Bodies`: a data table of several `<tbody>` elements
  the console draws, such as one per page of a query's answer, stacked
  below 40rem like any other rows and keeping the edge between bodies on a
  wide screen (FerroHEALTH/FerroUI#94).

### Changed

- Below 30rem a stat card is a compact tile: the glyph, the figure and the
  label on one line, at most 64px tall, the label cut with an ellipsis and
  named whole in its `title`, and a mark under the label. Wider screens
  keep the tile as it was (FerroHEALTH/FerroUI#93).

### Fixed

- A disabled input, select or textarea keeps the control's edge, so inside
  a well, whose sunken ground is its own, it still reads as a field in the
  dark theme (FerroHEALTH/FerroUI#92).
- Below 30rem every field and button of a `field_flow` takes the whole
  row, the label above its control and the button on its own line after
  the input, so a field with a trailing button, such as a code search,
  stacks on a phone (FerroHEALTH/FerroUI#95).
- A fieldset set apart as a well draws its heading's rule in the stronger
  edge, which the dark theme's edge matched on the well's ground.

## [0.1.3] - 2026-10-19

The top bar, combobox and stacked table fixes the FerroTERM console review
asked for, the stat card alignment FerroFED reported and the editor's owner
FerroEHR reported. No public signature changes; the additions are a new function, a
new struct and a new class.

### Added

- `widget::combobox::keyed_combobox` and `KeyedCombobox`: a combobox whose
  id, name, label and placeholder a console makes at run time, such as one
  per clause of a list, and whose input names its help line in
  `aria-describedby`; `combobox` keeps its signature and draws through it
  (FerroHEALTH/FerroUI#85).
- `fui-truncate`: one line that ends in an ellipsis when its box is
  narrower, kept to 28rem in a table cell and to the card in a stacked row.

### Fixed

- The top bar never covers its search: from 48rem the search shrinks first,
  to 11rem, and the operator's name then ends in an ellipsis (at most 16rem)
  with the whole name as the button's title, at every width from 390 to
  1920 px (FerroHEALTH/FerroUI#87).
- A stacked table takes its container's width, so a cell that keeps its line
  no longer carries every card past a phone's edge
  (FerroHEALTH/FerroUI#86).
- A visually hidden legend in a `fui-fieldset` keeps the hidden box rather
  than the group heading's width (FerroHEALTH/FerroUI#89).
- Stat cards keep their glyphs and figures in line whether or not a card
  carries a mark under its label (FerroHEALTH/FerroUI#84).
- `code_editor` allocates no reactive value of its own: its grammar is a
  plain value, so an editor drawn inside a reloading `Transition` holds
  nothing its caller's owner disposes under it (FerroHEALTH/FerroUI#88).
- A field's control keeps to its field: a date and time input, whose own
  minimum width follows the platform's fonts, no longer widens a row of
  fields past a narrow screen.

## [0.1.2] - 2026-10-19

The fixes the owner's review of the four consoles asked for: links, fields,
selects, buttons, definition grids, empty states, tabs, the top bar, the
pager and the charts, each fixed once here so every console gains it. No
public signature changes; the additions are new functions and classes.

### Added

- `view::surface::grid_start`: cards side by side at their own heights, for
  cards of unlike length (FerroHEALTH/FerroUI#81).
- `view::shell::About`, `footer_about`, `shell_about` and `centred_about`:
  the footer's About link can be a full page load (`rel="external"`) for an
  About page the server renders once (FerroHEALTH/FerroUI#82).

- `view::form::FIELD_FLOW` and `field_flow`: fields in one row share its
  width, each growing from 10rem so a placeholder is not cut, and the row's
  buttons keep their own (FerroHEALTH/FerroUI#59).
- `widget::document::document_pane_opening`: a document pane open on the view
  a deep link names, offering its rendered view only while the document has
  one (FerroHEALTH/FerroUI#50).
- `view::surface::page_heading_live`: a page heading whose title follows a
  signal, so it stays true under client-side navigation
  (FerroHEALTH/FerroUI#51).
- `widget::combobox::combobox`: a long list to choose from, filtered as the
  reader types, after the WAI-ARIA 1.2 combobox pattern over a listbox (Down,
  Up, Home, End, Enter, Escape), each option its words over a muted second
  line, the chosen one checked, a "No match" row, a bounded height that
  scrolls; before hydration it is a native select a form posts
  (FerroHEALTH/FerroUI#72).
- `table::model::filled_columns`: the columns some row fills, so a table
  draws no column that is empty in every row (FerroHEALTH/FerroUI#75).
- `view::surface::link_card_glyph`, and a link card shows a chevron after its
  title and the accent edge under the pointer and the keyboard
  (FerroHEALTH/FerroUI#70).
- `table::view::data_table` stacks its rows on a phone: below 40rem each
  row is a card, every cell with a `data-label` named by its column, so a
  phone shows every column with no sideways scroll; the change table names
  its cells the same way. `table::view::data_table_unstacked` is the opt-out
  for a small table that fits a phone as it is (FerroHEALTH/FerroUI#56).
- `widget::tree::tree_view_collapsing`: the console is told the key of every
  item that closes (FerroHEALTH/FerroUI#47). The gallery's tree keeps its
  open items in its address, and a journey closes an item by the toggle and
  by the keyboard and reloads.
- A fieldset set apart as a well (`fui-fieldset fui-well`) keeps the well's
  frame and padding, its legend inside (FerroHEALTH/FerroUI#68).
- The `--control` and `--control-small` heights, shared by buttons, inputs
  and selects, and a definition list's pair may sit in a `<div>` of its own.
- The README's rule for what the top bar folds first.

### Changed

- The upload field's text area is the highlighted editor, numbered, its
  grammar read from the text (XML for an OPT, JSON for a JSON document), its
  placeholder in the muted ink. An ancestor with the new `fui-editor-short`
  class makes an editor 6rem tall for a short value.
- The minimum supported Rust is 1.97, checked on the host and the
  `wasm32-unknown-unknown` target, so a console on Rust 1.97 builds the
  crate (FerroEHR's minimum).
- A link is the accent with an underline under the pointer; a link in running
  prose keeps its underline, and every component that draws an anchor sets
  its own decoration (FerroHEALTH/FerroUI#38). A link in a table cell keeps
  that look whatever a console's own anchor style says.
- The select draws FerroUI's chevron (Lucide's chevron-down, a 16 px box
  inset 12 px) with its room reserved and the chosen words ending in an
  ellipsis, at the text input's height and frame, with a disabled state; the
  date and time inputs share the frame (FerroHEALTH/FerroUI#60,
  FerroHEALTH/FerroUI#71).
- A fieldset's legend is a step up from a field label, with a rule under it
  (FerroHEALTH/FerroUI#73); a checkbox in a row of fields sits on the
  controls' line (FerroHEALTH/FerroUI#74).
- Stacked rows are cards with the row's name as their title; a long token in
  a card title wraps inside the card (FerroHEALTH/FerroUI#66); the phone
  drawer's Close stands apart from the navigation (FerroHEALTH/FerroUI#67).
- The button docs say an inline remove takes the danger outline and a
  disabled action names its reason through `aria-describedby`
  (FerroHEALTH/FerroUI#76).
- A disabled button keeps its variant's look, faded, so it is never read as
  an enabled secondary action (FerroHEALTH/FerroUI#61); one label style for
  every field.
- The value axis of the line and activity charts steps in round numbers
  (FerroHEALTH/FerroUI#62).
- The line chart's last shown series cannot be hidden, and each legend chip
  names its series in `data-series` (FerroHEALTH/FerroUI#49).
- A definition grid breaks a value at its spaces and hyphens, and keeps the
  card's gap after a title (FerroHEALTH/FerroUI#63, FerroHEALTH/FerroUI#52).
- An empty state inside a card or a panel takes the quiet form
  (FerroHEALTH/FerroUI#64).
- Below 40rem the tab strip is one row that scrolls inside itself with a
  shade at the edge that has more tabs (FerroHEALTH/FerroUI#65).
- The top bar keeps one row at every width: the lockup's accent word folds
  away, and a status pill in the end keeps its full words, shown from 64rem
  with the new `fui-roomy-only` utility, never cut to its dot
  (FerroHEALTH/FerroUI#41).
- A pager over one page draws the range and the sizes alone
  (FerroHEALTH/FerroUI#39).
- A page header in a stack leaves the stack's gap and no more
  (FerroHEALTH/FerroUI#57); fields side by side keep their labels and
  controls on one line whatever help each carries (FerroHEALTH/FerroUI#58).
- A button in a table's actions cell sits on the row's first line.
- Below 30rem the stat grid holds one stat per row, so every label stays on
  one line (FerroHEALTH/FerroUI#46).
- A status's detail reads in the status's own tone, which holds 4.5:1 on its
  ground in both themes (FerroHEALTH/FerroUI#43).

### Fixed

- A button's label breaks between words when it cannot fit its column, its
  glyph beside the first line, so a long label never scrolls a phone page
  sideways (FerroHEALTH/FerroUI#78).
- A reveal nested in a part of another answers to its own choice alone
  (FerroHEALTH/FerroUI#79).
- A form in a table's actions cell sits on the row beside its links
  (FerroHEALTH/FerroUI#80).
- The scroll padding under the sticky action bar matches the bar's height,
  so a control the browser scrolls into view lands clear of it (WCAG 2.2
  SC 2.4.11), pinned by a gallery journey that tabs through a long form
  (FerroHEALTH/FerroUI#83).

- From 64rem up, an identifier longer than its column wraps inside the cell,
  so a table never runs past its card (FerroHEALTH/FerroUI#53).
- A bullet item with a long URL wraps inside its card on a phone
  (FerroHEALTH/FerroUI#54).
- An activity chart over one day marks its point, where it drew nothing.
- A page heading in the body face writes no empty `class` attribute
  (FerroHEALTH/FerroUI#55).
- An unavailable pager step reads "First (unavailable)", pinned by a test
  (FerroHEALTH/FerroUI#45).
- A tree item carries `aria-level`, where it wrote a literal
  `attr:aria-level` (FerroHEALTH/FerroUI#48).
- The scroll padding beside the sticky top bar, which keeps a focused
  control clear of it (WCAG 2.2 SC 2.4.11), is pinned by a test and by a
  gallery journey that moves the focus back to a control above the fold
  (FerroHEALTH/FerroUI#40).
- A stacked card's loading bars and empty state span the card, and a
  panel's stacked cards keep the panel's inset.

## [0.1.1] - 2026-10-18

Every component the four FerroHEALTH consoles draw, compared across them and
restyled to one visual language, so FerroEHR, FerroFED and FerroTERM can drop
their own component kits, thaw, leptos-use and leptos-chartistry. This
release changes public signatures (listed under Changed); under strict
semantic versioning before 1.0 that would be a minor bump, and it ships as
0.1.1 by the owner's call.

### Added

- Overlays: `widget::dialog::modal_dialog`, a native `<dialog>` a signal
  opens, narrow or wide, plain or danger; `view::dialog::danger_dialog`;
  `widget::side_panel`, the panel an operator opens from a menu.
- Upload: `widget::upload::upload_field`, choose a file, drop it, or paste
  its text, read in the browser and refused by type or size in words.
- Charts drawn as SVG on the server with no style attribute:
  `view::chart::line_svg` and `widget::chart::line_chart` (several series, a
  time axis, a legend that hides series, the figures as a table),
  `view::chart::activity_chart`, `view::chart::proportion`, and the chart
  palette `--chart-1` to `--chart-6`.
- `widget::copy::copy_button` and `widget::document::document_pane`, a wire
  document highlighted, raw or rendered, with its format choice and a copy.
- `widget::tree::tree_view`: nested items that fold, one tab stop, the arrow
  keys, Home and End, lazy children, links and row actions (WAI-ARIA
  Authoring Practices, the tree view pattern).
- `widget::picker::picker_field`, choose from what the server holds or type
  another; `widget::help` with one help switch per form and
  `explained_field`; `view::form::{check_field, chip_choices, PROSE, NARROW,
  MEDIUM, SWITCH}`, `fieldset` and `columns`.
- Segmented controls: `widget::segmented`, `view::segmented::{segmented_links,
  choice_group, choice_reveal, segmented_form}`.
- Layouts: `view::layout::{three_panes, master_detail, main_aside, stack_at,
  divided_list, field_row, action_bar, tile_list, bullet_list}` and
  `view::surface::{cluster, split, stat_grid, wide_card, panel,
  titled_panel, link_card, facts_row, well, chip, page_meta}`.
- The top bar's middle (`view::shell::topbar_with`) and
  `view::search::search_box`; `view::shell::brand_mark`; the theme toggle
  `view::theme_switch::theme_toggle` and `widget::theme::theme_toggle`
  (one 32 px icon button, a moon in the light theme and a sun in the dark,
  a plain form post with no script that a hydrated console swaps in place
  and persists through its callback) and `density_switch`, with `theme::Density::ALL`, `label`, `Display` and
  `FromStr`.
- `view::surface::page_heading` with breadcrumbs (`Crumb`) and a title in
  the code face; `view::notice::notice`; `view::empty::{busy, Busy,
  empty_quiet}`; `view::tabs::keyed_tabs`; `view::change::change_table`.
- Paging: `table::model::{Noun, Sizes, OpenWindow, partial_note}`,
  `Paging::read_within` and `write_within`, `Window::within` and
  `range_of`; `table::view::{Listing, open_pager, link_open_pager}` and
  `widget::pager::session_open_pager`; `Column::owned` and `mono`; every
  pager step carries `data-page`.
- The refusal names the request, keeps an unreadable body as evidence and
  shows an issue's detail codes (`Refusal::request`, `evidence`,
  `Issue::details`, `Issue::new`); a refusal with no status says "No
  answer".
- The sign-in card takes several notices, link words read at run time and
  stable ids (`Notice::new`, `Notice::id`, `sign_in::REFUSAL_ID`).
- A leading per step of type (`--leading-display` to `--leading-micro`),
  written wherever a step is.
- 32 Lucide glyphs, 82 in all, `rotate-ccw-clock` among them (Lucide's
  `history`).
- Quality control: the side-by-side comparison under `docs/qa/`, the
  interaction states captured in every brand, light and dark
  (`tests/it/states.rs`), and the gallery's Overlays, Charts and Layouts
  pages.

### Changed

- `Stat::label` is a `Cow<'static, str>`; `Column` holds `Cow` fields and
  is no longer `Copy`; `Refusal` and `Issue` gain fields; `SignIn::notice`
  became `notices` and `links` takes owned words. Consoles that build these
  with struct literals add the new fields (or use the constructors).
- `pager`, `link_pager` and `session_pager` take `impl Into<Listing>` (a
  `&'static str` still works) and return `AnyView`; the range names the
  listing's noun.
- `operator_menu`, `topbar`, the field, card and panel views take `'static`
  bodies and erase them, so a console's page stays within the compiler's
  query depth.
- The look follows the best of the four consoles, harmonized: FerroEHR's stat
  tile, pill, tabs and segmented control; a 20 px page title and 14 px card
  titles; one hover and focus treatment; the sidebar marks one entry
  current; the table header in the small step and sentence case; the
  compact density at FerroTERM's rhythm.
- `css/tailwind-theme.css` is an `@theme inline reference` block, so it no
  longer writes self-referencing variables that unset the type, radius and
  shadow tokens in a console's Tailwind build.
- Charts keep the type scale at every width: the outer SVG has no
  `viewBox`, the labels sit at percentages on a halo, and the lines are
  drawn in a stretched inner SVG with strokes that do not scale.
- On a phone (below 40 rem) a pager's steps are glyphs with their words
  kept for assistive technology, the operator's name is visually hidden,
  stat tiles take two to a row, an issue's head and text stack beside its
  badge, and a row of fields keeps its actions at their own width.
- A fact across a row takes its own width, so an identifier stays on one
  line; an identifier in a table cell keeps its line and the table scrolls;
  a card in a grid row that holds an empty state centres it in the row's
  height.
- `Variant::Destructive`, the solid danger button that confirms a danger
  dialog (a danger button in a danger dialog's actions draws solid too);
  dialogs, menus, toasts and the side panel take the `--shadow-overlay`
  elevation.
- A disabled control draws neutral, on the sunken ground in muted text with
  no accent; a pager step that cannot be taken says "(unavailable)" in
  visually hidden text; every pager is one row with the range, the steps and
  the sizes.
- Chart series take the role tones (`chart::ACCENT`, `INFO`, `OK`,
  `WARN`, `BAD`, `NEUTRAL`) in place of the `--chart-1` to `--chart-6`
  palette; the Y labels sit in a gutter left of the plot; a line chart marks
  its points and bridges a missing value with a dashed line its legend
  explains.
- A table's row header is in the body's weight.
- The gallery's browser tests run at most four at once, and its bundle
  budget is 600 KiB of brotli-compressed WebAssembly: 0.1.1 measures 518566
  bytes with every component in it (0.1.0: 325387).

## [0.1.0] - 2026-10-18

The first release of `ferrohealth-ui`, the shared components of the
FerroHEALTH consoles.

### Added

- The stylesheet `css/ferrohealth-ui.css`: role tokens for the light and
  dark themes, a type scale of six steps and four spacing steps, a compact
  density, and every `fui-` component class, in Tailwind 4's layer order
  (`style::STYLESHEET`), with `css/tailwind-theme.css` mapping the roles to
  Tailwind utilities (`style::TAILWIND_THEME`).
- A theme file per product (`themes/ferrobridge.css`, `ferrofed.css`,
  `ferroehr.css`, `ferroterm.css`, `style::Brand::theme`), each setting the
  five accent properties in three blocks. FerroEHR's light accent is teal
  700 and its dark tint translucent, the smallest moves from its viewer's
  palette that pass WCAG 2.2 AA.
- Static view functions (`view`): buttons, pills, badges, statuses, banners,
  page header, cards, stats, definitions, disclosure, form fields and checks,
  link tabs, empty states and skeletons, refusals with their issues, code
  panels and numbered code lines, the theme switch, the operator menu, the
  confirmation dialog, the shell with skip link, top bar and footer, a centred
  page, and the sign-in card with a basic form, a provider button, or both.
- Interactive widgets (`widget`): the navigation sidebar and drawer, keyboard
  tabs, toasts, the highlighted overlay editor with line marks, the session
  pager and the table filter.
- One table model (`table`): defensive paging (sizes 25, 50 and 100, at most
  200), the row window and its range label, `aria-sort` sorting, the URL or
  session state source, and the data table with its link pager. The table
  shows every row of its page; from 64rem up its header sticks below the
  top bar, and below 64rem it scrolls sideways inside its card.
- Highlighters for YAML, JSON, XML and AQL (`highlight::html`).
- 50 Lucide 1.51.0 icons, vendored verbatim with their ISC licence
  (`icon::icon`).
- The contrast test over every brand and theme, and the stylesheet rules
  test.
- The gallery (`app/ferrohealth-ui-gallery`, never published): every
  component in every product theme, light and dark, with browser journeys,
  a painted-colour check and the screenshots under `docs/screenshots/`.

[Unreleased]: https://github.com/FerroHEALTH/FerroUI/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.8...v0.6.0
[0.5.8]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.7...v0.5.8
[0.5.7]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.6...v0.5.7
[0.5.6]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.5...v0.5.6
[0.5.5]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/FerroHEALTH/FerroUI/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/FerroHEALTH/FerroUI/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/FerroHEALTH/FerroUI/compare/v0.3.7...v0.4.0
[0.3.7]: https://github.com/FerroHEALTH/FerroUI/compare/v0.3.6...v0.3.7
[0.3.6]: https://github.com/FerroHEALTH/FerroUI/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/FerroHEALTH/FerroUI/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/FerroHEALTH/FerroUI/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/FerroHEALTH/FerroUI/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/FerroHEALTH/FerroUI/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/FerroHEALTH/FerroUI/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/FerroHEALTH/FerroUI/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/FerroHEALTH/FerroUI/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/FerroHEALTH/FerroUI/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.7...v0.2.0
[0.1.7]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/FerroHEALTH/FerroUI/releases/tag/v0.1.0
