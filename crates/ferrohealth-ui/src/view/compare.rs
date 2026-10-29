// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The record compare: several records side by side, one column per
//! record and one row per field, each value marked agree, differ or absent
//! against the leading record.
//!
//! A data steward reads where the records of one person disagree, such as
//! the source records of a patient in identifier domains of any country.
//! The table names its rows by the field (`th scope="row"`) and carries a
//! caption (WAI-ARIA 1.2, the `table` role); a mark is a glyph and a word,
//! the word visually hidden where agreement is quiet, never colour alone
//! (WCAG 2.2 SC 1.4.1). The columns have fixed widths, so a value that is
//! revealed or hidden in a cell moves nothing, and below 64 rem the table
//! scrolls sideways inside its card, never the page (SC 1.4.10), with the
//! field and the leading record held at the start. The stacked form reads
//! one record at a time under a choice of radios, which the stylesheet
//! answers with no script; past [`STACKED_MAX`] other records it lists
//! every record as a disclosure that opens with no script, so no record is
//! out of reach on a phone. A compare with no other record says so instead
//! of saying every field agrees. No specification governs the layout: our
//! own design.

use crate::html::Html;
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::status::badge;

/// One field the records are compared on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareField {
    /// The field's key, which the verdict names.
    pub key: String,
    /// The field's name, in the console's words.
    pub label: String,
}

/// One record: its title and a value per field, in the order of the
/// fields.
#[derive(Clone)]
pub struct CompareRecord {
    /// The record's key.
    pub key: String,
    /// The record's title, such as its source and domain.
    pub title: String,
    /// A quiet line under the title, when the record has one.
    pub sub: Option<String>,
    /// The record's value of each field, `None` where it has none, such as
    /// a masked identifier or a list of identifiers; it is drawn in each
    /// form that shows it.
    pub values: Vec<Option<Html>>,
}

impl std::fmt::Debug for CompareRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CompareRecord")
            .field("key", &self.key)
            .field("values", &self.values.len())
            .finish_non_exhaustive()
    }
}

/// The verdict on one record's value of one field, against the leading
/// record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The value agrees with the leading record's.
    Agree,
    /// The value differs from the leading record's.
    Differ,
    /// The record has no value where the leading record has one.
    Absent,
    /// No verdict is drawn.
    Unmarked,
}

impl Verdict {
    /// The mark's class, glyph and word, and whether the word shows.
    fn mark(self) -> Option<(&'static str, Glyph, Text, bool)> {
        match self {
            Self::Agree => Some((
                "fui-compare-mark fui-compare-agree",
                Glyph::CHECK,
                Text::CompareAgrees,
                false,
            )),
            Self::Differ => Some((
                "fui-compare-mark fui-compare-differ",
                Glyph::EQUAL_NOT,
                Text::CompareDiffers,
                true,
            )),
            Self::Absent => Some((
                "fui-compare-mark fui-compare-absent",
                Glyph::MINUS,
                Text::CompareAbsent,
                true,
            )),
            Self::Unmarked => None,
        }
    }
}

/// What a record compare shows.
#[derive(Clone, Copy)]
pub struct Compare<'a> {
    /// The compare's id, unique on the page, which names its parts.
    pub id: &'a str,
    /// The caption, which names the table.
    pub caption: &'a str,
    /// The fields, in order.
    pub fields: &'a [CompareField],
    /// The record the others are marked against.
    pub leading: &'a CompareRecord,
    /// The other records, any number of them.
    pub others: &'a [CompareRecord],
    /// The verdict on a field (by key) of the other record at an index.
    pub verdict: &'a dyn Fn(&str, usize) -> Verdict,
    /// Whether the rows where a record differs come first, then the rows
    /// where one is absent.
    pub differ_first: bool,
}

impl std::fmt::Debug for Compare<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Compare")
            .field("id", &self.id)
            .field("fields", &self.fields.len())
            .field("others", &self.others.len())
            .field("differ_first", &self.differ_first)
            .finish_non_exhaustive()
    }
}

/// The rank of a row by its verdicts: a difference first, then an absence,
/// then agreement, then none.
fn rank(verdicts: &[Verdict]) -> u8 {
    if verdicts.contains(&Verdict::Differ) {
        0
    } else if verdicts.contains(&Verdict::Absent) {
        1
    } else if verdicts.contains(&Verdict::Agree) {
        2
    } else {
        3
    }
}

