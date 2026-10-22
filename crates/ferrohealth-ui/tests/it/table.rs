// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The data table, its cells, the list card, the pagers and the find field
//! rendered to HTML: the sort, the filter and the paging are links and
//! forms that work with no script, a listing that keeps its position out of
//! the URL posts it, and every text they are given is escaped.

use ferrohealth_ui::i18n::Message;
use ferrohealth_ui::table::filter::{Find, find_form};
use ferrohealth_ui::table::model::{
    Direction, Noun, OpenWindow, Paging, Sort, StateSource, Window,
};
use ferrohealth_ui::table::pager::{Listing, PostTo, link_pager, post_open_pager, post_pager};
use ferrohealth_ui::table::view::{
    Body, Column, DataTable, EXCLUDED, Role, RowCheck, Sorting, cell, check_cell, count_cell,
    data_table, data_table_unstacked, group_row, menu_cell, title_cell,
};
use ferrohealth_ui::view::template::list::{ListCard, list_card};
use leptos::prelude::*;

use crate::markup::{has, tags};

fluent_templates::static_loader! {
    static CONSOLE = {
        locales: "./tests/it/console",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// The HTML `view` renders to, under a reactive owner as a server render
/// runs.
fn html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new();
    owner.with(|| view().to_html())
}

/// The fields of a listing's query: its filter, its size, its sort and its
/// page.
fn query() -> Vec<(String, String)> {
    [
        ("q", "obs"),
        ("size", "50"),
        ("sort", "name"),
        ("order", "asc"),
        ("page", "3"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect()
}

#[test]
fn the_data_table_marks_its_sort_and_the_pager_names_an_unavailable_step() {
    let columns = [
        Column::text("name", "Name").sortable(),
        Column::text("rows", "Rows").numeric(),
    ];
    let sort = Sort::read(Some("name"), Some("asc"), &["name"]);
    let table = html(|| {
        data_table(DataTable {
            caption: "Mapping files",
            columns: &columns,
            sorting: Sorting {
                sort: sort.as_ref(),
                ..Sorting::default()
            },
            body: Body::Rows(
                view! { <tr><td>"a"</td><td class="fui-num">"1"</td></tr> }.into_any(),
            ),
        })
        .into_any()
    });
    assert!(table.contains(r#"aria-sort="ascending""#), "{table}");
    assert!(table.contains("<tbody>"), "{table}");
    assert!(
        table.contains("<caption>Mapping files</caption>"),
        "{table}"
    );
    let window = Window::over(30, Paging::default());
    let pager = html(|| {
        link_pager("Pages", window, &|paging: Paging| {
            format!("/files?page={}", paging.page)
        })
        .into_any()
    });
    assert!(pager.contains("1-25 of 30"), "{pager}");
    assert_eq!(2, pager.matches("(unavailable)").count(), "{pager}");
    assert!(pager.contains(r#"href="/files?page=2""#), "{pager}");
    assert!(pager.contains("Page 1 of 2"), "{pager}");
    let loading = html(|| {
        data_table(DataTable {
            caption: "Loading",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Loading(2),
        })
        .into_any()
    });
    assert!(loading.contains(r#"aria-busy="true""#), "{loading}");
}

#[test]
fn a_pager_over_one_page_draws_the_range_and_the_sizes_alone() {
    let window = Window::over(2, Paging::default());
    let pager = html(|| {
        link_pager("Pages", window, &|paging: Paging| {
            format!("/files?page={}", paging.page)
        })
        .into_any()
    });
    assert!(pager.contains("1-2 of 2"), "{pager}");
    assert!(pager.contains("Rows per page"), "{pager}");
    assert!(!pager.contains("fui-pager-steps"), "{pager}");
    assert!(!pager.contains("(unavailable)"), "{pager}");
}

#[test]
fn an_unavailable_step_reads_its_words_and_the_suffix_apart() {
    let window = Window::over(30, Paging::default());
    let pager = html(|| {
        link_pager("Pages", window, &|paging: Paging| {
            format!("/files?page={}", paging.page)
        })
        .into_any()
    });
    assert!(
        pager.contains(r#"First</span><span class="fui-sr-only"> (unavailable)</span>"#),
        "{pager}"
    );
}

#[test]
fn a_data_table_stacks_on_a_phone_unless_it_opts_out() {
    let columns = [Column::text("name", "Name"), Column::text("kind", "Kind")];
    let rows = || view! { <tr><td>"a"</td><td data-label="Kind">"model"</td></tr> }.into_any();
    let table = html(|| {
        data_table(DataTable {
            caption: "Files",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(rows()),
        })
        .into_any()
    });
    assert!(
        table.contains(r#"class="fui-table-scroll fui-table-stack""#),
        "{table}"
    );
    let fitting = html(|| {
        data_table_unstacked(DataTable {
            caption: "Files",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(rows()),
        })
        .into_any()
    });
    assert!(fitting.contains(r#"class="fui-table-scroll""#), "{fitting}");
    assert!(!fitting.contains("fui-table-stack"), "{fitting}");
}

#[test]
fn a_table_of_several_bodies_writes_them_as_they_are_under_its_header() {
    let columns = [Column::text("uid", "Uid"), Column::text("rate", "Rate")];
    let table = html(|| {
        data_table(DataTable {
            caption: "Pages",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Bodies(
                view! {
                    <tbody data-page="1"><tr><td>"a"</td><td data-label="Rate">"1"</td></tr></tbody>
                    <tbody data-page="2"><tr><td>"b"</td><td data-label="Rate">"2"</td></tr></tbody>
                }
                .into_any(),
            ),
        })
        .into_any()
    });
    assert_eq!(table.matches("<tbody").count(), 2, "{table}");
    assert!(
        table.contains(r#"</thead><tbody data-page="1">"#),
        "{table}"
    );
    assert!(table.contains("fui-table-stack"), "{table}");
}

#[test]
fn a_header_title_rides_its_header_alone_sortable_or_not() {
    let columns = [
        Column::owned("patient", "patient")
            .mono()
            .titled("e/ehr_status/subject"),
        Column::text("uid", "Uid")
            .sortable()
            .titled(String::from("c/uid/value")),
        Column::text("rate", "Rate"),
    ];
    let table = html(|| {
        data_table(DataTable {
            caption: "Results",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(view! { <tr><td>"a"</td><td>"b"</td><td>"1"</td></tr> }.into_any()),
        })
        .into_any()
    });
    assert!(
        has(
            &table,
            "th",
            &[
                ("scope", "col"),
                ("title", "e/ehr_status/subject"),
                ("class", "fui-th-mono")
            ]
        ),
        "{table}"
    );
    assert!(
        has(
            &table,
            "th",
            &[("title", "c/uid/value"), ("aria-sort", "none")]
        ),
        "a sortable header carries its title beside its sort: {table}"
    );
    assert_eq!(
        2,
        table.matches("title=").count(),
        "only the titled headers carry one: {table}"
    );
}

#[test]
fn a_plain_header_writes_no_class_and_a_sortable_one_carries_its_sort_beside_the_rest() {
    let columns = [
        Column::text("name", "Name"),
        Column::text("rows", "Rows")
            .numeric()
            .sortable()
            .titled("rows read"),
    ];
    let sort = Sort::read(Some("rows"), Some("desc"), &["rows"]);
    let table = html(|| {
        data_table(DataTable {
            caption: "Files",
            columns: &columns,
            sorting: Sorting {
                sort: sort.as_ref(),
                ..Sorting::default()
            },
            body: Body::Rows(view! { <tr><td>"a"</td><td>"1"</td></tr> }.into_any()),
        })
        .into_any()
    });
    let headers = tags(&table, "th");
    assert_eq!(2, headers.len(), "{table}");
    assert!(
        !headers[0].contains_key("class"),
        "a plain header writes no class, not an empty one: {table}"
    );
    assert!(!table.contains(r#"class="""#), "{table}");
    // WAI-ARIA 1.2, aria-sort: on the column header the sort applies to.
    assert!(
        has(
            &table,
            "th",
            &[
                ("scope", "col"),
                ("class", "fui-num"),
                ("title", "rows read"),
                ("aria-sort", "descending")
            ]
        ),
        "{table}"
    );
}

#[test]
fn a_sortable_header_is_a_button_in_a_get_form_that_carries_the_rest_of_the_query() {
    let columns = [
        Column::text("name", "Name").sortable(),
        Column::text("rows", "Rows"),
    ];
    let sort = Sort::read(Some("name"), Some("asc"), &["name"]);
    let fields = query();
    let table = html(|| {
        data_table(DataTable {
            caption: "Files",
            columns: &columns,
            sorting: Sorting {
                sort: sort.as_ref(),
                fields: &fields,
                ..Sorting::default()
            },
            body: Body::Rows(view! { <tr><td>"a"</td><td>"1"</td></tr> }.into_any()),
        })
        .into_any()
    });
    // design/COMPONENTS.md, § The table: a sort is a button, never a link.
    assert!(!table.contains("<a "), "no link sorts: {table}");
    assert!(
        has(
            &table,
            "form",
            &[("method", "get"), ("class", "fui-sort-form")]
        ),
        "{table}"
    );
    assert!(
        !tags(&table, "form")[0].contains_key("action"),
        "a URL listing's sort submits to the page's own address: {table}"
    );
    assert!(
        has(
            &table,
            "button",
            &[
                ("type", "submit"),
                ("class", "fui-sort"),
                ("name", "sort"),
                ("value", "name")
            ]
        ),
        "{table}"
    );
    assert!(
        has(
            &table,
            "input",
            &[("type", "hidden"), ("name", "order"), ("value", "desc")]
        ),
        "a sorted column asks for the other direction: {table}"
    );
    assert!(
        has(&table, "input", &[("name", "q"), ("value", "obs")]),
        "{table}"
    );
    assert!(
        has(&table, "input", &[("name", "size"), ("value", "50")]),
        "{table}"
    );
    let names: Vec<String> = tags(&table, "input")
        .iter()
        .filter_map(|input| input.get("name").cloned())
        .collect();
    assert_eq!(
        vec!["q", "size", "order"],
        names,
        "a new sort replaces the sort and starts from the first page: {table}"
    );
    // WAI-ARIA 1.2, aria-sort: on the sorted column's header.
    assert!(has(&table, "th", &[("aria-sort", "ascending")]), "{table}");
}

#[test]
fn a_session_listing_posts_its_sort_to_the_consoles_action() {
    let columns = [Column::text("name", "Name").sortable()];
    let table = html(|| {
        data_table(DataTable {
            caption: "Patients",
            columns: &columns,
            sorting: Sorting {
                source: StateSource::Session,
                action: Some("/listing/sort"),
                ..Sorting::default()
            },
            body: Body::Rows(view! { <tr><td>"a"</td></tr> }.into_any()),
        })
        .into_any()
    });
    assert!(
        has(
            &table,
            "form",
            &[("method", "post"), ("action", "/listing/sort")]
        ),
        "{table}"
    );
    assert!(
        has(&table, "input", &[("name", "order"), ("value", "asc")]),
        "{table}"
    );
    assert!(has(&table, "th", &[("aria-sort", "none")]), "{table}");
}

#[test]
fn a_stated_order_rides_the_header_of_the_column_that_leads_it() {
    let columns = [
        Column::text("name", "Name").sortable(),
        Column::text("grade", "Grade").sortable(),
    ];
    let sort = Sort {
        column: String::from("grade"),
        direction: Direction::Descending,
    };
    let table = html(|| {
        data_table(DataTable {
            caption: "Candidates",
            columns: &columns,
            sorting: Sorting {
                sort: Some(&sort),
                stated: Some("Sorted by grade, then score"),
                ..Sorting::default()
            },
            body: Body::Rows(view! { <tr><td>"a"</td><td>"b"</td></tr> }.into_any()),
        })
        .into_any()
    });
    assert!(
        has(
            &table,
            "th",
            &[
                ("aria-sort", "descending"),
                ("title", "Sorted by grade, then score")
            ]
        ),
        "{table}"
    );
    assert_eq!(
        1,
        table.matches("title=").count(),
        "only the leading column: {table}"
    );
}

#[test]
fn a_row_menu_or_checkbox_header_names_its_column_for_assistive_technology_alone() {
    let columns = [
        Column::text("select", "Selected").role(Role::Check),
        Column::text("actions", "Actions").role(Role::Menu),
    ];
    let table = html(|| {
        data_table(DataTable {
            caption: "Files",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(().into_any()),
        })
        .into_any()
    });
    assert!(has(&table, "th", &[("class", "fui-row-check")]), "{table}");
    assert!(
        table.contains(r#"<span class="fui-sr-only">Selected</span>"#),
        "{table}"
    );
    assert!(
        table.contains(r#"<span class="fui-sr-only">Actions</span>"#),
        "{table}"
    );
}

#[test]
fn the_cells_write_their_label_and_role_and_the_name_is_the_rows_target() {
    let kind = Column::text("kind", "Kind <b>");
    let lines = Column::text("lines", "Lines").numeric().role(Role::Quiet);
    let wide = Column::text("changed", "Changed").role(Role::Wide);
    let rows = html(|| {
        view! {
            <tr class=EXCLUDED>
                {title_cell(
                    Some("/files/a?b=<c>"),
                    "synthetic <a>",
                    Some(view! { "quiet" }.into_any()),
                )}
                {cell(&kind, "model <i>")}
                {count_cell(&lines, 3, Noun::ROWS)}
                {cell(&wide, "yesterday")}
                {menu_cell(view! { <button type="button">"⋯"</button> })}
            </tr>
        }
        .into_any()
    });
    // Family rule 4: the name is a row link, the whole row its target.
    assert!(
        has(
            &rows,
            "a",
            &[("class", "fui-row-link"), ("href", "/files/a?b=&lt;c&gt;")]
        ),
        "{rows}"
    );
    assert!(rows.contains("synthetic &lt;a&gt;"), "{rows}");
    assert!(
        has(&rows, "th", &[("scope", "row"), ("data-role", "title")]),
        "{rows}"
    );
    assert!(
        rows.contains(r#"<span class="fui-row-sub">quiet</span>"#),
        "{rows}"
    );
    assert!(
        has(
            &rows,
            "td",
            &[("data-label", "Kind &lt;b&gt;"), ("data-role", "meta")]
        ),
        "{rows}"
    );
    assert!(rows.contains("<span>model &lt;i&gt;</span>"), "{rows}");
    assert!(
        has(
            &rows,
            "td",
            &[
                ("class", "fui-num"),
                ("data-role", "meta-q"),
                ("data-unit", "rows")
            ]
        ),
        "a count carries its noun for a phone: {rows}"
    );
    assert!(has(&rows, "td", &[("data-role", "wide")]), "{rows}");
    assert!(
        has(
            &rows,
            "td",
            &[("class", "fui-row-menu"), ("data-role", "menu")]
        ),
        "{rows}"
    );
    assert!(has(&rows, "tr", &[("class", "is-excluded")]), "{rows}");
}

#[test]
fn a_count_cell_takes_its_noun_from_the_consoles_catalog_by_count() {
    let profiles = Noun::new(Message::new(&*CONSOLE, "value-sets"));
    let column = Column::text("sets", "Sets").numeric();
    let one = html(|| count_cell(&column, 1, profiles));
    let many = html(|| count_cell(&column, 3, profiles));
    assert!(has(&one, "td", &[("data-unit", "value set")]), "{one}");
    assert!(has(&many, "td", &[("data-unit", "value sets")]), "{many}");
}

#[test]
fn a_row_checkbox_is_native_labelled_and_names_the_row_by_its_id() {
    let check = html(|| {
        check_cell(RowCheck {
            id: "pick-a",
            name: "files",
            value: "synthetic-a",
            label: "Select <a>",
            form: Some("bulk"),
            checked: true,
        })
    });
    assert!(
        has(
            &check,
            "td",
            &[("class", "fui-row-check"), ("data-role", "check")]
        ),
        "{check}"
    );
    assert!(
        has(
            &check,
            "input",
            &[
                ("type", "checkbox"),
                ("id", "pick-a"),
                ("name", "files"),
                ("value", "synthetic-a"),
                ("form", "bulk"),
                ("checked", "")
            ]
        ),
        "{check}"
    );
    // WCAG 2.2 SC 1.3.1: the label names the checkbox by `for`.
    assert!(
        has(
            &check,
            "label",
            &[("for", "pick-a"), ("class", "fui-sr-only")]
        ),
        "{check}"
    );
    assert!(check.contains("Select &lt;a&gt;"), "{check}");
}

#[test]
fn a_group_row_spans_every_column_and_heads_its_body() {
    let columns = [
        Column::text("a", "A"),
        Column::text("b", "B"),
        Column::text("c", "C"),
    ];
    let row = html(|| group_row(&columns, "models <x>"));
    assert!(has(&row, "tr", &[("class", "fui-group-row")]), "{row}");
    assert!(
        has(&row, "th", &[("scope", "rowgroup"), ("colspan", "3")]),
        "{row}"
    );
    assert!(row.contains("models &lt;x&gt;"), "{row}");
}

#[test]
fn the_list_card_states_an_order_of_several_keys_beside_its_count() {
    let card = |order: Option<&str>| {
        html(|| {
            list_card(ListCard {
                title: String::from("Candidates"),
                count: Some(38),
                order: order.map(str::to_owned),
                tabs: None,
                actions: None,
                body: view! { <p>"table"</p> }.into_any(),
                pager: None,
            })
        })
    };
    // design/COMPONENTS.md, T1: the documented exception, a stated multi-key sort.
    let stated = card(Some("Sorted by grade, then score <s>"));
    assert!(
        stated.contains(r#"<p class="fui-sort-note">Sorted by grade, then score &lt;s&gt;</p>"#),
        "{stated}"
    );
    let head = stated.find("fui-list-head").unwrap_or_default();
    let note = stated.find("fui-sort-note").unwrap_or_default();
    let actions = stated.find("fui-actions").unwrap_or_default();
    assert!(
        head < note && note < actions,
        "the order sits in the header row before its actions: {stated}"
    );
    let plain = card(None);
    assert!(!plain.contains("fui-sort-note"), "{plain}");
}

/// The fields a session pager posts beside the page and the size.
fn posted() -> Vec<(String, String)> {
    [
        ("listing", "files"),
        ("back", "/a?<b>"),
        ("page", "9"),
        ("size", "9"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect()
}

#[test]
fn a_session_pager_posts_the_page_and_the_size_and_names_no_url() {
    let pager = second_page();
    assert!(!pager.contains("href="), "no position in a URL: {pager}");
    assert!(
        has(
            &pager,
            "form",
            &[
                ("id", "files-steps"),
                ("method", "post"),
                ("action", "/paging")
            ]
        ),
        "{pager}"
    );
    assert!(has(&pager, "form", &[("id", "files-sizes")]), "{pager}");
    assert!(
        has(
            &pager,
            "button",
            &[
                ("type", "submit"),
                ("form", "files-steps"),
                ("name", "page"),
                ("value", "3")
            ]
        ),
        "Next carries the page after: {pager}"
    );
    assert!(
        has(
            &pager,
            "button",
            &[
                ("form", "files-steps"),
                ("value", "1"),
                ("data-page", "first")
            ]
        ),
        "{pager}"
    );
    assert!(
        has(
            &pager,
            "button",
            &[
                ("form", "files-sizes"),
                ("name", "size"),
                ("value", "50"),
                ("aria-pressed", "false")
            ]
        ),
        "{pager}"
    );
    assert!(
        has(
            &pager,
            "button",
            &[("value", "25"), ("aria-pressed", "true")]
        ),
        "{pager}"
    );
    assert!(pager.contains("26-50 of 137"), "{pager}");
    assert!(pager.contains("Page 2 of 6"), "{pager}");
}

/// The session pager of 137 rows on its second page of 25.
fn second_page() -> String {
    let fields = posted();
    html(|| {
        post_pager(
            "Pages",
            Window::over(137, Paging { page: 2, size: 25 }),
            PostTo {
                id: "files",
                action: "/paging",
                fields: &fields,
            },
        )
    })
}

#[test]
fn a_session_pager_carries_the_size_or_the_first_page_and_the_consoles_fields() {
    let pager = second_page();
    assert!(
        has(
            &pager,
            "input",
            &[("type", "hidden"), ("name", "size"), ("value", "25")]
        ),
        "a step keeps the size in force: {pager}"
    );
    assert!(
        has(
            &pager,
            "input",
            &[("type", "hidden"), ("name", "page"), ("value", "1")]
        ),
        "a size starts from the first page: {pager}"
    );
    assert!(
        has(
            &pager,
            "input",
            &[("name", "back"), ("value", "/a?&lt;b&gt;")]
        ),
        "{pager}"
    );
    assert_eq!(
        0,
        tags(&pager, "input")
            .iter()
            .filter(|input| input.get("value").map(String::as_str) == Some("9"))
            .count(),
        "a page or a size among the fields is left out: {pager}"
    );
}

#[test]
fn a_session_pager_step_that_cannot_be_taken_is_disabled_and_says_so() {
    let pager = html(|| {
        post_pager(
            "Pages",
            Window::over(30, Paging::default()),
            PostTo {
                id: "p",
                action: "/paging",
                fields: &[],
            },
        )
    });
    let disabled = tags(&pager, "button")
        .into_iter()
        .filter(|button| button.contains_key("disabled"))
        .count();
    assert_eq!(2, disabled, "First and Previous: {pager}");
    assert_eq!(2, pager.matches("(unavailable)").count(), "{pager}");
}

#[test]
fn an_open_session_pager_posts_previous_and_next() {
    let pager = html(|| {
        post_open_pager(
            "Pages",
            OpenWindow::of(Paging { page: 2, size: 25 }, 25),
            PostTo {
                id: "o",
                action: "/paging",
                fields: &[],
            },
        )
    });
    assert!(
        has(
            &pager,
            "button",
            &[("form", "o-steps"), ("value", "1"), ("data-page", "prev")]
        ),
        "{pager}"
    );
    assert!(
        has(
            &pager,
            "button",
            &[("form", "o-steps"), ("value", "3"), ("data-page", "next")]
        ),
        "{pager}"
    );
    assert!(!pager.contains("href="), "{pager}");
}

#[test]
fn the_find_field_is_a_get_form_the_server_answers() {
    let fields = query();
    let found = html(|| {
        find_form(
            Find {
                id: "find",
                name: "q",
                label: "Find a file <f>",
                value: "obs \"x\"",
                source: StateSource::Url,
                action: None,
                fields: &fields,
            },
            (),
        )
    });
    assert!(
        has(
            &found,
            "form",
            &[
                ("role", "search"),
                ("method", "get"),
                ("class", "fui-list-toolbar")
            ]
        ),
        "{found}"
    );
    assert!(!tags(&found, "form")[0].contains_key("action"), "{found}");
    assert!(
        has(
            &found,
            "input",
            &[
                ("id", "find"),
                ("name", "q"),
                ("type", "search"),
                ("value", "obs &quot;x&quot;")
            ]
        ),
        "{found}"
    );
    assert!(has(&found, "label", &[("for", "find")]), "{found}");
    assert!(found.contains("Find a file &lt;f&gt;"), "{found}");
    let hidden: Vec<String> = tags(&found, "input")
        .iter()
        .filter(|input| input.get("type").map(String::as_str) == Some("hidden"))
        .filter_map(|input| input.get("name").cloned())
        .collect();
    assert_eq!(
        vec!["size", "sort", "order"],
        hidden,
        "a new filter keeps the size and sort and starts from the first page: {found}"
    );
}

#[test]
fn a_find_field_whose_words_can_be_clinical_posts() {
    let found = html(|| {
        find_form(
            Find {
                id: "find",
                name: "words",
                label: "Find a patient",
                value: "",
                source: StateSource::Session,
                action: Some("/patients/find"),
                fields: &[],
            },
            (),
        )
    });
    // .claude/rules/leptos-ui.md §5: a clinical field is never a GET form.
    assert!(
        has(
            &found,
            "form",
            &[("method", "post"), ("action", "/patients/find")]
        ),
        "{found}"
    );
}

#[test]
fn the_link_pager_still_pages_by_links() {
    let pager = html(|| {
        link_pager(
            Listing::new("Pages"),
            Window::over(30, Paging::default()),
            &|paging| format!("/files?page={}", paging.page),
        )
    });
    assert!(pager.contains(r#"href="/files?page=2""#), "{pager}");
    assert!(!pager.contains("<form"), "{pager}");
}

#[test]
fn an_unstacked_table_keeps_its_sort_form() {
    let columns = [Column::text("name", "Name").sortable()];
    let table = html(|| {
        data_table_unstacked(DataTable {
            caption: "Files",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(().into_any()),
        })
        .into_any()
    });
    assert!(table.contains("fui-sort-form"), "{table}");
}
