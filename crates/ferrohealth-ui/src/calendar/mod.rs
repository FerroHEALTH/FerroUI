// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The calendar the date field and its picker count in: a day, a month, a
//! week that starts on the reader's first day, and a typed value read
//! year first.
//!
//! A day is a date of the proleptic Gregorian calendar from the year 1 to
//! the year 9999, written `YYYY-MM-DD` (ISO 8601-1:2019, 5.2.2.1, the
//! calendar date in its extended format), and a typed value is that date
//! with a time to the minute when the field takes one (`YYYY-MM-DD HH:MM`).
//! A field also takes a date typed in the order its page's language writes
//! one ([`DateOrder`], the catalog's `date-order`), such as `23.10.2026`.
//! A console's server reads a posted value with the same functions the
//! picker reads a typed one with, so both refuse a value for the same
//! reason. No specification governs the picker's arithmetic otherwise: our
//! own design.

mod clock;
mod day;
mod month;
mod range;

use std::fmt;

use serde::{Deserialize, Serialize};

/// A day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Weekday {
    /// Monday, the first day of the ISO 8601 week.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl Weekday {
    /// Every day of the week, from Monday.
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    /// The day's place in the ISO week, Monday 0 to Sunday 6.
    #[must_use]
    pub const fn index(self) -> u8 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }

    /// The day `index` places in the ISO week, counted round from Monday.
    #[must_use]
    pub const fn from_index(index: u8) -> Self {
        match index % 7 {
            0 => Self::Monday,
            1 => Self::Tuesday,
            2 => Self::Wednesday,
            3 => Self::Thursday,
            4 => Self::Friday,
            5 => Self::Saturday,
            _ => Self::Sunday,
        }
    }

    /// The seven days of a week that starts on `self`.
    #[must_use]
    pub fn week(self) -> [Self; 7] {
        let start = self.index();
        [0, 1, 2, 3, 4, 5, 6].map(|step| Self::from_index(start + step))
    }
}

/// Whether `year` is a leap year in the proleptic Gregorian calendar.
#[must_use]
pub const fn leap(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// The number of days in `month` of `year`, 0 for a month that is not one.
#[must_use]
pub const fn days_in(year: u16, month: u8) -> u8 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}

/// The first and the last year a day may fall in.
const YEARS: std::ops::RangeInclusive<u16> = 1..=9999;

/// A month of a year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Month {
    /// The year, 1 to 9999.
    year: u16,
    /// The month, 1 to 12.
    month: u8,
}

/// A day of the proleptic Gregorian calendar, from 0001-01-01 to
/// 9999-12-31. It crosses to the browser as its `YYYY-MM-DD` text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Day {
    /// The year, 1 to 9999.
    year: u16,
    /// The month, 1 to 12.
    month: u8,
    /// The day of the month, 1 to the month's last.
    date: u8,
}

/// Why a typed text is no day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotADay {
    /// The text is not written `YYYY-MM-DD` (or `YYYY-MM-DD HH:MM` where a
    /// time is asked for).
    Format,
    /// The month is not one of the twelve.
    Month {
        /// The month as typed.
        month: u8,
    },
    /// The month has fewer days than the day typed.
    Day {
        /// The year as typed, which decides February.
        year: u16,
        /// The month as typed.
        month: u8,
    },
    /// The time is not a time of day.
    Time,
}

impl fmt::Display for NotADay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format => f.write_str("not written YYYY-MM-DD"),
            Self::Month { month } => write!(f, "there is no month {month}"),
            Self::Day { year, month } => {
                write!(
                    f,
                    "month {month} of {year} has {} days",
                    days_in(*year, *month)
                )
            }
            Self::Time => f.write_str("not a time of day"),
        }
    }
}

impl std::error::Error for NotADay {}

