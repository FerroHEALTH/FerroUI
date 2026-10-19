// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The segmented control bound to state: two to five choices as joined
//! buttons, the one in force pressed.
//!
//! Each choice is a `<button aria-pressed>` in a `role="group"` named by
//! its label, so a reader hears which is chosen in words, and the eye sees
//! it filled with the accent. The words are the choice's own, visible text.
//! The forms with no script, links for a choice kept in the URL and radios
//! for a choice a form posts, are [`crate::view::segmented`].

use leptos::prelude::*;

/// Returns the segmented control named `label` over `choices` (each its key
/// and its words), the one whose key `current` reads pressed, calling
/// `on_select` with the key a press chooses.
#[must_use]
pub fn segmented(
    label: &'static str,
    choices: &'static [(&'static str, &'static str)],
    current: Signal<&'static str>,
    on_select: impl Fn(&'static str) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let choices = choices
        .iter()
        .map(|(key, words)| (*key, String::from(*words)))
        .collect();
    segmented_words(String::from(label), choices, current, on_select)
}

/// Returns [`segmented`] over words made at run time, such as the words of
/// the page's locale.
pub(crate) fn segmented_words(
    label: String,
    choices: Vec<(&'static str, String)>,
    current: Signal<&'static str>,
    on_select: impl Fn(&'static str) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let buttons = choices
        .into_iter()
        .map(|(key, words)| {
            let on_select = on_select.clone();
            view! {
                <button
                    type="button"
                    class="fui-segment"
                    data-choice=key
                    aria-pressed=move || (current.get() == key).to_string()
                    on:click=move |_| on_select(key)
                >
                    {words}
                </button>
            }
        })
        .collect_view();
    view! {
        <div role="group" aria-label=label class="fui-segmented">
            {buttons}
        </div>
    }
}
