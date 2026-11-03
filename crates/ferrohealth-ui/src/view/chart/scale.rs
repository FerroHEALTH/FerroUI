// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The value scale of a line chart: its range, its ticks and their labels.

/// The width of the inner drawing's `viewBox`, which is stretched to the
/// plot's width; the X labels are placed at the same share of the width.
pub(super) const WIDTH: f64 = 1000.0;

/// The room above the plot, and below it for the X labels, in CSS pixels;
/// the Y gutter's padding in the stylesheet is each less half a line.
pub(super) const TOP: f64 = 10.0;
pub(super) const BOTTOM: f64 = 26.0;

/// A linear map from a data range onto a drawing range.
#[derive(Debug, Clone, Copy)]
pub(super) struct Scale {
    pub(super) from: (f64, f64),
    pub(super) to: (f64, f64),
    /// The distance between two ticks, a round number; zero on an axis
    /// whose ticks the chart names itself.
    pub(super) step: f64,
}

impl Scale {
    pub(super) fn at(self, value: f64) -> f64 {
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
pub(super) fn put(out: &mut String, args: std::fmt::Arguments<'_>) {
    use std::fmt::Write as _;
    // NOTE: the Rust standard library, `impl Write for String`: writing to a
    // String never fails, so there is no error to carry.
    if out.write_fmt(args).is_err() {
        out.clear();
    }
}

/// The smallest and the largest finite value of `values`, or `None`.
pub(super) fn extent(values: impl Iterator<Item = f64>) -> Option<(f64, f64)> {
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
pub(super) fn nice_range((low, high): (f64, f64)) -> ((f64, f64), f64) {
    let step = nice_step((high - low) / 3.0);
    let low = (low / step).floor() * step;
    let high = (high / step).ceil() * step;
    let high = if high - low < step { low + step } else { high };
    ((low, high), step)
}

/// The values of the round ticks over `scale`, the lowest first.
pub(super) fn ticks(scale: Scale) -> Vec<f64> {
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