/// The order a language writes a date's day, month and year in, which its
/// catalog names as `date-order`: `ymd`, `dmy` or `mdy`.
///
/// A field always takes `YYYY-MM-DD`; a language that writes the day or the
/// month first adds its own order, its parts split by `.`, `/` or `-`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DateOrder {
    /// Year, month, day: ISO 8601 alone.
    #[default]
    Ymd,
    /// Day, month, year, such as `23.10.2026`.
    Dmy,
    /// Month, day, year, such as `10/23/2026`.
    Mdy,
}

impl DateOrder {
    /// The order a catalog names by `key` (`ymd`, `dmy`, `mdy`).
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        match key.trim() {
            "ymd" => Some(Self::Ymd),
            "dmy" => Some(Self::Dmy),
            "mdy" => Some(Self::Mdy),
            _ => None,
        }
    }

    /// The order of a page in `locale`, from its catalog's `date-order`.
    #[must_use]
    pub fn of(locale: &crate::i18n::Locale) -> Self {
        // NOTE: no specification governs this: our own design; a catalog that names
        // no order the library knows takes ISO 8601 alone.
        Self::parse(crate::i18n::tr(locale, crate::i18n::Text::DateOrder).text())
            .unwrap_or_default()
    }
}

/// A time of day to the minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Clock {
    /// The hour, 0 to 23.
    hour: u8,
    /// The minute, 0 to 59.
    minute: u8,
}

/// What a date field holds: a day, and its time when the field takes one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Moment {
    /// The day.
    pub day: Day,
    /// The time of day, for a field that takes one.
    pub time: Option<Clock>,
}

/// The days a field allows: from its earliest to its latest, each when it
/// has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Bounds {
    /// The first day allowed, or none.
    pub earliest: Option<Day>,
    /// The last day allowed, or none: today for a field that forbids the
    /// future.
    pub latest: Option<Day>,
}

/// A choice that fills a single date field from today: tomorrow, in three
/// days, next Monday.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quick {
    /// The day `n` days from today (today itself for 0).
    Days(i16),
    /// The next day of the week after today, a week on when today is that
    /// day.
    Next(Weekday),
}

/// A preset of a range picker, which fills both ends from today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    /// Today alone.
    Today,
    /// The last `n` days: today and the `n - 1` days before it, so the range
    /// holds exactly `n` days.
    LastDays(u16),
    /// The month so far: its first day to today.
    ThisMonth,
}

#[cfg(test)]
mod tests {
    use super::{
        Bounds, Clock, DateOrder, Day, Moment, Month, NotADay, Preset, Quick, Weekday, days_in,
    };

    fn day(text: &str) -> Day {
        Day::parse(text).expect("a test day")
    }

    #[test]
    fn a_date_is_read_in_the_order_of_the_pages_language_and_always_as_iso() {
        assert_eq!(
            Ok(day("2026-10-23")),
            Day::parse_in("23.10.2026", DateOrder::Dmy)
        );
        assert_eq!(
            Ok(day("2026-03-04")),
            Day::parse_in("4/3/2026", DateOrder::Dmy)
        );
        assert_eq!(
            Ok(day("2026-10-23")),
            Day::parse_in("10/23/2026", DateOrder::Mdy)
        );
        assert_eq!(
            Ok(day("2026-10-23")),
            Day::parse_in("2026-10-23", DateOrder::Dmy)
        );
        assert_eq!(
            Err(NotADay::Format),
            Day::parse_in("23.10.2026", DateOrder::Ymd),
            "ISO 8601 alone where the language writes the year first"
        );
        assert_eq!(
            Err(NotADay::Month { month: 13 }),
            Day::parse_in("01.13.2026", DateOrder::Dmy)
        );
        assert_eq!(
            Err(NotADay::Day {
                year: 2026,
                month: 2
            }),
            Day::parse_in("30.02.2026", DateOrder::Dmy)
        );
        assert_eq!(
            Err(NotADay::Format),
            Day::parse_in("23.10.26", DateOrder::Dmy)
        );
        assert_eq!(
            Err(NotADay::Format),
            Day::parse_in("123.10.2026", DateOrder::Dmy)
        );
        assert_eq!(Some(DateOrder::Dmy), DateOrder::parse("dmy"));
        assert_eq!(None, DateOrder::parse("day first"));
        let moment = Moment::parse("23.10.2026 14:30", true, DateOrder::Dmy).expect("a moment");
        assert_eq!("2026-10-23 14:30", moment.to_string());
    }

