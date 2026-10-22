// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tables page.
//!
//! It shows the listing in its list card with the find field over it, its
//! filter, page, size and sort in the URL through links and `GET` forms; the
//! same rows paged by a `POST` form that keeps the position out of the URL;
//! a grouped work-list with a checkbox per row, an excluded row and an order
//! of several keys stated in its header row; a result set whose headers
//! arrive at run time, a result whose pages are bodies of their own, the
//! pagers of a listing with no stated total, the loading and empty tables,
//! and a small table that fits a phone unstacked. Every listing works with
//! no WebAssembly, and below 40rem every row is a line under a hairline.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::filter::{Find, find_form};
use ferrohealth_ui::table::model::{
    Direction, Noun, OpenWindow, Paging, Sort, StateSource, Window, fields, href,
};
use ferrohealth_ui::table::pager::{
    Listing, PostTo, link_open_pager, link_pager, post_open_pager, post_pager,
};
use ferrohealth_ui::table::view::{
    ACTIONS, Body, Column, DataTable, EXCLUDED, NUM, Role, RowCheck, Sorting, cell, check_cell,
    count_cell, data_table, data_table_unstacked, depth, group_row, menu_cell, title_cell,
};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::change::{Change, change_table};
use ferrohealth_ui::view::empty::empty_quiet;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{card, page_header, stack, titled_panel};
use ferrohealth_ui::view::template::list::{ListCard, list_card};
use ferrohealth_ui::view::time::timestamp;
use ferrohealth_ui::widget::action_menu::{self, Act, Action, Item, Menu};
use ferrohealth_ui::widget::filter::keeps;
use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_query_map};

use crate::app::{PAGING_ROUTE, session_paging};
use crate::data::{Row, SORTABLE, rows, sorted};
use crate::locale::{Msg, message, tr, tr_with};

/// What the gallery's rows are, named from its catalog.
fn files() -> Noun {
    Noun::new(message(Msg::NounMappingFiles))
}

/// What a file's size is counted in, named from its catalog.
fn lines() -> Noun {
    Noun::new(message(Msg::NounLines))
}

/// The rows the synthetic server says it counted, more than it sent, so the
/// pager says the listing is part of the answer.
const COUNTED: u32 = 165;

/// The query key of the listing's find field.
const FIND_KEY: &str = "q";

/// The columns of the gallery's table.
fn listing_columns() -> [Column; 6] {
    [
        Column::text("name", tr(Msg::FileColumn))
            .sortable()
            .role(Role::Title),
        Column::text("kind", tr(Msg::Kind)),
        Column::text("lines", tr(Msg::Lines))
            .numeric()
            .sortable()
            .role(Role::Quiet),
        Column::text("changed", tr(Msg::Changed)).role(Role::Wide),
        Column::text("state", tr(Msg::State)),
        Column::text("actions", tr(Msg::Actions)).role(Role::Menu),
    ]
}

/// The same columns, none sorting: a table that does not sort draws no
/// sort forms.
fn plain_columns() -> [Column; 6] {
    listing_columns().map(|column| Column {
        sortable: false,
        ..column
    })
}

/// The tone of a row's state.
fn tone(state: &str) -> Tone {
    match state {
        "refused" => Tone::Bad,
        "warning" => Tone::Warn,
        "excluded" => Tone::Neutral,
        _ => Tone::Ok,
    }
}

/// The row menu of the file `name`: the ⋯ button, named for its row, and
/// its menu (F13).
fn row_menu(name: &str) -> AnyView {
    let code = crate::app::route_brand().map_or_else(
        || String::from("/"),
        |brand| format!("/{}/code", brand.key()),
    );
    action_menu::row_menu(Menu::new(
        format!("row-{name}"),
        tr_with(Msg::ActionsFor, &[("name", name.into())]),
        vec![Item::new(Action::new(
            Some(Glyph::FILE_CODE),
            tr(Msg::OpenTheDocument),
            Act::Link(code),
        ))],
    ))
}

