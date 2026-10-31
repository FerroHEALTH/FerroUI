// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The dates written twice (#548): the library's `calendar` and the date
//! field's words in Rust, and the `calendar` and `date_range` modules that
//! draw the picker in the browser, held to one answer over tables of days,
//! typed texts and bounds (`parity` holds the harness).

use std::error::Error;

use ferrohealth_ui::calendar::{
    Bounds, DateOrder, Day, Moment, Month, NotADay, Weekday, days_in, leap,
};
use ferrohealth_ui::i18n::Locale;
use ferrohealth_ui::view::date::{DateWords, range_status, range_summary};
use serde_json::{Value, json};

use crate::parity::held;

/// The first day the calendar counts from, 1970-01-01.
fn epoch() -> Day {
    Day::new(1970, 1, 1).expect("1970-01-01 is a day")
}

/// The day `day` as the modules count it, in days from 1970-01-01.
fn number(day: Day) -> i64 {
    let through = i64::from(epoch().days_through(day)) - 1;
    if day < epoch() { -through } else { through }
}

/// The days the tables read: each end of the calendar, the leap days and
/// the years around them, the turn of every month of a common and a leap
/// year, and the days around 1970-01-01.
fn days() -> Vec<Day> {
    let mut out = Vec::new();
    for year in [
        1, 4, 99, 100, 400, 1582, 1600, 1900, 1969, 1970, 2000, 2024, 2026, 9999,
    ] {
        for month in 1..=12 {
            for date in [1, 28, days_in(year, month)] {
                out.extend(Day::new(year, month, date));
            }
        }
    }
    out.extend(Day::new(9999, 12, 31));
    out.sort();
    out.dedup();
    out
}

/// A table of bounds: none, either end, both, and a single day.
fn bounds() -> Vec<Bounds> {
    let day = |text| Day::parse(text).ok();
    vec![
        Bounds::default(),
        Bounds {
            earliest: day("2026-10-15"),
            latest: None,
        },
        Bounds {
            earliest: None,
            latest: day("2026-10-15"),
        },
        Bounds {
            earliest: day("2026-02-28"),
            latest: day("2026-03-01"),
        },
        Bounds {
            earliest: day("2024-02-29"),
            latest: day("2024-02-29"),
        },
    ]
}

/// `bounds` as the modules read it, each end as its day number or `null`.
fn bounds_json(bounds: Bounds) -> Value {
    json!({
        "earliest": bounds.earliest.map(number),
        "latest": bounds.latest.map(number),
    })
}

/// The texts a field is typed: ISO and the two written orders, each part
/// too long, too short, out of range or not a digit, with and without a
/// time, and white space the two languages trim differently.
const TYPED: &[&str] = &[
    "",
    "   ",
    "2026-10-23",
    " 2026-10-23 ",
    "2026-02-29",
    "2024-02-29",
    "2026-13-01",
    "2026-00-10",
    "2026-04-31",
    "0000-01-01",
    "9999-12-31",
    "2026-1-5",
    "20261023",
    "2026/10/23",
    "2026-10-2x",
    "\u{ff12}026-10-23",
    "23.10.2026",
    "4/3/2026",
    "10/23/2026",
    "31.02.2024",
    "29-02-2024",
    "1.13.2026",
    "13.1.2026",
    "001.1.2026",
    "1.1.26",
    "1.1.0000",
    "1..2026",
    "+1.1.2026",
    "2026-10-23 09:30",
    "2026-10-23T23:59",
    "2026-10-23 24:00",
    "2026-10-23 9:30",
    "2026-10-23  09:30",
    "2026-10-23 09:60",
    "23.10.2026 09:30",
    "2026-10-23T",
    "T09:30",
    "\u{feff}2026-10-23",
    "2026-10-23\u{feff}",
    "\u{85}2026-10-23",
    "2026-10-23\u{85}",
    "\u{a0}2026-10-23\u{3000}",
];

/// The orders a page's language writes a date in, as the modules name them.
const ORDERS: [(DateOrder, &str); 3] = [
    (DateOrder::Ymd, "Ymd"),
    (DateOrder::Dmy, "Dmy"),
    (DateOrder::Mdy, "Mdy"),
];

