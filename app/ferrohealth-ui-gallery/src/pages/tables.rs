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

use ferrohealth_ui::html::Html;
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
use ferrohealth_ui::view::action_menu::{self, Act, Action, Item, Menu};
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::change::{Change, change_table};
use ferrohealth_ui::view::empty::empty_quiet;
use ferrohealth_ui::view::filter::keeps;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{card, page_header, stack, titled_panel};
use ferrohealth_ui::view::template::list::{HeadingCount, ListCard, list_card};
use ferrohealth_ui::view::time::timestamp;

use crate::app::{PAGING_ROUTE, Page, paging_of};
use crate::data::{Row, SORTABLE, rows, sorted};
use crate::locale::{Msg, message};
use crate::markup::{h, t};
use crate::visit::Visit;

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
fn listing_columns(v: &Visit) -> [Column; 6] {
    [
        Column::text("name", v.tr(Msg::FileColumn))
            .sortable()
            .role(Role::Title),
        Column::text("kind", v.tr(Msg::Kind)),
        Column::text("lines", v.tr(Msg::Lines))
            .numeric()
            .sortable()
            .role(Role::Quiet),
        Column::text("changed", v.tr(Msg::Changed)).role(Role::Wide),
        Column::text("state", v.tr(Msg::State)),
        Column::text("actions", v.tr(Msg::Actions)).role(Role::Menu),
    ]
}