/// The table row of `row` under `columns`, its name over a canonical too
/// long for a phone, muted when the set leaves it out.
fn row(row: &Row, columns: &[Column; 6]) -> AnyView {
    let [_, kind, size, changed, state, _] = columns;
    let canonical = format!(
        "https://mappings.example/fhirconnect/{}/synthetic-model-with-a-long-path",
        row.name
    );
    let sub = view! {
        <span class="fui-truncate fui-small" title=canonical.clone()>
            {canonical.clone()}
        </span>
    }
    .into_any();
    let excluded = (row.state == "excluded").then_some(EXCLUDED);
    view! {
        <tr>
            {title_cell(Some("code"), view! { <code>{row.name.clone()}</code> }, Some(sub))}
            {cell(kind, row.kind)}
            {count_cell(size, row.lines, lines())}
            {cell(changed, timestamp(&row.changed))}
            {cell(state, badge(tone(row.state), row.state))}
            {menu_cell(row_menu(&row.name))}
        </tr>
    }
    .attr("class", excluded)
    .into_any()
}

/// The listing whose filter, page, size and sort live in the URL: the find
/// field and the sortable headers are `GET` forms and the pager is links,
/// so each works before the page hydrates, and back and forward walk them.
fn url_table() -> AnyView {
    let query = use_query_map();
    let location = use_location();
    let drawn = move || {
        let query = query.get();
        let path = location.pathname.get();
        let paging = Paging::from_query(&query);
        let sort = Sort::from_query(&query, &SORTABLE);
        let words = query.get_str(FIND_KEY).unwrap_or_default().to_owned();
        let found: Vec<Row> = rows()
            .into_iter()
            .filter(|row| keeps(&words, &[row.name.as_str()]))
            .collect();
        let all = match &sort {
            Some(sort) => sorted(found, &sort.column, sort.direction == Direction::Descending),
            None => found,
        };
        let total = u32::try_from(all.len()).unwrap_or(u32::MAX);
        let window = Window::over(total, paging);
        let columns = listing_columns();
        let body = window
            .slice(&all)
            .iter()
            .map(|each| row(each, &columns))
            .collect_view()
            .into_any();
        let carried = fields(&query);
        let page_href = move |paging: Paging| {
            let mut query = query.clone();
            paging.write(&mut query);
            href(&path, &query)
        };
        let label = tr(Msg::FindFile);
        let find = find_form(
            Find {
                id: "gallery-find",
                name: FIND_KEY,
                label: &label,
                value: &words,
                source: StateSource::Url,
                action: None,
                fields: &carried,
            },
            (),
        );
        let table = data_table(DataTable {
            caption: &tr(Msg::MappingFilesPagedUrl),
            columns: &columns,
            sorting: Sorting {
                sort: sort.as_ref(),
                fields: &carried,
                ..Sorting::default()
            },
            body: Body::Rows(body),
        })
        .into_any();
        let pager = link_pager(
            Listing::new(tr(Msg::PagesMappingFiles))
                .noun(files())
                .stated(Some(COUNTED)),
            window,
            &page_href,
        );
        view! {
            {find}
            {list_card(ListCard {
                title: tr(Msg::PagedUrl),
                count: Some(u64::from(total)),
                order: None,
                tabs: None,
                actions: None,
                body: table,
                pager: Some(pager),
            })}
        }
    };
    view! { <div class="fui-list-page">{drawn}</div> }.into_any()
}

/// The same rows paged by a `POST` form whose buttons carry the page and the
/// size, as a screen that can show patient data pages them: the gallery
/// keeps the position in a session cookie and draws that page, so it never
/// reaches a URL or the history, and no row reaches the page as island data.
fn session_table() -> AnyView {
    let paging = session_paging("files");
    let back = use_location().pathname.get_untracked();
    let window = Window::over(crate::data::ROWS, paging);
    let columns = plain_columns();
    let all = rows();
    let body = window
        .slice(&all)
        .iter()
        .map(|each| row(each, &columns))
        .collect_view()
        .into_any();
    let posted = [
        (String::from("listing"), String::from("files")),
        (String::from("back"), back),
    ];
    let table = data_table(DataTable {
        caption: &tr(Msg::MappingFilesPagedPage),
        columns: &columns,
        sorting: Sorting::default(),
        body: Body::Rows(body),
    })
    .into_any();
    let pager = post_pager(
        Listing::new(tr(Msg::PagesMappingFilesPage)).noun(files()),
        window,
        PostTo {
            id: "files-pager",
            action: PAGING_ROUTE,
            fields: &posted,
        },
    );
    list_card(ListCard {
        title: tr(Msg::PagedPage),
        count: Some(u64::from(crate::data::ROWS)),
        order: None,
        tabs: None,
        actions: None,
        body: table,
        pager: Some(pager),
    })
}

