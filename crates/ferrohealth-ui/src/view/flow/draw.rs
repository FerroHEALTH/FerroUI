// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The SVG drawing of a flow: its edges, its cards and the lane folds.

use std::collections::{BTreeMap, BTreeSet};

use crate::html::Html;
use crate::i18n::{Arg, Locale, Text, tr, tr_with};
use crate::icon::{Glyph, icon};

use super::layout::{LANE, Place, curve, ends, fit, units};
use super::plan::{Drawn, Folded, LINE, LINE_MOST, NAME_MOST, Plan};
use super::{Flow, FlowCard, FlowState};

/// Where a problem edge draws its label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LabelAt {
    /// Beside the end mark, at the child end.
    Mark,
    /// After the start, where several labelled edges meet one card.
    Start,
}

/// The drawing of an edge from `from` to `to` in `state`: its line, and on
/// a problem edge the end mark at the child end while `marked`, and the
/// label where `label_at` says.
pub(crate) fn edge_line(
    from: Place,
    to: Place,
    state: FlowState,
    (label, label_at): (Option<&str>, LabelAt),
    on_path: bool,
    marked: bool,
) -> Html {
    let line = ends(from, to);
    let ((x1, y1), (x2, y2)) = line;
    let toward = if x2 >= x1 { 1.0 } else { -1.0 };
    let at = format!("translate({:.1} {y2:.1})", x2 - toward * 11.0);
    let mark = match (marked, state) {
        (true, FlowState::Unreachable) => format!(
            "<g transform=\"{at}\" class=\"fui-edge-mark fui-edge-mark-bad\"><circle r=\"7\"></circle><path d=\"M-3 -3 L3 3 M3 -3 L-3 3\"></path></g>"
        ),
        (true, FlowState::Degraded) => format!(
            "<g transform=\"{at}\" class=\"fui-edge-mark fui-edge-mark-warn\"><circle r=\"7\"></circle><path d=\"M0 -3.5 V1 M0 3.4 V3.5\"></path></g>"
        ),
        _ => String::new(),
    };
    let label = label
        .filter(|_| state.is_problem())
        .map_or_else(String::new, |label| {
            let width = 8.0 + units(label.chars().count()) * 5.7;
            let (left, y) = match (label_at, toward > 0.0) {
                (LabelAt::Mark, true) => (x2 - 22.0 - width, y2),
                (LabelAt::Mark, false) => (x2 + 22.0, y2),
                (LabelAt::Start, true) => (x1 + 12.0, y1),
                (LabelAt::Start, false) => (x1 - 12.0 - width, y1),
            };
            format!(
                "<g transform=\"translate({left:.1} {:.1})\" class=\"fui-edge-label\"><rect width=\"{width:.1}\" height=\"18\" rx=\"9\"></rect><text x=\"{:.1}\" y=\"12.5\" text-anchor=\"middle\">{}</text></g>",
                y - 9.0,
                width / 2.0,
                Html::text(label)
            )
        });
    let class = if on_path {
        format!("{} is-path", state.edge())
    } else {
        state.edge().to_owned()
    };
    Html::raw(format!(
        "<g><path d=\"{}\" class=\"{class}\"></path>{mark}{label}</g>",
        curve(line)
    ))
}

/// The drawing of `edge` between `places`: its line, and on a problem edge
/// its end mark and its label where `crowded` says several labels meet.
/// The drawing draws every problem edge last, so no line covers a label.
pub(super) fn edge_view(
    edge: &Drawn<'_>,
    places: &BTreeMap<&str, Place>,
    crowded: &BTreeSet<&str>,
    on_path: bool,
) -> Option<Html> {
    let from = places.get(edge.from.as_str())?;
    let to = places.get(edge.to.as_str())?;
    let label_at = if crowded.contains(edge.to.as_str()) {
        LabelAt::Start
    } else {
        LabelAt::Mark
    };
    Some(edge_line(
        *from,
        *to,
        edge.state,
        (edge.label, label_at),
        on_path,
        true,
    ))
}

