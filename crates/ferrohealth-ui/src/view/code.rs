// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code viewer (B1): a read-only file or document, its lines numbered,
//! each diagnostic under the line it names.
//!
//! A finding on line 0, about the whole file, sits above the first line,
//! and one past the last line sits under the last line, so no finding is
//! left out. The findings are indexed by line once, so a long file with
//! many findings draws in time linear in both.
//!
//! A head names the file: its path as the title in the prose font, its
//! tags (the language, the owner), the line count and the short digest.
//! In a card ([`code_card`]) the card edge is its frame; in a panel section
//! ([`code_section`]) it runs edge to edge and the section's heading row is
//! its one head; with no head ([`code_viewer`] with `None`) it is the bare
//! box a page with no file to name draws. Code never wraps inside a token:
//! a long line scrolls sideways under the scroll cue (`fui-scroll-cue`, a
//! thin scrollbar and a soft shadow on the clipped edge, decorative), and a
//! diagnostic stays in view while its line scrolls. A final line break
//! ends the last line and opens none, as a file reads. The scrolling box is
//! focusable, so the keyboard scrolls it (WCAG 2.2 SC 2.1.1). No
//! specification governs the viewer: our own design.

use std::collections::BTreeMap;

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::highlight::{self, Language};
use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::copy::{CopySource, CopyWords, copy_from};
use crate::view::surface::{SectionHead, panel_section};

/// How grave a finding on a line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gravity {
    /// The line is wrong: the bad tone.
    Error,
    /// The line may be wrong: the warn tone.
    Warning,
}

/// A finding drawn under the line it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineDiagnostic {
    /// The 1-based line it names: 0 draws it above the first line, and a
    /// line past the last draws it under the last.
    pub line: u32,
    /// How grave it is.
    pub gravity: Gravity,
    /// Its code, such as `FC-201`, drawn in the code face.
    pub code: String,
    /// What it says, a sentence.
    pub message: String,
}

/// The text a code viewer draws.
#[derive(Debug, Clone)]
pub struct Code {
    /// The scrolling box's accessible name, such as "The mapping file".
    pub label: String,
    /// The grammar the text is highlighted as.
    pub language: Language,
    /// The text, as the server sent it.
    pub text: String,
    /// The id prefix each line takes before its number, which a link to a
    /// line targets; `None` for lines with no id.
    pub ids: Option<String>,
    /// The line a link or a chosen diagnostic points at, marked current.
    pub current: Option<u32>,
    /// The findings, each drawn under its line.
    pub diagnostics: Vec<LineDiagnostic>,
}

/// A short digest of the bytes, as the head shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    /// What the head shows, such as "SHA-256 5712e3f7…f2e0e3fa".
    pub text: String,
    /// The tooltip that says what it is a digest of.
    pub title: String,
}

/// What a code viewer's head shows.
#[derive(Debug, Default)]
pub struct CodeHead {
    /// The file's path, or the document's name, the head's title.
    pub title: String,
    /// Short tags after it, such as the language and the owner.
    pub tags: Vec<String>,
    /// The short digest, when the console has one.
    pub digest: Option<Digest>,
    /// The id of the head's Copy button, which copies the text; `None`
    /// draws none.
    pub copy: Option<String>,
    /// The actions at the head's end, after Copy, such as Download.
    pub actions: Option<Html>,
}

/// The class of line `line` of `code`: the gravest finding on it marks
/// it; the current line takes `is-current` beside.
#[must_use]
pub fn line_class(code: &Code, line: u32) -> &'static str {
    class_of(code.diagnostics.iter().filter(|found| found.line == line))
}

/// The class of a line on which `found` are the findings.
fn class_of<'a>(found: impl IntoIterator<Item = &'a LineDiagnostic>) -> &'static str {
    let mut found = found.into_iter().peekable();
    if found.peek().is_none() {
        "fui-code-line"
    } else if found.any(|found| found.gravity == Gravity::Error) {
        "fui-code-line fui-mark-error"
    } else {
        "fui-code-line fui-mark-warning"
    }
}

/// The findings of `diagnostics` by the line they are drawn at, each in
/// the order given: 0 above the first line, `last` plus one for every
/// line past the last.
fn by_line(diagnostics: &[LineDiagnostic], last: u32) -> BTreeMap<u32, Vec<&LineDiagnostic>> {
    let after = last.saturating_add(1);
    let mut out: BTreeMap<u32, Vec<&LineDiagnostic>> = BTreeMap::new();
    for found in diagnostics {
        out.entry(found.line.min(after)).or_default().push(found);
    }
    out
}

