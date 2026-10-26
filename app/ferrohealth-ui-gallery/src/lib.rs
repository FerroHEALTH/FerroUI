// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery of the FerroHEALTH UI components: every component in every
//! product theme, light and dark, rendered on the server, with its interactive
//! components hydrated in the browser as islands, as the consoles render them.
//!
//! It is the library's visual review surface and its proof in CI: the
//! journeys drive it in a browser and its screenshots show every page in
//! every brand. [`app`] is the page tree, [`pages`] the pages, [`data`] the
//! synthetic rows and texts they show, [`locale`] English and the
//! pseudo-locale, and [`server`] the HTTP service with
//! the security headers a console sends. Nothing it shows is clinical data:
//! every row is invented for the gallery. No specification governs the
//! gallery: our own design.

pub mod acknowledge;
pub mod app;
pub mod clinical_data;
pub mod cookies;
pub mod data;
pub mod islands;
pub mod locale;
pub mod pages;
pub mod records_data;
pub mod routes;
pub mod search;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;
pub mod terminology_data;
pub mod trees_data;

/// Hydrates the page the server rendered, whole, its lazy routes loading
/// their code from their own chunks.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    ferrohealth_ui::widget::hydration::explain();
    leptos::mount::hydrate_lazy(app::App);
}
