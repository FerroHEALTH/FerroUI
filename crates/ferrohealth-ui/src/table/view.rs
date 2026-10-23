// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The data table, its cells and the list card it sits in, drawn as static
//! views.
//!
//! The table is drawn to the design's measures (`design/COMPONENTS.md`,
//! § The table): inside a card or a panel the card edge is its only frame,
//! the header is the small step on the sunken ground, numbers align right
//! and the last row has no line under it. It shows every row of its page.
//! A sortable header is the submit button of a form that carries the sort
//! it asks for (`GET` for a listing that keeps its state in the URL, `POST`
//! for one that keeps it out), so a page sorts with no script, and the
//! sorted column's header carries `aria-sort` (WAI-ARIA 1.2).
//!
//! Below 40rem the rows stack: inside a card or a panel each row is a line
//! under a hairline that keeps the panel's inset. In a list card
//! ([`crate::view::template::list::list_card`]) the row is the compact
//! work-list line the cells' [`Role`] places: the
//! checkbox, the name and the row menu on the first line, the facts on the
//! second, a wide column left out. The cell helpers ([`cell`],
//! [`title_cell`], [`count_cell`], [`menu_cell`], [`check_cell`],
//! [`group_row`]) write each cell's label and role, so a console's rows
//! take that form. No specification governs the measures: our own design.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::table::model::{Noun, ORDER_KEY, PAGE_KEY, SORT_KEY, Sort, StateSource};
use crate::view::path::path;

/// Where a column's cells sit in the compact phone row of a list card
/// ([`crate::view::template::list::list_card`]).
///
/// Outside a list card a stacked row shows every cell under its column's
/// name, and the role changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Role {
    /// The row's name, which opens the first line.
    Title,
    /// A quiet second line beside the name.
    Sub,
    /// A fact on the second line.
    #[default]
    Meta,
    /// A quiet fact on the second line, such as a count.
    Quiet,
    /// A column left out of the phone row. It holds only what a phone
    /// reader finds again on the row's own page (`design/COMPONENTS.md`,
    /// § The table): never a selection, a state or the row's name.
    Wide,
    /// The row menu, at the end of the first line.
    Menu,
    /// The row's checkbox, at the start of the first line.
    Check,
}

impl Role {
    /// The cell's `data-role` value, which the stylesheet places by.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Sub => "sub",
            Self::Meta => "meta",
            Self::Quiet => "meta-q",
            Self::Wide => "wide",
            Self::Menu => "menu",
            Self::Check => "check",
        }
    }

    /// Whether the column's header names it for assistive technology
    /// alone: a row menu or a checkbox needs no visible header.
    const fn hides_header(self) -> bool {
        matches!(self, Self::Menu | Self::Check)
    }
}

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
    /// The header's title, such as the path a query column reads, which the
    /// header carries once so no cell repeats it.
    pub title: Option<Cow<'static, str>>,
    /// Where the column's cells sit in the compact phone row.
    pub role: Role,
}

