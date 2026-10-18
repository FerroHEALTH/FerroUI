// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The one table and paging model every listing shares.
//!
//! [`model`] is the arithmetic: the page and size a request names, read
//! defensively and clamped, the window of rows a page shows, the range
//! label, the sort a column header names, and where that state lives.
//! [`view`] draws the data table and the pager. A screen that can show
//! patient data keeps its page, size and sort out of the URL
//! ([`model::StateSource::Session`], [`crate::widget::pager`]); every other
//! screen keeps them in the URL, so the address bar is shareable, the
//! browser's back and forward walk the pages, and the links work before the
//! page hydrates.

pub mod model;
pub mod view;
