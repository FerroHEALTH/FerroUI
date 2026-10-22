// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's HTTP surface in-process: the stylesheet and the themes,
//! the theme choice, the security headers, every page in every brand, and
//! the `404`.

use std::error::Error;

use axum::body::Body;
use ferrohealth_ui::style::{Brand, STYLESHEET};
use http::{Request, StatusCode, header};
use tower::ServiceExt as _;

/// The gallery's service, its site bundle under a root no test reads.
fn gallery() -> axum::Router {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    ferrohealth_ui_gallery::server::router(ferrohealth_ui_gallery::server::leptos_options(
        "target/site",
        addr,
    ))
}

/// What the gallery answers `request` with: its status, headers and body.
async fn send(request: Request<Body>) -> Result<(http::response::Parts, String), Box<dyn Error>> {
    let response = gallery().oneshot(request).await?;
    let (parts, body) = response.into_parts();
    let bytes = axum::body::to_bytes(body, 8 * 1024 * 1024).await?;
    Ok((parts, String::from_utf8(bytes.to_vec())?))
}

/// What the gallery answers a `GET` of `path` with.
async fn get(path: &str) -> Result<(http::response::Parts, String), Box<dyn Error>> {
    send(Request::get(path).body(Body::empty())?).await
}

#[tokio::test]
async fn the_stylesheet_and_every_theme_are_served_as_css() -> Result<(), Box<dyn Error>> {
    let (parts, body) = get("/ferrohealth-ui.css").await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert_eq!(STYLESHEET, body);
    assert!(
        parts.headers[header::CONTENT_TYPE]
            .to_str()?
            .starts_with("text/css")
    );
    for brand in Brand::ALL {
        let (parts, body) = get(&format!("/themes/{}.css", brand.key())).await?;
        assert_eq!(StatusCode::OK, parts.status, "{brand:?}");
        assert_eq!(brand.theme(), body, "{brand:?}");
    }
    let (parts, _) = get("/themes/../ferrohealth-ui.css").await?;
    assert_ne!(StatusCode::OK, parts.status);
    let (parts, _) = get("/themes/unknown.css").await?;
    assert_eq!(StatusCode::NOT_FOUND, parts.status);
    Ok(())
}

#[tokio::test]
async fn every_page_of_every_brand_renders_with_its_theme_and_the_headers()
-> Result<(), Box<dyn Error>> {
    for brand in Brand::ALL {
        for page in ["", "/forms", "/tables", "/code"] {
            let path = format!("/{}{page}", brand.key());
            let (parts, body) = get(&path).await?;
            assert_eq!(StatusCode::OK, parts.status, "{path}");
            assert!(
                body.contains(&format!("/themes/{}.css", brand.key())),
                "{path}: {body}"
            );
            assert!(body.contains(brand.product()), "{path}");
            let policy = parts.headers[header::CONTENT_SECURITY_POLICY].to_str()?;
            assert!(policy.contains("style-src 'self';"), "{policy}");
            assert_eq!("no-store", parts.headers[header::CACHE_CONTROL]);
            assert_eq!("nosniff", parts.headers[header::X_CONTENT_TYPE_OPTIONS]);
            assert!(
                !body.contains(" style=\""),
                "{path} carries a style attribute"
            );
        }
        for variant in crate::journeys::SIGN_INS {
            let path = format!("/sign-in/{}/{variant}", brand.key());
            let (parts, body) = get(&path).await?;
            assert_eq!(StatusCode::OK, parts.status, "{path}");
            assert!(body.contains("fui-sign-in"), "{path}");
        }
    }
    Ok(())
}

#[tokio::test]
async fn a_path_the_gallery_does_not_serve_answers_404() -> Result<(), Box<dyn Error>> {
    let (parts, body) = get("/no/such/page").await?;
    assert_eq!(StatusCode::NOT_FOUND, parts.status);
    assert!(body.contains("404"), "{body}");
    Ok(())
}

#[tokio::test]
async fn a_theme_choice_is_kept_in_a_cookie_and_returns_to_the_gallery_alone()
-> Result<(), Box<dyn Error>> {
    let post = |body: &'static str| {
        Request::post("/preference")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
    };
    let (parts, _) = send(post("theme=dark&back=%2Fferrofed%2Ftables")?).await?;
    assert!(parts.status.is_redirection(), "{}", parts.status);
    assert_eq!("/ferrofed/tables", parts.headers[header::LOCATION]);
    let cookie = parts.headers[header::SET_COOKIE].to_str()?;
    assert!(cookie.starts_with("gallery-theme=dark;"), "{cookie}");
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    let (parts, _) = send(post("theme=dark&back=%2F%2Fevil.example")?).await?;
    assert_eq!("/", parts.headers[header::LOCATION]);
    let (parts, _) = send(post("theme=sepia&back=%2F")?).await?;
    assert_eq!(StatusCode::BAD_REQUEST, parts.status);
    let (_, body) = send(
        Request::get("/ferroehr")
            .header(header::COOKIE, "gallery-theme=dark")
            .body(Body::empty())?,
    )
    .await?;
    assert!(
        body.contains(r#"data-theme="dark""#),
        "the page arrives in its theme"
    );
    Ok(())
}

