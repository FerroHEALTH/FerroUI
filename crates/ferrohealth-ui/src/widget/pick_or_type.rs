// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! One value a reader picks from a list or types.
//!
//! The field holds a picker (a select, or a combobox for a long list) and a
//! text input behind the picker's last option, such as "Another canonical",
//! one of the two shown at a time.
//!
//! Each control carries its own label, hidden and shown with it, because a
//! label names one control and only one of the two shows (WAI-ARIA 1.2,
//! accessible name computation; HTML Living Standard, the `hidden`
//! attribute). The console owns the value, the controls and the signal
//! that says which shows; the field draws them as one.
//! [`crate::widget::picker`] is the whole widget, value and state included,
//! for a console that holds neither.

use leptos::prelude::*;

/// The picker of a [`PickOrType`].
#[derive(Debug)]
pub enum Picker {
    /// A control, such as a select, whose id is `id`, which the field
    /// labels.
    Control {
        /// The id of the control, which its label names with `for`.
        id: String,
        /// The control.
        control: AnyView,
    },
    /// A control that carries its own label, such as a combobox
    /// ([`crate::widget::combobox`]).
    Labelled(AnyView),
}

/// What [`pick_or_type`] draws.
#[derive(Debug)]
pub struct PickOrType {
    /// The label both controls carry, such as "Code system".
    pub label: String,
    /// The picker.
    pub picker: Picker,
    /// The id of the text input, which its label names with `for`.
    pub typed_id: String,
    /// The text input.
    pub typed: AnyView,
    /// Whether the text input shows; the picker shows while it is false.
    pub typing: Signal<bool>,
    /// What follows both controls, such as the help line both name in
    /// `aria-describedby`.
    pub after: Option<AnyView>,
}

/// Returns the field `field`: its picker with its label while
/// `field.typing` is false, its text input with its label while it is
/// true, and what follows both.
#[must_use]
pub fn pick_or_type(field: PickOrType) -> AnyView {
    let PickOrType {
        label,
        picker,
        typed_id,
        typed,
        typing,
        after,
    } = field;
    let picker = match picker {
        Picker::Control { id, control } => {
            let label = label.clone();
            view! {
                <label for=id class="fui-label">
                    {label}
                </label>
                {control}
            }
            .into_any()
        }
        Picker::Labelled(control) => control,
    };
    view! {
        <div class="fui-field">
            <div class="fui-field" data-side="pick" hidden=move || typing.get()>
                {picker}
            </div>
            <div class="fui-field" data-side="type" hidden=move || !typing.get()>
                <label for=typed_id class="fui-label">
                    {label}
                </label>
                {typed}
            </div>
            {after}
        </div>
    }
    .into_any()
}
