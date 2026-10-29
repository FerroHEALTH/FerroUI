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
//! Every page works with no script: a card is a link to its page, its
//! quick look a link, a group's collapse a link, the Status menu a `GET`
//! form, and the List twin is the same graph as an indented tree, the
//! phone's view and the screen reader's. The `topology` module adds zoom
//! by the cluster and the keys ([`stepped`], [`asked`]), panning by the
//! pointer, the quick look on "i" or Space, and the arrows along the List
//! ([`moved`]). The zoom cluster's buttons arrive disabled, saying why,
//! and the module enables them. Zoom is the drawing's width in CSS pixels
//! per unit, written as the drawing's `width` and `height` attributes,
//! never a style; panning is the canvas's own scroll. No specification
//! governs the drawing: our own design; the List twin and its keys follow
//! the WAI-ARIA Authoring Practices tree view pattern.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Arg, Locale, Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::table::filter::{FilterChoice, FilterMenu, MenuForm, filter_menu};
use crate::view::button::Variant;
use crate::view::flow::{
    CardDrawing, FlowState, LabelAt, Place, card_drawing, curve, edge_line, ends, line_sample,
    path_drawing, units,
};

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
    pub body: Html,
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
    pub tools: Option<Html>,
    /// The console's own words at the toolbar's end, such as when the map
    /// was read.
    pub updated: Option<Html>,
    /// Collapse every group, or `None` where nothing folds.
    pub collapse_all: Option<ClusterLink>,
    /// The minimap's switch, or `None` for a map that has none; while it is
    /// on, the minimap shows.
    pub minimap: Option<ClusterLink>,
    /// The quick look that is open, if any.
    pub look: Option<QuickLook>,
}

/// The words the `topology` module writes as the scale changes, resolved
/// on the server.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TopologyWords {
    /// The readout, such as "125 %", its `value` left open.
    pub readout: Phrase,
    /// The readout's accessible name, such as "Zoom 125 %", its `value`
    /// left open.
    pub level: Phrase,
}

impl TopologyWords {
    /// The words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            readout: Phrase::new(tr_with(
                locale,
                Text::TopologyZoom,
                &[("value", Phrase::open("value"))],
            )),
            level: Phrase::new(tr_with(
                locale,
                Text::TopologyZoomLevel,
                &[("value", Phrase::open("value"))],
            )),
        }
    }
}

/// The scales the zoom steps through, in CSS pixels per unit.
pub const STEPS: [f64; 9] = [0.25, 0.33, 0.5, 0.67, 0.8, 1.0, 1.25, 1.5, 2.0];

/// The scale one step in (`inward`) or out from `scale`: the next of
/// [`STEPS`] past it, held at the first and the last.
#[must_use]
pub fn stepped(scale: f64, inward: bool) -> f64 {
    let next = if inward {
        STEPS.iter().copied().find(|step| *step > scale + 0.005)
    } else {
        STEPS
            .iter()
            .rev()
            .copied()
            .find(|step| *step < scale - 0.005)
    };
    let held = if inward { STEPS.last() } else { STEPS.first() };
    next.or_else(|| held.copied()).unwrap_or(scale)
}

/// What a key pressed in the topology asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    /// One step in.
    ZoomIn,
    /// One step out.
    ZoomOut,
    /// The whole drawing in the canvas's width.
    Fit,
    /// The quick look of the card with the focus.
    Look,
}

/// What the key `key` asks for: on the canvas `+` and `=` zoom in, `-`
/// zooms out and `0` fits; on a card "i" and Space open its quick look.
#[must_use]
pub fn asked(key: &str, on_card: bool) -> Option<Ask> {
    match key {
        "+" | "=" => Some(Ask::ZoomIn),
        "-" | "_" => Some(Ask::ZoomOut),
        "0" => Some(Ask::Fit),
        "i" | " " if on_card => Some(Ask::Look),
        _ => None,
    }
}