/// The cards that more than one labelled problem edge enters.
pub(super) fn crowded<'p>(plan: &'p Plan<'_>) -> BTreeSet<&'p str> {
    let mut entered: BTreeMap<&str, usize> = BTreeMap::new();
    for edge in &plan.edges {
        if edge.state.is_problem() && edge.label.is_some() {
            *entered.entry(edge.to.as_str()).or_default() += 1;
        }
    }
    entered
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(id, _)| id)
        .collect()
}

/// The shape of a card's dot in `state`, centred on `(x, y)`.
pub(crate) fn dot(state: FlowState, x: f64, y: f64) -> Html {
    Html::raw(match state {
        FlowState::Up => {
            format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"4\" class=\"fui-tdot fui-tone-ok\"></circle>"
            )
        }
        FlowState::Degraded => format!(
            "<rect x=\"{}\" y=\"{}\" width=\"7\" height=\"7\" transform=\"rotate(45 {x} {y})\" class=\"fui-tdot fui-tone-warn\"></rect>",
            x - 3.5,
            y - 3.5
        ),
        FlowState::Unreachable => format!(
            "<path d=\"M{:.1} {:.1} L{:.1} {:.1} M{:.1} {:.1} L{:.1} {:.1}\" class=\"fui-tdot fui-tdot-cross fui-tone-bad\"></path>",
            x - 3.5,
            y - 3.5,
            x + 3.5,
            y + 3.5,
            x + 3.5,
            y - 3.5,
            x - 3.5,
            y + 3.5
        ),
        FlowState::NotChecked | FlowState::Off => {
            format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"3.5\" class=\"fui-tdot fui-tdot-ring\"></circle>"
            )
        }
    })
}

/// The accessible name of a card of a flow or a topology, titled `name`
/// over the note `line`: a hub's without its state `state`.
pub(crate) fn card_label(
    locale: &Locale,
    name: &str,
    line: &str,
    state: FlowState,
    hub: bool,
) -> String {
    let mut args = vec![("title", Arg::from(name)), ("note", Arg::from(line))];
    let text = if hub {
        Text::FlowHub
    } else {
        args.push(("status", state.word(locale).into()));
        Text::FlowCard
    };
    String::from(tr_with(locale, text, &args))
}

/// One card as a drawing draws it.
#[derive(Debug)]
pub(crate) struct CardDrawing {
    /// The card's kind class, such as `fui-tnode-node` or `fui-tnode-gateway`.
    pub(crate) kind: &'static str,
    /// Where it sits.
    pub(crate) place: Place,
    /// The glyph before its name.
    pub(crate) glyph: Glyph,
    /// Its name.
    pub(crate) name: String,
    /// Its line under the name.
    pub(crate) line: String,
    /// Its state.
    pub(crate) state: FlowState,
    /// The page it opens.
    pub(crate) href: String,
    /// Its accessible name, also its tooltip.
    pub(crate) label: String,
    /// Whether it is a hub: the accent card with no dot.
    pub(crate) hub: bool,
    /// Whether its name is an identifier, drawn in the code face.
    pub(crate) mono: bool,
    /// Whether it is the card whose quick look is open.
    pub(crate) selected: bool,
    /// The most characters of its name and of its line it shows before an
    /// ellipsis.
    pub(crate) most: (usize, usize),
    /// The lines it draws under its line, each in the line's room; the
    /// card's height leaves room for them.
    pub(crate) more: Vec<String>,
    /// Its controls beside the link, such as its quick look and its
    /// collapse button, drawn in the card's own units.
    pub(crate) controls: Option<Html>,
}

