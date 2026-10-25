// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Submits the page's forms through the router, so a form loads no new
//! document (FerroUI #410).
//!
//! `leptos_router` follows an HTML link in the page by itself; a form, and a
//! link of an SVG drawing, which the router leaves to the browser, need this.
//! A `GET` form, a filter, a sort, a search, navigates to its address with
//! its fields (and its pressed submit's name and value) in the query. A
//! `POST` form whose route answers with a redirect, the post-redirect-get
//! the library's preference, theme, rail and pager forms keep, says so with
//! [`NAVIGATES`]: it is posted in the background and the router draws the
//! page the redirect names. A post the browser could not send falls back to
//! the plain post, a load of a new document; an answer that is no redirect
//! loads its address as a document, so the page is never left blank or
//! stale. Every other post, and a form marked [`RELOADS`] or with a
//! `target`, submits as the browser does. With no WebAssembly every form
//! submits as the browser does. No specification governs this: our own
//! design, after the router's own `Form` (`leptos_router` 0.8 `src/form.rs`).

use leptos::prelude::*;

/// The attribute that marks a `POST` form whose route answers with a
/// redirect, so the router may draw the page it names.
pub const NAVIGATES: &str = "data-ferroui-navigates";

/// The attribute that keeps a form, or one submit of it, to the browser's
/// own submit: a download, an export, a route that answers with no page.
pub const RELOADS: &str = "data-ferroui-reloads";

/// How many posts the router has drawn in the page.
///
/// A console that reads state after a navigation reads it again when this
/// changes, since a post that redirects to the page it came from changes no
/// address. It is `0` on the server.
#[must_use]
pub fn posts() -> Signal<u32> {
    #[cfg(target_arch = "wasm32")]
    {
        let posts = browser::POSTS.with(Clone::clone);
        Signal::derive(move || posts.get())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Signal::stored(0)
    }
}

/// Follows `href` as a press on a link to it does.
///
/// The console's router draws the page in the browser when it has one, and
/// the browser loads it otherwise. A widget that leads elsewhere, a tab, a
/// tree's level, a search hit, follows through this, never by setting the
/// address.
pub fn follow(href: &str) {
    #[cfg(target_arch = "wasm32")]
    browser::follow(href);
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = href;
}

/// How the navigator takes a submit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// The browser submits it, a load of a new document.
    Browser,
    /// The router navigates to the address with the fields in its query.
    Query,
    /// It is posted in the background, the router drawing the redirect.
    Post,
}

/// How a submit is taken: by its `method`, whether the form is
/// [`NAVIGATES`], [`RELOADS`] or has a `target`, and whether its `action`
/// is `same_origin`.
#[must_use]
pub fn route(method: &str, navigates: bool, reloads: bool, same_origin: bool) -> Route {
    if reloads || !same_origin {
        return Route::Browser;
    }
    match method.to_ascii_lowercase().as_str() {
        "get" => Route::Query,
        "post" if navigates => Route::Post,
        _ => Route::Browser,
    }
}

