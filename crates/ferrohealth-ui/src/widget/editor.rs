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
//! text area is the source of the live text. The console's text comes back
//! into it only as an outside change: a text the editor handed to the
//! console coming back is an echo and never written, so a key typed before
//! the echo of an earlier one arrives is never overwritten. The layer is escaped
//! as [`crate::highlight::html`] escapes it, so the server draws it too. The
//! text area takes keys from the first paint, with no WebAssembly too: until
//! its island runs it shows its own text over the layer's gutter, and what
//! it holds as the page hydrates goes to the console, so no keystroke is lost.
//! A console that keeps its own copy of the text takes each edit instead of
//! the whole text ([`code_editor_by_edit`], [`editor_panel_by_edit`]).
//! After an edit the text is tokenized once, in time linear in its length,
//! and the layer redraws only the lines whose highlighting or mark changed:
//! the lines an edit leaves alike keep their elements, and the lines after
//! an added or removed line change only their ids.
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
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};
use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::ops::Range;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::web_sys::HtmlTextAreaElement;
use serde::{Deserialize, Serialize};

use crate::highlight::{Language, MAX_HIGHLIGHT_BYTES, escape, lines_html, push_line};
use crate::i18n::number::Grouping;
use crate::i18n::{Phrase, Text, Word, tr, tr_with};
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

/// One line of the highlighted layer: the class its mark gives it and its
/// highlighted, escaped HTML.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LayerLine {
    /// The mark's class, if a diagnostic names the line.
    class: Option<&'static str>,
    /// The line's tokens as HTML, shared with the next layer while the
    /// text stays the same.
    body: Rc<str>,
}

/// The lines of the layer of `text` in `language`, marked by `marks`
/// while the cursor's marked line is `current`.
fn layer_lines(
    language: Language,
    text: &str,
    marks: &[LineMark],
    current: Option<u32>,
) -> Vec<LayerLine> {
    let kinds = mark_index(marks);
    (1_u32..)
        .zip(lines_html(language, text))
        .map(|(line, body)| LayerLine {
            class: layer_class(kinds.get(&line).copied(), line, current),
            body: Rc::from(body),
        })
        .collect()
}

/// The 1-based number of the line at the 0-based index `at`.
fn number(at: usize) -> u32 {
    u32::try_from(at).map_or(u32::MAX, |at| at.saturating_add(1))
}

/// The length of `text` in UTF-16 code units, as a text area counts it.
fn units(text: &str) -> u32 {
    u32::try_from(text.encode_utf16().count()).unwrap_or(u32::MAX)
}

/// Where a line of a text starts: its byte offset, and its offset in UTF-16
/// code units, as a text area's selection counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Start {
    /// The offset in bytes.
    byte: usize,
    /// The offset in UTF-16 code units.
    unit: u32,
}

/// The starts of the lines that follow each line break of `part`, a part of
/// a text that begins at `at`.
fn breaks(part: &str, at: Start) -> Vec<Start> {
    let mut out = Vec::new();
    let mut unit = at.unit;
    for (index, character) in part.char_indices() {
        unit = unit.saturating_add(u32::try_from(character.len_utf16()).unwrap_or(2));
        if character == '\n' {
            out.push(Start {
                byte: at.byte.saturating_add(index).saturating_add(1),
                unit,
            });
        }
    }
    out
}

/// How many bytes `old` and `new` share at their start.
fn shared_head(old: &[u8], new: &[u8]) -> usize {
    // NOTE: no specification governs this: our own design; whole blocks compare as one
    // `memcmp`, so the bytes an edit leaves alike cost no loop of their own (#458).
    const BLOCK: usize = 64;
    let mut at = 0_usize;
    while let (Some(left), Some(right)) = (
        old.get(at..at.saturating_add(BLOCK)),
        new.get(at..at.saturating_add(BLOCK)),
    ) && left == right
    {
        at = at.saturating_add(BLOCK);
    }
    let rest = old
        .get(at..)
        .unwrap_or_default()
        .iter()
        .zip(new.get(at..).unwrap_or_default())
        .take_while(|(left, right)| left == right)
        .count();
    at.saturating_add(rest)
}

/// How many bytes `old` and `new` share at their end.
fn shared_tail(old: &[u8], new: &[u8]) -> usize {
    /// The block of `bytes` that ends `at` bytes before their end.
    fn block(bytes: &[u8], at: usize) -> Option<&[u8]> {
        let end = bytes.len().checked_sub(at)?;
        bytes.get(end.checked_sub(BLOCK)?..end)
    }
    /// The bytes of `bytes` before their last `at`.
    fn before(bytes: &[u8], at: usize) -> &[u8] {
        bytes
            .get(..bytes.len().saturating_sub(at))
            .unwrap_or_default()
    }
    const BLOCK: usize = 64;
    let mut at = 0_usize;
    while let (Some(left), Some(right)) = (block(old, at), block(new, at))
        && left == right
    {
        at = at.saturating_add(BLOCK);
    }
    let rest = before(old, at)
        .iter()
        .rev()
        .zip(before(new, at).iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    at.saturating_add(rest)
}

/// What an edit replaced: the old lines from the 0-based `first`, `removed`
/// of them, became `inserted` new lines, and the old bytes
/// `start..old_end` became the new bytes `start..new_end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Edit {
    /// The first line the edit touched.
    first: usize,
    /// The old lines it replaced, from `first`.
    removed: usize,
    /// The new lines in their place.
    inserted: usize,
    /// The first byte it changed, on a character boundary.
    start: usize,
    /// The end of the bytes it replaced in the old text.
    old_end: usize,
    /// The end of the bytes it put in their place in the new text.
    new_end: usize,
}

/// A text and where each of its lines starts, which an edit brings up to
/// date by the part it changed: the text is compared in blocks, and only
/// the changed part is read character by character.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Indexed {
    /// The text.
    text: String,
    /// Where each line starts, the first at 0.
    starts: Vec<Start>,
}

impl Indexed {
    /// The text `text`, its lines indexed.
    fn of(text: &str) -> Self {
        let first = Start { byte: 0, unit: 0 };
        let mut starts = vec![first];
        starts.extend(breaks(text, first));
        Self {
            text: text.to_owned(),
            starts,
        }
    }

    /// The 0-based line the byte `byte` is on.
    fn line_of(&self, byte: usize) -> usize {
        self.starts
            .partition_point(|start| start.byte <= byte)
            .saturating_sub(1)
    }

    /// The text of the 0-based line `line`, without its line break.
    fn line(&self, line: usize) -> &str {
        let start = self
            .starts
            .get(line)
            .map_or(self.text.len(), |start| start.byte);
        let end = self
            .starts
            .get(line.saturating_add(1))
            .map_or(self.text.len(), |next| next.byte.saturating_sub(1));
        self.text.get(start..end).unwrap_or_default()
    }

    /// The length of the text in UTF-16 code units.
    fn len_units(&self) -> u32 {
        let last = self.starts.len().saturating_sub(1);
        let start = self.starts.last().map_or(0, |start| start.unit);
        start.saturating_add(units(self.line(last)))
    }

    /// The place of the caret at the UTF-16 offset `offset`, as
    /// [`caret_at`] reads it, found by its line alone.
    fn caret(&self, offset: u32) -> Position {
        let line = self
            .starts
            .partition_point(|start| start.unit <= offset)
            .saturating_sub(1);
        let start = self.starts.get(line).map_or(0, |start| start.unit);
        let within = caret_at(self.line(line), offset.saturating_sub(start));
        Position {
            line: number(line),
            column: within.column,
        }
    }

