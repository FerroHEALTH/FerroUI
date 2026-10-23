// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The repeatable rows (F7): a list of the same few fields, each row
//! removable by its own labelled button at its end, and one Add button
//! under the list.
//!
//! A row's Remove is its last control; on a phone the fields stack and
//! Remove stays on the row's first line, beside the first field. A row the
//! list keeps ends in a disabled lock, hidden from a reader, and says why
//! in a visible line under it, never in a tooltip alone (F13's rule for a
//! disabled item). M4's
//! filter rows and M6's target rows are F7 rows ([`Fields::Three`]). A
//! row of one field ([`Fields::One`]) keeps the field and its Remove on
//! one line, and a row of many fields is a sub-card ([`Fields::Card`]): its
//! heading with Remove beside it, then its fields, a list of its own among
//! them when it has one.
//!
//! Remove and Add are submit buttons of the console's form, so with no
//! WebAssembly a press posts the form with the button's `name` and `value`
//! and the console answers with the list one row shorter or longer
//! (`leptos-ui.md` §15.8). The island [`crate::island::Repeater`] does both
//! in place: Remove takes its row out and moves the focus to the next row's
//! first control (the previous row's at the end, Add when none is left),
//! and Add inserts a copy of the blank row, its key token
//! ([`Repeat::token`], [`ROW`] by default) replaced by a fresh key, and
//! focuses its first control. A sub-card's first control is its first
//! field, never the Remove in its head. A list nested in a sub-card has a
//! token of its own, so a copied card's nested list adds its rows in place
//! too. Either way the console reads the rows the form posts as the whole
//! list. No specification governs the layout: our own design.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// The key token of a list's blank row, which the island replaces in the
/// row's every id, name and key with `new-1`, `new-2` and so on, the first
/// one no row of the list holds.
///
/// A list nested in a [`Fields::Card`] row names a token of its own in
/// [`Repeat::token`], one that neither holds nor is held in the token of
/// the list around it.
pub const ROW: &str = "__row__";

/// How a row's fields share its width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fields {
    /// One field across the row, such as a profile's URL or one value of a
    /// list setting.
    One,
    /// Two fields, the second the wider, such as a name and its value.
    #[default]
    Two,
    /// Three fields, such as a filter's property, operator and value, or a
    /// mapping's target, relationship and comment.
    Three,
    /// A sub-card: the row's heading ([`RepeatRow::heading`]) with Remove
    /// beside it, then its fields as a form lays them out, such as an
    /// endpoint with its id, its base URL, a switch and a list of its own.
    Card,
}

impl Fields {
    /// The classes of a row.
    const fn row(self) -> &'static str {
        match self {
            Self::One => "fui-repeater-row fui-repeater-one",
            Self::Two => "fui-repeater-row",
            Self::Three => "fui-clause-filter",
            Self::Card => "fui-repeater-card",
        }
    }

    /// The classes of the list of rows.
    const fn rows(self) -> &'static str {
        match self {
            Self::One | Self::Two | Self::Card => "fui-repeat-rows fui-repeater",
            Self::Three => "fui-repeat-rows",
        }
    }
}

/// What ends a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowEnd {
    /// Its Remove: an icon button named `label`, such as "Remove this
    /// filter", that submits `name` with `value`.
    Remove {
        /// The button's name, which says which row it removes.
        label: String,
        /// The field it submits as.
        name: String,
        /// The value it submits, the row's key.
        value: String,
    },
    /// A row the list keeps: a disabled lock at its end and `reason` as a
    /// visible line under it, such as a property the editor writes itself.
    Kept {
        /// Why the row cannot be removed.
        reason: String,
    },
}

