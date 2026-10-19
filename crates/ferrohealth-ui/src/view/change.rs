// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The change table: a row per setting with its value before and after,
//! as a plan shows the difference between a running and a staged
//! configuration.
//!
//! The value removed is marked "−" and the value added "+", in words as
//! well as tone, and an absent value reads "not set", so a change never
//! rides on colour alone.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::table::view::{Body, Column, DataTable, data_table};

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
fn columns() -> [Column; 3] {
    [
        Column::owned("key", tr(Text::ChangeSetting)),
        Column::owned("from", tr(Text::ChangeBefore)),
        Column::owned("to", tr(Text::ChangeAfter)),
    ]
}

/// Returns one side of a change: `value` marked with `sign` in the removed
/// or the added tone, or "not set".
fn side(value: Option<&String>, sign: &'static str, removed: bool) -> AnyView {
    let class = if removed {
        "fui-change fui-change-removed"
    } else {
        "fui-change fui-change-added"
    };
    match value {
        Some(value) => view! {
            <span class=class>
                <span class="fui-change-sign" aria-hidden="true">
                    {sign}
                </span>
                <span class="fui-sr-only">
                    {format!("{} ", tr(if removed { Text::ChangeRemoved } else { Text::ChangeAdded }))}
                </span>
                <code>{value.clone()}</code>
            </span>
        }
        .into_any(),
        None => view! { <span class="fui-muted">{tr(Text::ChangeNotSet)}</span> }.into_any(),
    }
}

/// Returns the table of `changes` captioned `caption`; an empty list reads
/// as a sentence.
#[must_use]
pub fn change_table(caption: &'static str, changes: &[Change]) -> impl IntoView + use<> {
    if changes.is_empty() {
        return view! { <p class="fui-muted">{tr(Text::ChangeNone)}</p> }.into_any();
    }
    let before = tr(Text::ChangeBefore);
    let after = tr(Text::ChangeAfter);
    let rows = changes
        .iter()
        .map(|change| {
            view! {
                <tr>
                    <th scope="row">
                        <code>{change.key.clone()}</code>
                    </th>
                    <td data-label=before.clone()>{side(change.from.as_ref(), "−", true)}</td>
                    <td data-label=after.clone()>{side(change.to.as_ref(), "+", false)}</td>
                </tr>
            }
        })
        .collect_view()
        .into_any();
    data_table(DataTable {
        caption,
        columns: &columns(),
        sort: None,
        sort_href: &|_| String::new(),
        body: Body::Rows(rows),
    })
    .into_any()
}
