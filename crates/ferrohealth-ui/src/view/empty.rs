// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The empty state and the skeleton that stands in for loading content.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};

/// Returns an empty state: `glyph` in an accent round, `title` under a
/// level-two heading, the sentence `body`, and `action` when there is one.
#[must_use]
pub fn empty_state(
    glyph: Glyph,
    title: impl Into<String>,
    body: impl Into<String>,
    action: impl IntoView,
) -> impl IntoView {
    view! {
        <div class="fui-empty">
            <span class="fui-empty-glyph">{icon(glyph, "24")}</span>
            <h2>{title.into()}</h2>
            <p class="fui-empty-body">{body.into()}</p>
            {action}
        </div>
    }
}

/// Returns `lines` skeleton bars standing in for content that is loading,
/// announced as busy, with the sentence `label` for a screen reader.
#[must_use]
pub fn skeleton(lines: u8, label: &'static str) -> impl IntoView + use<> {
    let bars = (0..lines)
        .map(|_| view! { <span class="fui-skeleton"></span> })
        .collect_view();
    view! {
        <div aria-busy="true">
            <span class="fui-sr-only" role="status">
                {label}
            </span>
            {bars}
        </div>
    }
}
