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
    ACTIONS, Body, Column, DataTable, Listing, NUM, data_table, data_table_unstacked, depth,
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
use crate::locale::{Msg, message, tr};

/// What the gallery's rows are, named from its catalog.
fn files() -> Noun {
    Noun::new(message(Msg::NounMappingFiles))
}

/// The rows the synthetic server says it counted, more than it sent, so the
/// pager says the listing is part of the answer.
const COUNTED: u32 = 165;

/// The columns of the gallery's table.
fn listing_columns() -> [Column; 6] {
    [
        Column::text("name", tr(Msg::FileColumn)).sortable(),
        Column::text("kind", tr(Msg::Kind)),
        Column::text("lines", tr(Msg::Lines)).numeric().sortable(),
        Column::text("changed", tr(Msg::Changed)),
        Column::text("state", tr(Msg::State)),
        Column::text("actions", tr(Msg::Actions)),
    ]
}

/// The same columns, none sorting: a table that does not sort draws no
/// sort links.
fn plain_columns() -> [Column; 6] {
    [
        Column::text("name", tr(Msg::FileColumn)),
        Column::text("kind", tr(Msg::Kind)),
        Column::text("lines", tr(Msg::Lines)).numeric(),
        Column::text("changed", tr(Msg::Changed)),
        Column::text("state", tr(Msg::State)),
        Column::text("actions", tr(Msg::Actions)),
    ]
}

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
            <td data-label=tr(Msg::Kind)>{row.kind}</td>
            <td data-label=tr(Msg::Lines) class=NUM>{row.lines.to_string()}</td>
            <td data-label=tr(Msg::Changed)>{timestamp(&row.changed)}</td>
            <td data-label=tr(Msg::State)>{badge(tone, row.state)}</td>
            <td data-label=tr(Msg::Actions) class=ACTIONS>
                <button type="button" class=Variant::Secondary.small()>
                    {tr(Msg::Open)}
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
                caption: &tr(Msg::MappingFilesPagedUrl),
                columns: &listing_columns(),
                sort: sort.as_ref(),
                sort_href: &sort_href,
                body: Body::Rows(body),
            })}
            {link_pager(
                Listing::new(tr(Msg::PagesMappingFiles)).noun(files()).stated(Some(COUNTED)),
                window,
                &page_href,
            )}
        }
    };
    card(tr(Msg::PagedUrl), drawn).into_any()
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
                caption: &tr(Msg::MappingFilesPagedPage),
                columns: &plain_columns(),
                sort: None,
                sort_href: &|_| String::new(),
                body: Body::Rows(body),
            })}
            {session_pager(
                Listing::new(tr(Msg::PagesMappingFilesPage)).noun(files()),
                window,
                paging,
            )}
        }
    };
    card(tr(Msg::PagedPage), drawn).into_any()
}

/// A result set whose columns are the query's own aliases, known only once
/// the server answers, drawn in the code face, each header titled with the
/// path it reads so no cell repeats it.
fn result_set() -> AnyView {
    let columns: Vec<Column> = [("uid", "c/uid/value"), ("name", "c/name/value")]
        .into_iter()
        .map(|(alias, path)| Column::owned(alias, alias).mono().titled(path))
        .chain([Column::owned("count", "count")
            .mono()
            .numeric()
            .titled("count(*)")])
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
                    <td data-label=tr(Msg::NameLower)>{tr(Msg::SyntheticVitalSigns)}</td>
                    <td data-label=tr(Msg::Count) class=NUM>{(index * 7).to_string()}</td>
                </tr>
            }
        })
        .collect_view();
    // A cell holding an identifier longer than its column can hold.
    let long = view! {
        <tr>
            <td><code>{tr(Msg::HttpExampleOrgFhirStructuredefinition)}</code></td>
            <td data-label=tr(Msg::NameLower)>{tr(Msg::SyntheticProfile)}</td>
            <td data-label=tr(Msg::Count) class=NUM>"1"</td>
        </tr>
    };
    let body = view! {
        {rows}
        {long}
    }
    .into_any();
    card(
        tr(Msg::ResultSet),
        data_table(DataTable {
            caption: &tr(Msg::RowsSyntheticQueryAnswered),
            columns: &columns,
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Rows(body),
        }),
    )
    .into_any()
}

/// The columns of a result that arrives in pages.
fn paged_columns() -> [Column; 3] {
    [
        Column::text("uid", tr(Msg::Composition)).mono(),
        Column::text("committed", tr(Msg::Committed)),
        Column::text("rate", tr(Msg::Rate)).numeric(),
    ]
}

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
                            <td data-label=tr(Msg::Committed)>
                                {timestamp(&format!("2026-10-1{index}T08:{index:02}:00.250+00:00"))}
                            </td>
                            <td data-label=tr(Msg::Rate) class=NUM>{(60 + index * 4).to_string()}</td>
                        </tr>
                    }
                })
                .collect_view();
            view! { <tbody data-page=page.to_string()>{rows}</tbody> }
        })
        .collect_view()
        .into_any();
    card(
        tr(Msg::ResultPages),
        data_table(DataTable {
            caption: &tr(Msg::SyntheticQuerySRowsBody),
            columns: &paged_columns(),
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Bodies(bodies),
        }),
    )
    .into_any()
}

