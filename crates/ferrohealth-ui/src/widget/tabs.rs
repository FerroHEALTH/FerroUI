// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keyboard tabs that switch panels in place, and the phone's select
//! over binder tabs.
//!
//! The keyboard tabs are binder tabs joined to their panel, as
//! [`crate::view::tabs::binder`] draws them, and follow WAI-ARIA Authoring
//! Practices 1.2, the tabs pattern with automatic activation: one tab stop,
//! Left and Right move between the tabs and wrap, Home and End go to the
//! first and the last, and the panel of the selected tab is shown. Before
//! the page hydrates the first panel shows and the others are hidden; the
//! link tabs of [`crate::view::tabs`] keep their state in the URL instead,
//! and are what a page uses where every panel must be reachable with no
//! WebAssembly.
//!
//! The panels are drawn on the server by [`tab_panels`], every one of them,
//! the ones not chosen hidden, and handed to the island
//! [`crate::island::KeyboardTabs`] as its children; the island draws the
//! tab list and shows the panel of the tab chosen. The panels' content never
//! reaches the WebAssembly.

use leptos::prelude::*;

/// The id of tab `index` of the tabs `prefix`.
fn tab_id(prefix: &str, index: usize) -> String {
    format!("{prefix}-tab-{index}")
}

/// The id of panel `index` of the tabs `prefix`.
fn panel_id(prefix: &str, index: usize) -> String {
    format!("{prefix}-panel-{index}")
}

/// The tab a key moves to from `current` of `count`, or `None` for a key
/// the tabs do not take.
#[must_use]
pub fn moved(key: &str, current: usize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match key {
        "ArrowRight" => Some(if current >= last { 0 } else { current + 1 }),
        "ArrowLeft" => Some(if current == 0 { last } else { current - 1 }),
        "Home" => Some(0),
        "End" => Some(last),
        _ => None,
    }
}

/// Returns the panels of the keyboard tabs `prefix`, one per body in
/// `bodies`, the first shown and the others hidden.
#[must_use]
pub fn tab_panels(prefix: &str, bodies: Vec<AnyView>) -> AnyView {
    bodies
        .into_iter()
        .enumerate()
        .map(|(index, body)| {
            view! {
                <div
                    role="tabpanel"
                    id=panel_id(prefix, index)
                    class="fui-tab-panel"
                    aria-labelledby=tab_id(prefix, index)
                    hidden=index != 0
                    tabindex="0"
                >
                    {body}
                </div>
            }
        })
        .collect_view()
        .into_any()
}

/// Returns the tab list named `label` of the keyboard tabs `prefix`, one tab
/// per word in `tabs`, above `panels`, which [`tab_panels`] drew.
///
/// Choosing a tab shows its panel and hides the others. A console draws it
/// as the island [`crate::island::KeyboardTabs`].
#[must_use]
pub fn tab_list(label: String, prefix: String, tabs: Vec<String>, panels: AnyView) -> AnyView {
    let selected = RwSignal::new(0_usize);
    let count = tabs.len();
    let prefix = StoredValue::new(prefix);
    let list = NodeRef::<leptos::html::Div>::new();
    let on_key = move |event: leptos::ev::KeyboardEvent| {
        let Some(next) = moved(&event.key(), selected.get_untracked(), count) else {
            return;
        };
        event.prevent_default();
        selected.set(next);
        focus(list, next);
    };
    Effect::new(move |_| {
        let chosen = selected.get();
        prefix.with_value(|prefix| show(prefix, chosen, count));
    });
    let buttons = tabs
        .into_iter()
        .enumerate()
        .map(|(index, words)| {
            let is = move || selected.get() == index;
            prefix.with_value(|prefix| {
                view! {
                    <button
                        type="button"
                        role="tab"
                        id=tab_id(prefix, index)
                        class="fui-tab"
                        aria-controls=panel_id(prefix, index)
                        aria-selected=move || is().to_string()
                        tabindex=move || if is() { "0" } else { "-1" }
                        on:click=move |_| selected.set(index)
                    >
                        {words}
                    </button>
                }
            })
        })
        .collect_view();
    view! {
        <div class="fui-tabbed">
            <div role="tablist" aria-label=label class="fui-tabs" node_ref=list on:keydown=on_key>
                {buttons}
            </div>
            {panels}
        </div>
    }
    .into_any()
}

