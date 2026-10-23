// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The topology (F9): a federation drawn as a hub with its two sides, and
//! its List twin.
//!
//! The hub sits in the middle with its callers above it. The groups of the
//! end side and their children sit to its right, and those of the start
//! side to its left, mirrored. Every edge is a Bézier curve with horizontal
//! tangents, drawn with the flow's (B9) edges and cards: its line style and
//! colour say the state, and a problem edge carries one mark at its child
//! end and its label (family rule 12). A collapsed group is one card whose
//! edge carries the mark of the worst child it holds. Hovering or focusing
//! a card raises its path from the hub with the pulse, paint only (family
//! rule 3), and reduced motion stops the pulse. The toolbar holds the
//! Status menu, which is the legend, and the Map and List switch; the zoom
//! cluster sits at the canvas's foot, each icon named in a tooltip above
//! it, with a minimap for a large federation and the quick look beside the
//! canvas.
//!
//! Every page works with no WebAssembly: a card is a link to its page, its
//! quick look a link, a group's collapse a link, the Status menu a `GET`
//! form, and the List twin is the same graph as an indented tree, the
//! phone's view and the screen reader's. The island
//! [`crate::island::TopologyKeys`] adds zoom by the cluster and the keys,
//! panning by the pointer, the quick look on "i" or Space, and the arrows
//! along the List. No specification governs the drawing: our own design;
//! the List twin follows the WAI-ARIA Authoring Practices tree view pattern.

use leptos::prelude::*;

use crate::i18n::{Arg, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::flow::{
    CardDrawing, FlowState, LabelAt, Place, card_drawing, curve, edge_line, ends, path_drawing,
    units,
};
use crate::widget::topology::TopologyWords;

/// One card of a topology: the hub, a caller, a group or a child.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyCard {
    /// The id the quick look names it by.
    pub id: String,
    /// The glyph before its name.
    pub glyph: Glyph,
    /// Its name, such as "Testkliniek Oost" or "node-b".
    pub name: String,
    /// Its line under the name, such as "Degraded · 503".
    pub line: String,
    /// Its state; a group's is the worst of its children.
    pub state: FlowState,
    /// The page it opens.
    pub href: String,
    /// Whether its name is an identifier, drawn in the code face.
    pub mono: bool,
    /// The label of the edge into it, such as "503", drawn only on a
    /// problem edge.
    pub edge_label: Option<String>,
    /// The page that opens its quick look, such as `?look=node-b`; `None`
    /// for a card with none.
    pub look: Option<String>,
    /// The List twin's measure, such as "median 38 ms".
    pub measure: Option<String>,
    /// The List twin's note, such as "probed 13 s ago".
    pub note: Option<String>,
    /// The List twin's action on its row, such as Probe now; the map's
    /// quick look holds the same action.
    pub action: Option<RowAction>,
}

/// An action on a row of the List twin: a `POST` form with one field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowAction {
    /// Its words, such as "Probe now".
    pub label: String,
    /// The glyph before its words.
    pub glyph: Glyph,
    /// The console's route the form posts to.
    pub action: String,
    /// The name of the field it posts.
    pub name: String,
    /// The value it posts, such as the card's id.
    pub value: String,
}

/// One group of a side: a card over its children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyGroup {
    /// The group's own card, such as an organisation.
    pub card: TopologyCard,
    /// Its children, such as the organisation's nodes.
    pub children: Vec<TopologyCard>,
    /// Whether it is folded to its card.
    pub collapsed: bool,
    /// The page that folds or unfolds it, such as `?collapse=org-b`;
    /// `None` for a group that does not fold.
    pub toggle: Option<String>,
}

/// One side of the hub: its title and its groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologySide {
    /// Its title, such as "Organisations and nodes" on the map and
    /// "Organisations" in the List.
    pub title: String,
    /// The List's quiet line after the title, such as "and their nodes".
    pub line: String,
    /// Its groups, top to bottom.
    pub groups: Vec<TopologyGroup>,
}

/// The callers above the hub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyCallers {
    /// Their title, such as "Calls the gateway".
    pub title: String,
    /// The List's quiet line after the title, such as "who calls the
    /// gateway".
    pub line: String,
    /// The callers, left to right.
    pub cards: Vec<TopologyCard>,
}

/// One state of the Status menu: how many cards are in it, and whether
/// the map shows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCount {
    /// The state.
    pub state: FlowState,
    /// How many cards are in it.
    pub count: u32,
    /// Whether the map and the List show them.
    pub shown: bool,
}

/// The Status menu, which is the legend: each state with its line, its dot
/// and its count, a checkbox of a `GET` form that filters the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMenu {
    /// The console's page the form gets, such as the topology's own.
    pub action: String,
    /// The name each checked state is sent under, such as `show`.
    pub name: String,
    /// The states in the order the menu lists them.
    pub states: Vec<StatusCount>,
}

/// Which view the topology shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TopologyView {
    /// The map from 48 rem up and the List below it.
    #[default]
    Auto,
    /// The map at every width.
    Map,
    /// The List at every width.
    List,
}

