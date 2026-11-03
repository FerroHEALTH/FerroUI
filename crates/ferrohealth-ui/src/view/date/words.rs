// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The date pickers' words in the page's locale: the month and weekday names
//! and the phrases the picker fills.

use crate::calendar::{Bounds, DateOrder, Day, Moment, Month, NotADay};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};

use super::DateWords;

/// The library's message `text` in the page's locale with each of `names`
/// left open for the picker to fill as the reader chooses.
fn phrase(locale: &Locale, text: Text, names: &[&'static str]) -> Phrase {
    let args: Vec<(&'static str, crate::i18n::Arg)> = names
        .iter()
        .map(|name| (*name, Phrase::open(name)))
        .collect();
    Phrase::new(tr_with(locale, text, &args))
}

/// The longest range whose number of days the picker's line states.
const COUNTED: u32 = 366;

impl DateWords {
    /// The words in `locale`, for a field that takes a time when
    /// `with_time`; a range's words count its days ([`DateWords::counted`]).
    #[must_use]
    pub fn of(locale: &Locale, with_time: bool) -> Self {
        let (format, format_hint, type_format, not_format) = if with_time {
            (
                Text::DateTimeFormat,
                Text::DateTimeFormatHint,
                Text::DateTypeFormatTime,
                Text::DateNotFormatTime,
            )
        } else {
            (
                Text::DateFormat,
                Text::DateFormatHint,
                Text::DateTypeFormat,
                Text::DateNotFormat,
            )
        };
        let tr = |text| tr(locale, text);
        let months: Vec<Word> = MONTHS.iter().map(|month| tr(*month)).collect();
        let mut month_lengths: Vec<Word> = (1_u8..=12)
            .zip(&months)
            .map(|(month, name)| {
                let days = crate::calendar::days_in(2026, month);
                tr_with(
                    locale,
                    Text::DateMonthLength,
                    &[
                        ("month", name.clone().into()),
                        ("count", u32::from(days).into()),
                    ],
                )
            })
            .collect();
        month_lengths.push(tr_with(
            locale,
            Text::DateMonthLength,
            &[
                ("month", months.get(1).cloned().unwrap_or_default().into()),
                ("count", 29_u32.into()),
            ],
        ));
        Self {
            format: tr(format),
            order: DateOrder::of(locale),
            format_hint: tr(format_hint),
            choose: tr(Text::DateChoose),
            needs_script: tr(Text::DateNeedsScript),
            previous_month: tr(Text::DatePreviousMonth),
            next_month: tr(Text::DateNextMonth),
            previous_year: tr(Text::DatePreviousYear),
            next_year: tr(Text::DateNextYear),
            back_to_days: tr(Text::DateBackToDays),
            today: tr(Text::DateToday),
            clear: tr(Text::DateClear),
            cancel: tr(Text::DateCancel),
            apply: tr(Text::DateApply),
            time: tr(Text::DateTime),
            time_hint: tr(Text::DateTimeHint),
            hours: tr(Text::DateHours),
            minutes: tr(Text::DateMinutes),
            quick: tr(Text::DateQuick),
            ranges: tr(Text::DateRanges),
            any: tr(Text::DateAny),
            type_format: tr(type_format),
            months,
            weekdays: WEEKDAYS.iter().map(|day| tr(*day)).collect(),
            weekdays_short: WEEKDAYS_SHORT.iter().map(|day| tr(*day)).collect(),
            month_lengths,
            month_title: phrase(locale, Text::DateMonthTitle, &["month", "year"]),
            day_name: phrase(
                locale,
                Text::DateDayName,
                &["weekday", "day", "month", "year"],
            ),
            choose_month: phrase(locale, Text::DateChooseMonth, &["title"]),
            months_of: phrase(locale, Text::DateMonthsOf, &["year"]),
            range_words: phrase(locale, Text::DateRangeWords, &["from", "to"]),
            range_half: phrase(locale, Text::DateRangeHalf, &["from"]),
            range_full: phrase(locale, Text::DateRangeFull, &["from", "to", "days"]),
            not_a_day: phrase(locale, Text::DateNotADay, &["value", "why"]),
            no_month: phrase(locale, Text::DateNoMonth, &["value", "month"]),
            not_a_time: phrase(locale, Text::DateNotATime, &["value"]),
            not_format: phrase(locale, not_format, &["value"]),
            before: phrase(locale, Text::DateBefore, &["value", "earliest"]),
            after: phrase(locale, Text::DateAfter, &["value", "latest"]),
            day_counts: Vec::new(),
        }
    }

    /// These words with the number of days a range holds, from 1 to
    /// `COUNTED` (62), each with its plural in `locale`.
    #[must_use]
    pub fn counted(self, locale: &Locale) -> Self {
        Self {
            day_counts: (1..=COUNTED)
                .map(|count| tr_with(locale, Text::DateDays, &[("count", count.into())]))
                .collect(),
            ..self
        }
    }

    /// The name of `month`, 1 to 12.
    #[must_use]
    pub fn month_name(&self, month: u8) -> &str {
        self.months
            .get(usize::from(month.saturating_sub(1)))
            .map_or("", Word::text)
    }

    /// The title of `month`, as the picker heads it.
    #[must_use]
    pub fn month_title(&self, month: Month) -> Word {
        self.month_title.fill(&[
            ("month", self.month_name(month.month())),
            ("year", &month.year().to_string()),
        ])
    }

    /// The full name of `day`, its accessible name in the grid.
    #[must_use]
    pub fn day_name(&self, day: Day) -> Word {
        let weekday = self
            .weekdays
            .get(usize::from(day.weekday().index()))
            .map_or("", Word::text);
        self.day_name.fill(&[
            ("weekday", weekday),
            ("day", &day.day().to_string()),
            ("month", self.month_name(day.month())),
            ("year", &day.year().to_string()),
        ])
    }

    /// The number of days `count`, for a range of at most `COUNTED` (62)
    /// days.
    #[must_use]
    pub fn days(&self, count: u32) -> Option<&str> {
        let place = usize::try_from(count.checked_sub(1)?).ok()?;
        self.day_counts.get(place).map(Word::text)
    }

    /// Why the typed `text` is no value the field takes, or `None` when it
    /// is one or is empty: the sentence the picker says, which a console's
    /// handler says too when it refuses a posted value.
    #[must_use]
    pub fn reason(&self, text: &str, with_time: bool, bounds: Bounds) -> Option<Word> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let moment = match Moment::parse(text, with_time, self.order) {
            Ok(moment) => moment,
            Err(NotADay::Format) => return Some(self.not_format.fill(&[("value", text)])),
            Err(NotADay::Time) => return Some(self.not_a_time.fill(&[("value", text)])),
            Err(NotADay::Month { month }) => {
                return Some(
                    self.no_month
                        .fill(&[("value", text), ("month", &month.to_string())]),
                );
            }
            Err(NotADay::Day { year, month }) => {
                let place = if month == 2 && crate::calendar::leap(year) {
                    12
                } else {
                    usize::from(month.saturating_sub(1))
                };
                let why = self.month_lengths.get(place).map_or("", Word::text);
                return Some(self.not_a_day.fill(&[("value", text), ("why", why)]));
            }
        };
        let day = moment.day;
        if let Some(earliest) = bounds.earliest.filter(|earliest| day < *earliest) {
            return Some(
                self.before
                    .fill(&[("value", text), ("earliest", &earliest.to_string())]),
            );
        }
        if let Some(latest) = bounds.latest.filter(|latest| day > *latest) {
            return Some(
                self.after
                    .fill(&[("value", text), ("latest", &latest.to_string())]),
            );
        }
        None
    }
}

/// The months' messages, January first.
const MONTHS: [Text; 12] = [
    Text::Month1,
    Text::Month2,
    Text::Month3,
    Text::Month4,
    Text::Month5,
    Text::Month6,
    Text::Month7,
    Text::Month8,
    Text::Month9,
    Text::Month10,
    Text::Month11,
    Text::Month12,
];

/// The days' messages, Monday first.
const WEEKDAYS: [Text; 7] = [
    Text::WeekdayMonday,
    Text::WeekdayTuesday,
    Text::WeekdayWednesday,
    Text::WeekdayThursday,
    Text::WeekdayFriday,
    Text::WeekdaySaturday,
    Text::WeekdaySunday,
];

/// The days' short messages, Monday first.
const WEEKDAYS_SHORT: [Text; 7] = [
    Text::WeekdayShortMonday,
    Text::WeekdayShortTuesday,
    Text::WeekdayShortWednesday,
    Text::WeekdayShortThursday,
    Text::WeekdayShortFriday,
    Text::WeekdayShortSaturday,
    Text::WeekdayShortSunday,
];
