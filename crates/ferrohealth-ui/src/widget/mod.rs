// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The interactive components: each needs browser code for what HTML alone
//! does not do.
//!
//! The browser code runs only inside an event handler or an effect, behind
//! `#[cfg(target_arch = "wasm32")]`, so each widget renders the same on the
//! server, in a hydrated page and under client-side rendering, and its first
//! paint is the server's. A widget is a plain function where its state fits
//! in its arguments, and none fetches.

pub mod chart;
pub mod combobox;
pub mod copy;
pub mod dialog;
pub mod document;
pub mod editor;
pub mod filter;
pub mod help;
pub mod nav;
pub mod pager;
pub mod picker;
pub mod segmented;
pub mod side_panel;
pub mod tabs;
pub mod theme;
pub mod toast;
pub mod tree;
pub mod upload;