/// One row of the list.
#[derive(Debug)]
pub struct RepeatRow {
    /// The row's key, stable for the row and unique in the list.
    pub key: String,
    /// The heading of a [`Fields::Card`] row, such as "Query API endpoint
    /// node-c-pub", which names its group; the other layouts draw none.
    pub heading: Option<String>,
    /// The row's fields, from [`crate::view::form::Field`] with ids made
    /// from the key.
    pub fields: AnyView,
    /// What ends the row.
    pub end: RowEnd,
    /// A line under the row, such as its error, which goes with it.
    pub after: Option<AnyView>,
}

/// The Add button under the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatAdd {
    /// Its words, such as "Add filter".
    pub label: String,
    /// The field it submits as.
    pub name: String,
    /// The value it submits.
    pub value: String,
}

/// What a list of repeatable rows shows.
#[derive(Debug)]
pub struct Repeat {
    /// The list's id.
    pub id: String,
    /// The list's name, such as "Filters of the include".
    pub label: String,
    /// How each row's fields share its width.
    pub fields: Fields,
    /// The headings over the fields of [`Fields::One`] or [`Fields::Two`]
    /// rows whose labels are hidden, one per field; empty for rows whose
    /// fields show their labels.
    pub head: Vec<String>,
    /// The rows, in order.
    pub rows: Vec<RepeatRow>,
    /// The key token of the blank row: [`ROW`], or a token of its own for
    /// a list nested in a [`Fields::Card`] row, such as `__profile__`.
    pub token: &'static str,
    /// The blank row Add inserts with WebAssembly, its key, ids and names
    /// holding [`Repeat::token`]; `None` leaves Add to the form post.
    pub blank: Option<RepeatRow>,
    /// The Add button, if the list grows here.
    pub add: Option<RepeatAdd>,
    /// The quiet line the list shows with no row, such as "No filters
    /// yet."
    pub empty: String,
    /// The id of the form Remove and Add submit, when the list is not
    /// inside it.
    pub form: Option<String>,
}

/// Returns the end of a row.
fn row_end(end: RowEnd, form: Option<String>) -> AnyView {
    match end {
        RowEnd::Remove { label, name, value } => {
            let title = label.clone();
            view! {
                <button
                    type="submit"
                    class="fui-icon-button"
                    name=name
                    value=value
                    form=form
                    formnovalidate=true
                    aria-label=label
                    title=title
                    data-repeat="remove"
                >
                    {icon(Glyph::X, "16")}
                </button>
            }
            .into_any()
        }
        RowEnd::Kept { .. } => view! {
            <button type="button" class="fui-icon-button" disabled=true aria-hidden="true">
                {icon(Glyph::LOCK, "16")}
            </button>
        }
        .into_any(),
    }
}

/// Returns the row `row` of the list `list`, laid out as `fields`.
fn row(list: &str, row: RepeatRow, fields: Fields, form: Option<String>) -> AnyView {
    let RepeatRow {
        key,
        heading,
        fields: inside,
        end,
        after,
    } = row;
    let kept = match &end {
        RowEnd::Kept { reason } => {
            Some(view! { <p class="fui-help fui-repeat-kept">{reason.clone()}</p> })
        }
        RowEnd::Remove { .. } => None,
    };
    let end = row_end(end, form);
    let body = if fields == Fields::Card {
        let heading_id = format!("{list}-{key}-heading");
        let labelled_by = heading_id.clone();
        view! {
            <div class=fields.row() role="group" aria-labelledby=labelled_by>
                <div class="fui-repeater-card-head">
                    <h3 id=heading_id>{heading}</h3>
                    {end}
                </div>
                {inside}
            </div>
        }
        .into_any()
    } else {
        view! { <div class=fields.row()>{inside} {end}</div> }.into_any()
    };
    view! {
        <div class="fui-repeat-item" data-row=key>
            {body}
            {kept}
            {after}
        </div>
    }
    .into_any()
}