/// The row the arrows move to in the List from row `at` of `count`:
/// down and up one, Home the first, End the last, held at the ends.
#[must_use]
pub fn moved(key: &str, at: usize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match key {
        "ArrowDown" => Some((at + 1).min(last)),
        "ArrowUp" => Some(at.saturating_sub(1)),
        "Home" => Some(0),
        "End" => Some(last),
        _ => None,
    }
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

/// The accessible name of `card` in `locale`, the hub's without a state.
fn card_name(locale: &Locale, card: &TopologyCard, hub: bool) -> String {
    String::from(if hub {
        tr_with(
            locale,
            Text::FlowHub,
            &[
                ("title", card.name.clone().into()),
                ("note", card.line.clone().into()),
            ],
        )
    } else {
        tr_with(
            locale,
            Text::FlowCard,
            &[
                ("title", card.name.clone().into()),
                ("note", card.line.clone().into()),
                ("status", card.state.word(locale).into()),
            ],
        )
    })
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
        label: card_name(locale, card, hub),
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

/// The minimap of `layout`: a rectangle per card in its tone, and the
/// frame of what the map shows, the whole drawing at first. A tall
/// federation narrows it to keep it inside its tallest measure.
fn minimap(locale: &Locale, layout: &Layout<'_>) -> Html {
    let scale = (MINIMAP / layout.width.max(1.0)).min(MINIMAP_TALL / layout.height.max(1.0));
    let (width, height) = (layout.width * scale, layout.height * scale);
    let rects: Vec<String> = layout
        .cards
        .iter()
        .map(|laid| {
            format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"1\" class=\"fui-minimap-node {}\"></rect>",
                laid.place.x * scale,
                laid.place.y * scale,
                laid.place.width * scale,
                (laid.place.height * scale).max(2.0),
                laid.card.state.tone().class()
            )
        })
        .collect();
    let (outer_width, outer_height) = (width + 8.0, height + 8.0);
    Html::raw(format!(
        "<div role=\"img\" aria-label=\"{}\" class=\"fui-topology-minimap\"><svg viewBox=\"-4 -4 {outer_width:.0} {outer_height:.0}\" width=\"{outer_width:.0}\" height=\"{outer_height:.0}\" aria-hidden=\"true\">{}<rect data-scale=\"{scale:.6}\" x=\"0\" y=\"0\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"2\" class=\"fui-minimap-view\"></rect></svg></div>",
        Html::text(tr(locale, Text::TopologyMinimapFrame).text()),
        rects.concat()
    ))
}

/// One icon button of the zoom cluster with its tooltip above it, waiting
/// for the `topology` module, which `wait` says.
fn cluster_button(glyph: Glyph, words: &Word, zoom: &'static str, wait: &Word) -> String {
    format!(
        "<span class=\"fui-anchor\"><button type=\"button\" aria-label=\"{}\" data-zoom=\"{zoom}\" disabled title=\"{}\" class=\"fui-icon-button\">{}</button><span role=\"tooltip\" class=\"fui-tooltip fui-tooltip-above\">{words}</span></span>",
        Html::text(words.text()),
        Html::text(wait.text()),
        icon(glyph, "16")
    )
}

/// One link of the zoom cluster that the server answers, pressed while on.
fn cluster_link(glyph: Glyph, words: &Word, link: &ClusterLink) -> String {
    format!(
        "<span class=\"fui-anchor\"><a href=\"{}\" aria-label=\"{}\" aria-pressed=\"{}\" class=\"fui-icon-button\">{}</a><span role=\"tooltip\" class=\"fui-tooltip fui-tooltip-above\">{words}</span></span>",
        Html::text(&link.href),
        Html::text(words.text()),
        link.on,
        icon(glyph, "16")
    )
}

/// The zoom cluster: zoom in and out, the readout and Fit, which wait for
/// the `topology` module, then the server's links.
fn cluster(
    locale: &Locale,
    collapse_all: Option<&ClusterLink>,
    minimap: Option<&ClusterLink>,
) -> String {
    let wait = tr(locale, Text::TopologyZoomWait);
    let level = tr_with(
        locale,
        Text::TopologyZoomLevel,
        &[("value", Arg::from(100))],
    );
    let readout = tr_with(locale, Text::TopologyZoom, &[("value", Arg::from(100))]);
    let links = collapse_all.is_some() || minimap.is_some();
    let collapse = collapse_all.map_or_else(String::new, |link| {
        let (glyph, text) = if link.on {
            (Glyph::CHEVRON_DOWN, Text::TopologyExpandAll)
        } else {
            (Glyph::CHEVRON_UP, Text::TopologyCollapseAll)
        };
        cluster_link(glyph, &tr(locale, text), link)
    });
    let minimap = minimap.map_or_else(String::new, |link| {
        cluster_link(Glyph::LAYERS, &tr(locale, Text::TopologyMinimap), link)
    });
    let sep = if links {
        "<span aria-hidden=\"true\" class=\"fui-topology-zoom-sep\"></span>"
    } else {
        ""
    };
    format!(
        "<div role=\"toolbar\" aria-label=\"{controls}\" class=\"fui-topology-zoom\">{zoom_in}{zoom_out}<span role=\"status\" aria-label=\"{level}\" data-zoom-readout=\"\" class=\"fui-topology-zoom-level\">{readout}</span><button type=\"button\" data-zoom=\"fit\" disabled title=\"{wait_text}\" data-title=\"{fit_tip}\" class=\"{ghost}\">{fit}</button>{sep}{collapse}{minimap}</div>",
        controls = Html::text(tr(locale, Text::TopologyControls).text()),
        zoom_in = cluster_button(Glyph::PLUS, &tr(locale, Text::TopologyZoomIn), "in", &wait),
        zoom_out = cluster_button(
            Glyph::MINUS,
            &tr(locale, Text::TopologyZoomOut),
            "out",
            &wait
        ),
        level = Html::text(level.text()),
        wait_text = Html::text(wait.text()),
        fit_tip = Html::text(tr(locale, Text::TopologyFitWhole).text()),
        ghost = Variant::Ghost.small(),
        fit = tr(locale, Text::TopologyFit),
    )
}

/// The health dot of `state` with its word as its accessible name; a state
/// switched off has no shape.
fn state_dot(locale: &Locale, state: FlowState) -> String {
    let word = state.word(locale);
    if state == FlowState::Off {
        return format!("<span class=\"fui-sr-only\">{word}</span>");
    }
    let text = Html::text(word.text());
    format!(
        "<span role=\"img\" aria-label=\"{text}\" title=\"{text}\" class=\"fui-dot {}\"></span>",
        state.tone().class()
    )
}

/// A state as its shape and its word (F12), with a quiet detail after it;
/// a state switched off draws no shape, as its muted row says.
fn state_word(locale: &Locale, state: FlowState, detail: Option<String>) -> Html {
    let word = state.word(locale);
    if state == FlowState::Off {
        let detail = detail.map_or_else(String::new, |detail| {
            format!(
                " <span class=\"fui-status-detail\">{}</span>",
                Html::text(&detail)
            )
        });
        return Html::raw(format!(
            "<span class=\"fui-state fui-tone-neutral is-muted\">{word}{detail}</span>"
        ));
    }
    crate::view::state::state(state.into(), word, detail)
}

/// The Status menu: the filter menu of the states, each its line, its dot,
/// its word and its count.
fn status_menu(locale: &Locale, menu: &StatusMenu) -> Html {
    let choices = menu
        .states
        .iter()
        .map(|each| {
            FilterChoice::new(
                each.state.key(),
                each.state.word(locale),
                u64::from(each.count),
                each.shown,
            )
            .mark(Html::raw(format!(
                "{}{}",
                line_sample(each.state),
                state_dot(locale, each.state)
            )))
        })
        .collect();
    filter_menu(
        locale,
        FilterMenu {
            id: "topology-show",
            name: &menu.name,
            glyph: Glyph::FUNNEL,
            label: tr(locale, Text::TopologyStatus),
            title: tr(locale, Text::TopologyStates).text().to_owned(),
            choices,
            note: Some(tr(locale, Text::TopologyLegend)),
            form: MenuForm::Own {
                action: Some(&menu.action),
                fields: &[],
            },
        },
    )
}

/// The Map and List switch: two links, the view in force pressed, and in
/// the automatic view the stylesheet presses the List below 48 rem.
fn view_switch(
    locale: &Locale,
    view_in_force: TopologyView,
    (map, list): &(String, String),
) -> String {
    let map_on = view_in_force != TopologyView::List;
    let class = if view_in_force == TopologyView::Auto {
        "fui-segmented fui-view-switch fui-view-auto"
    } else {
        "fui-segmented fui-view-switch"
    };
    format!(
        "<div role=\"group\" aria-label=\"{label}\" class=\"{class}\"><a href=\"{map}\" aria-pressed=\"{map_on}\" class=\"fui-segment fui-view-map\">{map_glyph}{map_words}</a><a href=\"{list}\" aria-pressed=\"{list_on}\" class=\"fui-segment fui-view-list\">{list_glyph}{list_words}</a></div>",
        label = Html::text(tr(locale, Text::TopologyView).text()),
        map = Html::text(map),
        list = Html::text(list),
        list_on = !map_on,
        map_glyph = icon(Glyph::NETWORK, "14"),
        map_words = tr(locale, Text::TopologyViewMap),
        list_glyph = icon(Glyph::LIST_CHECKS, "14"),
        list_words = tr(locale, Text::TopologyViewList),
    )
}

/// The quick look beside the canvas.
fn quick_look(locale: &Locale, look: QuickLook) -> String {
    let QuickLook {
        card: _,
        title,
        mono,
        line,
        close,
        body,
    } = look;
    let title = if mono {
        format!("<code>{}</code>", Html::text(&title))
    } else {
        Html::text(&title).to_string()
    };
    let line = line.map_or_else(String::new, |line| {
        format!("<p class=\"fui-small fui-muted\">{}</p>", Html::text(&line))
    });
    let close_words = Html::text(tr(locale, Text::TopologyLookClose).text());
    format!(
        "<aside aria-labelledby=\"topology-look-title\" class=\"fui-topology-panel\"><div class=\"fui-side-panel-head\"><div><h2 id=\"topology-look-title\" class=\"fui-card-title\">{title}</h2>{line}</div><a href=\"{close}\" aria-label=\"{close_words}\" title=\"{close_words}\" class=\"fui-icon-button\">{x}</a></div><div class=\"fui-side-panel-body fui-stack\">{body}</div></aside>",
        close = Html::text(&close),
        x = icon(Glyph::X, "16"),
        body = String::from(body),
    )
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
fn map(locale: &Locale, topology: &Topology, layout: &Layout<'_>, parts: MapParts) -> String {
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
fn lane_row(title: &str, line: &str) -> String {
    format!(
        "<li role=\"presentation\" class=\"fui-tree-lane\"><span>{}</span><span class=\"fui-small fui-muted\">{}</span></li>",
        Html::text(title),
        Html::text(line)
    )
}

#[derive(Template)]
#[template(
    source = r#"<li role="treeitem"{% if let Some(expanded) = expanded %} aria-expanded="{{ expanded }}"{% endif %} aria-level="{{ level }}" aria-posinset="{{ posinset }}" aria-setsize="{{ setsize }}" class="{{ class }}{% if muted %} is-excluded{% endif %}"><span class="fui-tree-name">{% if let Some(fold) = fold %}{{ fold }}{% endif %}{{ glyph }}<a href="{{ href }}">{{ name }}</a></span><span>{{ state }}</span><span class="fui-small fui-muted">{% if let Some(measure) = measure %}{{ measure }}{% endif %}</span><span class="fui-small fui-muted">{% if let Some(note) = note %}{{ note }}{% endif %}</span><span class="fui-tree-act">{% if let Some(action) = action %}<form method="post" action="{{ action.action }}"><input type="hidden" name="{{ action.name }}" value="{{ action.value }}"><button type="submit" class="{{ ghost }}">{{ action_glyph }}{{ action.label }}</button></form>{% endif %}</span></li>"#,
    ext = "html"
)]
struct ItemView<'a> {
    expanded: Option<bool>,
    level: u8,
    posinset: usize,
    setsize: usize,
    class: &'static str,
    muted: bool,
    fold: Option<Html>,
    glyph: Html,
    href: &'a str,
    name: Html,
    state: Html,
    measure: Option<&'a str>,
    note: Option<&'a str>,
    action: Option<&'a RowAction>,
    action_glyph: Html,
    ghost: &'static str,
}

