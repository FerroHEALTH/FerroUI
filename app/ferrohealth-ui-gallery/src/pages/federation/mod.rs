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

mod asked;
mod map;
mod organisations;
mod outcome;

use ferrohealth_ui::html::Html;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::flow::FlowState;
use ferrohealth_ui::view::nav::{Nest, NestItem};
use ferrohealth_ui::view::replica::Replica;
use ferrohealth_ui::view::surface::{Container, Crumb, Header, card, page_header, page_heading};
use ferrohealth_ui::view::template::settings::{How, Setting, SettingsPage, Source, settings_page};

use crate::locale::Msg;
use crate::page::Page;
use crate::visit::Visit;

use asked::asked;
use map::federation_map;
use outcome::{outcome, outcome_none_asked};

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

/// The federation page: the fan-out outcomes and the topology.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let brand = v.brand;
    let asked = asked(v);
    let header = page_header(
        &v.locale,
        v.tr(Msg::Federation),
        Some(v.tr(Msg::FedSubtitle)),
        ActionGroup::default(),
    );
    let map = card(v.tr(Msg::FedTopology), federation_map(v, brand, &asked));
    let body = Container::Stack.wrap(Html::join([
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
        value: Html::raw(format!("<code>{}</code>", Html::text(value))),
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
            actions: ActionGroup::default(),
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
            ActionGroup::default(),
        );
        Html::join([
            heading,
            card(
                v.tr(Msg::FedInsideTitle),
                Html::raw(format!("<p>{}</p>", Html::text(v.tr(Msg::FedInsideBody)))),
            ),
        ])
    };
    Page::shell(v.tr(Msg::Federation), body)
}
