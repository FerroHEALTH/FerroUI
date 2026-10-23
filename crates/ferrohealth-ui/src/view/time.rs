// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The timestamp: a date-time drawn for reading, on one line, wherever a
//! console lists or states one (a table cell, a stacked card, a definition).
//!
//! An instant, a date-time with an offset such as
//! `2026-10-18T23:18:37.039437+00:00`, reads as `2026-10-18 23:18:37 UTC`:
//! moved to UTC and cut to the second. A date-time without an offset reads
//! as it stands, with a space for the `T`; a date reads as itself. The
//! element is `<time>` with the value exactly as the server sent it in its
//! `title`, and anything that is no date-time is drawn verbatim. No line
//! ever breaks inside the value (`fui-time`).

use leptos::prelude::*;

use crate::i18n::{Phrase, Text, Word, tr_with};

/// A date, `YYYY-MM-DD`, its parts checked against the calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Date {
    /// The year, 0 to 9999.
    year: u32,
    /// The month, 1 to 12.
    month: u32,
    /// The day of the month, 1 to the month's last.
    day: u32,
}

/// A time of day to the minute or the second, with the first three digits
/// of a fraction of the second when it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Clock {
    /// The minute of the day, 0 to 1439.
    minute: u32,
    /// The second, 0 to 59, when the value states one.
    second: Option<u32>,
    /// One to three digits of the fraction of the second.
    fraction: Option<String>,
}

/// A value read as a date, a date-time, or an instant.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Parsed {
    /// A date alone.
    Date(Date),
    /// A date and a time with no offset: a floating time, read as it stands.
    Local(Date, Clock),
    /// A date and a time with an offset, already moved to UTC.
    Instant(Date, Clock),
}

/// Whether `year` is a leap year in the proleptic Gregorian calendar.
const fn leap(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// The number of days in `month` of `year`.
const fn days_in(year: u32, month: u32) -> u32 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// The date `year`-`month`-`day` when it is one of the calendar's.
    fn new(year: u32, month: u32, day: u32) -> Option<Self> {
        ((1..=12).contains(&month) && (1..=days_in(year, month)).contains(&day)).then_some(Self {
            year,
            month,
            day,
        })
    }

    /// The day before, `None` before the year 0.
    fn previous(self) -> Option<Self> {
        if self.day > 1 {
            return Some(Self {
                day: self.day - 1,
                ..self
            });
        }
        if self.month > 1 {
            let month = self.month - 1;
            return Some(Self {
                month,
                day: days_in(self.year, month),
                ..self
            });
        }
        let year = self.year.checked_sub(1)?;
        Some(Self {
            year,
            month: 12,
            day: 31,
        })
    }

    /// The day after, `None` after the year 9999.
    fn next(self) -> Option<Self> {
        if self.day < days_in(self.year, self.month) {
            return Some(Self {
                day: self.day + 1,
                ..self
            });
        }
        if self.month < 12 {
            return Some(Self {
                month: self.month + 1,
                day: 1,
                ..self
            });
        }
        let year = self.year + 1;
        (year <= 9999).then_some(Self {
            year,
            month: 1,
            day: 1,
        })
    }
}

/// The bytes of a value read from the front.
#[derive(Debug)]
struct Cursor<'a> {
    /// What is left to read.
    rest: &'a [u8],
}