impl TopologyView {
    /// The topology's classes in this view.
    const fn class(self) -> &'static str {
        match self {
            Self::Auto => "fui-topology fui-topology-view-auto",
            Self::Map => "fui-topology fui-topology-view-map",
            Self::List => "fui-topology fui-topology-view-list",
        }
    }
}

/// The quick look of one card, beside the canvas.
#[derive(Debug)]
pub struct QuickLook {
    /// The id of the card it is about, which the map marks selected.
    pub card: String,
    /// Its title, such as the card's name.
    pub title: String,
    /// Whether the title is an identifier, drawn in the code face.
    pub mono: bool,
    /// A quiet line under the title, such as "Node of Testkliniek Oost".
    pub line: Option<String>,
    /// The page that closes it.
    pub close: String,
    /// Its body: the state, the facts and the actions.
    pub body: AnyView,
}

/// A link of the zoom cluster that the server answers, such as Collapse
/// every group or the minimap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterLink {
    /// The page it leads to.
    pub href: String,
    /// Whether what it turns on is on, drawn pressed.
    pub on: bool,
}

/// What a topology shows.
#[derive(Debug)]
pub struct Topology {
    /// Its id: the map's title and description take it with `-title` and
    /// `-desc`.
    pub id: String,
    /// The map's accessible name, such as "The federation as a map".
    pub label: String,
    /// The List twin's accessible name, such as "The federation as a list".
    pub list_label: String,
    /// The hub.
    pub hub: TopologyCard,
    /// The callers above the hub, if any.
    pub callers: Option<TopologyCallers>,
    /// The side to the hub's left, drawn mirrored, if any.
    pub start: Option<TopologySide>,
    /// The side to the hub's right, if any.
    pub end: Option<TopologySide>,
    /// The view in force.
    pub view: TopologyView,
    /// The page that shows the map, and the one that shows the List.
    pub views: (String, String),
    /// The Status menu.
    pub status: StatusMenu,
    /// The console's own tools at the toolbar's start, such as a find
    /// field, after the Status menu.
    pub tools: Option<AnyView>,
    /// The console's own words at the toolbar's end, such as when the map
    /// was read.
    pub updated: Option<AnyView>,
    /// Collapse every group, or `None` where nothing folds.
    pub collapse_all: Option<ClusterLink>,
    /// The minimap's switch, or `None` for a map that has none; while it is
    /// on, the minimap shows.
    pub minimap: Option<ClusterLink>,
    /// The quick look that is open, if any.
    pub look: Option<QuickLook>,
}

/// A card's height, a row's height, the gap between groups, the hub's
/// width and height, a caller's width and the room above the sides when
/// there are callers and when there are none.
const CARD: f64 = 56.0;
const ROW: f64 = 66.0;
const GROUP_GAP: f64 = 14.0;
const HUB_WIDTH: f64 = 176.0;
const HUB_HEIGHT: f64 = 72.0;
const CALLER: f64 = 204.0;
const TOP_CALLED: f64 = 118.0;
const TOP: f64 = 34.0;

/// The widths of a side's children and groups, the gap between the two
/// columns, and the gap between the groups and the hub.
const END_CHILD: f64 = 182.0;
const END_GROUP: f64 = 272.0;
const START_CHILD: f64 = 222.0;
const START_GROUP: f64 = 236.0;
const OUTER_GAP: f64 = 92.0;
const HUB_GAP: f64 = 84.0;

/// The minimap's widest and tallest measure.
const MINIMAP: f64 = 168.0;
const MINIMAP_TALL: f64 = 300.0;

/// One edge laid out.
#[derive(Debug, Clone, Copy)]
struct Edge<'a> {
    from: Place,
    to: Place,
    state: FlowState,
    label: Option<&'a str>,
    marked: bool,
    down: bool,
}

/// One card laid out: where it sits, what it is, and the cards its path
/// from the hub runs through.
#[derive(Debug)]
struct Laid<'a> {
    card: &'a TopologyCard,
    place: Place,
    kind: &'static str,
    group: Option<&'a TopologyGroup>,
    via: Vec<Place>,
    edges: Vec<Edge<'a>>,
}

/// The layout of a topology: every card, every edge, the lane titles and
/// the drawing's width and height.
#[derive(Debug)]
struct Layout<'a> {
    cards: Vec<Laid<'a>>,
    edges: Vec<Edge<'a>>,
    lanes: Vec<(f64, f64, &'a str, bool)>,
    rule: Option<(f64, f64)>,
    width: f64,
    height: f64,
}

/// The groups of a side, each with its place and its children's places.
type Stacked<'a> = Vec<(&'a TopologyGroup, Place, Vec<Place>)>;

