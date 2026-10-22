// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The operation parts rendered to HTML: the load report (B4), the
//! transform pair with its options row, Run and the request (B5), the OMOP
//! row preview (B6) and the HL7 v2 viewer (B8).

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::form::{Field, INPUT};
use ferrohealth_ui::view::hl7v2::{Hl7Field, Hl7Segment, hl7v2_raw, hl7v2_segments};
use ferrohealth_ui::view::load_report::{Arrival, Count, LoadReport, Outcome, load_report};
use ferrohealth_ui::view::row_preview::{RowField, RowTable, RowValue, row_preview};
use ferrohealth_ui::view::transform::{
    Answer, AnswerState, Density, Request, Run, RunState, TransformPane, field_checks, request,
    run_button, transform_options, transform_pair,
};
use leptos::prelude::*;

use crate::markup::{has, tags};
use crate::render::{html, page_html};

/// A load report of `outcome` arriving as `arrival`, with two counts and an
/// action when `action`.
fn report(outcome: Outcome, arrival: Arrival, action: bool) -> String {
    html(move || {
        load_report(LoadReport {
            outcome,
            arrival,
            title: view! { "Loaded " <strong>"12 mappings"</strong> }.into_any(),
            counts: vec![
                Count {
                    value: String::from("0"),
                    noun: String::from("errors"),
                },
                Count {
                    value: String::from("3"),
                    noun: String::from("warnings <b>"),
                },
            ],
            action: action.then(|| view! { <a href="#issues">"Show"</a> }.into_any()),
        })
    })
}

#[test]
fn a_load_report_draws_its_tone_glyph_title_counts_and_one_action() {
    for (outcome, tone) in [
        (Outcome::Ok, "fui-tone-ok"),
        (Outcome::Warn, "fui-tone-warn"),
        (Outcome::Bad, "fui-tone-bad"),
    ] {
        let line = report(outcome, Arrival::WithPage, true);
        let class = format!("fui-load-report {tone}");
        assert!(has(&line, "div", &[("class", &class)]), "{line}");
        assert_eq!(1, tags(&line, "svg").len(), "one glyph: {line}");
        assert!(line.contains("<strong>12 mappings</strong>"), "{line}");
        assert!(has(&line, "ul", &[("class", "fui-load-counts")]), "{line}");
        assert!(line.contains("<strong>3</strong>"), "{line}");
        assert!(line.contains("warnings &lt;b&gt;"), "escaped: {line}");
        assert!(has(&line, "div", &[("class", "fui-actions")]), "{line}");
        assert!(!line.contains("style="), "{line}");
    }
    let bare = html(|| {
        load_report(LoadReport {
            outcome: Outcome::Ok,
            arrival: Arrival::WithPage,
            title: "Found".into_any(),
            counts: Vec::new(),
            action: None,
        })
    });
    assert!(!bare.contains("fui-load-counts"), "{bare}");
    assert!(!bare.contains("fui-actions"), "{bare}");
}

#[test]
fn a_load_report_is_a_status_message_only_after_an_action() {
    // NOTE: WAI-ARIA 1.2, the `status` role: an advisory message that appears
    // after the reader acts; a page that loads with it reads it in place.
    let after = report(Outcome::Bad, Arrival::AfterAction, false);
    assert!(has(&after, "div", &[("role", "status")]), "{after}");
    let loaded = report(Outcome::Bad, Arrival::WithPage, false);
    assert!(!loaded.contains("role="), "{loaded}");
}

/// Run in `state`.
fn run(state: RunState) -> String {
    html(move || {
        run_button(Run {
            id: String::from("pg-run"),
            label: String::from("Convert to FHIR"),
            busy: String::from("Converting"),
            glyph: Glyph::PLAY,
            shortcut: Some(String::from("Ctrl Enter")),
            operation: Some(String::from("Runs $tofhir")),
            state,
        })
    })
}

#[test]
fn run_posts_the_form_says_its_keys_waits_while_running_and_says_why_it_is_off() {
    let ready = run(RunState::Ready);
    assert!(
        has(
            &ready,
            "button",
            &[
                ("id", "pg-run"),
                ("type", "submit"),
                ("aria-keyshortcuts", "Control+Enter"),
                ("title", "Runs $tofhir"),
            ]
        ),
        "{ready}"
    );
    assert_eq!(2, tags(&ready, "kbd").len(), "{ready}");
    assert!(
        has(&ready, "div", &[("class", "fui-transform-run")]),
        "{ready}"
    );
    let running = run(RunState::Running);
    assert!(
        has(
            &running,
            "button",
            &[("aria-busy", "true"), ("disabled", ""), ("type", "button")]
        ),
        "{running}"
    );
    assert!(running.contains("Converting"), "{running}");
    assert!(!running.contains("<kbd"), "{running}");
    let off = run(RunState::Off(String::from("Conversions are off")));
    assert!(
        has(
            &off,
            "button",
            &[("disabled", ""), ("aria-describedby", "pg-run-why")]
        ),
        "{off}"
    );
    assert!(
        has(
            &off,
            "span",
            &[("id", "pg-run-why"), ("class", "fui-sr-only")]
        ),
        "{off}"
    );
}

