// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! FerroUI: the shared UI components of the FerroHEALTH consoles.
//!
//! The library has three parts a page draws with, for a console that Leptos
//! renders in islands mode. [`view`] holds static view functions: plain
//! functions returning a typed view, with no reactive props and no
//! `#[component]`, drawn on the server alone and shipped to the browser as
//! HTML. [`widget`] holds the interactive components, each where
//! interaction needs browser code, as functions a page calls inside an
//! island. [`island`] holds the widgets that need nothing from the console's
//! code as ready islands, such as the toast region, the navigation drawer,
//! the user menu, the keyboard tabs, the action menus and the date picker.
//! [`table`] is the one paging and sorting model every listing shares,
//! [`calendar`] the days a date field counts in, [`highlight`] the
//! tokenizers of YAML,
//! JSON, XML and AQL, [`icon`] the vendored Lucide outlines, [`tone`] and
//! [`theme`] the status tones and the theme choice, [`i18n`] the locale and
//! the catalogs of every word the components draw, and [`style`] the
//! stylesheet and the brand themes as text a console serves.
//!
//! Every page works with no WebAssembly: what an island adds is behaviour on
//! top of complete server-rendered HTML, with a link or a form behind every
//! action. Data arrives as props, nothing fetches, and browser-only code
//! runs inside an event handler or an effect. No component writes a
//! `style` attribute or a `<style>` element, so a console keeps
//! `style-src 'self'`, and no component defines a server function or
//! touches browser storage. No specification governs the components: our
//! own design.
#![doc(test(attr(deny(warnings))))]

pub mod calendar;
pub mod highlight;
pub mod i18n;
pub mod icon;
pub mod island;
pub mod style;
pub mod table;
pub mod theme;
pub mod tone;
pub mod view;
pub mod widget;
