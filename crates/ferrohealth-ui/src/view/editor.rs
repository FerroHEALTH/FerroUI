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
//! ([`line_id`]) scrolls the box to it.
//!
//! The server writes the text area's text into the page, escaped by
//! [`crate::highlight::escape`] (the HTML Living Standard's escapable raw
//! text, so `</textarea>` in the text stays text), and the layer as
//! [`crate::highlight::html`] escapes it, so a form around the editor posts
//! the text with no script and the reader sees it highlighted. With no
//! script the text area shows its own text over the layer's gutter. The
//! `editor` module highlights each edit with the same tokenizers written
//! again in JavaScript, redrawing only the lines whose highlighting or mark
//! changed.
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

use std::borrow::Cow;
use std::collections::BTreeMap;

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::highlight::{Language, escape, lines_html, push_line};
use crate::html::{Html, Render as _};
use crate::i18n::number::Grouping;
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
use crate::view::diagnostics::{DiagnosticsWords, FileFindings, Position, rows};

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

/// The kind of each line `marks` names: an error outranks a warning.
fn mark_index(marks: &[LineMark]) -> BTreeMap<u32, MarkKind> {
    let mut out = BTreeMap::new();
    for mark in marks {
        out.entry(mark.line)
            .and_modify(|kind| {
                if mark.kind == MarkKind::Error {
                    *kind = MarkKind::Error;
                }
            })
            .or_insert(mark.kind);
    }
    out
}

/// The class of a line marked `kind`, if any, while the cursor's marked
/// line is `current`: a marked line the cursor is on is current too.
fn layer_class(kind: Option<MarkKind>, line: u32, current: Option<u32>) -> Option<&'static str> {
    let kind = kind?;
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

/// The layer of `text` in `language` as the server draws it: one line
/// element per line, each with its id after `prefix` and the class its
/// mark gives it while the cursor's marked line is `current`.
fn layer(
    language: Language,
    text: &str,
    marks: &[LineMark],
    current: Option<u32>,
    prefix: &str,
) -> String {
    let kinds = mark_index(marks);
    let mut out = String::new();
    for (line, body) in (1_u32..).zip(lines_html(language, text)) {
        let class = layer_class(kinds.get(&line).copied(), line, current);
        push_line(&mut out, line, class, Some(prefix), &body);
    }
    out
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
    // the server writes it escaped, and a form posts it with no script.
    escape(text, &mut out);
    out
}

/// The key the `editor` module highlights a grammar by.
const fn grammar_key(language: Language) -> &'static str {
    match language {
        Language::Yaml => "yaml",
        Language::Json => "json",
        Language::Xml => "xml",
        Language::Aql => "aql",
        Language::Hl7v2 => "hl7v2",
        Language::Plain => "plain",
    }
}

/// What the editor edits and shows.
#[derive(Debug, Clone)]
pub struct Editor {
    /// The text area's id and name: a static id, or one made at run time,
    /// such as one per staged change of a page that holds several.
    pub id: Cow<'static, str>,
    /// The grammar the text is highlighted as.
    pub language: Language,
    /// The text.
    pub text: String,
    /// The lines a diagnostic names.
    pub marks: Vec<LineMark>,
    /// Whether the text may be edited; false for a text the reader may
    /// only read.
    pub editable: bool,
}

/// What [`drawn`] draws: an [`Editor`] whose text area may say what goes
/// in it, and the marked line the cursor is on.
pub(crate) struct Drawn<'a> {
    /// The editor.
    pub(crate) editor: &'a Editor,
    /// The text area's placeholder, if any.
    pub(crate) placeholder: Option<String>,
    /// The text area's accessible name, if any.
    pub(crate) label: Option<String>,
    /// The marked line the cursor is on.
    pub(crate) current: Option<u32>,
    /// Whether the grammar follows the text as it changes
    /// ([`Language::detect`]), as the upload field's does.
    pub(crate) detect: bool,
}

#[derive(Template)]
#[template(path = "editor/editor.html", block = "field")]
struct FieldView<'a> {
    language: &'static str,
    detect: bool,
    marks: String,
    current: Option<u32>,
    layer: Html,
    id: &'a str,
    label: Option<String>,
    invalid: bool,
    placeholder: Option<String>,
    editable: bool,
    text: Html,
}

/// Returns the editor `drawn`, as [`code_editor`], [`editor_panel`] and the
/// upload field draw it.
pub(crate) fn drawn(drawn: Drawn<'_>) -> Html {
    let Drawn {
        editor,
        placeholder,
        label,
        current,
        detect,
    } = drawn;
    let prefix = format!("{}-L", editor.id);
    FieldView {
        language: grammar_key(editor.language),
        detect,
        // NOTE: serde_json `to_string` fails only for a map with a key that is no
        // string, and a list of marks holds none.
        marks: serde_json::to_string(&editor.marks).unwrap_or_default(),
        current,
        layer: Html::raw(layer(
            editor.language,
            &editor.text,
            &editor.marks,
            current,
            &prefix,
        )),
        id: &editor.id,
        label,
        invalid: editor.marks.iter().any(|mark| mark.kind == MarkKind::Error),
        placeholder,
        editable: editor.editable,
        text: Html::raw(server_text(&editor.text)),
    }
    .html()
}

