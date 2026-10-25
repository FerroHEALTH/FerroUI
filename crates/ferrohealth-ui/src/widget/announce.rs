// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Announces the page the router drew in the browser and moves the focus to
//! its heading, as a new document would (WCAG 2.2 SC 2.4.3 Focus Order, SC
//! 4.1.3 Status Messages).
//!
//! A navigation in the page loads no document, so a screen reader hears
//! nothing and the focus stays on the link that was pressed. Once the
//! address has changed and a new page heading (`main h1`) with words of its
//! own is in the page,
//! the announcer moves the focus to that heading, which takes
//! `tabindex="-1"` so it can hold the focus without joining the tab order,
//! and says the document's title in a polite live region. The first page is
//! never announced: the browser already reads a loaded document. It watches
//! the page itself, so it works with any router. No specification governs
//! the timing: our own design.

use leptos::prelude::*;

/// Returns the polite region that says the title of each page the router
/// draws; a console places it once, at its root, beside the toast region.
#[must_use]
pub fn page_announcer() -> impl IntoView {
    let said = RwSignal::new(String::new());
    Effect::new(move |_| watch(said));
    view! {
        <p role="status" aria-live="polite" class="fui-sr-only" data-ferroui-announcer="">
            {move || said.get()}
        </p>
    }
}

/// What the announcer last saw: the address and the heading's text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Seen {
    /// The address.
    pub href: String,
    /// The text of the page's heading.
    pub heading: String,
}

/// Whether the page `now` is a new page to announce after `before`: the
/// address changed and a new heading is in the page, so a route still
/// loading, which shows the old page, is not announced as the new one.
#[must_use]
pub fn arrived(before: &Seen, now: &Seen) -> bool {
    now.href != before.href && !now.heading.trim().is_empty() && now.heading != before.heading
}

/// Watches the page for a new address with its heading, and announces it in
/// `said`.
fn watch(said: RwSignal<String>) {
    #[cfg(target_arch = "wasm32")]
    browser::watch(said);
    #[cfg(not(target_arch = "wasm32"))]
    let _: RwSignal<String> = said;
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;
    use std::rc::Rc;

    use leptos::prelude::*;
    use leptos::wasm_bindgen::JsCast as _;
    use leptos::wasm_bindgen::closure::Closure;

    use super::{Seen, arrived};

    /// The page as it stands: its address and its heading.
    fn seen() -> (Seen, Option<web_sys::HtmlElement>) {
        let heading = document()
            .query_selector("main h1")
            .ok()
            .flatten()
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
        let seen = Seen {
            href: window().location().href().unwrap_or_default(),
            heading: heading
                .as_ref()
                .and_then(|heading| heading.text_content())
                .unwrap_or_default(),
        };
        (seen, heading)
    }

    /// Moves the focus to `heading` and says the document's title.
    fn announce(heading: &web_sys::HtmlElement, said: RwSignal<String>) {
        if !heading.has_attribute("tabindex") && heading.set_attribute("tabindex", "-1").is_err() {
            leptos::logging::warn!("the browser refused the heading's tabindex");
        }
        if heading.focus().is_err() {
            leptos::logging::warn!("the browser refused the focus on the page's heading");
        }
        said.set(document().title());
    }

    pub(super) fn watch(said: RwSignal<String>) {
        let last = Rc::new(RefCell::new(seen().0));
        let check = {
            let last = Rc::clone(&last);
            move || {
                let (now, heading) = seen();
                if !arrived(&last.borrow(), &now) {
                    return;
                }
                *last.borrow_mut() = now;
                if let Some(heading) = heading {
                    announce(&heading, said);
                }
            }
        };
        // NOTE: the router may change the address after it draws the page, so the check runs
        // in the frame after the page changed.
        let check = Rc::new(check);
        let callback = Closure::<dyn FnMut()>::new(move || {
            let check = Rc::clone(&check);
            request_animation_frame(move || check());
        });
        let Ok(observer) = web_sys::MutationObserver::new(callback.as_ref().unchecked_ref()) else {
            leptos::logging::warn!("the browser refused the page announcer");
            return;
        };
        let options = web_sys::MutationObserverInit::new();
        options.set_child_list(true);
        options.set_subtree(true);
        options.set_character_data(true);
        if let Some(body) = document().body()
            && observer.observe_with_options(&body, &options).is_err()
        {
            leptos::logging::warn!("the browser refused to watch the page");
        }
        callback.forget();
        let kept = StoredValue::new_local(observer);
        on_cleanup(move || {
            let _: Option<()> = kept.try_with_value(web_sys::MutationObserver::disconnect);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{Seen, arrived};

    /// A page at `href` whose heading is `heading`.
    fn page(href: &str, heading: &str) -> Seen {
        Seen {
            href: String::from(href),
            heading: String::from(heading),
        }
    }

    #[test]
    fn a_new_address_with_its_heading_is_announced() {
        assert!(arrived(&page("/a", "A"), &page("/b", "B")));
        assert!(arrived(&page("/a?page=2", "A"), &page("/b", "B")));
    }

    #[test]
    fn the_same_address_or_no_heading_yet_is_not() {
        assert!(
            !arrived(&page("/a", "A"), &page("/a", "B")),
            "a change inside the page"
        );
        assert!(
            !arrived(&page("/a", "A"), &page("/b", "  ")),
            "the next page has not drawn"
        );
        assert!(
            !arrived(&page("/a", "A"), &page("/b", "A")),
            "the old page shows while the next loads"
        );
    }
}
