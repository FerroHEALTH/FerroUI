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

use leptos::prelude::*;

use crate::i18n::{Text, tr, tr_with};
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
    /// The record's value of each field, `None` where it has none. A value
    /// is drawn each time a form shows it, so it may be an island such as
    /// a masked identifier with its reveal, or a list of identifiers.
    pub values: Vec<Option<ViewFn>>,
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
fn caption(table: &Compare<'_>, rows: &[Row<'_>]) -> String {
    if table.others.is_empty() {
        tr_with(
            Text::CompareNoOthers,
            &[("label", table.caption.to_owned().into())],
        )
    } else if all_agree(rows) {
        tr_with(
            Text::CompareAllAgree,
            &[("label", table.caption.to_owned().into())],
        )
    } else {
        table.caption.to_owned()
    }
}

/// Returns the mark of `verdict`: its glyph and its word, the word shown
/// for a difference or an absence and visually hidden for agreement.
#[must_use]
pub fn mark(verdict: Verdict) -> Option<AnyView> {
    let (class, glyph, text, shown) = verdict.mark()?;
    let word = tr(text);
    let title = word.clone();
    Some(
        view! {
            <span class=class title=title>
                {icon(glyph, "12")}
                <span class:fui-sr-only=!shown>{word}</span>
            </span>
        }
        .into_any(),
    )
}

/// Returns the words of a value that is not there: a dash for the eye and
/// the word "absent" for a reader.
fn none() -> AnyView {
    let word = tr(Text::CompareAbsent);
    let title = word.clone();
    view! {
        <span class="fui-compare-value fui-compare-none" title=title>
            <span aria-hidden="true">"\u{2013}"</span>
            <span class="fui-sr-only">{word}</span>
        </span>
    }
    .into_any()
}

/// The value of `record` at field `index`, when it has one.
fn value_of(record: &CompareRecord, index: usize) -> Option<AnyView> {
    record
        .values
        .get(index)
        .and_then(Option::as_ref)
        .map(ViewFn::run)
}

/// Returns a cell of the table: the value and its mark.
fn cell(record: &CompareRecord, index: usize, verdict: Option<Verdict>) -> AnyView {
    let leading = verdict.is_none();
    let marked = verdict.and_then(mark);
    let value = match value_of(record, index) {
        Some(value) => Some(view! { <span class="fui-compare-value">{value}</span> }.into_any()),
        None if marked.is_some() => None,
        None => Some(none()),
    };
    view! {
        <td class:fui-compare-leading=leading data-label=record.title.clone()>
            <span class="fui-compare-cell">{value} {marked}</span>
        </td>
    }
    .into_any()
}

/// Returns a record's column header: its title, the leading badge, and its
/// quiet line.
fn head(record: &CompareRecord, leading: bool) -> AnyView {
    let badge = leading.then(|| badge(Tone::Accent, &tr(Text::CompareLeading)));
    let sub = record
        .sub
        .clone()
        .map(|sub| view! { <span class="fui-compare-sub">{sub}</span> });
    view! {
        <th scope="col" class:fui-compare-leading=leading>
            <span class="fui-compare-title">{record.title.clone()} {badge}</span>
            {sub}
        </th>
    }
    .into_any()
}

/// Returns the compare `table` as a table: the field names down the first
/// column, the leading record next, every other record marked against it.
///
/// It shows at every width, scrolling sideways inside its card when the
/// card is narrower than its columns, the field and the leading record held
/// in view. A caption whose fields all agree says so.
#[must_use]
pub fn compare(table: Compare<'_>) -> AnyView {
    drawn_table(&table, false)
}

