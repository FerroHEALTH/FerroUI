// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Every view rendered to HTML on the host, as a console's server draws it
//! and as a read-only screen hands it to the browser: what each must say,
//! what each must never carry, and the escaping of what it is given.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::i18n::Phrase;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::island::{Combobox, KeyboardTabs, PickerField, ToastRegion};
use ferrohealth_ui::theme::Theme;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::{ActionGroup, action_button, action_group};
use ferrohealth_ui::view::button::{Variant, link_button};
use ferrohealth_ui::view::chart::{LineChart, Series};
use ferrohealth_ui::view::code::{
    Code, CodeHead, Digest, Gravity, LineDiagnostic, code_card, code_section, code_viewer,
};
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog};
use ferrohealth_ui::view::document::{Document, document_pane};
use ferrohealth_ui::view::empty::{empty_state, skeleton};
use ferrohealth_ui::view::form::{
    ERROR, Field, INPUT, SELECT, action_field, described_by, error_id, field, help_id, keyed_check,
    keyed_check_field, keyed_field,
};
use ferrohealth_ui::view::popover::{PopoverForm, popover_form};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, refusal};
use ferrohealth_ui::view::shell::{Footer, footer, lockup, mark};
use ferrohealth_ui::view::surface::{
    Header, SectionHead, Stat, add_row, definitions, page_header, page_heading, panel_section,
    panel_sections, section_intro, source_split, stat_card,
};
use ferrohealth_ui::view::tabs::{Tab, binder};
use ferrohealth_ui::view::theme_switch::theme_switch;
use ferrohealth_ui::widget::action_menu::{
    Act, Action, Choice, Item, Listbox, Menu, Pick, listbox, row_menu,
};
use ferrohealth_ui::widget::chart::{ChartWords, line_chart};
use ferrohealth_ui::widget::combobox::{ComboOption, ComboboxWords};
use ferrohealth_ui::widget::document::DocumentView;
use ferrohealth_ui::widget::editor::{Editor, code_editor};
use ferrohealth_ui::widget::help::explained_check_field;
use ferrohealth_ui::widget::pick_or_type::{PickOrType, Picker, pick_or_type};
use ferrohealth_ui::widget::picker::{self, picker_field};
use ferrohealth_ui::widget::tabs::tab_panels;
use ferrohealth_ui::widget::toast::{Flash, RegionWords, ToastLink};
use ferrohealth_ui::widget::tree::{Tree, TreeNode, tree_view};
use leptos::prelude::*;

use crate::markup::{has, tags};

/// The HTML `view` renders to, under a reactive owner as a server render
/// runs.
pub(crate) fn html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new();
    owner.with(|| view().to_html())
}

/// `view` rendered to HTML in a page tree with the server's shared
/// context, as a page draws an island that takes children.
pub(crate) fn page_html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new_root(Some(std::sync::Arc::new(
        hydration_context::SsrSharedContext::new(),
    )));
    owner.with(|| view().to_html())
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

/// A code viewer's text: a long line, a current line and two findings.
fn code(text: &str) -> Code {
    Code {
        label: String::from("The <file>"),
        language: Language::Yaml,
        text: text.to_owned(),
        ids: Some(String::from("f-L")),
        current: Some(2),
        diagnostics: vec![
            LineDiagnostic {
                line: 1,
                gravity: Gravity::Error,
                code: String::from("FC-<1>"),
                message: String::from("An <unknown> key."),
            },
            LineDiagnostic {
                line: 3,
                gravity: Gravity::Warning,
                code: String::from("FC-2"),
                message: String::from("A key read twice."),
            },
        ],
    }
}

