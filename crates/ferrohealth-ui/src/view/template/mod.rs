// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page templates every console page is built on.
//!
//! They are T1 the list page ([`list`]), T2 the detail page ([`detail`]),
//! T3 the settings and reference page ([`settings`]), and the simple About
//! page ([`about`]). Each opens with [`crate::view::surface::page_heading`]
//! and draws its optional parts only where they earn their place (family
//! rule 8): a list of ten items or fewer has no summary strip and no
//! filter menus, the active filter chips show only while a filter is set,
//! and a file-only configuration has no mode badge and no per-row
//! Next start marker. The rhythm is fixed: 24 px under the page header
//! (its own margin), then 16 px between the parts.
//!
//! Each template is a server view that takes plain values and erased
//! parts, so it ships no WebAssembly; the parts a console brings (a table,
//! a form, binder tabs, an action group) arrive as [`leptos::prelude::AnyView`].
//! No specification governs the templates: our own design.

pub mod about;
pub mod detail;
pub mod list;
pub mod settings;

use leptos::prelude::*;

/// Returns the binder tabs `tabs` joined to the one panel under them,
/// holding `panel`, with the panel's extra `class` when it has one.
fn tabbed(tabs: AnyView, panel: AnyView, class: &'static str) -> AnyView {
    // TODO(#152): the binder tabs component draws the row and its panel as one.
    view! {
        <div class=class>
            {tabs}
            <div class="fui-tab-panel">{panel}</div>
        </div>
    }
    .into_any()
}
