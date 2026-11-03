// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page's activity line and the ETL run reports.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::chart::{ActivityChart, activity_chart};
use ferrohealth_ui::view::funnel::{Bar, Funnel, FunnelPart, bar_list, funnel};
use ferrohealth_ui::view::surface::{Container, card};

use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The activity line of thirty synthetic days.
pub(super) fn activity(v: &Visit) -> Html {
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
pub(super) fn tiny_report(v: &Visit) -> Html {
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
pub(super) fn run_report(v: &Visit, refused: bool) -> Html {
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
    card(v.tr(title), Container::Stack.wrap(body))
}
