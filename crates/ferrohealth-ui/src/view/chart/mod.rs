// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts, drawn as SVG on the server: one or more lines over a time or
//! an order axis, and the small activity line of a dashboard.
//!
//! The drawing measures nothing in the browser: the server draws it whole. The outer SVG has a fixed height and its card's
//! width with no `viewBox`, so its labels keep the type scale at every
//! width: the Y labels sit above their grid lines at the left edge and the
//! X labels at percentages of the width. The lines are drawn in an inner SVG
//! stretched to the plot (`preserveAspectRatio="none"`) with strokes that do
//! not scale. The lines take the chart
//! palette's roles, each 3:1 or more against the raised ground in both
//! themes, and a series' colour is a class per palette index, never a
//! `style`. Each series also takes a point mark of its own beside its
//! colour, so no two series differ by colour alone (WCAG 2.2 SC 1.4.1). The X
//! labels the chart's width has no room for are left out by container
//! queries, chosen on the server from each label's length. Every chart carries its name and a summary as its `title` and
//! `desc`, and the line chart its figures as a table for assistive
//! technology, so no figure is shown in the picture alone (WCAG 2.2 SC
//! 1.1.1). The line chart's legend ([`legend::legend`]) is a static key
//! with no script, each series' swatch beside its name; once the `chart` module
//! runs each entry is a `<button aria-pressed>`, and a press sets the
//! `hidden` attribute of that series' groups alone, so the browser parses
//! no SVG again; hiding one series recolours none, since each keeps its
//! palette index, and the last series shown cannot be hidden. No
//! specification governs the drawing: our own design.

pub mod figures;
mod frame;
pub mod legend;
pub mod line;
pub mod scale;

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};

use figures::{FiguresWords, figures};
use frame::{drawn, frame};
use legend::{LegendEntry, legend};
use line::{has_gaps, line_svg_grouped, path, scales};
use scale::{WIDTH, put};

/// The number of colours a series can take: the role tones below, so a
/// colour means what it means elsewhere and follows the brand's accent.
pub const PALETTE: u8 = 6;

/// The palette index of a series in the brand's accent: the main measure.
pub const ACCENT: u8 = 0;

/// The palette index of a series in the information tone.
pub const INFO: u8 = 1;

/// The palette index of a series in the good tone, such as successes.
pub const OK: u8 = 2;

/// The palette index of a series in the warning tone.
pub const WARN: u8 = 3;

/// The palette index of a series in the bad tone, such as refusals.
pub const BAD: u8 = 4;

/// The palette index of a series in the neutral tone: a measure beside the
/// main one.
pub const NEUTRAL: u8 = 5;

/// The number of point marks a series can take.
///
/// The marks are a circle, a square, a diamond and a triangle. A series'
/// palette index picks its colour (`palette % PALETTE`) and its mark
/// (`(palette + palette / PALETTE) % MARKS`), so each of the first 24
/// indices takes its own pair, two series of one colour never share a
/// mark, and neighbours differ by their mark as well as their colour.
pub const MARKS: u8 = 4;

/// The colour and the mark of the series at palette index `palette`.
#[must_use]
pub(crate) const fn series_look(palette: u8) -> (u8, u8) {
    (
        palette % PALETTE,
        (palette % MARKS + palette.div_euclid(PALETTE) % MARKS) % MARKS,
    )
}

/// The SVG path of the point mark `mark`, centred on the origin and about
/// 7 px across: a circle, a square, a diamond or a triangle.
#[must_use]
pub(crate) const fn mark_path(mark: u8) -> &'static str {
    match mark {
        1 => "M-3 -3H3V3H-3Z",
        2 => "M0 -4L4 0L0 4L-4 0Z",
        3 => "M0 -4L3.6 3H-3.6Z",
        _ => "M-3 0A3 3 0 1 0 3 0A3 3 0 1 0 -3 0Z",
    }
}

/// The most points a series marks; a longer series marks every few, evenly,
/// so its mark still names it and the page stays small.
const MARKED: usize = 60;

