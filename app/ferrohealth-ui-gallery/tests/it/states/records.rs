// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The patient-record components: the work-list's focused row on the
//! first, the last and a row the keys scrolled into view, the masked
//! identifiers revealed and refused, the stacked compare on its second
//! record, and a timeline event opened.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{press, shot_of};
use crate::browser::{Browser, until};
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "worklist-focus-first",
    "worklist-focus-scrolled",
    "worklist-focus-last",
    "identifier-revealed",
    "compare-stacked-second",
    "timeline-open",
];

/// Presses `key` on whatever holds the focus.
async fn key(browser: &Browser, key: impl Into<TypingData>) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(key)
        .perform()
        .await?;
    Ok(())
}

/// The row of the work-list that holds the focus.
async fn focused(browser: &Browser) -> Result<WebElement, Box<dyn Error>> {
    Ok(browser.driver().active_element().await?)
}

/// The work-list walked by the keys, the identifiers revealed, the stacked
/// compare's second record and an opened timeline event.
pub(super) async fn records(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    browser
        .element(By::Css(".fui-worklist-grid table[role='grid']"))
        .await?;
    let first = browser
        .element(By::Css(".fui-worklist-grid tbody tr[tabindex='0']"))
        .await?;
    browser
        .driver()
        .execute("arguments[0].focus();", vec![first.to_json()?])
        .await?;
    shot_of(browser, brand, theme, "worklist-focus-first", &first).await?;
    for _ in 0..13 {
        key(browser, "j").await?;
    }
    let scrolled = focused(browser).await?;
    until("the keys reach the fourteenth row", async || {
        Ok(scrolled.attr("data-key").await?.as_deref() == Some("pair-14"))
    })
    .await?;
    let card = browser
        .element(By::Css(".fui-list-card:has(.fui-worklist-grid)"))
        .await?;
    browser.settled().await?;
    let png = browser.driver().screenshot_as_png().await?;
    super::write(brand, "worklist-focus-scrolled", theme, &png)?;
    key(browser, Key::End).await?;
    shot_of(browser, brand, theme, "worklist-focus-last", &card).await?;
    let stacked = browser
        .element(By::Css("label[for='records-compare-stacked-read-1']"))
        .await?;
    press(browser, &stacked).await?;
    let compare = browser.element(By::Css(".fui-compare-alone")).await?;
    shot_of(browser, brand, theme, "compare-stacked-second", &compare).await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let hydrated = browser
        .element(By::Css("#identifier-bsn .fui-identifier-press"))
        .await?;
    until("the identifiers have hydrated", async || {
        Ok(hydrated.attr("hidden").await?.is_none())
    })
    .await?;
    for each in ["bsn", "ssn"] {
        let reveal = browser
            .element(By::Css(format!(
                "#identifier-{each} button[aria-pressed='false']"
            )))
            .await?;
        press(browser, &reveal).await?;
    }
    browser
        .element(By::Css("#identifier-ssn .fui-identifier-refused"))
        .await?;
    let value = browser
        .element(By::Css("#identifier-bsn .fui-identifier-value"))
        .await?;
    until("the value shows", async || {
        Ok(value.text().await?.contains("999"))
    })
    .await?;
    let table = browser.element(By::Id("identifiers")).await?;
    shot_of(browser, brand, theme, "identifier-revealed", &table).await?;
    browser
        .goto(&gallery.url(&format!("/{}/layouts", brand.key())))
        .await?;
    let details = browser.element(By::Css(".fui-timeline details")).await?;
    let summary = details.find(By::Css("summary")).await?;
    press(browser, &summary).await?;
    let list = browser.element(By::Css(".fui-timeline")).await?;
    shot_of(browser, brand, theme, "timeline-open", &list).await
}
