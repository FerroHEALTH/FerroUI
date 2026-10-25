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
//!
//! The widgets that own their state are ready islands. The others are
//! island parts: each takes a signal, a callback or a closure that ties it
//! to the console's own state, which no island prop can carry, so a console
//! calls it inside an island of its own. They are the highlighted editor
//! ([`editor::code_editor`]) and the code editor with its diagnostics
//! ([`editor::editor_panel`], [`editor::editor_diagnostics`]), the list filter ([`filter::table_filter`]),
//! the pager whose steps the console draws ([`crate::table::pager::pager`],
//! as a session pager does), the modal dialog ([`dialog::modal_dialog`]),
//! the tree ([`tree::tree_view`]), the upload field
//! ([`upload::upload_field`]), the help switch with the fields it
//! explains ([`crate::view::help`]), the masked identifier with its reveal
//! ([`identifier::identifier`]) and the work-list with a console's own Enter
//! ([`worklist::worklist`]).

pub mod action_menu;
pub mod announce;
pub mod chart;
pub mod combobox;
pub mod copy;
pub mod countdown;
pub mod date;
pub mod dialog;
pub mod document;
pub mod editor;
pub mod filter;
pub mod focus;
pub mod hydration;
pub mod identifier;
pub mod masked;
pub mod menu;
pub mod nav;
pub mod pick_or_type;
pub mod picker;
pub mod preference;
pub mod prefetch;
pub mod repeat;
pub mod search;
pub mod segmented;
pub mod shortcuts;
pub mod side_panel;
pub mod submit;
pub mod tabs;
pub mod toast;
pub mod topology;
pub mod tree;
pub mod upload;
pub mod version;
pub mod worklist;
