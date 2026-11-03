// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The lane order that keeps the edges from crossing, by sweeps and
//! transpositions.

use std::collections::BTreeMap;

use super::layout::units;

/// How many pairs of edges cross between the lanes `lanes`, each lane its
/// ids top to bottom: two edges between the same two lanes cross when
/// their ends lie in opposite orders.
pub(super) fn crossings(lanes: &[Vec<&str>], edges: &[(&str, &str)]) -> usize {
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
pub(super) fn ordered<'a>(
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
