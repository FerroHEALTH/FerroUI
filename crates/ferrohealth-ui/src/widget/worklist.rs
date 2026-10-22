// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The work-list: a dense listing a steward walks with the keyboard, a
//! focused row moved by `j`, `k` and the arrow keys, Enter to open it, and
//! a foot of actions that stays in view below the rows (R9).
//!
//! The rows are the server's: a data table whose rows [`row`] draws, each
//! with its `data-key`, the first one focused, inside a list card with the
//! foot [`worklist_foot`] draws. [`worklist`] wraps that table in a
//! console's island, or a page draws the ready island
//! [`crate::island::WorkList`], whose Enter follows the row's link. With no
//! WebAssembly the table is a table: every row's link opens it, every
//! checkbox posts with its form.
//!
//! Once hydrated the table is a grid (WAI-ARIA Authoring Practices 1.2, the
//! grid pattern) with a roving tabindex: one tab stop for the whole list,
//! the focused row's, which carries `aria-current="true"`; the controls of
//! the other rows leave the tab order, and those of the focused row follow
//! it. `j` and Down move to the next row, `k` and Up to the one before,
//! Home and End to the first and the last, and Enter opens the row; the
//! focused row is scrolled clear of the sticky header and of the foot
//! (WCAG 2.2 SC 2.4.11). The grid role arrives with the keys that make it
//! one. A header checkbox selects every row and shows the mixed state while
//! some are selected; each selected row carries `aria-selected`.

use leptos::prelude::*;

/// What a work-list moves through.
#[derive(Debug, Clone, Copy)]
pub struct WorkList {
    /// The key of the focused row, which the widget keeps in step; `None`
    /// until a row takes the focus, when the server's first row leads.
    pub focused: RwSignal<Option<String>>,
    /// Runs with the focused row's key when Enter opens it.
    pub on_open: Callback<String>,
}

/// One row of a work-list.
#[derive(Debug)]
pub struct WorkRow<'a> {
    /// The row's key, its `data-key`: an id, never a record.
    pub key: &'a str,
    /// Whether the row is the one focused when the page is drawn: the
    /// first row, so the server's page and the hydrated one agree.
    pub focused: bool,
    /// The row's class, such as [`crate::table::view::EXCLUDED`].
    pub class: Option<&'static str>,
    /// The row's cells, drawn with the cell helpers of
    /// [`crate::table::view`].
    pub cells: AnyView,
}

/// Returns a row of a work-list: its key, and the tab stop and
/// `aria-current` when it is the focused row.
#[must_use]
pub fn row(row: WorkRow<'_>) -> AnyView {
    let WorkRow {
        key,
        focused,
        class,
        cells,
    } = row;
    view! {
        <tr
            data-key=key.to_owned()
            tabindex=if focused { "0" } else { "-1" }
            aria-current=focused.then_some("true")
        >
            {cells}
        </tr>
    }
    .attr("class", class)
    .into_any()
}

/// Returns the foot of a work-list: the shortcut hint at its start and the
/// pager at its end, in view below the rows inside the card.
#[must_use]
pub fn worklist_foot(hint: Option<AnyView>, pager: Option<AnyView>) -> AnyView {
    view! { <div class="fui-worklist-foot">{hint} {pager}</div> }.into_any()
}

/// The row a key moves the focus to from `current` of `count`, or `None`
/// for a key the work-list does not take; the ends do not wrap.
#[must_use]
pub fn moved(key: &str, current: usize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match key {
        "j" | "ArrowDown" => Some(current.saturating_add(1).min(last)),
        "k" | "ArrowUp" => Some(current.saturating_sub(1).min(last)),
        "Home" => Some(0),
        "End" => Some(last),
        _ => None,
    }
}

/// How much of a page's rows is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// No row.
    None,
    /// Some rows, not every one: the header checkbox is mixed.
    Some,
    /// Every row.
    All,
}

