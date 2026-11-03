// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The plan of a flow: the folded lanes, the edges between what is shown, and
//! the order.

use std::collections::{BTreeMap, BTreeSet};

use crate::i18n::{Locale, Text, tr_with};

use super::layout::{CARD, units};
use super::order::ordered;
use super::{FOLD_PAST, Flow, FlowCard, FlowState};

/// The summary of a lane's folded quiet cards.
#[derive(Debug)]
pub(super) struct Folded<'a> {
    /// The locale its words are drawn in.
    pub(super) locale: &'a Locale,
    /// Its id, which the folded cards' edges and paths name instead.
    id: String,
    /// The lane's title.
    pub(super) title: &'a str,
    /// How many folded cards are in each state, in the order of the states.
    counts: Vec<(FlowState, usize)>,
    /// The page that unfolds the lane.
    pub(super) href: &'a str,
}

impl Folded<'_> {
    /// How many cards it holds.
    pub(super) fn count(&self) -> usize {
        self.counts.iter().map(|(_, count)| count).sum()
    }

    /// The state its dot shows: up while any card it holds is up.
    pub(super) fn state(&self) -> FlowState {
        self.counts
            .first()
            .map_or(FlowState::Up, |(state, _)| *state)
    }

    /// Each state with its count.
    fn parts(&self) -> Vec<String> {
        self.counts
            .iter()
            .map(|(state, count)| {
                String::from(tr_with(
                    self.locale,
                    Text::FlowFoldCount,
                    &[
                        ("status", state.word(self.locale).into()),
                        ("count", (*count).into()),
                    ],
                ))
            })
            .collect()
    }

    /// Its line: each state with its count.
    pub(super) fn line(&self) -> String {
        self.parts()
            .into_iter()
            .reduce(|first, next| joined(self.locale, &first, &next))
            .unwrap_or_default()
    }

    /// Its line broken into the lines its card draws, each at most
    /// [`LINE_MOST`] characters, so no count is cut.
    pub(super) fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for part in self.parts() {
            let longer = lines.last().map(|last| joined(self.locale, last, &part));
            match (lines.last_mut(), longer) {
                (Some(last), Some(longer)) if longer.chars().count() <= LINE_MOST => *last = longer,
                _ => lines.push(part),
            }
        }
        lines
    }

    /// The height of its card: one more line's room for each line after
    /// the first.
    pub(super) fn height(&self) -> f64 {
        CARD + LINE * units(self.lines().len().saturating_sub(1))
    }
}

/// The counts `first` and `next` of a summary card's line, joined.
fn joined(locale: &Locale, first: &str, next: &str) -> String {
    tr_with(
        locale,
        Text::FlowFoldJoin,
        &[("first", first.into()), ("last", next.into())],
    )
    .into()
}

/// The most characters of a flow card's name and of its line.
pub(super) const NAME_MOST: usize = 23;
pub(super) const LINE_MOST: usize = 22;

/// The room a card's line takes.
pub(super) const LINE: f64 = 14.0;

/// One card as the drawing shows it: a card of the flow, or the summary of
/// its lane's folded cards.
#[derive(Debug)]
pub(super) enum Shown<'a> {
    Card(&'a FlowCard),
    Fold(Folded<'a>),
}

impl Shown<'_> {
    /// The id its edges name it by.
    pub(super) fn id(&self) -> &str {
        match self {
            Self::Card(card) => &card.id,
            Self::Fold(folded) => &folded.id,
        }
    }
}

/// One lane as the drawing shows it: its cards in their drawn order, the
/// summary last, and its fold link with whether it is folded.
#[derive(Debug)]
pub(super) struct Planned<'a> {
    pub(super) title: &'a str,
    pub(super) shown: Vec<Shown<'a>>,
    pub(super) toggle: Option<(bool, &'a str)>,
}

/// One edge as the drawing draws it, its ends the shown cards' ids.
#[derive(Debug)]
pub(super) struct Drawn<'a> {
    pub(super) from: String,
    pub(super) to: String,
    pub(super) state: FlowState,
    pub(super) label: Option<&'a str>,
}

/// A flow as the drawing shows it: the lanes folded and ordered, the
/// edges between the shown cards, and the summary that stands for each
/// folded card.
#[derive(Debug)]
pub(super) struct Plan<'a> {
    pub(super) lanes: Vec<Planned<'a>>,
    pub(super) edges: Vec<Drawn<'a>>,
    alias: BTreeMap<&'a str, String>,
}

impl Plan<'_> {
    /// The id of the shown card that stands for the card `id`.
    pub(super) fn shown_id<'b>(&'b self, id: &'b str) -> &'b str {
        self.alias.get(id).map_or(id, String::as_str)
    }
}

/// The state of an edge that stands for quiet edges in `states`: up while
/// any is up, else not checked while any is, else off.
fn quietest(states: impl Iterator<Item = FlowState>) -> FlowState {
    states
        .min_by_key(|state| match state {
            FlowState::Up => 0,
            FlowState::NotChecked => 1,
            FlowState::Off => 2,
            FlowState::Degraded => 3,
            FlowState::Unreachable => 4,
        })
        .unwrap_or_default()
}