    /// Brings the text to `new` and returns the lines that changed, or
    /// `None` when `new` is the text already.
    fn edit(&mut self, new: &str) -> Option<Edit> {
        let old = self.text.as_str();
        if old == new {
            return None;
        }
        let mut head = shared_head(old.as_bytes(), new.as_bytes());
        while !old.is_char_boundary(head) || !new.is_char_boundary(head) {
            head = head.saturating_sub(1);
        }
        let room = old.len().min(new.len()).saturating_sub(head);
        let mut tail = shared_tail(old.as_bytes(), new.as_bytes()).min(room);
        while !old.is_char_boundary(old.len().saturating_sub(tail))
            || !new.is_char_boundary(new.len().saturating_sub(tail))
        {
            tail = tail.saturating_sub(1);
        }
        let old_end = old.len().saturating_sub(tail);
        let new_end = new.len().saturating_sub(tail);
        let (Some(old_part), Some(new_part)) = (old.get(head..old_end), new.get(head..new_end))
        else {
            let old_end = old.len();
            *self = Self::of(new);
            return Some(Edit {
                first: 0,
                removed: usize::MAX,
                inserted: self.starts.len(),
                start: 0,
                old_end,
                new_end: new.len(),
            });
        };
        let first = self.line_of(head);
        let last = self.line_of(old_end);
        let line_start = self
            .starts
            .get(first)
            .copied()
            .unwrap_or(Start { byte: 0, unit: 0 });
        let at = Start {
            byte: head,
            unit: line_start
                .unit
                .saturating_add(units(old.get(line_start.byte..head).unwrap_or_default())),
        };
        let old_end_unit = at.unit.saturating_add(units(old_part));
        let new_end_unit = at.unit.saturating_add(units(new_part));
        let added = breaks(new_part, at);
        let inserted = added.len().saturating_add(1);
        let mut rest = self
            .starts
            .split_off(last.saturating_add(1).min(self.starts.len()));
        for start in &mut rest {
            start.byte = start.byte.saturating_sub(old_end).saturating_add(new_end);
            start.unit = start
                .unit
                .saturating_sub(old_end_unit)
                .saturating_add(new_end_unit);
        }
        self.starts.truncate(first.saturating_add(1));
        self.starts.extend(added);
        self.starts.extend(rest);
        self.text.replace_range(head..old_end, new_part);
        Some(Edit {
            first,
            removed: last.saturating_sub(first).saturating_add(1),
            inserted,
            start: head,
            old_end,
            new_end,
        })
    }
}

/// One edit the reader made to an editor's text: the bytes `range` of the
/// text before the edit became `text`.
///
/// The offsets count UTF-8 bytes and fall on character boundaries, so a
/// console that keeps the text as a `String` brings it up to date with
/// [`TextEdit::apply`], at the cost of the edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    /// The bytes of the text before the edit that it replaced.
    pub range: Range<usize>,
    /// What took their place.
    pub text: String,
}

impl TextEdit {
    /// The edit `edit` that brought a text to `new`.
    fn of(edit: Edit, new: &str) -> Self {
        Self {
            range: edit.start..edit.old_end,
            text: new
                .get(edit.start..edit.new_end)
                .unwrap_or_default()
                .to_owned(),
        }
    }

    /// Applies the edit to `text`, the text before it, and returns whether
    /// it could: when `range` is no range of `text` on character
    /// boundaries, as when `text` is not the editor's text before the edit,
    /// it leaves `text` as it was and returns `false`.
    #[must_use]
    pub fn apply(&self, text: &mut String) -> bool {
        if text.get(self.range.clone()).is_none() {
            return false;
        }
        text.replace_range(self.range.clone(), &self.text);
        true
    }
}

/// Where a text differs from the text it was opened with: before the byte
/// `head` and in the last `tail` bytes the two agree, so an edit compares
/// only the bytes between, and never the whole text.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Since {
    /// The text it was opened with.
    opened: String,
    /// How many bytes the two share at their start, at least.
    head: usize,
    /// How many bytes the two share at their end, at least.
    tail: usize,
    /// Whether the text differs from the opened one.
    changed: bool,
}

impl Since {
    /// How `text` differs from `opened`, measured once.
    fn of(opened: String, text: &str) -> Self {
        let (new, old) = (text.as_bytes(), opened.as_bytes());
        let head = shared_head(new, old);
        let room = new.len().min(old.len()).saturating_sub(head);
        let tail = shared_tail(new, old).min(room);
        let changed = new.len() != old.len() || head.saturating_add(tail) < new.len();
        Self {
            opened,
            head,
            tail,
            changed,
        }
    }

    /// Takes `edit`, which brought a text `old_len` bytes long to `text`:
    /// the bytes before it and after it still agree with the opened text
    /// where they did, and only the bytes between are compared.
    fn edit(&mut self, edit: &Edit, old_len: usize, text: &str) {
        self.head = self.head.min(edit.start);
        self.tail = self.tail.min(old_len.saturating_sub(edit.old_end));
        let (new, old) = (text.as_bytes(), self.opened.as_bytes());
        if new.len() != old.len() {
            self.changed = true;
            return;
        }
        let end = new.len().saturating_sub(self.tail).max(self.head);
        let (Some(left), Some(right)) = (new.get(self.head..end), old.get(self.head..end)) else {
            *self = Self::of(std::mem::take(&mut self.opened), text);
            return;
        };
        let head = shared_head(left, right);
        let room = left.len().saturating_sub(head);
        let tail = shared_tail(left, right).min(room);
        self.head = self.head.saturating_add(head);
        self.tail = self.tail.saturating_add(tail);
        self.changed = self.head.saturating_add(self.tail) < new.len();
    }
}

/// What the code editor's panel shows of its text: the toolbar's line
/// count, whether the status line says it changed, and whether the panel
/// is drawn empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Measure {
    /// How many lines the text has, as [`crate::view::code::line_count`]
    /// counts them.
    lines: u32,
    /// Whether it differs from the text it was opened with.
    changed: bool,
    /// Whether it holds nothing but white space.
    blank: bool,
}

/// What a panel keeps measured of its editor's text, edit by edit.
#[derive(Debug)]
struct Measured {
    /// How the text differs from the opened one; `None` for a new file.
    since: Option<Since>,
    /// The measure the panel draws.
    measure: RwSignal<Measure>,
}

/// The editor's own record of its text area's text: its lines indexed and,
/// in a panel, what the panel shows of it, each brought up to date by an
/// edit at the cost of the edit.
#[derive(Debug)]
pub(crate) struct Typed {
    /// The text, its lines indexed.
    index: Indexed,
    /// What the panel shows of it, when the editor is in one.
    measured: Option<Measured>,
}

impl Typed {
    /// The record of `text`.
    fn of(text: &str) -> Self {
        Self {
            index: Indexed::of(text),
            measured: None,
        }
    }

    /// The record of `text` in a panel that measures it against `opened`,
    /// and the measure it keeps.
    fn measured(text: &str, opened: &Opened) -> (Self, RwSignal<Measure>) {
        let index = Indexed::of(text);
        let since = match opened {
            Opened::New => None,
            Opened::Text(first) => Some(Since::of(first.clone(), text)),
        };
        let measure = RwSignal::new(Self::measure(&index, since.as_ref()));
        let typed = Self {
            index,
            measured: Some(Measured { since, measure }),
        };
        (typed, measure)
    }

    /// What a panel shows of the text `index` holds, measured by `since`.
    fn measure(index: &Indexed, since: Option<&Since>) -> Measure {
        let breaks = index.starts.len().saturating_sub(1);
        let trailing = usize::from(!index.text.is_empty() && index.text.ends_with('\n'));
        Measure {
            lines: number(breaks.saturating_sub(trailing)),
            changed: since.is_some_and(|since| since.changed),
            blank: index.text.trim().is_empty(),
        }
    }

    /// Brings the record to `new` and returns the edit, or `None` when
    /// `new` is the text already.
    fn edit(&mut self, new: &str) -> Option<Edit> {
        let old_len = self.index.text.len();
        let edit = self.index.edit(new)?;
        if let Some(measured) = &mut self.measured {
            if let Some(since) = &mut measured.since {
                since.edit(&edit, old_len, &self.index.text);
            }
            let now = Self::measure(&self.index, measured.since.as_ref());
            if measured.measure.get_untracked() != now {
                measured.measure.set(now);
            }
        }
        Some(edit)
    }
}

/// What the console hears of the reader's typing.
#[derive(Clone)]
pub(crate) enum Heard {
    /// The whole text after every edit.
    Text(Rc<dyn Fn(String)>),
    /// Each edit, by itself.
    Edit(Rc<dyn Fn(TextEdit)>),
}

/// The editor's record of its text area's text, and whom an edit the
/// reader makes is handed to.
struct Record {
    /// The record.
    typed: RefCell<Typed>,
    /// The console's edit callback, when it takes edits.
    on_edit: Option<Rc<dyn Fn(TextEdit)>>,
}