/// The rows of `side`, top first: each group with its place and its
/// children's places, the next free row after the last.
fn stack(
    side: &TopologySide,
    top: f64,
    x_group: f64,
    group_width: f64,
    x_child: f64,
    child_width: f64,
) -> (Stacked<'_>, f64) {
    let mut y = top;
    let mut rows = Vec::new();
    for group in &side.groups {
        let open = !group.collapsed && !group.children.is_empty();
        let count = if open { group.children.len() } else { 1 };
        let last = y + units(count.saturating_sub(1)) * ROW;
        let place = Place {
            x: x_group,
            y: f64::midpoint(y, last),
            width: group_width,
            height: CARD,
        };
        let children = if open {
            group
                .children
                .iter()
                .enumerate()
                .map(|(index, _)| Place {
                    x: x_child,
                    y: y + units(index) * ROW,
                    width: child_width,
                    height: CARD,
                })
                .collect()
        } else {
            Vec::new()
        };
        rows.push((group, place, children));
        y = last + ROW + GROUP_GAP;
    }
    (rows, y - GROUP_GAP)
}

/// Lays `topology` out.
fn layout(topology: &Topology) -> Layout<'_> {
    let top = if topology.callers.is_some() {
        TOP_CALLED
    } else {
        TOP
    };
    let x_hub = if topology.start.is_some() {
        START_CHILD + OUTER_GAP + START_GROUP + HUB_GAP
    } else {
        0.0
    };
    let x_end_group = x_hub + HUB_WIDTH + HUB_GAP;
    let x_end_child = x_end_group + END_GROUP + OUTER_GAP;
    let start = topology.start.as_ref().map(|side| {
        stack(
            side,
            top,
            START_CHILD + OUTER_GAP,
            START_GROUP,
            0.0,
            START_CHILD,
        )
    });
    let end = topology
        .end
        .as_ref()
        .map(|side| stack(side, top, x_end_group, END_GROUP, x_end_child, END_CHILD));
    let bottom = [&start, &end]
        .into_iter()
        .flatten()
        .map(|(_, bottom)| *bottom)
        .fold(top + CARD, f64::max);
    let middle = (top + bottom - (ROW - CARD)) / 2.0;
    let hub = Place {
        x: x_hub,
        y: middle - HUB_HEIGHT / 2.0,
        width: HUB_WIDTH,
        height: HUB_HEIGHT,
    };
    let mut laid = Layout {
        cards: vec![Laid {
            card: &topology.hub,
            place: hub,
            kind: "fui-tnode-gateway",
            group: None,
            via: Vec::new(),
            edges: Vec::new(),
        }],
        edges: Vec::new(),
        lanes: Vec::new(),
        rule: None,
        width: x_hub + HUB_WIDTH,
        height: bottom.max(hub.y + hub.height),
    };
    if let Some(callers) = &topology.callers {
        place_callers(&mut laid, callers, hub);
    }
    for (side, rows, title_x) in [
        (topology.start.as_ref(), start, 0.0),
        (topology.end.as_ref(), end, x_end_group),
    ] {
        if let (Some(side), Some((rows, _))) = (side, rows) {
            laid.lanes
                .push((title_x, top - 16.0, side.title.as_str(), false));
            place_side(&mut laid, rows, hub);
        }
    }
    if topology.callers.is_some() {
        laid.rule = Some((0.0, laid.width));
    }
    laid
}

/// Places `callers` side by side above the `hub`, each with its edge down
/// to it, under their title.
fn place_callers<'a>(laid: &mut Layout<'a>, callers: &'a TopologyCallers, hub: Place) {
    let count = units(callers.cards.len().max(1));
    let span = count * CALLER + (count - 1.0) * 24.0;
    let left = hub.x + hub.width / 2.0 - span / 2.0;
    laid.lanes
        .push((hub.x + hub.width / 2.0, 12.0, callers.title.as_str(), true));
    for (index, card) in callers.cards.iter().enumerate() {
        let place = Place {
            x: left + units(index) * (CALLER + 24.0),
            y: 22.0,
            width: CALLER,
            height: CARD,
        };
        laid.edges.push(Edge {
            from: place,
            to: hub,
            state: card.state,
            label: None,
            marked: false,
            down: true,
        });
        laid.cards.push(Laid {
            card,
            place,
            kind: "fui-tnode-node",
            group: None,
            via: Vec::new(),
            edges: Vec::new(),
        });
    }
}

/// Places the stacked groups `rows` of one side, each with its edge from
/// the `hub` and its children with theirs.
fn place_side<'a>(laid: &mut Layout<'a>, rows: Stacked<'a>, hub: Place) {
    for (group, place, children) in rows {
        let folded = children.is_empty();
        let into_group = Edge {
            from: hub,
            to: place,
            state: group.card.state,
            label: group.card.edge_label.as_deref().filter(|_| folded),
            marked: folded,
            down: false,
        };
        laid.edges.push(into_group);
        laid.width = laid.width.max(place.x + place.width);
        laid.cards.push(Laid {
            card: &group.card,
            place,
            kind: "fui-tnode-org",
            group: Some(group),
            via: vec![hub, place],
            edges: vec![into_group],
        });
        for (card, child) in group.children.iter().zip(children) {
            let into_child = Edge {
                from: place,
                to: child,
                state: card.state,
                label: card.edge_label.as_deref(),
                marked: true,
                down: false,
            };
            laid.edges.push(into_child);
            laid.width = laid.width.max(child.x + child.width);
            laid.cards.push(Laid {
                card,
                place: child,
                kind: "fui-tnode-node",
                group: None,
                via: vec![hub, place, child],
                edges: vec![into_group, into_child],
            });
        }
    }
}