/// The same columns, none sorting: a table that does not sort draws no
/// sort forms.
fn plain_columns(v: &Visit) -> [Column; 6] {
    listing_columns(v).map(|column| Column {
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
fn row_menu(v: &Visit, name: &str) -> Html {
    let code = format!("/{}/code", v.brand.key());
    action_menu::row_menu(
        &v.locale,
        &Menu::new(
            format!("row-{name}"),
            v.tr_with(Msg::ActionsFor, &[("name", name.into())]),
            vec![Item::new(Action::new(
                Some(Glyph::FILE_CODE),
                v.tr(Msg::OpenTheDocument),
                Act::Link(code),
            ))],
        ),
    )
}

/// The table row of `row` under `columns`, its name over a canonical too
/// long for a phone, muted when the set leaves it out.
fn row(v: &Visit, row: &Row, columns: &[Column; 6]) -> Html {
    let [_, kind, size, changed, state, _] = columns;
    let canonical = t(format!(
        "https://mappings.example/fhirconnect/{}/synthetic-model-with-a-long-path",
        row.name
    ));
    let sub = h(format!(
        "<span class=\"fui-truncate fui-small\" title=\"{canonical}\">{canonical}</span>"
    ));
    let class = if row.state == "excluded" {
        format!(" class=\"{EXCLUDED}\"")
    } else {
        String::new()
    };
    h(format!(
        "<tr{class}>{}{}{}{}{}{}</tr>",
        title_cell(
            Some("code"),
            h(format!("<code>{}</code>", t(&row.name))),
            Some(sub)
        ),
        cell(kind, t(row.kind)),
        count_cell(&v.locale, size, row.lines, lines()),
        cell(changed, timestamp(&v.locale, &row.changed)),
        cell(state, badge(tone(row.state), row.state)),
        menu_cell(row_menu(v, &row.name)),
    ))
}

/// The listing whose filter, page, size and sort live in the URL: the find
/// field and the sortable headers are `GET` forms and the pager is links,
/// so each works before the page hydrates, and back and forward walk them.
fn url_table(v: &Visit) -> Html {
    let query = &v.query;
    let path = &v.path;
    let paging = Paging::from_query(query);
    let sort = Sort::from_query(query, &SORTABLE);
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
    let columns = listing_columns(v);
    let body = window
        .slice(&all)
        .iter()
        .map(|each| row(v, each, &columns))
        .collect();
    let carried = fields(query);
    let page_href = |paging: Paging| {
        let mut query = query.clone();
        paging.write(&mut query);
        href(path, &query)
    };
    let label = v.tr(Msg::FindFile);
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
        Html::empty(),
    );
    let table = data_table(DataTable {
        caption: &v.tr(Msg::MappingFilesPagedUrl),
        columns: &columns,
        sorting: Sorting {
            sort: sort.as_ref(),
            fields: &carried,
            ..Sorting::default()
        },
        body: Body::Rows(body),
    });
    let pager = link_pager(
        &v.locale,
        Listing::new(v.tr(Msg::PagesMappingFiles))
            .noun(files())
            .stated(Some(COUNTED)),
        window,
        &page_href,
    );
    let card = list_card(
        &v.locale,
        ListCard {
            title: v.tr(Msg::PagedUrl),
            count: Some(HeadingCount::All(u64::from(total))),
            order: None,
            tabs: None,
            actions: None,
            body: table,
            pager: Some(pager),
        },
    );
    h(format!("<div class=\"fui-list-page\">{find}{card}</div>"))
}

/// The same rows paged by a `POST` form whose buttons carry the page and the
/// size, as a screen that can show patient data pages them: the gallery
/// keeps the position in a session cookie and draws that page, so it never
/// reaches a URL or the history, and no row reaches the page as island data.
fn session_table(v: &Visit) -> Html {
    let paging = paging_of(&v.cookies, "files");
    let back = v.path.clone();
    let window = Window::over(crate::data::ROWS, paging);
    let columns = plain_columns(v);
    let all = rows();
    let body = window
        .slice(&all)
        .iter()
        .map(|each| row(v, each, &columns))
        .collect();
    let posted = [
        (String::from("listing"), String::from("files")),
        (String::from("back"), back),
    ];
    let table = data_table(DataTable {
        caption: &v.tr(Msg::MappingFilesPagedPage),
        columns: &columns,
        sorting: Sorting::default(),
        body: Body::Rows(body),
    });
    let pager = post_pager(
        &v.locale,
        Listing::new(v.tr(Msg::PagesMappingFilesPage)).noun(files()),
        window,
        PostTo {
            id: "files-pager",
            action: PAGING_ROUTE,
            fields: &posted,
        },
    );
    list_card(
        &v.locale,
        ListCard {
            title: v.tr(Msg::PagedPage),
            count: Some(HeadingCount::All(u64::from(crate::data::ROWS))),
            order: None,
            tabs: None,
            actions: None,
            body: table,
            pager: Some(pager),
        },
    )
}

/// The columns of the work-list.
fn review_columns(v: &Visit) -> [Column; 5] {
    [
        Column::text("select", v.tr(Msg::SelectedColumn)).role(Role::Check),
        Column::text("name", v.tr(Msg::FileColumn)).role(Role::Title),
        Column::text("lines", v.tr(Msg::Lines))
            .numeric()
            .role(Role::Quiet),
        Column::text("state", v.tr(Msg::State)).sortable(),
        Column::text("actions", v.tr(Msg::Actions)).role(Role::Menu),
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
fn review_list(v: &Visit) -> Html {
    let columns = review_columns(v);
    let [_, _, size, state, _] = &columns;
    let mut picked: Vec<Row> = rows().into_iter().take(15).collect();
    picked
        .sort_by(|a, b| (rank(a.state), a.lines, &a.name).cmp(&(rank(b.state), b.lines, &b.name)));
    let total = u32::try_from(picked.len()).unwrap_or(u32::MAX);
    let bodies = ["model", "context", "extension"]
        .into_iter()
        .map(|kind| {
            let members: Html = picked
                .iter()
                .filter(|each| each.kind == kind)
                .map(|each| {
                    let id = format!("review-{}", each.name);
                    let label = v.tr_with(Msg::SelectFile, &[("name", each.name.as_str().into())]);
                    let class = if each.state == "excluded" {
                        format!(" class=\"{EXCLUDED}\"")
                    } else {
                        String::new()
                    };
                    h(format!(
                        "<tr{class}>{}{}{}{}{}</tr>",
                        check_cell(RowCheck {
                            id: &id,
                            name: "files",
                            value: &each.name,
                            label: &label,
                            form: None,
                            checked: each.lines % 2 == 0,
                        }),
                        title_cell(
                            Some("code"),
                            h(format!("<code>{}</code>", t(&each.name))),
                            None
                        ),
                        count_cell(&v.locale, size, each.lines, lines()),
                        cell(state, badge(tone(each.state), each.state)),
                        menu_cell(row_menu(v, &each.name)),
                    ))
                })
                .collect();
            h(format!(
                "<tbody>{}{members}</tbody>",
                group_row(&columns, t(kind))
            ))
        })
        .collect();
    let order = v.tr(Msg::ReviewOrder);
    let sort = Sort {
        column: String::from("state"),
        direction: Direction::Descending,
    };
    let table = data_table(DataTable {
        caption: &v.tr(Msg::ReviewCaption),
        columns: &columns,
        sorting: Sorting {
            sort: Some(&sort),
            stated: Some(&order),
            ..Sorting::default()
        },
        body: Body::Bodies(bodies),
    });
    list_card(
        &v.locale,
        ListCard {
            title: v.tr(Msg::ReviewFiles),
            count: Some(HeadingCount::All(u64::from(total))),
            order: Some(order.clone()),
            tabs: None,
            actions: None,
            body: table,
            pager: None,
        },
    )
}

/// A result set whose columns are the query's own aliases, known only once
/// the server answers, drawn in the code face, each header titled with the
/// path it reads so no cell repeats it.
fn result_set(v: &Visit) -> Html {
    let columns: Vec<Column> = [("uid", "c/uid/value"), ("name", "c/name/value")]
        .into_iter()
        .map(|(alias, path)| Column::owned(alias, alias).mono().titled(path))
        .chain([Column::owned("count", "count")
            .mono()
            .numeric()
            .titled("count(*)")])
        .collect();
    let name = t(v.tr(Msg::NameLower));
    let count = t(v.tr(Msg::Count));
    let rows = (1..=3_u32).map(|index| {
        h(format!(
            "<tr><td><a href=\"#result-{index}\"><code>{index:08}-synthetic::node-a::1</code></a></td><td data-label=\"{name}\">{}</td><td data-label=\"{count}\" class=\"{NUM}\">{}</td></tr>",
            t(v.tr(Msg::SyntheticVitalSigns)),
            index * 7
        ))
    });
    // A cell holding an identifier longer than its column can hold.
    let long = h(format!(
        "<tr><td><code>{}</code></td><td data-label=\"{name}\">{}</td><td data-label=\"{count}\" class=\"{NUM}\">1</td></tr>",
        t(v.tr(Msg::HttpExampleOrgFhirStructuredefinition)),
        t(v.tr(Msg::SyntheticProfile))
    ));
    let body = rows.chain([long]).collect();
    card(
        v.tr(Msg::ResultSet),
        data_table(DataTable {
            caption: &v.tr(Msg::RowsSyntheticQueryAnswered),
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(body),
        }),
    )
}

/// The columns of a result that arrives in pages.
fn paged_columns(v: &Visit) -> [Column; 3] {
    [
        Column::text("uid", v.tr(Msg::Composition)).mono(),
        Column::text("committed", v.tr(Msg::Committed)),
        Column::text("rate", v.tr(Msg::Rate)).numeric(),
    ]
}

/// A result whose rows arrive a page at a time, each page its own body,
/// stacked on a phone like any other rows.
fn result_pages(v: &Visit) -> Html {
    let committed = t(v.tr(Msg::Committed));
    let rate = t(v.tr(Msg::Rate));
    let bodies = (1..=2_u32)
        .map(|page| {
            let rows: Html = (1..=2_u32)
                .map(|row| {
                    let index = (page - 1) * 2 + row;
                    h(format!(
                        "<tr><td><code>{index:08}-synthetic::node-b::1</code></td><td data-label=\"{committed}\">{}</td><td data-label=\"{rate}\" class=\"{NUM}\">{}</td></tr>",
                        timestamp(
                            &v.locale,
                            &format!("2026-10-1{index}T08:{index:02}:00.250+00:00")
                        ),
                        60 + index * 4
                    ))
                })
                .collect();
            h(format!("<tbody data-page=\"{page}\">{rows}</tbody>"))
        })
        .collect();
    card(
        v.tr(Msg::ResultPages),
        data_table(DataTable {
            caption: &v.tr(Msg::SyntheticQuerySRowsBody),
            columns: &paged_columns(v),
            sorting: Sorting::default(),
            body: Body::Bodies(bodies),
        }),
    )
}

/// The columns of a small table that fits a phone as it is.
fn fitting_columns(v: &Visit) -> [Column; 2] {
    [
        Column::text("indicator", v.tr(Msg::Indicator)),
        Column::text("status", v.tr(Msg::Status)),
    ]
}

/// A small table of two short columns, which fits a phone as it is and so
/// opts out of the stacked rows.
fn fitting(v: &Visit) -> Html {
    let rows = [
        ("cdr", Tone::Ok, "up"),
        ("terminology", Tone::Ok, "up"),
        ("store", Tone::Warn, "slow"),
    ]
    .into_iter()
    .map(|(name, tone, state)| {
        h(format!(
            "<tr><th scope=\"row\"><code>{name}</code></th><td>{}</td></tr>",
            badge(tone, state)
        ))
    })
    .collect();
    card(
        v.tr(Msg::FitsPhone),
        data_table_unstacked(DataTable {
            caption: &v.tr(Msg::HealthIndicatorsUnstackedEveryWidth),
            columns: &fitting_columns(v),
            sorting: Sorting::default(),
            body: Body::Rows(rows),
        }),
    )
}

/// The columns of the hierarchy table.
fn hierarchy_columns(v: &Visit) -> [Column; 2] {
    [
        Column::text("code", v.tr(Msg::Code)).mono(),
        Column::text("display", v.tr(Msg::Display)),
    ]
}

/// A synthetic hierarchy, each concept's code cell stepped in by its level,
/// one row past the deepest step to show that deeper levels hold it.
fn hierarchy(v: &Visit) -> Html {
    let display = t(v.tr(Msg::Display));
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
    .map(|(level, code, words)| {
        h(format!(
            "<tr><th scope=\"row\" class=\"{}\"><code>{code}</code></th><td data-label=\"{display}\">{}</td></tr>",
            depth(level),
            t(v.tr(words))
        ))
    })
    .collect();
    card(
        v.tr(Msg::HierarchyInTable),
        data_table(DataTable {
            caption: &v.tr(Msg::ConceptsSyntheticHierarchy),
            columns: &hierarchy_columns(v),
            sorting: Sorting::default(),
            body: Body::Rows(rows),
        }),
    )
}

/// The columns of the table whose rows end in a link and a form.
fn acted_columns(v: &Visit) -> [Column; 3] {
    [
        Column::text("member", v.tr(Msg::Member)),
        Column::text("base", v.tr(Msg::BaseUrl)),
        Column::text("actions", v.tr(Msg::Actions)),
    ]
}

/// Rows whose actions are an Edit link and a Remove form, side by side on
/// the row's first line.
fn row_actions(v: &Visit) -> Html {
    let base = t(v.tr(Msg::BaseUrl));
    let actions = t(v.tr(Msg::Actions));
    let rows = ["node-a", "node-b"]
        .into_iter()
        .map(|member| {
            h(format!(
                "<tr><th scope=\"row\"><code>{}</code></th><td data-label=\"{base}\"><code>{}</code></td><td data-label=\"{actions}\" class=\"{ACTIONS}\"><a href=\"#edit\" class=\"{}\">{}</a> <form method=\"post\" action=\"#remove\"><button type=\"submit\" class=\"{}\">{}</button></form></td></tr>",
                t(format!("synthetic-{member}")),
                t(format!("https://{member}.example.org/")),
                Variant::Secondary.small(),
                t(v.tr(Msg::Edit)),
                Variant::Danger.small(),
                t(v.tr(Msg::Remove))
            ))
        })
        .collect();
    card(
        v.tr(Msg::RowActions),
        data_table(DataTable {
            caption: &v.tr(Msg::MembersEachEditLinkRemove),
            columns: &acted_columns(v),
            sorting: Sorting::default(),
            body: Body::Rows(rows),
        }),
    )
}

/// The rows a server with no total answers for `paging`: a full page until
/// the last, which is short.
fn open_rows(paging: Paging) -> u32 {
    let start = paging.page.saturating_sub(1).saturating_mul(paging.size);
    crate::data::ROWS.saturating_sub(start).min(paging.size)
}

/// The pagers of a listing whose total the server does not report: as links
/// under their own query keys, and posted, the position kept out of the URL.
fn open_listing(v: &Visit) -> Html {
    let ehrs = Noun::new(message(Msg::NounEhrs));
    let paging = Paging::read(v.param("open-page"), v.param("open-size"));
    let window = OpenWindow::of(paging, open_rows(paging));
    let path = &v.path;
    let page_href =
        |paging: Paging| format!("{path}?open-page={}&open-size={}", paging.page, paging.size);
    let linked = link_open_pager(
        &v.locale,
        Listing::new(v.tr(Msg::PagesEhrsLinks)).noun(ehrs),
        window,
        &page_href,
    );
    let paging = paging_of(&v.cookies, "ehrs");
    let posted = [
        (String::from("listing"), String::from("ehrs")),
        (String::from("back"), v.path.clone()),
    ];
    let kept = post_open_pager(
        &v.locale,
        Listing::new(v.tr(Msg::PagesEhrsPage)).noun(ehrs),
        OpenWindow::of(paging, open_rows(paging)),
        PostTo {
            id: "ehrs-pager",
            action: PAGING_ROUTE,
            fields: &posted,
        },
    );
    card(v.tr(Msg::NoStatedTotal), Html::join([linked, kept]))
}

/// The table while it loads, and the table with no row.
fn states(v: &Visit) -> Html {
    let loading = data_table(DataTable {
        caption: &v.tr(Msg::Loading),
        columns: &plain_columns(v),
        sorting: Sorting::default(),
        body: Body::Loading(3),
    });
    let empty = data_table(DataTable {
        caption: &v.tr(Msg::NoRows),
        columns: &plain_columns(v),
        sorting: Sorting::default(),
        body: Body::Empty(empty_quiet(
            Glyph::INBOX,
            v.tr(Msg::NoMappingFile),
            v.tr(Msg::BridgeServesNoMappingFile),
        )),
    });
    card(v.tr(Msg::LoadingEmpty), Html::join([loading, empty]))
}

/// The change table of a staged configuration, in a panel with its action.
fn changes(v: &Visit) -> Html {
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
        v.tr(Msg::StagedChanges),
        h(format!(
            "<button type=\"button\" class=\"{}\">{}</button>",
            Variant::Primary.small(),
            t(v.tr(Msg::Apply))
        )),
        change_table(
            &v.locale,
            v.tr(Msg::StagedConfigurationAgainstRunningOne),
            &staged,
        ),
    )
}

/// The tables page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let header = page_header(
        &v.locale,
        v.tr(Msg::TablesPaging),
        Some(v.tr(Msg::OneTablePagerModelUrl)),
        ActionGroup::none(),
    );
    let body = stack(Html::join([
        url_table(v),
        session_table(v),
        review_list(v),
        result_set(v),
        result_pages(v),
        open_listing(v),
        states(v),
        fitting(v),
        row_actions(v),
        hierarchy(v),
        changes(v),
        super::records::compare_section(v),
        super::records::worklist_section(v),
    ]));
    Page::shell(v.tr(Msg::TablesPaging), Html::join([header, body]))
}
