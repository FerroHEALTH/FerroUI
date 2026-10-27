// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Says when the page has hydrated, so nothing changes the page the server
//! drew before then.
//!
//! The browser hydrates the page by walking the nodes the server drew. A
//! script, a test or a console's own code that adds, removes or rewrites a
//! node before that walk ends leaves the walk a node it did not expect, and
//! Leptos stops with an internal error that names no cause (the Leptos book,
//! "Hydration Bugs"). `hydrate`, the browser entry, writes [`HYDRATED`] on the
//! root element once the page and its lazy route have hydrated, which anything
//! that changes the page waits for, and leaves every click and submit to the
//! browser until then; in a debug build [`explain`] adds the likely cause to a
//! panic that comes before it. No specification governs this: our own design.

use leptos::prelude::*;

/// The attribute the root element carries once the page has hydrated.
pub const HYDRATED: &str = "data-ferroui-hydrated";

/// The attribute the root element carries while the page hydrates, from the
/// start of `hydrate` until [`HYDRATED`] replaces it.
pub const HYDRATING: &str = "data-ferroui-hydrating";

/// Hydrates the page the server drew with the root component `app`.
///
/// Its lazy routes load their code from their own chunks, and the root
/// element takes [`HYDRATED`] once the page and its lazy route have hydrated.
///
/// Call it in the browser entry in place of `leptos::mount::hydrate_lazy`,
/// after the panic hook and [`explain`]. Until the page has hydrated, a
/// click or a submit is left to the browser: a link loads its page and a
/// form posts as with no WebAssembly. The router, which takes a link from
/// the moment it is drawn, would otherwise draw the next page while the
/// lazy route still hydrates, and the hydration walk would stop on a node
/// it did not expect (FerroUI #448).
#[cfg(target_arch = "wasm32")]
pub fn hydrate<F, N>(app: F)
where
    F: FnOnce() -> N + 'static,
    N: IntoView,
{
    browser::guard();
    browser::mark(HYDRATING, None);
    wasm_bindgen_futures::spawn_local(async move {
        let Some(body) = document().body() else {
            leptos::logging::warn!("the page has no body to hydrate");
            return;
        };
        // NOTE: leptos 0.8 `src/mount.rs`: `hydrate_from_async` resolves once the
        // whole page, its lazy routes included, has hydrated.
        let page = hydrate_from_async(body, app).await;
        page.forget();
        browser::mark(HYDRATED, Some(HYDRATING));
    });
}

/// Writes [`HYDRATED`] on the root element from an effect of the console's
/// root component.
///
/// An effect runs on the next tick, which can come while a lazy route's
/// code is still on its way, so the mark can come before the page has
/// hydrated. `hydrate` writes it when hydration ends; with `hydrate` as the
/// browser entry this does nothing.
#[deprecated(
    since = "0.5.2",
    note = "use widget::hydration::hydrate as the browser entry, which writes the mark when hydration ends"
)]
pub fn mark() {
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        if !document()
            .document_element()
            .is_some_and(|root| root.has_attribute(HYDRATING))
        {
            browser::mark(HYDRATED, None);
        }
    });
}

/// Whether the root element carries [`HYDRATED`].
#[must_use]
pub fn hydrated() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        document()
            .document_element()
            .is_some_and(|root| root.has_attribute(HYDRATED))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// In a debug build, has a panic before the page hydrated say its likely
/// cause first: something changed the page before hydration ended.
///
/// Call it once in the browser entry, after the panic hook and before
/// `hydrate`; it keeps the panic hook already set, such as
/// `console_error_panic_hook`, and runs it after. A release build sets
/// nothing.
pub fn explain() {
    #[cfg(all(target_arch = "wasm32", debug_assertions))]
    {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !hydrated() {
                leptos::logging::error!(
                    "FerroUI: this panic came before the page hydrated; was the page the server drew changed before its data-ferroui-hydrated mark?"
                );
            }
            previous(info);
        }));
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use leptos::wasm_bindgen::JsCast as _;
    use leptos::wasm_bindgen::closure::Closure;

    use super::hydrated;

    /// Writes the attribute `name` on the root element, and takes `gone` off
    /// it.
    pub(super) fn mark(name: &str, gone: Option<&str>) {
        if let Some(root) = document().document_element()
            && (root.set_attribute(name, "").is_err()
                || gone.is_some_and(|gone| root.remove_attribute(gone).is_err()))
        {
            leptos::logging::warn!("the browser refused the hydration guard or its mark");
        }
    }

    /// Leaves every click and submit to the browser until the page has
    /// hydrated: a listener on the window's capture phase, which runs before
    /// the router's and the form navigator's, stops the event there, so the
    /// browser's own action follows.
    pub(super) fn guard() {
        let held = Closure::<dyn Fn(web_sys::Event)>::new(move |event: web_sys::Event| {
            if !hydrated() {
                event.stop_immediate_propagation();
            }
        });
        for kind in ["click", "submit"] {
            if window()
                .add_event_listener_with_callback_and_bool(
                    kind,
                    held.as_ref().unchecked_ref(),
                    true,
                )
                .is_err()
            {
                leptos::logging::warn!("the browser refused the hydration guard or its mark");
            }
        }
        // NOTE: no specification governs this: our own design; the guard lasts as
        // long as the page and lets every event through once the page has hydrated.
        held.forget();
    }
}
