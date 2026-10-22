// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of a refusal and its issues.

use crate::i18n::Text;

texts! {
    /// An issue that carries no text.
    IssueBlank = "issue-no-text",
    /// The word between an issue's code and where it points.
    IssuePlace = "issue-place",
    /// The information issues with no text that are counted, `$count` of them.
    IssuesLeftOut = "issues-left-out",
    /// The accessible name of the body a refusal keeps as evidence.
    RefusalEvidence = "refusal-evidence",
    /// A refusal's title when no answer came.
    RefusalNoAnswer = "refusal-no-answer",
    /// A refusal's title: the upstream's `$status` and the `$summary`.
    RefusalStatus = "refusal-status",
    /// An issue's severity: error.
    SeverityError = "severity-error",
    /// An issue's severity: fatal.
    SeverityFatal = "severity-fatal",
    /// An issue's severity: information.
    SeverityInformation = "severity-information",
    /// An issue's severity: warning.
    SeverityWarning = "severity-warning",
    /// An instant read in UTC: its `$date` and `$time`.
    TimestampUtc = "timestamp-utc",
}