/// Returns the list `repeat`: its rows, the blank row the island copies,
/// the quiet line for no row and Add, inside the island
/// [`crate::island::Repeater`].
#[must_use]
pub fn repeat(repeat: Repeat) -> AnyView {
    let Repeat {
        id,
        label,
        fields,
        head,
        rows,
        token,
        blank,
        add,
        empty,
        form,
    } = repeat;
    let none = rows.is_empty();
    let head = (matches!(fields, Fields::One | Fields::Two) && !head.is_empty()).then(|| {
        let labels = head
            .into_iter()
            .map(|words| view! { <span class="fui-label">{words}</span> })
            .collect_view();
        view! {
            <div class="fui-repeater-head" class:is-one=fields == Fields::One aria-hidden="true">
                {labels}
            </div>
        }
    });
    let drawn = rows
        .into_iter()
        .map(|each| row(&id, each, fields, form.clone()))
        .collect_view();
    let blank = blank.map(|each| {
        let inside = row(&id, each, fields, form.clone());
        view! { <template data-repeat="blank" data-token=token>{inside}</template> }
    });
    let add = add.map(|RepeatAdd { label, name, value }| {
        view! {
            <div class="fui-add-row">
                <button
                    type="submit"
                    class=Variant::Ghost.small()
                    name=name
                    value=value
                    form=form.clone()
                    formnovalidate=true
                    data-repeat="add"
                >
                    {icon(Glyph::PLUS, "16")}
                    {label}
                </button>
            </div>
        }
    });
    view! {
        <crate::island::Repeater id label>
            {head}
            <div class=fields.rows() data-repeat="rows">
                {drawn}
            </div>
            <p class="fui-transform-empty" data-repeat="empty" hidden=!none>
                {empty}
            </p>
            {blank}
            {add}
        </crate::island::Repeater>
    }
    .into_any()
}

/// Returns the list `id` named `label` around `inside`, the parts
/// [`repeat`] drew on the server, with Remove and Add working in place.
#[must_use]
pub fn repeater(id: String, label: String, inside: AnyView) -> AnyView {
    view! {
        <div id=id class="fui-repeat" role="group" aria-label=label on:click=|event| browser::clicked(&event)>
            {inside}
        </div>
    }
    .into_any()
}

/// The key the island gives the next row Add inserts: `new-` and the
/// first count from 1 that `taken` does not hold.
#[must_use]
pub fn fresh_key(taken: impl Fn(&str) -> bool) -> String {
    let mut count: u32 = 1;
    loop {
        let key = format!("new-{count}");
        if !taken(&key) {
            return key;
        }
        match count.checked_add(1) {
            Some(next) => count = next,
            None => return key,
        }
    }
}

/// What the browser code does. Each step runs in the click handler, so the
/// server and the hydrated page draw the same.
mod browser {
    #[cfg(target_arch = "wasm32")]
    use leptos::wasm_bindgen::JsCast as _;

    /// The selector of a row's controls, the first of which takes the
    /// focus.
    #[cfg(target_arch = "wasm32")]
    const CONTROLS: &str = "input:not([type=hidden]):not([disabled]),select:not([disabled]),textarea:not([disabled]),button:not([disabled])";

    /// The marker of a list inside a row the island inserted: the browser
    /// never hydrates it, so the list around it takes its presses.
    #[cfg(target_arch = "wasm32")]
    const COPIED: &str = "data-repeat-copied";

    /// The nearest list around `element`, if it has one.
    #[cfg(target_arch = "wasm32")]
    fn list_of(element: &web_sys::Element) -> Option<web_sys::Element> {
        element.closest(".fui-repeat").ok().flatten()
    }

    /// The list whose handler takes a press inside `list`: `list` itself,
    /// or for a list in a copied row the nearest list around it the browser
    /// hydrated.
    #[cfg(target_arch = "wasm32")]
    fn handler_of(list: &web_sys::Element) -> Option<web_sys::Element> {
        let mut at = list.clone();
        while at.has_attribute(COPIED) {
            at = at.parent_element().as_ref().and_then(list_of)?;
        }
        Some(at)
    }

