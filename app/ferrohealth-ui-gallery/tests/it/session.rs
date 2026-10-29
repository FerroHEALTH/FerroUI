// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the session's end (#444): a page whose session ends
//! leaves for sign-in with its return address and no user action, the idle
//! warning keeps the session with Stay signed in and gives the focus back,
//! a tab shown again asks whether its session still holds, sign-out in one
//! tab ends every tab, so does a search that meets the end of the session,
//! and a read of the session never extends it.

use std::error::Error;

use ferrohealth_ui::view::session::WARNING_ID;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// Starts a session of `idle` and `absolute` seconds and waits for the
/// brand's overview to be enhanced.
async fn start(
    browser: &Browser,
    gallery: &Gallery,
    idle: u32,
    absolute: u32,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!(
            "/session/start/ferrobridge?idle={idle}&absolute={absolute}"
        )))
        .await?;
    browser.at(&gallery.url("/ferrobridge")).await?;
    browser.hydrated().await
}

/// Whether the session warning shows.
async fn warning_open(browser: &Browser) -> Result<bool, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute(
            "return document.getElementById(arguments[0]).open;",
            vec![serde_json::Value::from(WARNING_ID)],
        )
        .await?
        .convert()?)
}

/// The session cookie's value, which only the server writes.
async fn session_cookie(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .driver()
        .get_named_cookie(ferrohealth_ui_gallery::session::SESSION_COOKIE)
        .await?
        .value)
}

/// Asserts the browser logged nothing but the session route's `401`.
async fn only_the_refusal(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let errors = browser.console_errors().await?;
    assert!(
        errors.iter().all(|error| error.contains("401")),
        "only the session route's refusal is logged: {errors:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_page_whose_session_ends_leaves_for_sign_in_with_its_return_address()
-> Result<(), Box<dyn Error>> {
    // OWASP Session Management Cheat Sheet, idle timeout: the page leaves on its own.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        start(browser, &gallery, 4, 600).await?;
        browser
            .at(&gallery.url("/sign-in/ferrobridge/expired?return=/ferrobridge"))
            .await?;
        only_the_refusal(browser).await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_idle_warning_keeps_the_session_and_gives_the_focus_back() -> Result<(), Box<dyn Error>>
{
    // WCAG 2.2 SC 2.2.1: warned before the time limit, with at least 20 seconds to extend it.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        start(browser, &gallery, 22, 600).await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        search.focus().await?;
        until("the warning opens", async || warning_open(browser).await).await?;
        let dialog = browser.element(By::Id(WARNING_ID)).await?;
        assert_eq!(Some("alertdialog".to_owned()), dialog.attr("role").await?);
        let stay = browser
            .element(By::Css(format!("#{WARNING_ID} button[autofocus]")))
            .await?;
        let focused = browser.driver().active_element().await?;
        assert_eq!(
            stay.element_id(),
            focused.element_id(),
            "Stay signed in takes the focus (APG alert dialog)"
        );
        let before = session_cookie(browser).await?;
        stay.click().await?;
        until("the warning closes", async || {
            Ok(!warning_open(browser).await?)
        })
        .await?;
        assert_ne!(
            before,
            session_cookie(browser).await?,
            "Stay signed in extends the session"
        );
        let focused = browser.driver().active_element().await?;
        assert_eq!(
            search.element_id(),
            focused.element_id(),
            "the focus returns where it was (HTML, the dialog element)"
        );
        assert_eq!(
            gallery.url("/ferrobridge"),
            browser.driver().current_url().await?.as_str(),
            "the page stays"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_tab_shown_again_asks_whether_its_session_holds_and_a_read_never_extends_it()
-> Result<(), Box<dyn Error>> {
    // OWASP Session Management Cheat Sheet: only the user's activity renews the idle timeout.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        start(browser, &gallery, 600, 3600).await?;
        let kept = session_cookie(browser).await?;
        for kind in ["visibilitychange", "focus"] {
            browser
                .driver()
                .execute(
                    "window.dispatchEvent(new Event(arguments[0]));",
                    vec![serde_json::Value::from(kind)],
                )
                .await?;
        }
        until("both re-checks are answered", async || {
            let asked: u64 = browser
                .driver()
                .execute(
                    "return performance.getEntriesByType('resource').filter((entry) => new URL(entry.name).pathname === arguments[0]).length;",
                    vec![serde_json::Value::from(ferrohealth_ui_gallery::session::SESSION_ROUTE)],
                )
                .await?
                .convert()?;
            Ok(asked >= 2)
        })
        .await?;
        assert_eq!(
            kept,
            session_cookie(browser).await?,
            "a re-check reads the session and never extends it"
        );
        browser
            .driver()
            .delete_cookie(ferrohealth_ui_gallery::session::SESSION_COOKIE)
            .await?;
        browser
            .driver()
            .execute("document.dispatchEvent(new Event('visibilitychange', { bubbles: true }));", vec![])
            .await?;
        browser
            .at(&gallery.url("/sign-in/ferrobridge/expired?return=/ferrobridge"))
            .await?;
        only_the_refusal(browser).await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn sign_out_in_one_tab_ends_every_tab() -> Result<(), Box<dyn Error>> {
    // OWASP Session Management Cheat Sheet: a logout ends the session in every tab.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        start(browser, &gallery, 600, 3600).await?;
        let first = browser.driver().window().await?;
        let second = browser.driver().new_tab().await?;
        browser.driver().switch_to_window(second).await?;
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        browser.hydrated().await?;
        let sign_out = browser
            .element(By::Css(format!(
                "form[action='{}/ferrobridge'] button[type='submit']",
                ferrohealth_ui_gallery::app::SIGN_OUT_ROUTE
            )))
            .await?;
        browser
            .driver()
            .execute(
                "arguments[0].form.requestSubmit(arguments[0]);",
                vec![sign_out.to_json()?],
            )
            .await?;
        browser
            .at(&gallery.url("/sign-in/ferrobridge/signed-out"))
            .await?;
        browser.driver().switch_to_window(first).await?;
        browser
            .at(&gallery.url("/sign-in/ferrobridge/expired?return=/ferrobridge"))
            .await?;
        only_the_refusal(browser).await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_search_that_meets_the_end_of_the_session_ends_every_tab() -> Result<(), Box<dyn Error>> {
    // FerroUI #474: the search's 401 loads the signed-out page in its own tab and tells every
    // other tab, which leaves for sign-in although its own session route still answers.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        start(browser, &gallery, 600, 3600).await?;
        let first = browser.driver().window().await?;
        let second = browser.driver().new_tab().await?;
        browser.driver().switch_to_window(second.clone()).await?;
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        browser.hydrated().await?;
        browser.driver().switch_to_window(first.clone()).await?;
        let search = browser
            .element(By::Css("#gallery-search[role='combobox']"))
            .await?;
        search.click().await?;
        search
            .send_keys(ferrohealth_ui_gallery::search::SIGNED_OUT)
            .await?;
        let signed_out = gallery.url("/sign-in/ferrobridge/expired");
        browser.at(&signed_out).await?;
        browser.driver().switch_to_window(second).await?;
        browser
            .at(&gallery.url("/sign-in/ferrobridge/expired?return=/ferrobridge/tables"))
            .await?;
        only_the_refusal(browser).await?;
        browser.driver().switch_to_window(first).await?;
        assert_eq!(
            browser.driver().current_url().await?.as_str(),
            signed_out,
            "the searching tab stays on the signed-out page"
        );
        only_the_refusal(browser).await
    })
    .await
}
