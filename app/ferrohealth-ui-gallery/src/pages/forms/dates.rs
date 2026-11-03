// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page's date fields and ranges.

use ferrohealth_ui::calendar::{Bounds, Day, Preset, Quick, Weekday};
use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Phrase;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::date::{
    DateField, DateRange, DateWords, QuickChoice, RangeEnd, RangePreset, date_field, date_range,
};
use ferrohealth_ui::view::form::{FIELD_FLOW, FORM};
use ferrohealth_ui::view::surface::card;

use crate::locale::Msg;
use crate::visit::Visit;

/// The day the gallery takes as today, so every capture draws the same
/// month.
#[expect(
    clippy::expect_used,
    reason = "2026-10-21 is a day of the calendar, by inspection"
)]
pub(super) fn today() -> Day {
    Day::new(2026, 10, 21).expect("the gallery's today should be a day")
}

/// A date field `id` posting as `name`, labelled `label`, holding `value`
/// and allowing `bounds`, its week from Monday.
pub(super) fn date(
    v: &Visit,
    id: &str,
    name: &str,
    label: String,
    value: &str,
    bounds: Bounds,
) -> DateField {
    DateField {
        id: id.to_owned(),
        name: name.to_owned(),
        label,
        value: value.to_owned(),
        help: None,
        error: None,
        today: today(),
        bounds,
        first_day: Weekday::Monday,
        with_time: false,
        quick: Vec::new(),
        chosen: None,
        words: DateWords::of(&v.locale, false),
    }
}

/// The date fields: an audit's From that forbids the future, a deferral's
/// date with its quick choices, and a history filter's range.
pub(super) fn dates(v: &Visit) -> Html {
    let past = Bounds {
        earliest: None,
        latest: Some(today()),
    };
    let from = DateField {
        help: Some(v.tr(Msg::DaysAfterTodayNotOffered)),
        ..date(
            v,
            "audit-from",
            "from",
            v.tr(Msg::FromDay),
            "2026-10-14",
            past,
        )
    };
    let quick = [
        (Quick::Days(1), Msg::Tomorrow),
        (Quick::Days(3), Msg::InThreeDays),
        (Quick::Next(Weekday::Monday), Msg::NextMonday),
        (Quick::Days(7), Msg::InAWeek),
    ]
    .into_iter()
    .map(|(quick, words)| QuickChoice {
        quick,
        words: v.tr(words),
    })
    .collect();
    let defer = DateField {
        quick,
        chosen: Some(Phrase::new(
            v.tr_with(Msg::BackInQueueOn, &[("day", Phrase::open("day"))]),
        )),
        ..date(
            v,
            "defer-until",
            "until",
            v.tr(Msg::DeferUntil),
            "2026-10-28",
            Bounds {
                earliest: today().step(1),
                latest: None,
            },
        )
    };
    let range = DateRange {
        id: String::from("audit-range"),
        label: v.tr(Msg::ActionsBetween),
        from: RangeEnd {
            id: String::from("audit-range-from"),
            name: String::from("from"),
            label: v.tr(Msg::FromDay),
            value: String::from("2026-10-15"),
        },
        to: RangeEnd {
            id: String::from("audit-range-to"),
            name: String::from("to"),
            label: v.tr(Msg::ToDay),
            value: String::from("2026-10-21"),
        },
        today: today(),
        bounds: past,
        first_day: Weekday::Monday,
        presets: [
            Preset::Today,
            Preset::LastDays(7),
            Preset::LastDays(30),
            Preset::ThisMonth,
        ]
        .into_iter()
        .map(|preset| RangePreset::of(&v.locale, preset))
        .collect(),
        words: DateWords::of(&v.locale, false).counted(&v.locale),
    };
    card(
        v.tr(Msg::DateFields),
        Html::raw(format!(
            "<div class=\"{FORM}\">{}{}<form method=\"get\" action=\"#date-range\" id=\"date-range\" class=\"{FIELD_FLOW}\">{}<button type=\"submit\" class=\"{}\">{}</button></form></div>",
            date_field(from),
            date_field(defer),
            date_range(range),
            Variant::Secondary.class(),
            Html::text(v.tr(Msg::ShowActions)),
        )),
    )
}
