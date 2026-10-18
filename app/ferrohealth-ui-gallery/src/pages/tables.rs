// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tables page: the data table with its page, size and sort in the URL,
//! the same rows paged in the page's own state, and the loading and empty
//! tables.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::model::{Paging, Sort, Window, href};
use ferrohealth_ui::table::view::{ACTIONS, Body, Column, DataTable, NUM, data_table, link_pager};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::empty::empty_state;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{card, page_header, stack};
use ferrohealth_ui::widget::pager::session_pager;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_query_map};

use crate::data::{Row, SORTABLE, rows, sorted};

/// The columns of the gallery's table.
const COLUMNS: [Column; 5] = [
    Column::text("name", "File").sortable(),
    Column::text("kind", "Kind"),
    Column::text("lines", "Lines").numeric().sortable(),
    Column::text("state", "State"),
    Column::text("actions", "Actions"),
];

/// The same columns, none sorting: a table that does not sort draws no
/// sort links.
const PLAIN: [Column; 5] = [
    Column::text("name", "File"),
    Column::text("kind", "Kind"),
    Column::text("lines", "Lines").numeric(),
    Column::text("state", "State"),
    Column::text("actions", "Actions"),
];

/// The table row of `row`.
fn row(row: &Row) -> AnyView {
    let tone = match row.state {
        "refused" => Tone::Bad,
        "warning" => Tone::Warn,
        _ => Tone::Ok,
    };
    view! {
        <tr>
            <th scope="row">
                <code>{row.name.clone()}</code>
            </th>
            <td>{row.kind}</td>
            <td class=NUM>{row.lines.to_string()}</td>
            <td>{badge(tone, row.state)}</td>
            <td class=ACTIONS>
                <button type="button" class=Variant::Secondary.small()>
                    "Open"
                </button>
            </td>
        </tr>
    }
    .into_any()
}

/// The table whose page, size and sort live in the URL: links that work
/// before the page hydrates and that back and forward walk.
fn url_table() -> AnyView {
    let query = use_query_map();
    let location = use_location();
    let drawn = move || {
        let query = query.get();
        let path = location.pathname.get();
        let paging = Paging::from_query(&query);
        let sort = Sort::from_query(&query, &SORTABLE);
        let all = match &sort {
            Some(sort) => sorted(
                rows(),
                &sort.column,
                sort.direction == ferrohealth_ui::table::model::Direction::Descending,
            ),
            None => rows(),
        };
        let total = u32::try_from(all.len()).unwrap_or(u32::MAX);
        let window = Window::over(total, paging);
        let body = window.slice(&all).iter().map(row).collect_view().into_any();
        let sort_href = {
            let query = query.clone();
            let path = path.clone();
            move |next: &Sort| {
                let mut query = query.clone();
                next.write(&mut query);
                href(&path, &query)
            }
        };
        let page_href = move |paging: Paging| {
            let mut query = query.clone();
            paging.write(&mut query);
            href(&path, &query)
        };
        view! {
            {data_table(DataTable {
                caption: "Mapping files, paged in the URL",
                columns: &COLUMNS,
                sort: sort.as_ref(),
                sort_href: &sort_href,
                body: Body::Rows(body),
            })}
            {link_pager("Pages of the mapping files", window, &page_href)}
        }
    };
    card("Paged in the URL", drawn).into_any()
}

/// The same rows paged in the page's own state, never in the URL, as a
/// screen that can show patient data pages them.
fn session_table() -> AnyView {
    let paging = RwSignal::new(Paging::default());
    let all = StoredValue::new(rows());
    let drawn = move || {
        let window = Window::over(crate::data::ROWS, paging.get());
        let body = all
            .with_value(|all| window.slice(all).iter().map(row).collect_view())
            .into_any();
        view! {
            {data_table(DataTable {
                caption: "Mapping files, paged in the page",
                columns: &PLAIN,
                sort: None,
                sort_href: &|_| String::new(),
                body: Body::Rows(body),
            })}
            {session_pager("Pages of the mapping files, in the page", window, paging)}
        }
    };
    card("Paged in the page", drawn).into_any()
}

/// The table while it loads, and the table with no row.
fn states() -> AnyView {
    let loading = data_table(DataTable {
        caption: "Loading",
        columns: &PLAIN,
        sort: None,
        sort_href: &|_| String::new(),
        body: Body::Loading(3),
    });
    let empty = data_table(DataTable {
        caption: "No rows",
        columns: &PLAIN,
        sort: None,
        sort_href: &|_| String::new(),
        body: Body::Empty(
            empty_state(
                Glyph::INBOX,
                "No mapping file",
                "The bridge serves no mapping file yet.",
                (),
            )
            .into_any(),
        ),
    });
    card(
        "Loading and empty",
        view! {
            {loading}
            {empty}
        },
    )
    .into_any()
}

/// The tables page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn TablesPage() -> impl IntoView {
    let header = page_header(
        "Tables and paging",
        Some(String::from("One table and pager model: in the URL, or in the page for a screen that can show patient data.")),
        (),
    )
    .into_any();
    let sections = (url_table(), session_table(), states());
    view! {
        {super::title("Tables and paging")}
        {header}
        {stack(view! {
            {sections.0}
            {sections.1}
            {sections.2}
        })}
    }
}
