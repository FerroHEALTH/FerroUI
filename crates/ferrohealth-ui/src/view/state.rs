// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The state mark (F12): a shape, a colour and a word together.
//!
//! A table cell, a fact or a title badge draws a state such as
//! "Reachable", "Degraded" or "Compiles" this way, never by the colour
//! alone (family rule 12).
//!
//! The shape follows the mark: a filled circle, a diamond, a cross, a
//! hollow ring, and the hollow ring with a muted word for a state switched
//! off. The word is the state's name, so the shape is hidden from
//! assistive technology (WCAG 2.2 SC 1.1.1, SC 1.4.1). No specification
//! governs the shapes: our own design.

use leptos::prelude::*;

use crate::i18n::Word;
use crate::tone::Tone;
use crate::view::flow::FlowState;

/// What a state mark says, which sets its shape and its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// It works: a filled circle in the ok tone.
    Ok,
    /// It works with a concern: a diamond in the warn tone.
    Warn,
    /// It does not work: a cross in the bad tone.
    Bad,
    /// Nothing is known yet: a hollow ring in the neutral tone.
    Neutral,
    /// It is switched off or cleared: a hollow ring with the word muted.
    Off,
}

impl Mark {
    /// The tone the mark is drawn in.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Ok => Tone::Ok,
            Self::Warn => Tone::Warn,
            Self::Bad => Tone::Bad,
            Self::Neutral | Self::Off => Tone::Neutral,
        }
    }
}

impl From<FlowState> for Mark {
    fn from(state: FlowState) -> Self {
        match state {
            FlowState::Up => Self::Ok,
            FlowState::Degraded => Self::Warn,
            FlowState::Unreachable => Self::Bad,
            FlowState::NotChecked => Self::Neutral,
            FlowState::Off => Self::Off,
        }
    }
}

/// Returns the state mark `mark` beside `word`, its first letter a
/// capital ("Reachable"), with the quiet `detail` after it when there is
/// one, such as a latency or a count.
///
/// # Examples
///
/// ```
/// use ferrohealth_ui::view::state::{Mark, state};
/// use leptos::prelude::*;
///
/// let html = Owner::new().with(|| state(Mark::Warn, "Degraded", Some(String::from("2,940 ms"))).to_html());
/// assert!(html.contains("fui-state fui-tone-warn"));
/// assert!(html.contains("2,940 ms"));
/// ```
#[must_use]
pub fn state(mark: Mark, word: impl Into<Word>, detail: Option<String>) -> AnyView {
    let word: Word = word.into();
    let off = mark == Mark::Off;
    let tone = mark.tone().class();
    let detail = detail.map(|detail| view! { " " <span class="fui-status-detail">{detail}</span> });
    view! {
        <span class=format!("fui-state {tone}") class:is-muted=off>
            <span class=format!("fui-dot {tone}") class:is-muted=off aria-hidden="true"></span>
            {word}
            {detail}
        </span>
    }
    .into_any()
}