impl Record {
    /// The record an editor keeps of `text`: `typed`, a panel's, or one of
    /// its own when it follows the caret (`caret`) or `heard` takes edits;
    /// `None` when nothing reads one.
    fn kept(
        typed: Option<Typed>,
        caret: bool,
        heard: &Heard,
        text: Signal<String>,
    ) -> Option<Rc<Self>> {
        let on_edit = match heard {
            Heard::Text(_) => None,
            Heard::Edit(on_edit) => Some(Rc::clone(on_edit)),
        };
        // NOTE: no specification governs this: our own design; the caret, the panel's measure and
        // the console's edits are read from an index of the text area's lines, so a key costs its
        // edit, not the text (#458, #477, #478).
        let typed = typed.or_else(|| {
            (caret || on_edit.is_some()).then(|| text.with_untracked(|text| Typed::of(text)))
        })?;
        Some(Rc::new(Self {
            typed: RefCell::new(typed),
            on_edit,
        }))
    }

    /// Brings the record to `new`; an edit the reader made (`by_reader`)
    /// goes to the console when it takes edits.
    fn follow(&self, new: &str, by_reader: bool) {
        let edit = self.typed.borrow_mut().edit(new);
        if let (Some(edit), true, Some(on_edit)) = (edit, by_reader, &self.on_edit) {
            on_edit(TextEdit::of(edit, new));
        }
    }
}

/// Whether the grammar `language` reads each line by itself, so a line
/// highlighted alone is highlighted as the whole text would highlight it.
#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(dead_code, reason = "the layer is patched only in the browser half")
)]
const fn reads_lines_alone(language: Language) -> bool {
    matches!(language, Language::Yaml | Language::Plain)
}

/// How the layer changed, for the browser half to draw.
#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(dead_code, reason = "the layer is patched only in the browser half")
)]
#[derive(Debug)]
enum Redraw {
    /// The lines [`Patch`] names changed, and the kept lines at `classes`
    /// changed their class; `exact` is false while the changed lines are
    /// highlighted by themselves in a grammar that reads across lines.
    Edit {
        /// The lines that changed.
        patch: Patch,
        /// The 0-based kept lines whose class changed.
        classes: Vec<usize>,
        /// Whether the changed lines are highlighted as the whole text
        /// highlights them.
        exact: bool,
    },
    /// The layer was made again: the lines it had.
    Lines(Vec<LayerLine>),
}

/// What the layer draws: the grammar and the text it highlighted, and its
/// lines.
#[derive(Debug)]
struct Layer {
    /// The grammar the text was highlighted as.
    language: Language,
    /// The text it highlighted, its lines indexed.
    index: Indexed,
    /// Its lines.
    lines: Vec<LayerLine>,
}

impl Layer {
    /// The layer of `text` in `language`, marked by `marks` while the
    /// cursor's marked line is `current`.
    fn of(language: Language, text: &str, marks: &[LineMark], current: Option<u32>) -> Self {
        Self {
            language,
            index: Indexed::of(text),
            lines: layer_lines(language, text, marks, current),
        }
    }

    /// The layer after this one for `text` in `language`, `marks` and
    /// `current`: while the text and the grammar stay, as when the caret
    /// moves or a check marks other lines, it keeps this layer's
    /// highlighting and changes only the marks.
    fn next(
        &self,
        language: Language,
        text: &str,
        marks: &[LineMark],
        current: Option<u32>,
    ) -> Self {
        if language != self.language || text != self.index.text {
            return Self::of(language, text, marks, current);
        }
        let kinds = mark_index(marks);
        let lines = (1_u32..)
            .zip(&self.lines)
            .map(|(line, drawn)| LayerLine {
                class: layer_class(kinds.get(&line).copied(), line, current),
                body: Rc::clone(&drawn.body),
            })
            .collect();
        Self {
            language,
            index: self.index.clone(),
            lines,
        }
    }

    /// The grammar the lines are highlighted as: none past
    /// [`MAX_HIGHLIGHT_BYTES`], as [`crate::highlight::tokens`] decides
    /// from the whole text.
    #[cfg_attr(
        all(not(target_arch = "wasm32"), not(test)),
        expect(dead_code, reason = "the layer is patched only in the browser half")
    )]
    fn grammar(&self) -> Language {
        if self.index.text.len() > MAX_HIGHLIGHT_BYTES {
            Language::Plain
        } else {
            self.language
        }
    }

    /// Brings this layer to `text` in `language`, `marks` and `current`,
    /// and says how it changed. An edit of the text in the same grammar
    /// highlights only the lines it touched, by themselves, and keeps every
    /// other line; anything else makes the layer again.
    #[cfg_attr(
        all(not(target_arch = "wasm32"), not(test)),
        expect(dead_code, reason = "the layer is patched only in the browser half")
    )]
    fn update(
        &mut self,
        language: Language,
        text: &str,
        marks: &[LineMark],
        current: Option<u32>,
    ) -> Redraw {
        let over = |text: &str| text.len() > MAX_HIGHLIGHT_BYTES;
        if language != self.language || over(text) != over(&self.index.text) {
            let next = self.next(language, text, marks, current);
            return Redraw::Lines(std::mem::replace(self, next).lines);
        }
        let kinds = mark_index(marks);
        let count = self.lines.len();
        let patch = if let Some(edit) = self.index.edit(text) {
            let Some(patch) = self.splice(edit, &kinds, current) else {
                let next = Self::of(language, text, marks, current);
                return Redraw::Lines(std::mem::replace(self, next).lines);
            };
            patch
        } else {
            Patch {
                head: count,
                removed: 0,
                inserted: 0,
                tail: 0,
            }
        };
        let classes = self.classes(&kinds, current, &patch);
        Redraw::Edit {
            patch,
            classes,
            exact: reads_lines_alone(self.grammar()),
        }
    }

    /// Replaces the lines `edit` names with the changed text's lines, each
    /// highlighted again, keeping those that come back alike, and returns
    /// the patch; `None` when the index and the lines disagree.
    #[cfg_attr(
        all(not(target_arch = "wasm32"), not(test)),
        expect(dead_code, reason = "the layer is patched only in the browser half")
    )]
    fn splice(
        &mut self,
        edit: Edit,
        kinds: &BTreeMap<u32, MarkKind>,
        current: Option<u32>,
    ) -> Option<Patch> {
        let Edit {
            first,
            removed,
            inserted,
            ..
        } = edit;
        let end = first.checked_add(removed)?;
        let old = self.lines.get(first..end)?;
        let start = self.index.starts.get(first)?.byte;
        let stop = match self.index.starts.get(first.checked_add(inserted)?) {
            Some(next) => next.byte.checked_sub(1)?,
            None => self.index.text.len(),
        };
        let part = self.index.text.get(start..stop)?;
        let bodies = lines_html(self.grammar(), part);
        if bodies.len() != inserted {
            return None;
        }
        let fresh: Vec<LayerLine> = (first..)
            .zip(bodies)
            .map(|(at, body)| {
                let line = number(at);
                LayerLine {
                    class: layer_class(kinds.get(&line).copied(), line, current),
                    body: Rc::from(body),
                }
            })
            .collect();
        let lead = old
            .iter()
            .zip(&fresh)
            .take_while(|(old, fresh)| old == fresh)
            .count();
        let room = old.len().min(fresh.len()).saturating_sub(lead);
        let trail = old
            .iter()
            .rev()
            .zip(fresh.iter().rev())
            .take(room)
            .take_while(|(old, fresh)| old.body == fresh.body)
            .count();
        let removed = removed.saturating_sub(lead).saturating_sub(trail);
        let inserted = inserted.saturating_sub(lead).saturating_sub(trail);
        let head = first.saturating_add(lead);
        let changed = fresh.into_iter().skip(lead).take(inserted);
        self.lines
            .splice(head..head.saturating_add(removed), changed);
        Some(Patch {
            head,
            removed,
            inserted,
            tail: self
                .lines
                .len()
                .saturating_sub(head)
                .saturating_sub(inserted),
        })
    }

    /// Gives every kept line that is marked, or was, the class `kinds` and
    /// `current` give it, and returns the 0-based lines whose class changed.
    #[cfg_attr(
        all(not(target_arch = "wasm32"), not(test)),
        expect(dead_code, reason = "the layer is patched only in the browser half")
    )]
    fn classes(
        &mut self,
        kinds: &BTreeMap<u32, MarkKind>,
        current: Option<u32>,
        patch: &Patch,
    ) -> Vec<usize> {
        let drawn = patch.head..patch.head.saturating_add(patch.inserted);
        let mut lines: Vec<usize> = (0..)
            .zip(&self.lines)
            .filter(|(_, line)| line.class.is_some())
            .map(|(at, _)| at)
            .chain(
                kinds
                    .keys()
                    .filter_map(|line| usize::try_from(*line).ok()?.checked_sub(1)),
            )
            .filter(|at| !drawn.contains(at))
            .collect();
        lines.sort_unstable();
        lines.dedup();
        lines.retain(|&at| {
            let line = number(at);
            let class = layer_class(kinds.get(&line).copied(), line, current);
            match self.lines.get_mut(at) {
                Some(drawn) if drawn.class != class => {
                    drawn.class = class;
                    true
                }
                _ => false,
            }
        });
        lines
    }

    /// Highlights the whole text again, keeping each line's class and every
    /// line that comes back alike, and returns the lines it had.
    #[cfg_attr(
        all(not(target_arch = "wasm32"), not(test)),
        expect(dead_code, reason = "the layer is patched only in the browser half")
    )]
    fn rehighlight(&mut self) -> Vec<LayerLine> {
        let fresh = lines_html(self.language, &self.index.text);
        let lines = fresh
            .into_iter()
            .enumerate()
            .map(|(at, body)| match self.lines.get(at) {
                Some(old) if *old.body == *body => old.clone(),
                old => LayerLine {
                    class: old.and_then(|old| old.class),
                    body: Rc::from(body),
                },
            })
            .collect();
        std::mem::replace(&mut self.lines, lines)
    }
}

