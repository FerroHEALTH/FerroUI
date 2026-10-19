// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tabs that lead between the pages of one place.
//!
//! They are plain links, the one for the current page marked
//! `aria-current`, so they work with no script and keep their state in the
//! URL. The keyboard tabs that switch panels in place are
//! [`crate::widget::tabs`].

use std::borrow::Cow;

use leptos::prelude::*;

/// One tab: where it leads and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// The path the tab leads to.
    pub href: &'static str,
    /// What the tab says.
    pub label: Cow<'static, str>,
}

/// One tab whose path is known only at run time, such as a detail screen's
/// tab kept in its query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyedTab {
    /// The key that names the tab, which the current one is matched by.
    pub key: &'static str,
    /// The href the tab leads to.
    pub href: String,
    /// What the tab says.
    pub label: Cow<'static, str>,
}

/// Returns the link tabs `tabs` named `label`, marked by key.
///
/// The one whose key is `current` is marked as the page: the form of [`link_tabs`] whose hrefs
/// differ only in their query, so the path cannot say which is current.
#[must_use]
pub fn keyed_tabs(label: impl Into<String>, tabs: Vec<KeyedTab>, current: &str) -> AnyView {
    let label = label.into();
    let tabs = tabs
        .into_iter()
        .map(|tab| {
            let here = (tab.key == current).then_some("page");
            view! {
                <a href=tab.href class="fui-tab" aria-current=here data-tab=tab.key>
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
    .into_any()
}

/// Returns the link tabs `tabs` named `label`, the one whose path is
/// `current` marked as the page, on the accent's tint and named current.
#[must_use]
pub fn link_tabs(label: impl Into<String>, tabs: &[Tab], current: &str) -> AnyView {
    let label = label.into();
    let tabs = tabs
        .iter()
        .map(|tab| {
            let here = (tab.href == current).then_some("page");
            view! {
                <a href=tab.href class="fui-tab" aria-current=here>
                    {tab.label.clone()}
                </a>
            }
        })
        .collect_view();
    view! {
        <nav class="fui-tabs" aria-label=label>
            {tabs}
        </nav>
    }
    .into_any()
}
