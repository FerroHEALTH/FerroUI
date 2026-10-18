// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! FerroUI: the shared UI components of the FerroHEALTH consoles.
//!
//! The library has two tiers. [`view`] holds static view functions: plain
//! functions returning a typed view, with no reactive props and no
//! `#[component]`, so they cost the browser bundle little and render to an
//! HTML string on the server for a read-only screen drawn with `inner_html`.
//! [`widget`] holds the few interactive components, each where interaction
//! needs browser code: the navigation drawer, keyboard tabs, the
//! confirmation dialog, the toast region, the session pager, the table
//! filter and the highlighted editor. [`table`] is the one paging and
//! sorting model every listing shares, [`highlight`] the tokenizers of YAML,
//! JSON, XML and AQL, [`icon`] the vendored Lucide outlines, [`tone`] and
//! [`theme`] the status tones and the theme choice, and [`style`] the
//! stylesheet and the brand themes as text a console serves.
//!
//! Every component works in both halves of a console and under client-side
//! rendering alike: data arrives as props, nothing fetches, and browser-only
//! code runs inside an event handler or an effect. No component writes a
//! `style` attribute or a `<style>` element, so a console keeps
//! `style-src 'self'`, and no component defines a server function or
//! touches browser storage. No specification governs the components: our
//! own design.
#![doc(test(attr(deny(warnings))))]

pub mod highlight;
pub mod icon;
pub mod style;
pub mod table;
pub mod theme;
pub mod tone;
pub mod view;
pub mod widget;
