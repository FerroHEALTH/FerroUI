// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The write-only secret field (F3): what is set now, by reference and
//! never by value, and one choice of Keep, Set a value, Read from a file or
//! Clear, which shows the one control it needs.
//!
//! A secret's value never reaches the page: the state names the file the
//! secret is read from, or says it is set without saying where, and the
//! value field is a password input the page never fills, with
//! `autocomplete="off"`. The choices are native radios of one name, so the
//! keyboard moves through them as through any radio group (WAI-ARIA
//! Authoring Practices, the radio group pattern), and the stylesheet shows
//! the chosen choice's control with `:has()`, so the field works with no
//! script. Every control posts under the field's name: the choice as
//! `<name>[mode]` (`keep`, `value`, `file` or `clear`), the value as
//! `<name>[value]` and the path as `<name>[file]`. A console in file mode
//! draws the state alone ([`secret_state`]) in its read-only settings. No
//! specification governs the component: our own design.

use leptos::prelude::*;

use crate::i18n::{Phrase, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::form::{ERROR, described_by, error_id, help_id};

/// What is set now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretState {
    /// Set, read from the file at this path.
    Set(String),
    /// Set, where the page may not say where (an environment variable).
    Redacted,
    /// Not set.
    Unset,
    /// Set, read from the file at this path, which cannot be read.
    Unreadable(String),
}

/// What the field's choice is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecretChoice {
    /// Keep what is set, or leave a secret that is not set unset.
    #[default]
    Keep,
    /// Set a new value, which the server writes to a new file.
    Value,
    /// Read the secret from a file at a path.
    File,
    /// Remove the secret and its reference.
    Clear,
}

impl SecretChoice {
    /// The value the choice posts as.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Value => "value",
            Self::File => "file",
            Self::Clear => "clear",
        }
    }
}

/// A write-only secret field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretField {
    /// The field's id, which its controls' ids start with, and the name its
    /// controls post under.
    pub id: String,
    /// The secret's name, such as "Password" or "Private key".
    pub label: String,
    /// What is set now.
    pub state: SecretState,
    /// The choice in force.
    pub choice: SecretChoice,
    /// The path the file choice holds, such as one a refused save posted
    /// back; empty for none.
    pub path: String,
    /// The error under the chosen control, such as a path no replica can
    /// read; [`SecretField::refuse_value`] sets the refusal of a value.
    pub error: Option<String>,
}

impl SecretField {
    /// The field `id` for the secret `label`, set as `state`, with Keep
    /// chosen.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, state: SecretState) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state,
            choice: SecretChoice::Keep,
            path: String::new(),
            error: None,
        }
    }

    /// This field with `choice` chosen.
    #[must_use]
    pub fn choose(self, choice: SecretChoice) -> Self {
        Self { choice, ..self }
    }

    /// This field with the file choice holding `path`.
    #[must_use]
    pub fn path(self, path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..self
        }
    }

    /// This field with `error` under its chosen control.
    #[must_use]
    pub fn error(self, error: impl Into<String>) -> Self {
        Self {
            error: Some(error.into()),
            ..self
        }
    }

    /// This field with Set a value chosen and refused, because the server
    /// has no secrets directory to write a value to; reading it from a file
    /// still works.
    #[must_use]
    pub fn refuse_value(self) -> Self {
        Self {
            choice: SecretChoice::Value,
            error: Some(tr(Text::SecretNoDirectory).into()),
            ..self
        }
    }
}

/// Returns the state `state` with its glyph: by reference, in the muted
/// words a settings row or a secret field's head draws.
///
/// ```
/// use ferrohealth_ui::view::secret::{SecretState, secret_state};
/// use leptos::prelude::*;
///
/// let html = Owner::new().with(|| {
///     secret_state(&SecretState::Set(String::from("/run/secrets/db-url"))).to_html()
/// });
/// assert!(html.contains("<code>/run/secrets/db-url</code>"));
/// ```
#[must_use]
pub fn secret_state(state: &SecretState) -> AnyView {
    state_view(state, None)
}

/// The state [`secret_state`] draws, with the id `id` when it has one.
fn state_view(state: &SecretState, id: Option<String>) -> AnyView {
    let (glyph, words) = match state {
        SecretState::Set(file) => (Glyph::LOCK, around(Text::SecretSet, file)),
        SecretState::Redacted => (Glyph::LOCK, tr(Text::SecretRedacted).into_render()),
        SecretState::Unset => (Glyph::LOCK, tr(Text::SecretUnset).into_render()),
        SecretState::Unreadable(file) => (Glyph::TRIANGLE_ALERT, around(Text::SecretBroken, file)),
    };
    let modifier = match state {
        SecretState::Unset => Some("is-unset"),
        SecretState::Unreadable(_) => Some("is-broken"),
        SecretState::Set(_) | SecretState::Redacted => None,
    };
    let class = modifier.map_or_else(
        || String::from("fui-secret-now"),
        |modifier| format!("fui-secret-now {modifier}"),
    );
    view! {
        <span class=class id=id>
            {icon(glyph, "12")}
            {words}
        </span>
    }
    .into_any()
}

/// Returns the message `text` around the path `file`, the path in the code
/// face.
fn around(text: Text, file: &str) -> AnyView {
    let mark = Phrase::open_mark("file");
    let words = tr_with(text, &[("file", Phrase::open("file"))]);
    let lang = words.lang();
    let (before, after) = words.text().split_once(&mark).unwrap_or((words.text(), ""));
    let (before, after) = (before.to_owned(), after.to_owned());
    let file = file.to_owned();
    let drawn = view! {
        {before}
        <code>{file}</code>
        {after}
    };
    match lang {
        Some(lang) => view! { <span lang=lang>{drawn}</span> }.into_any(),
        None => drawn.into_any(),
    }
}

