// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The diagnostics list (B3): one row per finding, errors first, grouped by
//! file when the list spans files.
//!
//! A row says the finding's gravity in shape, colour and word (a cross or a
//! triangle, the bad or warn tone, "Error" or "Warning"; family rule 12),
//! its code in the code face, its line and column as a link to the line,
//! its message, and the mapping it belongs to. The link targets the id the
//! code viewer ([`crate::view::code`]) and the editor
//! ([`crate::widget::editor::line_id`]) give each line, and the row for the
//! current line takes `is-current` and `aria-current`, in step with the
//! line. A file's name heads its rows as a link in the ink with no
//! underline (family rule 4), with its counts. On a phone a row is one
//! column, and a long code, message, mapping or path breaks rather than
//! widen the page (WCAG 2.2 SC 1.4.10). A file whose findings run past
//! [`WINDOW`] draws the first [`WINDOW`] and one quiet row with the link
//! to them all, when the console gives one; a list with no finding says
//! so in one line. No specification governs the list: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::code::Gravity;

/// The most rows one file draws while the console gives the link to all
/// of them ([`FileFindings::all`]).
pub const WINDOW: usize = 100;

/// A place in a text: its 1-based line and 1-based column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    /// The 1-based line.
    pub line: u32,
    /// The 1-based column, in characters.
    pub column: u32,
}

impl Default for Position {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

/// One finding of a check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// How grave it is.
    pub gravity: Gravity,
    /// Its code, such as `FC-201`, drawn in the code face.
    pub code: String,
    /// Where it is; `None` for a finding about the whole document.
    pub at: Option<Position>,
    /// What it says, a sentence.
    pub message: String,
    /// The name of the mapping it belongs to, when it belongs to one.
    pub mapping: Option<String>,
}

/// The findings of one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFindings {
    /// The file's path, the group's head in a list that spans files.
    pub file: String,
    /// The page that shows the file, which the head links to.
    pub href: Option<String>,
    /// What a link to a line is, before the line's number: such as
    /// `#draft-L` for an editor on the page, or `/files/a.yml#L` for a code
    /// viewer on another.
    pub lines: String,
    /// The line the reader is on, whose rows are current.
    pub current: Option<u32>,
    /// The findings, in any order: the list draws errors first.
    pub findings: Vec<Finding>,
    /// The link to the page that lists every finding of the file: while it
    /// is set, a file with more than [`WINDOW`] findings draws the first
    /// [`WINDOW`] and a row that offers the rest; `None` draws every
    /// finding, as the page that link opens does.
    pub all: Option<String>,
}

/// The rank of `gravity` in the list: errors first.
const fn rank(gravity: Gravity) -> u8 {
    match gravity {
        Gravity::Error => 0,
        Gravity::Warning => 1,
    }
}

/// `findings` in the order the list draws them: errors before warnings,
/// then a finding about the whole document, then by line and column.
#[must_use]
pub fn ordered(findings: &[Finding]) -> Vec<&Finding> {
    let mut out: Vec<&Finding> = findings.iter().collect();
    out.sort_by_key(|finding| (rank(finding.gravity), finding.at));
    out
}

/// `files` in the order the list draws them: a file with an error before
/// one with warnings alone, each group in the order given.
#[must_use]
pub fn ordered_files(files: &[FileFindings]) -> Vec<&FileFindings> {
    let mut out: Vec<&FileFindings> = files.iter().collect();
    out.sort_by_key(|file| {
        !file
            .findings
            .iter()
            .any(|finding| finding.gravity == Gravity::Error)
    });
    out
}

/// The counts of `file` as its head says them, such as "2 errors, 1
/// warning".
fn counts(file: &FileFindings) -> Word {
    let errors = file
        .findings
        .iter()
        .filter(|finding| finding.gravity == Gravity::Error)
        .count();
    let warnings = file.findings.len().saturating_sub(errors);
    let errors =
        (errors > 0).then(|| tr_with(Text::DiagnosticsErrors, &[("count", errors.into())]));
    let warnings =
        (warnings > 0).then(|| tr_with(Text::DiagnosticsWarnings, &[("count", warnings.into())]));
    match (errors, warnings) {
        (Some(errors), Some(warnings)) => tr_with(
            Text::DiagnosticsErrorsWarnings,
            &[("errors", errors.into()), ("warnings", warnings.into())],
        ),
        (Some(one), None) | (None, Some(one)) => one,
        (None, None) => Word::default(),
    }
}

