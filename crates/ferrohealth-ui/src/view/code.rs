// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code panel: highlighted text in a bounded box that scrolls.
//!
//! It never takes the page's full height, and draws plain or with numbered
//! lines a link can target. A final line break ends the last line and opens
//! none, as a file reads.

use leptos::prelude::*;

use crate::highlight::{Language, Lines, html};

/// Returns `text` in `language`, highlighted in a bounded panel named
/// `label`, focusable so the keyboard scrolls it (WCAG 2.2 SC 2.1.1).
#[must_use]
pub fn code_panel(label: &str, language: Language, text: &str) -> impl IntoView + use<> {
    let text = text.strip_suffix('\n').unwrap_or(text);
    let lines = html(language, text, Lines::default(), |_| None);
    view! {
        <div class="fui-code-panel" role="region" aria-label=label.to_owned() tabindex="0">
            <pre inner_html=lines></pre>
        </div>
    }
}

/// Returns `text` in `language` as [`code_panel`] draws it, capped at
/// 24 rem with its own scroll, for a product's raw self-description on its
/// About page.
///
/// A line never wraps: a long one scrolls sideways, on a phone too.
#[must_use]
pub fn self_description(label: &str, language: Language, text: &str) -> impl IntoView + use<> {
    let text = text.strip_suffix('\n').unwrap_or(text);
    let lines = html(language, text, Lines::default(), |_| None);
    view! {
        <div
            class="fui-code-panel fui-self-description"
            role="region"
            aria-label=label.to_owned()
            tabindex="0"
        >
            <pre inner_html=lines></pre>
        </div>
    }
}

/// Returns `text` in `language` as [`code_panel`] draws it, its lines
/// numbered.
///
/// Each line has the id `ids` and its number, which a link such as a
/// diagnostic's targets; the targeted line is marked by an edge as well as a
/// tint.
#[must_use]
pub fn code_lines(label: &str, language: Language, text: &str, ids: &str) -> impl IntoView + use<> {
    let text = text.strip_suffix('\n').unwrap_or(text);
    let lines = html(language, text, Lines { ids: Some(ids) }, |_| None);
    view! {
        <div
            class="fui-code-panel fui-code-panel-numbered"
            role="region"
            aria-label=label.to_owned()
            tabindex="0"
        >
            <pre inner_html=lines></pre>
        </div>
    }
}
