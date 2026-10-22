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

use leptos::prelude::*;

use crate::i18n::{Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::surface::{Header, page_heading};
use crate::view::template::tabbed;

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

/// The card that holds a list.
#[derive(Debug)]
pub struct ListCard {
    /// The list's name: the card's title and its accessible name.
    pub title: String,
    /// How many items the header row counts after the title, when the list
    /// states it.
    pub count: Option<u64>,
    /// The binder tabs over the card, when the list has two or more real
    /// views; they carry the title and the count, so the card draws no
    /// header row of its own.
    pub tabs: Option<AnyView>,
    /// The header row's actions at its far end, when it has any.
    pub actions: Option<AnyView>,
    /// The list: a table edge to edge, or [`list_empty`].
    pub body: AnyView,
    /// The pager in the card's foot, when there are more rows than one page
    /// holds.
    pub pager: Option<AnyView>,
}

/// What a list page shows.
#[derive(Debug)]
pub struct ListPage {
    /// The title, its one-sentence intro and, under a parent, the trail.
    pub header: Header,
    /// The title row's one primary action, when items can be created here;
    /// it stays when the list is empty.
    pub primary: Option<AnyView>,
    /// How many items the list holds, unfiltered; `None` for a list with no
    /// stated end (a history, an audit log, routes).
    pub total: Option<u64>,
    /// The summary strip: chips that each filter the list, with a count.
    pub summary: Option<AnyView>,
    /// The toolbar: the find field and the filter menus.
    pub toolbar: Option<AnyView>,
    /// The filters set on the list.
    pub filters: ActiveFilters,
    /// The card that holds the list.
    pub card: ListCard,
}

/// Returns a list page (T1): its header, then the summary strip and the
/// toolbar where the list is long enough to need them, the active filter
/// chips while a filter is set, and the card.
#[must_use]
pub fn list_page(page: ListPage) -> AnyView {
    let ListPage {
        header,
        primary,
        total,
        summary,
        toolbar,
        filters,
        card,
    } = page;
    let heading = page_heading(header, primary).into_any();
    let summary = summary.filter(|_| shows_summary(total));
    let toolbar = toolbar.filter(|_| shows_toolbar(total));
    let filters = (!filters.chips.is_empty()).then(|| active_filters(filters, total));
    view! {
        {heading}
        <div class="fui-list-page">{summary} {toolbar} {filters} {list_card(card)}</div>
    }
    .into_any()
}

/// Returns the active filter chips of a list of `total` items: "Showing 7
/// of 21" where the list counts them, each filter with the link that
/// removes it, and Clear filters (R14).
///
/// Each removal is a link to the list without that filter, so it works
/// with no script and the filters stay in the address.
#[must_use]
pub fn active_filters(filters: ActiveFilters, total: Option<u64>) -> AnyView {
    let ActiveFilters {
        chips,
        clear,
        shown,
    } = filters;
    let showing = match (shown, total) {
        (Some(shown), Some(total)) => tr_with(
            Text::FilterShowingOf,
            &[("shown", shown.into()), ("total", total.into())],
        ),
        _ => tr(Text::FilterShowing),
    };
    let chips = chips
        .into_iter()
        .map(|chip| {
            let name = tr_with(Text::FilterRemove, &[("filter", chip.label.clone().into())]);
            view! {
                <span class="fui-filter-chip">
                    {chip.label} <a href=chip.remove aria-label=name>
                        {icon(Glyph::X, "12")}
                    </a>
                </span>
            }
        })
        .collect_view();
    view! {
        <div class="fui-filter-chips">
            <span class="fui-small fui-muted">{showing}</span>
            {chips}
            <a href=clear class="fui-button fui-button-ghost fui-button-small">
                {icon(Glyph::X, "16")}
                {tr(Text::FilterClear)}
            </a>
        </div>
    }
    .into_any()
}

/// Returns the card that holds a list, as [`list_page`] draws it.
///
/// Without tabs it is a section named by the list's title, opening with
/// the header row; with tabs, the tabs' panel holds the list.
#[must_use]
pub fn list_card(card: ListCard) -> AnyView {
    let ListCard {
        title,
        count,
        tabs,
        actions,
        body,
        pager,
    } = card;
    let foot = pager.map(|pager| view! { <div class="fui-list-foot">{pager}</div> });
    if let Some(tabs) = tabs {
        let panel = view! {
            <section class="fui-worklist fui-list-panel" aria-label=title>
                {body}
                {foot}
            </section>
        }
        .into_any();
        return tabbed(tabs, panel, "fui-tabbed");
    }
    let name = title.clone();
    view! {
        <section class="fui-card fui-panel fui-worklist fui-list-card" aria-label=name>
            <div class="fui-panel-head fui-list-head">
                <h2 class="fui-card-title">
                    <span>
                        {match count {
                            Some(count) => counted(title, count),
                            None => title.into_any(),
                        }}
                    </span>
                </h2>
                <div class="fui-actions">{actions}</div>
            </div>
            {body}
            {foot}
        </section>
    }
    .into_any()
}

/// Returns `title` with its `count` after a middle dot, muted, for a list
/// card's header row or a binder tab.
#[must_use]
pub fn counted(title: String, count: u64) -> AnyView {
    // NOTE: no specification governs this: our own design; the count is a number,
    // drawn the same in every locale.
    let count = format!("· {count}");
    view! {
        {title}
        " "
        <span class="fui-list-count">{count}</span>
    }
    .into_any()
}

/// Returns an empty list's one quiet line: the console's `sentence` and,
/// when items can be created here, the same action as the title row's
/// primary as a text link (`label` to `href`), never a second button.
#[must_use]
pub fn list_empty(sentence: String, action: Option<(String, String)>) -> AnyView {
    let link = action.map(|(label, href)| view! { " " <a href=href>{label}</a> });
    view! { <p class="fui-transform-empty">{sentence} {link}</p> }.into_any()
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
