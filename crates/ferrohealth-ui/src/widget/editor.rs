// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The highlighted editor: a text area the operator types into, over a
//! highlighted, numbered layer that draws the same text with the same
//! tokenizer a code panel uses; and the code editor (B2) around it.
//!
//! The text area keeps the browser's own editing, selection, undo and the
//! exact characters typed: its text is transparent, and only its caret and
//! selection show over the layer. Both sit in one grid cell of one scrolling
//! box, drawn with the same face, size, padding and wrapping, so the layer
//! sizes the text area and the two scroll together, and a link to a line
//! ([`line_id`]) scrolls the box to it. No script is written by hand and no
//! editor library is used.
//!
//! The server writes the text area's text into the page, escaped by
//! [`crate::highlight::escape`] (the HTML Living Standard's escapable raw
//! text, so `</textarea>` in the text stays text), and a form around the
//! editor posts it before any WebAssembly loads; once the page hydrates the
//! text area takes the live text through `prop:value`. The layer is escaped
//! by [`crate::highlight::html`], so the server draws it too. The text area
//! is read-only until the page hydrates, so no keystroke is lost.
//!
//! The code editor ([`editor_panel`]) puts F8's toolbar above the editor
//! (what the text is, its line count and the console's tools) and a status
//! line below (the cursor's line and column, the console's facts such as
//! the encoding, and whether the draft changed since it was opened, with
//! the unsaved dot). A line a diagnostic names is marked in the gutter, and
//! while the cursor is on it the line and its row of the diagnostics list
//! beside ([`editor_diagnostics`], in [`editor_split`]) are current and the
//! status line says so politely. Lines never wrap. The text area is the
//! editor's one tab stop. No specification governs the editor: our own
//! design.

use leptos::prelude::*;
use leptos::web_sys::HtmlTextAreaElement;
use serde::{Deserialize, Serialize};

use crate::highlight::{Language, Lines, html};
use crate::i18n::{Phrase, Text, tr, tr_with};
use crate::view::diagnostics::{FileFindings, Position, rows};

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

    /// The class of a line of this kind that is current too.
    const fn current_class(self) -> &'static str {
        match self {
            Self::Error => "fui-mark-error is-current",
            Self::Warning => "fui-mark-warning is-current",
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

/// The kind line `line` is marked with under `marks`: an error outranks a
/// warning.
#[must_use]
pub fn mark_kind(marks: &[LineMark], line: u32) -> Option<MarkKind> {
    let mut on_line = marks.iter().filter(|mark| mark.line == line).peekable();
    on_line.peek()?;
    Some(if on_line.any(|mark| mark.kind == MarkKind::Error) {
        MarkKind::Error
    } else {
        MarkKind::Warning
    })
}

/// The class of line `line` under `marks`: an error outranks a warning.
#[must_use]
pub fn mark_class(marks: &[LineMark], line: u32) -> Option<&'static str> {
    mark_kind(marks, line).map(MarkKind::class)
}

/// The class of line `line` under `marks` while the cursor's marked line is
/// `current`: a marked line the cursor is on is current too.
fn layer_class(marks: &[LineMark], line: u32, current: Option<u32>) -> Option<&'static str> {
    let kind = mark_kind(marks, line)?;
    Some(if current == Some(line) {
        kind.current_class()
    } else {
        kind.class()
    })
}

/// The place in `text` of the caret at `offset`, a count of UTF-16 code
/// units as a text area's `selectionStart` gives it: its line, and its
/// column in characters. An offset past the end is the end.
#[must_use]
pub fn caret_at(text: &str, offset: u32) -> Position {
    let mut at = Position::default();
    let mut units = 0_u32;
    for character in text.chars() {
        if units >= offset {
            break;
        }
        units = units.saturating_add(u32::try_from(character.len_utf16()).unwrap_or(2));
        if character == '\n' {
            at.line = at.line.saturating_add(1);
            at.column = 1;
        } else {
            at.column = at.column.saturating_add(1);
        }
    }
    at
}

/// The offset in UTF-16 code units of the place `at` in `text`, as a text
/// area's selection takes it: a column past its line's end is the line's
/// end, and a line past the last is the end of the text.
#[must_use]
pub fn offset_of(text: &str, at: Position) -> u32 {
    let mut here = Position::default();
    let mut units = 0_u32;
    for character in text.chars() {
        if here.line > at.line || (here.line == at.line && here.column >= at.column) {
            break;
        }
        if character == '\n' && here.line == at.line {
            break;
        }
        units = units.saturating_add(u32::try_from(character.len_utf16()).unwrap_or(2));
        if character == '\n' {
            here.line = here.line.saturating_add(1);
            here.column = 1;
        } else {
            here.column = here.column.saturating_add(1);
        }
    }
    units
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
            label: None,
            caret: None,
            current: Signal::stored(None),
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
    /// The text area's accessible name, when no label names it.
    pub(crate) label: Option<String>,
    /// Where the caret is, which the editor writes as it moves.
    pub(crate) caret: Option<RwSignal<Position>>,
    /// The marked line the cursor is on, which the layer marks current.
    pub(crate) current: Signal<Option<u32>>,
}

