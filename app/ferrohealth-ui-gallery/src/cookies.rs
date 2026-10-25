// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The cookies the page knows, read by the server for the first page and
//! asked of it again after each navigation, as a console reads its own
//! state: never on the server alone.
#![cfg_attr(
    target_arch = "wasm32",
    allow(
        clippy::unused_async_trait_impl,
        reason = "the server function macro writes the browser half's call as an async trait method; only that target has it"
    )
)]

use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// The cookies the page knows.
///
/// The server reads them for the first page and the browser takes them
/// serialized, then asks the server again after each navigation
/// ([`requested_cookies`]), so both halves draw the same page and no state
/// is read on the server alone.
#[derive(Debug, Clone, Copy)]
pub struct Cookies(pub RwSignal<String>);

/// The cookies the page knows ([`Cookies`]), or the request's own outside
/// the page tree.
#[must_use]
pub fn cookies_requested() -> String {
    use_context::<Cookies>().map_or_else(server_cookies, |cookies| cookies.0.get())
}

/// The request's `Cookie` header on the server; in the browser, which reads
/// the value the server serialized, empty.
pub(crate) fn server_cookies() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        request_cookies()
    }
    #[cfg(target_arch = "wasm32")]
    {
        String::new()
    }
}

/// The cookies of the request that asks, for the page after a navigation.
///
/// # Errors
///
/// When the server function cannot be reached or answered.
#[server(endpoint = "cookies")]
pub async fn requested_cookies() -> Result<String, ServerFnError> {
    let headers: http::HeaderMap = leptos_axum::extract().await?;
    Ok(headers
        .get(http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned())
}

/// The request's `Cookie` header, empty for none.
#[cfg(not(target_arch = "wasm32"))]
fn request_cookies() -> String {
    use_context::<http::request::Parts>()
        .and_then(|parts| {
            parts
                .headers
                .get(http::header::COOKIE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

/// Asks the server for the cookies again after each navigation and each
/// post drawn in the page, so a preference a post just kept reaches the
/// page drawn in the browser.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub(crate) fn FollowCookies() -> impl IntoView {
    let location = use_location();
    let cookies = use_context::<Cookies>();
    Effect::watch(
        move || {
            (
                location.pathname.get(),
                location.search.get(),
                ferrohealth_ui::widget::submit::posts().get(),
            )
        },
        move |_, _, _| {
            if let Some(Cookies(cookies)) = cookies {
                leptos::task::spawn_local(async move {
                    match requested_cookies().await {
                        Ok(now) => {
                            follow_root(&now);
                            // NOTE: no specification governs this: our own design; the same
                            // cookies redraw nothing, so a page keeps the focus it took.
                            if cookies.get_untracked() != now {
                                cookies.set(now);
                            }
                        }
                        Err(error) => leptos::logging::warn!("the cookies did not come: {error}"),
                    }
                });
            }
        },
        false,
    );
}

/// Sets the root element's theme, density, motion, hints and language from
/// the cookies `now`, as the server writes them on a document it draws, so a
/// preference a post just kept takes effect in the page.
fn follow_root(now: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(root) = document().document_element() else {
            return;
        };
        let preferences = crate::app::preferences_of(now);
        let attributes = [
            ("data-theme", preferences.theme.key().to_owned()),
            ("data-density", preferences.density.key().to_owned()),
            ("data-motion", preferences.motion.key().to_owned()),
            ("data-hints", preferences.hints.key().to_owned()),
            ("lang", crate::app::locale_of(now).tag()),
        ];
        // NOTE: no specification governs this: our own design; the words of a page are
        // drawn in its locale, so a new language loads the page as a document.
        let language = crate::app::locale_of(now).tag();
        if root
            .get_attribute("lang")
            .is_some_and(|lang| lang != language)
        {
            if window().location().reload().is_err() {
                leptos::logging::warn!("the browser refused to load the page in its language");
            }
            return;
        }
        for (name, value) in attributes {
            if root.get_attribute(name).as_deref() != Some(value.as_str())
                && root.set_attribute(name, &value).is_err()
            {
                leptos::logging::warn!("the browser refused the root's {name}");
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = now;
}
