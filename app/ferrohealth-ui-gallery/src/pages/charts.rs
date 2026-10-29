// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page: the charts and the figures drawn as SVG.
//!
//! A line chart of three synthetic series over a week, its legend showing
//! and hiding each, and one of eight lanes of small values over twelve
//! days; the activity line of a dashboard, the ETL run report with
//! refusals, with none and with a tiny part, the flow at rest and with
//! problem edges, a busy bridge of thirty callers and thirty upstreams as
//! a flow whose lanes fold, and the score breakdown, among close
//! thresholds too, and its compact cell. The query's `open` names the busy
//! flow's lanes unfolded (`callers`, `upstreams`).

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::chart::{
    ACCENT, ActivityChart, BAD, LineChart, NEUTRAL, Series, activity_chart,
};
use ferrohealth_ui::view::chart::{ChartWords, line_chart};
use ferrohealth_ui::view::flow::{
    EdgeRule, Flow, FlowCard, FlowEdge, FlowLane, FlowRule, FlowState, LaneFold, flow, rule_button,
    rule_dialog,
};
use ferrohealth_ui::view::funnel::{Bar, Funnel, FunnelPart, bar_list, funnel};
use ferrohealth_ui::view::score::{Contribution, Score, Threshold, score_bar, score_breakdown};
use ferrohealth_ui::view::surface::{card, definitions, page_header, stack, titled_panel};

use crate::app::Page;
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The days of the synthetic week, as the X axis labels them.
fn days(v: &Visit) -> [String; 7] {
    [
        Msg::Mon,
        Msg::Tue,
        Msg::Wed,
        Msg::Thu,
        Msg::Fri,
        Msg::Sat,
        Msg::Sun,
    ]
    .map(|msg| v.tr(msg))
}

/// The line chart of three synthetic series, one with a gap, its legend
/// the library's island.
fn lines(v: &Visit) -> Html {
    card(v.tr(Msg::LineChart), requests_chart(RequestsWords::of(v)))
}

/// The words the line chart draws, resolved on the server.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestsWords {
    label: String,
    summary: String,
    reads: String,
    writes: String,
    refusals: String,
    days: [String; 7],
    chart: ChartWords,
}

impl RequestsWords {
    /// The words in the visit's locale.
    fn of(v: &Visit) -> Self {
        Self {
            label: v.tr(Msg::RequestsPerDay),
            summary: v.tr(Msg::ThreeSyntheticSeriesOverWeek),
            reads: v.tr(Msg::Reads),
            writes: v.tr(Msg::Writes),
            refusals: v.tr(Msg::Refusals),
            days: days(v),
            chart: ChartWords::of(&v.locale),
        }
    }
}

/// The line chart of three synthetic series in `words`, its legend showing
/// and hiding each.
fn requests_chart(words: RequestsWords) -> Html {
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
        x_labels: Vec::new(),
        y_range: None,
    };
    line_chart(&chart, chart_words)
}

/// The line chart of eight synthetic lanes of small values over twelve
/// days: past the six colours a lane takes its own mark, the value axis
/// keeps the decimals of its step, and a narrow card thins the dates.
fn many_lines(v: &Visit) -> Html {
    let series = (0_u8..8)
        .map(|lane| Series {
            name: v.tr_with(Msg::ManyLane, &[("index", (lane + 1).to_string().into())]),
            points: (0_u8..12)
                .map(|day| {
                    (
                        f64::from(day),
                        (f64::from(lane) + 1.0 + f64::from(day % 4) * 0.25) * 0.0001,
                    )
                })
                .collect(),
            palette: lane,
        })
        .collect();
    let chart = LineChart {
        id: "lanes",
        label: v.tr(Msg::ManySeriesLabel),
        summary: v.tr(Msg::ManySeriesSummary),
        series,
        x_ticks: (0_u8..12)
            .map(|day| (f64::from(day), format!("2026-10-{:02}", day + 1)))
            .collect(),
        x_labels: Vec::new(),
        y_range: None,
    };
    card(
        v.tr(Msg::LineChartMany),
        line_chart(&chart, ChartWords::of(&v.locale)),
    )
}

