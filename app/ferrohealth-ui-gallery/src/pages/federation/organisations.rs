// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The synthetic federation's organisations, their nodes and their groups.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::flow::FlowState;
use ferrohealth_ui::view::topology::{RowAction, TopologyCard, TopologyGroup};

use crate::app::PROBE_ROUTE;
use crate::locale::Msg;
use crate::visit::Visit;

use super::asked::Asked;
use super::{ITS_NODES, ORGANISATION, home};

/// A card of the synthetic federation.
pub(super) struct Part {
    pub(super) id: String,
    pub(super) glyph: Glyph,
    pub(super) name: String,
    pub(super) line: String,
    pub(super) state: FlowState,
    pub(super) mono: bool,
    pub(super) label: Option<String>,
    pub(super) measure: Option<String>,
    pub(super) note: Option<String>,
}

impl Part {
    /// A part with no edge label, measure or note.
    pub(super) fn new(
        id: &str,
        glyph: Glyph,
        name: String,
        line: String,
        state: FlowState,
    ) -> Self {
        Self {
            id: id.to_owned(),
            glyph,
            name,
            line,
            state,
            mono: false,
            label: None,
            measure: None,
            note: None,
        }
    }
}

/// The topology's card of `part`, leading to its page in `brand`, its quick
/// look a link of the state `asked`.
pub(super) fn card_of(
    v: &Visit,
    part: Part,
    brand: Brand,
    asked: &Asked,
    probe: bool,
) -> TopologyCard {
    let look = asked.query(|next| next.look = Some(part.id.clone()));
    let action = (probe && part.state != FlowState::Off).then(|| RowAction {
        label: v.tr(Msg::FedProbeNow),
        glyph: Glyph::ACTIVITY,
        action: String::from(PROBE_ROUTE),
        name: String::from("node"),
        value: part.id.clone(),
    });
    TopologyCard {
        href: format!("{}/{}", home(brand), part.id),
        look: Some(format!("{}{look}", home(brand))),
        id: part.id,
        glyph: part.glyph,
        name: part.name,
        line: part.line,
        state: part.state,
        mono: part.mono,
        edge_label: part.label,
        measure: part.measure,
        note: part.note,
        action,
    }
}

/// A node of the synthetic federation in `state`, its edge labelled with
/// `label` when it is a problem.
fn fed_node(v: &Visit, id: &str, state: FlowState, line: Msg, label: Option<Msg>) -> Part {
    let measure =
        matches!(state, FlowState::Up | FlowState::Degraded).then(|| v.tr(Msg::FedMedian));
    let note = match state {
        FlowState::Off => v.tr(Msg::FedAskedNothing),
        FlowState::NotChecked => v.tr(Msg::FedNeverProbed),
        _ => v.tr(Msg::FedProbed),
    };
    Part {
        mono: true,
        label: label.map(|label| v.tr(label)),
        measure,
        note: Some(note),
        ..Part::new(id, Glyph::SERVER, id.to_owned(), v.tr(line), state)
    }
}

