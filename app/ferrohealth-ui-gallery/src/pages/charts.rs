// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page: the charts and the figures drawn as SVG.
//!
//! A line chart of three synthetic series over a week, its legend showing
//! and hiding each, the activity line of a dashboard, the ETL run report
//! with refusals and with none, the flow at rest and with problem edges,
//! a busy bridge of thirty callers and thirty upstreams as a flow whose
//! lanes fold, and the score breakdown and its compact cell. The query's
//! `open` names the busy flow's lanes unfolded (`callers`, `upstreams`).

use crate::locale::{Msg, tr};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::chart::{
    ACCENT, ActivityChart, BAD, LineChart, NEUTRAL, Series, activity_chart,
};
use ferrohealth_ui::view::flow::{Flow, FlowCard, FlowEdge, FlowLane, FlowState, LaneFold, flow};
use ferrohealth_ui::view::funnel::{Bar, Funnel, FunnelPart, bar_list, funnel};
use ferrohealth_ui::view::score::{Contribution, Score, Threshold, score_bar, score_breakdown};
use ferrohealth_ui::view::surface::{card, definitions, page_header, stack, titled_panel};
use ferrohealth_ui::widget::chart::{ChartWords, line_chart};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

/// The days of the synthetic week, as the X axis labels them.
fn days() -> [String; 7] {
    [
        Msg::Mon,
        Msg::Tue,
        Msg::Wed,
        Msg::Thu,
        Msg::Fri,
        Msg::Sat,
        Msg::Sun,
    ]
    .map(tr)
}

/// The line chart of three synthetic series, one with a gap, its legend
/// the library's island.
fn lines() -> AnyView {
    card(tr(Msg::LineChart), requests_chart(RequestsWords::of_page())).into_any()
}

/// The words the line chart draws, resolved on the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestsWords {
    label: String,
    summary: String,
    reads: String,
    writes: String,
    refusals: String,
    days: [String; 7],
    chart: ChartWords,
}

impl RequestsWords {
    /// The words in the page's locale.
    fn of_page() -> Self {
        Self {
            label: tr(Msg::RequestsPerDay),
            summary: tr(Msg::ThreeSyntheticSeriesOverWeek),
            reads: tr(Msg::Reads),
            writes: tr(Msg::Writes),
            refusals: tr(Msg::Refusals),
            days: days(),
            chart: ChartWords::of_page(),
        }
    }
}

/// The line chart of three synthetic series in `words`, its legend showing
/// and hiding each.
fn requests_chart(words: RequestsWords) -> AnyView {
    let RequestsWords {
        label,
        summary,
        reads,
        writes,
        refusals,
        days,
        chart: chart_words,
    } = words;
    let series = |name: &str, palette: u8, values: [f64; 7]| Series {
        name: name.to_owned(),
        points: (0_u8..7)
            .zip(values)
            .map(|(day, value)| (f64::from(day), value))
            .collect(),
        palette,
    };
    let chart = LineChart {
        id: "requests",
        label,
        summary,
        series: vec![
            series(&reads, ACCENT, [22.0, 30.0, 27.0, 35.0, 48.0, 31.0, 26.0]),
            series(&writes, NEUTRAL, [10.0, 12.0, 14.0, 11.0, 13.0, 9.0, 12.0]),
            series(&refusals, BAD, [3.0, 5.0, f64::NAN, 4.0, 6.0, 2.0, 1.0]),
        ],
        x_ticks: (0_u8..7)
            .zip(days)
            .map(|(day, label)| (f64::from(day), label))
            .collect(),
        y_range: None,
    };
    line_chart(&chart, chart_words)
}

