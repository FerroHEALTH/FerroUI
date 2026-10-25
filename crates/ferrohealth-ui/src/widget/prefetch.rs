// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Loads a page's code before you open it, so a first visit costs no more
//! than a return.
//!
//! A console splits its code by route: each page is a `leptos_router`
//! `LazyRoute`, whose code loads from chunks of its own under
//! `cargo leptos build --split` (the Leptos book, "Code Splitting"). A
//! console names its routes and their `LazyRoute::preload` once, with
//! [`routes`]: a link to one of them loads that route's code the moment the
//! pointer rests on it or the keyboard focuses it, and once the first page
//! has hydrated and the browser is idle, the routes marked `idle` (the main
//! navigation's) load too. Each route loads once. No specification governs
//! this: our own design.

use std::future::Future;
use std::pin::Pin;

/// A route's code loader: `LazyRoute::preload` of its lazy route.
pub type Preload = fn() -> Pin<Box<dyn Future<Output = ()>>>;

/// One route a console splits out.
#[derive(Debug, Clone, Copy)]
pub struct Route {
    /// The route's path as the router matches it, `:name` standing for a
    /// segment, such as `/:brand/forms`.
    pub path: &'static str,
    /// Its loader.
    pub preload: Preload,
    /// Whether it loads once the first page is idle: a route of the main
    /// navigation.
    pub idle: bool,
}

/// Whether the address path `path` is one the route pattern `pattern`
/// matches, segment by segment, `:name` matching any one segment.
#[must_use]
pub fn matches(pattern: &str, path: &str) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or_default();
    let mut wanted = pattern.split('/').filter(|part| !part.is_empty());
    let mut given = path.split('/').filter(|part| !part.is_empty());
    loop {
        match (wanted.next(), given.next()) {
            (None, None) => return true,
            (Some(part), Some(segment)) => {
                if !part.starts_with(':') && part != segment {
                    return false;
                }
            }
            _ => return false,
        }
    }
}

/// Loads the code of the routes in `routes` ahead of a visit.
///
/// A route loads on a link's hover or focus, and the `idle` ones once the
/// browser is idle. Call it once, from the console's root component; it
/// does its work in the browser alone.
pub fn routes(routes: &'static [Route]) {
    #[cfg(target_arch = "wasm32")]
    browser::start(routes);
    #[cfg(not(target_arch = "wasm32"))]
    let _: &[Route] = routes;
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    use leptos::prelude::*;
    use leptos::wasm_bindgen::JsCast as _;

    use super::{Route, matches};

    thread_local! {
        /// The paths of the routes whose code has started loading.
        static LOADED: RefCell<BTreeSet<&'static str>> = const { RefCell::new(BTreeSet::new()) };
    }

    /// Starts loading `route`'s code, unless it has started already.
    fn load(route: &Route) {
        let first = LOADED.with(|loaded| loaded.borrow_mut().insert(route.path));
        if first {
            leptos::task::spawn_local((route.preload)());
        }
    }

    /// Loads the code of the route among `routes` the link under `event`
    /// leads to, if it is one of them on this origin.
    fn hovered(routes: &'static [Route], event: &web_sys::Event) {
        let Some(link) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .and_then(|element| element.closest("a[href]").ok().flatten())
            .and_then(|element| element.dyn_into::<web_sys::HtmlAnchorElement>().ok())
        else {
            return;
        };
        if link.origin() != window().location().origin().unwrap_or_default() {
            return;
        }
        let path = link.pathname();
        if let Some(route) = routes.iter().find(|route| matches(route.path, &path)) {
            load(route);
        }
    }

    pub(super) fn start(routes: &'static [Route]) {
        let pointer = window_event_listener(leptos::ev::pointerover, move |event| {
            hovered(routes, &event);
        });
        let focus = window_event_listener(leptos::ev::focusin, move |event| {
            hovered(routes, &event);
        });
        request_idle_callback(move || {
            for route in routes.iter().filter(|route| route.idle) {
                load(route);
            }
        });
        on_cleanup(move || {
            pointer.remove();
            focus.remove();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn a_pattern_matches_its_paths_segment_by_segment() {
        assert!(matches("/:brand/forms", "/ferrobridge/forms"));
        assert!(matches("/:brand/forms", "/ferrobridge/forms?q=a#top"));
        assert!(matches("/:brand", "/ferrobridge/"));
        assert!(!matches("/:brand/forms", "/ferrobridge/tables"));
        assert!(!matches("/:brand/forms", "/ferrobridge/forms/more"));
        assert!(!matches("/:brand/forms", "/ferrobridge"));
        assert!(matches("/", "/"));
    }
}
