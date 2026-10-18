// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tabs that lead between the pages of one place.
//!
//! They are plain links, the one for the current page marked
//! `aria-current`, so they work with no script and keep their state in the
//! URL. The keyboard tabs that switch panels in place are
//! [`crate::widget::tabs`].

use leptos::prelude::*;

/// One tab: where it leads and what it says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tab {
    /// The path the tab leads to.
    pub href: &'static str,
    /// What the tab says.
    pub label: &'static str,
}

/// Returns the link tabs `tabs` named `label`, the one whose path is
/// `current` marked as the page, by its edge and weight as well as its
/// colour.
#[must_use]
pub fn link_tabs(label: &'static str, tabs: &[Tab], current: &str) -> impl IntoView + use<> {
    let tabs = tabs
        .iter()
        .map(|tab| {
            let here = (tab.href == current).then_some("page");
            view! {
                <a href=tab.href class="fui-tab" aria-current=here>
                    {tab.label}
                </a>
            }
        })
        .collect_view();
    view! {
        <nav class="fui-tabs" aria-label=label>
            {tabs}
        </nav>
    }
}
