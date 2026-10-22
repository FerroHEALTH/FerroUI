// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the operations page.

use super::Msg;

messages! {
    /// `Active concepts only`
    OpActiveOnly = "op-active-only",
    /// `Answered`
    OpAnswered = "op-answered",
    /// `1 Observation · 84 ms · 200`
    OpAnsweredDetail = "op-answered-detail",
    /// `Before the first run`
    OpBeforeFirstRun = "op-before-first-run",
    /// `Bundle`
    OpBundle = "op-bundle",
    /// `The Bundle as JSON`
    OpBundleJson = "op-bundle-json",
    /// `The Bundle, as the bridge sent it`
    OpBundleSent = "op-bundle-sent",
    /// `Canonical`
    OpCanonical = "op-canonical",
    /// `Character set if MSH-18 is empty`
    OpCharacterSet = "op-character-set",
    /// `ASCII, the default`
    OpCharacterSetAscii = "op-character-set-ascii",
    /// `Clear`
    OpClear = "op-clear",
    /// `Composition`
    OpComposition = "op-composition",
    /// `Composition, canonical or FLAT JSON`
    OpCompositionInput = "op-composition-input",
    /// `Convert to FHIR`
    OpConvert = "op-convert",
    /// `Convert to see the Bundle.`
    OpConvertIdle = "op-convert-idle",
    /// `Converted`
    OpConverted = "op-converted",
    /// `Converted the composition to 1 Observation`
    OpConvertedOne = "op-converted-one",
    /// `Converting`
    OpConverting = "op-converting",
    /// `Converting to FHIR`
    OpConvertingFhir = "op-converting-fhir",
    /// `Copy`
    OpCopy = "op-copy",
    /// `Dense options`
    OpDense = "op-dense",
    /// `EHR id`
    OpEhrId = "op-ehr-id",
    /// `errors`
    OpErrors = "op-errors",
    /// `Composition format`
    OpFormat = "op-format",
    /// `HL7 v2 message`
    OpHl7Message = "op-hl7-message",
    /// `Include designations`
    OpIncludeDesignations = "op-include-designations",
    /// `Load report`
    OpLoadReport = "op-load-report",
    /// `The running set is loaded`
    OpLoaded = "op-loaded",
    /// `The running set is loaded with warnings`
    OpLoadedWarnings = "op-loaded-warnings",
    /// `Map the message`
    OpMap = "op-map",
    /// `Mapping`
    OpMapping = "op-mapping",
    /// `mappings`
    OpMappings = "op-mappings",
    /// `Message`
    OpMessage = "op-message",
    /// `The message, one segment per line`
    OpMessageLines = "op-message-lines",
    /// `The message as read`
    OpMessageRead = "op-message-read",
    /// `The message as read, segment by segment`
    OpMessageReadList = "op-message-read-list",
    /// `OMOP row preview`
    OpOmopRows = "op-omop-rows",
    /// `Open the diagnostics`
    OpOpenDiagnostics = "op-open-diagnostics",
    /// `Open the mappings`
    OpOpenMappings = "op-open-mappings",
    /// `Patient`
    OpPatient = "op-patient",
    /// `Preview the rows`
    OpPreview = "op-preview",
    /// `Previewing`
    OpPreviewing = "op-previewing",
    /// `Refused`
    OpRefused = "op-refused",
    /// `1 error · 422`
    OpRefusedDetail = "op-refused-detail",
    /// `the bridge refused the conversion.`
    OpRefusedSummary = "op-refused-summary",
    /// `The body weight unit is lb, which synthetic.vital_signs.v1 does not allow: use kg or g.`
    OpRefusedUnit = "op-refused-unit",
    /// `The reload was refused: the draft has errors`
    OpReloadRefused = "op-reload-refused",
    /// `The request`
    OpRequest = "op-request",
    /// `Rows`
    OpRows = "op-rows",
    /// `in 2 tables`
    OpRowsDetail = "op-rows-detail",
    /// `3 values with no matching concept`
    OpRowsZero = "op-rows-zero",
    /// `Running`
    OpRunning = "op-running",
    /// `Runs { $name }`
    OpRuns = "op-runs",
    /// `Show the warnings`
    OpShowWarnings = "op-show-warnings",
    /// `An operation's outcome, its options and Run, its input beside its answer, and what a message and a composition become.`
    OpSubtitle = "op-subtitle",
    /// `Template`
    OpTemplate = "op-template",
    /// `warning`
    OpWarning = "op-warning",
    /// `warnings`
    OpWarnings = "op-warnings",
    /// `Would answer AA`
    OpWouldAnswer = "op-would-answer",
    /// `4 segments · 1 resource`
    OpWouldAnswerDetail = "op-would-answer-detail",
}
