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
//! text area is read-only until the page hydrates, so no keystroke is lost.
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

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::rc::Rc;

use leptos::prelude::*;
use leptos::web_sys::HtmlTextAreaElement;
use serde::{Deserialize, Serialize};

use crate::highlight::{Language, escape, lines_html, push_line};
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

/// What the layer draws: the grammar and the text it highlighted, and its
/// lines.
#[derive(Debug)]
struct Layer {
    /// The grammar the text was highlighted as.
    language: Language,
    /// The text it highlighted.
    text: String,
    /// Its lines.
    lines: Vec<LayerLine>,
}

impl Layer {
    /// The layer of `text` in `language`, marked by `marks` while the
    /// cursor's marked line is `current`.
    fn of(language: Language, text: &str, marks: &[LineMark], current: Option<u32>) -> Self {
        Self {
            language,
            text: text.to_owned(),
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
        if language != self.language || text != self.text {
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
            text: self.text.clone(),
            lines,
        }
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
    use leptos::wasm_bindgen::JsCast as _;
    let nodes = layer.child_nodes();
    if usize::try_from(nodes.length()).ok() != Some(old.len()) {
        let mut html = String::new();
        push_layer(&mut html, new, prefix, 1);
        layer.set_inner_html(&html);
        return;
    }
    let Patch {
        head,
        removed,
        inserted,
        tail,
    } = patch(old, new);
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
        let first = u32::try_from(head).map_or(u32::MAX, |head| head.saturating_add(1));
        let lines = new
            .get(head..head.saturating_add(inserted))
            .unwrap_or_default();
        push_layer(&mut html, lines, prefix, first);
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
    let shifted = old.len() != new.len();
    let old_tail = old.iter().skip(old.len().saturating_sub(tail));
    let new_start = new.len().saturating_sub(tail);
    for ((at, line), was) in (new_start..).zip(new.iter().skip(new_start)).zip(old_tail) {
        if !shifted && line.class == was.class {
            continue;
        }
        let Some(element) = nodes
            .item(index(at))
            .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
        else {
            continue;
        };
        if shifted {
            element.set_id(&format!("{prefix}{}", at.saturating_add(1)));
        }
        if line.class != was.class {
            let class = line.class.map_or_else(
                || String::from("fui-code-line"),
                |class| format!("fui-code-line {class}"),
            );
            element.set_class_name(&class);
        }
    }
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

/// How many texts the editor remembers handing to the console before the
/// console hands any back.
const ECHOES: usize = 64;

/// The texts the editor handed to the console that have not come back yet,
/// each kept as its hash, so a console that never hands one back holds the
/// editor to [`ECHOES`] numbers.
#[derive(Debug, Default)]
struct Echoes {
    /// The hashes of the texts handed over, oldest first.
    pending: VecDeque<u64>,
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
        if self.pending.len() >= ECHOES {
            self.pending.pop_front();
        }
        self.pending.push_back(Self::hash(text));
    }

    /// Whether the console's `text` is one the editor handed it, coming
    /// back; it forgets that text and every one handed before it. Any other
    /// text is an outside change, and it forgets them all.
    fn came_back(&mut self, text: &str) -> bool {
        let hash = Self::hash(text);
        if let Some(at) = self.pending.iter().position(|held| *held == hash) {
            self.pending.drain(..=at);
            true
        } else {
            self.pending.clear();
            false
        }
    }
}

/// Writes the console's `text` into the text area `area_ref` when it is an
/// outside change, and returns the record of the texts handed to the
/// console, which the text area's input fills.
fn follow_console(
    text: Signal<String>,
    area_ref: NodeRef<leptos::html::Textarea>,
) -> Rc<RefCell<Echoes>> {
    let echoes = Rc::new(RefCell::new(Echoes::default()));
    let heard = Rc::clone(&echoes);
    // NOTE: the text area is the source of truth while it is edited; the console's text is
    // written only as an outside change, never as the late echo of an earlier key (#333).
    Effect::new(move || {
        let console = text.get();
        if echoes.borrow_mut().came_back(&console) {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(area) = area_ref.get()
            && area.value() != console
        {
            area.set_value(&console);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let (_console, _area) = (console, area_ref);
    });
    heard
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
    let mut first = Some(drawn_layer);
    Effect::new(move |old: Option<Layer>| {
        let Some(old) = old.or_else(|| first.take()) else {
            return text.with(|text| Layer::of(language.get(), text, &marks.read(), current.get()));
        };
        let new = text.with(|text| old.next(language.get(), text, &marks.read(), current.get()));
        #[cfg(target_arch = "wasm32")]
        {
            let Some(element) = layer_ref.get() else {
                return old;
            };
            apply(&element, &prefix, &old.lines, &new.lines);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let (_old, _prefix) = (old, &prefix);
        new
    });
    let name = id.clone();
    let first = text.with_untracked(|text| server_text(text));
    let area_ref = NodeRef::<leptos::html::Textarea>::new();
    let heard = follow_console(text, area_ref);
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
                        let typed = event.target().value();
                        heard.borrow_mut().typed(&typed);
                        on_input(typed);
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
    let is_new = matches!(opened, Opened::New);
    let opened = StoredValue::new(opened);
    let dirty = move || {
        opened.with_value(|opened| match opened {
            Opened::New => false,
            Opened::Text(first) => text.with(|text| text != first),
        })
    };
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
    let Editor {
        id, text, marks, ..
    } = editor;
    let current = current_line(caret, marks);
    let on_line = Callback::new(move |at: Position| {
        caret.set(at);
        place_caret(id, text, at);
    });
    let rows = rows(file, words, current, Some(on_line));
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
        ECHOES, Echoes, Layer, LayerLine, LineMark, MarkKind, Patch, caret_at, layer_class,
        layer_lines, line_id, mark_class, mark_index, offset_of, patch, push_layer, server_text,
    };
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
}
