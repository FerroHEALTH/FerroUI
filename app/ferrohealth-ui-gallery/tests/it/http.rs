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
        for variant in ["basic", "provider", "both"] {
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
        Request::post("/theme")
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
