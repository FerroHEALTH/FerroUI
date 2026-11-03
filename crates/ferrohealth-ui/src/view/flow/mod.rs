// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The flow (B9): F9's cards and edges laid out as a pipeline.
//!
//! Its lanes run from the callers on the left to the upstreams on the right,
//! with one quiet foot line, and on a phone the same cards are a list, lane
//! by lane.
//!
//! An edge's state is its line style, its colour and, on a problem edge,
//! one end mark and a label (family rule 12). The drawing is two SVG
//! layers of one size: the lanes and the edges, an image whose description
//! names each problem edge (WCAG 2.2 SC 1.1.1), and over it the cards, each
//! a link to its page. Hovering or focusing a card shows its path with the
//! pulse; the stylesheet does it, so it needs no script, changes paint
//! only (family rule 3) and stills under reduced motion. No specification
//! governs the drawing: our own design.

pub(super) mod draw;
pub(super) mod layout;
mod list;
mod order;
mod plan;

use std::borrow::Cow;

use crate::html::Html;
use crate::i18n::{Locale, Text, Word, tr};
use crate::icon::Glyph;
use crate::tone::Tone;
use crate::view::button::{Variant, label};
use crate::view::dialog::{Explanation, Width, info_dialog};

use draw::{Scene, card_view, crowded, edge_view, fold_view, lane_toggle, problems};
use layout::{GAP, LANE, layout, units};
use list::list;
use plan::{Shown, plan};

/// The state of a card or an edge, drawn as a shape, a colour and a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowState {
    /// It answers: a solid line, a filled dot.
    #[default]
    Up,
    /// It answers slowly or with an error: a dashed line marked "!", a
    /// diamond.
    Degraded,
    /// It does not answer: a dotted line marked with a cross, a cross.
    Unreachable,
    /// No probe has read it: a dash-dot line, a hollow ring.
    NotChecked,
    /// It is switched off: a thin faded line, a muted card.
    Off,
}

impl FlowState {
    /// The state's word in the page's locale.
    #[must_use]
    pub fn word(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::Up => Text::FlowUp,
                Self::Degraded => Text::FlowDegraded,
                Self::Unreachable => Text::FlowUnreachable,
                Self::NotChecked => Text::FlowNotChecked,
                Self::Off => Text::FlowOff,
            },
        )
    }

    /// The key a state is sent and named by, such as `degraded`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Degraded => "degraded",
            Self::Unreachable => "unreachable",
            Self::NotChecked => "not-checked",
            Self::Off => "off",
        }
    }

    /// Whether an edge in this state is a problem, which carries a mark and
    /// a label.
    #[must_use]
    pub const fn is_problem(self) -> bool {
        matches!(self, Self::Degraded | Self::Unreachable)
    }

    /// The tone of a card or a dot in this state.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Up => Tone::Ok,
            Self::Degraded => Tone::Warn,
            Self::Unreachable => Tone::Bad,
            Self::NotChecked | Self::Off => Tone::Neutral,
        }
    }

    /// How an edge in this state is drawn, such as "dashed, marked !", in
    /// the page's locale.
    #[must_use]
    pub fn line(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::Up => Text::FlowStyleUp,
                Self::Degraded => Text::FlowStyleDegraded,
                Self::Unreachable => Text::FlowStyleUnreachable,
                Self::NotChecked => Text::FlowStyleNotChecked,
                Self::Off => Text::FlowStyleOff,
            },
        )
    }

    /// The classes of an edge in this state.
    pub(crate) const fn edge(self) -> &'static str {
        match self {
            Self::Up => "fui-edge fui-edge-ok",
            Self::Degraded => "fui-edge fui-edge-warn",
            Self::Unreachable => "fui-edge fui-edge-bad",
            Self::NotChecked => "fui-edge fui-edge-neutral",
            Self::Off => "fui-edge fui-edge-muted",
        }
    }
}

