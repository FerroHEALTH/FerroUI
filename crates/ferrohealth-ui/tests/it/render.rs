// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Every view rendered to HTML on the host, as a console's server draws it
//! and as a read-only screen hands it to the browser: what each must say,
//! what each must never carry, and the escaping of what it is given.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::island::{KeyboardTabs, ToastRegion};
use ferrohealth_ui::table::model::{Paging, Sort, Window};
use ferrohealth_ui::table::view::{
    Body, Column, DataTable, data_table, data_table_unstacked, link_pager,
};
use ferrohealth_ui::theme::Theme;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, link_button};
use ferrohealth_ui::view::chart::{LineChart, Series};
use ferrohealth_ui::view::code::{code_lines, code_panel};
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog, confirm_trigger};
use ferrohealth_ui::view::empty::{empty_state, skeleton};
use ferrohealth_ui::view::form::{
    ERROR, Field, INPUT, SELECT, action_field, described_by, error_id, field, help_id, keyed_check,
    keyed_check_field, keyed_field,
};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, refusal};
use ferrohealth_ui::view::shell::{Footer, footer, lockup, mark};
use ferrohealth_ui::view::sign_in::{Basic, Notice, Provider, SignIn, sign_in};
use ferrohealth_ui::view::status::{badge, banner, pill, status};
use ferrohealth_ui::view::surface::{
    Header, Stat, definitions, page_header, page_heading, stat_card,
};
use ferrohealth_ui::view::tabs::{Tab, link_tabs};
use ferrohealth_ui::view::theme_switch::{theme_switch, theme_toggle};
use ferrohealth_ui::widget::chart::line_chart;
use ferrohealth_ui::widget::document::{DocumentPane, document_pane_opening};
use ferrohealth_ui::widget::editor::{Editor, code_editor};
use ferrohealth_ui::widget::help::explained_check_field;
use ferrohealth_ui::widget::nav::{Entry, Group, sidebar};
use ferrohealth_ui::widget::pick_or_type::{PickOrType, Picker, pick_or_type};
use ferrohealth_ui::widget::picker::{self, picker_field};
use ferrohealth_ui::widget::tabs::tab_panels;
use ferrohealth_ui::widget::toast::{Flash, RegionWords};
use ferrohealth_ui::widget::tree::{Tree, TreeNode, tree_view};
use leptos::prelude::*;

use crate::markup::{has, tags};

/// The HTML `view` renders to, under a reactive owner as a server render
/// runs.
fn html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new();
    owner.with(|| view().to_html())
}

/// `view` rendered to HTML in a page tree with the server's shared
/// context, as a page draws an island that takes children.
fn page_html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new_root(Some(std::sync::Arc::new(
        hydration_context::SsrSharedContext::new(),
    )));
    owner.with(|| view().to_html())
}

/// A sign-in card with the methods `basic` and `provider`.
fn card(basic: bool, provider: bool, refusal: Option<&str>) -> String {
    html(|| {
        sign_in(SignIn {
            lockup: lockup("/", mark(Glyph::SERVER), "FerroBRIDGE", "Console").into_any(),
            notices: vec![
                Notice::new(Tone::Warn, "Your session ended. Sign in again to continue.")
                    .id("session-expired"),
                Notice::new(Tone::Accent, "A sandbox.\nUse synthetic / synthetic."),
            ],
            refusal: refusal.map(str::to_owned),
            basic: basic.then(|| Basic {
                action: "/login",
                next: String::from("/files?x=\"<y>\""),
            }),
            provider: provider.then(|| Provider {
                href: String::from("/auth/oidc/login"),
                label: String::from("Sign in with OIDC"),
            }),
            links: vec![(
                String::from("API reference"),
                String::from("https://example.org/docs"),
            )],
        })
        .into_any()
    })
}