    /// The elements matching `selector` inside `list` that belong to it,
    /// leaving out those of the lists nested in its rows.
    #[cfg(target_arch = "wasm32")]
    fn own(list: &web_sys::Element, selector: &str) -> Vec<web_sys::Element> {
        let Ok(found) = list.query_selector_all(selector) else {
            return Vec::new();
        };
        (0..found.length())
            .filter_map(|index| found.item(index))
            .filter_map(|node| node.dyn_into::<web_sys::Element>().ok())
            .filter(|element| list_of(element).is_some_and(|nearest| &nearest == list))
            .collect()
    }

    /// The first element matching `selector` that belongs to `list`.
    #[cfg(target_arch = "wasm32")]
    fn own_first(list: &web_sys::Element, selector: &str) -> Option<web_sys::Element> {
        own(list, selector).into_iter().next()
    }

    /// Takes a press on Remove or Add inside the list.
    pub(super) fn clicked(event: &leptos::ev::MouseEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            let Some(root) = event
                .current_target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };
            let Some(button) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .and_then(|target| target.closest("[data-repeat]").ok().flatten())
            else {
                return;
            };
            // A list nested in a sub-card takes its own presses, unless it
            // sits in a row this island inserted.
            let Some(list) = list_of(&button) else {
                return;
            };
            if handler_of(&list).is_none_or(|handler| handler != root) {
                return;
            }
            match button.get_attribute("data-repeat").as_deref() {
                Some("remove") => {
                    event.prevent_default();
                    remove(&list, &button);
                }
                Some("add") if add(&list) => event.prevent_default(),
                _ => {}
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: &leptos::ev::MouseEvent = event;
    }

    /// The row element `from` or the nearest row before or after it, as
    /// `step` walks.
    #[cfg(target_arch = "wasm32")]
    fn sibling_row(
        from: &web_sys::Element,
        step: fn(&web_sys::Element) -> Option<web_sys::Element>,
    ) -> Option<web_sys::Element> {
        let mut at = step(from);
        while let Some(each) = at {
            if each.matches(".fui-repeat-item").unwrap_or(false) {
                return Some(each);
            }
            at = step(&each);
        }
        None
    }

    /// Moves the focus to `control`.
    #[cfg(target_arch = "wasm32")]
    fn focus(control: web_sys::Element) -> bool {
        let Ok(control) = control.dyn_into::<web_sys::HtmlElement>() else {
            return false;
        };
        if control.focus().is_err() {
            leptos::logging::warn!("the browser refused the focus on a repeatable row");
        }
        true
    }

    /// Moves the focus to the first control of the row `row`: its first
    /// field, a sub-card's head left out, or its Remove when it has none.
    #[cfg(target_arch = "wasm32")]
    fn focus_row(row: &web_sys::Element) -> bool {
        let Ok(found) = row.query_selector_all(CONTROLS) else {
            return false;
        };
        let controls: Vec<web_sys::Element> = (0..found.length())
            .filter_map(|index| found.item(index))
            .filter_map(|node| node.dyn_into::<web_sys::Element>().ok())
            .collect();
        let in_head = |control: &web_sys::Element| {
            control
                .closest(".fui-repeater-card-head")
                .ok()
                .flatten()
                .is_some()
        };
        let first = controls
            .iter()
            .find(|control| !in_head(control))
            .or_else(|| controls.first())
            .cloned();
        first.is_some_and(focus)
    }

    /// Shows the quiet line when the list `root` holds no row, and hides it
    /// otherwise.
    #[cfg(target_arch = "wasm32")]
    fn mark_empty(root: &web_sys::Element) {
        let Some(empty) = own_first(root, "[data-repeat='empty']") else {
            return;
        };
        let any = !own(root, "[data-repeat='rows'] > .fui-repeat-item").is_empty();
        let done = if any {
            empty.set_attribute("hidden", "")
        } else {
            empty.remove_attribute("hidden")
        };
        if done.is_err() {
            leptos::logging::warn!("the browser refused to mark the empty list");
        }
    }

