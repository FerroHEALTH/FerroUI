// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The data table and the pager, drawn as static views.
//!
//! The table shows every row of its page, holds its header below the top
//! bar on a wide screen and scrolls sideways inside its card on a narrow
//! one, right aligns its numeric columns, keeps row actions at the end, and draws
//! skeleton rows while it loads and the empty state when there is nothing.
//! A sortable header is a link carrying `aria-sort`. The pager states the
//! range, then First, Previous, the position, Next and Last, and the sizes;
//! a step that cannot be taken stays where it is and says "(unavailable)"
//! in text, so colour never carries it (WCAG 2.2 SC 1.4.1).

use std::borrow::Cow;

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::table::model::{Noun, OpenWindow, Paging, SIZES, Sort, Window, partial_note};
use crate::view::button::Variant;

/// One column of a data table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    /// The column's key, as a sort names it.
    pub key: Cow<'static, str>,
    /// The header's words.
    pub label: Cow<'static, str>,
    /// Whether the column holds numbers, which align right.
    pub numeric: bool,
    /// Whether the listing sorts by the column.
    pub sortable: bool,
    /// Whether the header is code (a path, an alias), drawn in the code
    /// face and in its own case.
    pub mono: bool,
}

impl Column {
    /// A text column that does not sort.
    #[must_use]
    pub const fn text(key: &'static str, label: &'static str) -> Self {
        Self {
            key: Cow::Borrowed(key),
            label: Cow::Borrowed(label),
            numeric: false,
            sortable: false,
            mono: false,
        }
    }

    /// A text column whose key and header are known only at run time, such
    /// as the columns of a query's result set.
    #[must_use]
    pub fn owned(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: Cow::Owned(key.into()),
            label: Cow::Owned(label.into()),
            numeric: false,
            sortable: false,
            mono: false,
        }
    }

    /// This column, its header in the code face.
    #[must_use]
    pub const fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    /// This column, sortable.
    #[must_use]
    pub const fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// This column, numeric.
    #[must_use]
    pub const fn numeric(mut self) -> Self {
        self.numeric = true;
        self
    }
}

/// The classes of a numeric cell.
pub const NUM: &str = "fui-num";

/// The classes of the cell that holds a row's actions.
pub const ACTIONS: &str = "fui-table-actions";

/// What the body of a data table holds.
#[derive(Debug)]
pub enum Body {
    /// The rows, each a `<tr>` the console draws.
    Rows(AnyView),
    /// This many skeleton rows, while the rows load.
    Loading(u8),
    /// The empty state, when there is no row.
    Empty(AnyView),
}

/// What a data table shows.
pub struct DataTable<'a> {
    /// The caption, which names the table.
    pub caption: &'a str,
    /// The columns, in order.
    pub columns: &'a [Column],
    /// The sort in force, if any.
    pub sort: Option<&'a Sort>,
    /// The href a sortable header's link leads to, for the sort it asks for.
    pub sort_href: &'a dyn Fn(&Sort) -> String,
    /// The body.
    pub body: Body,
}

impl std::fmt::Debug for DataTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTable")
            .field("caption", &self.caption)
            .field("columns", &self.columns)
            .field("sort", &self.sort)
            .field("body", &self.body)
            .finish_non_exhaustive()
    }
}

/// Returns the header cell of `column` under the sort `sort`.
fn header(column: &Column, sort: Option<&Sort>, sort_href: &dyn Fn(&Sort) -> String) -> AnyView {
    let class = match (column.numeric, column.mono) {
        (true, true) => Some("fui-num fui-th-mono"),
        (true, false) => Some(NUM),
        (false, true) => Some("fui-th-mono"),
        (false, false) => None,
    };
    let label = column.label.to_string();
    if !column.sortable {
        return view! {
            <th scope="col" class=class>
                {label}
            </th>
        }
        .into_any();
    }
    let aria = Sort::aria(sort, &column.key);
    let glyph = match aria {
        "ascending" => Glyph::ARROW_UP,
        "descending" => Glyph::ARROW_DOWN,
        _ => Glyph::ARROW_UP_DOWN,
    };
    let href = sort_href(&Sort::toggled(sort, &column.key));
    view! {
        <th scope="col" class=class aria-sort=aria>
            <a href=href class="fui-sort">
                {label}
                {icon(glyph, "12")}
            </a>
        </th>
    }
    .into_any()
}

