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

use leptos::prelude::*;

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
pub fn stack_at(gap: Gap, body: impl IntoView + 'static) -> AnyView {
    let body = body.into_any();
    view! { <div class=gap.stack()>{body}</div> }.into_any()
}

/// One reading pane: its accessible name and its content.
#[derive(Debug)]
pub struct Pane {
    /// The pane's name, read when the keyboard reaches it.
    pub label: &'static str,
    /// What it shows.
    pub body: AnyView,
}

impl Pane {
    /// The pane `label` showing `body`.
    #[must_use]
    pub fn new(label: &'static str, body: impl IntoView + 'static) -> Self {
        Self {
            label,
            body: body.into_any(),
        }
    }
}

/// Returns a pane as a named region that the keyboard can reach and scroll.
fn pane(pane: Pane) -> AnyView {
    view! {
        <section class="fui-pane" role="region" aria-label=pane.label tabindex="0">
            {pane.body}
        </section>
    }
    .into_any()
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
pub fn three_panes(start: Pane, middle: Pane, end: Pane) -> AnyView {
    view! { <div class="fui-panes">{pane(start)} {pane(middle)} {pane(end)}</div> }.into_any()
}

/// Returns a list of about 24 rem beside the item it opens, from 64 rem;
/// the list above the item below it.
#[must_use]
pub fn master_detail(list: impl IntoView + 'static, detail: impl IntoView + 'static) -> AnyView {
    let list = list.into_any();
    let detail = detail.into_any();
    view! {
        <div class="fui-master-detail">
            <div class="fui-master">{list}</div>
            <div class="fui-detail">{detail}</div>
        </div>
    }
    .into_any()
}

/// Returns `main` (a runner's form) beside `aside` (its answer), three to
/// two, from 64 rem; the main part first below it.
#[must_use]
pub fn main_aside(main: impl IntoView + 'static, aside: impl IntoView + 'static) -> AnyView {
    let main = main.into_any();
    let aside = aside.into_any();
    view! {
        <div class="fui-main-aside">
            <div class="fui-main-part">{main}</div>
            <div class="fui-aside-part">{aside}</div>
        </div>
    }
    .into_any()
}

/// Returns `items` one under the other as a list, a hairline between two
/// items and none after the last, with list semantics kept.
#[must_use]
pub fn divided_list(label: Option<&'static str>, items: Vec<AnyView>) -> AnyView {
    let items = items
        .into_iter()
        .map(|item| view! { <li>{item}</li> })
        .collect_view();
    view! { <ul class="fui-divided" aria-label=label>{items}</ul> }.into_any()
}

/// The columns of a row of fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldRow {
    /// Two equal fields.
    Halves,
    /// A narrow field and a wide one: `1fr 2fr`.
    NarrowWide,
    /// A field and its actions: `1fr auto`.
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
pub fn field_row(row: FieldRow, body: impl IntoView + 'static) -> AnyView {
    let body = body.into_any();
    view! { <div class=row.class()>{body}</div> }.into_any()
}

/// Returns the actions `body` of a form in a bar at the foot of its card.
///
/// On a window tall enough the bar stays in view at the bottom while the
/// form scrolls, the page's scroll padding keeping a focused control clear
/// of it (WCAG 2.2 SC 2.4.11).
#[must_use]
pub fn action_bar(body: impl IntoView + 'static) -> AnyView {
    let body = body.into_any();
    view! { <div class="fui-action-bar">{body}</div> }.into_any()
}

/// Returns `items` as tiles in a row that wraps, with list semantics kept.
///
/// Each tile is a small bordered box holding a few facts of one thing, such
/// as a node's name, state and latency.
#[must_use]
pub fn tile_list(label: &'static str, items: Vec<AnyView>) -> AnyView {
    let items = items
        .into_iter()
        .map(|item| view! { <li class="fui-tile">{item}</li> })
        .collect_view();
    view! { <ul class="fui-tiles" aria-label=label>{items}</ul> }.into_any()
}

/// Returns `items` as a bulleted list of prose.
#[must_use]
pub fn bullet_list(items: Vec<AnyView>) -> AnyView {
    let items = items
        .into_iter()
        .map(|item| view! { <li>{item}</li> })
        .collect_view();
    view! { <ul class="fui-bullets">{items}</ul> }.into_any()
}
