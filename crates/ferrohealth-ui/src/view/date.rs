// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The date field and its picker (R23, R25), and the date range of a
//! history filter (R22).
//!
//! A date field is a text input, year first, with its format stated after
//! its label and the calendar behind a button inside its edge (R10). The
//! typed text is the field's value: the form posts it as it stands, with
//! or without script. Until the `date` module runs, the button is disabled
//! and its name says to type the date. Once it runs, the button opens the
//! picker: a popover under the field from 40 rem, a bottom sheet below it,
//! with one month, its arrows, a month and year chooser behind the title,
//! and the field's quick choices or a time row. A range shows two months
//! with its presets beside them; until the module runs, its From and To
//! fields show in its button's place.
//!
//! The picker follows the WAI-ARIA Authoring Practices date picker dialog
//! pattern: a grid of days, each named by its full date, the arrows moving
//! a day or a week, Page Up and Page Down a month (with Shift a year), Home
//! and End to the week's ends, Enter choosing, and Escape closing with the
//! focus back on the field. Choosing a day writes the field; Cancel and
//! Escape put back what it held when the picker opened, and Apply keeps
//! it. A typed text that is no date, or a date the field does not allow,
//! marks the field invalid, says why under it, and disables Apply. The
//! server draws the field with the picker closed, and the `date` module
//! (the `date_range` module for a range) draws the picker from the words and
//! the days the server hands it. No
//! specification governs the picker's look: our own design.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::calendar::{Bounds, DateOrder, Day, Moment, Month, NotADay, Preset, Quick, Weekday};
use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::form::{ERROR, INNER_BUTTON, error_id, help_id};

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

/// Every word the date field and its picker draw, resolved on the server
/// in the page's locale ([`DateWords::of`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateWords {
    /// The format the field is typed in, its placeholder.
    pub format: Word,
    /// The order the page's language writes a date in, which a field takes
    /// beside `YYYY-MM-DD`.
    pub order: DateOrder,
    /// The format again, as the quiet hint after the label.
    pub format_hint: Word,
    /// The calendar button's name.
    pub choose: Word,
    /// The calendar button's name before the `date` module runs.
    pub needs_script: Word,
    /// The arrow to the month before.
    pub previous_month: Word,
    /// The arrow to the month after.
    pub next_month: Word,
    /// The month chooser's arrow to the year before.
    pub previous_year: Word,
    /// The month chooser's arrow to the year after.
    pub next_year: Word,
    /// The month chooser's title, back to the days.
    pub back_to_days: Word,
    /// The button that chooses today.
    pub today: Word,
    /// The button that empties the field.
    pub clear: Word,
    /// The button that closes the picker and keeps the field as it was.
    pub cancel: Word,
    /// The button that keeps the choice and closes the picker.
    pub apply: Word,
    /// The time row's label.
    pub time: Word,
    /// The time row's quiet hint.
    pub time_hint: Word,
    /// The hours' name.
    pub hours: Word,
    /// The minutes' name.
    pub minutes: Word,
    /// The quick choices' name.
    pub quick: Word,
    /// The presets' name.
    pub ranges: Word,
    /// A range button with no range.
    pub any: Word,
    /// The picker's line for a typed text that is no date.
    pub type_format: Word,
    /// The twelve months' names, January first.
    pub months: Vec<Word>,
    /// The seven days' names, Monday first.
    pub weekdays: Vec<Word>,
    /// The seven days' short names over the columns, Monday first.
    pub weekdays_short: Vec<Word>,
    /// How many days each month has, January to December, then February
    /// of a leap year.
    pub month_lengths: Vec<Word>,
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
    /// its plural on the server; a single field leaves them out.
    pub day_counts: Vec<Word>,
}

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
        let moment = match Moment::parse_in(text, with_time, self.order) {
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
    pub words: Word,
}

impl RangePreset {
    /// The preset `preset` with the library's words for it in the page's
    /// locale: "Today", "Last 7 days", "This month".
    #[must_use]
    pub fn of(locale: &Locale, preset: Preset) -> Self {
        let words = match preset {
            Preset::Today => tr(locale, Text::PresetToday),
            Preset::LastDays(count) => tr_with(
                locale,
                Text::PresetLastDays,
                &[("count", u32::from(count).into())],
            ),
            Preset::ThisMonth => tr(locale, Text::PresetThisMonth),
        };
        Self { preset, words }
    }
}