/// Returns the data table `table`.
#[must_use]
pub fn data_table(table: DataTable<'_>) -> impl IntoView + use<> {
    let headers = table
        .columns
        .iter()
        .map(|column| header(column, table.sort, table.sort_href))
        .collect_view();
    let width = u8::try_from(table.columns.len()).unwrap_or(u8::MAX);
    let (body, busy) = match table.body {
        Body::Rows(rows) => (rows, false),
        Body::Loading(count) => {
            let rows = (0..count)
                .map(|_| {
                    let cells = (0..width)
                        .map(|_| view! { <td><span class="fui-skeleton"></span></td> })
                        .collect_view();
                    view! { <tr>{cells}</tr> }
                })
                .collect_view()
                .into_any();
            (rows, true)
        }
        Body::Empty(empty) => (
            view! {
                <tr>
                    <td colspan=width.to_string()>{empty}</td>
                </tr>
            }
            .into_any(),
            false,
        ),
    };
    let label = table.caption.to_owned();
    let caption = label.clone();
    view! {
        <div class="fui-table-scroll" role="region" aria-label=label tabindex="0">
            <table class="fui-table" aria-busy=busy.then_some("true")>
                <caption>{caption}</caption>
                <thead>
                    <tr>{headers}</tr>
                </thead>
                <tbody>{body}</tbody>
            </table>
        </div>
    }
}

/// A step of the pager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The first page.
    First,
    /// The page before.
    Previous,
    /// The page after.
    Next,
    /// The last page.
    Last,
}

impl Step {
    /// Every step, in the order the pager draws them around the position.
    pub const ALL: [Self; 4] = [Self::First, Self::Previous, Self::Next, Self::Last];

    /// The step's `data-page` value, a stable hook for a journey.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Previous => "prev",
            Self::Next => "next",
            Self::Last => "last",
        }
    }

    /// The step's words.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::First => "First",
            Self::Previous => "Previous",
            Self::Next => "Next",
            Self::Last => "Last",
        }
    }

    /// The step's icon.
    #[must_use]
    pub const fn glyph(self) -> Glyph {
        match self {
            Self::First => Glyph::CHEVRONS_LEFT,
            Self::Previous => Glyph::CHEVRON_LEFT,
            Self::Next => Glyph::CHEVRON_RIGHT,
            Self::Last => Glyph::CHEVRONS_RIGHT,
        }
    }

    /// The page the step leads to from `window`, or `None` when it cannot be
    /// taken.
    #[must_use]
    pub const fn target(self, window: Window) -> Option<u32> {
        match self {
            Self::First | Self::Previous if !window.has_previous() => None,
            Self::Next | Self::Last if !window.has_next() => None,
            Self::First => Some(1),
            Self::Previous => Some(window.page - 1),
            Self::Next => Some(window.page + 1),
            Self::Last => Some(window.pages),
        }
    }
}

/// Returns the inside of a pager step's control: the icon beside the words,
/// the icon after them for a step forward, and "(unavailable)" in text when
/// it cannot be taken.
#[must_use]
pub fn step_label(step: Step, available: bool) -> impl IntoView + use<> {
    let forward = matches!(step, Step::Next | Step::Last);
    view! {
        {(!forward).then(|| icon(step.glyph(), "14"))}
        <span class="fui-step-words">
            {step.label()} {(!available).then_some("(unavailable)")}
        </span>
        {forward.then(|| icon(step.glyph(), "14"))}
    }
}

/// What a pager says about the listing it pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Listing {
    /// The pager's accessible name, which tells two pagers on one screen
    /// apart.
    pub label: &'static str,
    /// What the rows are, named in the range: "1-25 of 312 value sets".
    pub noun: Noun,
    /// The one-click page sizes; none when the size is a setting.
    pub sizes: &'static [u32],
    /// The rows the server counted, when it stated a count that the rows in
    /// hand may fall short of (a FHIR search's `Bundle.total`).
    pub stated: Option<u32>,
}

impl Listing {
    /// A listing named `label` of rows, offering 25, 50 and 100.
    #[must_use]
    pub const fn new(label: &'static str) -> Self {
        Self {
            label,
            noun: Noun::ROWS,
            sizes: &SIZES,
            stated: None,
        }
    }

    /// This listing, its rows named `noun`.
    #[must_use]
    pub const fn noun(self, noun: Noun) -> Self {
        Self { noun, ..self }
    }

    /// This listing, offering `sizes` (none when the size is a setting).
    #[must_use]
    pub const fn sizes(self, sizes: &'static [u32]) -> Self {
        Self { sizes, ..self }
    }

    /// This listing, its server having counted `stated` rows.
    #[must_use]
    pub const fn stated(self, stated: Option<u32>) -> Self {
        Self { stated, ..self }
    }
}

impl From<&'static str> for Listing {
    fn from(label: &'static str) -> Self {
        Self::new(label)
    }
}