/// The activity line of thirty synthetic days.
fn activity() -> AnyView {
    let counts = [
        0, 1, 0, 2, 3, 1, 0, 0, 4, 2, 1, 0, 3, 5, 2, 0, 1, 1, 0, 2, 6, 3, 1, 0, 0, 2, 4, 1, 0, 2,
    ];
    let points = (0_u32..).zip(counts).collect();
    card(
        tr(Msg::Activity),
        activity_chart(&ActivityChart {
            name: "commits",
            label: tr(Msg::CommitsPerDay),
            summary: tr(Msg::N48SyntheticCommitsOver30),
            points,
            ends: (tr(Msg::Sep19), tr(Msg::Oct18)),
        }),
    )
    .into_any()
}

/// One part of a run report's stacked bar.
fn part(label: Msg, count: u64, figure: &str, tone: Tone) -> FunnelPart {
    FunnelPart {
        label: tr(label),
        count,
        figure: figure.to_owned(),
        tone,
    }
}

/// One row of a run report's bar list.
fn table_row(name: &str, count: u64, figure: &str) -> Bar {
    Bar {
        name: name.to_owned(),
        count,
        figure: figure.to_owned(),
    }
}

/// The run report of a synthetic ETL run: the stacked bar of where the
/// compositions went, with refusals or with none, and the rows per table.
fn run_report(refused: bool) -> AnyView {
    let (figure, label, parts) = if refused {
        (
            "1,240",
            tr(Msg::RunSentence),
            vec![
                part(Msg::RunWritten, 1180, "1,180", Tone::Ok),
                part(Msg::RunSkipped, 52, "52", Tone::Neutral),
                part(Msg::RunRefused, 8, "8", Tone::Bad),
            ],
        )
    } else {
        (
            "860",
            tr(Msg::RunCleanSentence),
            vec![
                part(Msg::RunWritten, 812, "812", Tone::Ok),
                part(Msg::RunSkipped, 48, "48", Tone::Neutral),
                part(Msg::RunRefused, 0, "0", Tone::Bad),
            ],
        )
    };
    let bars = if refused {
        vec![
            table_row("person", 1180, "1,180"),
            table_row("visit_occurrence", 2410, "2,410"),
            table_row("condition_occurrence", 3022, "3,022"),
            table_row("measurement", 9874, "9,874"),
            table_row("observation", 640, "640"),
        ]
    } else {
        vec![
            table_row("person", 812, "812"),
            table_row("visit_occurrence", 1530, "1,530"),
            table_row("measurement", 6120, "6,120"),
        ]
    };
    let title = if refused {
        Msg::RunReport
    } else {
        Msg::RunReportClean
    };
    let body = view! {
        {funnel(&Funnel {
            figure: figure.to_owned(),
            words: tr(Msg::RunCompositionsRead),
            label,
            parts,
        })}
        <h3 class="fui-small fui-muted">{tr(Msg::RunRowsPerTable)}</h3>
        {bar_list(&tr(Msg::RunBarlistLabel), &bars)}
    };
    card(tr(title), stack(body)).into_any()
}