/// Returns the finding `found`, drawn under its line: the gravity's glyph
/// and word (the word for assistive technology), the code and the message.
fn diagnostic(locale: &Locale, found: &LineDiagnostic) -> Html {
    let (tone, glyph, word) = match found.gravity {
        Gravity::Error => (
            "fui-code-diag fui-tone-bad",
            Glyph::CIRCLE_X,
            Text::CodeError,
        ),
        Gravity::Warning => (
            "fui-code-diag fui-tone-warn",
            Glyph::TRIANGLE_ALERT,
            Text::CodeWarning,
        ),
    };
    DiagnosticView {
        tone,
        glyph: icon(glyph, "14"),
        word: tr(locale, word),
        code: &found.code,
        message: &found.message,
    }
    .html()
}

#[derive(Template)]
#[template(path = "code/code.html", block = "diagnostic")]
struct DiagnosticView<'a> {
    tone: &'static str,
    glyph: Html,
    word: Word,
    code: &'a str,
    message: &'a str,
}

/// Returns the numbered lines of `code`, each finding under its line, a
/// finding on line 0 above the first and one past the last under the last.
fn numbered_lines(locale: &Locale, code: &Code) -> Html {
    let text = code.text.strip_suffix('\n').unwrap_or(&code.text);
    let lines = highlight::lines(code.language, text);
    let last = u32::try_from(lines.len()).unwrap_or(u32::MAX);
    let found = by_line(&code.diagnostics, last);
    let drawn = |line: u32| -> Html {
        found
            .get(&line)
            .map(|found| {
                found
                    .iter()
                    .map(|found| diagnostic(locale, found))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut out = drawn(0);
    for (line, tokens) in (1_u32..).zip(lines) {
        let id = code.ids.as_ref().map_or_else(String::new, |prefix| {
            format!(" id=\"{}\"", Html::text(format!("{prefix}{line}")))
        });
        let mut class = class_of(found.get(&line).into_iter().flatten().copied()).to_owned();
        if code.current == Some(line) {
            class.push_str(" is-current");
        }
        out = out
            .then(Html::raw(format!(
                "<span{id} class=\"{class}\">{}</span>",
                highlight::line_html(&tokens)
            )))
            .then(drawn(line));
    }
    out.then(drawn(last.saturating_add(1)))
}

/// The number of lines `text` reads as: a final line break opens none.
#[must_use]
pub fn line_count(text: &str) -> u32 {
    let text = text.strip_suffix('\n').unwrap_or(text);
    let breaks = text.bytes().filter(|byte| *byte == b'\n').count();
    u32::try_from(breaks).map_or(u32::MAX, |breaks| breaks.saturating_add(1))
}

/// The id of the scrolling box whose head's Copy button is `copy`, which
/// the button copies the lines of.
fn box_id(copy: &str) -> String {
    format!("{copy}-text")
}

/// Returns the scrolling box of `code`, with the id `id` when it has one:
/// its numbered lines under the scroll cue.
fn code_box(locale: &Locale, code: &Code, id: Option<String>) -> Html {
    CodeBox {
        id,
        label: tr_with(
            locale,
            Text::CodeRegion,
            &[("code", code.label.clone().into())],
        )
        .text()
        .to_owned(),
        lines: numbered_lines(locale, code),
    }
    .html()
}

#[derive(Template)]
#[template(path = "code/code.html", block = "code_box")]
struct CodeBox {
    id: Option<String>,
    label: String,
    lines: Html,
}

/// Returns the head's meta: its tags, the line count of `text`, its digest,
/// the Copy button `copy`, which copies the lines of the box it names
/// ([`box_id`]), and its actions.
fn meta(
    locale: &Locale,
    tags: Vec<String>,
    text: &str,
    digest: Option<Digest>,
    copy: Option<String>,
    actions: Option<Html>,
) -> Html {
    let copy = copy
        .map(|id| {
            let source = CopySource::Lines {
                id: box_id(&id),
                final_break: text.ends_with('\n'),
            };
            copy_from(
                id,
                tr(locale, Text::DocumentCopy),
                source,
                CopyWords::of(locale),
            )
        })
        .unwrap_or_default();
    MetaView {
        tags: tags.into_iter().map(crate::view::tag::tag).collect(),
        lines: tr_with(
            locale,
            Text::CodeLines,
            &[("count", line_count(text).into())],
        ),
        digest,
        copy,
        actions: actions.unwrap_or_default(),
    }
    .html()
}

#[derive(Template)]
#[template(path = "code/code.html", block = "meta")]
struct MetaView {
    tags: Html,
    lines: Word,
    digest: Option<Digest>,
    copy: Html,
    actions: Html,
}

/// Returns `title` as a path that may break after each slash and never
/// inside a name.
pub(crate) fn path(title: &str) -> Html {
    let mut parts = title.split_inclusive('/').peekable();
    let mut out = Html::empty();
    while let Some(part) = parts.next() {
        out = out.then(Html::text(part));
        if parts.peek().is_some() {
            out = out.then(Html::raw(String::from("<wbr>")));
        }
    }
    out
}

/// Returns the code viewer of `code`, under `head` when it has one.
///
/// The head's title is a level-three heading, as in a tab panel beside its
/// sections ([`crate::view::surface::source_split`]); with no head it is the
/// bare box.
#[must_use]
pub fn code_viewer(locale: &Locale, head: Option<CodeHead>, code: &Code) -> Html {
    let (head, source) = match head {
        Some(CodeHead {
            title,
            tags,
            digest,
            copy,
            actions,
        }) => {
            let source = copy.as_deref().map(box_id);
            let meta = meta(locale, tags, &code.text, digest, copy, actions);
            (Some((path(&title), meta)), source)
        }
        None => (None, None),
    };
    ViewerView {
        head,
        body: code_box(locale, code, source),
    }
    .html()
}

#[derive(Template)]
#[template(path = "code/code.html", block = "viewer")]
struct ViewerView {
    head: Option<(Html, Html)>,
    body: Html,
}

/// Returns the code viewer of `code` in a card of its own, named by its
/// head, whose title is the card's level-two heading.
#[must_use]
pub fn code_card(locale: &Locale, head: CodeHead, code: &Code) -> Html {
    let CodeHead {
        title,
        tags,
        digest,
        copy,
        actions,
    } = head;
    let source = copy.as_deref().map(box_id);
    let meta = meta(locale, tags, &code.text, digest, copy, actions);
    CardView {
        path: path(&title),
        name: title,
        meta,
        body: code_box(locale, code, source),
    }
    .html()
}

#[derive(Template)]
#[template(path = "code/code.html", block = "card")]
struct CardView {
    name: String,
    path: Html,
    meta: Html,
    body: Html,
}

/// Returns the code viewer of `code` as a panel section (B10): the
/// section's heading row carries the head's title and meta, its one head,
/// and the viewer runs edge to edge under it.
#[must_use]
pub fn code_section(locale: &Locale, head: CodeHead, code: &Code) -> Html {
    let CodeHead {
        title,
        tags,
        digest,
        copy,
        actions,
    } = head;
    let source = copy.as_deref().map(box_id);
    let meta = meta(locale, tags, &code.text, digest, copy, actions);
    let body = String::from(code_box(locale, code, source));
    let viewer = Html::raw(format!("<div class=\"fui-codeview\">{body}</div>"));
    panel_section(
        locale,
        Some(SectionHead {
            title,
            count: None,
            actions: Some(meta),
        }),
        viewer,
    )
}

/// Returns `text` in `language`, highlighted in a box capped at 24 rem
/// with its own scroll and the scroll cue, for a product's raw
/// self-description on its About page.
///
/// A line never wraps: a long one scrolls sideways, on a phone too.
#[must_use]
pub fn self_description(label: &str, language: Language, text: &str) -> Html {
    let text = text.strip_suffix('\n').unwrap_or(text);
    SelfDescriptionView {
        label,
        lines: Html::raw(highlight::html(
            language,
            text,
            highlight::Lines::default(),
            |_| None,
        )),
    }
    .html()
}

#[derive(Template)]
#[template(path = "code/code.html", block = "self_description")]
struct SelfDescriptionView<'a> {
    label: &'a str,
    lines: Html,
}

#[cfg(test)]
mod tests {
    use super::{Code, Gravity, LineDiagnostic, line_class, line_count};
    use crate::highlight::Language;

    #[test]
    fn a_final_line_break_opens_no_line() {
        assert_eq!(1, line_count(""));
        assert_eq!(1, line_count("a\n"));
        assert_eq!(2, line_count("a\nb"));
        assert_eq!(3, line_count("a\n\nb\n"));
    }

    #[test]
    fn an_error_outranks_a_warning_on_its_line() {
        let found = |line, gravity| LineDiagnostic {
            line,
            gravity,
            code: String::new(),
            message: String::new(),
        };
        let code = Code {
            label: String::new(),
            language: Language::Plain,
            text: String::from("a\nb\nc"),
            ids: None,
            current: Some(2),
            diagnostics: vec![
                found(2, Gravity::Warning),
                found(2, Gravity::Error),
                found(3, Gravity::Warning),
            ],
        };
        assert_eq!("fui-code-line", line_class(&code, 1));
        assert_eq!("fui-code-line fui-mark-error", line_class(&code, 2));
        assert_eq!("fui-code-line fui-mark-warning", line_class(&code, 3));
    }
}
