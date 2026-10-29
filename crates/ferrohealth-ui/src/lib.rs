// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! FerroUI: the shared UI components of the FerroHEALTH consoles.
//!
//! The library renders HTML on the server. Every component is a Rust
//! struct over an Askama template compiled into the crate, and returns
//! [`html::Html`], markup a console's own template writes with `{{ part }}`
//! or hands to the next component as its child. [`view`] holds the
//! components; [`table`] is the one paging and sorting model every listing
//! shares, [`calendar`] the days a date field counts in, [`highlight`] the
//! tokenizers of YAML, JSON, XML and AQL, [`icon`] the vendored Lucide
//! outlines, [`tone`] and [`theme`] the status tones and the theme choice,
//! [`i18n`] the locale and the catalogs of every word the components draw,
//! [`style`] the stylesheet and the brand themes as text, [`script`] the
//! plain script modules as text, and [`serve`] the router a console mounts
//! to serve them.
//!
//! Every page works with no script: what a module adds is behaviour on
//! top of complete server-rendered HTML, with a link or a form behind every
//! action. No component writes a `style` attribute, a `<style>` element or
//! an inline script, so a console keeps `style-src 'self'` and `script-src
//! 'self'`; no component touches browser storage beyond what the tree keeps
//! of its open items for the session. No specification governs the
//! components: our own design (FerroUI #510).
#![doc(test(attr(deny(warnings))))]

pub mod calendar;
pub mod highlight;
pub mod html;
pub mod i18n;
pub mod icon;
pub mod query;
pub mod script;
pub mod serve;
pub mod style;
pub mod table;
pub mod theme;
pub mod tone;
pub mod view;
