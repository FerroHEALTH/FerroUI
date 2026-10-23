// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keys and the busy state of the top bar's search box, the island
//! [`crate::island::Search`] draws through [`search_control`].
//!
//! The search owns its own shortcut (#144): its key (`/` by default) from
//! anywhere outside an element that takes text focuses the field with the
//! caret after its text, unless a listener of the page took the key first;
//! Escape in the field closes an open answer and keeps the words, and on a
//! closed answer clears the words, the focus staying in the field each time
//! (WAI-ARIA Authoring Practices 1.2, the combobox pattern's Escape; WCAG 2.2
//! SC 2.4.3); typing a new query closes the answer; Enter on an open answer
//! follows the answer's first hit, and otherwise submits the form. A submit
//! turns the glyph into the spinner while the next page loads. With no
//! WebAssembly the form submits as it is ([`crate::view::search`]). No
//! specification governs the other keys: our own design; the shortcut is
//! announced through `aria-keyshortcuts` (WAI-ARIA 1.2).

use leptos::either::Either;
use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::view::search::SearchBox;

/// What Escape in the search field does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escape {
    /// Closes the open answer and keeps the words.
    Close,
    /// Clears the words of a field whose answer is closed.
    Clear,
    /// Leaves the key to the page: the field is empty and nothing is open.
    Pass,
}

/// What Escape does in a field whose answer is `open` and whose words are
/// `empty` or not; the focus stays in the field in every case.
#[must_use]
pub fn escaped(open: bool, empty: bool) -> Escape {
    if open {
        Escape::Close
    } else if empty {
        Escape::Pass
    } else {
        Escape::Clear
    }
}

/// Whether the key `key` of an event is the shortcut `shortcut`.
#[must_use]
pub fn is_shortcut(key: &str, shortcut: char) -> bool {
    let mut chars = key.chars();
    chars.next() == Some(shortcut) && chars.next().is_none()
}

/// Returns the search box `search`, its answer `answer` under it, open when
/// `open`, Enter following the hit `selected` while the query is the one
/// answered.
#[must_use]
pub fn search_control(
    search: SearchBox,
    selected: Option<String>,
    open: bool,
    answer: AnyView,
) -> impl IntoView {
    let SearchBox {
        id,
        name,
        label,
        placeholder,
        action,
        value,
        submit,
        shortcut,
        busy,
        disabled,
        post,
    } = search;
    let field = NodeRef::<leptos::html::Input>::new();
    let busy = RwSignal::new(busy);
    let closed = RwSignal::new(!open);
    let off = disabled.is_some();
    let answered = StoredValue::new(value.clone());
    let selected = StoredValue::new(selected);
    Effect::new(move |_| listen(field, shortcut, busy));
    let on_key = move |event: leptos::ev::KeyboardEvent| match event.key().as_str() {
        "Escape" => escape(&event, field, closed),
        "Enter" if !closed.get_untracked() => {
            let unchanged =
                answered.with_value(|answered| field_value(field).as_deref() == Some(answered));
            if unchanged && let Some(hit) = selected.get_value() {
                event.prevent_default();
                follow(&hit);
            }
        }
        _ => {}
    };
    let named = submit.clone();
    let why = format!("{id}-why");
    let described = off.then(|| why.clone());
    let reason = disabled.clone();
    let hint = shortcut
        .filter(|_| !off)
        .map(|key| view! { <kbd class="fui-kbd fui-search-hint" aria-hidden="true">{key.to_string()}</kbd> });
    let keys = shortcut.filter(|_| !off).map(|key| key.to_string());
    let shown = disabled.clone().unwrap_or(placeholder);
    let glyph = move || {
        if busy.get() {
            Either::Left(view! { <span class="fui-spinner-ring" aria-hidden="true"></span> })
        } else {
            Either::Right(icon(Glyph::SEARCH, "16"))
        }
    };
    view! {
        <form
            role="search"
            method=if post { "post" } else { "get" }
            action=action
            class="fui-search"
            aria-busy=move || busy.get().then_some("true")
            on:submit=move |_| busy.set(true)
        >
            <label for=id.clone() class="fui-sr-only">
                {label}
            </label>
            <div class="fui-search-field">
                <span class="fui-search-glyph">{glyph}</span>
                <input
                    id=id
                    name=name
                    type="search"
                    class="fui-input"
                    placeholder=shown
                    value=value
                    autocomplete="off"
                    spellcheck="false"
                    aria-keyshortcuts=keys
                    aria-describedby=described
                    title=reason.clone()
                    disabled=off
                    node_ref=field
                    on:keydown=on_key
                    on:input=move |_| closed.set(true)
                />
                {hint}
            </div>
            <button
                type="submit"
                class="fui-search-submit"
                aria-label=named
                disabled=move || off || busy.get()
            >
                {icon(Glyph::SEARCH, "16")}
                <span class="fui-search-word">{submit}</span>
            </button>
            {reason.map(|reason| view! { <span id=why class="fui-sr-only">{reason}</span> })}
            <div hidden=move || closed.get()>{answer}</div>
        </form>
    }
}

