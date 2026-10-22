// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The document pane switched in place to its raw view.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::shot_of;
use crate::browser::{Browser, until};
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &["document-raw"];

/// The document pane switched in place to its raw view.
pub(super) async fn document(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/code", brand.key())))
        .await?;
    browser
        .element(By::Css(".fui-document-views.is-live"))
        .await?;
    browser
        .element(By::Css(".fui-document-switch a[data-view='raw']"))
        .await?
        .click()
        .await?;
    let raw = browser.element(By::Id("document-raw")).await?;
    until("the raw view shows", async || {
        Ok(raw.is_displayed().await?)
    })
    .await?;
    let pane = browser.element(By::Css(".fui-document")).await?;
    shot_of(browser, brand, theme, "document-raw", &pane).await
}
