// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The date field and its picker (R23, R25), and the date range of a
//! history filter (R22).
//!
//! A date field is a text input, year first, with its format stated after
//! its label and the calendar behind a button inside its edge (R10). The
//! typed text is the field's value: the form posts it as it stands, with
//! or without the page's WebAssembly. Until the WebAssembly runs, the
//! button is disabled and its name says to type the date. Once it runs,
//! the button opens the picker: a popover under the field from 40 rem, a
//! bottom sheet below it, with one month, its arrows, a month and year
//! chooser behind the title, and the field's quick choices or a time row.
//! A range shows two months with its presets beside them; until the
//! WebAssembly runs, its From and To fields show in its button's place.
//!
//! The picker follows the WAI-ARIA Authoring Practices date picker dialog
//! pattern: a grid of days, each named by its full date, the arrows moving
//! a day or a week, Page Up and Page Down a month (with Shift a year), Home
//! and End to the week's ends, Enter choosing, and Escape closing with the
//! focus back on the field. Choosing a day writes the field; Cancel and
//! Escape put back what it held when the picker opened, and Apply keeps
//! it. A typed text that is no date, or a date the field does not allow,
//! marks the field invalid, says why under it, and disables Apply. The
//! server draws the picker closed. No specification governs the picker's
//! look: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::calendar::{Bounds, Day, Moment, Month, NotADay, Preset, Quick, Weekday};
use crate::i18n::{Phrase, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::form::{ERROR, INNER_BUTTON, error_id, help_id};

/// The library's message `text` in the page's locale with each of `names`
/// left open for the picker to fill as the reader chooses.
fn phrase(text: Text, names: &[&'static str]) -> Phrase {
    let args: Vec<(&'static str, crate::i18n::Arg)> = names
        .iter()
        .map(|name| (*name, Phrase::open(name)))
        .collect();
    Phrase::new(tr_with(text, &args))
}

/// The longest range whose number of days the picker's line states.
const COUNTED: u32 = 366;

/// Every word the date field and its picker draw, resolved on the server
/// in the page's locale ([`DateWords::of_page`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateWords {
    /// The format the field is typed in, its placeholder.
    pub format: String,
    /// The format again, as the quiet hint after the label.
    pub format_hint: String,
    /// The calendar button's name.
    pub choose: String,
    /// The calendar button's name before the page's WebAssembly runs.
    pub needs_script: String,
    /// The arrow to the month before.
    pub previous_month: String,
    /// The arrow to the month after.
    pub next_month: String,
    /// The month chooser's arrow to the year before.
    pub previous_year: String,
    /// The month chooser's arrow to the year after.
    pub next_year: String,
    /// The month chooser's title, back to the days.
    pub back_to_days: String,
    /// The button that chooses today.
    pub today: String,
    /// The button that empties the field.
    pub clear: String,
    /// The button that closes the picker and keeps the field as it was.
    pub cancel: String,
    /// The button that keeps the choice and closes the picker.
    pub apply: String,
    /// The time row's label.
    pub time: String,
    /// The time row's quiet hint.
    pub time_hint: String,
    /// The hours' name.
    pub hours: String,
    /// The minutes' name.
    pub minutes: String,
    /// The quick choices' name.
    pub quick: String,
    /// The presets' name.
    pub ranges: String,
    /// A range button with no range.
    pub any: String,
    /// The picker's line for a typed text that is no date.
    pub type_format: String,
    /// The twelve months' names, January first.
    pub months: Vec<String>,
    /// The seven days' names, Monday first.
    pub weekdays: Vec<String>,
    /// The seven days' short names over the columns, Monday first.
    pub weekdays_short: Vec<String>,
    /// How many days each month has, January to December, then February
    /// of a leap year.
    pub month_lengths: Vec<String>,
    /// A month's title: the month and the year.
    pub month_title: Phrase,
    /// A day's full name: its weekday, its day, its month and its year.
    pub day_name: Phrase,
    /// The title's name, which opens the month chooser: the month's title.
    pub choose_month: Phrase,
    /// The month chooser's name: the year.
    pub months_of: Phrase,
    /// A range: its first and last day.
    pub range_words: Phrase,
    /// The range picker's line with the first day chosen.
    pub range_half: Phrase,
    /// The range picker's line with both days chosen, and the days held.
    pub range_full: Phrase,
    /// A day its month does not have: the text and the month's length.
    pub not_a_day: Phrase,
    /// A month that is not one: the text and the month.
    pub no_month: Phrase,
    /// A time that is no time of day: the text.
    pub not_a_time: Phrase,
    /// A text not written as the format: the text.
    pub not_format: Phrase,
    /// A day before the earliest allowed: the text and the earliest.
    pub before: Phrase,
    /// A day after the latest allowed: the text and the latest.
    pub after: Phrase,
    /// The number of days a range holds, from 1 to 366, each resolved with
    /// its plural on the server.
    pub day_counts: Vec<String>,
}

impl DateWords {
    /// The words in the locale of the page being drawn, for a field that
    /// takes a time when `with_time`.
    #[must_use]
    pub fn of_page(with_time: bool) -> Self {
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
        let months: Vec<String> = MONTHS.iter().map(|month| tr(*month)).collect();
        let mut month_lengths: Vec<String> = (1_u8..=12)
            .zip(&months)
            .map(|(month, name)| {
                let days = crate::calendar::days_in(2026, month);
                tr_with(
                    Text::DateMonthLength,
                    &[
                        ("month", name.clone().into()),
                        ("count", u32::from(days).into()),
                    ],
                )
            })
            .collect();
        month_lengths.push(tr_with(
            Text::DateMonthLength,
            &[
                ("month", months.get(1).cloned().unwrap_or_default().into()),
                ("count", 29_u32.into()),
            ],
        ));
        Self {
            format: tr(format),
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
            month_title: phrase(Text::DateMonthTitle, &["month", "year"]),
            day_name: phrase(Text::DateDayName, &["weekday", "day", "month", "year"]),
            choose_month: phrase(Text::DateChooseMonth, &["title"]),
            months_of: phrase(Text::DateMonthsOf, &["year"]),
            range_words: phrase(Text::DateRangeWords, &["from", "to"]),
            range_half: phrase(Text::DateRangeHalf, &["from"]),
            range_full: phrase(Text::DateRangeFull, &["from", "to", "days"]),
            not_a_day: phrase(Text::DateNotADay, &["value", "why"]),
            no_month: phrase(Text::DateNoMonth, &["value", "month"]),
            not_a_time: phrase(Text::DateNotATime, &["value"]),
            not_format: phrase(not_format, &["value"]),
            before: phrase(Text::DateBefore, &["value", "earliest"]),
            after: phrase(Text::DateAfter, &["value", "latest"]),
            day_counts: (1..=COUNTED)
                .map(|count| tr_with(Text::DateDays, &[("count", count.into())]))
                .collect(),
        }
    }

    /// The name of `month`, 1 to 12.
    fn month_name(&self, month: u8) -> &str {
        self.months
            .get(usize::from(month.saturating_sub(1)))
            .map_or("", String::as_str)
    }

    /// The title of `month`.
    fn month_title(&self, month: Month) -> String {
        self.month_title.fill(&[
            ("month", self.month_name(month.month())),
            ("year", &month.year().to_string()),
        ])
    }

    /// The full name of `day`, its accessible name in the grid.
    fn day_name(&self, day: Day) -> String {
        let weekday = self
            .weekdays
            .get(usize::from(day.weekday().index()))
            .map_or("", String::as_str);
        self.day_name.fill(&[
            ("weekday", weekday),
            ("day", &day.day().to_string()),
            ("month", self.month_name(day.month())),
            ("year", &day.year().to_string()),
        ])
    }

    /// The number of days `count`, for a range of at most [`COUNTED`] days.
    fn days(&self, count: u32) -> Option<&str> {
        let place = usize::try_from(count.checked_sub(1)?).ok()?;
        self.day_counts.get(place).map(String::as_str)
    }

    /// Why the typed `text` is no value the field takes, or `None` when it
    /// is one or is empty.
    fn reason(&self, text: &str, with_time: bool, bounds: Bounds) -> Option<String> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let moment = match Moment::parse(text, with_time) {
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
                let why = self.month_lengths.get(place).map_or("", String::as_str);
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

/// A quick choice of a single date field: what it means and its words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickChoice {
    /// The day it chooses, counted from today.
    pub quick: Quick,
    /// Its words, such as "In a week", from the console's catalog.
    pub words: String,
}

/// A preset of a range: what it means and its words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangePreset {
    /// The range it fills.
    pub preset: Preset,
    /// Its words, which name exactly that range.
    pub words: String,
}

impl RangePreset {
    /// The preset `preset` with the library's words for it in the page's
    /// locale: "Today", "Last 7 days", "This month".
    #[must_use]
    pub fn of_page(preset: Preset) -> Self {
        let words = match preset {
            Preset::Today => tr(Text::PresetToday),
            Preset::LastDays(count) => {
                tr_with(Text::PresetLastDays, &[("count", u32::from(count).into())])
            }
            Preset::ThisMonth => tr(Text::PresetThisMonth),
        };
        Self { preset, words }
    }
}

/// What a date field shows ([`date_field`]), plain data a page hands the
/// island [`crate::island::DatePicker`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateField {
    /// The input's id.
    pub id: String,
    /// The name the input posts as.
    pub name: String,
    /// The label.
    pub label: String,
    /// The value the field holds, as the server has it.
    pub value: String,
    /// The help line under the field.
    pub help: Option<String>,
    /// The console's refusal of the posted value, under the help.
    pub error: Option<String>,
    /// Today, in the deployment's time zone.
    pub today: Day,
    /// The days the field allows: a field that forbids the future ends
    /// today.
    pub bounds: Bounds,
    /// The first day of the week, from the reader's preference (family
    /// rule 9).
    pub first_day: Weekday,
    /// Whether the field takes a time of day after the date.
    pub with_time: bool,
    /// The quick choices above the grid, such as Tomorrow and In a week.
    pub quick: Vec<QuickChoice>,
    /// The picker's line once a day is chosen, such as "Back in the queue
    /// on …", its message looked up with `day` left open
    /// ([`Phrase::open`]) where the day's name goes.
    pub chosen: Option<Phrase>,
    /// The words it draws ([`DateWords::of_page`]).
    pub words: DateWords,
}

/// One end of a range: the input it posts and what it holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeEnd {
    /// The input's id.
    pub id: String,
    /// The name the input posts as.
    pub name: String,
    /// The label of the field that shows without the WebAssembly.
    pub label: String,
    /// The day it holds, as the server has it.
    pub value: String,
}

