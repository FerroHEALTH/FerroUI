// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Every view rendered to HTML on the host, as a console's handler draws
//! it: what each must say, what each must never carry, and the escaping of
//! what it is given.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::{Locale, Phrase, Word};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::{ActionGroup, action_button, action_group};
use ferrohealth_ui::view::action_menu::{
    Act, Action, Choice, Item, Listbox, Menu, Pick, labelled_menu, listbox, more_menu, row_menu,
};
use ferrohealth_ui::view::button::{Variant, link_button};
use ferrohealth_ui::view::chart::{ActivityChart, LineChart, Series, activity_chart};
use ferrohealth_ui::view::chart::{ChartWords, line_chart};
use ferrohealth_ui::view::code::{
    Code, CodeHead, Digest, Gravity, LineDiagnostic, code_card, code_section, code_viewer,
};
use ferrohealth_ui::view::combobox::{ComboOption, Combobox, ComboboxWords, combobox};
use ferrohealth_ui::view::copy::{CopySource, CopyWords, copy_from};
use ferrohealth_ui::view::dialog::{
    ConfirmForm, Confirmation, Explanation, confirm_dialog, danger_form_dialog, info_dialog,
};
use ferrohealth_ui::view::document::DocumentView;
use ferrohealth_ui::view::document::{Document, document_pane};
use ferrohealth_ui::view::editor::{Editor, code_editor};
use ferrohealth_ui::view::empty::{empty_state, skeleton};
use ferrohealth_ui::view::form::{
    ERROR, Field, INPUT, SELECT, action_field, described_by, error_id, field, help_id, keyed_check,
    keyed_check_field, keyed_field,
};
use ferrohealth_ui::view::help::{HelpWords, explained_check_field, help_switch};
use ferrohealth_ui::view::pick_or_type::{PickOrType, Picker, pick_or_type};
use ferrohealth_ui::view::picker::{self, picker_field};
use ferrohealth_ui::view::popover::{PopoverForm, popover_form};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, refusal};
use ferrohealth_ui::view::shell::{Footer, footer, lockup, mark};
use ferrohealth_ui::view::side_panel::{Opener, SidePanel, SidePanelWords, side_panel};
use ferrohealth_ui::view::surface::{
    Header, SectionHead, Stat, add_row, definitions, page_header, page_heading, panel_section,
    panel_sections, section_intro, source_split, stat_card,
};
use ferrohealth_ui::view::tabs::{Tab, binder};
use ferrohealth_ui::view::tabs::{keyboard_tabs, tab_panels};
use ferrohealth_ui::view::template::list::HeadingCount;
use ferrohealth_ui::view::toast::{Flash, ToastLink, toast_region};

use crate::markup::{has, tags};

/// The locale every render test draws in.
pub(crate) fn en() -> Locale {
    Locale::english()
}

/// The markup `view` renders, as a string a test reads.
pub(crate) fn html(view: Html) -> String {
    view.into()
}

#[test]
fn a_refusal_lists_its_issues_with_their_text_and_counts_the_silent_ones() {
    let shown = html({
        refusal(
            &en(),
            &Refusal {
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
            },
        )
    });
    assert!(
        shown.contains("422: the bridge refused the request."),
        "{shown}"
    );
    assert!(
        shown.contains("no program maps &#60;the&#62; template"),
        "{shown}"
    );
    assert!(
        shown.contains("1 information issue with no text left out."),
        "{shown}"
    );
    assert!(!shown.contains("informational"), "{shown}");
    assert!(
        shown.contains("&#60;b&#62;MSG&#60;/b&#62;"),
        "a detail code is text: {shown}"
    );
    assert!(
        shown.contains("GET /fhir?q=&#60;script&#62;alert(1)&#60;/script&#62;"),
        "the request is text: {shown}"
    );
    assert!(
        shown.contains("&#60;img src=x onerror=alert(1)&#62;"),
        "the evidence is text: {shown}"
    );
    assert!(
        !shown.contains("<script") && !shown.contains("<img"),
        "{shown}"
    );
}