/// The columns a long series' line is thinned to, each keeping its lowest
/// and its highest point, so no peak is lost.
const COLUMNS: u16 = 500;

/// One line of a chart.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    /// Its name, as the legend and the table show it.
    pub name: String,
    /// Its points, `(x, y)`; a `y` that is NaN is a gap in the line.
    pub points: Vec<(f64, f64)>,
    /// The palette index that pins its colour, so hiding another series
    /// recolours none.
    pub palette: u8,
}

/// What a line chart shows.
#[derive(Debug, Clone, PartialEq)]
pub struct LineChart {
    /// The chart's id; its title and description take it with `-title` and
    /// `-desc`.
    pub id: &'static str,
    /// Its accessible name.
    pub label: String,
    /// The sentence that summarises what it shows.
    pub summary: String,
    /// The lines.
    pub series: Vec<Series>,
    /// The ticks of the X axis, each its value and its label.
    pub x_ticks: Vec<(f64, String)>,
    /// The label of each X value in the figures table, each its value and
    /// its label, such as a point's time on a time axis; a value with none
    /// takes the label of a tick on it, then its number. The axis draws
    /// [`LineChart::x_ticks`] alone.
    pub x_labels: Vec<(f64, String)>,
    /// The Y range, or `None` to take it from the data.
    pub y_range: Option<(f64, f64)>,
}

/// What an activity chart shows: events per day.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivityChart {
    /// The chart's id and its `data-activity-chart` hook.
    pub name: &'static str,
    /// Its accessible name, such as "Commits per day".
    pub label: String,
    /// The sentence that summarises it, such as "12 commits over 30 days,
    /// most on 2026-10-03".
    pub summary: String,
    /// The counts, `(day index, count)`.
    pub points: Vec<(u32, u32)>,
    /// The days the axis names, `(day index, words)`, such as the first
    /// day, every tenth and the last.
    pub ticks: Vec<(u32, String)>,
}

/// Returns the activity chart `chart`: one line in the accent over a fixed
/// height, the days its ticks name labelled.
#[must_use]
pub fn activity_chart(chart: &ActivityChart) -> Html {
    const HEIGHT: f64 = 132.0;
    let line = LineChart {
        id: chart.name,
        label: chart.label.clone(),
        summary: chart.summary.clone(),
        series: vec![Series {
            name: chart.label.clone(),
            points: chart
                .points
                .iter()
                .map(|(day, count)| (f64::from(*day), f64::from(*count)))
                .collect(),
            palette: 0,
        }],
        x_ticks: chart
            .ticks
            .iter()
            .map(|(day, words)| (f64::from(*day), words.clone()))
            .collect(),
        x_labels: Vec::new(),
        y_range: None,
    };
    let (x, y) = scales(&line, HEIGHT);
    let frame = frame(HEIGHT, x, y, &line.x_ticks);
    let lines = line.series.first().map_or_else(String::new, |series| {
        format!(
            "<path class=\"fui-chart-line fui-chart-accent\" d=\"{}\"/>",
            path(&series.points, x, y)
        )
    });
    // One day draws no line, so its point is marked instead.
    let mut marks = String::new();
    if let Some(&(px, py)) = line
        .series
        .first()
        .filter(|series| series.points.len() == 1)
        .and_then(|series| series.points.first())
    {
        put(
            &mut marks,
            format_args!(
                "<circle class=\"fui-chart-point fui-chart-accent\" cx=\"{:.2}%\" cy=\"{:.1}\" r=\"3\"/>",
                x.at(px) / WIDTH * 100.0,
                y.at(py)
            ),
        );
    }
    let svg = drawn(
        chart.name,
        &line.label,
        &line.summary,
        &frame,
        &lines,
        &marks,
    );
    Html::raw(format!(
        "<figure data-activity-chart=\"{}\" class=\"fui-chart fui-chart-activity\">{svg}</figure>",
        Html::text(chart.name)
    ))
}