/// One item row of the List: the name with its glyph as a link to its
/// page, a group's fold as a link, the state, the measure, the note and
/// the action.
fn item_row(locale: &Locale, row: Row<'_>) -> String {
    let Row {
        level,
        card,
        group,
        detail,
        posinset,
        setsize,
    } = row;
    let fold = group.and_then(|group| {
        let href = group.toggle.as_deref()?;
        let (glyph, text) = if group.collapsed {
            (Glyph::CHEVRON_RIGHT, Text::TopologyExpand)
        } else {
            (Glyph::CHEVRON_DOWN, Text::TopologyCollapse)
        };
        let name = tr_with(locale, text, &[("title", card.name.clone().into())]);
        Some(Html::raw(format!(
            "<a href=\"{}\" aria-label=\"{}\" tabindex=\"-1\" class=\"fui-tree-fold\">{}</a>",
            Html::text(href),
            Html::text(name.text()),
            icon(glyph, "14")
        )))
    });
    let name = if card.mono {
        Html::raw(format!("<code>{}</code>", Html::text(&card.name)))
    } else if level == 1 {
        Html::raw(format!("<strong>{}</strong>", Html::text(&card.name)))
    } else {
        Html::text(&card.name)
    };
    let class = match level {
        1 => "fui-tree-row",
        2 => "fui-tree-row fui-tree-l2",
        _ => "fui-tree-row fui-tree-l3",
    };
    ItemView {
        expanded: group.map(|group| !group.collapsed),
        level,
        posinset,
        setsize,
        class,
        muted: card.state == FlowState::Off,
        fold,
        glyph: icon(card.glyph, "16"),
        href: &card.href,
        name,
        state: state_word(locale, card.state, detail),
        measure: card.measure.as_deref(),
        note: card.note.as_deref(),
        action: card.action.as_ref(),
        action_glyph: card
            .action
            .as_ref()
            .map_or_else(Html::default, |action| icon(action.glyph, "14")),
        ghost: Variant::Ghost.small(),
    }
    .html()
    .into()
}

