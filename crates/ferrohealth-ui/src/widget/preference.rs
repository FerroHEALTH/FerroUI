// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The form of one preference in Your preferences: a plain post that a
//! change sends at once once the page has hydrated.
//!
//! The form posts the preference's control and the page to return to to
//! the console's route, which keeps the choice in its cookie (family rule
//! 9) and draws the page again with the row marked Saved. Before the page
//! has hydrated, and with no WebAssembly, a Save button follows the control
//! and posts it. Once hydrated, a change to the control submits the form
//! and the button leaves, so a preference applies at once (R28), and the
//! row just saved takes the focus back on its control, so a keyboard
//! carries on where it was. No specification governs this: our own design.

use leptos::prelude::*;

use crate::view::button::Variant;

/// Returns the form around the preference control `control`, posting it
/// and `back` to `action`, with the Save button saying `save` until the
/// page has hydrated; the control takes the focus when the row is `saved`.
#[must_use]
pub fn preference_form(
    action: String,
    back: String,
    save: crate::i18n::Word,
    saved: bool,
    control: AnyView,
) -> AnyView {
    let hydrated = RwSignal::new(false);
    let form = NodeRef::<leptos::html::Form>::new();
    Effect::new(move |_| {
        hydrated.set(true);
        if saved {
            refocus(form);
        }
    });
    let changed = move |_: leptos::ev::Event| submit(form);
    view! {
        <form
            method="post"
            action=action
            class="fui-preference-form"
            node_ref=form
            on:change=changed
        >
            <input type="hidden" name="back" value=back />
            {control}
            <button type="submit" class=Variant::Secondary.small() hidden=move || hydrated.get()>
                {save}
            </button>
        </form>
    }
    .into_any()
}

/// Submits `form` as its Save button would.
fn submit(form: NodeRef<leptos::html::Form>) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(form) = form.get_untracked()
            && form.request_submit().is_err()
        {
            leptos::logging::warn!("the browser refused to post a preference");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: NodeRef<leptos::html::Form> = form;
}

/// Gives the focus to the chosen control of `form`: its checked radio, its
/// select, or its switch.
fn refocus(form: NodeRef<leptos::html::Form>) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        // NOTE: leptos_macro 0.8 `src/lib.rs`, island limitation 1: the control is the
        // island's server children, out of reach of a node reference, so it is found in the form.
        let control = form
            .get_untracked()
            .and_then(|form| {
                form.query_selector("input:checked, select, input[type='checkbox']")
                    .ok()
                    .flatten()
            })
            .and_then(|control| control.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(control) = control
            && control.focus().is_err()
        {
            leptos::logging::warn!("the browser refused the focus on a preference");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: NodeRef<leptos::html::Form> = form;
}
