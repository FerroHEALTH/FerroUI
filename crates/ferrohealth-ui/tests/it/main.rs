// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The integration tests of the library, one binary: the contrast of every
//! role pairing in every brand's themes, the stylesheet's own rules, and
//! every component rendered to HTML on the host.
#![cfg(not(target_arch = "wasm32"))]
// NOTE: The Rust Reference, "Limits", `recursion_limit`: every view rendered here stays
// 48 levels under rustc's default 128, which a console's own nesting spends (FerroUI #450).
#![recursion_limit = "80"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::missing_assert_message,
    reason = "test assertions"
)]

mod chrome;
mod contrast;
mod controls;
mod dock;
mod fields;
mod filter;
mod kbd;
mod markup;
mod preferences;
mod render;
mod shell;
mod state;
mod stylesheet;
mod table;
mod templates;