/// What a parse answers, as the `calendar` module writes it.
fn read(parsed: Result<Moment, NotADay>) -> Value {
    match parsed {
        Ok(Moment { day, time: None }) => json!({"day": number(day)}),
        Ok(Moment {
            day,
            time: Some(time),
        }) => json!({"day": number(day), "hour": time.hour(), "minute": time.minute()}),
        Err(NotADay::Format) => json!({"error": "format"}),
        Err(NotADay::Time) => json!({"error": "time"}),
        Err(NotADay::Month { month }) => json!({"error": "month", "month": month}),
        Err(NotADay::Day { year, month }) => {
            json!({"error": "day", "year": year, "month": month})
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_calendar_counts_names_and_places_a_day_alike() -> Result<(), Box<dyn Error>> {
    let cases = days()
        .into_iter()
        .map(|day| {
            (
                json!([day.year(), day.month(), day.day()]),
                json!([number(day), day.to_string(), day.weekday().index()]),
            )
        })
        .collect();
    held(
        "calendar",
        "(m, y, mo, d) => { const n = m.toNumber(y, mo, d); const back = m.fromNumber(n); return back.y === y && back.m === mo && back.d === d ? [n, m.iso(n), m.weekday(n)] : ['no round trip', back]; }",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_calendar_month_lengths_and_steps_agree() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for year in [1, 4, 100, 400, 1900, 2000, 2024, 2026, 9999] {
        for month in 1..=12 {
            let stepped: Vec<Value> = [-120_009, -13, -12, -1, 0, 1, 11, 12, 13, 120_009]
                .into_iter()
                .map(|count| {
                    Month::new(year, month)
                        .and_then(|shown| shown.step(count))
                        .map_or(Value::Null, |to| json!({"y": to.year(), "m": to.month()}))
                })
                .collect();
            cases.push((
                json!([year, month]),
                json!([leap(year), days_in(year, month), stepped]),
            ));
        }
    }
    held(
        "calendar",
        "(m, y, mo) => [m.leap(y), m.daysIn(y, mo), [-120009, -13, -12, -1, 0, 1, 11, 12, 13, 120009].map((c) => m.stepMonth(y, mo, c))]",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_calendar_weeks_of_a_month_agree() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for (year, month) in [
        (1, 1),
        (2024, 2),
        (2026, 2),
        (2026, 10),
        (2026, 11),
        (9999, 12),
    ] {
        let Some(shown) = Month::new(year, month) else {
            continue;
        };
        for first in Weekday::ALL {
            let weeks: Vec<Vec<i64>> = shown
                .weeks(first)
                .iter()
                .map(|week| week.iter().map(|day| number(*day)).collect())
                .collect();
            cases.push((json!([year, month, first.index()]), json!(weeks)));
        }
    }
    held(
        "calendar",
        "(m, y, mo, first) => m.weeks(y, mo, first)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_calendar_reads_a_typed_day_and_time_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for text in TYPED {
        for (order, key) in ORDERS {
            for with_time in [false, true] {
                let parsed = read(Moment::parse_in(text, with_time, order));
                cases.push((json!([text, with_time, key]), parsed));
            }
            let day = read(Day::parse_in(text, order).map(|day| Moment { day, time: None }));
            cases.push((json!([text, null, key]), day));
        }
    }
    held(
        "calendar",
        "(m, text, withTime, order) => withTime === null ? m.parseDay(text, order) : m.parseMoment(text, withTime, order)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_calendar_bounds_allow_and_reach_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for limits in bounds() {
        for day in days() {
            cases.push((
                json!(["allows", bounds_json(limits), number(day)]),
                json!(limits.allows(day)),
            ));
        }
        for (year, month) in [
            (2024, 2),
            (2026, 2),
            (2026, 3),
            (2026, 9),
            (2026, 10),
            (2026, 11),
        ] {
            let Some(shown) = Month::new(year, month) else {
                continue;
            };
            cases.push((
                json!(["reaches", bounds_json(limits), year, month]),
                json!(limits.reaches(shown)),
            ));
        }
    }
    held(
        "calendar",
        "(m, ask, bounds, a, b) => ask === 'allows' ? m.allows(bounds, a) : m.reaches(bounds, a, b)",
        cases,
    )
    .await
}

/// The date words in English, in each of the orders, with the day counts.
fn words() -> Vec<DateWords> {
    let english = Locale::english();
    ORDERS
        .iter()
        .flat_map(|(order, _)| {
            [false, true].map(|with_time| DateWords {
                order: *order,
                ..DateWords::of(&english, with_time).counted(&english)
            })
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_date_field_says_why_a_typed_text_is_refused_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for words in words() {
        let wire = serde_json::to_value(&words)?;
        for limits in bounds() {
            for text in TYPED {
                for with_time in [false, true] {
                    let reason = words
                        .reason(text, with_time, limits)
                        .map(|word| word.text().to_owned());
                    cases.push((
                        json!([wire, text, with_time, bounds_json(limits)]),
                        json!(reason),
                    ));
                }
            }
        }
    }
    held(
        "calendar",
        "(m, words, text, withTime, bounds) => m.reason(words, text, withTime, bounds)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_date_picker_names_a_month_and_a_day_alike() -> Result<(), Box<dyn Error>> {
    let english = Locale::english();
    let words = DateWords::of(&english, false);
    let wire = serde_json::to_value(&words)?;
    let cases = days()
        .into_iter()
        .map(|day| {
            (
                json!([wire, number(day)]),
                json!([
                    words.month_title(day.month_of()).text(),
                    words.day_name(day).text()
                ]),
            )
        })
        .collect();
    held(
        "calendar",
        "(m, words, n) => { const { y, m: mo } = m.fromNumber(n); return [m.monthTitle(words, y, mo), m.dayName(words, n)]; }",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_date_range_summary_and_status_read_alike() -> Result<(), Box<dyn Error>> {
    let english = Locale::english();
    let words = DateWords::of(&english, false).counted(&english);
    let wire = serde_json::to_value(&words)?;
    let ends = [
        "",
        "2026-10-01",
        "2026-10-31",
        "2026-12-01",
        "2026-02-30",
        " 2026-10-05",
        "2026-10-05\u{85}",
        "2026-10-05\u{feff}",
        "05.10.2026",
    ];
    let mut cases = Vec::new();
    for from in ends {
        for to in ends {
            let first = Day::parse(from).ok();
            let last = Day::parse(to).ok();
            cases.push((
                json!([wire, from, to]),
                json!([
                    range_summary(&words, from, to).text(),
                    range_status(&words, first, last).text()
                ]),
            ));
        }
    }
    held(
        "date_range",
        "(m, words, from, to) => [m.summary(words, from, to), m.status(words, from, to)]",
        cases,
    )
    .await
}
