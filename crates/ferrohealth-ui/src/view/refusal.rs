// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The refusal and the fault: an upstream's status and the issues it
//! answered with, each with the text it carries, and the console's own
//! fault. An error never renders as nothing.
//!
//! The library reads no upstream document itself: a console turns an
//! `OperationOutcome` or a diagnostic list into [`Issue`]s with the types
//! its own crates model, and the refusal draws them.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr, tr_with};
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
    pub fn word(self, locale: &Locale) -> Word {
        tr(locale, self.text())
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
pub fn issue_list(locale: &Locale, issues: &[Issue]) -> Html {
    let (shown, left_out) = shown_issues(issues);
    let (open, folded, counted) = split_issues(&shown);
    IssuesView {
        open: issue_items(locale, open),
        more: (!folded.is_empty()).then(|| {
            (
                tr_with(locale, Text::IssuesMore, &[("count", folded.len().into())]),
                issue_items(locale, folded),
            )
        }),
        not_drawn: (counted > 0)
            .then(|| tr_with(locale, Text::IssuesNotDrawn, &[("count", counted.into())])),
        left: (left_out > 0)
            .then(|| tr_with(locale, Text::IssuesLeftOut, &[("count", left_out.into())])),
    }
    .html()
}

#[derive(Template)]
#[template(path = "refusal/refusal.html", block = "issues")]
struct IssuesView {
    open: Html,
    more: Option<(Word, Html)>,
    not_drawn: Option<Word>,
    left: Option<Word>,
}

/// One issue as the list draws it.
struct IssueView<'a> {
    badge: &'static str,
    severity: Word,
    code: &'a str,
    place: Option<&'a str>,
    text: Option<&'a str>,
    details: &'a [String],
}

#[derive(Template)]
#[template(path = "refusal/refusal.html", block = "items")]
struct ItemsView<'a> {
    issues: Vec<IssueView<'a>>,
    place_words: Word,
    blank: Word,
}

/// Returns `issues` as the items of an issue list.
fn issue_items(locale: &Locale, issues: &[&Issue]) -> Html {
    ItemsView {
        issues: issues
            .iter()
            .map(|issue| IssueView {
                badge: issue.severity.tone().badge(),
                severity: issue.severity.word(locale),
                code: &issue.code,
                place: Some(issue.place.as_str()).filter(|place| !place.is_empty()),
                text: issue.text.as_deref(),
                details: &issue.details,
            })
            .collect(),
        place_words: tr(locale, Text::IssuePlace),
        blank: tr(locale, Text::IssueBlank),
    }
    .html()
}

/// The visible title of `refusal`: its status before its summary, or the
/// summary alone when no answer came, as the design draws it.
///
/// [`refusal`] leads a title with no status with "No answer" for
/// assistive technology alone ([`refusal_lead`]).
#[must_use]
pub fn refusal_title(locale: &Locale, refusal: &Refusal) -> Word {
    match refusal.status {
        Some(status) => tr_with(
            locale,
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
pub fn refusal_lead(locale: &Locale, refusal: &Refusal) -> Option<Word> {
    refusal
        .status
        .is_none()
        .then(|| tr(locale, Text::RefusalNoAnswer))
}

/// Returns `refusal` as an alert: its status, its summary, the request it
/// answered, its issues, and the evidence of a body that held none.
///
/// The request and the evidence are text, never markup. The title alone is
/// the live region (WAI-ARIA 1.2, `alert`), so an assistive technology says
/// what was refused and never reads a long list of issues at once.
#[must_use]
pub fn refusal(locale: &Locale, refusal: &Refusal) -> Html {
    RefusalView {
        glyph: icon(Glyph::CIRCLE_ALERT, "16"),
        lead: refusal_lead(locale, refusal),
        title: refusal_title(locale, refusal),
        request: refusal.request.as_deref(),
        issues: if refusal.issues.is_empty() {
            Html::empty()
        } else {
            issue_list(locale, &refusal.issues)
        },
        evidence: refusal.evidence.as_deref(),
        evidence_label: tr(locale, Text::RefusalEvidence),
    }
    .html()
}

#[derive(Template)]
#[template(path = "refusal/refusal.html", block = "refusal")]
struct RefusalView<'a> {
    glyph: Html,
    lead: Option<Word>,
    title: Word,
    request: Option<&'a str>,
    issues: Html,
    evidence: Option<&'a str>,
    evidence_label: Word,
}

/// Returns the console's own fault `message` as an alert.
#[must_use]
pub fn fault(message: impl Into<String>) -> Html {
    Html::raw(format!(
        "<div role=\"alert\" class=\"fui-alert\"><p class=\"fui-alert-title\">{} {}</p></div>",
        icon(Glyph::TRIANGLE_ALERT, "16"),
        Html::text(message.into())
    ))
}

#[cfg(test)]
mod tests {
    use super::{Issue, Refusal, Severity, refusal_lead, refusal_title, shown_issues};
    use crate::i18n::{Locale, Word};

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
            refusal_title(&Locale::english(), &none)
        );
        let answered = Refusal {
            status: Some(404),
            ..none
        };
        assert_eq!(
            "404: The terminology server did not answer.",
            refusal_title(&Locale::english(), &answered)
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
            refusal_lead(&Locale::english(), &none)
                .as_ref()
                .map(Word::text)
        );
        let answered = Refusal {
            status: Some(503),
            ..none
        };
        assert_eq!(None, refusal_lead(&Locale::english(), &answered));
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
