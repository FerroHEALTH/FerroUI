// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code viewer (B1) and the document pane at real size: a finding on
//! no line of the file, a long file with many findings, and a large
//! document, which the page carries no more often than its views draw it.

use std::time::{Duration, Instant};

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::view::code::{Code, CodeHead, Gravity, LineDiagnostic, code_card};
use ferrohealth_ui::view::document::{Document, document_pane};
use ferrohealth_ui::widget::document::DocumentView;

use crate::markup::has;
use crate::render::page_html;

/// A text of `count` lines, each naming its number.
fn numbered(count: u32) -> String {
    (1..=count)
        .map(|line| format!("key_{line}: value {line}\n"))
        .collect::<Vec<_>>()
        .concat()
}

/// A finding of `gravity` on `line`, its code `code`.
fn found(line: u32, gravity: Gravity, code: &str) -> LineDiagnostic {
    LineDiagnostic {
        line,
        gravity,
        code: code.to_owned(),
        message: format!("{code} says why."),
    }
}

/// The code of `text` with `diagnostics`.
fn code(text: String, diagnostics: Vec<LineDiagnostic>) -> Code {
    Code {
        label: String::from("The file"),
        language: Language::Yaml,
        text,
        ids: Some(String::from("f-L")),
        current: None,
        diagnostics,
    }
}

/// `code` in a card headed `f.yml`, with Copy when `copy` names its id.
fn card(code: &Code, copy: Option<&str>) -> String {
    page_html(|| {
        code_card(
            CodeHead {
                title: String::from("f.yml"),
                copy: copy.map(str::to_owned),
                ..CodeHead::default()
            },
            code,
        )
    })
}

#[test]
fn a_finding_on_line_zero_sits_above_the_first_line_and_one_past_the_end_under_the_last() {
    let drawn = card(
        &code(
            String::from("a: 1\nb: 2\n"),
            vec![
                found(9, Gravity::Error, "E9"),
                found(0, Gravity::Error, "E0"),
                found(2, Gravity::Warning, "W2"),
            ],
        ),
        None,
    );
    let at = |part: &str| {
        drawn
            .find(part)
            .unwrap_or_else(|| panic!("{part} is drawn: {drawn}"))
    };
    let first = at(r#"id="f-L1""#);
    let last = at(r#"id="f-L2""#);
    assert!(at(">E0<") < first, "line 0 sits above line 1: {drawn}");
    assert!(
        last < at(">W2<") && at(">W2<") < at(">E9<"),
        "a finding past the end sits under the last line, after its own: {drawn}"
    );
    assert_eq!(
        3,
        drawn.matches("fui-code-diag fui-tone").count(),
        "no finding is left out: {drawn}"
    );
    assert!(
        has(
            &drawn,
            "span",
            &[("class", "fui-code-line fui-mark-warning"), ("id", "f-L2")]
        ),
        "a finding off the file marks no line: {drawn}"
    );
}

/// The quickest of three draws of a viewer of `lines` lines with a
/// finding on every line.
fn viewer_time(lines: u32) -> Duration {
    let code = code(
        numbered(lines),
        (1..=lines)
            .map(|line| found(line, Gravity::Warning, "W"))
            .collect(),
    );
    (0..3)
        .map(|_| {
            let start = Instant::now();
            let drawn = card(&code, None);
            let took = start.elapsed();
            assert!(drawn.contains("fui-code-diag"), "the findings are drawn");
            took
        })
        .min()
        .unwrap_or_default()
}

#[test]
fn a_long_file_with_a_finding_on_every_line_draws_in_linear_time() {
    // Sixteen times the lines and findings: linear work takes about
    // sixteen times as long, and a scan of every finding for every line 256.
    let small = viewer_time(1_000);
    let large = viewer_time(16_000);
    assert!(
        large < small * 64,
        "16000 lines took {large:?} against {small:?} for 1000"
    );
}

#[test]
fn copy_reads_the_viewers_lines_and_carries_no_copy_of_the_text() {
    let text = format!("{}marker_unique: 1\n", numbered(20));
    let drawn = card(
        &code(text, vec![found(3, Gravity::Error, "E3")]),
        Some("file-copy"),
    );
    assert_eq!(
        1,
        drawn.matches("marker_unique").count(),
        "the text is drawn once, in the lines, never as island data: {drawn}"
    );
    assert!(
        has(
            &drawn,
            "div",
            &[("id", "file-copy-text"), ("role", "region")]
        ),
        "Copy reads the box it names: {drawn}"
    );
}

/// The document pane `doc` over `text`.
fn pane(text: &str) -> String {
    page_html(|| {
        document_pane(Document {
            id: String::from("doc"),
            label: String::from("Bundle"),
            formats: vec![
                (String::from("?format=json"), String::from("JSON")),
                (String::from("?format=xml"), String::from("XML")),
            ],
            format: 0,
            language: Language::Json,
            text: text.to_owned(),
            opening: DocumentView::Highlighted,
            rendered: None,
        })
    })
}

#[test]
fn a_document_pane_carries_its_text_once_per_view_and_copies_from_the_raw_view() {
    let text: String = (0..2_000)
        .map(|entry| format!("{{\"k{entry}\": \"value {entry}\"}},\n"))
        .collect::<Vec<_>>()
        .concat();
    let drawn = pane(&format!("{text}{{\"marker_unique\": 1}}\n"));
    assert_eq!(
        2,
        drawn.matches("marker_unique").count(),
        "the highlighted and the raw view draw the text; Copy carries none: {}",
        drawn.len()
    );
    assert!(
        has(&drawn, "pre", &[("id", "doc-text")]),
        "the raw view's text is what Copy reads"
    );
    assert!(
        drawn.contains("doc-text"),
        "the copy island names the raw view's text"
    );
}

#[test]
fn a_raw_view_keeps_the_line_break_a_document_opens_with() {
    let drawn = pane("\n{\"a\": 1}");
    assert!(
        drawn.contains("<pre id=\"doc-text\">\n\n{"),
        "the parser drops one line break after <pre>, so the server writes two: {drawn}"
    );
}
