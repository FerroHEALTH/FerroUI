// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The score breakdown: why a match scored what it did, one signed
//! contribution per compared field, the total, and the thresholds on one
//! scale with the score as a point (R16).
//!
//! Each bar is SVG drawn here, its length an attribute, so no style
//! attribute is written. A bar repeats the weight printed beside it in
//! words and figures, and the tones are the role tones, so nothing rests on
//! colour alone (WCAG 2.2 SC 1.4.1). The scale is an image whose name reads
//! the score, the grade, the thresholds and every contribution (WCAG 2.2
//! SC 1.1.1). No specification governs the drawing: our own design.

use leptos::prelude::*;

use crate::i18n::{Text, tr, tr_with};
use crate::tone::Tone;
use crate::view::chart::proportion;
use crate::view::status::pill;

/// One compared field's part of a score.
#[derive(Debug, Clone, PartialEq)]
pub struct Contribution {
    /// The plain words first, such as "Family name: close match".
    pub label: String,
    /// The comparator and its reading, such as "Jaro-Winkler 0.94", shown
    /// as a quiet hint after the words.
    pub reading: Option<String>,
    /// The weight it added (positive) or removed (negative).
    pub weight: f64,
}

/// One threshold on the scale.
#[derive(Debug, Clone, PartialEq)]
pub struct Threshold {
    /// The grade it starts, such as "Possible".
    pub label: String,
    /// The score it starts at, between 0 and 1.
    pub at: f64,
}

/// What a score breakdown shows.
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    /// The sum of the weights.
    pub total: f64,
    /// The score the total maps to, between 0 and 1.
    pub value: f64,
    /// The grade the score falls in, such as "Probable".
    pub grade: String,
    /// The contributions, in the order they are drawn.
    pub contributions: Vec<Contribution>,
    /// The thresholds, lowest first: up to three, which take the possible,
    /// the probable and the certain band of the scale.
    pub thresholds: Vec<Threshold>,
}

impl Score {
    /// The tone of the score's grade: the band of the highest threshold it
    /// reaches, or neutral under the first.
    #[must_use]
    pub fn tone(&self) -> Tone {
        match self.reached() {
            None => Tone::Neutral,
            Some(0) => Tone::Warn,
            Some(1) => Tone::Info,
            Some(_) => Tone::Ok,
        }
    }

    /// The index of the highest threshold the score reaches.
    fn reached(&self) -> Option<usize> {
        self.thresholds
            .iter()
            .rposition(|threshold| self.value >= threshold.at)
    }
}

/// The band classes of the scale, after the one under the first threshold.
const BANDS: [&str; 3] = [
    "fui-score-scale-band-possible",
    "fui-score-scale-band-probable",
    "fui-score-scale-band-certain",
];

/// The share of a bar's width left of its zero line.
const ZERO: f64 = 40.0;

/// `weight` as the breakdown prints it: a sign and one decimal, a true
/// minus for a negative weight, and a bare zero.
fn signed(weight: f64) -> String {
    if weight > 0.0 {
        format!("+{weight:.1}")
    } else if weight < 0.0 {
        format!("\u{2212}{:.1}", weight.abs())
    } else {
        String::from("0")
    }
}

/// A score as the breakdown prints it, with two decimals.
fn figure(score: f64) -> String {
    format!("{score:.2}")
}

/// The bar of one weight on a track whose ends stand for `largest`.
fn weight_bar(weight: f64, largest: f64) -> AnyView {
    let share = (weight.abs() / largest).min(1.0);
    let fill = if weight > 0.0 {
        let width = share * (100.0 - ZERO);
        Some(("fui-score-bar-add", ZERO, width))
    } else if weight < 0.0 {
        let width = share * ZERO;
        Some(("fui-score-bar-remove", ZERO - width, width))
    } else {
        None
    };
    let fill = fill.map(|(class, x, width)| {
        let x = format!("{x:.1}");
        let width = format!("{width:.1}");
        view! { <rect class=class x=x y="1" width=width height="12" rx="2"></rect> }
    });
    let zero = format!("{ZERO:.0}");
    view! {
        <svg
            class="fui-score-bar"
            viewBox="0 0 100 14"
            preserveAspectRatio="none"
            focusable="false"
            aria-hidden="true"
        >
            <rect class="fui-score-bar-ground" x="0" y="1" width="100" height="12" rx="2"></rect>
            {fill}
            <line
                class="fui-score-bar-zero"
                x1=zero.clone()
                x2=zero
                y1="0"
                y2="14"
                vector-effect="non-scaling-stroke"
            ></line>
        </svg>
    }
    .into_any()
}

/// The row of one contribution: the words, the comparator after them, the
/// bar and the weight.
fn row(contribution: &Contribution, largest: f64) -> AnyView {
    let weight = contribution.weight;
    let tone = if weight > 0.0 {
        "fui-score-weight fui-num fui-score-add"
    } else if weight < 0.0 {
        "fui-score-weight fui-num fui-score-remove"
    } else {
        "fui-score-weight fui-num fui-score-zero"
    };
    let reading = contribution.reading.clone().map(|reading| {
        view! {
            " "
            <span class="fui-score-reading">{reading}</span>
        }
    });
    view! {
        <div class="fui-score-row">
            <span class="fui-score-label">{contribution.label.clone()} {reading}</span>
            {weight_bar(weight, largest)}
            <span class=tone>{signed(weight)}</span>
        </div>
    }
    .into_any()
}

