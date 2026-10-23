// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The subject line (E1): whose record this is, under the record's title.
//!
//! The line holds the label Subject, then the subject's identifier, masked
//! with its namespace as the tag and the audited Reveal (R13), or "No
//! external subject" in words; then the record's capability states as F12
//! marks: Queryable and Modifiable, a cleared one the hollow ring and its
//! negation ("Not queryable"). It wraps between its parts and never inside
//! the identifier. The two states are the `is_queryable` and
//! `is_modifiable` flags of the openEHR `EHR_STATUS` class (openEHR RM, EHR
//! Information Model, § `EHR_STATUS`). No specification governs the line:
//! our own design.

use leptos::prelude::*;

use crate::i18n::{Text, Word, tr};
use crate::view::state::{Mark, state};

/// Whose record it is.
#[derive(Debug)]
pub enum Subject {
    /// The record names an external subject: the identifier as the console
    /// draws it, a [`crate::widget::identifier::identifier`] inside its own
    /// island with the namespace as its domain, or
    /// [`crate::widget::identifier::identifier_static`] where no reveal is
    /// offered.
    External(AnyView),
    /// The record names no external subject.
    None,
}

/// What a subject line says.
#[derive(Debug)]
pub struct SubjectLine {
    /// Whose record it is.
    pub subject: Subject,
    /// Whether queries read the record.
    pub queryable: bool,
    /// Whether the record takes new versions.
    pub modifiable: bool,
}

/// Returns the F12 state of a capability: the ok mark beside `word` when
/// `on`, the mark of a cleared state when not.
fn on_off(on: bool, word: Word) -> AnyView {
    state(if on { Mark::Ok } else { Mark::Off }, word, None)
}

/// Returns the subject line `line`.
///
/// # Examples
///
/// ```
/// use ferrohealth_ui::view::subject::{Subject, SubjectLine, subject_line};
/// use leptos::prelude::*;
///
/// let html = Owner::new().with(|| {
///     subject_line(SubjectLine {
///         subject: Subject::None,
///         queryable: true,
///         modifiable: false,
///     })
///     .to_html()
/// });
/// assert!(html.contains("No external subject"));
/// assert!(html.contains("Not modifiable"));
/// ```
#[must_use]
pub fn subject_line(line: SubjectLine) -> AnyView {
    let SubjectLine {
        subject,
        queryable,
        modifiable,
    } = line;
    let who = match subject {
        Subject::External(identifier) => identifier,
        Subject::None => {
            view! { <span class="fui-subject-none">{tr(Text::SubjectNone)}</span> }.into_any()
        }
    };
    let queryable = on_off(
        queryable,
        tr(if queryable {
            Text::SubjectQueryable
        } else {
            Text::SubjectNotQueryable
        }),
    );
    let modifiable = on_off(
        modifiable,
        tr(if modifiable {
            Text::SubjectModifiable
        } else {
            Text::SubjectNotModifiable
        }),
    );
    view! {
        <div class="fui-subject">
            <span class="fui-subject-who">
                <span class="fui-subject-label">{tr(Text::SubjectLabel)}</span>
                {who}
            </span>
            <span class="fui-subject-states">{queryable}{modifiable}</span>
        </div>
    }
    .into_any()
}
