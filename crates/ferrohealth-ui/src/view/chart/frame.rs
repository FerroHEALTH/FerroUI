// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The frame of a line chart: the axes, the gridlines and the levels of the
//! tick labels.

use crate::highlight::escape;

use super::scale::{Scale, WIDTH, put, tick_label, ticks};

/// The parts every chart's drawing shares that no series changes: the
/// labels and grid lines of the Y ticks and the labels of the X ticks.
#[derive(Debug, Clone)]
pub(super) struct Frame {
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
pub(super) const TICK_WIDTHS: [(u8, &str); 9] = [
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
pub(super) fn label_rem(text: &str) -> f64 {
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
pub(super) fn tick_levels(labels: &[(f64, f64)], gutter: f64) -> Vec<Option<usize>> {
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
pub(super) fn frame(height: f64, x: Scale, y: Scale, x_ticks: &[(f64, String)]) -> Frame {
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
pub(super) fn drawn(
    id: &str,
    label: &str,
    summary: &str,
    frame: &Frame,
    lines: &str,
    marks: &str,
) -> String {
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
