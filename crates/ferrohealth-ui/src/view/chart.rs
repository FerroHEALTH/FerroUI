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
//! 1.1.1). The line chart's legend ([`legend`]) is a static key with no
//! script, each series' swatch beside its name; once the `chart` module
//! runs each entry is a `<button aria-pressed>`, and a press sets the
//! `hidden` attribute of that series' groups alone, so the browser parses
//! no SVG again; hiding one series recolours none, since each keeps its
//! palette index, and the last series shown cannot be hidden. No
//! specification governs the drawing: our own design.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::highlight::escape;
use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};

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

/// The width of the inner drawing's `viewBox`, which is stretched to the
/// plot's width; the X labels are placed at the same share of the width.
const WIDTH: f64 = 1000.0;

/// The room above the plot, and below it for the X labels, in CSS pixels;
/// the Y gutter's padding in the stylesheet is each less half a line.
const TOP: f64 = 10.0;
const BOTTOM: f64 = 26.0;

/// A linear map from a data range onto a drawing range.
#[derive(Debug, Clone, Copy)]
struct Scale {
    from: (f64, f64),
    to: (f64, f64),
    /// The distance between two ticks, a round number; zero on an axis
    /// whose ticks the chart names itself.
    step: f64,
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

/// The round step (1, 2 or 5 times a power of ten) nearest `raw`.
pub(crate) fn nice_step(raw: f64) -> f64 {
    if !raw.is_finite() || raw <= 0.0 {
        return 1.0;
    }
    let power = 10_f64.powf(raw.log10().floor());
    let fraction = raw / power;
    let nice = if fraction < 1.5 {
        1.0
    } else if fraction < 3.0 {
        2.0
    } else if fraction < 7.0 {
        5.0
    } else {
        10.0
    };
    nice * power
}

/// The range `(low, high)` widened to round ticks three to seven steps apart,
/// and the step.
fn nice_range((low, high): (f64, f64)) -> ((f64, f64), f64) {
    let step = nice_step((high - low) / 3.0);
    let low = (low / step).floor() * step;
    let high = (high / step).ceil() * step;
    let high = if high - low < step { low + step } else { high };
    ((low, high), step)
}

/// The values of the round ticks over `scale`, the lowest first.
fn ticks(scale: Scale) -> Vec<f64> {
    let (low, high) = scale.from;
    if scale.step <= 0.0 {
        return vec![low, high];
    }
    let count = ((high - low) / scale.step).round();
    let count = if (1.0..=20.0).contains(&count) {
        count
    } else {
        1.0
    };
    let mut values = Vec::new();
    let mut index = 0.0;
    while index <= count {
        values.push(low + scale.step * index);
        index += 1.0;
    }
    values
}

/// The decimals a tick on an axis `step` apart needs: the fewest that
/// write the step itself exactly, so no two ticks print alike.
fn decimals(step: f64) -> usize {
    if !(step.is_finite() && step > 0.0) {
        return 0;
    }
    (0..=12_usize)
        .zip(0..=12_i32)
        .find(|(_, power)| {
            let scaled = step * 10_f64.powi(*power);
            (scaled - scaled.round()).abs() < 1e-6 * scaled.max(1.0)
        })
        .map_or(12, |(places, _)| places)
}

/// `value` written with `places` decimals, with no minus on a zero.
fn fixed(value: f64, places: usize) -> String {
    let text = format!("{value:.places$}");
    if text.starts_with('-') && text.trim_start_matches(['-', '0', '.']).is_empty() {
        text.trim_start_matches('-').to_owned()
    } else {
        text
    }
}

/// `value` as the label of a tick on an axis `step` apart: as many
/// decimals as the step has, so ticks 0.00002 apart read 0.00002, 0.00004
/// and so on, and whole steps read without a fraction.
#[must_use]
pub fn tick_label(value: f64, step: f64) -> String {
    fixed(value, decimals(step))
}

/// `value` as the figures table writes it: up to six significant digits,
/// with no trailing zeros, so a small value never reads as zero.
#[must_use]
pub fn value_label(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let places = (0..=12_usize)
        .zip(0..=12_i32)
        .find(|(_, power)| {
            let scaled = (value * 10_f64.powi(*power)).abs();
            scaled >= 1e5 || (scaled - scaled.round()).abs() < 1e-6
        })
        .map_or(12, |(places, _)| places);
    let text = fixed(value, places);
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text
    }
}

