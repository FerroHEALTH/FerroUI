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
//!
//! An action whose answer is the point (a check, a run, a load report)
//! draws [`answer_field`] in its form instead. The console's router,
//! wrapped in [`draw_answers`], answers such a plain post with the page the
//! field names, drawn with the function's answer, or its error, as the
//! action's value, and [`answered`] hands the page the fields the form
//! posted. With WebAssembly the action runs in the page as before.

use leptos::prelude::*;
use leptos::server_fn::codec::{Post, PostUrl};
use leptos::server_fn::error::{
    FromServerFnError, IntoAppError as _, ServerFnErrorErr, ServerFnUrlError,
};
use leptos::server_fn::{Bytes, Decodes, Http, ServerFn};
use serde::{Deserialize, Serialize};

/// The name of the field that carries the page to return to, which the
/// console's server function takes as an argument of this name, an
/// `Option<String>`.
pub const RETURN: &str = "ferroui_return";

/// The name of the field that asks for a plain post's answer drawn on the
/// page it names ([`answer_field`]).
pub const ANSWER: &str = "ferroui_answer";

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

/// Returns the hidden field that asks for a plain post's answer drawn on
/// the page in view, its path and its query, inside a console's
/// `<ActionForm>`.
///
/// With no WebAssembly, [`draw_answers`] answers the post with that page,
/// the function's answer as the action's value and the posted fields
/// handed to [`answered`]. It reads the router's address as
/// [`return_field`] does; an empty field leaves the post to the function.
#[must_use]
pub fn answer_field() -> impl IntoView {
    let path = crate::widget::nav::current_path(String::new());
    view! { <input type="hidden" name=ANSWER value=move || path.get() /> }
}

/// The fields a plain post of an [`answer_field`] form carried, as the page
/// drawn with its answer reads them back.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Posted {
    /// Each field's name and value, in the order the form posted them.
    fields: Vec<(String, String)>,
}

impl Posted {
    /// The value of the first field named `name`, as the form posted it
    /// (`arg[field]` for a nested argument).
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value.as_str())
    }

    /// Whether the page was drawn with no posted fields: on every request
    /// but a plain post of this action.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// What [`draw_answers`] hands the page it draws: the function's answer,
/// encoded as the function answers its own client, and the posted fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Answered {
    /// The address of the server function that answered.
    function: String,
    /// Whether it answered with success, so `body` is its output, or with
    /// its error.
    ok: bool,
    /// The function's answer as its encoding wrote it.
    body: String,
    /// The fields of the plain post.
    posted: Posted,
}

/// Fills `action`'s value with the answer to a plain post of its form, when
/// [`draw_answers`] draws this page with one, and returns the fields that
/// post carried, empty otherwise.
///
/// Call it where the action is made, on the server and in the browser
/// alike: the answer crosses to the browser with the page, so the page
/// hydrates as the server drew it. The value is the function's output or
/// its error, decoded as the action's own call decodes it.
#[must_use = "the posted fields are what the form draws again"]
pub fn answered<S, Enc>(action: &Action<S, Result<S::Output, S::Error>>) -> Posted
where
    S: ServerFn<Protocol = Http<PostUrl, Post<Enc>>> + Send + Sync + 'static,
    S::Output: Send + Sync + 'static,
    S::Error: Send + Sync + 'static,
    Enc: Decodes<S::Output>,
{
    let carried = SharedValue::new(move || carried(S::PATH)).into_inner();
    let Some(answered) = carried else {
        return Posted::default();
    };
    action.value().set(Some(outcome::<S::Output, S::Error, Enc>(
        answered.ok,
        answered.body,
    )));
    answered.posted
}

/// The answer [`draw_answers`] left on this request for the function at
/// `function`.
#[cfg(not(target_arch = "wasm32"))]
fn carried(function: &str) -> Option<Answered> {
    use_context::<http::request::Parts>()?
        .extensions
        .get::<Answered>()
        .filter(|answered| answered.function == function)
        .cloned()
}

/// The browser draws a page with no request of its own; a page the server
/// drew carries its answer across in the page.
#[cfg(target_arch = "wasm32")]
fn carried(_function: &str) -> Option<Answered> {
    None
}

