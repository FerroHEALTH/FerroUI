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
//! The `drawer` module adds what HTML leaves out: the focus moves to the
//! first link as it opens, the drawer closes when one of its links is
//! followed, and Escape hides a rail entry's tooltip (SC 1.4.13). From the
//! first breakpoint up the closed dialog is the column beside the page. The
//! entries are plain links, the current page marked `aria-current="page"`.
//! The rail is the console's choice, kept as the others are (family rule
//! 9): Collapse and Expand post a plain form and the console draws the next
//! page folded or not.

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::number::grouped;
use crate::i18n::{Locale, Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::state::Mark;

/// The count an entry carries (R12): a number while one is known, nothing
/// at zero, and a muted dash with the reason when none is known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Count {
    /// The count is known.
    Known {
        /// The number, drawn on the entry; nothing is drawn at zero.
        count: u32,
        /// What it counts as a whole phrase, such as "2 errors in the
        /// running load": the entry's accessible name and its tooltip.
        words: String,
    },
    /// The count is not known, such as while the product does not answer.
    Unknown {
        /// Why, in the dash's tooltip and the entry's accessible name.
        why: String,
    },
}

/// One entry of the navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Where it leads.
    pub href: Cow<'static, str>,
    /// What it says.
    pub label: Cow<'static, str>,
    /// The glyph beside it, which alone stands for it in the rail.
    pub glyph: Glyph,
    /// Whether it is marked on its own path alone, rather than on every
    /// path under it too (a section root such as `/`).
    pub exact: bool,
    /// Its count, if it carries one (R12).
    pub count: Option<Count>,
}

/// One group of the navigation, under its heading.
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
    /// The nested section under one entry (F10), or `None`.
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
/// zero.
fn count(locale: &Locale, count: Count) -> Html {
    let (shown, words) = match count {
        Count::Known { count: 0, .. } => return Html::empty(),
        Count::Known { count, words } => (count, words),
        Count::Unknown { why } => {
            return Html::raw(format!(
                "<span class=\"fui-nav-count fui-nav-count-unknown\" title=\"{}\"><span aria-hidden=\"true\">\u{2013}</span><span class=\"fui-sr-only\">{}</span></span>",
                Html::text(&why),
                Html::text(&why)
            ));
        }
    };
    let long = shown > RAIL_COUNT_MAX;
    let short = if long {
        format!("<span class=\"fui-nav-count-short\" aria-hidden=\"true\">{RAIL_COUNT_MAX}+</span>")
    } else {
        String::new()
    };
    let class = if long {
        "fui-nav-count is-long"
    } else {
        "fui-nav-count"
    };
    Html::raw(format!(
        "<span class=\"{class}\" title=\"{}\"><span class=\"fui-nav-count-whole\" aria-hidden=\"true\">{}</span>{short}<span class=\"fui-sr-only\">{}</span></span>",
        Html::text(&words),
        grouped(locale, u64::from(shown)),
        Html::text(&words)
    ))
}

struct EntryView {
    href: String,
    current: Option<&'static str>,
    glyph: Html,
    label: String,
    count: Html,
    tooltip: Option<String>,
    nest: Option<Html>,
}

struct GroupView {
    id: &'static str,
    title: String,
    entries: Vec<EntryView>,
}

struct FootView {
    action: String,
    back: String,
    value: &'static str,
    class: &'static str,
    expanded: bool,
    glyph: Html,
    words: Word,
}

#[derive(Template)]
#[template(path = "nav/sidebar.html")]
struct SidebarView {
    id: &'static str,
    label: String,
    rail: bool,
    close_class: String,
    close_glyph: Html,
    close: Word,
    groups: Vec<GroupView>,
    foot: Option<FootView>,
}

/// Returns the navigation `sidebar`, its page marked, which the button of
/// the top bar opens as a drawer, folded to its glyphs from 48 rem up while
/// its rail is collapsed.
#[must_use]
pub fn sidebar(locale: &Locale, sidebar: Sidebar) -> Html {
    let Sidebar {
        id,
        label,
        groups,
        current,
        rail,
        mut nest,
    } = sidebar;
    let collapsed = rail.as_ref().is_some_and(|rail| rail.collapsed);
    let groups = groups
        .into_iter()
        .filter(|group| !group.entries.is_empty())
        .map(|group| GroupView {
            id: group.id,
            title: group.title.into_owned(),
            entries: group
                .entries
                .into_iter()
                .map(|entry| {
                    let nested = nest.take_if(|nest| nest.under == entry.href);
                    let here = if nested.is_some() {
                        Some("location")
                    } else {
                        marks(&entry.href, entry.exact, &current).then_some("page")
                    };
                    EntryView {
                        href: entry.href.into_owned(),
                        current: here,
                        glyph: icon(entry.glyph, "16"),
                        // NOTE: WCAG 2.2 SC 1.4.13; a title shows on hover alone, so the rail draws its own tooltip.
                        tooltip: collapsed.then(|| entry.label.to_string()),
                        label: entry.label.into_owned(),
                        count: entry.count.map(|each| count(locale, each)).unwrap_or_default(),
                        nest: nested.map(|nest| section(locale, nest, &current)),
                    }
                })
                .collect(),
        })
        .collect();
    let foot = rail.map(|rail| {
        let (value, glyph, words) = if rail.collapsed {
            ("full", Glyph::PANEL_LEFT_OPEN, tr(locale, Text::SidebarExpand))
        } else {
            ("rail", Glyph::PANEL_LEFT_CLOSE, tr(locale, Text::SidebarCollapse))
        };
        FootView {
            action: rail.action,
            back: rail.back,
            value,
            class: Variant::Ghost.small(),
            expanded: !rail.collapsed,
            glyph: icon(glyph, "16"),
            words,
        }
    });
    SidebarView {
        id,
        label,
        rail: collapsed,
        close_class: format!("{} fui-drawer-only", Variant::Ghost.class()),
        close_glyph: icon(Glyph::X, "16"),
        close: tr(locale, Text::Close),
        groups,
        foot,
    }
    .html()
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
    pub state: Mark,
    /// The state as the dot's accessible name and tooltip.
    pub state_words: String,
    /// Whether the label is an identifier, drawn in the code face.
    pub mono: bool,
}