/// A short sample of an edge's line in `state`, for a legend or a rule.
pub(crate) fn line_sample(state: FlowState) -> Html {
    Html::raw(format!(
        "<svg width=\"26\" height=\"10\" aria-hidden=\"true\" class=\"fui-line-sample\"><path d=\"M1 5 H25\" class=\"{}\"></path></svg>",
        state.edge()
    ))
}

/// One state of an edge rule and what it means where the console draws
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeRule {
    /// The state.
    pub state: FlowState,
    /// What it means, such as "it answers slowly or with an error".
    pub meaning: String,
}

/// Returns the edge rule `rules`: one line per state, its line sample, its
/// word, how its line is drawn and what it means, such as "Degraded ·
/// dashed, marked !: it answers slowly or with an error".
///
/// The line sample is hidden from a reader, who reads the words; the same
/// rule explains the flow (B9) and the topology (F9).
#[must_use]
pub fn edge_rule(locale: &Locale, rules: Vec<EdgeRule>) -> Html {
    let items: Html = rules
        .into_iter()
        .map(|EdgeRule { state, meaning }| {
            let rest = state
                .line(locale)
                .map(|line| format!(" · {line}: {meaning}"));
            Html::raw(format!(
                "<li>{}<span><strong>{}</strong>{}</span></li>",
                line_sample(state),
                state.word(locale),
                Html::text(rest.text())
            ))
        })
        .collect();
    Html::raw(format!("<ul class=\"fui-topology-rule\">{items}</ul>"))
}

/// What a flow's "How the flow is drawn" dialog explains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowRule {
    /// The dialog's id, which [`rule_button`] opens.
    pub id: Cow<'static, str>,
    /// The rule, one state a line.
    pub rules: Vec<EdgeRule>,
    /// The closing paragraph, such as what a card shows and opens.
    pub note: String,
}

/// Returns "How the flow is drawn": the short informational dialog with
/// the edge rule and its closing paragraph, and Close as its only action.
#[must_use]
pub fn rule_dialog(locale: &Locale, rule: FlowRule) -> Html {
    let FlowRule { id, rules, note } = rule;
    let body = edge_rule(locale, rules).then(Html::raw(format!(
        "<p class=\"fui-small fui-muted\">{}</p>",
        Html::text(note)
    )));
    info_dialog(
        locale,
        Explanation {
            id,
            title: tr(locale, Text::FlowRuleTitle),
        },
        body,
        Width::Narrow,
    )
}

/// Returns "How it is drawn", the small ghost button of a flow's panel
/// head that opens the dialog `dialog` ([`rule_dialog`]) with the
/// `show-modal` command, so it opens with no script.
#[must_use]
pub fn rule_button(locale: &Locale, dialog: &str) -> Html {
    Html::raw(format!(
        "<button type=\"button\" command=\"show-modal\" commandfor=\"{}\" aria-haspopup=\"dialog\" class=\"{}\">{}</button>",
        Html::text(dialog),
        Variant::Ghost.small(),
        label(Some(Glyph::INFO), tr(locale, Text::FlowRuleButton))
    ))
}

/// One card of a flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowCard {
    /// The id its edges and paths name it by.
    pub id: String,
    /// The glyph before its name.
    pub glyph: Glyph,
    /// Its name, such as "openEHR CDR".
    pub name: String,
    /// Its line under the name, such as "FerroEHR · Up".
    pub line: String,
    /// Its state.
    pub state: FlowState,
    /// The page it opens.
    pub href: String,
    /// The ids of the cards its path runs through, in order, such as the
    /// caller, the face, the hub and the upstream; hovering or focusing the
    /// card shows the edges between them with the pulse. Empty for none.
    pub path: Vec<String>,
    /// Whether it is the hub of the flow: the accent card with no dot.
    pub hub: bool,
}