/// Returns a thin bar whose length is `percent` (clamped to 100) of its
/// width, such as a latency's share of the slowest beside it.
///
/// The length is an SVG attribute, so no style attribute is written. With
/// no `label` the bar repeats a figure shown beside it and is hidden from
/// assistive technology; with one it stands alone as an image of that name.
/// It never carries the number by itself (WCAG 2.2 SC 1.4.1).
#[must_use]
pub fn proportion(percent: u8, label: Option<String>) -> Html {
    let width = percent.min(100);
    let named = label.map_or_else(
        || String::from(" aria-hidden=\"true\""),
        |label| format!(" role=\"img\" aria-label=\"{}\"", Html::text(label)),
    );
    Html::raw(format!(
        "<svg viewBox=\"0 0 100 6\" preserveAspectRatio=\"none\" focusable=\"false\"{named} class=\"fui-proportion\"><rect width=\"100\" height=\"6\" rx=\"3\" class=\"fui-proportion-ground\"></rect><rect width=\"{width}\" height=\"6\" rx=\"3\" class=\"fui-proportion-fill\"></rect></svg>"
    ))
}

/// The words a line chart draws itself, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartWords {
    /// The legend's note on a gap in a line.
    pub no_value: Word,
    /// The legend's name.
    pub series: Word,
    /// The figures table's words.
    pub figures: FiguresWords,
}

impl ChartWords {
    /// The chart's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            no_value: tr(locale, Text::ChartNoValue),
            series: tr(locale, Text::ChartSeries),
            figures: FiguresWords::of(locale),
        }
    }
}

/// The id of the plot of the chart `id`, which the legend shows and hides
/// series in.
fn plot_id(id: &str) -> String {
    format!("{id}-plot")
}

#[derive(Template)]
#[template(path = "chart/chart.html", block = "line_chart")]
struct LineChartView {
    plot: String,
    drawn: Html,
    legend: Html,
    table: Html,
}

