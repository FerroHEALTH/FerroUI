// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! What the browser adds to the topology (F9): zoom by the cluster and the
//! keys, panning by the pointer, the quick look on "i" or Space, and the
//! arrows along the List twin.
//!
//! The topology is server HTML ([`crate::view::topology`]): every card a
//! link, every fold a link and the Status menu a form, so it works with no
//! WebAssembly. Its island wraps that HTML and listens on it. The zoom
//! cluster's buttons arrive disabled, saying why, and are enabled once the
//! island runs (`leptos-ui.md` §15.8). Zoom is the drawing's width in CSS
//! pixels per unit, written as the drawing's `width` and `height`
//! attributes, never a style (§0); panning is the canvas's own scroll. No
//! specification governs the behaviour: our own design; the List's keys
//! follow the WAI-ARIA Authoring Practices tree view pattern.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Phrase, Text, tr_with};

/// The words the island writes as the scale changes, resolved on the
/// server.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TopologyWords {
    /// The readout, such as "125 %", its `value` left open.
    pub readout: Phrase,
    /// The readout's accessible name, such as "Zoom 125 %", its `value`
    /// left open.
    pub level: Phrase,
}

impl TopologyWords {
    /// The words in the page's locale.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            readout: Phrase::new(tr_with(
                Text::TopologyZoom,
                &[("value", Phrase::open("value"))],
            )),
            level: Phrase::new(tr_with(
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

/// Returns the topology's wrapper with the class `class` around the
/// server's `children`, listening for the zoom, the pan and the keys.
/// A console draws it through [`crate::view::topology::topology`].
#[must_use]
pub fn topology_keys(class: String, words: TopologyWords, children: AnyView) -> AnyView {
    let root = NodeRef::<leptos::html::Div>::new();
    let drag = StoredValue::new(None::<(i32, i32, i32, i32)>);
    let scale = StoredValue::new(None::<f64>);
    #[cfg(target_arch = "wasm32")]
    let ready_words = words.clone();
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if let Some(root) = root.get() {
            browser::ready(&root);
            browser::zoom(&root, scale, &ready_words, Ask::Fit);
        }
    });
    let keys_words = words.clone();
    view! {
        <div
            class=class
            node_ref=root
            on:click=move |event| browser::clicked(root, scale, &words, &event)
            on:keydown=move |event| browser::pressed(root, scale, &keys_words, &event)
            on:mousedown=move |event| browser::grabbed(root, drag, &event)
            on:mousemove=move |event| browser::dragged(root, drag, &event)
            on:mouseup=move |_| drag.set_value(None)
            on:mouseleave=move |_| drag.set_value(None)
            on:scroll:capture=move |_| browser::framed(root)
        >
            {children}
        </div>
    }
    .into_any()
}

/// The browser's side: every call reads and writes the page's DOM, and
/// does nothing on the server.
mod browser {
    use leptos::prelude::*;

    use super::TopologyWords;

    /// The topology's root element.
    type Root = NodeRef<leptos::html::Div>;

    /// Where a pan started: the pointer's `x` and `y` and the canvas's
    /// scroll left and top.
    type Hold = StoredValue<Option<(i32, i32, i32, i32)>>;

    /// Enables the zoom cluster's buttons once the island runs.
    #[cfg(target_arch = "wasm32")]
    pub(super) fn ready(root: &web_sys::HtmlDivElement) {
        if let Ok(buttons) = root.query_selector_all("[data-zoom]") {
            for index in 0..buttons.length() {
                if let Some(button) = buttons.item(index).and_then(|node| {
                    leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::Element>(node).ok()
                }) {
                    let titled = match button.get_attribute("data-title") {
                        Some(title) => button.set_attribute("title", &title),
                        None => button.remove_attribute("title"),
                    };
                    if titled.is_err() || button.remove_attribute("disabled").is_err() {
                        leptos::logging::warn!("the topology could not enable its zoom");
                    }
                }
            }
        }
    }

    /// Zooms as the cluster's button under `event` asks.
    pub(super) fn clicked(
        root: Root,
        scale: StoredValue<Option<f64>>,
        words: &TopologyWords,
        event: &leptos::ev::MouseEvent,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast as _;
            let Some(root) = root.get_untracked() else {
                return;
            };
            let Some(button) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .and_then(|target| target.closest("[data-zoom]").ok().flatten())
            else {
                return;
            };
            let ask = match button.get_attribute("data-zoom").as_deref() {
                Some("in") => super::Ask::ZoomIn,
                Some("out") => super::Ask::ZoomOut,
                Some("fit") => super::Ask::Fit,
                _ => return,
            };
            zoom(&root, scale, words, ask);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (
            Root,
            StoredValue<Option<f64>>,
            &TopologyWords,
            &leptos::ev::MouseEvent,
        ) = (root, scale, words, event);
    }

