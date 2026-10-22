// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's islands: the widgets that own their state and need nothing
//! from a console's code, each ready to place in a page Leptos renders in
//! islands mode.
//!
//! An island is the only part of a page the browser hydrates (the Leptos
//! book, "Guide: Islands"). Its props cross to the browser as JSON in the
//! page, so they are plain data, and the words it draws are among them,
//! resolved on the server in the page's locale. Each island here is
//! a thin wrapper over its widget function in [`crate::widget`], so a
//! console that needs the widget beside state of its own calls that function
//! inside an island of its own instead.
#![expect(
    clippy::must_use_candidate,
    reason = "the #[island] macro drops a #[must_use] from the function it writes"
)]
#![expect(
    missing_docs,
    reason = "the #[island] macro writes an undocumented struct of each island's props as they cross to the browser"
)]
#![expect(
    clippy::mem_forget,
    reason = "the island macro of leptos_macro 0.8 keeps each hydrated island alive by forgetting its view state, in the browser entry it writes"
)]

use leptos::prelude::*;

use crate::widget::toast::{Flash, RegionWords};

/// The toast region, which draws `flash` as the page arrives and every
/// toast [`crate::widget::toast::raise`] shows after. A console draws it
/// once, in its shell.
#[island]
pub fn ToastRegion(
    /// The words it draws, in the page's locale
    /// ([`RegionWords::of_page`]).
    words: RegionWords,
    /// The toasts the server shows as the page arrives.
    flash: Vec<Flash>,
) -> impl IntoView {
    crate::widget::toast::toast_region(words, flash)
}

/// The keyboard tabs `prefix`, named `label`, one tab per word in `tabs`,
/// over the panels [`crate::widget::tabs::tab_panels`] drew as its
/// children.
#[island]
pub fn KeyboardTabs(
    /// The tab list's name, in the page's locale.
    label: String,
    /// The prefix of every tab's and panel's id.
    prefix: String,
    /// Each tab's words, in the page's locale.
    tabs: Vec<String>,
    /// The panels, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::tabs::tab_list(label, prefix, tabs, children().into_any())
}

/// The view switch of the document pane `id`, named `label`, one link per
/// choice in `views`, over the views the server drew as its children; the
/// view `opening` shows first. A console draws it through
/// [`crate::view::document::document_pane`].
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn DocumentViews(
    /// The pane's id, which each view's id starts with.
    id: String,
    /// The switch's name, in the page's locale.
    label: String,
    /// The views offered, each with its words in the page's locale.
    views: Vec<crate::widget::document::ViewChoice>,
    /// The view shown first.
    opening: crate::widget::document::DocumentView,
    /// The views, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::document::view_switch(&id, label, views, opening, children().into_any())
}

/// The key hint of a setting, copying its `key` when pressed (R15). The
/// settings template draws it on each setting's label line.
#[island]
pub fn KeyHint(
    /// The configuration key it names and copies.
    key: String,
    /// Its words, in the page's locale ([`crate::widget::copy::KeyHintWords::of_page`]).
    words: crate::widget::copy::KeyHintWords,
) -> impl IntoView {
    crate::widget::copy::key_hint(key, words)
}

/// The navigation drawer `id`, named `label`, around the entries the
/// server drew as its children, its close button saying `close`, folded to
/// its glyphs while `rail`. A console draws it through
/// [`crate::widget::nav::sidebar`].
#[island]
pub fn NavDrawer(
    /// The navigation's id, which the drawer button opens.
    id: String,
    /// The navigation's name, in the page's locale.
    label: String,
    /// The close button's words, in the page's locale.
    close: String,
    /// Whether the navigation is folded to its rail of glyphs.
    rail: bool,
    /// The groups of entries, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::nav::drawer(id, label, close, rail, children().into_any())
}

/// The user menu `id`, named `label`, its button showing `name`, around the
/// items the server drew as its children, its sheet's Cancel saying
/// `cancel`. A console draws it through [`crate::widget::menu::user_menu`].
#[island]
pub fn UserMenuButton(
    /// The menu's id, which its button opens.
    id: String,
    /// The menu's name and its sheet's title, in the page's locale.
    label: String,
    /// The name its button shows, or `None` for its icon alone.
    name: Option<String>,
    /// The sheet's Cancel, in the page's locale.
    cancel: String,
    /// The items, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::menu::menu_button(id, label, name, cancel, children().into_any())
}
