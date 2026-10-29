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

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::form::{ERROR, described_by, error_id, help_id, secret_mark};

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
    pub fn refuse_value(self, locale: &Locale) -> Self {
        Self {
            choice: SecretChoice::Value,
            error: Some(locale.text(Text::SecretNoDirectory)),
            ..self
        }
    }
}

/// Returns the state `state` with its glyph, in `locale`: by reference,
/// in the muted words a settings row or a secret field's head draws.
///
/// ```
/// use ferrohealth_ui::i18n::Locale;
/// use ferrohealth_ui::view::secret::{SecretState, secret_state};
///
/// let html = secret_state(
///     &Locale::english(),
///     &SecretState::Set(String::from("/run/secrets/db-url")),
/// ).to_string();
/// assert!(html.contains("<code>/run/secrets/db-url</code>"));
/// ```
#[must_use]
pub fn secret_state(locale: &Locale, state: &SecretState) -> Html {
    state_view(locale, state, None)
}

/// The state [`secret_state`] draws, with the id `id` when it has one.
fn state_view(locale: &Locale, state: &SecretState, id: Option<&str>) -> Html {
    let (glyph, words) = match state {
        SecretState::Set(file) => (Glyph::LOCK, around(locale, Text::SecretSet, file)),
        SecretState::Redacted => (Glyph::LOCK, tr(locale, Text::SecretRedacted).into()),
        SecretState::Unset => (Glyph::LOCK, tr(locale, Text::SecretUnset).into()),
        SecretState::Unreadable(file) => (
            Glyph::TRIANGLE_ALERT,
            around(locale, Text::SecretBroken, file),
        ),
    };
    let modifier = match state {
        SecretState::Unset => " is-unset",
        SecretState::Unreadable(_) => " is-broken",
        SecretState::Set(_) | SecretState::Redacted => "",
    };
    let id = id.map_or_else(String::new, |id| format!(" id=\"{}\"", Html::text(id)));
    Html::raw(format!(
        "<span{id} class=\"fui-secret-now{modifier}\">{}{words}</span>",
        icon(glyph, "12")
    ))
}

/// Returns the message `text` around the path `file`, the path in the code
/// face.
fn around(locale: &Locale, text: Text, file: &str) -> Html {
    let mark = Phrase::open_mark("file");
    let words: Word = tr_with(locale, text, &[("file", Phrase::open("file"))]);
    let (before, after) = words.text().split_once(&mark).unwrap_or((words.text(), ""));
    let drawn = format!(
        "{}<code>{}</code>{}",
        Html::text(before),
        Html::text(file),
        Html::text(after)
    );
    Html::raw(match words.lang() {
        Some(lang) => format!("<span lang=\"{lang}\">{drawn}</span>"),
        None => drawn,
    })
}

/// One choice of the field's segmented radios.
struct Segment {
    radio: String,
    key: &'static str,
    checked: bool,
    words: Word,
}

#[derive(Template)]
#[template(path = "secret/secret.html", block = "field")]
struct FieldView<'a> {
    id: &'a str,
    label: &'a str,
    hint: Option<Html>,
    now: Html,
    mode: String,
    segments: Vec<Segment>,
    parts: Vec<Html>,
}

#[derive(Template)]
#[template(path = "secret/secret.html", block = "control")]
struct ControlView<'a> {
    id: &'a str,
    name: &'a str,
    label: Word,
    mark: Option<Html>,
    kind: &'static str,
    mono: bool,
    value: &'a str,
    invalid: bool,
    described: Option<String>,
    error: Option<&'a str>,
    error_id: String,
    error_class: &'static str,
    alert: Html,
    help_id: String,
    help: Word,
}

/// Returns the secret field `field` in `locale`, with `hint` (a key hint,
/// R15) after its label.
///
/// The head names the secret and what is set now; under it the choices,
/// Keep reading Leave unset and Clear left out for a secret that is not
/// set; under them the chosen choice's control: the value, the path, or
/// the warning that Clear removes the secret.
#[must_use]
pub fn secret_field(locale: &Locale, field: &SecretField, hint: Option<Html>) -> Html {
    let SecretField {
        id,
        label,
        state,
        choice,
        path,
        error,
    } = field;
    let set = *state != SecretState::Unset;
    let choices: &[(SecretChoice, Text)] = if set {
        &[
            (SecretChoice::Keep, Text::SecretKeep),
            (SecretChoice::Value, Text::SecretValue),
            (SecretChoice::File, Text::SecretFile),
            (SecretChoice::Clear, Text::SecretClear),
        ]
    } else {
        &[
            (SecretChoice::Keep, Text::SecretLeaveUnset),
            (SecretChoice::Value, Text::SecretValue),
            (SecretChoice::File, Text::SecretFile),
        ]
    };
    let now_id = format!("{id}-now");
    let error_on = |on: SecretChoice| error.as_deref().filter(|_| *choice == on);
    let segments = choices
        .iter()
        .map(|&(each, words)| Segment {
            radio: format!("{id}-mode-{}", each.key()),
            key: each.key(),
            checked: each == *choice,
            words: tr(locale, words),
        })
        .collect();
    let parts = choices
        .iter()
        .map(|&(each, _)| match each {
            SecretChoice::Keep => Html::default(),
            SecretChoice::Value => control(
                locale,
                id,
                Control::Value,
                "",
                error_on(SecretChoice::Value),
            ),
            SecretChoice::File => control(
                locale,
                id,
                Control::File,
                path,
                error_on(SecretChoice::File),
            ),
            SecretChoice::Clear => Html::raw(format!(
                "<p class=\"fui-secret-clear\">{}{}</p>",
                icon(Glyph::TRIANGLE_ALERT, "14"),
                tr(locale, Text::SecretClearNote)
            )),
        })
        .collect();
    FieldView {
        id,
        label,
        hint,
        now: state_view(locale, state, Some(&now_id)),
        mode: format!("{id}[mode]"),
        segments,
        parts,
    }
    .html()
}

/// The two controls a choice shows.
#[derive(Clone, Copy)]
enum Control {
    /// Set a value: a password input the page never fills.
    Value,
    /// Read from a file: the path, in the code face.
    File,
}

/// The control `which` of the field `field`, holding `value`, with the
/// error line `error` when there is one.
fn control(locale: &Locale, field: &str, which: Control, value: &str, error: Option<&str>) -> Html {
    let (suffix, label, help) = match which {
        Control::Value => ("value", Text::SecretValueLabel, Text::SecretValueHelp),
        Control::File => ("file", Text::SecretFileLabel, Text::SecretFileHelp),
    };
    let id = format!("{field}-{suffix}");
    let name = format!("{field}[{suffix}]");
    let is_value = matches!(which, Control::Value);
    ControlView {
        id: &id,
        name: &name,
        label: tr(locale, label),
        mark: is_value.then(|| secret_mark(&name)),
        kind: if is_value { "password" } else { "text" },
        mono: !is_value,
        value,
        invalid: error.is_some(),
        described: described_by(&id, true, error.is_some()),
        error,
        error_id: error_id(&id),
        error_class: ERROR,
        alert: icon(Glyph::CIRCLE_ALERT, "14"),
        help_id: help_id(&id),
        help: tr(locale, help),
    }
    .html()
}