#[test]
fn the_options_row_marks_a_dense_row_an_id_field_and_a_check_group() {
    let row = |density| {
        html(move || {
            let id = Field::new("pg-uid", "Version uid")
                .holds_id()
                .view(view! { <input id="pg-uid" name="uid" class=INPUT /> });
            let checks = field_checks(view! { <input id="pg-active" type="checkbox" /> });
            transform_options(density, view! { {id} {checks} }, view! { <span></span> })
        })
    };
    let plain = row(Density::Plain);
    assert!(
        has(&plain, "div", &[("class", "fui-transform-options")]),
        "{plain}"
    );
    assert!(
        has(&plain, "div", &[("class", "fui-field is-id")]),
        "{plain}"
    );
    assert!(
        has(&plain, "div", &[("class", "fui-field fui-field-checks")]),
        "{plain}"
    );
    let dense = row(Density::Dense);
    assert!(
        has(
            &dense,
            "div",
            &[("class", "fui-transform-options is-dense")]
        ),
        "{dense}"
    );
    let ordinary = html(|| Field::new("pg-x", "X").view(view! { <input id="pg-x" /> }));
    assert!(
        has(&ordinary, "div", &[("class", "fui-field")]),
        "{ordinary}"
    );
}

/// A pair whose answer pane holds `answer`, its state `state`.
fn pair(answer: Answer, state: Option<AnswerState>) -> String {
    html(move || {
        transform_pair(
            TransformPane {
                title: String::from("Composition"),
                tags: vec![String::from("FLAT")],
                state: None,
                actions: None,
                body: view! { <textarea id="pg-in" name="in"></textarea> }.into_any(),
            },
            TransformPane {
                title: String::from("Bundle"),
                tags: vec![String::from("FHIR R4")],
                state,
                actions: Some(view! { <button type="button">"Copy"</button> }.into_any()),
                body: answer.view(),
            },
        )
    })
}

