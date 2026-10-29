// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The revision lifecycle and the pending card (F1), and the actions of a
//! revision on trial with its commit-confirm countdown (F2).
//!
//! A configuration revision goes through Saved, Approval, Apply and
//! Confirm. Each step is done, current, waiting, not needed or failed, and
//! says so in a shape, a colour and words together (family rule 12): the
//! dot's glyph and colour repeat the step's words and its hidden state
//! word, so no state rides on colour or shape alone. A step a revision does
//! not take is drawn as not needed, never left out, so every revision reads
//! the same way. The steps are an ordered list, the current one
//! `aria-current="step"` (WAI-ARIA 1.2, `aria-current`); a phone stacks
//! them. The pending card puts the revision's words, its actions and its
//! lifecycle together. No specification governs the components: our own
//! design.

use crate::html::Html;
use crate::i18n::{Locale, Text, tr};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::action::small_action_button;
use crate::view::action_menu::Action;
use crate::view::button::Variant;
use crate::view::countdown::{Countdown, countdown};

/// Where a step of a lifecycle stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    /// The step is done: a filled check.
    Done,
    /// The step is the one in progress: a ring in the accent.
    Current,
    /// The step is still to come: a hollow dot.
    Waiting,
    /// The revision does not take the step: a dashed dot and a dash.
    Skipped,
    /// The step failed: a filled cross in the bad tone.
    Failed,
}

impl StepState {
    /// The class the step's list item takes.
    const fn class(self) -> &'static str {
        match self {
            Self::Done => "is-done",
            Self::Current => "is-current",
            Self::Waiting => "is-waiting",
            Self::Skipped => "is-skipped",
            Self::Failed => "is-failed",
        }
    }

    /// The glyph in the step's dot, when it has one.
    const fn glyph(self) -> Option<Glyph> {
        match self {
            Self::Done => Some(Glyph::CHECK),
            Self::Failed => Some(Glyph::X),
            Self::Skipped => Some(Glyph::MINUS),
            Self::Current | Self::Waiting => None,
        }
    }

    /// The state's hidden word.
    const fn word(self) -> Text {
        match self {
            Self::Done => Text::LifecycleDone,
            Self::Current => Text::LifecycleCurrent,
            Self::Waiting => Text::LifecycleWaiting,
            Self::Skipped => Text::LifecycleSkipped,
            Self::Failed => Text::LifecycleFailed,
        }
    }
}

/// One step of a lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// Where the step stands.
    pub state: StepState,
    /// The step's name, such as "Saved", "Approval" or "Not confirmed".
    pub title: String,
    /// Who and when, or why: "by you, 10:02", "another operator", "not
    /// needed".
    pub line: String,
}

impl Step {
    /// The step `title` in `state`, with `line` under it.
    #[must_use]
    pub fn new(state: StepState, title: impl Into<String>, line: impl Into<String>) -> Self {
        Self {
            state,
            title: title.into(),
            line: line.into(),
        }
    }
}

/// The steps of a revision, named for assistive technology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifecycle {
    /// The list's name, such as "Where this revision stands".
    pub label: String,
    /// The steps, in order.
    pub steps: Vec<Step>,
}

/// Returns the lifecycle `lifecycle`: one line of steps from 40 rem up,
/// stacked below it.
///
/// ```
/// use ferrohealth_ui::i18n::Locale;
/// use ferrohealth_ui::view::lifecycle::{Lifecycle, Step, StepState, lifecycle};
///
/// let html = lifecycle(
///     &Locale::english(),
///     Lifecycle {
///         label: String::from("Where this revision stands"),
///         steps: vec![
///             Step::new(StepState::Done, "Saved", "by you, 09:51"),
///             Step::new(StepState::Current, "Apply", "yours, or any operator's"),
///         ],
///     },
/// ).to_string();
/// assert!(html.contains(r#"aria-current="step""#));
/// ```
#[must_use]
pub fn lifecycle(locale: &Locale, lifecycle: Lifecycle) -> Html {
    let Lifecycle { label, steps } = lifecycle;
    let steps: String = steps
        .into_iter()
        .map(|each| step(locale, each))
        .collect::<Vec<_>>()
        .concat();
    Html::raw(format!(
        "<ol aria-label=\"{}\" class=\"fui-lifecycle\">{steps}</ol>",
        Html::text(label)
    ))
}

