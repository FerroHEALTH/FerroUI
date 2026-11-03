// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The topology's layout: the hub, the callers, the sides and their groups
//! placed on the map.

use crate::view::flow::FlowState;
use crate::view::flow::layout::{Place, units};

use super::{Topology, TopologyCallers, TopologyCard, TopologyGroup, TopologySide};

/// A card's height, a row's height, the gap between groups, the hub's
/// width and height, a caller's width and the room above the sides when
/// there are callers and when there are none.
const CARD: f64 = 56.0;
const ROW: f64 = 66.0;
const GROUP_GAP: f64 = 14.0;
const HUB_WIDTH: f64 = 176.0;
const HUB_HEIGHT: f64 = 72.0;
const CALLER: f64 = 204.0;
const TOP_CALLED: f64 = 118.0;
const TOP: f64 = 34.0;

/// The widths of a side's children and groups, the gap between the two
/// columns, and the gap between the groups and the hub.
const END_CHILD: f64 = 182.0;
const END_GROUP: f64 = 272.0;
const START_CHILD: f64 = 222.0;
const START_GROUP: f64 = 236.0;
const OUTER_GAP: f64 = 92.0;
const HUB_GAP: f64 = 84.0;

/// The minimap's widest and tallest measure.
pub(super) const MINIMAP: f64 = 168.0;
pub(super) const MINIMAP_TALL: f64 = 300.0;

/// One edge laid out.
#[derive(Debug, Clone, Copy)]
pub(super) struct Edge<'a> {
    pub(super) from: Place,
    pub(super) to: Place,
    pub(super) state: FlowState,
    pub(super) label: Option<&'a str>,
    pub(super) marked: bool,
    pub(super) down: bool,
}

/// One card laid out: where it sits, what it is, and the cards its path
/// from the hub runs through.
#[derive(Debug)]
pub(super) struct Laid<'a> {
    pub(super) card: &'a TopologyCard,
    pub(super) place: Place,
    pub(super) kind: &'static str,
    pub(super) group: Option<&'a TopologyGroup>,
    pub(super) via: Vec<Place>,
    pub(super) edges: Vec<Edge<'a>>,
}

/// The layout of a topology: every card, every edge, the lane titles and
/// the drawing's width and height.
#[derive(Debug)]
pub(super) struct Layout<'a> {
    pub(super) cards: Vec<Laid<'a>>,
    pub(super) edges: Vec<Edge<'a>>,
    pub(super) lanes: Vec<(f64, f64, &'a str, bool)>,
    pub(super) rule: Option<(f64, f64)>,
    pub(super) width: f64,
    pub(super) height: f64,
}

/// The groups of a side, each with its place and its children's places.
type Stacked<'a> = Vec<(&'a TopologyGroup, Place, Vec<Place>)>;

/// The rows of `side`, top first: each group with its place and its
/// children's places, the next free row after the last.
fn stack(
    side: &TopologySide,
    top: f64,
    x_group: f64,
    group_width: f64,
    x_child: f64,
    child_width: f64,
) -> (Stacked<'_>, f64) {
    let mut y = top;
    let mut rows = Vec::new();
    for group in &side.groups {
        let open = !group.collapsed && !group.children.is_empty();
        let count = if open { group.children.len() } else { 1 };
        let last = y + units(count.saturating_sub(1)) * ROW;
        let place = Place {
            x: x_group,
            y: f64::midpoint(y, last),
            width: group_width,
            height: CARD,
        };
        let children = if open {
            group
                .children
                .iter()
                .enumerate()
                .map(|(index, _)| Place {
                    x: x_child,
                    y: y + units(index) * ROW,
                    width: child_width,
                    height: CARD,
                })
                .collect()
        } else {
            Vec::new()
        };
        rows.push((group, place, children));
        y = last + ROW + GROUP_GAP;
    }
    (rows, y - GROUP_GAP)
}