/// The nested section under one entry of the navigation (F10): the parent
/// in view and its children, one level more under the one entry, never a
/// second sidebar (family rule 2). The rail leaves it out; the drawer keeps
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nest {
    /// The `href` of the entry it opens under.
    pub under: Cow<'static, str>,
    /// The accessible name of the section.
    pub label: String,
    /// The parent in view.
    pub parent: NestItem,
    /// The number of children as a whole phrase, such as "2 nodes".
    pub count_words: String,
    /// The children, in order; past [`NEST_SHOWN`] the section draws that
    /// many, the page's own among them, and a link to the parent's page.
    pub children: Vec<NestItem>,
    /// The words of that link, a whole phrase such as "All 300 nodes".
    pub all: String,
}

/// The most children a nested section draws.
pub const NEST_SHOWN: usize = 8;

/// The children `children` the section draws: the first [`NEST_SHOWN`], the
/// last of them giving way to the page in view at `current` when it comes
/// later.
#[must_use]
pub fn shown(mut children: Vec<NestItem>, current: &str) -> Vec<NestItem> {
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

struct NestItemView {
    href: String,
    label: String,
    current: bool,
    dot: Html,
    word: Html,
}

impl NestItemView {
    fn of(item: NestItem, current: &str) -> Self {
        let dot = if item.state == Mark::Off {
            Html::raw(format!(
                "<span class=\"fui-sr-only\">{}</span>",
                Html::text(&item.state_words)
            ))
        } else {
            Html::raw(format!(
                "<span class=\"fui-dot {}\" role=\"img\" aria-label=\"{}\" title=\"{}\"></span>",
                item.state.tone().class(),
                Html::text(&item.state_words),
                Html::text(&item.state_words)
            ))
        };
        let word = if item.mono {
            Html::raw(format!(
                "<span class=\"fui-nav-word\"><code>{}</code></span>",
                Html::text(&item.label)
            ))
        } else {
            Html::raw(format!(
                "<span class=\"fui-nav-word\">{}</span>",
                Html::text(&item.label)
            ))
        };
        Self {
            current: marks(&item.href, true, current),
            href: item.href,
            label: item.label,
            dot,
            word,
        }
    }
}

#[derive(Template)]
#[template(path = "nav/nest.html")]
struct NestView {
    label: String,
    parent: NestItemView,
    parent_current: &'static str,
    count: String,
    count_words: String,
    children: Vec<NestItemView>,
    all: Option<String>,
}

/// Returns the nested section `nest`: the parent with its worst dot and
/// its count, marked as the page while `current` is its own page and as
/// the location while a child's is, then its children.
fn section(locale: &Locale, nest: Nest, current: &str) -> Html {
    let Nest {
        under: _,
        label,
        parent,
        count_words,
        children,
        all,
    } = nest;
    let total = children.len();
    let parent = NestItemView::of(parent, current);
    NestView {
        label,
        parent_current: if parent.current { "page" } else { "location" },
        parent,
        count: grouped(locale, u64::try_from(total).unwrap_or(u64::MAX)),
        count_words,
        children: shown(children, current)
            .into_iter()
            .map(|item| NestItemView::of(item, current))
            .collect(),
        all: (total > NEST_SHOWN).then_some(all),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{NEST_SHOWN, NestItem, marks, shown};
    use crate::view::state::Mark;

    fn nodes(count: usize) -> Vec<NestItem> {
        (0..count)
            .map(|at| NestItem {
                label: format!("node-{at}"),
                href: format!("/org/node-{at}"),
                state: Mark::Ok,
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
        assert_eq!(3, shown(nodes(3), "/org").len(), "a few are all drawn");
        let first = shown(nodes(300), "/org/node-2");
        assert_eq!(NEST_SHOWN, first.len());
        assert_eq!(Some(&"/org/node-0"), hrefs(&first).first());
        let late = shown(nodes(300), "/org/node-250");
        assert_eq!(NEST_SHOWN, late.len());
        assert_eq!(Some(&"/org/node-250"), hrefs(&late).last(), "{:?}", hrefs(&late));
        assert_eq!(Some(&"/org/node-0"), hrefs(&late).first());
    }

    #[test]
    fn an_entry_marks_its_own_page_and_the_pages_under_it() {
        assert!(marks("/ferrofed/tables", true, "/ferrofed/tables"));
        assert!(marks("/ferrofed/tables", true, "/ferrofed/tables?page=2"));
        assert!(marks("/ferrofed/tables", true, "/ferrofed/tables/"));
        assert!(!marks("/ferrofed", true, "/ferrofed/tables"));
        assert!(marks("/files", false, "/files/synthetic-007"));
        assert!(!marks("/files", false, "/filestore"));
        assert!(marks("/", true, "/"));
        assert!(!marks("/", true, "/ferrofed"));
    }
}