/// Returns the secret field `field`, with `hint` (a key hint, R15) after
/// its label.
///
/// The head names the secret and what is set now; under it the choices,
/// Keep reading Leave unset and Clear left out for a secret that is not
/// set; under them the chosen choice's control: the value, the path, or
/// the warning that Clear removes the secret.
#[must_use]
pub fn secret_field(field: SecretField, hint: Option<AnyView>) -> AnyView {
    let SecretField {
        id,
        label,
        state,
        choice,
        path,
        error,
    } = field;
    let set = state != SecretState::Unset;
    let choices: Vec<(SecretChoice, Text)> = if set {
        vec![
            (SecretChoice::Keep, Text::SecretKeep),
            (SecretChoice::Value, Text::SecretValue),
            (SecretChoice::File, Text::SecretFile),
            (SecretChoice::Clear, Text::SecretClear),
        ]
    } else {
        vec![
            (SecretChoice::Keep, Text::SecretLeaveUnset),
            (SecretChoice::Value, Text::SecretValue),
            (SecretChoice::File, Text::SecretFile),
        ]
    };
    let label_id = format!("{id}-label");
    let now_id = format!("{id}-now");
    let mode = format!("{id}[mode]");
    let segments = choices
        .iter()
        .map(|&(each, words)| {
            let radio = format!("{id}-mode-{}", each.key());
            let target = radio.clone();
            view! {
                <label for=target class="fui-segment">
                    <input
                        type="radio"
                        id=radio
                        name=mode.clone()
                        value=each.key()
                        checked=each == choice
                        class="fui-sr-only"
                    />
                    {tr(words)}
                </label>
            }
        })
        .collect_view();
    let error_on = |on: SecretChoice| error.as_deref().filter(|_| choice == on);
    let parts = choices
        .iter()
        .map(|&(each, _)| {
            let part = match each {
                SecretChoice::Keep => ().into_any(),
                SecretChoice::Value => value_control(&id, error_on(SecretChoice::Value)),
                SecretChoice::File => file_control(&id, path.clone(), error_on(SecretChoice::File)),
                SecretChoice::Clear => view! {
                    <p class="fui-secret-clear">
                        {icon(Glyph::TRIANGLE_ALERT, "14")}
                        {tr(Text::SecretClearNote)}
                    </p>
                }
                .into_any(),
            };
            view! { <div class="fui-revealed">{part}</div> }
        })
        .collect_view();
    view! {
        <div class="fui-secret" id=id.clone()>
            <div class="fui-secret-head">
                <span class="fui-label" id=label_id.clone()>
                    {label}
                    {hint}
                </span>
                {state_view(&state, Some(now_id.clone()))}
            </div>
            <div class="fui-reveal">
                <fieldset
                    class="fui-fieldset fui-choice"
                    aria-labelledby=label_id
                    aria-describedby=now_id
                >
                    <div class="fui-segmented">{segments}</div>
                </fieldset>
                <div class="fui-reveal-parts">{parts}</div>
            </div>
        </div>
    }
    .into_any()
}

/// The error line `error` of the control `id`, when there is one.
fn error_line(id: &str, error: Option<&str>) -> Option<AnyView> {
    error.map(|error| {
        let error = error.to_owned();
        view! {
            <p id=error_id(id) class=ERROR>
                {icon(Glyph::CIRCLE_ALERT, "14")}
                " "
                {error}
            </p>
        }
        .into_any()
    })
}

/// The control of Set a value: a password input the page never fills.
fn value_control(field: &str, error: Option<&str>) -> AnyView {
    let id = format!("{field}-value");
    let name = format!("{field}[value]");
    let mark = crate::widget::action::secret_mark(&name);
    let invalid = error.is_some().then_some("true");
    let line = error_line(&id, error);
    view! {
        <div class="fui-secret-control">
            <label for=id.clone() class="fui-label">
                {tr(Text::SecretValueLabel)}
            </label>
            {mark}
            <input
                id=id.clone()
                name=name
                type="password"
                class="fui-input"
                autocomplete="off"
                spellcheck="false"
                aria-invalid=invalid
                aria-describedby=described_by(&id, true, line.is_some())
            />
            {line}
            <p id=help_id(&id) class="fui-help">
                {tr(Text::SecretValueHelp)}
            </p>
        </div>
    }
    .into_any()
}

/// The control of Read from a file: the path, in the code face.
fn file_control(field: &str, path: String, error: Option<&str>) -> AnyView {
    let id = format!("{field}-file");
    let name = format!("{field}[file]");
    let invalid = error.is_some().then_some("true");
    let line = error_line(&id, error);
    view! {
        <div class="fui-secret-control">
            <label for=id.clone() class="fui-label">
                {tr(Text::SecretFileLabel)}
            </label>
            <input
                id=id.clone()
                name=name
                type="text"
                class="fui-input fui-mono"
                value=path
                spellcheck="false"
                autocomplete="off"
                aria-invalid=invalid
                aria-describedby=described_by(&id, true, line.is_some())
            />
            {line}
            <p id=help_id(&id) class="fui-help">
                {tr(Text::SecretFileHelp)}
            </p>
        </div>
    }
    .into_any()
}