/// Lays `topology` out.
pub(super) fn layout(topology: &Topology) -> Layout<'_> {
    let top = if topology.callers.is_some() {
        TOP_CALLED
    } else {
        TOP
    };
    let x_hub = if topology.start.is_some() {
        START_CHILD + OUTER_GAP + START_GROUP + HUB_GAP
    } else {
        0.0
    };
    let x_end_group = x_hub + HUB_WIDTH + HUB_GAP;
    let x_end_child = x_end_group + END_GROUP + OUTER_GAP;
    let start = topology.start.as_ref().map(|side| {
        stack(
            side,
            top,
            START_CHILD + OUTER_GAP,
            START_GROUP,
            0.0,
            START_CHILD,
        )
    });
    let end = topology
        .end
        .as_ref()
        .map(|side| stack(side, top, x_end_group, END_GROUP, x_end_child, END_CHILD));
    let bottom = [&start, &end]
        .into_iter()
        .flatten()
        .map(|(_, bottom)| *bottom)
        .fold(top + CARD, f64::max);
    let middle = (top + bottom - (ROW - CARD)) / 2.0;
    let hub = Place {
        x: x_hub,
        y: middle - HUB_HEIGHT / 2.0,
        width: HUB_WIDTH,
        height: HUB_HEIGHT,
    };
    let mut laid = Layout {
        cards: vec![Laid {
            card: &topology.hub,
            place: hub,
            kind: "fui-tnode-gateway",
            group: None,
            via: Vec::new(),
            edges: Vec::new(),
        }],
        edges: Vec::new(),
        lanes: Vec::new(),
        rule: None,
        width: x_hub + HUB_WIDTH,
        height: bottom.max(hub.y + hub.height),
    };
    if let Some(callers) = &topology.callers {
        place_callers(&mut laid, callers, hub);
    }
    for (side, rows, title_x) in [
        (topology.start.as_ref(), start, 0.0),
        (topology.end.as_ref(), end, x_end_group),
    ] {
        if let (Some(side), Some((rows, _))) = (side, rows) {
            laid.lanes
                .push((title_x, top - 16.0, side.title.as_str(), false));
            place_side(&mut laid, rows, hub);
        }
    }
    if topology.callers.is_some() {
        laid.rule = Some((0.0, laid.width));
    }
    laid
}

/// Places `callers` side by side above the `hub`, each with its edge down
/// to it, under their title.
fn place_callers<'a>(laid: &mut Layout<'a>, callers: &'a TopologyCallers, hub: Place) {
    let count = units(callers.cards.len().max(1));
    let span = count * CALLER + (count - 1.0) * 24.0;
    let left = hub.x + hub.width / 2.0 - span / 2.0;
    laid.lanes
        .push((hub.x + hub.width / 2.0, 12.0, callers.title.as_str(), true));
    for (index, card) in callers.cards.iter().enumerate() {
        let place = Place {
            x: left + units(index) * (CALLER + 24.0),
            y: 22.0,
            width: CALLER,
            height: CARD,
        };
        laid.edges.push(Edge {
            from: place,
            to: hub,
            state: card.state,
            label: None,
            marked: false,
            down: true,
        });
        laid.cards.push(Laid {
            card,
            place,
            kind: "fui-tnode-node",
            group: None,
            via: Vec::new(),
            edges: Vec::new(),
        });
    }
}

/// Places the stacked groups `rows` of one side, each with its edge from
/// the `hub` and its children with theirs.
fn place_side<'a>(laid: &mut Layout<'a>, rows: Stacked<'a>, hub: Place) {
    for (group, place, children) in rows {
        let folded = children.is_empty();
        let into_group = Edge {
            from: hub,
            to: place,
            state: group.card.state,
            label: group.card.edge_label.as_deref().filter(|_| folded),
            marked: folded,
            down: false,
        };
        laid.edges.push(into_group);
        laid.width = laid.width.max(place.x + place.width);
        laid.cards.push(Laid {
            card: &group.card,
            place,
            kind: "fui-tnode-org",
            group: Some(group),
            via: vec![hub, place],
            edges: vec![into_group],
        });
        for (card, child) in group.children.iter().zip(children) {
            let into_child = Edge {
                from: place,
                to: child,
                state: card.state,
                label: card.edge_label.as_deref(),
                marked: true,
                down: false,
            };
            laid.edges.push(into_child);
            laid.width = laid.width.max(child.x + child.width);
            laid.cards.push(Laid {
                card,
                place: child,
                kind: "fui-tnode-node",
                group: None,
                via: vec![hub, place, child],
                edges: vec![into_group, into_child],
            });
        }
    }
}
