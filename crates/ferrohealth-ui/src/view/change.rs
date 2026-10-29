// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The change table: a row per setting with its value before and after,
//! as a plan shows the difference between a running and a staged
//! configuration.
//!
//! The value removed is marked "−" and the value added "+", in words as
//! well as tone, and an absent value reads "not set", so a change never
//! rides on colour alone.

use crate::html::Html;
use crate::i18n::{Locale, Text, tr};
use crate::table::view::{Body, Column, DataTable, Sorting, data_table};

/// One setting's change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The setting's name or path.
    pub key: String,
    /// Its value before, `None` when it was not set.
    pub from: Option<String>,
    /// Its value after, `None` when it is removed.
    pub to: Option<String>,
}

/// The columns of a change table, named in the page's locale.
fn columns(locale: &Locale) -> [Column; 3] {
    [
        Column::owned("key", tr(locale, Text::ChangeSetting)),
        Column::owned("from", tr(locale, Text::ChangeBefore)),
        Column::owned("to", tr(locale, Text::ChangeAfter)),
    ]
}

/// Returns one side of a change: `value` marked with `sign` in the removed
/// or the added tone, or "not set".
fn side(locale: &Locale, value: Option<&String>, sign: &'static str, removed: bool) -> String {
    let class = if removed {
        "fui-change fui-change-removed"
    } else {
        "fui-change fui-change-added"
    };
    match value {
        Some(value) => {
            let said = tr(
                locale,
                if removed {
                    Text::ChangeRemoved
                } else {
                    Text::ChangeAdded
                },
            )
            .map(|words| format!("{words} "));
            format!(
                "<span class=\"{class}\"><span aria-hidden=\"true\" class=\"fui-change-sign\">{sign}</span><span class=\"fui-sr-only\">{said}</span><code>{}</code></span>",
                Html::text(value)
            )
        }
        None => format!(
            "<span class=\"fui-muted\">{}</span>",
            tr(locale, Text::ChangeNotSet)
        ),
    }
}

/// Returns the table of `changes` captioned `caption`; an empty list reads
/// as a sentence.
#[must_use]
pub fn change_table(locale: &Locale, caption: impl Into<String>, changes: &[Change]) -> Html {
    let caption = caption.into();
    if changes.is_empty() {
        return Html::raw(format!(
            "<p class=\"fui-muted\">{}</p>",
            tr(locale, Text::ChangeNone)
        ));
    }
    let before = Html::text(tr(locale, Text::ChangeBefore).text());
    let after = Html::text(tr(locale, Text::ChangeAfter).text());
    let rows: Html = changes
        .iter()
        .map(|change| {
            Html::raw(format!(
                "<tr><th scope=\"row\"><code>{}</code></th><td data-label=\"{before}\">{}</td><td data-label=\"{after}\">{}</td></tr>",
                Html::text(&change.key),
                side(locale, change.from.as_ref(), "\u{2212}", true),
                side(locale, change.to.as_ref(), "+", false)
            ))
        })
        .collect();
    data_table(DataTable {
        caption: &caption,
        columns: &columns(locale),
        sorting: Sorting::default(),
        body: Body::Rows(rows),
    })
}
