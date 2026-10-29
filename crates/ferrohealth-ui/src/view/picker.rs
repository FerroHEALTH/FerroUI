// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The picker field: choose from what the server holds, or type another.
//!
//! A `<select>` with no `name` (it submits nothing) offers the empty choice,
//! the console's choices and "another"; the text input beside it carries the
//! `name` and is what the form submits. Choosing an option writes its value
//! into the input; choosing "another" shows the input for typing, and moves
//! the focus to it only when chosen by pointer or by Enter: a closed select
//! that changes at each arrow key, as on Windows and Linux, keeps the focus
//! while the reader arrows through it (WCAG 2.2 SC 3.2.2). Both controls
//! carry a visible label. A value that is not among the choices opens on
//! the input and stays there, so a reader's own value never snaps back to
//! the list. The server marks the choice and fills the input; the `picker`
//! module does the rest. "Another" is the last option, known by its place
//! and marked `data-other`, so it reserves no value a choice could take.
//! With no script both controls show and the form posts the input, so
//! nothing depends on script.

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::view::form::{INPUT, SELECT, help_id};
use crate::view::help::help_line;

/// Which option of the select stands for the input's value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Selection {
    /// The empty choice: no value.
    Empty,
    /// A listed choice, by the console's value.
    Listed(String),
    /// "Another", which opens the input. It is the select's last option,
    /// told apart by its place and its `data-other` marker, never by a
    /// value, so no value a console passes can take it.
    Other,
}

/// The place among the select's options of `selection` over `choices`:
/// the empty choice first, then the choices, then "another".
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the picker module counts the place in the browser"
    )
)]
fn place(selection: &Selection, choices: &[(String, String)]) -> i32 {
    let at = match selection {
        Selection::Empty => 0,
        Selection::Listed(value) => choices
            .iter()
            .position(|(choice, _)| choice == value)
            .map_or(0, |at| at.saturating_add(1)),
        Selection::Other => choices.len().saturating_add(1),
    };
    i32::try_from(at).unwrap_or(i32::MAX)
}

/// What a picker field shows.
#[derive(Debug, Clone)]
pub struct Picker {
    /// The text input's id; the select's is this with `-choice`.
    pub id: String,
    /// The name the input submits as.
    pub name: String,
    /// The input's label, such as "Value set".
    pub label: Cow<'static, str>,
    /// The select's label, read by a screen reader, such as "Choose a value
    /// set this server holds".
    pub choose_label: Cow<'static, str>,
    /// The help line under both, which both name, under the form's help
    /// switch ([`crate::view::help::help_switch`]) when the form has one.
    pub help: Option<Cow<'static, str>>,
    /// The choices, value and words.
    pub choices: Vec<(String, String)>,
    /// The value the input holds.
    pub value: String,
    /// The empty choice's words, such as "Choose one this server holds".
    pub empty_word: Cow<'static, str>,
    /// The words of the choice that opens the input, such as "Another
    /// canonical".
    pub other_word: Cow<'static, str>,
}

/// The option for `value` among `choices`: its listed choice when it is
/// listed, the empty choice for no value, and "another" otherwise.
fn chosen(value: &str, choices: &[(String, String)]) -> Selection {
    if value.is_empty() {
        Selection::Empty
    } else if choices.iter().any(|(choice, _)| choice == value) {
        Selection::Listed(value.to_owned())
    } else {
        Selection::Other
    }
}

#[derive(Template)]
#[template(path = "picker/picker.html", block = "picker")]
struct PickerView {
    id: String,
    name: String,
    choose_label: Cow<'static, str>,
    label: Cow<'static, str>,
    described: Option<String>,
    select: &'static str,
    input: &'static str,
    selection: Selection,
    empty_word: Cow<'static, str>,
    choices: Vec<(String, String)>,
    other_word: Cow<'static, str>,
    value: String,
    help: Html,
}

/// Returns the picker field `picker`.
#[must_use]
pub fn picker_field(picker: Picker) -> Html {
    let Picker {
        id,
        name,
        label,
        choose_label,
        help,
        choices,
        value,
        empty_word,
        other_word,
    } = picker;
    let described = help.as_ref().map(|_| help_id(&id));
    let help = help.map(|help| help_line(&id, help)).unwrap_or_default();
    PickerView {
        selection: chosen(&value, &choices),
        id,
        name,
        choose_label,
        label,
        described,
        select: SELECT,
        input: INPUT,
        empty_word,
        choices,
        other_word,
        value,
        help,
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{Selection, chosen, place};

    #[test]
    fn a_listed_value_is_chosen_and_any_other_opens_the_input() {
        let choices = vec![(
            String::from("http://example.org/vs/a"),
            String::from("Synthetic A"),
        )];
        assert_eq!(Selection::Empty, chosen("", &choices));
        assert_eq!(
            Selection::Listed(String::from("http://example.org/vs/a")),
            chosen("http://example.org/vs/a", &choices)
        );
        assert_eq!(
            Selection::Other,
            chosen("http://example.org/vs/mine", &choices)
        );
        assert_eq!(
            Selection::Other,
            chosen("http://example.org/vs/a", &[]),
            "before the choices arrive"
        );
    }

    #[test]
    fn no_value_a_console_passes_takes_the_other_choice() {
        for name in [
            "another",
            "\u{FFFD}another",
            "Another canonical",
            "data-other",
        ] {
            let choices = vec![
                (String::from("first"), String::from("First")),
                (name.to_owned(), String::from("A real choice")),
            ];
            let selection = chosen(name, &choices);
            assert_eq!(Selection::Listed(name.to_owned()), selection, "{name:?}");
            assert_eq!(2, place(&selection, &choices), "{name:?} is its own option");
            assert_eq!(3, place(&Selection::Other, &choices), "another is last");
        }
        assert_eq!(0, place(&Selection::Empty, &[]));
        assert_eq!(1, place(&Selection::Other, &[]));
    }
}