impl Column {
    /// A text column that does not sort, its header `label` a fixed word or
    /// one the console looks up at run time.
    ///
    /// A column is built where its table is drawn, so its header follows the
    /// page's locale; a console keeps its columns in a function rather than
    /// a `const`.
    #[must_use]
    pub fn text(key: &'static str, label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            key: Cow::Borrowed(key),
            label: label.into(),
            numeric: false,
            sortable: false,
            mono: false,
            title: None,
            role: Role::default(),
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
            title: None,
            role: Role::default(),
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

    /// This column, its cells placed by `role` in the compact phone row.
    #[must_use]
    pub const fn role(mut self, role: Role) -> Self {
        self.role = role;
        self
    }

    /// This column, its header carrying the title `title`.
    ///
    /// A title is extra detail a pointer shows and a keyboard does not, so
    /// the header's words still name the column on their own (HTML Living
    /// Standard, the `title` attribute).
    #[must_use]
    pub fn titled(mut self, title: impl Into<Cow<'static, str>>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// The classes of a numeric cell.
pub const NUM: &str = "fui-num";

/// The classes of the cell that holds a row's actions as buttons.
pub const ACTIONS: &str = "fui-table-actions";

/// The class of a row that is excluded: muted, its State cell saying so in
/// words (`design/COMPONENTS.md`, § The table; family rule 12).
pub const EXCLUDED: &str = "is-excluded";

/// The indentation classes of a hierarchy row's leading cell, one per level.
///
/// A class per level rather than a `style` attribute, so a console keeps
/// `style-src 'self'`; the last step stands for every deeper level.
const DEPTHS: [&str; 9] = [
    "fui-depth-0",
    "fui-depth-1",
    "fui-depth-2",
    "fui-depth-3",
    "fui-depth-4",
    "fui-depth-5",
    "fui-depth-6",
    "fui-depth-7",
    "fui-depth-8",
];

/// The class that indents the leading cell of a table row `depth` levels down
/// a hierarchy, such as a concept under its parents in an expansion.
///
/// Every level past the eighth takes the eighth's indentation. Put the class
/// on the one cell that names the row; the other cells keep their column.
#[must_use]
pub fn depth(depth: u32) -> &'static str {
    let index = usize::try_from(depth).unwrap_or(usize::MAX);
    DEPTHS
        .get(index)
        .or(DEPTHS.last())
        .copied()
        .unwrap_or_default()
}

/// What the body of a data table holds.
#[derive(Debug)]
pub enum Body {
    /// The rows, each a `<tr>` the console draws.
    Rows(AnyView),
    /// This many skeleton rows, while the rows load.
    Loading(u8),
    /// The empty state, when there is no row.
    Empty(AnyView),
    /// Several bodies, each a `<tbody>` of `<tr>` rows the console draws,
    /// such as one per page of an answer or one per group under its
    /// [`group_row`], written as they are under the header and stacked below
    /// 40rem like any other rows.
    Bodies(AnyView),
}

/// How a table's sortable headers sort.
///
/// Each sortable header is the submit button of a form of its own, which
/// carries the sort it asks for as `sort` and `order` beside `fields`. A
/// new sort starts from the first page, so the form never carries `page`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Sorting<'a> {
    /// The sort in force, if any.
    pub sort: Option<&'a Sort>,
    /// Where the listing keeps its sort: [`StateSource::Url`] submits a
    /// `GET` form, so the sort lands in the query and a page sorts before
    /// it hydrates; [`StateSource::Session`] posts it, so a listing that
    /// can show patient data keeps its order out of the URL and the history.
    pub source: StateSource,
    /// The URL the form submits to; `None` submits to the page's own
    /// address, which is what a `GET` form of a URL listing wants.
    pub action: Option<&'a str>,
    /// The other fields the form carries as they are: the rest of the
    /// listing's query ([`crate::table::model::fields`]), such as its size
    /// and its filter, or what the console's handler reads. A `sort`, an
    /// `order` or a `page` among them is left out.
    pub fields: &'a [(String, String)],
    /// The words of an order of several keys ("Sorted by grade, then score,
    /// then the longest wait"), which the header of the column that leads it
    /// carries as its title (`design/COMPONENTS.md`, T1, the documented
    /// exception). The card's header row states it with
    /// [`crate::view::template::list::ListCard::order`].
    pub stated: Option<&'a str>,
}

/// What a data table shows.
#[derive(Debug)]
pub struct DataTable<'a> {
    /// The caption, which names the table.
    pub caption: &'a str,
    /// The columns, in order.
    pub columns: &'a [Column],
    /// How the sortable headers sort; [`Sorting::default`] for a table with
    /// no sortable column.
    pub sorting: Sorting<'a>,
    /// The body.
    pub body: Body,
}

/// The hidden fields of a form: `fields` without the keys in `dropped`.
pub(crate) fn hidden_fields(fields: &[(String, String)], dropped: &[&str]) -> AnyView {
    fields
        .iter()
        .filter(|(name, _)| !dropped.contains(&name.as_str()))
        .map(|(name, value)| {
            view! { <input type="hidden" name=name.clone() value=value.clone() /> }
        })
        .collect_view()
        .into_any()
}

