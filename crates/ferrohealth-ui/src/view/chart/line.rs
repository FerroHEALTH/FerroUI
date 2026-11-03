// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The lines of a line chart: the thinned points, the paths, the gaps and the
//! SVG.

use super::frame::{Frame, drawn, frame};
use super::scale::{BOTTOM, Scale, TOP, WIDTH, extent, nice_range, put};
use super::{COLUMNS, LineChart, MARKED, Series, mark_path, series_look};

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
pub(super) fn path(points: &[(f64, f64)], x: Scale, y: Scale) -> String {
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

/// The scales of `chart` within a drawing `height` high.
pub(super) fn scales(chart: &LineChart, height: f64) -> (Scale, Scale) {
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
