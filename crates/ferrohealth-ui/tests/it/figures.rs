// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The figures drawn as SVG on the server: the ETL run report's stacked bar
//! and bar list, the flow, and the score breakdown with its compact cell.
//! Each names what it draws for assistive technology (WCAG 2.2 SC 1.1.1),
//! prints every number beside its bar so nothing rests on colour (SC
//! 1.4.1), and writes no style attribute.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::flow::{
    FOLD_PAST, Flow, FlowCard, FlowEdge, FlowLane, FlowState, LaneFold, flow,
};
use ferrohealth_ui::view::funnel::{Bar, Funnel, FunnelPart, bar_list, funnel};
use ferrohealth_ui::view::score::{Contribution, Score, Threshold, score_bar, score_breakdown};

use crate::render::html;

fn part(label: &str, count: u64, tone: Tone) -> FunnelPart {
    FunnelPart {
        label: label.to_owned(),
        count,
        figure: count.to_string(),
        tone,
    }
}

fn run(refused: u64) -> Funnel {
    Funnel {
        figure: String::from("100"),
        words: String::from("compositions read"),
        label: format!("Of 100 read: 80 written, 20 skipped, {refused} refused <now>"),
        parts: vec![
            part("Written", 80, Tone::Ok),
            part("Skipped", 20 - refused, Tone::Neutral),
            part("Refused", refused, Tone::Bad),
        ],
    }
}

