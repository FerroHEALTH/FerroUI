// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The copy button and a setting's key hint: each puts a text on the
//! clipboard and says so.
//!
//! It is a small secondary button with the `copy` glyph. Pressed, it writes
//! the text with the browser's asynchronous clipboard (W3C Clipboard API and
//! events, `writeText`), then reads "Copied" with a check for two seconds,
//! or says the browser refused, as it does on a page served over plain
//! HTTP, which has no clipboard; a polite live region announces either. A
//! page draws it through [`crate::island::CopyButton`]. The server draws it
//! hidden and it shows once the page has hydrated, so with no WebAssembly
//! the text it copies stays on the page as selectable text and no button
//! offers what only script can do (`leptos-ui.md` §15.8).
//!
//! What it copies is a [`CopySource`]: a short text carried as the island's
//! prop, or the text of an element the page already draws, which keeps a
//! large document out of the island's data so the page carries it once
//! less.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// How long "Copied" stays, in milliseconds.
pub const COPIED_MS: u64 = 2000;

/// The words the copy button draws itself, resolved on the server in the
/// page's locale, so an island that draws it carries no catalog.
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
    /// The copy button's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            copied: tr(Text::CopyCopied),
            said_copied: tr(Text::CopySaidCopied),
            said_refused: tr(Text::CopySaidRefused),
            refused: tr(Text::CopyRefused),
        }
    }
}

/// What a copy button puts on the clipboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopySource {
    /// This text, carried as the island's prop: for a short text such as
    /// a request line.
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

impl CopySource {
    /// The text the source names now, or `None` when the page holds no
    /// element of its id.
    #[cfg(target_arch = "wasm32")]
    fn read(&self) -> Option<String> {
        match self {
            Self::Text(text) => Some(text.clone()),
            Self::Element(id) => document()
                .get_element_by_id(id)
                .map(|element| element.text_content().unwrap_or_default()),
            Self::Lines { id, final_break } => {
                let holder = document().get_element_by_id(id)?;
                let lines = holder.query_selector_all(".fui-code-line").ok()?;
                let mut text = String::new();
                for index in 0..lines.length() {
                    if index > 0 {
                        text.push('\n');
                    }
                    if let Some(line) = lines.item(index) {
                        text.push_str(&line.text_content().unwrap_or_default());
                    }
                }
                if *final_break {
                    text.push('\n');
                }
                Some(text)
            }
        }
    }
}

/// What the copy button says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Ready to copy.
    Ready,
    /// The text is on the clipboard.
    Copied,
    /// The browser refused the clipboard.
    Refused,
}

/// What a drawn copy button copies: a text a signal holds, or a source.
enum Copied {
    /// The text the signal holds when pressed.
    Live(Signal<String>),
    /// What the source names when pressed.
    Source(CopySource),
}

/// Returns the copy button `id` labelled `label`, copying `text` when
/// pressed and saying what came of it in `words`.
#[must_use]
pub fn copy_button(
    id: impl Into<String>,
    label: impl Into<Word>,
    text: Signal<String>,
    words: CopyWords,
) -> impl IntoView {
    drawn(id.into(), label.into(), Copied::Live(text), words)
}

/// Returns the copy button `id` labelled `label`, copying what `source`
/// names when pressed and saying what came of it in `words`, as
/// [`crate::island::CopyButton`] draws it.
#[must_use]
pub fn copy_from(
    id: impl Into<String>,
    label: impl Into<Word>,
    source: CopySource,
    words: CopyWords,
) -> impl IntoView {
    drawn(id.into(), label.into(), Copied::Source(source), words)
}

/// Returns the copy button `id` labelled `label` over `what`, in `words`.
fn drawn(id: String, label: Word, what: Copied, words: CopyWords) -> impl IntoView {
    let state = RwSignal::new(State::Ready);
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let press = move |_| match &what {
        Copied::Live(text) => write(&text.get_untracked(), state),
        Copied::Source(source) => copy(source, state),
    };
    let CopyWords {
        copied,
        said_copied,
        said_refused,
        refused,
    } = words;
    let said = move || {
        match state.get() {
            State::Ready => Word::default(),
            State::Copied => said_copied.clone(),
            State::Refused => said_refused.clone(),
        }
        .into_render()
    };
    view! {
        <span class="fui-copy">
            <button
                type="button"
                id=id
                class=Variant::Secondary.small()
                hidden=move || !hydrated.get()
                on:click=press
            >
                {move || match state.get() {
                    State::Copied => {
                        view! {
                            {icon(Glyph::CHECK, "14")}
                            {copied.clone()}
                        }
                            .into_any()
                    }
                    State::Ready | State::Refused => {
                        view! {
                            {icon(Glyph::COPY, "14")}
                            {label.clone()}
                        }
                            .into_any()
                    }
                }}
            </button>
            <span class="fui-sr-only" role="status">
                {said}
            </span>
            {move || {
                (state.get() == State::Refused)
                    .then(|| {
                        view! {
                            <span class="fui-copy-refused" aria-hidden="true">
                                {refused.clone()}
                            </span>
                        }
                    })
            }}
        </span>
    }
}

