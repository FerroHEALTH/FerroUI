// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The federation page and the pages inside it.
//!
//! The page holds the fan-out outcome with every node answering, one node
//! failing and every node failing, and the topology of a synthetic
//! federation with its List twin. Its query names the topology's state:
//! `view` (`map` or `list`), `look` (the card whose quick look is open),
//! `collapse` (`all` or the groups folded), `show` (the states shown),
//! `large` (`1` for a federation of twelve organisations, `50` for one of
//! fifty organisations and two hundred nodes) and `minimap`. Inside it
//! an organisation's and a node's page open the nested sidebar section,
//! and the gateway's self-description is a reference page with the replica
//! line. Every name is synthetic.

use std::collections::BTreeSet;

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::fanout::{Answer, Fanout, FanoutNode, fanout};
use ferrohealth_ui::view::flow::FlowState;
use ferrohealth_ui::view::nav::{Nest, NestItem};
use ferrohealth_ui::view::replica::Replica;
use ferrohealth_ui::view::surface::{Crumb, Header, card, page_header, page_heading, stack};
use ferrohealth_ui::view::template::settings::{How, Setting, SettingsPage, Source, settings_page};
use ferrohealth_ui::view::topology::{
    ClusterLink, QuickLook, RowAction, StatusCount, StatusMenu, Topology, TopologyCallers,
    TopologyCard, TopologyGroup, TopologySide, TopologyView, topology,
};

use crate::app::{PROBE_ROUTE, Page};
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The organisation the nested sidebar section opens, and its nodes.
const ORGANISATION: &str = "voorbeeldstad";

/// The nodes of [`ORGANISATION`], each with its state.
const ITS_NODES: [(&str, FlowState); 2] =
    [("node-a", FlowState::Up), ("node-a-lab", FlowState::Up)];

/// The every state of the Status menu, in its order.
const STATES: [FlowState; 5] = [
    FlowState::Up,
    FlowState::Degraded,
    FlowState::Unreachable,
    FlowState::NotChecked,
    FlowState::Off,
];

/// The path of the federation page of `brand`.
fn home(brand: Brand) -> String {
    format!("/{}/federation", brand.key())
}

/// One node of a fan-out: its name, how it answered, its word and term,
/// what it gave and why it failed.
fn node(
    v: &Visit,
    name: &str,
    answer: Answer,
    word: Msg,
    term: &str,
    measure: Msg,
    reason: Option<Msg>,
) -> FanoutNode {
    FanoutNode {
        name: name.to_owned(),
        answer,
        word: v.tr(word),
        term: Some(term.to_owned()),
        measure: v.tr(measure),
        reason: reason.map(|reason| v.tr(reason)),
    }
}

/// The fan-out of one query: every node answered, one failed, or all did.
fn outcome(v: &Visit, failing: usize) -> Html {
    let failed = |at: usize, name: &str, rows: Msg| {
        if at < failing {
            node(
                v,
                name,
                Answer::Failed,
                Msg::FedNodeError,
                "node-error",
                Msg::FedMs390,
                Some(Msg::FedAnswered503),
            )
        } else {
            node(
                v,
                name,
                Answer::Answered,
                Msg::FedAnswered,
                "active",
                rows,
                None,
            )
        }
    };
    let nodes = vec![
        failed(3, "node-a-pub", Msg::FedRows3),
        failed(1, "node-a-lab-pub", Msg::FedRows1),
        failed(0, "node-b-pub", Msg::FedRows2),
        failed(2, "node-c-pub", Msg::FedRows0),
        node(
            v,
            "node-e-pub",
            Answer::Skipped,
            Msg::FedExcluded,
            "excluded",
            Msg::FedNotAsked,
            Some(Msg::FedExcludedByOperator),
        ),
    ];
    let (title, line, missing) = match failing {
        0 => (
            Msg::FedAllAnswered,
            Msg::FedLineComplete,
            Msg::FedMissingNone,
        ),
        1 => (Msg::FedOneFailing, Msg::FedLinePartial, Msg::FedMissingOne),
        _ => (Msg::FedAllFailing, Msg::FedLineFailed, Msg::FedMissingAll),
    };
    let body = fanout(
        &v.locale,
        &Fanout {
            label: v.tr(Msg::FedWhatEachDid),
            line: v.tr(line),
            missing: v.tr(missing),
            nodes,
        },
    );
    card(v.tr(title), body)
}

