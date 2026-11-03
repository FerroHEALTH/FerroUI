// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The one table and paging model every listing shares.
//!
//! [`model`] is the arithmetic: the page and size a request names, read
//! defensively and clamped, the window of rows a page shows, the range
//! label, the sort a column header names, and where that state lives.
//! [`view`] draws the data table, its cells and the list card, [`pager`]
//! the pagers and [`filter`] the find field; [`change`], [`mapping`] and
//! [`versions`] are the tables of a change, a concept map and a version
//! history. The table draws with the view primitives (`crate::view`), and
//! no view primitive draws with the table. Each is a server view whose
//! sort, filter and paging are links and forms, so a listing works with no
//! script and no row reaches the page as script data. A screen that can
//! show patient data keeps its page, size and sort out of the URL
//! ([`model::StateSource::Session`]): its forms post them. Every other
//! screen keeps them in the URL, so the address bar is shareable, the
//! browser's back and forward walk the pages, and the links work with no
//! script.

pub mod change;
pub mod filter;
pub mod mapping;
pub mod model;
pub mod pager;
pub mod versions;
pub mod view;
