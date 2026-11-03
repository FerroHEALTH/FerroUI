// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! A day read from and written to its text, its weekday and its steps, over
//! the proleptic Gregorian count of days.

use std::fmt;

use super::{DateOrder, Day, Month, NotADay, Weekday, YEARS, days_in};

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
pub(super) fn number(digits: &[u8]) -> Option<u32> {
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