/// Returns the line chart `chart` in `words`: its drawing, its legend and
/// its figures table.
#[must_use]
pub fn line_chart(chart: &LineChart, words: ChartWords) -> Html {
    let ChartWords {
        no_value,
        series: label,
        figures: figure_words,
    } = words;
    let entries = chart
        .series
        .iter()
        .map(|series| LegendEntry {
            name: series.name.clone(),
            palette: series.palette,
        })
        .collect::<Vec<_>>();
    let gap = has_gaps(chart).then_some(no_value);
    let plot = plot_id(chart.id);
    LineChartView {
        legend: legend(&plot, label.into(), entries, gap),
        drawn: Html::raw(line_svg_grouped(chart, &[])),
        table: figures(chart, figure_words),
        plot,
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::frame::{TICK_WIDTHS, label_rem, tick_levels};
    use super::line::{line_svg, line_svg_grouped};
    use super::scale::{Scale, nice_range, tick_label, ticks, value_label};
    use super::{LineChart, Series};

    fn chart() -> LineChart {
        LineChart {
            id: "rates",
            label: String::from("Rates <per day>"),
            summary: String::from("Two synthetic series."),
            series: vec![
                Series {
                    name: String::from("a"),
                    points: vec![(0.0, 1.0), (1.0, f64::NAN), (2.0, 3.0)],
                    palette: 0,
                },
                Series {
                    name: String::from("b"),
                    points: vec![(0.0, 2.0), (2.0, 2.0)],
                    palette: 7,
                },
            ],
            x_ticks: vec![(0.0, String::from("Mon")), (2.0, String::from("Wed"))],
            x_labels: Vec::new(),
            y_range: None,
        }
    }

    #[test]
    fn a_gap_breaks_the_line_and_the_labels_are_escaped() {
        let svg = line_svg(&chart(), &[true, true]);
        assert!(svg.contains("Rates &lt;per day&gt;"), "{svg}");
        let first = svg
            .split("fui-series-0\" d=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("the first line");
        assert_eq!(2, first.matches('M').count(), "{first}");
        assert!(svg.contains("fui-series-1"), "a palette index wraps: {svg}");
        assert!(!svg.contains("style="), "{svg}");
    }

    #[test]
    fn a_hidden_series_is_not_drawn() {
        let svg = line_svg(&chart(), &[true, false]);
        assert!(svg.contains("fui-series-0") && !svg.contains("fui-series-1"));
        assert_eq!("3", tick_label(3.0, 1.0), "a whole step");
        assert_eq!("2.5", tick_label(2.5, 0.5), "a step of a half");
    }

    #[test]
    fn the_grouped_drawing_keeps_a_hidden_series_drawn_and_hidden() {
        let svg = line_svg_grouped(&chart(), &[true, false]);
        assert!(
            svg.contains("fui-series-0") && svg.contains("fui-series-1"),
            "{svg}"
        );
        assert!(svg.contains("<g data-series-lines=\"0\">"), "{svg}");
        assert!(svg.contains("<g data-series-lines=\"1\" hidden>"), "{svg}");
        assert!(svg.contains("<g data-series-marks=\"1\" hidden>"), "{svg}");
        assert!(svg.contains("Rates &lt;per day&gt;"), "{svg}");
    }

    #[test]
    fn the_value_axis_ticks_are_round_steps_over_the_values() {
        let cases = [
            ((0.0, 58.0), (0.0, 60.0), 20.0),
            ((0.0, 39.1), (0.0, 40.0), 10.0),
            ((0.0, 7.0), (0.0, 8.0), 2.0),
            ((0.0, 1.0), (0.0, 1.0), 0.5),
            ((-12.0, 30.0), (-20.0, 30.0), 10.0),
        ];
        for (range, expected, step) in cases {
            assert_eq!((expected, step), nice_range(range), "{range:?}");
        }
        let scale = Scale {
            from: (0.0, 60.0),
            to: (0.0, 1.0),
            step: 20.0,
        };
        assert_eq!(vec![0.0, 20.0, 40.0, 60.0], ticks(scale));
    }

    #[test]
    fn an_activity_chart_of_one_day_marks_its_point() {
        let chart = super::ActivityChart {
            name: "contributions",
            label: String::from("Contributions per day"),
            summary: String::from("5 contributions over 1 day."),
            points: vec![(0, 5)],
            ticks: vec![(0, String::from("2026-10-18"))],
        };
        let html = String::from(super::activity_chart(&chart));
        assert!(html.contains("fui-chart-point fui-chart-accent"), "{html}");
    }

    #[test]
    fn a_tick_label_takes_the_decimals_of_its_step() {
        let scale = Scale {
            from: (0.0, 0.0008),
            to: (0.0, 1.0),
            step: 0.0002,
        };
        let labels: Vec<String> = ticks(scale)
            .into_iter()
            .map(|value| tick_label(value, scale.step))
            .collect();
        assert_eq!(
            vec!["0.0000", "0.0002", "0.0004", "0.0006", "0.0008"],
            labels,
            "values under 0.1 never all read 0.0"
        );
        assert_eq!("0.00002", tick_label(0.00002, 0.00001), "a tiny step");
        assert_eq!("40", tick_label(40.0, 20.0), "a whole step");
        assert_eq!("0.0", tick_label(-0.0, 0.5), "no minus on zero");
    }

    #[test]
    fn a_figure_keeps_six_significant_digits_and_drops_trailing_zeros() {
        assert_eq!("0.00012", value_label(0.00012), "a small value");
        assert_eq!("1.23457", value_label(1.234_567_89), "six digits");
        assert_eq!("120", value_label(120.0), "a whole value");
        assert_eq!("-2.5", value_label(-2.5), "a negative value");
        assert_eq!("0", value_label(-0.0), "no minus on zero");
    }

    /// Twelve date labels a tenth of the axis apart, as the audit drew them.
    fn dates() -> Vec<(f64, f64)> {
        (0..12_u8)
            .map(|day| {
                (
                    f64::from(day) * 10.0 / 119.0,
                    label_rem(&format!("2026-10-{:02}", day + 1)),
                )
            })
            .collect()
    }

    #[test]
    fn the_x_labels_a_width_shows_never_overlap() {
        let labels = dates();
        let gutter = 2.0;
        let levels = tick_levels(&labels, gutter);
        assert_eq!(
            Some(0),
            levels.first().copied().flatten(),
            "the first label always shows"
        );
        for (level, (width, _)) in TICK_WIDTHS.iter().enumerate() {
            let plot = f64::from(*width) - gutter;
            let mut last = f64::NEG_INFINITY;
            for (label, from) in labels.iter().zip(&levels) {
                if from.is_none_or(|from| from > level) {
                    continue;
                }
                let at = label.0 * plot;
                let (start, end) = if label.0 <= 0.01 {
                    (at, at + label.1)
                } else {
                    (at - label.1 / 2.0, at + label.1 / 2.0)
                };
                assert!(start >= last, "labels overlap at {width} rem: {levels:?}");
                last = end;
            }
        }
        assert!(
            levels.iter().filter(|level| **level == Some(0)).count() < labels.len(),
            "a phone shows fewer than twelve dates: {levels:?}"
        );
        assert!(
            2 * levels.iter().filter(|level| level.is_some()).count() >= labels.len(),
            "a wide chart shows every other date at least: {levels:?}"
        );
    }

    #[test]
    fn a_thinned_label_carries_the_class_of_the_width_it_shows_from() {
        let mut chart = chart();
        chart.x_ticks = (0..12_u8)
            .map(|day| (f64::from(day) * 0.2, format!("2026-10-{:02}", day + 1)))
            .collect();
        let svg = line_svg(&chart, &[true, true]);
        assert!(
            TICK_WIDTHS
                .iter()
                .any(|(_, hides)| !hides.is_empty() && svg.contains(hides)),
            "a date a phone has no room for is hidden there: {svg}"
        );
        assert!(
            svg.contains("<text class=\"fui-chart-label\" x=\"0.00%\""),
            "the first date shows at every width: {svg}"
        );
    }

    #[test]
    fn every_series_takes_a_colour_and_mark_pair_of_its_own() {
        let pairs: std::collections::BTreeSet<(u8, u8)> =
            (0..24_u8).map(super::series_look).collect();
        assert_eq!(24, pairs.len(), "24 series, 24 pairs");
        assert_ne!(
            super::series_look(0).1,
            super::series_look(2).1,
            "the accent and the good tone, alike in a green brand, differ by their mark"
        );
    }

    #[test]
    fn a_seventh_series_takes_the_first_colour_with_its_own_mark() {
        let mut chart = chart();
        chart.series = (0..8_u8)
            .map(|palette| Series {
                name: format!("s{palette}"),
                points: vec![(0.0, 1.0), (1.0, 2.0)],
                palette,
            })
            .collect();
        let svg = line_svg(&chart, &[]);
        assert!(svg.contains("<path id=\"rates-mark-3\""), "{svg}");
        assert!(
            svg.contains("<use class=\"fui-chart-point fui-series-0\" href=\"#rates-mark-3\""),
            "the seventh series is marked by triangles: {svg}"
        );
        assert!(
            svg.contains("<circle class=\"fui-chart-point fui-series-0\""),
            "the first series keeps its circles: {svg}"
        );
    }

    #[test]
    fn a_long_series_is_thinned_and_keeps_its_peaks() {
        let mut chart = chart();
        chart.series = vec![Series {
            name: String::from("long"),
            points: (0..5000_u16)
                .map(|at| {
                    (
                        f64::from(at),
                        if at == 2345 {
                            500.0
                        } else {
                            f64::from(at % 97)
                        },
                    )
                })
                .collect(),
            palette: 0,
        }];
        chart.x_ticks = Vec::new();
        let svg = line_svg(&chart, &[]);
        assert!(
            svg.matches("<circle").count() <= 61,
            "the marks are thinned"
        );
        let line = svg
            .split("fui-series-0\" d=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("the line");
        assert!(line.matches('L').count() <= 1000, "the line is thinned");
        // The Y axis runs to 600, so the peak of 500 sits at 44.0 of the 214 to 10 plot.
        assert!(line.contains(" 44.0"), "the peak is kept: {line}");
    }
}
