// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The surfaces a page is built from: the page header, the card, the stat
//! card, the table card, the definition grid, the disclosure, and the stack
//! and grid that lay cards out.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};

/// Returns the header that opens a page: its `title`, one `subtitle` line
/// when there is one, and `actions` at the far end.
#[must_use]
pub fn page_header(
    title: impl Into<String>,
    subtitle: Option<String>,
    actions: impl IntoView,
) -> impl IntoView {
    view! {
        <header class="fui-page-header">
            <div>
                <h1>{title.into()}</h1>
                {subtitle.map(|line| view! { <p class="fui-page-subtitle">{line}</p> })}
            </div>
            <div class="fui-actions">{actions}</div>
        </header>
    }
}

/// Returns a card titled `title` holding `body`, a section under a
/// level-two heading.
#[must_use]
pub fn card(title: impl Into<String>, body: impl IntoView) -> impl IntoView {
    view! {
        <section class="fui-card">
            <h2 class="fui-card-title">{title.into()}</h2>
            {body}
        </section>
    }
}

/// Returns a card holding `body` with no heading of its own.
#[must_use]
pub fn panel(body: impl IntoView) -> impl IntoView {
    view! { <div class="fui-card">{body}</div> }
}

/// What a stat card shows.
#[derive(Debug, Clone)]
pub struct Stat {
    /// The icon beside the value.
    pub glyph: Glyph,
    /// The measure, as text.
    pub value: String,
    /// What the measure is.
    pub label: &'static str,
    /// The page the card leads to, when it leads anywhere.
    pub href: Option<String>,
}

/// Returns a stat card: the value of a measure beside its icon, the label
/// under it, the whole card a link when the stat leads to a page.
#[must_use]
pub fn stat_card(stat: Stat) -> impl IntoView {
    let inner = view! {
        <span class="fui-stat-glyph">{icon(stat.glyph, "20")}</span>
        <span>
            <span class="fui-stat-value">{stat.value}</span>
            <br />
            <span class="fui-stat-label">{stat.label}</span>
        </span>
    };
    match stat.href {
        Some(href) => view! {
            <a href=href class="fui-card fui-stat">
                {inner}
            </a>
        }
        .into_any(),
        None => view! { <div class="fui-card fui-stat">{inner}</div> }.into_any(),
    }
}

/// Returns `table` in a box named `label` that scrolls sideways inside the
/// page, showing every row, focusable so the keyboard scrolls it (WCAG 2.2
/// SC 1.4.10, SC 2.1.1).
#[must_use]
pub fn table_card(label: impl Into<String>, table: impl IntoView) -> impl IntoView {
    view! {
        <div class="fui-table-scroll" role="region" aria-label=label.into() tabindex="0">
            {table}
        </div>
    }
}

/// Returns a compact definition grid: each label beside its value from
/// 40 rem up, one above the other below it.
#[must_use]
pub fn definitions(rows: Vec<(&'static str, AnyView)>) -> impl IntoView {
    let rows = rows
        .into_iter()
        .map(|(label, value)| {
            view! {
                <dt>{label}</dt>
                <dd>{value}</dd>
            }
        })
        .collect_view();
    view! { <dl class="fui-facts">{rows}</dl> }
}

/// Returns `value`, or the muted word "none" when it is empty, for a
/// definition whose value can be absent.
#[must_use]
pub fn or_none(value: String) -> AnyView {
    if value.is_empty() {
        view! { <span class="fui-muted">"none"</span> }.into_any()
    } else {
        value.into_any()
    }
}

/// Returns a disclosure: `summary` the browser folds `body` under.
#[must_use]
pub fn disclosure(summary: impl Into<String>, body: impl IntoView) -> impl IntoView {
    view! {
        <details class="fui-disclosure">
            <summary>{summary.into()}</summary>
            <div>{body}</div>
        </details>
    }
}

/// Returns `children` one above the other with the loose gap between.
#[must_use]
pub fn stack(children: impl IntoView) -> impl IntoView {
    view! { <div class="fui-stack">{children}</div> }
}

/// Returns `children` in a grid of columns at least 16 rem wide.
#[must_use]
pub fn grid(children: impl IntoView) -> impl IntoView {
    view! { <div class="fui-grid">{children}</div> }
}
