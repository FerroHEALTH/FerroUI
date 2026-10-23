// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The clinical record parts: the subject revealed, a version row under
//! the pointer, At a time's picker open, a group's choice focused, the
//! results pane's Export menu open, and the subject line, the history, the
//! builder and the scopes at 390 px.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, away, focus, hover, press, shot_of, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "clinical-subject-revealed",
    "clinical-version-hover",
    "clinical-at-time-open",
    "clinical-group-focus",
    "clinical-export-open",
    "clinical-phone-subject",
    "clinical-phone-versions",
    "clinical-phone-query",
    "clinical-phone-results",
    "clinical-phone-scopes",
];

/// The phone width the parts are captured at.
const PHONE: u32 = 390;

/// The clinical records page of `brand`, its states, then its parts at a
/// phone's width.
pub(super) async fn clinical(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let page = gallery.url(&format!("/{}/clinical", brand.key()));
    browser.goto(&page).await?;
    let reveal = browser
        .element(By::Css("#subject-identifier button[aria-pressed='false']"))
        .await?;
    let hydrated = browser
        .element(By::Css("#subject-identifier .fui-identifier-press"))
        .await?;
    until("the subject identifier has hydrated", async || {
        Ok(hydrated.attr("hidden").await?.is_none())
    })
    .await?;
    press(browser, &reveal).await?;
    let value = browser
        .element(By::Css("#subject-identifier .fui-identifier-value"))
        .await?;
    until("the subject shows", async || {
        Ok(value.text().await?.contains("537"))
    })
    .await?;
    let subject = browser.element(By::Id("subject")).await?;
    shot_of(browser, brand, theme, "clinical-subject-revealed", &subject).await?;
    let row = browser
        .element(By::Css("#versions tbody tr[aria-current='true']"))
        .await?;
    hover(browser, &row).await?;
    let versions = browser.element(By::Id("versions")).await?;
    shot_of(browser, brand, theme, "clinical-version-hover", &versions).await?;
    away(browser).await?;
    let picker = crate::dates::open_picker(browser, "history-at").await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![picker.to_json()?],
        )
        .await?;
    shot_of_view(browser, brand, theme, "clinical-at-time-open").await?;
    browser.goto(&page).await?;
    let all = browser.element(By::Id("g2-match-all")).await?;
    focus(browser, &all).await?;
    // An arrow key moves the focus by the keyboard, so the segment draws its ring.
    browser
        .driver()
        .action_chain()
        .send_keys(Key::Right)
        .perform()
        .await?;
    until("the focus reaches Any", async || {
        Ok(browser
            .driver()
            .active_element()
            .await?
            .id()
            .await?
            .as_deref()
            == Some("g2-match-any"))
    })
    .await?;
    let query = browser.element(By::Id("query")).await?;
    shot_of(browser, brand, theme, "clinical-group-focus", &query).await?;
    // The menu's id is a digest of its exports, so its trigger is found in the pane.
    let export = browser
        .element(By::Css(
            "#results-form summary[aria-controls^='results-export-']",
        ))
        .await?;
    let menu = export
        .attr("aria-controls")
        .await?
        .ok_or("the Export trigger names no menu")?;
    let export = crate::menus::hydrated_trigger(browser, &menu).await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![export.to_json()?],
        )
        .await?;
    press(browser, &export).await?;
    until("the Export menu opens", async || {
        Ok(export.attr("aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    shot_of_view(browser, brand, theme, "clinical-export-open").await?;
    window(browser, PHONE, HEIGHT).await?;
    browser.goto(&page).await?;
    for (id, state) in [
        ("subject", "clinical-phone-subject"),
        ("versions", "clinical-phone-versions"),
        ("query", "clinical-phone-query"),
        ("results", "clinical-phone-results"),
        ("scopes", "clinical-phone-scopes"),
    ] {
        let part = browser.element(By::Id(id)).await?;
        shot_of(browser, brand, theme, state, &part).await?;
    }
    window(browser, WIDTH, HEIGHT).await?;
    Ok(())
}
