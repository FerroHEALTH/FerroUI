// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The root matrix (M5): what each FHIR root of a server declares, one
//! column per root.
//!
//! Each column head is a header cell carrying the root, its release and
//! whether it answered, as an F12 state: the dot's shape and colour and the
//! word together (family rule 12). The first column, the declaration's
//! name, stays while the rest scroll sideways inside the card under the
//! scroll cue, never the page (WCAG 2.2 SC 1.4.10). A cell a root does not
//! declare says "not declared"; every cell of a root that did not answer
//! says "no answer", so no cell is ever blank. No specification governs the
//! layout: our own design.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::tone::Tone;

/// One root of the server: a column of the matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRoot {
    /// The root's name, such as "R4".
    pub name: String,
    /// Its release, such as "FHIR 4.0.1".
    pub release: String,
    /// Whether it answered.
    pub answered: bool,
}

/// One declaration: a row of the matrix.
#[derive(Debug)]
pub struct MatrixRow {
    /// The declaration's name, which heads the row and stays in view.
    pub name: AnyView,
    /// What each root declares, in the order of the roots: `None` for a
    /// root that does not declare it. A root that did not answer shows "no
    /// answer" whatever its cell holds.
    pub cells: Vec<Option<AnyView>>,
}

/// What a root matrix shows.
#[derive(Debug)]
pub struct RootMatrix {
    /// The caption, which names the table and its scrolling region.
    pub caption: String,
    /// The header of the first column, such as "Declaration".
    pub corner: String,
    /// The roots, in the order of the columns.
    pub roots: Vec<MatrixRoot>,
    /// The declarations, in order.
    pub rows: Vec<MatrixRow>,
}

/// Returns the head of the column of `root`: its name, its release and
/// whether it answered.
fn root_head(root: &MatrixRoot) -> AnyView {
    let (tone, word) = if root.answered {
        (Tone::Ok, tr(Text::RootAnswered))
    } else {
        (Tone::Bad, tr(Text::RootNoAnswer))
    };
    let state = format!("fui-state {}", tone.class());
    let dot = format!("fui-dot {}", tone.class());
    let name = root.name.clone();
    let release = root.release.clone();
    view! {
        <th scope="col">
            <span class="fui-root-head">
                <strong>{name}</strong>
                <span class="fui-muted">{release}</span>
                <span class=state>
                    <span class=dot aria-hidden="true"></span>
                    {word}
                </span>
            </span>
        </th>
    }
    .into_any()
}

/// Returns the row `row` under the columns of `roots`.
fn matrix_row(roots: &[MatrixRoot], row: MatrixRow) -> AnyView {
    let mut cells = row.cells.into_iter();
    let drawn = roots
        .iter()
        .map(|root| {
            let value = cells.next().flatten();
            let label = root.name.clone();
            match (root.answered, value) {
                (false, _) => view! {
                    <td data-label=label class="is-none">
                        {tr(Text::RootCellNoAnswer)}
                    </td>
                }
                .into_any(),
                (true, None) => view! {
                    <td data-label=label class="is-none">
                        {tr(Text::RootNotDeclared)}
                    </td>
                }
                .into_any(),
                (true, Some(value)) => view! { <td data-label=label>{value}</td> }.into_any(),
            }
        })
        .collect_view();
    view! {
        <tr>
            <th scope="row" data-role="title">
                {row.name}
            </th>
            {drawn}
        </tr>
    }
    .into_any()
}

/// Returns the root matrix `matrix`.
#[must_use]
pub fn root_matrix(matrix: RootMatrix) -> AnyView {
    let RootMatrix {
        caption,
        corner,
        roots,
        rows,
    } = matrix;
    let heads = roots.iter().map(root_head).collect_view();
    let body = rows
        .into_iter()
        .map(|row| matrix_row(&roots, row))
        .collect_view();
    let label = caption.clone();
    view! {
        <div class="fui-root-matrix">
            <div class="fui-table-scroll" role="region" aria-label=label tabindex="0">
                <table class="fui-table">
                    <caption class="fui-sr-only">{caption}</caption>
                    <thead>
                        <tr>
                            <th scope="col">{corner}</th>
                            {heads}
                        </tr>
                    </thead>
                    <tbody>{body}</tbody>
                </table>
            </div>
        </div>
    }
    .into_any()
}
