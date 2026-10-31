// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's HTTP service: the pages, the library's assets, the
//! preference and paging posts and the answer routes, under the browser
//! security headers a FerroHEALTH console sends.
//!
//! Every response carries a Content-Security-Policy whose script and style
//! sources are the gallery itself, `nosniff`, `DENY` framing and no
//! referrer, and every page is `no-store`. The stylesheet, the themes and
//! the script modules are served by the library's asset router, as a
//! console mounts it. No specification governs the gallery's HTTP surface:
//! our own design.

use std::process::ExitCode;

use axum::body::Body;
use axum::extract::{Form, Path, Query, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::table::model::Paging;
use ferrohealth_ui::theme::{
    Density, Hints, Language, Motion, Preference, Preferences, Theme, WeekStart,
};
use http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, REFERRER_POLICY, SET_COOKIE,
    X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};
use http::{HeaderValue, Method, StatusCode};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::app::{
    COOKIE_PREFIX, DRAFT_ROUTE, EXPORT_ROUTE, OWN_PREFERENCES, PAGED_LISTINGS, PAGING_COOKIE,
    PAGING_ROUTE, PREFERENCE_ROUTE, PROBE_ROUTE, REMOVE_ROUTE, RESET_ROUTE, SIDEBAR_COOKIE,
    SIDEBAR_ROUTE, SIGN_OUT_ROUTE,
};
use crate::visit::Visit;

/// Builds the gallery's whole HTTP service.
pub fn router() -> axum::Router {
    let pages = axum::Router::new()
        .nest(
            ferrohealth_ui::serve::PREFIX,
            ferrohealth_ui::serve::axum_router::router(),
        )
        .route("/favicon.svg", get(favicon))
        .route("/favicon.ico", get(favicon))
        .route(PREFERENCE_ROUTE, post(choose_preference))
        .route(PROBE_ROUTE, post(probe))
        .route(REMOVE_ROUTE, post(remove_member))
        .route(RESET_ROUTE, post(reset_preferences))
        .route(PAGING_ROUTE, post(choose_paging))
        .route(DRAFT_ROUTE, post(receive_draft))
        .route(&format!("{EXPORT_ROUTE}/{{format}}"), get(export))
        .route(SIDEBAR_ROUTE, post(choose_sidebar))
        .route(&format!("{SIGN_OUT_ROUTE}/{{brand}}"), post(sign_out))
        .route(
            crate::session::SESSION_ROUTE,
            get(session_left).post(session_extended),
        )
        .route(
            &format!("{}/{{brand}}", crate::session::SESSION_START),
            get(session_started),
        )
        .route(
            &format!("{}/{{brand}}", crate::search::SUGGEST_ROUTE),
            get(suggest),
        )
        .fallback(page);
    with_security_headers(pages)
}

/// Draws the page the request names: a `GET` reads it, and a `POST` to a
/// page that takes one draws it with the post's answer.
async fn page(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let visit = Visit::of(&parts);
    let drawn = match parts.method {
        Method::GET | Method::HEAD => crate::routes::draw(&visit),
        Method::POST => {
            let Ok(bytes) = axum::body::to_bytes(body, POST_MAX).await else {
                return StatusCode::PAYLOAD_TOO_LARGE.into_response();
            };
            match crate::routes::posted(&visit, &String::from_utf8_lossy(&bytes)) {
                Some(drawn) => drawn,
                None => return StatusCode::METHOD_NOT_ALLOWED.into_response(),
            }
        }
        _ => return StatusCode::METHOD_NOT_ALLOWED.into_response(),
    };
    let status = drawn.status;
    let html = crate::app::render(&visit, drawn);
    let mut response = axum::response::Html(html.as_str().to_owned()).into_response();
    *response.status_mut() = status;
    response
}

/// The largest form body a page takes, 64 KiB.
const POST_MAX: usize = 64 * 1024;

