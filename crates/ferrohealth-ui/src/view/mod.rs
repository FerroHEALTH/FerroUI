// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The components: each a plain function over an Askama template that
//! returns [`crate::html::Html`].
//!
//! They are the shell, the navigation and the menus, the page templates,
//! the surfaces, the status marks, the buttons and fields, the help switch,
//! the tag, the tabs, the empty states, the refusal, the error page, the
//! dialogs, the toasts and the rest. Each takes plain values and the
//! children a page composed, as `Html`, and the locale where it draws a
//! word of its own; what a component needs to behave it gets from the
//! browser: a popover opens and closes with no script, a `<details>`
//! folds, a form posts, and a script module adds the keys and the in-page
//! switches on top.

pub mod action;
pub mod action_menu;
pub mod button;
pub mod change;
pub mod chart;
pub mod clause;
pub mod code;
pub mod coding;
pub mod combobox;
pub mod compare;
pub mod conditions;
pub mod copy;
pub mod countdown;
pub mod date;
pub mod diagnostics;
pub mod dialog;
pub mod dock;
pub mod document;
pub mod editor;
pub mod empty;
pub mod error_page;
pub mod filter;
pub mod flow;
pub mod form;
pub mod help;
pub mod hl7v2;
pub mod identifier;
pub mod kbd;
pub mod layout;
pub mod lifecycle;
pub mod load_report;
pub mod mapping;
pub mod masked;
pub mod menu;
pub mod nav;
pub mod notice;
pub mod path;
pub mod pick_or_type;
pub mod picker;
pub mod popover;
pub mod preferences;
pub mod reading;
pub mod refusal;
pub mod replica;
pub mod results;
pub mod root_matrix;
pub mod row_preview;
pub mod segmented;
pub mod shell;
pub mod shortcuts;
pub mod side_panel;
pub mod sign_in;
pub mod state;
pub mod status;
pub mod subject;
pub mod surface;
pub mod tabs;
pub mod tag;
pub mod template;
pub mod time;
pub mod timeline;
pub mod toast;
pub mod transform;
pub mod version;
pub mod versions;