/// Returns the size choices of `listing` around `current`, or nothing when
/// it offers none.
fn size_choices(listing: Listing, current: u32, size: &dyn Fn(u32, bool) -> AnyView) -> AnyView {
    if listing.sizes.is_empty() {
        return ().into_any();
    }
    let sizes = listing
        .sizes
        .iter()
        .map(|choice| size(*choice, *choice == current))
        .collect_view();
    view! {
        <div class="fui-pager-sizes">
            <span class="fui-muted">"Rows per page"</span>
            {sizes}
        </div>
    }
    .into_any()
}

/// Returns a pager over `window`, each step and size drawn by `step` and
/// `size` (a link for URL state, a button for session state).
///
/// The range names the rows by the listing's noun, and a second sentence
/// says so when the server counted more than the listing holds.
#[must_use]
pub fn pager(
    listing: impl Into<Listing>,
    window: Window,
    step: &dyn Fn(Step, Option<u32>) -> AnyView,
    size: &dyn Fn(u32, bool) -> AnyView,
) -> AnyView {
    let listing = listing.into();
    let [first, previous, next, last] = Step::ALL.map(|each| step(each, each.target(window)));
    let range = match partial_note(window.total, listing.stated, listing.noun) {
        Some(note) => format!("{}. {note}", window.range_of(listing.noun)),
        None => window.range_of(listing.noun),
    };
    let sizes = size_choices(listing, window.size, size);
    view! {
        <nav class="fui-pager" aria-label=listing.label>
            <p role="status">{range}</p>
            <div class="fui-pager-steps">
                {first} {previous}
                <span class="fui-pager-position">
                    {format!("Page {} of {}", window.page, window.pages)}
                </span> {next} {last}
            </div>
            {sizes}
        </nav>
    }
    .into_any()
}

/// Returns the pager of a listing whose total the server does not report:
/// Previous and Next around the range, which names no total ("Rows 26 to
/// 50"), each step and size drawn by `step` and `size`.
#[must_use]
pub fn open_pager(
    listing: impl Into<Listing>,
    window: OpenWindow,
    step: &dyn Fn(Step, Option<u32>) -> AnyView,
    size: &dyn Fn(u32, bool) -> AnyView,
) -> AnyView {
    let listing = listing.into();
    let previous = step(
        Step::Previous,
        window.has_previous().then(|| window.page - 1),
    );
    let next = step(
        Step::Next,
        window.has_next().then(|| window.page.saturating_add(1)),
    );
    let sizes = size_choices(listing, window.size, size);
    view! {
        <nav class="fui-pager" aria-label=listing.label>
            <p role="status">{window.range_of(listing.noun)}</p>
            <div class="fui-pager-steps">{previous} {next}</div>
            {sizes}
        </nav>
    }
    .into_any()
}

/// Returns the pager over `window` whose steps and sizes are links, the
/// href of each from `href`: the pager of a listing that keeps its state in
/// the URL, working before the page hydrates.
#[must_use]
pub fn link_pager(
    listing: impl Into<Listing>,
    window: Window,
    href: &dyn Fn(Paging) -> String,
) -> AnyView {
    let paging = window.paging();
    pager(
        listing,
        window,
        &|each, target| link_step(href, paging, each, target),
        &|choice, current| link_size(href, choice, current),
    )
}

/// Returns the pager over `window`, a listing with no stated total, whose
/// steps and sizes are links from `href`.
#[must_use]
pub fn link_open_pager(
    listing: impl Into<Listing>,
    window: OpenWindow,
    href: &dyn Fn(Paging) -> String,
) -> AnyView {
    let paging = window.paging();
    open_pager(
        listing,
        window,
        &|each, target| link_step(href, paging, each, target),
        &|choice, current| link_size(href, choice, current),
    )
}

/// Returns a pager step as a link to its page, or as a span that says
/// "(unavailable)" when it cannot be taken.
fn link_step(
    href: &dyn Fn(Paging) -> String,
    paging: Paging,
    each: Step,
    target: Option<u32>,
) -> AnyView {
    match target {
        Some(page) => view! {
            <a href=href(paging.at(page)) class=Variant::Secondary.small() data-page=each.key()>
                {step_label(each, true)}
            </a>
        }
        .into_any(),
        None => view! {
            <span class=Variant::Secondary.small() aria-disabled="true" data-page=each.key()>
                {step_label(each, false)}
            </span>
        }
        .into_any(),
    }
}

/// Returns a size choice as a link to the first page at that size.
fn link_size(href: &dyn Fn(Paging) -> String, choice: u32, current: bool) -> AnyView {
    view! {
        <a
            href=href(Paging::sized(choice))
            class="fui-pager-size"
            aria-current=current.then_some("true")
        >
            {choice.to_string()}
        </a>
    }
    .into_any()
}
