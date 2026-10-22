// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The field shapes with their error lines, and the pick-or-type fields
//! switched to their typed inputs.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{card, focus, shot_of};
use crate::browser::{Browser, until};
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "field-shapes",
    "pick-or-type-typed",
    "pick-or-type-combobox-typed",
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
    shot_of(browser, brand, theme, "pick-or-type-combobox-typed", &picks).await
}
