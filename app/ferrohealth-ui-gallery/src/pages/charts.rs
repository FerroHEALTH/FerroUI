// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page: a line chart of three synthetic series over a week,
//! its legend showing and hiding each, and the activity line of a
//! dashboard.

use crate::locale::{Msg, tr};
use ferrohealth_ui::view::chart::{
    ACCENT, ActivityChart, BAD, LineChart, NEUTRAL, Series, activity_chart,
};
use ferrohealth_ui::view::surface::{card, page_header, stack};
use ferrohealth_ui::widget::chart::{ChartWords, line_chart};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The days of the synthetic week, as the X axis labels them.
fn days() -> [String; 7] {
    [
        Msg::Mon,
        Msg::Tue,
        Msg::Wed,
        Msg::Thu,
        Msg::Fri,
        Msg::Sat,
        Msg::Sun,
    ]
    .map(tr)
}

/// The line chart of three synthetic series, one with a gap, drawn by the
/// island [`crate::islands::RequestsChart`].
fn lines() -> AnyView {
    card(
        tr(Msg::LineChart),
        view! { <crate::islands::RequestsChart words=RequestsWords::of_page() /> },
    )
    .into_any()
}

/// The words the line chart's island draws, resolved on the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestsWords {
    label: String,
    summary: String,
    reads: String,
    writes: String,
    refusals: String,
    days: [String; 7],
    chart: ChartWords,
}

impl RequestsWords {
    /// The words in the page's locale.
    pub(crate) fn of_page() -> Self {
        Self {
            label: tr(Msg::RequestsPerDay),
            summary: tr(Msg::ThreeSyntheticSeriesOverWeek),
            reads: tr(Msg::Reads),
            writes: tr(Msg::Writes),
            refusals: tr(Msg::Refusals),
            days: days(),
            chart: ChartWords::of_page(),
        }
    }
}

/// The line chart of three synthetic series in `words`, its legend showing
/// and hiding each.
pub(crate) fn requests_chart(words: RequestsWords) -> AnyView {
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
        y_range: None,
    };
    let shown = RwSignal::new(vec![true, true, true]);
    line_chart(chart, shown, chart_words)
}

/// The activity line of thirty synthetic days.
fn activity() -> AnyView {
    let counts = [
        0, 1, 0, 2, 3, 1, 0, 0, 4, 2, 1, 0, 3, 5, 2, 0, 1, 1, 0, 2, 6, 3, 1, 0, 0, 2, 4, 1, 0, 2,
    ];
    let points = (0_u32..).zip(counts).collect();
    card(
        tr(Msg::Activity),
        activity_chart(&ActivityChart {
            name: "commits",
            label: tr(Msg::CommitsPerDay),
            summary: tr(Msg::N48SyntheticCommitsOver30),
            points,
            ends: (tr(Msg::Sep19), tr(Msg::Oct18)),
        }),
    )
    .into_any()
}

/// The charts page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn ChartsPage() -> impl IntoView {
    let header =
        page_header(tr(Msg::Charts), Some(tr(Msg::DrawnSvgServerScaledCard)), ()).into_any();
    let sections = (lines(), activity());
    view! {
        {super::title(tr(Msg::Charts))}
        {header}
        {stack(
            view! {
                {sections.0}
                {sections.1}
            },
        )}
    }
}
