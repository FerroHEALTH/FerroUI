// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page layouts a screen is built from.
//!
//! They are three reading panes, a list beside the item it opens, a form
//! beside its answer, a stack at each step of space, a divided list, and a
//! row of fields with columns of its own.
//!
//! Each lays its parts side by side from 64 rem (a row of fields from
//! 40 rem) and one above the other below it, so a page reflows at 320 px
//! with no sideways scroll (WCAG 2.2 SC 1.4.10). Pure CSS, no script, no
//! style attribute. The simpler layouts (the stack, the grid, the cluster,
//! the split) are in [`crate::view::surface`].

use std::borrow::Cow;

use crate::html::Html;

/// `body` between the markup `open` and `close`, both the library's own.
fn wrap(open: &str, body: Html, close: &str) -> Html {
    let mut out = String::from(open);
    out.push_str(&String::from(body));
    out.push_str(close);
    Html::raw(out)
}

/// `items` each between `open` and `close`.
fn each(items: Vec<Html>, open: &str, close: &str) -> String {
    items
        .into_iter()
        .map(|item| String::from(wrap(open, item, close)))
        .collect::<Vec<_>>()
        .concat()
}


/// A step of space between the parts of a stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    /// The tight step, between a line and its note.
    Tight,
    /// The default step, between the fields of a group.
    Default,
    /// The loose step, between cards.
    Loose,
    /// The section step, between the sections of a page.
    Section,
}

impl Gap {
    /// The stack's classes at this step.
    const fn stack(self) -> &'static str {
        match self {
            Self::Tight => "fui-stack fui-gap-tight",
            Self::Default => "fui-stack fui-gap-default",
            Self::Loose => "fui-stack",
            Self::Section => "fui-stack fui-gap-section",
        }
    }
}

/// Returns `body` one part under the other, `gap` apart.
#[must_use]
pub fn stack_at(gap: Gap, body: Html) -> Html {
    wrap(&format!("<div class=\"{}\">", gap.stack()), body, "</div>")
}

/// One reading pane: its accessible name and its content.
#[derive(Debug)]
pub struct Pane {
    /// The pane's name, read when the keyboard reaches it.
    pub label: Cow<'static, str>,
    /// What it shows.
    pub body: Html,
}

impl Pane {
    /// The pane `label` showing `body`.
    #[must_use]
    pub fn new(label: impl Into<Cow<'static, str>>, body: Html) -> Self {
        Self {
            label: label.into(),
            body,
        }
    }
}

/// Returns a pane as a named region that the keyboard can reach and scroll.
fn pane(pane: Pane) -> Html {
    wrap(
        &format!(
            "<section role=\"region\" aria-label=\"{}\" tabindex=\"0\" class=\"fui-pane\">",
            Html::text(&pane.label)
        ),
        pane.body,
        "</section>",
    )
}

/// Returns three reading panes: a narrow `start` (a search), the `middle`
/// (a hierarchy) and a wide `end` (the item).
///
/// From 64 rem they stand side by side, each scrolling on its own between
/// two bounds so walking one never moves another; below 64 rem they stand
/// one above the other and scroll with the page.
///
/// Each pane is a named, focusable region, so the keyboard scrolls it
/// (WCAG 2.2 SC 2.1.1).
#[must_use]
pub fn three_panes(start: Pane, middle: Pane, end: Pane) -> Html {
    Html::raw(format!(
        "<div class=\"fui-panes\">{}{}{}</div>",
        pane(start),
        pane(middle),
        pane(end)
    ))
}

/// Returns a list of about 24 rem beside the item it opens, from 64 rem;
/// the list above the item below it.
#[must_use]
pub fn master_detail(list: Html, detail: Html) -> Html {
    let (list, detail) = (String::from(list), String::from(detail));
    Html::raw(format!(
        "<div class=\"fui-master-detail\"><div class=\"fui-master\">{list}</div><div class=\"fui-detail\">{detail}</div></div>"
    ))
}

/// Returns `main` (a runner's form) beside `aside` (its answer), three to
/// two, from 64 rem; the main part first below it.
#[must_use]
pub fn main_aside(main: Html, aside: Html) -> Html {
    let (main, aside) = (String::from(main), String::from(aside));
    Html::raw(format!(
        "<div class=\"fui-main-aside\"><div class=\"fui-main-part\">{main}</div><div class=\"fui-aside-part\">{aside}</div></div>"
    ))
}

/// Returns `items` one under the other as a list, a hairline between two
/// items and none after the last, with list semantics kept.
#[must_use]
pub fn divided_list(label: Option<Cow<'static, str>>, items: Vec<Html>) -> Html {
    let label = label
        .map(|label| format!(" aria-label=\"{}\"", Html::text(label)))
        .unwrap_or_default();
    Html::raw(format!(
        "<ul{label} class=\"fui-divided\">{}</ul>",
        each(items, "<li>", "</li>")
    ))
}

/// The columns of a row of fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldRow {
    /// Two equal fields.
    Halves,
    /// A narrow field and a wide one: `1fr 2fr`.
    NarrowWide,
    /// A field and its actions: `1fr auto`, one row at every width.
    WithActions,
    /// A code, a display and two actions: `1fr 2fr auto auto`.
    CodeDisplay,
    /// A code, a display, a definition and an action: `1fr 1fr 2fr auto`.
    CodeDisplayDefinition,
}

impl FieldRow {
    /// The row's classes.
    const fn class(self) -> &'static str {
        match self {
            Self::Halves => "fui-field-row fui-field-row-halves",
            Self::NarrowWide => "fui-field-row fui-field-row-narrow-wide",
            Self::WithActions => "fui-field-row fui-field-row-actions",
            Self::CodeDisplay => "fui-field-row fui-field-row-code-display",
            Self::CodeDisplayDefinition => "fui-field-row fui-field-row-definition",
        }
    }
}

/// Returns the fields `body` in one row of the columns `row` from 40 rem,
/// one above the other below it; an action button sits at the foot of its
/// column, level with the controls.
#[must_use]
pub fn field_row(row: FieldRow, body: Html) -> Html {
    wrap(&format!("<div class=\"{}\">", row.class()), body, "</div>")
}

/// Returns [`field_row`] with the id `id`, for a row a console adds,
/// removes or finds by its id, such as one value of a list.
#[must_use]
pub fn keyed_field_row(id: &str, row: FieldRow, body: Html) -> Html {
    wrap(
        &format!(
            "<div id=\"{}\" class=\"{}\">",
            Html::text(id),
            row.class()
        ),
        body,
        "</div>",
    )
}

/// Returns `items` as tiles in a row that wraps, with list semantics kept.
///
/// Each tile is a small bordered box holding a few facts of one thing, such
/// as a node's name, state and latency.
#[must_use]
pub fn tile_list(label: impl Into<Cow<'static, str>>, items: Vec<Html>) -> Html {
    Html::raw(format!(
        "<ul aria-label=\"{}\" class=\"fui-tiles\">{}</ul>",
        Html::text(label.into()),
        each(items, "<li class=\"fui-tile\">", "</li>")
    ))
}

/// Returns `items` as a bulleted list of prose.
#[must_use]
pub fn bullet_list(items: Vec<Html>) -> Html {
    Html::raw(format!(
        "<ul class=\"fui-bullets\">{}</ul>",
        each(items, "<li>", "</li>")
    ))
}
