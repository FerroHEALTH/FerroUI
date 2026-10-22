// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The interactive components: each needs browser code for what HTML alone
//! does not do.
//!
//! A widget is a plain function a page calls inside an island, the only
//! part of a page in islands mode the browser hydrates: inside one of the
//! library's own ([`crate::island`]), or inside a console's own island where
//! the widget shares state with the console's code. Its browser code runs
//! only inside an event handler or an effect, behind
//! `#[cfg(target_arch = "wasm32")]`, so it renders the same on the server
//! and in the browser, and its first paint is the server's. None fetches.

pub mod action_menu;
pub mod chart;
pub mod combobox;
pub mod copy;
pub mod dialog;
pub mod document;
pub mod editor;
pub mod filter;
pub mod help;
pub mod menu;
pub mod nav;
pub mod pick_or_type;
pub mod picker;
pub mod segmented;
pub mod side_panel;
pub mod tabs;
pub mod toast;
pub mod tree;
pub mod upload;
