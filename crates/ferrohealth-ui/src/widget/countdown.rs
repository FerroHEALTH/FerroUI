// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The commit-confirm countdown (F2): a change applied on trial runs until
//! an operator confirms it, and unless someone does by its deadline, the
//! server rolls it back.
//!
//! The countdown is a ring that empties toward the deadline and the time
//! left in words beside it, with the clock time it ends. The console
//! supplies the seconds left as it draws the page and the deadline in its
//! words, so the server draws the same time the browser starts from and
//! hydration matches. In the browser the widget counts down from there by
//! the time elapsed since it hydrated, read from the page's monotonic
//! clock, so a skewed system clock or a throttled background tab never
//! shifts it. The time is a `timer` that is not live; a polite live region
//! says the time left at [`ANNOUNCE_AT`] and that the time is up (WAI-ARIA
//! 1.2, `timer` and `status`). The ring is decorative and its sweep is a
//! transition, which reduced motion stops. No specification governs the
//! component: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr, tr_with};

/// The seconds left at which the live region says how many remain.
pub const ANNOUNCE_AT: [u32; 3] = [60, 30, 10];

/// The ring's circumference, for a radius of 13 on a 32 grid.
const RING: f64 = 81.68;

/// The countdown's words, resolved on the server in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CountdownWords {
    /// The line under the time left, such as "left, until 10:14:42".
    pub left: String,
    /// The line once the time is up, such as "ended at 10:14:42".
    pub ended: String,
    /// What the live region says at each of [`ANNOUNCE_AT`], in order.
    pub said: Vec<String>,
    /// What the live region says once the time is up.
    pub up: String,
}

impl CountdownWords {
    /// The words of a countdown ending at `until` (the deadline as the
    /// console writes a time, such as "10:14:42"), in the locale of the page
    /// being drawn.
    #[must_use]
    pub fn of_page(until: &str) -> Self {
        Self {
            left: tr_with(Text::CountdownLeft, &[("time", until.into())]),
            ended: tr_with(Text::CountdownEnded, &[("time", until.into())]),
            said: ANNOUNCE_AT
                .iter()
                .map(|&count| tr_with(Text::CountdownSaid, &[("count", count.into())]))
                .collect(),
            up: tr(Text::CountdownUp),
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
    /// Its words ([`CountdownWords::of_page`]).
    pub words: CountdownWords,
}

impl Countdown {
    /// The countdown `id` with `left` of `total` seconds left, ending at
    /// `until` in the console's words, its words in the page's locale.
    #[must_use]
    pub fn new(id: impl Into<String>, left: u32, total: u32, until: &str) -> Self {
        Self {
            id: id.into(),
            left,
            total,
            words: CountdownWords::of_page(until),
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

/// Returns the countdown `countdown`, counting down by the second once the
/// page hydrates and stopping at zero.
#[must_use]
pub fn countdown(countdown: Countdown) -> AnyView {
    let Countdown {
        id,
        left: start,
        total,
        words,
    } = countdown;
    let total = total.max(start);
    let CountdownWords {
        left: left_line,
        ended,
        said: points,
        up,
    } = words;
    let left = RwSignal::new(start);
    let said = RwSignal::new(String::new());
    tick(start, left, said, points, up);
    view! {
        <span class="fui-countdown" id=id>
            <svg viewBox="0 0 32 32" aria-hidden="true">
                <circle class="fui-countdown-track" cx="16" cy="16" r="13"></circle>
                <circle
                    class="fui-countdown-left"
                    cx="16"
                    cy="16"
                    r="13"
                    stroke-dasharray=RING.to_string()
                    stroke-dashoffset=move || ring_offset(left.get(), total)
                ></circle>
            </svg>
            <span class="fui-countdown-words" role="timer" aria-live="off">
                <strong>{move || clock(left.get())}</strong>
                <span>{move || if left.get() == 0 { ended.clone() } else { left_line.clone() }}</span>
            </span>
            <span class="fui-sr-only" role="status">
                {move || said.get()}
            </span>
        </span>
    }
    .into_any()
}

/// Counts `left` down from `start` in the browser by the seconds elapsed
/// since the page hydrated, putting the words of each set point crossed
/// (`points`) and of the end (`up`) in `said`.
fn tick(start: u32, left: RwSignal<u32>, said: RwSignal<String>, points: Vec<String>, up: String) {
    #[cfg(target_arch = "wasm32")]
    {
        let handle: StoredValue<Option<IntervalHandle>> = StoredValue::new(None);
        let words = StoredValue::new((points, up));
        Effect::new(move |_| {
            if start == 0 || handle.get_value().is_some() {
                return;
            }
            let Some(performance) = web_sys::window().and_then(|window| window.performance())
            else {
                return;
            };
            let began = performance.now();
            let step = move || {
                let before = left.get_untracked();
                let now = start.saturating_sub(whole_seconds(performance.now() - began));
                if now == before {
                    return;
                }
                left.set(now);
                words.with_value(|(points, up)| {
                    if now == 0 {
                        said.set(up.clone());
                    } else if let Some(words) = crossed(before, now).and_then(|at| points.get(at)) {
                        said.set(words.clone());
                    }
                });
                if now == 0
                    && let Some(started) = handle.get_value()
                {
                    started.clear();
                    handle.set_value(None);
                }
            };
            if let Ok(started) =
                set_interval_with_handle(step, std::time::Duration::from_millis(250))
            {
                handle.set_value(Some(started));
            }
        });
        on_cleanup(move || {
            if let Some(started) = handle.get_value() {
                started.clear();
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: (u32, RwSignal<u32>, RwSignal<String>) = (start, left, said);
        drop((points, up));
    }
}

#[cfg(test)]
mod tests {
    use super::{ANNOUNCE_AT, clock, crossed, ring_offset, whole_seconds};

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