impl Cursor<'_> {
    /// Reads exactly `count` ASCII digits as a number.
    fn digits(&mut self, count: usize) -> Option<u32> {
        let (head, rest) = self.rest.split_at_checked(count)?;
        let mut number = 0_u32;
        for byte in head {
            if !byte.is_ascii_digit() {
                return None;
            }
            number = number * 10 + u32::from(byte - b'0');
        }
        self.rest = rest;
        Some(number)
    }

    /// Reads `byte` when it comes next.
    fn eat(&mut self, byte: u8) -> bool {
        match self.rest.split_first() {
            Some((first, rest)) if *first == byte => {
                self.rest = rest;
                true
            }
            _ => false,
        }
    }

    /// Reads one of `bytes` when it comes next.
    fn eat_any(&mut self, bytes: &[u8]) -> Option<u8> {
        let (first, rest) = self.rest.split_first()?;
        bytes.contains(first).then(|| {
            self.rest = rest;
            *first
        })
    }

    /// Reads a run of ASCII digits, at least one, keeping the first three.
    fn fraction(&mut self) -> Option<String> {
        let count = self.rest.iter().take_while(|b| b.is_ascii_digit()).count();
        let (head, rest) = self.rest.split_at_checked(count)?;
        if head.is_empty() {
            return None;
        }
        self.rest = rest;
        Some(head.iter().take(3).map(|b| char::from(*b)).collect())
    }

    /// Whether everything has been read.
    const fn done(&self) -> bool {
        self.rest.is_empty()
    }
}

/// Reads `YYYY-MM-DD`.
fn date(cursor: &mut Cursor<'_>) -> Option<Date> {
    let year = cursor.digits(4)?;
    cursor.eat(b'-').then_some(())?;
    let month = cursor.digits(2)?;
    cursor.eat(b'-').then_some(())?;
    let day = cursor.digits(2)?;
    Date::new(year, month, day)
}

/// Reads `HH:MM[:SS[.F…]]`.
fn clock(cursor: &mut Cursor<'_>) -> Option<Clock> {
    let hour = cursor.digits(2).filter(|hour| *hour < 24)?;
    cursor.eat(b':').then_some(())?;
    let minute = cursor.digits(2).filter(|minute| *minute < 60)?;
    let (second, fraction) = if cursor.eat(b':') {
        let second = cursor.digits(2).filter(|second| *second < 60)?;
        let fraction = if cursor.eat_any(b".,").is_some() {
            Some(cursor.fraction()?)
        } else {
            None
        };
        (Some(second), fraction)
    } else {
        (None, None)
    };
    Some(Clock {
        minute: hour * 60 + minute,
        second,
        fraction,
    })
}

/// Reads `Z` or `±HH:MM` (`±HHMM`) as the minutes the offset adds to UTC.
fn offset(cursor: &mut Cursor<'_>) -> Option<i64> {
    if cursor.eat_any(b"Zz").is_some() {
        return Some(0);
    }
    let sign = match cursor.eat_any(b"+-")? {
        b'-' => -1,
        _ => 1,
    };
    let hours = cursor.digits(2).filter(|hours| *hours < 24)?;
    cursor.eat(b':');
    let minutes = cursor.digits(2).filter(|minutes| *minutes < 60)?;
    Some(sign * i64::from(hours * 60 + minutes))
}

/// Moves `date` and `clock`, which are `offset` minutes ahead of UTC, to
/// UTC, `None` when that leaves the years 0 to 9999.
fn to_utc(date: Date, clock: Clock, offset: i64) -> Option<(Date, Clock)> {
    let minute = i64::from(clock.minute) - offset;
    let (date, minute) = if minute < 0 {
        (date.previous()?, minute + 1440)
    } else if minute >= 1440 {
        (date.next()?, minute - 1440)
    } else {
        (date, minute)
    };
    let minute = u32::try_from(minute).ok()?;
    Some((date, Clock { minute, ..clock }))
}

impl Parsed {
    /// Reads `raw` as a date, a date-time or an instant, as HTML's valid
    /// date and time strings and RFC 3339 write them; `None` for anything
    /// else.
    fn parse(raw: &str) -> Option<Self> {
        let mut cursor = Cursor {
            rest: raw.trim().as_bytes(),
        };
        let date = date(&mut cursor)?;
        if cursor.done() {
            return Some(Self::Date(date));
        }
        cursor.eat_any(b"Tt ")?;
        let clock = clock(&mut cursor)?;
        if cursor.done() {
            return Some(Self::Local(date, clock));
        }
        let offset = offset(&mut cursor)?;
        cursor.done().then_some(())?;
        let (date, clock) = to_utc(date, clock, offset)?;
        Some(Self::Instant(date, clock))
    }

