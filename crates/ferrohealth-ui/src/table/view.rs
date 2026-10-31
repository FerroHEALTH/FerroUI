// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The data table, its cells and the rows of a list card.
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
//! Below 40rem the rows stack: each row is a line under a hairline that
//! keeps the panel's inset, the cells placed by their [`Role`]. The cell
//! helpers write each cell's label and role, so a console's rows take that
//! form. No specification governs the measures: our own design.

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Word};
use crate::icon::{Glyph, icon};
use crate::table::model::{Noun, ORDER_KEY, PAGE_KEY, SORT_KEY, Sort, StateSource};
use crate::view::path::path;

/// Where a column's cells sit in the compact phone row of a list card.
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
    /// A column left out of the phone row: only what a phone reader finds
    /// again on the row's own page, never a selection, a state or the name.
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
    pub label: Word,
    /// Whether the column holds numbers, which align right.
    pub numeric: bool,
    /// Whether the listing sorts by the column.
    pub sortable: bool,
    /// Whether the header is code (a path, an alias), drawn in the code
    /// face and in its own case.
    pub mono: bool,
    /// The header's title, such as the path a query column reads.
    pub title: Option<Cow<'static, str>>,
    /// Where the column's cells sit in the compact phone row.
    pub role: Role,
}

impl Column {
    /// A text column that does not sort, its header `label`.
    #[must_use]
    pub fn text(key: &'static str, label: impl Into<Word>) -> Self {
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

    /// A text column whose key and header are known only at run time.
    #[must_use]
    pub fn owned(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: Cow::Owned(key.into()),
            label: Word::new(label),
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
/// words (family rule 12).
pub const EXCLUDED: &str = "is-excluded";

/// The indentation classes of a hierarchy row's leading cell, one per level.
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
/// a hierarchy; every level past the eighth takes the eighth's indentation.
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
    Rows(Html),
    /// This many skeleton rows, while the rows load.
    Loading(u8),
    /// The empty state, when there is no row.
    Empty(Html),
    /// Several bodies, each a `<tbody>` of `<tr>` rows the console draws,
    /// such as one per group under its [`group_row`].
    Bodies(Html),
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
    /// `GET` form; [`StateSource::Session`] posts it, so a listing that can
    /// show patient data keeps its order out of the URL and the history.
    pub source: StateSource,
    /// The URL the form submits to; `None` submits to the page's own
    /// address.
    pub action: Option<&'a str>,
    /// The other fields the form carries as they are, such as the listing's
    /// size and its filter. A `sort`, an `order` or a `page` among them is
    /// left out.
    pub fields: &'a [(String, String)],
    /// The words of an order of several keys, which the header of the
    /// column that leads it carries as its title.
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
pub(crate) fn hidden_fields(fields: &[(String, String)], dropped: &[&str]) -> Html {
    fields
        .iter()
        .filter(|(name, _)| !dropped.contains(&name.as_str()))
        .map(|(name, value)| {
            Html::raw(format!(
                "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
                Html::text(name),
                Html::text(value)
            ))
        })
        .collect()
}

/// The `method` of a form for a listing that keeps its state in `source`.
pub(crate) const fn method(source: StateSource) -> &'static str {
    match source {
        StateSource::Url => "get",
        StateSource::Session => "post",
    }
}

#[derive(Template)]
#[template(
    source = r#"<th scope="col"{% if let Some(class) = class %} class="{{ class }}"{% endif %}{% if let Some(title) = title %} title="{{ title }}"{% endif %}{% if let Some(aria) = aria_sort %} aria-sort="{{ aria }}"{% endif %}>{{ inside }}</th>"#,
    ext = "html"
)]
struct Header {
    class: Option<String>,
    title: Option<String>,
    aria_sort: Option<&'static str>,
    inside: Html,
}

#[derive(Template)]
#[template(
    source = r#"<form method="{{ method }}"{% if let Some(action) = action %} action="{{ action }}"{% endif %} class="fui-sort-form">{{ carried }}<input type="hidden" name="{{ order_key }}" value="{{ order }}"><button type="submit" class="fui-sort" name="{{ sort_key }}" value="{{ column }}">{{ words }}{{ glyph }}</button></form>"#,
    ext = "html"
)]
struct SortForm {
    method: &'static str,
    action: Option<String>,
    carried: Html,
    order_key: &'static str,
    order: &'static str,
    sort_key: &'static str,
    column: String,
    words: Html,
    glyph: Html,
}