/// The edge from a caller above the hub down to the hub: a vertical curve
/// from the caller's foot to the hub's top.
fn caller_edge(from: Place, to: Place, state: FlowState) -> AnyView {
    let (x1, y1) = (from.x + from.width / 2.0, from.y + from.height);
    let (x2, y2) = (to.x + to.width / 2.0, to.y);
    let bend = f64::midpoint(y1, y2);
    let d = format!("M{x1:.1} {y1:.1} C{x1:.1} {bend:.1}, {x2:.1} {bend:.1}, {x2:.1} {y2:.1}");
    view! { <path class=state.edge() d=d></path> }.into_any()
}

/// The drawing of one edge of the layout.
fn edge(edge: Edge<'_>, on_path: bool) -> AnyView {
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

/// The accessible name of `card`, the hub's without a state.
fn card_name(card: &TopologyCard, hub: bool) -> String {
    String::from(if hub {
        tr_with(
            Text::FlowHub,
            &[
                ("title", card.name.clone().into()),
                ("note", card.line.clone().into()),
            ],
        )
    } else {
        tr_with(
            Text::FlowCard,
            &[
                ("title", card.name.clone().into()),
                ("note", card.line.clone().into()),
                ("status", card.state.word().into()),
            ],
        )
    })
}

/// The controls at a card's end: its quick look, and a group's collapse.
fn controls(card: &TopologyCard, group: Option<&TopologyGroup>, width: f64) -> Option<AnyView> {
    let toggle = group.and_then(|group| {
        let href = group.toggle.clone()?;
        let (glyph, text) = if group.collapsed {
            (Glyph::CHEVRON_RIGHT, Text::TopologyExpand)
        } else {
            (Glyph::CHEVRON_DOWN, Text::TopologyCollapse)
        };
        let name = tr_with(text, &[("title", card.name.clone().into())]);
        let tip = name.clone();
        let at = format!("translate({:.1} 9)", width - 26.0);
        Some(view! {
            <g class="fui-tnode-toggle" transform=at>
                <a href=href aria-label=name>
                    <title>{tip}</title>
                    <rect width="16" height="16" fill="transparent"></rect>
                    {icon(glyph, "16")}
                </a>
            </g>
        })
    });
    let look = card.look.clone().map(|href| {
        let name = tr_with(Text::TopologyLookOf, &[("title", card.name.clone().into())]);
        let left = width - if toggle.is_some() { 50.0 } else { 26.0 };
        let at = format!("translate({left:.1} 8)");
        view! {
            <g class="fui-tnode-info" transform=at>
                <a href=href aria-label=name>
                    <title>{tr(Text::TopologyLook)}</title>
                    <rect width="18" height="18" rx="5"></rect>
                    <g transform="translate(3 3)">{icon(Glyph::INFO, "12")}</g>
                </a>
            </g>
        }
    });
    (toggle.is_some() || look.is_some()).then(|| view! { {look} {toggle} }.into_any())
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
fn card_view(laid: &Laid<'_>, selected: Option<&str>) -> AnyView {
    let card = laid.card;
    let hub = laid.kind == "fui-tnode-gateway";
    let path = (laid.via.len() > 1).then(|| {
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
        path_drawing(lines, route)
    });
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
        label: card_name(card, hub),
        hub,
        mono: card.mono,
        selected: selected == Some(card.id.as_str()),
        most: most(laid.place.width, card.mono, count, hub),
        more: Vec::new(),
        controls: controls(card, laid.group, laid.place.width),
    });
    view! {
        <g class="fui-pipeline-card">
            {path}
            {drawing}
        </g>
    }
    .into_any()
}

/// The description of the map: each problem edge with its state and
/// label, or that there is none, then the List's sentence.
fn problems(topology: &Topology) -> String {
    let mut sentences = Vec::new();
    let mut note = |from: &TopologyCard, to: &TopologyCard| {
        if !to.state.is_problem() {
            return;
        }
        let mut args = vec![
            ("from", from.name.clone().into()),
            ("to", to.name.clone().into()),
            ("status", to.state.word().into()),
        ];
        let text = match &to.edge_label {
            Some(label) => {
                args.push(("note", label.clone().into()));
                Text::FlowEdgeNoted
            }
            None => Text::FlowEdge,
        };
        sentences.push(String::from(tr_with(text, &args)));
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
        tr(Text::FlowSteady).into()
    } else {
        format!("{}.", sentences.join(". "))
    };
    format!("{lead} {}", tr(Text::TopologyMapList).text())
}

