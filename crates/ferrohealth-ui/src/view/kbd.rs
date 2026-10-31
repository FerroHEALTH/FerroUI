// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keys of a keyboard shortcut: the key cap, a shortcut and the words of
//! what it does, and the one-line hint at the foot of a listing (R1).
//!
//! A key is drawn as it is printed on the keyboard, in a `<kbd>` element
//! (HTML Living Standard, the `kbd` element): a letter as a capital, an
//! arrow as its arrow. The `keys` module binds a key a control announces
//! through [`aria_keys`] ([`crate::view::shortcuts`]), so a console writes
//! no listener of its own. Below 40 rem every key cap and hint is hidden,
//! since a phone has no keyboard (family rule 6); the shortcut legend keeps
//! its keys.
//!
//! A shortcut's keys are written as words separated by spaces, each word a
//! key pressed in turn or held together with a modifier (`g d`, `Ctrl s`),
//! and ` | ` separates the alternatives of one shortcut (`j | down`). No
//! specification governs the notation: our own design.

use serde::{Deserialize, Serialize};

use crate::html::Html;
use crate::i18n::{Locale, Text, Word, tr};

/// A shortcut of the page and the words of what it does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    /// The keys, such as `?`, `g d` or `j | down`.
    pub keys: String,
    /// What it does, in the page's locale, such as "Next row".
    pub does: String,
}

/// The shortcuts of one part of a page under its title, such as "Lists".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutGroup {
    /// The group's title, in the page's locale.
    pub title: String,
    /// Its shortcuts, in the order the legend lists them.
    pub shortcuts: Vec<Shortcut>,
}

/// The modifier keys, as `aria-keyshortcuts` names them, by the word a
/// shortcut writes for each.
const MODIFIERS: [(&str, &str); 5] = [
    ("ctrl", "Control"),
    ("control", "Control"),
    ("shift", "Shift"),
    ("alt", "Alt"),
    ("meta", "Meta"),
];

/// The key `word` names, as printed on the keyboard: a letter as a capital,
/// an arrow as its arrow, a named key by the word on its cap.
#[must_use]
pub fn key_cap(word: &str) -> String {
    let mut letters = word.chars();
    if let (Some(letter), None) = (letters.next(), letters.next())
        && letter.is_alphabetic()
    {
        return letter.to_uppercase().collect();
    }
    // NOTE: no specification governs this: our own design; each word is the
    // name printed on the key, which a catalog does not translate.
    match word.to_ascii_lowercase().as_str() {
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "esc" | "escape" => "Esc",
        "enter" => "Enter",
        "tab" => "Tab",
        "space" => "Space",
        "ctrl" | "control" => "Ctrl",
        "shift" => "Shift",
        "alt" => "Alt",
        "meta" => "Meta",
        _ => word,
    }
    .to_owned()
}

/// The key `word` names, as the `aria-keyshortcuts` attribute writes it
/// (WAI-ARIA 1.2, `aria-keyshortcuts`).
fn aria_key(word: &str) -> String {
    let mut letters = word.chars();
    if let (Some(letter), None) = (letters.next(), letters.next())
        && letter.is_alphabetic()
    {
        return letter.to_uppercase().collect();
    }
    match word.to_ascii_lowercase().as_str() {
        "up" => "ArrowUp",
        "down" => "ArrowDown",
        "left" => "ArrowLeft",
        "right" => "ArrowRight",
        "esc" | "escape" => "Escape",
        "enter" => "Enter",
        "tab" => "Tab",
        "space" => "Space",
        _ => word,
    }
    .to_owned()
}