/// The columns of a small table that fits a phone as it is.
fn fitting_columns() -> [Column; 2] {
    [
        Column::text("indicator", tr(Msg::Indicator)),
        Column::text("status", tr(Msg::Status)),
    ]
}

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
        tr(Msg::FitsPhone),
        data_table_unstacked(DataTable {
            caption: &tr(Msg::HealthIndicatorsUnstackedEveryWidth),
            columns: &fitting_columns(),
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Rows(rows),
        }),
    )
    .into_any()
}

/// The columns of the hierarchy table.
fn hierarchy_columns() -> [Column; 2] {
    [
        Column::text("code", tr(Msg::Code)).mono(),
        Column::text("display", tr(Msg::Display)),
    ]
}

/// A synthetic hierarchy, each concept's code cell stepped in by its level,
/// one row past the deepest step to show that deeper levels hold it.
fn hierarchy() -> AnyView {
    let rows = [
        (0, "root", Msg::HierarchyRoot),
        (1, "branch-a", Msg::HierarchyBranchA),
        (2, "leaf-a1", Msg::HierarchyLeafA1),
        (2, "leaf-a2", Msg::HierarchyLeafA2),
        (1, "branch-b", Msg::HierarchyBranchB),
        (2, "twig-b1", Msg::HierarchyTwigB1),
        (3, "leaf-b1x", Msg::HierarchyLeafB1x),
        (9, "deep", Msg::HierarchyDeep),
    ]
    .into_iter()
    .map(|(level, code, display)| {
        view! {
            <tr>
                <th scope="row" class=depth(level)><code>{code}</code></th>
                <td data-label=tr(Msg::Display)>{tr(display)}</td>
            </tr>
        }
    })
    .collect_view()
    .into_any();
    card(
        tr(Msg::HierarchyInTable),
        data_table(DataTable {
            caption: &tr(Msg::ConceptsSyntheticHierarchy),
            columns: &hierarchy_columns(),
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Rows(rows),
        }),
    )
    .into_any()
}

/// The columns of the table whose rows end in a link and a form.
fn acted_columns() -> [Column; 3] {
    [
        Column::text("member", tr(Msg::Member)),
        Column::text("base", tr(Msg::BaseUrl)),
        Column::text("actions", tr(Msg::Actions)),
    ]
}

/// Rows whose actions are an Edit link and a Remove form, side by side on
/// the row's first line.
fn row_actions() -> AnyView {
    let rows = ["node-a", "node-b"]
        .into_iter()
        .map(|member| {
            view! {
                <tr>
                    <th scope="row"><code>{format!("synthetic-{member}")}</code></th>
                    <td data-label=tr(Msg::BaseUrl)><code>{format!("https://{member}.example.org/")}</code></td>
                    <td data-label=tr(Msg::Actions) class=ACTIONS>
                        <a href="#edit" class=Variant::Secondary.small()>{tr(Msg::Edit)}</a>
                        " "
                        <form method="post" action="#remove">
                            <button type="submit" class=Variant::Danger.small()>{tr(Msg::Remove)}</button>
                        </form>
                    </td>
                </tr>
            }
        })
        .collect_view()
        .into_any();
    card(
        tr(Msg::RowActions),
        data_table(DataTable {
            caption: &tr(Msg::MembersEachEditLinkRemove),
            columns: &acted_columns(),
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
    let ehrs = Noun::new(message(Msg::NounEhrs));
    let linked = move || {
        let paging = query
            .with(|query| Paging::read(query.get_str("open-page"), query.get_str("open-size")));
        let window = OpenWindow::of(paging, open_rows(paging));
        let path = location.pathname.get();
        let page_href = move |paging: Paging| {
            format!("{path}?open-page={}&open-size={}", paging.page, paging.size)
        };
        link_open_pager(
            Listing::new(tr(Msg::PagesEhrsLinks)).noun(ehrs),
            window,
            &page_href,
        )
    };
    let paging = RwSignal::new(Paging::default());
    let kept = move || {
        let window = OpenWindow::of(paging.get(), open_rows(paging.get()));
        session_open_pager(
            Listing::new(tr(Msg::PagesEhrsPage)).noun(ehrs),
            window,
            paging,
        )
    };
    card(
        tr(Msg::NoStatedTotal),
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
        caption: &tr(Msg::Loading),
        columns: &plain_columns(),
        sort: None,
        sort_href: &|_| String::new(),
        body: Body::Loading(3),
    });
    let empty = data_table(DataTable {
        caption: &tr(Msg::NoRows),
        columns: &plain_columns(),
        sort: None,
        sort_href: &|_| String::new(),
        body: Body::Empty(
            empty_quiet(
                Glyph::INBOX,
                tr(Msg::NoMappingFile),
                tr(Msg::BridgeServesNoMappingFile),
            )
            .into_any(),
        ),
    });
    card(
        tr(Msg::LoadingEmpty),
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
        tr(Msg::StagedChanges),
        view! {
            <button type="button" class=Variant::Primary.small()>
                {tr(Msg::Apply)}
            </button>
        },
        change_table(tr(Msg::StagedConfigurationAgainstRunningOne), &staged),
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
        tr(Msg::TablesPaging),
        Some(tr(Msg::OneTablePagerModelUrl)),
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
        hierarchy(),
    );
    view! {
        {super::title(tr(Msg::TablesPaging))}
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
            {sections.9}
            {sections.6}
        })}
    }
}