/// A card of a flow.
fn flow_card(
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
        name: tr(name),
        line: tr(line),
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
    cdr: (FlowState, Msg),
    term: (FlowState, Msg),
    hl7: (FlowState, Msg),
) -> Vec<FlowLane> {
    let to_cdr = ["clients", "facade", "hub", "cdr"];
    let to_term = ["clients", "facade", "hub", "term"];
    let from_senders = ["senders", "hl7", "hub", "cdr"];
    let lane = |title: Msg, cards: Vec<FlowCard>| FlowLane {
        title: tr(title),
        cards,
        fold: None,
    };
    vec![
        lane(
            Msg::FlowCallers,
            vec![
                flow_card(
                    "clients",
                    Glyph::USERS,
                    Msg::FlowFhirClients,
                    Msg::FlowSmart,
                    FlowState::Up,
                    &to_cdr,
                ),
                flow_card(
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
                    "facade",
                    Glyph::NETWORK,
                    Msg::FlowFacade,
                    Msg::FlowFacadeOn,
                    FlowState::Up,
                    &to_cdr,
                ),
                flow_card(
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
                flow_card("cdr", Glyph::DATABASE, Msg::FlowCdr, cdr.1, cdr.0, &to_cdr),
                flow_card(
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
fn bridge_flow(problems: bool) -> AnyView {
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
    let lanes = bridge_lanes((cdr, cdr_line), (term, term_line), (hl7, hl7_line));
    let edges = vec![
        flow_edge("clients", "facade", FlowState::Up, None),
        flow_edge("senders", "hl7", hl7, None),
        flow_edge("facade", "hub", FlowState::Up, None),
        flow_edge("hl7", "hub", hl7, None),
        flow_edge("hub", "cdr", cdr, Some(tr(Msg::FlowNoAnswerLabel))),
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
    let drawn = flow(&Flow {
        id: id.to_owned(),
        label: tr(Msg::FlowBridgeAsFlow),
        list_label: tr(Msg::FlowBridgeLaneByLane),
        lanes,
        edges,
        foot: tr(foot),
    });
    titled_panel(tr(title), (), drawn).into_any()
}

/// A card of the busy bridge, named by its kind and its number.
fn busy_card(
    (id, glyph, kind, number): (String, Glyph, Msg, u8),
    (state, line): (FlowState, Msg),
    path: Vec<String>,
) -> FlowCard {
    FlowCard {
        href: format!("#{id}"),
        id,
        glyph,
        name: format!("{} {number:02}", tr(kind)),
        line: tr(line),
        state,
        path,
        hub: false,
    }
}

/// The thirty callers of the busy bridge, each with the faces it calls
/// and the label of its edges: most call the FHIR facade, every sixth is
/// an HL7 v2 sender, the third of every six calls the operator API, and
/// every fifth client calls the operator API too.
fn busy_callers() -> Vec<(FlowCard, Vec<&'static str>, Option<String>)> {
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
                    Some(tr(Msg::FlowBusyAe)),
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
            let card = busy_card((id, glyph, kind, number), (state, line), path);
            (card, faces, label)
        })
        .collect()
}

/// The thirty upstreams of the busy bridge, each with the label of its
/// edge: CDRs, terminology servers and identity services in turn, two
/// degraded, one unreachable, one switched off and one not yet checked.
fn busy_upstreams() -> Vec<(FlowCard, Option<String>)> {
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
                    Some(tr(Msg::FlowNoAnswerLabel)),
                ),
                11 => (
                    FlowState::Degraded,
                    Msg::FlowAnswers503,
                    Some(String::from("503")),
                ),
                19 => (
                    FlowState::Degraded,
                    Msg::FlowBusySlow,
                    Some(tr(Msg::FlowBusySlowLabel)),
                ),
                23 => (FlowState::Off, Msg::FlowBusyOff, None),
                27 => (FlowState::NotChecked, Msg::FlowBusyNotChecked, None),
                _ => (FlowState::Up, Msg::FlowBusyUp, None),
            };
            let id = format!("upstream-{number:02}");
            let path = vec![String::from("hub"), id.clone()];
            let card = busy_card((id, glyph, kind, number), (state, line), path);
            (card, label)
        })
        .collect()
}

/// A busy synthetic bridge: thirty callers through three faces into the
/// mapping set and out to thirty upstreams, some of each in a problem
/// state, its callers and upstreams folded unless `open` names them.
fn busy_flow(open: &[String]) -> AnyView {
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
    let callers = busy_callers();
    let upstreams = busy_upstreams();
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
        flow_card(id, glyph, name, line, FlowState::Up, &[])
    };
    let lanes = vec![
        FlowLane {
            title: tr(Msg::FlowCallers),
            cards: callers.into_iter().map(|(card, _, _)| card).collect(),
            fold: fold("callers"),
        },
        FlowLane {
            title: tr(Msg::FlowFaces),
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
            title: tr(Msg::FlowMappingSet),
            cards: vec![flow_card(
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
            title: tr(Msg::FlowUpstreams),
            cards: upstreams.into_iter().map(|(card, _)| card).collect(),
            fold: fold("upstreams"),
        },
    ];
    let drawn = flow(&Flow {
        id: String::from("flow-busy"),
        label: tr(Msg::FlowBusyLabel),
        list_label: tr(Msg::FlowBusyList),
        lanes,
        edges,
        foot: tr(Msg::FlowBusyFoot),
    });
    view! { <div id="busy-flow">{titled_panel(tr(Msg::FlowBusyTitle), (), drawn)}</div> }.into_any()
}

/// One contribution of a synthetic score.
fn contribution(label: Msg, reading: Msg, weight: f64) -> Contribution {
    Contribution {
        label: tr(label),
        reading: Some(tr(reading)),
        weight,
    }
}

/// The thresholds of the synthetic scores.
fn thresholds() -> Vec<Threshold> {
    [
        (Msg::ScorePossible, 0.60),
        (Msg::ScoreProbable, 0.80),
        (Msg::ScoreCertain, 0.95),
    ]
    .map(|(label, at)| Threshold {
        label: tr(label),
        at,
    })
    .to_vec()
}

/// A synthetic score breakdown: between two thresholds, or at one, with a
/// positive, a negative and a zero weight.
fn breakdown(at_threshold: bool) -> AnyView {
    let score = if at_threshold {
        Score {
            total: 1.4,
            value: 0.60,
            grade: tr(Msg::ScorePossible),
            contributions: vec![
                contribution(Msg::ScoreBirthDate, Msg::ScoreExact, 3.1),
                contribution(Msg::ScoreGivenName, Msg::ScoreJaroFar, -1.7),
                contribution(Msg::ScorePostalCode, Msg::ScoreMissing, 0.0),
            ],
            thresholds: thresholds(),
        }
    } else {
        Score {
            total: 4.1,
            value: 0.87,
            grade: tr(Msg::ScoreProbable),
            contributions: vec![
                contribution(Msg::ScoreBirthDate, Msg::ScoreExact, 3.1),
                contribution(Msg::ScoreFamilyName, Msg::ScoreJaroNear, 2.4),
                contribution(Msg::ScoreSex, Msg::ScoreExact, 0.3),
                contribution(Msg::ScorePostalCode, Msg::ScoreMissing, 0.0),
                contribution(Msg::ScoreGivenName, Msg::ScoreJaroFar, -1.7),
            ],
            thresholds: thresholds(),
        }
    };
    let title = if at_threshold {
        Msg::ScoreAtThreshold
    } else {
        Msg::ScoreBreakdown
    };
    card(tr(title), score_breakdown(&score)).into_any()
}

/// The compact score of a table cell, once per grade.
fn score_cells() -> AnyView {
    card(
        tr(Msg::ScoreCells),
        definitions(vec![
            (tr(Msg::ScoreCertain), score_bar(0.97, Tone::Ok)),
            (tr(Msg::ScoreProbable), score_bar(0.87, Tone::Info)),
            (tr(Msg::ScorePossible), score_bar(0.64, Tone::Warn)),
        ]),
    )
    .into_any()
}

/// The charts page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn ChartsPage() -> impl IntoView {
    let header = page_header(
        tr(Msg::Charts),
        Some(tr(Msg::DrawnSvgServerScaledCard)),
        ActionGroup::none(),
    )
    .into_any();
    let sections = (lines(), activity());
    let reports = (run_report(true), run_report(false));
    let open: Vec<String> = use_query_map().with_untracked(|query| {
        query
            .get_str("open")
            .unwrap_or_default()
            .split(',')
            .filter(|lane| !lane.is_empty())
            .map(str::to_owned)
            .collect()
    });
    let flows = (bridge_flow(false), bridge_flow(true), busy_flow(&open));
    let scores = (breakdown(false), breakdown(true), score_cells());
    view! {
        {super::title(tr(Msg::Charts))}
        {header}
        {stack(
            view! {
                {sections.0}
                {sections.1}
                {reports.0}
                {reports.1}
                {flows.0}
                {flows.1}
                {flows.2}
                {scores.0}
                {scores.1}
                {scores.2}
            },
        )}
    }
}