/// The text area's content as the server writes it for `text`: escaped,
/// with one more line break before a text that opens with one.
fn server_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(1));
    // NOTE: HTML Living Standard §13.2.6.4.7: the parser drops one line break right after
    // `<textarea>`, so a text that opens with one is written with two.
    if text.starts_with('\n') {
        out.push('\n');
    }
    // NOTE: HTML Living Standard §13.1.2.2: a text area's text is escapable raw text, so
    // the server writes it escaped, and a form posts it before any WebAssembly loads.
    crate::highlight::escape(text, &mut out);
    out
}

/// Returns the editor `drawn`, as [`code_editor`], [`editor_panel`] and
/// the upload field draw it.
pub(crate) fn drawn(drawn: Drawn, on_input: impl Fn(String) + 'static) -> AnyView {
    let Drawn {
        id,
        language,
        text,
        marks,
        editable,
        placeholder,
        label,
        caret,
        current,
    } = drawn;
    let prefix = format!("{id}-L");
    let layer = move || {
        let marks = marks.get();
        let current = current.get();
        let language = language.get();
        text.with(|text| {
            html(language, text, Lines { ids: Some(&prefix) }, |line| {
                layer_class(&marks, line, current)
            })
        })
    };
    let name = id.clone();
    let first = text.with_untracked(|text| server_text(text));
    let follow = move |area: &HtmlTextAreaElement| {
        let Some(caret) = caret else {
            return;
        };
        if let Ok(Some(start)) = area.selection_start() {
            caret.set(caret_at(&area.value(), start));
        }
    };
    let invalid = move || {
        marks
            .read()
            .iter()
            .any(|mark| mark.kind == MarkKind::Error)
            .then_some("true")
    };
    view! {
        <div class="fui-editor">
            <div class="fui-editor-stack">
                <pre class="fui-editor-layer" aria-hidden="true" inner_html=layer></pre>
                <textarea
                    id=id
                    name=name
                    class="fui-editor-input"
                    aria-label=label
                    aria-invalid=invalid
                    autocomplete="off"
                    autocapitalize="off"
                    spellcheck="false"
                    wrap="soft"
                    placeholder=placeholder
                    readonly=move || !editable.get()
                    inner_html=first
                    prop:value=move || text.get()
                    on:input:target=move |event| {
                        on_input(event.target().value());
                        follow(&event.target());
                    }
                    on:keyup:target=move |event| follow(&event.target())
                    on:click:target=move |event| follow(&event.target())
                    on:select:target=move |event| follow(&event.target())
                ></textarea>
            </div>
        </div>
    }
    .into_any()
}

/// The words the code editor draws itself, resolved on the server in the
/// page's locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorWords {
    /// The toolbar's count of one line, whole.
    pub one_line: String,
    /// The toolbar's count of lines other than one, around the open
    /// `count`.
    pub lines: Phrase,
    /// The cursor's place, around the open `line` and `column`.
    pub position: Phrase,
    /// The state of a file not saved yet.
    pub new_file: String,
    /// The state of a draft changed since it was opened.
    pub changed: String,
    /// The state of a draft as it was opened.
    pub unchanged: String,
    /// What the status line says on a line an error names, around the
    /// open `line`.
    pub error_on: Phrase,
    /// What the status line says on a line a warning names, around the
    /// open `line`.
    pub warning_on: Phrase,
}

impl EditorWords {
    /// The code editor's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        let line = || ("line", Phrase::open("line"));
        Self {
            one_line: tr_with(Text::CodeLines, &[("count", 1_u32.into())]),
            lines: Phrase::new(tr_with(
                Text::CodeLines,
                &[("count", Phrase::open("count"))],
            )),
            position: Phrase::new(tr_with(
                Text::EditorPosition,
                &[line(), ("column", Phrase::open("column"))],
            )),
            new_file: tr(Text::EditorNew),
            changed: tr(Text::EditorChanged),
            unchanged: tr(Text::EditorUnchanged),
            error_on: Phrase::new(tr_with(Text::EditorErrorOn, &[line()])),
            warning_on: Phrase::new(tr_with(Text::EditorWarningOn, &[line()])),
        }
    }

    /// The toolbar's line count of `count` lines.
    fn line_count(&self, count: u32) -> String {
        if count == 1 {
            self.one_line.clone()
        } else {
            self.lines.fill(&[("count", &count.to_string())])
        }
    }
}