/// The minimap of `layout`: a rectangle per card in its tone, and the
/// frame of what the map shows, the whole drawing at first. A tall
/// federation narrows it to keep it inside its tallest measure.
fn minimap(layout: &Layout<'_>) -> AnyView {
    let scale = (MINIMAP / layout.width.max(1.0)).min(MINIMAP_TALL / layout.height.max(1.0));
    let (width, height) = (layout.width * scale, layout.height * scale);
    let rects = layout
        .cards
        .iter()
        .map(|laid| {
            let class = format!("fui-minimap-node {}", laid.card.state.tone().class());
            view! {
                <rect
                    class=class
                    x=format!("{:.1}", laid.place.x * scale)
                    y=format!("{:.1}", laid.place.y * scale)
                    width=format!("{:.1}", laid.place.width * scale)
                    height=format!("{:.1}", (laid.place.height * scale).max(2.0))
                    rx="1"
                ></rect>
            }
        })
        .collect_view();
    let view_box = format!("-4 -4 {:.0} {:.0}", width + 8.0, height + 8.0);
    view! {
        <div class="fui-topology-minimap" role="img" aria-label=tr(Text::TopologyMinimapFrame)>
            <svg
                viewBox=view_box
                width=format!("{:.0}", width + 8.0)
                height=format!("{:.0}", height + 8.0)
                aria-hidden="true"
            >
                {rects}
                <rect
                    class="fui-minimap-view"
                    data-scale=format!("{scale:.6}")
                    x="0"
                    y="0"
                    width=format!("{width:.1}")
                    height=format!("{height:.1}")
                    rx="2"
                ></rect>
            </svg>
        </div>
    }
    .into_any()
}

/// One icon button of the zoom cluster with its tooltip above it, waiting
/// for the page's script while `waits`.
fn cluster_button(glyph: Glyph, words: Word, zoom: &'static str, wait: &Word) -> AnyView {
    let tip = words.clone();
    let why = wait.clone();
    view! {
        <span class="fui-anchor">
            <button
                type="button"
                class="fui-icon-button"
                aria-label=words
                data-zoom=zoom
                disabled=true
                title=why
            >
                {icon(glyph, "16")}
            </button>
            <span class="fui-tooltip fui-tooltip-above" role="tooltip">
                {tip}
            </span>
        </span>
    }
    .into_any()
}

/// One link of the zoom cluster that the server answers, pressed while on.
fn cluster_link(glyph: Glyph, words: Word, link: ClusterLink) -> AnyView {
    let tip = words.clone();
    view! {
        <span class="fui-anchor">
            <a
                href=link.href
                class="fui-icon-button"
                aria-label=words
                aria-pressed=link.on.to_string()
            >
                {icon(glyph, "16")}
            </a>
            <span class="fui-tooltip fui-tooltip-above" role="tooltip">
                {tip}
            </span>
        </span>
    }
    .into_any()
}

/// The zoom cluster: zoom in and out, the readout and Fit, which wait for
/// the page's script, then the server's links.
fn cluster(collapse_all: Option<ClusterLink>, minimap: Option<ClusterLink>) -> AnyView {
    let wait = tr(Text::TopologyZoomWait);
    let level = tr_with(Text::TopologyZoomLevel, &[("value", Arg::from(100))]);
    let readout = tr_with(Text::TopologyZoom, &[("value", Arg::from(100))]);
    let links = collapse_all.is_some() || minimap.is_some();
    let collapse = collapse_all.map(|link| {
        let (glyph, text) = if link.on {
            (Glyph::CHEVRON_DOWN, Text::TopologyExpandAll)
        } else {
            (Glyph::CHEVRON_UP, Text::TopologyCollapseAll)
        };
        cluster_link(glyph, tr(text), link)
    });
    let minimap = minimap.map(|link| cluster_link(Glyph::LAYERS, tr(Text::TopologyMinimap), link));
    let fit_tip = tr(Text::TopologyFitWhole);
    view! {
        <div class="fui-topology-zoom" role="toolbar" aria-label=tr(Text::TopologyControls)>
            {cluster_button(Glyph::PLUS, tr(Text::TopologyZoomIn), "in", &wait)}
            {cluster_button(Glyph::MINUS, tr(Text::TopologyZoomOut), "out", &wait)}
            <span class="fui-topology-zoom-level" role="status" aria-label=level data-zoom-readout="">
                {readout}
            </span>
            <button
                type="button"
                class=Variant::Ghost.small()
                data-zoom="fit"
                disabled=true
                title=wait.clone()
                data-title=fit_tip
            >
                {tr(Text::TopologyFit)}
            </button>
            {links.then(|| view! { <span class="fui-topology-zoom-sep" aria-hidden="true"></span> })}
            {collapse}
            {minimap}
        </div>
    }
    .into_any()
}

/// A short sample of an edge's line in `state`, for the legend.
fn line_sample(state: FlowState) -> AnyView {
    view! {
        <svg class="fui-line-sample" width="26" height="10" aria-hidden="true">
            <path class=state.edge() d="M1 5 H25"></path>
        </svg>
    }
    .into_any()
}