#[test]
fn a_code_viewer_escapes_its_text_numbers_its_lines_and_draws_each_finding_under_its_line() {
    let viewer = html(|| code_viewer(None, &code("a: \"</pre>\"\nb: 2\nc: 3\n")));
    assert!(viewer.contains("&lt;/pre&gt;"), "{viewer}");
    assert!(
        !viewer.contains("<unknown>") && !viewer.contains("FC-<1>"),
        "{viewer}"
    );
    assert!(viewer.contains("An &lt;unknown&gt; key."), "{viewer}");
    assert!(
        has(
            &viewer,
            "span",
            &[("id", "f-L1"), ("class", "fui-code-line fui-mark-error")]
        ),
        "{viewer}"
    );
    assert!(
        has(
            &viewer,
            "span",
            &[("id", "f-L2"), ("class", "fui-code-line is-current")]
        ),
        "{viewer}"
    );
    assert!(
        !viewer.contains(r#"id="f-L4""#),
        "a final line break opens no line: {viewer}"
    );
    let first = viewer.find(r#"id="f-L1""#).unwrap_or(usize::MAX);
    let finding = viewer.find("fui-code-diag fui-tone-bad").unwrap_or(0);
    let second = viewer.find(r#"id="f-L2""#).unwrap_or(0);
    assert!(
        first < finding && finding < second,
        "the finding sits under its line: {viewer}"
    );
    assert!(
        has(
            &viewer,
            "span",
            &[("class", "fui-code-diag fui-tone-warn"), ("role", "note")]
        ),
        "{viewer}"
    );
    assert!(
        has(
            &viewer,
            "div",
            &[
                (
                    "class",
                    "fui-code-panel fui-code-panel-numbered fui-scroll-cue"
                ),
                ("role", "region"),
                ("tabindex", "0"),
                ("aria-label", "The &lt;file&gt;, one numbered line per line"),
            ]
        ),
        "{viewer}"
    );
    assert!(!viewer.contains("fui-codeview-head"), "no head: {viewer}");
    assert!(!viewer.contains("style="), "{viewer}");
}

#[test]
fn a_code_card_names_its_path_in_its_head_with_the_tags_the_count_and_the_digest() {
    let card = html(|| {
        code_card(
            CodeHead {
                title: String::from("fhirconnect/a <b>.yml"),
                tags: vec![String::from("FHIRconnect")],
                digest: Some(Digest {
                    text: String::from("SHA-256 5712…"),
                    title: String::from("SHA-256 of the bytes"),
                }),
                actions: None,
            },
            &code("a: 1\nb: 2\nc: 3"),
        )
    });
    assert!(
        has(&card, "h2", &[("class", "fui-codeview-path")])
            && card.contains("fhirconnect/<wbr>a &lt;b&gt;.yml"),
        "the path is the title and breaks after its slash alone: {card}"
    );
    assert!(
        has(&card, "section", &[("class", "fui-card fui-codeview-card")]),
        "{card}"
    );
    assert!(
        card.contains(r#"<span class="fui-tag">FHIRconnect</span>"#),
        "{card}"
    );
    assert!(card.contains("3 lines"), "{card}");
    assert!(
        has(
            &card,
            "span",
            &[
                ("class", "fui-codeview-sha"),
                ("title", "SHA-256 of the bytes")
            ]
        ),
        "{card}"
    );
}

#[test]
fn a_code_section_carries_the_head_in_its_heading_row_and_the_viewer_has_none() {
    let section = html(|| {
        code_section(
            CodeHead {
                title: String::from("Source"),
                ..CodeHead::default()
            },
            &code("a: 1"),
        )
    });
    assert!(
        section.contains(r#"<h3 class="fui-panel-section-title">Source</h3>"#),
        "{section}"
    );
    assert!(section.contains("1 line<"), "{section}");
    assert!(!section.contains("fui-codeview-head"), "{section}");
    assert!(section.contains(r#"class="fui-codeview""#), "{section}");
}

#[test]
fn panel_sections_draw_a_heading_row_with_its_count_and_no_card_of_their_own() {
    let panel = html(|| {
        panel_sections(view! {
            {panel_section(
                Some(SectionHead {
                    title: String::from("Models <x>"),
                    count: Some(3),
                    actions: Some(view! { <a href="/add">"Add"</a> }.into_any()),
                }),
                view! {
                    {section_intro("One <sentence>.")}
                    {add_row(view! { <button type="button">"Add"</button> })}
                },
            )}
            {panel_section(None, view! { <p>"Prose"</p> })}
        })
    });
    assert!(panel.starts_with(r#"<div class="fui-panel">"#), "{panel}");
    assert!(
        !panel.contains("fui-card"),
        "never a card in the panel: {panel}"
    );
    assert_eq!(2, tags(&panel, "section").len(), "{panel}");
    assert!(panel.contains("Models &lt;x&gt;"), "{panel}");
    assert!(
        panel.contains(r#"<span class="fui-list-count">· 3</span>"#),
        "{panel}"
    );
    assert!(has(&panel, "div", &[("class", "fui-actions")]), "{panel}");
    assert!(panel.contains("One &lt;sentence&gt;."), "{panel}");
    assert!(
        has(
            &panel,
            "p",
            &[("class", "fui-small fui-muted fui-section-intro")]
        ),
        "{panel}"
    );
    assert!(
        has(&panel, "div", &[("class", "fui-cluster fui-add-row")]),
        "{panel}"
    );
    assert_eq!(
        1,
        tags(&panel, "h3").len(),
        "a section with no head draws no heading: {panel}"
    );
    let split = html(|| {
        source_split(
            code_viewer(
                Some(CodeHead {
                    title: String::from("a.yml"),
                    ..CodeHead::default()
                }),
                &code("a: 1"),
            ),
            panel_section(None, "Diagnostics".into_any()),
        )
    });
    assert!(
        split.starts_with(r#"<div class="fui-panel fui-source-split">"#),
        "{split}"
    );
    assert!(
        has(&split, "h3", &[("class", "fui-codeview-path")]) && split.contains(">a.yml<"),
        "{split}"
    );
}

#[test]
fn a_page_heading_in_the_body_face_writes_no_class_on_its_title() {
    let plain = html(|| page_header("Dashboard", None, ActionGroup::none()).into_any());
    assert!(plain.contains("<h1>Dashboard</h1>"), "{plain}");
    let mono = html(|| {
        page_heading(
            Header {
                title: String::from("vitals.v1"),
                mono: true,
                ..Header::default()
            },
            ActionGroup::none(),
        )
        .into_any()
    });
    assert!(
        mono.contains(r#"<h1 class="fui-page-title-mono">vitals.v1</h1>"#),
        "{mono}"
    );
}

/// A document pane over `text`, opening on `opening`, with a rendered view
/// when `rendered`.
fn pane(opening: DocumentView, rendered: bool) -> String {
    page_html(move || {
        document_pane(Document {
            id: String::from("doc"),
            label: String::from("Composition"),
            formats: vec![
                (String::from("?format=json"), String::from("JSON")),
                (String::from("?format=xml"), String::from("XML")),
            ],
            format: 0,
            language: Language::Json,
            text: String::from("{\"a\": \"</pre>\"}"),
            opening,
            rendered: rendered.then(|| view! { <p>"A reading"</p> }.into_any()),
        })
    })
}

#[test]
fn a_document_pane_draws_every_view_on_the_server_each_reached_by_a_link() {
    let raw = pane(DocumentView::Raw, true);
    for view in ["highlighted", "raw", "rendered"] {
        assert!(
            has(
                &raw,
                "div",
                &[
                    ("id", &format!("doc-{view}")),
                    ("data-view", view),
                    ("class", "fui-document-view")
                ]
            ),
            "the {view} view is drawn: {raw}"
        );
        assert!(
            has(
                &raw,
                "a",
                &[("href", &format!("#doc-{view}")), ("data-view", view)]
            ),
            "a link reaches the {view} view: {raw}"
        );
    }
    assert!(
        has(&raw, "a", &[("data-view", "raw"), ("aria-current", "true")]),
        "{raw}"
    );
    assert!(has(&raw, "div", &[("data-shown", "raw")]), "{raw}");
    assert!(
        raw.contains("<leptos-island"),
        "the switch is an island: {raw}"
    );
    assert!(
        raw.contains("<leptos-children"),
        "the views are its server children: {raw}"
    );
    assert!(raw.contains("&lt;/pre&gt;"), "{raw}");
    assert!(!raw.contains("</pre>\""), "{raw}");
    assert!(
        has(
            &raw,
            "a",
            &[("href", "?format=json"), ("aria-current", "true")]
        ),
        "{raw}"
    );
    assert!(
        !raw.contains("<button"),
        "no control works only with script: {raw}"
    );
}

#[test]
fn a_document_pane_with_no_rendered_view_offers_none_and_opens_highlighted() {
    let plain = pane(DocumentView::Rendered, false);
    assert!(!plain.contains(r#"data-view="rendered""#), "{plain}");
    assert!(
        has(&plain, "div", &[("data-shown", "highlighted")]),
        "{plain}"
    );
}

/// A line chart of two synthetic series, named `label`.
fn trend(label: &str) -> LineChart {
    LineChart {
        id: "trend",
        label: label.to_owned(),
        summary: String::from("Two series."),
        series: vec![
            Series {
                name: String::from("systolic"),
                points: vec![(0.0, 120.0), (1.0, 130.0)],
                palette: 0,
            },
            Series {
                name: String::from("diastolic <b>"),
                points: vec![(0.0, 80.0), (1.0, 85.0)],
                palette: 1,
            },
        ],
        x_ticks: vec![(0.0, String::from("Mon")), (1.0, String::from("Tue"))],
        y_range: None,
    }
}

#[test]
fn the_legend_is_a_static_key_before_the_page_hydrates() {
    let page = html(|| line_chart(&trend("Readings"), ChartWords::of_page()));
    assert!(
        page.contains("<leptos-island"),
        "the legend is an island: {page}"
    );
    let keys = tags(&page, "span")
        .into_iter()
        .filter(|span| span.get("class").map(String::as_str) == Some("fui-legend-key"))
        .filter_map(|span| span.get("data-series").cloned())
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            String::from("systolic"),
            String::from("diastolic &lt;b&gt;")
        ],
        keys,
        "every series is named beside its swatch, escaped: {page}"
    );
    assert!(page.contains("fui-swatch fui-series-0"), "{page}");
    assert!(page.contains("fui-swatch fui-series-1"), "{page}");
    assert!(
        tags(&page, "button").is_empty(),
        "no chip offers a press before the page hydrates: {page}"
    );
    assert!(
        has(&page, "div", &[("role", "group"), ("aria-label", "Series")]),
        "{page}"
    );
}

#[test]
fn every_series_is_drawn_once_and_shown_in_the_plot_the_legend_names() {
    let page = html(|| line_chart(&trend("Readings <mmHg>"), ChartWords::of_page()));
    assert!(page.contains("fui-chart-line fui-series-0"), "{page}");
    assert!(page.contains("fui-chart-line fui-series-1"), "{page}");
    assert!(
        tags(&page, "g")
            .iter()
            .all(|group| !group.contains_key("hidden")),
        "every series shows as the page arrives: {page}"
    );
    assert!(page.contains("Readings &lt;mmHg&gt;"), "{page}");
    assert!(
        has(
            &page,
            "div",
            &[("id", "trend-plot"), ("class", "fui-chart-plot")]
        ),
        "{page}"
    );
    assert!(
        page.contains("trend-plot") && page.contains("data-props"),
        "the island is told the plot it shows and hides series in: {page}"
    );
    assert!(
        has(
            &page,
            "svg",
            &[("aria-labelledby", "trend-title trend-desc")]
        ),
        "{page}"
    );
    assert!(
        page.contains("<table"),
        "the figures table reads alone: {page}"
    );
}

/// The picker over two synthetic value sets, holding `value`.
fn picker_holding(value: &str) -> String {
    let value = value.to_owned();
    html(move || {
        picker_field(picker::Picker {
            id: String::from("run-value-set"),
            name: String::from("url"),
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
        chosen.first().and_then(|option| option.get("title")),
        "{listed}"
    );
    assert_ne!(
        Some(&String::from("another")),
        chosen.first().and_then(|option| option.get("value")),
        "a listed choice never takes the other choice's value: {listed}"
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
fn the_surfaces_render_and_none_writes_a_style_attribute() {
    let page = html(|| {
        view! {
            {lockup("/", mark(Glyph::LAYERS), "FerroUI", "Gallery")}
            {stat_card(Stat { glyph: Glyph::USERS, value: String::from("12"), label: "Members".into(), href: Some(String::from("/members")) })}
            {definitions(vec![("Version", "0.1.0".into_any())])}
            {binder("Views", vec![Tab::new("a", "/a", "A"), Tab::new("b", "/b", "B")], "b", ().into_any())}
            {empty_state(Glyph::INBOX, "Nothing here", "No file yet.", ())}
            {skeleton(3, "Loading the files.")}
            {link_button(String::from("/new"), Variant::Primary, Some(Glyph::PLUS), "New")}
            {action_button(Action::new(Some(Glyph::REFRESH_CW), "Reload", Act::Dialog(String::from("delete"))), Variant::Secondary)}
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
        page.contains(
            r#"<div class="fui-tabbed"><div role="tablist" aria-label="Answer" class="fui-tabs">"#
        ),
        "the tabs are joined to their panels as binder tabs: {page}"
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
                flash=vec![
                    Flash {
                        tone: Tone::Ok,
                        title: String::from("Saved <now>."),
                        line: Some(String::from("3 files compiled.")),
                        action: Some(ToastLink {
                            label: String::from("Open History"),
                            href: String::from("/history"),
                        }),
                    },
                    Flash { tone: Tone::Bad, title: String::from("Refused."), line: None, action: None },
                ]
            />
        }
        .into_any()
    });
    assert!(
        page.contains(r#"role="status" aria-live="polite" aria-label="Notices""#),
        "{page}"
    );
    assert!(
        page.contains("<strong>Saved &lt;now&gt;.</strong><span>3 files compiled.</span>"),
        "the title over its line, escaped: {page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[
                ("href", "/history"),
                ("class", "fui-button fui-button-secondary fui-button-small")
            ]
        ),
        "the action is a small link button under the text: {page}"
    );
    // WAI-ARIA 1.2, the alert role: the bad tone interrupts, the others wait.
    let toasts: Vec<_> = tags(&page, "div")
        .into_iter()
        .filter(|div| {
            div.get("class")
                .is_some_and(|class| class.starts_with("fui-toast "))
        })
        .collect();
    assert_eq!(2, toasts.len(), "{page}");
    assert_eq!(None, toasts[0].get("role"), "an ok toast is polite: {page}");
    assert_eq!(
        Some(&String::from("alert")),
        toasts[1].get("role"),
        "{page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[("aria-label", "Dismiss"), ("hidden", "")]
        ),
        "Dismiss is named, and absent until the page hydrates: {page}"
    );
}

#[test]
fn binder_tabs_join_their_panel_and_mark_the_current_page_with_its_count_and_draft() {
    let page = page_html(|| {
        binder(
            "Sections",
            vec![
                Tab::new("nodes", "/nodes", "Nodes").count(12, "12 nodes"),
                Tab::new("consent", "/consent", "Consent <draft>").draft(),
                Tab::new("keyboard", "/keyboard", "Keyboard"),
            ],
            "nodes",
            view! { <p>"The panel"</p> }.into_any(),
        )
    });
    assert!(page.starts_with(r#"<div class="fui-tabbed">"#), "{page}");
    assert!(
        has(
            &page,
            "nav",
            &[("aria-label", "Sections"), ("class", "fui-tabs")]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[
                ("href", "/nodes"),
                ("aria-current", "page"),
                ("data-tab", "nodes")
            ]
        ),
        "{page}"
    );
    assert!(
        page.contains(
            r#"<span aria-hidden="true">12</span><span class="fui-sr-only">12 nodes</span>"#
        ),
        "the count carries its noun: {page}"
    );
    assert!(
        page.contains(r#"<span class="fui-tab-draft">draft</span>"#),
        "{page}"
    );
    assert!(
        page.contains("Consent &lt;draft&gt;"),
        "the label is escaped: {page}"
    );
    assert!(has(&page, "a", &[("data-tab", "keyboard")]), "{page}");
    assert!(
        page.contains(r#"<div class="fui-tab-panel"><p>The panel</p></div>"#),
        "{page}"
    );
    assert!(
        !page.contains("fui-tabs-select"),
        "five tabs or fewer keep the row: {page}"
    );
}

#[test]
fn past_five_binder_tabs_a_phone_select_waits_disabled_for_the_webassembly() {
    let page = page_html(|| {
        let tabs = (1..=7)
            .map(|index| {
                let tab = Tab::new(
                    format!("t{index}"),
                    format!("/t{index}"),
                    format!("Tab {index}"),
                );
                if index == 2 { tab.draft() } else { tab }
            })
            .chain([Tab::new("keyboard", "/keyboard", "Keyboard")])
            .collect();
        binder("Sections", tabs, "t3", ().into_any())
    });
    assert!(
        page.contains(r#"class="fui-tabbed fui-tabbed-many""#),
        "{page}"
    );
    assert!(
        has(
            &page,
            "select",
            &[("id", "fui-tabs-select"), ("disabled", "")]
        ),
        "no-WebAssembly floor: the row of links serves until the select works: {page}"
    );
    assert!(
        has(
            &page,
            "form",
            &[("class", "fui-tabs-select"), ("method", "get")]
        ),
        "the select sits in a GET form: {page}"
    );
    assert!(
        has(&page, "button", &[("type", "submit")]) && page.contains(">Go</button>"),
        "a visible Go button leads to the tab chosen, never the select's change: {page}"
    );
    assert!(
        has(&page, "option", &[("value", "/t3"), ("selected", "")]),
        "{page}"
    );
    assert!(
        page.contains("Tab 2 (draft)"),
        "a draft is named in the select: {page}"
    );
    assert!(
        !has(&page, "option", &[("value", "/keyboard")]),
        "a phone leaves Keyboard out: {page}"
    );
    assert_eq!(
        8,
        tags(&page, "a").len(),
        "every tab is still a link: {page}"
    );
}

#[test]
fn the_popover_form_is_a_native_popover_inside_its_anchor() {
    let page = html(|| {
        popover_form(PopoverForm {
            id: "save",
            variant: Variant::Secondary,
            glyph: Glyph::SAVE,
            label: "Save <it>".into(),
            title: "Save to the CDR".into(),
            body: view! { <form method="post"><button type="submit">"Save"</button></form> }
                .into_any(),
        })
    });
    assert!(page.starts_with(r#"<span class="fui-anchor">"#), "{page}");
    assert!(
        has(
            &page,
            "button",
            &[("popovertarget", "save"), ("aria-controls", "save")]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "div",
            &[
                ("id", "save"),
                ("popover", "auto"),
                ("role", "dialog"),
                ("aria-labelledby", "save-title")
            ]
        ),
        "{page}"
    );
    assert!(
        page.contains(r#"<p id="save-title" class="fui-popover-title">Save to the CDR</p>"#),
        "{page}"
    );
    assert!(page.contains("Save &lt;it&gt;"), "{page}");
}

#[test]
fn a_phrase_fills_its_open_placeables_and_leaves_the_rest() {
    let phrase = Phrase::new(format!(
        "{} of {} rows",
        Phrase::open_mark("first"),
        Phrase::open_mark("total")
    ));
    assert_eq!(
        "1 of 312 rows",
        phrase.fill(&[("first", "1"), ("total", "312")])
    );
    assert_eq!(
        Phrase::new("No rows").fill(&[("first", "1")]),
        "No rows",
        "a phrase with nothing open is its words"
    );
}

/// The picker over choices that include one named like the old reserved
/// value and one named like the other choice, holding `value`.
fn picker_named_like_another(value: &str) -> String {
    let value = value.to_owned();
    html(move || {
        picker_field(picker::Picker {
            id: String::from("run-value-set"),
            name: String::from("url"),
            label: "Value set".into(),
            choose_label: "Choose a value set".into(),
            help: None,
            choices: Signal::stored(vec![
                (
                    String::from("\u{FFFD}another"),
                    String::from("Old sentinel"),
                ),
                (String::from("another"), String::from("Named another")),
            ]),
            value: RwSignal::new(value),
            empty_word: "Choose one".into(),
            other_word: "Another".into(),
        })
    })
}

#[test]
fn a_choice_named_like_the_other_choice_is_chosen_as_itself() {
    for name in ["\u{FFFD}another", "another"] {
        let page = picker_named_like_another(name);
        let chosen: Vec<_> = tags(&page, "option")
            .into_iter()
            .filter(|option| option.contains_key("selected"))
            .collect();
        assert_eq!(1, chosen.len(), "{name:?}: {page}");
        assert_eq!(
            Some(&name.to_owned()),
            chosen.first().and_then(|option| option.get("title")),
            "the listed choice is chosen, not the other one: {page}"
        );
        let other = tags(&page, "option").pop().expect("the other choice");
        assert!(
            other.contains_key("data-other") && other.get("value").is_some_and(String::is_empty),
            "another is marked, holding no value a choice could take: {page}"
        );
    }
}

/// A file's More menu: two plain items, a disabled one with its reason, and
/// the destructive delete a confirmation follows.
fn file_menu() -> Menu {
    Menu::new(
        "file-more",
        "More actions for the file",
        vec![
            Item::new(Action::new(
                Some(Glyph::COPY),
                "Copy path",
                Act::Link(String::from("/files/a?copy")),
            )),
            Item::new(Action::new(
                Some(Glyph::LIST_CHECKS),
                "Check against the set",
                Act::Post {
                    action: String::from("/check"),
                    fields: vec![(String::from("path"), String::from("a <b>.yml"))],
                },
            ))
            .note("Compiles the file; writes nothing"),
            Item::new(Action::new(
                Some(Glyph::DOWNLOAD),
                "Download",
                Act::Link(String::from("/files/a/raw")),
            ))
            .disabled("The file is provisioned & read-only"),
        ],
    )
    .destructive(vec![Item::new(Action::new(
        Some(Glyph::TRASH),
        "Delete the file",
        Act::Dialog(String::from("delete-file")),
    ))])
}

/// The action group of an Edit link and the file's More menu, rendered.
fn file_group() -> String {
    page_html(|| {
        action_group(
            ActionGroup::primary(Action::new(
                Some(Glyph::PENCIL),
                "Edit",
                Act::Link(String::from("/files/a/edit")),
            ))
            .more(file_menu()),
        )
    })
}

#[test]
fn the_action_group_draws_one_primary_then_more_with_the_destructive_item_last() {
    let page = file_group();
    assert!(
        has(
            &page,
            "a",
            &[
                ("href", "/files/a/edit"),
                ("class", "fui-button fui-button-primary")
            ]
        ),
        "the primary is a link: {page}"
    );
    assert_eq!(
        1,
        page.matches("fui-button-primary").count(),
        "one primary action: {page}"
    );
    assert!(
        !page.contains("fui-button-danger") && !page.contains("fui-button-destructive"),
        "no danger button beside the primary: {page}"
    );
    assert!(
        has(
            &page,
            "details",
            &[("class", "fui-anchor"), ("name", "fui-menu")]
        ),
        "More is a details the browser opens with no script: {page}"
    );
    assert!(
        has(
            &page,
            "summary",
            &[
                ("aria-haspopup", "menu"),
                ("aria-controls", "file-more"),
                ("aria-label", "More actions for the file")
            ]
        ),
        "{page}"
    );
    assert!(
        !page.contains("aria-expanded"),
        "the server leaves the state to the summary until the island runs: {page}"
    );
    let separator = page.find(r#"role="separator""#).expect("a separator");
    let delete = page
        .find("Delete the file\u{2026}")
        .expect("the delete, with an ellipsis");
    let download = page.find("Download").expect("the download");
    assert!(
        download < separator && separator < delete,
        "the destructive item is last: {page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[
                ("role", "menuitem"),
                ("popovertarget", "delete-file"),
                ("class", "fui-menu-item fui-menu-danger")
            ]
        ),
        "the delete opens its confirmation with no script: {page}"
    );
}

#[test]
fn every_item_of_more_works_with_no_webassembly_and_a_disabled_one_says_why() {
    let page = file_group();
    assert!(
        has(&page, "form", &[("method", "post"), ("action", "/check")]),
        "an item posts a plain form: {page}"
    );
    assert!(
        has(
            &page,
            "input",
            &[
                ("type", "hidden"),
                ("name", "path"),
                ("value", "a &lt;b&gt;.yml")
            ]
        ),
        "its fields are escaped: {page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[("role", "menuitem"), ("aria-disabled", "true")]
        ),
        "{page}"
    );
    assert!(
        page.contains(
            r#"<span class="fui-menu-item-desc">The file is provisioned &amp; read-only</span>"#
        ),
        "a disabled item says its reason as a line of its own: {page}"
    );
    assert!(
        !tags(&page, "button")
            .iter()
            .any(|button| button.get("aria-disabled").is_some() && button.contains_key("title")),
        "the reason is never a tooltip alone: {page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[("class", "fui-menu-cancel"), ("hidden", "")]
        ),
        "Cancel waits for the island: {page}"
    );
    assert!(
        page.contains("fui-menu-sheet"),
        "a sheet below 40 rem: {page}"
    );
    assert!(!page.contains("style="), "{page}");
}

#[test]
fn a_split_primary_carries_its_variants_and_more_alone_has_no_primary() {
    let split = page_html(|| {
        action_group(
            ActionGroup::primary(Action::new(
                Some(Glyph::SAVE),
                "Save",
                Act::Submit(String::from("editor")),
            ))
            .variants(Menu::new(
                "save-variants",
                "Ways to save",
                vec![Item::new(Action::new(
                    None,
                    "Save as a draft",
                    Act::Submit(String::from("editor-draft")),
                ))],
            )),
        )
    });
    assert!(split.contains("fui-split-button"), "{split}");
    assert!(
        has(&split, "button", &[("type", "submit"), ("form", "editor")]),
        "{split}"
    );
    assert!(
        has(
            &split,
            "summary",
            &[
                ("class", "fui-button fui-button-secondary fui-split-toggle"),
                ("aria-label", "Save: variants")
            ]
        ),
        "{split}"
    );
    let alone = page_html(|| action_group(ActionGroup::more_only(file_menu())));
    assert!(!alone.contains("fui-button-primary"), "{alone}");
    assert!(alone.contains("fui-more"), "{alone}");
    let none = page_html(|| action_group(ActionGroup::none()));
    assert!(!none.contains("<div"), "{none}");
    let empty = page_html(|| action_group(ActionGroup::more_only(Menu::new("x", "X", Vec::new()))));
    assert!(
        !empty.contains("<div"),
        "a More with no item is left out: {empty}"
    );
}

#[test]
fn a_row_menu_is_a_named_icon_button() {
    let page = page_html(|| row_menu(file_menu()));
    assert!(
        has(
            &page,
            "summary",
            &[
                ("class", "fui-icon-button"),
                ("aria-label", "More actions for the file")
            ]
        ),
        "{page}"
    );
}

#[test]
fn the_select_button_shows_its_choice_and_each_option_works_with_no_webassembly() {
    let page = page_html(|| {
        listbox(Listbox {
            id: String::from("lang-list"),
            label: String::from("Language"),
            value: String::from("English"),
            choices: vec![
                Choice {
                    label: String::from("English"),
                    note: None,
                    lang: Some(String::from("en")),
                    selected: true,
                    pick: Pick::Post {
                        action: String::from("/locale"),
                        fields: vec![(String::from("locale"), String::from("en"))],
                    },
                },
                Choice {
                    label: String::from("1.0.0"),
                    note: Some(String::from("Older")),
                    lang: None,
                    selected: false,
                    pick: Pick::Link(String::from("?version=1.0.0")),
                },
            ],
        })
    });
    assert!(
        has(&page, "details", &[("class", "fui-select-anchor")]),
        "{page}"
    );
    assert!(
        has(
            &page,
            "summary",
            &[
                ("class", "fui-select fui-select-button"),
                ("aria-haspopup", "listbox"),
                ("aria-label", "Language: English")
            ]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "div",
            &[
                ("role", "listbox"),
                ("id", "lang-list"),
                ("aria-label", "Language")
            ]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[
                ("type", "submit"),
                ("role", "option"),
                ("aria-selected", "true"),
                ("lang", "en")
            ]
        ),
        "the choice in force posts and is selected: {page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[
                ("role", "option"),
                ("href", "?version=1.0.0"),
                ("aria-selected", "false")
            ]
        ),
        "a choice is a link: {page}"
    );
    assert!(
        page.contains(r#"<span class="fui-listbox-note">Older</span>"#),
        "{page}"
    );
}

#[test]
fn the_combobox_and_picker_islands_draw_their_native_floor_on_the_server() {
    let combo = html(|| {
        view! {
            <Combobox
                id=String::from("choose-system")
                name=String::from("system")
                label=String::from("Code system")
                placeholder=String::from("Choose one")
                options=vec![ComboOption::new("loinc", "LOINC <lab>")]
                selected=Some(String::from("loinc"))
                described_by=None
                words=ComboboxWords { no_match: String::from("No match") }
            />
        }
        .into_any()
    });
    assert!(combo.contains("<leptos-island"), "{combo}");
    assert!(
        has(
            &combo,
            "select",
            &[("id", "choose-system"), ("name", "system")]
        ),
        "a native select posts with no WebAssembly: {combo}"
    );
    assert!(
        has(&combo, "option", &[("value", "loinc"), ("selected", "")]),
        "{combo}"
    );
    assert!(combo.contains("LOINC &lt;lab&gt;"), "{combo}");
    let picker = html(|| {
        view! {
            <PickerField
                id=String::from("run-value-set")
                name=String::from("url")
                label=String::from("Value set")
                choose_label=String::from("Choose a value set")
                help=Some(String::from("The set to expand."))
                choices=vec![(String::from("http://example.org/vs/a"), String::from("Synthetic A"))]
                value=String::from("http://example.org/vs/mine")
                empty_word=String::from("Choose one")
                other_word=String::from("Another")
            />
        }
        .into_any()
    });
    assert!(picker.contains("<leptos-island"), "{picker}");
    assert!(
        has(
            &picker,
            "input",
            &[("name", "url"), ("value", "http://example.org/vs/mine")]
        ),
        "the input posts the value with no WebAssembly: {picker}"
    );
    assert!(
        picker.contains(r#"<p id="run-value-set-help" class="fui-help">The set to expand.</p>"#),
        "{picker}"
    );
}
