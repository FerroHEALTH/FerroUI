// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The load report (B4): a loaded set's outcome, or an operation's
//! verdict, in one line.
//!
//! The line holds the outcome's glyph, a title, the counts and at most one
//! action at its end, under the page title or at the top of a panel. Ok,
//! warn and bad; the bad outcome takes the bad ground. It replaces a toast
//! for the same outcome. On a phone the glyph keeps its line beside the
//! title and the counts wrap under it. The glyph's shape and the title's
//! words carry the outcome, never the colour alone (WCAG 2.2 SC 1.4.1).
//! No specification governs the line: our own design.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::tone::Tone;

/// How a set loaded, or how an operation answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// It loaded, or the answer is what was asked for.
    Ok,
    /// It loaded with findings to look at.
    Warn,
    /// It did not load, or the answer is a refusal.
    Bad,
}

impl Outcome {
    /// Every outcome, in the order the gallery shows them.
    pub const ALL: [Self; 3] = [Self::Ok, Self::Warn, Self::Bad];

    /// The tone the line is drawn in.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Ok => Tone::Ok,
            Self::Warn => Tone::Warn,
            Self::Bad => Tone::Bad,
        }
    }

    /// The glyph before the title: a check, a triangle or a cross.
    #[must_use]
    pub const fn glyph(self) -> Glyph {
        match self {
            Self::Ok => Glyph::CIRCLE_CHECK,
            Self::Warn => Glyph::TRIANGLE_ALERT,
            Self::Bad => Glyph::CIRCLE_X,
        }
    }
}

/// When the report reaches the reader, which decides whether it is
/// announced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Arrival {
    /// The page loads with it: plain content, read in its place.
    #[default]
    WithPage,
    /// It appears after the reader's action, such as a reload or a run: a
    /// status message, read when the reader is free (WAI-ARIA 1.2, the
    /// `status` role).
    AfterAction,
}

/// One count of the report, such as "3 warnings".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Count {
    /// The number, as the console writes it, drawn strong.
    pub value: String,
    /// The words after it, in the form the number takes.
    pub noun: String,
}

/// What a load report says.
#[derive(Debug)]
pub struct LoadReport {
    /// How it went.
    pub outcome: Outcome,
    /// When it reaches the reader.
    pub arrival: Arrival,
    /// The title, a sentence that may hold a `<strong>` count or a
    /// `<code>` name.
    pub title: AnyView,
    /// The counts after the title, none for a verdict with nothing to count.
    pub counts: Vec<Count>,
    /// The one action at the line's end, such as a link to the findings.
    pub action: Option<AnyView>,
}

/// Returns the load report `report`.
#[must_use]
pub fn load_report(report: LoadReport) -> AnyView {
    let LoadReport {
        outcome,
        arrival,
        title,
        counts,
        action,
    } = report;
    let class = format!("fui-load-report {}", outcome.tone().class());
    let role = (arrival == Arrival::AfterAction).then_some("status");
    let counts = (!counts.is_empty()).then(|| {
        let items = counts
            .into_iter()
            .map(|Count { value, noun }| {
                view! {
                    <li>
                        <strong>{value}</strong>
                        " "
                        {noun}
                    </li>
                }
            })
            .collect_view();
        view! { <ul class="fui-load-counts">{items}</ul> }
    });
    let action = action.map(|action| view! { <div class="fui-actions">{action}</div> });
    view! {
        <div class=class role=role>
            {icon(outcome.glyph(), "18")}
            <span class="fui-load-report-title">{title}</span>
            {counts}
            {action}
        </div>
    }
    .into_any()
}
