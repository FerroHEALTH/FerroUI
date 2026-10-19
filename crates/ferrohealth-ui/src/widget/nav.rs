// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The navigation: groups of entries under their headings, the current page
//! marked, a column beside the page from the first breakpoint and a drawer
//! below it.
//!
//! The drawer is an HTML popover the top bar's button opens, so the browser
//! opens it, closes it on Escape and returns the focus to the button before
//! the page has hydrated and with no script. The browser code adds what HTML
//! leaves out: the focus moves into the drawer as it opens, and the drawer
//! closes when one of its links is followed (WCAG 2.2 SC 2.4.3). The entries
//! are the router's links, which mark themselves `aria-current="page"`, so
//! the mark follows a client-side navigation.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// One entry of the navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    /// Where it leads.
    pub href: &'static str,
    /// What it says.
    pub label: &'static str,
    /// The icon beside it.
    pub glyph: Glyph,
    /// Whether it is marked on its own path alone, rather than on every
    /// path under it too (a section root such as `/` or a brand's home).
    pub exact: bool,
}

/// One group of the navigation, under its heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    /// The id its heading carries, which names its list.
    pub id: &'static str,
    /// Its heading.
    pub title: &'static str,
    /// Its entries, in order; a group with none is not drawn.
    pub entries: &'static [Entry],
}

/// Returns the entry `entry`, marked while it is the page, or a page under
/// it unless it is exact.
fn entry(entry: Entry) -> impl IntoView {
    let exact = entry.exact;
    view! {
        <li>
            <A href=entry.href exact=exact attr:class="fui-nav-entry">
                {icon(entry.glyph, "16")}
                {entry.label}
            </A>
        </li>
    }
}

/// Returns the group `group`, its heading naming its list; nothing for a
/// group with no entry.
fn group(group: Group) -> Option<impl IntoView> {
    if group.entries.is_empty() {
        return None;
    }
    let entries = group.entries.iter().copied().map(entry).collect_view();
    Some(view! {
        <p id=group.id class="fui-nav-group">
            {group.title}
        </p>
        <ul class="fui-nav-list" aria-labelledby=group.id>
            {entries}
        </ul>
    })
}

/// Returns the navigation `id`, named `label`, of `groups`, which the
/// button of [`crate::view::shell::drawer_button`] opens as a drawer.
#[must_use]
pub fn sidebar(id: &'static str, label: &'static str, groups: &[Group]) -> AnyView {
    let nav = NodeRef::<leptos::html::Nav>::new();
    let groups = groups.iter().copied().map(group).collect_view();
    let close = format!("{} fui-drawer-only", Variant::Ghost.class());
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
            <button type="button" class=close popovertarget=id popovertargetaction="hide">
                {icon(Glyph::X, "16")}
                {tr(Text::Close)}
            </button>
            {groups}
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