#[test]
fn the_pair_draws_two_panes_with_their_heads_and_the_answer_in_each_state() {
    let waiting = pair(
        Answer::Waiting(String::from("Convert to see the Bundle.")),
        None,
    );
    assert!(
        has(&waiting, "div", &[("class", "fui-transform")]),
        "{waiting}"
    );
    assert_eq!(
        2,
        tags(&waiting, "section").len(),
        "an input and an answer pane: {waiting}"
    );
    assert_eq!(2, tags(&waiting, "h3").len(), "{waiting}");
    assert_eq!(
        2,
        tags(&waiting, "div")
            .iter()
            .filter(|a| a.get("class").map(String::as_str) == Some("fui-transform-head"))
            .count(),
        "{waiting}"
    );
    assert!(
        has(&waiting, "p", &[("class", "fui-transform-empty")]),
        "{waiting}"
    );
    assert!(!waiting.contains("fui-transform-status"), "{waiting}");
    let running = pair(Answer::Running(String::from("Converting to FHIR")), None);
    assert!(
        has(&running, "span", &[("role", "status")]),
        "a run in progress is announced: {running}"
    );
    let answered = pair(
        Answer::Answered(view! { <div class="fui-code-panel"></div> }.into_any()),
        Some(AnswerState {
            outcome: Outcome::Ok,
            word: String::from("Converted"),
            detail: Some(String::from("84 ms")),
        }),
    );
    assert!(
        has(&answered, "span", &[("class", "fui-transform-status")]),
        "{answered}"
    );
    assert!(
        has(&answered, "span", &[("class", "fui-state fui-tone-ok")]),
        "{answered}"
    );
    assert!(
        has(
            &answered,
            "span",
            &[("class", "fui-dot fui-tone-ok"), ("aria-hidden", "true")]
        ),
        "the word carries the state, the dot its shape: {answered}"
    );
    let refused = pair(
        Answer::Refused(view! { <div role="alert" class="fui-alert"></div> }.into_any()),
        Some(AnswerState {
            outcome: Outcome::Bad,
            word: String::from("Refused"),
            detail: None,
        }),
    );
    assert!(
        refused.contains(r#"<div class="fui-transform-body"><div role="alert""#),
        "the refusal sits in the answer pane: {refused}"
    );
    assert!(!refused.contains("style="), "{refused}");
}

#[test]
fn a_request_breaks_after_slashes_and_ampersands_and_carries_copy_and_the_raw_answer() {
    let line = page_html(|| {
        request(Request {
            text: String::from("GET /fhir/CodeSystem/$lookup?system=a&code=<1>"),
            copy: Some(String::from("rq-copy")),
            raw: Some((
                String::from("The Parameters"),
                view! { <pre>"{}"</pre> }.into_any(),
            )),
        })
    });
    assert!(has(&line, "div", &[("class", "fui-request")]), "{line}");
    assert!(line.contains("GET /<wbr>fhir/<wbr>"), "{line}");
    assert!(line.contains("system=a&amp;<wbr>code=&lt;1&gt;"), "{line}");
    assert!(has(&line, "button", &[("id", "rq-copy")]), "{line}");
    assert!(
        has(&line, "details", &[("class", "fui-disclosure")]),
        "{line}"
    );
}

#[test]
fn the_row_preview_counts_each_tables_rows_and_marks_an_unresolved_concept_in_words() {
    let field = |column: &str, value: RowValue| RowField {
        column: column.to_owned(),
        value,
    };
    let preview = html(move || {
        row_preview(vec![
            RowTable {
                name: String::from("MEASUREMENT"),
                detail: Some(String::from("synthetic_measurement")),
                rows: vec![
                    vec![field(
                        "value_as_number",
                        RowValue::Value(String::from("72.4")),
                    )],
                    vec![field(
                        "value_as_number",
                        RowValue::Value(String::from("<73>")),
                    )],
                ],
            },
            RowTable {
                name: String::from("OBSERVATION"),
                detail: None,
                rows: vec![vec![field("observation_concept_id", RowValue::ConceptZero)]],
            },
        ])
    });
    assert!(
        has(&preview, "div", &[("class", "fui-rowgraph")]),
        "{preview}"
    );
    assert_eq!(2, tags(&preview, "h4").len(), "{preview}");
    assert_eq!(3, tags(&preview, "dl").len(), "one list per row: {preview}");
    assert!(
        preview.contains("2 rows · synthetic_measurement"),
        "{preview}"
    );
    assert!(preview.contains("1 row<"), "{preview}");
    assert!(preview.contains("&lt;73&gt;"), "escaped: {preview}");
    assert!(
        has(&preview, "span", &[("class", "fui-concept-zero")]),
        "{preview}"
    );
    assert!(
        preview.contains("0, no matching concept"),
        "WCAG 2.2 SC 1.4.1: the mark says it in words: {preview}"
    );
}

#[test]
fn the_raw_message_tints_segment_ids_and_separators_one_segment_per_line() {
    let raw = html(|| {
        hl7v2_raw(
            "The message",
            "MSH|^~\\&|GAL\rPID|1||<SYN>^^^GAL\r\nOBX|1\r",
        )
    });
    assert!(
        has(
            &raw,
            "pre",
            &[
                ("class", "fui-hl7-raw"),
                ("role", "region"),
                ("aria-label", "The message"),
                ("tabindex", "0")
            ]
        ),
        "a focusable region the keyboard scrolls: {raw}"
    );
    assert!(
        raw.contains(r#"<span class="fui-hl7-segid">PID</span>"#),
        "{raw}"
    );
    assert!(
        raw.contains(r#"<span class="fui-hl7-sep">^^^</span>"#),
        "{raw}"
    );
    assert!(raw.contains("&lt;SYN&gt;"), "escaped: {raw}");
    assert_eq!(
        2,
        raw.matches('\n').count(),
        "three lines, no last break: {raw}"
    );
    assert!(!raw.contains('\r'), "{raw}");
}

#[test]
fn the_parsed_message_draws_a_block_per_segment_with_position_name_and_value() {
    let parsed = html(|| {
        hl7v2_segments(
            "The message as read",
            vec![Hl7Segment {
                id: String::from("PID"),
                name: String::from("Patient identification"),
                fields: vec![Hl7Field {
                    position: String::from("PID-3"),
                    name: String::from("Patient identifier list"),
                    value: String::from("SYN-0042^^^GAL&<x>"),
                }],
            }],
        )
    });
    assert!(
        has(
            &parsed,
            "ul",
            &[
                ("class", "fui-hl7-segs"),
                ("aria-label", "The message as read")
            ]
        ),
        "{parsed}"
    );
    assert_eq!(1, tags(&parsed, "li").len(), "{parsed}");
    assert!(parsed.contains("<dt>PID-3</dt>"), "{parsed}");
    assert!(
        has(&parsed, "dd", &[("class", "fui-hl7-name")]),
        "the name a phone drops: {parsed}"
    );
    assert!(parsed.contains("SYN-0042^^^GAL&amp;&lt;x&gt;"), "{parsed}");
}
