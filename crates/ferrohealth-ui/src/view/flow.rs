// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The flow (B9): F9's cards and edges laid out as a pipeline.
//!
//! Its lanes run from the callers on the left to the upstreams on the right,
//! with one quiet foot line, and on a phone the same cards are a list, lane
//! by lane.
//!
//! An edge's state is its line style, its colour and, on a problem edge,
//! one end mark and a label (family rule 12). The drawing is two SVG
//! layers of one size: the lanes and the edges, an image whose description
//! names each problem edge (WCAG 2.2 SC 1.1.1), and over it the cards, each
//! a link to its page. Hovering or focusing a card shows its path with the
//! pulse; the stylesheet does it, so it needs no script, changes paint
//! only (family rule 3) and stills under reduced motion. No specification
//! governs the drawing: our own design.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use crate::html::Html;
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::button::{Variant, label};
use crate::view::dialog::{Explanation, explain};

/// The state of a card or an edge, drawn as a shape, a colour and a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowState {
    /// It answers: a solid line, a filled dot.
    #[default]
    Up,
    /// It answers slowly or with an error: a dashed line marked "!", a
    /// diamond.
    Degraded,
    /// It does not answer: a dotted line marked with a cross, a cross.
    Unreachable,
    /// No probe has read it: a dash-dot line, a hollow ring.
    NotChecked,
    /// It is switched off: a thin faded line, a muted card.
    Off,
}

impl FlowState {
    /// The state's word in the page's locale.
    #[must_use]
    pub fn word(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::Up => Text::FlowUp,
                Self::Degraded => Text::FlowDegraded,
                Self::Unreachable => Text::FlowUnreachable,
                Self::NotChecked => Text::FlowNotChecked,
                Self::Off => Text::FlowOff,
            },
        )
    }

    /// The key a state is sent and named by, such as `degraded`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Degraded => "degraded",
            Self::Unreachable => "unreachable",
            Self::NotChecked => "not-checked",
            Self::Off => "off",
        }
    }

    /// Whether an edge in this state is a problem, which carries a mark and
    /// a label.
    #[must_use]
    pub const fn is_problem(self) -> bool {
        matches!(self, Self::Degraded | Self::Unreachable)
    }

    /// The tone of a card or a dot in this state.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Up => Tone::Ok,
            Self::Degraded => Tone::Warn,
            Self::Unreachable => Tone::Bad,
            Self::NotChecked | Self::Off => Tone::Neutral,
        }
    }

    /// How an edge in this state is drawn, such as "dashed, marked !", in
    /// the page's locale.
    #[must_use]
    pub fn line(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::Up => Text::FlowStyleUp,
                Self::Degraded => Text::FlowStyleDegraded,
                Self::Unreachable => Text::FlowStyleUnreachable,
                Self::NotChecked => Text::FlowStyleNotChecked,
                Self::Off => Text::FlowStyleOff,
            },
        )
    }

    /// The classes of an edge in this state.
    pub(crate) const fn edge(self) -> &'static str {
        match self {
            Self::Up => "fui-edge fui-edge-ok",
            Self::Degraded => "fui-edge fui-edge-warn",
            Self::Unreachable => "fui-edge fui-edge-bad",
            Self::NotChecked => "fui-edge fui-edge-neutral",
            Self::Off => "fui-edge fui-edge-muted",
        }
    }
}

/// A short sample of an edge's line in `state`, for a legend or a rule.
pub(crate) fn line_sample(state: FlowState) -> Html {
    Html::raw(format!(
        "<svg width=\"26\" height=\"10\" aria-hidden=\"true\" class=\"fui-line-sample\"><path d=\"M1 5 H25\" class=\"{}\"></path></svg>",
        state.edge()
    ))
}

/// One state of an edge rule and what it means where the console draws
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeRule {
    /// The state.
    pub state: FlowState,
    /// What it means, such as "it answers slowly or with an error".
    pub meaning: String,
}

/// Returns the edge rule `rules`: one line per state, its line sample, its
/// word, how its line is drawn and what it means, such as "Degraded ·
/// dashed, marked !: it answers slowly or with an error".
///
/// The line sample is hidden from a reader, who reads the words; the same
/// rule explains the flow (B9) and the topology (F9).
#[must_use]
pub fn edge_rule(locale: &Locale, rules: Vec<EdgeRule>) -> Html {
    let items: Html = rules
        .into_iter()
        .map(|EdgeRule { state, meaning }| {
            let rest = state
                .line(locale)
                .map(|line| format!(" · {line}: {meaning}"));
            Html::raw(format!(
                "<li>{}<span><strong>{}</strong>{}</span></li>",
                line_sample(state),
                state.word(locale),
                Html::text(rest.text())
            ))
        })
        .collect();
    Html::raw(format!("<ul class=\"fui-topology-rule\">{items}</ul>"))
}

/// What a flow's "How the flow is drawn" dialog explains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowRule {
    /// The dialog's id, which [`rule_button`] opens.
    pub id: Cow<'static, str>,
    /// The rule, one state a line.
    pub rules: Vec<EdgeRule>,
    /// The closing paragraph, such as what a card shows and opens.
    pub note: String,
}

/// Returns "How the flow is drawn": the short informational dialog with
/// the edge rule and its closing paragraph, and Close as its only action.
#[must_use]
pub fn rule_dialog(locale: &Locale, rule: FlowRule) -> Html {
    let FlowRule { id, rules, note } = rule;
    let body = edge_rule(locale, rules).then(Html::raw(format!(
        "<p class=\"fui-small fui-muted\">{}</p>",
        Html::text(note)
    )));
    explain(
        locale,
        Explanation {
            id,
            title: tr(locale, Text::FlowRuleTitle),
        },
        body,
        false,
    )
}

/// Returns "How it is drawn", the small ghost button of a flow's panel
/// head that opens the dialog `dialog` ([`rule_dialog`]) with the
/// `show-modal` command, so it opens with no script.
#[must_use]
pub fn rule_button(locale: &Locale, dialog: &str) -> Html {
    Html::raw(format!(
        "<button type=\"button\" command=\"show-modal\" commandfor=\"{}\" aria-haspopup=\"dialog\" class=\"{}\">{}</button>",
        Html::text(dialog),
        Variant::Ghost.small(),
        label(Some(Glyph::INFO), tr(locale, Text::FlowRuleButton))
    ))
}