/// The fan-out of a query no node was asked for, every node excluded,
/// each named by a long host name.
fn outcome_none_asked(v: &Visit) -> Html {
    let nodes = ["node-eu-west-1", "node-eu-central-2", "node-eu-north-3"]
        .map(|host| {
            node(
                v,
                &format!("{host}.ferrofed.internal.example.org"),
                Answer::Skipped,
                Msg::FedExcluded,
                "excluded",
                Msg::FedNotAsked,
                Some(Msg::FedExcludedByOperator),
            )
        })
        .to_vec();
    let body = fanout(
        &v.locale,
        &Fanout {
            label: v.tr(Msg::FedWhatEachDid),
            line: v.tr(Msg::FedLineNoneAsked),
            missing: String::new(),
            nodes,
        },
    );
    card(v.tr(Msg::FedNoneAsked), body)
}

/// The state of the topology the query names.
#[derive(Debug, Clone, Default)]
struct Asked {
    view: TopologyView,
    look: Option<String>,
    collapse: BTreeSet<String>,
    all_folded: bool,
    show: Option<BTreeSet<String>>,
    large: Option<u8>,
    minimap: bool,
}

impl Asked {
    /// The query of this state, with `change` applied, as a link's query.
    fn query(&self, change: impl FnOnce(&mut Self)) -> String {
        let mut next = self.clone();
        change(&mut next);
        let mut parts = Vec::new();
        match next.view {
            TopologyView::Auto => {}
            TopologyView::Map => parts.push(String::from("view=map")),
            TopologyView::List => parts.push(String::from("view=list")),
        }
        if let Some(look) = &next.look {
            parts.push(format!("look={look}"));
        }
        if next.all_folded {
            parts.push(String::from("collapse=all"));
        } else if !next.collapse.is_empty() {
            let folded: Vec<&str> = next.collapse.iter().map(String::as_str).collect();
            parts.push(format!("collapse={}", folded.join(",")));
        }
        if let Some(show) = &next.show {
            parts.extend(show.iter().map(|state| format!("show={state}")));
        }
        if let Some(large) = next.large {
            parts.push(format!("large={large}"));
        }
        if next.minimap {
            parts.push(String::from("minimap=on"));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }

    /// Whether the group `id` is folded.
    fn folded(&self, id: &str) -> bool {
        self.all_folded || self.collapse.contains(id)
    }

    /// Whether a card in `state` is shown.
    fn shows(&self, state: FlowState) -> bool {
        self.show
            .as_ref()
            .is_none_or(|show| show.contains(state.key()))
    }
}

/// The state the page's query names.
fn asked(v: &Visit) -> Asked {
    let query = &v.query;
    let view = match query.get_str("view") {
        Some("map") => TopologyView::Map,
        Some("list") => TopologyView::List,
        _ => TopologyView::Auto,
    };
    let collapse = query.get_str("collapse").unwrap_or_default();
    let show: Vec<String> = query
        .pairs()
        .iter()
        .filter(|(name, _)| name == "show")
        .map(|(_, state)| state.clone())
        .collect();
    Asked {
        view,
        look: query.get_str("look").map(str::to_owned),
        all_folded: collapse == "all",
        collapse: collapse
            .split(',')
            .filter(|id| !id.is_empty() && *id != "all")
            .map(str::to_owned)
            .collect(),
        show: (!show.is_empty()).then(|| show.into_iter().collect()),
        large: match query.get_str("large") {
            Some("1") => Some(1),
            Some("50") => Some(50),
            _ => None,
        },
        minimap: query.get_str("minimap") == Some("on"),
    }
}

/// A card of the synthetic federation.
struct Part {
    id: String,
    glyph: Glyph,
    name: String,
    line: String,
    state: FlowState,
    mono: bool,
    label: Option<String>,
    measure: Option<String>,
    note: Option<String>,
}

impl Part {
    /// A part with no edge label, measure or note.
    fn new(id: &str, glyph: Glyph, name: String, line: String, state: FlowState) -> Self {
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
fn card_of(v: &Visit, part: Part, brand: Brand, asked: &Asked, probe: bool) -> TopologyCard {
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
fn organisations(v: &Visit) -> Vec<Organisation> {
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
fn many_organisations(v: &Visit, fifty: bool) -> Vec<(String, String, FlowState, Vec<Part>)> {
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
fn group(
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

/// The topology of the synthetic federation in the state `asked`.
fn federation_map(v: &Visit, brand: Brand, asked: &Asked) -> Html {
    let sources: Vec<(String, String, String, FlowState, Vec<Part>)> =
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
        };
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

/// The federation page: the fan-out outcomes and the topology.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let brand = v.brand;
    let asked = asked(v);
    let header = page_header(
        &v.locale,
        v.tr(Msg::Federation),
        Some(v.tr(Msg::FedSubtitle)),
        ActionGroup::none(),
    );
    let map = card(v.tr(Msg::FedTopology), federation_map(v, brand, &asked));
    let body = stack(Html::join([
        map,
        outcome(v, 0),
        outcome(v, 1),
        outcome(v, 5),
        outcome_none_asked(v),
    ]));
    Page::shell(v.tr(Msg::Federation), Html::join([header, body]))
}

/// The nested sidebar section of the page at `path` in `brand`: the
/// organisation and its nodes under the federation's entry, on the
/// organisation's page and its nodes' pages, or `None` elsewhere.
#[must_use]
pub fn nest(v: &Visit, brand: Brand, path: &str, under: &'static str) -> Option<Nest> {
    let part = path.strip_prefix(&format!("{}/", home(brand)))?;
    let inside = part == ORGANISATION || ITS_NODES.iter().any(|(id, _)| *id == part);
    inside.then(|| Nest {
        under: under.into(),
        label: v.tr(Msg::FedNestLabel),
        parent: NestItem {
            label: v.tr(Msg::FedOrgVoorbeeldstad),
            href: format!("{}/{ORGANISATION}", home(brand)),
            state: FlowState::Up,
            state_words: v.tr(Msg::FedWorstReachable),
            mono: false,
        },
        count_words: v.tr(Msg::FedTwoNodes),
        children: ITS_NODES
            .iter()
            .map(|(id, state)| NestItem {
                label: (*id).to_owned(),
                href: format!("{}/{id}", home(brand)),
                state: *state,
                state_words: v.tr(Msg::FedReachable),
                mono: true,
            })
            .collect(),
        all: v.tr(Msg::FedAllNodes),
    })
}

/// The gateway's self-description: a reference page (T3) with the
/// replica line under its title.
fn self_description(v: &Visit, brand: Brand) -> Html {
    let setting = |label: Msg, description: Msg, value: &str, key: &str| Setting {
        label: v.tr(label),
        description: v.tr(description),
        value: h(format!("<code>{}</code>", t(value))),
        keys: vec![key.to_owned()],
        next_start: false,
        source: None,
    };
    settings_page(
        &v.locale,
        SettingsPage {
            header: Header {
                title: v.tr(Msg::FedSelfDescription).into(),
                subtitle: Some(v.tr(Msg::FedSelfDescriptionLine).into()),
                crumbs: vec![Crumb::new(v.tr(Msg::Federation), home(brand))],
                ..Header::default()
            },
            actions: ActionGroup::none(),
            source: Source::File {
                file: String::from("ferrofed.toml"),
                how: How::Page(home(brand)),
            },
            tabs: None,
            settings: vec![
                setting(
                    Msg::FedFederationId,
                    Msg::FedFederationIdLine,
                    "fed-demo-01",
                    "federation.id",
                ),
                setting(
                    Msg::FedPerNode,
                    Msg::FedPerNodeLine,
                    "2500",
                    "timeout.per_node_ms",
                ),
            ],
            replica: Some(Replica {
                name: String::from("gateway-0"),
                at: Some(String::from("09:42:11")),
            }),
        },
    )
}

/// A page inside the federation: an organisation's or a node's, which open
/// the nested sidebar section, or the gateway's self-description.
#[must_use]
pub fn part(v: &Visit) -> Page {
    let brand = v.brand;
    let part = v.segment(2).unwrap_or_default().to_owned();
    let body = if part == "self-description" {
        self_description(v, brand)
    } else {
        let organisation = part == ORGANISATION;
        let title = if organisation {
            v.tr(Msg::FedOrgVoorbeeldstad)
        } else {
            part.clone()
        };
        let heading = page_heading(
            &v.locale,
            Header {
                title: title.into(),
                subtitle: Some(
                    v.tr(if organisation {
                        Msg::FedOrganisationPage
                    } else {
                        Msg::FedNodePage
                    })
                    .into(),
                ),
                subtitle_view: None,
                crumbs: vec![Crumb::new(v.tr(Msg::Federation), home(brand))],
                mono: !organisation,
                badge: None,
            },
            ActionGroup::none(),
        );
        Html::join([
            heading,
            card(
                v.tr(Msg::FedInsideTitle),
                h(format!("<p>{}</p>", t(v.tr(Msg::FedInsideBody)))),
            ),
        ])
    };
    Page::shell(v.tr(Msg::Federation), body)
}
