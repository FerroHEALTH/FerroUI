// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The work-list: a dense listing a steward walks with the keyboard, a
//! focused row moved by `j`, `k` and the arrow keys, Enter to open it, and
//! a foot of actions that stays in view below the rows (R9).
//!
//! The rows are the server's: a data table whose rows [`row`] draws, each
//! with its `data-key`, the first one focused, inside a list card with the
//! foot [`worklist_foot`] draws. [`worklist`] wraps that table for the
//! `worklist` module. With no script the table is a table: every row's link
//! opens it, every checkbox posts with its form.
//!
//! Once the module runs the table is a grid (WAI-ARIA Authoring Practices
//! 1.2, the grid pattern) with a roving tabindex: one tab stop for the whole
//! list, the focused row's, which carries `aria-current="true"`; the
//! controls of the other rows leave the tab order, and those of the focused
//! row follow it. `j` and Down move to the next row, `k` and Up to the one
//! before ([`moved`]), Home and End to the first and the last, and Enter
//! follows the row's link; the focused row is scrolled clear of the sticky
//! header and of the foot (WCAG 2.2 SC 2.4.11). The grid role arrives with
//! the keys that make it one. A header checkbox selects every row and
//! shows the mixed state while some are selected ([`selection`]); each
//! selected row carries `aria-selected`.
//!
//! A row's own key reaches a control inside it, such as `o` on the opener
//! of the row's side panel ([`crate::view::side_panel::side_panel_with_shortcut`]):
//! the `keys` module, not this one, binds it, and only while that row holds
//! the focus. The panel returns the focus to the row when it closes, and the
//! shortcut hint at the foot ([`worklist_foot`]) and the legend can list it.

use crate::html::Html;

/// One row of a work-list.
#[derive(Debug)]
pub struct WorkRow<'a> {
    /// The row's key, its `data-key`: an id, never a record.
    pub key: &'a str,
    /// Whether the row is the one focused when the page is drawn, such as
    /// the first row.
    pub focused: bool,
    /// The row's class, such as [`crate::table::view::EXCLUDED`].
    pub class: Option<&'static str>,
    /// The row's cells, drawn with the cell helpers of
    /// [`crate::table::view`].
    pub cells: Html,
}

/// Returns a row of a work-list: its key, and the tab stop and
/// `aria-current` when it is the focused row.
#[must_use]
pub fn row(row: WorkRow<'_>) -> Html {
    let WorkRow {
        key,
        focused,
        class,
        cells,
    } = row;
    Html::raw(format!(
        "<tr data-key=\"{}\" tabindex=\"{}\"{}{}>{cells}</tr>",
        Html::text(key),
        if focused { "0" } else { "-1" },
        if focused {
            " aria-current=\"true\""
        } else {
            ""
        },
        class
            .map(|class| format!(" class=\"{class}\""))
            .unwrap_or_default()
    ))
}

/// Returns the foot of a work-list: the shortcut hint at its start and the
/// pager at its end, in view below the rows inside the card.
///
/// The card sits in the page's column, at most two levels down, where the
/// stylesheet looks for its foot.
#[must_use]
pub fn worklist_foot(hint: Option<Html>, pager: Option<Html>) -> Html {
    Html::raw(format!(
        "<div class=\"fui-worklist-foot\">{}{}</div>",
        hint.unwrap_or_default(),
        pager.unwrap_or_default()
    ))
}

/// The row a key moves the focus to from `current` of `count`, or `None`
/// for a key the work-list does not take; the ends do not wrap.
#[must_use]
pub fn moved(key: &str, current: usize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match key {
        "j" | "ArrowDown" => Some(current.saturating_add(1).min(last)),
        "k" | "ArrowUp" => Some(current.saturating_sub(1).min(last)),
        "Home" => Some(0),
        "End" => Some(last),
        _ => None,
    }
}

/// How much of a page's rows is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// No row.
    None,
    /// Some rows, not every one: the header checkbox is mixed.
    Some,
    /// Every row.
    All,
}

/// The selection of `checked` of `count` rows.
#[must_use]
pub const fn selection(checked: usize, count: usize) -> Selection {
    if checked == 0 || count == 0 {
        Selection::None
    } else if checked >= count {
        Selection::All
    } else {
        Selection::Some
    }
}

/// Returns the work-list around `table`, the data table the server drew
/// with its rows from [`row`], for the `worklist` module.
#[must_use]
pub fn worklist(table: Html) -> Html {
    let table = String::from(table);
    Html::raw(format!(
        "<div data-fui=\"worklist\" class=\"fui-worklist-grid\">{table}</div>"
    ))
}

#[cfg(test)]
mod tests {
    use super::{Selection, moved, selection};

    #[test]
    fn the_keys_move_one_row_and_stop_at_the_ends() {
        assert_eq!(Some(1), moved("j", 0, 3));
        assert_eq!(Some(1), moved("ArrowDown", 0, 3));
        assert_eq!(Some(2), moved("j", 2, 3));
        assert_eq!(Some(0), moved("k", 0, 3));
        assert_eq!(Some(1), moved("ArrowUp", 2, 3));
        assert_eq!(Some(0), moved("Home", 2, 3));
        assert_eq!(Some(2), moved("End", 0, 3));
        assert_eq!(None, moved("x", 0, 3));
        assert_eq!(None, moved("j", 0, 0));
    }

    #[test]
    fn some_rows_selected_is_the_mixed_state() {
        assert_eq!(Selection::None, selection(0, 4));
        assert_eq!(Selection::Some, selection(2, 4));
        assert_eq!(Selection::All, selection(4, 4));
        assert_eq!(Selection::None, selection(0, 0));
    }
}