/// One card of a flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowCard {
    /// The id its edges and paths name it by.
    pub id: String,
    /// The glyph before its name.
    pub glyph: Glyph,
    /// Its name, such as "openEHR CDR".
    pub name: String,
    /// Its line under the name, such as "FerroEHR · Up".
    pub line: String,
    /// Its state.
    pub state: FlowState,
    /// The page it opens.
    pub href: String,
    /// The ids of the cards its path runs through, in order, such as the
    /// caller, the face, the hub and the upstream; hovering or focusing the
    /// card shows the edges between them with the pulse. Empty for none.
    pub path: Vec<String>,
    /// Whether it is the hub of the flow: the accent card with no dot.
    pub hub: bool,
}

/// One lane of a flow, a column of cards under its title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowLane {
    /// Its title, such as "Callers".
    pub title: String,
    /// Its cards, top to bottom as the console passes them; the drawing
    /// orders them to cross fewer edges.
    pub cards: Vec<FlowCard>,
    /// How it folds past [`FOLD_PAST`] cards, or `None` for a lane that
    /// never folds.
    pub fold: Option<LaneFold>,
}

/// The most cards a lane shows before it folds its quiet ones: eight rows
/// keep the drawing about one screen tall at 1440 px.
pub const FOLD_PAST: usize = 8;

/// How a lane past [`FOLD_PAST`] cards folds its quiet cards, those in no
/// problem state and on no problem edge, into one summary card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneFold {
    /// Whether the reader unfolded it to show every card.
    pub open: bool,
    /// The page that unfolds the lane while it is folded and folds it while
    /// it is open, such as `?open=callers`.
    pub toggle: String,
}

/// One edge of a flow, from a card's right side to a card's left side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowEdge {
    /// The id of the card it leaves.
    pub from: String,
    /// The id of the card it reaches, where its end mark sits.
    pub to: String,
    /// Its state.
    pub state: FlowState,
    /// Its label, such as "503", drawn only on a problem edge.
    pub label: Option<String>,
}

/// What a flow shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    /// The drawing's id; its title and description take it with `-title`
    /// and `-desc`.
    pub id: String,
    /// The drawing's accessible name, such as "The bridge as a flow".
    pub label: String,
    /// The accessible name of the cards and of the phone's list, such as
    /// "The bridge as a flow, lane by lane".
    pub list_label: String,
    /// The lanes, left to right.
    pub lanes: Vec<FlowLane>,
    /// The edges. One that names no card is not drawn.
    pub edges: Vec<FlowEdge>,
    /// The quiet foot line, such as "3 faces on · 3 of 3 upstreams up.".
    pub foot: String,
}

/// A lane's width, the gap between lanes, a card's height, the hub's
/// height, a row's height and the room above the first row, in the
/// drawing's units.
const LANE: f64 = 200.0;
const GAP: f64 = 100.0;
const CARD: f64 = 56.0;
const HUB: f64 = 72.0;
const ROW: f64 = 66.0;
const TOP: f64 = 34.0;

/// Where a card sits in a drawing: its left, top, width and height. The
/// flow (B9) and the topology (F9) lay their cards out with it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Place {
    /// Its left edge.
    pub(crate) x: f64,
    /// Its top edge.
    pub(crate) y: f64,
    /// Its width.
    pub(crate) width: f64,
    /// Its height.
    pub(crate) height: f64,
}

/// A count as a drawing measure.
pub(crate) fn units(count: usize) -> f64 {
    u32::try_from(count).map_or(f64::from(u32::MAX), f64::from)
}

/// The summary of a lane's folded quiet cards.
#[derive(Debug)]
struct Folded<'a> {
    /// The locale its words are drawn in.
    locale: &'a Locale,
    /// Its id, which the folded cards' edges and paths name instead.
    id: String,
    /// The lane's title.
    title: &'a str,
    /// How many folded cards are in each state, in the order of the states.
    counts: Vec<(FlowState, usize)>,
    /// The page that unfolds the lane.
    href: &'a str,
}

