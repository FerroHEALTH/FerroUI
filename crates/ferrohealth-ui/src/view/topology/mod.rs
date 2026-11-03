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

mod draw;
mod layout;
mod list;
mod tools;

use serde::{Deserialize, Serialize};

use crate::html::Html;
use crate::i18n::{Locale, Phrase, Text, tr_with};
use crate::icon::Glyph;
use crate::view::flow::FlowState;

use draw::{MapParts, map};
use layout::layout;
use list::list;
use tools::{status_menu, view_switch};

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
