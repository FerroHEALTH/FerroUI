// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The navigation: groups of entries under their headings, the current page
//! marked, a column beside the page from the first breakpoint and a drawer
//! below it.
//!
//! The drawer is an HTML popover the top bar's button opens, so the browser
//! opens it, closes it on Escape and returns the focus to the button with
//! no script. The island [`crate::island::NavDrawer`] adds what HTML leaves
//! out: the focus moves into the drawer as it opens, and the drawer closes
//! when one of its links is followed (WCAG 2.2 SC 2.4.3). The entries are
//! plain links the server draws, the current page marked
//! `aria-current="page"`, so following one loads the page, with or without
//! WebAssembly.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// One entry of the navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Where it leads.
    pub href: &'static str,
    /// What it says.
    pub label: Cow<'static, str>,
    /// The icon beside it.
    pub glyph: Glyph,
    /// Whether it is marked on its own path alone, rather than on every
    /// path under it too (a section root such as `/` or a brand's home).
    pub exact: bool,
}

/// One group of the navigation, under its heading.
///
/// A group and its entries are built where the navigation is drawn, so
/// their words can come from the console's catalog in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// The id its heading carries, which names its list.
    pub id: &'static str,
    /// Its heading.
    pub title: Cow<'static, str>,
    /// Its entries, in order; a group with none is not drawn.
    pub entries: Vec<Entry>,
}

/// Whether the entry leading to `href` marks the page at `current`: on its
/// own path, or on a path under it too unless it is `exact`.
#[must_use]
pub fn marks(href: &str, exact: bool, current: &str) -> bool {
    let current = current.split(['?', '#']).next().unwrap_or_default();
    let current = current
        .strip_suffix('/')
        .filter(|path| !path.is_empty())
        .unwrap_or(current);
    current == href
        || (!exact
            && current
                .strip_prefix(href)
                .is_some_and(|rest| rest.starts_with('/') || href.ends_with('/')))
}

/// Returns the entry `entry`, marked while `current` is its page, or a
/// page under it unless it is exact.
fn entry(entry: Entry, current: &str) -> impl IntoView + use<> {
    let here = marks(entry.href, entry.exact, current).then_some("page");
    view! {
        <li>
            <a href=entry.href class="fui-nav-entry" aria-current=here>
                {icon(entry.glyph, "16")}
                {entry.label}
            </a>
        </li>
    }
}

/// Returns the group `group`, its entries marked against `current`, its
/// heading naming its list; nothing for a group with no entry.
fn group(group: Group, current: &str) -> Option<impl IntoView + use<>> {
    if group.entries.is_empty() {
        return None;
    }
    let entries = group
        .entries
        .into_iter()
        .map(|each| entry(each, current))
        .collect_view();
    Some(view! {
        <p id=group.id class="fui-nav-group">
            {group.title}
        </p>
        <ul class="fui-nav-list" aria-labelledby=group.id>
            {entries}
        </ul>
    })
}

/// Returns the navigation `id`, named `label`, of `groups`, the page at the
/// path `current` marked, which the button of
/// [`crate::view::shell::drawer_button`] opens as a drawer.
///
/// The groups are drawn on the server and handed to the island
/// [`crate::island::NavDrawer`], which carries the drawer's behaviour.
#[must_use]
pub fn sidebar(
    id: &'static str,
    label: impl Into<String>,
    groups: Vec<Group>,
    current: &str,
) -> AnyView {
    let label = label.into();
    let close = tr(Text::Close);
    let groups = groups
        .into_iter()
        .map(|each| group(each, current))
        .collect_view();
    view! {
        <crate::island::NavDrawer id=id.to_owned() label close>
            {groups}
        </crate::island::NavDrawer>
    }
    .into_any()
}

/// Returns the drawer `id`, named `label`, around `entries`, its close
/// button saying `close`, with the drawer's behaviour. A console draws it
/// through [`sidebar`].
#[must_use]
pub fn drawer(id: String, label: String, close: String, entries: AnyView) -> AnyView {
    let nav = NodeRef::<leptos::html::Nav>::new();
    let button = format!("{} fui-drawer-only", Variant::Ghost.class());
    let target = id.clone();
    view! {
        <nav
            id=id
            popover="auto"
            aria-label=label
            class="fui-sidebar"
            node_ref=nav
            on:toggle=move |_| drawer::opened(nav)
            on:click=move |event| drawer::followed(nav, &event)
        >
            <button type="button" class=button popovertarget=target popovertargetaction="hide">
                {icon(Glyph::X, "16")}
                {close}
            </button>
            {entries}
        </nav>
    }
    .into_any()
}

/// What the browser code adds to the drawer. From the first breakpoint up
/// the navigation is no popover the browser opens, so neither applies.
mod drawer {
    use leptos::prelude::*;

    /// Moves the focus to the drawer's first link, once it is open.
    pub(super) fn opened(nav: NodeRef<leptos::html::Nav>) {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast as _;
            let Some(nav) = nav.get_untracked() else {
                return;
            };
            if !nav.matches(":popover-open").unwrap_or(false) {
                return;
            }
            let first = nav
                .query_selector("a")
                .ok()
                .flatten()
                .and_then(|link| link.dyn_into::<web_sys::HtmlElement>().ok());
            if let Some(first) = first
                && first.focus().is_err()
            {
                leptos::logging::warn!("the browser refused the focus on the navigation");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: NodeRef<leptos::html::Nav> = nav;
    }

    /// Closes the open drawer when `event` follows one of its links.
    pub(super) fn followed(nav: NodeRef<leptos::html::Nav>, event: &leptos::ev::MouseEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast as _;
            let Some(nav) = nav.get_untracked() else {
                return;
            };
            let on_link = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .and_then(|target| target.closest("a").ok().flatten())
                .is_some();
            if on_link
                && nav.matches(":popover-open").unwrap_or(false)
                && nav.hide_popover().is_err()
            {
                leptos::logging::warn!("the browser refused to close the navigation");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (NodeRef<leptos::html::Nav>, &leptos::ev::MouseEvent) = (nav, event);
    }
}

#[cfg(test)]
mod tests {
    use super::marks;

    #[test]
    fn an_entry_marks_its_own_page_and_the_pages_under_it() {
        assert!(marks("/ferrofed/tables", true, "/ferrofed/tables"));
        assert!(
            marks("/ferrofed/tables", true, "/ferrofed/tables?page=2"),
            "a query names the same page"
        );
        assert!(
            marks("/ferrofed/tables", true, "/ferrofed/tables/"),
            "a trailing slash names the same page"
        );
        assert!(
            !marks("/ferrofed", true, "/ferrofed/tables"),
            "an exact entry marks its own page alone"
        );
        assert!(
            marks("/files", false, "/files/synthetic-007"),
            "a section marks the pages under it"
        );
        assert!(
            !marks("/files", false, "/filestore"),
            "a longer name is no page under it"
        );
        assert!(marks("/", true, "/"), "the root marks itself");
        assert!(!marks("/", true, "/ferrofed"));
    }
}
