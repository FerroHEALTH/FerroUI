// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The timeline: events along a rail, newest first, each with its kind,
//! title, time, actor and reason and a body that opens.
//!
//! It is an ordered list whose order is the reading order. Each event's
//! dot takes its tone and is hidden from assistive technology; the kind is
//! a word with a capital, its raw event code a quiet hint after it (family
//! rule 12), and the time is absolute, as a record's times are (family rule
//! 11), through [`crate::view::time::timestamp`]. An event about
//! identifiers lists them masked, with their domains, any number of them.
//! The body folds in a native `<details>`, so it opens with no script. The
//! list has no cap in height. No specification governs the component: our
//! own design.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::tone::Tone;
use crate::view::surface::disclosure;
use crate::view::time::timestamp;
use crate::widget::identifier::{Masked, identifier_static};

/// One event of a timeline.
#[derive(Clone)]
pub struct Event {
    /// The event's key, unique in its timeline.
    pub key: String,
    /// When it happened, an RFC 3339 date-time in UTC, so the newest sorts
    /// first by its text.
    pub at: String,
    /// What kind of event it is, as a word ("Merged", "Linked
    /// automatically"); the timeline gives it its capital.
    pub kind: String,
    /// The raw event code, drawn as a quiet hint after the kind.
    pub code: Option<String>,
    /// The tone of the event's dot and kind.
    pub tone: Tone,
    /// Who acted, when someone did.
    pub actor: Option<String>,
    /// The event's one line.
    pub title: String,
    /// The identifiers the event concerns, masked, in the order given.
    pub identifiers: Vec<Masked>,
    /// The reason the actor gave, when there is one.
    pub reason: Option<String>,
    /// The details that open under the event, such as a version's diff.
    pub body: Option<ViewFn>,
}

impl std::fmt::Debug for Event {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Event")
            .field("key", &self.key)
            .field("at", &self.at)
            .field("tone", &self.tone)
            .finish_non_exhaustive()
    }
}

/// Returns `word` with its first letter a capital.
#[must_use]
pub fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// The events of `events` newest first: the latest `at` leads, events of
/// one moment keep the order given.
fn newest_first(events: &[Event]) -> Vec<&Event> {
    let mut sorted: Vec<&Event> = events.iter().collect();
    sorted.sort_by(|a, b| b.at.cmp(&a.at));
    sorted
}

/// Returns one event of the timeline.
fn event(event: &Event) -> AnyView {
    let class = format!("fui-timeline-event {}", event.tone.class());
    let kind = capitalised(&event.kind);
    let code = event
        .code
        .clone()
        .map(|code| view! { <span class="fui-timeline-code">{code}</span> });
    let actor = event
        .actor
        .clone()
        .map(|actor| view! { <span class="fui-timeline-meta">{actor}</span> });
    let identifiers = (!event.identifiers.is_empty()).then(|| {
        let items = event
            .identifiers
            .iter()
            .map(|masked| identifier_static(masked, true))
            .collect_view()
            .into_any();
        view! { <div class="fui-timeline-identifiers">{items}</div> }
    });
    let reason = event.reason.clone().map(|reason| {
        view! {
            <p class="fui-timeline-reason">
                <span class="fui-timeline-reason-label">{tr(Text::TimelineReason)}</span>
                " "
                {reason}
            </p>
        }
    });
    let details = event
        .body
        .as_ref()
        .map(|body| disclosure(tr(Text::TimelineDetails), body.run()));
    let body = (reason.is_some() || details.is_some()).then(|| {
        view! {
            <div class="fui-timeline-body">
                {reason}
                {details}
            </div>
        }
    });
    view! {
        <li class=class data-key=event.key.clone()>
            <span class="fui-timeline-dot" aria-hidden="true"></span>
            <div>
                <div class="fui-timeline-head">
                    <span class=event.tone.badge()>{kind}</span>
                    {code}
                    <span class="fui-timeline-title">{event.title.clone()}</span>
                    <span class="fui-timeline-meta">{timestamp(&event.at)}</span>
                    {actor}
                </div>
                {identifiers}
                {body}
            </div>
        </li>
    }
    .into_any()
}

/// Returns the timeline named `label` of `events`, newest first.
#[must_use]
pub fn timeline(label: impl Into<String>, events: &[Event]) -> AnyView {
    let label = label.into();
    let items = newest_first(events)
        .into_iter()
        .map(event)
        .collect_view()
        .into_any();
    view! {
        <ol class="fui-timeline" aria-label=label>
            {items}
        </ol>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::capitalised;

    #[test]
    fn a_kind_takes_its_capital_in_any_script() {
        assert_eq!("Merged", capitalised("merged"));
        assert_eq!("Élu", capitalised("élu"));
        assert_eq!("Linked automatically", capitalised("Linked automatically"));
        assert_eq!("", capitalised(""));
    }
}