/// The selection of `checked` of `count` rows.
#[must_use]
pub const fn selection(checked: usize, count: usize) -> Selection {
    if checked == 0 || count == 0 {
        Selection::None
    } else if checked >= count {
        Selection::All
    } else {
        Selection::Some
    }
}

/// Returns the work-list `list` around `table`, the data table the server
/// drew with its rows from [`row`].
///
/// A console calls it inside its own island, the table and the foot its
/// server children; `on_open` runs on Enter.
#[must_use]
pub fn worklist(list: WorkList, table: AnyView) -> AnyView {
    let WorkList { focused, on_open } = list;
    let root = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| browser::start(root, focused));
    let on_key =
        move |event: leptos::ev::KeyboardEvent| browser::key(root, focused, on_open, &event);
    let on_focus = move |event: leptos::ev::FocusEvent| browser::focus_in(root, focused, &event);
    let on_change = move |event: leptos::ev::Event| browser::changed(root, &event);
    view! {
        <div class="fui-worklist-grid" node_ref=root on:keydown=on_key on:focusin=on_focus on:change=on_change>
            {table}
        </div>
    }
    .into_any()
}

/// Opens the row `key` by following its row link, the ready island's
/// Enter.
pub(crate) fn follow(key: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let selector = format!("tr[data-key=\"{}\"] a.fui-row-link", key.replace('"', ""));
        if let Some(link) = document()
            .query_selector(&selector)
            .ok()
            .flatten()
            .and_then(|link| link.dyn_into::<web_sys::HtmlElement>().ok())
        {
            link.click();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = key;
}

/// The browser half: the rows are the island's server children, out of
/// reach of a node reference, so they are found under the island's root.
#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use leptos::wasm_bindgen::JsCast as _;
    use web_sys::{Element, HtmlElement, HtmlInputElement};

    use super::{Selection, moved, selection};

    /// The controls a row holds, which leave the tab order with their row.
    const CONTROLS: &str = "a[href], button, input, select, textarea";

    /// Sets the attribute `name` of `element` to `value`, or removes it.
    fn set(element: &Element, name: &str, value: Option<&str>) {
        let done = match value {
            Some(value) => element.set_attribute(name, value),
            None => element.remove_attribute(name),
        };
        if done.is_err() {
            leptos::logging::warn!("the browser refused an attribute of the work-list");
        }
    }

    /// The island's root element.
    fn root_of(root: NodeRef<leptos::html::Div>) -> Option<Element> {
        root.get_untracked().map(Into::into)
    }

    /// Every element under `root` that `selector` matches, in order.
    fn all(root: &Element, selector: &str) -> Vec<Element> {
        let Ok(list) = root.query_selector_all(selector) else {
            return Vec::new();
        };
        (0..list.length())
            .filter_map(|index| list.item(index))
            .filter_map(|node| node.dyn_into::<Element>().ok())
            .collect()
    }

    /// The rows of the work-list.
    fn rows(root: &Element) -> Vec<Element> {
        all(root, "tbody tr[data-key]")
    }

    /// Makes row `index` the focused one: its tab stop, its controls in
    /// the tab order, `aria-current`; every other row leaves the order.
    fn apply(rows: &[Element], index: usize) {
        for (at, row) in rows.iter().enumerate() {
            let on = at == index;
            set(row, "tabindex", Some(if on { "0" } else { "-1" }));
            if on {
                set(row, "aria-current", Some("true"));
            } else {
                set(row, "aria-current", None);
            }
            for control in all(row, CONTROLS) {
                if on {
                    set(&control, "tabindex", None);
                } else {
                    set(&control, "tabindex", Some("-1"));
                }
            }
        }
    }

    /// Turns the table into the grid once the island has hydrated.
    pub(super) fn start(root: NodeRef<leptos::html::Div>, focused: RwSignal<Option<String>>) {
        let Some(root) = root_of(root) else {
            return;
        };
        if let Ok(Some(table)) = root.query_selector("table") {
            set(&table, "role", Some("grid"));
        }
        if let Ok(Some(region)) = root.query_selector(".fui-table-scroll") {
            set(&region, "tabindex", Some("-1"));
        }
        let rows = rows(&root);
        let wanted = focused.get_untracked();
        let index = rows
            .iter()
            .position(|row| row.get_attribute("data-key") == wanted)
            .or_else(|| {
                rows.iter()
                    .position(|row| row.has_attribute("aria-current"))
            })
            .unwrap_or(0);
        apply(&rows, index);
        if let Some(header) = header(&root) {
            header.set_hidden(false);
        }
        sync(&root);
    }

    /// The header checkbox that selects every row.
    fn header(root: &Element) -> Option<HtmlInputElement> {
        root.query_selector("thead .fui-select-all")
            .ok()
            .flatten()
            .and_then(|input| input.dyn_into::<HtmlInputElement>().ok())
    }

    /// The rows' checkboxes.
    fn checks(root: &Element) -> Vec<HtmlInputElement> {
        all(root, "tbody td.fui-row-check input[type=checkbox]")
            .into_iter()
            .filter_map(|input| input.dyn_into::<HtmlInputElement>().ok())
            .collect()
    }

    /// Brings the header checkbox and each row's `aria-selected` into step
    /// with the rows' checkboxes.
    fn sync(root: &Element) {
        let checks = checks(root);
        if checks.is_empty() {
            return;
        }
        let checked = checks.iter().filter(|input| input.checked()).count();
        for input in &checks {
            if let Ok(Some(row)) = input.closest("tr") {
                let selected = if input.checked() { "true" } else { "false" };
                set(&row, "aria-selected", Some(selected));
            }
        }
        if let Some(header) = header(root) {
            let state = selection(checked, checks.len());
            header.set_checked(state == Selection::All);
            header.set_indeterminate(state == Selection::Some);
        }
    }

    /// Selects every row or none as the header checkbox says, or follows a
    /// row's checkbox.
    pub(super) fn changed(root: NodeRef<leptos::html::Div>, event: &leptos::ev::Event) {
        let Some(root) = root_of(root) else {
            return;
        };
        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        if target.class_list().contains("fui-select-all") {
            for input in checks(&root) {
                input.set_checked(target.checked());
            }
        }
        sync(&root);
    }

    /// Keeps the focused row in step when the focus lands in a row by the
    /// pointer or by Tab.
    pub(super) fn focus_in(
        root: NodeRef<leptos::html::Div>,
        focused: RwSignal<Option<String>>,
        event: &leptos::ev::FocusEvent,
    ) {
        let Some(root) = root_of(root) else {
            return;
        };
        let Some(row) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
            .and_then(|target| target.closest("tbody tr[data-key]").ok().flatten())
        else {
            return;
        };
        let rows = rows(&root);
        if let Some(index) = rows.iter().position(|each| *each == row)
            && !row.has_attribute("aria-current")
        {
            apply(&rows, index);
        }
        focused.set(row.get_attribute("data-key"));
    }

    /// Moves the focused row, or opens it, as the key `event` asks.
    pub(super) fn key(
        root: NodeRef<leptos::html::Div>,
        focused: RwSignal<Option<String>>,
        on_open: Callback<String>,
        event: &leptos::ev::KeyboardEvent,
    ) {
        if event.ctrl_key() || event.alt_key() || event.meta_key() {
            return;
        }
        let Some(root) = root_of(root) else {
            return;
        };
        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
            .filter(|target| target.tag_name().eq_ignore_ascii_case("tr"))
        else {
            return;
        };
        let rows = rows(&root);
        let Some(current) = rows.iter().position(|row| *row == target) else {
            return;
        };
        let key = event.key();
        if key == "Enter" {
            event.prevent_default();
            if let Some(open) = target.get_attribute("data-key") {
                on_open.run(open);
            }
            return;
        }
        let Some(next) = moved(&key, current, rows.len()) else {
            return;
        };
        event.prevent_default();
        apply(&rows, next);
        let Some(row) = rows.get(next) else {
            return;
        };
        focused.set(row.get_attribute("data-key"));
        if let Ok(element) = row.clone().dyn_into::<HtmlElement>()
            && element.focus().is_err()
        {
            leptos::logging::warn!("the browser refused the focus on a work-list row");
        }
        clear(&root, row);
    }

    /// Scrolls `row` clear of the sticky header above it and of the foot
    /// below it (WCAG 2.2 SC 2.4.11).
    fn clear(root: &Element, row: &Element) {
        let rect = row.get_bounding_client_rect();
        let view = document()
            .document_element()
            .map_or(0.0, |html| f64::from(html.client_height()));
        let top = root
            .query_selector("thead")
            .ok()
            .flatten()
            .map_or(0.0, |head| {
                head.get_bounding_client_rect().bottom().max(0.0)
            });
        let bottom = root
            .closest(".fui-worklist")
            .ok()
            .flatten()
            .and_then(|card| card.query_selector(".fui-worklist-foot").ok().flatten())
            .map_or(view, |foot| foot.get_bounding_client_rect().top().min(view));
        if rect.top() < top {
            window().scroll_by_with_x_and_y(0.0, rect.top() - top);
        } else if rect.bottom() > bottom {
            window().scroll_by_with_x_and_y(0.0, rect.bottom() - bottom);
        }
    }
}

