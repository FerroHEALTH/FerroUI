// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's HTTP service: the pages, the library's stylesheet and the
//! brand themes, the theme choice and the site bundle, under the browser
//! security headers a FerroHEALTH console sends.
//!
//! Every response carries a Content-Security-Policy whose script source is
//! a nonce minted for that response alone and whose style source is the
//! gallery itself, `nosniff`, `DENY` framing and no referrer, and every page
//! is `no-store`. The stylesheet and the themes are served from the
//! library's constants, as a console serves them. No specification governs
//! the gallery's HTTP surface: our own design.

use std::process::ExitCode;

use axum::extract::{Form, Path, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use ferrohealth_ui::style::{Brand, STYLESHEET};
use ferrohealth_ui::table::model::Paging;
use ferrohealth_ui::theme::{Density, Theme};
use http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, REFERRER_POLICY, SET_COOKIE,
    X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};
use http::{HeaderValue, StatusCode};
use leptos::config::LeptosOptions;
use leptos::nonce::Nonce;
use leptos_axum::LeptosRoutes as _;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::app::{
    DENSITY_COOKIE, DENSITY_ROUTE, LOCALE_COOKIE, LOCALE_ROUTE, PAGED_LISTINGS, PAGING_COOKIE,
    PAGING_ROUTE, STYLESHEET as STYLESHEET_PATH, THEME_COOKIE, THEME_ROUTE,
};

/// The name cargo-leptos gives the site bundle's files under `pkg/`.
pub const OUTPUT_NAME: &str = "ferrohealth-ui-gallery";

/// The Leptos options the gallery renders with: its site bundle under
/// `site_root`.
#[must_use]
pub fn leptos_options(site_root: &str, addr: std::net::SocketAddr) -> LeptosOptions {
    LeptosOptions::builder()
        .output_name(OUTPUT_NAME)
        .site_root(site_root)
        .site_pkg_dir("pkg")
        .site_addr(addr)
        .build()
}

