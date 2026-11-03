// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The federation page's topology: its services, its quick look, its hub and
//! callers and the map.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::flow::FlowState;
use ferrohealth_ui::view::topology::{
    ClusterLink, QuickLook, StatusCount, StatusMenu, Topology, TopologyCallers, TopologyCard,
    TopologyGroup, TopologySide, TopologyView, topology,
};

use crate::app::PROBE_ROUTE;
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

use super::asked::Asked;
use super::organisations::{Part, card_of, group, many_organisations, organisations};
use super::{STATES, home};

/// The service groups the synthetic gateway asks.
fn services(v: &Visit, brand: Brand, asked: &Asked) -> TopologySide {
    let service = |id: &str, name: Msg, state: FlowState, line: Msg, label: Option<Msg>| Part {
        label: label.map(|label| v.tr(label)),
        measure: (state == FlowState::Up).then(|| v.tr(Msg::FedMedian)),
        ..Part::new(id, Glyph::LINK, v.tr(name), v.tr(line), state)
    };
    let identity = (
        String::from("identity"),
        v.tr(Msg::FedIdentity),
        v.tr(Msg::FedTwoServicesReachable),
        FlowState::Up,
        vec![
            service("pixm", Msg::FedPixm, FlowState::Up, Msg::FedReachable, None),
            service("pdqm", Msg::FedPdqm, FlowState::Up, Msg::FedReachable, None),
        ],
    );
    let consent = (
        String::from("consent"),
        v.tr(Msg::FedConsent),
        v.tr(Msg::FedDegraded),
        FlowState::Degraded,
        vec![service(
            "registry",
            Msg::FedConsentRegistry,
            FlowState::Degraded,
            Msg::FedDegraded503,
            Some(Msg::Fed503),
        )],
    );
    TopologySide {
        title: v.tr(Msg::FedServices),
        line: v.tr(Msg::FedServicesLine),
        groups: [identity, consent]
            .into_iter()
            .filter_map(|each| group(v, each, brand, asked))
            .collect(),
    }
}

/// The quick look of the card `id`, when it is one of the federation's.
fn quick_look(v: &Visit, id: &str, brand: Brand, asked: &Asked) -> QuickLook {
    let close = asked.query(|next| next.look = None);
    let probe = h(format!(
        "<form method=\"post\" action=\"{PROBE_ROUTE}\" class=\"fui-actions\"><input type=\"hidden\" name=\"node\" value=\"{}\"><button type=\"submit\" class=\"fui-button fui-button-primary fui-button-small\">{}{}</button><a href=\"{}/{}\" class=\"fui-button fui-button-secondary fui-button-small\">{}</a></form>",
        t(id),
        ferrohealth_ui::icon::icon(Glyph::ACTIVITY, "14"),
        t(v.tr(Msg::FedProbeNow)),
        t(home(brand)),
        t(id),
        t(v.tr(Msg::FedOpenPage))
    ));
    QuickLook {
        card: id.to_owned(),
        title: id.to_owned(),
        mono: true,
        line: Some(v.tr(Msg::FedLookLine)),
        close: format!("{}{close}", home(brand)),
        body: h(format!(
            "<p class=\"fui-small\">{}</p>{probe}",
            t(v.tr(Msg::FedLookBody))
        )),
    }
}

/// The gateway at the hub and the planned contact point that calls it,
/// each leading to the self-description.
fn hub_and_caller(v: &Visit, brand: Brand, asked: &Asked) -> (TopologyCard, TopologyCard) {
    let hub = card_of(
        v,
        Part::new(
            "gateway",
            Glyph::NETWORK,
            v.tr(Msg::FedGateway),
            v.tr(Msg::FedGatewayLine),
            FlowState::Up,
        ),
        brand,
        asked,
        false,
    );
    let hub = TopologyCard {
        href: format!("{}/self-description", home(brand)),
        look: None,
        ..hub
    };
    let caller = TopologyCard {
        look: None,
        href: format!("{}/self-description", home(brand)),
        ..card_of(
            v,
            Part::new(
                "ncp",
                Glyph::RADIO_TOWER,
                v.tr(Msg::FedContactPoint),
                v.tr(Msg::FedContactPointLine),
                FlowState::NotChecked,
            ),
            brand,
            asked,
            false,
        )
    };
    (hub, caller)
}