/// Returns the editor over `editor`: the text area over its highlighted,
/// numbered layer, which a form around it posts as it stands.
///
/// The editor is at least 16rem tall; an ancestor with the class
/// `fui-editor-short` makes it 6rem, for a short value such as a parameter
/// object.
#[must_use]
pub fn code_editor(editor: &Editor) -> Html {
    drawn(Drawn {
        editor,
        placeholder: None,
        label: None,
        current: None,
        detect: false,
    })
}

/// The words the code editor draws itself, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorWords {
    /// The toolbar's count of one line, whole.
    pub one_line: Word,
    /// The toolbar's count of lines other than one, around the open
    /// `count`.
    pub lines: Phrase,
    /// The cursor's place, around the open `line` and `column`.
    pub position: Phrase,
    /// The state of a file not saved yet.
    pub new_file: Word,
    /// The state of a draft changed since it was opened.
    pub changed: Word,
    /// The state of a draft as it was opened.
    pub unchanged: Word,
    /// What the status line says on a line an error names, around the
    /// open `line`.
    pub error_on: Phrase,
    /// What the status line says on a line a warning names, around the
    /// open `line`.
    pub warning_on: Phrase,
    /// The mark the line count and the cursor's place are grouped by.
    pub grouping: Grouping,
}

impl EditorWords {
    /// The code editor's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        let line = || ("line", Phrase::open("line"));
        Self {
            one_line: tr_with(locale, Text::CodeLines, &[("count", 1_u32.into())]),
            lines: Phrase::new(tr_with(
                locale,
                Text::CodeLines,
                &[("count", Phrase::open("count"))],
            )),
            position: Phrase::new(tr_with(
                locale,
                Text::EditorPosition,
                &[line(), ("column", Phrase::open("column"))],
            )),
            new_file: tr(locale, Text::EditorNew),
            changed: tr(locale, Text::EditorChanged),
            unchanged: tr(locale, Text::EditorUnchanged),
            error_on: Phrase::new(tr_with(locale, Text::EditorErrorOn, &[line()])),
            warning_on: Phrase::new(tr_with(locale, Text::EditorWarningOn, &[line()])),
            grouping: Grouping::of(locale),
        }
    }

    /// The toolbar's line count of `count` lines, its digits grouped as
    /// the page's language groups them.
    fn line_count(&self, count: u32) -> Word {
        if count == 1 {
            self.one_line.clone()
        } else {
            self.lines.fill(&[("count", &self.number(count))])
        }
    }

    /// The number `number` with its digits grouped.
    fn number(&self, number: u32) -> String {
        self.grouping.write(u64::from(number))
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
    /// Where the caret is as the page arrives, such as the place of the
    /// finding a link chose; the `editor` module moves it with the cursor.
    pub caret: Position,
    /// The console's tools at the toolbar's end, such as Check.
    pub tools: Option<Html>,
    /// What the status line shows at its end, such as the owner.
    pub end: Option<Html>,
    /// The words it draws itself ([`EditorWords::of`]).
    pub words: EditorWords,
}

/// The marked line the cursor at `caret` is on: the caret's line when a
/// diagnostic names it.
#[must_use]
pub fn current_line(caret: Position, marks: &[LineMark]) -> Option<u32> {
    marks
        .iter()
        .any(|mark| mark.line == caret.line)
        .then_some(caret.line)
}

/// What the status line says while the cursor is on the marked line
/// `current` under `marks`, in `words`: nothing on an unmarked line.
fn said(current: Option<u32>, marks: &[LineMark], words: &EditorWords) -> Option<Word> {
    let line = current?;
    let kind = mark_kind(marks, line)?;
    let phrase = match kind {
        MarkKind::Error => &words.error_on,
        MarkKind::Warning => &words.warning_on,
    };
    Some(phrase.fill(&[("line", &words.number(line))]))
}

#[derive(Template)]
#[template(path = "editor/editor.html", block = "panel")]
struct PanelView {
    words: String,
    is_new: bool,
    opened: Option<String>,
    blank: bool,
    lead: String,
    bar: Html,
    tools: Html,
    field: Html,
    position: Word,
    facts: Vec<String>,
    dirty: bool,
    state: Word,
    said: Option<Word>,
    end: Option<Html>,
}