/// Returns `keys` as the `aria-keyshortcuts` attribute writes them: each
/// alternative separated by a space, a modifier joined to its key by `+`.
///
/// A shortcut of keys pressed in turn (`g d`) has no form in the attribute,
/// which names keys pressed together, so it gives `None`, and so does a
/// shortcut with no key.
#[must_use]
pub fn aria_keys(keys: &str) -> Option<String> {
    let mut alternatives = Vec::new();
    for alternative in keys.split('|') {
        let mut held = Vec::new();
        let mut key = None;
        for word in alternative.split_whitespace() {
            let lower = word.to_ascii_lowercase();
            if let Some((_, name)) = MODIFIERS.iter().find(|(said, _)| *said == lower) {
                held.push(*name);
            } else if key.is_some() {
                return None;
            } else {
                key = Some(aria_key(word));
            }
        }
        let key = key?;
        let mut written = held.join("+");
        if !written.is_empty() {
            written.push('+');
        }
        written.push_str(&key);
        alternatives.push(written);
    }
    Some(alternatives.join(" "))
}

/// Returns the key caps of `keys`: one cap for a key, and a cap per key for
/// a shortcut of several, such as `g d` or `Ctrl s`.
///
/// It draws one alternative; [`shortcut_keys`] draws them all.
#[must_use]
pub fn kbd(keys: &str) -> Html {
    keys.split_whitespace()
        .map(|word| {
            Html::raw(format!(
                "<kbd class=\"fui-kbd\">{}</kbd>",
                Html::text(key_cap(word))
            ))
        })
        .collect()
}

/// Returns the keys of `keys` with each alternative's caps, the word `or`
/// between them, such as "J or ↓".
#[must_use]
pub fn shortcut_keys(keys: &str, or: impl Into<Word>) -> Html {
    let or: Word = or.into();
    let alternatives: Vec<&str> = keys
        .split('|')
        .map(str::trim)
        .filter(|alternative| !alternative.is_empty())
        .collect();
    let last = alternatives.len().saturating_sub(1);
    alternatives
        .into_iter()
        .enumerate()
        .map(|(place, alternative)| {
            let mut drawn = kbd(alternative);
            if place < last {
                drawn = drawn.then(Html::raw(format!("<span class=\"fui-muted\">{or}</span>")));
            }
            drawn
        })
        .collect()
}

/// Returns the one-line hint at the foot of a listing, each shortcut's keys
/// before the words of what it does, "or" between the alternatives of one.
///
/// The hint is hidden from a reader (`aria-hidden`): each control it names
/// announces its own shortcut through `aria-keyshortcuts`. It is hidden
/// below 40 rem, and a console leaves it out where the person turned the
/// hints off in Your preferences.
#[must_use]
pub fn shortcut_hint(locale: &Locale, shortcuts: &[Shortcut]) -> Html {
    let or = tr(locale, Text::KeysOr);
    let items: Html = shortcuts
        .iter()
        .map(|shortcut| {
            Html::raw(format!(
                "<span>{}{}</span>",
                shortcut_keys(&shortcut.keys, &or),
                Html::text(&shortcut.does)
            ))
        })
        .collect();
    Html::raw(format!(
        "<p class=\"fui-shortcut-hint\" aria-hidden=\"true\">{items}</p>"
    ))
}

#[cfg(test)]
mod tests {
    use super::{aria_keys, kbd, key_cap};

    #[test]
    fn a_letter_is_a_capital_and_an_arrow_its_arrow() {
        assert_eq!("J", key_cap("j"));
        assert_eq!("↓", key_cap("down"));
        assert_eq!("Esc", key_cap("Esc"));
        assert_eq!("?", key_cap("?"));
        assert_eq!("/", key_cap("/"));
        assert_eq!(
            "<kbd class=\"fui-kbd\">G</kbd><kbd class=\"fui-kbd\">D</kbd>",
            kbd("g d").as_str()
        );
    }

    #[test]
    fn the_keyshortcuts_name_each_alternative_and_join_a_modifier() {
        assert_eq!(Some(String::from("J ArrowDown")), aria_keys("j | down"));
        assert_eq!(Some(String::from("Control+S")), aria_keys("Ctrl s"));
        assert_eq!(Some(String::from("Escape")), aria_keys("Esc"));
        assert_eq!(Some(String::from("/")), aria_keys("/"));
    }

    #[test]
    fn keys_pressed_in_turn_have_no_keyshortcuts() {
        assert_eq!(None, aria_keys("g d"));
        assert_eq!(None, aria_keys(""));
    }
}
