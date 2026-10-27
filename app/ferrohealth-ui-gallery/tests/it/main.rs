// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's tests, one binary: the HTTP surface in-process, and the
//! browser journeys and screenshots behind their gates.
//!
//! The journeys drive headless Chrome over `WebDriver`, through a
//! chromedriver of their own, against the gallery served in-process from its
//! release site bundle, so the browser hydrates the pages the server
//! rendered. Without `FERROUI_JOURNEYS=1` each journey returns at once, and
//! without `FERROUI_SCREENSHOTS=1` the capture does. Each journey fails on
//! any `SEVERE` entry the browser logs and waits on elements, never on a
//! timer. No specification governs the tests: our own design.
#![cfg(not(target_arch = "wasm32"))]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::missing_assert_message,
    reason = "test assertions"
)]

mod browser;
mod clinical;
mod configuration;
mod dates;
mod federation;
mod filter;
mod form_edges;
mod http;
mod journeys;
mod keys;
mod menus;
mod navigation;
mod no_wasm;
mod operations;
mod painted;
mod records;
mod screenshots;
mod session;
mod stack;
mod states;
mod sticky;
mod tabs_and_toasts;
mod terminology;
mod trees;
mod typeahead;
mod widgets;

/// The environment variable that opts in to the browser journeys.
const JOURNEYS_GATE: &str = "FERROUI_JOURNEYS";

/// Whether the browser journeys run: only when [`JOURNEYS_GATE`] is `1`.
fn journeys_enabled() -> bool {
    std::env::var(JOURNEYS_GATE).is_ok_and(|value| value == "1")
}
