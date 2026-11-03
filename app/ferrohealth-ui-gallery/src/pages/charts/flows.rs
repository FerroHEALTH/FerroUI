// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page's flows: the bridge at rest and with problem edges, and the busy bridge whose lanes fold.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::flow::{
    EdgeRule, Flow, FlowCard, FlowEdge, FlowLane, FlowRule, FlowState, LaneFold, flow, rule_button,
    rule_dialog,
};
use ferrohealth_ui::view::surface::titled_panel;

use crate::locale::Msg;
use crate::visit::Visit;

/// A card of a flow.
fn flow_card(
    v: &Visit,
    id: &str,
    glyph: Glyph,
    name: Msg,
    line: Msg,
    state: FlowState,
    path: &[&str],
) -> FlowCard {
    FlowCard {
        id: id.to_owned(),
        glyph,
        name: v.tr(name),
        line: v.tr(line),
        state,
        href: format!("#{id}"),
        path: path.iter().map(|step| (*step).to_owned()).collect(),
        hub: id == "hub",
    }
}

/// An edge of a flow.
fn flow_edge(from: &str, to: &str, state: FlowState, label: Option<String>) -> FlowEdge {
    FlowEdge {
        from: from.to_owned(),
        to: to.to_owned(),
        state,
        label,
    }
}

/// A synthetic bridge as a flow: at rest, or with an unreachable CDR, a
/// The lanes of the synthetic bridge, the CDR, the terminology server and
/// the HL7 v2 face each in the state and line given.
fn bridge_lanes(
    v: &Visit,
    cdr: (FlowState, Msg),
    term: (FlowState, Msg),
    hl7: (FlowState, Msg),
) -> Vec<FlowLane> {
    let to_cdr = ["clients", "facade", "hub", "cdr"];
    let to_term = ["clients", "facade", "hub", "term"];
    let from_senders = ["senders", "hl7", "hub", "cdr"];
    let lane = |title: Msg, cards: Vec<FlowCard>| FlowLane {
        title: v.tr(title),
        cards,
        fold: None,
    };
    vec![
        lane(
            Msg::FlowCallers,
            vec![
                flow_card(
                    v,
                    "clients",
                    Glyph::USERS,
                    Msg::FlowFhirClients,
                    Msg::FlowSmart,
                    FlowState::Up,
                    &to_cdr,
                ),
                flow_card(
                    v,
                    "senders",
                    Glyph::RADIO_TOWER,
                    Msg::FlowHl7Senders,
                    Msg::FlowAccepted,
                    FlowState::Up,
                    &from_senders,
                ),
            ],
        ),
        lane(
            Msg::FlowFaces,
            vec![
                flow_card(
                    v,
                    "facade",
                    Glyph::NETWORK,
                    Msg::FlowFacade,
                    Msg::FlowFacadeOn,
                    FlowState::Up,
                    &to_cdr,
                ),
                flow_card(
                    v,
                    "hl7",
                    Glyph::ROUTE,
                    Msg::FlowHl7,
                    hl7.1,
                    hl7.0,
                    &from_senders,
                ),
            ],
        ),
        lane(
            Msg::FlowMappingSet,
            vec![flow_card(
                v,
                "hub",
                Glyph::LAYERS,
                Msg::FlowBridge,
                Msg::FlowContexts,
                FlowState::Up,
                &[],
            )],
        ),
        lane(
            Msg::FlowUpstreams,
            vec![
                flow_card(
                    v,
                    "cdr",
                    Glyph::DATABASE,
                    Msg::FlowCdr,
                    cdr.1,
                    cdr.0,
                    &to_cdr,
                ),
                flow_card(
                    v,
                    "term",
                    Glyph::BOOK_A,
                    Msg::FlowTerm,
                    term.1,
                    term.0,
                    &to_term,
                ),
            ],
        ),
    ]
}

