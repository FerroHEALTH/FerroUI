// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page's score breakdowns and the compact score cell.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::score::{Contribution, Score, Threshold, score_bar, score_breakdown};
use ferrohealth_ui::view::surface::{card, definitions};

use crate::locale::Msg;
use crate::visit::Visit;

/// One contribution of a synthetic score.
fn contribution(v: &Visit, label: Msg, reading: Msg, weight: f64) -> Contribution {
    Contribution {
        label: v.tr(label),
        reading: Some(v.tr(reading)),
        weight,
    }
}

/// The thresholds of the synthetic scores.
fn thresholds(v: &Visit) -> Vec<Threshold> {
    [
        (Msg::ScorePossible, 0.60),
        (Msg::ScoreProbable, 0.80),
        (Msg::ScoreCertain, 0.95),
    ]
    .map(|(label, at)| Threshold {
        label: v.tr(label),
        at,
    })
    .to_vec()
}

/// A synthetic score breakdown: between two thresholds, or at one, with a
/// positive, a negative and a zero weight.
pub(super) fn breakdown(v: &Visit, at_threshold: bool) -> Html {
    let score = if at_threshold {
        Score {
            total: 1.4,
            value: 0.60,
            grade: v.tr(Msg::ScorePossible),
            contributions: vec![
                contribution(v, Msg::ScoreBirthDate, Msg::ScoreExact, 3.1),
                contribution(v, Msg::ScoreGivenName, Msg::ScoreJaroFar, -1.7),
                contribution(v, Msg::ScorePostalCode, Msg::ScoreMissing, 0.0),
            ],
            thresholds: thresholds(v),
        }
    } else {
        Score {
            total: 4.1,
            value: 0.87,
            grade: v.tr(Msg::ScoreProbable),
            contributions: vec![
                contribution(v, Msg::ScoreBirthDate, Msg::ScoreExact, 3.1),
                contribution(v, Msg::ScoreFamilyName, Msg::ScoreJaroNear, 2.4),
                contribution(v, Msg::ScoreSex, Msg::ScoreExact, 0.3),
                contribution(v, Msg::ScorePostalCode, Msg::ScoreMissing, 0.0),
                contribution(v, Msg::ScoreGivenName, Msg::ScoreJaroFar, -1.7),
            ],
            thresholds: thresholds(v),
        }
    };
    let title = if at_threshold {
        Msg::ScoreAtThreshold
    } else {
        Msg::ScoreBreakdown
    };
    card(v.tr(title), score_breakdown(&v.locale, &score))
}

/// A score among four close thresholds, its field's long word wrapping:
/// the labels too close for one row take more.
pub(super) fn close_breakdown(v: &Visit) -> Html {
    let thresholds = [
        (Msg::ScorePossible, 0.80),
        (Msg::ScoreProbable, 0.83),
        (Msg::ScoreCertain, 0.86),
        (Msg::ScoreExactBand, 0.99),
    ]
    .map(|(label, at)| Threshold {
        label: v.tr(label),
        at,
    })
    .to_vec();
    let score = Score {
        total: 2.9,
        value: 0.84,
        grade: v.tr(Msg::ScoreProbable),
        contributions: vec![
            contribution(v, Msg::ScoreLongField, Msg::ScoreExact, 2.4),
            contribution(v, Msg::ScoreSex, Msg::ScoreExact, 0.5),
        ],
        thresholds,
    };
    card(v.tr(Msg::ScoreClose), score_breakdown(&v.locale, &score))
}

/// The compact score of a table cell, once per grade.
pub(super) fn score_cells(v: &Visit) -> Html {
    card(
        v.tr(Msg::ScoreCells),
        definitions(vec![
            (
                v.tr(Msg::ScoreCertain),
                score_bar(&v.locale, 0.97, Tone::Ok),
            ),
            (
                v.tr(Msg::ScoreProbable),
                score_bar(&v.locale, 0.87, Tone::Info),
            ),
            (
                v.tr(Msg::ScorePossible),
                score_bar(&v.locale, 0.64, Tone::Warn),
            ),
        ]),
    )
}
