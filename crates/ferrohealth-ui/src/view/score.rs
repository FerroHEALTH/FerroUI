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

use std::cmp::Ordering;

use leptos::prelude::*;

use crate::i18n::{Text, Word, tr, tr_with};
use crate::tone::Tone;
use crate::view::chart::{nice_step, proportion};
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
    /// The score the total maps to, usually between 0 and 1; the scale
    /// widens to show a score outside its thresholds, and a score that is
    /// not a number reads "no value" with no point drawn.
    pub value: f64,
    /// The grade the score falls in, such as "Probable".
    pub grade: String,
    /// The contributions, in the order they are drawn.
    pub contributions: Vec<Contribution>,
    /// The thresholds, lowest first, as many as the matcher has: the band
    /// of the first takes the possible tone, the band of the last the
    /// certain tone, and every band between them the probable tone.
    pub thresholds: Vec<Threshold>,
}

impl Score {
    /// The tone of the score's grade: the band of the highest threshold it
    /// reaches, or neutral under the first.
    #[must_use]
    pub fn tone(&self) -> Tone {
        match self
            .reached()
            .map(|index| band(index, self.thresholds.len()))
        {
            None => Tone::Neutral,
            Some(Band::Possible) => Tone::Warn,
            Some(Band::Probable) => Tone::Info,
            Some(Band::Certain) => Tone::Ok,
        }
    }

    /// The index of the highest threshold the score reaches.
    fn reached(&self) -> Option<usize> {
        self.thresholds
            .iter()
            .rposition(|threshold| self.value >= threshold.at)
    }
}

/// The grade band a threshold starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Band {
    /// The first band: the possible tone.
    Possible,
    /// A band between the first and the last: the probable tone.
    Probable,
    /// The last band: the certain tone.
    Certain,
}

/// The band the threshold at `index` of `count` starts: the last is
/// certain, the first possible, and every one between them probable.
fn band(index: usize, count: usize) -> Band {
    if index + 1 >= count {
        Band::Certain
    } else if index == 0 {
        Band::Possible
    } else {
        Band::Probable
    }
}

impl Band {
    /// The class of the band on the scale.
    const fn class(self) -> &'static str {
        match self {
            Self::Possible => "fui-score-scale-band-possible",
            Self::Probable => "fui-score-scale-band-probable",
            Self::Certain => "fui-score-scale-band-certain",
        }
    }
}

/// The width of one character of a scale label, in rem: the micro step of
/// the type scale (0.75 rem) at 0.6 em, wide for the system face.
const CHAR_REM: f64 = 0.45;

/// The share of its box the scale's track takes, as the inner drawing's
/// `width` says.
const TRACK: f64 = 0.84;

/// The box widths, in rem, the scale's label rows are worked out for: a
/// phone's card, and from 40 rem, where the stylesheet moves each label
/// to the row the wider track gives it.
const NARROW_REM: f64 = 20.0;
const WIDE_REM: f64 = 40.0;

/// The most rows the scale's labels take, and the height of one.
const ROWS: usize = 4;
const ROW_PX: f64 = 14.0;

/// The classes of a scale whose labels take `narrow` rows on a phone and
/// `wide` rows from 40 rem.
fn rows_class(narrow: usize, wide: usize) -> String {
    let narrow = match narrow {
        0 | 1 => "",
        2 => " fui-score-rows-2",
        3 => " fui-score-rows-3",
        _ => " fui-score-rows-4",
    };
    let wide = match wide {
        0 | 1 => "fui-score-wide-1",
        2 => "fui-score-wide-2",
        3 => "fui-score-wide-3",
        _ => "fui-score-wide-4",
    };
    format!("fui-score-scale{narrow} {wide}")
}

/// The class that moves a label from its row on a phone, `row`, to its
/// row from 40 rem, `wide`.
fn shift_class(row: usize, wide: usize) -> &'static str {
    match (row.cmp(&wide), row.abs_diff(wide)) {
        (Ordering::Equal, _) => "fui-score-mark-label",
        (Ordering::Greater, 1) => "fui-score-mark-label fui-score-up-1",
        (Ordering::Greater, 2) => "fui-score-mark-label fui-score-up-2",
        (Ordering::Greater, _) => "fui-score-mark-label fui-score-up-3",
        (Ordering::Less, 1) => "fui-score-mark-label fui-score-down-1",
        (Ordering::Less, 2) => "fui-score-mark-label fui-score-down-2",
        (Ordering::Less, _) => "fui-score-mark-label fui-score-down-3",
    }
}