#[test]
fn a_refusal_with_no_answer_shows_its_summary_alone_and_says_no_answer_to_assistive_technology() {
    // WAI-ARIA 1.2 `alert`: the live title still carries "No answer", hidden from sight.
    let shown = html({
        refusal(
            &en(),
            &Refusal {
                summary: String::from("The identifier index did not answer."),
                ..Refusal::default()
            },
        )
    });
    assert!(
        !shown.contains("No answer: The identifier index"),
        "the visible title is the summary alone: {shown}"
    );
    let title = shown
        .split(r#"role="alert""#)
        .nth(1)
        .and_then(|rest| rest.split("</p>").next())
        .unwrap_or_default();
    assert!(
        title.contains(r#"<span class="fui-sr-only">No answer:</span>"#),
        "the live title leads with a hidden No answer: {shown}"
    );
    assert!(
        title.contains("The identifier index did not answer."),
        "{shown}"
    );
    let answered = html({
        refusal(
            &en(),
            &Refusal {
                status: Some(503),
                summary: String::from("The identifier index is down."),
                ..Refusal::default()
            },
        )
    });
    assert!(
        !answered.contains("No answer"),
        "a status says what came: {answered}"
    );
}

#[test]
fn a_page_header_carries_a_subtitle_with_inline_markup_in_the_subtitle_style() {
    let shown = html({
        page_heading(
            &en(),
            Header {
                title: Word::from("vital-signs.context.yml"),
                subtitle: Some(Word::from("A text that the view replaces.")),
                subtitle_view: Some(Html::raw(String::from(
                    r#"Context of <strong>Observation</strong> on <a href="/templates/synthetic.vital_signs.v1"><code>synthetic.vital_signs.v1</code></a>, saved by the console."#,
                ))),
                ..Header::default()
            },
            ActionGroup::none(),
        )
    });
    assert!(
        shown.contains(
            r#"<p class="fui-page-subtitle">Context of <strong>Observation</strong> on <a href="/templates/synthetic.vital_signs.v1"><code>synthetic.vital_signs.v1</code></a>, saved by the console.</p>"#
        ),
        "{shown}"
    );
    assert!(
        !shown.contains("A text that the view replaces."),
        "the view takes the text's place: {shown}"
    );
    let plain = html({
        page_heading(
            &en(),
            Header {
                title: Word::from("Files"),
                subtitle: Some(Word::from("Every mapping file the bridge serves.")),
                ..Header::default()
            },
            ActionGroup::none(),
        )
    });
    assert!(
        plain.contains(r#"<p class="fui-page-subtitle">Every mapping file the bridge serves.</p>"#),
        "the text form stays: {plain}"
    );
}

#[test]
fn each_row_gives_its_confirmation_and_popover_form_an_id_of_its_own() {
    // HTML Living Standard: an id is unique in its tree, so each row's trigger opens its own dialog.
    let shown = html({
        ["node-b", "node-c"]
            .into_iter()
            .map(|member| {
                let confirm = confirm_dialog(
                    &en(),
                    Confirmation {
                        id: format!("remove-{member}").into(),
                        title: Word::from("Remove it?"),
                        message: Word::from("It leaves the federation."),
                    },
                    Html::default(),
                );
                let popover = popover_form(PopoverForm {
                    id: format!("rename-{member}").into(),
                    variant: Variant::Secondary,
                    glyph: Glyph::PENCIL,
                    label: "Rename".into(),
                    title: "Rename the member".into(),
                    body: Html::default(),
                });
                confirm.then(popover)
            })
            .collect::<Html>()
    });
    for member in ["node-b", "node-c"] {
        let remove = format!("remove-{member}");
        let rename = format!("rename-{member}");
        assert!(has(&shown, "dialog", &[("id", &remove)]), "{shown}");
        assert!(
            has(
                &shown,
                "button",
                &[("command", "close"), ("commandfor", &remove)]
            ),
            "{shown}"
        );
        assert!(has(&shown, "dialog", &[("id", &rename)]), "{shown}");
        assert!(
            has(
                &shown,
                "button",
                &[("command", "show-modal"), ("commandfor", &rename)]
            ),
            "{shown}"
        );
    }
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
    let viewer = html(code_viewer(
        &en(),
        None,
        &code("a: \"</pre>\"\nb: 2\nc: 3\n"),
    ));
    assert!(viewer.contains("&lt;/pre&gt;"), "{viewer}");
    assert!(
        !viewer.contains("<unknown>") && !viewer.contains("FC-<1>"),
        "{viewer}"
    );
    assert!(viewer.contains("An &#60;unknown&#62; key."), "{viewer}");
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
                (
                    "aria-label",
                    "The &#60;file&#62;, one numbered line per line"
                ),
            ]
        ),
        "{viewer}"
    );
    assert!(!viewer.contains("fui-codeview-head"), "no head: {viewer}");
    assert!(!viewer.contains("style="), "{viewer}");
}

