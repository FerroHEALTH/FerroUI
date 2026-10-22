// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The picker field: choose from what the server holds, or type another.
//!
//! A `<select>` with no `name` (it submits nothing) offers the empty choice,
//! the console's choices and "another"; the text input beside it carries the
//! `name` and is what the form submits. Choosing an option writes its value
//! into the input; choosing "another" shows the input for typing. A value
//! that is not among the choices opens on the input and stays there, so a
//! reader's own value never snaps back to the list, and choices that arrive
//! after the value select it once they do. The select follows the value
//! through its `selectedIndex` property, since an option's `selected`
//! attribute is its initial state only; the server marks the initial choice
//! and fills the input. "Another" is the last option, known by its place
//! and marked `data-other`, so it reserves no value a choice could take. Before the page hydrates both controls show and the form
//! posts the input, so nothing depends on script.

use std::borrow::Cow;

use leptos::prelude::*;
use leptos::web_sys::HtmlSelectElement;

use crate::view::form::{INPUT, SELECT, help_id};

/// Which option of the select stands for the input's value.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Selection {
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
    /// The help line under both, which both name.
    pub help: Option<Cow<'static, str>>,
    /// The choices, value and words, which may arrive after the value.
    pub choices: Signal<Vec<(String, String)>>,
    /// The value the input holds.
    pub value: RwSignal<String>,
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

/// Returns the picker field `picker`.
#[must_use]
pub fn picker_field(picker: Picker) -> AnyView {
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
    let typing = RwSignal::new(false);
    let hydrated = RwSignal::new(false);
    Effect::new(move || hydrated.set(true));
    let selected = move || {
        if typing.get() {
            Selection::Other
        } else {
            value.with(|value| choices.with(|choices| chosen(value, choices)))
        }
    };
    let input_hidden = move || hydrated.get() && selected() != Selection::Other;
    let selected_place = move || {
        let selection = selected();
        choices.with(|choices| place(&selection, choices))
    };
    let described = help.as_ref().map(|_| help_id(&id));
    let select_id = format!("{id}-choice");
    let focus_id = id.clone();
    let help = help.map(|help| picker_help(&id, help));
    // NOTE: HTML Living Standard, "The option element": the
    // `selected` attribute sets an option's selectedness only until the
    // select is used, so it marks the choice an option is created with.
    let options = move |(choice, words): (String, String)| {
        let on = untrack(selected) == Selection::Listed(choice.clone());
        view! {
            <option value=choice.clone() title=choice selected=on>
                {words}
            </option>
        }
    };
    let initial = untrack(selected);
    let typed = value.get_untracked();
    let on_change = move |event: leptos::ev::Event| {
        let select = event_target::<HtmlSelectElement>(&event);
        let last = i32::try_from(select.length()).map_or(0, |count| count.saturating_sub(1));
        if select.selected_index() == last {
            typing.set(true);
            focus_input(&focus_id);
        } else {
            typing.set(false);
            value.set(select.value());
        }
    };
    view! {
        <div class="fui-field fui-picker">
            <label for=select_id.clone() class="fui-sr-only">
                {choose_label}
            </label>
            <select
                id=select_id
                class=SELECT
                aria-describedby=described.clone()
                prop:selectedIndex=selected_place
                on:change=on_change
            >
                <option value="" selected=initial == Selection::Empty>
                    {empty_word}
                </option>
                <For each=move || choices.get() key=|(choice, _)| choice.clone() children=options />
                <option value="" data-other="" selected=initial == Selection::Other>
                    {other_word}
                </option>
            </select>
            <div class="fui-field" class:fui-picker-hidden=input_hidden>
                <label for=id.clone() class="fui-label">
                    {label}
                </label>
                <input
                    id=id
                    name=name
                    class=INPUT
                    aria-describedby=described
                    autocomplete="off"
                    spellcheck="false"
                    value=typed
                    prop:value=move || value.get()
                    on:input=move |event| value.set(event_target_value(&event))
                />
            </div>
            {help}
        </div>
    }
    .into_any()
}

/// Returns the picker's help line: under the form's help switch when the
/// form has one, always shown when it has none.
fn picker_help(id: &str, help: Cow<'static, str>) -> AnyView {
    if use_context::<crate::widget::help::HelpShown>().is_some() {
        crate::widget::help::help_line(id, help)
    } else {
        view! { <p id=help_id(id) class="fui-help">{help}</p> }.into_any()
    }
}

/// Moves the keyboard focus to the input `id`, once it shows.
fn focus_input(id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let id = id.to_owned();
        request_animation_frame(move || {
            let input = document()
                .get_element_by_id(&id)
                .and_then(|input| input.dyn_into::<web_sys::HtmlElement>().ok());
            if let Some(input) = input
                && input.focus().is_err()
            {
                leptos::logging::warn!("the browser refused the focus on a picker's input");
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = id;
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