/// The activity line of thirty synthetic days.
fn activity(v: &Visit) -> Html {
    let counts = [
        0, 1, 0, 2, 3, 1, 0, 0, 4, 2, 1, 0, 3, 5, 2, 0, 1, 1, 0, 2, 6, 3, 1, 0, 0, 2, 4, 1, 0, 2,
    ];
    let points = (0_u32..).zip(counts).collect();
    card(
        v.tr(Msg::Activity),
        activity_chart(&ActivityChart {
            name: "commits",
            label: v.tr(Msg::CommitsPerDay),
            summary: v.tr(Msg::N48SyntheticCommitsOver30),
            points,
            ticks: vec![
                (0, v.tr(Msg::Sep19)),
                (9, v.tr(Msg::Sep28)),
                (19, v.tr(Msg::Oct8)),
                (29, v.tr(Msg::Oct18)),
            ],
        }),
    )
}

/// The run report of a run that refused 3 compositions of two million: the
/// refused part keeps a width that shows.
fn tiny_report(v: &Visit) -> Html {
    let body = funnel(&Funnel {
        figure: String::from("2,000,003"),
        words: v.tr(Msg::RunCompositionsRead),
        label: v.tr(Msg::RunTinySentence),
        parts: vec![
            part(v, Msg::RunWritten, 2_000_000, "2,000,000", Tone::Ok),
            part(v, Msg::RunSkipped, 0, "0", Tone::Neutral),
            part(v, Msg::RunRefused, 3, "3", Tone::Bad),
        ],
    });
    card(v.tr(Msg::RunReportTiny), body)
}