/// What a draft is measured against: whether it changed since it was
/// opened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opened {
    /// A file not saved yet: the status line says so.
    New,
    /// The text the draft was opened with: the status line says whether
    /// the draft still holds it.
    Text(String),
}

/// What the code editor draws around its editor.
#[derive(Debug)]
pub struct Panel {
    /// The editor: its text area, text, marks and editable state.
    pub editor: Editor,
    /// The text area's accessible name, such as "The draft of
    /// `body_weight.context.yml`".
    pub label: String,
    /// What the text is, at the toolbar's start, such as "YAML · mapping
    /// context".
    pub lead: String,
    /// The facts the status line says after the cursor's place, such as
    /// "UTF-8 · LF".
    pub facts: Vec<String>,
    /// What the draft is measured against.
    pub opened: Opened,
    /// Where the caret is: the editor writes it as the cursor moves, and
    /// the diagnostics beside ([`editor_diagnostics`]) read it.
    pub caret: RwSignal<Position>,
    /// The console's tools at the toolbar's end, such as Check.
    pub tools: Option<AnyView>,
    /// What the status line shows at its end, such as the owner.
    pub end: Option<AnyView>,
    /// The words it draws itself ([`EditorWords::of_page`]).
    pub words: EditorWords,
}

/// The marked line the cursor is on: the caret's line when a diagnostic
/// names it.
#[must_use]
pub fn current_line(
    caret: RwSignal<Position>,
    marks: Signal<Vec<LineMark>>,
) -> Signal<Option<u32>> {
    Signal::derive(move || {
        let line = caret.get().line;
        marks
            .read()
            .iter()
            .any(|mark| mark.line == line)
            .then_some(line)
    })
}

/// What the status line says while the cursor is on the marked line
/// `current` under `marks`, in `words`: nothing on an unmarked line.
fn said(
    current: Signal<Option<u32>>,
    marks: Signal<Vec<LineMark>>,
    words: StoredValue<EditorWords>,
) -> Option<String> {
    let line = current.get()?;
    let kind = marks.with(|marks| mark_kind(marks, line))?;
    let line = line.to_string();
    Some(words.with_value(|words| {
        let phrase = match kind {
            MarkKind::Error => &words.error_on,
            MarkKind::Warning => &words.warning_on,
        };
        phrase.fill(&[("line", &line)])
    }))
}

/// Returns the code editor (B2) over `panel`: F8's toolbar, the editor
/// with its gutter marks, and the status line; `on_input` receives the
/// text after every edit, exactly as typed.
///
/// The editor is 24rem tall, and 6rem while it holds no text; in a panel
/// section (B10) it has no card frame and runs edge to edge.
#[must_use]
pub fn editor_panel(panel: Panel, on_input: impl Fn(String) + 'static) -> AnyView {
    let Panel {
        editor,
        label,
        lead,
        facts,
        opened,
        caret,
        tools,
        end,
        words,
    } = panel;
    let Editor {
        id,
        language,
        text,
        marks,
        editable,
    } = editor;
    let current = current_line(caret, marks);
    let words = StoredValue::new(words);
    let bar = move || {
        let count = text.with(|text| crate::view::code::line_count(text));
        let lines = words.with_value(|words| words.line_count(count));
        format!("{lead} · {lines}")
    };
    let position = move || {
        let at = caret.get();
        words.with_value(|words| {
            words.position.fill(&[
                ("line", &at.line.to_string()),
                ("column", &at.column.to_string()),
            ])
        })
    };
    let is_new = matches!(opened, Opened::New);
    let opened = StoredValue::new(opened);
    let dirty = move || {
        opened.with_value(|opened| match opened {
            Opened::New => false,
            Opened::Text(first) => text.with(|text| text != first),
        })
    };
    let state = move || {
        words.with_value(|words| match (is_new, dirty()) {
            (true, _) => words.new_file.clone(),
            (false, true) => words.changed.clone(),
            (false, false) => words.unchanged.clone(),
        })
    };
    let said = move || said(current, marks, words);
    let facts = facts
        .into_iter()
        .map(|fact| view! { <span>{fact}</span> })
        .collect_view();
    let end = end.map(|end| view! { <span class="fui-editor-status-end">{end}</span> });
    let field = drawn(
        Drawn {
            id: id.to_owned(),
            language: Grammar::Fixed(language),
            text,
            marks,
            editable,
            placeholder: None,
            label: Some(label),
            caret: Some(caret),
            current,
        },
        on_input,
    );
    view! {
        <section
            class="fui-card fui-panel fui-mapping-editor"
            class:is-empty=move || text.with(|text| text.trim().is_empty())
        >
            <div class="fui-editor-bar">
                <span>{bar}</span>
                <div class="fui-actions">{tools}</div>
            </div>
            {field}
            <div class="fui-editor-status">
                <span>{position}</span>
                {facts}
                <span class:fui-editor-dirty=dirty>{state}</span>
                <span class="fui-sr-only" role="status">
                    {said}
                </span>
                {end}
            </div>
        </section>
    }
    .into_any()
}

