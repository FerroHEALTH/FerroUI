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

impl Month {
    /// The month `month` of `year`, when both are in the calendar.
    #[must_use]
    pub fn new(year: u16, month: u8) -> Option<Self> {
        (YEARS.contains(&year) && (1..=12).contains(&month)).then_some(Self { year, month })
    }

    /// The year.
    #[must_use]
    pub const fn year(self) -> u16 {
        self.year
    }

    /// The month of the year, 1 to 12.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// The month `count` months on (or back, when negative), `None` past
    /// the calendar's ends.
    #[must_use]
    pub fn step(self, count: i32) -> Option<Self> {
        let place = i32::from(self.year) * 12 + i32::from(self.month) - 1 + count;
        let year = u16::try_from(place.div_euclid(12)).ok()?;
        let month = u8::try_from(place.rem_euclid(12) + 1).ok()?;
        Self::new(year, month)
    }

    /// The month's first day.
    #[must_use]
    pub const fn first(self) -> Day {
        Day {
            year: self.year,
            month: self.month,
            date: 1,
        }
    }

    /// The month's last day.
    #[must_use]
    pub const fn last(self) -> Day {
        Day {
            year: self.year,
            month: self.month,
            date: days_in(self.year, self.month),
        }
    }

    /// The month's weeks, each starting on `first`, from the week of its
    /// first day to the week of its last; the days before and after the
    /// month fill the first and last weeks.
    #[must_use]
    pub fn weeks(self, first: Weekday) -> Vec<[Day; 7]> {
        let last = self.last();
        let mut start = self.first().week_start(first);
        let mut weeks = Vec::with_capacity(6);
        while start <= last {
            weeks.push([0, 1, 2, 3, 4, 5, 6].map(|place| start.step(place).unwrap_or(start)));
            match start.step(7) {
                Some(next) => start = next,
                None => break,
            }
        }
        weeks
    }
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

/// The number of days from 1970-01-01 to the date `year`-`month`-`day`.
// NOTE: Howard Hinnant, "chrono-Compatible Low-Level Date Algorithms",
// `days_from_civil`: the year counted from March, so the leap day ends it.
#[expect(
    clippy::integer_division,
    reason = "the algorithm counts whole days, months and eras, each division truncating by design"
)]
fn days_from_civil(year: u16, month: u8, day: u8) -> i32 {
    let year = i32::from(year) - i32::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let month = i32::from(month);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i32::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The date `days` days after 1970-01-01, the inverse of
/// [`days_from_civil`].
#[expect(
    clippy::integer_division,
    reason = "the algorithm counts whole days, months and eras, each division truncating by design"
)]
fn civil_from_days(days: i32) -> Option<(u16, u8, u8)> {
    let days = days.checked_add(719_468)?;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_place = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_place + 2) / 5 + 1;
    let month = if month_place < 10 {
        month_place + 3
    } else {
        month_place - 9
    };
    let year = year_of_era + era * 400 + i32::from(month <= 2);
    Some((
        u16::try_from(year).ok()?,
        u8::try_from(month).ok()?,
        u8::try_from(day).ok()?,
    ))
}

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
        Self::parse(crate::i18n::tr(locale, crate::i18n::Text::DateOrder).text()).unwrap_or_default()
    }
}

impl Day {
    /// Reads `text` as a day written `YYYY-MM-DD`, or in `order` with its
    /// day and month of one or two digits and its year of four, split by
    /// `.`, `/` or `-`; surrounding space ignored.
    ///
    /// # Errors
    ///
    /// [`NotADay`] says why the text is no day.
    pub fn parse_in(text: &str, order: DateOrder) -> Result<Self, NotADay> {
        let iso = Self::parse(text);
        if order == DateOrder::Ymd || !matches!(iso, Err(NotADay::Format)) {
            return iso;
        }
        let parts: Vec<&str> = text.trim().split(['.', '/', '-']).collect();
        let [first, second, year] = parts.as_slice() else {
            return Err(NotADay::Format);
        };
        let (day, month) = if order == DateOrder::Dmy {
            (first, second)
        } else {
            (second, first)
        };
        let short = |part: &str| {
            (1..=2)
                .contains(&part.len())
                .then(|| number(part.as_bytes()))
                .flatten()
                .and_then(|value| u8::try_from(value).ok())
        };
        let (Some(day), Some(month)) = (short(day), short(month)) else {
            return Err(NotADay::Format);
        };
        let year = (year.len() == 4)
            .then(|| number(year.as_bytes()))
            .flatten()
            .and_then(|year| u16::try_from(year).ok())
            .filter(|year| YEARS.contains(year))
            .ok_or(NotADay::Format)?;
        if !(1..=12).contains(&month) {
            return Err(NotADay::Month { month });
        }
        Self::new(year, month, day).ok_or(NotADay::Day { year, month })
    }