/// The row of each label on a track `width` rem wide, the labels given as
/// their share of the track and their length in rem, in order along it:
/// each takes the first row where it clears the label before it.
fn label_rows(labels: &[(f64, f64)], width: f64) -> Vec<usize> {
    let track = width * TRACK;
    let mut ends: Vec<f64> = Vec::new();
    labels
        .iter()
        .map(|&(share, length)| {
            let start = share * track - length / 2.0;
            let row = ends
                .iter()
                .position(|end| start >= end + 0.5)
                .unwrap_or(ends.len())
                .min(ROWS - 1);
            match ends.get_mut(row) {
                Some(end) => *end = end.max(start + length),
                None => ends.push(start + length),
            }
            row
        })
        .collect()
}

/// The range the scale draws: from a round step under the lowest
/// threshold or score to 1, or to a round step over the highest when
/// something lies past 1.
fn domain(score: &Score) -> (f64, f64) {
    let values: Vec<f64> = score
        .thresholds
        .iter()
        .map(|threshold| threshold.at)
        .chain([score.value])
        .filter(|value| value.is_finite())
        .collect();
    let Some((least, most)) = values
        .iter()
        .fold(None, |range: Option<(f64, f64)>, &value| match range {
            None => Some((value, value)),
            Some((low, high)) => Some((low.min(value), high.max(value))),
        })
    else {
        return (0.0, 1.0);
    };
    let step = nice_step(((most.max(1.0) - least) / 4.0).max(0.01));
    let high = if most <= 1.0 {
        1.0
    } else {
        (most / step - 1e-9).ceil() * step
    };
    let low = (least / step + 1e-9).floor() * step - step;
    let low = if least >= 0.0 { low.max(0.0) } else { low };
    (low, high)
}

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

