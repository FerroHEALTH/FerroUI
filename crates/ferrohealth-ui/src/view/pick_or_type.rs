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
//! attribute). The console owns the value and the controls and says which
//! shows; the field draws them as one, and the `picker` module shows the
//! input when the picker's option marked `data-other` is chosen (a
//! combobox's [`crate::view::combobox::ComboOption::other`]), and the
//! picker again on a control in `after` marked `data-fui-pick-back`, on the
//! listed option chosen before the other one or on its first, so choosing
//! the other option again shows the input.
//! [`crate::view::picker`] is the whole widget, for a console that holds
//! no controls of its own.

use askama::Template;

use crate::html::{Html, Render as _};

/// The picker of a [`PickOrType`].
#[derive(Debug)]
pub enum Picker {
    /// A control, such as a select, whose id is `id`, which the field
    /// labels.
    Control {
        /// The id of the control, which its label names with `for`.
        id: String,
        /// The control.
        control: Html,
    },
    /// A control that carries its own label, such as a combobox
    /// ([`crate::view::combobox`]).
    Labelled(Html),
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
    pub typed: Html,
    /// Whether the text input shows; the picker shows while it is false.
    pub typing: bool,
    /// What follows both controls, such as the help line both name in
    /// `aria-describedby`.
    pub after: Option<Html>,
}

/// Returns the field `field`: its picker with its label while
/// `field.typing` is false, its text input with its label while it is
/// true, and what follows both.
#[must_use]
pub fn pick_or_type(field: PickOrType) -> Html {
    let PickOrType {
        label,
        picker,
        typed_id,
        typed,
        typing,
        after,
    } = field;
    let picker = match picker {
        Picker::Control { id, control } => Html::raw(format!(
            "<label for=\"{}\" class=\"fui-label\">{}</label>{control}",
            Html::text(id),
            Html::text(&label)
        )),
        Picker::Labelled(control) => control,
    };
    PickOrTypeView {
        typing,
        picker,
        typed_id,
        label,
        typed,
        after: after.unwrap_or_default(),
    }
    .html()
}

#[derive(Template)]
#[template(path = "picker/picker.html", block = "pick_or_type")]
struct PickOrTypeView {
    typing: bool,
    picker: Html,
    typed_id: String,
    label: String,
    typed: Html,
    after: Html,
}
