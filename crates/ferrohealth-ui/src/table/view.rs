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

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::table::model::{SIZES, Sort, Window};
use crate::view::button::Variant;

/// One column of a data table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    /// The column's key, as a sort names it.
    pub key: &'static str,
    /// The header's words.
    pub label: &'static str,
    /// Whether the column holds numbers, which align right.
    pub numeric: bool,
    /// Whether the listing sorts by the column.
    pub sortable: bool,
}

impl Column {
    /// A text column that does not sort.
    #[must_use]
    pub const fn text(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            numeric: false,
            sortable: false,
        }
    }

    /// This column, sortable.
    #[must_use]
    pub const fn sortable(self) -> Self {
        Self {
            sortable: true,
            ..self
        }
    }

    /// This column, numeric.
    #[must_use]
    pub const fn numeric(self) -> Self {
        Self {
            numeric: true,
            ..self
        }
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
    let class = column.numeric.then_some(NUM);
    if !column.sortable {
        return view! {
            <th scope="col" class=class>
                {column.label}
            </th>
        }
        .into_any();
    }
    let aria = Sort::aria(sort, column.key);
    let glyph = match aria {
        "ascending" => Glyph::ARROW_UP,
        "descending" => Glyph::ARROW_DOWN,
        _ => Glyph::ARROW_UP_DOWN,
    };
    let href = sort_href(&Sort::toggled(sort, column.key));
    view! {
        <th scope="col" class=class aria-sort=aria>
            <a href=href class="fui-sort">
                {column.label}
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
        {step.label()}
        {(!available).then_some(" (unavailable)")}
        {forward.then(|| icon(step.glyph(), "14"))}
    }
}

/// Returns a pager over `window`, each step and size drawn by `step` and
/// `size` (a link for URL state, a button for session state).
#[must_use]
pub fn pager(
    label: &'static str,
    window: Window,
    step: &dyn Fn(Step, Option<u32>) -> AnyView,
    size: &dyn Fn(u32, bool) -> AnyView,
) -> impl IntoView + use<> {
    let [first, previous, next, last] = Step::ALL.map(|each| step(each, each.target(window)));
    let sizes = SIZES
        .into_iter()
        .map(|choice| size(choice, choice == window.size))
        .collect_view();
    view! {
        <nav class="fui-pager" aria-label=label>
            <p role="status">{window.range_label()}</p>
            <div class="fui-pager-steps">
                {first} {previous}
                <span class="fui-pager-position">
                    {format!("Page {} of {}", window.page, window.pages)}
                </span> {next} {last}
            </div>
            <div class="fui-pager-sizes">
                <span class="fui-muted">"Rows per page"</span>
                {sizes}
            </div>
        </nav>
    }
}

/// Returns the pager over `window` whose steps and sizes are links, the
/// href of each from `href`: the pager of a listing that keeps its state in
/// the URL, working before the page hydrates.
#[must_use]
pub fn link_pager(
    label: &'static str,
    window: Window,
    href: &dyn Fn(crate::table::model::Paging) -> String,
) -> impl IntoView + use<> {
    let paging = window.paging();
    let step = |each: Step, target: Option<u32>| match target {
        Some(page) => view! {
            <a href=href(paging.at(page)) class=Variant::Secondary.small()>
                {step_label(each, true)}
            </a>
        }
        .into_any(),
        None => view! {
            <span class=Variant::Secondary.small() aria-disabled="true">
                {step_label(each, false)}
            </span>
        }
        .into_any(),
    };
    let size = |choice: u32, current: bool| {
        let variant = if current {
            Variant::Primary
        } else {
            Variant::Ghost
        };
        view! {
            <a
                href=href(crate::table::model::Paging::sized(choice))
                class=variant.small()
                aria-current=current.then_some("true")
            >
                {choice.to_string()}
            </a>
        }
        .into_any()
    };
    pager(label, window, &step, &size)
}
