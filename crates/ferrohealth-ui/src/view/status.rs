// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The status marks: the pill, the badge, the status indicator and the
//! banner. Each is a word first and a tint second (WCAG 2.2 SC 1.4.1).

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::tone::Tone;

/// Returns a status pill: a dot and `word` on the ground of `tone`.
#[must_use]
pub fn pill(tone: Tone, word: &str) -> impl IntoView + use<> {
    view! { <span class=tone.pill()>{word.to_owned()}</span> }
}

/// Returns a badge: `word` on the ground of `tone`, with no dot.
#[must_use]
pub fn badge(tone: Tone, word: &str) -> impl IntoView + use<> {
    view! { <span class=tone.badge()>{word.to_owned()}</span> }
}

/// Returns a status indicator: `glyph` beside `word`, in `tone`, with a
/// muted `detail` after it when there is one.
#[must_use]
pub fn status(
    tone: Tone,
    glyph: Glyph,
    word: &str,
    detail: Option<String>,
) -> impl IntoView + use<> {
    let class = format!("fui-status {}", tone.class());
    view! {
        <span class=class>
            {icon(glyph, "14")}
            {word.to_owned()}
            {detail.map(|detail| view! { <span class="fui-status-detail">{detail}</span> })}
        </span>
    }
}

/// Returns a banner across the page in `tone`: `glyph`, the bold `title`
/// and the sentence `body`, such as the notice that a deployment is a
/// sandbox that must hold no real patient data.
#[must_use]
pub fn banner(tone: Tone, glyph: Glyph, title: impl Into<String>, body: &str) -> AnyView {
    let title = title.into();
    let words = title.clone();
    let class = format!("fui-banner {}", tone.class());
    view! {
        <div role="note" aria-label=title class=class>
            {icon(glyph, "16")}
            <p>
                <strong>{words} ": "</strong>
                {body.to_owned()}
            </p>
        </div>
    }
    .into_any()
}
