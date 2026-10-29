// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The repeatable rows (F7): a list of the same few fields, each row
//! removable by its own labelled button at its end, and one Add button
//! under the list.
//!
//! A row's Remove is its last control; on a phone the fields stack and
//! Remove stays on the row's first line, beside the first field. A row the
//! list keeps ends in a disabled lock, hidden from a reader, and says why
//! in a visible line under it, never in a tooltip alone (F13's rule for a
//! disabled item). M4's
//! filter rows and M6's target rows are F7 rows ([`Fields::Three`]). A
//! row of one field ([`Fields::One`]) keeps the field and its Remove on
//! one line, and a row of many fields is a sub-card ([`Fields::Card`]): its
//! heading with Remove beside it, then its fields, a list of its own among
//! them when it has one.
//!
//! Remove and Add are submit buttons of the console's form, so with no
//! script a press posts the form with the button's `name` and `value` and
//! the console answers with the list one row shorter or longer. The
//! `repeat` module does both in place: Remove takes its row out and moves the focus to the next row's
//! first control (the previous row's at the end, Add when none is left),
//! and Add inserts a copy of the blank row, its key token
//! ([`Repeat::token`], [`ROW`] by default) replaced by a fresh key, and
//! focuses its first control. A sub-card's first control is its first
//! field, never the Remove in its head. A list nested in a sub-card has a
//! token of its own, so a copied card's nested list adds its rows in place
//! too. Either way the console reads the rows the form posts as the whole
//! list. No specification governs the layout: our own design.

use crate::html::Html;
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// The key token of a list's blank row, which the module replaces in the
/// row's every id, name and key with `new-1`, `new-2` and so on, the first
/// one no row of the list holds.
///
/// A list nested in a [`Fields::Card`] row names a token of its own in
/// [`Repeat::token`], one that neither holds nor is held in the token of
/// the list around it.
pub const ROW: &str = "__row__";

/// How a row's fields share its width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fields {
    /// One field across the row, such as a profile's URL or one value of a
    /// list setting.
    One,
    /// Two fields, the second the wider, such as a name and its value.
    #[default]
    Two,
    /// Three fields, such as a filter's property, operator and value, or a
    /// mapping's target, relationship and comment.
    Three,
    /// A sub-card: the row's heading ([`RepeatRow::heading`]) with Remove
    /// beside it, then its fields as a form lays them out, such as an
    /// endpoint with its id, its base URL, a switch and a list of its own.
    Card,
}

impl Fields {
    /// The classes of a row.
    const fn row(self) -> &'static str {
        match self {
            Self::One => "fui-repeater-row fui-repeater-one",
            Self::Two => "fui-repeater-row",
            Self::Three => "fui-clause-filter",
            Self::Card => "fui-repeater-card",
        }
    }

    /// The classes of the list of rows.
    const fn rows(self) -> &'static str {
        match self {
            Self::One | Self::Two | Self::Card => "fui-repeat-rows fui-repeater",
            Self::Three => "fui-repeat-rows",
        }
    }
}

/// What ends a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowEnd {
    /// Its Remove: an icon button named `label`, such as "Remove this
    /// filter", that submits `name` with `value`.
    Remove {
        /// The button's name, which says which row it removes.
        label: String,
        /// The field it submits as.
        name: String,
        /// The value it submits, the row's key.
        value: String,
    },
    /// A row the list keeps: a disabled lock at its end and `reason` as a
    /// visible line under it, such as a property the editor writes itself.
    Kept {
        /// Why the row cannot be removed.
        reason: String,
    },
}

/// One row of the list.
#[derive(Debug)]
pub struct RepeatRow {
    /// The row's key, stable for the row and unique in the list.
    pub key: String,
    /// The heading of a [`Fields::Card`] row, such as "Query API endpoint
    /// node-c-pub", which names its group; the other layouts draw none.
    pub heading: Option<String>,
    /// The row's fields, from [`crate::view::form::Field`] with ids made
    /// from the key.
    pub fields: Html,
    /// What ends the row.
    pub end: RowEnd,
    /// A line under the row, such as its error, which goes with it.
    pub after: Option<Html>,
}

/// The Add button under the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatAdd {
    /// Its words, such as "Add filter".
    pub label: String,
    /// The field it submits as.
    pub name: String,
    /// The value it submits.
    pub value: String,
}

/// What a list of repeatable rows shows.
#[derive(Debug)]
pub struct Repeat {
    /// The list's id.
    pub id: String,
    /// The list's name, such as "Filters of the include".
    pub label: String,
    /// How each row's fields share its width.
    pub fields: Fields,
    /// The headings over the fields of [`Fields::One`] or [`Fields::Two`]
    /// rows whose labels are hidden, one per field; empty for rows whose
    /// fields show their labels.
    pub head: Vec<String>,
    /// The rows, in order.
    pub rows: Vec<RepeatRow>,
    /// The key token of the blank row: [`ROW`], or a token of its own for
    /// a list nested in a [`Fields::Card`] row, such as `__profile__`.
    pub token: &'static str,
    /// The blank row Add inserts once the module runs, its key, ids and names
    /// holding [`Repeat::token`]; `None` leaves Add to the form post.
    pub blank: Option<RepeatRow>,
    /// The Add button, if the list grows here.
    pub add: Option<RepeatAdd>,
    /// The quiet line the list shows with no row, such as "No filters
    /// yet."
    pub empty: String,
    /// The id of the form Remove and Add submit, when the list is not
    /// inside it.
    pub form: Option<String>,
}

