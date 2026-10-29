// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The commit-confirm countdown (F2): a change applied on trial runs until
//! an operator confirms it, and unless someone does by its deadline, the
//! server rolls it back.
//!
//! The countdown is a ring that empties toward the deadline and the time
//! left in words beside it, with the clock time it ends. The console
//! supplies the seconds left as it draws the page and the deadline in its
//! words, so the server draws the time the browser starts from. The
//! `countdown` module counts down from there by the time elapsed since the
//! page loaded ([`elapsed`]): the page's monotonic
//! clock, so a system clock set back never adds time, or the wall clock
//! when it has run further, since the monotonic clock may stand still while
//! the machine sleeps, and a countdown must never show time that has gone. The time is a `timer` that is not live; a polite live region
//! says the time left at [`ANNOUNCE_AT`] and that the time is up (WAI-ARIA
//! 1.2, `timer` and `status`). The ring is decorative and its sweep is a
//! transition, which reduced motion stops. No specification governs the
//! component: our own design.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr, tr_with};

/// The seconds left at which the live region says how many remain.
pub const ANNOUNCE_AT: [u32; 3] = [60, 30, 10];

/// The ring's circumference, for a radius of 13 on a 32 grid.
pub const RING: f64 = 81.68;

/// The countdown's words, resolved on the server in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CountdownWords {
    /// The line under the time left, such as "left, until 10:14:42".
    pub left: Word,
    /// The line once the time is up, such as "ended at 10:14:42".
    pub ended: Word,
    /// What the live region says at each of [`ANNOUNCE_AT`], in order.
    pub said: Vec<Word>,
    /// What the live region says once the time is up.
    pub up: Word,
}

impl CountdownWords {
    /// The words of a countdown ending at `until` (the deadline as the
    /// console writes a time, such as "10:14:42"), in `locale`.
    #[must_use]
    pub fn of(locale: &Locale, until: &str) -> Self {
        Self {
            left: tr_with(locale, Text::CountdownLeft, &[("time", until.into())]),
            ended: tr_with(locale, Text::CountdownEnded, &[("time", until.into())]),
            said: ANNOUNCE_AT
                .iter()
                .map(|&count| tr_with(locale, Text::CountdownSaid, &[("count", count.into())]))
                .collect(),
            up: tr(locale, Text::CountdownUp),
        }
    }
}

/// A countdown to a deadline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Countdown {
    /// The countdown's id, unique on the page.
    pub id: String,
    /// The seconds left as the server draws the page: the deadline less
    /// the console's time now, never below zero.
    pub left: u32,
    /// The trial's whole length in seconds, which the ring is a share of.
    pub total: u32,
    /// Its words ([`CountdownWords::of`]).
    pub words: CountdownWords,
}

impl Countdown {
    /// The countdown `id` with `left` of `total` seconds left, ending at
    /// `until` in the console's words, its words in `locale`.
    #[must_use]
    pub fn new(locale: &Locale, id: impl Into<String>, left: u32, total: u32, until: &str) -> Self {
        Self {
            id: id.into(),
            left,
            total,
            words: CountdownWords::of(locale, until),
        }
    }
}

/// Returns `seconds` as a clock reads: minutes and two-digit seconds, with
/// hours before them from an hour up ("1:42", "0:09", "1:02:03").
#[must_use]
pub fn clock(seconds: u32) -> String {
    let (hours, rest) = (seconds.div_euclid(3600), seconds.rem_euclid(3600));
    let (minutes, seconds) = (rest.div_euclid(60), rest.rem_euclid(60));
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// The ring's dash offset with `left` of `total` seconds remaining: none
/// when full, the whole circumference when empty.
#[must_use]
pub fn ring_offset(left: u32, total: u32) -> String {
    let share = f64::from(left.min(total)) / f64::from(total.max(1));
    format!("{:.2}", RING * (1.0 - share))
}

/// The set point of [`ANNOUNCE_AT`] the count crossed going from `before`
/// to `now` seconds left, the lowest when it crossed several; `None` when
/// it crossed none.
#[must_use]
pub fn crossed(before: u32, now: u32) -> Option<usize> {
    ANNOUNCE_AT
        .iter()
        .enumerate()
        .rev()
        .find(|&(_, &point)| now <= point && point < before)
        .map(|(index, _)| index)
}

/// The whole seconds in `millis` milliseconds, none for a negative span.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::as_conversions,
    reason = "the value is floored, at least zero and capped at u32::MAX before the cast"
)]
pub fn whole_seconds(millis: f64) -> u32 {
    let seconds = (millis / 1000.0).floor();
    if seconds.is_nan() || seconds <= 0.0 {
        0
    } else if seconds >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        seconds as u32
    }
}

