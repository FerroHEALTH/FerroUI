// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the clinical records page.

use super::Msg;

messages! {
    /// `{ $day } Aug`
    ClAugust = "cl-august",
    /// `Blood pressure`
    ClBloodPressure = "cl-blood-pressure",
    /// `Body temperature`
    ClBodyTemperature = "cl-body-temperature",
    /// `Calibration`
    ClCalibration = "cl-calibration",
    /// `Creation`
    ClChangeCreation = "cl-change-creation",
    /// `Deletion`
    ClChangeDeletion = "cl-change-deletion",
    /// `Modification`
    ClChangeModification = "cl-change-modification",
    /// `Systolic and diastolic pressure over time`
    ClChartLabel = "cl-chart-label",
    /// `Systolic from 151 to 128 and diastolic from 94 to 82 mm Hg, 11 Aug to 21 Oct 2026.`
    ClChartSummary = "cl-chart-summary",
    /// `Composer`
    ClComposer = "cl-composer",
    /// `Context`
    ClContext = "cl-context",
    /// `Cuff size`
    ClCuffSize = "cl-cuff-size",
    /// `Device`
    ClDevice = "cl-device",
    /// `Device name`
    ClDeviceName = "cl-device-name",
    /// `Diastolic`
    ClDiastolic = "cl-diastolic",
    /// `The uids, the language, the territory and the encoding are folded away.`
    ClFolded = "cl-folded",
    /// `Last calibrated`
    ClLastCalibrated = "cl-last-calibrated",
    /// `{ $count } min ago`
    ClMinutesAgo = "cl-minutes-ago",
    /// `No information: unknown`
    ClNoInformation = "cl-no-information",
    /// `Not a SMART on openEHR scope. Use composition-<template>, for example composition-*.rs.`
    ClNotOpenehr = "cl-not-openehr",
    /// `{ $day } Oct`
    ClOctober = "cl-october",
    /// `Position`
    ClPosition = "cl-position",
    /// `Pulse`
    ClPulse = "cl-pulse",
    /// `Query builder`
    ClQuery = "cl-query",
    /// `Rate`
    ClRate = "cl-rate",
    /// `Clinical reading`
    ClReading = "cl-reading",
    /// `Regularity`
    ClRegularity = "cl-regularity",
    /// `An answer as a chart`
    ClResultsChart = "cl-results-chart",
    /// `An answer as a table`
    ClResultsTable = "cl-results-table",
    /// `Results`
    ClResultsTitle = "cl-results-title",
    /// `Rows 1 to 6 of 6, inside the query's own LIMIT 50.`
    ClResultsWindow = "cl-results-window",
    /// `Scope grant`
    ClScopes = "cl-scopes",
    /// `Scopes the app holds`
    ClScopesHeld = "cl-scopes-held",
    /// `{ $day } Sep`
    ClSeptember = "cl-september",
    /// `Setting`
    ClSetting = "cl-setting",
    /// `Show the full document`
    ClShowDocument = "cl-show-document",
    /// `Start time`
    ClStartTime = "cl-start-time",
    /// `Subject line`
    ClSubject = "cl-subject",
    /// `A cleared state`
    ClSubjectCleared = "cl-subject-cleared",
    /// `Masked, with Reveal`
    ClSubjectMasked = "cl-subject-masked",
    /// `No external subject`
    ClSubjectNone = "cl-subject-none",
    /// `The parts a console reads a clinical record with, on synthetic data.`
    ClSubtitle = "cl-subtitle",
    /// `Systolic`
    ClSystolic = "cl-systolic",
    /// `Temperature`
    ClTemperature = "cl-temperature",
    /// `Version history`
    ClVersions = "cl-versions",
    /// `Versions of a deleted composition`
    ClVersionsDeleted = "cl-versions-deleted",
    /// `Opens the version current then.`
    ClVersionsOpens = "cl-versions-opens",
    /// `Versions`
    ClVersionsTitle = "cl-versions-title",
}
