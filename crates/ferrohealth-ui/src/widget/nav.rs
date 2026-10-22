// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The navigation: groups of entries under their headings, the current page
//! marked, a column beside the page from the first breakpoint that folds to
//! a rail of glyphs, and a drawer below it.
//!
//! The drawer is a modal HTML `dialog` the top bar's button opens through
//! the `show-modal` command, so with no script the page behind is inert
//! while it shows, the focus moves into it, Escape and a press outside close
//! it, and the focus returns to the button (HTML Living Standard, the
//! `dialog` element and the `command` attribute; WAI-ARIA Authoring
//! Practices, the dialog (modal) pattern; WCAG 2.2 SC 2.4.3 and SC 2.4.11).
//! The island [`crate::island::NavDrawer`] adds what HTML leaves out: the
//! focus moves to the first link as it opens, the drawer closes when one of
//! its links is followed, and Escape hides a rail entry's tooltip
//! (SC 1.4.13). From the first breakpoint up the closed dialog is the
//! column beside the page. The entries are
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
use crate::view::flow::FlowState;
use crate::widget::focus::focus_on_open;

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
    /// The nested section under one entry (F10), such as the organisation
    /// in view and its nodes, or `None`.
    pub nest: Option<Nest>,
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

/// The largest count the rail's badge draws whole; a larger one draws as
/// this number and a plus, so the badge stays on its glyph.
pub const RAIL_COUNT_MAX: u32 = 99;

/// Returns the count `count` an entry carries, the number hidden from
/// assistive technology and the whole phrase read in its place; nothing at
/// zero. A count over [`RAIL_COUNT_MAX`] also carries the short form the
/// rail's badge shows in its place.
fn count(count: Count) -> Option<AnyView> {
    let (shown, words) = match count {
        Count::Known { count: 0, .. } => return None,
        Count::Known { count, words } => (count, words),
        Count::Unknown { why } => {
            let tooltip = why.clone();
            return Some(
                view! {
                    <span class="fui-nav-count fui-nav-count-unknown" title=tooltip>
                        <span aria-hidden="true">"\u{2013}"</span>
                        <span class="fui-sr-only">{why}</span>
                    </span>
                }
                .into_any(),
            );
        }
    };
    let tooltip = words.clone();
    let short = (shown > RAIL_COUNT_MAX).then(|| {
        view! {
            <span class="fui-nav-count-short" aria-hidden="true">
                {format!("{RAIL_COUNT_MAX}+")}
            </span>
        }
    });
    let long = short.is_some();
    Some(
        view! {
            <span class="fui-nav-count" class:is-long=long title=tooltip>
                <span class="fui-nav-count-whole" aria-hidden="true">
                    {shown.to_string()}
                </span>
                {short}
                <span class="fui-sr-only">{words}</span>
            </span>
        }
        .into_any(),
    )
}

/// Returns the entry `entry`, marked while `current` is its page, or a
/// page under it unless it is exact; in the rail its words show as its
/// tooltip on hover and on focus. The nested section `nest` opens under it
/// when it names the entry, which then marks the location the page is
/// inside.
fn entry(
    entry: Entry,
    current: &str,
    rail: bool,
    nest: &mut Option<Nest>,
) -> impl IntoView + use<> {
    let nest = nest.take_if(|nest| nest.under == entry.href);
    let here = if nest.is_some() {
        Some("location")
    } else {
        marks(entry.href, entry.exact, current).then_some("page")
    };
    // NOTE: WCAG 2.2 SC 1.4.13; a title shows on hover alone, so the rail draws its own tooltip.
    let tooltip = rail.then(|| {
        view! {
            <span class="fui-tooltip" aria-hidden="true">
                {entry.label.to_string()}
            </span>
        }
    });
    let count = entry.count.and_then(count);
    let nest = nest.map(|nest| nested::section(nest, current));
    view! {
        <li>
            <a href=entry.href class="fui-nav-entry" aria-current=here>
                {icon(entry.glyph, "16")}
                <span class="fui-nav-word">{entry.label}</span>
                {count}
                {tooltip}
            </a>
            {nest}
        </li>
    }
}

