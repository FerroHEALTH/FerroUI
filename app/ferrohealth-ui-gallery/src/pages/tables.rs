// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tables page.
//!
//! It shows the data table, stacked as cards on a phone, with its page, size
//! and sort in the URL, the same rows paged in the page's own state, a result
//! set whose headers arrive at run time, a result whose pages are bodies of
//! their own, the pagers of a listing with no stated total, the loading and
//! empty tables, and a small table that fits a phone unstacked.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::model::{Noun, OpenWindow, Paging, Sort, Window, href};
use ferrohealth_ui::table::view::{
    ACTIONS, Body, Column, DataTable, Listing, NUM, data_table, data_table_unstacked,
    link_open_pager, link_pager,
};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::change::{Change, change_table};
use ferrohealth_ui::view::empty::empty_quiet;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{card, page_header, stack, titled_panel};
use ferrohealth_ui::view::time::timestamp;
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
const COLUMNS: [Column; 6] = [
    Column::text("name", "File").sortable(),
    Column::text("kind", "Kind"),
    Column::text("lines", "Lines").numeric().sortable(),
    Column::text("changed", "Changed"),
    Column::text("state", "State"),
    Column::text("actions", "Actions"),
];

/// The same columns, none sorting: a table that does not sort draws no
/// sort links.
const PLAIN: [Column; 6] = [
    Column::text("name", "File"),
    Column::text("kind", "Kind"),
    Column::text("lines", "Lines").numeric(),
    Column::text("changed", "Changed"),
    Column::text("state", "State"),
    Column::text("actions", "Actions"),
];

/// The table row of `row`, its name over a canonical too long for a phone.
fn row(row: &Row) -> AnyView {
    let canonical = format!(
        "https://mappings.example/fhirconnect/{}/synthetic-model-with-a-long-path",
        row.name
    );
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
                <span class="fui-truncate fui-small fui-muted" title=canonical.clone()>
                    {canonical.clone()}
                </span>
            </th>
            <td data-label="Kind">{row.kind}</td>
            <td data-label="Lines" class=NUM>{row.lines.to_string()}</td>
            <td data-label="Changed">{timestamp(&row.changed)}</td>
            <td data-label="State">{badge(tone, row.state)}</td>
            <td data-label="Actions" class=ACTIONS>
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
    let rows = (1..=3_u32)
        .map(|index| {
            view! {
                <tr>
                    <td>
                        <a href=format!("#result-{index}")>
                            <code>{format!("{index:08}-synthetic::node-a::1")}</code>
                        </a>
                    </td>
                    <td data-label="c/name/value">"Synthetic vital signs"</td>
                    <td data-label="count(*)" class=NUM>{(index * 7).to_string()}</td>
                </tr>
            }
        })
        .collect_view();
    // A cell holding an identifier longer than its column can hold.
    let long = view! {
        <tr>
            <td><code>"http://example.org/fhir/StructureDefinition/ferrohealth-synthetic-journey-diagnose"</code></td>
            <td data-label="c/name/value">"Synthetic profile"</td>
            <td data-label="count(*)" class=NUM>"1"</td>
        </tr>
    };
    let body = view! {
        {rows}
        {long}
    }
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

/// The columns of a result that arrives in pages.
const PAGED: [Column; 3] = [
    Column::text("uid", "Composition").mono(),
    Column::text("committed", "Committed"),
    Column::text("rate", "Rate").numeric(),
];

/// A result whose rows arrive a page at a time, each page its own body,
/// stacked on a phone like any other rows.
fn result_pages() -> AnyView {
    let bodies = (1..=2_u32)
        .map(|page| {
            let rows = (1..=2_u32)
                .map(|row| {
                    let index = (page - 1) * 2 + row;
                    view! {
                        <tr>
                            <td><code>{format!("{index:08}-synthetic::node-b::1")}</code></td>
                            <td data-label="Committed">
                                {timestamp(&format!("2026-10-1{index}T08:{index:02}:00.250+00:00"))}
                            </td>
                            <td data-label="Rate" class=NUM>{(60 + index * 4).to_string()}</td>
                        </tr>
                    }
                })
                .collect_view();
            view! { <tbody data-page=page.to_string()>{rows}</tbody> }
        })
        .collect_view()
        .into_any();
    card(
        "A result in pages",
        data_table(DataTable {
            caption: "The synthetic query's rows, a body per page",
            columns: &PAGED,
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Bodies(bodies),
        }),
    )
    .into_any()
}

/// The columns of a small table that fits a phone as it is.
const FITTING: [Column; 2] = [
    Column::text("indicator", "Indicator"),
    Column::text("status", "Status"),
];

/// A small table of two short columns, which fits a phone as it is and so
/// opts out of the stacked rows.
fn fitting() -> AnyView {
    let rows = [
        ("cdr", Tone::Ok, "up"),
        ("terminology", Tone::Ok, "up"),
        ("store", Tone::Warn, "slow"),
    ]
    .into_iter()
    .map(|(name, tone, state)| {
        view! {
            <tr>
                <th scope="row"><code>{name}</code></th>
                <td>{badge(tone, state)}</td>
            </tr>
        }
    })
    .collect_view()
    .into_any();
    card(
        "Fits a phone as it is",
        data_table_unstacked(DataTable {
            caption: "Health indicators, unstacked at every width",
            columns: &FITTING,
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Rows(rows),
        }),
    )
    .into_any()
}

/// The columns of the table whose rows end in a link and a form.
const ACTED: [Column; 3] = [
    Column::text("member", "Member"),
    Column::text("base", "Base URL"),
    Column::text("actions", "Actions"),
];

/// Rows whose actions are an Edit link and a Remove form, side by side on
/// the row's first line.
fn row_actions() -> AnyView {
    let rows = ["node-a", "node-b"]
        .into_iter()
        .map(|member| {
            view! {
                <tr>
                    <th scope="row"><code>{format!("synthetic-{member}")}</code></th>
                    <td data-label="Base URL"><code>{format!("https://{member}.example.org/")}</code></td>
                    <td data-label="Actions" class=ACTIONS>
                        <a href="#edit" class=Variant::Secondary.small()>"Edit"</a>
                        " "
                        <form method="post" action="#remove">
                            <button type="submit" class=Variant::Danger.small()>"Remove"</button>
                        </form>
                    </td>
                </tr>
            }
        })
        .collect_view()
        .into_any();
    card(
        "Row actions",
        data_table(DataTable {
            caption: "Members, each with an Edit link and a Remove form",
            columns: &ACTED,
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Rows(rows),
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
        result_pages(),
        open_listing(),
        states(),
        changes(),
        fitting(),
        row_actions(),
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
            {sections.7}
            {sections.8}
            {sections.6}
        })}
    }
}