/// What a date field shows ([`date_field`]).
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
    /// The words it draws ([`DateWords::of`]).
    pub words: DateWords,
}

/// One end of a range: the input it posts and what it holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeEnd {
    /// The input's id.
    pub id: String,
    /// The name the input posts as.
    pub name: String,
    /// The label of the field that shows with no script.
    pub label: String,
    /// The day it holds, as the server has it.
    pub value: String,
}

/// What a date range shows ([`date_range`]).
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
    /// The presets beside the months ([`RangePreset::of`]).
    pub presets: Vec<RangePreset>,
    /// The words it draws ([`DateWords::of`]).
    pub words: DateWords,
}

/// The days and words the `date` module draws a single picker from.
#[derive(Serialize)]
struct FieldConfig<'a> {
    id: &'a str,
    label: &'a str,
    today: String,
    earliest: Option<String>,
    latest: Option<String>,
    first: u8,
    with_time: bool,
    quick: Vec<(Option<String>, &'a str)>,
    chosen: Option<&'a str>,
    words: &'a DateWords,
}

#[derive(Template)]
#[template(path = "date/date.html", block = "field")]
struct FieldView<'a> {
    config: String,
    id: &'a str,
    name: &'a str,
    label: &'a str,
    format_hint: &'a Word,
    format: &'a str,
    value: &'a str,
    error: Option<Word>,
    described: String,
    needs_script: &'a str,
    inner_button: &'static str,
    calendar: Html,
    help: Option<&'a str>,
    help_id: String,
    error_id: String,
    error_class: &'static str,
    icons: Html,
}

/// The icons the `date` module draws the picker's arrows with, in a
/// template the page never shows.
fn icons() -> Html {
    Html::raw(format!(
        "<template data-fui-icons><span data-icon=\"chevron-left\">{}</span><span data-icon=\"chevron-right\">{}</span></template>",
        icon(Glyph::CHEVRON_LEFT, "16"),
        icon(Glyph::CHEVRON_RIGHT, "16")
    ))
}

/// The data a module reads from an attribute, as JSON.
fn json<T: Serialize>(value: &T) -> String {
    // NOTE: serde_json `to_string` fails only for a map with a key that is no
    // string or a value whose `Serialize` fails, and these hold neither.
    serde_json::to_string(value).unwrap_or_default()
}

/// Returns the date field `field`, with its picker closed (R23, R25).
#[must_use]
pub fn date_field(field: DateField) -> Html {
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
    let config = json(&FieldConfig {
        id: &id,
        label: &label,
        today: today.to_string(),
        earliest: bounds.earliest.map(|day| day.to_string()),
        latest: bounds.latest.map(|day| day.to_string()),
        first: first_day.index(),
        with_time,
        quick: quick
            .iter()
            .map(|choice| {
                let day = choice
                    .quick
                    .day(today)
                    .filter(|day| bounds.allows(*day))
                    .map(|day| day.to_string());
                (day, choice.words.as_str())
            })
            .collect(),
        chosen: chosen
            .as_ref()
            .map(|chosen| chosen.fill(&[]).text().to_owned())
            .as_deref(),
        words: &words,
    });
    let mut described = Vec::new();
    if help.is_some() {
        described.push(help_id(&id));
    }
    described.push(error_id(&id));
    FieldView {
        config,
        id: &id,
        name: &name,
        label: &label,
        format_hint: &words.format_hint,
        format: words.format.text(),
        value: &value,
        error: error.map(Word::new),
        described: described.join(" "),
        needs_script: words.needs_script.text(),
        inner_button: INNER_BUTTON,
        calendar: icon(Glyph::CALENDAR, "16"),
        help: help.as_deref(),
        help_id: help_id(&id),
        error_id: error_id(&id),
        error_class: ERROR,
        icons: icons(),
    }
    .html()
}

