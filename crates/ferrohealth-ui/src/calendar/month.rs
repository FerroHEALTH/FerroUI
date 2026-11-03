// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! A month's whole weeks, laid out from the first day of the week chosen.

use super::{Day, Month, Weekday, YEARS, days_in};

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