    /// The day `year`-`month`-`day`, when it is one of the calendar's.
    #[must_use]
    pub fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        (YEARS.contains(&year) && (1..=days_in(year, month)).contains(&day)).then_some(Self {
            year,
            month,
            date: day,
        })
    }

    /// Reads `text` as a day written `YYYY-MM-DD`, surrounding space
    /// ignored.
    ///
    /// # Errors
    ///
    /// [`NotADay`] says why the text is no day: not written so, no such
    /// month, or no such day in that month.
    pub fn parse(text: &str) -> Result<Self, NotADay> {
        let bytes = text.trim().as_bytes();
        let [y1, y2, y3, y4, b'-', m1, m2, b'-', d1, d2] = *bytes else {
            return Err(NotADay::Format);
        };
        let year = number(&[y1, y2, y3, y4])
            .and_then(|year| u16::try_from(year).ok())
            .ok_or(NotADay::Format)?;
        let month = number(&[m1, m2])
            .and_then(|month| u8::try_from(month).ok())
            .ok_or(NotADay::Format)?;
        let day = number(&[d1, d2])
            .and_then(|day| u8::try_from(day).ok())
            .ok_or(NotADay::Format)?;
        if !YEARS.contains(&year) {
            return Err(NotADay::Format);
        }
        if !(1..=12).contains(&month) {
            return Err(NotADay::Month { month });
        }
        Self::new(year, month, day).ok_or(NotADay::Day { year, month })
    }

    /// The year.
    #[must_use]
    pub const fn year(self) -> u16 {
        self.year
    }

    /// The month of the year, 1 to 12.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// The day of the month.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.date
    }

    /// The month the day falls in.
    #[must_use]
    pub const fn month_of(self) -> Month {
        Month {
            year: self.year,
            month: self.month,
        }
    }

    /// The day's place counted from 1970-01-01.
    fn number(self) -> i32 {
        days_from_civil(self.year, self.month, self.date)
    }

    /// The day of the week it falls on.
    #[must_use]
    pub fn weekday(self) -> Weekday {
        // NOTE: 1970-01-01 was a Thursday, the ISO week's day 3.
        let index = (self.number() + 3).rem_euclid(7);
        Weekday::from_index(u8::try_from(index).unwrap_or(0))
    }

    /// The day `count` days on (or back, when negative), `None` past the
    /// calendar's ends.
    #[must_use]
    pub fn step(self, count: i32) -> Option<Self> {
        let (year, month, day) = civil_from_days(self.number().checked_add(count)?)?;
        Self::new(year, month, day)
    }

    /// The day `count` months on (or back), on the same day of the month or
    /// on the month's last day when it is shorter.
    #[must_use]
    pub fn step_months(self, count: i32) -> Option<Self> {
        let month = self.month_of().step(count)?;
        Self::new(
            month.year,
            month.month,
            self.date.min(days_in(month.year, month.month)),
        )
    }

    /// The number of days from `self` to `other`, both counted: 1 for the
    /// same day, and the same whichever comes first.
    #[must_use]
    pub fn days_through(self, other: Self) -> u32 {
        (other.number() - self.number()).unsigned_abs() + 1
    }

    /// The first day of the week holding the day, for a week that starts
    /// on `first`.
    #[must_use]
    pub fn week_start(self, first: Weekday) -> Self {
        let back = (7 + self.weekday().index() - first.index()) % 7;
        self.step(-i32::from(back)).unwrap_or(self)
    }
}

/// The value of `digits` as a decimal number, when each is an ASCII digit.
fn number(digits: &[u8]) -> Option<u32> {
    digits.iter().try_fold(0_u32, |value, digit| {
        digit
            .is_ascii_digit()
            .then(|| value * 10 + u32::from(digit - b'0'))
    })
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.date)
    }
}

impl From<Day> for String {
    fn from(day: Day) -> Self {
        day.to_string()
    }
}

impl TryFrom<String> for Day {
    type Error = NotADay;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::parse(&text)
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

impl Clock {
    /// The time `hour`:`minute`, when it is a time of day.
    #[must_use]
    pub fn new(hour: u8, minute: u8) -> Option<Self> {
        (hour < 24 && minute < 60).then_some(Self { hour, minute })
    }

    /// The hour, 0 to 23.
    #[must_use]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// The minute, 0 to 59.
    #[must_use]
    pub const fn minute(self) -> u8 {
        self.minute
    }

