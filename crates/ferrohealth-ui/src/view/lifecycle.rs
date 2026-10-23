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

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::island::CommitCountdown;
use crate::tone::Tone;
use crate::view::action::small_action_button;
use crate::view::button::Variant;
use crate::widget::action_menu::Action;
use crate::widget::countdown::Countdown;

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
/// use ferrohealth_ui::view::lifecycle::{Lifecycle, Step, StepState, lifecycle};
/// use leptos::prelude::*;
///
/// let html = Owner::new().with(|| {
///     lifecycle(Lifecycle {
///         label: String::from("Where this revision stands"),
///         steps: vec![
///             Step::new(StepState::Done, "Saved", "by you, 09:51"),
///             Step::new(StepState::Current, "Apply", "yours, or any operator's"),
///         ],
///     })
///     .to_html()
/// });
/// assert!(html.contains(r#"aria-current="step""#));
/// ```
#[must_use]
pub fn lifecycle(lifecycle: Lifecycle) -> AnyView {
    let Lifecycle { label, steps } = lifecycle;
    let steps = steps.into_iter().map(step).collect_view();
    view! {
        <ol class="fui-lifecycle" aria-label=label>
            {steps}
        </ol>
    }
    .into_any()
}

/// Returns one step as its list item.
fn step(step: Step) -> AnyView {
    let Step { state, title, line } = step;
    let current = (state == StepState::Current).then_some("step");
    view! {
        <li class=state.class() aria-current=current>
            <span class="fui-lifecycle-dot" aria-hidden="true">
                {state.glyph().map(|glyph| icon(glyph, "12"))}
            </span>
            <span class="fui-lifecycle-text">
                <strong>
                    <span class="fui-sr-only">{tr(state.word()).map(|words| format!("{words} "))}</span>
                    {title}
                </strong>
                <span>{line}</span>
            </span>
        </li>
    }
    .into_any()
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
/// WebAssembly; the countdown of a revision on trial is the one island it
/// may hold ([`trial_actions`]).
#[must_use]
pub fn pending_card(card: PendingCard, actions: Option<AnyView>, body: Option<AnyView>) -> AnyView {
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
    let role = tone.map(|_| "status");
    let actions = actions.map(|actions| view! { <div class="fui-actions">{actions}</div> });
    let name = title.clone();
    view! {
        <section class=class role=role aria-label=name>
            <div class="fui-pending-head">
                {icon(glyph, "18")}
                <div class="fui-revision-text">
                    <strong>{title}</strong>
                    <span>{line}</span>
                </div>
                {actions}
            </div>
            {lifecycle(steps)}
            {body}
        </section>
    }
    .into_any()
}

/// Returns the actions of a revision on trial (F2): the countdown
/// `countdown`, then `roll_back` ("Roll back now") and the primary
/// `confirm` beside it, both small buttons.
///
/// The countdown is the island [`CommitCountdown`]; with no WebAssembly it
/// shows the time left as the server drew it, and both actions still work.
#[must_use]
pub fn trial_actions(countdown: Countdown, roll_back: Action, confirm: Action) -> AnyView {
    view! {
        <CommitCountdown countdown=countdown />
        {small_action_button(roll_back, Variant::Secondary)}
        {small_action_button(confirm, Variant::Primary)}
    }
    .into_any()
}