/// What a date range shows ([`date_range`]), plain data a page hands the
/// island [`crate::island::DateRangePicker`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRange {
    /// The picker's id.
    pub id: String,
    /// The picker's name, such as "Actions between".
    pub label: String,
    /// The first day.
    pub from: RangeEnd,
    /// The last day.
    pub to: RangeEnd,
    /// Today, in the deployment's time zone.
    pub today: Day,
    /// The days the range allows.
    pub bounds: Bounds,
    /// The first day of the week, from the reader's preference.
    pub first_day: Weekday,
    /// The presets beside the months ([`RangePreset::of_page`]).
    pub presets: Vec<RangePreset>,
    /// The words it draws ([`DateWords::of_page`]).
    pub words: DateWords,
}

/// The days the reader chooses from, and where the choice goes.
#[derive(Debug, Clone, Copy)]
enum Pick {
    /// One day, written into one field's text, with a time when it takes
    /// one.
    One {
        /// The field's text.
        text: RwSignal<String>,
        /// The time row's hours and minutes, for a field that takes them.
        clock: Option<(RwSignal<String>, RwSignal<String>)>,
    },
    /// A range, written into two texts, the day under the pointer previewing
    /// its end.
    Two {
        /// The first day's text.
        from: RwSignal<String>,
        /// The last day's text.
        to: RwSignal<String>,
        /// The day under the pointer while the end is to be chosen.
        hover: RwSignal<Option<Day>>,
    },
}

impl Pick {
    /// The range chosen so far: the first day and the last, each when
    /// chosen.
    fn ends(self) -> (Option<Day>, Option<Day>) {
        match self {
            Self::One { text, clock } => {
                let day = text.with(|text| {
                    Moment::parse(text, clock.is_some())
                        .ok()
                        .map(|moment| moment.day)
                });
                (day, day)
            }
            Self::Two { from, to, .. } => (
                from.with(|text| Day::parse(text).ok()),
                to.with(|text| Day::parse(text).ok()),
            ),
        }
    }

    /// Whether `day` is chosen: the field's day, or an end of the range.
    fn chosen(self, day: Day) -> bool {
        let (first, last) = self.ends();
        first == Some(day) || last == Some(day)
    }