/// One row of the compare: the field's place and every other record's
/// verdict on it.
struct Row<'a> {
    /// The field's index among the fields.
    index: usize,
    /// The field.
    field: &'a CompareField,
    /// The other records' verdicts, in their order.
    verdicts: Vec<Verdict>,
}

impl Row<'_> {
    /// Whether any record differs on this row.
    fn differs(&self) -> bool {
        self.verdicts.contains(&Verdict::Differ)
    }
}

/// The rows of `table`, the differing ones first when it asks.
fn rows<'a>(table: &Compare<'a>) -> Vec<Row<'a>> {
    let mut rows: Vec<Row<'a>> = table
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| Row {
            index,
            field,
            verdicts: (0..table.others.len())
                .map(|other| (table.verdict)(&field.key, other))
                .collect(),
        })
        .collect();
    if table.differ_first {
        rows.sort_by_key(|row| rank(&row.verdicts));
    }
    rows
}

/// Whether every verdict of `rows` is agreement or none: no record
/// differs and none is absent.
fn all_agree(rows: &[Row<'_>]) -> bool {
    rows.iter().all(|row| {
        row.verdicts
            .iter()
            .all(|verdict| matches!(verdict, Verdict::Agree | Verdict::Unmarked))
    })
}

/// The caption: the console's, the console's saying there is no other
/// record, or the console's saying every field agrees.
fn caption(locale: &Locale, table: &Compare<'_>, rows: &[Row<'_>]) -> Word {
    if table.others.is_empty() {
        tr_with(
            locale,
            Text::CompareNoOthers,
            &[("label", table.caption.to_owned().into())],
        )
    } else if all_agree(rows) {
        tr_with(
            locale,
            Text::CompareAllAgree,
            &[("label", table.caption.to_owned().into())],
        )
    } else {
        Word::new(table.caption)
    }
}

/// Returns the mark of `verdict`: its glyph and its word, the word shown
/// for a difference or an absence and visually hidden for agreement.
#[must_use]
pub fn mark(locale: &Locale, verdict: Verdict) -> Option<Html> {
    let (class, glyph, text, shown) = verdict.mark()?;
    let word = tr(locale, text);
    Some(Html::raw(format!(
        "<span title=\"{}\" class=\"{class}\">{}<span{}>{word}</span></span>",
        Html::text(word.text()),
        icon(glyph, "12"),
        if shown { "" } else { " class=\"fui-sr-only\"" }
    )))
}

/// Returns the words of a value that is not there: a dash for the eye and
/// the word "absent" for a reader.
fn none(locale: &Locale) -> Html {
    let word = tr(locale, Text::CompareAbsent);
    Html::raw(format!(
        "<span title=\"{}\" class=\"fui-compare-value fui-compare-none\"><span aria-hidden=\"true\">\u{2013}</span><span class=\"fui-sr-only\">{word}</span></span>",
        Html::text(word.text())
    ))
}

/// The value of `record` at field `index`, when it has one.
fn value_of(record: &CompareRecord, index: usize) -> Option<Html> {
    record.values.get(index).and_then(Option::clone)
}

/// Returns a cell of the table: the value and its mark.
fn cell(locale: &Locale, record: &CompareRecord, index: usize, verdict: Option<Verdict>) -> String {
    let leading = verdict.is_none();
    let marked = verdict.and_then(|verdict| mark(locale, verdict));
    let value = match value_of(record, index) {
        Some(value) => format!("<span class=\"fui-compare-value\">{value}</span>"),
        None if marked.is_some() => String::new(),
        None => String::from(none(locale)),
    };
    format!(
        "<td data-label=\"{}\"{}><span class=\"fui-compare-cell\">{value}{}</span></td>",
        Html::text(&record.title),
        if leading {
            " class=\"fui-compare-leading\""
        } else {
            ""
        },
        marked.unwrap_or_default()
    )
}

/// Returns a record's column header: its title, the leading badge, and its
/// quiet line.
fn head(locale: &Locale, record: &CompareRecord, leading: bool) -> String {
    let badge = if leading {
        String::from(badge(Tone::Accent, tr(locale, Text::CompareLeading)))
    } else {
        String::new()
    };
    let sub = record
        .sub
        .as_ref()
        .map(|sub| format!("<span class=\"fui-compare-sub\">{}</span>", Html::text(sub)))
        .unwrap_or_default();
    format!(
        "<th scope=\"col\"{}><span class=\"fui-compare-title\">{}{badge}</span>{sub}</th>",
        if leading {
            " class=\"fui-compare-leading\""
        } else {
            ""
        },
        Html::text(&record.title)
    )
}

