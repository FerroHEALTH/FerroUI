// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The date field and its picker in each of their states, the range of a
//! history filter, and a field's inner parts.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, away, escape, focus, hover, shot_of, shot_of_view};
use crate::browser::Browser;
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "date-open",
    "date-hover",
    "date-keyboard",
    "date-time",
    "date-quick",
    "date-months",
    "date-range-half",
    "date-range",
    "date-invalid",
    "date-sheet",
    "date-sheet-range",
    "edge-parts",
];

/// Writes the viewport as `state` with the picker `picker` in its middle.
async fn picker_shot(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
    picker: &WebElement,
) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![picker.to_json()?],
        )
        .await?;
    shot_of_view(browser, brand, theme, state).await
}

/// Loads the forms page of `brand` afresh, so each state starts from the
/// fields as the server drew them.
async fn forms_afresh(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    crate::dates::calendar(browser, "audit-from").await?;
    Ok(())
}

/// Every state of the date picker, as the design's states sheet draws it,
/// then the phone's sheet at 390 px, then the fields with a part inside
/// their edge, one focused.
pub(super) async fn date_states(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    forms_afresh(browser, gallery, brand).await?;
    let picker = crate::dates::open_picker(browser, "audit-from").await?;
    picker_shot(browser, brand, theme, "date-open", &picker).await?;
    let day = picker
        .find(By::Css("button[data-day='2026-10-08']"))
        .await?;
    hover(browser, &day).await?;
    picker_shot(browser, brand, theme, "date-hover", &picker).await?;
    away(browser).await?;
    let start = picker
        .find(By::Css("button[data-day='2026-10-14']"))
        .await?;
    focus(browser, &start).await?;
    start.send_keys(Key::Right).await?;
    let next = picker
        .find(By::Css("button[data-day='2026-10-15']"))
        .await?;
    next.send_keys(Key::Right).await?;
    picker_shot(browser, brand, theme, "date-keyboard", &picker).await?;
    picker
        .find(By::Css("button.fui-datepicker-title"))
        .await?
        .click()
        .await?;
    picker.find(By::Css(".fui-datepicker-months")).await?;
    picker_shot(browser, brand, theme, "date-months", &picker).await?;
    escape(browser).await?;
    let at = browser.element(By::Id("row-at")).await?;
    at.send_keys("2026-10-14 08:30").await?;
    let timed = crate::dates::open_picker(browser, "row-at").await?;
    timed.find(By::Css(".fui-datepicker-time")).await?;
    picker_shot(browser, brand, theme, "date-time", &timed).await?;
    escape(browser).await?;
    let defer = crate::dates::open_picker(browser, "defer-until").await?;
    picker_shot(browser, brand, theme, "date-quick", &defer).await?;
    escape(browser).await?;
    let from = browser.element(By::Id("audit-from")).await?;
    from.clear().await?;
    from.send_keys("2026-10-41").await?;
    let invalid = crate::dates::open_picker(browser, "audit-from").await?;
    picker_shot(browser, brand, theme, "date-invalid", &invalid).await?;
    escape(browser).await?;
    forms_afresh(browser, gallery, brand).await?;
    let range = crate::dates::open_range(browser).await?;
    picker_shot(browser, brand, theme, "date-range", &range).await?;
    range
        .find(By::Css("button[data-day='2026-10-15']"))
        .await?
        .click()
        .await?;
    let end = range.find(By::Css("button[data-day='2026-10-19']")).await?;
    hover(browser, &end).await?;
    picker_shot(browser, brand, theme, "date-range-half", &range).await?;
    escape(browser).await?;
    window(browser, 390, 844).await?;
    forms_afresh(browser, gallery, brand).await?;
    crate::dates::open_picker(browser, "audit-from").await?;
    shot_of_view(browser, brand, theme, "date-sheet").await?;
    escape(browser).await?;
    crate::dates::open_range(browser).await?;
    shot_of_view(browser, brand, theme, "date-sheet-range").await?;
    escape(browser).await?;
    window(browser, WIDTH, HEIGHT).await?;
    let edge = browser
        .element(By::XPath(
            "//input[@id = 'edge-timeout']/ancestor::section[contains(@class, 'fui-card')]",
        ))
        .await?;
    let timeout = browser.element(By::Id("edge-timeout")).await?;
    focus(browser, &timeout).await?;
    shot_of(browser, brand, theme, "edge-parts", &edge).await
}