    #[test]
    fn a_day_reads_year_first_and_says_why_it_is_not_one() {
        assert_eq!(Ok(day("2026-10-21")), Day::parse(" 2026-10-21 "));
        assert_eq!(Err(NotADay::Format), Day::parse("21-10-2026"));
        assert_eq!(Err(NotADay::Format), Day::parse("2026-1-21"));
        assert_eq!(Err(NotADay::Format), Day::parse("２０２６-10-21"));
        assert_eq!(Err(NotADay::Month { month: 13 }), Day::parse("2026-13-01"));
        assert_eq!(
            Err(NotADay::Day {
                year: 2026,
                month: 10
            }),
            Day::parse("2026-10-41")
        );
        assert_eq!(
            Err(NotADay::Day {
                year: 2026,
                month: 2
            }),
            Day::parse("2026-02-29")
        );
        assert!(Day::parse("2028-02-29").is_ok(), "a leap day");
        assert_eq!(Err(NotADay::Format), Day::parse("0000-01-01"));
    }

    #[test]
    fn a_day_knows_its_weekday_across_eras() {
        assert_eq!(Weekday::Wednesday, day("2026-10-21").weekday());
        assert_eq!(Weekday::Thursday, day("1970-01-01").weekday());
        assert_eq!(Weekday::Monday, day("0001-01-01").weekday());
        assert_eq!(Weekday::Friday, day("9999-12-31").weekday());
        assert_eq!(Weekday::Tuesday, day("2000-02-29").weekday());
    }

    #[test]
    fn a_step_crosses_months_years_and_stops_at_the_ends() {
        assert_eq!(Some(day("2026-11-01")), day("2026-10-31").step(1));
        assert_eq!(Some(day("2025-12-31")), day("2026-01-01").step(-1));
        assert_eq!(Some(day("2028-02-29")), day("2028-02-28").step(1));
        assert_eq!(None, day("9999-12-31").step(1));
        assert_eq!(None, day("0001-01-01").step(-1));
        assert_eq!(Some(day("2026-02-28")), day("2026-01-31").step_months(1));
        assert_eq!(Some(day("2025-10-21")), day("2026-10-21").step_months(-12));
    }

    #[test]
    fn a_month_lays_out_whole_weeks_from_the_first_day_chosen() {
        let october = Month::new(2026, 10).expect("a month");
        let monday = october.weeks(Weekday::Monday);
        assert_eq!(5, monday.len());
        assert_eq!(day("2026-09-28"), monday[0][0]);
        assert_eq!(day("2026-11-01"), monday[4][6]);
        let sunday = october.weeks(Weekday::Sunday);
        assert_eq!(day("2026-09-27"), sunday[0][0]);
        assert_eq!(Weekday::Sunday, sunday[0][0].weekday());
        assert!(
            sunday
                .iter()
                .flatten()
                .any(|each| *each == day("2026-10-31"))
        );
        let february = Month::new(2021, 2).expect("a month").weeks(Weekday::Monday);
        assert_eq!(4, february.len(), "February 2021 starts on a Monday");
        let six = Month::new(2026, 8).expect("a month").weeks(Weekday::Monday);
        assert_eq!(6, six.len(), "August 2026 spans six weeks");
        assert_eq!(day("2026-08-31"), six[5][0]);
    }

