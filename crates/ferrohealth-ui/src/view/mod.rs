// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The static view functions.
//!
//! They are the shell, the surfaces, the status marks, the buttons and
//! fields, the tabs as links, the empty and loading states, the refusal, the
//! code panel, the two sign-in screens, the theme switch, the operator menu
//! and the confirmation dialog. Each is a plain function returning its typed
//! view, never a
//! `#[component]`: a component writes a props type and a builder the browser
//! bundle carries for each, and these take no reactive input. Each renders
//! the same on the server, in a hydrated page and under client-side
//! rendering, and through `to_html()` into a string a read-only screen draws
//! with `inner_html`. What a view needs to behave, it gets from the browser
//! itself: a popover opens and closes with no script, a `<details>` folds,
//! a form posts.

pub mod button;
pub mod change;
pub mod chart;
pub mod code;
pub mod dialog;
pub mod empty;
pub mod form;
pub mod layout;
pub mod menu;
pub mod notice;
pub mod refusal;
pub mod search;
pub mod segmented;
pub mod shell;
pub mod sign_in;
pub mod status;
pub mod surface;
pub mod tabs;
pub mod theme_switch;
