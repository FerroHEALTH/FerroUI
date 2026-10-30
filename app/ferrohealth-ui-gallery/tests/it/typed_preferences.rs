// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The typed preferences of FerroTERM's Language and region (#527): a text
//! and a number field, each in the plain form of one preference, posted
//! through its Save button with the modules refused and on change once the
//! `preference` module runs, the field just saved focused again, and the
//! rows reflowed at 320 px.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::no_script::{leave, withhold_the_modules};
use crate::stack::Gallery;

/// The tab that holds the typed preferences.
const LANGUAGE: &str = "/ferroterm/preferences/language";

/// The id of the element that holds the focus.
async fn focused(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .driver()
        .active_element()
        .await?
        .attr("id")
        .await?
        .unwrap_or_default())
}

/// The value the field `id` shows.
async fn value(browser: &Browser, id: &str) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .element(By::Id(id))
        .await?
        .prop("value")
        .await?
        .unwrap_or_default())
}

/// Empties the field `id` and types `words` into it. WebDriver's Element
/// Clear fires `change`, which would post the form, so the field is emptied
/// in the page.
async fn retype(browser: &Browser, id: &str, words: &str) -> Result<WebElement, Box<dyn Error>> {
    let field = browser.element(By::Id(id)).await?;
    browser
        .driver()
        .execute("arguments[0].value = '';", vec![field.to_json()?])
        .await?;
    field.send_keys(words).await?;
    Ok(field)
}

/// The Save button of the form that holds the field `id`.
async fn save_of(browser: &Browser, id: &str) -> Result<WebElement, Box<dyn Error>> {
    browser
        .element(By::XPath(format!(
            "//form[contains(@class, 'fui-preference-form')][.//input[@id = '{id}']]//button[@type = 'submit']"
        )))
        .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_typed_preference_posts_on_change_and_keeps_its_field_focused()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url(LANGUAGE)).await?;
        let form = browser
            .element(By::Css(
                "form.fui-preference-form[data-fui-ready]:has(#size)",
            ))
            .await?;
        let save = save_of(browser, "size").await?;
        assert!(!save.is_displayed().await?, "the module hides Save");
        assert!(form.is_displayed().await?);
        // Enter changes the field and submits its form: one post, its page.
        let size = retype(browser, "size", "50").await?;
        let old = browser.loader().await?;
        size.send_keys(Key::Enter).await?;
        until("the post keeps the size", async || {
            Ok(browser.loader().await? != old
                && browser.driver().current_url().await?.as_str()
                    == gallery.url(&format!("{LANGUAGE}?saved=size")))
        })
        .await?;
        browser
            .element(By::Css(".fui-setting-saved[role='status']"))
            .await?;
        assert_eq!("50", value(browser, "size").await?);
        until("the field just saved takes the focus back", async || {
            Ok(focused(browser).await? == "size")
        })
        .await?;
        // A text field posts when the focus leaves it changed.
        browser
            .element(By::Css(
                "form.fui-preference-form[data-fui-ready]:has(#concepts)",
            ))
            .await?;
        retype(browser, "concepts", "nl-NL").await?;
        let old = browser.loader().await?;
        browser.element(By::Id("size")).await?.focus().await?;
        until("the post keeps the concept language", async || {
            Ok(browser.loader().await? != old
                && browser.driver().current_url().await?.as_str()
                    == gallery.url(&format!("{LANGUAGE}?saved=concepts")))
        })
        .await?;
        assert_eq!("nl-NL", value(browser, "concepts").await?);
        until("the text field takes the focus back", async || {
            Ok(focused(browser).await? == "concepts")
        })
        .await?;
        // The browser holds a number to its bounds: 500 posts nothing.
        let size = retype(browser, "size", "500").await?;
        let old = browser.loader().await?;
        size.send_keys(Key::Enter).await?;
        let invalid: bool = browser
            .driver()
            .execute(
                "return !document.getElementById('size').validity.valid;",
                vec![],
            )
            .await?
            .convert()?;
        assert!(invalid, "500 is over the field's max");
        assert_eq!(
            old,
            browser.loader().await?,
            "an invalid size stays on the page"
        );
        browser.console_clean("the typed preferences").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_typed_preference_posts_through_its_save_button_with_the_modules_refused()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_modules(browser).await?;
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url(LANGUAGE)).await?;
        let save = save_of(browser, "size").await?;
        assert!(
            save.is_displayed().await?,
            "with no script Save follows the field"
        );
        retype(browser, "size", "75").await?;
        browser.click_and_load(&save).await?;
        browser
            .at(&gallery.url(&format!("{LANGUAGE}?saved=size")))
            .await?;
        assert_eq!("75", value(browser, "size").await?);
        let save = save_of(browser, "concepts").await?;
        retype(browser, "concepts", "de-CH").await?;
        browser.click_and_load(&save).await?;
        browser
            .at(&gallery.url(&format!("{LANGUAGE}?saved=concepts")))
            .await?;
        assert_eq!("de-CH", value(browser, "concepts").await?);
        leave(browser, "the typed preferences with no script").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_typed_preferences_reflow_at_320_pixels() -> Result<(), Box<dyn Error>> {
    // WCAG 2.2 SC 1.4.10: each field takes the row's width, never the page's.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 320, 800).await?;
        browser.goto(&gallery.url(LANGUAGE)).await?;
        browser.element(By::Id("concepts")).await?;
        let (page, view, fields): (f64, f64, bool) = browser
            .driver()
            .execute(
                r"const root = document.documentElement;
                const inside = [...document.querySelectorAll('.fui-setting-input')].every((field) => {
                  const row = field.closest('.fui-setting-row').getBoundingClientRect();
                  const box = field.getBoundingClientRect();
                  return box.left >= row.left - 0.5 && box.right <= row.right + 0.5;
                });
                return [root.scrollWidth, root.clientWidth, inside];",
                vec![],
            )
            .await?
            .convert()?;
        assert!(page <= view + 0.5, "the page scrolls sideways: {page} in {view}");
        assert!(fields, "a typed field stays inside its row");
        browser.console_clean("the typed preferences at 320 px").await
    })
    .await
}
