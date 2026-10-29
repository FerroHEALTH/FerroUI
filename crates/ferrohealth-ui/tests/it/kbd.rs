// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keys of a shortcut (#141): the key cap and a chord, the one-line
//! hint, the legend dialog over the page, and the rule that hides every key
//! on a phone.

use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::view::kbd::{Shortcut, ShortcutGroup, kbd, shortcut_hint, shortcut_keys};
use ferrohealth_ui::view::shortcuts::shortcut_legend;

use crate::markup::{has, tags};
use crate::render::en;

/// A shortcut of `keys` that does `does`.
fn shortcut(keys: &str, does: &str) -> Shortcut {
    Shortcut {
        keys: String::from(keys),
        does: String::from(does),
    }
}

#[test]
fn a_key_is_one_cap_as_printed_and_a_chord_a_cap_per_key() {
    let key = kbd("j").to_string();
    assert_eq!(1, tags(&key, "kbd").len(), "{key}");
    assert!(key.contains(r#"<kbd class="fui-kbd">J</kbd>"#), "{key}");
    let chord = kbd("g d").to_string();
    assert_eq!(2, tags(&chord, "kbd").len(), "{chord}");
    assert!(
        chord.contains(">G</kbd>") && chord.contains(">D</kbd>"),
        "{chord}"
    );
    let held = kbd("Ctrl s").to_string();
    assert!(
        held.contains(">Ctrl</kbd>") && held.contains(">S</kbd>"),
        "{held}"
    );
}

#[test]
fn alternative_keys_are_joined_by_the_word_or() {
    let keys = shortcut_keys("j | down", "or").to_string();
    assert!(keys.contains(">J</kbd>"), "{keys}");
    assert!(keys.contains(">↓</kbd>"), "{keys}");
    assert!(
        keys.contains(r#"<span class="fui-muted">or</span>"#),
        "{keys}"
    );
}

#[test]
fn the_hint_is_one_line_hidden_from_a_reader_with_its_words_escaped() {
    let hint = shortcut_hint(
        &en(),
        &[shortcut("/", "Find <a> file"), shortcut("?", "Keys")],
    )
    .to_string();
    assert!(
        has(
            &hint,
            "p",
            &[("class", "fui-shortcut-hint"), ("aria-hidden", "true")]
        ),
        "{hint}"
    );
    assert_eq!(1, tags(&hint, "p").len(), "one line: {hint}");
    assert_eq!(2, tags(&hint, "span").len(), "{hint}");
    assert!(hint.contains("Find &#60;a&#62; file"), "{hint}");
}

#[test]
fn the_legend_is_a_dialog_listing_each_group_of_keys_and_words() {
    let legend = shortcut_legend(
        &en(),
        vec![
            ShortcutGroup {
                title: String::from("Lists"),
                shortcuts: vec![
                    shortcut("j | down", "Next row"),
                    shortcut("k | up", "Previous row"),
                ],
            },
            ShortcutGroup {
                title: String::from("Patient"),
                shortcuts: vec![shortcut("m", "Merge into…")],
            },
        ],
    )
    .to_string();
    assert!(
        has(
            &legend,
            "dialog",
            &[("id", "shortcuts"), ("aria-labelledby", "shortcuts-title")]
        ),
        "{legend}"
    );
    assert!(
        tags(&legend, "dialog")
            .iter()
            .all(|dialog| !dialog.contains_key("open")),
        "the server draws it closed: {legend}"
    );
    assert!(legend.contains("Keyboard shortcuts"), "{legend}");
    assert!(
        legend.contains("A shortcut acts only when no field has the focus."),
        "{legend}"
    );
    assert_eq!(2, tags(&legend, "section").len(), "{legend}");
    assert!(
        has(
            &legend,
            "section",
            &[("aria-labelledby", "shortcuts-group-2")]
        ),
        "a group of one renders: {legend}"
    );
    assert_eq!(3, tags(&legend, "dt").len(), "{legend}");
    assert!(
        legend.contains(">J</kbd>") && legend.contains(">↓</kbd>"),
        "{legend}"
    );
    assert!(legend.contains("Merge into…"), "{legend}");
    assert!(!legend.contains("style="), "{legend}");
}

#[test]
fn no_key_cap_or_hint_shows_on_a_phone_but_the_legend_keeps_its_keys() {
    let rule = "  @media (width < 40rem) {\n    .fui-kbd:not(.fui-shortcuts .fui-kbd),\n    .fui-shortcut-hint {\n      display: none !important;";
    assert!(
        STYLESHEET.contains(rule),
        "a key shows on a phone, whatever the preference"
    );
    let touch = "  @media (hover: none) and (pointer: coarse) {\n    :root:not([data-hints=\"always\"]) :is(.fui-kbd:not(.fui-shortcuts .fui-kbd), .fui-shortcut-hint) {\n      display: none !important;";
    assert!(
        STYLESHEET.contains(touch),
        "a key shows on a touch-only device unless the reader asked for every wide screen"
    );
}

#[test]
fn the_shortcut_hints_preference_never_takes_every_hint_but_the_legend_s_away() {
    let never = "  :root[data-hints=\"never\"] :is(.fui-kbd:not(.fui-shortcuts .fui-kbd), .fui-shortcut-hint) {\n    display: none !important;";
    assert!(STYLESHEET.contains(never), "Never keeps the hints");
}
