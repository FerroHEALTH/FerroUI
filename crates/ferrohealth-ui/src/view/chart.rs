// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts, drawn as SVG on the server: one or more lines over a time or
//! an order axis, and the small activity line of a dashboard.
//!
//! The drawing measures nothing in the browser, so the server pass and
//! hydration draw the same: the SVG has a fixed `viewBox` and the
//! stylesheet scales it to its card's width. The lines take the chart
//! palette's roles, each 3:1 or more against the raised ground in both
//! themes, and a series' colour is a class per palette index, never a
//! `style`. Every chart carries its name and a summary as its `title` and
//! `desc`, and the line chart its figures as a table for assistive
//! technology, so no figure is shown in the picture alone (WCAG 2.2 SC
//! 1.1.1). The interactive legend is [`crate::widget::chart`]. No
//! specification governs the drawing: our own design.

use leptos::prelude::*;

use crate::highlight::escape;

/// The number of colours of the chart palette, `--chart-1` to `--chart-6`.
pub const PALETTE: u8 = 6;

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
    /// The Y range, or `None` to take it from the data.
    pub y_range: Option<(f64, f64)>,
}

/// The drawing's width and height, in the units of its `viewBox`.
const WIDTH: f64 = 640.0;

/// The room left of the plot for the Y labels, and around it.
const LEFT: f64 = 44.0;
const RIGHT: f64 = 12.0;
const TOP: f64 = 12.0;
const BOTTOM: f64 = 28.0;

/// A linear map from a data range onto a drawing range.
#[derive(Debug, Clone, Copy)]
struct Scale {
    from: (f64, f64),
    to: (f64, f64),
}

impl Scale {
    fn at(self, value: f64) -> f64 {
        let span = self.from.1 - self.from.0;
        let share = if span.abs() < f64::EPSILON {
            0.5
        } else {
            (value - self.from.0) / span
        };
        self.to.0 + share * (self.to.1 - self.to.0)
    }
}

/// Appends the formatted `args` to `out`.
fn put(out: &mut String, args: std::fmt::Arguments<'_>) {
    use std::fmt::Write as _;
    // NOTE: the Rust standard library, `impl Write for String`: writing to a
    // String never fails, so there is no error to carry.
    if out.write_fmt(args).is_err() {
        out.clear();
    }
}

/// The smallest and the largest finite value of `values`, or `None`.
fn extent(values: impl Iterator<Item = f64>) -> Option<(f64, f64)> {
    values
        .filter(|value| value.is_finite())
        .fold(None, |range, value| match range {
            None => Some((value, value)),
            Some((low, high)) => Some((low.min(value), high.max(value))),
        })
}

/// `value` as a tick label: whole numbers without a fraction, others with
/// one decimal.
#[must_use]
pub fn tick_label(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}

/// The SVG path of `points` under the two scales, a gap where `y` is not
/// finite.
fn path(points: &[(f64, f64)], x: Scale, y: Scale) -> String {
    let mut out = String::new();
    let mut pen_down = false;
    for (px, py) in points {
        if !(px.is_finite() && py.is_finite()) {
            pen_down = false;
            continue;
        }
        let command = if pen_down { 'L' } else { 'M' };
        put(
            &mut out,
            format_args!("{command}{:.1} {:.1} ", x.at(*px), y.at(*py)),
        );
        pen_down = true;
    }
    out.trim_end().to_owned()
}