/// The words a key hint draws, resolved on the server in the page's
/// locale, so the island carries no catalog.
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
    /// The words of the hint of `key` in the locale of the page being drawn.
    #[must_use]
    pub fn of_page(key: &str) -> Self {
        Self {
            label: tr_with(Text::KeyCopy, &[("key", key.into())]),
            title: tr(Text::KeyCopyTitle),
            copied: tr(Text::CopySaidCopied),
            refused: tr(Text::CopySaidRefused),
        }
    }
}

/// Returns the key hint of a setting: its TOML `key` in the code face, a
/// button named "Copy the key" and the key that copies it when pressed
/// (R15); a page draws it through [`crate::island::KeyHint`].
///
/// It sits on the setting's label line. The stylesheet shows it while the
/// row is hovered or holds the focus, and always below 40 rem, where no
/// pointer hovers; it keeps its room while hidden, so nothing moves. Until
/// the page has hydrated the key is selectable text in the hint's place
/// and the button is hidden, as the copy button is.
#[must_use]
pub fn key_hint(key: String, words: KeyHintWords) -> impl IntoView {
    let KeyHintWords {
        label,
        title,
        copied: said_copied,
        refused: said_refused,
    } = words;
    let state = RwSignal::new(State::Ready);
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let copied = key.clone();
    let press = move |_| write(&copied, state);
    let said = move || {
        match state.get() {
            State::Ready => Word::default(),
            State::Copied => said_copied.clone(),
            State::Refused => said_refused.clone(),
        }
        .into_render()
    };
    let glyph = move || {
        if state.get() == State::Copied {
            icon(Glyph::CHECK, "12")
        } else {
            icon(Glyph::COPY, "12")
        }
    };
    let text = key.clone();
    view! {
        <span class="fui-key-hint is-text" hidden=move || hydrated.get()>
            <code>{text}</code>
        </span>
        <button
            type="button"
            class="fui-key-hint"
            aria-label=label
            title=title
            hidden=move || !hydrated.get()
            on:click=press
        >
            <code>{key}</code>
            {glyph}
        </button>
        <span class="fui-sr-only" role="status">
            {said}
        </span>
    }
}

/// Writes what `source` names to the clipboard and sets `state` to what
/// came of it; a source the page no longer holds is refused.
fn copy(source: &CopySource, state: RwSignal<State>) {
    #[cfg(target_arch = "wasm32")]
    match source.read() {
        Some(value) => write(&value, state),
        None => state.set(State::Refused),
    }
    #[cfg(not(target_arch = "wasm32"))]
    let (_source, _state): (&CopySource, RwSignal<State>) = (source, state);
}

/// Writes `value` to the clipboard from a menu's copy item and says what
/// came of it in a toast, in `words`: the menu closes as it is chosen, so
/// the toast region's live region speaks for it.
#[cfg(target_arch = "wasm32")]
pub(crate) fn copy_and_say(value: &str, words: &CopyWords) {
    {
        let said_copied = String::from(words.said_copied.clone());
        let said_refused = String::from(words.said_refused.clone());
        let refused = move || {
            crate::widget::toast::raise(crate::widget::toast::Toast::new(
                crate::tone::Tone::Bad,
                said_refused,
            ));
        };
        let Some(clipboard) = clipboard() else {
            refused();
            return;
        };
        let promise = clipboard.write_text(value);
        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(_) => crate::widget::toast::raise(crate::widget::toast::Toast::new(
                    crate::tone::Tone::Ok,
                    said_copied,
                )),
                Err(_) => refused(),
            }
        });
    }
}

/// The browser's clipboard, or `None` where it has none.
#[cfg(target_arch = "wasm32")]
fn clipboard() -> Option<web_sys::Clipboard> {
    use leptos::wasm_bindgen::JsCast as _;
    // NOTE: W3C Clipboard API and events, `Navigator.clipboard` is
    // [SecureContext]: over plain HTTP it is undefined, which is a refusal too.
    web_sys::window()
        .map(|window| window.navigator())
        .and_then(|navigator| web_sys::js_sys::Reflect::get(&navigator, &"clipboard".into()).ok())
        .filter(|clipboard| !clipboard.is_undefined() && !clipboard.is_null())
        .and_then(|clipboard| clipboard.dyn_into::<web_sys::Clipboard>().ok())
}

/// Writes `value` to the clipboard and sets `state` to what came of it.
fn write(value: &str, state: RwSignal<State>) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(clipboard) = clipboard() else {
            state.set(State::Refused);
            return;
        };
        let promise = clipboard.write_text(value);
        wasm_bindgen_futures::spawn_local(async move {
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(_) => {
                    state.set(State::Copied);
                    set_timeout(
                        move || state.set(State::Ready),
                        std::time::Duration::from_millis(COPIED_MS),
                    );
                }
                Err(_) => state.set(State::Refused),
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let (_value, _state): (&str, RwSignal<State>) = (value, state);
}