/// The health dot of `state` with its word as its accessible name; a state
/// switched off has no shape.
fn state_dot(state: FlowState) -> AnyView {
    let word = state.word();
    if state == FlowState::Off {
        return view! { <span class="fui-sr-only">{word}</span> }.into_any();
    }
    let class = format!("fui-dot {}", state.tone().class());
    let tip = word.clone();
    view! { <span class=class role="img" aria-label=word title=tip></span> }.into_any()
}

/// A state as its shape and its word (F12), with a quiet detail after it;
/// a state switched off draws no shape, as its muted row says.
fn state_word(state: FlowState, detail: Option<String>) -> AnyView {
    let word = state.word();
    if state == FlowState::Off {
        let detail = detail.map(|detail| view! { <span class="fui-status-detail">{detail}</span> });
        return view! { <span class="fui-state fui-tone-neutral is-muted">{word} {detail}</span> }
            .into_any();
    }
    crate::view::state::state(state.into(), word, detail)
}

/// The Status menu: a `<details>` whose summary is the button, holding the
/// `GET` form of the states, each its line, its dot, its word and its
/// count.
fn status_menu(menu: StatusMenu) -> AnyView {
    let StatusMenu {
        action,
        name,
        states,
    } = menu;
    let total = u32::try_from(states.len()).unwrap_or(u32::MAX);
    let shown = u32::try_from(states.iter().filter(|each| each.shown).count()).unwrap_or(u32::MAX);
    let filtered = shown < total;
    let words = if filtered {
        tr_with(
            Text::TopologyStatusSome,
            &[("shown", Arg::from(shown)), ("total", Arg::from(total))],
        )
    } else {
        tr(Text::TopologyStatus)
    };
    let items = states
        .into_iter()
        .map(|each| {
            let id = format!("topology-show-{}", each.state.key());
            let label_for = id.clone();
            view! {
                <label class="fui-menu-item fui-menu-check" for=label_for>
                    <input
                        type="checkbox"
                        id=id
                        name=name.clone()
                        value=each.state.key()
                        checked=each.shown
                    />
                    {line_sample(each.state)}
                    {state_dot(each.state)}
                    <span class="fui-menu-item-text">
                        <span class="fui-menu-item-label">{each.state.word()}</span>
                    </span>
                    <span class="fui-menu-count">{each.count}</span>
                </label>
            }
        })
        .collect_view();
    let class = Variant::Secondary.class();
    view! {
        <details class="fui-anchor">
            <summary class=class class:is-filtered=filtered>
                {icon(Glyph::FUNNEL, "16")} {words} {icon(Glyph::CHEVRON_DOWN, "16")}
            </summary>
            <form
                method="get"
                action=action
                class="fui-menu fui-menu-compact fui-menu-anchored fui-menu-start"
                aria-label=tr(Text::TopologyStates)
            >
                <div class="fui-menu-items">{items}</div>
                <div class="fui-menu-sep" role="separator"></div>
                <div class="fui-menu-foot">
                    <p class="fui-small fui-muted">{tr(Text::TopologyLegend)}</p>
                    <button type="submit" class=Variant::Secondary.small()>
                        {tr(Text::TopologyShow)}
                    </button>
                </div>
            </form>
        </details>
    }
    .into_any()
}

/// The Map and List switch: two links, the view in force pressed, and in
/// the automatic view the stylesheet presses the List below 48 rem.
fn view_switch(view_in_force: TopologyView, (map, list): (String, String)) -> AnyView {
    let map_on = view_in_force != TopologyView::List;
    let class = if view_in_force == TopologyView::Auto {
        "fui-segmented fui-view-switch fui-view-auto"
    } else {
        "fui-segmented fui-view-switch"
    };
    view! {
        <div role="group" aria-label=tr(Text::TopologyView) class=class>
            <a href=map class="fui-segment fui-view-map" aria-pressed=map_on.to_string()>
                {icon(Glyph::NETWORK, "14")}
                {tr(Text::TopologyViewMap)}
            </a>
            <a href=list class="fui-segment fui-view-list" aria-pressed=(!map_on).to_string()>
                {icon(Glyph::LIST_CHECKS, "14")}
                {tr(Text::TopologyViewList)}
            </a>
        </div>
    }
    .into_any()
}

/// The quick look beside the canvas.
fn quick_look(look: QuickLook) -> AnyView {
    let QuickLook {
        card: _,
        title,
        mono,
        line,
        close,
        body,
    } = look;
    let title = if mono {
        view! { <code>{title}</code> }.into_any()
    } else {
        title.into_any()
    };
    let line = line.map(|line| view! { <p class="fui-small fui-muted">{line}</p> });
    let close_words = tr(Text::TopologyLookClose);
    view! {
        <aside class="fui-topology-panel" aria-labelledby="topology-look-title">
            <div class="fui-side-panel-head">
                <div>
                    <h2 id="topology-look-title" class="fui-card-title">{title}</h2>
                    {line}
                </div>
                <a href=close class="fui-icon-button" aria-label=close_words.clone() title=close_words>
                    {icon(Glyph::X, "16")}
                </a>
            </div>
            <div class="fui-side-panel-body fui-stack">{body}</div>
        </aside>
    }
    .into_any()
}