/// Returns nothing to draw, and takes the page's form submits through the
/// router; a console draws it once, inside its `<Router>`.
#[must_use]
pub fn form_navigator() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        let navigate = leptos_router::hooks::use_navigate();
        Effect::new(move |_| browser::listen(navigate.clone()));
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use leptos::wasm_bindgen::{JsCast as _, JsValue};
    use leptos_router::NavigateOptions;
    use wasm_bindgen_futures::JsFuture;

    use super::{NAVIGATES, RELOADS, Route, route};

    /// The marker of a form the navigator gave back to the browser, so its
    /// next submit is the browser's own.
    const NATIVE: &str = "data-ferroui-native";

    thread_local! {
        /// The posts drawn in the page ([`super::posts`]), a signal of the
        /// browser's one thread that no component owns, so it lives as long as
        /// the page.
        pub(super) static POSTS: ArcRwSignal<u32> = ArcRwSignal::new(0);
    }

    /// A link to `href`, put in the page for the one press, so the
    /// router's own listener takes it.
    pub(super) fn follow(href: &str) {
        let Some(body) = document().body() else {
            return;
        };
        let Some(link) = document()
            .create_element("a")
            .ok()
            .and_then(|element| element.dyn_into::<web_sys::HtmlAnchorElement>().ok())
        else {
            return;
        };
        link.set_href(href);
        link.set_hidden(true);
        if body.append_child(&link).is_err() {
            leptos::logging::warn!("the browser refused to follow a link");
            return;
        }
        link.click();
        link.remove();
    }

    pub(super) fn listen(navigate: impl Fn(&str, NavigateOptions) + Clone + 'static) {
        let submits = {
            let navigate = navigate.clone();
            window_event_listener(leptos::ev::submit, move |event| {
                submitted(&event, navigate.clone());
            })
        };
        let clicks = window_event_listener(leptos::ev::click, move |event| {
            pressed(&event, navigate.clone());
        });
        on_cleanup(move || {
            submits.remove();
            clicks.remove();
        });
    }

    /// Follows a press on a link of an SVG drawing (a topology's or a
    /// flow's card) in the page, which the router's own listener leaves to
    /// the browser, since it takes HTML links alone.
    fn pressed(event: &web_sys::MouseEvent, navigate: impl Fn(&str, NavigateOptions)) {
        if event.default_prevented()
            || event.button() != 0
            || event.meta_key()
            || event.alt_key()
            || event.ctrl_key()
            || event.shift_key()
        {
            return;
        }
        let Some(link) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .and_then(|element| element.closest("a").ok().flatten())
            .filter(|link| link.namespace_uri().as_deref() == Some("http://www.w3.org/2000/svg"))
        else {
            return;
        };
        if link.has_attribute("download")
            || link
                .get_attribute("target")
                .is_some_and(|target| !target.is_empty())
            || link
                .get_attribute("rel")
                .is_some_and(|rel| rel.split_whitespace().any(|part| part == "external"))
        {
            return;
        }
        let Some(href) = link.get_attribute("href") else {
            return;
        };
        let Ok(url) =
            web_sys::Url::new_with_base(&href, &window().location().href().unwrap_or_default())
        else {
            return;
        };
        if url.origin() != window().location().origin().unwrap_or_default() {
            return;
        }
        event.prevent_default();
        navigate(
            &format!("{}{}{}", url.pathname(), url.search(), url.hash()),
            NavigateOptions::default(),
        );
    }

    /// The attribute `name` of the submit `by` when it has one, else of the
    /// form.
    fn either(
        form: &web_sys::HtmlFormElement,
        by: Option<&web_sys::HtmlElement>,
        name: &str,
    ) -> Option<String> {
        by.and_then(|by| by.get_attribute(&format!("form{name}")))
            .or_else(|| form.get_attribute(name))
    }

    /// The form's fields with the pressed submit's name and value.
    fn fields(
        form: &web_sys::HtmlFormElement,
        by: Option<&web_sys::HtmlElement>,
    ) -> Option<web_sys::FormData> {
        let data = web_sys::FormData::new_with_form(form).ok()?;
        if let Some(by) = by
            && let Some(name) = by.get_attribute("name").filter(|name| !name.is_empty())
        {
            let value = by.get_attribute("value").unwrap_or_default();
            data.append_with_str(&name, &value).ok()?;
        }
        Some(data)
    }

    fn submitted(event: &web_sys::SubmitEvent, navigate: impl Fn(&str, NavigateOptions) + 'static) {
        if event.default_prevented() {
            return;
        }
        let Some(form) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlFormElement>().ok())
        else {
            return;
        };
        if form.has_attribute(NATIVE) {
            drop(form.remove_attribute(NATIVE));
            return;
        }
        let by = event.submitter();
        let action = either(&form, by.as_ref(), "action").unwrap_or_default();
        let Ok(url) =
            web_sys::Url::new_with_base(&action, &window().location().href().unwrap_or_default())
        else {
            return;
        };
        let same_origin = url.origin() == window().location().origin().unwrap_or_default();
        let method = either(&form, by.as_ref(), "method").unwrap_or_else(|| String::from("get"));
        let reloads = form.has_attribute(RELOADS)
            || by.as_ref().is_some_and(|by| by.has_attribute(RELOADS))
            || form
                .get_attribute("target")
                .is_some_and(|target| !target.is_empty());
        let navigates = form.has_attribute(NAVIGATES);
        let chosen = route(&method, navigates, reloads, same_origin);
        if chosen == Route::Browser {
            return;
        }
        let Some(data) = fields(&form, by.as_ref()) else {
            return;
        };
        event.prevent_default();
        match chosen {
            Route::Query => {
                let Ok(query) = web_sys::UrlSearchParams::new_with_str_sequence_sequence(&data)
                else {
                    return;
                };
                let query = String::from(query.to_string());
                let address = format!("{}?{query}", url.pathname());
                navigate(&address, NavigateOptions::default());
                at_fragment(&address, &url.hash());
            }
            Route::Post => {
                let path = url.href();
                leptos::task::spawn_local(async move {
                    match post(&path, &data, &form).await {
                        Ok(response) if response.redirected() => {
                            let Ok(next) = web_sys::Url::new(&response.url()) else {
                                return;
                            };
                            if next.origin() == window().location().origin().unwrap_or_default() {
                                navigate(
                                    &format!("{}{}", next.pathname(), next.search()),
                                    NavigateOptions::default(),
                                );
                                POSTS.with(|posts| {
                                    posts.update(|count| *count = count.wrapping_add(1));
                                });
                            } else if window().location().assign(&response.url()).is_err() {
                                leptos::logging::warn!(
                                    "the browser refused to follow the redirect"
                                );
                            }
                        }
                        // NOTE: no specification governs this: our own design; an answer that
                        // is no redirect is loaded as a document, never posted twice.
                        Ok(response) => {
                            if window().location().assign(&response.url()).is_err() {
                                leptos::logging::warn!("the browser refused to load the answer");
                            }
                        }
                        Err(error) => {
                            leptos::logging::warn!(
                                "the form could not be posted in the page: {error:?}"
                            );
                            native(&form, by.as_ref());
                        }
                    }
                });
            }
            Route::Browser => {}
        }
    }

    /// Puts the fragment `hash` of a form's address back on the address
    /// `address` the router drew, and brings its element into view, as the
    /// browser's own submit would.
    fn at_fragment(address: &str, hash: &str) {
        let Some(id) = hash.strip_prefix('#').filter(|id| !id.is_empty()) else {
            return;
        };
        let address = format!("{address}{hash}");
        let id = id.to_owned();
        request_animation_frame(move || {
            let Ok(history) = window().history() else {
                return;
            };
            let state = history.state().unwrap_or(JsValue::NULL);
            if history
                .replace_state_with_url(&state, "", Some(&address))
                .is_err()
            {
                leptos::logging::warn!("the browser refused the form's fragment");
            }
            if let Some(element) = document().get_element_by_id(&id) {
                element.scroll_into_view();
            }
        });
    }

    /// Posts `data` to `path` as the form `form` encodes it.
    async fn post(
        path: &str,
        data: &web_sys::FormData,
        form: &web_sys::HtmlFormElement,
    ) -> Result<web_sys::Response, JsValue> {
        let init = web_sys::RequestInit::new();
        init.set_method("POST");
        init.set_credentials(web_sys::RequestCredentials::SameOrigin);
        init.set_redirect(web_sys::RequestRedirect::Follow);
        if form.enctype() == "multipart/form-data" {
            init.set_body(data);
        } else {
            let body = web_sys::UrlSearchParams::new_with_str_sequence_sequence(data)?;
            init.set_body(&body);
        }
        let request = web_sys::Request::new_with_str_and_init(path, &init)?;
        JsFuture::from(window().fetch_with_request(&request))
            .await?
            .dyn_into::<web_sys::Response>()
    }

    /// Gives `form` back to the browser, which posts it as a new document,
    /// the pressed submit `by` included.
    fn native(form: &web_sys::HtmlFormElement, by: Option<&web_sys::HtmlElement>) {
        if form.set_attribute(NATIVE, "").is_err() {
            return;
        }
        let sent = form.request_submit_with_submitter(by);
        if sent.is_err() {
            leptos::logging::warn!("the browser refused to post the form");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Route, route};

    #[test]
    fn a_get_form_navigates_and_a_post_only_when_it_says_its_route_redirects() {
        assert_eq!(Route::Query, route("get", false, false, true));
        assert_eq!(Route::Query, route("GET", true, false, true));
        assert_eq!(Route::Post, route("post", true, false, true));
        assert_eq!(
            Route::Browser,
            route("post", false, false, true),
            "its answer may be a page"
        );
        assert_eq!(Route::Browser, route("dialog", true, false, true));
    }

    #[test]
    fn a_form_that_reloads_or_leaves_the_origin_is_the_browsers() {
        assert_eq!(Route::Browser, route("get", false, true, true));
        assert_eq!(Route::Browser, route("post", true, false, false));
    }
}
