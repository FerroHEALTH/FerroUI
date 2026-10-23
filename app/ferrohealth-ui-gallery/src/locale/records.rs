// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the patient-record components: the record compare and
//! the work-list on the tables page, the timeline on the layouts page and
//! the masked identifiers on the forms page.

use super::Msg;

messages! {
    /// `Actions`
    RecordsActions = "records-actions",
    /// `Actions for { $name }`
    RecordsActionsFor = "records-actions-for",
    /// `Birth date`
    RecordsBirthDate = "records-birth-date",
    /// `Candidates, sorted by grade, then score, then the longest wait`
    RecordsCandidatesCaption = "records-candidates-caption",
    /// `Record compare`
    RecordsCompare = "records-compare",
    /// `Records of one synthetic person`
    RecordsCompareCaption = "records-compare-caption",
    /// `Two records that agree`
    RecordsCompareCaptionAgree = "records-compare-caption-agree",
    /// `Record compare on a phone`
    RecordsCompareStacked = "records-compare-stacked",
    /// `Domain`
    RecordsDomain = "records-domain",
    /// `Domains`
    RecordsDomains = "records-domains",
    /// `Created from the hospital feed`
    RecordsEventCreated = "records-event-created",
    /// `Linked two records of other countries`
    RecordsEventLinked = "records-event-linked",
    /// `Merged a second person into this one`
    RecordsEventMerged = "records-event-merged",
    /// `Version 3 of the demographics`
    RecordsEventVersion = "records-event-version",
    /// `Family name`
    RecordsFamilyName = "records-family-name",
    /// `Given names`
    RecordsGivenNames = "records-given-names",
    /// `Grade`
    RecordsGrade = "records-grade",
    /// `History of a synthetic person`
    RecordsHistory = "records-history",
    /// `Version 3 against version 2`
    RecordsHistoryVersion = "records-history-version",
    /// `Identifier`
    RecordsIdentifier = "records-identifier",
    /// `Identifiers`
    RecordsIdentifiers = "records-identifiers",
    /// `Identifiers of a synthetic person`
    RecordsIdentifiersOf = "records-identifiers-of",
    /// `created`
    RecordsKindCreated = "records-kind-created",
    /// `linked automatically`
    RecordsKindLinked = "records-kind-linked",
    /// `merged`
    RecordsKindMerged = "records-kind-merged",
    /// `new version`
    RecordsKindVersion = "records-kind-version",
    /// `Masked identifiers`
    RecordsMasked = "records-masked",
    /// `A reveal shows the value for 30 seconds and is recorded.`
    RecordsMaskedNote = "records-masked-note",
    /// `Matcher`
    RecordsMatcher = "records-matcher",
    /// `move`
    RecordsMove = "records-move",
    /// `open`
    RecordsOpen = "records-open",
    /// `Open the pair`
    RecordsOpenPair = "records-open-pair",
    /// `Postal code`
    RecordsPostalCode = "records-postal-code",
    /// `Same person, confirmed by phone with the synthetic clinic.`
    RecordsReasonMerged = "records-reason-merged",
    /// `The family name was misspelt at registration.`
    RecordsReasonVersion = "records-reason-version",
    /// `Record A`
    RecordsRecordA = "records-record-a",
    /// `Record B`
    RecordsRecordB = "records-record-b",
    /// `the hourly limit is reached; try again at 10:05`
    RecordsRefusedWhy = "records-refused-why",
    /// `a console's reveal page asks the reason here and reveals with a plain form.`
    RecordsRevealPage = "records-reveal-page",
    /// `Leave it masked`
    RecordsRevealStay = "records-reveal-stay",
    /// `Reveal { $name }`
    RecordsRevealTitle = "records-reveal-title",
    /// `Review queue`
    RecordsReviewQueue = "records-review-queue",
    /// `Select { $name }`
    RecordsSelect = "records-select",
    /// `Select every candidate on this page`
    RecordsSelectEvery = "records-select-every",
    /// `Sorted by grade, then score, then the longest wait`
    RecordsSortWords = "records-sort-words",
    /// `Synthetic steward`
    RecordsSteward = "records-steward",
    /// `Timeline`
    RecordsTimeline = "records-timeline",
    /// `Waiting`
    RecordsWaiting = "records-waiting",
}
