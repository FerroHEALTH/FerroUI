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

/// Returns a quiet empty state for a section of a page with nothing in it:
/// `glyph`, `title` under a level-two heading and the one `line`, with no
/// frame, inside the card that holds it.
#[must_use]
pub fn empty_quiet(
    glyph: Glyph,
    title: impl Into<String>,
    line: impl Into<String>,
) -> impl IntoView {
    view! {
        <div class="fui-empty fui-empty-quiet">
            <span class="fui-empty-glyph">{icon(glyph, "20")}</span>
            <h2>{title.into()}</h2>
            <p class="fui-empty-body">{line.into()}</p>
        </div>
    }
}

/// Returns the busy indicator a section shows while its answer arrives: a
/// turning ring beside the words `label`, announced politely; the ring
/// stands still for a reader who asks for less motion.
#[must_use]
pub fn spinner(label: impl Into<String>) -> AnyView {
    busy(label, Busy::Inline).into_any()
}

/// Where a busy mark sits, which sets its size and whether its words show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Busy {
    /// Inside a line of text, its words beside the ring.
    Inline,
    /// Under a section heading, a step larger, its words beside the ring.
    Section,
    /// Beside a control that is waiting, such as a sign-in button that
    /// posts: the ring alone in sight, its words read by a screen reader.
    Control,
}

/// Returns the busy mark for something waiting on the server: a turning
/// ring and the sentence `label` that says what it waits for ("Reading the
/// capability statement"), in a polite status region.
///
/// The ring stands still under `prefers-reduced-motion`, so it then reads as
/// a static mark beside its words.
#[must_use]
pub fn busy(label: impl Into<String>, at: Busy) -> AnyView {
    let label = label.into();
    let (class, words) = match at {
        Busy::Inline => ("fui-spinner", "fui-spinner-text"),
        Busy::Section => ("fui-spinner fui-spinner-section", "fui-spinner-text"),
        Busy::Control => ("fui-spinner", "fui-sr-only"),
    };
    view! {
        <span class=class role="status">
            <span class="fui-spinner-ring" aria-hidden="true"></span>
            <span class=words>{label}</span>
        </span>
    }
    .into_any()
}

/// Returns `lines` skeleton bars standing in for content that is loading,
/// announced as busy, with the sentence `label` for a screen reader.
#[must_use]
pub fn skeleton(lines: u8, label: impl Into<String>) -> AnyView {
    let label = label.into();
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
    .into_any()
}