    #[test]
    fn a_week_starts_on_the_first_day_chosen() {
        assert_eq!(
            [Weekday::Sunday, Weekday::Monday],
            [Weekday::Sunday.week()[0], Weekday::Sunday.week()[1]]
        );
        assert_eq!(
            day("2026-10-19"),
            day("2026-10-21").week_start(Weekday::Monday)
        );
        assert_eq!(
            day("2026-10-18"),
            day("2026-10-21").week_start(Weekday::Sunday)
        );
    }

    #[test]
    fn a_moment_takes_a_time_only_where_the_field_asks_for_one() {
        let moment = Moment::parse("2026-10-14 08:30", true, DateOrder::Ymd).expect("a moment");
        assert_eq!(Some(Clock::new(8, 30).expect("a time")), moment.time);
        assert_eq!("2026-10-14 08:30", moment.to_string());
        assert_eq!(
            Ok(moment),
            Moment::parse("2026-10-14T08:30", true, DateOrder::Ymd)
        );
        assert_eq!(
            Err(NotADay::Time),
            Moment::parse("2026-10-14 24:00", true, DateOrder::Ymd)
        );
        assert_eq!(
            Err(NotADay::Format),
            Moment::parse("2026-10-14", true, DateOrder::Ymd)
        );
        assert_eq!(
            Err(NotADay::Day {
                year: 2026,
                month: 10
            }),
            Moment::parse("2026-10-41", true, DateOrder::Ymd)
        );
        assert_eq!(
            Err(NotADay::Format),
            Moment::parse("2026-10-14 08:30", false, DateOrder::Ymd)
        );
    }

    #[test]
    fn bounds_allow_days_and_reach_months_inclusively() {
        let today = day("2026-10-21");
        let past = Bounds {
            earliest: None,
            latest: Some(today),
        };
        assert!(past.allows(today));
        assert!(!past.allows(day("2026-10-22")));
        assert!(past.reaches(today.month_of()));
        assert!(!past.reaches(Month::new(2026, 11).expect("a month")));
        let future = Bounds {
            earliest: today.step(1),
            latest: None,
        };
        assert!(!future.allows(today));
        assert!(future.reaches(today.month_of()));
        assert!(!future.reaches(Month::new(2026, 9).expect("a month")));
    }

    #[test]
    fn a_preset_holds_the_days_its_words_name() {
        let today = day("2026-10-21");
        let (from, to) = Preset::LastDays(7).range(today).expect("a range");
        assert_eq!((day("2026-10-15"), today), (from, to));
        assert_eq!(7, from.days_through(to), "Last 7 days is 7 days");
        let (from, to) = Preset::LastDays(30).range(today).expect("a range");
        assert_eq!(30, from.days_through(to));
        assert_eq!(Some((today, today)), Preset::Today.range(today));
        assert_eq!(
            Some((day("2026-10-01"), today)),
            Preset::ThisMonth.range(today)
        );
    }

    #[test]
    fn a_quick_choice_counts_from_today() {
        let today = day("2026-10-21");
        assert_eq!(Some(day("2026-10-22")), Quick::Days(1).day(today));
        assert_eq!(Some(day("2026-10-28")), Quick::Days(7).day(today));
        assert_eq!(
            Some(day("2026-10-26")),
            Quick::Next(Weekday::Monday).day(today)
        );
        assert_eq!(
            Some(day("2026-10-28")),
            Quick::Next(Weekday::Wednesday).day(today),
            "a week on when today is that day"
        );
    }

    #[test]
    fn a_day_crosses_to_the_browser_as_its_text() {
        let text = serde_json::to_string(&day("2026-10-21")).expect("serialized");
        assert_eq!("\"2026-10-21\"", text);
        assert_eq!(
            day("2026-10-21"),
            serde_json::from_str::<Day>(&text).expect("read")
        );
        assert!(serde_json::from_str::<Day>("\"2026-10-41\"").is_err());
        assert_eq!(31, days_in(2026, 10));
        assert_eq!(0, days_in(2026, 13));
    }
}
