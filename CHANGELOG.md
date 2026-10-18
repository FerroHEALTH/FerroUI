<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
  session state source, and the data table with its link pager.
- Highlighters for YAML, JSON, XML and AQL (`highlight::html`).
- 50 Lucide 1.51.0 icons, vendored verbatim with their ISC licence
  (`icon::icon`).
- The contrast test over every brand and theme, and the stylesheet rules
  test.

[Unreleased]: https://github.com/FerroHEALTH/FerroUI/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/FerroHEALTH/FerroUI/releases/tag/v0.1.0
