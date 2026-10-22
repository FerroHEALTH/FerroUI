// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code editor (B2) and the diagnostics list (B3) rendered as a
//! console's server draws them: the toolbar, the gutter marks and the
//! status line; the rows errors first, grouped by file, with their line
//! links and the current row.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::view::code::Gravity;
use ferrohealth_ui::view::diagnostics::{
    DiagnosticsWords, FileFindings, Finding, Position, diagnostics, diagnostics_by_file,
};
use ferrohealth_ui::widget::editor::{
    Editor, EditorWords, LineMark, MarkKind, Opened, Panel, editor_diagnostics, editor_panel,
    editor_split,
};
use leptos::prelude::*;

use crate::markup::{has, tags};
use crate::render::html;

/// A finding of `gravity` with `code` at `line` and `column`.
fn finding(gravity: Gravity, code: &str, at: Option<(u32, u32)>) -> Finding {
    Finding {
        gravity,
        code: code.to_owned(),
        at: at.map(|(line, column)| Position { line, column }),
        message: format!("{code} says <why> & how."),
        mapping: Some(String::from("synthetic_weight")),
    }
}

/// The findings of a synthetic file `name`, linked to its lines as `lines`.
fn file(name: &str, lines: &str, current: Option<u32>, findings: Vec<Finding>) -> FileFindings {
    FileFindings {
        file: name.to_owned(),
        href: Some(format!("/files/{name}")),
        lines: lines.to_owned(),
        current,
        findings,
    }
}

