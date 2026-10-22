// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code viewer (B1): a read-only file or document, its lines numbered,
//! each diagnostic under the line it names.
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

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::highlight::{self, Language};
use crate::i18n::{Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::island::CopyButton;
use crate::view::surface::{SectionHead, panel_section};
use crate::widget::copy::CopyWords;

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
    /// The 1-based line it names.
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
    pub actions: Option<AnyView>,
}

/// The class of line `line` of `code`: the gravest finding on it marks
/// it; the current line takes `is-current` beside.
#[must_use]
pub fn line_class(code: &Code, line: u32) -> &'static str {
    let mut found = code.diagnostics.iter().filter(|found| found.line == line);
    if found.clone().next().is_none() {
        "fui-code-line"
    } else if found.any(|found| found.gravity == Gravity::Error) {
        "fui-code-line fui-mark-error"
    } else {
        "fui-code-line fui-mark-warning"
    }
}

/// Returns the finding `found`, drawn under its line: the gravity's glyph
/// and word (the word for assistive technology), the code and the message.
fn diagnostic(found: &LineDiagnostic) -> AnyView {
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
    view! {
        <span class=tone role="note">
            {icon(glyph, "14")}
            <span class="fui-code-diag-text">
                <span class="fui-sr-only">{tr(word)} " "</span>
                <span class="fui-code-diag-code">{found.code.clone()}</span>
                <span class="fui-sr-only">" "</span>
                {found.message.clone()}
            </span>
        </span>
    }
    .into_any()
}

/// Returns the numbered lines of `code`, each finding under its line.
fn numbered_lines(code: &Code) -> AnyView {
    let text = code.text.strip_suffix('\n').unwrap_or(&code.text);
    (1_u32..)
        .zip(highlight::lines(code.language, text))
        .map(|(line, tokens)| {
            let id = code.ids.as_ref().map(|prefix| format!("{prefix}{line}"));
            let found = code
                .diagnostics
                .iter()
                .filter(|found| found.line == line)
                .map(diagnostic)
                .collect_view();
            let class = line_class(code, line);
            let current = code.current == Some(line);
            let html = highlight::line_html(&tokens);
            view! { <span class=class class:is-current=current id=id inner_html=html></span>{found} }
        })
        .collect_view()
        .into_any()
}

/// The number of lines `text` reads as: a final line break opens none.
#[must_use]
pub fn line_count(text: &str) -> u32 {
    let text = text.strip_suffix('\n').unwrap_or(text);
    let breaks = text.bytes().filter(|byte| *byte == b'\n').count();
    u32::try_from(breaks).map_or(u32::MAX, |breaks| breaks.saturating_add(1))
}

/// Returns the scrolling box of `code`: its numbered lines under the
/// scroll cue.
fn code_box(code: &Code) -> AnyView {
    let label = tr_with(Text::CodeRegion, &[("code", code.label.clone().into())]);
    view! {
        <div
            class="fui-code-panel fui-code-panel-numbered fui-scroll-cue"
            role="region"
            aria-label=label
            tabindex="0"
        >
            <pre>{numbered_lines(code)}</pre>
        </div>
    }
    .into_any()
}

/// Returns the head's meta: its tags, the line count of `text`, its digest,
/// the Copy button `copy` over `text` and its actions.
fn meta(
    tags: Vec<String>,
    text: &str,
    digest: Option<Digest>,
    copy: Option<String>,
    actions: Option<AnyView>,
) -> AnyView {
    let copy = copy.map(|id| {
        view! {
            <CopyButton
                id=id
                label=tr(Text::DocumentCopy)
                text=text.to_owned()
                words=CopyWords::of_page()
            />
        }
    });
    let tags = tags.into_iter().map(crate::view::tag::tag).collect_view();
    let lines = tr_with(Text::CodeLines, &[("count", line_count(text).into())]);
    let digest = digest.map(|Digest { text, title }| {
        view! {
            <span class="fui-codeview-sha" title=title>
                {text}
            </span>
        }
    });
    view! {
        <span class="fui-codeview-meta">
            {tags} <span>{lines}</span> {digest} {copy} {actions}
        </span>
    }
    .into_any()
}

/// Returns `title` as a path that may break after each slash and never
/// inside a name.
pub(crate) fn path(title: &str) -> AnyView {
    let mut parts = title.split_inclusive('/').peekable();
    let mut out = Vec::new();
    while let Some(part) = parts.next() {
        let more = parts.peek().is_some();
        out.push(view! { {part.to_owned()}{more.then(|| view! { <wbr /> })} });
    }
    out.collect_view().into_any()
}

/// Returns the code viewer of `code`, under `head` when it has one.
///
/// The head's title is a level-three heading, as in a tab panel beside its
/// sections ([`crate::view::surface::source_split`]); with no head it is the
/// bare box.
#[must_use]
pub fn code_viewer(head: Option<CodeHead>, code: &Code) -> AnyView {
    let head = head.map(
        |CodeHead {
             title,
             tags,
             digest,
             copy,
             actions,
         }| {
            let meta = meta(tags, &code.text, digest, copy, actions);
            view! {
                <div class="fui-codeview-head">
                    <h3 class="fui-codeview-path">{path(&title)}</h3>
                    {meta}
                </div>
            }
        },
    );
    let body = code_box(code);
    view! { <div class="fui-codeview">{head} {body}</div> }.into_any()
}

/// Returns the code viewer of `code` in a card of its own, named by its
/// head, whose title is the card's level-two heading.
#[must_use]
pub fn code_card(head: CodeHead, code: &Code) -> AnyView {
    let CodeHead {
        title,
        tags,
        digest,
        copy,
        actions,
    } = head;
    let name = title.clone();
    let meta = meta(tags, &code.text, digest, copy, actions);
    let body = code_box(code);
    view! {
        <section class="fui-card fui-codeview-card" aria-label=name>
            <div class="fui-codeview">
                <div class="fui-codeview-head">
                    <h2 class="fui-codeview-path">{path(&title)}</h2>
                    {meta}
                </div>
                {body}
            </div>
        </section>
    }
    .into_any()
}

/// Returns the code viewer of `code` as a panel section (B10): the
/// section's heading row carries the head's title and meta, its one head,
/// and the viewer runs edge to edge under it.
#[must_use]
pub fn code_section(head: CodeHead, code: &Code) -> AnyView {
    let CodeHead {
        title,
        tags,
        digest,
        copy,
        actions,
    } = head;
    let meta = meta(tags, &code.text, digest, copy, actions);
    let viewer = code_viewer(None, code);
    panel_section(
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
pub fn self_description(label: &str, language: Language, text: &str) -> impl IntoView + use<> {
    let text = text.strip_suffix('\n').unwrap_or(text);
    let lines = highlight::html(language, text, highlight::Lines::default(), |_| None);
    view! {
        <div
            class="fui-code-panel fui-self-description fui-scroll-cue"
            role="region"
            aria-label=label.to_owned()
            tabindex="0"
        >
            <pre inner_html=lines></pre>
        </div>
    }
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
