// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! One help switch per form: "Explain these parameters" shows every field's
//! help line while it is on.
//!
//! The switch is a native checkbox with `role="switch"`, and the stylesheet
//! shows the help lines of the form that holds it while it is checked, so
//! it needs no script and works on every page. A help line that is off
//! stays in the accessibility tree as visually hidden text, so its control
//! keeps `aria-describedby` (WCAG 2.2 technique C7). A help line in no form
//! with a switch always shows.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
use crate::view::form::{SWITCH, help_id};

/// The help switch's words, resolved in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpWords {
    /// The switch's label, such as "Explain these parameters".
    pub label: Word,
}

impl HelpWords {
    /// The switch's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            label: tr(locale, Text::HelpShow),
        }
    }
}

/// The classes of a help line the form's switch shows and hides.
const EXPLAINED: &str = "fui-help fui-explain";

#[derive(Template)]
#[template(
    source = r#"<label class="fui-help-switch"><input type="checkbox" role="switch" id="{{ id }}" class="{{ switch }}"><span>{{ label }}</span></label>"#,
    ext = "html"
)]
struct Switch {
    id: &'static str,
    switch: &'static str,
    label: Word,
}

/// Returns the form's help switch `id` in `words`: a checkbox drawn as a
/// switch that shows every help line of the form while it is checked. It
/// has no `name`, so the form submits nothing for it.
#[must_use]
pub fn help_switch(id: &'static str, words: HelpWords) -> Html {
    Switch {
        id,
        switch: SWITCH,
        label: words.label,
    }
    .html()
}

/// Returns the help line `text` of the control `id`, its id [`help_id`],
/// shown while the form's switch is on and visually hidden while it is off.
#[must_use]
pub fn help_line(id: &str, text: impl Into<String>) -> Html {
    Html::raw(format!(
        "<p id=\"{}\" class=\"{EXPLAINED}\">{}</p>",
        Html::text(help_id(id)),
        Html::text(text.into())
    ))
}

#[derive(Template)]
#[template(
    source = r#"<div class="fui-field"><label for="{{ id }}" class="fui-label">{{ label }}</label>{{ control }}{{ help }}</div>"#,
    ext = "html"
)]
struct ExplainedField {
    id: &'static str,
    label: String,
    control: Html,
    help: Html,
}

/// Returns the field of the control `id` as [`crate::view::form::field`]
/// draws it, its help line under the form's switch.
#[must_use]
pub fn explained_field(
    id: &'static str,
    label: impl Into<String>,
    help: impl Into<String>,
    control: Html,
) -> Html {
    ExplainedField {
        id,
        label: label.into(),
        control,
        help: help_line(id, help),
    }
    .html()
}

#[derive(Template)]
#[template(
    source = r#"<div class="fui-check">{{ checkbox }}<span class="fui-check-text"><label for="{{ id }}">{{ label }}</label><span id="{{ help_id }}" class="fui-help fui-explain">{{ help }}</span></span></div>"#,
    ext = "html"
)]
struct ExplainedCheck {
    id: &'static str,
    help_id: String,
    label: String,
    help: String,
    checkbox: Html,
}

/// Returns the checkbox `checkbox`, whose id is `id`, beside its label
/// `label` as [`crate::view::form::check_field`] draws it, its help line
/// under the words and under the form's switch.
#[must_use]
pub fn explained_check_field(
    id: &'static str,
    label: impl Into<String>,
    help: impl Into<String>,
    checkbox: Html,
) -> Html {
    ExplainedCheck {
        id,
        help_id: help_id(id),
        label: label.into(),
        help: help.into(),
        checkbox,
    }
    .html()
}