/// One organisation of the synthetic federation: its id, name, line, state
/// and nodes.
type Organisation = (&'static str, Msg, Msg, FlowState, Vec<Part>);

/// The organisations of the synthetic federation.
pub(super) fn organisations(v: &Visit) -> Vec<Organisation> {
    vec![
        (
            ORGANISATION,
            Msg::FedOrgVoorbeeldstad,
            Msg::FedTwoNodesReachable,
            FlowState::Up,
            ITS_NODES
                .iter()
                .map(|(id, state)| fed_node(v, id, *state, Msg::FedReachable, None))
                .collect(),
        ),
        (
            "oost",
            Msg::FedOrgOost,
            Msg::FedDegraded,
            FlowState::Degraded,
            vec![fed_node(
                v,
                "node-b",
                FlowState::Degraded,
                Msg::FedDegraded503,
                Some(Msg::Fed503),
            )],
        ),
        (
            "proefdorp",
            Msg::FedOrgProefdorp,
            Msg::FedUnreachable,
            FlowState::Unreachable,
            vec![fed_node(
                v,
                "node-c",
                FlowState::Unreachable,
                Msg::FedUnreachableNoAnswer,
                Some(Msg::FedNoAnswer),
            )],
        ),
        (
            "testmeer",
            Msg::FedOrgTestmeer,
            Msg::FedExcluded,
            FlowState::Off,
            vec![fed_node(
                v,
                "node-e",
                FlowState::Off,
                Msg::FedExcluded,
                None,
            )],
        ),
    ]
}

/// The organisations of a large synthetic federation, each named by its
/// number: twelve with two to four nodes, one degraded and one not yet
/// probed, or fifty with two to six nodes, two hundred in all, a few in
/// each problem state.
pub(super) fn many_organisations(
    v: &Visit,
    fifty: bool,
) -> Vec<(String, String, FlowState, Vec<Part>)> {
    let last = if fifty { 50 } else { 12 };
    (1_u8..=last)
        .map(|number| {
            let count = if fifty {
                2 + usize::from(number % 5)
            } else {
                2 + usize::from(number % 3)
            };
            let state = match (fifty, number) {
                (false, 5) | (true, 5 | 23 | 41) => FlowState::Degraded,
                (true, 17 | 34) => FlowState::Unreachable,
                (false, 8) | (true, 8 | 29) => FlowState::NotChecked,
                (true, 46) => FlowState::Off,
                _ => FlowState::Up,
            };
            let nodes = (0..count)
                .map(|index| {
                    let letter = char::from(b'a' + u8::try_from(index).unwrap_or(0));
                    let id = format!("node-{number:02}{letter}");
                    let (node_state, line, label) = match (state, index) {
                        (FlowState::Degraded, 1) => {
                            (FlowState::Degraded, Msg::FedDegraded503, Some(Msg::Fed503))
                        }
                        (FlowState::Unreachable, 0) => (
                            FlowState::Unreachable,
                            Msg::FedUnreachableNoAnswer,
                            Some(Msg::FedNoAnswer),
                        ),
                        (FlowState::NotChecked, _) => {
                            (FlowState::NotChecked, Msg::FedNotProbed, None)
                        }
                        (FlowState::Off, _) => (FlowState::Off, Msg::FedExcluded, None),
                        _ => (FlowState::Up, Msg::FedReachable, None),
                    };
                    fed_node(v, &id, node_state, line, label)
                })
                .collect();
            (
                format!("org-{number:02}"),
                format!("{} {number:02}", v.tr(Msg::FedOrganisation)),
                state,
                nodes,
            )
        })
        .collect()
}

/// The group of an organisation, folded as `asked` says, its nodes those
/// whose state is shown.
pub(super) fn group(
    v: &Visit,
    (id, name, line, state, nodes): (String, String, String, FlowState, Vec<Part>),
    brand: Brand,
    asked: &Asked,
) -> Option<TopologyGroup> {
    let children: Vec<TopologyCard> = nodes
        .into_iter()
        .filter(|node| asked.shows(node.state))
        .map(|node| card_of(v, node, brand, asked, true))
        .collect();
    if children.is_empty() {
        return None;
    }
    let collapsed = asked.folded(&id);
    let toggle = asked.query(|next| {
        next.all_folded = false;
        if collapsed {
            next.collapse.remove(&id);
        } else {
            next.collapse.insert(id.clone());
        }
    });
    let label = children
        .iter()
        .find(|child| child.state.is_problem())
        .and_then(|child| child.edge_label.clone());
    let card = card_of(
        v,
        Part {
            label,
            ..Part::new(&id, Glyph::USERS, name, line, state)
        },
        brand,
        asked,
        false,
    );
    Some(TopologyGroup {
        card,
        children,
        collapsed,
        toggle: Some(format!("{}{toggle}", home(brand))),
    })
}