#[test]
fn the_stacked_bar_is_an_image_named_by_its_sentence_and_the_legend_prints_each_figure() {
    let shown = html(|| funnel(&run(5)));
    assert!(shown.contains(r#"role="img""#), "{shown}");
    assert!(
        shown.contains("aria-label=\"Of 100 read: 80 written, 20 skipped, 5 refused &lt;now&gt;\""),
        "the sentence is the name, escaped: {shown}"
    );
    assert_eq!(3, shown.matches("<rect").count(), "{shown}");
    assert!(shown.contains(r#"x="80.00%""#), "{shown}");
    assert_eq!(2, shown.matches("fui-funnel-cut").count(), "{shown}");
    for figure in ["<dd>80</dd>", "<dd>5</dd>"] {
        assert!(shown.contains(figure), "{figure}: {shown}");
    }
    assert!(!shown.contains("style="), "{shown}");
}

#[test]
fn a_run_with_nothing_refused_draws_no_segment_and_keeps_its_zero_in_the_legend() {
    let shown = html(|| funnel(&run(0)));
    assert_eq!(2, shown.matches("<rect").count(), "{shown}");
    assert!(!shown.contains("fui-funnel-part fui-tone-bad"), "{shown}");
    assert!(shown.contains("fui-funnel-swatch fui-tone-bad"), "{shown}");
    assert!(shown.contains("<dd>0</dd>"), "{shown}");
}

#[test]
fn a_bar_list_measures_each_row_against_the_largest_and_prints_its_number() {
    let bars = [("person", 50), ("measurement", 200)].map(|(name, count)| Bar {
        name: name.to_owned(),
        count,
        figure: count.to_string(),
    });
    let shown = html(|| bar_list("Rows per table", &bars));
    assert!(shown.contains(r#"aria-label="Rows per table""#), "{shown}");
    assert!(shown.contains(r#"width="25.0%""#), "{shown}");
    assert!(shown.contains(r#"width="100.0%""#), "{shown}");
    assert_eq!(2, shown.matches(r#"aria-hidden="true""#).count(), "{shown}");
    assert!(shown.contains(">200</span>"), "{shown}");
    assert!(!shown.contains("style="), "{shown}");
}

fn card(id: &str, state: FlowState, path: &[&str]) -> FlowCard {
    FlowCard {
        id: id.to_owned(),
        glyph: Glyph::SERVER,
        name: format!("Card {id}"),
        line: format!("Line of {id}"),
        state,
        href: format!("/{id}"),
        path: path.iter().map(|step| (*step).to_owned()).collect(),
        hub: id == "hub",
    }
}

fn bridge(cdr: FlowState, term: FlowState) -> Flow {
    let path = ["caller", "hub", "cdr"];
    Flow {
        id: String::from("bridge"),
        label: String::from("The bridge as a flow"),
        list_label: String::from("The bridge, lane by lane"),
        lanes: vec![
            FlowLane {
                title: String::from("Callers"),
                cards: vec![card("caller", FlowState::Up, &path)],
                fold: None,
            },
            FlowLane {
                title: String::from("Mapping set"),
                cards: vec![card("hub", FlowState::Up, &[])],
                fold: None,
            },
            FlowLane {
                title: String::from("Upstreams"),
                cards: vec![card("cdr", cdr, &path), card("term", term, &[])],
                fold: None,
            },
        ],
        edges: vec![
            FlowEdge {
                from: String::from("caller"),
                to: String::from("hub"),
                state: FlowState::Up,
                label: Some(String::from("never shown")),
            },
            FlowEdge {
                from: String::from("hub"),
                to: String::from("cdr"),
                state: cdr,
                label: Some(String::from("no answer")),
            },
            FlowEdge {
                from: String::from("hub"),
                to: String::from("term"),
                state: term,
                label: Some(String::from("503")),
            },
        ],
        foot: String::from("1 face on."),
    }
}

#[test]
fn a_flow_draws_each_state_as_a_line_style_and_marks_and_labels_only_its_problem_edges() {
    let shown = html(|| flow(&bridge(FlowState::Unreachable, FlowState::Degraded)));
    for class in [
        "fui-edge fui-edge-ok",
        "fui-edge fui-edge-bad",
        "fui-edge fui-edge-warn",
    ] {
        assert!(shown.contains(class), "{class}: {shown}");
    }
    assert!(shown.contains("fui-edge-mark fui-edge-mark-bad"), "{shown}");
    assert!(
        shown.contains("fui-edge-mark fui-edge-mark-warn"),
        "{shown}"
    );
    assert!(
        shown.contains(">no answer</text>") && shown.contains(">503</text>"),
        "{shown}"
    );
    assert!(
        !shown.contains("never shown"),
        "an edge that is up carries no label: {shown}"
    );
    assert!(!shown.contains("style="), "{shown}");
}

#[test]
fn the_flow_drawing_is_an_image_whose_description_names_each_problem_edge() {
    let shown = html(|| flow(&bridge(FlowState::Unreachable, FlowState::Up)));
    assert!(shown.contains(r#"role="img""#), "{shown}");
    assert!(
        shown.contains(r#"aria-labelledby="bridge-title bridge-desc""#),
        "{shown}"
    );
    assert!(
        shown.contains(
            "<desc id=\"bridge-desc\">Card hub to Card cdr: Unreachable, no answer.</desc>"
        ),
        "{shown}"
    );
    let steady = html(|| flow(&bridge(FlowState::Up, FlowState::Up)));
    assert!(
        steady.contains("No edge is degraded or unreachable."),
        "{steady}"
    );
}

#[test]
fn every_flow_card_is_a_named_link_with_its_path_and_pulse_beside_it() {
    let shown = html(|| flow(&bridge(FlowState::Up, FlowState::Up)));
    assert!(shown.contains(r#"href="/cdr""#), "{shown}");
    assert!(
        shown.contains(r#"aria-label="Card cdr, Line of cdr: Up. Opens its page.""#),
        "{shown}"
    );
    assert!(shown.contains("fui-tnode fui-tnode-gateway"), "{shown}");
    assert_eq!(
        2,
        shown.matches(r#"class="fui-pipeline-path""#).count(),
        "the caller and the CDR carry a path, the hub and the terminology server none: {shown}"
    );
    assert!(shown.contains(r#"class="fui-path-pulse""#), "{shown}");
    assert!(shown.contains(r#"pathLength="100""#), "{shown}");
}

#[test]
fn the_phone_list_holds_a_heading_per_lane_and_a_row_per_card_with_its_state_in_words() {
    let shown = html(|| flow(&bridge(FlowState::Unreachable, FlowState::Off)));
    assert!(shown.contains(r#"class="fui-pipeline-list""#), "{shown}");
    assert_eq!(3, shown.matches("fui-pipeline-lane").count(), "{shown}");
    assert_eq!(4, shown.matches("fui-pipeline-name").count(), "{shown}");
    assert!(
        shown.contains(
            r#"role="img" aria-label="Unreachable" title="Unreachable" class="fui-dot fui-tone-bad""#
        ),
        "{shown}"
    );
    assert!(
        shown.contains("fui-dot fui-tone-neutral is-muted"),
        "{shown}"
    );
    assert!(
        shown.contains(r#"<p class="fui-pipeline-foot">1 face on.</p>"#),
        "{shown}"
    );
}

/// A bridge whose callers lane holds twelve callers into the hub, two of
/// them in problem states and one switched off, its fold `open` or not.
fn crowded(fold: Option<bool>) -> Flow {
    let callers: Vec<FlowCard> = (0..12)
        .map(|index| {
            let state = match index {
                3 => FlowState::Degraded,
                7 => FlowState::Unreachable,
                9 => FlowState::Off,
                _ => FlowState::Up,
            };
            card(&format!("caller-{index:02}"), state, &[])
        })
        .collect();
    let edges = callers
        .iter()
        .map(|caller| FlowEdge {
            from: caller.id.clone(),
            to: String::from("hub"),
            state: caller.state,
            label: caller.state.is_problem().then(|| String::from("401")),
        })
        .collect();
    Flow {
        id: String::from("busy"),
        label: String::from("A busy bridge"),
        list_label: String::from("A busy bridge, lane by lane"),
        lanes: vec![
            FlowLane {
                title: String::from("Callers"),
                cards: callers,
                fold: fold.map(|open| LaneFold {
                    open,
                    toggle: String::from("?open=callers"),
                }),
            },
            FlowLane {
                title: String::from("Mapping set"),
                cards: vec![card("hub", FlowState::Up, &[])],
                fold: None,
            },
        ],
        edges,
        foot: String::from("12 callers."),
    }
}

#[test]
fn a_lane_past_the_threshold_folds_its_quiet_cards_into_one_summary_that_unfolds_by_a_link() {
    const { assert!(12 > FOLD_PAST) };
    let shown = html(|| flow(&crowded(Some(false))));
    for kept in ["/caller-03", "/caller-07"] {
        assert!(
            shown.contains(&format!("href=\"{kept}\"")),
            "a problem card is never folded: {kept}: {shown}"
        );
    }
    assert!(
        !shown.contains(r#"href="/caller-00""#) && !shown.contains(r#"href="/caller-09""#),
        "a quiet card folds: {shown}"
    );
    assert!(
        shown.contains(r#"aria-label="10 more in Callers: Up 9 · Off 1. Shows every card.""#),
        "{shown}"
    );
    assert!(shown.contains(">10 more</text>"), "{shown}");
    assert!(
        shown.contains(r#"href="?open=callers" aria-label="Show every card of Callers""#),
        "the lane's title carries its unfold link: {shown}"
    );
    assert_eq!(
        1,
        shown.matches(r#"class="fui-edge fui-edge-ok""#).count(),
        "the folded callers' edges are one edge into the hub: {shown}"
    );
    assert!(
        shown.contains(r#"<span class="fui-pipeline-name">10 more</span>"#),
        "the phone's list folds as the drawing does: {shown}"
    );
    assert!(!shown.contains("style="), "{shown}");
}

#[test]
fn an_open_lane_shows_every_card_and_folds_again_by_a_link_and_a_lane_with_no_fold_never_folds() {
    let open = html(|| flow(&crowded(Some(true))));
    assert_eq!(
        13,
        open.matches(r#"class="fui-tnode-link""#).count(),
        "every caller and the hub: {open}"
    );
    assert_eq!(
        3,
        open.matches("Fold the quiet cards of Callers").count(),
        "the drawing's link, its tooltip and the list's row: {open}"
    );
    let never = html(|| flow(&crowded(None)));
    assert_eq!(13, never.matches(r#"class="fui-tnode-link""#).count());
    assert!(!never.contains("fui-tnode-toggle"), "{never}");
    assert!(!never.contains("more in Callers"), "{never}");
}

#[test]
fn labelled_problem_edges_that_meet_one_card_carry_their_labels_at_their_starts() {
    let shown = html(|| flow(&crowded(Some(true))));
    assert_eq!(
        2,
        shown
            .matches(r#"<text x="12.6" y="12.5" text-anchor="middle">401</text>"#)
            .count(),
        "{shown}"
    );
    assert!(
        shown.contains(r#"transform="translate(212.0 251.0)" class="fui-edge-label""#),
        "the label sits just after the degraded caller's side: {shown}"
    );
}

fn score(score: f64, weights: &[f64]) -> Score {
    Score {
        total: weights.iter().sum(),
        value: score,
        grade: String::from("Probable"),
        contributions: weights
            .iter()
            .map(|weight| Contribution {
                label: format!("Field {weight}: close match"),
                reading: Some(String::from("Jaro-Winkler 0.94")),
                weight: *weight,
            })
            .collect(),
        thresholds: [("Possible", 0.60), ("Probable", 0.80), ("Certain", 0.95)]
            .map(|(label, at)| Threshold {
                label: label.to_owned(),
                at,
            })
            .to_vec(),
    }
}

#[test]
fn a_breakdown_draws_a_positive_a_negative_and_a_zero_weight_in_figures() {
    let shown = html(|| score_breakdown(&score(0.87, &[2.4, -1.2, 0.0])));
    assert!(shown.contains("fui-score-bar-add"), "{shown}");
    assert!(shown.contains("fui-score-bar-remove"), "{shown}");
    for weight in [">+2.4<", ">\u{2212}1.2<", ">0<"] {
        assert!(shown.contains(weight), "{weight}: {shown}");
    }
    assert!(
        shown.contains("fui-score-weight fui-num fui-score-zero"),
        "{shown}"
    );
    assert!(
        shown.contains("fui-score-figure fui-num\">0.87<"),
        "{shown}"
    );
    assert!(shown.contains("fui-pill fui-tone-info"), "{shown}");
    assert!(!shown.contains("style="), "{shown}");
}

#[test]
fn a_rows_words_come_before_its_comparator() {
    let shown = html(|| score_breakdown(&score(0.87, &[2.4])));
    let words = shown.find("Field 2.4: close match").expect("the words");
    let reading = shown.find("Jaro-Winkler 0.94").expect("the comparator");
    assert!(words < reading, "{shown}");
    assert!(
        shown.contains(r#"<span class="fui-score-reading">"#),
        "{shown}"
    );
}

#[test]
fn the_scale_is_an_image_that_reads_the_score_the_grade_and_every_contribution() {
    let shown = html(|| score_breakdown(&score(0.80, &[2.4, -1.2])));
    assert!(
        shown.contains(
            "aria-label=\"Score 0.80, Probable, on the scale: Possible from 0.60, Probable from 0.80, Certain from 0.95. Field 2.4: close match +2.4, Field -1.2: close match \u{2212}1.2.\""
        ),
        "{shown}"
    );
    assert!(shown.contains(r#"role="img""#), "{shown}");
}

#[test]
fn every_threshold_label_is_centred_on_its_mark_and_the_start_keeps_its_label() {
    let shown = html(|| score_breakdown(&score(0.70, &[1.0])));
    assert_eq!(
        4,
        shown.matches(r#"text-anchor="middle""#).count(),
        "three thresholds and the start: {shown}"
    );
    assert!(!shown.contains(r#"text-anchor="end""#), "{shown}");
    assert!(
        shown.contains(">0.50</text>"),
        "the scale starts at 0.50: {shown}"
    );
    assert!(!shown.contains("fui-score-scale-tick"), "{shown}");
    assert!(
        shown.contains(r#"cx="40.0%""#),
        "a score between thresholds: {shown}"
    );
}

#[test]
fn the_compact_score_is_a_bar_in_the_grades_tone_beside_its_figure() {
    let shown = html(|| score_bar(0.87, Tone::Info));
    assert!(shown.contains("fui-score-cell fui-tone-info"), "{shown}");
    assert!(shown.contains(r#"width="87""#), "{shown}");
    assert!(
        shown.contains(r#"<span class="fui-num">0.87</span>"#),
        "{shown}"
    );
    assert!(shown.contains(r#"aria-hidden="true""#), "{shown}");
}