/// Returns the phone's select over `choices`, named `label`, on the one
/// leading to `chosen`, in a form whose submit button says `go`.
///
/// Choosing an option changes nothing else (WCAG 2.2 SC 3.2.2); once the
/// page has hydrated, pressing the button leads to the route chosen. Until
/// then the select is disabled and the row of links serves. A console draws
/// it through [`crate::view::tabs::binder`], as the island
/// [`crate::island::TabSelect`].
#[must_use]
pub fn tab_select(
    label: String,
    go_words: String,
    choices: Vec<crate::view::tabs::TabChoice>,
    chosen: &str,
) -> AnyView {
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let picker = NodeRef::<leptos::html::Select>::new();
    let options = choices
        .into_iter()
        .map(|choice| {
            let selected = choice.href == chosen;
            view! {
                <option value=choice.href selected=selected>
                    {choice.words}
                </option>
            }
        })
        .collect_view();
    let on_submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if let Some(select) = picker.get_untracked() {
            go(&select.value());
        }
    };
    view! {
        <form class="fui-tabs-select" method="get" on:submit=on_submit>
            <label class="fui-sr-only" for="fui-tabs-select">
                {label}
            </label>
            <select
                id="fui-tabs-select"
                class=crate::view::form::SELECT
                disabled=move || !hydrated.get()
                node_ref=picker
            >
                {options}
            </select>
            <button type="submit" class=crate::view::button::Variant::Secondary.class()>
                {go_words}
            </button>
        </form>
    }
    .into_any()
}

/// Leads the browser to `href`.
fn go(href: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if window().location().assign(href).is_err() {
            leptos::logging::warn!("the browser refused to follow a tab");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = href;
}

/// Shows panel `chosen` of the `count` panels of the tabs `prefix` and hides
/// the others.
fn show(prefix: &str, chosen: usize, count: usize) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        // NOTE: leptos_macro 0.8 `src/lib.rs`, island limitation 1: the panels are the
        // island's server children, out of reach of a node reference, so they are found by id.
        for index in 0..count {
            if let Some(panel) = document()
                .get_element_by_id(&panel_id(prefix, index))
                .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
            {
                panel.set_hidden(index != chosen);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (&str, usize, usize) = (prefix, chosen, count);
}

/// Moves the focus to tab `index` of the tab list `list`.
fn focus(list: NodeRef<leptos::html::Div>, index: usize) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let mut tab = list
            .get_untracked()
            .and_then(|list| list.first_element_child());
        for _ in 0..index {
            tab = tab.and_then(|before| before.next_element_sibling());
        }
        if let Some(tab) = tab.and_then(|tab| tab.dyn_into::<web_sys::HtmlElement>().ok())
            && tab.focus().is_err()
        {
            leptos::logging::warn!("the browser refused the focus on a tab");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (NodeRef<leptos::html::Div>, usize) = (list, index);
}

#[cfg(test)]
mod tests {
    use super::moved;

    #[test]
    fn the_arrows_wrap_and_home_and_end_go_to_the_ends() {
        assert_eq!(Some(1), moved("ArrowRight", 0, 3));
        assert_eq!(Some(0), moved("ArrowRight", 2, 3), "wraps forward");
        assert_eq!(Some(2), moved("ArrowLeft", 0, 3), "wraps back");
        assert_eq!(Some(0), moved("Home", 2, 3));
        assert_eq!(Some(2), moved("End", 0, 3));
        assert_eq!(None, moved("Enter", 0, 3), "a key the tabs leave alone");
        assert_eq!(None, moved("End", 0, 0), "no tab to move to");
    }
}