/// The List twin: the same graph as an indented tree, the hub first, then
/// each lane with its groups and their children, folded as the map is.
fn list(locale: &Locale, topology: &Topology) -> String {
    let mut rows = vec![item_row(
        locale,
        Row {
            level: 1,
            card: &topology.hub,
            group: None,
            detail: None,
            posinset: 1,
            setsize: 1,
        },
    )];
    if let Some(callers) = &topology.callers {
        rows.push(lane_row(&callers.title, &callers.line));
        let size = callers.cards.len();
        for (index, card) in callers.cards.iter().enumerate() {
            rows.push(item_row(
                locale,
                Row {
                    level: 2,
                    card,
                    group: None,
                    detail: None,
                    posinset: index + 1,
                    setsize: size,
                },
            ));
        }
    }
    for side in [&topology.start, &topology.end].into_iter().flatten() {
        rows.push(lane_row(&side.title, &side.line));
        let size = side.groups.len();
        for (index, group) in side.groups.iter().enumerate() {
            rows.push(item_row(
                locale,
                Row {
                    level: 2,
                    card: &group.card,
                    group: Some(group),
                    detail: group.card.edge_label.clone(),
                    posinset: index + 1,
                    setsize: size,
                },
            ));
            if group.collapsed {
                continue;
            }
            let count = group.children.len();
            for (place, child) in group.children.iter().enumerate() {
                rows.push(item_row(
                    locale,
                    Row {
                        level: 3,
                        card: child,
                        group: None,
                        detail: child
                            .edge_label
                            .clone()
                            .filter(|_| child.state.is_problem()),
                        posinset: place + 1,
                        setsize: count,
                    },
                ));
            }
        }
    }
    format!(
        "<ul role=\"tree\" aria-label=\"{}\" class=\"fui-topology-list\">{}</ul>",
        Html::text(&topology.list_label),
        rows.concat()
    )
}