/// The accessible name of the scale: the score, the grade, the thresholds
/// and every contribution.
fn scale_name(score: &Score) -> String {
    let thresholds = score
        .thresholds
        .iter()
        .map(|threshold| {
            tr_with(
                Text::ScoreThreshold,
                &[
                    ("title", threshold.label.clone().into()),
                    ("value", figure(threshold.at).into()),
                ],
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let contributions = score
        .contributions
        .iter()
        .map(|contribution| {
            tr_with(
                Text::ScoreContribution,
                &[
                    ("title", contribution.label.clone().into()),
                    ("value", signed(contribution.weight).into()),
                ],
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    tr_with(
        Text::ScoreName,
        &[
            ("value", figure(score.value).into()),
            ("status", score.grade.clone().into()),
            ("range", thresholds.into()),
            ("summary", contributions.into()),
        ],
    )
}

/// The scale of `score`: the bands between the thresholds, each threshold's
/// label centred on its mark, the start labelled, and the score as the
/// point.
fn scale(score: &Score) -> AnyView {
    let floor = score
        .thresholds
        .iter()
        .map(|threshold| threshold.at)
        .chain([score.value])
        .fold(f64::INFINITY, f64::min);
    let low = if floor >= 0.5 { 0.5 } else { 0.0 };
    let at = move |value: f64| ((value - low) / (1.0 - low)).clamp(0.0, 1.0) * 100.0;
    let starts: Vec<f64> = score
        .thresholds
        .iter()
        .map(|threshold| threshold.at)
        .collect();
    let bands = starts
        .iter()
        .zip(BANDS)
        .enumerate()
        .map(|(index, (start, class))| {
            let end = starts.get(index + 1).copied().unwrap_or(1.0);
            let x = format!("{:.1}%", at(*start));
            let width = format!("{:.1}%", (at(end) - at(*start)).max(0.0));
            view! { <rect class=class x=x y="8" width=width height="10"></rect> }
        })
        .collect_view();
    let marks = score
        .thresholds
        .iter()
        .map(|threshold| {
            let x = format!("{:.1}%", at(threshold.at));
            let words = format!("{} {}", threshold.label, figure(threshold.at));
            view! {
                <line class="fui-score-mark" x1=x.clone() x2=x.clone() y1="4" y2="24"></line>
                <text class="fui-score-mark-label" x=x y="38" text-anchor="middle">
                    {words}
                </text>
            }
        })
        .collect_view();
    let point = format!("{:.1}%", at(score.value));
    view! {
        <svg
            class="fui-score-scale"
            width="100%"
            height="44"
            role="img"
            aria-label=scale_name(score)
        >
            <svg x="8%" width="84%" height="44" overflow="visible">
                <rect
                    class="fui-score-scale-band-none"
                    x="0"
                    y="8"
                    width="100%"
                    height="10"
                    rx="3"
                ></rect>
                {bands}
                {marks}
                <text class="fui-score-mark-label" x="0" y="38" text-anchor="middle">
                    {figure(low)}
                </text>
                <circle class="fui-score-point" cx=point cy="13" r="6"></circle>
            </svg>
        </svg>
    }
    .into_any()
}

/// Returns the breakdown of `score`: one row per contribution, its words
/// first and its comparator after them as a quiet hint, then the total
/// weight, the score with its grade, and the scale.
///
/// The bars share one track, whose ends stand for the largest weight, with
/// the zero line at 40 % of its width. A threshold's label is centred on
/// its mark at every width, and the start of the scale keeps its label on
/// a phone.
#[must_use]
pub fn score_breakdown(score: &Score) -> AnyView {
    let largest = score
        .contributions
        .iter()
        .map(|contribution| contribution.weight.abs())
        .fold(1.0, f64::max);
    let rows = score
        .contributions
        .iter()
        .map(|contribution| row(contribution, largest))
        .collect_view();
    view! {
        <div class="fui-score">{rows}</div>
        <div class="fui-score-total">
            <span>
                {tr(Text::ScoreTotalWeight)} " "
                <span class="fui-muted fui-num">{signed(score.total)}</span>
            </span>
            <span>
                {tr(Text::ScoreWord)} " " <span class="fui-score-figure fui-num">{figure(score.value)}</span>
                " " {pill(score.tone(), &score.grade)}
            </span>
        </div>
        {scale(score)}
    }
    .into_any()
}

/// Returns the compact score for a table cell: a short bar in `tone`, the
/// grade's, beside the score in figures.
///
/// The bar repeats the figure beside it and is hidden from assistive
/// technology.
#[must_use]
pub fn score_bar(score: f64, tone: Tone) -> AnyView {
    let class = format!("fui-score-cell {}", tone.class());
    let share = (score.clamp(0.0, 1.0) * 100.0).round();
    let percent = (0..=100_u8)
        .rev()
        .find(|whole| f64::from(*whole) <= share)
        .unwrap_or(0);
    view! {
        <span class=class>
            {proportion(percent, None)}
            <span class="fui-num">{figure(score)}</span>
        </span>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::{Score, Threshold, signed};
    use crate::tone::Tone;

    #[test]
    fn a_weight_prints_its_sign_and_a_true_minus() {
        assert_eq!("+2.4", signed(2.4));
        assert_eq!("\u{2212}1.1", signed(-1.1));
        assert_eq!("0", signed(0.0));
    }

    #[test]
    fn the_grade_tone_is_the_band_of_the_highest_threshold_reached() {
        let mut score = Score {
            total: 0.0,
            value: 0.80,
            grade: String::new(),
            contributions: Vec::new(),
            thresholds: [0.60, 0.80, 0.95]
                .map(|at| Threshold {
                    label: String::new(),
                    at,
                })
                .to_vec(),
        };
        assert_eq!(
            Tone::Info,
            score.tone(),
            "a score at a threshold reaches it"
        );
        score.value = 0.70;
        assert_eq!(Tone::Warn, score.tone());
        score.value = 0.97;
        assert_eq!(Tone::Ok, score.tone());
        score.value = 0.40;
        assert_eq!(Tone::Neutral, score.tone());
    }
}