impl Folded<'_> {
    /// How many cards it holds.
    fn count(&self) -> usize {
        self.counts.iter().map(|(_, count)| count).sum()
    }

    /// The state its dot shows: up while any card it holds is up.
    fn state(&self) -> FlowState {
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
    fn line(&self) -> String {
        self.parts()
            .into_iter()
            .reduce(|first, next| joined(self.locale, &first, &next))
            .unwrap_or_default()
    }

    /// Its line broken into the lines its card draws, each at most
    /// [`LINE_MOST`] characters, so no count is cut.
    fn lines(&self) -> Vec<String> {
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
    fn height(&self) -> f64 {
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
const NAME_MOST: usize = 23;
const LINE_MOST: usize = 22;

/// The room a card's line takes.
const LINE: f64 = 14.0;

/// One card as the drawing shows it: a card of the flow, or the summary of
/// its lane's folded cards.
#[derive(Debug)]
enum Shown<'a> {
    Card(&'a FlowCard),
    Fold(Folded<'a>),
}

impl Shown<'_> {
    /// The id its edges name it by.
    fn id(&self) -> &str {
        match self {
            Self::Card(card) => &card.id,
            Self::Fold(folded) => &folded.id,
        }
    }
}

/// One lane as the drawing shows it: its cards in their drawn order, the
/// summary last, and its fold link with whether it is folded.
#[derive(Debug)]
struct Planned<'a> {
    title: &'a str,
    shown: Vec<Shown<'a>>,
    toggle: Option<(bool, &'a str)>,
}

/// One edge as the drawing draws it, its ends the shown cards' ids.
#[derive(Debug)]
struct Drawn<'a> {
    from: String,
    to: String,
    state: FlowState,
    label: Option<&'a str>,
}

/// A flow as the drawing shows it: the lanes folded and ordered, the
/// edges between the shown cards, and the summary that stands for each
/// folded card.
#[derive(Debug)]
struct Plan<'a> {
    lanes: Vec<Planned<'a>>,
    edges: Vec<Drawn<'a>>,
    alias: BTreeMap<&'a str, String>,
}

impl Plan<'_> {
    /// The id of the shown card that stands for the card `id`.
    fn shown_id<'b>(&'b self, id: &'b str) -> &'b str {
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

/// How many pairs of edges cross between the lanes `lanes`, each lane its
/// ids top to bottom: two edges between the same two lanes cross when
/// their ends lie in opposite orders.
fn crossings(lanes: &[Vec<&str>], edges: &[(&str, &str)]) -> usize {
    let at: BTreeMap<&str, (usize, usize)> = lanes
        .iter()
        .enumerate()
        .flat_map(|(lane, ids)| {
            ids.iter()
                .enumerate()
                .map(move |(index, id)| (*id, (lane, index)))
        })
        .collect();
    let spans: Vec<(usize, usize, usize, usize)> = edges
        .iter()
        .filter_map(|(from, to)| {
            let (one, two) = (at.get(from)?, at.get(to)?);
            match one.0.cmp(&two.0) {
                std::cmp::Ordering::Less => Some((one.0, one.1, two.0, two.1)),
                std::cmp::Ordering::Greater => Some((two.0, two.1, one.0, one.1)),
                std::cmp::Ordering::Equal => None,
            }
        })
        .collect();
    let mut count = 0_usize;
    for (index, first) in spans.iter().enumerate() {
        for second in spans.iter().skip(index + 1) {
            let same_lanes = first.0 == second.0 && first.2 == second.2;
            let opposite = (first.1 < second.1 && first.3 > second.3)
                || (first.1 > second.1 && first.3 < second.3);
            if same_lanes && opposite {
                count += 1;
            }
        }
    }
    count
}

/// How many times the lanes are swept each way at most.
const SWEEPS: usize = 6;

/// Reorders the lane `lane` of `lanes` by the barycentre of each card's
/// neighbours in the lane `by`, its last card held while `pinned`; a card
/// with no neighbour there keeps its place, and the console's order
/// `first` breaks a tie.
fn sweep(
    lanes: &mut [Vec<&str>],
    pinned: &[bool],
    edges: &[(&str, &str)],
    first: &BTreeMap<&str, usize>,
    (lane, by): (usize, usize),
) {
    let spread = |ids: &[&str]| -> BTreeMap<String, f64> {
        let count = units(ids.len().max(1));
        ids.iter()
            .enumerate()
            .map(|(index, id)| ((*id).to_owned(), (units(index) + 0.5) / count))
            .collect()
    };
    let Some(reference) = lanes.get(by).map(|ids| spread(ids)) else {
        return;
    };
    let Some(ids) = lanes.get_mut(lane) else {
        return;
    };
    let current = spread(ids);
    let held = usize::from(pinned.get(lane).copied().unwrap_or(false)).min(ids.len());
    let movable = ids.len() - held;
    let mut keyed: Vec<(f64, usize, &str)> = ids
        .iter()
        .take(movable)
        .map(|id| {
            let (sum, count) = edges
                .iter()
                .filter_map(|(from, to)| {
                    if from == id {
                        reference.get(*to)
                    } else if to == id {
                        reference.get(*from)
                    } else {
                        None
                    }
                })
                .fold((0.0, 0_usize), |(sum, count), at| (sum + at, count + 1));
            let key = if count == 0 {
                current.get(*id).copied().unwrap_or_default()
            } else {
                sum / units(count)
            };
            (key, first.get(id).copied().unwrap_or_default(), *id)
        })
        .collect();
    keyed.sort_by(|one, two| one.0.total_cmp(&two.0).then(one.1.cmp(&two.1)));
    for (slot, (_, _, id)) in ids.iter_mut().zip(keyed) {
        *slot = id;
    }
}

/// The order of `lanes` that crosses the fewest of `edges`: barycentre
/// sweeps left to right and back from the console's order, which is kept
/// unless a sweep crosses strictly fewer, each lane's last card held while
/// `pinned`.
fn ordered<'a>(
    lanes: &[Vec<&'a str>],
    pinned: &[bool],
    edges: &[(&'a str, &'a str)],
) -> Vec<Vec<&'a str>> {
    let first: BTreeMap<&str, usize> = lanes
        .iter()
        .flat_map(|ids| ids.iter().enumerate().map(|(index, id)| (*id, index)))
        .collect();
    let mut best = lanes.to_vec();
    let mut fewest = crossings(&best, edges);
    let mut tried = best.clone();
    let count = lanes.len();
    for _ in 0..SWEEPS {
        if fewest == 0 || count < 2 {
            break;
        }
        let before = fewest;
        let rightward = (1..count).map(|lane| (lane, lane - 1));
        let leftward = (0..count - 1).rev().map(|lane| (lane, lane + 1));
        for pass in [rightward.collect::<Vec<_>>(), leftward.collect()] {
            for pair in pass {
                sweep(&mut tried, pinned, edges, &first, pair);
            }
            let now = crossings(&tried, edges);
            if now < fewest {
                fewest = now;
                best.clone_from(&tried);
            }
        }
        if fewest == before {
            break;
        }
    }
    transposed(&mut best, pinned, edges);
    best
}

/// How many pairs of edges of `upper` and `lower` cross, `upper` drawn
/// above `lower` in one lane, with every other card at its place in `at`.
fn pair_crossings(
    (upper, lower): (&str, &str),
    lane: usize,
    edges: &[(&str, &str)],
    at: &BTreeMap<&str, (usize, usize)>,
) -> usize {
    let others = |id: &str| -> Vec<(usize, usize)> {
        edges
            .iter()
            .filter_map(|(from, to)| {
                let other = if *from == id {
                    to
                } else if *to == id {
                    from
                } else {
                    return None;
                };
                at.get(other).copied().filter(|(each, _)| *each != lane)
            })
            .collect()
    };
    let (above, below) = (others(upper), others(lower));
    above
        .iter()
        .map(|(lane_a, a)| {
            below
                .iter()
                .filter(|(lane_b, b)| lane_a == lane_b && a > b)
                .count()
        })
        .sum()
}

/// Swaps neighbouring cards of each lane of `lanes` while a swap crosses
/// fewer of `edges`, each lane's last card held while `pinned`.
fn transposed(lanes: &mut [Vec<&str>], pinned: &[bool], edges: &[(&str, &str)]) {
    for _ in 0..SWEEPS {
        let mut swapped = false;
        for lane in 0..lanes.len() {
            let held = usize::from(pinned.get(lane).copied().unwrap_or(false));
            let movable = lanes
                .get(lane)
                .map_or(0, |ids| ids.len().saturating_sub(held));
            for index in 1..movable {
                let at: BTreeMap<&str, (usize, usize)> = lanes
                    .iter()
                    .enumerate()
                    .flat_map(|(each, ids)| {
                        ids.iter()
                            .enumerate()
                            .map(move |(place, id)| (*id, (each, place)))
                    })
                    .collect();
                let Some(ids) = lanes.get_mut(lane) else {
                    continue;
                };
                let (Some(upper), Some(lower)) =
                    (ids.get(index - 1).copied(), ids.get(index).copied())
                else {
                    continue;
                };
                let now = pair_crossings((upper, lower), lane, edges, &at);
                let then = pair_crossings((lower, upper), lane, edges, &at);
                if then < now {
                    ids.swap(index - 1, index);
                    swapped = true;
                }
            }
        }
        if !swapped {
            break;
        }
    }
}

/// `flow` as the drawing shows it: each lane folded as its fold says,
/// then ordered to cross the fewest edges.
fn plan<'a>(locale: &'a Locale, flow: &'a Flow) -> Plan<'a> {
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

/// The place of every shown card by its id, and the drawing's width and
/// height.
fn layout<'p>(plan: &'p Plan<'_>) -> (BTreeMap<&'p str, Place>, f64, f64) {
    let rows = plan
        .lanes
        .iter()
        .map(|lane| lane.shown.len())
        .max()
        .unwrap_or(1)
        .max(1);
    let mut places = BTreeMap::new();
    for (column, lane) in plan.lanes.iter().enumerate() {
        let x = units(column) * (LANE + GAP);
        let slots = units(rows) / units(lane.shown.len().max(1));
        for (index, shown) in lane.shown.iter().enumerate() {
            let middle = TOP + ((units(index) + 0.5) * slots - 0.5) * ROW + CARD / 2.0;
            let (y, height) = match shown {
                Shown::Card(card) if card.hub => (middle - HUB / 2.0, HUB),
                Shown::Card(_) => (middle - CARD / 2.0, CARD),
                Shown::Fold(folded) => (middle - CARD / 2.0, folded.height()),
            };
            places.insert(
                shown.id(),
                Place {
                    x,
                    y,
                    width: LANE,
                    height,
                },
            );
        }
    }
    let lanes = units(plan.lanes.len().max(1));
    let width = lanes * LANE + (lanes - 1.0) * GAP;
    let height = places
        .values()
        .map(|place| place.y + place.height + ROW - CARD)
        .fold(TOP + units(rows) * ROW, f64::max);
    (places, width, height)
}

/// The two ends of an edge from `from` to `to`: it leaves the side of
/// `from` that faces `to` and enters the facing side of `to`, so an edge
/// runs left to right or, on a mirrored side, right to left.
pub(crate) fn ends(from: Place, to: Place) -> ((f64, f64), (f64, f64)) {
    if to.x >= from.x {
        (
            (from.x + from.width, from.y + from.height / 2.0),
            (to.x, to.y + to.height / 2.0),
        )
    } else {
        (
            (from.x, from.y + from.height / 2.0),
            (to.x + to.width, to.y + to.height / 2.0),
        )
    }
}

/// The Bézier curve between two ends, with horizontal tangents.
pub(crate) fn curve(((x1, y1), (x2, y2)): ((f64, f64), (f64, f64))) -> String {
    let pull = (x2 - x1) * 0.48;
    format!(
        "M{x1:.1} {y1:.1} C{:.1} {y1:.1}, {:.1} {y2:.1}, {x2:.1} {y2:.1}",
        x1 + pull,
        x2 - pull
    )
}

/// `text` cut to `most` characters with an ellipsis.
pub(crate) fn fit(text: &str, most: usize) -> String {
    if text.chars().count() <= most {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(most.saturating_sub(1)).collect();
    cut.push('\u{2026}');
    cut
}

/// Where a problem edge draws its label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LabelAt {
    /// Beside the end mark, at the child end.
    Mark,
    /// After the start, where several labelled edges meet one card.
    Start,
}

/// The drawing of an edge from `from` to `to` in `state`: its line, and on
/// a problem edge the end mark at the child end while `marked`, and the
/// label where `label_at` says.
pub(crate) fn edge_line(
    from: Place,
    to: Place,
    state: FlowState,
    (label, label_at): (Option<&str>, LabelAt),
    on_path: bool,
    marked: bool,
) -> Html {
    let line = ends(from, to);
    let ((x1, y1), (x2, y2)) = line;
    let toward = if x2 >= x1 { 1.0 } else { -1.0 };
    let at = format!("translate({:.1} {y2:.1})", x2 - toward * 11.0);
    let mark = match (marked, state) {
        (true, FlowState::Unreachable) => format!(
            "<g transform=\"{at}\" class=\"fui-edge-mark fui-edge-mark-bad\"><circle r=\"7\"></circle><path d=\"M-3 -3 L3 3 M3 -3 L-3 3\"></path></g>"
        ),
        (true, FlowState::Degraded) => format!(
            "<g transform=\"{at}\" class=\"fui-edge-mark fui-edge-mark-warn\"><circle r=\"7\"></circle><path d=\"M0 -3.5 V1 M0 3.4 V3.5\"></path></g>"
        ),
        _ => String::new(),
    };
    let label = label
        .filter(|_| state.is_problem())
        .map_or_else(String::new, |label| {
            let width = 8.0 + units(label.chars().count()) * 5.7;
            let (left, y) = match (label_at, toward > 0.0) {
                (LabelAt::Mark, true) => (x2 - 22.0 - width, y2),
                (LabelAt::Mark, false) => (x2 + 22.0, y2),
                (LabelAt::Start, true) => (x1 + 12.0, y1),
                (LabelAt::Start, false) => (x1 - 12.0 - width, y1),
            };
            format!(
                "<g transform=\"translate({left:.1} {:.1})\" class=\"fui-edge-label\"><rect width=\"{width:.1}\" height=\"18\" rx=\"9\"></rect><text x=\"{:.1}\" y=\"12.5\" text-anchor=\"middle\">{}</text></g>",
                y - 9.0,
                width / 2.0,
                Html::text(label)
            )
        });
    let class = if on_path {
        format!("{} is-path", state.edge())
    } else {
        state.edge().to_owned()
    };
    Html::raw(format!(
        "<g><path d=\"{}\" class=\"{class}\"></path>{mark}{label}</g>",
        curve(line)
    ))
}

/// The drawing of `edge` between `places`: its line, and on a problem edge
/// its end mark and its label where `crowded` says several labels meet.
/// The drawing draws every problem edge last, so no line covers a label.
fn edge_view(
    edge: &Drawn<'_>,
    places: &BTreeMap<&str, Place>,
    crowded: &BTreeSet<&str>,
    on_path: bool,
) -> Option<Html> {
    let from = places.get(edge.from.as_str())?;
    let to = places.get(edge.to.as_str())?;
    let label_at = if crowded.contains(edge.to.as_str()) {
        LabelAt::Start
    } else {
        LabelAt::Mark
    };
    Some(edge_line(
        *from,
        *to,
        edge.state,
        (edge.label, label_at),
        on_path,
        true,
    ))
}

/// The cards that more than one labelled problem edge enters.
fn crowded<'p>(plan: &'p Plan<'_>) -> BTreeSet<&'p str> {
    let mut entered: BTreeMap<&str, usize> = BTreeMap::new();
    for edge in &plan.edges {
        if edge.state.is_problem() && edge.label.is_some() {
            *entered.entry(edge.to.as_str()).or_default() += 1;
        }
    }
    entered
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(id, _)| id)
        .collect()
}

