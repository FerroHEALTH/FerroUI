// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keyboard tabs that switch panels in place.
//!
//! They follow WAI-ARIA Authoring Practices 1.2, the tabs pattern with
//! automatic activation: one tab stop,
//! Left and Right move between the tabs and wrap, Home and End go to the
//! first and the last, and the panel of the selected tab is shown. Before
//! the page hydrates the first panel shows and the others are hidden; the
//! link tabs of [`crate::view::tabs`] keep their state in the URL instead.

use leptos::prelude::*;

/// One tab and its panel.
#[derive(Debug)]
pub struct Panel {
    /// The tab's words.
    pub label: &'static str,
    /// The panel.
    pub body: AnyView,
}

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

/// Returns the keyboard tabs named `label`, whose ids start with `prefix`,
/// over `panels`.
#[must_use]
pub fn tabs(label: &'static str, prefix: &'static str, panels: Vec<Panel>) -> impl IntoView {
    let selected = RwSignal::new(0_usize);
    let count = panels.len();
    let on_key = move |event: leptos::ev::KeyboardEvent| {
        let Some(next) = moved(&event.key(), selected.get_untracked(), count) else {
            return;
        };
        event.prevent_default();
        selected.set(next);
        focus(&tab_id(prefix, next));
    };
    let (buttons, bodies): (Vec<_>, Vec<_>) = panels
        .into_iter()
        .enumerate()
        .map(|(index, panel)| {
            let is = move || selected.get() == index;
            let button = view! {
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
                    {panel.label}
                </button>
            };
            let body = view! {
                <div
                    role="tabpanel"
                    id=panel_id(prefix, index)
                    class="fui-tab-panel"
                    aria-labelledby=tab_id(prefix, index)
                    hidden=move || !is()
                    tabindex="0"
                >
                    {panel.body}
                </div>
            };
            (button, body)
        })
        .unzip();
    view! {
        <div>
            <div role="tablist" aria-label=label class="fui-tabs" on:keydown=on_key>
                {buttons}
            </div>
            {bodies}
        </div>
    }
}

/// Moves the focus to the element `id`.
fn focus(id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let element = document()
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<leptos::web_sys::HtmlElement>().ok());
        if let Some(element) = element
            && element.focus().is_err()
        {
            leptos::logging::warn!("the browser refused the focus on a tab");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = id;
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