    /// Answers a key: the zoom keys on the canvas, "i" and Space on a card,
    /// and the arrows in the List.
    pub(super) fn pressed(
        root: Root,
        scale: StoredValue<Option<f64>>,
        words: &TopologyWords,
        event: &leptos::ev::KeyboardEvent,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast as _;
            let Some(root) = root.get_untracked() else {
                return;
            };
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };
            if event.ctrl_key() || event.meta_key() || event.alt_key() {
                return;
            }
            let key = event.key();
            if target
                .closest(".fui-topology-list")
                .ok()
                .flatten()
                .is_some()
            {
                if along(&root, &target, &key) {
                    event.prevent_default();
                }
                return;
            }
            if target
                .closest(".fui-topology-scroll")
                .ok()
                .flatten()
                .is_none()
            {
                return;
            }
            let on_card = target.matches(".fui-tnode-link").unwrap_or(false);
            match super::asked(&key.to_ascii_lowercase(), on_card) {
                Some(super::Ask::Look) => {
                    let look = target
                        .parent_element()
                        .and_then(|card| card.query_selector(".fui-tnode-info a").ok().flatten())
                        .and_then(|link| link.get_attribute("href"));
                    if let Some(look) = look {
                        event.prevent_default();
                        crate::widget::submit::follow(&look);
                    }
                }
                Some(ask) => {
                    event.prevent_default();
                    zoom(&root, scale, words, ask);
                }
                None => {}
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (
            Root,
            StoredValue<Option<f64>>,
            &TopologyWords,
            &leptos::ev::KeyboardEvent,
        ) = (root, scale, words, event);
    }

    /// Moves the focus along the List's rows for `key`, from the row that
    /// holds `target`; Left goes to the row's parent. Whether it moved.
    #[cfg(target_arch = "wasm32")]
    fn along(root: &web_sys::HtmlDivElement, target: &web_sys::Element, key: &str) -> bool {
        use leptos::wasm_bindgen::JsCast as _;
        let Ok(links) =
            root.query_selector_all(".fui-topology-list [role='treeitem'] .fui-tree-name > a")
        else {
            return false;
        };
        let rows: Vec<web_sys::HtmlElement> = (0..links.length())
            .filter_map(|index| links.item(index))
            .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
            .collect();
        let Some(at) = rows.iter().position(|row| {
            let node: &web_sys::Node = row;
            node.is_same_node(Some(target))
        }) else {
            return false;
        };
        let to = if key == "ArrowLeft" {
            let level = |row: &web_sys::HtmlElement| {
                row.closest("[role='treeitem']")
                    .ok()
                    .flatten()
                    .and_then(|item| item.get_attribute("aria-level"))
                    .and_then(|level| level.parse::<u8>().ok())
                    .unwrap_or(1)
            };
            let own = rows.get(at).map_or(1, level);
            rows.get(..at)
                .and_then(|before| before.iter().rposition(|row| level(row) < own))
        } else {
            super::moved(key, at, rows.len())
        };
        match to.and_then(|to| rows.get(to)) {
            Some(row) => row.focus().is_ok(),
            None => false,
        }
    }