    /// The classes of `day`'s button, `out` when it falls outside the month
    /// it is drawn in.
    fn classes(self, day: Day, out: bool) -> String {
        let mut classes = String::from("fui-day");
        if out {
            classes.push_str(" fui-day-out");
        }
        let Self::Two { hover, .. } = self else {
            return classes;
        };
        let (first, last) = self.ends();
        let tentative = last.is_none();
        let Some(first) = first else {
            return classes;
        };
        let Some(end) = last.or_else(|| hover.get().filter(|hovered| *hovered > first)) else {
            return classes;
        };
        if first < day && day < end {
            classes.push_str(if tentative {
                " fui-day-between fui-day-tentative"
            } else {
                " fui-day-between"
            });
        }
        if first < end && day == first {
            classes.push_str(" fui-day-start");
        }
        if first < end && day == end {
            classes.push_str(if tentative {
                " fui-day-end fui-day-tentative-end"
            } else {
                " fui-day-end"
            });
        }
        classes
    }

    /// Chooses `day`: the field's day, or the next end of the range.
    fn choose(self, day: Day) {
        match self {
            Self::One { text, clock } => text.set(compose(day, clock)),
            Self::Two { from, to, hover } => {
                let (first, last) = self.ends();
                match (first, last) {
                    (Some(first), None) if day < first => {
                        from.set(day.to_string());
                        to.set(first.to_string());
                    }
                    (Some(_), None) => to.set(day.to_string()),
                    _ => {
                        from.set(day.to_string());
                        to.set(String::new());
                    }
                }
                hover.set(None);
            }
        }
    }
}

/// The text of `day` for a field, with the time row's time when it takes
/// one.
fn compose(day: Day, clock: Option<(RwSignal<String>, RwSignal<String>)>) -> String {
    match clock {
        None => day.to_string(),
        Some((hours, minutes)) => format!(
            "{day} {:0>2}:{:0>2}",
            hours.get_untracked().trim(),
            minutes.get_untracked().trim()
        ),
    }
}

/// What the picker's grid needs: today, the allowed days, the week, the
/// focused day, the choice and the words.
#[derive(Debug, Clone, Copy)]
struct Grid {
    /// Today.
    today: Day,
    /// The allowed days.
    bounds: Bounds,
    /// The first day of the week.
    first: Weekday,
    /// The day that holds the grid's keyboard focus.
    focused: RwSignal<Day>,
    /// The month shown: the one month, or a range's later month.
    shown: RwSignal<Month>,
    /// Whether two months show.
    pair: bool,
    /// The choice.
    pick: Pick,
    /// The words.
    words: StoredValue<DateWords>,
    /// The picker, which the focus moves within.
    panel: NodeRef<leptos::html::Div>,
}

impl Grid {
    /// Shows the month that holds `day`, if it is not shown.
    fn reveal(self, day: Day) {
        let month = day.month_of();
        let shown = self.shown.get_untracked();
        if self.pair {
            let earlier = shown.step(-1).unwrap_or(shown);
            if month > shown {
                self.shown.set(month);
            } else if month < earlier {
                self.shown.set(month.step(1).unwrap_or(month));
            }
        } else if month != shown {
            self.shown.set(month);
        }
    }

    /// Moves the keyboard focus to `day`, when it is allowed.
    fn focus(self, day: Day) {
        if !self.bounds.allows(day) {
            return;
        }
        self.focused.set(day);
        self.reveal(day);
        focus_day(self.panel, day);
    }

    /// Moves the focus by the key of `event`, the grid's keyboard path: an
    /// arrow a day or a week, Page Up and Page Down a month (with Shift a
    /// year), Home and End the week's ends.
    fn key(self, event: &leptos::ev::KeyboardEvent) {
        let from = self.focused.get_untracked();
        let months = if event.shift_key() { 12 } else { 1 };
        let target = match event.key().as_str() {
            "ArrowLeft" => from.step(-1),
            "ArrowRight" => from.step(1),
            "ArrowUp" => from.step(-7),
            "ArrowDown" => from.step(7),
            "PageUp" => from.step_months(-months),
            "PageDown" => from.step_months(months),
            "Home" => Some(from.week_start(self.first)),
            "End" => from.week_start(self.first).step(6),
            _ => return,
        };
        event.prevent_default();
        if let Some(day) = target {
            self.focus(day);
        }
    }
}

