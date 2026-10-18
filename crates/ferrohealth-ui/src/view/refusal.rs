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
    /// The codes that class it (an `OperationOutcome` issue's
    /// `details.coding` codes), shown after its text.
    pub details: Vec<String>,
}

impl Issue {
    /// An issue of `severity` with `code`, carrying no text, pointing
    /// nowhere, with no detail codes.
    #[must_use]
    pub fn new(severity: Severity, code: impl Into<String>) -> Self {
        Self {
            severity,
            code: code.into(),
            text: None,
            place: String::new(),
            details: Vec::new(),
        }
    }

    /// This issue, carrying `text`.
    #[must_use]
    pub fn text(self, text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..self
        }
    }

    /// This issue, pointing at `place`.
    #[must_use]
    pub fn place(self, place: impl Into<String>) -> Self {
        Self {
            place: place.into(),
            ..self
        }
    }

    /// This issue, classed by the codes `details`.
    #[must_use]
    pub fn details(self, details: Vec<String>) -> Self {
        Self { details, ..self }
    }
}

/// What a refusal shows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Refusal {
    /// The upstream's status, such as `422`, when it answered one; `None`
    /// when no answer came, which the title says in words.
    pub status: Option<u16>,
    /// The sentence that says what was refused.
    pub summary: String,
    /// The issues it answered with.
    pub issues: Vec<Issue>,
    /// The request the console made, so a reader can repeat it by hand.
    pub request: Option<String>,
    /// The start of a body that carried no readable issues, or the
    /// transport's message, kept as evidence and shown as text.
    pub evidence: Option<String>,
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
            let details = issue
                .details
                .iter()
                .map(|code| view! { <code class="fui-issue-detail">{code.clone()}</code> })
                .collect_view();
            view! {
                <li>
                    <span class=issue.severity.tone().badge()>{issue.severity.word()}</span>
                    <span class="fui-issue-head">
                        <code>{issue.code.clone()}</code>
                        {place}
                        ":"
                    </span>
                    <span class="fui-issue-text">{text} {details}</span>
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

/// The title of `refusal`: its status, or "No answer" when none came,
/// before its summary.
#[must_use]
pub fn refusal_title(refusal: &Refusal) -> String {
    match refusal.status {
        Some(status) => format!("{status}: {}", refusal.summary),
        None => format!("No answer: {}", refusal.summary),
    }
}

/// Returns `refusal` as an alert: its status, its summary, the request it
/// answered, its issues, and the evidence of a body that held none.
///
/// The request and the evidence are text, never markup.
#[must_use]
pub fn refusal(refusal: &Refusal) -> impl IntoView + use<> {
    let title = refusal_title(refusal);
    let request = refusal
        .request
        .clone()
        .map(|request| view! { <p class="fui-alert-request"><code>{request}</code></p> });
    let issues = (!refusal.issues.is_empty()).then(|| issue_list(&refusal.issues));
    let evidence = refusal.evidence.clone().map(|evidence| {
        view! {
            <div class="fui-code-panel fui-alert-evidence" role="region" aria-label="What the server sent" tabindex="0">
                <pre>{evidence}</pre>
            </div>
        }
    });
    view! {
        <div role="alert" class="fui-alert">
            <p class="fui-alert-title">{icon(Glyph::CIRCLE_ALERT, "16")} {title}</p>
            {request}
            {issues}
            {evidence}
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
    use super::{Issue, Refusal, Severity, refusal_title, shown_issues};

    /// An issue of `severity`, carrying `text` when given.
    fn issue(severity: Severity, text: Option<&str>) -> Issue {
        let issue = Issue::new(severity, "processing");
        match text {
            Some(text) => issue.text(text),
            None => issue,
        }
    }

    #[test]
    fn a_refusal_with_no_status_says_no_answer_came() {
        let none = Refusal {
            summary: String::from("the terminology server did not answer."),
            ..Refusal::default()
        };
        assert_eq!(
            "No answer: the terminology server did not answer.",
            refusal_title(&none)
        );
        let answered = Refusal {
            status: Some(404),
            ..none
        };
        assert_eq!(
            "404: the terminology server did not answer.",
            refusal_title(&answered)
        );
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
