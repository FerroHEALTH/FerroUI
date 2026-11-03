// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The drawing of a topology map: its edges, its cards and the image
//! description.

use crate::html::Html;
use crate::i18n::{Locale, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::flow::FlowState;
use crate::view::flow::draw::{
    CardDrawing, LabelAt, card_drawing, card_label, edge_line, path_drawing,
};
use crate::view::flow::layout::{Place, curve, ends, units};

use super::layout::{Edge, Laid, Layout};
use super::tools::{cluster, minimap, quick_look};
use super::{ClusterLink, QuickLook, Topology, TopologyCard, TopologyGroup};

/// The edge from a caller above the hub down to the hub: a vertical curve
/// from the caller's foot to the hub's top.
fn caller_edge(from: Place, to: Place, state: FlowState) -> Html {
    let (x1, y1) = (from.x + from.width / 2.0, from.y + from.height);
    let (x2, y2) = (to.x + to.width / 2.0, to.y);
    let bend = f64::midpoint(y1, y2);
    Html::raw(format!(
        "<path d=\"M{x1:.1} {y1:.1} C{x1:.1} {bend:.1}, {x2:.1} {bend:.1}, {x2:.1} {y2:.1}\" class=\"{}\"></path>",
        state.edge()
    ))
}

/// The drawing of one edge of the layout.
fn edge(edge: Edge<'_>, on_path: bool) -> Html {
    if edge.down {
        caller_edge(edge.from, edge.to, edge.state)
    } else {
        edge_line(
            edge.from,
            edge.to,
            edge.state,
            (edge.label, LabelAt::Mark),
            on_path,
            edge.marked,
        )
    }
}

/// The controls at a card's end: its quick look, and a group's collapse.
fn controls(
    locale: &Locale,
    card: &TopologyCard,
    group: Option<&TopologyGroup>,
    width: f64,
) -> Option<Html> {
    let toggle = group.and_then(|group| {
        let href = group.toggle.as_deref()?;
        let (glyph, text) = if group.collapsed {
            (Glyph::CHEVRON_RIGHT, Text::TopologyExpand)
        } else {
            (Glyph::CHEVRON_DOWN, Text::TopologyCollapse)
        };
        let name = Html::text(tr_with(locale, text, &[("title", card.name.clone().into())]).text());
        Some(format!(
            "<g transform=\"translate({:.1} 9)\" class=\"fui-tnode-toggle\"><a href=\"{}\" aria-label=\"{name}\"><title>{name}</title><rect width=\"16\" height=\"16\" fill=\"transparent\"></rect>{}</a></g>",
            width - 26.0,
            Html::text(href),
            icon(glyph, "16")
        ))
    });
    let look = card.look.as_deref().map(|href| {
        let name = tr_with(locale, Text::TopologyLookOf, &[("title", card.name.clone().into())]);
        let left = width - if toggle.is_some() { 50.0 } else { 26.0 };
        format!(
            "<g transform=\"translate({left:.1} 8)\" class=\"fui-tnode-info\"><a href=\"{}\" aria-label=\"{}\"><title>{}</title><rect width=\"18\" height=\"18\" rx=\"5\"></rect><g transform=\"translate(3 3)\">{}</g></a></g>",
            Html::text(href),
            Html::text(name.text()),
            tr(locale, Text::TopologyLook),
            icon(Glyph::INFO, "12")
        )
    });
    (toggle.is_some() || look.is_some()).then(|| {
        Html::raw(format!(
            "{}{}",
            look.unwrap_or_default(),
            toggle.unwrap_or_default()
        ))
    })
}

/// The most characters of a card's name and line its width leaves room
/// for, after its controls.
fn most(width: f64, mono: bool, controls: usize, hub: bool) -> (usize, usize) {
    let room = width - 46.0 - 10.0 - units(controls) * 24.0;
    let name = room / if mono { 7.4 } else { 6.7 };
    let line = (width - if hub { 52.0 } else { 66.0 }) / 6.2;
    (count_of(name), count_of(line))
}

/// A measure as a whole count, rounded down, at least four.
fn count_of(measure: f64) -> usize {
    let mut count = 4_usize;
    while units(count + 1) <= measure {
        count += 1;
    }
    count
}

/// The drawing of one laid-out card: its path from the hub with the pulse,
/// then the card itself.
fn card_view(locale: &Locale, laid: &Laid<'_>, selected: Option<&str>) -> Html {
    let card = laid.card;
    let hub = laid.kind == "fui-tnode-gateway";
    let path = if laid.via.len() > 1 {
        let lines = laid.edges.iter().map(|each| edge(*each, true)).collect();
        let route = laid
            .via
            .windows(2)
            .filter_map(|pair| match pair {
                [from, to] => Some(curve(ends(*from, *to))),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        path_drawing(lines, &route)
    } else {
        Html::default()
    };
    let count = usize::from(card.look.is_some())
        + usize::from(laid.group.is_some_and(|group| group.toggle.is_some()));
    let drawing = card_drawing(CardDrawing {
        kind: laid.kind,
        place: laid.place,
        glyph: card.glyph,
        name: card.name.clone(),
        line: card.line.clone(),
        state: card.state,
        href: card.href.clone(),
        label: card_label(locale, &card.name, &card.line, card.state, hub),
        hub,
        mono: card.mono,
        selected: selected == Some(card.id.as_str()),
        most: most(laid.place.width, card.mono, count, hub),
        more: Vec::new(),
        controls: controls(locale, card, laid.group, laid.place.width),
    });
    Html::raw(format!(
        "<g class=\"fui-pipeline-card\">{path}{drawing}</g>"
    ))
}

/// The description of the map in `locale`: each problem edge with its
/// state and label, or that there is none, then the List's sentence.
fn problems(locale: &Locale, topology: &Topology) -> String {
    let mut sentences = Vec::new();
    let mut note = |from: &TopologyCard, to: &TopologyCard| {
        if !to.state.is_problem() {
            return;
        }
        let mut args = vec![
            ("from", from.name.clone().into()),
            ("to", to.name.clone().into()),
            ("status", to.state.word(locale).into()),
        ];
        let text = match &to.edge_label {
            Some(label) => {
                args.push(("note", label.clone().into()));
                Text::FlowEdgeNoted
            }
            None => Text::FlowEdge,
        };
        sentences.push(String::from(tr_with(locale, text, &args)));
    };
    for side in [&topology.start, &topology.end].into_iter().flatten() {
        for group in &side.groups {
            note(&topology.hub, &group.card);
            if !group.collapsed {
                for child in &group.children {
                    note(&group.card, child);
                }
            }
        }
    }
    let lead = if sentences.is_empty() {
        tr(locale, Text::FlowSteady).into()
    } else {
        format!("{}.", sentences.join(". "))
    };
    format!("{lead} {}", tr(locale, Text::TopologyMapList).text())
}

/// The parts of the map the server answers: the zoom cluster's links and
/// the quick look.
#[derive(Debug)]
pub(super) struct MapParts {
    pub(super) collapse_all: Option<ClusterLink>,
    pub(super) minimap: Option<ClusterLink>,
    pub(super) look: Option<QuickLook>,
}

/// The map: the canvas with its two layers, the minimap and the zoom
/// cluster, and the quick look beside it.
pub(super) fn map(
    locale: &Locale,
    topology: &Topology,
    layout: &Layout<'_>,
    parts: MapParts,
) -> String {
    let MapParts {
        collapse_all,
        minimap: minimap_link,
        look,
    } = parts;
    let selected = look.as_ref().map(|look| look.card.clone());
    let (width, height) = (layout.width + 16.0, layout.height + 16.0);
    let lanes: Vec<String> = layout
        .lanes
        .iter()
        .map(|(x, y, title, middle)| {
            let anchor = if *middle { "middle" } else { "start" };
            format!(
                "<text x=\"{x:.1}\" y=\"{y:.1}\" text-anchor=\"{anchor}\" class=\"fui-topology-lane\">{}</text>",
                Html::text(title)
            )
        })
        .collect();
    let rule = layout.rule.map_or_else(String::new, |(from, to)| {
        format!(
            "<line x1=\"{from:.1}\" x2=\"{to:.1}\" y1=\"84\" y2=\"84\" class=\"fui-topology-hairline\"></line>"
        )
    });
    let edges: Html = layout.edges.iter().map(|each| edge(*each, false)).collect();
    let cards: Html = layout
        .cards
        .iter()
        .map(|laid| card_view(locale, laid, selected.as_deref()))
        .collect();
    let id = Html::text(&topology.id);
    let minimap_on = minimap_link.as_ref().is_some_and(|link| link.on);
    let shown_minimap = if minimap_on {
        minimap(locale, layout)
    } else {
        Html::default()
    };
    let cluster = cluster(locale, collapse_all.as_ref(), minimap_link.as_ref());
    let has_panel = if look.is_some() { " has-panel" } else { "" };
    let look = look.map_or_else(String::new, |look| quick_look(locale, look));
    let label = Html::text(&topology.label);
    format!(
        "<div class=\"fui-topology-map{has_panel}\"><div class=\"fui-topology-canvas\"><div tabindex=\"0\" role=\"group\" aria-label=\"{keys}\" aria-keyshortcuts=\"+ - 0 i Space\" class=\"fui-topology-scroll\"><div data-width=\"{width:.0}\" data-height=\"{height:.0}\" class=\"fui-pipeline-stage\"><svg viewBox=\"-8 -8 {width:.0} {height:.0}\" width=\"{width:.0}\" height=\"{height:.0}\" role=\"img\" aria-labelledby=\"{id}-title {id}-desc\" class=\"fui-topology-svg fui-pipeline-edges\"><title id=\"{id}-title\">{label}</title><desc id=\"{id}-desc\">{desc}</desc><g>{lanes}{rule}</g><g>{edges}</g></svg><svg viewBox=\"-8 -8 {width:.0} {height:.0}\" width=\"{width:.0}\" height=\"{height:.0}\" role=\"group\" aria-label=\"{label}\" class=\"fui-topology-svg fui-pipeline-cards\">{cards}</svg></div></div>{shown_minimap}{cluster}</div>{look}</div>",
        keys = Html::text(tr(locale, Text::TopologyMapKeys).text()),
        desc = Html::text(problems(locale, topology)),
        lanes = lanes.concat(),
    )
}
