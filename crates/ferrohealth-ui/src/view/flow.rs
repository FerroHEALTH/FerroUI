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
//! pulse; the stylesheet does it, so it needs no WebAssembly, changes paint
//! only (family rule 3) and stills under reduced motion. No specification
//! governs the drawing: our own design.

use std::collections::BTreeMap;

use leptos::prelude::*;

use crate::i18n::{Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;

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
    pub fn word(self) -> String {
        tr(match self {
            Self::Up => Text::FlowUp,
            Self::Degraded => Text::FlowDegraded,
            Self::Unreachable => Text::FlowUnreachable,
            Self::NotChecked => Text::FlowNotChecked,
            Self::Off => Text::FlowOff,
        })
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
    /// Its cards, top to bottom.
    pub cards: Vec<FlowCard>,
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

/// The place of every card by its id, and the drawing's width and height.
fn layout(flow: &Flow) -> (BTreeMap<&str, Place>, f64, f64) {
    let rows = flow
        .lanes
        .iter()
        .map(|lane| lane.cards.len())
        .max()
        .unwrap_or(1)
        .max(1);
    let mut places = BTreeMap::new();
    for (column, lane) in flow.lanes.iter().enumerate() {
        let x = units(column) * (LANE + GAP);
        let slots = units(rows) / units(lane.cards.len().max(1));
        for (index, card) in lane.cards.iter().enumerate() {
            let height = if card.hub { HUB } else { CARD };
            let middle = TOP + ((units(index) + 0.5) * slots - 0.5) * ROW + CARD / 2.0;
            places.insert(
                card.id.as_str(),
                Place {
                    x,
                    y: middle - height / 2.0,
                    width: LANE,
                    height,
                },
            );
        }
    }
    let lanes = units(flow.lanes.len().max(1));
    let width = lanes * LANE + (lanes - 1.0) * GAP;
    (places, width, TOP + units(rows) * ROW)
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

/// The drawing of an edge from `from` to `to` in `state`: its line, and on
/// a problem edge the end mark at the child end while `marked`, and the
/// label beside the mark.
pub(crate) fn edge_line(
    from: Place,
    to: Place,
    state: FlowState,
    label: Option<&str>,
    on_path: bool,
    marked: bool,
) -> AnyView {
    let line = ends(from, to);
    let ((x1, _), (x2, y2)) = line;
    let toward = if x2 >= x1 { 1.0 } else { -1.0 };
    let at = format!("translate({:.1} {y2:.1})", x2 - toward * 11.0);
    let mark = match (marked, state) {
        (true, FlowState::Unreachable) => Some(
            view! {
                <g class="fui-edge-mark fui-edge-mark-bad" transform=at>
                    <circle r="7"></circle>
                    <path d="M-3 -3 L3 3 M3 -3 L-3 3"></path>
                </g>
            }
            .into_any(),
        ),
        (true, FlowState::Degraded) => Some(
            view! {
                <g class="fui-edge-mark fui-edge-mark-warn" transform=at>
                    <circle r="7"></circle>
                    <path d="M0 -3.5 V1 M0 3.4 V3.5"></path>
                </g>
            }
            .into_any(),
        ),
        _ => None,
    };
    let label = label.filter(|_| state.is_problem()).map(|label| {
        let width = 8.0 + units(label.chars().count()) * 5.7;
        let left = if toward > 0.0 {
            x2 - 22.0 - width
        } else {
            x2 + 22.0
        };
        let place = format!("translate({left:.1} {:.1})", y2 - 9.0);
        let wide = format!("{width:.1}");
        let middle = format!("{:.1}", width / 2.0);
        let label = label.to_owned();
        view! {
            <g class="fui-edge-label" transform=place>
                <rect width=wide height="18" rx="9"></rect>
                <text x=middle y="12.5" text-anchor="middle">
                    {label}
                </text>
            </g>
        }
    });
    view! {
        <g>
            <path class=state.edge() class:is-path=on_path d=curve(line)></path>
            {mark}
            {label}
        </g>
    }
    .into_any()
}

/// The drawing of `edge` between `places`: its line, and on a problem edge
/// its end mark and its label.
fn edge_view(edge: &FlowEdge, places: &BTreeMap<&str, Place>, on_path: bool) -> Option<AnyView> {
    let from = places.get(edge.from.as_str())?;
    let to = places.get(edge.to.as_str())?;
    Some(edge_line(
        *from,
        *to,
        edge.state,
        edge.label.as_deref(),
        on_path,
        true,
    ))
}

/// The shape of a card's dot in `state`, centred on `(x, y)`.
pub(crate) fn dot(state: FlowState, x: f64, y: f64) -> AnyView {
    match state {
        FlowState::Up => {
            view! { <circle class="fui-tdot fui-tone-ok" cx=x cy=y r="4"></circle> }.into_any()
        }
        FlowState::Degraded => {
            let turn = format!("rotate(45 {x} {y})");
            view! {
                <rect
                    class="fui-tdot fui-tone-warn"
                    x=x - 3.5
                    y=y - 3.5
                    width="7"
                    height="7"
                    transform=turn
                ></rect>
            }
            .into_any()
        }
        FlowState::Unreachable => {
            let cross = format!(
                "M{:.1} {:.1} L{:.1} {:.1} M{:.1} {:.1} L{:.1} {:.1}",
                x - 3.5,
                y - 3.5,
                x + 3.5,
                y + 3.5,
                x + 3.5,
                y - 3.5,
                x - 3.5,
                y + 3.5
            );
            view! { <path class="fui-tdot fui-tdot-cross fui-tone-bad" d=cross></path> }.into_any()
        }
        FlowState::NotChecked | FlowState::Off => {
            view! { <circle class="fui-tdot fui-tdot-ring" cx=x cy=y r="3.5"></circle> }.into_any()
        }
    }
}

/// The accessible name of `card`.
fn card_name(card: &FlowCard) -> String {
    if card.hub {
        tr_with(
            Text::FlowHub,
            &[
                ("title", card.name.clone().into()),
                ("note", card.line.clone().into()),
            ],
        )
    } else {
        tr_with(
            Text::FlowCard,
            &[
                ("title", card.name.clone().into()),
                ("note", card.line.clone().into()),
                ("status", card.state.word().into()),
            ],
        )
    }
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
    /// Its controls beside the link, such as its quick look and its
    /// collapse button, drawn in the card's own units.
    pub(crate) controls: Option<AnyView>,
}

/// The drawing of a card: its link with the tone bar, the glyph, the name,
/// the dot and the line, and beside the link its controls.
pub(crate) fn card_drawing(card: CardDrawing) -> AnyView {
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
        controls,
    } = card;
    let height = place.height;
    let muted = state == FlowState::Off;
    let class = format!("fui-tnode {kind} {}", state.tone().class());
    let shown_dot = (!hub).then(|| dot(state, 47.0, height / 2.0 + 10.0));
    let line_x = if hub { 42.0 } else { 56.0 };
    let tip = label.clone();
    let name_class = if mono {
        "fui-tnode-name fui-tnode-mono"
    } else {
        "fui-tnode-name"
    };
    view! {
        <g
            class=class
            class:is-muted=muted
            class:is-selected=selected
            transform=format!("translate({:.1} {:.1})", place.x, place.y)
        >
            <a class="fui-tnode-link" href=href aria-label=label>
                <title>{tip}</title>
                <rect class="fui-tnode-box" width=place.width height=height rx="8"></rect>
                <rect class="fui-tnode-bar" width="4" height=height - 16.0 x="0" y="8" rx="2"></rect>
                <g class="fui-tnode-glyph" transform=format!("translate(14 {:.1})", height / 2.0 - 9.0)>
                    {icon(glyph, "18")}
                </g>
                <text class=name_class x="42" y=height / 2.0 - 4.0>
                    {fit(&name, name_most)}
                </text>
                {shown_dot}
                <text class="fui-tnode-line" x=line_x y=height / 2.0 + 14.0>
                    {fit(&line, line_most)}
                </text>
            </a>
            {controls}
        </g>
    }
    .into_any()
}

/// The path a hovered or focused card shows: `lines` raised, the glow and
/// the pulse along `route`, hidden from assistive technology.
pub(crate) fn path_drawing(lines: Vec<AnyView>, route: String) -> AnyView {
    view! {
        <g class="fui-pipeline-path" aria-hidden="true">
            {lines}
            <path class="fui-path-glow" d=route.clone()></path>
            <path class="fui-path-pulse" d=route pathLength="100"></path>
        </g>
    }
    .into_any()
}

/// The drawing of `card` at `place`, a link to its page, after its path
/// with the pulse, which shows while the card is hovered or focused.
fn card_view(
    card: &FlowCard,
    place: Place,
    flow: &Flow,
    places: &BTreeMap<&str, Place>,
) -> AnyView {
    let steps: Vec<&FlowEdge> = card
        .path
        .windows(2)
        .filter_map(|pair| match pair {
            [from, to] => flow
                .edges
                .iter()
                .find(|edge| edge.from == *from && edge.to == *to),
            _ => None,
        })
        .collect();
    let path = (!steps.is_empty()).then(|| {
        let lines = steps
            .iter()
            .filter_map(|edge| edge_view(edge, places, true))
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
        path_drawing(lines, route)
    });
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
        label: card_name(card),
        hub: card.hub,
        mono: false,
        selected: false,
        most: (23, 22),
        controls: None,
    });
    view! {
        <g class="fui-pipeline-card">
            {path}
            {drawing}
        </g>
    }
    .into_any()
}