/// The host half: the server draws the table as it is.
#[cfg(not(target_arch = "wasm32"))]
mod browser {
    use leptos::prelude::*;

    /// Nothing to start on the server.
    pub(super) fn start(root: NodeRef<leptos::html::Div>, focused: RwSignal<Option<String>>) {
        let _: (NodeRef<leptos::html::Div>, RwSignal<Option<String>>) = (root, focused);
    }

    /// No key reaches the server.
    pub(super) fn key(
        root: NodeRef<leptos::html::Div>,
        focused: RwSignal<Option<String>>,
        on_open: Callback<String>,
        event: &leptos::ev::KeyboardEvent,
    ) {
        let _: (
            NodeRef<leptos::html::Div>,
            RwSignal<Option<String>>,
            Callback<String>,
        ) = (root, focused, on_open);
        let _: &leptos::ev::KeyboardEvent = event;
    }

    /// No focus reaches the server.
    pub(super) fn focus_in(
        root: NodeRef<leptos::html::Div>,
        focused: RwSignal<Option<String>>,
        event: &leptos::ev::FocusEvent,
    ) {
        let _: (NodeRef<leptos::html::Div>, RwSignal<Option<String>>) = (root, focused);
        let _: &leptos::ev::FocusEvent = event;
    }

    /// No change reaches the server.
    pub(super) fn changed(root: NodeRef<leptos::html::Div>, event: &leptos::ev::Event) {
        let _: NodeRef<leptos::html::Div> = root;
        let _: &leptos::ev::Event = event;
    }
}

#[cfg(test)]
mod tests {
    use super::{Selection, moved, selection};

    #[test]
    fn the_keys_move_one_row_and_stop_at_the_ends() {
        assert_eq!(Some(1), moved("j", 0, 3));
        assert_eq!(Some(1), moved("ArrowDown", 0, 3));
        assert_eq!(Some(2), moved("j", 2, 3));
        assert_eq!(Some(0), moved("k", 0, 3));
        assert_eq!(Some(1), moved("ArrowUp", 2, 3));
        assert_eq!(Some(0), moved("Home", 2, 3));
        assert_eq!(Some(2), moved("End", 0, 3));
        assert_eq!(None, moved("x", 0, 3));
        assert_eq!(None, moved("j", 0, 0));
    }

    #[test]
    fn some_rows_selected_is_the_mixed_state() {
        assert_eq!(Selection::None, selection(0, 4));
        assert_eq!(Selection::Some, selection(2, 4));
        assert_eq!(Selection::All, selection(4, 4));
        assert_eq!(Selection::None, selection(0, 0));
    }
}
