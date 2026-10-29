<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Quality control: the consoles side by side

Each component of the library is compared with the four consoles' versions
(FerroEHR, FerroFED, FerroTERM, FerroBRIDGE), and the best one on its merits
wins: looks, density, hover and focus, accessibility, keyboard path. The
winner is then restyled to the library's one type scale, spacing scale,
radius and shadow set, hover and focus treatment and role tokens
(`.claude/rules/console-ui.md` §11). The reference screenshots are under
`reference/` (see its `PROVENANCE.md`); the gallery's are under
`../screenshots/`, a page per brand in light and dark and the interaction
states under `../screenshots/states/`.

## Screen pairs

| Reference | Gallery | What to compare |
|---|---|---|
| `reference/ferroehr/dashboard.png`, `dashboard-dark.png` | `screenshots/ferroehr/overview.png`, `overview-dark.png` | stat tiles, cards, type scale, sidebar, top bar |
| `reference/ferroehr/user-menu.png` | `screenshots/states/ferroehr/user-menu.png` | the user menu |
| `reference/ferroehr/scopes-drawer.png` | `screenshots/states/ferroehr/side-panel.png` | the side panel |
| `reference/ferroehr/templates.png`, `templates-admin-delete.png` | `screenshots/ferroehr/tables.png`, `states/ferroehr/dialog-danger.png` | the data table, row actions, the danger dialog |
| `reference/ferroehr/template-detail.png` | `screenshots/ferroehr/code.png` | breadcrumbs, mono title, tabs, the document pane |
| `reference/ferroehr/query-builder.png`, `query-results-chart.png` | `screenshots/ferroehr/charts.png`, `layouts.png` | the line chart and its legend, a form beside its answer |
| `reference/ferroehr/system.png` | `screenshots/ferroehr/overview.png` | definitions and facts |
| `reference/ferroehr/login.png` | `screenshots/ferroehr/sign-in-basic.png` | the sign-in card |
| `reference/ferrofed/dashboard.png` | `screenshots/ferrofed/overview.png` | marked stat cards, the page meta line |
| `reference/ferrofed/members.png` | `screenshots/ferrofed/tables.png` | the table and its pager |
| the design's `templates/` set (`design/tools/render.sh templates`) and each console's `forbidden`, `not-found` and `unreadable` | `screenshots/<brand>/templates/*.png`, `screenshots/<brand>/fault.png` | the page templates T1 to T3, About, and the error page |
| each console's `preferences`, `preferences-saved`, `preferences-keyboard`, `preferences-reset` and `preferences-dark` (`design/tools/render.sh <console>`) | `screenshots/<brand>/preferences.png`, `preferences/language.png`, `preferences/keyboard.png`, `states/<brand>/preferences-saved.png`, `preferences-reset.png`, `preferences-phone.png` | Your preferences: the three tabs, the preview cards, Saved in its slot, the reset confirmation, the phone |
| `reference/ferrofed/configuration-member-form.png` | `screenshots/ferrofed/forms.png`, `states/ferrofed/reveal-replace.png` | fieldsets, the segmented radio and what it reveals, the change table |
| `reference/ferrofed/query-complete.png` | `screenshots/ferrofed/forms.png`, `layouts.png` | checkbox chips, the switch, the action bar, node tiles |
| `reference/ferroterm/overview.png` | `screenshots/ferroterm/overview.png`, `states/ferroterm/focus-search.png` | the search box in the top bar, the type scale |
| `reference/ferroterm/overview.png` | `screenshots/states/ferroterm/stats-compact-320.png`, `stats-compact-390.png` | the compact stat tiles on a phone |
| `reference/ferroehr/templates.png` | `screenshots/states/ferroehr/timestamp-stacked-320.png`, `screenshots/ferroehr/tables-narrow.png` | a timestamp in a stacked card, on one line |
| `reference/ferroterm/expand.png` | `screenshots/ferroterm/layouts.png`, `states/ferroterm/help-on.png`, `picker-other.png` | a runner's form beside its answer, the help switch, the picker |

## Which version won

| Component | Winner | Why |
|---|---|---|
| Stat tile | FerroEHR | the glyph on a 2.5 rem tinted square beside a tabular value reads at a glance; a linked tile shows its link by the accent edge |
| Pill and badge | FerroEHR | an edge and a dot beside the word, so the tone never rides on colour alone |
| Tabs | FerroEHR (`tab_bar.rs`) | rounded tabs, a tint for the current one, a clear hover; FerroFED's underline tabs lose the current tab in dark |
| Segmented control | FerroEHR (`format_view.rs`) | joined segments, the pressed one filled with the accent, the clearest pressed state of the four |
| Sidebar | FerroTERM | one entry current, headings hidden behind a hairline, the densest list that still meets the target size |
| Page header | FerroEHR | breadcrumbs, a mono title for an identifier, actions on the right |
| Type scale | FerroEHR, with FerroTERM's leading per step | a 20 px page title and 14 px card titles; FerroTERM's leadings keep each step's rhythm |
| Table header | FerroTERM | the small step in sentence case reads as words and costs less width than upper case |
| Pager | FerroTERM | the noun in the range, the stated total, the console's sizes, "(unavailable)" in words |
| Empty state | FerroEHR | the dashed frame with the action that fills it, and a quiet form for a section |
| Refusal | FerroTERM | the request, the evidence of an unreadable body and the detail codes, so the reader can act |
| Dialogs | FerroEHR | a native modal with the focus inside, a danger variant with an outline danger button |
| Side panel | FerroEHR (scopes drawer) | slides from the end, a scrim, Escape and the close button return the focus |
| Upload | FerroEHR | drop, choose or paste, refused in words by type and size |
| Charts | FerroEHR | the legend that hides a series, recolouring none; drawn as SVG with the figures as a table |
| Toasts | FerroEHR | one per outcome, the tone's glyph beside the words, a dismiss button |
| Theme control | FerroEHR (owner's choice) | one 32 px icon button in the top bar, a moon in light and a sun in dark, one press flips the theme; FerroUI's form post and server cookie stay behind it, and the three-way choice lives in a settings form |
| Search box | FerroTERM | one field and a Find button, its label read, centred in the top bar |
| Help switch, picker, field kinds | FerroTERM | the help stays named while hidden; the picker never snaps a typed value back |
| Checkbox chips, switch, action bar | FerroFED | the tick and the accent edge both say chosen; the knob's place says on |
| Tree | FerroTERM and FerroEHR | FerroTERM's lazy links and keyboard path, FerroEHR's row actions; indentation by nesting |
| Proportion bar | FerroTERM | the length as an SVG attribute, repeated beside its figure |
| Timestamp | FerroEHR (viewer) | a `<time>` with the UTC reading form cut to the second and the value as sent in its title, never broken; FerroUI adds HTML's machine form in `datetime` |
| Stat tile on a phone | FerroEHR's tile, made compact | below 30 rem the glyph, figure and label share one line in a tile at most 64 px tall, the label cut with an ellipsis and named in its title |

FerroBRIDGE's console was compared for every row and won none: its
components came from FerroUI 0.1.0, which this release replaces.
