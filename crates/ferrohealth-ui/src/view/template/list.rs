// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! T1, the list page: a title with its one primary action, the tools a
//! long list needs, and one card holding the list.
//!
//! The card is the list's only frame: a table runs edge to edge inside it.
//! Without tabs it opens with a header row (the title, the count, its
//! actions); with binder tabs, the tabs carry the title and the count and
//! the card is their panel. The summary strip and the toolbar appear only
//! past ten items ([`TOOLS_PAST`]), the toolbar also for a list with no
//! stated end (a history, an audit log); the active filter chips only while
//! a filter is set. An empty list keeps the title row's primary action and
//! says so in one quiet line with the same action as a text link
//! ([`list_empty`]), never a second primary button.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::number::grouped;
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::action::ActionGroup;
use crate::view::surface::{Header, page_heading};
use crate::view::tabs::{TabRow, joined};

/// The number of items a list shows with no summary strip and no toolbar
/// (family rule 8).
pub const TOOLS_PAST: u64 = 10;

/// Whether a list of `total` items draws its summary strip: only past
/// [`TOOLS_PAST`] items, and never for a list with no stated end.
#[must_use]
pub const fn shows_summary(total: Option<u64>) -> bool {
    matches!(total, Some(count) if count > TOOLS_PAST)
}

/// Whether a list of `total` items draws its toolbar: past [`TOOLS_PAST`]
/// items, or for a list with no stated end (`None`).
#[must_use]
pub const fn shows_toolbar(total: Option<u64>) -> bool {
    match total {
        Some(count) => count > TOOLS_PAST,
        None => true,
    }
}

/// One active filter: its words and the address of the list without it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterChip {
    /// What the filter keeps, such as "State: degraded".
    pub label: String,
    /// The list's address with this filter removed.
    pub remove: String,
}

/// The filters set on a list (R14).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActiveFilters {
    /// The filters, in the order they were set; none when no filter is set.
    pub chips: Vec<FilterChip>,
    /// The list's address with no filter.
    pub clear: String,
    /// How many items the filters keep, when the list counts them.
    pub shown: Option<u64>,
}

/// How many items a heading counts after its title: every item, or the
/// items shown of a larger set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingCount {
    /// Every item, drawn as "· 15".
    All(u64),
    /// The first items of a larger set, drawn as "· 6 of 15", such as a
    /// dashboard card that shows the start of a listing.
    Shown {
        /// How many items the heading's list shows.
        shown: u64,
        /// How many items the whole set holds.
        total: u64,
    },
}

impl From<u64> for HeadingCount {
    fn from(all: u64) -> Self {
        Self::All(all)
    }
}

/// The card that holds a list.
#[derive(Debug)]
pub struct ListCard {
    /// The list's name: the card's title and its accessible name.
    pub title: String,
    /// How many items the header row counts after the title, when the list
    /// states it.
    pub count: Option<HeadingCount>,
    /// The words of an order of several keys, stated beside the count ("Sorted
    /// by grade, then score, then the longest wait"): one arrow on one column
    /// cannot say it, and the leading column still carries the arrow
    /// (`design/COMPONENTS.md`, T1, the documented exception). The tabs of a
    /// tabbed card carry no order.
    pub order: Option<String>,
    /// The binder tabs over the card, when the list has two or more real
    /// views; they carry the title and the count, so the card draws no
    /// header row of its own.
    pub tabs: Option<TabRow>,
    /// The header row's actions at its far end, when it has any.
    pub actions: Option<Html>,
    /// The list: a table edge to edge, or [`list_empty`].
    pub body: Html,
    /// The pager in the card's foot, when there are more rows than one page
    /// holds.
    pub pager: Option<Html>,
}

/// What a list page shows.
#[derive(Debug)]
pub struct ListPage {
    /// The title, its one-sentence intro and, under a parent, the trail.
    pub header: Header,
    /// The title row's action group (F14): its one primary action when
    /// items can be created here, which stays when the list is empty.
    pub actions: ActionGroup,
    /// How many items the list holds, unfiltered; `None` for a list with no
    /// stated end (a history, an audit log, routes).
    pub total: Option<u64>,
    /// The summary strip: chips that each filter the list, with a count.
    pub summary: Option<Html>,
    /// The toolbar: the find field and the filter menus.
    pub toolbar: Option<Html>,
    /// The filters set on the list.
    pub filters: ActiveFilters,
    /// The card that holds the list.
    pub card: ListCard,
}

/// Returns a list page (T1): its header, then the summary strip and the
/// toolbar where the list is long enough to need them, the active filter
/// chips while a filter is set, and the card.
#[must_use]
pub fn list_page(locale: &Locale, page: ListPage) -> Html {
    let ListPage {
        header,
        actions,
        total,
        summary,
        toolbar,
        filters,
        card,
    } = page;
    let heading = page_heading(locale, header, actions);
    let summary = summary.filter(|_| shows_summary(total)).unwrap_or_default();
    let toolbar = toolbar.filter(|_| shows_toolbar(total)).unwrap_or_default();
    let filters = if filters.chips.is_empty() {
        Html::empty()
    } else {
        active_filters(locale, filters, total)
    };
    Html::raw(format!(
        "{heading}<div class=\"fui-list-page\">{summary}{toolbar}{filters}{}</div>",
        list_card(locale, card)
    ))
}