    /// Reads `text` as `HH:MM`.
    ///
    /// # Errors
    ///
    /// [`NotADay::Time`] when the text is no time of day written so.
    pub fn parse(text: &str) -> Result<Self, NotADay> {
        let [h1, h2, b':', m1, m2] = *text.trim().as_bytes() else {
            return Err(NotADay::Time);
        };
        let hour = number(&[h1, h2]).and_then(|hour| u8::try_from(hour).ok());
        let minute = number(&[m1, m2]).and_then(|minute| u8::try_from(minute).ok());
        hour.zip(minute)
            .and_then(|(hour, minute)| Self::new(hour, minute))
            .ok_or(NotADay::Time)
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// What a date field holds: a day, and its time when the field takes one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Moment {
    /// The day.
    pub day: Day,
    /// The time of day, for a field that takes one.
    pub time: Option<Clock>,
}

impl Moment {
    /// Reads `text` as a field reads it: `YYYY-MM-DD`, or with `with_time`
    /// `YYYY-MM-DD HH:MM` (a `T` in place of the space too).
    ///
    /// # Errors
    ///
    /// [`NotADay`] says why the text is no such value.
    pub fn parse(text: &str, with_time: bool) -> Result<Self, NotADay> {
        Self::parse_in(text, with_time, DateOrder::Ymd)
    }

    /// Reads `text` as [`Moment::parse`] does, its date also in `order`
    /// ([`Day::parse_in`]).
    ///
    /// # Errors
    ///
    /// [`NotADay`] says why the text is no such value.
    pub fn parse_in(text: &str, with_time: bool, order: DateOrder) -> Result<Self, NotADay> {
        let text = text.trim();
        if !with_time {
            return Day::parse_in(text, order).map(|day| Self { day, time: None });
        }
        let (date, time) = text
            .split_once([' ', 'T'])
            .ok_or_else(|| Day::parse_in(text, order).err().unwrap_or(NotADay::Format))?;
        let day = Day::parse_in(date, order)?;
        let time = Clock::parse(time)?;
        Ok(Self {
            day,
            time: Some(time),
        })
    }
}

impl fmt::Display for Moment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.time {
            Some(time) => write!(f, "{} {time}", self.day),
            None => write!(f, "{}", self.day),
        }
    }
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

impl Bounds {
    /// Whether `day` is allowed.
    #[must_use]
    pub fn allows(self, day: Day) -> bool {
        self.earliest.is_none_or(|earliest| day >= earliest)
            && self.latest.is_none_or(|latest| day <= latest)
    }

    /// Whether some day of `month` is allowed.
    #[must_use]
    pub fn reaches(self, month: Month) -> bool {
        self.earliest
            .is_none_or(|earliest| month.last() >= earliest)
            && self.latest.is_none_or(|latest| month.first() <= latest)
    }
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

impl Quick {
    /// The day the choice means when today is `today`.
    #[must_use]
    pub fn day(self, today: Day) -> Option<Day> {
        match self {
            Self::Days(count) => today.step(i32::from(count)),
            Self::Next(weekday) => {
                let ahead = (7 + weekday.index() - today.weekday().index()) % 7;
                today.step(i32::from(if ahead == 0 { 7 } else { ahead }))
            }
        }
    }
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

impl Preset {
    /// The first and last day of the preset when today is `today`.
    #[must_use]
    pub fn range(self, today: Day) -> Option<(Day, Day)> {
        match self {
            Self::Today => Some((today, today)),
            Self::LastDays(count) => {
                let back = i32::from(count.max(1)) - 1;
                Some((today.step(-back)?, today))
            }
            Self::ThisMonth => Some((today.month_of().first(), today)),
        }
    }
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
        let moment = Moment::parse_in("23.10.2026 14:30", true, DateOrder::Dmy).expect("a moment");
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
        let moment = Moment::parse("2026-10-14 08:30", true).expect("a moment");
        assert_eq!(Some(Clock::new(8, 30).expect("a time")), moment.time);
        assert_eq!("2026-10-14 08:30", moment.to_string());
        assert_eq!(Ok(moment), Moment::parse("2026-10-14T08:30", true));
        assert_eq!(Err(NotADay::Time), Moment::parse("2026-10-14 24:00", true));
        assert_eq!(Err(NotADay::Format), Moment::parse("2026-10-14", true));
        assert_eq!(
            Err(NotADay::Day {
                year: 2026,
                month: 10
            }),
            Moment::parse("2026-10-41", true)
        );
        assert_eq!(
            Err(NotADay::Format),
            Moment::parse("2026-10-14 08:30", false)
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