    /// The form a person reads in `words`: `2026-10-18 23:18:37 UTC` for
    /// an instant.
    fn reading(&self, words: &TimeWords) -> Word {
        match self {
            Self::Date(date) => Word::new(day(*date)),
            Self::Local(date, clock) => Word::new(format!("{} {}", day(*date), time(clock, false))),
            Self::Instant(date, clock) => words
                .utc
                .fill(&[("date", &day(*date)), ("time", &time(clock, false))]),
        }
    }

    /// The machine form for the `datetime` attribute: a valid date string,
    /// local date and time string, or global date and time string in UTC.
    fn machine(&self) -> String {
        match self {
            Self::Date(date) => day(*date),
            Self::Local(date, clock) => format!("{}T{}", day(*date), time(clock, true)),
            Self::Instant(date, clock) => format!("{}T{}Z", day(*date), time(clock, true)),
        }
    }
}

/// `YYYY-MM-DD`.
fn day(date: Date) -> String {
    format!("{:04}-{:02}-{:02}", date.year, date.month, date.day)
}

/// `HH:MM[:SS]`, with the fraction when `fraction` asks for it.
fn time(clock: &Clock, fraction: bool) -> String {
    let (hour, minute) = (clock.minute.div_euclid(60), clock.minute.rem_euclid(60));
    match (clock.second, clock.fraction.as_deref().filter(|_| fraction)) {
        (None, _) => format!("{hour:02}:{minute:02}"),
        (Some(second), None) => format!("{hour:02}:{minute:02}:{second:02}"),
        (Some(second), Some(digits)) => format!("{hour:02}:{minute:02}:{second:02}.{digits}"),
    }
}

/// The words of a timestamp, resolved on the server in the page's locale,
/// so an island that draws one carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TimeWords {
    /// An instant in UTC, around the open `date` and `time`.
    pub utc: Phrase,
}

impl TimeWords {
    /// The timestamp's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            utc: Phrase::new(tr_with(
                Text::TimestampUtc,
                &[
                    ("date", Phrase::open("date")),
                    ("time", Phrase::open("time")),
                ],
            )),
        }
    }
}

/// The moment `raw` names, as a key that orders moments in time: an
/// instant moved to UTC, a date-time with no offset as it stands and a
/// date at its start, to the millisecond; `None` for anything that is no
/// date-time.
///
/// Two instants of different offsets order by when they happened
/// (`10:00+02:00` before `09:30Z`), which their text does not.
#[must_use]
pub(crate) fn moment(raw: &str) -> Option<(u32, u32, u32, u32, u32, u32)> {
    let (date, clock) = match Parsed::parse(raw)? {
        Parsed::Date(date) => (date, None),
        Parsed::Local(date, clock) | Parsed::Instant(date, clock) => (date, Some(clock)),
    };
    let (minute, second, millis) = clock.map_or((0, 0, 0), |clock| {
        let millis = clock.fraction.as_deref().map_or(0, |digits| {
            digits
                .bytes()
                .chain(std::iter::repeat(b'0'))
                .take(3)
                .fold(0, |sum, digit| {
                    sum * 10 + u32::from(digit.saturating_sub(b'0'))
                })
        });
        (clock.minute, clock.second.unwrap_or(0), millis)
    });
    Some((date.year, date.month, date.day, minute, second, millis))
}

/// The reading form of `raw`: `2026-10-18 23:18:37 UTC` for the instant
/// `2026-10-18T23:18:37.039437+00:00`, a date-time with no offset as it
/// stands, and anything that is no date-time verbatim.
#[must_use]
pub fn reading(raw: &str) -> String {
    reading_in(raw, &TimeWords::of_page())
}

/// [`reading`] in `words`.
#[must_use]
pub fn reading_in(raw: &str, words: &TimeWords) -> String {
    Parsed::parse(raw).map_or_else(|| raw.to_owned(), |parsed| parsed.reading(words).into())
}

