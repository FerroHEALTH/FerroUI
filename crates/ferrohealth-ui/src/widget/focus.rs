// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The focus directive: an element that opens as a popover or a dialog
//! moves the keyboard focus to its first control as it opens.
//!
//! A directive (`use:focus_on_open=selector`) runs in the browser when the
//! element is created or hydrated, never on the server (tachys 0.2.19
//! `src/html/directive.rs`; the leptos `v0.8.22` example
//! `examples/directives`), so one function serves every overlay that needs
//! it and no widget repeats the browser code. It listens for the element's
//! `toggle` event and lets the listener go when the element's owner is
//! cleaned up.

use leptos::tachys::renderer::types::Element;

/// Moves the focus into `element` each time it opens.
///
/// It listens for the element's `toggle` event, and as it opens, as a
/// popover or as a dialog, moves the keyboard focus to the first control
/// inside it that `control` selects, such as `"a"` for a navigation's first
/// link. It is a directive: `<nav popover="auto" use:focus_on_open="a">`.
pub fn focus_on_open(element: Element, control: &'static str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::{StoredValue, WithValue as _, on_cleanup};
        use leptos::wasm_bindgen::JsCast as _;
        use leptos::wasm_bindgen::closure::Closure;
        let target = element.clone();
        let listener = Closure::<dyn Fn()>::new(move || focus_first(&target, control));
        if element
            .add_event_listener_with_callback("toggle", listener.as_ref().unchecked_ref())
            .is_err()
        {
            leptos::logging::warn!("the browser refused to watch an overlay open");
            return;
        }
        let kept = StoredValue::new_local((element, listener));
        on_cleanup(move || {
            kept.try_with_value(|(element, listener)| {
                if element
                    .remove_event_listener_with_callback(
                        "toggle",
                        listener.as_ref().unchecked_ref(),
                    )
                    .is_err()
                {
                    leptos::logging::warn!("the browser refused to stop watching an overlay");
                }
            });
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        drop(element);
        let _: &str = control;
    }
}

/// Moves the focus to the first element of `overlay` that `control`
/// selects, when `overlay` is open.
#[cfg(target_arch = "wasm32")]
fn focus_first(overlay: &Element, control: &str) {
    use leptos::wasm_bindgen::JsCast as _;
    if !overlay.matches(":popover-open, [open]").unwrap_or(false) {
        return;
    }
    let first = overlay
        .query_selector(control)
        .ok()
        .flatten()
        .and_then(|first| first.dyn_into::<web_sys::HtmlElement>().ok());
    if let Some(first) = first
        && first.focus().is_err()
    {
        leptos::logging::warn!("the browser refused the focus on an overlay's first control");
    }
}