/// Returns one step as its list item.
fn step(locale: &Locale, step: Step) -> String {
    let Step { state, title, line } = step;
    let current = if state == StepState::Current {
        " aria-current=\"step\""
    } else {
        ""
    };
    let word = tr(locale, state.word()).map(|words| format!("{words} "));
    format!(
        "<li{current} class=\"{}\"><span aria-hidden=\"true\" class=\"fui-lifecycle-dot\">{}</span><span class=\"fui-lifecycle-text\"><strong><span class=\"fui-sr-only\">{word}</span>{}</strong><span>{}</span></span></li>",
        state.class(),
        state
            .glyph()
            .map(|glyph| icon(glyph, "12"))
            .unwrap_or_default(),
        Html::text(title),
        Html::text(line)
    )
}

/// A revision waiting on a step: its words, its glyph, its tone and its
/// lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCard {
    /// What stands, such as "Revision 10 waits for a second operator"; the
    /// card's name too.
    pub title: String,
    /// One sentence on what it changes and who acts next.
    pub line: String,
    /// The glyph before the words.
    pub glyph: Glyph,
    /// The card's tone; `None` for the accent of a revision going its
    /// normal way, a tone for one on trial, confirmed or rolled back, whose
    /// card is then a polite status region.
    pub tone: Option<Tone>,
    /// The revision's steps.
    pub lifecycle: Lifecycle,
}

/// Returns the pending card `card`: its glyph and words with `actions` at
/// the row's end, then its lifecycle, then `body` (a line on why an action
/// is not open yet, or what to check first).
///
/// Each action is a link or a form post, so the card works with no
/// script; the countdown of a revision on trial is the one part that
/// counts in the browser ([`trial_actions`]).
#[must_use]
pub fn pending_card(
    locale: &Locale,
    card: PendingCard,
    actions: Option<Html>,
    body: Option<Html>,
) -> Html {
    let PendingCard {
        title,
        line,
        glyph,
        tone,
        lifecycle: steps,
    } = card;
    let class = tone.map_or_else(
        || String::from("fui-pending"),
        |tone| format!("fui-pending {}", tone.class()),
    );
    let role = if tone.is_some() {
        " role=\"status\""
    } else {
        ""
    };
    let actions = actions
        .map(|actions| format!("<div class=\"fui-actions\">{actions}</div>"))
        .unwrap_or_default();
    let name = Html::text(&title);
    Html::raw(format!(
        "<section{role} aria-label=\"{name}\" class=\"{class}\"><div class=\"fui-pending-head\">{}<div class=\"fui-revision-text\"><strong>{name}</strong><span>{}</span></div>{actions}</div>{}{}</section>",
        icon(glyph, "18"),
        Html::text(line),
        lifecycle(locale, steps),
        body.unwrap_or_default()
    ))
}

/// Returns the actions of a revision on trial (F2): the countdown
/// `countdown`, then `roll_back` ("Roll back now") and the primary
/// `confirm` beside it, both small buttons.
///
/// The countdown is [`crate::view::countdown::countdown`]; with no script it
/// shows the time left as the server drew it, and both actions still work.
#[must_use]
pub fn trial_actions(
    locale: &Locale,
    timer: Countdown,
    roll_back: Action,
    confirm: Action,
) -> Html {
    countdown(timer)
        .then(small_action_button(locale, roll_back, Variant::Secondary))
        .then(small_action_button(locale, confirm, Variant::Primary))
}
