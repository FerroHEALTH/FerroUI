// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The integration tests of the library, one binary: the contrast of every
//! role pairing in every brand's themes, the stylesheet's own rules, and
//! every view rendered to HTML on the host.
#![cfg(not(target_arch = "wasm32"))]
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
mod clinical;
mod code;
mod config;
mod console;
mod contrast;
mod controls;
mod dock;
mod editor;
mod federation;
mod fields;
mod figures;
mod filter;
mod kbd;
mod locale;
mod markup;
mod operations;
mod preferences;
mod records;
mod render;
mod search;
mod shell;
mod state;
mod stylesheet;
mod table;
mod templates;
mod terminology;
mod trees;