/// `points` thinned for drawing: a series longer than twice [`COLUMNS`]
/// keeps, per column of the plot, its lowest and its highest point in
/// their order, and every gap; a shorter one is drawn whole.
fn thinned(points: &[(f64, f64)], x: Scale) -> Vec<(f64, f64)> {
    /// The lowest and the highest point of one column, with their indices.
    type Column = (f64, (usize, (f64, f64)), (usize, (f64, f64)));
    fn flush(column: &mut Option<Column>, out: &mut Vec<(f64, f64)>) {
        if let Some((_, low, high)) = column.take() {
            let (first, second) = if low.0 <= high.0 {
                (low, high)
            } else {
                (high, low)
            };
            out.push(first.1);
            if second.0 != first.0 {
                out.push(second.1);
            }
        }
    }
    if points.len() <= 2 * usize::from(COLUMNS) {
        return points.to_vec();
    }
    let mut out = Vec::new();
    let mut column: Option<Column> = None;
    for (index, &(px, py)) in points.iter().enumerate() {
        if !px.is_finite() {
            continue;
        }
        if !py.is_finite() {
            flush(&mut column, &mut out);
            out.push((px, py));
            continue;
        }
        let at = (x.at(px) / WIDTH * f64::from(COLUMNS)).floor();
        match &mut column {
            Some((id, low, high)) if (*id - at).abs() < f64::EPSILON => {
                if py < low.1.1 {
                    *low = (index, (px, py));
                }
                if py > high.1.1 {
                    *high = (index, (px, py));
                }
            }
            _ => {
                flush(&mut column, &mut out);
                column = Some((at, (index, (px, py)), (index, (px, py))));
            }
        }
    }
    flush(&mut column, &mut out);
    out
}