/// The codes of the rows in `html`, in the order drawn.
fn codes(html: &str) -> Vec<String> {
    html.split("<code class=\"fui-diag-code\">")
        .skip(1)
        .filter_map(|rest| rest.split('<').next())
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_list_draws_errors_first_grouped_by_file_with_counts_and_line_links() {
    let files = vec![
        file(
            "fhirconnect/a.context.yml",
            "/files/a#L",
            None,
            vec![finding(Gravity::Warning, "FC-310", Some((4, 3)))],
        ),
        file(
            "fhirconnect/b.model.yml",
            "/files/b#L",
            Some(13),
            vec![
                finding(Gravity::Warning, "FC-311", Some((2, 1))),
                finding(Gravity::Error, "FC-201", Some((13, 5))),
                finding(Gravity::Error, "FC-100", None),
            ],
        ),
    ];
    let shown = html(|| diagnostics_by_file("Diagnostics, by file", &files));
    assert_eq!(
        vec!["FC-100", "FC-201", "FC-311", "FC-310"],
        codes(&shown),
        "the file with an error comes first, and errors before warnings in it: {shown}"
    );
    assert!(
        shown.contains("2 errors, 1 warning"),
        "a file head counts its findings: {shown}"
    );
    assert!(shown.contains(">1 warning<"), "{shown}");
    assert!(
        has(
            &shown,
            "a",
            &[
                ("href", "/files/fhirconnect/b.model.yml"),
                ("class", "fui-row-link")
            ]
        ),
        "a file's name links to its page: {shown}"
    );
    assert!(
        has(
            &shown,
            "a",
            &[("href", "/files/b#L13"), ("class", "fui-diag-where")]
        ),
        "a line and column link to the line: {shown}"
    );
    assert!(shown.contains("line 13, column 5"), "{shown}");
    assert!(
        shown.contains("the whole document"),
        "a finding with no line says so, with no link: {shown}"
    );
    assert!(shown.contains("Mapping synthetic_weight"), "{shown}");
    assert!(
        shown.contains("FC-201 says &lt;why&gt; &amp; how."),
        "a message is text: {shown}"
    );
    assert!(!shown.contains(" style="), "{shown}");
}

#[test]
fn a_row_says_its_gravity_in_shape_colour_and_word() {
    // Family rule 12: a state is a shape, a colour and a word together.
    let one = file(
        "a.yml",
        "#L",
        None,
        vec![
            finding(Gravity::Error, "E", Some((1, 1))),
            finding(Gravity::Warning, "W", Some((2, 1))),
        ],
    );
    let shown = html(|| diagnostics("Diagnostics", &one));
    assert!(
        has(&shown, "li", &[("class", "fui-diag fui-tone-bad")]),
        "{shown}"
    );
    assert!(
        has(&shown, "li", &[("class", "fui-diag fui-tone-warn")]),
        "{shown}"
    );
    assert!(shown.contains("Error</span>"), "{shown}");
    assert!(shown.contains("Warning</span>"), "{shown}");
    assert_eq!(
        2,
        tags(&shown, "svg").len(),
        "each row draws its glyph: {shown}"
    );
    assert!(
        !shown.contains("fui-diags-file"),
        "a list of one file draws no file head: {shown}"
    );
}

#[test]
fn the_row_of_the_current_line_is_current() {
    let one = file(
        "a.yml",
        "#L",
        Some(4),
        vec![
            finding(Gravity::Warning, "W", Some((4, 1))),
            finding(Gravity::Error, "E", Some((9, 1))),
        ],
    );
    let shown = html(|| diagnostics("Diagnostics", &one));
    assert!(
        has(
            &shown,
            "li",
            &[
                ("class", "fui-diag fui-tone-warn is-current"),
                ("aria-current", "true")
            ]
        ),
        "{shown}"
    );
    assert_eq!(
        1,
        tags(&shown, "li")
            .iter()
            .filter(|row| row.contains_key("aria-current"))
            .count(),
        "one row is current: {shown}"
    );
}

/// The editor over `text` with an error on line 3 and a warning on line 2.
fn editor(text: &str) -> Editor {
    Editor {
        id: "draft",
        language: Language::Yaml,
        text: Signal::stored(text.to_owned()),
        marks: Signal::stored(vec![
            LineMark {
                line: 3,
                kind: MarkKind::Error,
            },
            LineMark {
                line: 2,
                kind: MarkKind::Warning,
            },
        ]),
        editable: Signal::stored(false),
    }
}

/// The code editor over `text`, opened as `opened`, its caret at `at`.
fn panel(text: &str, opened: Opened, at: Position) -> AnyView {
    editor_panel(
        Panel {
            editor: editor(text),
            label: String::from("The draft of a.yml"),
            lead: String::from("YAML · mapping context"),
            facts: vec![String::from("UTF-8 · LF")],
            opened,
            caret: RwSignal::new(at),
            tools: Some(view! { <button type="submit">"Check"</button> }.into_any()),
            end: Some(view! { <span>"console"</span> }.into_any()),
            words: EditorWords::of_page(),
        },
        |_| {},
    )
}

#[test]
fn the_editor_draws_its_toolbar_marks_and_status_line() {
    let text = "a: 1\nb: </textarea>&\nc: 3\n";
    let shown = html(|| {
        panel(
            text,
            Opened::Text(String::from("a: 1\n")),
            Position::default(),
        )
    });
    assert!(
        has(
            &shown,
            "section",
            &[("class", "fui-card fui-panel fui-mapping-editor")]
        ),
        "{shown}"
    );
    assert!(
        shown.contains("YAML · mapping context · 3 lines"),
        "the toolbar says what the text is and its lines: {shown}"
    );
    assert!(shown.contains(">Check</button>"), "the tools: {shown}");
    assert!(
        has(
            &shown,
            "span",
            &[
                ("class", "fui-code-line fui-mark-error"),
                ("id", "draft-L3")
            ]
        ),
        "an error marks its line: {shown}"
    );
    assert!(
        has(
            &shown,
            "span",
            &[
                ("class", "fui-code-line fui-mark-warning"),
                ("id", "draft-L2")
            ]
        ),
        "a warning marks its line: {shown}"
    );
    assert!(shown.contains("Ln 1, Col 1"), "the cursor's place: {shown}");
    assert!(shown.contains("UTF-8 · LF"), "{shown}");
    assert!(
        has(&shown, "span", &[("class", "fui-editor-dirty")])
            && shown.contains("Changed since opened"),
        "a changed draft shows the unsaved dot: {shown}"
    );
    assert!(
        has(
            &shown,
            "span",
            &[("class", "fui-sr-only"), ("role", "status")]
        ),
        "the status line has a polite live region: {shown}"
    );
    assert!(
        has(
            &shown,
            "textarea",
            &[
                ("aria-label", "The draft of a.yml"),
                ("aria-invalid", "true")
            ]
        ),
        "the text area is named, and invalid while an error marks it: {shown}"
    );
    assert!(
        shown.contains(">a: 1\nb: &lt;/textarea&gt;&amp;\nc: 3\n</textarea>"),
        "the text area holds its text, escaped (#205): {shown}"
    );
    assert!(!shown.contains(" style="), "{shown}");
}

#[test]
fn the_status_line_says_unchanged_or_new() {
    let text = "a: 1\n";
    let same = html(|| panel(text, Opened::Text(text.to_owned()), Position::default()));
    assert!(same.contains("Unchanged"), "{same}");
    assert!(!same.contains("fui-editor-dirty"), "{same}");
    let new = html(|| panel("", Opened::New, Position::default()));
    assert!(new.contains("New file"), "{new}");
    assert!(
        has(
            &new,
            "section",
            &[("class", "fui-card fui-panel fui-mapping-editor is-empty")]
        ),
        "an empty editor is short: {new}"
    );
    assert!(new.contains("· 1 line<"), "{new}");
}

#[test]
fn on_a_marked_line_the_line_and_its_row_are_current_and_the_status_says_so() {
    let text = "a: 1\nb: 2\nc: 3\n";
    let caret = Position { line: 3, column: 2 };
    let shown = html(|| panel(text, Opened::New, caret));
    assert!(shown.contains("Ln 3, Col 2"), "{shown}");
    assert!(
        has(
            &shown,
            "span",
            &[
                ("class", "fui-code-line fui-mark-error is-current"),
                ("id", "draft-L3")
            ]
        ),
        "{shown}"
    );
    assert!(shown.contains("Line 3 has an error."), "{shown}");
    let one = file(
        "a.yml",
        "#draft-L",
        None,
        vec![
            finding(Gravity::Error, "E", Some((3, 1))),
            finding(Gravity::Warning, "W", Some((2, 1))),
        ],
    );
    let beside = html(|| {
        editor_split(
            panel(text, Opened::New, caret),
            editor_diagnostics(
                "Diagnostics",
                &one,
                &DiagnosticsWords::of_page(),
                editor(text),
                RwSignal::new(caret),
            ),
        )
    });
    assert!(
        has(&beside, "div", &[("class", "fui-editor-split")]),
        "{beside}"
    );
    assert!(
        has(
            &beside,
            "li",
            &[
                ("class", "fui-diag fui-tone-bad is-current"),
                ("aria-current", "true")
            ]
        ),
        "the row of the line the cursor is on is current: {beside}"
    );
    assert!(has(&beside, "a", &[("href", "#draft-L3")]), "{beside}");
}
