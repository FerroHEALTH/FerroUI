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

/// The classes of a text area for prose, such as a definition: the body
/// face, two rows high unless the console sets `rows`.
pub const PROSE: &str = "fui-textarea fui-textarea-prose";

/// The width class of a control that holds a number or a short code, which
/// never takes the full row; add it to the control's classes.
pub const NARROW: &str = "fui-control-narrow";

/// The width class of a control that holds a name or a version.
pub const MEDIUM: &str = "fui-control-medium";

/// The classes of a checkbox drawn as a switch.
///
/// The knob's place says on or off as well as its colour (WCAG 2.2
/// SC 1.4.1); give the input `role="switch"` and use it where
/// [`check_field`] takes a checkbox.
pub const SWITCH: &str = "fui-switch";

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
    control: impl IntoView + 'static,
) -> impl IntoView {
    let control = control.into_any();
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

/// Returns the related controls `body` under the legend `legend`, the
/// browser's fieldset frame reset to the card's look (HTML Living Standard,
/// the `fieldset` element; WCAG 2.2 SC 1.3.1).
#[must_use]
pub fn fieldset(legend: &'static str, body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! {
        <fieldset class="fui-fieldset">
            <legend>{legend}</legend>
            {body}
        </fieldset>
    }
}

/// Returns the fields `body` in pairs from 40 rem up, one above the other
/// below it.
#[must_use]
pub fn columns(body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! { <div class="fui-columns">{body}</div> }
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

/// Returns the checkbox `checkbox` beside its label with the line `help`
/// under the words.
///
/// The row is [`check`]'s; the checkbox, whose id is `id`, names the help
/// with `aria-describedby` set to [`help_id`].
#[must_use]
pub fn check_field(
    id: &'static str,
    label: &'static str,
    help: &'static str,
    checkbox: impl IntoView + 'static,
) -> AnyView {
    let checkbox = checkbox.into_any();
    view! {
        <div class="fui-check">
            {checkbox}
            <span class="fui-check-text">
                <label for=id>{label}</label>
                <span id=help_id(id) class="fui-help">
                    {help}
                </span>
            </span>
        </div>
    }
    .into_any()
}

/// Returns a group of checkboxes named `name` under the legend `legend`,
/// each drawn as a chip.
///
/// A ticked chip takes the accent's edge, so the tick and the edge both say
/// so: a choice of several, such as the nodes a query asks.
#[must_use]
pub fn chip_choices(
    name: &'static str,
    legend: &'static str,
    options: Vec<(String, String)>,
    checked: &[String],
) -> AnyView {
    let chips = options
        .into_iter()
        .map(|(value, words)| {
            let id = format!("{name}-{value}");
            let label_for = id.clone();
            let on = checked.contains(&value);
            view! {
                <label for=label_for class="fui-chip-check">
                    <input type="checkbox" id=id name=name value=value checked=on />
                    {words}
                </label>
            }
        })
        .collect_view();
    view! {
        <fieldset class="fui-fieldset fui-choice">
            <legend class="fui-label">{legend}</legend>
            <div class="fui-chip-checks">{chips}</div>
        </fieldset>
    }
    .into_any()
}