/// The organisations the topology draws in the state `asked`: each one's
/// id, name, line, state and nodes.
fn sources(v: &Visit, asked: &Asked) -> Vec<(String, String, String, FlowState, Vec<Part>)> {
    if let Some(large) = asked.large {
        many_organisations(v, large == 50)
            .into_iter()
            .map(|(id, name, state, nodes)| {
                let line = v.tr(match state {
                    FlowState::Up => Msg::FedNodesReachable,
                    FlowState::Off => Msg::FedExcluded,
                    _ => Msg::FedOneNotWell,
                });
                (id, name, line, state, nodes)
            })
            .collect()
    } else {
        organisations(v)
            .into_iter()
            .map(|(id, name, line, state, nodes)| {
                (id.to_owned(), v.tr(name), v.tr(line), state, nodes)
            })
            .collect()
    }
}

/// The topology of the synthetic federation in the state `asked`.
pub(super) fn federation_map(v: &Visit, brand: Brand, asked: &Asked) -> Html {
    let sources = sources(v, asked);
    let states: Vec<FlowState> = sources
        .iter()
        .flat_map(|(_, _, _, _, nodes)| nodes.iter().map(|node| node.state))
        .collect();
    let counted = |state: FlowState| -> u32 {
        u32::try_from(states.iter().filter(|each| **each == state).count()).unwrap_or(u32::MAX)
    };
    let groups: Vec<TopologyGroup> = sources
        .into_iter()
        .filter_map(|each| group(v, each, brand, asked))
        .collect();
    let (hub, caller) = hub_and_caller(v, brand, asked);
    let look = asked
        .look
        .as_deref()
        .map(|id| quick_look(v, id, brand, asked));
    topology(
        &v.locale,
        Topology {
            id: String::from("federation-map"),
            label: v.tr(Msg::FedMapLabel),
            list_label: v.tr(Msg::FedListLabel),
            hub,
            callers: asked.large.is_none().then(|| TopologyCallers {
                title: v.tr(Msg::FedCallers),
                line: v.tr(Msg::FedCallersLine),
                cards: vec![caller],
            }),
            start: asked.large.is_none().then(|| services(v, brand, asked)),
            end: Some(TopologySide {
                title: v.tr(Msg::FedOrganisations),
                line: v.tr(Msg::FedOrganisationsLine),
                groups,
            }),
            view: asked.view,
            views: (
                format!(
                    "{}{}",
                    home(brand),
                    asked.query(|next| next.view = TopologyView::Map)
                ),
                format!(
                    "{}{}",
                    home(brand),
                    asked.query(|next| next.view = TopologyView::List)
                ),
            ),
            status: StatusMenu {
                action: home(brand),
                name: String::from("show"),
                states: STATES
                    .into_iter()
                    .map(|state| StatusCount {
                        state,
                        count: counted(state),
                        shown: asked.shows(state),
                    })
                    .collect(),
            },
            tools: None,
            updated: Some(h(format!(
                "<span class=\"fui-small fui-muted fui-topology-updated\">{}</span>",
                t(v.tr(Msg::FedUpdated))
            ))),
            collapse_all: Some(ClusterLink {
                href: format!(
                    "{}{}",
                    home(brand),
                    asked.query(|next| {
                        next.all_folded = !next.all_folded;
                        next.collapse.clear();
                    })
                ),
                on: asked.all_folded,
            }),
            minimap: Some(ClusterLink {
                href: format!(
                    "{}{}",
                    home(brand),
                    asked.query(|next| next.minimap = !next.minimap)
                ),
                on: asked.minimap,
            }),
            look,
        },
    )
}
