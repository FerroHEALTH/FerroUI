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

use crate::i18n::{Text, Word, tr, tr_with};
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
    /// The word a reader sees, in the page's locale.
    #[must_use]
    pub fn word(self) -> Word {
        tr(self.text())
    }

    /// The word as a view, marked English on a page in another language
    /// when no catalog of the page's language holds it.
    fn drawn(self) -> AnyView {
        tr_with(self.text(), &[]).into_render()
    }

    /// The word's message.
    const fn text(self) -> Text {
        match self {
            Self::Fatal => Text::SeverityFatal,
            Self::Error => Text::SeverityError,
            Self::Warning => Text::SeverityWarning,
            Self::Information => Text::SeverityInformation,
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
    /// when no answer came, which the title's visually hidden lead says in
    /// words.
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

/// The issues [`issue_list`] lists in the open, gravest first.
pub const ISSUES_OPEN: usize = 20;

/// The most issues [`issue_list`] draws: past [`ISSUES_OPEN`] they fold
/// under a "more" summary, and past this many it counts the rest.
pub const ISSUES_DRAWN: usize = 200;

/// The issues of `shown` split as [`issue_list`] draws them: the ones in the
/// open, the ones folded under "more", and how many are counted only.
#[must_use]
pub fn split_issues<'a>(shown: &'a [&'a Issue]) -> (&'a [&'a Issue], &'a [&'a Issue], usize) {
    let drawn = shown.len().min(ISSUES_DRAWN);
    let (drawn, counted) = shown.split_at(drawn);
    let (open, folded) = drawn.split_at(drawn.len().min(ISSUES_OPEN));
    (open, folded, counted.len())
}

/// Returns `issues` as a list, gravest first, each with its severity as a
/// word, its code, where it points and its text, and a sentence for the
/// information issues left out.
///
/// An upstream may answer with thousands of issues: the list shows the
/// [`ISSUES_OPEN`] gravest, folds the rest up to [`ISSUES_DRAWN`] under a
/// native `<details>`, which opens with no script, and counts any past that.
#[must_use]
pub fn issue_list(issues: &[Issue]) -> impl IntoView + use<> {
    let (shown, left_out) = shown_issues(issues);
    let (open, folded, counted) = split_issues(&shown);
    let items = issue_items(open);
    let more = (!folded.is_empty()).then(|| {
        let summary = tr_with(Text::IssuesMore, &[("count", folded.len().into())]);
        let items = issue_items(folded);
        view! {
            <details class="fui-issues-more">
                <summary>{summary}</summary>
                <ul class="fui-issues">{items}</ul>
            </details>
        }
    });
    let not_drawn = (counted > 0).then(|| {
        view! {
            <p class="fui-muted">{tr_with(Text::IssuesNotDrawn, &[("count", counted.into())])}</p>
        }
    });
    let left = (left_out > 0).then(|| {
        view! {
            <p class="fui-muted">
                {tr_with(Text::IssuesLeftOut, &[("count", left_out.into())])}
            </p>
        }
    });
    view! {
        <ul class="fui-issues">{items}</ul>
        {more}
        {not_drawn}
        {left}
    }
}

/// Returns `issues` as the items of an issue list.
fn issue_items(issues: &[&Issue]) -> impl IntoView + use<> {
    issues
        .iter()
        .map(|issue| {
            let text = match &issue.text {
                Some(text) => text.clone().into_any(),
                None => view! {
                    <span class="fui-muted">{tr_with(Text::IssueBlank, &[])}</span>
                }
                .into_any(),
            };
            let place = (!issue.place.is_empty()).then(|| {
                view! {
                    " " {tr_with(Text::IssuePlace, &[])} " "
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
                    <span class=issue.severity.tone().badge()>{issue.severity.drawn()}</span>
                    <span class="fui-issue-head">
                        <code>{issue.code.clone()}</code>
                        {place}
                        ":"
                    </span>
                    <span class="fui-issue-text">{text} {details}</span>
                </li>
            }
        })
        .collect_view()
}

/// The visible title of `refusal`: its status before its summary, or the
/// summary alone when no answer came, as the design draws it.
///
/// [`refusal`] leads a title with no status with "No answer" for
/// assistive technology alone ([`refusal_lead`]).
#[must_use]
pub fn refusal_title(refusal: &Refusal) -> Word {
    match refusal.status {
        Some(status) => tr_with(
            Text::RefusalStatus,
            &[
                ("status", status.to_string().into()),
                ("summary", refusal.summary.clone().into()),
            ],
        ),
        None => Word::new(refusal.summary.clone()),
    }
}

/// The visually hidden lead of the title of `refusal`: "No answer" when
/// none came, so assistive technology says so, or `None` when the title's
/// status says what came.
#[must_use]
pub fn refusal_lead(refusal: &Refusal) -> Option<Word> {
    refusal.status.is_none().then(|| tr(Text::RefusalNoAnswer))
}

/// Returns `refusal` as an alert: its status, its summary, the request it
/// answered, its issues, and the evidence of a body that held none.
///
/// The request and the evidence are text, never markup. The title alone is
/// the live region (WAI-ARIA 1.2, `alert`), so an assistive technology says
/// what was refused and never reads a long list of issues at once.
#[must_use]
pub fn refusal(refusal: &Refusal) -> impl IntoView + use<> {
    let title = refusal_title(refusal);
    let lead = refusal_lead(refusal).map(|lead| {
        view! {
            <span class="fui-sr-only">{lead}</span>
            " "
        }
    });
    let request = refusal
        .request
        .clone()
        .map(|request| view! { <p class="fui-alert-request"><code>{request}</code></p> });
    let issues = (!refusal.issues.is_empty()).then(|| issue_list(&refusal.issues));
    let evidence = refusal.evidence.clone().map(|evidence| {
        view! {
            <div
                class="fui-code-panel fui-alert-evidence"
                role="region"
                aria-label=tr(Text::RefusalEvidence)
                tabindex="0"
            >
                <pre>{evidence}</pre>
            </div>
        }
    });
    view! {
        <div class="fui-alert">
            <p class="fui-alert-title" role="alert">{icon(Glyph::CIRCLE_ALERT, "16")} {lead}{title}</p>
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
    use super::{Issue, Refusal, Severity, refusal_lead, refusal_title, shown_issues};
    use crate::i18n::Word;

    /// An issue of `severity`, carrying `text` when given.
    fn issue(severity: Severity, text: Option<&str>) -> Issue {
        let issue = Issue::new(severity, "processing");
        match text {
            Some(text) => issue.text(text),
            None => issue,
        }
    }

    #[test]
    fn a_refusal_with_no_status_titles_itself_with_its_summary_alone() {
        let none = Refusal {
            summary: String::from("The terminology server did not answer."),
            ..Refusal::default()
        };
        assert_eq!(
            "The terminology server did not answer.",
            refusal_title(&none)
        );
        let answered = Refusal {
            status: Some(404),
            ..none
        };
        assert_eq!(
            "404: The terminology server did not answer.",
            refusal_title(&answered)
        );
    }

    #[test]
    fn a_refusal_with_no_status_leads_with_no_answer_and_one_with_a_status_does_not() {
        let none = Refusal {
            summary: String::from("The terminology server did not answer."),
            ..Refusal::default()
        };
        assert_eq!(
            Some("No answer:"),
            refusal_lead(&none).as_ref().map(Word::text)
        );
        let answered = Refusal {
            status: Some(503),
            ..none
        };
        assert_eq!(None, refusal_lead(&answered));
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