/// The words a diagnostics row draws, resolved on the server in the page's
/// locale, so an island that draws the list carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticsWords {
    /// The word of an error.
    pub error: Word,
    /// The word of a warning.
    pub warning: Word,
    /// The place of a finding about the whole document.
    pub whole: Word,
    /// The place of a finding, around the open `line` and `column`.
    pub line: Phrase,
    /// The mapping a finding belongs to, around the open `mapping`.
    pub mapping: Phrase,
    /// The row under a window, around the open `shown` and `total`.
    pub more: Phrase,
    /// That row's link to every finding, around the open `total`.
    pub more_all: Phrase,
    /// What a list with no finding says.
    pub none: Word,
}

impl DiagnosticsWords {
    /// The row words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            error: tr(Text::DiagnosticsError),
            warning: tr(Text::DiagnosticsWarning),
            whole: tr(Text::DiagnosticsWhole),
            line: Phrase::new(tr_with(
                Text::DiagnosticsLine,
                &[
                    ("line", Phrase::open("line")),
                    ("column", Phrase::open("column")),
                ],
            )),
            mapping: Phrase::new(tr_with(
                Text::DiagnosticsMapping,
                &[("mapping", Phrase::open("mapping"))],
            )),
            more: Phrase::new(tr_with(
                Text::DiagnosticsMore,
                &[
                    ("shown", Phrase::open("shown")),
                    ("total", Phrase::open("total")),
                ],
            )),
            more_all: Phrase::new(tr_with(
                Text::DiagnosticsMoreAll,
                &[("total", Phrase::open("total"))],
            )),
            none: tr(Text::DiagnosticsNone),
        }
    }
}

/// Returns the row of `finding` of a file whose lines are linked as
/// `lines`; it is current while `current` names its line, and `on_line`
/// hears a click on its line link.
fn row(
    finding: &Finding,
    lines: &str,
    words: &DiagnosticsWords,
    current: Signal<Option<u32>>,
    on_line: Option<Callback<Position>>,
) -> AnyView {
    let (class, glyph, word) = match finding.gravity {
        Gravity::Error => ("fui-diag fui-tone-bad", Glyph::CIRCLE_X, &words.error),
        Gravity::Warning => (
            "fui-diag fui-tone-warn",
            Glyph::TRIANGLE_ALERT,
            &words.warning,
        ),
    };
    let line = finding.at.map(|at| at.line);
    let is_current = move || line.is_some() && current.get() == line;
    let place = match finding.at {
        Some(at) => {
            let href = format!("{lines}{}", at.line);
            let words = words.line.fill(&[
                ("line", &at.line.to_string()),
                ("column", &at.column.to_string()),
            ]);
            view! {
                <a
                    class="fui-diag-where"
                    href=href
                    on:click=move |_| {
                        if let Some(on_line) = on_line {
                            on_line.run(at);
                        }
                    }
                >
                    {words}
                </a>
            }
            .into_any()
        }
        None => view! { <span class="fui-diag-where">{words.whole.clone()}</span> }.into_any(),
    };
    let mapping = finding.mapping.as_ref().map(|mapping| {
        let words = words.mapping.fill(&[("mapping", mapping)]);
        view! { <span class="fui-diag-mapping">{words}</span> }
    });
    view! {
        <li
            class=class
            class:is-current=is_current
            aria-current=move || is_current().then_some("true")
        >
            <span class="fui-diag-sev">{icon(glyph, "14")} {word.clone()}</span>
            <span class="fui-diag-head">
                <code class="fui-diag-code">{finding.code.clone()}</code>
                {place}
            </span>
            <p class="fui-diag-msg">{finding.message.clone()} {mapping}</p>
        </li>
    }
    .into_any()
}

/// Returns the row that says a list holds no finding, in `words`.
fn none_row(words: &DiagnosticsWords) -> AnyView {
    view! {
        <li class="fui-diags-empty">
            {icon(Glyph::CIRCLE_CHECK, "14")} <span>{words.none.clone()}</span>
        </li>
    }
    .into_any()
}

/// The number of rows of `file` the list draws: every finding, or the
/// first [`WINDOW`] while the console links to them all.
fn shown(file: &FileFindings) -> usize {
    match file.all {
        Some(_) => file.findings.len().min(WINDOW),
        None => file.findings.len(),
    }
}