/// The `method` of a form for a listing that keeps its state in `source`.
pub(crate) const fn method(source: StateSource) -> &'static str {
    match source {
        StateSource::Url => "get",
        StateSource::Session => "post",
    }
}

/// Returns the header cell of `column` under `sorting`.
fn header(column: &Column, sorting: &Sorting<'_>) -> AnyView {
    let mut classes: Vec<&str> = Vec::new();
    if column.numeric {
        classes.push(NUM);
    }
    if column.mono {
        classes.push("fui-th-mono");
    }
    if column.role == Role::Check {
        classes.push("fui-row-check");
    }
    let class = (!classes.is_empty()).then(|| classes.join(" "));
    let label = column.label.to_string();
    let leads = sorting
        .sort
        .is_some_and(|sort| sort.column == column.key.as_ref());
    let title = column
        .title
        .as_deref()
        .or(sorting.stated.filter(|_| leads))
        .map(str::to_owned);
    if column.role == Role::Check {
        // NOTE: leptos-ui.md §15.8: a select-all only script can work stays hidden until a
        // work-list island shows it (`crate::widget::worklist`).
        let name = label.clone();
        return view! {
            <th scope="col" title=title>
                <span class="fui-sr-only">{label}</span>
                <input type="checkbox" class="fui-select-all" aria-label=name hidden />
            </th>
        }
        .attr("class", class)
        .into_any();
    }
    if column.role.hides_header() {
        return view! {
            <th scope="col" title=title>
                <span class="fui-sr-only">{label}</span>
            </th>
        }
        .attr("class", class)
        .into_any();
    }
    // NOTE: no specification governs this: our own design; a code header breaks only
    // where a path does, so it never sets its column's width alone.
    let words = if column.mono {
        path(&label)
    } else {
        label.into_any()
    };
    if !column.sortable {
        return view! {
            <th scope="col" title=title>
                {words}
            </th>
        }
        .attr("class", class)
        .into_any();
    }
    let aria = Sort::aria(sorting.sort, &column.key);
    let glyph = match aria {
        "ascending" => Glyph::ARROW_UP,
        "descending" => Glyph::ARROW_DOWN,
        _ => Glyph::ARROW_UP_DOWN,
    };
    let next = Sort::toggled(sorting.sort, &column.key);
    let carried = hidden_fields(sorting.fields, &[SORT_KEY, ORDER_KEY, PAGE_KEY]);
    let action = sorting.action.map(str::to_owned);
    view! {
        <th scope="col" title=title aria-sort=aria>
            <form method=method(sorting.source) action=action class="fui-sort-form">
                {carried}
                <input type="hidden" name=ORDER_KEY value=next.direction.key() />
                <button type="submit" class="fui-sort" name=SORT_KEY value=next.column>
                    {words}
                    {icon(glyph, "12")}
                </button>
            </form>
        </th>
    }
    .attr("class", class)
    .into_any()
}

/// Returns the data table `table`, its rows stacked below 40rem.
///
/// Every body cell that carries `data-label` (its column's name) shows the
/// name beside its value, and the cell with none (the row's name) heads the
/// row, so a phone shows every column with no sideways scroll
/// (WCAG 2.2 SC 1.4.10). The cell helpers write `data-label`; a console
/// that draws its own cells writes it on every cell but the row's name.
#[must_use]
pub fn data_table(table: DataTable<'_>) -> impl IntoView + use<> {
    drawn(table, true, false)
}

/// Returns the data table `table` in a box that scrolls sideways at every
/// width, its rows stacked below 40rem.
///
/// It is the table of a column set no console bounds, such as a query's
/// result set, which can be wider than any card.
/// Its header gives up sticking to the viewport from 64rem, which a scroll
/// box would hold to itself: the table scrolls inside its card and never
/// widens the page (WCAG 2.2 SC 1.4.10). A table with a code header
/// ([`Column::mono`], a query's alias or path) is drawn this way by
/// [`data_table`] and [`data_table_unstacked`] too.
#[must_use]
pub fn data_table_wide(table: DataTable<'_>) -> impl IntoView + use<> {
    drawn(table, true, true)
}

