// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The refusal and the fault: an upstream's status and the issues it
//! answered with, each with the text it carries, and the console's own
//! fault. An error never renders as nothing.
//!
//! The library reads no upstream document itself: a console turns an
//! `OperationOutcome` or a diagnostic list into [`Issue`]s with the types
//! its own crates model, and the refusal draws them.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::tone::Tone;

/// How grave an issue is (the FHIR `IssueSeverity` value set, which every
/// FerroHEALTH upstream's issues map onto).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// The action failed and nothing more can be done.
    Fatal,
    /// The action failed.
    Error,
    /// The action succeeded with a concern.
    Warning,
    /// A fact the upstream reports.
    Information,
}

impl Severity {
    /// The word a reader sees.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Fatal => "fatal",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Information => "information",
        }
    }

    /// The tone the word is drawn in.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Fatal | Self::Error => Tone::Bad,
            Self::Warning => Tone::Warn,
            Self::Information => Tone::Neutral,
        }
    }
}

/// One issue an upstream answered with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// How grave it is.
    pub severity: Severity,
    /// Its code, such as `processing` or a diagnostic's code.
    pub code: String,
    /// The text it carries for a reader (an `OperationOutcome` issue's
    /// `diagnostics`, else its `details.text`), when it carries any.
    pub text: Option<String>,
    /// Where it points, such as an expression or a line, empty for nowhere.
    pub place: String,
}

/// What a refusal shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The upstream's status, such as `422`, when it answered one.
    pub status: Option<u16>,
    /// The sentence that says what was refused.
    pub summary: String,
    /// The issues it answered with.
    pub issues: Vec<Issue>,
}

/// Returns the issues of `issues` a reader is shown, gravest first, and how
/// many were left out.
///
/// An `information` issue that carries no text says nothing to act on, so it
/// is counted rather than listed; an issue of any other severity is always
/// listed.
#[must_use]
pub fn shown_issues(issues: &[Issue]) -> (Vec<&Issue>, usize) {
    let mut shown: Vec<&Issue> = issues
        .iter()
        .filter(|issue| issue.text.is_some() || issue.severity != Severity::Information)
        .collect();
    shown.sort_by_key(|issue| issue.severity);
    let left_out = issues.len().saturating_sub(shown.len());
    (shown, left_out)
}

/// Returns `issues` as a list, gravest first, each with its severity as a
/// word, its code, where it points and its text, and a sentence for the
/// information issues left out.
#[must_use]
pub fn issue_list(issues: &[Issue]) -> impl IntoView + use<> {
    let (shown, left_out) = shown_issues(issues);
    let items = shown
        .into_iter()
        .map(|issue| {
            let text = match &issue.text {
                Some(text) => text.clone().into_any(),
                None => view! {
                    <span class="fui-muted">"The issue carries no text."</span>
                }
                .into_any(),
            };
            let place = (!issue.place.is_empty()).then(|| {
                view! {
                    " at "
                    <code>{issue.place.clone()}</code>
                }
            });
            view! {
                <li>
                    <span class=issue.severity.tone().badge()>{issue.severity.word()}</span>
                    " "
                    <code>{issue.code.clone()}</code>
                    {place}
                    ": "
                    {text}
                </li>
            }
        })
        .collect_view();
    let noun = if left_out == 1 { "issue" } else { "issues" };
    let left = (left_out > 0).then(|| {
        view! {
            <p class="fui-muted">
                {format!("{left_out} information {noun} with no text left out.")}
            </p>
        }
    });
    view! {
        <ul class="fui-issues">{items}</ul>
        {left}
    }
}

/// Returns `refusal` as an alert: its status, its summary and its issues.
#[must_use]
pub fn refusal(refusal: &Refusal) -> impl IntoView + use<> {
    let title = match refusal.status {
        Some(status) => format!("{status}: {}", refusal.summary),
        None => refusal.summary.clone(),
    };
    let issues = (!refusal.issues.is_empty()).then(|| issue_list(&refusal.issues));
    view! {
        <div role="alert" class="fui-alert">
            <p class="fui-alert-title">{icon(Glyph::CIRCLE_ALERT, "16")} {title}</p>
            {issues}
        </div>
    }
}

/// Returns the console's own fault `message` as an alert.
#[must_use]
pub fn fault(message: impl Into<String>) -> impl IntoView {
    view! {
        <div role="alert" class="fui-alert">
            <p class="fui-alert-title">{icon(Glyph::TRIANGLE_ALERT, "16")} {message.into()}</p>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{Issue, Severity, shown_issues};

    /// An issue of `severity`, carrying `text` when given.
    fn issue(severity: Severity, text: Option<&str>) -> Issue {
        Issue {
            severity,
            code: String::from("processing"),
            text: text.map(str::to_owned),
            place: String::new(),
        }
    }

    #[test]
    fn an_information_issue_with_no_text_is_counted_and_others_are_listed_gravest_first() {
        let issues = [
            issue(Severity::Information, None),
            issue(Severity::Warning, None),
            issue(Severity::Information, Some("kept")),
            issue(Severity::Fatal, Some("stop")),
            issue(Severity::Information, None),
        ];
        let (shown, left_out) = shown_issues(&issues);
        assert_eq!(2, left_out, "two silent information issues");
        let severities: Vec<Severity> = shown.iter().map(|issue| issue.severity).collect();
        assert_eq!(
            vec![Severity::Fatal, Severity::Warning, Severity::Information],
            severities,
            "gravest first, a warning with no text kept"
        );
    }
}