/// The shape of a card's dot in `state`, centred on `(x, y)`.
pub(crate) fn dot(state: FlowState, x: f64, y: f64) -> Html {
    Html::raw(match state {
        FlowState::Up => {
            format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"4\" class=\"fui-tdot fui-tone-ok\"></circle>"
            )
        }
        FlowState::Degraded => format!(
            "<rect x=\"{}\" y=\"{}\" width=\"7\" height=\"7\" transform=\"rotate(45 {x} {y})\" class=\"fui-tdot fui-tone-warn\"></rect>",
            x - 3.5,
            y - 3.5
        ),
        FlowState::Unreachable => format!(
            "<path d=\"M{:.1} {:.1} L{:.1} {:.1} M{:.1} {:.1} L{:.1} {:.1}\" class=\"fui-tdot fui-tdot-cross fui-tone-bad\"></path>",
            x - 3.5,
            y - 3.5,
            x + 3.5,
            y + 3.5,
            x + 3.5,
            y - 3.5,
            x - 3.5,
            y + 3.5
        ),
        FlowState::NotChecked | FlowState::Off => {
            format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"3.5\" class=\"fui-tdot fui-tdot-ring\"></circle>"
            )
        }
    })
}

/// The accessible name of `card`.
fn card_name(locale: &Locale, card: &FlowCard) -> String {
    String::from(if card.hub {
        tr_with(
            locale,
            Text::FlowHub,
            &[
                ("title", card.name.clone().into()),
                ("note", card.line.clone().into()),
            ],
        )
    } else {
        tr_with(
            locale,
            Text::FlowCard,
            &[
                ("title", card.name.clone().into()),
                ("note", card.line.clone().into()),
                ("status", card.state.word(locale).into()),
            ],
        )
    })
}