/// Moves the keyboard focus to the button of `day` in `panel`, once the
/// browser has drawn it.
fn focus_day(panel: NodeRef<leptos::html::Div>, day: Day) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        request_animation_frame(move || {
            let button = panel
                .get_untracked()
                .and_then(|panel| {
                    panel
                        .query_selector(&format!("[data-day=\"{day}\"]"))
                        .ok()
                        .flatten()
                })
                .and_then(|button| button.dyn_into::<web_sys::HtmlElement>().ok());
            if let Some(button) = button
                && button.focus().is_err()
            {
                leptos::logging::warn!("the browser refused the focus on a day of the picker");
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (NodeRef<leptos::html::Div>, Day) = (panel, day);
}

/// Moves the keyboard focus to `input`, once the browser has drawn it.
fn focus_input(input: NodeRef<leptos::html::Input>) {
    #[cfg(target_arch = "wasm32")]
    request_animation_frame(move || {
        if let Some(input) = input.get_untracked()
            && input.focus().is_err()
        {
            leptos::logging::warn!("the browser refused the focus on a date field");
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    let _: NodeRef<leptos::html::Input> = input;
}

/// Closes the picker `open` when a press lands outside `root`, from the
/// effect that runs once the field is in the browser.
fn close_outside(root: NodeRef<leptos::html::Div>, open: RwSignal<bool>) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let handle = window_event_listener(leptos::ev::pointerdown, move |event| {
            if !open.get_untracked() {
                return;
            }
            let target = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Node>().ok());
            let inside = root
                .get_untracked()
                .is_some_and(|root| root.contains(target.as_ref()));
            if !inside {
                open.set(false);
            }
        });
        on_cleanup(move || handle.remove());
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (NodeRef<leptos::html::Div>, RwSignal<bool>) = (root, open);
}

/// Sets `flip` when the picker `panel`, opened under `root` and aligned to
/// its start (or with `from_end` to its end), would run past the screen's
/// edge, so it opens the other way.
fn place(
    root: NodeRef<leptos::html::Div>,
    panel: NodeRef<leptos::html::Div>,
    flip: RwSignal<bool>,
    from_end: bool,
) {
    #[cfg(target_arch = "wasm32")]
    request_animation_frame(move || {
        let (Some(root), Some(panel)) = (root.get_untracked(), panel.get_untracked()) else {
            return;
        };
        let width = window()
            .inner_width()
            .ok()
            .and_then(|width| width.as_f64())
            .unwrap_or(f64::MAX);
        let field = root.get_bounding_client_rect();
        let wide = panel.get_bounding_client_rect().width();
        // NOTE: no specification governs this: our own design; a 16 px gutter
        // keeps the picker clear of the screen's edge, as the page's own.
        let past = if from_end {
            field.right() - wide < 16.0
        } else {
            field.left() + wide > width - 16.0
        };
        flip.set(past);
    });
    #[cfg(not(target_arch = "wasm32"))]
    let _: (
        NodeRef<leptos::html::Div>,
        NodeRef<leptos::html::Div>,
        RwSignal<bool>,
        bool,
    ) = (root, panel, flip, from_end);
}

/// Submits the form `input` belongs to, as a press on its submit button
/// would.
fn submit_form(input: NodeRef<leptos::html::Input>) {
    #[cfg(target_arch = "wasm32")]
    {
        let form = input.get_untracked().and_then(|input| input.form());
        if let Some(form) = form
            && form.request_submit().is_err()
        {
            leptos::logging::warn!("the browser refused to submit the range's form");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: NodeRef<leptos::html::Input> = input;
}

/// The day the picker focuses as it opens: the chosen one, or today, or
/// the allowed day nearest today.
fn opening_day(chosen: Option<Day>, today: Day, bounds: Bounds) -> Day {
    chosen.unwrap_or_else(|| {
        if let Some(earliest) = bounds.earliest.filter(|earliest| today < *earliest) {
            earliest
        } else if let Some(latest) = bounds.latest.filter(|latest| today > *latest) {
            latest
        } else {
            today
        }
    })
}

/// Returns an arrow named `name` showing `glyph`, disabled when `blocked`.
fn arrow(
    glyph: Glyph,
    name: String,
    class: &'static str,
    blocked: impl Fn() -> bool + Send + Sync + 'static,
    press: impl Fn() + 'static,
) -> AnyView {
    view! {
        <button
            type="button"
            class=class
            aria-label=name
            disabled=blocked
            on:click=move |_| press()
        >
            {icon(glyph, "16")}
        </button>
    }
    .into_any()
}

/// Returns the weeks of `month` as a grid of day buttons.
fn month_grid(grid: Grid, month: Month) -> AnyView {
    let words = grid.words;
    let title = words.with_value(|words| words.month_title(month));
    let head = grid
        .first
        .week()
        .into_iter()
        .map(|weekday| {
            let place = usize::from(weekday.index());
            let (full, short) = words.with_value(|words| {
                (
                    words.weekdays.get(place).cloned().unwrap_or_default(),
                    words.weekdays_short.get(place).cloned().unwrap_or_default(),
                )
            });
            view! {
                <th scope="col" abbr=full>
                    {short}
                </th>
            }
        })
        .collect_view();
    let rows = month
        .weeks(grid.first)
        .into_iter()
        .map(|week| {
            let cells = week
                .into_iter()
                .map(|day| day_cell(grid, day, month))
                .collect_view();
            view! { <tr>{cells}</tr> }
        })
        .collect_view();
    view! {
        <table role="grid" class="fui-datepicker-grid" aria-label=title on:keydown=move |event| grid.key(&event)>
            <thead>
                <tr>{head}</tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>
    }
    .into_any()
}

/// Returns the cell of `day` in the grid of `month`.
fn day_cell(grid: Grid, day: Day, month: Month) -> AnyView {
    let out = day.month_of() != month;
    let allowed = grid.bounds.allows(day);
    let name = grid.words.with_value(|words| words.day_name(day));
    let pick = grid.pick;
    let selected = move || pick.chosen(day).then_some("true");
    let tab = move || {
        if !out && grid.focused.get() == day {
            "0"
        } else {
            "-1"
        }
    };
    let (key, out_key) = if out {
        (None, Some(day.to_string()))
    } else {
        (Some(day.to_string()), None)
    };
    let hover = move |_| {
        if let Pick::Two { hover, .. } = pick {
            hover.set(Some(day));
        }
    };
    let press = move |_| {
        pick.choose(day);
        grid.focused.set(day);
        grid.reveal(day);
    };
    view! {
        <td role="gridcell" aria-selected=selected aria-current=(day == grid.today).then_some("date")>
            <button
                type="button"
                class=move || pick.classes(day, out)
                aria-label=name
                data-day=key
                data-out-day=out_key
                tabindex=tab
                disabled=!allowed
                on:mouseenter=hover
                on:click=press
            >
                {day.day()}
            </button>
        </td>
    }
    .into_any()
}

/// Returns the head of `month`: its arrows and its title, which opens the
/// month chooser when `chooser` holds its switch.
fn month_head(
    grid: Grid,
    month: Month,
    back: Option<&'static str>,
    on: bool,
    chooser: Option<RwSignal<bool>>,
) -> AnyView {
    let words = grid.words;
    let bounds = grid.bounds;
    let shown = grid.shown;
    let title = words.with_value(|words| words.month_title(month));
    let previous = back.map(|class| {
        let earlier = month.step(-1);
        arrow(
            Glyph::CHEVRON_LEFT,
            words.with_value(|words| words.previous_month.clone()),
            class,
            move || earlier.is_none_or(|earlier| !bounds.reaches(earlier)),
            move || {
                if let Some(earlier) = shown.get_untracked().step(-1) {
                    shown.set(earlier);
                }
            },
        )
    });
    let next = on.then(|| {
        let later = month.step(1);
        arrow(
            Glyph::CHEVRON_RIGHT,
            words.with_value(|words| words.next_month.clone()),
            "fui-icon-button",
            move || later.is_none_or(|later| !bounds.reaches(later)),
            move || {
                if let Some(later) = shown.get_untracked().step(1) {
                    shown.set(later);
                }
            },
        )
    });
    let gap = |show: bool| (!show).then(|| view! { <span class="fui-datepicker-navgap"></span> });
    let title = match chooser {
        Some(months) => {
            let name = words.with_value(|words| words.choose_month.fill(&[("title", &title)]));
            view! {
                <button
                    type="button"
                    class="fui-datepicker-title"
                    aria-label=name
                    on:click=move |_| months.set(true)
                >
                    {title}
                    {icon(Glyph::CHEVRON_DOWN, "14")}
                </button>
            }
            .into_any()
        }
        None => view! { <span class="fui-datepicker-title">{title}</span> }.into_any(),
    };
    view! {
        <div class="fui-datepicker-head">
            {gap(back.is_some())}
            {previous}
            {title}
            {next}
            {gap(on)}
        </div>
    }
    .into_any()
}

/// Returns the month chooser of the single picker: the twelve months of a
/// year, its arrows stepping the year, its title back to the days.
fn month_chooser(grid: Grid, months: RwSignal<bool>) -> AnyView {
    let words = grid.words;
    let bounds = grid.bounds;
    let today = grid.today;
    let year = RwSignal::new(grid.shown.get_untracked().year());
    let reaches_year = move |year: Option<u16>| {
        year.and_then(|year| Month::new(year, 1))
            .is_some_and(|first| {
                (0..12).any(|step| first.step(step).is_some_and(|month| bounds.reaches(month)))
            })
    };
    let previous = arrow(
        Glyph::CHEVRON_LEFT,
        words.with_value(|words| words.previous_year.clone()),
        "fui-icon-button",
        move || !reaches_year(year.get().checked_sub(1)),
        move || year.update(|year| *year = year.saturating_sub(1).max(1)),
    );
    let next = arrow(
        Glyph::CHEVRON_RIGHT,
        words.with_value(|words| words.next_year.clone()),
        "fui-icon-button",
        move || !reaches_year(year.get().checked_add(1).filter(|year| *year <= 9999)),
        move || year.update(|year| *year = year.saturating_add(1).min(9999)),
    );
    let cells = move || {
        let shown = grid.shown.get();
        (1_u8..=12)
            .filter_map(|place| Month::new(year.get(), place))
            .map(|month| {
                let name = words.with_value(|words| {
                    words
                        .month_name(month.month())
                        .chars()
                        .take(3)
                        .collect::<String>()
                });
                let full = words.with_value(|words| words.month_title(month));
                let press = move |_| {
                    grid.shown.set(month);
                    months.set(false);
                };
                view! {
                    <button
                        type="button"
                        class="fui-datepicker-cell"
                        aria-label=full
                        aria-selected=(month == shown).then_some("true")
                        aria-current=(month == today.month_of()).then_some("date")
                        disabled=!bounds.reaches(month)
                        on:click=press
                    >
                        {name}
                    </button>
                }
            })
            .collect_view()
    };
    let label = move || {
        words.with_value(|words| words.months_of.fill(&[("year", &year.get().to_string())]))
    };
    view! {
        <div class="fui-datepicker-month">
            <div class="fui-datepicker-head">
                {previous}
                <button
                    type="button"
                    class="fui-datepicker-title"
                    aria-label=words.with_value(|words| words.back_to_days.clone())
                    on:click=move |_| months.set(false)
                >
                    {move || year.get()}
                    {icon(Glyph::CHEVRON_UP, "14")}
                </button>
                {next}
            </div>
            <div class="fui-datepicker-months" role="grid" aria-label=label>
                {cells}
            </div>
        </div>
    }
    .into_any()
}

/// Returns the date field `field` with its picker (R23, R25); a page draws
/// it through [`crate::island::DatePicker`].
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "the field, its button, its error line and its picker are one view whose parts share the field's signals"
)]
pub fn date_field(field: DateField) -> AnyView {
    let DateField {
        id,
        name,
        label,
        value,
        help,
        error,
        today,
        bounds,
        first_day,
        with_time,
        quick,
        chosen,
        words,
    } = field;
    let text = RwSignal::new(value.clone());
    let original = RwSignal::new(value.clone());
    let touched = RwSignal::new(false);
    let open = RwSignal::new(false);
    let months = RwSignal::new(false);
    let hydrated = RwSignal::new(false);
    Effect::new(move || hydrated.set(true));
    let initial = Moment::parse(&value, with_time).ok();
    let clock = with_time.then(|| {
        let time = initial.and_then(|moment| moment.time);
        (
            RwSignal::new(
                time.map_or_else(|| String::from("00"), |time| format!("{:02}", time.hour())),
            ),
            RwSignal::new(time.map_or_else(
                || String::from("00"),
                |time| format!("{:02}", time.minute()),
            )),
        )
    });
    let pick = Pick::One { text, clock };
    let opening = opening_day(initial.map(|moment| moment.day), today, bounds);
    let panel = NodeRef::<leptos::html::Div>::new();
    let root = NodeRef::<leptos::html::Div>::new();
    let input = NodeRef::<leptos::html::Input>::new();
    let flip = RwSignal::new(false);
    let grid = Grid {
        today,
        bounds,
        first: first_day,
        focused: RwSignal::new(opening),
        shown: RwSignal::new(opening.month_of()),
        pair: false,
        pick,
        words: StoredValue::new(words.clone()),
        panel,
    };
    Effect::new(move || close_outside(root, open));
    let reason = move || {
        grid.words
            .with_value(|words| text.with(|text| words.reason(text, with_time, bounds)))
    };
    let error = StoredValue::new(error);
    let refusal = move || {
        if touched.get() {
            reason()
        } else {
            error.get_value()
        }
    };
    let invalid = move || refusal().is_some().then_some("true");
    let picker_id = format!("{id}-picker");
    let error_line = error_id(&id);
    let described = {
        let mut parts = Vec::new();
        if help.is_some() {
            parts.push(help_id(&id));
        }
        parts.push(error_line.clone());
        parts.join(" ")
    };
    let follow = move |typed: String| {
        if let Ok(moment) = Moment::parse(&typed, with_time) {
            if let (Some((hours, minutes)), Some(time)) = (clock, moment.time) {
                hours.set(format!("{:02}", time.hour()));
                minutes.set(format!("{:02}", time.minute()));
            }
            grid.focused.set(moment.day);
            grid.reveal(moment.day);
        }
        touched.set(true);
        text.set(typed);
    };
    let close = move |restore: bool| {
        if restore {
            text.set(original.get_untracked());
        }
        open.set(false);
        months.set(false);
        focus_input(input);
    };
    let toggle = move |_| {
        if open.get_untracked() {
            close(true);
            return;
        }
        original.set(text.get_untracked());
        let day = opening_day(pick.ends().0, today, bounds);
        grid.focused.set(day);
        grid.shown.set(day.month_of());
        months.set(false);
        open.set(true);
        focus_day(panel, day);
        place(root, panel, flip, false);
    };
    let button_name = {
        let choose = words.choose.clone();
        let needs = words.needs_script.clone();
        move || {
            if hydrated.get() {
                choose.clone()
            } else {
                needs.clone()
            }
        }
    };
    let help = help.map(|help| view! { <p id=help_id(&id) class="fui-help">{help}</p> });
    let picker_label = label.clone();
    let panel_id = picker_id.clone();
    let picker = move || {
        open.get().then(|| {
            let words = grid.words.get_value();
            let (picker_label, panel_id) = (picker_label.clone(), panel_id.clone());
            let heading = picker_label.clone();
            let quick = (!quick.is_empty()).then(|| {
                let choices = quick
                    .iter()
                    .map(|choice| {
                        let day = choice.quick.day(today).filter(|day| bounds.allows(*day));
                        let pressed = move || (day.is_some() && pick.ends().0 == day).to_string();
                        let press = move |_| {
                            if let Some(day) = day {
                                touched.set(true);
                                pick.choose(day);
                                grid.focused.set(day);
                                grid.reveal(day);
                            }
                        };
                        view! {
                            <button
                                type="button"
                                class="fui-chip-check fui-datepicker-chip"
                                aria-pressed=pressed
                                disabled=day.is_none()
                                on:click=press
                            >
                                {choice.words.clone()}
                            </button>
                        }
                    })
                    .collect_view();
                view! {
                    <div class="fui-datepicker-quick" role="group" aria-label=words.quick.clone()>
                        {choices}
                    </div>
                }
            });
            let days = move || {
                if months.get() {
                    month_chooser(grid, months).into_any()
                } else {
                    let month = grid.shown.get();
                    view! {
                        <div class="fui-datepicker-month">
                            {month_head(grid, month, Some("fui-icon-button"), true, Some(months))}
                            {month_grid(grid, month)}
                        </div>
                    }
                    .into_any()
                }
            };
            let time = clock.map(|(hours, minutes)| {
                let set = move |part: RwSignal<String>, typed: String| {
                    part.set(typed);
                    if let Some(day) = pick.ends().0 {
                        touched.set(true);
                        pick.choose(day);
                    }
                };
                view! {
                    <div class="fui-datepicker-time">
                        <span class="fui-label">
                            {words.time.clone()} " "
                            <span class="fui-muted">{words.time_hint.clone()}</span>
                        </span>
                        <div class="fui-datepicker-time-fields" role="group" aria-label=words.time.clone()>
                            <input
                                class="fui-input"
                                inputmode="numeric"
                                maxlength="2"
                                aria-label=words.hours.clone()
                                prop:value=move || hours.get()
                                on:input=move |event| set(hours, event_target_value(&event))
                            />
                            <span aria-hidden="true">":"</span>
                            <input
                                class="fui-input"
                                inputmode="numeric"
                                maxlength="2"
                                aria-label=words.minutes.clone()
                                prop:value=move || minutes.get()
                                on:input=move |event| set(minutes, event_target_value(&event))
                            />
                        </div>
                    </div>
                }
            });
            let chosen = chosen.clone();
            let status = move || {
                if let Some(reason) = reason() {
                    return grid.words.with_value(|words| {
                        if text.with(|text| Moment::parse(text, with_time).is_err()) {
                            words.type_format.clone()
                        } else {
                            reason
                        }
                    });
                }
                match (pick.ends().0, chosen.as_ref()) {
                    (Some(day), Some(sentence)) => {
                        grid.words.with_value(|words| sentence.fill(&[("day", &words.day_name(day))]))
                    }
                    _ => String::new(),
                }
            };
            let today_allowed = bounds.allows(today);
            let body = view! {
            <div class="fui-datepicker-body">
                <div class="fui-datepicker-main">
                    {quick}
                    {days}
                    {time}
                    <p class="fui-datepicker-status" role="status" hidden=move || status().is_empty()>
                        {status.clone()}
                    </p>
                </div>
            </div>
            }
            .into_any();
            let foot = view! {
            <div class="fui-datepicker-foot">
                <div class="fui-actions">
                    <button
                        type="button"
                        class=Variant::Ghost.small()
                        disabled=!today_allowed
                        on:click=move |_| {
                            touched.set(true);
                            pick.choose(today);
                            grid.focused.set(today);
                            grid.reveal(today);
                        }
                    >
                        {words.today.clone()}
                    </button>
                    <button
                        type="button"
                        class=Variant::Ghost.small()
                        on:click=move |_| {
                            touched.set(true);
                            text.set(String::new());
                        }
                    >
                        {words.clear.clone()}
                    </button>
                </div>
                <div class="fui-actions">
                    <button type="button" class=Variant::Secondary.small() on:click=move |_| close(true)>
                        {words.cancel.clone()}
                    </button>
                    <button
                        type="button"
                        class=Variant::Primary.small()
                        disabled=move || reason().is_some()
                        on:click=move |_| close(false)
                    >
                        {words.apply.clone()}
                    </button>
                </div>
            </div>
            }
            .into_any();
            view! {
                <div
                    class="fui-datepicker"
                    class:is-end=move || flip.get()
                    id=panel_id
                    role="dialog"
                    aria-label=picker_label
                    node_ref=panel
                    on:keydown=move |event: leptos::ev::KeyboardEvent| {
                        if event.key() == "Escape" {
                            event.prevent_default();
                            close(true);
                        }
                    }
                >
                    <span class="fui-datepicker-handle" aria-hidden="true"></span>
                    <p class="fui-datepicker-label">{heading}</p>
                    {body}
                    {foot}
                </div>
            }
            .into_any()
        })
    };
    view! {
        <div class="fui-field fui-date-field" node_ref=root>
            <label for=id.clone() class="fui-label">
                {label.clone()} " " <span class="fui-muted">{words.format_hint.clone()}</span>
            </label>
            <div class="fui-input-group fui-date">
                <input
                    id=id
                    name=name
                    type="text"
                    class="fui-input"
                    inputmode="numeric"
                    autocomplete="off"
                    spellcheck="false"
                    placeholder=words.format.clone()
                    value=value
                    aria-invalid=invalid
                    aria-describedby=described
                    node_ref=input
                    prop:value=move || text.get()
                    on:input=move |event| follow(event_target_value(&event))
                />
                <button
                    type="button"
                    class=INNER_BUTTON
                    aria-label=button_name
                    aria-haspopup="dialog"
                    aria-expanded=move || open.get().to_string()
                    aria-controls=picker_id
                    disabled=move || !hydrated.get()
                    on:click=toggle
                >
                    {icon(Glyph::CALENDAR, "16")}
                </button>
            </div>
            {help}
            <p id=error_line class=ERROR aria-live="polite" hidden=move || refusal().is_none()>
                {move || refusal().unwrap_or_default()}
            </p>
            {picker}
        </div>
    }
    .into_any()
}

/// Returns a range end's field as it shows until the page's WebAssembly
/// runs: its label with the format, and its input bound to `text`.
fn range_field(
    end: RangeEnd,
    words: &DateWords,
    text: RwSignal<String>,
    input: NodeRef<leptos::html::Input>,
) -> AnyView {
    let RangeEnd {
        id,
        name,
        label,
        value,
    } = end;
    view! {
        <div class="fui-field">
            <label for=id.clone() class="fui-label">
                {label} " " <span class="fui-muted">{words.format_hint.clone()}</span>
            </label>
            <input
                id=id
                name=name
                type="text"
                class="fui-input"
                inputmode="numeric"
                autocomplete="off"
                spellcheck="false"
                placeholder=words.format.clone()
                value=value
                node_ref=input
                prop:value=move || text.get()
                on:input=move |event| text.set(event_target_value(&event))
            />
        </div>
    }
    .into_any()
}

/// The range's words for the button, or "Any date" when it holds none.
fn range_summary(words: &DateWords, from: &str, to: &str) -> String {
    match (Day::parse(from), Day::parse(to)) {
        (Ok(from), Ok(to)) => words
            .range_words
            .fill(&[("from", &from.to_string()), ("to", &to.to_string())]),
        _ => words.any.clone(),
    }
}

/// The range picker's line: the first day and what to do next, or the
/// whole range and the days it holds.
fn range_status(words: &DateWords, first: Option<Day>, last: Option<Day>) -> String {
    match (first, last) {
        (Some(first), Some(last)) => {
            let (from, to) = (first.to_string(), last.to_string());
            match words.days(first.days_through(last)) {
                Some(days) => {
                    words
                        .range_full
                        .fill(&[("from", &from), ("to", &to), ("days", days)])
                }
                None => words.range_words.fill(&[("from", &from), ("to", &to)]),
            }
        }
        (Some(first), None) => words.range_half.fill(&[("from", &first.to_string())]),
        _ => String::new(),
    }
}

/// Returns the date range `range` of a history filter with its picker
/// (R22, R25); a page draws it through
/// [`crate::island::DateRangePicker`].
///
/// Until the page's WebAssembly runs, its From and To fields show and post
/// with the filter's form. Once it runs, a secondary button naming the
/// range takes their place, the fields still posting, and opens the picker:
/// the presets beside two months. Apply writes both fields and submits the
/// form.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "the floor's fields, the button and the picker are one view whose parts share the range's signals"
)]
pub fn date_range(range: DateRange) -> AnyView {
    let DateRange {
        id,
        label,
        from,
        to,
        today,
        bounds,
        first_day,
        presets,
        words,
    } = range;
    let from_text = RwSignal::new(from.value.clone());
    let to_text = RwSignal::new(to.value.clone());
    let saved = RwSignal::new((from.value.clone(), to.value.clone()));
    let hover = RwSignal::new(None);
    let open = RwSignal::new(false);
    let hydrated = RwSignal::new(false);
    Effect::new(move || hydrated.set(true));
    let pick = Pick::Two {
        from: from_text,
        to: to_text,
        hover,
    };
    let (first, last) = (Day::parse(&from.value).ok(), Day::parse(&to.value).ok());
    let opening = opening_day(last.or(first), today, bounds);
    let panel = NodeRef::<leptos::html::Div>::new();
    let root = NodeRef::<leptos::html::Div>::new();
    let from_input = NodeRef::<leptos::html::Input>::new();
    let flip = RwSignal::new(false);
    let to_input = NodeRef::<leptos::html::Input>::new();
    let stored = StoredValue::new(words.clone());
    let grid = Grid {
        today,
        bounds,
        first: first_day,
        focused: RwSignal::new(opening),
        shown: RwSignal::new(opening.month_of()),
        pair: true,
        pick,
        words: stored,
        panel,
    };
    Effect::new(move || close_outside(root, open));
    let summary = move || {
        stored.with_value(|words| {
            if open.get() {
                saved.with(|(from, to)| range_summary(words, from, to))
            } else {
                from_text.with(|from| to_text.with(|to| range_summary(words, from, to)))
            }
        })
    };
    let picker_id = format!("{id}-picker");
    let close = move |restore: bool| {
        if restore {
            let (from, to) = saved.get_untracked();
            from_text.set(from);
            to_text.set(to);
        }
        hover.set(None);
        open.set(false);
    };
    let toggle = move |_| {
        if open.get_untracked() {
            close(true);
            return;
        }
        saved.set((from_text.get_untracked(), to_text.get_untracked()));
        let (first, last) = pick.ends();
        let day = opening_day(last.or(first), today, bounds);
        grid.focused.set(day);
        grid.shown.set(day.month_of());
        open.set(true);
        focus_day(panel, day);
        place(root, panel, flip, true);
    };
    let fields = view! {
        <div class="fui-date-range-fields" hidden=move || hydrated.get()>
            {range_field(from, &words, from_text, from_input)}
            {range_field(to, &words, to_text, to_input)}
        </div>
    };
    let panel_id = picker_id.clone();
    let picker = move || {
        open.get().then(|| {
            let words = stored.get_value();
            let presets = presets
                .iter()
                .map(|preset| {
                    let span = preset.preset.range(today).filter(|(first, last)| {
                        bounds.allows(*first) && bounds.allows(*last)
                    });
                    let pressed = move || {
                        let (first, last) = pick.ends();
                        (span.is_some() && first.zip(last) == span).to_string()
                    };
                    let press = move |_| {
                        if let Some((first, last)) = span {
                            from_text.set(first.to_string());
                            to_text.set(last.to_string());
                            hover.set(None);
                            grid.focused.set(last);
                            grid.reveal(last);
                        }
                    };
                    view! {
                        <button
                            type="button"
                            class="fui-datepicker-preset"
                            aria-pressed=pressed
                            disabled=span.is_none()
                            on:click=press
                        >
                            {preset.words.clone()}
                        </button>
                    }
                })
                .collect_view();
            let months = move || {
                let later = grid.shown.get();
                let earlier = later.step(-1).unwrap_or(later);
                view! {
                    <div class="fui-datepicker-month">
                        {month_head(grid, earlier, Some("fui-icon-button"), false, None)}
                        {month_grid(grid, earlier)}
                    </div>
                    <div class="fui-datepicker-month">
                        {month_head(grid, later, Some("fui-icon-button fui-datepicker-phone-prev"), true, None)}
                        {month_grid(grid, later)}
                    </div>
                }
            };
            let status = move || {
                let (first, last) = pick.ends();
                stored.with_value(|words| range_status(words, first, last))
            };
            let half = move || {
                let (first, last) = pick.ends();
                first.is_some() != last.is_some()
            };
            let heading = label.clone();
            let body = view! {
            <div class="fui-datepicker-body">
                <div class="fui-datepicker-presets" role="group" aria-label=words.ranges.clone()>
                    {presets}
                </div>
                <div class="fui-datepicker-main">
                    <div class="fui-datepicker-pair">{months}</div>
                    <p class="fui-datepicker-status" role="status" hidden=move || status().is_empty()>
                        {status}
                    </p>
                </div>
            </div>
            }
            .into_any();
            let foot = view! {
            <div class="fui-datepicker-foot">
                <div class="fui-actions">
                    <button
                        type="button"
                        class=Variant::Ghost.small()
                        on:click=move |_| {
                            from_text.set(String::new());
                            to_text.set(String::new());
                            hover.set(None);
                        }
                    >
                        {words.clear.clone()}
                    </button>
                </div>
                <div class="fui-actions">
                    <button type="button" class=Variant::Secondary.small() on:click=move |_| close(true)>
                        {words.cancel.clone()}
                    </button>
                    <button
                        type="button"
                        class=Variant::Primary.small()
                        disabled=half
                        on:click=move |_| {
                            close(false);
                            submit_form(from_input);
                        }
                    >
                        {words.apply.clone()}
                    </button>
                </div>
            </div>
            }
            .into_any();
            view! {
                <div
                    class="fui-datepicker fui-datepicker-range"
                    class:is-start=move || flip.get()
                    id=panel_id.clone()
                    role="dialog"
                    aria-label=label.clone()
                    node_ref=panel
                    on:keydown=move |event: leptos::ev::KeyboardEvent| {
                        if event.key() == "Escape" {
                            event.prevent_default();
                            close(true);
                        }
                    }
                    on:mouseleave=move |_| hover.set(None)
                >
                    <span class="fui-datepicker-handle" aria-hidden="true"></span>
                    <p class="fui-datepicker-label">{heading}</p>
                    {body}
                    {foot}
                </div>
            }
            .into_any()
        })
    };
    view! {
        <div class="fui-date-field" node_ref=root>
            {fields}
            <button
                type="button"
                class=Variant::Secondary.class()
                hidden=move || !hydrated.get()
                aria-haspopup="dialog"
                aria-expanded=move || open.get().to_string()
                aria-controls=picker_id
                on:click=toggle
            >
                {icon(Glyph::CALENDAR, "16")}
                {summary}
            </button>
            {picker}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use leptos::prelude::Owner;

    use super::{DateWords, range_status, range_summary};
    use crate::calendar::{Bounds, Day};
    use crate::i18n::Phrase;

    fn day(text: &str) -> Day {
        Day::parse(text).expect("a test day")
    }

    fn words(with_time: bool) -> DateWords {
        Owner::new().with(|| DateWords::of_page(with_time))
    }

    #[test]
    fn a_template_fills_its_slots_in_order_and_keeps_its_words() {
        let sentence = Phrase::new(format!(
            "Back in the queue on {}.",
            Phrase::open_mark("day")
        ));
        assert_eq!(
            "Back in the queue on Wednesday 28 October 2026.",
            sentence.fill(&[("day", "Wednesday 28 October 2026")])
        );
        let words = words(false);
        assert_eq!(
            "October 2026",
            words.month_title(day("2026-10-01").month_of())
        );
        assert_eq!(
            "Wednesday 28 October 2026",
            words.day_name(day("2026-10-28"))
        );
    }

    #[test]
    fn a_typed_text_that_is_no_date_says_why() {
        let words = words(false);
        let open = Bounds::default();
        assert_eq!(
            None,
            words.reason("", false, open),
            "an empty field is no refusal"
        );
        assert_eq!(None, words.reason("2026-10-14", false, open));
        assert_eq!(
            Some(String::from(
                "2026-10-41 is not a date: October has 31 days."
            )),
            words.reason("2026-10-41", false, open)
        );
        assert_eq!(
            Some(String::from(
                "2028-02-30 is not a date: February has 29 days."
            )),
            words.reason("2028-02-30", false, open),
            "a leap year's February"
        );
        assert_eq!(
            Some(String::from(
                "2026-13-01 is not a date: there is no month 13."
            )),
            words.reason("2026-13-01", false, open)
        );
        assert_eq!(
            Some(String::from("14/10/2026 is not written as YYYY-MM-DD.")),
            words.reason("14/10/2026", false, open)
        );
        let timed = self::words(true);
        assert_eq!(
            Some(String::from(
                "2026-10-14 25:00 is not a time of day: type it as HH:MM."
            )),
            timed.reason("2026-10-14 25:00", true, open)
        );
    }

    #[test]
    fn a_date_past_the_fields_bounds_says_which_bound() {
        let words = words(false);
        let past = Bounds {
            earliest: None,
            latest: Some(day("2026-10-21")),
        };
        assert_eq!(
            Some(String::from(
                "2026-10-22 is after 2026-10-21, the latest date allowed."
            )),
            words.reason("2026-10-22", false, past)
        );
        let future = Bounds {
            earliest: Some(day("2026-10-22")),
            latest: None,
        };
        assert_eq!(
            Some(String::from(
                "2026-10-21 is before 2026-10-22, the earliest date allowed."
            )),
            words.reason("2026-10-21", false, future)
        );
    }

    #[test]
    fn a_range_says_its_days_and_what_to_choose_next() {
        let words = words(false);
        assert_eq!(
            "2026-10-15 to 2026-10-21 · 7 days.",
            range_status(&words, Some(day("2026-10-15")), Some(day("2026-10-21")))
        );
        assert_eq!(
            "2026-10-21 to 2026-10-21 · 1 day.",
            range_status(&words, Some(day("2026-10-21")), Some(day("2026-10-21")))
        );
        assert_eq!(
            "From 2026-10-15: now choose the end.",
            range_status(&words, Some(day("2026-10-15")), None)
        );
        assert_eq!("", range_status(&words, None, None));
        assert_eq!(
            "2026-10-15 to 2026-10-21",
            range_summary(&words, "2026-10-15", "2026-10-21")
        );
        assert_eq!("Any date", range_summary(&words, "", ""));
    }
}
