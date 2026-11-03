// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! A time of day and a moment, a day with an optional time.

use std::fmt;

use super::day::number;
use super::{Clock, DateOrder, Day, Moment, NotADay};

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

impl Moment {
    /// Reads `text` as a field reads it: `YYYY-MM-DD`, or with `with_time`
    /// `YYYY-MM-DD HH:MM` (a `T` in place of the space too), its date in
    /// `order` ([`Day::parse_in`]; [`DateOrder::Ymd`] for the form above).
    ///
    /// # Errors
    ///
    /// [`NotADay`] says why the text is no such value.
    pub fn parse(text: &str, with_time: bool, order: DateOrder) -> Result<Self, NotADay> {
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