/// The range's words for the button, or "Any date" when it holds none.
#[must_use]
pub fn range_summary(words: &DateWords, from: &str, to: &str) -> Word {
    match (Day::parse(from), Day::parse(to)) {
        (Ok(from), Ok(to)) => words
            .range_words
            .fill(&[("from", &from.to_string()), ("to", &to.to_string())]),
        _ => words.any.clone(),
    }
}

/// The range picker's line: the first day and what to do next, or the
/// whole range and the days it holds.
#[must_use]
pub fn range_status(words: &DateWords, first: Option<Day>, last: Option<Day>) -> Word {
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
        _ => Word::default(),
    }
}

/// The days and words the `date` module draws a range picker from.
#[derive(Serialize)]
struct RangeConfig<'a> {
    id: &'a str,
    label: &'a str,
    today: String,
    earliest: Option<String>,
    latest: Option<String>,
    first: u8,
    presets: Vec<(Option<(String, String)>, &'a str)>,
    words: &'a DateWords,
}

#[derive(Template)]
#[template(path = "date/date.html", block = "range")]
struct RangeView<'a> {
    config: String,
    id: &'a str,
    ends: [RangeEnd; 2],
    format_hint: &'a Word,
    format: &'a str,
    secondary: &'static str,
    calendar: Html,
    summary: Word,
    icons: Html,
}

/// Returns the date range `range` of a history filter with its picker
/// closed (R22, R25).
///
/// With no script its From and To fields show and post with the filter's
/// form. Once the `date` module runs, a secondary button naming the range
/// takes their place, the fields still posting, and opens the picker: the
/// presets beside two months. Apply writes both fields and submits the
/// form.
#[must_use]
pub fn date_range(range: DateRange) -> Html {
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
    let config = json(&RangeConfig {
        id: &id,
        label: &label,
        today: today.to_string(),
        earliest: bounds.earliest.map(|day| day.to_string()),
        latest: bounds.latest.map(|day| day.to_string()),
        first: first_day.index(),
        presets: presets
            .iter()
            .map(|preset| {
                let span = preset
                    .preset
                    .range(today)
                    .filter(|(first, last)| bounds.allows(*first) && bounds.allows(*last))
                    .map(|(first, last)| (first.to_string(), last.to_string()));
                (span, preset.words.text())
            })
            .collect(),
        words: &words,
    });
    let summary = range_summary(&words, &from.value, &to.value);
    RangeView {
        config,
        id: &id,
        ends: [from, to],
        format_hint: &words.format_hint,
        format: words.format.text(),
        secondary: Variant::Secondary.class(),
        calendar: icon(Glyph::CALENDAR, "16"),
        summary,
        icons: icons(),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{DateWords, range_status, range_summary};
    use crate::calendar::{Bounds, Day};
    use crate::i18n::{Locale, Phrase};

    fn day(text: &str) -> Day {
        Day::parse(text).expect("a test day")
    }

    fn words(with_time: bool) -> DateWords {
        let en = Locale::english();
        DateWords::of(&en, with_time).counted(&en)
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
            words.reason("2026-10-41", false, open).map(String::from),
        );
        assert_eq!(
            Some(String::from(
                "2028-02-30 is not a date: February has 29 days."
            )),
            words.reason("2028-02-30", false, open).map(String::from),
            "a leap year's February"
        );
        assert_eq!(
            Some(String::from(
                "2026-13-01 is not a date: there is no month 13."
            )),
            words.reason("2026-13-01", false, open).map(String::from),
        );
        assert_eq!(
            Some(String::from("14/10/2026 is not written as YYYY-MM-DD.")),
            words.reason("14/10/2026", false, open).map(String::from),
        );
        let timed = self::words(true);
        assert_eq!(
            Some(String::from(
                "2026-10-14 25:00 is not a time of day: type it as HH:MM."
            )),
            timed
                .reason("2026-10-14 25:00", true, open)
                .map(String::from)
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
            words.reason("2026-10-22", false, past).map(String::from),
        );
        let future = Bounds {
            earliest: Some(day("2026-10-22")),
            latest: None,
        };
        assert_eq!(
            Some(String::from(
                "2026-10-21 is before 2026-10-22, the earliest date allowed."
            )),
            words.reason("2026-10-21", false, future).map(String::from),
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
