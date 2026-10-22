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
//! through its `value` property, since an option's `selected` attribute is
//! its initial state only; the server marks the initial choice and fills
//! the input. Before the page hydrates both controls show and the form
//! posts the input, so nothing depends on script.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::view::form::{INPUT, SELECT, help_id};

/// The select's value for the "another" choice, which no key a server holds
/// starts with.
// NOTE: HTML Living Standard, tokenization, "Attribute value (double-quoted)
// state": the parser turns U+0000 into U+FFFD, so a NUL never matches.
const OTHER: &str = "\u{FFFD}another";

/// What a picker field shows.
#[derive(Debug, Clone)]
pub struct Picker {
    /// The text input's id; the select's is this with `-choice`.
    pub id: &'static str,
    /// The name the input submits as.
    pub name: &'static str,
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

/// The select's value for `value` among `choices`: the value itself when it
/// is listed, the empty choice for no value, and "another" otherwise.
fn chosen(value: &str, choices: &[(String, String)]) -> String {
    if value.is_empty() {
        String::new()
    } else if choices.iter().any(|(choice, _)| choice == value) {
        value.to_owned()
    } else {
        String::from(OTHER)
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
            String::from(OTHER)
        } else {
            value.with(|value| choices.with(|choices| chosen(value, choices)))
        }
    };
    let input_hidden = move || hydrated.get() && selected() != OTHER;
    let described = help.as_ref().map(|_| help_id(id));
    let select_id = format!("{id}-choice");
    // NOTE: HTML Living Standard, "The option element": the
    // `selected` attribute sets an option's selectedness only until the
    // select is used, so it marks the choice an option is created with.
    let options = move |(choice, words): (String, String)| {
        let on = untrack(selected) == choice;
        view! {
            <option value=choice.clone() title=choice selected=on>
                {words}
            </option>
        }
    };
    let initial = untrack(selected);
    let typed = value.get_untracked();
    let on_change = move |event: leptos::ev::Event| {
        let option = event_target_value(&event);
        if option == OTHER {
            typing.set(true);
            focus_input(id);
        } else {
            typing.set(false);
            value.set(option);
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
                prop:value=selected
                on:change=on_change
            >
                <option value="" selected=initial.is_empty()>
                    {empty_word}
                </option>
                <For each=move || choices.get() key=|(choice, _)| choice.clone() children=options />
                <option value=OTHER selected=initial == OTHER>
                    {other_word}
                </option>
            </select>
            <div class="fui-field" class:fui-picker-hidden=input_hidden>
                <label for=id class="fui-label">
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
            {help.map(|help| picker_help(id, help))}
        </div>
    }
    .into_any()
}

/// Returns the picker's help line: under the form's help switch when the
/// form has one, always shown when it has none.
fn picker_help(id: &'static str, help: Cow<'static, str>) -> AnyView {
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
    use super::{OTHER, chosen};

    #[test]
    fn a_listed_value_is_chosen_and_any_other_opens_the_input() {
        let choices = vec![(
            String::from("http://example.org/vs/a"),
            String::from("Synthetic A"),
        )];
        assert_eq!("", chosen("", &choices));
        assert_eq!(
            "http://example.org/vs/a",
            chosen("http://example.org/vs/a", &choices)
        );
        assert_eq!(OTHER, chosen("http://example.org/vs/mine", &choices));
        assert_eq!(
            OTHER,
            chosen("http://example.org/vs/a", &[]),
            "before the choices arrive"
        );
    }
}