/// The parts of the map the server answers: the zoom cluster's links and
/// the quick look.
#[derive(Debug)]
struct MapParts {
    collapse_all: Option<ClusterLink>,
    minimap: Option<ClusterLink>,
    look: Option<QuickLook>,
}

/// The map: the canvas with its two layers, the minimap and the zoom
/// cluster, and the quick look beside it.
fn map(topology: &Topology, layout: &Layout<'_>, parts: MapParts) -> AnyView {
    let MapParts {
        collapse_all,
        minimap: minimap_link,
        look,
    } = parts;
    let selected = look.as_ref().map(|look| look.card.clone());
    let view_box = format!(
        "-8 -8 {:.0} {:.0}",
        layout.width + 16.0,
        layout.height + 16.0
    );
    let lanes = layout
        .lanes
        .iter()
        .map(|(x, y, title, middle)| {
            let anchor = if *middle { "middle" } else { "start" };
            let title = (*title).to_owned();
            view! {
                <text class="fui-topology-lane" x=format!("{x:.1}") y=format!("{y:.1}") text-anchor=anchor>
                    {title}
                </text>
            }
        })
        .collect_view();
    let rule = layout.rule.map(|(from, to)| {
        view! {
            <line
                class="fui-topology-hairline"
                x1=format!("{from:.1}")
                x2=format!("{to:.1}")
                y1="84"
                y2="84"
            ></line>
        }
    });
    let edges = layout
        .edges
        .iter()
        .map(|each| edge(*each, false))
        .collect_view();
    let cards = layout
        .cards
        .iter()
        .map(|laid| card_view(laid, selected.as_deref()))
        .collect_view();
    let title = format!("{}-title", topology.id);
    let desc = format!("{}-desc", topology.id);
    let named_by = format!("{title} {desc}");
    let minimap_on = minimap_link.as_ref().is_some_and(|link| link.on);
    let shown_minimap = minimap_on.then(|| minimap(layout));
    let cluster = cluster(collapse_all, minimap_link);
    let look = look.map(quick_look);
    let has_panel = look.is_some();
    let width = format!("{:.0}", layout.width + 16.0);
    let height = format!("{:.0}", layout.height + 16.0);
    view! {
        <div class="fui-topology-map" class:has-panel=has_panel>
            <div class="fui-topology-canvas">
                <div
                    class="fui-topology-scroll"
                    tabindex="0"
                    role="group"
                    aria-label=tr(Text::TopologyMapKeys)
                    aria-keyshortcuts="+ - 0 i Space"
                >
                    <div class="fui-pipeline-stage" data-width=width data-height=height>
                        <svg
                            class="fui-topology-svg fui-pipeline-edges"
                            viewBox=view_box.clone()
                            width=width.clone()
                            height=height.clone()
                            role="img"
                            aria-labelledby=named_by
                        >
                            <title id=title>{topology.label.clone()}</title>
                            <desc id=desc>{problems(topology)}</desc>
                            <g>{lanes} {rule}</g>
                            <g>{edges}</g>
                        </svg>
                        <svg
                            class="fui-topology-svg fui-pipeline-cards"
                            viewBox=view_box
                            width=width.clone()
                            height=height.clone()
                            role="group"
                            aria-label=topology.label.clone()
                        >
                            {cards}
                        </svg>
                    </div>
                </div>
                {shown_minimap}
                {cluster}
            </div>
            {look}
        </div>
    }
    .into_any()
}

/// One row of the List twin.
#[derive(Debug)]
struct Row<'a> {
    level: u8,
    card: &'a TopologyCard,
    group: Option<&'a TopologyGroup>,
    detail: Option<String>,
    posinset: usize,
    setsize: usize,
}

/// The lane row of the List: a side's title and its quiet line, a heading
/// for the rows under it and no item of the tree.
fn lane_row(title: &str, line: &str) -> AnyView {
    let (title, line) = (title.to_owned(), line.to_owned());
    view! {
        <li role="presentation" class="fui-tree-lane">
            <span>{title}</span>
            <span class="fui-small fui-muted">{line}</span>
        </li>
    }
    .into_any()
}