/// Returns the data table `table` as a grid at every width: the opt-out for
/// a small table that fits a phone as it is (two short columns), which
/// scrolls sideways inside its box if a value outgrows it.
#[must_use]
pub fn data_table_unstacked(table: DataTable<'_>) -> impl IntoView + use<> {
    drawn(table, false, false)
}

/// The table [`data_table`], [`data_table_wide`] and
/// [`data_table_unstacked`] draw.
fn drawn(table: DataTable<'_>, stacked: bool, wide: bool) -> impl IntoView + use<> {
    let wide = wide || table.columns.iter().any(|column| column.mono);
    let mut classes = vec!["fui-table-scroll"];
    if stacked {
        classes.push("fui-table-stack");
    }
    if wide {
        classes.push("fui-table-wide");
    }
    let class = classes.join(" ");
    let headers = table
        .columns
        .iter()
        .map(|column| header(column, &table.sorting))
        .collect_view();
    let width = u8::try_from(table.columns.len()).unwrap_or(u8::MAX);
    let (body, busy) = match table.body {
        Body::Bodies(bodies) => (bodies, false),
        Body::Rows(rows) => (view! { <tbody>{rows}</tbody> }.into_any(), false),
        Body::Loading(count) => {
            let rows = (0..count)
                .map(|_| {
                    let cells = (0..width)
                        .map(|_| view! { <td><span class="fui-skeleton"></span></td> })
                        .collect_view();
                    view! { <tr>{cells}</tr> }
                })
                .collect_view();
            (view! { <tbody>{rows}</tbody> }.into_any(), true)
        }
        Body::Empty(empty) => (
            view! {
                <tbody>
                    <tr>
                        <td colspan=width.to_string()>{empty}</td>
                    </tr>
                </tbody>
            }
            .into_any(),
            false,
        ),
    };
    let label = table.caption.to_owned();
    let caption = label.clone();
    view! {
        <div
            class=class
            role="region"
            aria-label=label
            tabindex="0"
        >
            <table class="fui-table" aria-busy=busy.then_some("true")>
                <caption>{caption}</caption>
                <thead>
                    <tr>{headers}</tr>
                </thead>
                {body}
            </table>
        </div>
    }
}

/// Returns the link that names a row and opens it: the name in the ink with
/// no underline, the whole row its target (family rule 4; R9's extension).
#[must_use]
pub fn row_link(href: &str, name: impl IntoView + 'static) -> AnyView {
    linked(href.to_owned(), name.into_any())
}

/// The link [`row_link`] draws, one concrete body for every name.
fn linked(href: String, name: AnyView) -> AnyView {
    view! {
        <a class="fui-row-link" href=href>
            {name}
        </a>
    }
    .into_any()
}

/// Returns the cell that names a row: its row header, the name a
/// [`row_link`] to `href` when the row opens, and `sub` a quiet second
/// line under it.
///
/// It is the row's header (HTML Living Standard, `th` with `scope="row"`)
/// and opens the phone row's first line.
#[must_use]
pub fn title_cell(
    href: Option<&str>,
    name: impl IntoView + 'static,
    sub: Option<AnyView>,
) -> AnyView {
    let name = name.into_any();
    titled_row(
        match href {
            Some(href) => linked(href.to_owned(), name),
            None => name,
        },
        sub,
    )
}

/// The cell [`title_cell`] draws, one concrete body for every name.
fn titled_row(name: AnyView, sub: Option<AnyView>) -> AnyView {
    let sub = sub.map(|sub| view! { <span class="fui-row-sub">{sub}</span> });
    view! {
        <th scope="row" data-role=Role::Title.key()>
            {name}
            {sub}
        </th>
    }
    .into_any()
}

/// Returns the cell of `column` holding `value`, named by the column on a
/// phone and placed by its role.
///
/// The value is one node, so the stacked row keeps it together beside its
/// name; a quiet note in it (`fui-muted`) takes its own line on a phone, and
/// an empty value takes no line.
#[must_use]
pub fn cell(column: &Column, value: impl IntoView + 'static) -> AnyView {
    valued(column, value.into_any())
}