/// One lane of a flow, a column of cards under its title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowLane {
    /// Its title, such as "Callers".
    pub title: String,
    /// Its cards, top to bottom as the console passes them; the drawing
    /// orders them to cross fewer edges.
    pub cards: Vec<FlowCard>,
    /// How it folds past [`FOLD_PAST`] cards, or `None` for a lane that
    /// never folds.
    pub fold: Option<LaneFold>,
}

/// The most cards a lane shows before it folds its quiet ones: eight rows
/// keep the drawing about one screen tall at 1440 px.
pub const FOLD_PAST: usize = 8;

/// How a lane past [`FOLD_PAST`] cards folds its quiet cards, those in no
/// problem state and on no problem edge, into one summary card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneFold {
    /// Whether the reader unfolded it to show every card.
    pub open: bool,
    /// The page that unfolds the lane while it is folded and folds it while
    /// it is open, such as `?open=callers`.
    pub toggle: String,
}

/// One edge of a flow, from a card's right side to a card's left side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowEdge {
    /// The id of the card it leaves.
    pub from: String,
    /// The id of the card it reaches, where its end mark sits.
    pub to: String,
    /// Its state.
    pub state: FlowState,
    /// Its label, such as "503", drawn only on a problem edge.
    pub label: Option<String>,
}

/// What a flow shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    /// The drawing's id; its title and description take it with `-title`
    /// and `-desc`.
    pub id: String,
    /// The drawing's accessible name, such as "The bridge as a flow".
    pub label: String,
    /// The accessible name of the cards and of the phone's list, such as
    /// "The bridge as a flow, lane by lane".
    pub list_label: String,
    /// The lanes, left to right.
    pub lanes: Vec<FlowLane>,
    /// The edges. One that names no card is not drawn.
    pub edges: Vec<FlowEdge>,
    /// The quiet foot line, such as "3 faces on · 3 of 3 upstreams up.".
    pub foot: String,
}

/// Returns `flow`: the drawing, which a phone replaces with its list, and
/// the quiet foot line.
///
/// Put it in a panel ([`crate::view::surface::titled_panel`]), whose head
/// holds the title and the actions. Each lane is ordered to cross the
/// fewest edges, with the console's order breaking a tie; cards in a lane
/// spread over the height of the fullest lane; a hub card is taller and
/// carries no dot. A lane past [`FOLD_PAST`] cards with a fold folds its
/// quiet cards into one summary card, a link that unfolds it, and keeps
/// every card in a problem state or on a problem edge.
#[must_use]
pub fn flow(locale: &Locale, flow: &Flow) -> Html {
    let plan = plan(locale, flow);
    let (places, width, height) = layout(&plan);
    let crowded = crowded(&plan);
    let scene = Scene {
        locale,
        plan: &plan,
        places: &places,
        crowded: &crowded,
    };
    let view_box = format!("-8 -4 {:.0} {height:.0}", width + 16.0);
    let lanes: Html = plan
        .lanes
        .iter()
        .enumerate()
        .map(|(column, lane)| {
            let x = units(column) * (LANE + GAP);
            Html::raw(format!(
                "<text x=\"{x}\" y=\"12\" class=\"fui-topology-lane\">{}</text>",
                Html::text(lane.title)
            ))
        })
        .collect();
    let toggles: Html = plan
        .lanes
        .iter()
        .enumerate()
        .filter_map(|(column, lane)| {
            let toggle = lane.toggle?;
            Some(lane_toggle(
                locale,
                units(column) * (LANE + GAP),
                lane.title,
                toggle,
            ))
        })
        .collect();
    let edges: Html = plan
        .edges
        .iter()
        .filter(|edge| !edge.state.is_problem())
        .chain(plan.edges.iter().filter(|edge| edge.state.is_problem()))
        .filter_map(|edge| edge_view(edge, &places, &crowded, false))
        .collect();
    let cards: Html = plan
        .lanes
        .iter()
        .flat_map(|lane| lane.shown.iter())
        .filter_map(|shown| {
            let place = *places.get(shown.id())?;
            Some(match shown {
                Shown::Card(card) => card_view(card, place, scene),
                Shown::Fold(folded) => fold_view(folded, place),
            })
        })
        .collect();
    let id = Html::text(&flow.id);
    Html::raw(format!(
        "<div class=\"fui-pipeline\"><div class=\"fui-topology-canvas\"><div class=\"fui-pipeline-stage\"><svg viewBox=\"{view_box}\" role=\"img\" aria-labelledby=\"{id}-title {id}-desc\" class=\"fui-topology-svg fui-pipeline-edges\"><title id=\"{id}-title\">{}</title><desc id=\"{id}-desc\">{}</desc><g>{lanes}</g><g>{edges}</g></svg><svg viewBox=\"{view_box}\" role=\"group\" aria-label=\"{}\" class=\"fui-topology-svg fui-pipeline-cards\">{toggles}{cards}</svg></div></div>{}<p class=\"fui-pipeline-foot\">{}</p></div>",
        Html::text(&flow.label),
        Html::text(problems(locale, flow)),
        Html::text(&flow.list_label),
        list(locale, &plan, &flow.list_label),
        Html::text(&flow.foot)
    ))
}

