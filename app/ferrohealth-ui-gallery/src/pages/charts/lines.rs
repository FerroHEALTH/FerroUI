// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page's line charts: three series over a week and eight lanes of
//! small values.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::view::chart::{ACCENT, BAD, LineChart, NEUTRAL, Series};
use ferrohealth_ui::view::chart::{ChartWords, line_chart};
use ferrohealth_ui::view::surface::card;

use crate::locale::Msg;
use crate::visit::Visit;

/// The days of the synthetic week, as the X axis labels them.
fn days(v: &Visit) -> [String; 7] {
    [
        Msg::Mon,
        Msg::Tue,
        Msg::Wed,
        Msg::Thu,
        Msg::Fri,
        Msg::Sat,
        Msg::Sun,
    ]
    .map(|msg| v.tr(msg))
}

/// The line chart of three synthetic series, one with a gap, its legend
/// the library's module.
pub(super) fn lines(v: &Visit) -> Html {
    card(v.tr(Msg::LineChart), requests_chart(RequestsWords::of(v)))
}

/// The words the line chart draws, resolved on the server.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestsWords {
    label: String,
    summary: String,
    reads: String,
    writes: String,
    refusals: String,
    days: [String; 7],
    chart: ChartWords,
}

impl RequestsWords {
    /// The words in the visit's locale.
    fn of(v: &Visit) -> Self {
        Self {
            label: v.tr(Msg::RequestsPerDay),
            summary: v.tr(Msg::ThreeSyntheticSeriesOverWeek),
            reads: v.tr(Msg::Reads),
            writes: v.tr(Msg::Writes),
            refusals: v.tr(Msg::Refusals),
            days: days(v),
            chart: ChartWords::of(&v.locale),
        }
    }
}

/// The line chart of three synthetic series in `words`, its legend showing
/// and hiding each.
fn requests_chart(words: RequestsWords) -> Html {
    let RequestsWords {
        label,
        summary,
        reads,
        writes,
        refusals,
        days,
        chart: chart_words,
    } = words;
    let series = |name: &str, palette: u8, values: [f64; 7]| Series {
        name: name.to_owned(),
        points: (0_u8..7)
            .zip(values)
            .map(|(day, value)| (f64::from(day), value))
            .collect(),
        palette,
    };
    let chart = LineChart {
        id: "requests",
        label,
        summary,
        series: vec![
            series(&reads, ACCENT, [22.0, 30.0, 27.0, 35.0, 48.0, 31.0, 26.0]),
            series(&writes, NEUTRAL, [10.0, 12.0, 14.0, 11.0, 13.0, 9.0, 12.0]),
            series(&refusals, BAD, [3.0, 5.0, f64::NAN, 4.0, 6.0, 2.0, 1.0]),
        ],
        x_ticks: (0_u8..7)
            .zip(days)
            .map(|(day, label)| (f64::from(day), label))
            .collect(),
        x_labels: Vec::new(),
        y_range: None,
    };
    line_chart(&chart, chart_words)
}

/// The line chart of eight synthetic lanes of small values over twelve
/// days: past the six colours a lane takes its own mark, the value axis
/// keeps the decimals of its step, and a narrow card thins the dates.
pub(super) fn many_lines(v: &Visit) -> Html {
    let series = (0_u8..8)
        .map(|lane| Series {
            name: v.tr_with(Msg::ManyLane, &[("index", (lane + 1).to_string().into())]),
            points: (0_u8..12)
                .map(|day| {
                    (
                        f64::from(day),
                        (f64::from(lane) + 1.0 + f64::from(day % 4) * 0.25) * 0.0001,
                    )
                })
                .collect(),
            palette: lane,
        })
        .collect();
    let chart = LineChart {
        id: "lanes",
        label: v.tr(Msg::ManySeriesLabel),
        summary: v.tr(Msg::ManySeriesSummary),
        series,
        x_ticks: (0_u8..12)
            .map(|day| (f64::from(day), format!("2026-10-{:02}", day + 1)))
            .collect(),
        x_labels: Vec::new(),
        y_range: None,
    };
    card(
        v.tr(Msg::LineChartMany),
        line_chart(&chart, ChartWords::of(&v.locale)),
    )
}
