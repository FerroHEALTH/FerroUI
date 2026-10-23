// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the charts.

use crate::i18n::Text;

texts! {
    /// The caption of a chart's table: `$chart` is the chart's name.
    ChartAsTable = "chart-as-table",
    /// The legend's note for a gap in a series.
    ChartNoValue = "chart-no-value",
    /// A cell of a chart's table with no value.
    ChartNoValueCell = "chart-no-value-cell",
    /// The accessible name of a chart's legend.
    ChartSeries = "chart-series",
    /// The header of a chart table's X column.
    ChartX = "chart-x",
    /// The accessible name of a flow's card: `$title` its name, `$note` its
    /// line, `$status` its state.
    FlowCard = "flow-card",
    /// A problem edge of a flow: `$from` and `$to` its ends, `$status` its
    /// state.
    FlowEdge = "flow-edge",
    /// A problem edge of a flow with its label as `$note`.
    FlowEdgeNoted = "flow-edge-noted",
    /// The fold link of an open lane: `$title` the lane's title.
    FlowFold = "flow-fold",
    /// The accessible name of a lane's summary card: `$count` the cards it
    /// holds, `$title` the lane's title, `$note` its counts.
    FlowFoldCard = "flow-fold-card",
    /// One state's count on a summary card: `$status` the state, `$count`
    /// how many.
    FlowFoldCount = "flow-fold-count",
    /// Two counts of a summary card's line, `$first` and `$last`, joined.
    FlowFoldJoin = "flow-fold-join",
    /// The name of a lane's summary card: `$count` the cards it holds.
    FlowFoldName = "flow-fold-name",
    /// The accessible name of a flow's hub: `$title` its name, `$note` its
    /// line.
    FlowHub = "flow-hub",
    /// The button that opens a flow's edge rule.
    FlowRuleButton = "flow-rule-button",
    /// The title of a flow's edge rule dialog.
    FlowRuleTitle = "flow-rule-title",
    /// A flow's degraded state.
    FlowDegraded = "flow-state-degraded",
    /// A flow's state that no probe has read.
    FlowNotChecked = "flow-state-not-checked",
    /// A flow's state of a part that is switched off.
    FlowOff = "flow-state-off",
    /// A flow's unreachable state.
    FlowUnreachable = "flow-state-unreachable",
    /// A flow's reachable state.
    FlowUp = "flow-state-up",
    /// The description of a flow with no problem edge.
    FlowSteady = "flow-steady",
    /// The line of a degraded edge.
    FlowStyleDegraded = "flow-style-degraded",
    /// The line of an edge no probe has read.
    FlowStyleNotChecked = "flow-style-not-checked",
    /// The line of an edge that is switched off.
    FlowStyleOff = "flow-style-off",
    /// The line of an unreachable edge.
    FlowStyleUnreachable = "flow-style-unreachable",
    /// The line of an edge that answers.
    FlowStyleUp = "flow-style-up",
    /// The fold link of a folded lane: `$title` the lane's title.
    FlowUnfold = "flow-unfold",
    /// One contribution in a score's name: `$title` its words, `$value` its
    /// signed weight.
    ScoreContribution = "score-contribution",
    /// The name of a score's scale: `$value` the score, `$status` the grade,
    /// `$range` the thresholds, `$summary` the contributions.
    ScoreName = "score-name",
    /// One threshold in a score's name: `$title` its grade, `$value` where it
    /// starts.
    ScoreThreshold = "score-threshold",
    /// The words before a score's total weight.
    ScoreTotalWeight = "score-total-weight",
    /// The word before a score's figure.
    ScoreWord = "score-word",
}