/// Returns the topology `topology` in `locale`, wrapped for the `topology`
/// module that adds zoom, panning and the keys.
///
/// It draws the toolbar with the Status menu and the view switch, the map
/// with its zoom cluster, minimap and quick look, and its List twin.
///
/// Put it on its own page under the page header; the map takes the width
/// of the column.
#[must_use]
pub fn topology(locale: &Locale, mut topology: Topology) -> Html {
    let status = status_menu(locale, &topology.status);
    let tools = topology.tools.take().unwrap_or_default();
    let updated = topology.updated.take().unwrap_or_default();
    let parts = MapParts {
        collapse_all: topology.collapse_all.take(),
        minimap: topology.minimap.take(),
        look: topology.look.take(),
    };
    let switch = view_switch(locale, topology.view, &topology.views);
    let list = list(locale, &topology);
    let map = map(locale, &topology, &layout(&topology), parts);
    // NOTE: serde_json `to_string` fails only for a map with a key that is no
    // string, and the words hold none.
    let words = serde_json::to_string(&TopologyWords::of(locale)).unwrap_or_default();
    Html::raw(format!(
        "<div data-fui=\"topology\" data-fui-words=\"{words}\" class=\"{class}\"><div class=\"fui-topology-bar\"><div class=\"fui-topology-bar-start\">{status}{tools}</div><div class=\"fui-topology-bar-end\">{updated}{switch}</div></div><div class=\"fui-topology-mapwrap\">{map}</div><div class=\"fui-topology-listwrap\">{list}</div></div>",
        words = Html::text(&words),
        class = topology.view.class(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{Ask, STEPS, asked, moved, stepped};

    #[test]
    fn a_step_moves_to_the_next_scale_and_holds_at_the_ends() {
        assert!((stepped(1.0, true) - 1.25).abs() < f64::EPSILON);
        assert!((stepped(1.0, false) - 0.8).abs() < f64::EPSILON);
        assert!(
            (stepped(0.71, true) - 0.8).abs() < f64::EPSILON,
            "a fitted scale between two steps goes to the next"
        );
        assert!((stepped(0.71, false) - 0.67).abs() < f64::EPSILON);
        assert!((stepped(2.0, true) - STEPS[STEPS.len() - 1]).abs() < f64::EPSILON);
        assert!((stepped(0.25, false) - STEPS[0]).abs() < f64::EPSILON);
    }

    #[test]
    fn the_keys_zoom_on_the_canvas_and_open_the_quick_look_on_a_card() {
        assert_eq!(Some(Ask::ZoomIn), asked("+", false));
        assert_eq!(Some(Ask::ZoomIn), asked("=", true));
        assert_eq!(Some(Ask::ZoomOut), asked("-", false));
        assert_eq!(Some(Ask::Fit), asked("0", false));
        assert_eq!(Some(Ask::Look), asked("i", true));
        assert_eq!(Some(Ask::Look), asked(" ", true));
        assert_eq!(None, asked(" ", false), "Space off a card scrolls");
        assert_eq!(None, asked("x", true));
    }

    #[test]
    fn the_arrows_walk_the_list_and_hold_at_its_ends() {
        assert_eq!(Some(1), moved("ArrowDown", 0, 3));
        assert_eq!(Some(2), moved("ArrowDown", 2, 3));
        assert_eq!(Some(0), moved("ArrowUp", 0, 3));
        assert_eq!(Some(0), moved("Home", 2, 3));
        assert_eq!(Some(2), moved("End", 0, 3));
        assert_eq!(None, moved("Enter", 0, 3));
        assert_eq!(None, moved("ArrowDown", 0, 0));
    }
}