/// One part of a run report's stacked bar.
fn part(v: &Visit, label: Msg, count: u64, figure: &str, tone: Tone) -> FunnelPart {
    FunnelPart {
        label: v.tr(label),
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
fn run_report(v: &Visit, refused: bool) -> Html {
    let (figure, label, parts) = if refused {
        (
            "1,240",
            v.tr(Msg::RunSentence),
            vec![
                part(v, Msg::RunWritten, 1180, "1,180", Tone::Ok),
                part(v, Msg::RunSkipped, 52, "52", Tone::Neutral),
                part(v, Msg::RunRefused, 8, "8", Tone::Bad),
            ],
        )
    } else {
        (
            "860",
            v.tr(Msg::RunCleanSentence),
            vec![
                part(v, Msg::RunWritten, 812, "812", Tone::Ok),
                part(v, Msg::RunSkipped, 48, "48", Tone::Neutral),
                part(v, Msg::RunRefused, 0, "0", Tone::Bad),
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
    let body = Html::join([
        funnel(&Funnel {
            figure: figure.to_owned(),
            words: v.tr(Msg::RunCompositionsRead),
            label,
            parts,
        }),
        h(format!(
            "<h3 class=\"fui-small fui-muted\">{}</h3>",
            t(v.tr(Msg::RunRowsPerTable))
        )),
        bar_list(&v.tr(Msg::RunBarlistLabel), &bars),
    ]);
    card(v.tr(title), stack(body))
}

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
fn bridge_flow(v: &Visit, problems: bool) -> Html {
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
fn busy_flow(v: &Visit, open: &[String]) -> Html {
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
    h(format!(
        "<div id=\"busy-flow\">{}</div>",
        titled_panel(v.tr(Msg::FlowBusyTitle), Html::empty(), drawn)
    ))
}

/// One contribution of a synthetic score.
fn contribution(v: &Visit, label: Msg, reading: Msg, weight: f64) -> Contribution {
    Contribution {
        label: v.tr(label),
        reading: Some(v.tr(reading)),
        weight,
    }
}

/// The thresholds of the synthetic scores.
fn thresholds(v: &Visit) -> Vec<Threshold> {
    [
        (Msg::ScorePossible, 0.60),
        (Msg::ScoreProbable, 0.80),
        (Msg::ScoreCertain, 0.95),
    ]
    .map(|(label, at)| Threshold {
        label: v.tr(label),
        at,
    })
    .to_vec()
}

/// A synthetic score breakdown: between two thresholds, or at one, with a
/// positive, a negative and a zero weight.
fn breakdown(v: &Visit, at_threshold: bool) -> Html {
    let score = if at_threshold {
        Score {
            total: 1.4,
            value: 0.60,
            grade: v.tr(Msg::ScorePossible),
            contributions: vec![
                contribution(v, Msg::ScoreBirthDate, Msg::ScoreExact, 3.1),
                contribution(v, Msg::ScoreGivenName, Msg::ScoreJaroFar, -1.7),
                contribution(v, Msg::ScorePostalCode, Msg::ScoreMissing, 0.0),
            ],
            thresholds: thresholds(v),
        }
    } else {
        Score {
            total: 4.1,
            value: 0.87,
            grade: v.tr(Msg::ScoreProbable),
            contributions: vec![
                contribution(v, Msg::ScoreBirthDate, Msg::ScoreExact, 3.1),
                contribution(v, Msg::ScoreFamilyName, Msg::ScoreJaroNear, 2.4),
                contribution(v, Msg::ScoreSex, Msg::ScoreExact, 0.3),
                contribution(v, Msg::ScorePostalCode, Msg::ScoreMissing, 0.0),
                contribution(v, Msg::ScoreGivenName, Msg::ScoreJaroFar, -1.7),
            ],
            thresholds: thresholds(v),
        }
    };
    let title = if at_threshold {
        Msg::ScoreAtThreshold
    } else {
        Msg::ScoreBreakdown
    };
    card(v.tr(title), score_breakdown(&v.locale, &score))
}

/// A score among four close thresholds, its field's long word wrapping:
/// the labels too close for one row take more.
fn close_breakdown(v: &Visit) -> Html {
    let thresholds = [
        (Msg::ScorePossible, 0.80),
        (Msg::ScoreProbable, 0.83),
        (Msg::ScoreCertain, 0.86),
        (Msg::ScoreExactBand, 0.99),
    ]
    .map(|(label, at)| Threshold {
        label: v.tr(label),
        at,
    })
    .to_vec();
    let score = Score {
        total: 2.9,
        value: 0.84,
        grade: v.tr(Msg::ScoreProbable),
        contributions: vec![
            contribution(v, Msg::ScoreLongField, Msg::ScoreExact, 2.4),
            contribution(v, Msg::ScoreSex, Msg::ScoreExact, 0.5),
        ],
        thresholds,
    };
    card(v.tr(Msg::ScoreClose), score_breakdown(&v.locale, &score))
}

/// The compact score of a table cell, once per grade.
fn score_cells(v: &Visit) -> Html {
    card(
        v.tr(Msg::ScoreCells),
        definitions(vec![
            (
                v.tr(Msg::ScoreCertain),
                score_bar(&v.locale, 0.97, Tone::Ok),
            ),
            (
                v.tr(Msg::ScoreProbable),
                score_bar(&v.locale, 0.87, Tone::Info),
            ),
            (
                v.tr(Msg::ScorePossible),
                score_bar(&v.locale, 0.64, Tone::Warn),
            ),
        ]),
    )
}

/// The charts page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let header = page_header(
        &v.locale,
        v.tr(Msg::Charts),
        Some(v.tr(Msg::DrawnSvgServerScaledCard)),
        ActionGroup::none(),
    );
    let open: Vec<String> = v
        .param("open")
        .unwrap_or_default()
        .split(',')
        .filter(|lane| !lane.is_empty())
        .map(str::to_owned)
        .collect();
    let body = stack(Html::join([
        lines(v),
        many_lines(v),
        activity(v),
        run_report(v, true),
        run_report(v, false),
        tiny_report(v),
        bridge_flow(v, false),
        bridge_flow(v, true),
        busy_flow(v, &open),
        breakdown(v, false),
        breakdown(v, true),
        close_breakdown(v),
        score_cells(v),
    ]));
    Page::shell(v.tr(Msg::Charts), Html::join([header, body]))
}