/// Appends `lines` to `out` as the layer's line elements, the first
/// numbered `first`, each with its id after `prefix`.
fn push_layer(out: &mut String, lines: &[LayerLine], prefix: &str, first: u32) {
    for (number, line) in (first..).zip(lines) {
        push_line(out, number, line.class, Some(prefix), &line.body);
    }
}

/// How the layer goes from its old lines to its new ones: the first `head`
/// lines stay, the next `removed` old lines go and `inserted` new lines
/// take their place, and the last `tail` lines keep their elements, with
/// a new id when the lines before them grew or shrank and a new class when
/// their mark changed.
#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(
        dead_code,
        reason = "the layer is patched only in the browser half; the server half draws it whole"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Patch {
    /// The lines at the start that stay as they are.
    head: usize,
    /// The old lines after them that go.
    removed: usize,
    /// The new lines in their place.
    inserted: usize,
    /// The lines at the end whose highlighting stays.
    tail: usize,
}

/// The patch from the layer lines `old` to `new`.
#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(
        dead_code,
        reason = "the layer is patched only in the browser half; the server half draws it whole"
    )
)]
fn patch(old: &[LayerLine], new: &[LayerLine]) -> Patch {
    let head = old
        .iter()
        .zip(new)
        .take_while(|(old, new)| old == new)
        .count();
    let room = old.len().min(new.len()).saturating_sub(head);
    let tail = old
        .iter()
        .skip(head)
        .rev()
        .zip(new.iter().skip(head).rev())
        .take(room)
        .take_while(|(old, new)| old.body == new.body)
        .count();
    Patch {
        head,
        removed: old.len().saturating_sub(head).saturating_sub(tail),
        inserted: new.len().saturating_sub(head).saturating_sub(tail),
        tail,
    }
}

/// Brings the layer element `layer`, which draws `old`, to `new`, its line
/// ids after `prefix`: only the lines [`patch`] names are touched, and a
/// layer that no longer holds one element per old line is drawn whole.
#[cfg(target_arch = "wasm32")]
fn apply(layer: &web_sys::Element, prefix: &str, old: &[LayerLine], new: &[LayerLine]) {
    let patch = patch(old, new);
    let start = new.len().saturating_sub(patch.tail);
    let old_tail = old.iter().skip(old.len().saturating_sub(patch.tail));
    let classes: Vec<usize> = (start..)
        .zip(new.iter().skip(start))
        .zip(old_tail)
        .filter(|((_, line), was)| line.class != was.class)
        .map(|((at, _), _)| at)
        .collect();
    redraw(layer, prefix, patch, new, &classes);
}

/// Brings the layer element `layer` to `lines` by `patch`, its line ids
/// after `prefix`: the changed lines are drawn again, the lines after them
/// take new ids when the count changed, and the kept lines at `classes`
/// take their new class. A layer that does not hold the element count the
/// patch starts from is drawn whole.
#[cfg(target_arch = "wasm32")]
fn redraw(
    layer: &web_sys::Element,
    prefix: &str,
    patch: Patch,
    lines: &[LayerLine],
    classes: &[usize],
) {
    use leptos::wasm_bindgen::JsCast as _;
    let Patch {
        head,
        removed,
        inserted,
        tail,
    } = patch;
    let nodes = layer.child_nodes();
    let old = head.saturating_add(removed).saturating_add(tail);
    if usize::try_from(nodes.length()).ok() != Some(old) {
        let mut html = String::new();
        push_layer(&mut html, lines, prefix, 1);
        layer.set_inner_html(&html);
        return;
    }
    let index = |at: usize| u32::try_from(at).unwrap_or(u32::MAX);
    for _ in 0..removed {
        if let Some(node) = nodes.item(index(head))
            && layer.remove_child(&node).is_err()
        {
            leptos::logging::warn!("the browser refused to remove an editor line");
        }
    }
    if inserted > 0 {
        let mut html = String::new();
        let drawn = lines
            .get(head..head.saturating_add(inserted))
            .unwrap_or_default();
        push_layer(&mut html, drawn, prefix, number(head));
        let placed = match nodes.item(index(head)) {
            Some(next) => next.dyn_into::<web_sys::Element>().map_or(Ok(()), |next| {
                next.insert_adjacent_html("beforebegin", &html)
            }),
            None => layer.insert_adjacent_html("beforeend", &html),
        };
        if placed.is_err() {
            leptos::logging::warn!("the browser refused the editor's new lines");
        }
    }
    let element = |at: usize| {
        nodes
            .item(index(at))
            .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
    };
    let kept = head.saturating_add(inserted);
    if removed != inserted {
        for at in kept..lines.len() {
            if let Some(element) = element(at) {
                element.set_id(&format!("{prefix}{}", number(at)));
            }
        }
    }
    let drawn = head..kept;
    for &at in classes {
        if drawn.contains(&at) {
            continue;
        }
        let (Some(element), Some(line)) = (element(at), lines.get(at)) else {
            continue;
        };
        let class = line.class.map_or_else(
            || String::from("fui-code-line"),
            |class| format!("fui-code-line {class}"),
        );
        element.set_class_name(&class);
    }
}

/// How long the editor waits after the last edit before it highlights a
/// text whose grammar reads across lines whole again.
#[cfg(target_arch = "wasm32")]
const SETTLE: std::time::Duration = std::time::Duration::from_millis(250);