/// Returns the code editor (B2) over `panel`: F8's toolbar, the editor
/// with its gutter marks, and the status line.
///
/// The editor is 24rem tall, and 6rem while it holds no text; in a panel
/// section (B10) it has no card frame and runs edge to edge. The toolbar's
/// line count, the cursor's place and the status line's changed state
/// follow each edit once the `editor` module runs.
#[must_use]
pub fn editor_panel(panel: Panel) -> Html {
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
    let current = current_line(caret, &editor.marks);
    let count = crate::view::code::line_count(&editor.text);
    let lines = words.line_count(count);
    let bar = Html::raw(format!("{} · {lines}", Html::text(&lead)));
    let (is_new, dirty, opened_text) = match opened {
        Opened::New => (true, false, None),
        Opened::Text(opened) => {
            let dirty = opened != editor.text;
            (false, dirty, dirty.then_some(opened))
        }
    };
    let state = match (is_new, dirty) {
        (true, _) => words.new_file.clone(),
        (false, true) => words.changed.clone(),
        (false, false) => words.unchanged.clone(),
    };
    let position = words.position.fill(&[
        ("line", &words.number(caret.line)),
        ("column", &words.number(caret.column)),
    ]);
    let field = drawn(Drawn {
        editor: &editor,
        placeholder: None,
        label: Some(label),
        current,
        detect: false,
    });
    PanelView {
        // NOTE: serde_json `to_string` fails only for a map with a key that is no
        // string, and the words hold none.
        words: serde_json::to_string(&words).unwrap_or_default(),
        is_new,
        opened: opened_text,
        blank: editor.text.trim().is_empty(),
        lead,
        bar,
        tools: tools.unwrap_or_default(),
        field,
        position,
        facts,
        dirty,
        state,
        said: said(current, &editor.marks, &words),
        end,
    }
    .html()
}

/// Returns the diagnostics list (B3) of the file the code editor `editor`
/// edits, named `label`, its rows in `words`, the row of the marked line
/// the cursor at `caret` is on current.
///
/// The `editor` module keeps the current row in step with the cursor, and
/// following a row's line link moves the caret to the finding's line and
/// column and gives the editor the focus.
#[must_use]
pub fn editor_diagnostics(
    label: &str,
    file: &FileFindings,
    words: &DiagnosticsWords,
    editor: &Editor,
    caret: Position,
) -> Html {
    let current = current_line(caret, &editor.marks);
    Html::raw(format!(
        "<ul aria-label=\"{}\" data-fui-for=\"{}\" class=\"fui-diags\">{}</ul>",
        Html::text(label),
        Html::text(&editor.id),
        rows(file, words, current)
    ))
}

/// Returns the code editor `editor` beside `side`, its diagnostics, at
/// wide widths, and over them under 64rem.
#[must_use]
pub fn editor_split(editor: Html, side: Html) -> Html {
    let (editor, side) = (String::from(editor), String::from(side));
    Html::raw(format!(
        "<div class=\"fui-editor-split\">{editor}{side}</div>"
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        Editor, LineMark, MarkKind, caret_at, current_line, layer, layer_class, line_id,
        mark_class, mark_index, offset_of, server_text,
    };
    use crate::highlight::{Language, Lines, html};
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
        let marks = mark_index(&[LineMark {
            line: 4,
            kind: MarkKind::Warning,
        }]);
        let kind = |line| marks.get(&line).copied();
        assert_eq!(
            Some("fui-mark-warning is-current"),
            layer_class(kind(4), 4, Some(4))
        );
        assert_eq!(Some("fui-mark-warning"), layer_class(kind(4), 4, Some(7)));
        assert_eq!(None, layer_class(kind(7), 7, Some(7)));
        let marked = [LineMark {
            line: 4,
            kind: MarkKind::Warning,
        }];
        assert_eq!(
            Some(4),
            current_line(Position { line: 4, column: 2 }, &marked)
        );
        assert_eq!(None, current_line(Position { line: 5, column: 1 }, &marked));
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

    #[test]
    fn the_layer_draws_the_lines_the_highlighter_draws() {
        let text = "a: 1\n# b <c> & 'd'\n";
        let marks = [LineMark {
            line: 2,
            kind: MarkKind::Error,
        }];
        let drawn = layer(Language::Yaml, text, &marks, None, "draft-L");
        let expected = html(
            Language::Yaml,
            text,
            Lines {
                ids: Some("draft-L"),
            },
            |line| mark_class(&marks, line),
        );
        assert_eq!(expected, drawn);
        let editor = Editor {
            id: "draft".into(),
            language: Language::Yaml,
            text: String::from(text),
            marks: marks.to_vec(),
            editable: true,
        };
        let page = super::code_editor(&editor);
        assert!(page.as_str().contains(r#"aria-invalid="true""#), "{page}");
        assert!(page.as_str().contains(r#"data-language="yaml""#), "{page}");
    }
}