/// The cell [`cell`] draws, one concrete body for every value.
fn valued(column: &Column, value: AnyView) -> AnyView {
    let class = column.numeric.then_some(NUM);
    view! {
        <td data-label=column.label.to_string() data-role=column.role.key()>
            <span>{value}</span>
        </td>
    }
    .attr("class", class)
    .into_any()
}

/// Returns the cell of `column` holding `count`, which carries its noun on a
/// phone ("3 profiles") from the console's `noun`.
///
/// The count aligns right in tabular figures; on a phone, where no header
/// names it, the noun follows it (`data-unit`).
#[must_use]
pub fn count_cell(column: &Column, count: u32, noun: Noun) -> AnyView {
    view! {
        <td
            data-label=column.label.to_string()
            data-role=column.role.key()
            data-unit=noun.word(count)
            class=NUM
        >
            <span>{crate::i18n::number::grouped(u64::from(count))}</span>
        </td>
    }
    .into_any()
}

/// Returns the cell that holds a row's menu, the ⋯ button the console
/// draws, at the end of the row and on the phone row's first line.
///
/// Give the button an accessible name that says whose actions it holds
/// ("Actions for synthetic-007").
#[must_use]
pub fn menu_cell(menu: impl IntoView + 'static) -> AnyView {
    menu_of(menu.into_any())
}

/// The cell [`menu_cell`] draws, one concrete body for every menu.
fn menu_of(menu: AnyView) -> AnyView {
    view! {
        <td data-label="" class="fui-row-menu" data-role=Role::Menu.key()>
            {menu}
        </td>
    }
    .into_any()
}

/// A row's checkbox: one choice of a selection that a form submits.
#[derive(Debug, Clone, Copy)]
pub struct RowCheck<'a> {
    /// The checkbox's id, unique on the page.
    pub id: &'a str,
    /// The field it submits as, such as the argument of the console's bulk
    /// action.
    pub name: &'a str,
    /// The value it submits: the row's id, never a record.
    pub value: &'a str,
    /// What it selects, its label for assistive technology ("Select
    /// synthetic-007").
    pub label: &'a str,
    /// The id of the form it belongs to, when the table is not inside it.
    pub form: Option<&'a str>,
    /// Whether it is checked.
    pub checked: bool,
}

/// Returns the cell that holds a row's checkbox, at the start of the row
/// and of the phone row's first line.
///
/// The checkbox is native: it submits with its form, with no script. Its
/// label is visually hidden and names it by `for` (WCAG 2.2 SC 1.3.1,
/// SC 4.1.2).
#[must_use]
pub fn check_cell(check: RowCheck<'_>) -> AnyView {
    let id = check.id.to_owned();
    view! {
        <td class="fui-row-check" data-role=Role::Check.key()>
            <input
                type="checkbox"
                id=id.clone()
                name=check.name.to_owned()
                value=check.value.to_owned()
                form=check.form.map(str::to_owned)
                checked=check.checked
            />
            <label for=id class="fui-sr-only">
                {check.label.to_owned()}
            </label>
        </td>
    }
    .into_any()
}

/// Returns a group row across all of `columns`, its `heading` in the sunken
/// band (a full-width band on a phone).
///
/// It heads the rows of its group: put each group in a `<tbody>` of its own
/// ([`Body::Bodies`]), the group row first (`scope="rowgroup"`). A link in
/// the heading is a name: ink, no underline (family rule 4).
#[must_use]
pub fn group_row(columns: &[Column], heading: impl IntoView + 'static) -> AnyView {
    grouped(columns.len().max(1).to_string(), heading.into_any())
}

/// The row [`group_row`] draws, one concrete body for every heading.
fn grouped(width: String, heading: AnyView) -> AnyView {
    view! {
        <tr class="fui-group-row">
            <th scope="rowgroup" colspan=width>
                {heading}
            </th>
        </tr>
    }
    .into_any()
}
