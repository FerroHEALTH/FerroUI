// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the clinical record parts: the subject line, the
//! version history, the clinical reading, the condition groups, the results
//! pane and the scope grant.

use crate::i18n::Text;

texts! {
    /// The ghost button that adds a condition to the query's first group.
    CondAddCondition = "cond-add-condition",
    /// The name of Add a condition in a nested group: `$label` is the group's number.
    CondAddConditionTo = "cond-add-condition-to",
    /// The ghost button that adds a nested group.
    CondAddGroup = "cond-add-group",
    /// The segment of a group that matches every condition.
    CondAll = "cond-all",
    /// The segment of a group that matches any condition.
    CondAny = "cond-any",
    /// The name of a group's All or Any choice, the head as one phrase: `$label` is its number.
    CondGroupLabel = "cond-group-label",
    /// The word before a group's All or Any choice.
    CondMatch = "cond-match",
    /// The check box that negates a group.
    CondNot = "cond-not",
    /// The words after a group's All or Any choice.
    CondOfThese = "cond-of-these",
    /// The label of a condition's operator.
    CondOperator = "cond-operator",
    /// The label of a condition's path.
    CondPath = "cond-path",
    /// The name of a condition's Remove: `$label` is its path's words.
    CondRemove = "cond-remove",
    /// The name of a nested group's Remove: `$label` is its number.
    CondRemoveGroup = "cond-remove-group",
    /// The label of a condition's value.
    CondValue = "cond-value",
    /// The tooltip of a quantity's unit: `$code` is its UCUM code.
    ReadingUcum = "reading-ucum",
    /// The segment that shows a query's answer as a chart.
    ResultsChart = "results-chart",
    /// The action that downloads a query's answer as CSV.
    ResultsExportCsv = "results-export-csv",
    /// The action that downloads a query's answer as JSON.
    ResultsExportJson = "results-export-json",
    /// The name of the Table or Chart switch.
    ResultsShowAs = "results-show-as",
    /// The segment that shows a query's answer as a table.
    ResultsTable = "results-table",
    /// The warn badge of a broad grant.
    ScopeBroad = "scope-broad",
    /// The operation `c` of a scope.
    ScopeCreate = "scope-create",
    /// The operation `d` of a scope.
    ScopeDelete = "scope-delete",
    /// What a scope with no compartment lacks.
    ScopeExpectedCompartment = "scope-expected-compartment",
    /// What a scope with no valid permissions lacks.
    ScopeExpectedPermissions = "scope-expected-permissions",
    /// What a scope with no resource lacks.
    ScopeExpectedResource = "scope-expected-resource",
    /// The hidden words after an operation a scope does not permit.
    ScopeNotPermitted = "scope-not-permitted",
    /// The compartment `patient` in words.
    ScopePatient = "scope-patient",
    /// The operation `r` of a scope.
    ScopeRead = "scope-read",
    /// The operation `s` of a scope.
    ScopeSearch = "scope-search",
    /// The compartment `system` in words.
    ScopeSystem = "scope-system",
    /// The operation `u` of a scope.
    ScopeUpdate = "scope-update",
    /// The compartment `user` in words.
    ScopeUser = "scope-user",
    /// The label before a record's subject.
    SubjectLabel = "subject-label",
    /// The state of a record that takes new versions.
    SubjectModifiable = "subject-modifiable",
    /// The words of a record with no external subject.
    SubjectNone = "subject-none",
    /// The state of a record that takes no new versions.
    SubjectNotModifiable = "subject-not-modifiable",
    /// The state of a record that queries leave out.
    SubjectNotQueryable = "subject-not-queryable",
    /// The state of a record that queries read.
    SubjectQueryable = "subject-queryable",
    /// The label of the At a time field.
    VersionsAtTime = "versions-at-time",
    /// A version's name in its table: `$value` is its number.
    VersionsCell = "versions-cell",
    /// The header of a version's change type.
    VersionsChange = "versions-change",
    /// The header of who committed a version.
    VersionsCommittedBy = "versions-committed-by",
    /// The header of a version's contribution.
    VersionsContribution = "versions-contribution",
    /// The mark of the current version.
    VersionsCurrent = "versions-current",
    /// The state of a version that deleted its object.
    VersionsDeleted = "versions-deleted",
    /// The link from an older version to the latest.
    VersionsLatest = "versions-latest",
    /// The button that opens the version at a time.
    VersionsOpen = "versions-open",
    /// The notice over an older version: `$value` is its number, `$total` how many there are.
    VersionsReading = "versions-reading",
    /// The header of a version's number.
    VersionsVersion = "versions-version",
    /// The header of when a version was committed.
    VersionsWhen = "versions-when",
}