/// The function's answer decoded: its output when `ok`, its error
/// otherwise, and a deserialization error when the output does not decode.
fn outcome<O, E, Enc>(ok: bool, body: String) -> Result<O, E>
where
    E: FromServerFnError,
    Enc: Decodes<O>,
{
    let bytes = Bytes::from(body);
    if ok {
        Enc::decode(bytes)
            .map_err(|error| ServerFnErrorErr::Deserialization(error.to_string()).into_app_error())
    } else {
        Err(E::de(bytes))
    }
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
    if !own_path(field) {
        return None;
    }
    Some(without_error(field))
}

/// Whether `field` is a path on the console's own origin.
pub(crate) fn own_path(field: &str) -> bool {
    field.starts_with('/')
        && !field.starts_with("//")
        && !field.contains('\\')
        && !field.chars().any(char::is_control)
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

/// The largest plain post, and the largest answer to it, that
/// [`draw_answers`] reads whole.
#[cfg(not(target_arch = "wasm32"))]
pub const ANSWER_LIMIT: usize = 16 * 1024 * 1024;

/// Wraps a console's whole router so a plain post of an [`answer_field`]
/// form is answered with the page the field names, drawn with the
/// function's answer.
///
/// Such a post, a form post that accepts HTML and carries the field, goes
/// to its server function without the field and asking for no HTML, so the
/// function answers as to its own client and redirects nowhere; then the
/// page is drawn as a `GET` of the field's address with the request's own
/// headers, and [`answered`] reads the answer and the posted fields from
/// it. The page answers with its own status, as the page a failed
/// redirect-mode post returns to does, with the function's error in the
/// action's value, with the cookies the function set and with
/// `Cache-Control: no-store`. A
/// function that redirects is followed, and every other request passes to
/// `router` untouched, a call from the page's own WebAssembly among them.
/// Wrap the router your routes and layers make, inside any layer that
/// checks the origin of a state-changing request: the forwarded post keeps
/// the request's headers, so such a check sees it as the browser sent it.
#[cfg(not(target_arch = "wasm32"))]
pub fn draw_answers(router: axum::Router) -> axum::Router {
    axum::Router::new()
        .fallback(move |request: axum::extract::Request| answer_plain_post(router.clone(), request))
}

/// Answers `request` through `router`, drawing the answer to a plain post
/// of an [`answer_field`] form on its page.
#[cfg(not(target_arch = "wasm32"))]
async fn answer_plain_post(
    router: axum::Router,
    request: axum::extract::Request,
) -> axum::response::Response {
    use axum::body::Body;
    use axum::response::IntoResponse as _;
    use http::header::{ACCEPT, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, SET_COOKIE};
    use http::{HeaderValue, Method, Request, StatusCode, Uri};

    let accept = header(request.headers(), ACCEPT).map(str::to_owned);
    let form = request.method() == Method::POST
        && header(request.headers(), CONTENT_TYPE)
            .is_some_and(|kind| kind.starts_with("application/x-www-form-urlencoded"));
    if !form
        || !accept
            .as_deref()
            .is_some_and(|accept| accept.contains("text/html"))
    {
        return pass(router, request).await;
    }
    let (parts, body) = request.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, ANSWER_LIMIT).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };
    let field = form_urlencoded::parse(&bytes)
        .find(|(name, _)| name == ANSWER)
        .map(|(_, value)| value.into_owned());
    let page =
        plain_return(field.as_deref(), accept.as_deref()).and_then(|page| Uri::try_from(page).ok());
    let Some(page) = page else {
        return pass(router, Request::from_parts(parts, Body::from(bytes))).await;
    };
    let posted = Posted {
        fields: form_urlencoded::parse(&bytes)
            .filter(|(name, _)| name != ANSWER)
            .map(|(name, value)| (name.into_owned(), value.into_owned()))
            .collect(),
    };
    let function = parts.uri.path().to_owned();
    let headers = parts.headers.clone();
    let extensions = parts.extensions.clone();
    let version = parts.version;
    let mut forwarded = Request::from_parts(parts, Body::from(without_answer(&bytes)));
    forwarded.headers_mut().remove(CONTENT_LENGTH);
    forwarded
        .headers_mut()
        .insert(ACCEPT, HeaderValue::from_static("*/*"));
    let reply = pass(router.clone(), forwarded).await;
    if reply.status().is_redirection() {
        return reply;
    }
    let (reply, body) = reply.into_parts();
    let Ok(body) = axum::body::to_bytes(body, ANSWER_LIMIT).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "the action's answer is too large to draw",
        )
            .into_response();
    };
    let Ok(body) = String::from_utf8(body.to_vec()) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "the action's answer is not text, which its page cannot draw",
        )
            .into_response();
    };
    let mut drawn = Request::new(Body::empty());
    *drawn.method_mut() = Method::GET;
    *drawn.uri_mut() = page;
    *drawn.version_mut() = version;
    *drawn.headers_mut() = headers;
    drawn.headers_mut().remove(CONTENT_TYPE);
    drawn.headers_mut().remove(CONTENT_LENGTH);
    *drawn.extensions_mut() = extensions;
    drawn.extensions_mut().insert(Answered {
        function,
        ok: reply.status.is_success(),
        body,
        posted,
    });
    let mut response = pass(router, drawn).await;
    for cookie in reply.headers.get_all(SET_COOKIE) {
        response.headers_mut().append(SET_COOKIE, cookie.clone());
    }
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// What `router` answers `request` with.
#[cfg(not(target_arch = "wasm32"))]
async fn pass(router: axum::Router, request: axum::extract::Request) -> axum::response::Response {
    use tower::ServiceExt as _;
    match router.oneshot(request).await {
        Ok(response) => response,
        Err(infallible) => match infallible {},
    }
}

