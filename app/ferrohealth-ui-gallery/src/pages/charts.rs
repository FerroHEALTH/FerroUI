// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page: the charts and the figures drawn as SVG.
//!
//! A line chart of three synthetic series over a week, its legend showing
//! and hiding each, the activity line of a dashboard, the ETL run report
//! with refusals and with none, the flow at rest and with problem edges,
//! and the score breakdown and its compact cell.

use crate::locale::{Msg, tr};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::chart::{
    ACCENT, ActivityChart, BAD, LineChart, NEUTRAL, Series, activity_chart,
};
use ferrohealth_ui::view::flow::{Flow, FlowCard, FlowEdge, FlowLane, FlowState, flow};
use ferrohealth_ui::view::funnel::{Bar, Funnel, FunnelPart, bar_list, funnel};
use ferrohealth_ui::view::score::{Contribution, Score, Threshold, score_bar, score_breakdown};
use ferrohealth_ui::view::surface::{card, definitions, page_header, stack, titled_panel};
use ferrohealth_ui::widget::chart::{ChartWords, line_chart};
use leptos::prelude::*;
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
fn flow_edge(from: &str, to: &str, state: FlowState, label: Option<Msg>) -> FlowEdge {
    FlowEdge {
        from: from.to_owned(),
        to: to.to_owned(),
        state,
        label: label.map(tr),
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
        flow_edge("hub", "cdr", cdr, Some(Msg::FlowNoAnswerLabel)),
        flow_edge("hub", "term", term, Some(Msg::FlowAnswers503)),
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
    let flows = (bridge_flow(false), bridge_flow(true));
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
                {scores.0}
                {scores.1}
                {scores.2}
            },
        )}
    }
}