/// Highlights the layer `state` draws in the element `layer` whole again
/// once [`SETTLE`] passes with no newer edit than `ticket` in `edits`.
#[cfg(target_arch = "wasm32")]
fn settle(
    state: Rc<RefCell<Layer>>,
    layer: web_sys::Element,
    prefix: String,
    edits: Rc<Cell<u64>>,
) {
    let ticket = edits.get().wrapping_add(1);
    edits.set(ticket);
    set_timeout(
        move || {
            if edits.get() != ticket {
                return;
            }
            let Ok(mut state) = state.try_borrow_mut() else {
                return;
            };
            let old = state.rehighlight();
            apply(&layer, &prefix, &old, &state.lines);
        },
        SETTLE,
    );
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
#[derive(Debug, Clone)]
pub struct Editor {
    /// The text area's id and name: a static id, or one made at run time,
    /// such as one per staged change of a page that holds several.
    pub id: Cow<'static, str>,
    /// The grammar the text is highlighted as.
    pub language: Language,
    /// The text, which the editor reads and never writes.
    pub text: Signal<String>,
    /// The lines a diagnostic names.
    pub marks: Signal<Vec<LineMark>>,
    /// Whether the text may be edited; false for a text the reader may
    /// only read. An editable text area takes keys from the first paint,
    /// with no WebAssembly too: before its island runs it shows its own
    /// text, and what it holds as the page hydrates goes to the console.
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
    plain_editor(editor, Heard::Text(Rc::new(on_input)))
}

/// Returns the editor over `editor`, as [`code_editor`] does, handing
/// `on_edit` each edit the reader makes instead of the whole text.
///
/// A console keeps its own copy of the text in step with
/// [`TextEdit::apply`], so a key costs it the edit, not the length of the
/// text. `editor.text` is the text the editor opens with and any outside
/// change, such as a reset: the console never writes an edited text back
/// into it, since the editor takes every text that signal brings as an
/// outside change and writes it into the text area. An outside change is
/// not handed to `on_edit`; the console that makes one knows it. A text
/// typed before the page hydrated comes to `on_edit` as one edit of the
/// server's text as the page hydrates.
#[must_use]
pub fn code_editor_by_edit(editor: Editor, on_edit: impl Fn(TextEdit) + 'static) -> AnyView {
    plain_editor(editor, Heard::Edit(Rc::new(on_edit)))
}

/// Returns the editor over `editor`, handing the console what `heard`
/// asks for.
fn plain_editor(editor: Editor, heard: Heard) -> AnyView {
    let Editor {
        id,
        language,
        text,
        marks,
        editable,
    } = editor;
    drawn(
        Drawn {
            id: id.into_owned(),
            language: Grammar::Fixed(language),
            text,
            marks,
            editable,
            placeholder: None,
            label: None,
            caret: None,
            current: Signal::stored(None),
            record: None,
        },
        heard,
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

    /// The grammar now, untracked.
    fn get_untracked(self) -> Language {
        match self {
            Self::Fixed(language) => language,
            Self::Following(language) => language.get_untracked(),
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
    /// Whether the text may be edited.
    pub(crate) editable: Signal<bool>,
    /// The text area's placeholder, shown while the text is empty.
    pub(crate) placeholder: Option<Cow<'static, str>>,
    /// The text area's accessible name, when no label names it.
    pub(crate) label: Option<String>,
    /// Where the caret is, which the editor writes as it moves.
    pub(crate) caret: Option<RwSignal<Position>>,
    /// The marked line the cursor is on, which the layer marks current.
    pub(crate) current: Signal<Option<u32>>,
    /// The record of the text a panel keeps measured; without one the
    /// editor makes its own when it follows the caret or hands over edits.
    pub(crate) record: Option<Typed>,
}

/// How many texts the editor remembers handing to the console before the
/// console hands any back.
const ECHOES: usize = 64;

/// The texts the editor handed to the console that have not come back yet:
/// the newest whole, and each one before it as its hash, so a console that
/// never hands one back holds the editor to [`ECHOES`] numbers and one
/// text. A console that hands each text back before the next key costs no
/// hash at all.
#[derive(Debug, Default)]
struct Echoes {
    /// The hashes of the texts handed over before the newest, oldest first.
    pending: VecDeque<u64>,
    /// The newest text handed over, until it comes back.
    newest: Option<String>,
}

impl Echoes {
    /// The hash of `text`.
    fn hash(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    /// Records that the editor handed `text` to the console.
    fn typed(&mut self, text: &str) {
        let Some(earlier) = self.newest.replace(text.to_owned()) else {
            return;
        };
        if self.pending.len() >= ECHOES {
            self.pending.pop_front();
        }
        self.pending.push_back(Self::hash(&earlier));
    }

    /// Whether the console's `text` is one the editor handed it, coming
    /// back; it forgets that text and every one handed before it. Any other
    /// text is an outside change, and it forgets them all.
    fn came_back(&mut self, text: &str) -> bool {
        if self.newest.as_deref() == Some(text) {
            self.pending.clear();
            self.newest = None;
            return true;
        }
        if self.pending.is_empty() {
            self.newest = None;
            return false;
        }
        let hash = Self::hash(text);
        if let Some(at) = self.pending.iter().position(|held| *held == hash) {
            self.pending.drain(..=at);
            true
        } else {
            self.pending.clear();
            self.newest = None;
            false
        }
    }
}

/// Writes the console's `text` into the text area `area_ref` when it is an
/// outside change, and returns the record of the texts handed to the
/// console, which the text area's input fills. A text typed before the
/// page hydrated is handed to the console through `heard` as the page
/// hydrates, and never written over. `record`, when the editor keeps one,
/// learns every text written or handed over this way, and so does `shown`,
/// the text the layer draws for a console that takes edits.
fn follow_console(
    text: Signal<String>,
    area_ref: NodeRef<leptos::html::Textarea>,
    heard: Heard,
    record: Option<Rc<Record>>,
    shown: Option<RwSignal<String>>,
) -> Rc<RefCell<Echoes>> {
    let echoes = Rc::new(RefCell::new(Echoes::default()));
    let handed = Rc::clone(&echoes);
    // NOTE: the text area is the source of truth while it is edited; the console's text is
    // written only as an outside change, never as the late echo of an earlier key (#333).
    Effect::new(move |hydrated: Option<()>| {
        text.with(|console| {
            if echoes.borrow_mut().came_back(console) {
                return;
            }
            #[cfg(target_arch = "wasm32")]
            if let Some(area) = area_ref.get()
                && area.value() != *console
            {
                if hydrated.is_some() {
                    area.set_value(console);
                    if let Some(record) = &record {
                        record.follow(console, false);
                    }
                    if let Some(shown) = shown {
                        shown.set(console.clone());
                    }
                } else {
                    // NOTE: the text area takes keys from the first paint (#356), so what it
                    // holds as the page hydrates is the reader's and goes to the console.
                    let typed = area.value();
                    if let Some(record) = &record {
                        record.follow(&typed, true);
                    }
                    hand_over(&heard, &echoes, shown, typed);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            let (_area, _hydrated, _heard, _record, _shown) =
                (area_ref, hydrated, &heard, &record, shown);
        });
    });
    handed
}

/// Hands the text area's `typed` text on: to the console when it takes the
/// whole text, recorded in `echoes`, and otherwise to `shown`, the text the
/// layer draws, the console having had the edit.
fn hand_over(
    heard: &Heard,
    echoes: &RefCell<Echoes>,
    shown: Option<RwSignal<String>>,
    typed: String,
) {
    match heard {
        Heard::Text(on_input) => {
            echoes.borrow_mut().typed(&typed);
            on_input(typed);
        }
        Heard::Edit(_) => {
            if let Some(shown) = shown {
                shown.set(typed);
            }
        }
    }
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
    escape(text, &mut out);
    out
}

/// What the layer follows: the grammar, the text, the marks and the marked
/// line the cursor is on.
#[derive(Clone, Copy)]
struct Followed {
    /// The grammar the text is highlighted as.
    language: Grammar,
    /// The text.
    text: Signal<String>,
    /// The lines a diagnostic names.
    marks: Signal<Vec<LineMark>>,
    /// The marked line the cursor is on.
    current: Signal<Option<u32>>,
}

/// Keeps the layer element `layer_ref`, drawn as `drawn` with its line ids
/// after `prefix`, in step with `followed`.
fn follow_layer(
    followed: Followed,
    drawn: Layer,
    layer_ref: NodeRef<leptos::html::Pre>,
    prefix: String,
) {
    let Followed {
        language,
        text,
        marks,
        current,
    } = followed;
    let state = Rc::new(RefCell::new(drawn));
    let edits = Rc::new(Cell::new(0_u64));
    // NOTE: no specification governs this: our own design; an edit highlights the lines it
    // touched, and a grammar that reads across lines is highlighted whole once typing rests (#458).
    Effect::new(move || {
        let language = language.get();
        let current = current.get();
        let marks = marks.read();
        text.with(|text| {
            #[cfg(target_arch = "wasm32")]
            {
                let Some(element) = layer_ref.get() else {
                    return;
                };
                let Ok(mut layer) = state.try_borrow_mut() else {
                    return;
                };
                match layer.update(language, text, &marks, current) {
                    Redraw::Edit {
                        patch,
                        classes,
                        exact,
                    } => {
                        redraw(&element, &prefix, patch, &layer.lines, &classes);
                        if !exact {
                            let element: &web_sys::Element = &element;
                            settle(
                                Rc::clone(&state),
                                element.clone(),
                                prefix.clone(),
                                Rc::clone(&edits),
                            );
                        }
                    }
                    Redraw::Lines(old) => apply(&element, &prefix, &old, &layer.lines),
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            let (_text, _language, _current, _state, _edits, _prefix, _layer) =
                (text, language, current, &state, &edits, &prefix, layer_ref);
        });
        drop(marks);
    });
}

/// What follows the caret of a text area into `caret`, read from `record`,
/// the index of the text area's text, which it brings up to date first
/// when the text area's length says the text changed.
fn follow_caret(
    caret: Option<RwSignal<Position>>,
    record: Option<Rc<Record>>,
) -> Rc<dyn Fn(&HtmlTextAreaElement)> {
    Rc::new(move |area: &HtmlTextAreaElement| {
        let (Some(caret), Some(record)) = (caret, &record) else {
            return;
        };
        let Ok(Some(start)) = area.selection_start() else {
            return;
        };
        let behind = area.text_length() != record.typed.borrow().index.len_units();
        if behind {
            record.follow(&area.value(), true);
        }
        caret.set(record.typed.borrow().index.caret(start));
    })
}

/// Returns the editor `drawn`, as [`code_editor`], [`editor_panel`] and
/// the upload field draw it, handing the console what `heard` asks for.
pub(crate) fn drawn(drawn: Drawn, heard: Heard) -> AnyView {
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
        record,
    } = drawn;
    let prefix = format!("{id}-L");
    // NOTE: no specification governs this: our own design; a console that takes edits never
    // hands the text back, so the layer draws the text area's text, moved in, not copied (#478).
    let shown = matches!(heard, Heard::Edit(_)).then(|| RwSignal::new(text.get_untracked()));
    let layer_text = shown.map_or(text, Signal::from);
    let drawn_layer = text.with_untracked(|text| {
        Layer::of(
            language.get_untracked(),
            text,
            &marks.get_untracked(),
            current.get_untracked(),
        )
    });
    let mut layer = String::new();
    push_layer(&mut layer, &drawn_layer.lines, &prefix, 1);
    let layer_ref = NodeRef::<leptos::html::Pre>::new();
    follow_layer(
        Followed {
            language,
            text: layer_text,
            marks,
            current,
        },
        drawn_layer,
        layer_ref,
        prefix,
    );
    let name = id.clone();
    let first = text.with_untracked(|text| server_text(text));
    let area_ref = NodeRef::<leptos::html::Textarea>::new();
    let record = Record::kept(record, caret.is_some(), &heard, text);
    let echoes = follow_console(text, area_ref, heard.clone(), record.clone(), shown);
    let input_record = record.clone();
    let live = RwSignal::new(false);
    Effect::new(move || live.set(true));
    let follow = follow_caret(caret, record);
    let (on_key, on_click, on_select) =
        (Rc::clone(&follow), Rc::clone(&follow), Rc::clone(&follow));
    let invalid = move || {
        marks
            .read()
            .iter()
            .any(|mark| mark.kind == MarkKind::Error)
            .then_some("true")
    };
    view! {
        <div class="fui-editor">
            <div class="fui-editor-stack" class:is-static=move || !live.get()>
                <pre
                    class="fui-editor-layer"
                    aria-hidden="true"
                    node_ref=layer_ref
                    inner_html=layer
                ></pre>
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
                    node_ref=area_ref
                    on:input:target=move |event| {
                        let area = event.target();
                        let typed = area.value();
                        if let Some(record) = &input_record {
                            record.follow(&typed, true);
                        }
                        hand_over(&heard, &echoes, shown, typed);
                        follow(&area);
                    }
                    on:keyup:target=move |event| on_key(&event.target())
                    on:click:target=move |event| on_click(&event.target())
                    on:select:target=move |event| on_select(&event.target())
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
            grouping: Grouping::of_page(),
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
) -> Option<Word> {
    let line = current.get()?;
    let kind = marks.with(|marks| mark_kind(marks, line))?;
    Some(words.with_value(|words| {
        let phrase = match kind {
            MarkKind::Error => &words.error_on,
            MarkKind::Warning => &words.warning_on,
        };
        phrase.fill(&[("line", &words.number(line))])
    }))
}

/// Returns the code editor (B2) over `panel`: F8's toolbar, the editor
/// with its gutter marks, and the status line; `on_input` receives the
/// text after every edit, exactly as typed.
///
/// The editor is 24rem tall, and 6rem while it holds no text; in a panel
/// section (B10) it has no card frame and runs edge to edge. The toolbar's
/// line count and the status line's changed state follow each edit at its
/// cost, never by a scan of the whole text.
#[must_use]
pub fn editor_panel(panel: Panel, on_input: impl Fn(String) + 'static) -> AnyView {
    measured_panel(panel, Heard::Text(Rc::new(on_input)))
}

/// Returns the code editor (B2) over `panel`, as [`editor_panel`] does,
/// handing `on_edit` each edit the reader makes instead of the whole text,
/// on the terms of [`code_editor_by_edit`].
#[must_use]
pub fn editor_panel_by_edit(panel: Panel, on_edit: impl Fn(TextEdit) + 'static) -> AnyView {
    measured_panel(panel, Heard::Edit(Rc::new(on_edit)))
}

/// Returns the code editor (B2) over `panel`, handing the console what
/// `heard` asks for.
fn measured_panel(panel: Panel, heard: Heard) -> AnyView {
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
    let is_new = matches!(opened, Opened::New);
    let (record, measure) = text.with_untracked(|text| Typed::measured(text, &opened));
    let bar = move || {
        let count = measure.with(|measure| measure.lines);
        let lines = words.with_value(|words| words.line_count(count));
        if lines.fell_back() {
            let lead = format!("{lead} · ");
            view! { {lead} {lines} }.into_any()
        } else {
            format!("{lead} · {}", lines.text()).into_any()
        }
    };
    let position = move || {
        let at = caret.get();
        words
            .with_value(|words| {
                words.position.fill(&[
                    ("line", &words.number(at.line)),
                    ("column", &words.number(at.column)),
                ])
            })
            .into_render()
    };
    let dirty = move || measure.with(|measure| measure.changed);
    let state = move || {
        words
            .with_value(|words| match (is_new, dirty()) {
                (true, _) => words.new_file.clone(),
                (false, true) => words.changed.clone(),
                (false, false) => words.unchanged.clone(),
            })
            .into_render()
    };
    let said = move || said(current, marks, words).map(IntoRender::into_render);
    let facts = facts
        .into_iter()
        .map(|fact| view! { <span>{fact}</span> })
        .collect_view();
    let end = end.map(|end| view! { <span class="fui-editor-status-end">{end}</span> });
    let field = drawn(
        Drawn {
            id: id.into_owned(),
            language: Grammar::Fixed(language),
            text,
            marks,
            editable,
            placeholder: None,
            label: Some(label),
            caret: Some(caret),
            current,
            record: Some(record),
        },
        heard,
    );
    view! {
        <section
            class="fui-card fui-panel fui-mapping-editor"
            class:is-empty=move || measure.with(|measure| measure.blank)
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
/// edits, named `label`, its rows in `words`, in step with the cursor at
/// `caret`.
///
/// The row of the marked line the cursor is on is current; following a
/// row's line link moves the caret to the finding's line and column and
/// gives the editor the focus.
#[must_use]
pub fn editor_diagnostics(
    label: &str,
    file: &FileFindings,
    words: &DiagnosticsWords,
    editor: Editor,
    caret: RwSignal<Position>,
) -> AnyView {
    let Editor { id, marks, .. } = editor;
    let current = current_line(caret, marks);
    let on_line = Callback::new(move |at: Position| {
        caret.set(at);
        place_caret(&id, at);
    });
    let rows = rows(file, words, current, Some(on_line));
    view! {
        <ul class="fui-diags" aria-label=label.to_owned()>
            {rows}
        </ul>
    }
    .into_any()
}

/// Moves the caret of the text area `id` to `at`, read in the text it
/// holds, and gives it the focus once the link followed has scrolled to
/// the line.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        unused_variables,
        reason = "the caret moves only in the browser half; the server half has no text area"
    )
)]
fn place_caret(id: &str, at: Position) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let Some(area) = document()
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<HtmlTextAreaElement>().ok())
        else {
            return;
        };
        let offset = offset_of(&area.value(), at);
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
        ECHOES, Echoes, Layer, LayerLine, LineMark, MarkKind, Patch, caret_at, layer_class,
        layer_lines, line_id, mark_class, mark_index, offset_of, patch, push_layer, server_text,
    };
    use super::{Indexed, Redraw, Since, TextEdit, Typed, shared_head, shared_tail};
    use crate::highlight::{Language, Lines, html};
    use crate::view::diagnostics::Position;
    use std::rc::Rc;

    #[test]
    fn a_late_echo_of_an_earlier_key_is_never_written() {
        let mut echoes = Echoes::default();
        echoes.typed("S");
        echoes.typed("SE");
        echoes.typed("SEL");
        assert!(echoes.came_back("S"), "the first key's echo is an echo");
        assert!(
            echoes.came_back("SEL"),
            "so is the newest one's, past the middle one"
        );
        assert!(
            echoes.pending.is_empty(),
            "every text handed over came back"
        );
    }

    #[test]
    fn a_text_the_editor_never_handed_over_is_an_outside_change() {
        let mut echoes = Echoes::default();
        echoes.typed("SELECT");
        assert!(
            !echoes.came_back("SELECT 1"),
            "a console's own text is written"
        );
        assert!(
            !echoes.came_back("SELECT"),
            "and the texts before it are forgotten"
        );
    }

    #[test]
    fn a_console_that_never_answers_holds_a_bounded_list() {
        let mut echoes = Echoes::default();
        for at in 0..ECHOES * 2 {
            echoes.typed(&at.to_string());
        }
        assert_eq!(ECHOES, echoes.pending.len());
        assert!(!echoes.came_back("0"), "the oldest text is forgotten");
    }

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

    /// The text of `count` lines, each naming its number.
    fn numbered(count: u32) -> String {
        (1..=count)
            .map(|line| format!("key_{line}: value {line}\n"))
            .collect::<Vec<_>>()
            .concat()
    }

    /// The layer of `text`, unmarked.
    fn layer(text: &str) -> Vec<LayerLine> {
        layer_lines(Language::Yaml, text, &[], None)
    }

    #[test]
    fn the_layer_draws_the_lines_the_highlighter_draws() {
        let text = "a: 1\n# b <c> & 'd'\n";
        let marks = [LineMark {
            line: 2,
            kind: MarkKind::Error,
        }];
        let mut drawn = String::new();
        push_layer(
            &mut drawn,
            &layer_lines(Language::Yaml, text, &marks, Some(2)),
            "d-L",
            1,
        );
        let marked = |line| (line == 2).then_some("fui-mark-error is-current");
        assert_eq!(
            html(Language::Yaml, text, Lines { ids: Some("d-L") }, marked),
            drawn
        );
    }

    #[test]
    fn one_keystroke_redraws_one_line_of_ten_thousand() {
        let text = numbered(10_000);
        let typed = text.replacen("value 5000\n", "value 50000\n", 1);
        assert_eq!(
            Patch {
                head: 4999,
                removed: 1,
                inserted: 1,
                tail: 5001,
            },
            patch(&layer(&text), &layer(&typed))
        );
    }

    #[test]
    fn a_new_line_inserts_one_line_and_keeps_the_rest() {
        let text = numbered(10_000);
        let typed = text.replacen("value 10\n", "value 10\n\n", 1);
        assert_eq!(
            Patch {
                head: 10,
                removed: 0,
                inserted: 1,
                tail: 9991,
            },
            patch(&layer(&text), &layer(&typed))
        );
        let joined = text.replacen("value 10\n", "value 10", 1);
        assert_eq!(
            Patch {
                head: 9,
                removed: 2,
                inserted: 1,
                tail: 9990,
            },
            patch(&layer(&text), &layer(&joined))
        );
    }

    #[test]
    fn a_moved_mark_touches_no_line_body() {
        let text = numbered(100);
        let mark = |line| {
            [LineMark {
                line,
                kind: MarkKind::Warning,
            }]
        };
        let before = layer_lines(Language::Yaml, &text, &mark(10), None);
        let after = layer_lines(Language::Yaml, &text, &mark(20), None);
        assert_eq!(
            Patch {
                head: 9,
                removed: 0,
                inserted: 0,
                tail: 92,
            },
            patch(&before, &after),
            "the lines whose class changed keep their elements"
        );
    }

    #[test]
    fn a_caret_move_keeps_the_highlighting_and_moves_the_current_mark() {
        let text = numbered(10);
        let marks = [LineMark {
            line: 4,
            kind: MarkKind::Error,
        }];
        let rested = Layer::of(Language::Yaml, &text, &marks, None);
        let moved = rested.next(Language::Yaml, &text, &marks, Some(4));
        assert!(
            rested
                .lines
                .iter()
                .zip(&moved.lines)
                .all(|(rested, moved)| Rc::ptr_eq(&rested.body, &moved.body)),
            "no line is highlighted again"
        );
        assert_eq!(
            Some("fui-mark-error is-current"),
            moved.lines.get(3).and_then(|line| line.class)
        );
        let typed = rested.next(Language::Yaml, &format!("{text}x"), &marks, None);
        assert!(
            !Rc::ptr_eq(&rested.lines[0].body, &typed.lines[0].body),
            "a changed text is highlighted again"
        );
    }

    #[test]
    fn an_edit_to_nothing_and_back_redraws_every_line() {
        let text = "a: 1\nb: 2\nc: 3";
        let empty = layer("");
        assert_eq!(
            Patch {
                head: 0,
                removed: 3,
                inserted: 1,
                tail: 0,
            },
            patch(&layer(text), &empty)
        );
        assert_eq!(
            Patch {
                head: 0,
                removed: 1,
                inserted: 3,
                tail: 0,
            },
            patch(&empty, &layer(text))
        );
    }

    /// Edits each text of `texts` into every other, by hand-picked changes.
    fn edits() -> Vec<(String, String)> {
        let base = "a: 1\nnaïve: ✓ 𝄞\n\nlast";
        let mut pairs = vec![
            (String::new(), String::from("x")),
            (String::from("x"), String::new()),
            (base.to_owned(), format!("x{base}")),
            (base.to_owned(), format!("{base}x")),
            (base.to_owned(), format!("{base}\n")),
            (base.to_owned(), base.replacen("✓", "✗", 1)),
            (base.to_owned(), base.replacen("✓ 𝄞\n", "", 1)),
            (base.to_owned(), base.replacen("\n\n", "\n", 1)),
            (base.to_owned(), base.replacen("1\n", "1\nb: 2\nc: 3\n", 1)),
            (base.to_owned(), base.replacen("𝄞", "𝄞𝄞", 1)),
            (String::from("ab\nab\nab"), String::from("ab\nab\nab\nab")),
        ];
        let long = numbered(300);
        pairs.push((
            long.clone(),
            long.replacen("value 150\n", "value 150x\n", 1),
        ));
        pairs.push((long.clone(), long.replacen("value 10\n", "", 1)));
        pairs.push((long.clone(), long.replacen("key_299", "key_299\n", 1)));
        pairs
    }

    #[test]
    fn an_index_follows_every_edit_by_its_changed_part() {
        for (old, new) in edits() {
            let mut index = Indexed::of(&old);
            let edit = index.edit(&new).expect("the text changed");
            assert_eq!(Indexed::of(&new), index, "{old:?} to {new:?}");
            let lines = |text: &str| text.split('\n').count();
            assert_eq!(
                lines(&new).wrapping_sub(lines(&old)),
                edit.inserted.wrapping_sub(edit.removed),
                "{old:?} to {new:?}"
            );
            assert_eq!(None, index.edit(&new), "an unchanged text is no edit");
        }
    }

    #[test]
    fn an_edit_handed_to_the_console_brings_its_text_to_the_new_one() {
        for (old, new) in edits() {
            let mut index = Indexed::of(&old);
            let edit = TextEdit::of(index.edit(&new).expect("the text changed"), &new);
            let mut kept = old.clone();
            assert!(edit.apply(&mut kept), "{old:?} to {new:?}");
            assert_eq!(new, kept, "{old:?} to {new:?} by {edit:?}");
            assert!(
                edit.text.len() < new.len() || old.len() < 3,
                "the edit carries its part, not the text: {edit:?}"
            );
        }
    }

    #[test]
    fn an_edit_out_of_step_with_the_text_leaves_it_alone() {
        let edit = TextEdit {
            range: 1..2,
            text: String::from("x"),
        };
        let mut multi_byte = String::from("✓ok");
        assert!(!edit.apply(&mut multi_byte), "inside a character");
        assert_eq!("✓ok", multi_byte);
        let mut short = String::new();
        assert!(!edit.apply(&mut short), "past the end");
        assert_eq!("", short);
    }

    #[test]
    fn the_changed_state_follows_each_edit_against_the_opened_text() {
        let opened = numbered(300);
        let steps = [
            opened.replacen("value 150\n", "value 15x\n", 1),
            opened.clone(),
            opened.replacen("key_1:", "key_I:", 1),
            opened
                .replacen("key_1:", "key_I:", 1)
                .replacen("value 299", "value 29x", 1),
            opened.replacen("value 299", "value 29x", 1),
            opened.clone(),
            format!("{opened}tail"),
            opened.clone(),
            opened.replacen("value 7\n", "value 𝄞\n", 1),
            opened.replacen("value 7\n", "value ✓\n", 1),
            String::new(),
            opened.clone(),
        ];
        let mut index = Indexed::of(&opened);
        let mut since = Since::of(opened.clone(), &opened);
        assert!(!since.changed, "the opened text is unchanged");
        for step in steps {
            let old_len = index.text.len();
            let Some(edit) = index.edit(&step) else {
                continue;
            };
            since.edit(&edit, old_len, &index.text);
            assert_eq!(step != opened, since.changed, "after {} bytes", step.len());
        }
    }

    #[test]
    fn a_draft_opened_changed_reads_changed_until_an_edit_undoes_it() {
        let opened = String::from("a: 1\nb: 2\n");
        let draft = String::from("a: 1\nb: 3\n");
        let mut index = Indexed::of(&draft);
        let mut since = Since::of(opened.clone(), &draft);
        assert!(since.changed);
        let edit = index.edit(&opened).expect("the text changed");
        since.edit(&edit, draft.len(), &index.text);
        assert!(!since.changed);
    }

    #[test]
    fn the_panel_counts_the_lines_as_the_code_view_does() {
        for (old, new) in edits() {
            let mut typed = Typed::of(&old);
            typed.edit(&new);
            let measure = Typed::measure(&typed.index, None);
            assert_eq!(
                crate::view::code::line_count(&new),
                measure.lines,
                "{new:?}"
            );
            assert_eq!(new.trim().is_empty(), measure.blank, "{new:?}");
        }
        for text in ["", "\n", "a", "a\n", "a\nb", "a\n\n", "  \n "] {
            let measure = Typed::measure(&Indexed::of(text), None);
            assert_eq!(
                crate::view::code::line_count(text),
                measure.lines,
                "{text:?}"
            );
            assert_eq!(text.trim().is_empty(), measure.blank, "{text:?}");
        }
    }

    #[test]
    fn the_shared_bytes_are_counted_across_whole_blocks() {
        let text = "x".repeat(200);
        for at in [0, 1, 63, 64, 65, 128, 199] {
            let mut changed = text.clone().into_bytes();
            changed[at] = b'y';
            assert_eq!(at, shared_head(text.as_bytes(), &changed), "head at {at}");
            assert_eq!(
                199 - at,
                shared_tail(text.as_bytes(), &changed),
                "tail at {at}"
            );
        }
        assert_eq!(200, shared_head(text.as_bytes(), text.as_bytes()));
        assert_eq!(3, shared_tail(b"abcdef", b"def"));
    }

    #[test]
    fn the_caret_read_from_its_line_reads_as_from_the_start() {
        let text = "ab\nnaïve ✓ 𝄞x\n\nlast";
        let index = Indexed::of(text);
        for offset in 0..30 {
            assert_eq!(
                caret_at(text, offset),
                index.caret(offset),
                "offset {offset}"
            );
        }
        assert_eq!(super::units(text), index.len_units());
    }

    /// The layer `old` brought to `new` with `marks`, and how it changed.
    fn updated(language: Language, old: &str, new: &str, marks: &[LineMark]) -> (Layer, Redraw) {
        let mut layer = Layer::of(language, old, marks, None);
        let redraw = layer.update(language, new, marks, None);
        (layer, redraw)
    }

    #[test]
    fn an_edit_highlights_its_lines_as_the_whole_text_does() {
        let marks = [LineMark {
            line: 3,
            kind: MarkKind::Error,
        }];
        for (old, new) in edits() {
            for language in [Language::Yaml, Language::Plain] {
                let (layer, redraw) = updated(language, &old, &new, &marks);
                assert_eq!(
                    Layer::of(language, &new, &marks, None).lines,
                    layer.lines,
                    "{language:?} {old:?} to {new:?}"
                );
                assert!(
                    matches!(redraw, Redraw::Edit { exact: true, .. }),
                    "{redraw:?}"
                );
            }
        }
    }

    #[test]
    fn one_key_in_ten_thousand_lines_highlights_one_line_and_moves_no_mark() {
        let text = numbered(10_000);
        let typed = text.replacen("value 5000\n", "value 50000\n", 1);
        let marks = [LineMark {
            line: 9000,
            kind: MarkKind::Warning,
        }];
        let (layer, redraw) = updated(Language::Yaml, &text, &typed, &marks);
        let Redraw::Edit { patch, classes, .. } = redraw else {
            panic!("an edit is patched: {redraw:?}");
        };
        assert_eq!(
            Patch {
                head: 4999,
                removed: 1,
                inserted: 1,
                tail: 5001,
            },
            patch
        );
        assert!(classes.is_empty(), "no mark moved: {classes:?}");
        assert_eq!(
            Layer::of(Language::Yaml, &typed, &marks, None).lines,
            layer.lines
        );
    }

    #[test]
    fn a_new_line_renumbers_the_marks_after_it() {
        let text = numbered(100);
        let typed = text.replacen("value 10\n", "value 10\n\n", 1);
        let marks = [LineMark {
            line: 50,
            kind: MarkKind::Error,
        }];
        let (layer, redraw) = updated(Language::Yaml, &text, &typed, &marks);
        let Redraw::Edit { patch, classes, .. } = redraw else {
            panic!("an edit is patched: {redraw:?}");
        };
        assert_eq!(
            Patch {
                head: 10,
                removed: 0,
                inserted: 1,
                tail: 91,
            },
            patch
        );
        assert_eq!(vec![49, 50], classes, "the mark stays on line 50");
        assert_eq!(
            Layer::of(Language::Yaml, &typed, &marks, None).lines,
            layer.lines
        );
    }

    #[test]
    fn new_marks_on_the_same_text_change_only_their_lines() {
        let text = numbered(100);
        let mark = |line| {
            [LineMark {
                line,
                kind: MarkKind::Warning,
            }]
        };
        let mut layer = Layer::of(Language::Yaml, &text, &mark(10), None);
        let redraw = layer.update(Language::Yaml, &text, &mark(20), Some(20));
        let Redraw::Edit { patch, classes, .. } = redraw else {
            panic!("new marks are patched: {redraw:?}");
        };
        assert_eq!(0, patch.removed + patch.inserted, "no line is drawn again");
        assert_eq!(vec![9, 19], classes);
        assert_eq!(
            Layer::of(Language::Yaml, &text, &mark(20), Some(20)).lines,
            layer.lines
        );
    }

    #[test]
    fn a_grammar_that_reads_across_lines_is_highlighted_whole_once_typing_rests() {
        let text = "<a>\n<!-- one\ntwo -->\n</a>";
        let typed = text.replacen("two", "two three", 1);
        let (mut layer, redraw) = updated(Language::Xml, text, &typed, &[]);
        assert!(
            matches!(redraw, Redraw::Edit { exact: false, .. }),
            "{redraw:?}"
        );
        let old = layer.rehighlight();
        assert_eq!(4, old.len());
        assert_eq!(
            Layer::of(Language::Xml, &typed, &[], None).lines,
            layer.lines
        );
        assert!(
            Rc::ptr_eq(&old[0].body, &layer.lines[0].body),
            "a line that comes back alike is kept"
        );
    }

    #[test]
    fn a_text_that_crosses_the_highlighting_limit_is_made_again() {
        let long = "a: 1\n".repeat(crate::highlight::MAX_HIGHLIGHT_BYTES.div_ceil(5));
        let (layer, redraw) = updated(Language::Yaml, "a: 1\n", &long, &[]);
        let Redraw::Lines(old) = redraw else {
            panic!("the layer is made again: {redraw:?}");
        };
        assert_eq!(2, old.len(), "the lines it had");
        assert_eq!(
            Layer::of(Language::Yaml, &long, &[], None).lines,
            layer.lines
        );
    }

    #[test]
    fn a_console_that_answers_every_key_costs_no_hash() {
        let mut echoes = Echoes::default();
        for text in ["S", "SE", "SEL"] {
            echoes.typed(text);
            assert!(echoes.pending.is_empty(), "the newest text is kept whole");
            assert!(echoes.came_back(text));
        }
        assert_eq!(None, echoes.newest);
    }
}
