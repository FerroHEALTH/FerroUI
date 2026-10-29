// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The segmented controls that need no script: a choice kept in the URL, as
//! links, a choice a form posts, as radios, and a choice each segment posts
//! at once, as submit buttons.
//!
//! Each draws joined segments, the one in force filled with the accent. The
//! link form marks it `aria-current="true"`; the radio form is a native
//! radio group under a legend, each radio visually hidden and its label the
//! segment, so the keyboard moves through it as through any radio group
//! (WAI-ARIA Authoring Practices, the radio group pattern, with native
//! radios).

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::Word;
use crate::icon::{Glyph, icon};

#[derive(Template)]
#[template(
    source = r#"<nav class="fui-segmented" aria-label="{{ label }}">{% for (href, words, here) in links %}<a href="{{ href }}" class="fui-segment"{% if *here %} aria-current="true"{% endif %}>{{ words }}</a>{% endfor %}</nav>"#,
    ext = "html"
)]
struct Links {
    label: String,
    links: Vec<(String, String, bool)>,
}

/// Returns the choices `choices` (each its href and its words) as joined
/// links named `label`, the one at index `current` marked current.
#[must_use]
pub fn segmented_links(
    label: impl Into<String>,
    choices: Vec<(String, impl Into<String>)>,
    current: usize,
) -> Html {
    Links {
        label: label.into(),
        links: choices
            .into_iter()
            .enumerate()
            .map(|(index, (href, words))| (href, words.into(), index == current))
            .collect(),
    }
    .html()
}

#[derive(Template)]
#[template(
    source = r#"<fieldset class="fui-fieldset fui-choice"><legend class="fui-label">{{ legend }}</legend><div class="fui-segmented">{% for (id, value, words, on) in options %}<label for="{{ id }}" class="fui-segment"><input type="radio" id="{{ id }}" name="{{ name }}" value="{{ value }}"{% if *on %} checked{% endif %} class="fui-sr-only">{{ words }}</label>{% endfor %}</div></fieldset>"#,
    ext = "html"
)]
struct Group {
    name: &'static str,
    legend: String,
    options: Vec<(String, &'static str, String, bool)>,
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
) -> Html {
    Group {
        name,
        legend: legend.into(),
        options: options
            .iter()
            .map(|(value, words)| {
                (
                    format!("{name}-{value}"),
                    *value,
                    words.as_ref().to_owned(),
                    *value == checked,
                )
            })
            .collect(),
    }
    .html()
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

#[derive(Template)]
#[template(
    source = r#"<form method="post" action="{{ action }}"><input type="hidden" name="back" value="{{ back }}"><div role="group" aria-label="{{ label }}" class="fui-segmented">{% if let Some((glyph, word)) = prefix %}<span class="fui-segmented-prefix">{{ glyph }} {{ word }}</span>{% endif %}{% for (value, words, pressed) in choices %}<button type="submit" name="{{ name }}" value="{{ value }}" class="fui-segment" aria-pressed="{{ pressed }}">{{ words }}</button>{% endfor %}</div></form>"#,
    ext = "html"
)]
struct Segments {
    name: &'static str,
    label: Cow<'static, str>,
    prefix: Option<(Html, Cow<'static, str>)>,
    choices: Vec<(String, Word, bool)>,
    action: String,
    back: String,
}

/// Returns the choice `form` as joined submit buttons of one plain form,
/// the one in force pressed; the console keeps the choice and sends the
/// browser back to `back`.
#[must_use]
pub fn segmented_form(form: SegmentedForm) -> Html {
    let SegmentedForm {
        name,
        label,
        prefix,
        choices,
        current,
        action,
        back,
    } = form;
    Segments {
        name,
        label,
        prefix: prefix.map(|(glyph, word)| (icon(glyph, "14"), word)),
        choices: choices
            .into_iter()
            .map(|(value, words)| {
                let pressed = value == current;
                (value, words, pressed)
            })
            .collect(),
        action,
        back,
    }
    .html()
}

/// The most choices [`choice_reveal`] reveals a part for.
pub const REVEAL_MAX: usize = 5;

/// Returns the radio choice `name` as [`choice_group`] draws it, with the
/// part of `parts` at each option's place shown while that option is chosen.
///
/// It suits the fields of a secret kept as a value or as a file. The
/// stylesheet reveals a part by its place with `:has()`, so it works with
/// no script; options past [`REVEAL_MAX`] reveal nothing. A part a choice
/// needs none for is [`Html::empty`].
#[must_use]
pub fn choice_reveal(
    name: &'static str,
    legend: impl Into<String>,
    options: &[(&'static str, impl AsRef<str>)],
    checked: &str,
    parts: Vec<Html>,
) -> Html {
    let group = choice_group(name, legend, options, checked);
    let parts: Html = parts
        .into_iter()
        .map(|part| Html::raw(format!("<div class=\"fui-revealed\">{part}</div>")))
        .collect();
    Html::raw(format!(
        "<div class=\"fui-reveal\">{group}<div class=\"fui-reveal-parts\">{parts}</div></div>"
    ))
}