/// Returns the active filter chips of a list of `total` items: "Showing 7
/// of 21" where the list counts them, each filter with the link that
/// removes it, and Clear filters (R14).
///
/// Each removal is a link to the list without that filter, so it works
/// with no script and the filters stay in the address.
#[must_use]
pub fn active_filters(locale: &Locale, filters: ActiveFilters, total: Option<u64>) -> Html {
    let ActiveFilters {
        chips,
        clear,
        shown,
    } = filters;
    let showing = match (shown, total) {
        (Some(shown), Some(total)) => tr_with(
            locale,
            Text::FilterShowingOf,
            &[("shown", shown.into()), ("total", total.into())],
        ),
        _ => tr(locale, Text::FilterShowing),
    };
    ChipsView {
        showing,
        chips: chips
            .into_iter()
            .map(|chip| {
                let name = tr_with(
                    locale,
                    Text::FilterRemove,
                    &[("filter", chip.label.clone().into())],
                );
                (chip, name.text().to_owned())
            })
            .collect(),
        x_small: icon(Glyph::X, "12"),
        clear,
        x: icon(Glyph::X, "16"),
        clear_words: tr(locale, Text::FilterClear),
    }
    .html()
}

#[derive(Template)]
#[template(path = "template/list.html", block = "chips")]
struct ChipsView {
    showing: Word,
    chips: Vec<(FilterChip, String)>,
    x_small: Html,
    clear: String,
    x: Html,
    clear_words: Word,
}

/// Returns the card that holds a list, as [`list_page`] draws it.
///
/// Without tabs it is a section named by the list's title, opening with
/// the header row; with tabs, the tabs' panel holds the list.
#[must_use]
pub fn list_card(locale: &Locale, card: ListCard) -> Html {
    let ListCard {
        title,
        count,
        order,
        tabs,
        actions,
        body,
        pager,
    } = card;
    let foot = pager
        .map(|pager| Html::raw(format!("<div class=\"fui-list-foot\">{pager}</div>")))
        .unwrap_or_default();
    if let Some(tabs) = tabs {
        let panel = Html::raw(format!(
            "<section aria-label=\"{}\" class=\"fui-worklist fui-list-panel\">{body}{foot}</section>",
            Html::text(&title)
        ));
        return joined(locale, tabs, panel, None);
    }
    CardView {
        name: title.clone(),
        title: match count {
            Some(count) => counted(locale, title, count),
            None => Html::text(title),
        },
        order,
        actions: actions.unwrap_or_default(),
        body,
        foot,
    }
    .html()
}

#[derive(Template)]
#[template(path = "template/list.html", block = "card")]
struct CardView {
    name: String,
    title: Html,
    order: Option<String>,
    actions: Html,
    body: Html,
    foot: Html,
}

/// Returns `title` with its `count` after a middle dot, muted, for a list
/// card's header row, a panel section's heading or a binder tab.
///
/// The count reads "· 15", or "· 6 of 15" for the items shown of a larger
/// set, each number with the locale's marks.
#[must_use]
pub fn counted(locale: &Locale, title: String, count: HeadingCount) -> Html {
    let count = match count {
        HeadingCount::All(all) => format!("· {}", grouped(locale, all)),
        HeadingCount::Shown { shown, total } => {
            let words = tr_with(
                locale,
                Text::CountShownOf,
                &[("shown", shown.into()), ("total", total.into())],
            );
            format!("· {words}")
        }
    };
    Html::raw(format!(
        "{} <span class=\"fui-list-count\">{count}</span>",
        Html::text(title)
    ))
}

/// Returns an empty list's one quiet line: the console's `sentence` and,
/// when items can be created here, the same action as the title row's
/// primary as a text link (`label` to `href`), never a second button.
#[must_use]
pub fn list_empty(sentence: String, action: Option<(String, String)>) -> Html {
    let link = action
        .map(|(label, href)| {
            format!(
                " <a href=\"{}\">{}</a>",
                Html::text(href),
                Html::text(label)
            )
        })
        .unwrap_or_default();
    Html::raw(format!(
        "<p class=\"fui-transform-empty\">{}{link}</p>",
        Html::text(sentence)
    ))
}

#[cfg(test)]
mod tests {
    use super::{TOOLS_PAST, shows_summary, shows_toolbar};

    #[test]
    fn a_short_list_has_no_summary_strip_and_no_toolbar() {
        assert!(!shows_summary(Some(TOOLS_PAST)));
        assert!(!shows_toolbar(Some(TOOLS_PAST)));
        assert!(!shows_toolbar(Some(0)));
    }

    #[test]
    fn a_long_list_has_both() {
        assert!(shows_summary(Some(TOOLS_PAST + 1)));
        assert!(shows_toolbar(Some(TOOLS_PAST + 1)));
    }

    #[test]
    fn a_list_with_no_stated_end_has_a_toolbar_and_no_summary() {
        assert!(shows_toolbar(None));
        assert!(!shows_summary(None));
    }
}
