// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The hold-to-show button of a masked text input (R10): the eye inside
//! the field's edge that shows what was typed while it is held and masks
//! it again on release.
//!
//! The input is the console's, drawn masked by
//! [`crate::view::form::masked_input`]; the button is the island beside it
//! ([`crate::island::HoldToShow`], placed by
//! [`crate::view::form::Inner::HoldToShow`]) and names the input by its id
//! alone, so the typed value is never a prop and never reaches the page's
//! data. Held by the pointer, by touch, or by Space or Enter on the
//! keyboard, the button switches the input to `type="text"` and says it is
//! pressed (`aria-pressed`, WAI-ARIA 1.2 `button`); released, left or
//! blurred, it switches it back to `type="password"`. Until the page's
//! WebAssembly runs, the button is disabled and its name says the field
//! stays masked (`leptos-ui.md` §15.8). No specification governs the
//! hold gesture: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::form::INNER_BUTTON;

/// The words the hold-to-show button draws, resolved on the server in the
/// page's locale, so its island carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldWords {
    /// The button's name once it works, such as "Show the identifier
    /// while held".
    pub name: Word,
    /// The button's name until the page's WebAssembly runs, which says the
    /// field stays masked.
    pub needs_script: Word,
}

impl HoldWords {
    /// The library's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            name: tr(Text::HoldShow),
            needs_script: tr(Text::HoldShowNeedsScript),
        }
    }

    /// These words, the button named `name`, which says what the field
    /// holds.
    #[must_use]
    pub fn named(self, name: impl Into<Word>) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }
}

/// Whether `key` holds the button down (WAI-ARIA Authoring Practices,
/// button pattern: Space and Enter press a button).
fn holds(key: &str) -> bool {
    matches!(key, " " | "Enter")
}

/// Returns the hold-to-show button of the masked input `input`, in
/// `words`, as [`crate::island::HoldToShow`] draws it.
#[must_use]
pub fn hold_to_show(input: String, words: HoldWords) -> impl IntoView {
    let shown = RwSignal::new(false);
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let input = StoredValue::new(input);
    Effect::new(move |_| {
        let on = shown.get();
        input.with_value(|input| show(input, on));
    });
    let HoldWords { name, needs_script } = words;
    let label = move || {
        String::from(if hydrated.get() {
            name.clone()
        } else {
            needs_script.clone()
        })
    };
    view! {
        <button
            type="button"
            class=format!("{INNER_BUTTON} fui-input-hold")
            aria-label=label
            aria-pressed=move || shown.get().to_string()
            disabled=move || !hydrated.get()
            on:pointerdown=move |event| {
                if event.button() == 0 {
                    shown.set(true);
                }
            }
            on:pointerup=move |_| shown.set(false)
            on:pointerleave=move |_| shown.set(false)
            on:pointercancel=move |_| shown.set(false)
            on:contextmenu=move |event| event.prevent_default()
            on:keydown=move |event| {
                if holds(&event.key()) {
                    event.prevent_default();
                    shown.set(true);
                }
            }
            on:keyup=move |event| {
                if holds(&event.key()) {
                    event.prevent_default();
                    shown.set(false);
                }
            }
            on:blur=move |_| shown.set(false)
        >
            {move || icon(if shown.get() { Glyph::EYE_OFF } else { Glyph::EYE }, "16")}
        </button>
    }
}

/// Shows the text of the input `input` when `on`, and masks it otherwise.
fn show(input: &str, on: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        if let Some(field) = document()
            .get_element_by_id(input)
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
        {
            field.set_type(if on { "text" } else { "password" });
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let (_input, _on): (&str, bool) = (input, on);
}

#[cfg(test)]
mod tests {
    use super::holds;

    #[test]
    fn space_and_enter_hold_the_button_and_other_keys_do_not() {
        assert!(holds(" "));
        assert!(holds("Enter"));
        assert!(!holds("Tab"));
        assert!(!holds("Escape"));
    }
}