/// Returns the rows of `file`, errors first, current while `current` names
/// their line, in `words`, the first [`WINDOW`] and a more row while the
/// console links to them all; `on_line` hears a click on a line link.
pub(crate) fn rows(
    file: &FileFindings,
    words: &DiagnosticsWords,
    current: Signal<Option<u32>>,
    on_line: Option<Callback<Position>>,
) -> AnyView {
    if file.findings.is_empty() {
        return none_row(words);
    }
    let shown = shown(file);
    let total = file.findings.len();
    let drawn = ordered(&file.findings)
        .into_iter()
        .take(shown)
        .map(|finding| row(finding, &file.lines, words, current, on_line))
        .collect_view();
    let more = file.all.as_ref().filter(|_| shown < total).map(|all| {
        let (shown, total) = (shown.to_string(), total.to_string());
        let said = words.more.fill(&[("shown", &shown), ("total", &total)]);
        let link = words.more_all.fill(&[("total", &total)]);
        view! {
            <li class="fui-diags-more">
                <span>{said} " " <a href=all.clone()>{link}</a></span>
            </li>
        }
    });
    view! { {drawn} {more} }.into_any()
}

/// Returns the diagnostics list of one file, named `label`, with no file
/// head: the list beside an editor or a code viewer of that file.
#[must_use]
pub fn diagnostics(label: &str, file: &FileFindings) -> AnyView {
    let rows = rows(
        file,
        &DiagnosticsWords::of_page(),
        Signal::stored(file.current),
        None,
    );
    view! {
        <ul class="fui-diags" aria-label=label.to_owned()>
            {rows}
        </ul>
    }
    .into_any()
}

/// Returns the diagnostics list of `files`, named `label`: each file's
/// rows under its head, the files with an error first.
#[must_use]
pub fn diagnostics_by_file(label: &str, files: &[FileFindings]) -> AnyView {
    let words = DiagnosticsWords::of_page();
    let groups = ordered_files(files)
        .into_iter()
        .map(|file| {
            let name = crate::view::code::path(&file.file);
            let name = match &file.href {
                Some(href) => view! {
                    <a href=href.clone() class="fui-row-link">
                        <code>{name}</code>
                    </a>
                }
                .into_any(),
                None => view! { <code>{name}</code> }.into_any(),
            };
            let rows = rows(file, &words, Signal::stored(file.current), None);
            view! {
                <li>
                    <p class="fui-diags-file">
                        {name} <span class="fui-muted">{counts(file)}</span>
                    </p>
                    <ul class="fui-diags">{rows}</ul>
                </li>
            }
        })
        .collect_view();
    let none = files.is_empty().then(|| none_row(&words));
    view! {
        <ul class="fui-diags" aria-label=label.to_owned()>
            {none}
            {groups}
        </ul>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::{FileFindings, Finding, Position, ordered, ordered_files};
    use crate::view::code::Gravity;

    fn finding(gravity: Gravity, at: Option<(u32, u32)>, code: &str) -> Finding {
        Finding {
            gravity,
            code: code.to_owned(),
            at: at.map(|(line, column)| Position { line, column }),
            message: String::new(),
            mapping: None,
        }
    }

    fn file(name: &str, findings: Vec<Finding>) -> FileFindings {
        FileFindings {
            file: name.to_owned(),
            href: None,
            lines: String::from("#L"),
            current: None,
            findings,
            all: None,
        }
    }

    #[test]
    fn errors_come_first_then_the_whole_document_then_by_line_and_column() {
        let findings = [
            finding(Gravity::Warning, Some((2, 1)), "w2"),
            finding(Gravity::Error, Some((9, 4)), "e9-4"),
            finding(Gravity::Error, Some((9, 2)), "e9-2"),
            finding(Gravity::Error, None, "e-whole"),
            finding(Gravity::Error, Some((3, 7)), "e3"),
        ];
        let codes: Vec<&str> = ordered(&findings)
            .iter()
            .map(|finding| finding.code.as_str())
            .collect();
        assert_eq!(vec!["e-whole", "e3", "e9-2", "e9-4", "w2"], codes);
    }

    #[test]
    fn a_file_with_an_error_comes_before_one_with_warnings_alone() {
        let files = [
            file("a.yml", vec![finding(Gravity::Warning, Some((1, 1)), "w")]),
            file("b.yml", vec![finding(Gravity::Error, Some((1, 1)), "e")]),
            file("c.yml", vec![finding(Gravity::Warning, Some((1, 1)), "w")]),
        ];
        let names: Vec<&str> = ordered_files(&files)
            .iter()
            .map(|file| file.file.as_str())
            .collect();
        assert_eq!(vec!["b.yml", "a.yml", "c.yml"], names);
    }
}