/// Serves the gallery's favicon: the Lucide layers outline in the
/// FerroHEALTH steel, which a browser asks for on every first page.
async fn favicon() -> Response {
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" \
         stroke=\"#3f6070\" stroke-width=\"2\" stroke-linecap=\"round\" \
         stroke-linejoin=\"round\">{}</svg>",
        ferrohealth_ui::icon::Glyph::LAYERS.markup()
    );
    (
        [(CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"))],
        svg,
    )
        .into_response()
}

/// The page `back` names when it is one of the gallery's own, plain path
/// segments of visible ASCII, and `/` for anything else, so a post never
/// redirects off the gallery.
#[must_use]
pub fn return_path(back: &str) -> &str {
    let plain = back.starts_with('/')
        && !back.starts_with("//")
        && back
            .chars()
            .all(|c| c.is_ascii_graphic() && !matches!(c, '\\' | '?' | '#'))
        && !back.split('/').any(|segment| segment == "..");
    if plain { back } else { "/" }
}

/// The value of the preference `name` as its cookie keeps it, when `value`
/// is one the gallery offers for it, and `None` for anything else.
#[must_use]
pub fn preference_value(name: &str, value: &str) -> Option<String> {
    let kept = |choice: Option<&str>| choice.map(str::to_owned);
    if name == Theme::COOKIE {
        kept(Theme::from_cookie(value).map(Theme::key))
    } else if name == Density::COOKIE {
        kept(Density::from_cookie(value).map(Density::key))
    } else if name == Motion::COOKIE {
        kept(Motion::from_cookie(value).map(Motion::key))
    } else if name == WeekStart::COOKIE {
        kept(WeekStart::from_cookie(value).map(WeekStart::key))
    } else if name == Hints::COOKIE {
        kept(Hints::from_cookie(value).map(Hints::key))
    } else if name == Language::COOKIE {
        let offered = |tag: &str| {
            crate::locale::choices()
                .iter()
                .any(|locale| locale.tag() == tag)
        };
        Language::from_cookie(value)
            .filter(|language| language.tag().is_none_or(offered))
            .map(|language| language.cookie_value().to_owned())
    } else if name == "reveal" {
        kept(
            ["10", "15", "30"]
                .into_iter()
                .find(|seconds| *seconds == value),
        )
    } else if name == "hide" {
        kept(["on", "off"].into_iter().find(|state| *state == value))
    } else if name == "concepts" {
        // NOTE: RFC 5646 section 2.1; a tag's subtags are letters and digits joined by hyphens.
        let tag = (2..=35).contains(&value.len())
            && value.split('-').all(|subtag| {
                !subtag.is_empty() && subtag.chars().all(|c| c.is_ascii_alphanumeric())
            });
        tag.then(|| value.to_owned())
    } else if name == "size" {
        value
            .parse::<u16>()
            .ok()
            .filter(|size| (10..=200).contains(size))
            .map(|size| size.to_string())
    } else {
        None
    }
}

/// Where a preference post returns: the page `back` names, and on Your
/// preferences the same page saying which preference it kept.
#[must_use]
pub fn preference_return(back: &str, name: &str) -> String {
    let page = return_path(back);
    if page.contains("/preferences") {
        format!("{page}?saved={name}")
    } else {
        page.to_owned()
    }
}

/// Keeps the one preference a plain form post chose in its cookie, and
/// sends the browser back to the page it came from.
///
/// The post carries `back` and one preference under its cookie's name; a
/// switch posts `off` before its `on`, and the last value wins.
async fn choose_preference(Form(fields): Form<Vec<(String, String)>>) -> Response {
    let back = fields
        .iter()
        .rev()
        .find(|(name, _)| name == "back")
        .map_or("/", |(_, back)| back.as_str());
    let chosen = fields
        .iter()
        .filter(|(name, _)| name != "back")
        .collect::<Vec<_>>();
    let Some((name, value)) = chosen.last() else {
        return (StatusCode::BAD_REQUEST, "the post names no preference").into_response();
    };
    if chosen.iter().any(|(other, _)| other != name) {
        return (StatusCode::BAD_REQUEST, "a post keeps one preference").into_response();
    }
    let Some(value) = preference_value(name, value) else {
        return (
            StatusCode::BAD_REQUEST,
            "the preference or its value is not one the gallery offers",
        )
            .into_response();
    };
    let cookie =
        format!("{COOKIE_PREFIX}{name}={value}; Path=/; SameSite=Lax; HttpOnly; Max-Age=31536000");
    redirect_with(&preference_return(back, name), &[cookie])
}

/// What Reset to defaults posts.
#[derive(Debug, serde::Deserialize)]
struct Reset {
    /// The page to return to.
    back: String,
}

/// Clears every preference cookie, the gallery's own among them, and sends
/// the browser back to the page it came from.
async fn reset_preferences(Form(reset): Form<Reset>) -> Response {
    let cleared = Preferences::COOKIES
        .into_iter()
        .chain(OWN_PREFERENCES)
        .map(|name| format!("{COOKIE_PREFIX}{name}=; Path=/; SameSite=Lax; HttpOnly; Max-Age=0"))
        .collect::<Vec<_>>();
    redirect_with(return_path(&reset.back), &cleared)
}

/// The redirect to `location` that sets each of `cookies`.
fn redirect_with(location: &str, cookies: &[String]) -> Response {
    let mut response = Redirect::to(location).into_response();
    for cookie in cookies {
        match HeaderValue::from_str(cookie) {
            Ok(value) => {
                response.headers_mut().append(SET_COOKIE, value);
            }
            Err(_invalid) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
    response
}

/// What the sidebar's Collapse or Expand posts.
#[derive(Debug, serde::Deserialize)]
struct SidebarChoice {
    /// `rail` to fold the navigation, `full` to unfold it.
    sidebar: String,
    /// The page to return to.
    back: String,
}

/// Keeps the sidebar's state a plain form post chose in its cookie and
/// sends the browser back to the page it came from.
async fn choose_sidebar(Form(choice): Form<SidebarChoice>) -> Response {
    if !matches!(choice.sidebar.as_str(), "rail" | "full") {
        return (StatusCode::BAD_REQUEST, "the sidebar is not rail or full").into_response();
    }
    kept(SIDEBAR_COOKIE, &choice.sidebar, &choice.back)
}

/// What an export of an answer riding its query's form receives: the
/// query, and the form's other fields, which it leaves alone.
#[derive(Debug, serde::Deserialize)]
struct Exported {
    /// The query the form runs.
    query: String,
}

/// Answers an export in `format`, `csv` or `json`, with the query that rode
/// along as plain text: the gallery holds no answer, so it says what it
/// would export.
async fn export(Path(format): Path<String>, Query(exported): Query<Exported>) -> Response {
    if !matches!(format.as_str(), "csv" | "json") {
        return StatusCode::NOT_FOUND.into_response();
    }
    (
        [(CONTENT_TYPE, "text/plain; charset=utf-8")],
        format!("{format}\n{}", exported.query),
    )
        .into_response()
}

/// What a member's removal posts: the member, why, and the page to return
/// to.
#[derive(Debug, serde::Deserialize)]
struct Removal {
    /// The member removed, one of [`crate::data::MEMBERS`].
    member: String,
    /// Why, which a console keeps with the revision.
    reason: String,
    /// The page to return to.
    back: String,
}

/// Removes a synthetic member: it sends the browser back to the page,
/// naming the member removed. A member the gallery does not list, or a
/// blank reason, answers `400`, as a console refuses a removal with no
/// reason.
async fn remove_member(Form(removal): Form<Removal>) -> Response {
    if !crate::data::MEMBERS.contains(&removal.member.as_str()) {
        return (StatusCode::BAD_REQUEST, "no such member").into_response();
    }
    if removal.reason.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "a removal needs a reason").into_response();
    }
    let back = return_path(&removal.back);
    Redirect::to(&format!("{back}?removed={}", removal.member)).into_response()
}

/// Answers the topology's Probe now: the gallery probes nothing, so it
/// sends the browser back to the page it came from.
async fn probe(headers: http::HeaderMap) -> Response {
    let back = headers
        .get(http::header::REFERER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<http::Uri>().ok())
        .map(|uri| uri.path().to_owned())
        .unwrap_or_default();
    Redirect::to(return_path(&back)).into_response()
}

/// Signs the gallery's synthetic operator out: it holds no session, so it
/// sends the browser to the brand's signed-out page.
/// What the typeahead asks the answer route: the words typed.
#[derive(Debug, serde::Deserialize)]
struct Asked {
    /// The words typed.
    #[serde(default)]
    q: String,
}

/// Answers the top bar's typeahead for `brand` with the search answer as
/// JSON, as a console's answer route does: slowly for
/// [`crate::search::SLOW`], so the panel's waiting row shows, and with
/// `503` for [`crate::search::FAILING`], so its error row shows, and with
/// `401` for [`crate::search::SIGNED_OUT`], so the page loads the brand's
/// signed-out sign-in.
async fn suggest(
    Path(brand): Path<String>,
    Query(asked): Query<Asked>,
    headers: http::HeaderMap,
) -> Response {
    let Some(brand) = Brand::parse(&brand) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let cookies = headers
        .get(http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let locale = Visit::new("/", "", cookies).locale;
    match asked.q.trim() {
        crate::search::FAILING => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        crate::search::SIGNED_OUT => StatusCode::UNAUTHORIZED.into_response(),
        crate::search::SLOW => {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            axum::Json(crate::search::answer(&locale, brand, &asked.q)).into_response()
        }
        _ => axum::Json(crate::search::answer(&locale, brand, &asked.q)).into_response(),
    }
}

async fn sign_out(Path(brand): Path<String>) -> Response {
    let Some(brand) = Brand::parse(&brand) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    redirect_with(
        &format!("/sign-in/{}/signed-out", brand.key()),
        &[session_ended()],
    )
}

/// The cookie that ends the gallery's session.
fn session_ended() -> String {
    format!(
        "{}=; Path=/; SameSite=Lax; HttpOnly; Max-Age=0",
        crate::session::SESSION_COOKIE
    )
}

/// The cookie that keeps the session `kept`.
fn session_cookie(kept: crate::session::Kept) -> String {
    format!(
        "{}={}; Path=/; SameSite=Lax; HttpOnly",
        crate::session::SESSION_COOKIE,
        kept.value()
    )
}

/// The lengths a journey starts a session with, in seconds.
#[derive(Debug, serde::Deserialize)]
struct Lengths {
    /// The idle timeout.
    idle: u32,
    /// The absolute lifetime.
    absolute: u32,
}

/// Starts a session of the `idle` and `absolute` lengths, and leads to the
/// brand's overview.
async fn session_started(Path(brand): Path<String>, Query(lengths): Query<Lengths>) -> Response {
    let Some(brand) = Brand::parse(&brand) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let kept = crate::session::Kept::started(crate::session::now(), lengths.idle, lengths.absolute);
    redirect_with(&format!("/{}", brand.key()), &[session_cookie(kept)])
}

/// The session the request's cookies keep, while it lasts.
fn session_of(headers: &http::HeaderMap) -> Option<crate::session::Kept> {
    let cookies = headers
        .get(http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    crate::session::Kept::read(cookies).filter(|kept| kept.left(crate::session::now()).is_some())
}

/// The answer that the session is gone: `401`, and its cookie cleared.
fn session_gone() -> Response {
    let mut response = StatusCode::UNAUTHORIZED.into_response();
    if let Ok(value) = HeaderValue::from_str(&session_ended()) {
        response.headers_mut().append(SET_COOKIE, value);
    }
    response
}

/// What the session has left, which a read never extends.
async fn session_left(headers: http::HeaderMap) -> Response {
    let now = crate::session::now();
    match session_of(&headers).and_then(|kept| kept.left(now)) {
        Some(left) => axum::Json(left).into_response(),
        None => session_gone(),
    }
}

/// The user's activity: the idle timeout starts again, and the answer says
/// what the session has left.
async fn session_extended(headers: http::HeaderMap) -> Response {
    let now = crate::session::now();
    let Some(kept) = session_of(&headers).map(|kept| kept.extended(now)) else {
        return session_gone();
    };
    let Some(left) = kept.left(now) else {
        return session_gone();
    };
    let mut response = axum::Json(left).into_response();
    match HeaderValue::from_str(&session_cookie(kept)) {
        Ok(value) => {
            response.headers_mut().append(SET_COOKIE, value);
            response
        }
        Err(_invalid) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// The redirect to `back` that keeps `value` in the cookie `name`.
fn kept(name: &str, value: &str, back: &str) -> Response {
    let cookie = format!("{name}={value}; Path=/; SameSite=Lax; HttpOnly; Max-Age=31536000");
    with_cookie(cookie, back)
}

/// The redirect to `back` that sets `cookie`.
fn with_cookie(cookie: String, back: &str) -> Response {
    redirect_with(return_path(back), &[cookie])
}

/// What a session listing's pager posts: a step carries the page beside the
/// size in force, a size the first page beside the size chosen.
#[derive(Debug, serde::Deserialize)]
struct PagingChoice {
    /// The key of the listing paged, one of [`PAGED_LISTINGS`].
    listing: String,
    /// The 1-based page asked for, read defensively.
    page: String,
    /// The page size asked for, read defensively and clamped.
    size: String,
    /// The page to return to.
    back: String,
}

/// Keeps the position a session listing's pager posted in a cookie that
/// lasts the browser's session, never in a URL, and sends the browser back
/// to the page, which the server draws at that position.
async fn choose_paging(Form(choice): Form<PagingChoice>) -> Response {
    if !PAGED_LISTINGS.contains(&choice.listing.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            "the listing is not one the gallery pages in its session",
        )
            .into_response();
    }
    let paging = Paging::read(Some(&choice.page), Some(&choice.size));
    let cookie = format!(
        "{PAGING_COOKIE}{}={}.{}; Path=/; SameSite=Lax; HttpOnly",
        choice.listing, paging.page, paging.size
    );
    with_cookie(cookie, &choice.back)
}

/// What the code editor's form posts: the draft, under its text area's name.
#[derive(serde::Deserialize)]
struct DraftPost {
    /// The draft as the form posted it.
    #[serde(rename = "gallery-mapping")]
    draft: String,
}

/// Answers the code editor's posted draft with a page that shows the draft
/// as it arrived, so a journey reads what the server received.
async fn receive_draft(Form(post): Form<DraftPost>) -> Response {
    // NOTE: HTML Living Standard §4.10.21.8: a form posts a text area's line breaks as
    // CR LF, which the server reads back as the LF the editor holds.
    let draft = post.draft.replace("\r\n", "\n");
    let mut page = String::from(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Draft received</title></head><body><pre id=\"received\">",
    );
    // NOTE: HTML Living Standard §13.2.6.4.7: the parser drops one line break right after
    // `<pre>`, so a draft that opens with one is written with two.
    if draft.starts_with('\n') {
        page.push('\n');
    }
    ferrohealth_ui::highlight::escape(&draft, &mut page);
    page.push_str("</pre></body></html>");
    axum::response::Html(page).into_response()
}

/// Wraps `router` in the browser security headers every response carries.
fn with_security_headers(router: axum::Router) -> axum::Router {
    router
        .layer(axum::middleware::from_fn(content_security_policy))
        .layer(axum::middleware::from_fn(cache_control))
        .layer(SetResponseHeaderLayer::overriding(
            X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
}

/// Answers with the Content-Security-Policy every response carries.
async fn content_security_policy(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(POLICY));
    response
}

/// The Content-Security-Policy of every response: scripts and styles from
/// the gallery alone, no inline script or style and no `eval` (Content
/// Security Policy Level 3).
pub const POLICY: &str = "default-src 'self'; \
     script-src 'self'; \
     style-src 'self'; \
     img-src 'self' data:; \
     connect-src 'self'; \
     object-src 'none'; \
     base-uri 'self'; \
     form-action 'self'; \
     frame-ancestors 'none'";

/// Marks every page `no-store`, and the library's assets `no-cache`, so a
/// rebuilt one is fetched again (RFC 9111 §5.2.2).
async fn cache_control(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path();
    let asset = path.starts_with(ferrohealth_ui::serve::PREFIX) || path == "/favicon.svg";
    let mut response = next.run(request).await;
    let value = if asset && response.status().is_success() {
        "no-cache"
    } else {
        "no-store"
    };
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(value));
    response
}

/// Serves the gallery until the process is asked to stop.
pub async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_absent| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let addr =
        std::env::var("GALLERY_ADDR").unwrap_or_else(|_absent| String::from("127.0.0.1:3000"));
    let Ok(addr) = addr.parse::<std::net::SocketAddr>() else {
        tracing::error!(%addr, "GALLERY_ADDR is no socket address");
        return ExitCode::FAILURE;
    };
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, %addr, "the gallery cannot listen");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(%addr, "the gallery serves");
    let router = router();
    let shutdown = async {
        if tokio::signal::ctrl_c().await.is_err() {
            tracing::warn!("the gallery cannot wait for a stop signal");
        }
    };
    match axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "the gallery stopped serving");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{POLICY, preference_return, preference_value, return_path};

    #[test]
    fn the_policy_allows_scripts_and_styles_from_the_gallery_alone() {
        let policy = POLICY;
        assert!(policy.contains("script-src 'self';"), "{policy}");
        assert!(policy.contains("style-src 'self';"), "{policy}");
        assert!(!policy.contains("unsafe-inline"), "{policy}");
        assert!(!policy.contains("unsafe-eval"), "{policy}");
        assert!(!policy.contains("nonce-"), "{policy}");
    }

    #[test]
    fn a_preference_post_keeps_only_a_value_the_gallery_offers() {
        assert_eq!(
            Some(String::from("dark")),
            preference_value("theme", "dark")
        );
        assert_eq!(None, preference_value("theme", "sepia"));
        assert_eq!(
            Some(String::from("reduce")),
            preference_value("motion", "reduce")
        );
        assert_eq!(
            Some(String::from("sunday")),
            preference_value("week-start", "sunday")
        );
        assert_eq!(
            Some(String::from("never")),
            preference_value("hints", "never")
        );
        assert_eq!(
            Some(String::from("browser")),
            preference_value("language", "browser")
        );
        assert_eq!(
            Some(String::from("qps")),
            preference_value("language", "qps")
        );
        assert_eq!(
            None,
            preference_value("language", "nl"),
            "a language the gallery lacks"
        );
        assert_eq!(Some(String::from("30")), preference_value("reveal", "30"));
        assert_eq!(
            None,
            preference_value("reveal", "300"),
            "the server bounds a reveal"
        );
        assert_eq!(Some(String::from("off")), preference_value("hide", "off"));
        assert_eq!(
            Some(String::from("nl-NL")),
            preference_value("concepts", "nl-NL")
        );
        assert_eq!(
            None,
            preference_value("concepts", "nl NL"),
            "a tag has no space"
        );
        assert_eq!(Some(String::from("50")), preference_value("size", "50"));
        assert_eq!(
            None,
            preference_value("size", "500"),
            "the server bounds a page size"
        );
        assert_eq!(None, preference_value("sidebar", "rail"), "no preference");
    }

    #[test]
    fn a_preference_post_returns_saying_what_it_kept_only_on_your_preferences() {
        assert_eq!(
            "/ferropix/preferences?saved=reveal",
            preference_return("/ferropix/preferences", "reveal")
        );
        assert_eq!("/ferrofed", preference_return("/ferrofed", "theme"));
        assert_eq!(
            "/",
            preference_return("//evil.example/preferences", "theme")
        );
    }

    #[test]
    fn a_return_path_never_leaves_the_gallery() {
        assert_eq!("/ferrofed/tables", return_path("/ferrofed/tables"));
        for hostile in [
            "//evil.example",
            "https://evil.example",
            "/a/../b",
            "/a?b",
            "",
            "/a b",
        ] {
            assert_eq!("/", return_path(hostile), "{hostile}");
        }
    }
}
