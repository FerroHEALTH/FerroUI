// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Your preferences: a row just saved, the reset confirmation open, and
//! the three tabs on a phone.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, press, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "preferences-saved",
    "preferences-reset",
    "preferences-phone",
    "preferences-language-phone",
];

/// A row just saved (FerroPIX's reveal duration, the density elsewhere),
/// the reset dialog open, and Appearance and Language and region at 390 px.
pub(super) async fn preferences(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let root = format!("/{}/preferences", brand.key());
    let saved = if brand == Brand::Pix {
        "reveal"
    } else {
        "density"
    };
    browser
        .goto(&gallery.url(&format!("{root}?saved={saved}")))
        .await?;
    browser
        .element(By::Css(".fui-setting-saved[role='status']"))
        .await?;
    shot_of_view(browser, brand, theme, "preferences-saved").await?;
    browser.goto(&gallery.url(&root)).await?;
    let reset = browser
        .element(By::Css("button[popovertarget='preferences-reset']"))
        .await?;
    press(browser, &reset).await?;
    let dialog = browser.element(By::Id("preferences-reset")).await?;
    until("the reset dialog opens", async || {
        Ok(dialog.is_displayed().await?)
    })
    .await?;
    shot_of_view(browser, brand, theme, "preferences-reset").await?;
    window(browser, 390, HEIGHT).await?;
    for (path, state) in [
        (root.clone(), "preferences-phone"),
        (format!("{root}/language"), "preferences-language-phone"),
    ] {
        browser.goto(&gallery.url(&path)).await?;
        browser.element(By::Css("main h1")).await?;
        shot_of_view(browser, brand, theme, state).await?;
    }
    window(browser, WIDTH, HEIGHT).await?;
    Ok(())
}