/// The parts every chart's SVG shares: its opening tag with title and
/// description, the grid lines and labels of four Y ticks, and the X ticks.
fn frame(
    id: &str,
    label: &str,
    summary: &str,
    height: f64,
    x: Scale,
    y: Scale,
    x_ticks: &[(f64, String)],
) -> String {
    let mut svg = String::new();
    put(
        &mut svg,
        format_args!(
            "<svg class=\"fui-chart-svg\" viewBox=\"0 0 {WIDTH:.0} {height:.0}\" role=\"img\" aria-labelledby=\"{id}-title {id}-desc\"><title id=\"{id}-title\">"
        ),
    );
    escape(label, &mut svg);
    put(&mut svg, format_args!("</title><desc id=\"{id}-desc\">"));
    escape(summary, &mut svg);
    svg.push_str("</desc>");
    let (low, high) = y.from;
    for step in 0..=3_u8 {
        let value = low + (high - low) * f64::from(step) / 3.0;
        let at = y.at(value);
        put(
            &mut svg,
            format_args!(
                "<line class=\"fui-chart-grid\" x1=\"{LEFT:.0}\" x2=\"{:.0}\" y1=\"{at:.1}\" y2=\"{at:.1}\"/>",
                WIDTH - RIGHT
            ),
        );
        put(
            &mut svg,
            format_args!(
                "<text class=\"fui-chart-label\" x=\"{:.0}\" y=\"{:.1}\" text-anchor=\"end\">",
                LEFT - 6.0,
                at + 4.0
            ),
        );
        escape(&tick_label(value), &mut svg);
        svg.push_str("</text>");
    }
    put(
        &mut svg,
        format_args!(
            "<line class=\"fui-chart-axis\" x1=\"{LEFT:.0}\" x2=\"{:.0}\" y1=\"{:.1}\" y2=\"{:.1}\"/>",
            WIDTH - RIGHT,
            height - BOTTOM,
            height - BOTTOM
        ),
    );
    for (value, text) in x_ticks {
        let at = x.at(*value);
        let anchor = if at <= LEFT + 1.0 {
            "start"
        } else if at >= WIDTH - RIGHT - 1.0 {
            "end"
        } else {
            "middle"
        };
        put(
            &mut svg,
            format_args!(
                "<text class=\"fui-chart-label\" x=\"{at:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\">",
                height - 8.0
            ),
        );
        escape(text, &mut svg);
        svg.push_str("</text>");
    }
    svg
}

/// The scales of `chart` within a drawing `height` high.
fn scales(chart: &LineChart, height: f64) -> (Scale, Scale) {
    let xs = extent(
        chart
            .series
            .iter()
            .flat_map(|series| series.points.iter().map(|point| point.0))
            .chain(chart.x_ticks.iter().map(|tick| tick.0)),
    )
    .unwrap_or((0.0, 1.0));
    let ys = chart
        .y_range
        .or_else(|| {
            extent(
                chart
                    .series
                    .iter()
                    .flat_map(|series| series.points.iter().map(|point| point.1)),
            )
        })
        .map_or((0.0, 1.0), |(low, high)| (low.min(0.0), high));
    let ys = if (ys.1 - ys.0).abs() < f64::EPSILON {
        (ys.0, ys.0 + 1.0)
    } else {
        ys
    };
    (
        Scale {
            from: xs,
            to: (LEFT, WIDTH - RIGHT),
        },
        Scale {
            from: ys,
            to: (height - BOTTOM, TOP),
        },
    )
}

/// The SVG of `chart`, drawing only the series `shown` keeps.
#[must_use]
pub fn line_svg(chart: &LineChart, shown: &[bool]) -> String {
    const HEIGHT: f64 = 240.0;
    let (x, y) = scales(chart, HEIGHT);
    let mut svg = frame(
        chart.id,
        &chart.label,
        &chart.summary,
        HEIGHT,
        x,
        y,
        &chart.x_ticks,
    );
    for (index, series) in chart.series.iter().enumerate() {
        if !shown.get(index).copied().unwrap_or(true) {
            continue;
        }
        put(
            &mut svg,
            format_args!(
                "<path class=\"fui-chart-line fui-series-{}\" d=\"{}\"/>",
                series.palette % PALETTE,
                path(&series.points, x, y)
            ),
        );
    }
    svg.push_str("</svg>");
    svg
}