/// Answers Escape in `field`: closes the answer `closed` names when it is
/// open, and otherwise clears the words, the focus staying in the field.
fn escape(
    event: &leptos::ev::KeyboardEvent,
    field: NodeRef<leptos::html::Input>,
    closed: RwSignal<bool>,
) {
    let empty = field_value(field).is_none_or(|words| words.is_empty());
    match escaped(!closed.get_untracked(), empty) {
        Escape::Close => {
            event.prevent_default();
            closed.set(true);
        }
        Escape::Clear => {
            event.prevent_default();
            clear(field);
        }
        Escape::Pass => {}
    }
}

/// Listens on the window for the key `shortcut`, which focuses `field`, and
/// for the page coming back from the history, which ends `busy`.
fn listen(field: NodeRef<leptos::html::Input>, shortcut: Option<char>, busy: RwSignal<bool>) {
    #[cfg(target_arch = "wasm32")]
    {
        let back = window_event_listener(leptos::ev::pageshow, move |_| busy.set(false));
        let Some(shortcut) = shortcut else {
            on_cleanup(move || back.remove());
            return;
        };
        let keys = window_event_listener(leptos::ev::keydown, move |event| {
            if event.default_prevented()
                || !is_shortcut(&event.key(), shortcut)
                || crate::widget::shortcuts::chorded(&event)
                || crate::widget::shortcuts::typing(&event)
            {
                return;
            }
            let Some(input) = field.get_untracked() else {
                return;
            };
            if input.disabled() {
                return;
            }
            event.prevent_default();
            if input.focus().is_err() {
                leptos::logging::warn!("the browser refused the focus on the search field");
            }
            let end = u32::try_from(input.value().encode_utf16().count()).unwrap_or(u32::MAX);
            if input.set_selection_range(end, end).is_err() {
                leptos::logging::warn!("the browser refused the caret in the search field");
            }
        });
        on_cleanup(move || {
            back.remove();
            keys.remove();
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (NodeRef<leptos::html::Input>, Option<char>, RwSignal<bool>) = (field, shortcut, busy);
}

/// The text in `field`, once it is in the page.
fn field_value(field: NodeRef<leptos::html::Input>) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        field.get_untracked().map(|input| input.value())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: NodeRef<leptos::html::Input> = field;
        None
    }
}

/// Clears `field`, which keeps the focus.
fn clear(field: NodeRef<leptos::html::Input>) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(input) = field.get_untracked() {
            input.set_value("");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: NodeRef<leptos::html::Input> = field;
}

/// Follows the hit `id` of the answer, as a press on its link does.
fn follow(id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        // NOTE: leptos_macro 0.8 `src/lib.rs`, island limitation 1: the answer is the
        // island's server children, out of reach of a node reference, so its hit is found by id.
        if let Some(link) = document()
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        {
            link.click();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = id;
}

#[cfg(test)]
mod tests {
    use super::{Escape, escaped, is_shortcut};

    #[test]
    fn escape_closes_the_answer_before_it_clears_the_words() {
        assert_eq!(Escape::Close, escaped(true, false), "the words stay");
        assert_eq!(Escape::Close, escaped(true, true));
        assert_eq!(Escape::Clear, escaped(false, false));
        assert_eq!(Escape::Pass, escaped(false, true), "nothing to undo");
    }

    #[test]
    fn only_the_key_itself_is_the_shortcut() {
        assert!(is_shortcut("/", '/'));
        assert!(!is_shortcut("?", '/'));
        assert!(
            !is_shortcut("Slash", '/'),
            "a key's name is not its character"
        );
        assert!(!is_shortcut("", '/'));
    }
}
