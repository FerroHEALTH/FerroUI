// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery of the FerroHEALTH UI components: every component in every
//! product theme, light and dark, drawn on the server as the consoles draw
//! them, with the library's plain script modules on top.
//!
//! It is the library's visual review surface and its proof in CI: the
//! journeys drive it in a browser and its screenshots show every page in
//! every brand. [`server`] is the HTTP service with the security headers a
//! console sends, [`routes`] picks the page a path names, [`app`] draws the
//! document and the shell, [`pages`] the pages, [`data`] the synthetic rows
//! and texts they show and [`locale`] English and the pseudo-locale.
//! Nothing it shows is clinical data: every row is invented for the
//! gallery. No specification governs the gallery: our own design.

pub mod app;
pub mod clinical_data;
pub mod data;
pub mod locale;
pub mod markup;
pub mod pages;
pub mod records_data;
pub mod routes;
pub mod search;
pub mod server;
pub mod session;
pub mod terminology_data;
pub mod trees_data;
pub mod visit;