/// One item row of the List: the name with its glyph as a link to its
/// page, a group's fold as a link, the state, the measure, the note and
/// the action.
fn item_row(row: Row<'_>) -> AnyView {
    let Row {
        level,
        card,
        group,
        detail,
        posinset,
        setsize,
    } = row;
    let expanded = group.map(|group| (!group.collapsed).to_string());
    let fold = group.and_then(|group| {
        let href = group.toggle.clone()?;
        let (glyph, text) = if group.collapsed {
            (Glyph::CHEVRON_RIGHT, Text::TopologyExpand)
        } else {
            (Glyph::CHEVRON_DOWN, Text::TopologyCollapse)
        };
        let name = tr_with(text, &[("title", card.name.clone().into())]);
        Some(view! {
            <a href=href class="fui-tree-fold" aria-label=name tabindex="-1">
                {icon(glyph, "14")}
            </a>
        })
    });
    let name = if card.mono {
        view! { <code>{card.name.clone()}</code> }.into_any()
    } else if level == 1 {
        view! { <strong>{card.name.clone()}</strong> }.into_any()
    } else {
        card.name.clone().into_any()
    };
    let action = card.action.clone().map(|action| {
        view! {
            <form method="post" action=action.action>
                <input type="hidden" name=action.name value=action.value />
                <button type="submit" class=Variant::Ghost.small()>
                    {icon(action.glyph, "14")}
                    {action.label}
                </button>
            </form>
        }
    });
    let class = match level {
        1 => "fui-tree-row",
        2 => "fui-tree-row fui-tree-l2",
        _ => "fui-tree-row fui-tree-l3",
    };
    let muted = card.state == FlowState::Off;
    view! {
        <li
            role="treeitem"
            aria-expanded=expanded
            class=class
            class:is-excluded=muted
        >
            <span class="fui-tree-name">
                {fold}
                {icon(card.glyph, "16")}
                <a href=card.href.clone()>{name}</a>
            </span>
            <span>{state_word(card.state, detail)}</span>
            <span class="fui-small fui-muted">{card.measure.clone()}</span>
            <span class="fui-small fui-muted">{card.note.clone()}</span>
            <span class="fui-tree-act">{action}</span>
        </li>
    }
    // Leptos names no typed `aria-level`, `aria-posinset` or `aria-setsize`
    // attribute, so they are set by name.
    .attr("aria-level", level.to_string())
    .attr("aria-posinset", posinset.to_string())
    .attr("aria-setsize", setsize.to_string())
    .into_any()
}

/// The List twin: the same graph as an indented tree, the hub first, then
/// each lane with its groups and their children, folded as the map is.
fn list(topology: &Topology) -> AnyView {
    let mut rows = vec![item_row(Row {
        level: 1,
        card: &topology.hub,
        group: None,
        detail: None,
        posinset: 1,
        setsize: 1,
    })];
    if let Some(callers) = &topology.callers {
        rows.push(lane_row(&callers.title, &callers.line));
        let size = callers.cards.len();
        for (index, card) in callers.cards.iter().enumerate() {
            rows.push(item_row(Row {
                level: 2,
                card,
                group: None,
                detail: None,
                posinset: index + 1,
                setsize: size,
            }));
        }
    }
    for side in [&topology.start, &topology.end].into_iter().flatten() {
        rows.push(lane_row(&side.title, &side.line));
        let size = side.groups.len();
        for (index, group) in side.groups.iter().enumerate() {
            rows.push(item_row(Row {
                level: 2,
                card: &group.card,
                group: Some(group),
                detail: group.card.edge_label.clone(),
                posinset: index + 1,
                setsize: size,
            }));
            if group.collapsed {
                continue;
            }
            let count = group.children.len();
            for (place, child) in group.children.iter().enumerate() {
                rows.push(item_row(Row {
                    level: 3,
                    card: child,
                    group: None,
                    detail: child
                        .edge_label
                        .clone()
                        .filter(|_| child.state.is_problem()),
                    posinset: place + 1,
                    setsize: count,
                }));
            }
        }
    }
    view! {
        <ul role="tree" aria-label=topology.list_label.clone() class="fui-topology-list">
            {rows}
        </ul>
    }
    .into_any()
}

/// Returns the topology `topology` inside the island that adds zoom,
/// panning and the keys.
///
/// It draws the toolbar with the Status menu and the view switch, the map
/// with its zoom cluster, minimap and quick look, and its List twin.
///
/// Put it on its own page under the page header; the map takes the width
/// of the column.
#[must_use]
pub fn topology(mut topology: Topology) -> AnyView {
    let class = topology.view.class().to_owned();
    let status = status_menu(topology.status.clone());
    let tools = topology.tools.take();
    let updated = topology.updated.take();
    let parts = MapParts {
        collapse_all: topology.collapse_all.take(),
        minimap: topology.minimap.take(),
        look: topology.look.take(),
    };
    let switch = view_switch(topology.view, topology.views.clone());
    let list = list(&topology);
    let map = map(&topology, &layout(&topology), parts);
    view! {
        <crate::island::TopologyKeys class words=TopologyWords::of_page()>
            <div class="fui-topology-bar">
                <div class="fui-topology-bar-start">{status} {tools}</div>
                <div class="fui-topology-bar-end">{updated} {switch}</div>
            </div>
            <div class="fui-topology-mapwrap">{map}</div>
            <div class="fui-topology-listwrap">{list}</div>
        </crate::island::TopologyKeys>
    }
    .into_any()
}
