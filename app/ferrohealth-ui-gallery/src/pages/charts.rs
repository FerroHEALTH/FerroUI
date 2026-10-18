// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page: a line chart of three synthetic series over a week,
//! its legend showing and hiding each, and the activity line of a
//! dashboard.

use ferrohealth_ui::view::chart::{ActivityChart, LineChart, Series, activity_chart};
use ferrohealth_ui::view::surface::{card, page_header, stack};
use ferrohealth_ui::widget::chart::line_chart;
use leptos::prelude::*;

/// The days of the synthetic week, as the X axis labels them.
const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// The line chart of three synthetic series, one with a gap.
fn lines() -> AnyView {
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
        label: String::from("Requests per day"),
        summary: String::from(
            "Three synthetic series over a week: reads rise to 48 on Friday, writes stay near 12, and refusals have no value on Wednesday.",
        ),
        series: vec![
            series("Reads", 0, [22.0, 30.0, 27.0, 35.0, 48.0, 31.0, 26.0]),
            series("Writes", 1, [10.0, 12.0, 14.0, 11.0, 13.0, 9.0, 12.0]),
            series("Refusals", 2, [3.0, 5.0, f64::NAN, 4.0, 6.0, 2.0, 1.0]),
        ],
        x_ticks: (0_u8..7)
            .zip(DAYS)
            .map(|(day, label)| (f64::from(day), label.to_owned()))
            .collect(),
        y_range: None,
    };
    let shown = RwSignal::new(vec![true, true, true]);
    card("Line chart", line_chart(chart, shown)).into_any()
}

/// The activity line of thirty synthetic days.
fn activity() -> AnyView {
    let counts = [
        0, 1, 0, 2, 3, 1, 0, 0, 4, 2, 1, 0, 3, 5, 2, 0, 1, 1, 0, 2, 6, 3, 1, 0, 0, 2, 4, 1, 0, 2,
    ];
    let points = (0_u32..).zip(counts).collect();
    card(
        "Activity",
        activity_chart(&ActivityChart {
            name: "commits",
            label: String::from("Commits per day"),
            summary: String::from("48 synthetic commits over 30 days, most on day 21."),
            points,
            ends: (String::from("Sep 19"), String::from("Oct 18")),
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
    let header = page_header(
        "Charts",
        Some(String::from(
            "Drawn as SVG on the server, scaled to the card, each with its figures for assistive technology.",
        )),
        (),
    )
    .into_any();
    let sections = (lines(), activity());
    view! {
        {super::title("Charts")}
        {header}
        {stack(
            view! {
                {sections.0}
                {sections.1}
            },
        )}
    }
}
