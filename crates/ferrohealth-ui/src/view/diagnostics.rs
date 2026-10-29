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
//! ([`crate::view::editor::line_id`]) give each line, and the row for the
//! current line takes `is-current` and `aria-current`, which the `editor`
//! module keeps in step with the caret. A file's name heads its rows as a link in the ink with no
//! underline (family rule 4), with its counts. On a phone a row is one
//! column, and a long code, message, mapping or path breaks rather than
//! widen the page (WCAG 2.2 SC 1.4.10). A file whose findings run past
//! [`WINDOW`] draws the first [`WINDOW`] and one quiet row with the link
//! to them all, when the console gives one; a list with no finding says
//! so in one line. No specification governs the list: our own design.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
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
fn counts(locale: &Locale, file: &FileFindings) -> Word {
    let errors = file
        .findings
        .iter()
        .filter(|finding| finding.gravity == Gravity::Error)
        .count();
    let warnings = file.findings.len().saturating_sub(errors);
    let errors =
        (errors > 0).then(|| tr_with(locale, Text::DiagnosticsErrors, &[("count", errors.into())]));
    let warnings = (warnings > 0).then(|| {
        tr_with(
            locale,
            Text::DiagnosticsWarnings,
            &[("count", warnings.into())],
        )
    });
    match (errors, warnings) {
        (Some(errors), Some(warnings)) => tr_with(
            locale,
            Text::DiagnosticsErrorsWarnings,
            &[("errors", errors.into()), ("warnings", warnings.into())],
        ),
        (Some(one), None) | (None, Some(one)) => one,
        (None, None) => Word::default(),
    }
}

/// The words a diagnostics row draws, in the page's locale.
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
    /// The row words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            error: tr(locale, Text::DiagnosticsError),
            warning: tr(locale, Text::DiagnosticsWarning),
            whole: tr(locale, Text::DiagnosticsWhole),
            line: Phrase::new(tr_with(
                locale,
                Text::DiagnosticsLine,
                &[
                    ("line", Phrase::open("line")),
                    ("column", Phrase::open("column")),
                ],
            )),
            mapping: Phrase::new(tr_with(
                locale,
                Text::DiagnosticsMapping,
                &[("mapping", Phrase::open("mapping"))],
            )),
            more: Phrase::new(tr_with(
                locale,
                Text::DiagnosticsMore,
                &[
                    ("shown", Phrase::open("shown")),
                    ("total", Phrase::open("total")),
                ],
            )),
            more_all: Phrase::new(tr_with(
                locale,
                Text::DiagnosticsMoreAll,
                &[("total", Phrase::open("total"))],
            )),
            none: tr(locale, Text::DiagnosticsNone),
        }
    }
}

/// Returns the row of `finding` of a file whose lines are linked as
/// `lines`; it is current while `current` names its line.
fn row(finding: &Finding, lines: &str, words: &DiagnosticsWords, current: Option<u32>) -> Html {
    let (class, glyph, word) = match finding.gravity {
        Gravity::Error => ("fui-diag fui-tone-bad", Glyph::CIRCLE_X, &words.error),
        Gravity::Warning => (
            "fui-diag fui-tone-warn",
            Glyph::TRIANGLE_ALERT,
            &words.warning,
        ),
    };
    let line = finding.at.map(|at| at.line);
    RowView {
        line,
        current: line.is_some() && current == line,
        class,
        glyph: icon(glyph, "14"),
        word,
        code: &finding.code,
        place: finding.at.map(|at| {
            (
                format!("{lines}{}", at.line),
                words.line.fill(&[
                    ("line", &at.line.to_string()),
                    ("column", &at.column.to_string()),
                ]),
                at.column,
            )
        }),
        whole: &words.whole,
        message: &finding.message,
        mapping: finding
            .mapping
            .as_ref()
            .map(|mapping| words.mapping.fill(&[("mapping", mapping)])),
    }
    .html()
}

#[derive(Template)]
#[template(path = "diagnostics/diagnostics.html", block = "row")]
struct RowView<'a> {
    line: Option<u32>,
    current: bool,
    class: &'static str,
    glyph: Html,
    word: &'a Word,
    code: &'a str,
    place: Option<(String, Word, u32)>,
    whole: &'a Word,
    message: &'a str,
    mapping: Option<Word>,
}

/// Returns the row that says a list holds no finding, in `words`.
fn none_row(words: &DiagnosticsWords) -> Html {
    NoneView {
        glyph: icon(Glyph::CIRCLE_CHECK, "14"),
        words: &words.none,
    }
    .html()
}

#[derive(Template)]
#[template(path = "diagnostics/diagnostics.html", block = "none")]
struct NoneView<'a> {
    glyph: Html,
    words: &'a Word,
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
/// console links to them all.
pub(crate) fn rows(file: &FileFindings, words: &DiagnosticsWords, current: Option<u32>) -> Html {
    if file.findings.is_empty() {
        return none_row(words);
    }
    let shown = shown(file);
    let total = file.findings.len();
    let drawn: Html = ordered(&file.findings)
        .into_iter()
        .take(shown)
        .map(|finding| row(finding, &file.lines, words, current))
        .collect();
    let more = file
        .all
        .as_ref()
        .filter(|_| shown < total)
        .map(|all| {
            let (shown, total) = (shown.to_string(), total.to_string());
            MoreView {
                said: words.more.fill(&[("shown", &shown), ("total", &total)]),
                all,
                link: words.more_all.fill(&[("total", &total)]),
            }
            .html()
        })
        .unwrap_or_default();
    drawn.then(more)
}

#[derive(Template)]
#[template(path = "diagnostics/diagnostics.html", block = "more")]
struct MoreView<'a> {
    said: Word,
    all: &'a str,
    link: Word,
}

/// Returns the diagnostics list of one file, named `label`, with no file
/// head: the list beside an editor or a code viewer of that file.
#[must_use]
pub fn diagnostics(locale: &Locale, label: &str, file: &FileFindings) -> Html {
    let rows = rows(file, &DiagnosticsWords::of(locale), file.current);
    Html::raw(format!(
        "<ul aria-label=\"{}\" class=\"fui-diags\">{rows}</ul>",
        Html::text(label)
    ))
}

/// Returns the diagnostics list of `files`, named `label`: each file's
/// rows under its head, the files with an error first.
#[must_use]
pub fn diagnostics_by_file(locale: &Locale, label: &str, files: &[FileFindings]) -> Html {
    let words = DiagnosticsWords::of(locale);
    let groups: Html = ordered_files(files)
        .into_iter()
        .map(|file| {
            let path = crate::view::code::path(&file.file);
            let name = match &file.href {
                Some(href) => Html::raw(format!(
                    "<a href=\"{}\" class=\"fui-row-link\"><code>{path}</code></a>",
                    Html::text(href)
                )),
                None => Html::raw(format!("<code>{path}</code>")),
            };
            FileView {
                name,
                counts: counts(locale, file),
                rows: rows(file, &words, file.current),
            }
            .html()
        })
        .collect();
    let none = if files.is_empty() {
        none_row(&words)
    } else {
        Html::empty()
    };
    Html::raw(format!(
        "<ul aria-label=\"{}\" class=\"fui-diags\">{none}{groups}</ul>",
        Html::text(label)
    ))
}

#[derive(Template)]
#[template(path = "diagnostics/diagnostics.html", block = "file")]
struct FileView {
    name: Html,
    counts: Word,
    rows: Html,
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
