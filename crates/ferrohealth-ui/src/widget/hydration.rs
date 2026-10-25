// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Says when the page has hydrated, so nothing changes the page the server
//! drew before then.
//!
//! The browser hydrates the page by walking the nodes the server drew. A
//! script, a test or a console's own code that adds, removes or rewrites a
//! node before that walk ends leaves the walk a node it did not expect, and
//! Leptos stops with an internal error that names no cause (the Leptos book,
//! "Hydration Bugs"). [`mark`] writes [`HYDRATED`] on the root element once
//! the page has hydrated, which anything that changes the page waits for,
//! and in a debug build [`explain`] adds the likely cause to a panic that
//! comes before it. No specification governs this: our own design.

use leptos::prelude::*;

/// The attribute the root element carries once the page has hydrated.
pub const HYDRATED: &str = "data-ferroui-hydrated";

/// Writes [`HYDRATED`] on the root element once the page has hydrated.
///
/// Call it once, from the console's root component. An effect runs only
/// in the browser and only after hydration, so the mark never comes early.
pub fn mark() {
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        if let Some(root) = document().document_element()
            && root.set_attribute(HYDRATED, "").is_err()
        {
            leptos::logging::warn!("the browser refused the hydrated mark");
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
/// Call it once, first thing in the browser entry, before
/// `leptos::mount::hydrate_lazy`; it keeps the panic hook already set, such
/// as `console_error_panic_hook`, and runs it after. A release build sets
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
