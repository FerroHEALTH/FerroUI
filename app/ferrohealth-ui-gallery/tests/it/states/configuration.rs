// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The configuration parts: the revision on trial with its countdown
//! running, the secret field's choice under the focus and moved to Read
//! from a file, and the pending cards, the lifecycle states, the secret
//! fields, the provenance table and the provisioned file's title row at
//! 390 px.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, focus, press, shot_of, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "pending-trial",
    "secret-choice-focus",
    "secret-choice-file",
    "pending-390",
    "lifecycle-390",
    "secrets-390",
    "provenance-390",
    "provisioned-390",
];

/// The card named `label`.
fn named(label: &str) -> By {
    By::XPath(format!("//section[@aria-label = '{label}']"))
}

/// The configuration page's states at 1440 px, then at 390 px.
pub(super) async fn configuration(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let page = gallery.url(&format!("/{}/configuration", brand.key()));
    browser.goto(&page).await?;
    let time = browser
        .element(By::Css("#trial-countdown .fui-countdown-words strong"))
        .await?;
    until("the countdown runs", async || {
        Ok(time.text().await? != "1:42")
    })
    .await?;
    let trial = browser.element(named("Revision 10 runs on trial")).await?;
    shot_of(browser, brand, theme, "pending-trial", &trial).await?;
    let secrets = browser.element(By::Id("secret-kept")).await?;
    let value = browser.element(By::Id("secret-kept-mode-value")).await?;
    focus(browser, &value).await?;
    shot_of(browser, brand, theme, "secret-choice-focus", &secrets).await?;
    let file = browser
        .element(By::Css("label[for='secret-kept-mode-file']"))
        .await?;
    press(browser, &file).await?;
    let path = browser.element(By::Id("secret-kept-file")).await?;
    until("the file choice reveals its path", async || {
        Ok(path.is_displayed().await?)
    })
    .await?;
    shot_of(browser, brand, theme, "secret-choice-file", &secrets).await?;

    window(browser, 390, HEIGHT).await?;
    browser.goto(&page).await?;
    let pending = browser.element(By::Id("pending")).await?;
    shot_of(browser, brand, theme, "pending-390", &pending).await?;
    let lifecycles = browser.element(named("Revision lifecycle")).await?;
    shot_of(browser, brand, theme, "lifecycle-390", &lifecycles).await?;
    let secrets = browser.element(named("Secret field")).await?;
    shot_of(browser, brand, theme, "secrets-390", &secrets).await?;
    let provenance = browser.element(By::Id("provenance")).await?;
    shot_of(browser, brand, theme, "provenance-390", &provenance).await?;
    browser
        .goto(&gallery.url(&format!("/{}/configuration/provisioned", brand.key())))
        .await?;
    browser.element(By::Css("main .fui-tag-locked")).await?;
    shot_of_view(browser, brand, theme, "provisioned-390").await?;
    window(browser, WIDTH, HEIGHT).await
}
