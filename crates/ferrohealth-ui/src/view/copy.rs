// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The copy button and a setting's key hint: each puts a text on the
//! clipboard and says so.
//!
//! It is a small secondary button with the `copy` glyph. Pressed, it writes
//! the text with the browser's asynchronous clipboard (W3C Clipboard API and
//! events, `writeText`), then reads "Copied" with a check for two seconds,
//! or says the browser refused, as it does on a page served over plain
//! HTTP, which has no clipboard; a polite live region announces either.
//! The server draws it hidden and the `copy` module shows it, so with no
//! script the text it copies stays on the page as selectable text and no
//! button offers what only script can do.
//!
//! What it copies is a [`CopySource`]: a short text the button carries, or
//! the text of an element the page already draws, which keeps a large
//! document in the page once.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// How long "Copied" stays, in milliseconds.
pub const COPIED_MS: u64 = 2000;

/// The words the copy button draws itself, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyWords {
    /// The button's words once the text is on the clipboard.
    pub copied: Word,
    /// What the live region says once the text is on the clipboard.
    pub said_copied: Word,
    /// What the live region says when the browser refused.
    pub said_refused: Word,
    /// The line beside the button when the browser refused.
    pub refused: Word,
}

impl CopyWords {
    /// The copy button's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            copied: tr(locale, Text::CopyCopied),
            said_copied: tr(locale, Text::CopySaidCopied),
            said_refused: tr(locale, Text::CopySaidRefused),
            refused: tr(locale, Text::CopyRefused),
        }
    }
}

/// What a copy button puts on the clipboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopySource {
    /// This text, carried by the button: for a short text such as a
    /// request line.
    Text(String),
    /// The text of the element with this id, as the page draws it: for a
    /// document the page shows whole, such as a raw view's `<pre>`.
    Element(String),
    /// The code lines (`fui-code-line`) inside the element with this id,
    /// one per line, with a final line break when `final_break` is true:
    /// for a code viewer, whose diagnostics sit between its lines.
    Lines {
        /// The id of the element that holds the lines.
        id: String,
        /// Whether the text ends in a line break, which opens no line.
        final_break: bool,
    },
}

#[derive(Template)]
#[template(path = "copy/copy.html", block = "button")]
struct ButtonView {
    source: CopySource,
    id: String,
    class: &'static str,
    copy_glyph: Html,
    label: Word,
    check_glyph: Html,
    copied: Word,
    said_copied: String,
    said_refused: String,
    refused: Word,
}

/// Returns the copy button `id` labelled `label`, copying what `source`
/// names when pressed and saying what came of it in `words`.
#[must_use]
pub fn copy_from(
    id: impl Into<String>,
    label: impl Into<Word>,
    source: CopySource,
    words: CopyWords,
) -> Html {
    let CopyWords {
        copied,
        said_copied,
        said_refused,
        refused,
    } = words;
    ButtonView {
        source,
        id: id.into(),
        class: Variant::Secondary.small(),
        copy_glyph: icon(Glyph::COPY, "14"),
        label: label.into(),
        check_glyph: icon(Glyph::CHECK, "14"),
        copied,
        said_copied: said_copied.text().to_owned(),
        said_refused: said_refused.text().to_owned(),
        refused,
    }
    .html()
}

/// Returns the copy button `id` labelled `label`, copying `text` when
/// pressed, in `locale`'s words: [`copy_from`] over [`CopySource::Text`].
#[must_use]
pub fn copy_button(
    locale: &Locale,
    id: impl Into<String>,
    label: impl Into<Word>,
    text: impl Into<String>,
) -> Html {
    copy_from(
        id,
        label,
        CopySource::Text(text.into()),
        CopyWords::of(locale),
    )
}

/// The words a key hint draws, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyHintWords {
    /// The button's name: "Copy the key" and the key.
    pub label: Word,
    /// The button's tooltip.
    pub title: Word,
    /// What the live region says once the key is copied.
    pub copied: Word,
    /// What the live region says when the browser refuses.
    pub refused: Word,
}

impl KeyHintWords {
    /// The words of the hint of `key` in `locale`.
    #[must_use]
    pub fn of(locale: &Locale, key: &str) -> Self {
        Self {
            label: tr_with(locale, Text::KeyCopy, &[("key", key.into())]),
            title: tr(locale, Text::KeyCopyTitle),
            copied: tr(locale, Text::CopySaidCopied),
            refused: tr(locale, Text::CopySaidRefused),
        }
    }
}

#[derive(Template)]
#[template(path = "copy/copy.html", block = "key_hint")]
struct KeyHintView {
    key: String,
    label: String,
    title: String,
    copy_glyph: Html,
    check_glyph: Html,
    copied: String,
    refused: String,
}

/// Returns the key hint of a setting: its TOML `key` in the code face, a
/// button named "Copy the key" and the key that copies it when pressed
/// (R15).
///
/// It sits on the setting's label line. The stylesheet shows it while the
/// row is hovered or holds the focus, and always below 40 rem, where no
/// pointer hovers; it keeps its room while hidden, so nothing moves. Until
/// the `copy` module runs the key is selectable text in the hint's place
/// and the button is hidden, as the copy button is.
#[must_use]
pub fn key_hint(key: String, words: KeyHintWords) -> Html {
    let KeyHintWords {
        label,
        title,
        copied,
        refused,
    } = words;
    KeyHintView {
        key,
        label: label.into(),
        title: title.into(),
        copy_glyph: icon(Glyph::COPY, "12"),
        check_glyph: icon(Glyph::CHECK, "12"),
        copied: copied.into(),
        refused: refused.into(),
    }
    .html()
}