#[cfg(test)]
mod tests {
    use super::order::crossings;
    use super::plan::{Planned, Shown, plan};
    use super::{FOLD_PAST, Flow, FlowCard, FlowEdge, FlowLane, FlowState, LaneFold};
    use crate::i18n::Locale;
    use crate::icon::Glyph;

    fn card(id: String, state: FlowState) -> FlowCard {
        FlowCard {
            name: id.clone(),
            line: String::new(),
            href: format!("/{id}"),
            id,
            glyph: Glyph::SERVER,
            state,
            path: Vec::new(),
            hub: false,
        }
    }

    fn edge(from: &str, to: &str, state: FlowState) -> FlowEdge {
        FlowEdge {
            from: from.to_owned(),
            to: to.to_owned(),
            state,
            label: Some(String::from("503")),
        }
    }

    /// A fixed busy bridge: 30 callers, each calling one or two of six
    /// faces, and 30 upstreams, each reached from one face, wired by a
    /// fixed sequence so the console's order crosses freely.
    fn busy(fold: Option<LaneFold>) -> Flow {
        let mut seed = 7_u32;
        let mut next = |bound: usize| {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345) % 2_147_483_648;
            usize::try_from(seed >> 8).unwrap_or_default() % bound
        };
        let lane = |title: &str, prefix: &str, count: usize, fold: Option<LaneFold>| FlowLane {
            title: title.to_owned(),
            cards: (0..count)
                .map(|index| {
                    let state = match index {
                        4 => FlowState::Degraded,
                        11 => FlowState::Unreachable,
                        17 => FlowState::Off,
                        _ => FlowState::Up,
                    };
                    card(format!("{prefix}{index:02}"), state)
                })
                .collect(),
            fold,
        };
        let mut edges = Vec::new();
        for caller in 0..30 {
            for _ in 0..=next(2) {
                let face = next(6);
                edges.push(edge(
                    &format!("c{caller:02}"),
                    &format!("f{face:02}"),
                    FlowState::Up,
                ));
            }
        }
        for upstream in 0..30 {
            let face = next(6);
            let state = if upstream == 23 {
                FlowState::Degraded
            } else {
                FlowState::Up
            };
            edges.push(edge(
                &format!("f{face:02}"),
                &format!("u{upstream:02}"),
                state,
            ));
        }
        Flow {
            id: String::from("busy"),
            label: String::new(),
            list_label: String::new(),
            lanes: vec![
                lane("Callers", "c", 30, fold.clone()),
                lane("Faces", "f", 6, None),
                lane("Upstreams", "u", 30, fold),
            ],
            edges,
            foot: String::new(),
        }
    }

    /// How many edges of `flow` cross with its lanes in `order`.
    fn crossed(flow: &Flow, order: &[Vec<&str>]) -> usize {
        let pairs: Vec<(&str, &str)> = flow
            .edges
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        crossings(order, &pairs)
    }

    #[test]
    fn the_lane_order_crosses_far_fewer_edges_than_the_consoles_and_is_deterministic() {
        let flow = busy(None);
        let console: Vec<Vec<&str>> = flow
            .lanes
            .iter()
            .map(|lane| lane.cards.iter().map(|card| card.id.as_str()).collect())
            .collect();
        let en = Locale::default();
        let planned = plan(&en, &flow);
        let drawn: Vec<Vec<&str>> = planned
            .lanes
            .iter()
            .map(|lane| lane.shown.iter().map(Shown::id).collect())
            .collect();
        let (before, after) = (crossed(&flow, &console), crossed(&flow, &drawn));
        assert_eq!((657, 201), (before, after), "{drawn:?}");
        let again = plan(&en, &flow);
        let redrawn: Vec<Vec<&str>> = again
            .lanes
            .iter()
            .map(|lane| lane.shown.iter().map(Shown::id).collect())
            .collect();
        assert_eq!(drawn, redrawn, "the same flow draws the same order");
    }

    #[test]
    fn a_flow_that_crosses_nothing_keeps_the_consoles_order() {
        let mut flow = busy(None);
        flow.edges = (0..6)
            .map(|index| {
                edge(
                    &format!("c{index:02}"),
                    &format!("f{index:02}"),
                    FlowState::Up,
                )
            })
            .collect();
        let en = Locale::default();
        let planned = plan(&en, &flow);
        for (lane, shown) in flow.lanes.iter().zip(&planned.lanes) {
            let ids: Vec<&str> = shown.lanes_ids();
            let console: Vec<&str> = lane.cards.iter().map(|card| card.id.as_str()).collect();
            assert_eq!(console, ids);
        }
    }

    #[test]
    fn a_folded_lane_keeps_every_problem_card_and_holds_the_rest_in_one_summary_last() {
        let flow = busy(Some(LaneFold {
            open: false,
            toggle: String::from("?open=all"),
        }));
        let en = Locale::default();
        let planned = plan(&en, &flow);
        let callers = planned.lanes.first().expect("the callers lane");
        let ids = callers.lanes_ids();
        assert!(ids.contains(&"c04") && ids.contains(&"c11"), "{ids:?}");
        assert!(!ids.contains(&"c17"), "an off card is quiet: {ids:?}");
        assert_eq!(Some(&"busy-fold-0"), ids.last(), "{ids:?}");
        assert_eq!(Some((true, "?open=all")), callers.toggle);
        let upstreams = planned.lanes.get(2).expect("the upstreams lane");
        assert!(
            upstreams.lanes_ids().contains(&"u23"),
            "a card on a problem edge stays"
        );
        let faces = planned.lanes.get(1).expect("the faces lane");
        assert_eq!(
            None, faces.toggle,
            "a lane of {FOLD_PAST} or fewer never folds"
        );
        let into_summary = planned
            .edges
            .iter()
            .filter(|edge| edge.from == "busy-fold-0")
            .count();
        assert!(
            into_summary <= 6,
            "one edge per face at most: {into_summary}"
        );
        let unfolded = busy(Some(LaneFold {
            open: true,
            toggle: String::from("?"),
        }));
        let open = plan(&en, &unfolded);
        let callers = open.lanes.first().expect("the callers lane");
        assert_eq!(30, callers.shown.len());
        assert_eq!(Some((false, "?")), callers.toggle);
    }

    impl Planned<'_> {
        fn lanes_ids(&self) -> Vec<&str> {
            self.shown.iter().map(Shown::id).collect()
        }
    }
}