/// The drawing of a card: its link with the tone bar, the glyph, the name,
/// the dot and the line, and beside the link its controls.
pub(crate) fn card_drawing(card: CardDrawing) -> Html {
    let CardDrawing {
        kind,
        place,
        glyph,
        name,
        line,
        state,
        href,
        label,
        hub,
        mono,
        selected,
        most: (name_most, line_most),
        more,
        controls,
    } = card;
    let height = place.height;
    let base = height - LINE * units(more.len());
    let mut class = format!("fui-tnode {kind} {}", state.tone().class());
    if state == FlowState::Off {
        class.push_str(" is-muted");
    }
    if selected {
        class.push_str(" is-selected");
    }
    let shown_dot = if hub {
        Html::empty()
    } else {
        dot(state, 47.0, base / 2.0 + 10.0)
    };
    let line_x = if hub { 42.0 } else { 56.0 };
    let more: Html = more
        .into_iter()
        .enumerate()
        .map(|(index, text)| {
            let y = base / 2.0 + 14.0 + LINE * (units(index) + 1.0);
            Html::raw(format!(
                "<text x=\"{line_x}\" y=\"{y}\" class=\"fui-tnode-line\">{}</text>",
                Html::text(fit(&text, line_most))
            ))
        })
        .collect();
    let name_class = if mono {
        "fui-tnode-name fui-tnode-mono"
    } else {
        "fui-tnode-name"
    };
    let label = Html::text(label);
    Html::raw(format!(
        "<g transform=\"translate({:.1} {:.1})\" class=\"{class}\"><a href=\"{}\" aria-label=\"{label}\" class=\"fui-tnode-link\"><title>{label}</title><rect width=\"{}\" height=\"{height}\" rx=\"8\" class=\"fui-tnode-box\"></rect><rect width=\"4\" height=\"{}\" x=\"0\" y=\"8\" rx=\"2\" class=\"fui-tnode-bar\"></rect><g transform=\"translate(14 {:.1})\" class=\"fui-tnode-glyph\">{}</g><text x=\"42\" y=\"{}\" class=\"{name_class}\">{}</text>{shown_dot}<text x=\"{line_x}\" y=\"{}\" class=\"fui-tnode-line\">{}</text>{more}</a>{}</g>",
        place.x,
        place.y,
        Html::text(href),
        place.width,
        height - 16.0,
        base / 2.0 - 9.0,
        icon(glyph, "18"),
        base / 2.0 - 4.0,
        Html::text(fit(&name, name_most)),
        base / 2.0 + 14.0,
        Html::text(fit(&line, line_most)),
        controls.unwrap_or_default()
    ))
}

/// The path a hovered or focused card shows: `lines` raised, the glow and
/// the pulse along `route`, hidden from assistive technology.
pub(crate) fn path_drawing(lines: Html, route: &str) -> Html {
    let lines = String::from(lines);
    Html::raw(format!(
        "<g aria-hidden=\"true\" class=\"fui-pipeline-path\">{lines}<path d=\"{route}\" class=\"fui-path-glow\"></path><path d=\"{route}\" pathLength=\"100\" class=\"fui-path-pulse\"></path></g>"
    ))
}

/// What every card of one drawing reads: the plan, the places and the
/// crowded cards.
#[derive(Debug, Clone, Copy)]
pub(super) struct Scene<'s, 'a> {
    pub(super) locale: &'s Locale,
    pub(super) plan: &'s Plan<'a>,
    pub(super) places: &'s BTreeMap<&'s str, Place>,
    pub(super) crowded: &'s BTreeSet<&'s str>,
}

/// The drawing of `card` at `place`, a link to its page, after its path
/// with the pulse, which shows while the card is hovered or focused; a
/// folded card on the path is its summary.
pub(super) fn card_view(card: &FlowCard, place: Place, scene: Scene<'_, '_>) -> Html {
    let Scene {
        locale,
        plan,
        places,
        crowded,
    } = scene;
    let steps: Vec<&Drawn<'_>> = card
        .path
        .windows(2)
        .filter_map(|pair| match pair {
            [from, to] => {
                let (from, to) = (plan.shown_id(from), plan.shown_id(to));
                plan.edges
                    .iter()
                    .find(|edge| edge.from == from && edge.to == to)
            }
            _ => None,
        })
        .collect();
    let path = if steps.is_empty() {
        Html::empty()
    } else {
        let lines: Html = steps
            .iter()
            .filter_map(|edge| edge_view(edge, places, crowded, true))
            .collect();
        let route = steps
            .iter()
            .filter_map(|edge| {
                let from = places.get(edge.from.as_str())?;
                let to = places.get(edge.to.as_str())?;
                Some(curve(ends(*from, *to)))
            })
            .collect::<Vec<_>>()
            .join(" ");
        path_drawing(lines, &route)
    };
    let drawing = card_drawing(CardDrawing {
        kind: if card.hub {
            "fui-tnode-gateway"
        } else {
            "fui-tnode-node"
        },
        place,
        glyph: card.glyph,
        name: card.name.clone(),
        line: card.line.clone(),
        state: card.state,
        href: card.href.clone(),
        label: card_label(locale, &card.name, &card.line, card.state, card.hub),
        hub: card.hub,
        mono: false,
        selected: false,
        most: (NAME_MOST, LINE_MOST),
        more: Vec::new(),
        controls: None,
    });
    Html::raw(format!(
        "<g class=\"fui-pipeline-card\">{path}{drawing}</g>"
    ))
}

