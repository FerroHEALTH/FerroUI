// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The field shapes with their error lines, the pick-or-type fields
//! switched to their typed inputs, and the masked identifier at rest, held
//! and with no WebAssembly.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::prelude::*;

use super::{card, focus, shot_of, shot_of_unhydrated};
use crate::browser::{Browser, until};
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "field-shapes",
    "pick-or-type-typed",
    "pick-or-type-combobox-typed",
    "masked-rest",
    "masked-held",
    "masked-no-wasm",
];

/// The field shapes card with its error lines showing, and the two
/// pick-or-type fields switched to their typed inputs.
pub(super) async fn field_states(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    browser
        .element(By::Css("#pick-canonical-choice[role='combobox']"))
        .await?;
    let shapes = browser.element(card("Field shapes")).await?;
    shot_of(browser, brand, theme, "field-shapes", &shapes).await?;
    let picks = browser.element(card("Pick or type")).await?;
    let select = browser.element(By::Id("pick-version-choice")).await?;
    select
        .find(By::Css("option[value='another']"))
        .await?
        .click()
        .await?;
    let typed = browser.element(By::Id("pick-version")).await?;
    until("the typed version shows", async || {
        typed.is_displayed().await.map_err(Into::into)
    })
    .await?;
    focus(browser, &typed).await?;
    shot_of(browser, brand, theme, "pick-or-type-typed", &picks).await?;
    browser
        .element(By::Id("pick-version-back"))
        .await?
        .click()
        .await?;
    let combo = browser
        .element(By::Css("#pick-canonical-choice[role='combobox']"))
        .await?;
    combo.click().await?;
    browser
        .element(By::Css(
            "#pick-canonical-choice-list li[role='option']:last-child",
        ))
        .await?
        .click()
        .await?;
    let canonical = browser.element(By::Id("pick-canonical")).await?;
    until("the typed canonical shows", async || {
        canonical.is_displayed().await.map_err(Into::into)
    })
    .await?;
    shot_of(browser, brand, theme, "pick-or-type-combobox-typed", &picks).await?;
    masked_states(browser, gallery, brand, theme).await
}

/// The field that holds the masked identifier.
fn masked_field() -> By {
    By::XPath("//div[contains(@class, 'fui-field')][.//input[@id='edge-identifier']]")
}

/// The masked identifier with a value typed: at rest, held by the pointer,
/// and with the page's WebAssembly withheld, its eye disabled.
async fn masked_states(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let input = browser.element(By::Id("edge-identifier")).await?;
    input.send_keys("123456782").await?;
    let eye = browser
        .element(By::Css("#edge-identifier + leptos-island > button"))
        .await?;
    until("the eye works", async || {
        eye.is_enabled().await.map_err(Into::into)
    })
    .await?;
    let field = browser.element(masked_field()).await?;
    shot_of(browser, brand, theme, "masked-rest", &field).await?;
    browser
        .driver()
        .action_chain()
        .move_to_element_center(&eye)
        .click_and_hold()
        .perform()
        .await?;
    until("the identifier shows", async || {
        Ok(input.attr("type").await?.as_deref() == Some("text"))
    })
    .await?;
    shot_of(browser, brand, theme, "masked-held", &field).await?;
    browser.driver().action_chain().release().perform().await?;
    let cdp = browser.driver().cdp();
    cdp.send_raw("Network.enable", json!({})).await?;
    cdp.send_raw("Network.setBlockedURLs", json!({ "urls": ["*/pkg/*"] }))
        .await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    // The combobox's native select stands until its island hydrates.
    browser.element(By::Css("select#choose-system")).await?;
    browser
        .element(By::Id("edge-identifier"))
        .await?
        .send_keys("123456782")
        .await?;
    let field = browser.element(masked_field()).await?;
    let shot = shot_of_unhydrated(browser, brand, theme, "masked-no-wasm", &field).await;
    cdp.send_raw("Network.setBlockedURLs", json!({ "urls": [] }))
        .await?;
    shot?;
    // The island loader logs the withheld bundle once the page is left; no
    // other error may come with it.
    browser.goto("about:blank").await?;
    let others: Vec<String> = browser
        .console_errors()
        .await?
        .into_iter()
        .filter(|entry| !entry.contains("/pkg/"))
        .collect();
    if others.is_empty() {
        Ok(())
    } else {
        Err(format!("the page with no WebAssembly logged: {others:?}").into())
    }
}
