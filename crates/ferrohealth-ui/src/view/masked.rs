// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The hold-to-show button of a masked text input (R10): the eye inside
//! the field's edge that shows what was typed while it is held and masks
//! it again on release.
//!
//! The input is the console's, drawn masked by
//! [`crate::view::form::masked_input`]; the button sits beside it
//! ([`crate::view::form::Inner::HoldToShow`]) and names the input by its id
//! alone, so the typed value never leaves its input. Held by the pointer,
//! by touch, or by Space or Enter on the keyboard ([`holds`]), the button
//! switches the input to `type="text"` and says it is pressed
//! (`aria-pressed`, WAI-ARIA 1.2 `button`); released, left or blurred, it
//! switches it back to `type="password"`. Until the `masked` module runs,
//! the button is disabled and its name says the field stays masked. No
//! specification governs the hold gesture: our own design.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::form::INNER_BUTTON;

/// The words the hold-to-show button draws, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldWords {
    /// The button's name once it works, such as "Show the identifier
    /// while held".
    pub name: Word,
    /// The button's name until the `masked` module runs, which says the
    /// field stays masked.
    pub needs_script: Word,
}

impl HoldWords {
    /// The library's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            name: tr(locale, Text::HoldShow),
            needs_script: tr(locale, Text::HoldShowNeedsScript),
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
/// button pattern: Space and Enter press a button); the `masked` module
/// holds it by the same keys.
#[must_use]
pub fn holds(key: &str) -> bool {
    matches!(key, " " | "Enter")
}

#[derive(Template)]
#[template(
    source = r#"<button type="button" aria-label="{{ needs_script }}" aria-pressed="false" disabled data-fui="masked" data-fui-input="{{ input }}" data-fui-name="{{ name }}" class="{{ class }}"><span data-fui-shown hidden>{{ shut }}</span><span data-fui-masked>{{ eye }}</span></button>"#,
    ext = "html"
)]
struct HoldView {
    needs_script: String,
    input: String,
    name: String,
    class: String,
    shut: Html,
    eye: Html,
}

/// Returns the hold-to-show button of the masked input `input`, in
/// `words`.
#[must_use]
pub fn hold_to_show(input: String, words: HoldWords) -> Html {
    let HoldWords { name, needs_script } = words;
    HoldView {
        needs_script: needs_script.into(),
        input,
        name: name.into(),
        class: format!("{INNER_BUTTON} fui-input-hold"),
        shut: icon(Glyph::EYE_OFF, "16"),
        eye: icon(Glyph::EYE, "16"),
    }
    .html()
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