/// Returns the header cell of `column` under `sorting`.
fn header(column: &Column, sorting: &Sorting<'_>) -> Html {
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
    let leads = sorting
        .sort
        .is_some_and(|sort| sort.column == column.key.as_ref());
    let title = column
        .title
        .as_deref()
        .or(sorting.stated.filter(|_| leads))
        .map(str::to_owned);
    let hidden_label = Html::raw(format!(
        "<span class=\"fui-sr-only\">{}</span>",
        column.label
    ));
    if column.role == Role::Check {
        // NOTE: a select-all only script can work stays hidden until the
        // work-list module shows it.
        let inside = hidden_label.then(Html::raw(format!(
            "<input type=\"checkbox\" class=\"fui-select-all\" aria-label=\"{}\" hidden>",
            Html::text(column.label.text())
        )));
        return Header {
            class,
            title,
            aria_sort: None,
            inside,
        }
        .html();
    }
    if column.role.hides_header() {
        return Header {
            class,
            title,
            aria_sort: None,
            inside: hidden_label,
        }
        .html();
    }
    // NOTE: no specification governs this: our own design; a code header breaks only
    // where a path does, so it never sets its column's width alone.
    let words = if column.mono {
        path(column.label.text())
    } else {
        column.label.clone().into()
    };
    if !column.sortable {
        return Header {
            class,
            title,
            aria_sort: None,
            inside: words,
        }
        .html();
    }
    let aria = Sort::aria(sorting.sort, &column.key);
    let glyph = match aria {
        "ascending" => Glyph::ARROW_UP,
        "descending" => Glyph::ARROW_DOWN,
        _ => Glyph::ARROW_UP_DOWN,
    };
    let next = Sort::toggled(sorting.sort, &column.key);
    let form = SortForm {
        method: method(sorting.source),
        action: sorting.action.map(str::to_owned),
        carried: hidden_fields(sorting.fields, &[SORT_KEY, ORDER_KEY, PAGE_KEY]),
        order_key: ORDER_KEY,
        order: next.direction.key(),
        sort_key: SORT_KEY,
        column: next.column,
        words,
        glyph: icon(glyph, "12"),
    }
    .html();
    Header {
        class,
        title,
        aria_sort: Some(aria),
        inside: form,
    }
    .html()
}

#[derive(Template)]
#[template(
    source = r#"<div class="{{ class }}" role="region" aria-label="{{ caption }}" tabindex="0"><table class="fui-table"{% if busy %} aria-busy="true"{% endif %}><caption>{{ caption }}</caption><thead><tr>{{ headers }}</tr></thead>{{ body }}</table></div>"#,
    ext = "html"
)]
struct TableView {
    class: String,
    caption: String,
    busy: bool,
    headers: Html,
    body: Html,
}

/// Returns the data table `table`, its rows stacked below 40rem.
///
/// Every body cell that carries `data-label` (its column's name) shows the
/// name beside its value, and the cell with none (the row's name) heads the
/// row, so a phone shows every column with no sideways scroll
/// (WCAG 2.2 SC 1.4.10).
#[must_use]
pub fn data_table(table: DataTable<'_>) -> Html {
    drawn(table, true, false)
}

/// Returns the data table `table` in a box that scrolls sideways at every
/// width, its rows stacked below 40rem.
///
/// It is the table of a column set no console bounds, such as a query's
/// result set, which never widens the page (WCAG 2.2 SC 1.4.10).
#[must_use]
pub fn data_table_wide(table: DataTable<'_>) -> Html {
    drawn(table, true, true)
}

/// Returns the data table `table` as a grid at every width: the opt-out for
/// a small table that fits a phone as it is.
#[must_use]
pub fn data_table_unstacked(table: DataTable<'_>) -> Html {
    drawn(table, false, false)
}

/// The table the three forms draw.
fn drawn(table: DataTable<'_>, stacked: bool, wide: bool) -> Html {
    let wide = wide || table.columns.iter().any(|column| column.mono);
    let mut classes = vec!["fui-table-scroll"];
    if stacked {
        classes.push("fui-table-stack");
    }
    if wide {
        classes.push("fui-table-wide");
    }
    let headers: Html = table
        .columns
        .iter()
        .map(|column| header(column, &table.sorting))
        .collect();
    let width = table.columns.len().max(1);
    let (body, busy) = match table.body {
        Body::Bodies(bodies) => (bodies, false),
        Body::Rows(rows) => (Html::raw(format!("<tbody>{rows}</tbody>")), false),
        Body::Loading(count) => {
            let cells: String = "<td><span class=\"fui-skeleton\"></span></td>".repeat(width);
            let rows: String = format!("<tr>{cells}</tr>").repeat(usize::from(count));
            (Html::raw(format!("<tbody>{rows}</tbody>")), true)
        }
        Body::Empty(empty) => (
            Html::raw(format!(
                "<tbody><tr><td colspan=\"{width}\">{empty}</td></tr></tbody>"
            )),
            false,
        ),
    };
    TableView {
        class: classes.join(" "),
        caption: table.caption.to_owned(),
        busy,
        headers,
        body,
    }
    .html()
}

/// Returns the link that names a row and opens it: the name in the ink with
/// no underline, the whole row its target (family rule 4; R9's extension).
#[must_use]
pub fn row_link(href: &str, name: Html) -> Html {
    let name = String::from(name);
    Html::raw(format!(
        "<a class=\"fui-row-link\" href=\"{}\">{name}</a>",
        Html::text(href)
    ))
}

/// Returns the cell that names a row: its row header, the name a
/// [`row_link`] to `href` when the row opens, and `sub` a quiet second
/// line under it (HTML Living Standard, `th` with `scope="row"`).
#[must_use]
pub fn title_cell(href: Option<&str>, name: Html, sub: Option<Html>) -> Html {
    title(None, href, name, sub)
}