#[test]
fn a_code_card_names_its_path_in_its_head_with_the_tags_the_count_and_the_digest() {
    let card = html({
        code_card(
            &en(),
            CodeHead {
                title: String::from("fhirconnect/a <b>.yml"),
                tags: vec![String::from("FHIRconnect")],
                digest: Some(Digest {
                    text: String::from("SHA-256 5712…"),
                    title: String::from("SHA-256 of the bytes"),
                }),
                copy: Some(String::from("file-copy")),
                actions: None,
            },
            &code("a: 1\nb: 2\nc: 3"),
        )
    });
    assert!(
        has(
            &card,
            "button",
            &[("id", "file-copy"), ("type", "button"), ("hidden", "")]
        ),
        "Copy is the copy island in the head, hidden until it works: {card}"
    );
    assert!(
        has(&card, "h2", &[("class", "fui-codeview-path")])
            && card.contains("fhirconnect/<wbr>a &#60;b&#62;.yml"),
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
    let section = html({
        code_section(
            &en(),
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
fn an_informational_dialog_explains_and_offers_only_close() {
    let dialog = html({
        info_dialog(
            &en(),
            Explanation {
                id: "how-config".into(),
                title: Word::from("How <configuration> works"),
            },
            Html::raw(String::from("<p>Read at start.</p>")),
        )
    });
    assert!(
        dialog.starts_with(
            r#"<dialog id="how-config" closedby="any" aria-labelledby="how-config-title""#
        ),
        "{dialog}"
    );
    assert!(
        has(&dialog, "h2", &[("id", "how-config-title")]),
        "{dialog}"
    );
    assert!(
        dialog.contains("How &#60;configuration&#62; works"),
        "{dialog}"
    );
    assert!(dialog.contains("<p>Read at start.</p>"), "{dialog}");
    assert_eq!(
        1,
        tags(&dialog, "button").len(),
        "Close is the one action: {dialog}"
    );
    assert!(
        has(
            &dialog,
            "button",
            &[
                ("command", "close"),
                ("commandfor", "how-config"),
                ("autofocus", "")
            ]
        ),
        "{dialog}"
    );
    assert!(dialog.contains("Close"), "{dialog}");
    assert!(!dialog.contains("style="), "{dialog}");
}

#[test]
fn a_heading_counts_the_items_it_shows_of_a_larger_set() {
    let section = html({
        panel_section(
            &en(),
            Some(SectionHead {
                title: String::from("Code systems"),
                count: Some(HeadingCount::Shown {
                    shown: 6,
                    total: 15_000,
                }),
                actions: None,
            }),
            Html::raw(String::from("<p>Rows</p>")),
        )
    });
    let count = section
        .split_once(r#"<span class="fui-list-count">"#)
        .and_then(|(_, rest)| rest.split_once("</span>"))
        .map(|(count, _)| count.replace("<!---->", ""))
        .unwrap_or_default();
    assert!(count.starts_with("· "), "after a middle dot: {section}");
    assert!(
        count.ends_with("6 of 15,000"),
        "shown of total, each number grouped: {section}"
    );
}

#[test]
fn panel_sections_draw_a_heading_row_with_its_count_and_no_card_of_their_own() {
    let panel = html({
        panel_sections(Html::join([
            panel_section(
                &en(),
                Some(SectionHead {
                    title: String::from("Models <x>"),
                    count: Some(HeadingCount::All(3)),
                    actions: Some(Html::raw(String::from(r#"<a href="/add">Add</a>"#))),
                }),
                Html::join([
                    section_intro("One <sentence>."),
                    add_row(Html::raw(String::from(
                        r#"<button type="button">Add</button>"#,
                    ))),
                ]),
            ),
            panel_section(&en(), None, Html::raw(String::from("<p>Prose</p>"))),
        ]))
    });
    assert!(panel.starts_with(r#"<div class="fui-panel">"#), "{panel}");
    assert!(
        !panel.contains("fui-card"),
        "never a card in the panel: {panel}"
    );
    assert_eq!(2, tags(&panel, "section").len(), "{panel}");
    assert!(panel.contains("Models &#60;x&#62;"), "{panel}");
    assert!(
        panel.contains(r#"<span class="fui-list-count">· 3</span>"#),
        "{panel}"
    );
    assert!(has(&panel, "div", &[("class", "fui-actions")]), "{panel}");
    assert!(panel.contains("One &#60;sentence&#62;."), "{panel}");
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
    let split = html({
        source_split(
            code_viewer(
                &en(),
                Some(CodeHead {
                    title: String::from("a.yml"),
                    ..CodeHead::default()
                }),
                &code("a: 1"),
            ),
            panel_section(&en(), None, Html::text("Diagnostics")),
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
    let plain = html(page_header(&en(), "Dashboard", None, ActionGroup::none()));
    assert!(plain.contains("<h1>Dashboard</h1>"), "{plain}");
    let mono = html({
        page_heading(
            &en(),
            Header {
                title: Word::from("vitals.v1"),
                mono: true,
                ..Header::default()
            },
            ActionGroup::none(),
        )
    });
    assert!(
        mono.contains(r#"<h1 class="fui-page-title-mono">vitals.v1</h1>"#),
        "{mono}"
    );
}

/// A document pane over `text`, opening on `opening`, with a rendered view
/// when `rendered`.
fn pane(opening: DocumentView, rendered: bool) -> String {
    html({
        document_pane(
            &en(),
            Document {
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
                rendered: rendered.then(|| Html::raw(String::from("<p>A reading</p>"))),
            },
        )
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
    assert!(raw.contains("&#60;/pre&#62;"), "{raw}");
    assert!(!raw.contains("</pre>\""), "{raw}");
    assert!(
        has(
            &raw,
            "a",
            &[("href", "?format=json"), ("aria-current", "true")]
        ),
        "{raw}"
    );
    let buttons = tags(&raw, "button");
    assert!(
        buttons.iter().all(|button| button.contains_key("hidden")),
        "no control that works only with script shows before it works: {raw}"
    );
    assert!(
        has(&raw, "button", &[("id", "doc-copy"), ("hidden", "")]),
        "Copy copies the document in the format in force: {raw}"
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
        x_labels: Vec::new(),
        y_range: None,
    }
}

#[test]
fn the_figures_table_names_each_point_by_its_own_label() {
    let mut chart = trend("Readings");
    chart.series[0].points.push((2.0, 125.0));
    chart.x_ticks = vec![(0.0, String::from("Mon")), (2.0, String::from("Wed"))];
    chart.x_labels = vec![
        (1.0, String::from("Tue 09:00")),
        (2.0, String::from("Wed 09:00")),
    ];
    let page = html(line_chart(&chart, ChartWords::of(&en())));
    let rows: Vec<&str> = page
        .split(r#"<th scope="row">"#)
        .skip(1)
        .filter_map(|row| row.split("</th>").next())
        .collect();
    assert_eq!(
        vec!["Mon", "Tue 09:00", "Wed 09:00"],
        rows,
        "a point's own label, then a tick's: {page}"
    );
    assert!(
        !page.contains(">Tue 09:00</text>") && page.contains("Wed"),
        "the axis keeps its ticks: {page}"
    );
}

#[test]
fn the_legend_is_a_static_key_before_the_page_hydrates() {
    let page = html(line_chart(&trend("Readings"), ChartWords::of(&en())));
    let keys = tags(&page, "span")
        .into_iter()
        .filter(|span| span.get("class").map(String::as_str) == Some("fui-legend-key"))
        .filter_map(|span| span.get("data-series").cloned())
        .collect::<Vec<_>>();
    assert_eq!(
        vec![
            String::from("systolic"),
            String::from("diastolic &#60;b&#62;")
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
    let page = html(line_chart(&trend("Readings <mmHg>"), ChartWords::of(&en())));
    assert!(page.contains("fui-chart-line fui-series-0"), "{page}");
    assert!(page.contains("fui-chart-line fui-series-1"), "{page}");
    assert!(
        tags(&page, "g")
            .iter()
            .all(|group| !group.contains_key("hidden")),
        "every series shows as the page arrives: {page}"
    );
    assert!(page.contains("Readings &#60;mmHg&#62;"), "{page}");
    assert!(
        has(
            &page,
            "div",
            &[("id", "trend-plot"), ("class", "fui-chart-plot")]
        ),
        "{page}"
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

#[test]
fn an_activity_chart_labels_every_day_its_ticks_name() {
    let chart = ActivityChart {
        name: "writes",
        label: String::from("Contributions per day"),
        summary: String::from("40 contributions over 30 days."),
        points: (0_u32..30).map(|day| (day, day % 4)).collect(),
        ticks: vec![
            (0, String::from("22 Sep")),
            (9, String::from("1 Oct")),
            (19, String::from("11 Oct")),
            (29, String::from("21 Oct")),
        ],
    };
    let page = html(activity_chart(&chart));
    let labels: Vec<&str> = page
        .split("<text class=\"fui-chart-label")
        .skip(1)
        .filter_map(|text| {
            text.split_once('>')
                .and_then(|(_, rest)| rest.split("</text>").next())
        })
        .collect();
    assert_eq!(
        vec!["22 Sep", "1 Oct", "11 Oct", "21 Oct"],
        labels,
        "the axis names the four days the console gives, as the FerroEHR mockups draw them: {page}"
    );
}

/// The picker over two synthetic value sets, holding `value`.
fn picker_holding(value: &str) -> String {
    let value = value.to_owned();
    html({
        picker_field(picker::Picker {
            id: String::from("run-value-set"),
            name: String::from("url"),
            label: "Value set".into(),
            choose_label: "Choose a value set".into(),
            help: None,
            choices: (vec![
                (
                    String::from("http://example.org/vs/a"),
                    String::from("Synthetic A"),
                ),
                (
                    String::from("http://example.org/vs/b"),
                    String::from("Synthetic B"),
                ),
            ]),
            value: (value),
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
fn the_surfaces_render_and_none_writes_a_style_attribute() {
    let page = html({
        Html::join([
            lockup("/", mark(Glyph::LAYERS), "FerroUI", "Gallery"),
            stat_card(Stat {
                glyph: Glyph::USERS,
                value: String::from("12"),
                label: "Members".into(),
                href: Some(String::from("/members")),
            }),
            definitions(vec![("Version", Html::text("0.1.0"))]),
            binder(
                &en(),
                "Views",
                vec![Tab::new("a", "/a", "A"), Tab::new("b", "/b", "B")],
                "b",
                Html::default(),
            ),
            empty_state(
                Glyph::INBOX,
                "Nothing here",
                "No file yet.",
                Html::default(),
            ),
            skeleton(3, "Loading the files."),
            link_button(
                String::from("/new"),
                Variant::Primary,
                Some(Glyph::PLUS),
                "New",
            ),
            action_button(
                &en(),
                Action::new(
                    Some(Glyph::REFRESH_CW),
                    "Reload",
                    Act::Dialog(String::from("delete")),
                ),
                Variant::Secondary,
            ),
            confirm_dialog(
                &en(),
                Confirmation {
                    id: "delete".into(),
                    title: "Delete it?".into(),
                    message: "It goes for good.".into(),
                },
                Html::default(),
            ),
            footer(
                &en(),
                Footer {
                    product: "FerroUI Gallery".into(),
                    version: "0.1.0".into(),
                    about: "/about".into(),
                },
            ),
        ])
    });
    for expected in [
        "fui-lockup-accent",
        "fui-stat-value",
        "fui-facts",
        r#"aria-current="page""#,
        "fui-empty",
        r#"aria-busy="true""#,
        r#"command="show-modal" commandfor="delete""#,
        r#"<dialog id="delete" closedby="any""#,
        "Cadasto B.V.",
    ] {
        assert!(page.contains(expected), "{expected}: {page}");
    }
    assert!(!page.contains("style="), "{page}");
}

#[test]
fn a_start_grid_writes_the_start_class_and_the_grid_does_not() {
    let start = html(ferrohealth_ui::view::surface::grid_start(Html::raw(
        String::from("<p>a</p>"),
    )));
    assert!(
        start.contains(r#"class="fui-grid fui-grid-start""#),
        "{start}"
    );
    let plain = html(ferrohealth_ui::view::surface::grid(Html::raw(
        String::from("<p>a</p>"),
    )));
    assert!(plain.contains(r#"class="fui-grid""#), "{plain}");
}

#[test]
fn a_timestamp_is_a_time_element_with_the_reading_form_and_the_raw_title() {
    let raw = "2026-10-18T23:18:37.039437+00:00";
    let drawn = html(ferrohealth_ui::view::time::timestamp(&en(), raw));
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
fn a_relative_time_carries_its_full_timestamp_as_its_title() {
    // NOTE: COMPONENTS.md family rule 11: relative in lists, the full timestamp in the tooltip.
    let drawn = html({
        ferrohealth_ui::view::time::relative(
            &en(),
            "2026-10-21T09:12:11+00:00",
            "2026-10-21T09:42:11Z",
        )
    });
    for expected in [
        r#"class="fui-time""#,
        r#"datetime="2026-10-21T09:12:11Z""#,
        r#"title="2026-10-21 09:12:11 UTC""#,
        ">30 min ago</time>",
    ] {
        assert!(drawn.contains(expected), "{expected}: {drawn}");
    }
    let verbatim = html(ferrohealth_ui::view::time::relative(
        &en(),
        "<never>",
        "2026-10-21T09:42:11Z",
    ));
    assert!(
        verbatim.contains("&#60;never&#62;") && !verbatim.contains("<time"),
        "{verbatim}"
    );
}

#[test]
fn a_timestamp_that_is_no_date_time_is_drawn_verbatim_and_escaped() {
    let drawn = html(ferrohealth_ui::view::time::timestamp(&en(), "<not a date>"));
    assert!(drawn.contains(r#"<span class="fui-time">"#), "{drawn}");
    assert!(drawn.contains("&#60;not a date&#62;"), "{drawn}");
    assert!(!drawn.contains("<time"), "{drawn}");
    assert!(!drawn.contains("datetime="), "{drawn}");
}

#[test]
fn a_stat_label_carries_its_whole_text_as_its_title() {
    let label = "Code systems with a long synthetic name";
    let drawn = html({
        stat_card(Stat {
            glyph: Glyph::USERS,
            value: String::from("12"),
            label: label.into(),
            href: None,
        })
    });
    assert!(
        drawn.contains(&format!(r#"<span title="{label}">{label}</span>"#)),
        "{drawn}"
    );
}

#[test]
fn a_field_with_an_id_made_per_row_names_its_own_control_and_help() {
    let rows = ["include-0", "exclude-1"];
    let fields = html({
        rows.iter()
            .map(|key| {
                let id = format!("compose-search-{key}");
                let help = help_id(&id);
                keyed_field(
                    id.clone(),
                    "Find a code",
                    Some("Type two letters or more.".to_owned()),
                    Html::raw(format!(
                        r#"<input id="{id}" name="q" class="{INPUT}" aria-describedby="{help}">"#
                    )),
                )
            })
            .collect::<Html>()
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
    let checks = html({
        Html::join([
            keyed_check(
                "keep-2",
                "Keep",
                Html::raw(String::from(r#"<input type="checkbox" id="keep-2">"#)),
            ),
            keyed_check_field(
                String::from("inactive-2"),
                "Inactive codes",
                "Codes a system retired.",
                Html::raw(String::from(
                    r#"<input type="checkbox" id="inactive-2" aria-describedby="inactive-2-help">"#,
                )),
            ),
        ])
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
    let fields = html({
        Html::join([
            field(
                "static-label",
                "Base URL",
                None,
                Html::raw(format!(r#"<input id="static-label" class="{INPUT}">"#)),
            ),
            keyed_field(
                format!("base-{member}"),
                format!("Base URL of {member}"),
                None,
                Html::raw(format!(r#"<input id="base-node-a" class="{INPUT}">"#)),
            ),
        ])
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
    let row = html({
        Field::new("value-3", "Value")
            .label_hidden()
            .view(Html::raw(format!(
                r#"<input id="value-3" class="{INPUT}">"#
            )))
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
    let drawn = html({
        let described = described_by(id, true, true);
        Field::new(id, "Base URL")
            .help("Where the gateway reaches the member.")
            .after(Html::raw(format!(
                r#"<p id="{}" class="{}" aria-live="polite">The address must be absolute.</p>"#,
                error_id(id),
                ERROR
            )))
            .view(Html::raw(format!(
                r#"<input id="{}" class="{}" aria-describedby="{}">"#,
                id,
                INPUT,
                described.unwrap_or_default()
            )))
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
    let drawn = html({
        action_field(
            Html::raw(format!(
                r#"<button type="button" id="add-value" aria-describedby="{}">Add value</button>"#,
                error_id("add-value")
            )),
            Html::raw(format!(
                r#"<p id="{}" class="{}" hidden></p>"#,
                error_id("add-value"),
                ERROR
            )),
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
    html({
        let select = Html::raw(format!(
            r#"<select id="version-pick" class="{}" aria-describedby="{}"><option value="">The one this server resolves to</option><option value="other">Another version</option></select>"#,
            SELECT,
            help_id("version")
        ));
        let picker = if labelled {
            Picker::Labelled(Html::raw(format!(
                r#"<div class="fui-field"><label for="version-pick" class="fui-label">Version</label>{select}</div>"#
            )))
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
            typed: Html::raw(format!(
                r#"<input id="version" name="version" class="{}" aria-describedby="{}">"#,
                INPUT,
                help_id("version")
            )),
            typing: (typing),
            after: Some(Html::raw(format!(
                r#"<p id="{}" class="fui-help">Its version.</p>"#,
                help_id("version")
            ))),
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
fn an_explained_checkbox_keeps_its_help_named_under_the_forms_switch() {
    let drawn = html({
        explained_check_field(
            "run-inactive",
            "Include inactive concepts",
            "An inactive concept is listed and marked.",
            Html::raw(format!(
                r#"<input type="checkbox" id="run-inactive" aria-describedby="{}">"#,
                help_id("run-inactive")
            )),
        )
    });
    assert!(has(&drawn, "label", &[("for", "run-inactive")]), "{drawn}");
    // WCAG 2.2 technique C7: the stylesheet hides it from sight, never from the tree.
    assert!(
        has(
            &drawn,
            "span",
            &[
                ("id", "run-inactive-help"),
                ("class", "fui-help fui-explain")
            ]
        ),
        "{drawn}"
    );
    assert!(has(&drawn, "div", &[("class", "fui-check")]), "{drawn}");
}

#[test]
fn the_help_switch_is_a_native_checkbox_that_submits_nothing() {
    let drawn = html(help_switch("run-help", HelpWords::of(&en())));
    assert!(
        has(
            &drawn,
            "input",
            &[
                ("type", "checkbox"),
                ("role", "switch"),
                ("id", "run-help"),
                ("class", "fui-switch")
            ]
        ),
        "{drawn}"
    );
    assert!(
        !drawn.contains("name="),
        "the switch is no form field: {drawn}"
    );
    assert!(
        has(&drawn, "label", &[("class", "fui-help-switch")]),
        "the words name the switch: {drawn}"
    );
    assert!(drawn.contains("Explain these parameters"), "{drawn}");
}

#[test]
fn a_keyed_field_row_carries_its_id_and_its_columns() {
    use ferrohealth_ui::view::layout::{FieldRow, field_row, keyed_field_row};
    let keyed = html(keyed_field_row(
        "values-2-row",
        FieldRow::WithActions,
        Html::text("x"),
    ));
    assert!(
        keyed.contains(
            r#"<div id="values-2-row" class="fui-field-row fui-field-row-actions">x</div>"#
        ),
        "{keyed}"
    );
    let plain = html(field_row(FieldRow::Halves, Html::text("y")));
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
    let page = html({
        code_editor(&Editor {
            id: "draft".into(),
            language: Language::Yaml,
            text: String::from("a: </textarea><b>&"),
            marks: Vec::new(),
            editable: true,
        })
    });
    assert!(
        page.contains(">a: &lt;/textarea&gt;&lt;b&gt;&amp;</textarea>"),
        "the text area holds its text, escaped: {page}"
    );
}

#[test]
fn the_keyboard_tabs_show_the_first_panel_and_hide_the_others_before_any_webassembly() {
    // WAI-ARIA Authoring Practices 1.2, the tabs pattern.
    let page = html({
        keyboard_tabs(
            "Answer",
            "answer",
            &[String::from("Table"), String::from("Raw")],
            tab_panels(
                "answer",
                vec![
                    Html::raw(String::from("<p>Rows</p>")),
                    Html::raw(String::from("<p>Text</p>")),
                ],
            ),
        )
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
            r#"<div class="fui-tabbed"><div role="tablist" aria-label="Answer" data-fui="tabs" class="fui-tabs">"#
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
    let page = html({
        toast_region(
            &en(),
            vec![
                Flash {
                    tone: Tone::Ok,
                    title: String::from("Saved <now>."),
                    line: Some(String::from("3 files compiled.")),
                    action: Some(ToastLink {
                        label: String::from("Open History"),
                        href: String::from("/history"),
                    }),
                },
                Flash {
                    tone: Tone::Bad,
                    title: String::from("Refused."),
                    line: None,
                    action: None,
                },
            ],
        )
    });
    assert!(
        page.contains(r#"role="status" aria-live="polite" aria-label="Notices""#),
        "{page}"
    );
    assert!(
        page.contains("<strong>Saved &#60;now&#62;.</strong><span>3 files compiled.</span>"),
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
    // Shell edge audit, row 3: with no WebAssembly a flash toast still closes.
    assert_eq!(
        2,
        page.matches(r#"<details open class="fui-toast-close"><summary aria-label="Dismiss" title="Dismiss" class="fui-icon-button">"#).count(),
        "each flash toast closes as a native disclosure: {page}"
    );
}

#[test]
fn the_toast_region_shows_a_few_toasts_and_keeps_each_refusal_and_each_with_an_action() {
    // Shell edge audit, row 3: four flash toasts covered the decision bar.
    let flash = |title: &str, action: bool| Flash {
        tone: if title == "Zeroth." {
            Tone::Bad
        } else {
            Tone::Ok
        },
        title: title.to_owned(),
        line: None,
        action: action.then(|| ToastLink {
            label: String::from("Open"),
            href: String::from("/open"),
        }),
    };
    let page = html({
        toast_region(
            &en(),
            vec![
                flash("Zeroth.", false),
                flash("First.", false),
                flash("Second.", true),
                flash("Third.", false),
                flash("Fourth.", false),
                flash("Fifth.", false),
            ],
        )
    });
    assert_eq!(
        ferrohealth_ui::view::toast::SHOWN_MAX,
        page.matches("<strong>").count(),
        "{page}"
    );
    for gone in ["First.", "Third.", "Fourth."] {
        assert!(
            !page.contains(&format!("<strong>{gone}</strong>")),
            "the oldest without an action leave: {page}"
        );
    }
    for kept in ["Zeroth.", "Second.", "Fifth."] {
        assert!(
            page.contains(&format!("<strong>{kept}</strong>")),
            "{kept} stays: {page}"
        );
    }
}

#[test]
fn binder_tabs_join_their_panel_and_mark_the_current_page_with_its_count_and_draft() {
    let page = html({
        binder(
            &en(),
            "Sections",
            vec![
                Tab::new("nodes", "/nodes", "Nodes").count(12, "12 nodes"),
                Tab::new("consent", "/consent", "Consent <draft>").draft(),
                Tab::new("keyboard", "/keyboard", "Keyboard"),
            ],
            "nodes",
            Html::raw(String::from("<p>The panel</p>")),
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
        page.contains("Consent &#60;draft&#62;"),
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
    let page = html({
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
        binder(&en(), "Sections", tabs, "t3", Html::default())
    });
    assert!(
        page.contains(r#"class="fui-tabbed fui-tabbed-many""#),
        "{page}"
    );
    assert!(
        has(&page, "select", &[("disabled", "")]),
        "no-WebAssembly floor: the row of links serves until the select works: {page}"
    );
    assert!(
        page.contains(
            r#"<label class="fui-tabs-select-field"><span class="fui-sr-only">Sections</span><select"#
        ),
        "the label wraps the select, so its name needs no id: {page}"
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
fn two_phone_tab_selects_on_one_page_repeat_no_id() {
    // NOTE: HTML Living Standard, the id attribute: an id is unique in its tree.
    let page = html({
        let row = |label: &str| {
            let tabs = (1..=7)
                .map(|index| Tab::new(format!("t{index}"), format!("/{label}/t{index}"), "Tab"))
                .collect();
            binder(&en(), label, tabs, "t1", Html::default())
        };
        Html::join([row("Sections"), row("Views")])
    });
    assert_eq!(2, tags(&page, "select").len(), "{page}");
    let mut ids: Vec<&str> = page
        .split(" id=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect();
    let all = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(all, ids.len(), "an id repeats: {page}");
}

#[test]
fn the_popover_form_is_a_native_modal_dialog_inside_its_anchor() {
    // WAI-ARIA Authoring Practices, the dialog (modal) pattern: the page behind is inert and the
    // focus stays inside; HTML Living Standard, the `command` attribute opens it with no script.
    let page = html({
        popover_form(PopoverForm {
            id: "save".into(),
            variant: Variant::Secondary,
            glyph: Glyph::SAVE,
            label: "Save <it>".into(),
            title: "Save to the CDR".into(),
            body: Html::raw(String::from(
                r#"<form method="post"><button type="submit">Save</button></form>"#,
            )),
        })
    });
    assert!(page.starts_with(r#"<span class="fui-anchor">"#), "{page}");
    assert!(
        has(
            &page,
            "button",
            &[
                ("command", "show-modal"),
                ("commandfor", "save"),
                ("aria-controls", "save"),
                ("aria-haspopup", "dialog")
            ]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "dialog",
            &[
                ("id", "save"),
                ("closedby", "any"),
                ("aria-labelledby", "save-title")
            ]
        ),
        "{page}"
    );
    assert!(
        !page.contains("popover="),
        "no popover, which lets the focus out: {page}"
    );
    assert!(
        page.contains(r#"<p id="save-title" class="fui-popover-title">Save to the CDR</p>"#),
        "{page}"
    );
    assert!(page.contains("Save &#60;it&#62;"), "{page}");
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
    html({
        picker_field(picker::Picker {
            id: String::from("run-value-set"),
            name: String::from("url"),
            label: "Value set".into(),
            choose_label: "Choose a value set".into(),
            help: None,
            choices: (vec![
                (
                    String::from("\u{FFFD}another"),
                    String::from("Old sentinel"),
                ),
                (String::from("another"), String::from("Named another")),
            ]),
            value: (value),
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
    html({
        action_group(
            &en(),
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
                ("command", "show-modal"),
                ("commandfor", "delete-file"),
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
                ("value", "a &#60;b&#62;.yml")
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
            r#"<span class="fui-menu-item-desc">The file is provisioned &#38; read-only</span>"#
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
    let split = html({
        action_group(
            &en(),
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
    let alone = html(action_group(&en(), ActionGroup::more_only(file_menu())));
    assert!(!alone.contains("fui-button-primary"), "{alone}");
    assert!(alone.contains("fui-more"), "{alone}");
    let none = html(action_group(&en(), ActionGroup::none()));
    assert!(!none.contains("<div"), "{none}");
    let empty = html(action_group(
        &en(),
        ActionGroup::more_only(Menu::new("x", "X", Vec::new())),
    ));
    assert!(
        !empty.contains("<div"),
        "a More with no item is left out: {empty}"
    );
}

#[test]
fn a_copy_item_waits_hidden_and_its_menu_carries_the_copy_words() {
    // NOTE: leptos-ui.md §15.8: a control only script can work is absent before hydration.
    let menu = Menu::new(
        "ehr-more",
        "More actions for the EHR",
        vec![Item::new(Action::new(
            Some(Glyph::COPY),
            "Copy EHR id",
            Act::Copy(String::from("7d44b88c-4199-4bad-97dc-d78268e01398")),
        ))],
    );
    let page = html(more_menu(&en(), &menu));
    assert!(
        has(
            &page,
            "button",
            &[
                ("type", "button"),
                ("role", "menuitem"),
                ("data-fui-copy", "7d44b88c-4199-4bad-97dc-d78268e01398")
            ]
        ),
        "{page}"
    );
    let item = tags(&page, "button")
        .into_iter()
        .find(|button| button.contains_key("data-fui-copy"))
        .expect("the copy item");
    assert!(item.contains_key("hidden"), "{page}");
    let plain = html(more_menu(&en(), &file_menu()));
    assert!(
        !plain.contains("Copied to the clipboard."),
        "a menu with no copy item carries no copy words: {plain}"
    );
    let button = html({
        action_button(
            &en(),
            Action::new(None, "Copy the id", Act::Copy(String::from("id-1"))),
            Variant::Secondary,
        )
    });
    assert!(button.contains("Copy the id"), "{button}");
}

#[test]
fn a_popover_form_sits_beside_the_primary_before_more() {
    let save = || PopoverForm {
        id: "save".into(),
        variant: Variant::Secondary,
        glyph: Glyph::SAVE,
        label: "Save to the CDR".into(),
        title: "Save to the CDR".into(),
        body: Html::raw(String::from(
            r#"<form method="post"><button type="submit">Save</button></form>"#,
        )),
    };
    let page = html({
        action_group(
            &en(),
            ActionGroup::primary(Action::new(
                Some(Glyph::PLAY),
                "Run",
                Act::Submit(String::from("query")),
            ))
            .popover(save())
            .more(file_menu()),
        )
    });
    let run = page.find(">Run<").expect("the primary");
    let anchor = page
        .find(r#"<span class="fui-anchor">"#)
        .expect("the popover's anchor");
    let more = page.find("fui-more").expect("More");
    assert!(run < anchor && anchor < more, "{page}");
    assert!(
        has(&page, "dialog", &[("id", "save"), ("class", "fui-popover")]),
        "{page}"
    );
    let alone = html(action_group(&en(), ActionGroup::none().popover(save())));
    assert!(
        alone.contains("fui-action-group"),
        "a popover alone draws: {alone}"
    );
}

#[test]
fn a_row_menu_is_a_named_icon_button() {
    let page = html(row_menu(&en(), &file_menu()));
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
    let page = html({
        listbox(
            &en(),
            &Listbox {
                id: String::from("lang-list"),
                label: String::from("Language"),
                value: Word::from("English"),
                choices: vec![
                    Choice {
                        label: Word::from("English"),
                        note: None,
                        lang: Some(String::from("en")),
                        selected: true,
                        pick: Pick::Post {
                            action: String::from("/locale"),
                            fields: vec![(String::from("locale"), String::from("en"))],
                        },
                    },
                    Choice {
                        label: Word::from("1.0.0"),
                        note: Some(Word::from("Older")),
                        lang: None,
                        selected: false,
                        pick: Pick::Link(String::from("?version=1.0.0")),
                    },
                ],
            },
        )
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
fn a_combobox_option_that_asks_for_a_typed_value_is_marked_other() {
    let combo = html(combobox(Combobox {
        id: "pick-canonical",
        name: "picked",
        label: "Code system".into(),
        placeholder: "Choose one".into(),
        options: vec![
            ComboOption::new("http://loinc.org", "LOINC"),
            ComboOption::new("another", "Another canonical").other(),
        ],
        selected: None,
        words: ComboboxWords {
            no_match: Word::from("No match"),
            more: Word::from("Type to narrow"),
        },
    }));
    assert!(
        has(
            &combo,
            "option",
            &[("value", "another"), ("data-other", "")]
        ),
        "{combo}"
    );
    assert!(
        tags(&combo, "option")
            .iter()
            .filter(|option| option.contains_key("data-other"))
            .count()
            == 1,
        "only the other option is marked: {combo}"
    );
}

#[test]
fn the_combobox_and_picker_draw_their_native_floor_on_the_server() {
    let combo = html(combobox(Combobox {
        id: "choose-system",
        name: "system",
        label: "Code system".into(),
        placeholder: "Choose one".into(),
        options: vec![ComboOption::new("loinc", "LOINC <lab>")],
        selected: Some(String::from("loinc")),
        words: ComboboxWords {
            no_match: Word::from("No match"),
            more: Word::from("Type to narrow"),
        },
    }));
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
    assert!(combo.contains("LOINC &#60;lab&#62;"), "{combo}");
    let picker = html(picker_field(picker::Picker {
        id: String::from("run-value-set"),
        name: String::from("url"),
        label: "Value set".into(),
        choose_label: "Choose a value set".into(),
        help: Some("The set to expand.".into()),
        choices: vec![(
            String::from("http://example.org/vs/a"),
            String::from("Synthetic A"),
        )],
        value: String::from("http://example.org/vs/mine"),
        empty_word: "Choose one".into(),
        other_word: "Another".into(),
    }));
    assert!(
        has(
            &picker,
            "input",
            &[("name", "url"), ("value", "http://example.org/vs/mine")]
        ),
        "the input posts the value with no WebAssembly: {picker}"
    );
    // The line follows the switch of a form that has one, and shows in any other.
    assert!(
        picker.contains(
            r#"<p id="run-value-set-help" class="fui-help fui-explain">The set to expand.</p>"#
        ),
        "{picker}"
    );
}

/// The copy button's words, as a test gives them.
fn copy_words() -> CopyWords {
    CopyWords {
        copied: Word::from("Copied"),
        said_copied: Word::from("Copied to the clipboard."),
        said_refused: Word::from("Copy failed."),
        refused: Word::from("The browser refused."),
    }
}

#[test]
fn the_copy_button_waits_hidden_and_leaves_its_text_to_the_page() {
    let copy = html(copy_from(
        "about-copy",
        "Copy <it>",
        CopySource::Text(String::from("{\"name\": \"<x>\"}")),
        copy_words(),
    ));
    assert!(
        has(
            &copy,
            "button",
            &[
                ("id", "about-copy"),
                ("type", "button"),
                ("class", "fui-button fui-button-secondary fui-button-small"),
                ("hidden", "")
            ]
        ),
        "§15.8: with no WebAssembly no Copy shows: {copy}"
    );
    assert!(copy.contains("Copy &#60;it&#62;"), "{copy}");
    assert!(
        !copy.contains("<x>"),
        "the text crosses to the browser escaped: {copy}"
    );
    assert!(
        has(
            &copy,
            "span",
            &[("class", "fui-sr-only"), ("role", "status")]
        ),
        "a live region says what came of a press: {copy}"
    );
}

#[test]
fn the_side_panel_opens_from_a_link_to_its_route_until_it_hydrates() {
    let panel = html(side_panel(SidePanel {
        id: String::from("access"),
        title: String::from("Access <now>"),
        opener: Opener {
            label: String::from("View access"),
            glyph: Some(Glyph::SHIELD_CHECK),
            variant: Variant::Secondary,
            href: String::from("/session/access"),
        },
        body: Html::raw(String::from("<p>Signed in as a synthetic operator.</p>")),
        words: SidePanelWords {
            close: Word::from("Close"),
        },
    }));
    let links = tags(&panel, "a");
    let link = links
        .iter()
        .find(|link| link.get("href").map(String::as_str) == Some("/session/access"))
        .expect("the opener is a link to the content's route");
    assert!(
        !link.contains_key("hidden"),
        "the link shows with no WebAssembly: {panel}"
    );
    assert!(
        has(
            &panel,
            "button",
            &[
                ("type", "button"),
                ("command", "show-modal"),
                ("commandfor", "access"),
                ("aria-controls", "access"),
                ("hidden", "")
            ]
        ),
        "the button that opens it in place waits for the WebAssembly: {panel}"
    );
    assert!(
        has(
            &panel,
            "dialog",
            &[
                ("id", "access"),
                ("closedby", "any"),
                ("aria-labelledby", "access-title"),
                ("class", "fui-side-panel")
            ]
        ),
        "a modal dialog, so the page behind is inert while it shows (WAI-ARIA Authoring Practices, the dialog (modal) pattern): {panel}"
    );
    assert!(
        panel.contains(r#"<h2 id="access-title" class="fui-card-title">Access &#60;now&#62;</h2>"#),
        "{panel}"
    );
    assert!(
        has(
            &panel,
            "button",
            &[
                ("class", "fui-icon-button fui-side-panel-close"),
                ("command", "close"),
                ("commandfor", "access"),
                ("aria-label", "Close")
            ]
        ),
        "{panel}"
    );
    assert!(
        has(
            &panel,
            "div",
            &[("id", "access-body"), ("class", "fui-side-panel-body")]
        ),
        "{panel}"
    );
}

/// A row whose menu's Remove… opens its own danger confirmation, which
/// asks for a required Reason inside the form it posts.
fn removal_row(member: &str) -> Html {
    let id = format!("remove-{member}");
    let menu = Menu::new(
        format!("row-{member}"),
        format!("Actions for {member}"),
        Vec::new(),
    )
    .destructive(vec![Item::new(Action::new(
        Some(Glyph::TRASH),
        "Remove",
        Act::Dialog(id.clone()),
    ))]);
    let dialog = danger_form_dialog(
        &en(),
        Confirmation {
            id: id.into(),
            title: format!("Remove {member}?").into(),
            message: String::from("Its endpoints go with it.").into(),
        },
        ConfirmForm {
            action: format!("/members/{member}/remove"),
            fields: Html::raw(format!(
                r#"<input type="hidden" name="member" value="{member}"><textarea id="reason" name="reason" required></textarea>"#
            )),
        },
        Html::raw(format!(
            r#"<button type="submit" class="{}">Remove</button>"#,
            Variant::Destructive.class()
        )),
    );
    row_menu(&en(), &menu).then(dialog)
}

#[test]
fn a_danger_confirmation_holds_the_consoles_fields_inside_the_form_it_posts() {
    let shown = html(removal_row("node-b"));
    assert!(
        has(
            &shown,
            "button",
            &[
                ("role", "menuitem"),
                ("command", "show-modal"),
                ("commandfor", "remove-node-b"),
                ("aria-haspopup", "dialog")
            ]
        ) && shown.contains("Remove\u{2026}"),
        "the row menu's destructive item opens the dialog with no script: {shown}"
    );
    assert!(
        has(
            &shown,
            "dialog",
            &[
                ("id", "remove-node-b"),
                ("class", "fui-dialog fui-dialog-danger"),
                ("aria-labelledby", "remove-node-b-title")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "form",
            &[
                ("method", "post"),
                ("action", "/members/node-b/remove"),
                ("class", "fui-dialog-form")
            ]
        ),
        "{shown}"
    );
    let dialog = shown
        .split_once("<dialog")
        .map(|(_, rest)| rest)
        .expect("the dialog");
    let at = |needle: &str| dialog.find(needle).unwrap_or(usize::MAX);
    assert!(
        at("<form") < at("<h2")
            && at("Its endpoints go with it.") < at("<textarea")
            && at("<textarea") < at("fui-dialog-actions")
            && at("fui-dialog-actions") < at("</form>"),
        "the question, the sentence, the fields, then the actions, all inside the form: {dialog}"
    );
    assert!(
        has(
            dialog,
            "button",
            &[("type", "button"), ("command", "close")]
        ),
        "Cancel closes and never submits: {dialog}"
    );
    assert!(
        has(dialog, "button", &[("type", "submit")]),
        "the console's confirm submits the form: {dialog}"
    );
    assert!(!shown.contains("style="), "{shown}");
}

#[test]
fn a_danger_confirmation_with_no_fields_draws_no_form_of_its_own() {
    let shown = html({
        ferrohealth_ui::view::dialog::danger_dialog(
            &en(),
            Confirmation {
                id: "delete".into(),
                title: "Delete it?".into(),
                message: "It goes for good.".into(),
            },
            Html::default(),
        )
    });
    assert!(tags(&shown, "form").is_empty(), "{shown}");
}

#[test]
fn a_menu_item_submits_the_consoles_form_to_another_address() {
    let shown = html({
        labelled_menu(
            &en(),
            Glyph::DOWNLOAD,
            "Export".into(),
            &Menu::new(
                "export",
                "Export",
                vec![
                    Item::new(Action::new(
                        None,
                        "CSV",
                        Act::SubmitTo {
                            action: String::from("/query/answer.csv"),
                            form: None,
                        },
                    )),
                    Item::new(Action::new(
                        None,
                        "JSON",
                        Act::SubmitTo {
                            action: String::from("/query/answer.json"),
                            form: Some(String::from("query")),
                        },
                    )),
                ],
            ),
        )
    });
    assert!(
        tags(&shown, "form").is_empty(),
        "a form inside the console's form is dropped by the parser: {shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[
                ("type", "submit"),
                ("formaction", "/query/answer.csv"),
                ("role", "menuitem")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[
                ("type", "submit"),
                ("form", "query"),
                ("formaction", "/query/answer.json")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "summary",
            &[
                ("class", "fui-button fui-button-secondary fui-button-small"),
                ("aria-haspopup", "menu"),
                ("aria-controls", "export"),
                ("aria-label", "Export")
            ]
        ) && shown.contains(">Export<"),
        "a small labelled trigger, named by its words (WCAG 2.2 SC 2.5.3): {shown}"
    );
    let button = html({
        action_button(
            &en(),
            Action::new(
                None,
                "Export CSV",
                Act::SubmitTo {
                    action: String::from("/query/answer.csv"),
                    form: Some(String::from("query")),
                },
            ),
            Variant::Secondary,
        )
    });
    assert!(
        has(
            &button,
            "button",
            &[
                ("type", "submit"),
                ("form", "query"),
                ("formaction", "/query/answer.csv")
            ]
        ),
        "{button}"
    );
}