/// The lanes of `flow` folded as their folds say, each fold's summary
/// standing for the cards it holds.
fn folded<'a>(locale: &'a Locale, flow: &'a Flow) -> (Vec<Planned<'a>>, BTreeMap<&'a str, String>) {
    let troubled: BTreeSet<&str> = flow
        .edges
        .iter()
        .filter(|edge| edge.state.is_problem())
        .flat_map(|edge| [edge.from.as_str(), edge.to.as_str()])
        .collect();
    let mut alias = BTreeMap::new();
    let lanes = flow
        .lanes
        .iter()
        .enumerate()
        .map(|(column, lane)| {
            let quiet = |card: &FlowCard| {
                !card.hub && !card.state.is_problem() && !troubled.contains(card.id.as_str())
            };
            let foldable = lane.fold.as_ref().filter(|_| {
                lane.cards.len() > FOLD_PAST
                    && lane.cards.iter().filter(|card| quiet(card)).count() > 1
            });
            let toggle = foldable.map(|fold| (!fold.open, fold.toggle.as_str()));
            let mut shown: Vec<Shown<'_>> = Vec::new();
            match foldable.filter(|fold| !fold.open) {
                Some(fold) => {
                    let id = format!("{}-fold-{column}", flow.id);
                    let mut counts = BTreeMap::new();
                    for card in &lane.cards {
                        if quiet(card) {
                            alias.insert(card.id.as_str(), id.clone());
                            *counts.entry(rank(card.state)).or_insert(0_usize) += 1;
                        } else {
                            shown.push(Shown::Card(card));
                        }
                    }
                    shown.push(Shown::Fold(Folded {
                        locale,
                        id,
                        title: &lane.title,
                        counts: counts
                            .into_iter()
                            .map(|(at, count)| (state_at(at), count))
                            .collect(),
                        href: &fold.toggle,
                    }));
                }
                None => shown.extend(lane.cards.iter().map(Shown::Card)),
            }
            Planned {
                title: &lane.title,
                shown,
                toggle,
            }
        })
        .collect();
    (lanes, alias)
}

/// The states in the order a summary counts them.
const STATES: [FlowState; 5] = [
    FlowState::Up,
    FlowState::NotChecked,
    FlowState::Off,
    FlowState::Degraded,
    FlowState::Unreachable,
];

/// The place of `state` in [`STATES`].
fn rank(state: FlowState) -> usize {
    STATES
        .iter()
        .position(|each| *each == state)
        .unwrap_or_default()
}

/// The state at `at` in [`STATES`].
fn state_at(at: usize) -> FlowState {
    STATES.get(at).copied().unwrap_or_default()
}

/// The edges of `flow` between the shown cards: an edge into or out of a
/// summary stands for every quiet edge it holds between the same two cards.
fn drawn<'a>(flow: &'a Flow, alias: &BTreeMap<&'a str, String>) -> Vec<Drawn<'a>> {
    let mut edges: Vec<Drawn<'a>> = Vec::new();
    let mut merged: BTreeMap<(String, String), usize> = BTreeMap::new();
    for edge in &flow.edges {
        let from = alias.get(edge.from.as_str());
        let to = alias.get(edge.to.as_str());
        let ends = (
            from.cloned().unwrap_or_else(|| edge.from.clone()),
            to.cloned().unwrap_or_else(|| edge.to.clone()),
        );
        if from.is_none() && to.is_none() {
            edges.push(Drawn {
                from: ends.0,
                to: ends.1,
                state: edge.state,
                label: edge.label.as_deref(),
            });
            continue;
        }
        if let Some(at) = merged.get(&ends).copied() {
            if let Some(kept) = edges.get_mut(at) {
                kept.state = quietest([kept.state, edge.state].into_iter());
            }
            continue;
        }
        merged.insert(ends.clone(), edges.len());
        edges.push(Drawn {
            from: ends.0,
            to: ends.1,
            state: edge.state,
            label: None,
        });
    }
    edges
}

/// `flow` as the drawing shows it: each lane folded as its fold says,
/// then ordered to cross the fewest edges.
pub(super) fn plan<'a>(locale: &'a Locale, flow: &'a Flow) -> Plan<'a> {
    let (mut lanes, alias) = folded(locale, flow);
    let edges = drawn(flow, &alias);
    let order = {
        let ids: Vec<Vec<&str>> = lanes
            .iter()
            .map(|lane| lane.shown.iter().map(Shown::id).collect())
            .collect();
        let pinned: Vec<bool> = lanes
            .iter()
            .map(|lane| matches!(lane.shown.last(), Some(Shown::Fold(_))))
            .collect();
        let pairs: Vec<(&str, &str)> = edges
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        ordered(&ids, &pinned, &pairs)
            .into_iter()
            .map(|lane| lane.into_iter().map(str::to_owned).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    };
    for (lane, ids) in lanes.iter_mut().zip(order) {
        let mut shown = std::mem::take(&mut lane.shown);
        for id in ids {
            if let Some(at) = shown.iter().position(|each| each.id() == id) {
                lane.shown.push(shown.remove(at));
            }
        }
        lane.shown.append(&mut shown);
    }
    Plan {
        lanes,
        edges,
        alias,
    }
}