/// Returns the cell that names a row `level` levels down a hierarchy: a
/// [`title_cell`] with the [`depth`] class on its `<th>`.
///
/// The name steps in by a loose space per level, and in a phone's stacked
/// card by a default space per level, so the card's heading keeps the level
/// readable in the narrower width. Every level past the eighth takes the
/// eighth's step.
#[must_use]
pub fn title_cell_at(level: u32, href: Option<&str>, name: Html, sub: Option<Html>) -> Html {
    title(Some(depth(level)), href, name, sub)
}

/// Returns a row's title cell, its `<th>` carrying `class` when given.
fn title(class: Option<&str>, href: Option<&str>, name: Html, sub: Option<Html>) -> Html {
    let name = match href {
        Some(href) => row_link(href, name),
        None => name,
    };
    let sub = sub
        .map(|sub| format!("<span class=\"fui-row-sub\">{sub}</span>"))
        .unwrap_or_default();
    let name = String::from(name);
    let class = class
        .map(|class| format!(" class=\"{class}\""))
        .unwrap_or_default();
    Html::raw(format!(
        "<th scope=\"row\"{class} data-role=\"{}\">{name}{sub}</th>",
        Role::Title.key()
    ))
}

/// Returns the cell of `column` holding `value`, named by the column on a
/// phone and placed by its role.
#[must_use]
pub fn cell(column: &Column, value: Html) -> Html {
    let value = String::from(value);
    let class = if column.numeric {
        format!(" class=\"{NUM}\"")
    } else {
        String::new()
    };
    Html::raw(format!(
        "<td data-label=\"{}\" data-role=\"{}\"{class}><span>{value}</span></td>",
        Html::text(column.label.text()),
        column.role.key()
    ))
}

/// Returns the cell of `column` holding `count`, which carries its noun on a
/// phone ("3 profiles") from the console's `noun`, in `locale`.
#[must_use]
pub fn count_cell(locale: &Locale, column: &Column, count: u32, noun: Noun) -> Html {
    Html::raw(format!(
        "<td data-label=\"{}\" data-role=\"{}\" data-unit=\"{}\" class=\"{NUM}\"><span>{}</span></td>",
        Html::text(column.label.text()),
        column.role.key(),
        Html::text(noun.word(locale, count).text()),
        Html::text(crate::i18n::number::grouped(locale, u64::from(count)))
    ))
}

/// Returns the cell that holds a row's menu, the ⋯ button the console
/// draws, at the end of the row and on the phone row's first line.
#[must_use]
pub fn menu_cell(menu: Html) -> Html {
    let menu = String::from(menu);
    Html::raw(format!(
        "<td data-label=\"\" class=\"fui-row-menu\" data-role=\"{}\">{menu}</td>",
        Role::Menu.key()
    ))
}

/// A row's checkbox: one choice of a selection that a form submits.
#[derive(Debug, Clone, Copy)]
pub struct RowCheck<'a> {
    /// The checkbox's id, unique on the page.
    pub id: &'a str,
    /// The field it submits as.
    pub name: &'a str,
    /// The value it submits: the row's id, never a record.
    pub value: &'a str,
    /// What it selects, its label for assistive technology.
    pub label: &'a str,
    /// The id of the form it belongs to, when the table is not inside it.
    pub form: Option<&'a str>,
    /// Whether it is checked.
    pub checked: bool,
}

#[derive(Template)]
#[template(
    source = r#"<td class="fui-row-check" data-role="check"><input type="checkbox" id="{{ id }}" name="{{ name }}" value="{{ value }}"{% if let Some(form) = form %} form="{{ form }}"{% endif %}{% if checked %} checked{% endif %}><label for="{{ id }}" class="fui-sr-only">{{ label }}</label></td>"#,
    ext = "html"
)]
struct CheckCell<'a> {
    id: &'a str,
    name: &'a str,
    value: &'a str,
    form: Option<&'a str>,
    checked: bool,
    label: &'a str,
}

/// Returns the cell that holds a row's checkbox, at the start of the row
/// and of the phone row's first line. The checkbox is native: it submits
/// with its form, with no script (WCAG 2.2 SC 1.3.1, SC 4.1.2).
#[must_use]
pub fn check_cell(check: RowCheck<'_>) -> Html {
    CheckCell {
        id: check.id,
        name: check.name,
        value: check.value,
        form: check.form,
        checked: check.checked,
        label: check.label,
    }
    .html()
}

/// Returns a group row across all of `columns`, its `heading` in the sunken
/// band: put each group in a `<tbody>` of its own ([`Body::Bodies`]), the
/// group row first (`scope="rowgroup"`).
#[must_use]
pub fn group_row(columns: &[Column], heading: Html) -> Html {
    let heading = String::from(heading);
    Html::raw(format!(
        "<tr class=\"fui-group-row\"><th scope=\"rowgroup\" colspan=\"{}\">{heading}</th></tr>",
        columns.len().max(1)
    ))
}
