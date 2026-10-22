// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The highlighted editor: a text area the operator types into, over a
//! highlighted, numbered layer that draws the same text with the same
//! tokenizer a code panel uses.
//!
//! The text area keeps the browser's own editing, selection, undo and the
//! exact characters typed: its text is transparent, and only its caret and
//! selection show over the layer. Both sit in one grid cell of one scrolling
//! box, drawn with the same face, size, padding and wrapping, so the layer
//! sizes the text area and the two scroll together, and a link to a line
//! ([`line_id`]) scrolls the box to it. No script is written by hand and no
//! editor library is used.
//!
//! The text area takes its text through `prop:value` alone, never a text
//! child: the server writes a text area's child as it stands, so a text
//! holding `</textarea>` would end the element. The layer is escaped by
//! [`crate::highlight::html`], so the server draws it, and the page reads
//! the text before it hydrates; the text area is read-only until then, so
//! no keystroke is lost.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::highlight::{Language, Lines, html};

/// How a marked line reads: the severity of the diagnostic that names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkKind {
    /// An error names the line.
    Error,
    /// A warning, and no error, names the line.
    Warning,
}

impl MarkKind {
    /// The class the stylesheet tints a line of this kind with.
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::Error => "fui-mark-error",
            Self::Warning => "fui-mark-warning",
        }
    }
}

/// A line of the editor a diagnostic names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineMark {
    /// The 1-based line.
    pub line: u32,
    /// The severity it is marked with.
    pub kind: MarkKind,
}

/// The id of line `line` of the editor whose text area is `id`, which a
/// diagnostic links to as `#` and this id.
#[must_use]
pub fn line_id(id: &str, line: u32) -> String {
    format!("{id}-L{line}")
}

/// The class of line `line` under `marks`: an error outranks a warning.
#[must_use]
pub fn mark_class(marks: &[LineMark], line: u32) -> Option<&'static str> {
    let mut on_line = marks.iter().filter(|mark| mark.line == line).peekable();
    on_line.peek()?;
    Some(if on_line.any(|mark| mark.kind == MarkKind::Error) {
        MarkKind::Error.class()
    } else {
        MarkKind::Warning.class()
    })
}

/// What the editor edits and shows.
#[derive(Debug, Clone, Copy)]
pub struct Editor {
    /// The text area's id and name.
    pub id: &'static str,
    /// The grammar the text is highlighted as.
    pub language: Language,
    /// The text, which the editor reads and never writes.
    pub text: Signal<String>,
    /// The lines a diagnostic names.
    pub marks: Signal<Vec<LineMark>>,
    /// False until the text area may be edited: before the page hydrates an
    /// edit would be lost.
    pub editable: Signal<bool>,
}

/// Returns the editor over `editor`; `on_input` receives the text after
/// every edit, exactly as typed.
///
/// The editor is at least 16rem tall; an ancestor with the class
/// `fui-editor-short` makes it 6rem, for a short value such as a parameter
/// object.
#[must_use]
pub fn code_editor(editor: Editor, on_input: impl Fn(String) + 'static) -> AnyView {
    let Editor {
        id,
        language,
        text,
        marks,
        editable,
    } = editor;
    drawn(
        Drawn {
            id: id.to_owned(),
            language: Grammar::Fixed(language),
            text,
            marks,
            editable,
            placeholder: None,
        },
        on_input,
    )
}

/// The grammar an editor highlights its text as: one the caller fixed, or
/// one that follows a signal the caller owns. Neither allocates a reactive
/// value, so an editor drawn inside a reloading `Transition` holds nothing
/// its caller's owner can dispose under it.
#[derive(Clone, Copy)]
pub(crate) enum Grammar {
    /// A grammar the caller fixed.
    Fixed(Language),
    /// A grammar that follows the caller's signal.
    Following(Signal<Language>),
}

impl Grammar {
    /// The grammar now, tracked when it follows a signal.
    fn get(self) -> Language {
        match self {
            Self::Fixed(language) => language,
            Self::Following(language) => language.get(),
        }
    }
}

/// What [`drawn`] draws: an [`Editor`] whose id is known at run time, whose
/// grammar may follow the text, and whose text area may say what goes in it.
pub(crate) struct Drawn {
    /// The text area's id and name.
    pub(crate) id: String,
    /// The grammar the text is highlighted as.
    pub(crate) language: Grammar,
    /// The text, which the editor reads and never writes.
    pub(crate) text: Signal<String>,
    /// The lines a diagnostic names.
    pub(crate) marks: Signal<Vec<LineMark>>,
    /// False until the text area may be edited.
    pub(crate) editable: Signal<bool>,
    /// The text area's placeholder, shown while the text is empty.
    pub(crate) placeholder: Option<std::borrow::Cow<'static, str>>,
}

/// Returns the editor `drawn`, as [`code_editor`] and the upload field draw
/// it.
pub(crate) fn drawn(drawn: Drawn, on_input: impl Fn(String) + 'static) -> AnyView {
    let Drawn {
        id,
        language,
        text,
        marks,
        editable,
        placeholder,
    } = drawn;
    let prefix = format!("{id}-L");
    let layer = move || {
        let marks = marks.get();
        let language = language.get();
        text.with(|text| {
            html(language, text, Lines { ids: Some(&prefix) }, |line| {
                mark_class(&marks, line)
            })
        })
    };
    let name = id.clone();
    // NOTE: HTML Living Standard §13.1.2.2: a text area's text is escapable raw text, so
    // the server writes it escaped, and a form posts it before any WebAssembly loads.
    let first = text.with_untracked(|text| {
        let mut out = String::with_capacity(text.len());
        crate::highlight::escape(text, &mut out);
        out
    });
    view! {
        <div class="fui-editor">
            <div class="fui-editor-stack">
                <pre class="fui-editor-layer" aria-hidden="true" inner_html=layer></pre>
                <textarea
                    id=id
                    name=name
                    class="fui-editor-input"
                    autocomplete="off"
                    autocapitalize="off"
                    spellcheck="false"
                    wrap="soft"
                    placeholder=placeholder
                    readonly=move || !editable.get()
                    inner_html=first
                    prop:value=move || text.get()
                    on:input:target=move |event| on_input(event.target().value())
                ></textarea>
            </div>
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::{LineMark, MarkKind, line_id, mark_class};

    #[test]
    fn an_error_outranks_a_warning_on_its_line() {
        let marks = [
            LineMark {
                line: 2,
                kind: MarkKind::Warning,
            },
            LineMark {
                line: 2,
                kind: MarkKind::Error,
            },
            LineMark {
                line: 5,
                kind: MarkKind::Warning,
            },
        ];
        assert_eq!(Some("fui-mark-error"), mark_class(&marks, 2));
        assert_eq!(Some("fui-mark-warning"), mark_class(&marks, 5));
        assert_eq!(None, mark_class(&marks, 1));
        assert_eq!("draft-L12", line_id("draft", 12));
    }
}