/// One card as a drawing draws it.
#[derive(Debug)]
pub(crate) struct CardDrawing {
    /// The card's kind class, such as `fui-tnode-node` or `fui-tnode-gateway`.
    pub(crate) kind: &'static str,
    /// Where it sits.
    pub(crate) place: Place,
    /// The glyph before its name.
    pub(crate) glyph: Glyph,
    /// Its name.
    pub(crate) name: String,
    /// Its line under the name.
    pub(crate) line: String,
    /// Its state.
    pub(crate) state: FlowState,
    /// The page it opens.
    pub(crate) href: String,
    /// Its accessible name, also its tooltip.
    pub(crate) label: String,
    /// Whether it is a hub: the accent card with no dot.
    pub(crate) hub: bool,
    /// Whether its name is an identifier, drawn in the code face.
    pub(crate) mono: bool,
    /// Whether it is the card whose quick look is open.
    pub(crate) selected: bool,
    /// The most characters of its name and of its line it shows before an
    /// ellipsis.
    pub(crate) most: (usize, usize),
    /// The lines it draws under its line, each in the line's room; the
    /// card's height leaves room for them.
    pub(crate) more: Vec<String>,
    /// Its controls beside the link, such as its quick look and its
    /// collapse button, drawn in the card's own units.
    pub(crate) controls: Option<Html>,
}

/// The drawing of a card: its link with the tone bar, the glyph, the name,
/// the dot and the line, and beside the link its controls.
pub(crate) fn card_drawing(card: CardDrawing) -> Html {
    let CardDrawing {
        kind,
        place,
        glyph,
        name,
        line,
        state,
        href,
        label,
        hub,
        mono,
        selected,
        most: (name_most, line_most),
        more,
        controls,
    } = card;
    let height = place.height;
    let base = height - LINE * units(more.len());
    let mut class = format!("fui-tnode {kind} {}", state.tone().class());
    if state == FlowState::Off {
        class.push_str(" is-muted");
    }
    if selected {
        class.push_str(" is-selected");
    }
    let shown_dot = if hub {
        Html::empty()
    } else {
        dot(state, 47.0, base / 2.0 + 10.0)
    };
    let line_x = if hub { 42.0 } else { 56.0 };
    let more: Html = more
        .into_iter()
        .enumerate()
        .map(|(index, text)| {
            let y = base / 2.0 + 14.0 + LINE * (units(index) + 1.0);
            Html::raw(format!(
                "<text x=\"{line_x}\" y=\"{y}\" class=\"fui-tnode-line\">{}</text>",
                Html::text(fit(&text, line_most))
            ))
        })
        .collect();
    let name_class = if mono {
        "fui-tnode-name fui-tnode-mono"
    } else {
        "fui-tnode-name"
    };
    let label = Html::text(label);
    Html::raw(format!(
        "<g transform=\"translate({:.1} {:.1})\" class=\"{class}\"><a href=\"{}\" aria-label=\"{label}\" class=\"fui-tnode-link\"><title>{label}</title><rect width=\"{}\" height=\"{height}\" rx=\"8\" class=\"fui-tnode-box\"></rect><rect width=\"4\" height=\"{}\" x=\"0\" y=\"8\" rx=\"2\" class=\"fui-tnode-bar\"></rect><g transform=\"translate(14 {:.1})\" class=\"fui-tnode-glyph\">{}</g><text x=\"42\" y=\"{}\" class=\"{name_class}\">{}</text>{shown_dot}<text x=\"{line_x}\" y=\"{}\" class=\"fui-tnode-line\">{}</text>{more}</a>{}</g>",
        place.x,
        place.y,
        Html::text(href),
        place.width,
        height - 16.0,
        base / 2.0 - 9.0,
        icon(glyph, "18"),
        base / 2.0 - 4.0,
        Html::text(fit(&name, name_most)),
        base / 2.0 + 14.0,
        Html::text(fit(&line, line_most)),
        controls.unwrap_or_default()
    ))
}