#[test]
fn the_basic_sign_in_is_a_plain_post_form_a_password_manager_reads() {
    let form = card(true, false, Some("Wrong username or password."));
    assert!(form.contains(r#"method="post""#), "{form}");
    assert!(form.contains(r#"action="/login""#), "{form}");
    assert!(form.contains(r#"name="username""#), "{form}");
    assert!(form.contains(r#"autocomplete="username""#), "{form}");
    assert!(
        form.contains(r#"autocomplete="current-password""#),
        "{form}"
    );
    assert!(form.contains(r#"type="password""#), "{form}");
    assert!(form.contains(r#"name="next""#), "{form}");
    assert!(
        form.contains("&lt;y&gt;"),
        "the destination is escaped: {form}"
    );
    assert!(form.contains("Wrong username or password."), "{form}");
    assert!(form.contains(r#"id="sign-in-refused""#), "{form}");
    assert!(form.contains(r#"id="session-expired""#), "{form}");
    assert!(form.contains("A sandbox."), "both notices show: {form}");
    assert!(form.contains("API reference"), "{form}");
    assert!(form.contains("Your session ended."), "{form}");
    assert!(
        !form.contains("rel=\"external\" class=\"fui-button"),
        "no provider: {form}"
    );
    assert!(
        !form.contains("fui-divider"),
        "one method, no divider: {form}"
    );
}

#[test]
fn the_oauth_sign_in_is_one_external_link_and_both_methods_split_by_or() {
    let button = card(false, true, None);
    assert!(button.contains(r#"href="/auth/oidc/login""#), "{button}");
    assert!(button.contains(r#"rel="external""#), "{button}");
    assert!(
        button.contains("fui-button-primary"),
        "the one method stands out: {button}"
    );
    assert!(!button.contains("<form"), "{button}");
    let both = card(true, true, None);
    assert!(both.contains("fui-divider"), "{both}");
    assert!(
        both.contains("fui-button-secondary"),
        "the provider steps back: {both}"
    );
    let none = card(false, false, None);
    assert!(none.contains("No sign-in method is available"), "{none}");
}

#[test]
fn a_status_is_a_word_as_well_as_a_tint() {
    let marks = html(|| {
        view! {
            {pill(Tone::Ok, "Bridge UP")}
            {badge(Tone::Bad, "down")}
            {status(Tone::Info, Glyph::INFO, "Serving", Some(String::from("R4")))}
            {banner(Tone::Warn, Glyph::TRIANGLE_ALERT, "Sandbox", "Holds no real patient data.")}
        }
        .into_any()
    });
    for word in [
        "Bridge UP",
        "down",
        "Serving",
        "R4",
        "Sandbox",
        "fui-tone-warn",
    ] {
        assert!(marks.contains(word), "{word}: {marks}");
    }
}

#[test]
fn the_theme_switch_posts_and_marks_the_theme_in_force_by_more_than_colour() {
    let switch = html(|| {
        theme_switch(Theme::Dark, String::from("/theme"), String::from("/files")).into_any()
    });
    assert!(switch.contains(r#"action="/theme""#), "{switch}");
    assert_eq!(
        1,
        switch.matches(r#"aria-pressed="true""#).count(),
        "{switch}"
    );
    assert_eq!(3, switch.matches(r#"name="theme""#).count(), "{switch}");
    assert!(
        switch.contains(Glyph::CHECK.markup()),
        "a check on the pressed one: {switch}"
    );
}

#[test]
fn the_theme_toggle_is_one_form_that_posts_the_other_theme() {
    let toggle = html(|| theme_toggle(String::from("/theme"), String::from("/files")).into_any());
    assert!(toggle.contains(r#"method="post""#), "{toggle}");
    assert!(toggle.contains(r#"action="/theme""#), "{toggle}");
    assert!(
        has(&toggle, "input", &[("name", "back"), ("value", "/files")]),
        "{toggle}"
    );
    assert!(
        toggle.contains(r#"value="dark""#) && toggle.contains(r#"value="light""#),
        "{toggle}"
    );
    assert!(
        toggle.contains("Switch to the dark theme"),
        "named, not a bare glyph: {toggle}"
    );
    assert!(
        toggle.contains(Glyph::MOON.markup()) && toggle.contains(Glyph::SUN.markup()),
        "{toggle}"
    );
}

#[test]
fn a_refusal_lists_its_issues_with_their_text_and_counts_the_silent_ones() {
    let shown = html(|| {
        refusal(&Refusal {
            status: Some(422),
            summary: String::from("the bridge refused the request."),
            issues: vec![
                Issue::new(Severity::Information, "informational"),
                Issue::new(Severity::Error, "processing")
                    .text("no program maps <the> template")
                    .place("Bundle.entry[0]")
                    .details(vec![String::from("<b>MSG</b>")]),
            ],
            request: Some(String::from("GET /fhir?q=<script>alert(1)</script>")),
            evidence: Some(String::from("<img src=x onerror=alert(1)>")),
        })
        .into_any()
    });
    assert!(
        shown.contains("422: the bridge refused the request."),
        "{shown}"
    );
    assert!(
        shown.contains("no program maps &lt;the&gt; template"),
        "{shown}"
    );
    assert!(
        shown.contains("1 information issue with no text left out."),
        "{shown}"
    );
    assert!(!shown.contains("informational"), "{shown}");
    assert!(
        shown.contains("&lt;b&gt;MSG&lt;/b&gt;"),
        "a detail code is text: {shown}"
    );
    assert!(
        shown.contains("GET /fhir?q=&lt;script&gt;alert(1)&lt;/script&gt;"),
        "the request is text: {shown}"
    );
    assert!(
        shown.contains("&lt;img src=x onerror=alert(1)&gt;"),
        "the evidence is text: {shown}"
    );
    assert!(
        !shown.contains("<script") && !shown.contains("<img"),
        "{shown}"
    );
}

#[test]
fn a_code_panel_escapes_its_text_and_numbers_its_lines_for_a_link() {
    let panel = html(|| code_panel("The answer", Language::Json, "{\"a\": \"</pre>\"}").into_any());
    assert!(panel.contains("&lt;/pre&gt;"), "{panel}");
    assert!(panel.contains(r#"role="region""#), "{panel}");
    let lines = html(|| code_lines("The file", Language::Yaml, "a: 1\nb: 2", "L").into_any());
    assert!(lines.contains(r#"id="L2""#), "{lines}");
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
            sort: sort.as_ref(),
            sort_href: &|sort: &Sort| format!("/files?sort={}", sort.column),
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
            sort: None,
            sort_href: &|_| String::new(),
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
fn a_page_heading_in_the_body_face_writes_no_class_on_its_title() {
    let plain = html(|| page_header("Dashboard", None, ()).into_any());
    assert!(plain.contains("<h1>Dashboard</h1>"), "{plain}");
    let mono = html(|| {
        page_heading(
            Header {
                title: String::from("vitals.v1"),
                mono: true,
                ..Header::default()
            },
            (),
        )
        .into_any()
    });
    assert!(
        mono.contains(r#"<h1 class="fui-page-title-mono">vitals.v1</h1>"#),
        "{mono}"
    );
}

#[test]
fn a_document_pane_opens_on_the_view_it_is_given_and_offers_rendered_only_when_available() {
    let pane = |rendered_available: bool| {
        html(move || {
            document_pane_opening(
                DocumentPane {
                    id: "doc",
                    label: "Composition".into(),
                    formats: Vec::new(),
                    format: Signal::stored("json"),
                    text: Signal::stored(String::from("{\"a\": 1}")),
                    language: Signal::stored(Language::Json),
                    rendered: Some(view! { <p>"A reading"</p> }.into_any()),
                },
                "raw",
                Signal::stored(rendered_available),
                |_| {},
            )
        })
    };
    let raw = pane(true);
    assert!(raw.contains(r#"id="doc-highlighted""#), "{raw}");
    assert!(
        has(
            &raw,
            "button",
            &[("data-choice", "raw"), ("aria-pressed", "true")]
        ),
        "{raw}"
    );
    assert!(raw.contains(r#"data-choice="rendered""#), "{raw}");
    let plain = pane(false);
    assert!(!plain.contains(r#"data-choice="rendered""#), "{plain}");
}

#[test]
fn the_last_series_shown_cannot_be_hidden_and_each_chip_names_its_series() {
    let chart = || LineChart {
        id: "trend",
        label: String::from("Readings"),
        summary: String::from("Two series."),
        series: vec![
            Series {
                name: String::from("systolic"),
                points: vec![(0.0, 120.0), (1.0, 130.0)],
                palette: 0,
            },
            Series {
                name: String::from("diastolic"),
                points: vec![(0.0, 80.0), (1.0, 85.0)],
                palette: 1,
            },
        ],
        x_ticks: vec![(0.0, String::from("Mon")), (1.0, String::from("Tue"))],
        y_range: None,
    };
    let both = html(move || line_chart(chart(), RwSignal::new(vec![true, true])));
    assert!(both.contains(r#"data-series="systolic""#), "{both}");
    assert!(!both.contains("disabled"), "{both}");
    let one = html(move || line_chart(chart(), RwSignal::new(vec![false, true])));
    assert_eq!(1, one.matches("disabled").count(), "{one}");
    assert!(
        has(
            &one,
            "button",
            &[
                ("data-series", "diastolic"),
                ("aria-pressed", "true"),
                ("disabled", "")
            ]
        ),
        "{one}"
    );
}

#[test]
fn every_series_is_drawn_once_and_a_hidden_one_is_hidden_in_place() {
    let chart = || LineChart {
        id: "trend",
        label: String::from("Readings <mmHg>"),
        summary: String::from("Two series."),
        series: vec![
            Series {
                name: String::from("systolic"),
                points: vec![(0.0, 120.0), (1.0, 130.0)],
                palette: 0,
            },
            Series {
                name: String::from("diastolic"),
                points: vec![(0.0, 80.0), (1.0, 85.0)],
                palette: 1,
            },
        ],
        x_ticks: vec![(0.0, String::from("Mon")), (1.0, String::from("Tue"))],
        y_range: None,
    };
    let one = html(move || line_chart(chart(), RwSignal::new(vec![false, true])));
    assert!(one.contains("fui-chart-line fui-series-0"), "{one}");
    assert!(one.contains("fui-chart-line fui-series-1"), "{one}");
    let groups = tags(&one, "g");
    let hidden = groups
        .iter()
        .filter(|group| group.contains_key("hidden"))
        .count();
    assert_eq!(2, hidden, "the hidden series' lines and points: {one}");
    assert!(one.contains("Readings &lt;mmHg&gt;"), "{one}");
    assert!(
        has(
            &one,
            "svg",
            &[("aria-labelledby", "trend-title trend-desc")]
        ),
        "{one}"
    );
    assert!(
        tags(&one, "button")
            .iter()
            .all(|chip| chip.contains_key("hidden")),
        "no chip offers a press before the page hydrates: {one}"
    );
    assert!(
        one.contains("<table"),
        "the figures table reads alone: {one}"
    );
}

/// The picker over two synthetic value sets, holding `value`.
fn picker_holding(value: &str) -> String {
    let value = value.to_owned();
    html(move || {
        picker_field(picker::Picker {
            id: "run-value-set",
            name: "url",
            label: "Value set".into(),
            choose_label: "Choose a value set".into(),
            help: None,
            choices: Signal::stored(vec![
                (
                    String::from("http://example.org/vs/a"),
                    String::from("Synthetic A"),
                ),
                (
                    String::from("http://example.org/vs/b"),
                    String::from("Synthetic B"),
                ),
            ]),
            value: RwSignal::new(value),
            empty_word: "Choose one".into(),
            other_word: "Another".into(),
        })
    })
}

#[test]
fn the_server_marks_the_pickers_initial_choice_and_fills_its_input() {
    let listed = picker_holding("http://example.org/vs/b");
    let options = tags(&listed, "option");
    let chosen: Vec<_> = options
        .iter()
        .filter(|option| option.contains_key("selected"))
        .collect();
    assert_eq!(1, chosen.len(), "{listed}");
    assert_eq!(
        Some(&String::from("http://example.org/vs/b")),
        chosen.first().and_then(|option| option.get("value")),
        "{listed}"
    );
    assert!(
        has(
            &listed,
            "input",
            &[("name", "url"), ("value", "http://example.org/vs/b")]
        ),
        "the form posts the value before the page hydrates: {listed}"
    );
    let typed = picker_holding("http://example.org/vs/mine");
    assert!(
        !typed.contains('\u{0}'),
        "the parser would turn a NUL into U+FFFD: {typed}"
    );
    let last = tags(&typed, "option").pop().expect("the other choice");
    assert!(last.contains_key("selected"), "{typed}");
    assert_eq!(
        1,
        tags(&typed, "option")
            .iter()
            .filter(|option| option.contains_key("selected"))
            .count(),
        "{typed}"
    );
    let empty = picker_holding("");
    let first = tags(&empty, "option")
        .into_iter()
        .next()
        .expect("the empty choice");
    assert!(
        first.contains_key("selected") && first.get("value").is_some_and(String::is_empty),
        "{empty}"
    );
}

#[test]
fn a_tree_item_states_its_level_by_name() {
    let tree = html(|| {
        let mut parent = TreeNode::new("a", "Parent");
        parent.expandable = true;
        parent.children = vec![TreeNode::new("b", "Child")];
        tree_view(Tree {
            id: "browse",
            label: "Concepts".into(),
            nodes: Signal::stored(vec![parent]),
            expanded: RwSignal::new(std::collections::BTreeSet::from([String::from("a")])),
            selected: Signal::stored(None),
            on_expand: None,
            actions: None,
        })
    });
    assert!(tree.contains(r#"aria-level="1""#), "{tree}");
    assert!(tree.contains(r#"aria-level="2""#), "{tree}");
    assert!(!tree.contains("attr:aria-level"), "{tree}");
}

#[test]
fn a_data_table_stacks_on_a_phone_unless_it_opts_out() {
    let columns = [Column::text("name", "Name"), Column::text("kind", "Kind")];
    let rows = || view! { <tr><td>"a"</td><td data-label="Kind">"model"</td></tr> }.into_any();
    let table = html(|| {
        data_table(DataTable {
            caption: "Files",
            columns: &columns,
            sort: None,
            sort_href: &|_| String::new(),
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
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Rows(rows()),
        })
        .into_any()
    });
    assert!(fitting.contains(r#"class="fui-table-scroll""#), "{fitting}");
    assert!(!fitting.contains("fui-table-stack"), "{fitting}");
}

#[test]
fn the_surfaces_render_and_none_writes_a_style_attribute() {
    let page = html(|| {
        view! {
            {lockup("/", mark(Glyph::LAYERS), "FerroUI", "Gallery")}
            {stat_card(Stat { glyph: Glyph::USERS, value: String::from("12"), label: "Members".into(), href: Some(String::from("/members")) })}
            {definitions(vec![("Version", "0.1.0".into_any())])}
            {link_tabs("Views", &[Tab { href: "/a", label: "A".into() }, Tab { href: "/b", label: "B".into() }], "/b")}
            {empty_state(Glyph::INBOX, "Nothing here", "No file yet.", ())}
            {skeleton(3, "Loading the files.")}
            {link_button(String::from("/new"), Variant::Primary, Some(Glyph::PLUS), "New")}
            {confirm_trigger("delete", Variant::Danger, Glyph::TRASH, "Delete")}
            {confirm_dialog(Confirmation { id: "delete", title: "Delete it?".into(), message: "It goes for good.".into() }, ())}
            {footer(Footer { product: "FerroUI Gallery".into(), version: "0.1.0", about: "/about" })}
        }
        .into_any()
    });
    for expected in [
        "fui-lockup-accent",
        "fui-stat-value",
        "fui-facts",
        r#"aria-current="page""#,
        "fui-empty",
        r#"aria-busy="true""#,
        r#"popovertarget="delete""#,
        r#"popover="auto""#,
        "Cadasto B.V.",
    ] {
        assert!(page.contains(expected), "{expected}: {page}");
    }
    assert!(!page.contains("style="), "{page}");
}

#[test]
fn a_start_grid_writes_the_start_class_and_the_grid_does_not() {
    let start = html(|| ferrohealth_ui::view::surface::grid_start(view! { <p>"a"</p> }).into_any());
    assert!(
        start.contains(r#"class="fui-grid fui-grid-start""#),
        "{start}"
    );
    let plain = html(|| ferrohealth_ui::view::surface::grid(view! { <p>"a"</p> }).into_any());
    assert!(plain.contains(r#"class="fui-grid""#), "{plain}");
}

#[test]
fn a_timestamp_is_a_time_element_with_the_reading_form_and_the_raw_title() {
    let raw = "2026-10-18T23:18:37.039437+00:00";
    let drawn = html(|| ferrohealth_ui::view::time::timestamp(raw).into_any());
    for expected in [
        "<time datetime=",
        r#"class="fui-time""#,
        r#"datetime="2026-10-18T23:18:37.039Z""#,
        r#"title="2026-10-18T23:18:37.039437+00:00""#,
        ">2026-10-18 23:18:37 UTC</time>",
    ] {
        assert!(drawn.contains(expected), "{expected}: {drawn}");
    }
}

#[test]
fn a_timestamp_that_is_no_date_time_is_drawn_verbatim_and_escaped() {
    let drawn = html(|| ferrohealth_ui::view::time::timestamp("<not a date>").into_any());
    assert!(drawn.contains(r#"<span class="fui-time">"#), "{drawn}");
    assert!(drawn.contains("&lt;not a date&gt;"), "{drawn}");
    assert!(!drawn.contains("<time"), "{drawn}");
    assert!(!drawn.contains("datetime="), "{drawn}");
}

#[test]
fn a_stat_label_carries_its_whole_text_as_its_title() {
    let label = "Code systems with a long synthetic name";
    let drawn = html(|| {
        stat_card(Stat {
            glyph: Glyph::USERS,
            value: String::from("12"),
            label: label.into(),
            href: None,
        })
        .into_any()
    });
    assert!(
        drawn.contains(&format!(r#"<span title="{label}">{label}</span>"#)),
        "{drawn}"
    );
}

#[test]
fn a_table_of_several_bodies_writes_them_as_they_are_under_its_header() {
    let columns = [Column::text("uid", "Uid"), Column::text("rate", "Rate")];
    let table = html(|| {
        data_table(DataTable {
            caption: "Pages",
            columns: &columns,
            sort: None,
            sort_href: &|_| String::new(),
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
            sort: None,
            sort_href: &|sort: &Sort| format!("/q?sort={}", sort.column),
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
fn a_field_with_an_id_made_per_row_names_its_own_control_and_help() {
    let rows = ["include-0", "exclude-1"];
    let fields = html(|| {
        rows.iter()
            .map(|key| {
                let id = format!("compose-search-{key}");
                let help = help_id(&id);
                keyed_field(
                    id.clone(),
                    "Find a code",
                    Some("Type two letters or more.".to_owned()),
                    view! { <input id=id name="q" class=INPUT aria-describedby=help /> },
                )
            })
            .collect_view()
            .into_any()
    });
    for key in rows {
        let id = format!("compose-search-{key}");
        assert!(
            fields.contains(&format!(r#"<label for="{id}""#)),
            "{fields}"
        );
        assert!(
            fields.contains(&format!(r#"<p id="{id}-help""#)),
            "{fields}"
        );
        assert!(
            fields.contains(&format!(r#"aria-describedby="{id}-help""#)),
            "{fields}"
        );
    }
    let checks = html(|| {
        view! {
            {keyed_check(String::from("keep-2"), "Keep", view! { <input type="checkbox" id="keep-2" /> })}
            {keyed_check_field(
                String::from("inactive-2"),
                "Inactive codes",
                "Codes a system retired.",
                view! { <input type="checkbox" id="inactive-2" aria-describedby="inactive-2-help" /> },
            )}
        }
        .into_any()
    });
    assert!(checks.contains(r#"<label for="keep-2">"#), "{checks}");
    assert!(checks.contains(r#"<label for="inactive-2">"#), "{checks}");
    assert!(
        has(
            &checks,
            "span",
            &[("id", "inactive-2-help"), ("class", "fui-help")]
        ),
        "{checks}"
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
            sort: sort.as_ref(),
            sort_href: &|sort: &Sort| format!("/files?sort={}", sort.column),
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
fn a_field_label_is_a_static_word_or_a_string_made_at_run_time() {
    let member = String::from("node-a");
    let fields = html(move || {
        view! {
            {field("static-label", "Base URL", None, view! { <input id="static-label" class=INPUT /> })}
            {keyed_field(
                format!("base-{member}"),
                format!("Base URL of {member}"),
                None,
                view! { <input id="base-node-a" class=INPUT /> },
            )}
        }
        .into_any()
    });
    assert!(
        has(
            &fields,
            "label",
            &[("for", "static-label"), ("class", "fui-label")]
        ),
        "{fields}"
    );
    assert!(
        has(
            &fields,
            "label",
            &[("for", "base-node-a"), ("class", "fui-label")]
        ),
        "{fields}"
    );
    assert!(fields.contains(">Base URL of node-a</label>"), "{fields}");
}

#[test]
fn a_field_with_a_hidden_label_keeps_it_bound_to_its_control() {
    let row = html(|| {
        Field::new("value-3", "Value")
            .label_hidden()
            .view(view! { <input id="value-3" class=INPUT /> })
    });
    // WCAG 2.2 SC 1.3.1 and SC 4.1.2: the name stays programmatic.
    assert!(
        has(
            &row,
            "label",
            &[("for", "value-3"), ("class", "fui-sr-only")]
        ),
        "{row}"
    );
    assert!(!row.contains("fui-label"), "{row}");
}

#[test]
fn a_field_draws_its_trailing_line_after_its_help_and_the_control_names_both() {
    let id = "member-base";
    let drawn = html(move || {
        let described = described_by(id, true, true);
        Field::new(id, "Base URL")
            .help("Where the gateway reaches the member.")
            .after(view! {
                <p id=error_id(id) class=ERROR aria-live="polite">
                    "The address must be absolute."
                </p>
            })
            .view(view! { <input id=id class=INPUT aria-describedby=described /> })
    });
    assert!(
        has(
            &drawn,
            "input",
            &[
                ("id", id),
                ("aria-describedby", "member-base-help member-base-error")
            ]
        ),
        "{drawn}"
    );
    let help = drawn
        .find(r#"id="member-base-help""#)
        .expect("the help line");
    let error = drawn
        .find(r#"id="member-base-error""#)
        .expect("the error line");
    assert!(help < error, "the help reads before the refusal: {drawn}");
    assert!(
        has(
            &drawn,
            "p",
            &[("id", "member-base-error"), ("class", "fui-help fui-error")]
        ),
        "{drawn}"
    );
    assert_eq!(None, described_by(id, false, false));
    assert_eq!(
        Some(String::from("member-base-error")),
        described_by(id, false, true)
    );
    assert_eq!(
        Some(String::from("member-base-help")),
        described_by(id, true, false)
    );
}

#[test]
fn an_action_field_holds_its_button_and_error_line_with_no_label() {
    let drawn = html(|| {
        action_field(
            view! {
                <button type="button" id="add-value" aria-describedby=error_id("add-value")>
                    "Add value"
                </button>
            },
            view! { <p id=error_id("add-value") class=ERROR hidden></p> },
        )
    });
    assert!(!drawn.contains("<label"), "{drawn}");
    assert!(
        has(
            &drawn,
            "button",
            &[("id", "add-value"), ("aria-describedby", "add-value-error")]
        ),
        "{drawn}"
    );
    assert!(has(&drawn, "div", &[("class", "fui-cluster")]), "{drawn}");
    assert!(has(&drawn, "p", &[("id", "add-value-error")]), "{drawn}");
}

/// A picker over two versions or a typed one, `typing` saying which shows.
fn version_field(typing: bool, labelled: bool) -> String {
    html(move || {
        let select = view! {
            <select id="version-pick" class=SELECT aria-describedby=help_id("version")>
                <option value="">"The one this server resolves to"</option>
                <option value="other">"Another version"</option>
            </select>
        }
        .into_any();
        let picker = if labelled {
            Picker::Labelled(
                view! {
                    <div class="fui-field">
                        <label for="version-pick" class="fui-label">"Version"</label>
                        {select}
                    </div>
                }
                .into_any(),
            )
        } else {
            Picker::Control {
                id: String::from("version-pick"),
                control: select,
            }
        };
        pick_or_type(PickOrType {
            label: String::from("Version"),
            picker,
            typed_id: String::from("version"),
            typed: view! { <input id="version" name="version" class=INPUT aria-describedby=help_id("version") /> }
                .into_any(),
            typing: Signal::stored(typing),
            after: Some(view! { <p id=help_id("version") class="fui-help">"Its version."</p> }.into_any()),
        })
    })
}

#[test]
fn a_pick_or_type_field_shows_one_control_with_its_own_label() {
    for labelled in [false, true] {
        let picking = version_field(false, labelled);
        assert!(
            has(&picking, "label", &[("for", "version-pick")]),
            "{picking}"
        );
        assert!(has(&picking, "label", &[("for", "version")]), "{picking}");
        assert_eq!(
            1,
            picking.matches(r#"for="version-pick""#).count(),
            "the picker has one label: {picking}"
        );
        assert!(
            has(&picking, "div", &[("data-side", "type"), ("hidden", "")]),
            "the typed side is hidden with its label: {picking}"
        );
        assert!(
            !has(&picking, "div", &[("data-side", "pick"), ("hidden", "")]),
            "{picking}"
        );
    }
    let typing = version_field(true, false);
    assert!(
        has(&typing, "div", &[("data-side", "pick"), ("hidden", "")]),
        "{typing}"
    );
    assert!(
        !has(&typing, "div", &[("data-side", "type"), ("hidden", "")]),
        "{typing}"
    );
    let help = typing
        .find(r#"<p id="version-help""#)
        .expect("the help line");
    let typed = typing.find(r#"name="version""#).expect("the typed input");
    assert!(typed < help, "the help follows both controls: {typing}");
}

#[test]
fn an_explained_checkbox_keeps_its_help_named_and_hidden_from_sight_until_the_switch() {
    let drawn = html(|| {
        explained_check_field(
            "run-inactive",
            "Include inactive concepts",
            "An inactive concept is listed and marked.",
            view! { <input type="checkbox" id="run-inactive" aria-describedby=help_id("run-inactive") /> },
        )
    });
    assert!(has(&drawn, "label", &[("for", "run-inactive")]), "{drawn}");
    // WCAG 2.2 technique C7: hidden from sight, still in the tree.
    assert!(
        has(
            &drawn,
            "span",
            &[
                ("id", "run-inactive-help"),
                ("class", "fui-help fui-sr-only")
            ]
        ),
        "{drawn}"
    );
    assert!(has(&drawn, "div", &[("class", "fui-check")]), "{drawn}");
}

#[test]
fn a_keyed_field_row_carries_its_id_and_its_columns() {
    use ferrohealth_ui::view::layout::{FieldRow, field_row, keyed_field_row};
    let keyed = html(|| keyed_field_row(String::from("values-2-row"), FieldRow::WithActions, "x"));
    assert!(
        keyed.contains(
            r#"<div id="values-2-row" class="fui-field-row fui-field-row-actions">x</div>"#
        ),
        "{keyed}"
    );
    let plain = html(|| field_row(FieldRow::Halves, "y"));
    assert!(
        plain.contains(r#"<div class="fui-field-row fui-field-row-halves">y</div>"#),
        "{plain}"
    );
}

// A hierarchy row's leading cell steps in by its level through a class, never
// a `style` attribute, and every level past the last step keeps that step.
#[test]
fn a_row_depth_names_its_level_and_holds_at_the_deepest_step() {
    use ferrohealth_ui::table::view::depth;
    assert_eq!(depth(0), "fui-depth-0");
    assert_eq!(depth(3), "fui-depth-3");
    assert_eq!(depth(8), "fui-depth-8");
    assert_eq!(depth(9), "fui-depth-8");
    assert_eq!(depth(u32::MAX), "fui-depth-8");
    for level in 1..=8 {
        let class = depth(level);
        assert!(
            ferrohealth_ui::style::STYLESHEET.contains(&format!(".{class} {{")),
            "{class} is defined"
        );
    }
}

#[test]
fn an_editor_carries_its_text_escaped_so_a_form_posts_it_with_no_webassembly() {
    // HTML Living Standard §13.1.2.2: a text area's text is escapable raw text.
    let page = html(|| {
        code_editor(
            Editor {
                id: "draft",
                language: Language::Yaml,
                text: Signal::stored(String::from("a: </textarea><b>&")),
                marks: Signal::stored(Vec::new()),
                editable: Signal::stored(true),
            },
            |_| {},
        )
    });
    assert!(
        page.contains(">a: &lt;/textarea&gt;&lt;b&gt;&amp;</textarea>"),
        "the text area holds its text, escaped: {page}"
    );
}

#[test]
fn the_navigation_is_plain_links_with_the_page_marked_inside_its_drawer_island() {
    let page = page_html(|| {
        sidebar(
            "sidebar",
            "Pages",
            vec![Group {
                id: "nav-pages",
                title: "Pages".into(),
                entries: vec![
                    Entry {
                        href: "/files",
                        label: "Files".into(),
                        glyph: Glyph::DATABASE,
                        exact: false,
                    },
                    Entry {
                        href: "/about",
                        label: "About".into(),
                        glyph: Glyph::INFO,
                        exact: true,
                    },
                ],
            }],
            "/files/synthetic-007",
        )
    });
    assert!(
        page.contains("<leptos-island"),
        "the drawer is an island: {page}"
    );
    assert!(
        page.contains(r#"<a href="/files" aria-current="page" class="fui-nav-entry">"#),
        "the section of the page is marked: {page}"
    );
    assert!(
        page.contains(r#"<a href="/about" class="fui-nav-entry">"#),
        "another page is a plain link, unmarked: {page}"
    );
}

#[test]
fn the_keyboard_tabs_show_the_first_panel_and_hide_the_others_before_any_webassembly() {
    // WAI-ARIA Authoring Practices 1.2, the tabs pattern.
    let page = page_html(|| {
        view! {
            <KeyboardTabs
                label=String::from("Answer")
                prefix=String::from("answer")
                tabs=vec![String::from("Table"), String::from("Raw")]
            >
                {tab_panels("answer", vec![
                    view! { <p>"Rows"</p> }.into_any(),
                    view! { <p>"Text"</p> }.into_any(),
                ])}
            </KeyboardTabs>
        }
        .into_any()
    });
    assert!(page.contains(r#"id="answer-tab-0""#), "{page}");
    assert!(
        page.contains(r#"aria-selected="true""#),
        "the first tab is selected: {page}"
    );
    assert!(
        page.contains(r#"id="answer-panel-0" aria-labelledby="answer-tab-0" tabindex="0" class="fui-tab-panel">"#),
        "the first panel shows: {page}"
    );
    assert!(
        page.contains(r#"id="answer-panel-1" aria-labelledby="answer-tab-1" hidden tabindex="0""#),
        "the second panel is hidden: {page}"
    );
}

#[test]
fn the_toast_region_draws_the_flash_toasts_the_server_hands_it() {
    let page = html(|| {
        view! {
            <ToastRegion
                words=RegionWords { region: String::from("Notices"), dismiss: String::from("Dismiss") }
                flash=vec![Flash { tone: Tone::Ok, text: String::from("Saved.") }]
            />
        }
        .into_any()
    });
    assert!(
        page.contains(r#"role="status" aria-live="polite" aria-label="Notices""#),
        "{page}"
    );
    assert!(
        page.contains(r#"<p class="fui-toast-text">Saved.</p>"#),
        "the flash toast shows: {page}"
    );
    assert!(
        page.contains("Dismiss"),
        "its close button is named: {page}"
    );
}