/// The description of the drawing: each problem edge with its state and
/// label, or that there is none.
fn problems(flow: &Flow) -> String {
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
                ("status", edge.state.word().into()),
            ];
            match &edge.label {
                Some(label) => {
                    args.push(("note", label.clone().into()));
                    tr_with(Text::FlowEdgeNoted, &args)
                }
                None => tr_with(Text::FlowEdge, &args),
            }
        })
        .collect();
    if sentences.is_empty() {
        tr(Text::FlowSteady)
    } else {
        format!("{}.", sentences.join(". "))
    }
}

/// The phone's form of `flow`: a heading per lane and a row per card, each
/// a link to its page with its state as a dot and a word.
fn list(flow: &Flow) -> AnyView {
    let lanes = flow
        .lanes
        .iter()
        .map(|lane| {
            let rows = lane
                .cards
                .iter()
                .map(|card| {
                    let class = format!("fui-dot {}", card.state.tone().class());
                    let muted = card.state == FlowState::Off;
                    let word = card.state.word();
                    view! {
                        <li>
                            <a class="fui-row-link" href=card.href.clone()>
                                <span class=class class:is-muted=muted role="img" aria-label=word.clone() title=word></span>
                                <span class="fui-pipeline-name">{card.name.clone()}</span>
                                <span class="fui-pipeline-line">{card.line.clone()}</span>
                            </a>
                        </li>
                    }
                })
                .collect_view();
            view! {
                <li>
                    <p class="fui-pipeline-lane">{lane.title.clone()}</p>
                    <ul>{rows}</ul>
                </li>
            }
        })
        .collect_view();
    view! { <ul class="fui-pipeline-list" aria-label=flow.list_label.clone()>{lanes}</ul> }
        .into_any()
}

