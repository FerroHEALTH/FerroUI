// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The flow's grid: the lane and row sizes, a card's place and the edge
//! geometry.

use std::collections::BTreeMap;

use super::plan::{Plan, Shown};

/// A lane's width, the gap between lanes, a card's height, the hub's
/// height, a row's height and the room above the first row, in the
/// drawing's units.
pub(super) const LANE: f64 = 200.0;
pub(super) const GAP: f64 = 100.0;
pub(super) const CARD: f64 = 56.0;
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

/// The place of every shown card by its id, and the drawing's width and
/// height.
pub(super) fn layout<'p>(plan: &'p Plan<'_>) -> (BTreeMap<&'p str, Place>, f64, f64) {
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
