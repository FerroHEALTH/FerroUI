// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The navigation: groups of entries under their headings, the current page
//! marked, a column beside the page from the first breakpoint that folds to
//! a rail of glyphs, and a drawer below it.
//!
//! The drawer is an HTML popover the top bar's button opens, so the browser
//! opens it, closes it on Escape and returns the focus to the button with
//! no script. The island [`crate::island::NavDrawer`] adds what HTML leaves
//! out: the focus moves into the drawer as it opens, and the drawer closes
//! when one of its links is followed (WCAG 2.2 SC 2.4.3). The entries are
//! plain links the server draws, the current page marked
//! `aria-current="page"`, so following one loads the page, with or without
//! WebAssembly. The rail is the console's choice, kept as the others are
//! (family rule 9): Collapse and Expand post a plain form and the console's
//! server draws the next page folded or not.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// The count an entry carries (R12): a number while one is known, nothing
/// at zero, and a muted dash with the reason when none is known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Count {
    /// The count is known.
    Known {
        /// The number, drawn on the entry; nothing is drawn at zero.
        count: u32,
        /// What it counts as a whole phrase, such as "2 errors in the
        /// running load": the entry's accessible name and its tooltip
        /// carry it.
        words: String,
    },
    /// The count is not known, such as while the product does not answer.
    Unknown {
        /// Why, such as "Not known: the bridge did not answer", in the
        /// dash's tooltip and the entry's accessible name.
        why: String,
    },
}

/// One entry of the navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Where it leads.
    pub href: &'static str,
    /// What it says.
    pub label: Cow<'static, str>,
    /// The glyph beside it, which alone stands for it in the rail.
    pub glyph: Glyph,
    /// Whether it is marked on its own path alone, rather than on every
    /// path under it too (a section root such as `/` or a brand's home).
    pub exact: bool,
    /// Its count, if it carries one (R12).
    pub count: Option<Count>,
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

/// The rail's state and the form that changes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rail {
    /// Whether the navigation is folded to its glyphs from 48 rem up.
    pub collapsed: bool,
    /// The console's route the foot's button posts `sidebar` to: `rail`
    /// to fold the navigation, `full` to unfold it.
    pub action: String,
    /// The page to return to, posted as `back`, which the console checks
    /// is one of its own.
    pub back: String,
}

/// What the navigation shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sidebar {
    /// Its id, which the drawer button opens.
    pub id: &'static str,
    /// Its accessible name.
    pub label: String,
    /// Its groups, in order.
    pub groups: Vec<Group>,
    /// The path of the page, whose entry is marked.
    pub current: String,
    /// The rail and its Collapse and Expand at the foot, or `None` for a
    /// navigation that never folds.
    pub rail: Option<Rail>,
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

/// Returns the count `count` an entry carries, the number hidden from
/// assistive technology and the whole phrase read in its place; nothing at
/// zero.
fn count(count: Count) -> Option<AnyView> {
    let (class, shown, words) = match count {
        Count::Known { count: 0, .. } => return None,
        Count::Known { count, words } => ("fui-nav-count", count.to_string(), words),
        Count::Unknown { why } => (
            "fui-nav-count fui-nav-count-unknown",
            String::from("\u{2013}"),
            why,
        ),
    };
    let tooltip = words.clone();
    Some(
        view! {
            <span class=class title=tooltip>
                <span aria-hidden="true">{shown}</span>
                <span class="fui-sr-only">{words}</span>
            </span>
        }
        .into_any(),
    )
}

/// Returns the entry `entry`, marked while `current` is its page, or a
/// page under it unless it is exact; in the rail its words become its
/// tooltip.
fn entry(entry: Entry, current: &str, rail: bool) -> impl IntoView + use<> {
    let here = marks(entry.href, entry.exact, current).then_some("page");
    let title = rail.then(|| entry.label.to_string());
    let count = entry.count.and_then(count);
    view! {
        <li>
            <a href=entry.href class="fui-nav-entry" aria-current=here title=title>
                {icon(entry.glyph, "16")}
                <span class="fui-nav-word">{entry.label}</span>
                {count}
            </a>
        </li>
    }
}

/// Returns the group `group`, its entries marked against `current`, its
/// heading naming its list; nothing for a group with no entry.
fn group(group: Group, current: &str, rail: bool) -> Option<impl IntoView + use<>> {
    if group.entries.is_empty() {
        return None;
    }
    let entries = group
        .entries
        .into_iter()
        .map(|each| entry(each, current, rail))
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

/// Returns the foot of the navigation `id`: Collapse, or Expand in the
/// rail, a plain form that posts the other state.
fn foot(id: &'static str, rail: Rail) -> impl IntoView + use<> {
    let (value, glyph, words) = if rail.collapsed {
        ("full", Glyph::PANEL_LEFT_OPEN, tr(Text::SidebarExpand))
    } else {
        ("rail", Glyph::PANEL_LEFT_CLOSE, tr(Text::SidebarCollapse))
    };
    view! {
        <form method="post" action=rail.action class="fui-sidebar-foot fui-wide-only">
            <input type="hidden" name="back" value=rail.back />
            <button
                type="submit"
                name="sidebar"
                value=value
                class=Variant::Ghost.small()
                aria-expanded=(!rail.collapsed).to_string()
                aria-controls=id
            >
                {icon(glyph, "16")}
                <span class="fui-nav-word">{words}</span>
            </button>
        </form>
    }
}

/// Returns the navigation `sidebar`, its page marked, which the button of
/// [`crate::view::shell::drawer_button`] opens as a drawer, folded to its
/// glyphs from 48 rem up while its rail is collapsed.
///
/// The groups and the foot are drawn on the server and handed to the
/// island [`crate::island::NavDrawer`], which carries the drawer's
/// behaviour.
#[must_use]
pub fn sidebar(sidebar: Sidebar) -> AnyView {
    let close = tr(Text::Close);
    let collapsed = sidebar.rail.as_ref().is_some_and(|rail| rail.collapsed);
    let current = sidebar.current;
    let groups = sidebar
        .groups
        .into_iter()
        .map(|each| group(each, &current, collapsed))
        .collect_view();
    let foot = sidebar.rail.map(|rail| foot(sidebar.id, rail));
    view! {
        <crate::island::NavDrawer
            id=sidebar.id.to_owned()
            label=sidebar.label
            close
            rail=collapsed
        >
            {groups}
            {foot}
        </crate::island::NavDrawer>
    }
    .into_any()
}

/// Returns the drawer `id`, named `label`, around `entries`, its close
/// button saying `close`, folded to its glyphs while `rail`, with the
/// drawer's behaviour. A console draws it through [`sidebar`].
#[must_use]
pub fn drawer(id: String, label: String, close: String, rail: bool, entries: AnyView) -> AnyView {
    let nav = NodeRef::<leptos::html::Nav>::new();
    let button = format!("{} fui-drawer-only", Variant::Ghost.class());
    let class = if rail {
        "fui-sidebar fui-sidebar-rail"
    } else {
        "fui-sidebar"
    };
    let target = id.clone();
    view! {
        <nav
            id=id
            popover="auto"
            aria-label=label
            class=class
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