/// degraded terminology server and the HL7 v2 face switched off.
pub(super) fn bridge_flow(v: &Visit, problems: bool) -> Html {
    let (cdr, cdr_line, term, term_line, hl7, hl7_line) = if problems {
        (
            FlowState::Unreachable,
            Msg::FlowNoAnswer,
            FlowState::Degraded,
            Msg::FlowAnswers503,
            FlowState::Off,
            Msg::FlowHl7Off,
        )
    } else {
        (
            FlowState::Up,
            Msg::FlowCdrUp,
            FlowState::Up,
            Msg::FlowTermUp,
            FlowState::Up,
            Msg::FlowHl7On,
        )
    };
    let lanes = bridge_lanes(v, (cdr, cdr_line), (term, term_line), (hl7, hl7_line));
    let edges = vec![
        flow_edge("clients", "facade", FlowState::Up, None),
        flow_edge("senders", "hl7", hl7, None),
        flow_edge("facade", "hub", FlowState::Up, None),
        flow_edge("hl7", "hub", hl7, None),
        flow_edge("hub", "cdr", cdr, Some(v.tr(Msg::FlowNoAnswerLabel))),
        flow_edge("hub", "term", term, Some(String::from("503"))),
    ];
    let (id, title, foot) = if problems {
        (
            "flow-problems",
            Msg::FlowTitleProblems,
            Msg::FlowFootProblems,
        )
    } else {
        ("flow-rest", Msg::FlowTitle, Msg::FlowFootSteady)
    };
    let drawn = flow(
        &v.locale,
        &Flow {
            id: id.to_owned(),
            label: v.tr(Msg::FlowBridgeAsFlow),
            list_label: v.tr(Msg::FlowBridgeLaneByLane),
            lanes,
            edges,
            foot: v.tr(foot),
        },
    );
    if problems {
        return titled_panel(v.tr(title), Html::empty(), drawn);
    }
    let rule = |state, meaning| EdgeRule {
        state,
        meaning: v.tr(meaning),
    };
    let help = rule_dialog(
        &v.locale,
        FlowRule {
            id: "flow-rule".into(),
            rules: vec![
                rule(FlowState::Up, Msg::FlowRuleUp),
                rule(FlowState::Degraded, Msg::FlowRuleDegraded),
                rule(FlowState::Unreachable, Msg::FlowRuleUnreachable),
                rule(FlowState::Off, Msg::FlowRuleOff),
            ],
            note: v.tr(Msg::FlowRuleNote),
        },
    );
    Html::join([
        titled_panel(v.tr(title), rule_button(&v.locale, "flow-rule"), drawn),
        help,
    ])
}

/// A card of the busy bridge, named by its kind and its number.
fn busy_card(
    v: &Visit,
    (id, glyph, kind, number): (String, Glyph, Msg, u8),
    (state, line): (FlowState, Msg),
    path: Vec<String>,
) -> FlowCard {
    FlowCard {
        href: format!("#{id}"),
        id,
        glyph,
        name: format!("{} {number:02}", v.tr(kind)),
        line: v.tr(line),
        state,
        path,
        hub: false,
    }
}

/// The thirty callers of the busy bridge, each with the faces it calls
/// and the label of its edges: most call the FHIR facade, every sixth is
/// an HL7 v2 sender, the third of every six calls the operator API, and
/// every fifth client calls the operator API too.
fn busy_callers(v: &Visit) -> Vec<(FlowCard, Vec<&'static str>, Option<String>)> {
    (1_u8..=30)
        .map(|number| {
            let faces = match (number % 6, number % 5) {
                (0, _) => vec!["hl7"],
                (3, _) => vec!["operator"],
                (_, 0) => vec!["facade", "operator"],
                _ => vec!["facade"],
            };
            let sender = number % 6 == 0;
            let (state, line, label) = match number {
                7 | 13 => (
                    FlowState::Degraded,
                    Msg::FlowBusyRefused,
                    Some(String::from("401")),
                ),
                18 => (
                    FlowState::Degraded,
                    Msg::FlowBusyRejected,
                    Some(v.tr(Msg::FlowBusyAe)),
                ),
                22 => (FlowState::Off, Msg::FlowBusyOff, None),
                26 => (FlowState::NotChecked, Msg::FlowBusyNotChecked, None),
                _ if sender => (FlowState::Up, Msg::FlowBusySending, None),
                _ => (FlowState::Up, Msg::FlowBusyCalling, None),
            };
            let id = format!("caller-{number:02}");
            let (glyph, kind) = if sender {
                (Glyph::RADIO_TOWER, Msg::FlowBusySender)
            } else {
                (Glyph::USERS, Msg::FlowBusyCaller)
            };
            let path = faces
                .first()
                .map(|face| vec![id.clone(), (*face).to_owned(), String::from("hub")])
                .unwrap_or_default();
            let card = busy_card(v, (id, glyph, kind, number), (state, line), path);
            (card, faces, label)
        })
        .collect()
}

