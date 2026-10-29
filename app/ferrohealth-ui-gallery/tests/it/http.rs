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

/// A row's danger confirmation is the `<ActionForm>` of its server
/// function: with no WebAssembly it posts there as a plain form, and the
/// function sends the browser back naming the member removed (#500).
#[tokio::test]
async fn a_removal_dialog_is_its_actions_form_and_a_plain_post_of_it_removes_the_row()
-> Result<(), Box<dyn Error>> {
    let (parts, body) = get("/ferrofed/overlays").await?;
    assert_eq!(StatusCode::OK, parts.status);
    let dialog = body
        .split(r#"<dialog id="remove-node-b""#)
        .nth(1)
        .and_then(|rest| rest.split("</dialog>").next())
        .unwrap_or_default();
    assert!(
        dialog.contains(r#"action="/api/remove_member""#)
            && dialog.contains(r#"method="post""#)
            && dialog.contains(r#"class="fui-dialog-form""#),
        "the dialog's form is the action's form: {dialog}"
    );
    assert!(
        dialog.contains(r#"name="member" value="node-b""#)
            && dialog.contains(r#"name="reason""#)
            && dialog.contains(r#"type="submit""#),
        "the fields and the confirm sit inside it: {dialog}"
    );
    let request = Request::post("/api/remove_member")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::ACCEPT, "text/html,application/xhtml+xml,*/*;q=0.8")
        .body(Body::from(
            "member=node-b&back=%2Fferrofed%2Foverlays&reason=Replaced+by+node-b2",
        ))?;
    let (parts, _) = send(request).await?;
    assert!(parts.status.is_redirection(), "{:?}", parts.status);
    assert_eq!(
        "/ferrofed/overlays?removed=node-b",
        parts.headers[header::LOCATION]
    );
    let request = Request::post("/api/remove_member")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::ACCEPT, "*/*")
        .body(Body::from(
            "member=node-b&back=%2Fferrofed%2Foverlays&reason=+",
        ))?;
    let (parts, _) = send(request).await?;
    assert!(
        !parts.status.is_success() && !parts.status.is_redirection(),
        "a blank reason is refused: {:?}",
        parts.status
    );
    Ok(())
}

#[tokio::test]
async fn a_removal_returns_naming_the_member_and_refuses_one_with_no_reason()
-> Result<(), Box<dyn Error>> {
    let (parts, _) = send(form(
        "/remove-member",
        "member=node-b&back=%2Fferrofed%2Foverlays&reason=Replaced+by+node-b2",
    )?)
    .await?;
    assert_eq!(StatusCode::SEE_OTHER, parts.status);
    assert_eq!(
        "/ferrofed/overlays?removed=node-b",
        parts.headers[header::LOCATION]
    );
    let (parts, _) = send(form(
        "/remove-member",
        "member=node-b&back=%2Fferrofed%2Foverlays&reason=+",
    )?)
    .await?;
    assert_eq!(StatusCode::BAD_REQUEST, parts.status, "a blank reason");
    let (parts, _) = send(form(
        "/remove-member",
        "member=node-z&back=%2F&reason=Gone",
    )?)
    .await?;
    assert_eq!(StatusCode::BAD_REQUEST, parts.status, "a member not listed");
    let (parts, body) = get("/ferrofed/overlays?removed=node-b").await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert!(
        !body.contains(r#"id="remove-node-b""#) && body.contains(r#"id="remove-node-c""#),
        "the list leaves the removed row out"
    );
    assert!(body.contains("node-b was removed"), "and says so");
    Ok(())
}

#[tokio::test]
async fn an_export_riding_the_query_form_receives_the_query() -> Result<(), Box<dyn Error>> {
    let (parts, body) = get("/export/csv?query=SELECT+1&shown=table").await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert_eq!("csv\nSELECT 1", body);
    let (parts, _) = get("/export/xml?query=SELECT+1").await?;
    assert_eq!(StatusCode::NOT_FOUND, parts.status);
    Ok(())
}

/// What the gallery answers a plain form post of `body` to the
/// `acknowledge` server function with, as a browser with no WebAssembly
/// and no `Referer` sends it.
async fn acknowledge(body: &str) -> Result<http::response::Parts, Box<dyn Error>> {
    let request = Request::post("/api/acknowledge")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::ACCEPT, "text/html,application/xhtml+xml,*/*;q=0.8")
        .body(Body::from(body.to_owned()))?;
    Ok(send(request).await?.0)
}

#[tokio::test]
async fn a_plain_post_with_no_referer_returns_to_the_page_its_form_names()
-> Result<(), Box<dyn Error>> {
    let parts = acknowledge("note=seen&ferroui_return=%2Fferropix%2Fforms%3Ftab%3Dtwo").await?;
    assert_eq!(StatusCode::FOUND, parts.status);
    assert_eq!("/ferropix/forms?tab=two", parts.headers[header::LOCATION]);
    Ok(())
}

#[tokio::test]
async fn a_failed_plain_post_returns_with_the_error_the_router_reads() -> Result<(), Box<dyn Error>>
{
    let parts = acknowledge("note=&ferroui_return=%2Fferropix%2Fforms").await?;
    assert_eq!(StatusCode::FOUND, parts.status);
    let location = parts.headers[header::LOCATION].to_str()?;
    assert!(
        location.starts_with("/ferropix/forms?__path=%2Fapi%2Facknowledge&__err="),
        "{location}"
    );
    Ok(())
}

/// What the gallery answers a plain form post of `body` to the
/// `count_words` server function with, as a browser with no WebAssembly
/// sends it: its status, headers and body.
async fn count_words(body: &str) -> Result<(http::response::Parts, String), Box<dyn Error>> {
    let request = Request::post("/api/count_words")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::ACCEPT, "text/html,application/xhtml+xml,*/*;q=0.8")
        .body(Body::from(body.to_owned()))?;
    send(request).await
}

#[tokio::test]
async fn a_plain_post_of_an_answer_form_draws_its_page_with_the_answer_and_the_note()
-> Result<(), Box<dyn Error>> {
    let (parts, body) = count_words("note=two+words&ferroui_answer=%2Fferrobridge%2Fforms").await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert_eq!("no-store", parts.headers[header::CACHE_CONTROL]);
    assert!(parts.headers.contains_key(header::CONTENT_SECURITY_POLICY));
    assert!(
        body.contains("<p id=\"note-answer\" class=\"fui-help\">The note has 2 words.</p>"),
        "the answer is drawn on the page: {body}"
    );
    let note = body
        .split("id=\"answer-note\"")
        .nth(1)
        .and_then(|rest| rest.split('>').next())
        .unwrap_or_default();
    assert!(
        note.contains("name=\"note\"") && note.contains("value=\"two words\""),
        "the posted note is kept in its field: {note}"
    );
    assert!(
        body.contains("name=\"ferroui_answer\" value=\"/ferrobridge/forms\""),
        "the page drawn names itself, not the function"
    );
    Ok(())
}

#[tokio::test]
async fn a_refused_plain_post_of_an_answer_form_draws_the_refusal_and_keeps_the_note()
-> Result<(), Box<dyn Error>> {
    let (parts, body) = count_words("note=+&ferroui_answer=%2Fferrobridge%2Fforms").await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert_eq!("no-store", parts.headers[header::CACHE_CONTROL]);
    assert!(
        body.contains("<p id=\"note-answer\" class=\"fui-help fui-error\">The note is empty.</p>"),
        "the refusal is drawn on the page: {body}"
    );
    assert!(body.contains("value=\" \""), "the posted note is kept");
    Ok(())
}

/// A field the form marks secret goes to the function alone: the page drawn
/// with the answer carries it neither in its markup nor in its hydration
/// data, where every other posted field crosses to the browser (#499).
#[tokio::test]
async fn a_secret_field_of_an_answer_form_appears_nowhere_on_the_page_it_draws()
-> Result<(), Box<dyn Error>> {
    let (parts, body) = count_words(
        "note=two+words&pin=synthetic-pin-4711&ferroui_secret=pin\
         &shown=synthetic-shown-4711&ferroui_answer=%2Fferrobridge%2Fforms",
    )
    .await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert!(
        body.contains("<p id=\"note-answer\" class=\"fui-help\">The note has 2 words.</p>"),
        "the function answered: {body}"
    );
    assert!(
        body.contains("synthetic-shown-4711"),
        "an unmarked field crosses to the browser with the page"
    );
    assert!(
        !body.contains("synthetic-pin-4711"),
        "the secret is nowhere in the page or its hydration data: {body}"
    );
    Ok(())
}

/// A form marks each of its secret fields under a name of its own, so two
/// marks post no name twice, and neither value reaches the page (#503).
#[tokio::test]
async fn two_secret_fields_of_an_answer_form_appear_nowhere_on_the_page_it_draws()
-> Result<(), Box<dyn Error>> {
    let (parts, body) = count_words(
        "note=two+words&pin=synthetic-pin-4712&ferroui_secret_70696e=pin\
         &key=synthetic-key-4712&ferroui_secret_6b6579=key&ferroui_answer=%2Fferrobridge%2Fforms",
    )
    .await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert!(
        body.contains("<p id=\"note-answer\" class=\"fui-help\">The note has 2 words.</p>"),
        "the function answered: {body}"
    );
    assert!(
        !body.contains("synthetic-pin-4712") && !body.contains("synthetic-key-4712"),
        "neither secret is in the page or its hydration data: {body}"
    );
    Ok(())
}

#[tokio::test]
async fn the_forms_page_drawn_on_its_own_carries_no_answer() -> Result<(), Box<dyn Error>> {
    let (parts, body) = get("/ferrobridge/forms").await?;
    assert_eq!(StatusCode::OK, parts.status);
    assert!(!body.contains("id=\"note-answer\""));
    assert!(body.contains("name=\"ferroui_answer\" value=\"/ferrobridge/forms\""));
    Ok(())
}

#[tokio::test]
async fn a_return_field_naming_another_origin_is_never_followed() -> Result<(), Box<dyn Error>> {
    for field in ["%2F%2Fevil.example%2F", "https%3A%2F%2Fevil.example%2F"] {
        let parts = acknowledge(&format!("note=seen&ferroui_return={field}")).await?;
        let location = parts
            .headers
            .get(header::LOCATION)
            .map(|value| value.to_str().map(str::to_owned))
            .transpose()?;
        assert!(
            location
                .as_deref()
                .is_none_or(|location| !location.contains("evil")),
            "{field}: {location:?}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn the_session_route_reads_without_extending_extends_on_a_post_and_refuses_a_gone_session()
-> Result<(), Box<dyn Error>> {
    // OWASP Session Management Cheat Sheet: only the user's activity renews the idle timeout.
    use ferrohealth_ui_gallery::session::{Kept, SESSION_COOKIE, SESSION_ROUTE, now};
    let kept = Kept::started(now().saturating_sub(10_000), 900, 3600);
    let cookie = format!("{SESSION_COOKIE}={}", kept.value());
    let ask = |method: &str, cookie: &str| {
        Request::builder()
            .method(method)
            .uri(SESSION_ROUTE)
            .header(header::COOKIE, cookie)
            .body(Body::empty())
    };
    let (parts, body) = send(ask("GET", &cookie)?).await?;
    assert_eq!(StatusCode::OK, parts.status, "{body}");
    assert!(
        parts.headers.get(header::SET_COOKIE).is_none(),
        "a read never extends"
    );
    let read: ferrohealth_ui::widget::session::Left = serde_json::from_str(&body)?;
    assert!((889..=890).contains(&read.idle), "{read:?}");
    let (parts, body) = send(ask("POST", &cookie)?).await?;
    assert_eq!(StatusCode::OK, parts.status, "{body}");
    let extended: ferrohealth_ui::widget::session::Left = serde_json::from_str(&body)?;
    assert!(extended.idle >= 899, "{extended:?}");
    assert!(
        parts.headers[header::SET_COOKIE]
            .to_str()?
            .starts_with(&format!("{SESSION_COOKIE}=")),
    );
    let (parts, _) = send(ask("GET", "gallery-theme=dark")?).await?;
    assert_eq!(StatusCode::UNAUTHORIZED, parts.status);
    Ok(())
}

/// The panics any thread of this test process has raised since
/// [`count_panics`] was called.
static PANICS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Counts every panic of the process in [`PANICS`], keeping the hook that
/// prints it.
fn count_panics() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        PANICS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        previous(info);
    }));
}

/// Reads the next frame of `body`: whether one came before its end.
async fn frame(body: &mut Body) -> Result<bool, axum::Error> {
    use axum::body::HttpBody as _;
    std::future::poll_fn(|cx| std::pin::Pin::new(&mut *body).poll_frame(cx))
        .await
        .transpose()
        .map(|frame| frame.is_some())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_forms_page_left_while_it_streams_raises_no_server_panic() -> Result<(), Box<dyn Error>> {
    count_panics();
    let service = gallery();
    let forms = || Request::get("/ferropix/forms").body(Body::empty());
    let mut frames = 0_usize;
    let mut body = service.clone().oneshot(forms()?).await?.into_body();
    while frame(&mut body).await? {
        frames += 1;
    }
    // Each response is dropped after one more frame than the last, as a
    // browser drops a page it leaves while the page still streams (#457).
    for _ in 0..5 {
        for kept in 0..frames {
            let response = service.clone().oneshot(forms()?).await?;
            assert_eq!(StatusCode::OK, response.status());
            let mut body = response.into_body();
            for _ in 0..kept {
                frame(&mut body).await?;
            }
            drop(body);
            tokio::task::yield_now().await;
        }
    }
    let rendered = service.oneshot(forms()?).await?;
    let bytes = axum::body::to_bytes(rendered.into_body(), 8 * 1024 * 1024).await?;
    assert!(!bytes.is_empty(), "the forms page drew nothing");
    assert_eq!(
        0,
        PANICS.load(std::sync::atomic::Ordering::SeqCst),
        "the server panicked while it drew the forms page"
    );
    Ok(())
}