/// Returns the group `group`, its entries marked against `current`, its
/// heading naming its list; nothing for a group with no entry.
fn group(
    group: Group,
    current: &str,
    rail: bool,
    nest: &mut Option<Nest>,
) -> Option<impl IntoView + use<>> {
    if group.entries.is_empty() {
        return None;
    }
    let entries = group
        .entries
        .into_iter()
        .map(|each| entry(each, current, rail, nest))
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
    let mut nest = sidebar.nest;
    let groups = sidebar
        .groups
        .into_iter()
        .map(|each| group(each, &current, collapsed, &mut nest))
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
    let dialog = NodeRef::<leptos::html::Dialog>::new();
    let quiet = RwSignal::new(false);
    let button = format!("{} fui-drawer-only", Variant::Ghost.class());
    let class = if rail {
        "fui-sidebar fui-sidebar-rail"
    } else {
        "fui-sidebar"
    };
    let target = id.clone();
    let name = label.clone();
    view! {
        <dialog
            id=id
            closedby="any"
            aria-label=name
            class=class
            data-tips=move || quiet.get().then_some("off")
            node_ref=dialog
            use:focus_on_open="a"
            on:click=move |event| drawer::followed(dialog, &event)
            on:keydown=move |event| {
                if event.key() == "Escape" {
                    quiet.set(true);
                }
            }
            on:pointerover=move |_| quiet.set(false)
            on:focusin=move |_| quiet.set(false)
        >
            <button type="button" class=button command="close" commandfor=target>
                {icon(Glyph::X, "16")}
                {close}
            </button>
            <nav aria-label=label class="fui-sidebar-nav">
                {entries}
            </nav>
        </dialog>
    }
    .into_any()
}

/// One item of a nested section: the parent in view or one of its
/// children, each with its health dot (F12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestItem {
    /// What it says, such as "Ziekenhuis Voorbeeldstad" or "node-a".
    pub label: String,
    /// Where it leads.
    pub href: String,
    /// Its state, drawn as the dot's shape and colour (family rule 12); an
    /// item switched off draws no shape, only its word.
    pub state: FlowState,
    /// The state as the dot's accessible name and tooltip, such as
    /// "Reachable", or "Worst node: degraded" for the parent.
    pub state_words: String,
    /// Whether the label is an identifier, drawn in the code face.
    pub mono: bool,
}

/// The nested section under one entry of the navigation (F10).
///
/// It holds the parent in view, such as an organisation, and its children,
/// such as its nodes, so a reader moves between the children without
/// leaving the sidebar. It keeps family rule 2: it is one level more under the one entry, never
/// a second sidebar. The rail leaves it out; the drawer keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nest {
    /// The `href` of the entry it opens under, such as the organisations.
    pub under: &'static str,
    /// The accessible name of the section, such as "Ziekenhuis
    /// Voorbeeldstad and its nodes".
    pub label: String,
    /// The parent in view.
    pub parent: NestItem,
    /// The number of children as a whole phrase, such as "2 nodes": the
    /// accessible name of the count badge beside the parent.
    pub count_words: String,
    /// The children, in order; past [`NEST_SHOWN`] the section draws that
    /// many, the page's own among them, and a link to the parent's page.
    pub children: Vec<NestItem>,
    /// The words of that link, a whole phrase such as "All 300 nodes".
    pub all: String,
}

/// The most children a nested section draws, so a parent with hundreds
/// leaves the groups under it in reach.
pub const NEST_SHOWN: usize = 8;

/// What the browser code adds to the drawer. From the first breakpoint up
/// the navigation is a closed dialog drawn as a column, so it does not
/// apply.
mod drawer {
    use leptos::prelude::*;

    /// Closes the open drawer when `event` follows one of its links.
    pub(super) fn followed(dialog: NodeRef<leptos::html::Dialog>, event: &leptos::ev::MouseEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast as _;
            let Some(dialog) = dialog.get_untracked() else {
                return;
            };
            let on_link = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .and_then(|target| target.closest("a").ok().flatten())
                .is_some();
            if on_link && dialog.open() {
                dialog.close();
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (NodeRef<leptos::html::Dialog>, &leptos::ev::MouseEvent) = (dialog, event);
    }
}

/// The nested section's drawing.
mod nested {
    use leptos::prelude::*;

    use super::{NEST_SHOWN, Nest, NestItem, marks};
    use crate::view::flow::FlowState;

    /// The `aria-current` of the parent while its own page is in view, and
    /// while a child's is.
    const PAGE: &str = "page";
    const LOCATION: &str = "location";

    /// Returns the health dot of `item`: its shape, with its words as its
    /// accessible name and tooltip; an item switched off has no shape and
    /// says its words to assistive technology alone.
    fn dot(item: &NestItem) -> AnyView {
        let words = item.state_words.clone();
        if item.state == FlowState::Off {
            return view! { <span class="fui-sr-only">{words}</span> }.into_any();
        }
        let class = format!("fui-dot {}", item.state.tone().class());
        let tooltip = words.clone();
        view! { <span class=class role="img" aria-label=words title=tooltip></span> }.into_any()
    }