/// Returns the compare `table` as a table: the field names down the first
/// column, the leading record next, every other record marked against it.
///
/// It shows at every width, scrolling sideways inside its card when the
/// card is narrower than its columns, the field and the leading record held
/// in view. A caption whose fields all agree says so.
#[must_use]
pub fn compare(locale: &Locale, table: Compare<'_>) -> Html {
    drawn_table(locale, &table, false)
}

/// The table [`compare`] and [`compare_pair`] draw; `pair` hides it below
/// 64 rem, where the stacked form takes its place.
fn drawn_table(locale: &Locale, table: &Compare<'_>, pair: bool) -> Html {
    let rows = rows(table);
    let caption = caption(locale, table, &rows);
    let heads: String = std::iter::once(head(locale, table.leading, true))
        .chain(table.others.iter().map(|other| head(locale, other, false)))
        .collect::<Vec<_>>()
        .concat();
    let columns = "<col class=\"fui-compare-col\">".repeat(table.others.len());
    let body: String = rows
        .iter()
        .map(|row| {
            let cells: String = table
                .others
                .iter()
                .zip(&row.verdicts)
                .map(|(other, verdict)| cell(locale, other, row.index, Some(*verdict)))
                .collect::<Vec<_>>()
                .concat();
            format!(
                "<tr{}><th scope=\"row\">{}</th>{}{cells}</tr>",
                if row.differs() {
                    " class=\"fui-compare-row-differ\""
                } else {
                    ""
                },
                Html::text(&row.field.label),
                cell(locale, table.leading, row.index, None)
            )
        })
        .collect::<Vec<_>>()
        .concat();
    let class = if pair {
        "fui-table-scroll fui-compare-scroll fui-compare-wide"
    } else {
        "fui-table-scroll fui-compare-scroll"
    };
    Html::raw(format!(
        "<div role=\"region\" aria-label=\"{}\" tabindex=\"0\" class=\"{class}\"><table class=\"fui-compare\"><caption>{caption}</caption><colgroup><col class=\"fui-compare-col-field\"><col class=\"fui-compare-col\">{columns}</colgroup><thead><tr><th scope=\"col\">{}</th>{heads}</tr></thead><tbody>{body}</tbody></table></div>",
        Html::text(caption.text()),
        tr(locale, Text::CompareField)
    ))
}

/// The most other records the stacked form offers as a choice; past it,
/// each record is a disclosure of its own.
pub const STACKED_MAX: usize = crate::view::segmented::REVEAL_MAX;

/// Returns the compare `table` one record at a time, for a phone.
///
/// A choice of the other records leads, `shown` chosen first (the last
/// record when `shown` is past the end), and the chosen record's fields
/// follow, marked against the leading record, whose value follows each one
/// that differs.
///
/// The choice is a native radio group the stylesheet answers, so it works
/// with no script. Past [`STACKED_MAX`] other records every record is a
/// native disclosure instead, `shown` open, so every record is offered
/// with no script. Every mark sits at the end of its value's line, so the
/// marks read as one column.
#[must_use]
pub fn compare_stacked(locale: &Locale, table: Compare<'_>, shown: usize) -> Html {
    stacked(locale, &table, shown, false)
}

/// Returns the compare `table` as the table from 64 rem and the stacked
/// form below it, `shown` the record the stacked form reads first.
#[must_use]
pub fn compare_pair(locale: &Locale, table: Compare<'_>, shown: usize) -> Html {
    drawn_table(locale, &table, true).then(stacked(locale, &table, shown, true))
}

