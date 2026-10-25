// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The segmented controls that need no script: a choice kept in the URL, as
//! links, a choice a form posts, as radios, and a choice each segment posts
//! at once, as submit buttons.
//!
//! Both draw the joined segments of [`crate::widget::segmented`], the one
//! in force filled with the accent. The link form marks it
//! `aria-current="true"`; the radio form is a native radio group under a
//! legend, each radio visually hidden and its label the segment, so the
//! keyboard moves through it as through any radio group (WAI-ARIA Authoring
//! Practices, the radio group pattern, with native radios).

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::Word;
use crate::icon::{Glyph, icon};

/// Returns the choices `choices` (each its href and its words) as joined
/// links named `label`, the one at index `current` marked current.
#[must_use]
pub fn segmented_links(
    label: impl Into<String>,
    choices: Vec<(String, impl Into<String>)>,
    current: usize,
) -> AnyView {
    let label = label.into();
    let links = choices
        .into_iter()
        .enumerate()
        .map(|(index, (href, words))| {
            let here = (index == current).then_some("true");
            let words = words.into();
            view! {
                <a href=href class="fui-segment" aria-current=here>
                    {words}
                </a>
            }
        })
        .collect_view();
    view! {
        <nav class="fui-segmented" aria-label=label>
            {links}
        </nav>
    }
    .into_any()
}

/// Returns the radio group `name` under `legend`, drawn as joined segments.
///
/// Its options are `(value, words)`, the one whose value is `checked`
/// checked; the radio ids are `name`, a dash and the value.
#[must_use]
pub fn choice_group(
    name: &'static str,
    legend: impl Into<String>,
    options: &[(&'static str, impl AsRef<str>)],
    checked: &str,
) -> AnyView {
    let legend = legend.into();
    let segments = options
        .iter()
        .map(|(value, words)| {
            let id = format!("{name}-{value}");
            let label_for = id.clone();
            let on = *value == checked;
            view! {
                <label for=label_for class="fui-segment">
                    <input
                        type="radio"
                        id=id
                        name=name
                        value=*value
                        checked=on
                        class="fui-sr-only"
                    />
                    {words.as_ref().to_owned()}
                </label>
            }
        })
        .collect_view();
    view! {
        <fieldset class="fui-fieldset fui-choice">
            <legend class="fui-label">{legend}</legend>
            <div class="fui-segmented">{segments}</div>
        </fieldset>
    }
    .into_any()
}

/// A choice each segment posts at once, such as the FHIR version a console
/// reads.
#[derive(Debug, Clone)]
pub struct SegmentedForm {
    /// The name each segment posts its value as.
    pub name: &'static str,
    /// The group's accessible name.
    pub label: Cow<'static, str>,
    /// A glyph and a word shown before the segments, such as "FHIR".
    pub prefix: Option<(Glyph, Cow<'static, str>)>,
    /// The choices, value and words.
    pub choices: Vec<(String, Word)>,
    /// The value in force.
    pub current: String,
    /// The console's route the form posts to.
    pub action: String,
    /// The page to return to, posted as `back`.
    pub back: String,
}

/// Returns the choice `form` as joined submit buttons of one plain form,
/// the one in force pressed; it works with no script, and the console keeps
/// the choice and sends the browser back to `back`.
#[must_use]
pub fn segmented_form(form: SegmentedForm) -> AnyView {
    let SegmentedForm {
        name,
        label,
        prefix,
        choices,
        current,
        action,
        back,
    } = form;
    let prefix = prefix.map(|(glyph, word)| {
        view! { <span class="fui-segmented-prefix">{icon(glyph, "14")} {word}</span> }
    });
    let buttons = choices
        .into_iter()
        .map(|(value, words)| {
            let pressed = (value == current).to_string();
            view! {
                <button type="submit" name=name value=value class="fui-segment" aria-pressed=pressed>
                    {words}
                </button>
            }
        })
        .collect_view();
    view! {
        <form method="post" action=action data-ferroui-navigates="">
            <input type="hidden" name="back" value=back />
            <div role="group" aria-label=label class="fui-segmented">
                {prefix}
                {buttons}
            </div>
        </form>
    }
    .into_any()
}

/// The most choices [`choice_reveal`] reveals a part for.
pub const REVEAL_MAX: usize = 5;

/// Returns the radio choice `name` as [`choice_group`] draws it, with the
/// part of `parts` at each option's place shown while that option is chosen.
///
/// It suits the fields of a secret kept as a value or as a file.
///
/// The stylesheet reveals a part by its place with `:has()`, so it works
/// before the page hydrates and with no script; options past
/// [`REVEAL_MAX`] reveal nothing. A part a choice needs none for is `()`.
#[must_use]
pub fn choice_reveal(
    name: &'static str,
    legend: impl Into<String>,
    options: &[(&'static str, impl AsRef<str>)],
    checked: &str,
    parts: Vec<AnyView>,
) -> AnyView {
    let group = choice_group(name, legend, options, checked).into_any();
    let parts = parts
        .into_iter()
        .map(|part| view! { <div class="fui-revealed">{part}</div> })
        .collect_view();
    view! {
        <div class="fui-reveal">
            {group}
            <div class="fui-reveal-parts">{parts}</div>
        </div>
    }
    .into_any()
}