    /// Returns the label of `item`, in the code face when it is an
    /// identifier.
    fn word(item: &NestItem) -> AnyView {
        let label = item.label.clone();
        if item.mono {
            view! { <span class="fui-nav-word"><code>{label}</code></span> }.into_any()
        } else {
            view! { <span class="fui-nav-word">{label}</span> }.into_any()
        }
    }

    /// Returns one child, marked as the page in view while `current` is
    /// its page; its full label is its tooltip, since a long one is cut.
    fn child(item: NestItem, current: &str) -> impl IntoView + use<> {
        let here = marks(&item.href, true, current).then_some("page");
        let dot = dot(&item);
        let word = word(&item);
        let title = item.label;
        view! {
            <li>
                <a
                    href=item.href
                    class="fui-nav-entry fui-nav-child"
                    aria-current=here
                    title=title
                >
                    {dot}
                    {word}
                </a>
            </li>
        }
    }

    /// The children `children` the section draws: the first
    /// [`NEST_SHOWN`], the last of them giving way to the page in view
    /// at `current` when it comes later.
    pub(super) fn shown(mut children: Vec<NestItem>, current: &str) -> Vec<NestItem> {
        if children.len() <= NEST_SHOWN {
            return children;
        }
        let here = children
            .iter()
            .position(|item| marks(&item.href, true, current))
            .filter(|at| *at >= NEST_SHOWN);
        let kept = here.map(|at| children.swap_remove(at));
        children.truncate(NEST_SHOWN - usize::from(kept.is_some()));
        children.extend(kept);
        children
    }

    /// Returns the nested section `nest`: the parent with its worst dot and
    /// its count, marked as the page while `current` is its own page and
    /// as the location while a child's is, then its children, at most
    /// [`NEST_SHOWN`] of them and a link to the parent's page for all.
    pub(super) fn section(nest: Nest, current: &str) -> AnyView {
        let Nest {
            under: _,
            label,
            parent,
            count_words,
            children,
            all,
        } = nest;
        let here = if marks(&parent.href, true, current) {
            PAGE
        } else {
            LOCATION
        };
        let total = children.len();
        let count = total.to_string();
        let dot = dot(&parent);
        let word = word(&parent);
        let title = parent.label.clone();
        let more = (total > NEST_SHOWN).then(|| {
            let href = parent.href.clone();
            view! {
                <li>
                    <a href=href class="fui-nav-entry fui-nav-child fui-nav-all">
                        <span class="fui-nav-word">{all}</span>
                    </a>
                </li>
            }
        });
        let children = (total > 0).then(|| {
            let rows = shown(children, current)
                .into_iter()
                .map(|item| child(item, current))
                .collect_view();
            view! { <ul class="fui-nav-children">{rows}{more}</ul> }
        });
        view! {
            <ul class="fui-nav-tree" aria-label=label>
                <li>
                    <a
                        href=parent.href
                        class="fui-nav-entry fui-nav-parent"
                        aria-current=here
                        title=title
                    >
                        {dot}
                        {word}
                        <span class="fui-nav-badge">
                            <span aria-hidden="true">{count}</span>
                            <span class="fui-sr-only">{count_words}</span>
                        </span>
                    </a>
                    {children}
                </li>
            </ul>
        }
        .into_any()
    }
}

#[cfg(test)]
mod tests {
    use super::{NEST_SHOWN, NestItem, marks, nested};
    use crate::view::flow::FlowState;

    fn nodes(count: usize) -> Vec<NestItem> {
        (0..count)
            .map(|at| NestItem {
                label: format!("node-{at}"),
                href: format!("/org/node-{at}"),
                state: FlowState::Up,
                state_words: String::from("Reachable"),
                mono: true,
            })
            .collect()
    }

    fn hrefs(items: &[NestItem]) -> Vec<&str> {
        items.iter().map(|item| item.href.as_str()).collect()
    }

    #[test]
    fn a_nested_section_draws_a_few_children_and_always_the_page_in_view() {
        assert_eq!(
            3,
            nested::shown(nodes(3), "/org").len(),
            "a few are all drawn"
        );
        let first = nested::shown(nodes(300), "/org/node-2");
        assert_eq!(NEST_SHOWN, first.len());
        assert_eq!(Some(&"/org/node-0"), hrefs(&first).first());
        let late = nested::shown(nodes(300), "/org/node-250");
        assert_eq!(NEST_SHOWN, late.len());
        assert_eq!(
            Some(&"/org/node-250"),
            hrefs(&late).last(),
            "the page in view is drawn last: {:?}",
            hrefs(&late)
        );
        assert_eq!(Some(&"/org/node-0"), hrefs(&late).first());
    }

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