/// Returns `raw` drawn for reading on one line, never broken.
///
/// A date-time is a `<time>` element whose text is the [`reading`] form,
/// whose `datetime` is the same moment in HTML's machine form, and whose
/// `title` is `raw` exactly as it came; anything else is `raw` verbatim.
#[must_use]
pub fn timestamp(raw: &str) -> impl IntoView + use<> {
    timestamp_in(raw, &TimeWords::of_page())
}

/// [`timestamp`] in `words`, for an island.
#[must_use]
pub fn timestamp_in(raw: &str, words: &TimeWords) -> impl IntoView + use<> {
    // NOTE: HTML Living Standard §4.5.14 and §2.3.5 allow at most three digits of a second's
    // fraction in `datetime`, so the attribute carries the UTC form and the title the raw value.
    match Parsed::parse(raw) {
        Some(parsed) => view! {
            <time class="fui-time" datetime=parsed.machine() title=raw.to_owned()>
                {parsed.reading(words)}
            </time>
        }
        .into_any(),
        None => view! { <span class="fui-time">{raw.to_owned()}</span> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Parsed, reading};

    /// The machine form of `raw`, when it is a date-time.
    fn machine(raw: &str) -> Option<String> {
        Parsed::parse(raw).map(|parsed| parsed.machine())
    }

    #[test]
    fn an_instant_reads_in_utc_cut_to_the_second() {
        assert_eq!(
            reading("2026-10-18T23:18:37.039437+00:00"),
            "2026-10-18 23:18:37 UTC"
        );
        assert_eq!(reading("2026-10-18T23:18:37Z"), "2026-10-18 23:18:37 UTC");
    }

    #[test]
    fn an_offset_moves_the_instant_across_a_day_month_and_year() {
        assert_eq!(
            reading("2026-10-19T01:18:37+02:00"),
            "2026-10-18 23:18:37 UTC"
        );
        assert_eq!(
            reading("2026-12-31T22:30:00-02:00"),
            "2027-01-01 00:30:00 UTC"
        );
        assert_eq!(
            reading("2024-03-01T00:15:00+0100"),
            "2024-02-29 23:15:00 UTC"
        );
        assert_eq!(
            reading("2023-03-01T00:15:00+01:00"),
            "2023-02-28 23:15:00 UTC"
        );
    }

    #[test]
    fn a_date_time_without_an_offset_reads_as_it_stands() {
        assert_eq!(reading("2026-10-18T23:18:37.5"), "2026-10-18 23:18:37");
        assert_eq!(reading("2026-10-18T23:18"), "2026-10-18 23:18");
        assert_eq!(reading("2026-10-18"), "2026-10-18");
    }

    #[test]
    fn the_machine_form_is_a_valid_html_date_time() {
        assert_eq!(
            machine("2026-10-18T23:18:37.039437+00:00").as_deref(),
            Some("2026-10-18T23:18:37.039Z")
        );
        assert_eq!(
            machine("2026-10-19 01:18+02:00").as_deref(),
            Some("2026-10-18T23:18Z")
        );
        assert_eq!(
            machine("2026-10-18T23:18:37").as_deref(),
            Some("2026-10-18T23:18:37")
        );
    }

    #[test]
    fn anything_that_is_no_date_time_is_verbatim() {
        for raw in [
            "",
            "yesterday",
            "2026-13-01",
            "2026-02-30",
            "2026-10-18T24:00:00Z",
            "2026-10-18T23:18:37+25:00",
            "2026-10-18T23:18:37Zjunk",
            "0000-01-01T00:30:00+01:00",
            "9999-12-31T23:30:00-01:00",
            "20261018T231837Z",
            "2026-10-18T23:18:37.",
            "2026-10-18\u{e9}",
        ] {
            assert_eq!(reading(raw), raw, "{raw:?} is no date-time");
            assert_eq!(machine(raw), None, "{raw:?} is no date-time");
        }
    }
}
