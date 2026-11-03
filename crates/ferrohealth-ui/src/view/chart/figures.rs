// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The line chart's figures: the same values as a table, for a reader who
//! wants the numbers.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};

use super::LineChart;
use super::scale::value_label;

/// The words of a chart's figures table, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FiguresWords {
    /// A cell with no value.
    pub no_value: Word,
    /// The head of the X column.
    pub x: Word,
    /// The caption, around the open `chart`, the chart's name.
    pub caption: Phrase,
}

impl FiguresWords {
    /// The table's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            no_value: tr(locale, Text::ChartNoValueCell),
            x: tr(locale, Text::ChartX),
            caption: Phrase::new(tr_with(
                locale,
                Text::ChartAsTable,
                &[("chart", Phrase::open("chart"))],
            )),
        }
    }
}

/// The figures of `chart` as a table for assistive technology, in `words`:
/// a row per X value and a column per series.
#[must_use]
pub fn figures(chart: &LineChart, words: FiguresWords) -> Html {
    let mut xs: Vec<f64> = chart
        .series
        .iter()
        .flat_map(|series| series.points.iter().map(|point| point.0))
        .filter(|value| value.is_finite())
        .collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    let FiguresWords {
        no_value,
        x,
        caption,
    } = words;
    // Each series and the ticks sorted by X once, so a row finds its cells
    // by a binary search.
    let mut ticks: Vec<&(f64, String)> = chart.x_ticks.iter().collect();
    ticks.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut labels: Vec<&(f64, String)> = chart.x_labels.iter().collect();
    labels.sort_by(|a, b| a.0.total_cmp(&b.0));
    let columns: Vec<Vec<(f64, f64)>> = chart
        .series
        .iter()
        .map(|series| {
            let mut points: Vec<(f64, f64)> = series
                .points
                .iter()
                .copied()
                .filter(|point| point.0.is_finite() && point.1.is_finite())
                .collect();
            points.sort_by(|a, b| a.0.total_cmp(&b.0));
            points
        })
        .collect();
    let rows = xs
        .into_iter()
        .map(|at| {
            let near = |value: f64| (value - at).abs() < 1e-9;
            let on = |marks: &[&(f64, String)]| {
                marks
                    .get(marks.partition_point(|mark| mark.0 < at - 1e-9))
                    .filter(|mark| near(mark.0))
                    .map(|mark| mark.1.clone())
            };
            let label = on(&labels)
                .or_else(|| on(&ticks))
                .unwrap_or_else(|| value_label(at));
            let cells = columns
                .iter()
                .map(|points| {
                    points
                        .get(points.partition_point(|point| point.0 < at - 1e-9))
                        .filter(|point| near(point.0))
                        .map_or_else(|| no_value.clone(), |point| value_label(point.1).into())
                })
                .collect();
            (label, cells)
        })
        .collect();
    // NOTE: CSS 2.1 § 17.5.2: a table ignores its 1 px width, so the box that hides it
    // is a block around it, which keeps it out of the page's scroll width.
    FiguresView {
        caption: caption.fill(&[("chart", &chart.label)]),
        x,
        heads: chart
            .series
            .iter()
            .map(|series| series.name.clone())
            .collect(),
        rows,
    }
    .html()
}

#[derive(Template)]
#[template(path = "chart/chart.html", block = "figures")]
struct FiguresView {
    caption: Word,
    x: Word,
    heads: Vec<String>,
    rows: Vec<(String, Vec<Word>)>,
}