/// The thirty upstreams of the busy bridge, each with the label of its
/// edge: CDRs, terminology servers and identity services in turn, two
/// degraded, one unreachable, one switched off and one not yet checked.
fn busy_upstreams(v: &Visit) -> Vec<(FlowCard, Option<String>)> {
    (1_u8..=30)
        .map(|number| {
            let (glyph, kind) = match number % 3 {
                1 => (Glyph::DATABASE, Msg::FlowCdr),
                2 => (Glyph::BOOK_A, Msg::FlowTerm),
                _ => (Glyph::SHIELD_CHECK, Msg::FlowBusyIdentity),
            };
            let (state, line, label) = match number {
                4 => (
                    FlowState::Unreachable,
                    Msg::FlowNoAnswer,
                    Some(v.tr(Msg::FlowNoAnswerLabel)),
                ),
                11 => (
                    FlowState::Degraded,
                    Msg::FlowAnswers503,
                    Some(String::from("503")),
                ),
                19 => (
                    FlowState::Degraded,
                    Msg::FlowBusySlow,
                    Some(v.tr(Msg::FlowBusySlowLabel)),
                ),
                23 => (FlowState::Off, Msg::FlowBusyOff, None),
                27 => (FlowState::NotChecked, Msg::FlowBusyNotChecked, None),
                _ => (FlowState::Up, Msg::FlowBusyUp, None),
            };
            let id = format!("upstream-{number:02}");
            let path = vec![String::from("hub"), id.clone()];
            let card = busy_card(v, (id, glyph, kind, number), (state, line), path);
            (card, label)
        })
        .collect()
}

/// A busy synthetic bridge: thirty callers through three faces into the
/// mapping set and out to thirty upstreams, some of each in a problem
/// state, its callers and upstreams folded unless `open` names them.
pub(super) fn busy_flow(v: &Visit, open: &[String]) -> Html {
    let fold = |lane: &str| {
        let unfolded = open.iter().any(|each| each == lane);
        let mut next: Vec<&str> = open
            .iter()
            .map(String::as_str)
            .filter(|each| *each != lane)
            .collect();
        if !unfolded {
            next.push(lane);
        }
        next.sort_unstable();
        let query = if next.is_empty() {
            String::from("?")
        } else {
            format!("?open={}", next.join(","))
        };
        Some(LaneFold {
            open: unfolded,
            toggle: format!("{query}#busy-flow"),
        })
    };
    let callers = busy_callers(v);
    let upstreams = busy_upstreams(v);
    let mut edges = Vec::new();
    for (card, faces, label) in &callers {
        for face in faces {
            edges.push(flow_edge(&card.id, face, card.state, label.clone()));
        }
    }
    for face in ["facade", "hl7", "operator"] {
        edges.push(flow_edge(face, "hub", FlowState::Up, None));
    }
    for (card, label) in &upstreams {
        edges.push(flow_edge("hub", &card.id, card.state, label.clone()));
    }
    let face = |id: &str, glyph: Glyph, name: Msg, line: Msg| {
        flow_card(v, id, glyph, name, line, FlowState::Up, &[])
    };
    let lanes = vec![
        FlowLane {
            title: v.tr(Msg::FlowCallers),
            cards: callers.into_iter().map(|(card, _, _)| card).collect(),
            fold: fold("callers"),
        },
        FlowLane {
            title: v.tr(Msg::FlowFaces),
            cards: vec![
                face("facade", Glyph::NETWORK, Msg::FlowFacade, Msg::FlowFacadeOn),
                face("hl7", Glyph::ROUTE, Msg::FlowHl7, Msg::FlowHl7On),
                face(
                    "operator",
                    Glyph::SETTINGS,
                    Msg::FlowBusyOperator,
                    Msg::FlowBusyOperatorOn,
                ),
            ],
            fold: None,
        },
        FlowLane {
            title: v.tr(Msg::FlowMappingSet),
            cards: vec![flow_card(
                v,
                "hub",
                Glyph::LAYERS,
                Msg::FlowBridge,
                Msg::FlowContexts,
                FlowState::Up,
                &[],
            )],
            fold: None,
        },
        FlowLane {
            title: v.tr(Msg::FlowUpstreams),
            cards: upstreams.into_iter().map(|(card, _)| card).collect(),
            fold: fold("upstreams"),
        },
    ];
    let drawn = flow(
        &v.locale,
        &Flow {
            id: String::from("flow-busy"),
            label: v.tr(Msg::FlowBusyLabel),
            list_label: v.tr(Msg::FlowBusyList),
            lanes,
            edges,
            foot: v.tr(Msg::FlowBusyFoot),
        },
    );
    Html::raw(format!(
        "<div id=\"busy-flow\">{}</div>",
        titled_panel(v.tr(Msg::FlowBusyTitle), Html::empty(), drawn)
    ))
}