/// The path a hovered or focused card shows: `lines` raised, the glow and
/// the pulse along `route`, hidden from assistive technology.
pub(crate) fn path_drawing(lines: Html, route: &str) -> Html {
    let lines = String::from(lines);
    Html::raw(format!(
        "<g aria-hidden=\"true\" class=\"fui-pipeline-path\">{lines}<path d=\"{route}\" class=\"fui-path-glow\"></path><path d=\"{route}\" pathLength=\"100\" class=\"fui-path-pulse\"></path></g>"
    ))
}

/// What every card of one drawing reads: the plan, the places and the
/// crowded cards.
#[derive(Debug, Clone, Copy)]
struct Scene<'s, 'a> {
    locale: &'s Locale,
    plan: &'s Plan<'a>,
    places: &'s BTreeMap<&'s str, Place>,
    crowded: &'s BTreeSet<&'s str>,
}

/// The drawing of `card` at `place`, a link to its page, after its path
/// with the pulse, which shows while the card is hovered or focused; a
/// folded card on the path is its summary.
fn card_view(card: &FlowCard, place: Place, scene: Scene<'_, '_>) -> Html {
    let Scene {
        locale,
        plan,
        places,
        crowded,
    } = scene;
    let steps: Vec<&Drawn<'_>> = card
        .path
        .windows(2)
        .filter_map(|pair| match pair {
            [from, to] => {
                let (from, to) = (plan.shown_id(from), plan.shown_id(to));
                plan.edges
                    .iter()
                    .find(|edge| edge.from == from && edge.to == to)
            }
            _ => None,
        })
        .collect();
    let path = if steps.is_empty() {
        Html::empty()
    } else {
        let lines: Html = steps
            .iter()
            .filter_map(|edge| edge_view(edge, places, crowded, true))
            .collect();
        let route = steps
            .iter()
            .filter_map(|edge| {
                let from = places.get(edge.from.as_str())?;
                let to = places.get(edge.to.as_str())?;
                Some(curve(ends(*from, *to)))
            })
            .collect::<Vec<_>>()
            .join(" ");
        path_drawing(lines, &route)
    };
    let drawing = card_drawing(CardDrawing {
        kind: if card.hub {
            "fui-tnode-gateway"
        } else {
            "fui-tnode-node"
        },
        place,
        glyph: card.glyph,
        name: card.name.clone(),
        line: card.line.clone(),
        state: card.state,
        href: card.href.clone(),
        label: card_name(locale, card),
        hub: card.hub,
        mono: false,
        selected: false,
        most: (NAME_MOST, LINE_MOST),
        more: Vec::new(),
        controls: None,
    });
    Html::raw(format!(
        "<g class=\"fui-pipeline-card\">{path}{drawing}</g>"
    ))
}

/// The summary card of a folded lane at `place`, a link that unfolds it.
fn fold_view(folded: &Folded<'_>, place: Place) -> Html {
    let mut lines = folded.lines().into_iter();
    let first = lines.next().unwrap_or_default();
    let more = lines.collect();
    let drawing = card_drawing(CardDrawing {
        kind: "fui-tnode-group",
        place,
        glyph: Glyph::ELLIPSIS,
        name: tr_with(
            folded.locale,
            Text::FlowFoldName,
            &[("count", folded.count().into())],
        )
        .into(),
        line: first,
        state: folded.state(),
        href: folded.href.to_owned(),
        label: fold_name(folded),
        hub: false,
        mono: false,
        selected: false,
        most: (NAME_MOST, LINE_MOST),
        more,
        controls: None,
    });
    Html::raw(format!("<g class=\"fui-pipeline-card\">{drawing}</g>"))
}

/// The accessible name of a summary card, also its tooltip.
fn fold_name(folded: &Folded<'_>) -> String {
    tr_with(
        folded.locale,
        Text::FlowFoldCard,
        &[
            ("count", folded.count().into()),
            ("title", folded.title.into()),
            ("note", folded.line().into()),
        ],
    )
    .into()
}

/// The words of a lane's fold link: unfold while `folded`, else fold.
fn toggle_words(locale: &Locale, folded: bool, title: &str) -> String {
    let text = if folded {
        Text::FlowUnfold
    } else {
        Text::FlowFold
    };
    tr_with(locale, text, &[("title", title.into())]).into()
}

/// The fold link beside the title of the lane at `x`.
fn lane_toggle(locale: &Locale, x: f64, title: &str, (folded, href): (bool, &str)) -> Html {
    let name = Html::text(toggle_words(locale, folded, title));
    let glyph = if folded {
        Glyph::CHEVRON_DOWN
    } else {
        Glyph::CHEVRON_UP
    };
    Html::raw(format!(
        "<g transform=\"translate({:.1} -1)\" class=\"fui-tnode-toggle\"><a href=\"{}\" aria-label=\"{name}\"><title>{name}</title><rect width=\"16\" height=\"16\" fill=\"transparent\"></rect>{}</a></g>",
        x + LANE - 16.0,
        Html::text(href),
        icon(glyph, "16")
    ))
}

/// The description of the drawing: each problem edge with its state and
/// label, or that there is none.
fn problems(locale: &Locale, flow: &Flow) -> String {
    let named = |id: &str| {
        flow.lanes
            .iter()
            .flat_map(|lane| lane.cards.iter())
            .find(|card| card.id == id)
            .map_or_else(|| id.to_owned(), |card| card.name.clone())
    };
    let sentences: Vec<String> = flow
        .edges
        .iter()
        .filter(|edge| edge.state.is_problem())
        .map(|edge| {
            let mut args = vec![
                ("from", named(&edge.from).into()),
                ("to", named(&edge.to).into()),
                ("status", edge.state.word(locale).into()),
            ];
            match &edge.label {
                Some(label) => {
                    args.push(("note", label.clone().into()));
                    tr_with(locale, Text::FlowEdgeNoted, &args)
                }
                None => tr_with(locale, Text::FlowEdge, &args),
            }
            .into()
        })
        .collect();
    if sentences.is_empty() {
        tr(locale, Text::FlowSteady).into()
    } else {
        format!("{}.", sentences.join(". "))
    }
}