/// Returns the diagnostics list (B3) of the file the code editor `editor`
/// edits, named `label`, in step with the cursor at `caret`.
///
/// The row of the marked line the cursor is on is current; following a
/// row's line link moves the caret to the finding's line and column and
/// gives the editor the focus.
#[must_use]
pub fn editor_diagnostics(
    label: &str,
    file: &FileFindings,
    editor: Editor,
    caret: RwSignal<Position>,
) -> AnyView {
    let Editor {
        id, text, marks, ..
    } = editor;
    let current = current_line(caret, marks);
    let on_line = Callback::new(move |at: Position| {
        caret.set(at);
        place_caret(id, text, at);
    });
    let rows = rows(file, current, Some(on_line));
    view! {
        <ul class="fui-diags" aria-label=label.to_owned()>
            {rows}
        </ul>
    }
    .into_any()
}

/// Moves the caret of the text area `id`, which holds `text`, to `at` and
/// gives it the focus once the link followed has scrolled to the line.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        unused_variables,
        reason = "the caret moves only in the browser half; the server half has no text area"
    )
)]
fn place_caret(id: &'static str, text: Signal<String>, at: Position) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let Some(area) = document()
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<HtmlTextAreaElement>().ok())
        else {
            return;
        };
        let offset = text.with_untracked(|text| offset_of(text, at));
        // NOTE: HTML Living Standard, "scroll to the fragment": following the link moves the focus to its
        // fragment after this handler, so the editor takes the focus a task later.
        set_timeout(
            move || {
                if area.set_selection_range(offset, offset).is_err() {
                    leptos::logging::warn!("the browser refused the editor's caret");
                }
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                if area.focus_with_options(&options).is_err() {
                    leptos::logging::warn!("the browser refused the editor the focus");
                }
            },
            std::time::Duration::ZERO,
        );
    }
}

/// Returns the code editor `editor` beside `side`, its diagnostics, at
/// wide widths, and over them under 64rem.
#[must_use]
pub fn editor_split(editor: AnyView, side: AnyView) -> AnyView {
    view! { <div class="fui-editor-split">{editor} {side}</div> }.into_any()
}

#[cfg(test)]
mod tests {
    use super::{
        LineMark, MarkKind, caret_at, layer_class, line_id, mark_class, offset_of, server_text,
    };
    use crate::view::diagnostics::Position;

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

    #[test]
    fn the_server_writes_the_text_escaped_and_keeps_a_first_line_break() {
        assert_eq!("a &lt;/textarea&gt; &amp;", server_text("a </textarea> &"));
        assert_eq!("\n\nb", server_text("\nb"));
        assert_eq!("", server_text(""));
    }

    #[test]
    fn a_marked_line_the_cursor_is_on_is_current_and_keeps_its_mark() {
        let marks = [LineMark {
            line: 4,
            kind: MarkKind::Warning,
        }];
        assert_eq!(
            Some("fui-mark-warning is-current"),
            layer_class(&marks, 4, Some(4))
        );
        assert_eq!(Some("fui-mark-warning"), layer_class(&marks, 4, Some(7)));
        assert_eq!(None, layer_class(&marks, 7, Some(7)));
    }

    #[test]
    fn the_caret_reads_as_a_line_and_a_column_of_characters() {
        let at = |line, column| Position { line, column };
        let text = "ab\nnaïve ✓ 𝄞x\n";
        assert_eq!(at(1, 1), caret_at(text, 0));
        assert_eq!(at(1, 3), caret_at(text, 2));
        assert_eq!(at(2, 1), caret_at(text, 3));
        // The clef is two UTF-16 code units and one character.
        assert_eq!(at(2, 10), caret_at(text, 13));
        assert_eq!(at(2, 11), caret_at(text, 14));
        assert_eq!(at(3, 1), caret_at(text, 99));
    }

    #[test]
    fn a_place_reads_back_as_the_offset_it_came_from() {
        let text = "ab\nnaïve ✓ 𝄞x\nlast";
        for offset in [0, 1, 2, 3, 6, 11, 13, 14, 15, 16, 19] {
            assert_eq!(
                offset,
                offset_of(text, caret_at(text, offset)),
                "offset {offset}"
            );
        }
        let at = |line, column| Position { line, column };
        assert_eq!(2, offset_of(text, at(1, 40)), "past the line is its end");
        assert_eq!(19, offset_of(text, at(9, 1)), "past the text is its end");
    }
}
