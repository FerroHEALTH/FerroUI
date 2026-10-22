// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the charts page.

use super::Msg;

messages! {
    /// `Activity`
    Activity = "activity",
    /// `Commits per day`
    CommitsPerDay = "commits-per-day",
    /// `Drawn as SVG on the server, scaled to the card, each with its figures for assistive...`
    DrawnSvgServerScaledCard = "drawn-svg-server-scaled-card",
    /// `2 accepted senders`
    FlowAccepted = "flow-accepted",
    /// `Degraded · answers 503`
    FlowAnswers503 = "flow-answers-503",
    /// `Bridge`
    FlowBridge = "flow-bridge",
    /// `The bridge as a flow`
    FlowBridgeAsFlow = "flow-bridge-as-flow",
    /// `The bridge as a flow, lane by lane`
    FlowBridgeLaneByLane = "flow-bridge-lane-by-lane",
    /// `Callers`
    FlowCallers = "flow-callers",
    /// `openEHR CDR`
    FlowCdr = "flow-cdr",
    /// `FerroEHR · Up`
    FlowCdrUp = "flow-cdr-up",
    /// `9 contexts · 4 OMOCL`
    FlowContexts = "flow-contexts",
    /// `FHIR facade`
    FlowFacade = "flow-facade",
    /// `On · /fhir`
    FlowFacadeOn = "flow-facade-on",
    /// `Faces`
    FlowFaces = "flow-faces",
    /// `FHIR clients`
    FlowFhirClients = "flow-fhir-clients",
    /// `2 faces on · 1 of 2 upstreams up, 1 degraded.`
    FlowFootProblems = "flow-foot-problems",
    /// `2 faces on · 2 of 2 upstreams up.`
    FlowFootSteady = "flow-foot-steady",
    /// `HL7 v2`
    FlowHl7 = "flow-hl7",
    /// `Off`
    FlowHl7Off = "flow-hl7-off",
    /// `On · MLLP :2575`
    FlowHl7On = "flow-hl7-on",
    /// `HL7 v2 senders`
    FlowHl7Senders = "flow-hl7-senders",
    /// `Mapping set`
    FlowMappingSet = "flow-mapping-set",
    /// `No answer · 3 attempts`
    FlowNoAnswer = "flow-no-answer",
    /// `no answer`
    FlowNoAnswerLabel = "flow-no-answer-label",
    /// `SMART App Launch`
    FlowSmart = "flow-smart",
    /// `Terminology server`
    FlowTerm = "flow-term",
    /// `FerroTERM · Up`
    FlowTermUp = "flow-term-up",
    /// `Flow`
    FlowTitle = "flow-title",
    /// `Flow with problem edges`
    FlowTitleProblems = "flow-title-problems",
    /// `Upstreams`
    FlowUpstreams = "flow-upstreams",
    /// `Fri`
    Fri = "fri",
    /// `Line chart`
    LineChart = "line-chart",
    /// `Mon`
    Mon = "mon",
    /// `48 synthetic commits over 30 days, most on day 21.`
    N48SyntheticCommitsOver30 = "n-48-synthetic-commits-over-30",
    /// `Oct 18`
    Oct18 = "oct-18",
    /// `Reads`
    Reads = "reads",
    /// `Refusals`
    Refusals = "refusals",
    /// `Requests per day`
    RequestsPerDay = "requests-per-day",
    /// `Rows written per table`
    RunBarlistLabel = "run-barlist-label",
    /// `Of 860 compositions read: 812 written, 48 skipped, 0 refused.`
    RunCleanSentence = "run-clean-sentence",
    /// `compositions read`
    RunCompositionsRead = "run-compositions-read",
    /// `Refused`
    RunRefused = "run-refused",
    /// `Run report`
    RunReport = "run-report",
    /// `Run report with nothing refused`
    RunReportClean = "run-report-clean",
    /// `Rows per table`
    RunRowsPerTable = "run-rows-per-table",
    /// `Of 1,240 compositions read: 1,180 written, 52 skipped, 8 refused.`
    RunSentence = "run-sentence",
    /// `Skipped`
    RunSkipped = "run-skipped",
    /// `Written`
    RunWritten = "run-written",
    /// `Sat`
    Sat = "sat",
    /// `Score at a threshold`
    ScoreAtThreshold = "score-at-threshold",
    /// `Birth date: same`
    ScoreBirthDate = "score-birth-date",
    /// `Score breakdown`
    ScoreBreakdown = "score-breakdown",
    /// `Score in a table cell`
    ScoreCells = "score-cells",
    /// `Certain`
    ScoreCertain = "score-certain",
    /// `exact match`
    ScoreExact = "score-exact",
    /// `Family name: close match`
    ScoreFamilyName = "score-family-name",
    /// `Given name: different`
    ScoreGivenName = "score-given-name",
    /// `Jaro-Winkler 0.41`
    ScoreJaroFar = "score-jaro-far",
    /// `Jaro-Winkler 0.94`
    ScoreJaroNear = "score-jaro-near",
    /// `missing on one side`
    ScoreMissing = "score-missing",
    /// `Possible`
    ScorePossible = "score-possible",
    /// `Postal code: no value`
    ScorePostalCode = "score-postal-code",
    /// `Probable`
    ScoreProbable = "score-probable",
    /// `Sex: same`
    ScoreSex = "score-sex",
    /// `Sep 19`
    Sep19 = "sep-19",
    /// `Sun`
    Sun = "sun",
    /// `Three synthetic series over a week: reads rise to 48 on Friday, writes stay near 12,...`
    ThreeSyntheticSeriesOverWeek = "three-synthetic-series-over-week",
    /// `Thu`
    Thu = "thu",
    /// `Tue`
    Tue = "tue",
    /// `Wed`
    Wed = "wed",
    /// `Writes`
    Writes = "writes",
}