/// One row of the phone's list: a link with its state as a dot and a
/// word, its name and its line.
fn list_row(locale: &Locale, href: &str, state: FlowState, name: &Word, line: &str) -> Html {
    let muted = if state == FlowState::Off {
        " is-muted"
    } else {
        ""
    };
    let word = Html::text(state.word(locale).text());
    Html::raw(format!(
        "<li><a href=\"{}\" class=\"fui-row-link\"><span role=\"img\" aria-label=\"{word}\" title=\"{word}\" class=\"fui-dot {}{muted}\"></span><span class=\"fui-pipeline-name\">{name}</span><span class=\"fui-pipeline-line\">{}</span></a></li>",
        Html::text(href),
        state.tone().class(),
        Html::text(line)
    ))
}

/// The phone's form of `plan`: a heading per lane and a row per card, each
/// a link to its page with its state as a dot and a word, folded as the
/// drawing is, and an open lane's fold link last.
fn list(locale: &Locale, plan: &Plan<'_>, label: &str) -> Html {
    let lanes: Html = plan
        .lanes
        .iter()
        .map(|lane| {
            let rows: Html = lane
                .shown
                .iter()
                .map(|shown| match shown {
                    Shown::Card(card) => list_row(
                        locale,
                        &card.href,
                        card.state,
                        &Word::new(card.name.clone()),
                        &card.line,
                    ),
                    Shown::Fold(folded) => list_row(
                        locale,
                        folded.href,
                        folded.state(),
                        &tr_with(
                            locale,
                            Text::FlowFoldName,
                            &[("count", folded.count().into())],
                        ),
                        &folded.line(),
                    ),
                })
                .collect();
            let fold = lane
                .toggle
                .filter(|(folded, _)| !folded)
                .map_or_else(Html::empty, |(_, href)| {
                    Html::raw(format!(
                        "<li><a href=\"{}\" class=\"fui-row-link\">{}<span class=\"fui-pipeline-name\">{}</span></a></li>",
                        Html::text(href),
                        icon(Glyph::CHEVRON_UP, "16"),
                        Html::text(toggle_words(locale, false, lane.title))
                    ))
                });
            Html::raw(format!(
                "<li><p class=\"fui-pipeline-lane\">{}</p><ul>{rows} {fold}</ul></li>",
                Html::text(lane.title)
            ))
        })
        .collect();
    Html::raw(format!(
        "<ul aria-label=\"{}\" class=\"fui-pipeline-list\">{lanes}</ul>",
        Html::text(label)
    ))
}

