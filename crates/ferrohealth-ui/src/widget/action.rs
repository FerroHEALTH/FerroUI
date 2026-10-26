// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page a console's action returns to when the browser has no
//! WebAssembly.
//!
//! A console's action is a server function behind `<ActionForm>`. With no
//! WebAssembly the form posts to the function's own address, and `server_fn`'s
//! `form-redirects` sends the browser back to the `Referer` (`server_fn`
//! 0.8.13 `src/lib.rs`, `run_on_server`), a header every console's
//! `Referrer-Policy: no-referrer` withholds (W3C Referrer Policy §3.1), so
//! the browser lands on `/`. [`return_field`] carries the page in the form
//! instead, and the function sends the browser back to [`plain_return`],
//! with the error the router reads into the action's value written by
//! [`with_error`] (`leptos_server` 0.8 `ServerAction::new`). No specification
//! governs this: our own design.
//!
//! The function takes the field as an argument named [`RETURN`], and on a
//! plain post calls `leptos_axum::redirect` with [`plain_return`], or with
//! [`with_error`] when it fails; the gallery's `acknowledge` server function
//! is the pattern.

use leptos::prelude::*;
use leptos::server_fn::error::{FromServerFnError, ServerFnUrlError};

/// The name of the field that carries the page to return to, which the
/// console's server function takes as an argument of this name, an
/// `Option<String>`.
pub const RETURN: &str = "ferroui_return";

/// The query keys `server_fn` writes an action's error under, which the
/// router reads back into the action's value.
const ERROR_KEYS: [&str; 2] = ["__path", "__err"];

/// The base [`with_error`] parses a path against, which never leaves it.
const BASE: &str = "http://return.invalid";

/// Returns the hidden field that carries the page in view, its path and its
/// query, inside a console's `<ActionForm>`.
///
/// It reads the router's address ([`crate::widget::nav::follow_path`]), so
/// the field names the page in view after every navigation in the page.
/// Without it the field is empty, and [`plain_return`] refuses it.
#[must_use]
pub fn return_field() -> impl IntoView {
    let path = crate::widget::nav::current_path(String::new());
    view! { <input type="hidden" name=RETURN value=move || path.get() /> }
}

/// The page a plain post of the form returns to, without the error of an
/// earlier post.
///
/// That is `field`, the [`RETURN`] field the form posted, when `accept`,
/// the request's `Accept` header, asks for HTML as a plain form post does
/// (`leptos_axum` 0.8 `redirect`) and the field is a path on the console's
/// own origin.
///
/// `None` for a call from the page's own WebAssembly, which draws the
/// outcome in place, and for a field that names another origin, a scheme or
/// no path, so the function never sends the browser elsewhere.
#[must_use]
pub fn plain_return(field: Option<&str>, accept: Option<&str>) -> Option<String> {
    if !accept.is_some_and(|accept| accept.contains("text/html")) {
        return None;
    }
    let field = field?;
    let own = field.starts_with('/')
        && !field.starts_with("//")
        && !field.contains('\\')
        && !field.chars().any(char::is_control);
    if !own {
        return None;
    }
    Some(without_error(field))
}

/// `back` with the error of the server function at `function` added, as
/// `server_fn`'s own `form-redirects` writes it, so the page it returns to
/// draws the error in the action's value.
#[must_use]
pub fn with_error<E: FromServerFnError>(back: &str, function: &str, error: E) -> String {
    match ServerFnUrlError::new(function, error).to_url(&format!("{BASE}{back}")) {
        Ok(url) => match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_owned(),
        },
        Err(_) => back.to_owned(),
    }
}

/// `path` without the query pairs `server_fn` writes an action's error under.
fn without_error(path: &str) -> String {
    let Some((route, query)) = path.split_once('?') else {
        return path.to_owned();
    };
    let kept = query
        .split('&')
        .filter(|pair| {
            let key = pair.split('=').next().unwrap_or_default();
            !pair.is_empty() && !ERROR_KEYS.contains(&key)
        })
        .collect::<Vec<_>>();
    if kept.is_empty() {
        route.to_owned()
    } else {
        format!("{route}?{}", kept.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use leptos::server_fn::error::{ServerFnError, ServerFnUrlError};

    use super::{plain_return, with_error};

    const HTML: Option<&str> = Some("text/html,application/xhtml+xml;q=0.9,*/*;q=0.8");

    #[test]
    fn a_plain_post_returns_to_its_own_page_with_its_query() {
        assert_eq!(
            Some(String::from("/files?sort=name")),
            plain_return(Some("/files?sort=name"), HTML)
        );
        assert_eq!(Some(String::from("/")), plain_return(Some("/"), HTML));
    }

    #[test]
    fn a_call_from_the_page_returns_nowhere_since_it_draws_the_outcome_in_place() {
        assert_eq!(None, plain_return(Some("/files"), Some("application/json")));
        assert_eq!(None, plain_return(Some("/files"), None));
    }

    #[test]
    fn a_field_naming_another_origin_a_scheme_or_no_path_is_refused() {
        for field in [
            "//evil.example/files",
            "/\\evil.example",
            "https://evil.example/",
            "javascript:alert(1)",
            "files",
            "",
            "/files\r\nLocation: /x",
        ] {
            assert_eq!(None, plain_return(Some(field), HTML), "{field:?}");
        }
        assert_eq!(None, plain_return(None, HTML));
    }

    #[test]
    fn the_error_of_an_earlier_post_is_left_behind() {
        assert_eq!(
            Some(String::from("/files?sort=name")),
            plain_return(
                Some("/files?__path=%2Fapi%2Fsave&sort=name&__err=e30"),
                HTML
            )
        );
        assert_eq!(
            Some(String::from("/files")),
            plain_return(Some("/files?__path=%2Fapi%2Fsave&__err=e30"), HTML)
        );
    }

    #[test]
    fn an_error_rides_back_as_the_router_reads_it() {
        let back = with_error(
            "/files?sort=name",
            "/api/save",
            ServerFnError::new("the name is taken"),
        );
        assert!(
            back.starts_with("/files?sort=name&__path=%2Fapi%2Fsave&__err="),
            "{back}"
        );
        let encoded = back
            .split('&')
            .find_map(|pair| pair.strip_prefix("__err="))
            .unwrap()
            .replace("%3D", "=");
        assert_eq!(
            ServerFnError::new("the name is taken"),
            ServerFnUrlError::<ServerFnError>::decode_err(&encoded)
        );
    }
}
