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

use crate::html::Html;
use crate::i18n::{Locale, Text, tr};
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
    pub name: Html,
    /// What each root declares, in the order of the roots: `None` for a
    /// root that does not declare it. A root that did not answer shows "no
    /// answer" whatever its cell holds.
    pub cells: Vec<Option<Html>>,
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
fn root_head(locale: &Locale, root: &MatrixRoot) -> String {
    let (tone, word) = if root.answered {
        (Tone::Ok, tr(locale, Text::RootAnswered))
    } else {
        (Tone::Bad, tr(locale, Text::RootNoAnswer))
    };
    format!(
        "<th scope=\"col\"><span class=\"fui-root-head\"><strong>{}</strong><span class=\"fui-muted\">{}</span><span class=\"fui-state {tone}\"><span aria-hidden=\"true\" class=\"fui-dot {tone}\"></span>{word}</span></span></th>",
        Html::text(&root.name),
        Html::text(&root.release),
        tone = tone.class()
    )
}

/// Returns the row `row` under the columns of `roots`.
fn matrix_row(locale: &Locale, roots: &[MatrixRoot], row: MatrixRow) -> String {
    let mut cells = row.cells.into_iter();
    let drawn: String = roots
        .iter()
        .map(|root| {
            let value = cells.next().flatten();
            let label = Html::text(&root.name);
            match (root.answered, value) {
                (false, _) => format!(
                    "<td data-label=\"{label}\" class=\"is-none\">{}</td>",
                    tr(locale, Text::RootCellNoAnswer)
                ),
                (true, None) => format!(
                    "<td data-label=\"{label}\" class=\"is-none\">{}</td>",
                    tr(locale, Text::RootNotDeclared)
                ),
                (true, Some(value)) => format!("<td data-label=\"{label}\">{value}</td>"),
            }
        })
        .collect::<Vec<_>>()
        .concat();
    format!(
        "<tr><th scope=\"row\" data-role=\"title\">{}</th>{drawn}</tr>",
        row.name
    )
}

/// Returns the root matrix `matrix`.
#[must_use]
pub fn root_matrix(locale: &Locale, matrix: RootMatrix) -> Html {
    let RootMatrix {
        caption,
        corner,
        roots,
        rows,
    } = matrix;
    let heads: String = roots
        .iter()
        .map(|root| root_head(locale, root))
        .collect::<Vec<_>>()
        .concat();
    let body: String = rows
        .into_iter()
        .map(|row| matrix_row(locale, &roots, row))
        .collect::<Vec<_>>()
        .concat();
    let caption = Html::text(caption);
    Html::raw(format!(
        "<div class=\"fui-root-matrix\"><div role=\"region\" aria-label=\"{caption}\" tabindex=\"0\" class=\"fui-table-scroll\"><table class=\"fui-table\"><caption class=\"fui-sr-only\">{caption}</caption><thead><tr><th scope=\"col\">{}</th>{heads}</tr></thead><tbody>{body}</tbody></table></div></div>",
        Html::text(corner)
    ))
}
