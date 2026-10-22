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
//! The body folds in a native `<details>`, so it opens with no script.
//! Events sort by the moment they name, so instants written with different
//! offsets keep the order they happened in. The list has no cap in height;
//! a long history shows its newest events through [`timeline_window`],
//! which says how many it shows and links to the older ones. No
//! specification governs the component: our own design.

use leptos::prelude::*;

use std::cmp::Ordering;

use crate::i18n::{Text, tr, tr_with};
use crate::tone::Tone;
use crate::view::surface::disclosure;
use crate::view::time::{moment, timestamp};
use crate::widget::identifier::{Masked, identifier_static};

/// One event of a timeline.
#[derive(Clone)]
pub struct Event {
    /// The event's key, unique in its timeline.
    pub key: String,
    /// When it happened, an RFC 3339 date-time with any offset; the
    /// timeline orders the events by the moment it names.
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

/// The events of `events` newest first: the latest moment leads, events
/// of one moment keep the order given, and an `at` that is no date-time
/// follows every one that is, newest by its text.
fn newest_first(events: &[Event]) -> Vec<&Event> {
    let mut sorted: Vec<(Option<_>, &Event)> = events
        .iter()
        .map(|event| (moment(&event.at), event))
        .collect();
    sorted.sort_by(|(a_moment, a), (b_moment, b)| match (a_moment, b_moment) {
        (Some(a_moment), Some(b_moment)) => b_moment.cmp(a_moment),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => b.at.cmp(&a.at),
    });
    sorted.into_iter().map(|(_, event)| event).collect()
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

/// Returns the timeline named `label` of `events`, newest first, every
/// event drawn.
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

/// How much of a long timeline shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window<'a> {
    /// The most events shown, the newest first.
    pub limit: usize,
    /// How many events the history holds in all, when the console passes
    /// only the newest; `None` counts the events passed.
    pub total: Option<usize>,
    /// The console's page that shows the older events, such as the same
    /// page with a larger limit, a plain link that works with no script.
    pub older: &'a str,
}

/// Returns the timeline named `label` of the newest `window.limit` of
/// `events`, and, when the history holds more, a foot saying how many show
/// of how many with a link to the older ones.
#[must_use]
pub fn timeline_window(label: impl Into<String>, events: &[Event], window: Window<'_>) -> AnyView {
    let label = label.into();
    let sorted = newest_first(events);
    let shown = sorted.len().min(window.limit);
    let total = window.total.unwrap_or(sorted.len()).max(sorted.len());
    let items = sorted
        .into_iter()
        .take(shown)
        .map(event)
        .collect_view()
        .into_any();
    let more = (total > shown).then(|| {
        let said = tr_with(
            Text::TimelineShown,
            &[("shown", shown.into()), ("total", total.into())],
        );
        view! {
            <p class="fui-timeline-more">
                <span class="fui-small fui-muted">{said}</span>
                <a href=window.older.to_owned()>{tr(Text::TimelineOlder)}</a>
            </p>
        }
    });
    view! {
        <ol class="fui-timeline" aria-label=label>
            {items}
        </ol>
        {more}
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