/// Builds the gallery's whole HTTP service, its site bundle under
/// `options`' site root.
pub fn router(options: LeptosOptions) -> axum::Router {
    let routes = leptos_axum::generate_route_list(crate::app::App);
    let pages = axum::Router::new()
        .route(STYLESHEET_PATH, get(stylesheet))
        .route("/favicon.svg", get(favicon))
        .route("/favicon.ico", get(favicon))
        .route("/themes/{file}", get(theme_file))
        .route(THEME_ROUTE, post(choose_theme))
        .route(DENSITY_ROUTE, post(choose_density))
        .route(LOCALE_ROUTE, post(choose_locale))
        .route(PAGING_ROUTE, post(choose_paging))
        .leptos_routes_with_context(&options, routes, provide_request_nonce, {
            let options = options.clone();
            move || crate::app::document(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler_with_context(
            provide_request_nonce,
            crate::app::document,
        ));
    with_security_headers(pages).with_state(options)
}

/// Serves the library's stylesheet.
async fn stylesheet() -> Response {
    css(STYLESHEET)
}

/// Serves the theme file `file` names, `<brand>.css`, or `404`.
async fn theme_file(Path(file): Path<String>) -> Response {
    match file.strip_suffix(".css").and_then(Brand::parse) {
        Some(brand) => css(brand.theme()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

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

/// A stylesheet response.
fn css(text: &'static str) -> Response {
    (
        [(
            CONTENT_TYPE,
            HeaderValue::from_static("text/css; charset=utf-8"),
        )],
        text,
    )
        .into_response()
}

/// What a theme choice posts.
#[derive(Debug, serde::Deserialize)]
struct Choice {
    /// The key of the theme chosen.
    theme: String,
    /// The page to return to.
    back: String,
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

/// Keeps the theme a plain form post chose in its cookie and sends the
/// browser back to the page it came from.
async fn choose_theme(Form(choice): Form<Choice>) -> Response {
    let Some(theme) = Theme::parse(&choice.theme) else {
        return (
            StatusCode::BAD_REQUEST,
            "the theme is not light, dark or system",
        )
            .into_response();
    };
    kept(THEME_COOKIE, theme.key(), &choice.back)
}

/// What a density choice posts.
#[derive(Debug, serde::Deserialize)]
struct DensityChoice {
    /// The key of the density chosen.
    density: String,
    /// The page to return to.
    back: String,
}

/// Keeps the density a plain form post chose in its cookie and sends the
/// browser back to the page it came from.
async fn choose_density(Form(choice): Form<DensityChoice>) -> Response {
    let Some(density) = Density::parse(&choice.density) else {
        return (
            StatusCode::BAD_REQUEST,
            "the density is not comfortable or compact",
        )
            .into_response();
    };
    kept(DENSITY_COOKIE, density.key(), &choice.back)
}

/// What a language choice posts.
#[derive(Debug, serde::Deserialize)]
struct LocaleChoice {
    /// The tag of the locale chosen.
    locale: String,
    /// The page to return to.
    back: String,
}

/// Keeps the locale a plain form post chose in its cookie and sends the
/// browser back to the page it came from.
async fn choose_locale(Form(choice): Form<LocaleChoice>) -> Response {
    let known = crate::locale::choices()
        .iter()
        .any(|locale| locale.tag() == choice.locale);
    if !known {
        return (
            StatusCode::BAD_REQUEST,
            "the locale is not one the gallery offers",
        )
            .into_response();
    }
    kept(LOCALE_COOKIE, &choice.locale, &choice.back)
}

/// The redirect to `back` that keeps `value` in the cookie `name`.
fn kept(name: &str, value: &str, back: &str) -> Response {
    let cookie = format!("{name}={value}; Path=/; SameSite=Lax; HttpOnly; Max-Age=31536000");
    with_cookie(&cookie, back)
}

/// The redirect to `back` that sets `cookie`.
fn with_cookie(cookie: &str, back: &str) -> Response {
    let mut response = Redirect::to(return_path(back)).into_response();
    match HeaderValue::from_str(cookie) {
        Ok(value) => {
            response.headers_mut().insert(SET_COOKIE, value);
            response
        }
        Err(_invalid) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
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
    with_cookie(&cookie, &choice.back)
}

/// Wraps `router` in the browser security headers every response carries.
fn with_security_headers(router: axum::Router<LeptosOptions>) -> axum::Router<LeptosOptions> {
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

/// Mints this response's script nonce, hands it to the renderer through the
/// request, and answers with the policy that names it.
async fn content_security_policy(mut request: Request, next: Next) -> Response {
    let nonce = Nonce::new();
    let policy = HeaderValue::from_str(&policy(&nonce));
    request.extensions_mut().insert(nonce);
    let mut response = next.run(request).await;
    match policy {
        Ok(policy) => {
            response
                .headers_mut()
                .insert(CONTENT_SECURITY_POLICY, policy);
            response
        }
        Err(_invalid) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Re-provides the request's nonce into the render, so the bootstrap script
/// carries the nonce the response header authorizes.
fn provide_request_nonce() {
    let Some(parts) = leptos::context::use_context::<http::request::Parts>() else {
        return;
    };
    if let Some(nonce) = parts.extensions.get::<Nonce>() {
        leptos::context::provide_context(nonce.clone());
    }
}

/// Returns the Content-Security-Policy of one response.
///
/// Scripts run by nonce alone, the WebAssembly compiles under
/// `'wasm-unsafe-eval'`, which admits no JavaScript `eval`, and styles come
/// from the gallery alone (Content Security Policy Level 3).
#[must_use]
pub fn policy(nonce: &Nonce) -> String {
    format!(
        "default-src 'self'; \
         script-src 'self' 'wasm-unsafe-eval' 'nonce-{nonce}'; \
         style-src 'self'; \
         img-src 'self' data:; \
         connect-src 'self'; \
         object-src 'none'; \
         base-uri 'self'; \
         form-action 'self'; \
         frame-ancestors 'none'"
    )
}

/// Marks every page `no-store`, and the site bundle and the stylesheets
/// `no-cache`, so a rebuilt one is fetched again (RFC 9111 §5.2.2).
async fn cache_control(request: Request, next: Next) -> Response {
    let path = request.uri().path();
    let asset =
        path.starts_with("/pkg/") || path == STYLESHEET_PATH || path.starts_with("/themes/");
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
    let site_root =
        std::env::var("GALLERY_SITE_ROOT").unwrap_or_else(|_absent| String::from("target/site"));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, %addr, "the gallery cannot listen");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(%addr, "the gallery serves");
    let router = router(leptos_options(&site_root, addr));
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
    use leptos::nonce::Nonce;

    use super::{policy, return_path};

    #[test]
    fn the_policy_allows_scripts_by_nonce_and_styles_from_the_gallery() {
        let nonce = Nonce::new();
        let policy = policy(&nonce);
        assert!(policy.contains(&format!("'nonce-{nonce}'")), "{policy}");
        assert!(policy.contains("style-src 'self';"), "{policy}");
        assert!(!policy.contains("unsafe-inline"), "{policy}");
        assert!(!policy.contains("'unsafe-eval'"), "{policy}");
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
