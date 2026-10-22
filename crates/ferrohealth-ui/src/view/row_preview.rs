// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The OMOP row preview (B6): the rows one input becomes, table by table.
//!
//! Each table is a block headed by its name in the code face with its row
//! count, then each row's column and value pairs, a hairline between rows.
//! A concept that did not resolve shows as `0` with the warn mark and its
//! words ([`RowValue::ConceptZero`]), so a missed mapping shows at a
//! glance and never by colour alone (WCAG 2.2 SC 1.4.1). On a phone each
//! column stacks over its value, so a value keeps the full width and an id
//! never breaks inside. No specification governs the preview: our own
//! design.

use leptos::prelude::*;

use crate::i18n::{Text, tr, tr_with};
use crate::icon::{Glyph, icon};

/// One value of a previewed row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowValue {
    /// A value as the engine would write it, such as `72.4` or
    /// `3025315 · Body weight`.
    Value(String),
    /// A concept id that did not resolve, written as `0`.
    ConceptZero,
}

/// One column of a previewed row and its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowField {
    /// The column's name, such as `measurement_concept_id`.
    pub column: String,
    /// Its value.
    pub value: RowValue,
}

/// One table of the preview and the rows it would take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowTable {
    /// The table's name, such as `MEASUREMENT`.
    pub name: String,
    /// What follows the row count, muted, such as the mapping that writes
    /// the rows.
    pub detail: Option<String>,
    /// The rows, each its columns in order.
    pub rows: Vec<Vec<RowField>>,
}

/// Returns `value` as its cell shows it.
fn value(value: RowValue) -> AnyView {
    match value {
        RowValue::Value(value) => value.into_any(),
        RowValue::ConceptZero => view! {
            <span class="fui-concept-zero">
                {icon(Glyph::TRIANGLE_ALERT, "12")}
                {tr(Text::RowsConceptZero)}
            </span>
        }
        .into_any(),
    }
}

/// Returns one row's columns and values.
fn row(fields: Vec<RowField>) -> AnyView {
    let fields = fields
        .into_iter()
        .map(
            |RowField {
                 column,
                 value: cell,
             }| {
                view! {
                    <div>
                        <dt>{column}</dt>
                        <dd>{value(cell)}</dd>
                    </div>
                }
            },
        )
        .collect_view();
    view! { <dl class="fui-rowgraph-fields">{fields}</dl> }.into_any()
}

/// Returns one table: its name with its row count over its rows.
fn table(table: RowTable) -> AnyView {
    let RowTable { name, detail, rows } = table;
    let count = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    let mut note = tr_with(Text::RowsCount, &[("count", count.into())]);
    if let Some(detail) = detail {
        note.push_str(" · ");
        note.push_str(&detail);
    }
    let rows = rows.into_iter().map(row).collect_view();
    view! {
        <section class="fui-rowgraph-table">
            <h4 class="fui-rowgraph-name">
                {name}
                " "
                <span class="fui-muted">{note}</span>
            </h4>
            {rows}
        </section>
    }
    .into_any()
}

/// Returns the preview of `tables`, each name a level-four heading under
/// the answer pane's title.
#[must_use]
pub fn row_preview(tables: Vec<RowTable>) -> AnyView {
    let tables = tables.into_iter().map(table).collect_view();
    view! { <div class="fui-rowgraph">{tables}</div> }.into_any()
}