/// A form post of `body` to `path`.
fn form(path: &str, body: &'static str) -> Result<Request<Body>, Box<dyn Error>> {
    Ok(Request::post(path)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))?)
}

#[tokio::test]
async fn the_sidebar_choice_is_kept_in_its_cookie_and_sign_out_leads_to_the_signed_out_card()
-> Result<(), Box<dyn Error>> {
    let (parts, _) = send(form("/sidebar", "sidebar=rail&back=%2Fferrofed%2Ftables")?).await?;
    assert_eq!(StatusCode::SEE_OTHER, parts.status);
    assert_eq!("/ferrofed/tables", parts.headers[header::LOCATION]);
    let cookie = parts.headers[header::SET_COOKIE].to_str()?;
    assert!(cookie.starts_with("gallery-sidebar=rail;"), "{cookie}");
    let (parts, _) = send(form("/sidebar", "sidebar=wide&back=%2F")?).await?;
    assert_eq!(StatusCode::BAD_REQUEST, parts.status);
    let (parts, _) = send(form("/sign-out/ferroehr", "")?).await?;
    assert_eq!(StatusCode::SEE_OTHER, parts.status);
    assert_eq!(
        "/sign-in/ferroehr/signed-out",
        parts.headers[header::LOCATION]
    );
    let (parts, body) = send(
        Request::get("/ferrofed/tables")
            .header(header::COOKIE, "gallery-sidebar=rail")
            .body(Body::empty())?,
    )
    .await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert!(
        body.contains("fui-sidebar fui-sidebar-rail"),
        "the server draws the rail the cookie asks for"
    );
    Ok(())
}

#[tokio::test]
async fn a_preference_kept_on_your_preferences_returns_saying_which() -> Result<(), Box<dyn Error>>
{
    let (parts, _) = send(form(
        "/preference",
        "back=%2Fferropix%2Fpreferences&density=compact",
    )?)
    .await?;
    assert_eq!(StatusCode::SEE_OTHER, parts.status);
    assert_eq!(
        "/ferropix/preferences?saved=density",
        parts.headers[header::LOCATION]
    );
    let cookie = parts.headers[header::SET_COOKIE].to_str()?;
    assert!(cookie.starts_with("gallery-density=compact;"), "{cookie}");
    assert!(cookie.contains("SameSite=Lax"), "{cookie}");
    assert!(cookie.contains("Max-Age=31536000"), "one year: {cookie}");
    let (parts, _) = send(form(
        "/preference",
        "back=%2Fferropix%2Fpreferences&hide=off&hide=on",
    )?)
    .await?;
    let cookie = parts.headers[header::SET_COOKIE].to_str()?;
    assert!(
        cookie.starts_with("gallery-hide=on;"),
        "a checked switch's on follows its off: {cookie}"
    );
    let (parts, _) = send(form("/preference", "back=%2F&theme=dark&density=compact")?).await?;
    assert_eq!(
        StatusCode::BAD_REQUEST,
        parts.status,
        "one preference a post"
    );
    let (parts, _) = send(form("/preference", "back=%2F")?).await?;
    assert_eq!(StatusCode::BAD_REQUEST, parts.status, "no preference");
    let (_, body) = send(
        Request::get("/ferropix/preferences?saved=density")
            .header(
                header::COOKIE,
                "gallery-density=compact; gallery-motion=reduce; gallery-hints=never",
            )
            .body(Body::empty())?,
    )
    .await?;
    assert!(body.contains(r#"data-density="compact""#), "{body}");
    assert!(body.contains(r#"data-motion="reduce""#), "{body}");
    assert!(body.contains(r#"data-hints="never""#), "{body}");
    assert_eq!(1, body.matches("fui-setting-saved").count(), "{body}");
    Ok(())
}

#[tokio::test]
async fn reset_to_defaults_clears_every_preference_cookie() -> Result<(), Box<dyn Error>> {
    let (parts, _) = send(form(
        "/preferences/reset",
        "back=%2Fferroehr%2Fpreferences%2Fkeyboard",
    )?)
    .await?;
    assert_eq!(StatusCode::SEE_OTHER, parts.status);
    assert_eq!(
        "/ferroehr/preferences/keyboard",
        parts.headers[header::LOCATION]
    );
    let cleared = parts
        .headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|cookie| cookie.to_str().map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    for name in [
        "theme",
        "density",
        "motion",
        "language",
        "week-start",
        "hints",
        "reveal",
        "hide",
    ] {
        assert!(
            cleared
                .iter()
                .any(|cookie| cookie.starts_with(&format!("gallery-{name}=;"))
                    && cookie.contains("Max-Age=0")),
            "{name}: {cleared:?}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn a_tab_your_preferences_lacks_answers_404() -> Result<(), Box<dyn Error>> {
    let (parts, _) = get("/ferrofed/preferences/account").await?;
    assert_eq!(
        StatusCode::NOT_FOUND,
        parts.status,
        "there is no fourth tab"
    );
    Ok(())
}