/// Returns `flow`: the drawing, which a phone replaces with its list, and
/// the quiet foot line.
///
/// Put it in a panel ([`crate::view::surface::titled_panel`]), whose head
/// holds the title and the actions. Cards in a lane spread over the height
/// of the fullest lane; a hub card is taller and carries no dot.
#[must_use]
pub fn flow(flow: &Flow) -> AnyView {
    let (places, width, height) = layout(flow);
    let view_box = format!("-8 -4 {:.0} {height:.0}", width + 16.0);
    let lanes = flow
        .lanes
        .iter()
        .enumerate()
        .map(|(column, lane)| {
            let x = units(column) * (LANE + GAP);
            view! {
                <text class="fui-topology-lane" x=x y="12">
                    {lane.title.clone()}
                </text>
            }
        })
        .collect_view();
    let edges = flow
        .edges
        .iter()
        .filter_map(|edge| edge_view(edge, &places, false))
        .collect_view();
    let cards = flow
        .lanes
        .iter()
        .flat_map(|lane| lane.cards.iter())
        .filter_map(|card| {
            let place = places.get(card.id.as_str())?;
            Some(card_view(card, *place, flow, &places))
        })
        .collect_view();
    let title = format!("{}-title", flow.id);
    let desc = format!("{}-desc", flow.id);
    let named_by = format!("{title} {desc}");
    view! {
        <div class="fui-pipeline">
            <div class="fui-topology-canvas">
                <div class="fui-pipeline-stage">
                    <svg
                        class="fui-topology-svg fui-pipeline-edges"
                        viewBox=view_box.clone()
                        role="img"
                        aria-labelledby=named_by
                    >
                        <title id=title>{flow.label.clone()}</title>
                        <desc id=desc>{problems(flow)}</desc>
                        <g>{lanes}</g>
                        <g>{edges}</g>
                    </svg>
                    <svg
                        class="fui-topology-svg fui-pipeline-cards"
                        viewBox=view_box
                        role="group"
                        aria-label=flow.list_label.clone()
                    >
                        {cards}
                    </svg>
                </div>
            </div>
            {list(flow)}
            <p class="fui-pipeline-foot">{flow.foot.clone()}</p>
        </div>
    }
    .into_any()
}