/// The table [`compare`] and [`compare_pair`] draw; `pair` hides it below
/// 64 rem, where the stacked form takes its place.
fn drawn_table(table: &Compare<'_>, pair: bool) -> AnyView {
    let rows = rows(table);
    let caption = caption(table, &rows);
    let label = caption.clone();
    let heads = std::iter::once(head(table.leading, true))
        .chain(table.others.iter().map(|other| head(other, false)))
        .collect_view()
        .into_any();
    let columns = (0..table.others.len())
        .map(|_| view! { <col class="fui-compare-col" /> })
        .collect_view()
        .into_any();
    let body = rows
        .iter()
        .map(|row| {
            let cells = table
                .others
                .iter()
                .zip(&row.verdicts)
                .map(|(other, verdict)| cell(other, row.index, Some(*verdict)))
                .collect_view()
                .into_any();
            view! {
                <tr class:fui-compare-row-differ=row.differs()>
                    <th scope="row">{row.field.label.clone()}</th>
                    {cell(table.leading, row.index, None)}
                    {cells}
                </tr>
            }
        })
        .collect_view()
        .into_any();
    let class = if pair {
        "fui-table-scroll fui-compare-scroll fui-compare-wide"
    } else {
        "fui-table-scroll fui-compare-scroll"
    };
    view! {
        <div class=class role="region" aria-label=label tabindex="0">
            <table class="fui-compare">
                <caption>{caption}</caption>
                <colgroup>
                    <col class="fui-compare-col-field" />
                    <col class="fui-compare-col" />
                    {columns}
                </colgroup>
                <thead>
                    <tr>
                        <th scope="col">{tr(Text::CompareField)}</th>
                        {heads}
                    </tr>
                </thead>
                <tbody>{body}</tbody>
            </table>
        </div>
    }
    .into_any()
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
pub fn compare_stacked(table: Compare<'_>, shown: usize) -> AnyView {
    stacked(&table, shown, false)
}

/// Returns the compare `table` as the table from 64 rem and the stacked
/// form below it, `shown` the record the stacked form reads first.
#[must_use]
pub fn compare_pair(table: Compare<'_>, shown: usize) -> AnyView {
    let wide = drawn_table(&table, true);
    let narrow = stacked(&table, shown, true);
    view! {
        {wide}
        {narrow}
    }
    .into_any()
}

/// The stacked form; `pair` hides it from 64 rem, where the table shows.
fn stacked(table: &Compare<'_>, shown: usize, pair: bool) -> AnyView {
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
            Text::CompareNoOthers,
            &[("label", table.caption.to_owned().into())],
        );
        return view! {
            <div class=class>
                <p class="fui-small fui-muted">{none}</p>
            </div>
        }
        .into_any();
    }
    let against = tr_with(
        Text::CompareMarkedAgainst,
        &[("title", table.leading.title.clone().into())],
    );
    if table.others.len() > STACKED_MAX {
        let records = disclosures(table, &rows, shown, &name);
        return view! {
            <div class=class>
                <p class="fui-small fui-muted">{against}</p>
                <div class="fui-compare-records">{records}</div>
            </div>
        }
        .into_any();
    }
    let segments = table
        .others
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let id = format!("{name}-{index}");
            let label_for = id.clone();
            view! {
                <label for=label_for class="fui-segment">
                    <input
                        type="radio"
                        id=id
                        name=name.clone()
                        value=record.key.clone()
                        checked=index == shown
                        class="fui-sr-only"
                    />
                    {record.title.clone()}
                </label>
            }
            .into_any()
        })
        .collect_view()
        .into_any();
    let parts = table
        .others
        .iter()
        .enumerate()
        .map(|(other, record)| {
            view! { <div class="fui-revealed">{record_rows(table, &rows, other, record)}</div> }
                .into_any()
        })
        .collect_view()
        .into_any();
    view! {
        <div class=class>
            <div class="fui-reveal">
                <fieldset class="fui-fieldset fui-choice">
                    <legend class="fui-label">{tr(Text::CompareRecordToRead)}</legend>
                    <div class="fui-segmented">{segments}</div>
                </fieldset>
                <p class="fui-small fui-muted">{against}</p>
                <div class="fui-reveal-parts">{parts}</div>
            </div>
        </div>
    }
    .into_any()
}

/// Returns every other record of `table` as a native disclosure, one open
/// at a time, `shown` open first.
fn disclosures(table: &Compare<'_>, rows: &[Row<'_>], shown: usize, name: &str) -> AnyView {
    table
        .others
        .iter()
        .enumerate()
        .map(|(other, record)| {
            let sub = record
                .sub
                .clone()
                .map(|sub| view! { <span class="fui-compare-sub">{sub}</span> });
            view! {
                <details class="fui-disclosure fui-compare-record" name=name.to_owned() open=other == shown>
                    <summary>
                        <span class="fui-compare-title">{record.title.clone()}</span>
                        {sub}
                    </summary>
                    <div>{record_rows(table, rows, other, record)}</div>
                </details>
            }
            .into_any()
        })
        .collect_view()
        .into_any()
}

/// Returns the fields of `record`, the other record at index `other`, as
/// the stacked form reads them: each marked against the leading record,
/// whose value follows each one that differs.
fn record_rows(
    table: &Compare<'_>,
    rows: &[Row<'_>],
    other: usize,
    record: &CompareRecord,
) -> AnyView {
    let lines = rows
        .iter()
        .map(|row| {
            let verdict = row.verdicts.get(other).copied().unwrap_or(Verdict::Unmarked);
            let differs = verdict == Verdict::Differ;
            let value = value_of(record, row.index);
            let leading = differs.then(|| {
                let lead = value_of(table.leading, row.index).unwrap_or_else(none);
                view! {
                    <span class="fui-small fui-muted fui-compare-lead">
                        {tr_with(Text::CompareLeadingValue, &[("title", table.leading.title.clone().into())])}
                        " "
                        {lead}
                    </span>
                }
                .into_any()
            });
            view! {
                <div class:fui-compare-row-differ=differs>
                    <dt>{row.field.label.clone()}</dt>
                    <dd>
                        <span class="fui-compare-cell">
                            <span class="fui-compare-value">{value}</span>
                            {mark(verdict)}
                        </span>
                        {leading}
                    </dd>
                </div>
            }
            .into_any()
        })
        .collect_view()
        .into_any();
    view! { <dl class="fui-compare-stacked-rows">{lines}</dl> }.into_any()
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