/// The SVG path of `points` under the two scales, a gap where `y` is not
/// finite.
fn path(points: &[(f64, f64)], x: Scale, y: Scale) -> String {
    let mut out = String::new();
    let mut pen_down = false;
    for (px, py) in &thinned(points, x) {
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

/// The SVG path that bridges each run of missing values in `points`: from
/// the last value before the run to the first after it, drawn dashed.
fn gaps(points: &[(f64, f64)], x: Scale, y: Scale) -> String {
    let mut out = String::new();
    let mut last: Option<(f64, f64)> = None;
    let mut missing = false;
    for &(px, py) in points {
        if !px.is_finite() {
            continue;
        }
        if !py.is_finite() {
            missing = last.is_some();
            continue;
        }
        if missing && let Some((lx, ly)) = last {
            put(
                &mut out,
                format_args!(
                    "M{:.1} {:.1} L{:.1} {:.1} ",
                    x.at(lx),
                    y.at(ly),
                    x.at(px),
                    y.at(py)
                ),
            );
        }
        missing = false;
        last = Some((px, py));
    }
    out.trim_end().to_owned()
}

/// Whether `chart` has a missing value inside a series, which its legend
/// then explains.
#[must_use]
pub fn has_gaps(chart: &LineChart) -> bool {
    chart
        .series
        .iter()
        .any(|series| series.points.iter().any(|point| !point.1.is_finite()))
}

/// The parts every chart's drawing shares that no series changes: the
/// labels and grid lines of the Y ticks and the labels of the X ticks.
#[derive(Debug, Clone)]
struct Frame {
    /// The drawing's height, as its `height` attribute.
    height: String,
    /// The `viewBox` of the stretched plot.
    view_box: String,
    /// The Y labels, a `<span>` each, the highest first.
    y_labels: String,
    /// The grid lines, the axis first.
    grid: String,
    /// The X labels, a `<text>` each.
    x_labels: String,
}

/// The width of one character of an axis label, in rem: the micro step
/// of the type scale (0.75 rem) at 0.6 em, wide for the system face.
const CHAR_REM: f64 = 0.45;

/// The least room between two X labels, in rem.
const LABEL_GAP_REM: f64 = 0.5;

/// The chart widths, in rem, from which more X labels show, each with the
/// class the stylesheet hides a label by below it, in a container query:
/// the first is the narrowest chart a phone draws, whose labels no class
/// hides.
const TICK_WIDTHS: [(u8, &str); 9] = [
    (12, ""),
    (15, "fui-chart-tick-15"),
    (20, "fui-chart-tick-20"),
    (25, "fui-chart-tick-25"),
    (30, "fui-chart-tick-30"),
    (40, "fui-chart-tick-40"),
    (50, "fui-chart-tick-50"),
    (60, "fui-chart-tick-60"),
    (80, "fui-chart-tick-80"),
];

/// The anchor of an X label at `share` of the plot's width.
fn anchor(share: f64) -> &'static str {
    if share <= 0.01 {
        "start"
    } else if share >= 0.99 {
        "end"
    } else {
        "middle"
    }
}

/// The length of `text` in rem, as an axis label draws it.
fn label_rem(text: &str) -> f64 {
    let count = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
    f64::from(count) * CHAR_REM
}

/// The index into [`TICK_WIDTHS`] from which each X label shows, or `None`
/// for a label no listed width has room for.
///
/// Each width keeps every label whose index is a multiple of the smallest
/// power of two that leaves the labels apart, and inside the frame (the
/// plot and the Y gutter left of it, `gutter` rem); a wider chart keeps a
/// smaller power, so a label shown at one width shows at every wider one.
fn tick_levels(labels: &[(f64, f64)], gutter: f64) -> Vec<Option<usize>> {
    let mut levels = vec![None; labels.len()];
    for (level, (width, _)) in TICK_WIDTHS.iter().enumerate() {
        let plot = f64::from(*width) - gutter;
        let span = |&(share, length): &(f64, f64)| {
            let at = share * plot;
            match anchor(share) {
                "start" => (at, at + length),
                "end" => (at - length, at),
                _ => (at - length / 2.0, at + length / 2.0),
            }
        };
        let fits = |stride: usize| {
            let mut last = f64::NEG_INFINITY;
            labels.iter().enumerate().all(|(index, label)| {
                if index % stride != 0 {
                    return true;
                }
                let (start, end) = span(label);
                if start < -gutter || end > plot {
                    return true;
                }
                let apart = start >= last + LABEL_GAP_REM;
                last = end;
                apart
            })
        };
        let mut stride = 1;
        while stride < labels.len() && !fits(stride) {
            stride *= 2;
        }
        for (index, label) in labels.iter().enumerate() {
            let (start, end) = span(label);
            let inside = start >= -gutter && end <= plot;
            if let Some(slot) = levels.get_mut(index)
                && slot.is_none()
                && index % stride == 0
                && inside
            {
                *slot = Some(level);
            }
        }
    }
    levels
}

/// The frame of a drawing `height` high over the two scales.
fn frame(height: f64, x: Scale, y: Scale, x_ticks: &[(f64, String)]) -> Frame {
    let values = ticks(y);
    let mut y_labels = String::new();
    let mut widest: f64 = 0.0;
    for value in values.iter().rev().copied() {
        let label = tick_label(value, y.step);
        widest = widest.max(label_rem(&label));
        y_labels.push_str("<span>");
        escape(&label, &mut y_labels);
        y_labels.push_str("</span>");
    }
    let mut grid = String::new();
    for (index, value) in values.iter().copied().enumerate() {
        let at = y.at(value);
        let class = if index == 0 {
            "fui-chart-axis"
        } else {
            "fui-chart-grid"
        };
        put(
            &mut grid,
            format_args!(
                "<line class=\"{class}\" x1=\"0\" x2=\"100%\" y1=\"{at:.1}\" y2=\"{at:.1}\"/>"
            ),
        );
    }
    let placed: Vec<(f64, f64)> = x_ticks
        .iter()
        .map(|(value, text)| ((x.at(*value) / WIDTH).clamp(0.0, 1.0), label_rem(text)))
        .collect();
    // The gutter is the widest Y label and the frame's column gap.
    let levels = tick_levels(&placed, widest + 0.5);
    let mut x_labels = String::new();
    for (((_, text), (share, _)), level) in x_ticks.iter().zip(&placed).zip(levels) {
        let Some(level) = level else {
            continue;
        };
        let class = match TICK_WIDTHS.get(level) {
            Some((_, hides)) if !hides.is_empty() => format!("fui-chart-label {hides}"),
            _ => String::from("fui-chart-label"),
        };
        put(
            &mut x_labels,
            format_args!(
                "<text class=\"{class}\" x=\"{:.2}%\" y=\"{:.1}\" text-anchor=\"{}\">",
                share * 100.0,
                height - 6.0,
                anchor(*share)
            ),
        );
        escape(text, &mut x_labels);
        x_labels.push_str("</text>");
    }
    Frame {
        height: format!("{height:.0}"),
        view_box: format!("0 0 {WIDTH:.0} {height:.0}"),
        y_labels,
        grid,
        x_labels,
    }
}

/// The SVG of a chart named `label` and summed up by `summary`, the frame
/// `frame` around `lines` in the stretched plot under the X labels, and
/// `marks` (the points) over them.
fn drawn(id: &str, label: &str, summary: &str, frame: &Frame, lines: &str, marks: &str) -> String {
    let Frame {
        height,
        view_box,
        y_labels,
        grid,
        x_labels,
    } = frame;
    let mut svg = String::from(
        "<div class=\"fui-chart-frame\"><div class=\"fui-chart-y\" aria-hidden=\"true\">",
    );
    svg.push_str(y_labels);
    put(
        &mut svg,
        format_args!(
            "</div><svg class=\"fui-chart-svg\" width=\"100%\" height=\"{height}\" role=\"img\" aria-labelledby=\"{id}-title {id}-desc\"><title id=\"{id}-title\">"
        ),
    );
    escape(label, &mut svg);
    put(&mut svg, format_args!("</title><desc id=\"{id}-desc\">"));
    escape(summary, &mut svg);
    svg.push_str("</desc>");
    svg.push_str(grid);
    put(
        &mut svg,
        format_args!(
            "<svg class=\"fui-chart-plot-lines\" width=\"100%\" height=\"{height}\" viewBox=\"{view_box}\" preserveAspectRatio=\"none\">"
        ),
    );
    svg.push_str(lines);
    svg.push_str("</svg>");
    svg.push_str(x_labels);
    svg.push_str(marks);
    svg.push_str("</svg></div>");
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
    let (ys, step) = nice_range(ys);
    (
        Scale {
            from: xs,
            to: (0.0, WIDTH),
            step: 0.0,
        },
        Scale {
            from: ys,
            to: (height - BOTTOM, TOP),
            step,
        },
    )
}

/// The drawing of one series: its line and the dashed bridges of its gaps,
/// and its points.
#[derive(Debug, Clone)]
struct SeriesDrawing {
    /// The paths, drawn in the stretched plot.
    lines: String,
    /// The points, drawn over the X labels.
    marks: String,
}

/// The drawing of `series` under the two scales, its marks referring to
/// the shapes the chart `id` defines.
fn series_drawing(id: &str, series: &Series, x: Scale, y: Scale) -> SeriesDrawing {
    let (tone, mark) = series_look(series.palette);
    let mut lines = String::new();
    let mut marks = String::new();
    put(
        &mut lines,
        format_args!(
            "<path class=\"fui-chart-line fui-series-{tone}\" d=\"{}\"/>",
            path(&series.points, x, y)
        ),
    );
    let bridge = gaps(&series.points, x, y);
    if !bridge.is_empty() {
        put(
            &mut lines,
            format_args!("<path class=\"fui-chart-gap fui-series-{tone}\" d=\"{bridge}\"/>"),
        );
    }
    let finite: Vec<(f64, f64)> = series
        .points
        .iter()
        .copied()
        .filter(|(px, py)| px.is_finite() && py.is_finite())
        .collect();
    let every = finite.len().div_ceil(MARKED).max(1);
    let last = finite.len().saturating_sub(1);
    for (index, (px, py)) in finite.into_iter().enumerate() {
        if index % every != 0 && index != last {
            continue;
        }
        let (cx, cy) = (x.at(px) / WIDTH * 100.0, y.at(py));
        if mark == 0 {
            put(
                &mut marks,
                format_args!(
                    "<circle class=\"fui-chart-point fui-series-{tone}\" cx=\"{cx:.2}%\" cy=\"{cy:.1}\" r=\"3\"/>"
                ),
            );
        } else {
            put(
                &mut marks,
                format_args!(
                    "<use class=\"fui-chart-point fui-series-{tone}\" href=\"#{id}-mark-{mark}\" x=\"{cx:.2}%\" y=\"{cy:.1}\"/>"
                ),
            );
        }
    }
    SeriesDrawing { lines, marks }
}

/// The height of a line chart's drawing.
const LINE_HEIGHT: f64 = 240.0;

/// The frame of `chart` and the drawing of each of its series, in order,
/// after the shapes of the marks past the circle its series take.
fn line_drawing(chart: &LineChart) -> (Frame, String, Vec<SeriesDrawing>) {
    let (x, y) = scales(chart, LINE_HEIGHT);
    let series = chart
        .series
        .iter()
        .map(|series| series_drawing(chart.id, series, x, y))
        .collect();
    let mut shapes: Vec<u8> = chart
        .series
        .iter()
        .map(|series| series_look(series.palette).1)
        .filter(|mark| *mark > 0)
        .collect();
    shapes.sort_unstable();
    shapes.dedup();
    let mut defs = String::new();
    if !shapes.is_empty() {
        defs.push_str("<defs>");
        for mark in shapes {
            put(
                &mut defs,
                format_args!(
                    "<path id=\"{}-mark-{mark}\" d=\"{}\"/>",
                    chart.id,
                    mark_path(mark)
                ),
            );
        }
        defs.push_str("</defs>");
    }
    (frame(LINE_HEIGHT, x, y, &chart.x_ticks), defs, series)
}

/// The SVG of `chart`, drawing only the series `shown` keeps.
#[must_use]
pub fn line_svg(chart: &LineChart, shown: &[bool]) -> String {
    let (frame, defs, series) = line_drawing(chart);
    let mut lines = String::new();
    let mut marks = defs;
    for (index, drawing) in series.iter().enumerate() {
        if shown.get(index).copied().unwrap_or(true) {
            lines.push_str(&drawing.lines);
            marks.push_str(&drawing.marks);
        }
    }
    drawn(
        chart.id,
        &chart.label,
        &chart.summary,
        &frame,
        &lines,
        &marks,
    )
}

/// The SVG of `chart` with every series drawn, its lines and its points
/// each in a group named by the series' index (`data-series-lines` and
/// `data-series-marks`), `hidden` where `shown` hides the series, so the
/// interactive chart shows or hides a series in place.
pub(crate) fn line_svg_grouped(chart: &LineChart, shown: &[bool]) -> String {
    let (frame, defs, series) = line_drawing(chart);
    let mut lines = String::new();
    let mut marks = defs;
    for (index, drawing) in series.iter().enumerate() {
        for (out, part, inner) in [
            (&mut lines, "lines", &drawing.lines),
            (&mut marks, "marks", &drawing.marks),
        ] {
            if shown.get(index).copied().unwrap_or(true) {
                put(
                    out,
                    format_args!("<g data-series-{part}=\"{index}\">{inner}</g>"),
                );
            } else {
                put(
                    out,
                    format_args!("<g data-series-{part}=\"{index}\" hidden>{inner}</g>"),
                );
            }
        }
    }
    drawn(
        chart.id,
        &chart.label,
        &chart.summary,
        &frame,
        &lines,
        &marks,
    )
}

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

/// One entry of a chart's legend: a series' name and its colour.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegendEntry {
    /// The series' name, as the table shows it.
    pub name: String,
    /// The series' palette index, which pins its colour.
    pub palette: u8,
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

/// Returns a series' swatch: its point mark `shape` in the colour `class`
/// names, as the plot draws it.
fn swatch_of(class: &str, shape: &str) -> Html {
    Html::raw(format!(
        "<svg viewBox=\"-5 -5 10 10\" focusable=\"false\" aria-hidden=\"true\" class=\"{class}\"><path d=\"{shape}\"></path></svg>"
    ))
}

#[derive(Template)]
#[template(path = "chart/chart.html", block = "legend")]
struct LegendView<'a> {
    label: String,
    plot: &'a str,
    entries: Vec<(String, Html)>,
    gap: Option<Word>,
}

/// Returns the legend of the plot `plot`, named `label`, one entry per
/// series in `entries`.
///
/// `gap` says what a break in a line means when the chart has one. It is
/// a static key the `chart` module turns into toggles; a console draws it
/// through [`line_chart`].
#[must_use]
pub fn legend(plot: &str, label: String, entries: Vec<LegendEntry>, gap: Option<Word>) -> Html {
    LegendView {
        label,
        plot,
        entries: entries
            .into_iter()
            .map(|entry| {
                let (tone, mark) = series_look(entry.palette);
                let swatch = swatch_of(&format!("fui-swatch fui-series-{tone}"), mark_path(mark));
                (entry.name, swatch)
            })
            .collect(),
        gap,
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{
        LineChart, Scale, Series, TICK_WIDTHS, label_rem, line_svg, line_svg_grouped, nice_range,
        tick_label, tick_levels, ticks, value_label,
    };

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