/// The figures of `chart` as a table for assistive technology: a row per X
/// value and a column per series.
#[must_use]
pub fn figures(chart: &LineChart) -> impl IntoView + use<> {
    let mut xs: Vec<f64> = chart
        .series
        .iter()
        .flat_map(|series| series.points.iter().map(|point| point.0))
        .filter(|value| value.is_finite())
        .collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    let heads = chart
        .series
        .iter()
        .map(|series| view! { <th scope="col">{series.name.clone()}</th> })
        .collect_view();
    let rows = xs
        .into_iter()
        .map(|at| {
            let label = chart
                .x_ticks
                .iter()
                .find(|tick| (tick.0 - at).abs() < 1e-9)
                .map_or_else(|| tick_label(at), |tick| tick.1.clone());
            let cells = chart
                .series
                .iter()
                .map(|series| {
                    let value = series
                        .points
                        .iter()
                        .find(|point| (point.0 - at).abs() < 1e-9 && point.1.is_finite())
                        .map_or_else(|| String::from("no value"), |point| tick_label(point.1));
                    view! { <td>{value}</td> }
                })
                .collect_view();
            view! {
                <tr>
                    <th scope="row">{label}</th>
                    {cells}
                </tr>
            }
        })
        .collect_view();
    let caption = format!("{}, as a table", chart.label);
    view! {
        <table class="fui-sr-only">
            <caption>{caption}</caption>
            <thead>
                <tr>
                    <th scope="col">"X"</th>
                    {heads}
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>
    }
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
    /// The label of the first and of the last day.
    pub ends: (String, String),
}

/// Returns the activity chart `chart`: one line in the accent over a fixed
/// height, the first and last day labelled.
#[must_use]
pub fn activity_chart(chart: &ActivityChart) -> impl IntoView + use<> {
    const HEIGHT: f64 = 132.0;
    let last = chart.points.iter().map(|point| point.0).max().unwrap_or(1);
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
        x_ticks: vec![
            (0.0, chart.ends.0.clone()),
            (f64::from(last.max(1)), chart.ends.1.clone()),
        ],
        y_range: None,
    };
    let (x, y) = scales(&line, HEIGHT);
    let mut svg = frame(
        chart.name,
        &line.label,
        &line.summary,
        HEIGHT,
        x,
        y,
        &line.x_ticks,
    );
    if let Some(series) = line.series.first() {
        put(
            &mut svg,
            format_args!(
                "<path class=\"fui-chart-line fui-chart-accent\" d=\"{}\"/>",
                path(&series.points, x, y)
            ),
        );
    }
    svg.push_str("</svg>");
    view! { <figure class="fui-chart fui-chart-activity" data-activity-chart=chart.name inner_html=svg></figure> }
}

/// Returns a thin bar whose length is `percent` (clamped to 100) of its
/// width, such as a latency's share of the slowest beside it.
///
/// The length is an SVG attribute, so no style attribute is written. With
/// no `label` the bar repeats a figure shown beside it and is hidden from
/// assistive technology; with one it stands alone as an image of that name.
/// It never carries the number by itself (WCAG 2.2 SC 1.4.1).
#[must_use]
pub fn proportion(percent: u8, label: Option<String>) -> AnyView {
    let width = percent.min(100).to_string();
    let hidden = label.is_none().then_some("true");
    let role = label.is_some().then_some("img");
    view! {
        <svg
            class="fui-proportion"
            viewBox="0 0 100 6"
            preserveAspectRatio="none"
            focusable="false"
            role=role
            aria-label=label
            aria-hidden=hidden
        >
            <rect class="fui-proportion-ground" width="100" height="6" rx="3"></rect>
            <rect class="fui-proportion-fill" width=width height="6" rx="3"></rect>
        </svg>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::{LineChart, Series, line_svg, tick_label};

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
        assert_eq!("3", tick_label(3.0));
        assert_eq!("2.5", tick_label(2.5));
    }
}