/// The columns of the work-list.
fn review_columns() -> [Column; 5] {
    [
        Column::text("select", tr(Msg::SelectedColumn)).role(Role::Check),
        Column::text("name", tr(Msg::FileColumn)).role(Role::Title),
        Column::text("lines", tr(Msg::Lines))
            .numeric()
            .role(Role::Quiet),
        Column::text("state", tr(Msg::State)).sortable(),
        Column::text("actions", tr(Msg::Actions)).role(Role::Menu),
    ]
}

/// The rank of a state in the work-list's order: the worst first.
fn rank(state: &str) -> u8 {
    match state {
        "refused" => 0,
        "warning" => 1,
        "compiled" => 2,
        _ => 3,
    }
}

/// A work-list grouped by kind, each group a body under its group row, a
/// checkbox at the start of every row, an excluded row, and its order of
/// three keys stated in the card's header row, the leading column carrying
/// the arrow.
fn review_list() -> AnyView {
    let columns = review_columns();
    let [_, _, size, state, _] = &columns;
    let mut picked: Vec<Row> = rows().into_iter().take(15).collect();
    picked
        .sort_by(|a, b| (rank(a.state), a.lines, &a.name).cmp(&(rank(b.state), b.lines, &b.name)));
    let total = u32::try_from(picked.len()).unwrap_or(u32::MAX);
    let bodies = ["model", "context", "extension"]
        .into_iter()
        .map(|kind| {
            let members = picked
                .iter()
                .filter(|each| each.kind == kind)
                .map(|each| {
                    let id = format!("review-{}", each.name);
                    let label = tr_with(Msg::SelectFile, &[("name", each.name.as_str().into())]);
                    let excluded = (each.state == "excluded").then_some(EXCLUDED);
                    view! {
                        <tr>
                            {check_cell(RowCheck {
                                id: &id,
                                name: "files",
                                value: &each.name,
                                label: &label,
                                form: None,
                                checked: each.lines % 2 == 0,
                            })}
                            {title_cell(Some("code"), view! { <code>{each.name.clone()}</code> }, None)}
                            {count_cell(size, each.lines, lines())}
                            {cell(state, badge(tone(each.state), each.state))}
                            {menu_cell(row_menu(&each.name))}
                        </tr>
                    }
                    .attr("class", excluded)
                })
                .collect_view();
            view! {
                <tbody>
                    {group_row(&columns, kind)}
                    {members}
                </tbody>
            }
        })
        .collect_view()
        .into_any();
    let order = tr(Msg::ReviewOrder);
    let sort = Sort {
        column: String::from("state"),
        direction: Direction::Descending,
    };
    let table = data_table(DataTable {
        caption: &tr(Msg::ReviewCaption),
        columns: &columns,
        sorting: Sorting {
            sort: Some(&sort),
            stated: Some(&order),
            ..Sorting::default()
        },
        body: Body::Bodies(bodies),
    })
    .into_any();
    list_card(ListCard {
        title: tr(Msg::ReviewFiles),
        count: Some(u64::from(total)),
        order: Some(order.clone()),
        tabs: None,
        actions: None,
        body: table,
        pager: None,
    })
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
            sorting: Sorting::default(),
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
            sorting: Sorting::default(),
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
            sorting: Sorting::default(),
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
            sorting: Sorting::default(),
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
            sorting: Sorting::default(),
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
/// under their own query keys, and posted, the position kept out of the URL.
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
    let paging = session_paging("ehrs");
    let posted = [
        (String::from("listing"), String::from("ehrs")),
        (String::from("back"), location.pathname.get_untracked()),
    ];
    let kept = post_open_pager(
        Listing::new(tr(Msg::PagesEhrsPage)).noun(ehrs),
        OpenWindow::of(paging, open_rows(paging)),
        PostTo {
            id: "ehrs-pager",
            action: PAGING_ROUTE,
            fields: &posted,
        },
    );
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
        sorting: Sorting::default(),
        body: Body::Loading(3),
    });
    let empty = data_table(DataTable {
        caption: &tr(Msg::NoRows),
        columns: &plain_columns(),
        sorting: Sorting::default(),
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
        ActionGroup::none(),
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
        review_list(),
    );
    view! {
        {super::title(tr(Msg::TablesPaging))}
        {header}
        {stack(view! {
            {sections.0}
            {sections.1}
            {sections.10}
            {sections.2}
            {sections.3}
            {sections.4}
            {sections.5}
            {sections.7}
            {sections.8}
            {sections.9}
            {sections.6}
            {super::records::compare_section()}
            {super::records::worklist_section()}
        })}
    }
}
