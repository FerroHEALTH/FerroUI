// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The copy button and a setting's key hint: each puts a text on the
//! clipboard and says so.
//!
//! It is a small secondary button with the `copy` glyph. Pressed, it writes
//! the text with the browser's asynchronous clipboard (W3C Clipboard API and
//! events, `writeText`), then reads "Copied" with a check for two seconds,
//! or says the browser refused; a polite live region announces either. The
//! server draws it disabled, and it turns on once the page has hydrated, so
//! a press before then is never lost silently.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// How long "Copied" stays, in milliseconds.
pub const COPIED_MS: u64 = 2000;

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

/// Returns the copy button `id` labelled `label`, copying `text` when
/// pressed.
#[must_use]
pub fn copy_button(
    id: impl Into<String>,
    label: impl Into<Oco<'static, str>>,
    text: Signal<String>,
) -> impl IntoView {
    let id = id.into();
    let label = label.into();
    let state = RwSignal::new(State::Ready);
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let press = move |_| {
        let value = text.get_untracked();
        write(&value, state);
    };
    // The words are read once, while the page's locale is in reach, and
    // drawn as the state moves.
    let said_copied = tr(Text::CopySaidCopied);
    let said_refused = tr(Text::CopySaidRefused);
    let copied = tr(Text::CopyCopied);
    let refused = tr(Text::CopyRefused);
    let said = move || match state.get() {
        State::Ready => String::new(),
        State::Copied => said_copied.clone(),
        State::Refused => said_refused.clone(),
    };
    view! {
        <span class="fui-copy">
            <button
                type="button"
                id=id
                class=Variant::Secondary.small()
                disabled=move || !hydrated.get()
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
    pub label: String,
    /// The button's tooltip.
    pub title: String,
    /// What the live region says once the key is copied.
    pub copied: String,
    /// What the live region says when the browser refuses.
    pub refused: String,
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
/// pointer hovers; it keeps its room while hidden, so nothing moves. The
/// server draws it disabled and it turns on once the page has hydrated, as
/// the copy button does; the key reads as text before then.
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
    let said = move || match state.get() {
        State::Ready => String::new(),
        State::Copied => said_copied.clone(),
        State::Refused => said_refused.clone(),
    };
    let glyph = move || {
        if state.get() == State::Copied {
            icon(Glyph::CHECK, "12")
        } else {
            icon(Glyph::COPY, "12")
        }
    };
    view! {
        <button
            type="button"
            class="fui-key-hint"
            aria-label=label
            title=title
            disabled=move || !hydrated.get()
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

/// Writes `value` to the clipboard and sets `state` to what came of it.
fn write(value: &str, state: RwSignal<State>) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            state.set(State::Refused);
            return;
        };
        let promise = window.navigator().clipboard().write_text(value);
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
