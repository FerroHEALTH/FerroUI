// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tables page.
//!
//! It shows the data table with its page, size and sort in the URL,
//! the same rows paged in the page's own state, a result set whose headers
//! arrive at run time, the pagers of a listing with no stated total, and the
//! loading and empty tables.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::model::{Noun, OpenWindow, Paging, Sort, Window, href};
use ferrohealth_ui::table::view::{
    ACTIONS, Body, Column, DataTable, Listing, NUM, data_table, link_open_pager, link_pager,
};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::change::{Change, change_table};
use ferrohealth_ui::view::empty::empty_quiet;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{card, page_header, stack, titled_panel};
use ferrohealth_ui::widget::pager::{session_open_pager, session_pager};
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_query_map};

use crate::data::{Row, SORTABLE, rows, sorted};

/// What the gallery's rows are.
const FILES: Noun = Noun::new("mapping file", "mapping files");

/// The rows the synthetic server says it counted, more than it sent, so the
/// pager says the listing is part of the answer.
const COUNTED: u32 = 165;

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
                <a href="code">
                    <code>{row.name.clone()}</code>
                </a>
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
            {link_pager(
                Listing::new("Pages of the mapping files").noun(FILES).stated(Some(COUNTED)),
                window,
                &page_href,
            )}
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
            {session_pager(
                Listing::new("Pages of the mapping files, in the page").noun(FILES),
                window,
                paging,
            )}
        }
    };
    card("Paged in the page", drawn).into_any()
}

/// A result set whose columns are the query's own paths, known only once
/// the server answers, drawn in the code face.
fn result_set() -> AnyView {
    let columns: Vec<Column> = ["c/uid/value", "c/name/value", "count(*)"]
        .into_iter()
        .map(|path| Column::owned(path, path).mono())
        .collect();
    let body = (1..=3_u32)
        .map(|index| {
            view! {
                <tr>
                    <td><code>{format!("{index:08}-synthetic::node-a::1")}</code></td>
                    <td>"Synthetic vital signs"</td>
                    <td class=NUM>{(index * 7).to_string()}</td>
                </tr>
            }
        })
        .collect_view()
        .into_any();
    card(
        "A result set",
        data_table(DataTable {
            caption: "The rows the synthetic query answered",
            columns: &columns,
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Rows(body),
        }),
    )
    .into_any()
}

/// The rows a server with no total answers for `paging`: a full page until
/// the last, which is short.
fn open_rows(paging: Paging) -> u32 {
    let start = paging.page.saturating_sub(1).saturating_mul(paging.size);
    crate::data::ROWS.saturating_sub(start).min(paging.size)
}

/// The pagers of a listing whose total the server does not report: as links
/// under their own query keys, and in the page's own state.
fn open_listing() -> AnyView {
    let query = use_query_map();
    let location = use_location();
    let ehrs = Noun::new("EHR", "EHRs");
    let linked = move || {
        let paging = query
            .with(|query| Paging::read(query.get_str("open-page"), query.get_str("open-size")));
        let window = OpenWindow::of(paging, open_rows(paging));
        let path = location.pathname.get();
        let page_href = move |paging: Paging| {
            format!("{path}?open-page={}&open-size={}", paging.page, paging.size)
        };
        link_open_pager(
            Listing::new("Pages of the EHRs, as links").noun(ehrs),
            window,
            &page_href,
        )
    };
    let paging = RwSignal::new(Paging::default());
    let kept = move || {
        let window = OpenWindow::of(paging.get(), open_rows(paging.get()));
        session_open_pager(
            Listing::new("Pages of the EHRs, in the page").noun(ehrs),
            window,
            paging,
        )
    };
    card(
        "No stated total",
        view! {
            {linked}
            {kept}
        },
    )
    .into_any()
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
            empty_quiet(
                Glyph::INBOX,
                "No mapping file",
                "The bridge serves no mapping file yet.",
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

/// The change table of a staged configuration, in a panel with its action.
fn changes() -> AnyView {
    let staged = [
        Change {
            key: String::from("members.node-a.base"),
            from: Some(String::from("https://node-a.example.org/")),
            to: Some(String::from("https://node-a2.example.org/")),
        },
        Change {
            key: String::from("members.node-c"),
            from: None,
            to: Some(String::from("https://node-c.example.org/")),
        },
        Change {
            key: String::from("routing.timeout"),
            from: Some(String::from("30s")),
            to: None,
        },
    ];
    titled_panel(
        "Staged changes",
        view! {
            <button type="button" class=Variant::Primary.small()>
                "Apply"
            </button>
        },
        change_table("The staged configuration against the running one", &staged),
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
    let sections = (
        url_table(),
        session_table(),
        result_set(),
        open_listing(),
        states(),
        changes(),
    );
    view! {
        {super::title("Tables and paging")}
        {header}
        {stack(view! {
            {sections.0}
            {sections.1}
            {sections.2}
            {sections.3}
            {sections.4}
            {sections.5}
        })}
    }
}