    /// Applies `ask` to the drawing: one step in or out, or the canvas's
    /// width, then writes the readout and the minimap's frame.
    #[cfg(target_arch = "wasm32")]
    pub(super) fn zoom(
        root: &web_sys::HtmlDivElement,
        scale: StoredValue<Option<f64>>,
        words: &TopologyWords,
        ask: super::Ask,
    ) {
        let Some(stage) = root.query_selector(".fui-pipeline-stage").ok().flatten() else {
            return;
        };
        let size = |name: &str| {
            stage
                .get_attribute(name)
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(1.0)
        };
        let (width, height) = (size("data-width"), size("data-height"));
        let fitted = stage
            .query_selector(":scope > svg")
            .ok()
            .flatten()
            .map_or(1.0, |svg| f64::from(svg.client_width()) / width);
        let now = scale.get_value().unwrap_or(fitted);
        let next = match ask {
            super::Ask::ZoomIn => Some(super::stepped(now, true)),
            super::Ask::ZoomOut => Some(super::stepped(now, false)),
            super::Ask::Fit | super::Ask::Look => None,
        };
        scale.set_value(next);
        let written = stage
            .class_list()
            .toggle_with_force("is-zoomed", next.is_some())
            .is_ok();
        if let Ok(drawings) = stage.query_selector_all(":scope > svg") {
            for index in 0..drawings.length() {
                let Some(svg) = drawings.item(index).and_then(|node| {
                    leptos::wasm_bindgen::JsCast::dyn_into::<web_sys::Element>(node).ok()
                }) else {
                    continue;
                };
                let shown = next.unwrap_or(1.0);
                let done = svg
                    .set_attribute("width", &format!("{:.0}", width * shown))
                    .and_then(|()| svg.set_attribute("height", &format!("{:.0}", height * shown)));
                if done.is_err() || !written {
                    leptos::logging::warn!("the topology could not zoom");
                }
            }
        }
        let shown = next.unwrap_or_else(|| {
            stage
                .query_selector(":scope > svg")
                .ok()
                .flatten()
                .map_or(1.0, |svg| f64::from(svg.client_width()) / width)
        });
        let percent = format!("{:.0}", shown * 100.0);
        if let Some(readout) = root.query_selector("[data-zoom-readout]").ok().flatten() {
            let said = words.readout.fill(&[("value", &percent)]);
            readout.set_text_content(Some(said.text()));
            let lang = if said.fell_back() {
                readout.set_attribute("lang", "en")
            } else {
                readout.remove_attribute("lang")
            };
            if lang.is_err()
                || readout
                    .set_attribute(
                        "aria-label",
                        words.level.fill(&[("value", &percent)]).text(),
                    )
                    .is_err()
            {
                leptos::logging::warn!("the topology could not name its zoom");
            }
        }
        frame(root);
    }

    /// Moves the minimap's frame to what the canvas shows.
    #[cfg(target_arch = "wasm32")]
    fn frame(root: &web_sys::HtmlDivElement) {
        let Some(view) = root.query_selector(".fui-minimap-view").ok().flatten() else {
            return;
        };
        let Some(scroll) = root.query_selector(".fui-topology-scroll").ok().flatten() else {
            return;
        };
        let Some(stage) = root.query_selector(".fui-pipeline-stage").ok().flatten() else {
            return;
        };
        let width = stage
            .get_attribute("data-width")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(1.0);
        let height = stage
            .get_attribute("data-height")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(1.0);
        let mini = view
            .get_attribute("data-scale")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(1.0);
        let shown = stage
            .query_selector(":scope > svg")
            .ok()
            .flatten()
            .map_or(1.0, |svg| f64::from(svg.client_width().max(1)) / width);
        let left = f64::from(scroll.scroll_left()) / shown;
        let top = f64::from(scroll.scroll_top()) / shown;
        let wide = (f64::from(scroll.client_width()) / shown).min(width);
        let tall = (f64::from(scroll.client_height()) / shown).min(height);
        for (name, value) in [("x", left), ("y", top), ("width", wide), ("height", tall)] {
            if view
                .set_attribute(name, &format!("{:.1}", value * mini))
                .is_err()
            {
                leptos::logging::warn!("the topology could not move its minimap");
            }
        }
    }

    /// Moves the minimap's frame as the canvas scrolls.
    pub(super) fn framed(root: Root) {
        #[cfg(target_arch = "wasm32")]
        if let Some(root) = root.get_untracked() {
            frame(&root);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: Root = root;
    }

    /// Takes hold of the canvas for a pan, when the main button goes down
    /// on it and not on a link or a control.
    pub(super) fn grabbed(root: Root, drag: Hold, event: &leptos::ev::MouseEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast as _;
            if event.button() != 0 || root.get_untracked().is_none() {
                return;
            }
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };
            if target.closest("a, button").ok().flatten().is_some() {
                return;
            }
            let Some(scroll) = target.closest(".fui-topology-scroll").ok().flatten() else {
                return;
            };
            drag.set_value(Some((
                event.client_x(),
                event.client_y(),
                scroll.scroll_left(),
                scroll.scroll_top(),
            )));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (Root, Hold, &leptos::ev::MouseEvent) = (root, drag, event);
    }

    /// Pans the canvas while it is held.
    pub(super) fn dragged(root: Root, drag: Hold, event: &leptos::ev::MouseEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            let Some((x, y, left, top)) = drag.get_value() else {
                return;
            };
            let Some(scroll) = root
                .get_untracked()
                .and_then(|root| root.query_selector(".fui-topology-scroll").ok().flatten())
            else {
                return;
            };
            event.prevent_default();
            scroll.set_scroll_left(left.saturating_sub(event.client_x().saturating_sub(x)));
            scroll.set_scroll_top(top.saturating_sub(event.client_y().saturating_sub(y)));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (Root, Hold, &leptos::ev::MouseEvent) = (root, drag, event);
    }
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
