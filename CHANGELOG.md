<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-10-18

The fixes the owner's review of the four consoles asked for: links, fields,
selects, buttons, definition grids, empty states, tabs, the top bar, the
pager and the charts, each fixed once here so every console gains it. No
public signature changes; the additions are new functions and classes.

### Added

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
- `table::view::data_table_stacked`: below 40rem each row is a block, every
  cell with a `data-label` named by its column, so a phone shows every column
  (FerroHEALTH/FerroUI#56).
- `widget::tree::tree_view_collapsing`: the console is told the key of every
  item that closes (FerroHEALTH/FerroUI#47).
- A fieldset set apart as a well (`fui-fieldset fui-well`) keeps the well's
  frame and padding, its legend inside (FerroHEALTH/FerroUI#68).
- The `--control` and `--control-small` heights, shared by buttons, inputs
  and selects, and a definition list's pair may sit in a `<div>` of its own.
- The README's rule for what the top bar folds first.

### Changed

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
  away, then a status pill in the end shortens to an ellipsis
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
  control clear of it (WCAG 2.2 SC 2.4.11), is pinned by a test
  (FerroHEALTH/FerroUI#40).

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

[Unreleased]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/FerroHEALTH/FerroUI/releases/tag/v0.1.0
