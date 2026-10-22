// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The static view functions.
//!
//! They are the title row's action group, the shell, the page templates,
//! the surfaces, the status marks, the buttons and fields, the help switch,
//! the tabs as links, the empty and loading states, the refusal, the error
//! page, the code viewer, the document pane, the sign-in card, the theme
//! switch, the confirmation dialog, the popover form and the timestamp.
//! Each is a plain function
//! returning its typed view, never a `#[component]`: a component
//! writes a props type and a builder the browser bundle carries for each,
//! and these take no reactive input. Each renders
//! the same on the server, in a hydrated page and under client-side
//! rendering, and through `to_html()` into a string a read-only screen draws
//! with `inner_html`. What a view needs to behave, it gets from the browser
//! itself: a popover opens and closes with no script, a `<details>` folds,
//! a form posts.

pub mod action;
pub mod button;
pub mod change;
pub mod chart;
pub mod code;
pub mod dialog;
pub mod dock;
pub mod document;
pub mod empty;
pub mod error_page;
pub mod form;
pub mod help;
pub mod kbd;
pub mod layout;
pub mod notice;
pub mod popover;
pub mod refusal;
pub mod search;
pub mod segmented;
pub mod shell;
pub mod sign_in;
pub mod status;
pub mod surface;
pub mod tabs;
pub mod template;
pub mod theme_switch;
pub mod time;