/// The value of the header `name`, when it is visible ASCII.
#[cfg(not(target_arch = "wasm32"))]
fn header(headers: &http::HeaderMap, name: http::header::HeaderName) -> Option<&str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// The form body `bytes` without its [`ANSWER`] field, which is the page's
/// and never the function's.
#[cfg(not(target_arch = "wasm32"))]
fn without_answer(bytes: &[u8]) -> Vec<u8> {
    bytes
        .split(|byte| *byte == b'&')
        .filter(|pair| {
            !pair.is_empty()
                && form_urlencoded::parse(pair)
                    .next()
                    .is_none_or(|(name, _)| name != ANSWER)
        })
        .collect::<Vec<_>>()
        .join(&b'&')
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod answers {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::response::{IntoResponse as _, Response};
    use axum::routing::{get, post};
    use http::header::{ACCEPT, CACHE_CONTROL, CONTENT_TYPE, LOCATION, SET_COOKIE};
    use http::{HeaderMap, StatusCode};
    use leptos::server_fn::codec::JsonEncoding;
    use leptos::server_fn::error::{FromServerFnError as _, ServerFnError};

    use std::error::Error;

    use super::{Answered, Posted, draw_answers, outcome};

    const HTML: &str = "text/html,application/xhtml+xml;q=0.9,*/*;q=0.8";

    /// What the function received, as it answers its own client: the
    /// `Accept` it saw and the body it read.
    async fn check(headers: HeaderMap, body: String) -> Response {
        let accept = headers
            .get(ACCEPT)
            .and_then(|accept| accept.to_str().ok())
            .unwrap_or_default();
        match serde_json::to_string(&format!("{accept}|{body}")) {
            Ok(answer) => ([(SET_COOKIE, "seen=1; Path=/")], answer).into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    /// A function that fails.
    async fn fail() -> Response {
        (
            StatusCode::BAD_REQUEST,
            ServerFnError::new("the note is empty").ser(),
        )
            .into_response()
    }

    /// A function that sends the browser elsewhere.
    async fn leave() -> Response {
        (StatusCode::FOUND, [(LOCATION, "/signed-out")]).into_response()
    }

    /// The page, drawing what it was handed as JSON with its own address.
    async fn page(request: Request) -> Response {
        let answered = request.extensions().get::<Answered>().cloned();
        let drawn = serde_json::json!({
            "address": request.uri().to_string(),
            "method": request.method().as_str(),
            "answered": answered,
        });
        ([(CACHE_CONTROL, "max-age=60")], drawn.to_string()).into_response()
    }

    fn console() -> axum::Router {
        draw_answers(
            axum::Router::new()
                .route("/api/check", post(check))
                .route("/api/fail", post(fail))
                .route("/api/leave", post(leave))
                .route("/page", get(page)),
        )
    }

    async fn send(
        path: &str,
        accept: &str,
        body: &str,
    ) -> Result<(http::response::Parts, String), Box<dyn Error>> {
        let request = http::Request::post(path)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(ACCEPT, accept)
            .body(Body::from(body.to_owned()))?;
        let response = tower::ServiceExt::oneshot(console(), request).await?;
        let (parts, body) = response.into_parts();
        let body = axum::body::to_bytes(body, usize::MAX).await?;
        Ok((parts, String::from_utf8(body.to_vec())?))
    }

    fn answered_of(body: &str) -> Result<(serde_json::Value, Answered), Box<dyn Error>> {
        let drawn: serde_json::Value = serde_json::from_str(body)?;
        let answered = serde_json::from_value(drawn.get("answered").cloned().unwrap_or_default())?;
        Ok((drawn, answered))
    }

    #[tokio::test]
    async fn a_plain_post_draws_its_page_with_the_answer_and_the_posted_fields() {
        let (parts, body) = send(
            "/api/check",
            HTML,
            "note=two+words&ferroui_answer=%2Fpage%3Ftab%3Dtwo&shown=%C3%A9",
        )
        .await
        .unwrap();
        assert_eq!(StatusCode::OK, parts.status);
        assert_eq!("no-store", parts.headers[CACHE_CONTROL]);
        assert_eq!("seen=1; Path=/", parts.headers[SET_COOKIE]);
        let (drawn, answered) = answered_of(&body).unwrap();
        assert_eq!("/page?tab=two", drawn["address"]);
        assert_eq!("GET", drawn["method"]);
        assert_eq!("/api/check", answered.function);
        assert!(answered.ok);
        assert_eq!(
            Ok(String::from("*/*|note=two+words&shown=%C3%A9")),
            outcome::<String, ServerFnError, JsonEncoding>(answered.ok, answered.body),
            "the function answers as to its own client, without the page's field"
        );
        assert_eq!(Some("two words"), answered.posted.field("note"));
        assert_eq!(Some("é"), answered.posted.field("shown"));
        assert_eq!(None, answered.posted.field("ferroui_answer"));
    }

    #[tokio::test]
    async fn a_failed_plain_post_draws_its_page_with_the_error_as_the_value() {
        let (parts, body) = send("/api/fail", HTML, "note=&ferroui_answer=%2Fpage")
            .await
            .unwrap();
        assert_eq!(
            StatusCode::OK,
            parts.status,
            "the page is drawn, with the error as the action's value"
        );
        assert_eq!("no-store", parts.headers[CACHE_CONTROL]);
        let (_, answered) = answered_of(&body).unwrap();
        assert!(!answered.ok);
        assert_eq!(
            Err(ServerFnError::new("the note is empty")),
            outcome::<String, ServerFnError, JsonEncoding>(answered.ok, answered.body)
        );
        assert_eq!(Some(""), answered.posted.field("note"));
    }

    #[tokio::test]
    async fn a_function_that_redirects_is_followed() {
        let (parts, _) = send("/api/leave", HTML, "ferroui_answer=%2Fpage")
            .await
            .unwrap();
        assert_eq!(StatusCode::FOUND, parts.status);
        assert_eq!("/signed-out", parts.headers[LOCATION]);
    }

    #[tokio::test]
    async fn every_other_request_reaches_the_router_untouched() {
        for (accept, body) in [
            (HTML, "note=seen"),
            ("application/json", "note=seen&ferroui_answer=%2Fpage"),
            (HTML, "note=seen&ferroui_answer=%2F%2Fevil.example%2F"),
            (HTML, "note=seen&ferroui_answer="),
        ] {
            let (parts, answer) = send("/api/check", accept, body).await.unwrap();
            assert_eq!(StatusCode::OK, parts.status, "{body}");
            assert_eq!(
                Ok(format!("{accept}|{body}")),
                outcome::<String, ServerFnError, JsonEncoding>(true, answer),
                "{accept} {body}"
            );
        }
    }

    #[test]
    fn an_answer_that_does_not_decode_is_an_error() {
        assert!(matches!(
            outcome::<u32, ServerFnError, JsonEncoding>(true, String::from("words")),
            Err(ServerFnError::Deserialization(_))
        ));
    }

    #[test]
    fn a_page_with_no_post_has_no_fields() {
        let posted = Posted::default();
        assert!(posted.is_empty());
        assert_eq!(None, posted.field("note"));
    }
}
