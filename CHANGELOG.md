<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

### Changed

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

### Removed

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

[Unreleased]: https://github.com/FerroHEALTH/FerroUI/compare/v0.2.2...HEAD
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