/// A score as the breakdown prints it, with two decimals, or "no value".
fn figure(score: f64) -> String {
    if score.is_finite() {
        format!("{score:.2}")
    } else {
        tr(Text::ChartNoValueCell).into()
    }
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
fn scale_name(score: &Score) -> Word {
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
        .map(String::from)
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
        .map(String::from)
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
    let (low, high) = domain(score);
    let share = move |value: f64| ((value - low) / (high - low)).clamp(0.0, 1.0);
    let at = move |value: f64| share(value) * 100.0;
    let count = score.thresholds.len();
    let starts: Vec<f64> = score
        .thresholds
        .iter()
        .map(|threshold| threshold.at)
        .collect();
    let bands = starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let end = starts.get(index + 1).copied().unwrap_or(high);
            let x = format!("{:.1}%", at(*start));
            let width = format!("{:.1}%", (at(end) - at(*start)).max(0.0));
            let class = band(index, count).class();
            view! { <rect class=class x=x y="8" width=width height="10"></rect> }
        })
        .collect_view();
    // The start's label and each threshold's, along the track, with the
    // rows they take on a phone and from 40 rem.
    let mut labels: Vec<(f64, String)> = vec![(low, figure(low))];
    labels.extend(score.thresholds.iter().map(|threshold| {
        (
            threshold.at,
            format!("{} {}", threshold.label, figure(threshold.at)),
        )
    }));
    labels.sort_by(|a, b| a.0.total_cmp(&b.0));
    let placed: Vec<(f64, f64)> = labels
        .iter()
        .map(|(value, words)| {
            let length = u32::try_from(words.chars().count()).unwrap_or(u32::MAX);
            (share(*value), f64::from(length) * CHAR_REM)
        })
        .collect();
    let narrow = label_rows(&placed, NARROW_REM);
    let wide = label_rows(&placed, WIDE_REM);
    let rows = |of: &[usize]| of.iter().copied().max().unwrap_or(0) + 1;
    let class = rows_class(rows(&narrow), rows(&wide));
    let height = format!(
        "{:.0}",
        30.0 + ROW_PX * f64::from(u8::try_from(rows(&narrow)).unwrap_or(1))
    );
    let marks = score
        .thresholds
        .iter()
        .map(|threshold| {
            let x = format!("{:.1}%", at(threshold.at));
            view! { <line class="fui-score-mark" x1=x.clone() x2=x y1="4" y2="24"></line> }
        })
        .collect_view();
    let words = labels
        .into_iter()
        .zip(narrow.into_iter().zip(wide))
        .map(|((value, words), (row, wide))| {
            let x = format!("{:.1}%", at(value));
            let y = format!(
                "{:.0}",
                38.0 + ROW_PX * f64::from(u8::try_from(row).unwrap_or(0))
            );
            let class = shift_class(row, wide);
            view! {
                <text class=class x=x y=y text-anchor="middle">
                    {words}
                </text>
            }
        })
        .collect_view();
    let point = score.value.is_finite().then(|| {
        let point = format!("{:.1}%", at(score.value));
        view! { <circle class="fui-score-point" cx=point cy="13" r="6"></circle> }
    });
    let inner = height.clone();
    view! {
        <div class="fui-score-scale-box">
            <svg class=class width="100%" height=height role="img" aria-label=scale_name(score)>
                <svg x="8%" width="84%" height=inner overflow="visible">
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
                    {words}
                    {point}
                </svg>
            </svg>
        </div>
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
    use super::{NARROW_REM, Score, Threshold, WIDE_REM, domain, label_rows, signed};
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

    fn graded(value: f64, at: &[f64]) -> Score {
        Score {
            total: 0.0,
            value,
            grade: String::new(),
            contributions: Vec::new(),
            thresholds: at
                .iter()
                .map(|at| Threshold {
                    label: String::new(),
                    at: *at,
                })
                .collect(),
        }
    }

    #[test]
    fn four_thresholds_take_four_bands_the_last_certain() {
        let at = [0.80, 0.83, 0.86, 0.99];
        assert_eq!(Tone::Warn, graded(0.81, &at).tone(), "the first band");
        assert_eq!(Tone::Info, graded(0.84, &at).tone(), "a middle band");
        assert_eq!(Tone::Info, graded(0.90, &at).tone(), "a middle band");
        assert_eq!(Tone::Ok, graded(0.995, &at).tone(), "the last band");
        assert_eq!(
            Tone::Ok,
            graded(0.9, &[0.5]).tone(),
            "one threshold is certain"
        );
    }

    #[test]
    fn the_scale_spans_the_thresholds_and_a_score_outside_them() {
        let close = domain(&graded(0.84, &[0.80, 0.83, 0.86, 0.99]));
        assert!(
            (close.0 - 0.75).abs() < 1e-9 && (close.1 - 1.0).abs() < 1e-9,
            "{close:?}"
        );
        let past = domain(&graded(1.3, &[0.6, 0.8]));
        assert!(past.1 >= 1.3, "a score past 1 is on the scale: {past:?}");
        let under = domain(&graded(-0.2, &[0.6, 0.8]));
        assert!(
            under.0 <= -0.2,
            "a score under 0 is on the scale: {under:?}"
        );
        let nothing = domain(&graded(f64::NAN, &[]));
        assert!(
            (nothing.1 - 1.0).abs() < 1e-9 && nothing.0.abs() < 1e-9,
            "{nothing:?}"
        );
    }

    #[test]
    fn close_labels_take_rows_on_a_phone_and_fewer_when_wide() {
        // The start and four thresholds at 0.75 to 1: 0.80, 0.83, 0.86, 0.99.
        let labels = [
            (0.0, 1.8),
            (0.2, 5.85),
            (0.32, 5.85),
            (0.44, 5.4),
            (0.96, 4.5),
        ];
        let narrow = label_rows(&labels, NARROW_REM);
        assert_eq!(
            vec![0, 1, 0, 2, 0],
            narrow,
            "a label clear of the one before shares its row"
        );
        for (width, rows) in [
            (NARROW_REM, &narrow),
            (WIDE_REM, &label_rows(&labels, WIDE_REM)),
        ] {
            let track = width * 0.84;
            for (index, (share, length)) in labels.iter().enumerate() {
                for (other, (next, next_length)) in labels.iter().enumerate().skip(index + 1) {
                    let apart =
                        (next * track - next_length / 2.0) >= (share * track + length / 2.0);
                    assert!(
                        rows.get(index) != rows.get(other) || apart,
                        "labels {index} and {other} overlap at {width} rem: {rows:?}"
                    );
                }
            }
        }
        let wide = label_rows(&labels, WIDE_REM);
        assert!(
            wide.iter().max() < narrow.iter().max(),
            "a wide track needs fewer rows: {wide:?}"
        );
    }
}
