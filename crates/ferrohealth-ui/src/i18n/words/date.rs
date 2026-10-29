// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the date field and its picker.

use crate::i18n::Text;

texts! {
    /// A typed date after the latest the field allows.
    DateAfter = "date-after",
    /// A range button with no range chosen.
    DateAny = "date-any",
    /// The picker's button that keeps the choice and closes it.
    DateApply = "date-apply",
    /// The name of the month chooser's title, which goes back to the days.
    DateBackToDays = "date-back-to-days",
    /// A typed date before the earliest the field allows.
    DateBefore = "date-before",
    /// The picker's button that closes it and keeps the field as it was.
    DateCancel = "date-cancel",
    /// The name of the button that opens the date picker.
    DateChoose = "date-choose",
    /// The name of the picker's title, which opens the month chooser.
    DateChooseMonth = "date-choose-month",
    /// The picker's button that empties the field.
    DateClear = "date-clear",
    /// A day's full name, its accessible name in the grid.
    DateDayName = "date-day-name",
    /// A number of days.
    DateDays = "date-days",
    /// The format a date field is typed in, stated after its label.
    DateFormat = "date-format",
    /// The quiet hint after a date field's label, its format.
    DateFormatHint = "date-format-hint",
    /// The name of the time row's hours.
    DateHours = "date-hours",
    /// The name of the time row's minutes.
    DateMinutes = "date-minutes",
    /// How many days a month has.
    DateMonthLength = "date-month-length",
    /// A month's title over its days.
    DateMonthTitle = "date-month-title",
    /// The name of the month chooser's grid.
    DateMonthsOf = "date-months-of",
    /// The name of the date picker's button before the `date` module runs.
    DateNeedsScript = "date-needs-script",
    /// The picker's arrow to the month after.
    DateNextMonth = "date-next-month",
    /// The month chooser's arrow to the year after.
    DateNextYear = "date-next-year",
    /// A typed month that is not one of the twelve.
    DateNoMonth = "date-no-month",
    /// A typed day its month does not have, and why.
    DateNotADay = "date-not-a-day",
    /// A typed time that is no time of day.
    DateNotATime = "date-not-a-time",
    /// A typed text not written as a date.
    DateNotFormat = "date-not-format",
    /// A typed text not written as a date and time.
    DateNotFormatTime = "date-not-format-time",
    /// The order the language writes a date in, beside ISO 8601: `ymd`,
    /// `dmy` or `mdy`, never shown.
    DateOrder = "date-order",
    /// The picker's arrow to the month before.
    DatePreviousMonth = "date-previous-month",
    /// The month chooser's arrow to the year before.
    DatePreviousYear = "date-previous-year",
    /// The name of the picker's quick choices.
    DateQuick = "date-quick",
    /// The range picker's line once both days are chosen, with the days the range holds.
    DateRangeFull = "date-range-full",
    /// The range picker's line once the first day is chosen.
    DateRangeHalf = "date-range-half",
    /// A range, its first and last day.
    DateRangeWords = "date-range-words",
    /// The name of the range picker's presets.
    DateRanges = "date-ranges",
    /// The label of the picker's time row.
    DateTime = "date-time",
    /// The format a date and time field is typed in.
    DateTimeFormat = "date-time-format",
    /// The quiet hint after a date and time field's label, its format.
    DateTimeFormatHint = "date-time-format-hint",
    /// The quiet hint after the time row's label.
    DateTimeHint = "date-time-hint",
    /// The picker's button that chooses today.
    DateToday = "date-today",
    /// The picker's line for a typed text that is no date.
    DateTypeFormat = "date-type-format",
    /// The picker's line for a typed text that is no date and time.
    DateTypeFormatTime = "date-type-format-time",
    /// The name of month 1.
    Month1 = "month-1",
    /// The name of month 10.
    Month10 = "month-10",
    /// The name of month 11.
    Month11 = "month-11",
    /// The name of month 12.
    Month12 = "month-12",
    /// The name of month 2.
    Month2 = "month-2",
    /// The name of month 3.
    Month3 = "month-3",
    /// The name of month 4.
    Month4 = "month-4",
    /// The name of month 5.
    Month5 = "month-5",
    /// The name of month 6.
    Month6 = "month-6",
    /// The name of month 7.
    Month7 = "month-7",
    /// The name of month 8.
    Month8 = "month-8",
    /// The name of month 9.
    Month9 = "month-9",
    /// The range preset of today and the days before it.
    PresetLastDays = "preset-last-days",
    /// The range preset of the month so far.
    PresetThisMonth = "preset-this-month",
    /// The range preset of today alone.
    PresetToday = "preset-today",
    /// The name of Friday.
    WeekdayFriday = "weekday-friday",
    /// The name of Monday.
    WeekdayMonday = "weekday-monday",
    /// The name of Saturday.
    WeekdaySaturday = "weekday-saturday",
    /// The two-letter name of Friday over a column of days.
    WeekdayShortFriday = "weekday-short-friday",
    /// The two-letter name of Monday over a column of days.
    WeekdayShortMonday = "weekday-short-monday",
    /// The two-letter name of Saturday over a column of days.
    WeekdayShortSaturday = "weekday-short-saturday",
    /// The two-letter name of Sunday over a column of days.
    WeekdayShortSunday = "weekday-short-sunday",
    /// The two-letter name of Thursday over a column of days.
    WeekdayShortThursday = "weekday-short-thursday",
    /// The two-letter name of Tuesday over a column of days.
    WeekdayShortTuesday = "weekday-short-tuesday",
    /// The two-letter name of Wednesday over a column of days.
    WeekdayShortWednesday = "weekday-short-wednesday",
    /// The name of Sunday.
    WeekdaySunday = "weekday-sunday",
    /// The name of Thursday.
    WeekdayThursday = "weekday-thursday",
    /// The name of Tuesday.
    WeekdayTuesday = "weekday-tuesday",
    /// The name of Wednesday.
    WeekdayWednesday = "weekday-wednesday",
}