/// The milliseconds elapsed since the countdown started: the larger of what
/// the monotonic clock (`monotonic`) and the wall clock (`wall`) measured.
///
/// High Resolution Time leaves the monotonic clock's course while the system
/// sleeps to the user agent (§ 2.1, § 9.2), and some stop it; the wall clock
/// runs on through a sleep. Taking the larger never shows time that has
/// passed, and a wall clock set back adds none.
#[must_use]
pub fn elapsed(monotonic: f64, wall: f64) -> f64 {
    let monotonic = if monotonic.is_finite() {
        monotonic
    } else {
        0.0
    };
    let wall = if wall.is_finite() { wall } else { 0.0 };
    monotonic.max(wall)
}

#[derive(Template)]
#[template(
    source = r#"<span id="{{ id }}" data-fui="countdown" data-left="{{ left }}" data-total="{{ total }}" data-words="{{ words }}" class="fui-countdown"><svg viewBox="0 0 32 32" aria-hidden="true"><circle cx="16" cy="16" r="13" class="fui-countdown-track"></circle><circle cx="16" cy="16" r="13" stroke-dasharray="{{ ring }}" stroke-dashoffset="{{ offset }}" class="fui-countdown-left"></circle></svg><span role="timer" aria-live="off" class="fui-countdown-words"><strong>{{ clock }}</strong><span>{{ line }}</span></span><span role="status" class="fui-sr-only"></span></span>"#,
    ext = "html"
)]
struct CountdownView {
    id: String,
    left: u32,
    total: u32,
    words: String,
    ring: f64,
    offset: String,
    clock: String,
    line: Word,
}

/// Returns the countdown `countdown` as the server draws it, which the
/// `countdown` module counts down by the second, stopping at zero.
#[must_use]
pub fn countdown(countdown: Countdown) -> Html {
    let Countdown {
        id,
        left,
        total,
        words,
    } = countdown;
    let total = total.max(left);
    let line = if left == 0 {
        words.ended.clone()
    } else {
        words.left.clone()
    };
    CountdownView {
        id,
        left,
        total,
        // NOTE: serde_json `to_string` fails only for a map with a key that is no
        // string, and the words hold none.
        words: serde_json::to_string(&words).unwrap_or_default(),
        ring: RING,
        offset: ring_offset(left, total),
        clock: clock(left),
        line,
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{ANNOUNCE_AT, clock, crossed, elapsed, ring_offset, whole_seconds};

    #[test]
    fn a_sleep_the_monotonic_clock_missed_still_counts_and_a_clock_set_back_adds_nothing() {
        assert_eq!(600_000.0, elapsed(2_000.0, 600_000.0), "ten minutes asleep");
        assert_eq!(
            2_000.0,
            elapsed(2_000.0, -3_600_000.0),
            "the wall clock set back an hour"
        );
        assert_eq!(2_000.0, elapsed(2_000.0, f64::NAN));
    }

    #[test]
    fn the_clock_reads_minutes_and_two_digit_seconds_and_hours_from_an_hour() {
        assert_eq!("1:42", clock(102));
        assert_eq!("0:09", clock(9));
        assert_eq!("0:00", clock(0));
        assert_eq!("59:59", clock(3599));
        assert_eq!("1:02:03", clock(3723));
    }

    #[test]
    fn the_ring_is_full_at_the_start_and_empty_at_the_end() {
        assert_eq!("0.00", ring_offset(120, 120));
        assert_eq!("81.68", ring_offset(0, 120));
        assert_eq!("40.84", ring_offset(60, 120));
        assert_eq!("0.00", ring_offset(200, 120), "never more than full");
        assert_eq!("81.68", ring_offset(0, 0), "a trial of no length is empty");
    }

    #[test]
    fn a_set_point_is_said_once_as_the_count_reaches_it() {
        assert_eq!(None, crossed(62, 61));
        assert_eq!(Some(0), crossed(61, 60));
        assert_eq!(None, crossed(60, 59), "said once");
        assert_eq!(Some(1), crossed(31, 30));
        assert_eq!(Some(2), crossed(11, 10));
        assert_eq!(None, crossed(10, 9));
        assert_eq!(
            Some(ANNOUNCE_AT.len() - 1),
            crossed(70, 5),
            "a tab that slept says the lowest point it passed"
        );
    }

    #[test]
    fn elapsed_milliseconds_are_whole_seconds_and_never_negative() {
        assert_eq!(0, whole_seconds(-5.0));
        assert_eq!(0, whole_seconds(999.9));
        assert_eq!(1, whole_seconds(1000.0));
        assert_eq!(102, whole_seconds(102_400.0));
        assert_eq!(0, whole_seconds(f64::NAN));
        assert_eq!(u32::MAX, whole_seconds(1e20));
    }
}
