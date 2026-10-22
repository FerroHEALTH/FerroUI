// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the patient-record components: the record
//! compare, the timeline, the masked identifier and the work-list.

use crate::i18n::Text;

texts! {
    /// A compare's mark of a field one record lacks.
    CompareAbsent = "compare-absent",
    /// A compare's mark of a field that agrees with the leading record.
    CompareAgrees = "compare-agrees",
    /// The caption of a compare whose fields all agree: `$label` is its caption.
    CompareAllAgree = "compare-all-agree",
    /// A compare's mark of a field that differs from the leading record.
    CompareDiffers = "compare-differs",
    /// The header of a compare's field column.
    CompareField = "compare-field",
    /// The badge of the leading record's column.
    CompareLeading = "compare-leading",
    /// The leading record named before its value beside a differing one: `$title`.
    CompareLeadingValue = "compare-leading-value",
    /// The line under the stacked compare's switch: `$title` is the leading record.
    CompareMarkedAgainst = "compare-marked-against",
    /// The legend of the stacked compare's switch.
    CompareRecordToRead = "compare-record-to-read",
    /// The Copy button of a revealed identifier.
    IdentifierCopy = "identifier-copy",
    /// The visible countdown: `$count` seconds left.
    IdentifierCountdown = "identifier-countdown",
    /// The note once a revealed identifier is hidden again.
    IdentifierExpired = "identifier-expired",
    /// What the live region says when the identifier is hidden again.
    IdentifierHiddenSaid = "identifier-hidden-said",
    /// The Hide button of a revealed identifier.
    IdentifierHide = "identifier-hide",
    /// What the live region says shortly before the identifier hides: `$count` seconds.
    IdentifierLeftSaid = "identifier-left-said",
    /// The hidden words of a mask: `$value` is the part left visible.
    IdentifierMasked = "identifier-masked",
    /// The line of a refused reveal: `$why` is the console's reason.
    IdentifierRefused = "identifier-refused",
    /// The Reveal button.
    IdentifierReveal = "identifier-reveal",
    /// The hidden words before a revealed value: `$label` is its domain.
    IdentifierRevealed = "identifier-revealed",
    /// What the live region says once revealed: `$count` seconds.
    IdentifierRevealedSaid = "identifier-revealed-said",
    /// The busy Reveal button, while the server answers.
    IdentifierRevealing = "identifier-revealing",
    /// The summary of a timeline event's details.
    TimelineDetails = "timeline-details",
    /// The label before a timeline event's reason.
    TimelineReason = "timeline-reason",
}