/// The stacked form; `pair` hides it from 64 rem, where the table shows.
fn stacked(locale: &Locale, table: &Compare<'_>, shown: usize, pair: bool) -> Html {
    let rows = rows(table);
    let shown = shown.min(table.others.len().saturating_sub(1));
    let name = format!("{}-read", table.id);
    let class = if pair {
        "fui-compare-stacked"
    } else {
        "fui-compare-stacked fui-compare-alone"
    };
    if table.others.is_empty() {
        let none = tr_with(
            locale,
            Text::CompareNoOthers,
            &[("label", table.caption.to_owned().into())],
        );
        return Html::raw(format!(
            "<div class=\"{class}\"><p class=\"fui-small fui-muted\">{none}</p></div>"
        ));
    }
    let against = tr_with(
        locale,
        Text::CompareMarkedAgainst,
        &[("title", table.leading.title.clone().into())],
    );
    if table.others.len() > STACKED_MAX {
        let records = disclosures(locale, table, &rows, shown, &name);
        return Html::raw(format!(
            "<div class=\"{class}\"><p class=\"fui-small fui-muted\">{against}</p><div class=\"fui-compare-records\">{records}</div></div>"
        ));
    }
    let name_text = Html::text(&name);
    let segments: String = table
        .others
        .iter()
        .enumerate()
        .map(|(index, record)| {
            format!(
                "<label for=\"{name_text}-{index}\" class=\"fui-segment\"><input type=\"radio\" id=\"{name_text}-{index}\" name=\"{name_text}\" value=\"{}\"{} class=\"fui-sr-only\">{}</label>",
                Html::text(&record.key),
                if index == shown { " checked" } else { "" },
                Html::text(&record.title)
            )
        })
        .collect::<Vec<_>>()
        .concat();
    let parts: String = table
        .others
        .iter()
        .enumerate()
        .map(|(other, record)| {
            format!(
                "<div class=\"fui-revealed\">{}</div>",
                record_rows(locale, table, &rows, other, record)
            )
        })
        .collect::<Vec<_>>()
        .concat();
    Html::raw(format!(
        "<div class=\"{class}\"><div class=\"fui-reveal\"><fieldset class=\"fui-fieldset fui-choice\"><legend class=\"fui-label\">{}</legend><div class=\"fui-segmented\">{segments}</div></fieldset><p class=\"fui-small fui-muted\">{against}</p><div class=\"fui-reveal-parts\">{parts}</div></div></div>",
        tr(locale, Text::CompareRecordToRead)
    ))
}

/// Returns every other record of `table` as a native disclosure, one open
/// at a time, `shown` open first.
fn disclosures(
    locale: &Locale,
    table: &Compare<'_>,
    rows: &[Row<'_>],
    shown: usize,
    name: &str,
) -> String {
    table
        .others
        .iter()
        .enumerate()
        .map(|(other, record)| {
            let sub = record
                .sub
                .as_ref()
                .map(|sub| format!("<span class=\"fui-compare-sub\">{}</span>", Html::text(sub)))
                .unwrap_or_default();
            format!(
                "<details name=\"{}\"{} class=\"fui-disclosure fui-compare-record\"><summary><span class=\"fui-compare-title\">{}</span>{sub}</summary><div>{}</div></details>",
                Html::text(name),
                if other == shown { " open" } else { "" },
                Html::text(&record.title),
                record_rows(locale, table, rows, other, record)
            )
        })
        .collect::<Vec<_>>()
        .concat()
}

/// Returns the fields of `record`, the other record at index `other`, as
/// the stacked form reads them: each marked against the leading record,
/// whose value follows each one that differs.
fn record_rows(
    locale: &Locale,
    table: &Compare<'_>,
    rows: &[Row<'_>],
    other: usize,
    record: &CompareRecord,
) -> String {
    let lines: String = rows
        .iter()
        .map(|row| {
            let verdict = row.verdicts.get(other).copied().unwrap_or(Verdict::Unmarked);
            let differs = verdict == Verdict::Differ;
            let value = value_of(record, row.index).unwrap_or_default();
            let leading = if differs {
                let lead = value_of(table.leading, row.index).unwrap_or_else(|| none(locale));
                format!(
                    "<span class=\"fui-small fui-muted fui-compare-lead\">{} {lead}</span>",
                    tr_with(
                        locale,
                        Text::CompareLeadingValue,
                        &[("title", table.leading.title.clone().into())]
                    )
                )
            } else {
                String::new()
            };
            format!(
                "<div{}><dt>{}</dt><dd><span class=\"fui-compare-cell\"><span class=\"fui-compare-value\">{value}</span>{}</span>{leading}</dd></div>",
                if differs {
                    " class=\"fui-compare-row-differ\""
                } else {
                    ""
                },
                Html::text(&row.field.label),
                mark(locale, verdict).unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .concat();
    format!("<dl class=\"fui-compare-stacked-rows\">{lines}</dl>")
}

#[cfg(test)]
mod tests {
    use super::{Verdict, rank};

    #[test]
    fn a_difference_ranks_before_an_absence_before_agreement() {
        assert_eq!(0, rank(&[Verdict::Agree, Verdict::Differ]));
        assert_eq!(1, rank(&[Verdict::Absent, Verdict::Agree]));
        assert_eq!(2, rank(&[Verdict::Agree, Verdict::Unmarked]));
        assert_eq!(3, rank(&[Verdict::Unmarked]));
    }
}
