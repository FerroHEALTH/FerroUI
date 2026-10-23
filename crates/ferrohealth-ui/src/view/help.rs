// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! One help switch per form: "Explain these parameters" shows every field's
//! help line while it is on.
//!
//! The switch is a native checkbox with `role="switch"`, and the stylesheet
//! shows the help lines of the form that holds it while it is checked, so
//! it needs no script, no signal and no context, and works before the page
//! hydrates and on a page whose WebAssembly never loads (FerroUI#196, layer
//! 1). A help line that is off stays in the accessibility tree as visually
//! hidden text, so its control keeps `aria-describedby` on every render
//! (WCAG 2.2 technique C7). A help line in no form with a switch always
//! shows.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, Word, tr};
use crate::view::form::{SWITCH, help_id};

/// The help switch's words, resolved on the server in the page's locale, so
/// an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpWords {
    /// The switch's label, such as "Explain these parameters".
    pub label: Word,
}

impl HelpWords {
    /// The switch's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            label: tr(Text::HelpShow),
        }
    }
}

/// The classes of a help line the form's switch shows and hides.
const EXPLAINED: &str = "fui-help fui-explain";

/// Returns the form's help switch `id` in `words`: a checkbox drawn as a
/// switch that shows every help line of the form while it is checked.
///
/// It has no `name`, so the form submits nothing for it.
#[must_use]
pub fn help_switch(id: &'static str, words: HelpWords) -> AnyView {
    view! {
        <label class="fui-help-switch">
            <input type="checkbox" role="switch" id=id class=SWITCH />
            <span>{words.label}</span>
        </label>
    }
    .into_any()
}

/// Returns the help line `text` of the control `id`, its id [`help_id`],
/// shown while the form's switch is on and visually hidden while it is off.
#[must_use]
pub fn help_line(id: &str, text: impl Into<String>) -> AnyView {
    let text = text.into();
    view! {
        <p id=help_id(id) class=EXPLAINED>
            {text}
        </p>
    }
    .into_any()
}

/// Returns the field of the control `id` as [`crate::view::form::field`]
/// draws it, its help line under the form's switch.
#[must_use]
pub fn explained_field(
    id: &'static str,
    label: impl Into<String>,
    help: impl Into<String>,
    control: impl IntoView + 'static,
) -> AnyView {
    let (label, help) = (label.into(), help.into());
    let control = control.into_any();
    view! {
        <div class="fui-field">
            <label for=id class="fui-label">
                {label}
            </label>
            {control}
            {help_line(id, help)}
        </div>
    }
    .into_any()
}

/// Returns the checkbox `checkbox`, whose id is `id`, beside its label
/// `label` as [`crate::view::form::check_field`] draws it, its help line
/// under the words and under the form's switch.
///
/// The checkbox names the help with `aria-describedby` set to [`help_id`].
#[must_use]
pub fn explained_check_field(
    id: &'static str,
    label: impl Into<String>,
    help: impl Into<String>,
    checkbox: impl IntoView + 'static,
) -> AnyView {
    let (label, help) = (label.into(), help.into());
    let checkbox = checkbox.into_any();
    view! {
        <div class="fui-check">
            {checkbox}
            <span class="fui-check-text">
                <label for=id>{label}</label>
                <span id=help_id(id) class=EXPLAINED>
                    {help}
                </span>
            </span>
        </div>
    }
    .into_any()
}