    /// Takes out the row of the Remove `button` and moves the focus on.
    #[cfg(target_arch = "wasm32")]
    fn remove(root: &web_sys::Element, button: &web_sys::Element) {
        let Some(item) = button.closest(".fui-repeat-item").ok().flatten() else {
            return;
        };
        let next = sibling_row(&item, web_sys::Element::next_element_sibling)
            .or_else(|| sibling_row(&item, web_sys::Element::previous_element_sibling));
        item.remove();
        mark_empty(root);
        let moved = next.is_some_and(|row| focus_row(&row));
        if !moved && let Some(add) = own_first(root, "[data-repeat='add']") {
            focus(add);
        }
    }

    /// Replaces `token` with `key` in every attribute of every element of
    /// `copy`, the content of its nested templates included: its key, its
    /// ids, its names and the labels that name them.
    #[cfg(target_arch = "wasm32")]
    fn rekey(copy: &web_sys::DocumentFragment, token: &str, key: &str) {
        let Ok(found) = copy.query_selector_all("*") else {
            return;
        };
        let elements = (0..found.length())
            .filter_map(|index| found.item(index))
            .filter_map(|node| node.dyn_into::<web_sys::Element>().ok());
        for element in elements {
            let attributes = element.attributes();
            for at in 0..attributes.length() {
                if let Some(attribute) = attributes.item(at) {
                    let value = attribute.value();
                    if value.contains(token) {
                        attribute.set_value(&value.replace(token, key));
                    }
                }
            }
            if let Some(template) = element.dyn_ref::<web_sys::HtmlTemplateElement>() {
                rekey(&template.content(), token, key);
            }
        }
    }

    /// Marks every list inside `copy` as one in a copied row, whose
    /// presses the list around it takes.
    #[cfg(target_arch = "wasm32")]
    fn mark_copied(copy: &web_sys::DocumentFragment) {
        let Ok(found) = copy.query_selector_all(".fui-repeat") else {
            return;
        };
        let lists = (0..found.length())
            .filter_map(|index| found.item(index))
            .filter_map(|node| node.dyn_into::<web_sys::Element>().ok());
        for list in lists {
            if list.set_attribute(COPIED, "").is_err() {
                leptos::logging::warn!("the browser refused to mark a copied list");
            }
        }
    }

    /// Inserts a copy of the blank row with a fresh key and focuses it;
    /// `false` when the list has no blank row, so the form posts.
    #[cfg(target_arch = "wasm32")]
    fn add(root: &web_sys::Element) -> bool {
        let (Some(blank), Some(rows)) = (
            own_first(root, "template[data-repeat='blank']"),
            own_first(root, "[data-repeat='rows']"),
        ) else {
            return false;
        };
        let token = blank
            .get_attribute("data-token")
            .unwrap_or_else(|| String::from(super::ROW));
        let key = super::fresh_key(|key| !own(root, &format!("[data-row='{key}']")).is_empty());
        let Some(copy) = blank
            .dyn_ref::<web_sys::HtmlTemplateElement>()
            .and_then(|template| template.content().clone_node_with_deep(true).ok())
            .and_then(|node| node.dyn_into::<web_sys::DocumentFragment>().ok())
        else {
            return false;
        };
        rekey(&copy, &token, &key);
        mark_copied(&copy);
        if rows.append_child(&copy).is_err() {
            leptos::logging::warn!("the browser refused to add a repeatable row");
            return false;
        }
        mark_empty(root);
        if let Some(added) = rows.last_element_child() {
            focus_row(&added);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::fresh_key;

    #[test]
    fn a_fresh_key_is_the_first_count_no_row_holds() {
        assert_eq!("new-1", fresh_key(|_| false));
        assert_eq!("new-3", fresh_key(|key| key == "new-1" || key == "new-2"));
        assert_eq!("new-2", fresh_key(|key| key == "new-1" || key == "new-3"));
    }
}