/// The summary card of a folded lane at `place`, a link that unfolds it.
pub(super) fn fold_view(folded: &Folded<'_>, place: Place) -> Html {
    let mut lines = folded.lines().into_iter();
    let first = lines.next().unwrap_or_default();
    let more = lines.collect();
    let drawing = card_drawing(CardDrawing {
        kind: "fui-tnode-group",
        place,
        glyph: Glyph::ELLIPSIS,
        name: tr_with(
            folded.locale,
            Text::FlowFoldName,
            &[("count", folded.count().into())],
        )
        .into(),
        line: first,
        state: folded.state(),
        href: folded.href.to_owned(),
        label: fold_name(folded),
        hub: false,
        mono: false,
        selected: false,
        most: (NAME_MOST, LINE_MOST),
        more,
        controls: None,
    });
    Html::raw(format!("<g class=\"fui-pipeline-card\">{drawing}</g>"))
}

/// The accessible name of a summary card, also its tooltip.
fn fold_name(folded: &Folded<'_>) -> String {
    tr_with(
        folded.locale,
        Text::FlowFoldCard,
        &[
            ("count", folded.count().into()),
            ("title", folded.title.into()),
            ("note", folded.line().into()),
        ],
    )
    .into()
}

/// The words of a lane's fold link: unfold while `folded`, else fold.
pub(super) fn toggle_words(locale: &Locale, folded: bool, title: &str) -> String {
    let text = if folded {
        Text::FlowUnfold
    } else {
        Text::FlowFold
    };
    tr_with(locale, text, &[("title", title.into())]).into()
}

/// The fold link beside the title of the lane at `x`.
pub(super) fn lane_toggle(
    locale: &Locale,
    x: f64,
    title: &str,
    (folded, href): (bool, &str),
) -> Html {
    let name = Html::text(toggle_words(locale, folded, title));
    let glyph = if folded {
        Glyph::CHEVRON_DOWN
    } else {
        Glyph::CHEVRON_UP
    };
    Html::raw(format!(
        "<g transform=\"translate({:.1} -1)\" class=\"fui-tnode-toggle\"><a href=\"{}\" aria-label=\"{name}\"><title>{name}</title><rect width=\"16\" height=\"16\" fill=\"transparent\"></rect>{}</a></g>",
        x + LANE - 16.0,
        Html::text(href),
        icon(glyph, "16")
    ))
}

/// The description of the drawing: each problem edge with its state and
/// label, or that there is none.
pub(super) fn problems(locale: &Locale, flow: &Flow) -> String {
    let named = |id: &str| {
        flow.lanes
            .iter()
            .flat_map(|lane| lane.cards.iter())
            .find(|card| card.id == id)
            .map_or_else(|| id.to_owned(), |card| card.name.clone())
    };
    let sentences: Vec<String> = flow
        .edges
        .iter()
        .filter(|edge| edge.state.is_problem())
        .map(|edge| {
            let mut args = vec![
                ("from", named(&edge.from).into()),
                ("to", named(&edge.to).into()),
                ("status", edge.state.word(locale).into()),
            ];
            match &edge.label {
                Some(label) => {
                    args.push(("note", label.clone().into()));
                    tr_with(locale, Text::FlowEdgeNoted, &args)
                }
                None => tr_with(locale, Text::FlowEdge, &args),
            }
            .into()
        })
        .collect();
    if sentences.is_empty() {
        tr(locale, Text::FlowSteady).into()
    } else {
        format!("{}.", sentences.join(". "))
    }
}