/// Returns `flow`: the drawing, which a phone replaces with its list, and
/// the quiet foot line.
///
/// Put it in a panel ([`crate::view::surface::titled_panel`]), whose head
/// holds the title and the actions. Each lane is ordered to cross the
/// fewest edges, with the console's order breaking a tie; cards in a lane
/// spread over the height of the fullest lane; a hub card is taller and
/// carries no dot. A lane past [`FOLD_PAST`] cards with a fold folds its
/// quiet cards into one summary card, a link that unfolds it, and keeps
/// every card in a problem state or on a problem edge.
#[must_use]
pub fn flow(locale: &Locale, flow: &Flow) -> Html {
    let plan = plan(locale, flow);
    let (places, width, height) = layout(&plan);
    let crowded = crowded(&plan);
    let scene = Scene {
        locale,
        plan: &plan,
        places: &places,
        crowded: &crowded,
    };
    let view_box = format!("-8 -4 {:.0} {height:.0}", width + 16.0);
    let lanes: Html = plan
        .lanes
        .iter()
        .enumerate()
        .map(|(column, lane)| {
            let x = units(column) * (LANE + GAP);
            Html::raw(format!(
                "<text x=\"{x}\" y=\"12\" class=\"fui-topology-lane\">{}</text>",
                Html::text(lane.title)
            ))
        })
        .collect();
    let toggles: Html = plan
        .lanes
        .iter()
        .enumerate()
        .filter_map(|(column, lane)| {
            let toggle = lane.toggle?;
            Some(lane_toggle(
                locale,
                units(column) * (LANE + GAP),
                lane.title,
                toggle,
            ))
        })
        .collect();
    let edges: Html = plan
        .edges
        .iter()
        .filter(|edge| !edge.state.is_problem())
        .chain(plan.edges.iter().filter(|edge| edge.state.is_problem()))
        .filter_map(|edge| edge_view(edge, &places, &crowded, false))
        .collect();
    let cards: Html = plan
        .lanes
        .iter()
        .flat_map(|lane| lane.shown.iter())
        .filter_map(|shown| {
            let place = *places.get(shown.id())?;
            Some(match shown {
                Shown::Card(card) => card_view(card, place, scene),
                Shown::Fold(folded) => fold_view(folded, place),
            })
        })
        .collect();
    let id = Html::text(&flow.id);
    Html::raw(format!(
        "<div class=\"fui-pipeline\"><div class=\"fui-topology-canvas\"><div class=\"fui-pipeline-stage\"><svg viewBox=\"{view_box}\" role=\"img\" aria-labelledby=\"{id}-title {id}-desc\" class=\"fui-topology-svg fui-pipeline-edges\"><title id=\"{id}-title\">{}</title><desc id=\"{id}-desc\">{}</desc><g>{lanes}</g><g>{edges}</g></svg><svg viewBox=\"{view_box}\" role=\"group\" aria-label=\"{}\" class=\"fui-topology-svg fui-pipeline-cards\">{toggles}{cards}</svg></div></div>{}<p class=\"fui-pipeline-foot\">{}</p></div>",
        Html::text(&flow.label),
        Html::text(problems(locale, flow)),
        Html::text(&flow.list_label),
        list(locale, &plan, &flow.list_label),
        Html::text(&flow.foot)
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        FOLD_PAST, Flow, FlowCard, FlowEdge, FlowLane, FlowState, LaneFold, Shown, crossings, plan,
    };
    use crate::i18n::Locale;
    use crate::icon::Glyph;

    fn card(id: String, state: FlowState) -> FlowCard {
        FlowCard {
            name: id.clone(),
            line: String::new(),
            href: format!("/{id}"),
            id,
            glyph: Glyph::SERVER,
            state,
            path: Vec::new(),
            hub: false,
        }
    }

    fn edge(from: &str, to: &str, state: FlowState) -> FlowEdge {
        FlowEdge {
            from: from.to_owned(),
            to: to.to_owned(),
            state,
            label: Some(String::from("503")),
        }
    }

    /// A fixed busy bridge: 30 callers, each calling one or two of six
    /// faces, and 30 upstreams, each reached from one face, wired by a
    /// fixed sequence so the console's order crosses freely.
    fn busy(fold: Option<LaneFold>) -> Flow {
        let mut seed = 7_u32;
        let mut next = |bound: usize| {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345) % 2_147_483_648;
            usize::try_from(seed >> 8).unwrap_or_default() % bound
        };
        let lane = |title: &str, prefix: &str, count: usize, fold: Option<LaneFold>| FlowLane {
            title: title.to_owned(),
            cards: (0..count)
                .map(|index| {
                    let state = match index {
                        4 => FlowState::Degraded,
                        11 => FlowState::Unreachable,
                        17 => FlowState::Off,
                        _ => FlowState::Up,
                    };
                    card(format!("{prefix}{index:02}"), state)
                })
                .collect(),
            fold,
        };
        let mut edges = Vec::new();
        for caller in 0..30 {
            for _ in 0..=next(2) {
                let face = next(6);
                edges.push(edge(
                    &format!("c{caller:02}"),
                    &format!("f{face:02}"),
                    FlowState::Up,
                ));
            }
        }
        for upstream in 0..30 {
            let face = next(6);
            let state = if upstream == 23 {
                FlowState::Degraded
            } else {
                FlowState::Up
            };
            edges.push(edge(
                &format!("f{face:02}"),
                &format!("u{upstream:02}"),
                state,
            ));
        }
        Flow {
            id: String::from("busy"),
            label: String::new(),
            list_label: String::new(),
            lanes: vec![
                lane("Callers", "c", 30, fold.clone()),
                lane("Faces", "f", 6, None),
                lane("Upstreams", "u", 30, fold),
            ],
            edges,
            foot: String::new(),
        }
    }

    /// How many edges of `flow` cross with its lanes in `order`.
    fn crossed(flow: &Flow, order: &[Vec<&str>]) -> usize {
        let pairs: Vec<(&str, &str)> = flow
            .edges
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        crossings(order, &pairs)
    }

    #[test]
    fn the_lane_order_crosses_far_fewer_edges_than_the_consoles_and_is_deterministic() {
        let flow = busy(None);
        let console: Vec<Vec<&str>> = flow
            .lanes
            .iter()
            .map(|lane| lane.cards.iter().map(|card| card.id.as_str()).collect())
            .collect();
        let en = Locale::english();
        let planned = plan(&en, &flow);
        let drawn: Vec<Vec<&str>> = planned
            .lanes
            .iter()
            .map(|lane| lane.shown.iter().map(Shown::id).collect())
            .collect();
        let (before, after) = (crossed(&flow, &console), crossed(&flow, &drawn));
        assert_eq!((657, 201), (before, after), "{drawn:?}");
        let again = plan(&en, &flow);
        let redrawn: Vec<Vec<&str>> = again
            .lanes
            .iter()
            .map(|lane| lane.shown.iter().map(Shown::id).collect())
            .collect();
        assert_eq!(drawn, redrawn, "the same flow draws the same order");
    }

    #[test]
    fn a_flow_that_crosses_nothing_keeps_the_consoles_order() {
        let mut flow = busy(None);
        flow.edges = (0..6)
            .map(|index| {
                edge(
                    &format!("c{index:02}"),
                    &format!("f{index:02}"),
                    FlowState::Up,
                )
            })
            .collect();
        let en = Locale::english();
        let planned = plan(&en, &flow);
        for (lane, shown) in flow.lanes.iter().zip(&planned.lanes) {
            let ids: Vec<&str> = shown.lanes_ids();
            let console: Vec<&str> = lane.cards.iter().map(|card| card.id.as_str()).collect();
            assert_eq!(console, ids);
        }
    }

    #[test]
    fn a_folded_lane_keeps_every_problem_card_and_holds_the_rest_in_one_summary_last() {
        let flow = busy(Some(LaneFold {
            open: false,
            toggle: String::from("?open=all"),
        }));
        let en = Locale::english();
        let planned = plan(&en, &flow);
        let callers = planned.lanes.first().expect("the callers lane");
        let ids = callers.lanes_ids();
        assert!(ids.contains(&"c04") && ids.contains(&"c11"), "{ids:?}");
        assert!(!ids.contains(&"c17"), "an off card is quiet: {ids:?}");
        assert_eq!(Some(&"busy-fold-0"), ids.last(), "{ids:?}");
        assert_eq!(Some((true, "?open=all")), callers.toggle);
        let upstreams = planned.lanes.get(2).expect("the upstreams lane");
        assert!(
            upstreams.lanes_ids().contains(&"u23"),
            "a card on a problem edge stays"
        );
        let faces = planned.lanes.get(1).expect("the faces lane");
        assert_eq!(
            None, faces.toggle,
            "a lane of {FOLD_PAST} or fewer never folds"
        );
        let into_summary = planned
            .edges
            .iter()
            .filter(|edge| edge.from == "busy-fold-0")
            .count();
        assert!(
            into_summary <= 6,
            "one edge per face at most: {into_summary}"
        );
        let unfolded = busy(Some(LaneFold {
            open: true,
            toggle: String::from("?"),
        }));
        let open = plan(&en, &unfolded);
        let callers = open.lanes.first().expect("the callers lane");
        assert_eq!(30, callers.shown.len());
        assert_eq!(Some((false, "?")), callers.toggle);
    }

    impl super::Planned<'_> {
        fn lanes_ids(&self) -> Vec<&str> {
            self.shown.iter().map(Shown::id).collect()
        }
    }
}
