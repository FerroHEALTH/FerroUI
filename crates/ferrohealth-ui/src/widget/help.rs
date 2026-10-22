// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! One help switch per form: "Explain these parameters" shows every field's
//! help line, "Hide help" hides it again.
//!
//! A help line that is hidden stays in the accessibility tree as visually
//! hidden text, so its control keeps `aria-describedby` on every render
//! (WCAG 2.2 technique C7); before the page hydrates the lines are hidden
//! from sight and still read. The switch and the lines share one signal
//! through the context [`provide_help`] sets.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::form::help_id;

/// The help switch's words, resolved on the server in the page's locale, so
/// an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpWords {
    /// The switch's words while the help is hidden.
    pub show: String,
    /// The switch's words while the help shows.
    pub hide: String,
}

impl HelpWords {
    /// The switch's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            show: tr(Text::HelpShow),
            hide: tr(Text::HelpHide),
        }
    }
}

/// Whether the help lines of a form show, shared through the context.
#[derive(Debug, Clone, Copy)]
pub struct HelpShown(pub RwSignal<bool>);

/// Provides the help switch's signal to the form below, hidden at first,
/// and returns it.
#[must_use]
pub fn provide_help() -> HelpShown {
    let shown = HelpShown(RwSignal::new(false));
    provide_context(shown);
    shown
}

/// The help signal of the form around, or a hidden one of its own when no
/// form provided it.
fn help_shown() -> HelpShown {
    use_context::<HelpShown>().unwrap_or_else(|| HelpShown(RwSignal::new(false)))
}

/// Returns the form's help switch in `words`: a button with `aria-pressed`
/// that shows and hides every help line of the form.
#[must_use]
pub fn help_switch(words: HelpWords) -> AnyView {
    let HelpShown(shown) = help_shown();
    let HelpWords {
        show: explain,
        hide,
    } = words;
    view! {
        <button
            type="button"
            class=Variant::Ghost.small()
            aria-pressed=move || shown.get().to_string()
            on:click=move |_| shown.update(|shown| *shown = !*shown)
        >
            {icon(Glyph::INFO, "14")}
            {move || if shown.get() { hide.clone() } else { explain.clone() }}
        </button>
    }
    .into_any()
}

/// Returns the help line `text` of the control `id`, its id [`help_id`],
/// shown while the form's switch is on and visually hidden while it is off.
#[must_use]
pub fn help_line(id: &str, text: impl Into<String>) -> AnyView {
    let text = text.into();
    let line_id = help_id(id);
    let HelpShown(shown) = help_shown();
    let class = move || {
        if shown.get() {
            "fui-help"
        } else {
            "fui-help fui-sr-only"
        }
    };
    view! {
        <p id=line_id class=class>
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
    let HelpShown(shown) = help_shown();
    let class = move || {
        if shown.get() {
            "fui-help"
        } else {
            "fui-help fui-sr-only"
        }
    };
    view! {
        <div class="fui-check">
            {checkbox}
            <span class="fui-check-text">
                <label for=id>{label}</label>
                <span id=help_id(id) class=class>
                    {help}
                </span>
            </span>
        </div>
    }
    .into_any()
}
