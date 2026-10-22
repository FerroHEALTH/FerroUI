// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The phone's compact stat tiles and stacked timestamps, and the disabled
//! fields inside a well.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, card, shot_of, shot_of_view};
use crate::browser::Browser;
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "stats-compact-320",
    "stats-compact-390",
    "timestamp-stacked-320",
    "disabled-well",
    "disabled-well-390",
];

/// The phone's compact stat tiles at 320 and 390 px, the stacked table's
/// timestamps at 320 px, and the disabled fields inside the forms page's
/// well.
pub(super) async fn phone(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    for (width, state) in [(320, "stats-compact-320"), (390, "stats-compact-390")] {
        window(browser, width, HEIGHT).await?;
        browser
            .goto(&gallery.url(&format!("/{}", brand.key())))
            .await?;
        let stats = browser.element(By::Css(".fui-grid-stats")).await?;
        shot_of(browser, brand, theme, state, &stats).await?;
    }
    window(browser, 320, HEIGHT).await?;
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    let first = browser
        .element(By::XPath("(//table)[1]/tbody/tr[1]"))
        .await?;
    browser
        .driver()
        .execute(
            "window.scrollTo(0, arguments[0].getBoundingClientRect().top + window.scrollY - 72);",
            vec![first.to_json()?],
        )
        .await?;
    shot_of_view(browser, brand, theme, "timestamp-stacked-320").await?;
    window(browser, WIDTH, HEIGHT).await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let grouped = browser.element(card("Grouped fields")).await?;
    shot_of(browser, brand, theme, "disabled-well", &grouped).await?;
    window(browser, 390, HEIGHT).await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let grouped = browser.element(card("Grouped fields")).await?;
    shot_of(browser, brand, theme, "disabled-well-390", &grouped).await?;
    window(browser, WIDTH, HEIGHT).await
}
