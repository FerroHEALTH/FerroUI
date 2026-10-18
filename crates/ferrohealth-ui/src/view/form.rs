// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The fields: a label above its control, the help under it, and the
//! checkbox with its label on one row.
//!
//! The control itself is the console's, because its value and listeners
//! are: a view here gives it the classes and the label and help their
//! associations. Every control carries an explicit, stable `id` and `name`
//! (no random id differs between the server pass and hydration), and a
//! control with help names it with `aria-describedby` set to [`help_id`].

use leptos::prelude::*;

/// The classes of a text input.
pub const INPUT: &str = "fui-input";

/// The classes of a select.
pub const SELECT: &str = "fui-select";

/// The classes of a text area in the code face.
pub const TEXTAREA: &str = "fui-textarea";

/// The classes of a form laid out as one column of fields.
pub const FORM: &str = "fui-form";

/// The id of the help line of the control `id`, which the control names in
/// `aria-describedby`.
#[must_use]
pub fn help_id(id: &str) -> String {
    format!("{id}-help")
}

/// Returns the field of the control `id`: the label `label` above
/// `control`, and the line `help` under it when there is one.
#[must_use]
pub fn field(
    id: &'static str,
    label: &'static str,
    help: Option<&'static str>,
    control: impl IntoView,
) -> impl IntoView {
    view! {
        <div class="fui-field">
            <label for=id class="fui-label">
                {label}
            </label>
            {control}
            {help.map(|help| view! { <p id=help_id(id) class="fui-help">{help}</p> })}
        </div>
    }
}

/// Returns the checkbox `checkbox`, whose id is `id`, beside its label
/// `label` on one row, the box clear of the words.
#[must_use]
pub fn check(id: &'static str, label: &'static str, checkbox: impl IntoView) -> impl IntoView {
    view! {
        <div class="fui-check">
            {checkbox} <label for=id>{label}</label>
        </div>
    }
}