/// Returns the end of a row.
fn row_end(end: RowEnd, form: Option<&str>) -> String {
    match end {
        RowEnd::Remove { label, name, value } => {
            let label = Html::text(label);
            format!(
                "<button type=\"submit\" name=\"{}\" value=\"{}\"{} formnovalidate aria-label=\"{label}\" title=\"{label}\" data-repeat=\"remove\" class=\"fui-icon-button\">{}</button>",
                Html::text(name),
                Html::text(value),
                form.map(|form| format!(" form=\"{}\"", Html::text(form)))
                    .unwrap_or_default(),
                icon(Glyph::X, "16")
            )
        }
        RowEnd::Kept { .. } => format!(
            "<button type=\"button\" disabled aria-hidden=\"true\" class=\"fui-icon-button\">{}</button>",
            icon(Glyph::LOCK, "16")
        ),
    }
}

/// Returns the row `row` of the list `list`, laid out as `fields`.
fn row(list: &str, row: RepeatRow, fields: Fields, form: Option<&str>) -> String {
    let RepeatRow {
        key,
        heading,
        fields: inside,
        end,
        after,
    } = row;
    let kept = match &end {
        RowEnd::Kept { reason } => format!(
            "<p class=\"fui-help fui-repeat-kept\">{}</p>",
            Html::text(reason)
        ),
        RowEnd::Remove { .. } => String::new(),
    };
    let end = row_end(end, form);
    let body = if fields == Fields::Card {
        let heading_id = Html::text(format!("{list}-{key}-heading"));
        format!(
            "<div role=\"group\" aria-labelledby=\"{heading_id}\" class=\"{}\"><div class=\"fui-repeater-card-head\"><h3 id=\"{heading_id}\">{}</h3>{end}</div>{inside}</div>",
            fields.row(),
            Html::text(heading.unwrap_or_default())
        )
    } else {
        format!("<div class=\"{}\">{inside}{end}</div>", fields.row())
    };
    format!(
        "<div data-row=\"{}\" class=\"fui-repeat-item\">{body}{kept}{}</div>",
        Html::text(&key),
        after.unwrap_or_default()
    )
}

/// Returns the list `repeat`: its rows, the blank row the module copies,
/// the quiet line for no row and Add.
#[must_use]
pub fn repeat(repeat: Repeat) -> Html {
    let Repeat {
        id,
        label,
        fields,
        head,
        rows,
        token,
        blank,
        add,
        empty,
        form,
    } = repeat;
    let none = rows.is_empty();
    let form = form.as_deref();
    let head = if matches!(fields, Fields::One | Fields::Two) && !head.is_empty() {
        let labels: String = head
            .into_iter()
            .map(|words| format!("<span class=\"fui-label\">{}</span>", Html::text(words)))
            .collect::<Vec<_>>()
            .concat();
        format!(
            "<div aria-hidden=\"true\" class=\"fui-repeater-head{}\">{labels}</div>",
            if fields == Fields::One { " is-one" } else { "" }
        )
    } else {
        String::new()
    };
    let drawn: String = rows
        .into_iter()
        .map(|each| row(&id, each, fields, form))
        .collect::<Vec<_>>()
        .concat();
    // NOTE: HTML Living Standard, the fieldset element: a disabled fieldset's controls
    // post nothing, so the hidden blank row never submits.
    let blank = blank
        .map(|each| {
            format!(
                "<fieldset data-repeat=\"blank\" data-token=\"{token}\" hidden disabled class=\"fui-repeat-blank\">{}</fieldset>",
                row(&id, each, fields, form)
            )
        })
        .unwrap_or_default();
    let add = add
        .map(|RepeatAdd { label, name, value }| {
            format!(
                "<div class=\"fui-add-row\"><button type=\"submit\" name=\"{}\" value=\"{}\"{} formnovalidate data-repeat=\"add\" class=\"{}\">{}{}</button></div>",
                Html::text(name),
                Html::text(value),
                form.map(|form| format!(" form=\"{}\"", Html::text(form)))
                    .unwrap_or_default(),
                Variant::Ghost.small(),
                icon(Glyph::PLUS, "16"),
                Html::text(label)
            )
        })
        .unwrap_or_default();
    Html::raw(format!(
        "<div id=\"{}\" role=\"group\" aria-label=\"{}\" data-fui=\"repeat\" class=\"fui-repeat\">{head}<div data-repeat=\"rows\" class=\"{}\">{drawn}</div><p data-repeat=\"empty\"{} class=\"fui-transform-empty\">{}</p>{blank}{add}</div>",
        Html::text(&id),
        Html::text(&label),
        fields.rows(),
        if none { "" } else { " hidden" },
        Html::text(empty)
    ))
}

/// The key the module gives the next row Add inserts: `new-` and the
/// first count from 1 that `taken` does not hold.
#[must_use]
pub fn fresh_key(taken: impl Fn(&str) -> bool) -> String {
    let mut count: u32 = 1;
    loop {
        let key = format!("new-{count}");
        if !taken(&key) {
            return key;
        }
        match count.checked_add(1) {
            Some(next) => count = next,
            None => return key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fresh_key;

    #[test]
    fn a_fresh_key_is_the_first_count_no_row_holds() {
        assert_eq!("new-1", fresh_key(|_| false));
        assert_eq!("new-3", fresh_key(|key| key == "new-1" || key == "new-2"));
        assert_eq!("new-2", fresh_key(|key| key == "new-1" || key == "new-3"));
    }
}
