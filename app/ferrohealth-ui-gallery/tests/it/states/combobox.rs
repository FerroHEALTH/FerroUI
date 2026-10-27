// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The combobox on the forms page: open, an option under the pointer, an option
//! the keyboard highlights, and open at 390 px.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, escape, focus, hover, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "combobox-open",
    "combobox-hover",
    "combobox-keyboard",
    "combobox-narrow",
];

/// The combobox on the forms page: open, an option under the pointer, an
/// option the keyboard highlights, and open at 390 px, each with the page
/// around it.
pub(super) async fn combobox_states(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let input = browser
        .element(By::Css("#choose-system[role='combobox']"))
        .await?;
    let place = |input: &WebElement| {
        let script = "arguments[0].scrollIntoView({block: 'start'}); window.scrollBy(0, -120);";
        let json = input.to_json();
        async move {
            browser.driver().execute(script, vec![json?]).await?;
            Ok::<(), Box<dyn Error>>(())
        }
    };
    place(&input).await?;
    input.click().await?;
    until("the combobox opens", async || {
        Ok(input.attr("aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    let label = browser
        .element(By::Css("label[for='choose-system']"))
        .await?;
    hover(browser, &label).await?;
    shot_of_view(browser, brand, theme, "combobox-open").await?;
    let third = browser
        .element(By::Css("#choose-system-list [role='option']:nth-child(3)"))
        .await?;
    hover(browser, &third).await?;
    shot_of_view(browser, brand, theme, "combobox-hover").await?;
    escape(browser).await?;
    focus(browser, &input).await?;
    input.send_keys(Key::Down).await?;
    input.send_keys(Key::Down).await?;
    input.send_keys(Key::Down).await?;
    hover(browser, &label).await?;
    shot_of_view(browser, brand, theme, "combobox-keyboard").await?;
    escape(browser).await?;
    window(browser, 390, 844).await?;
    let input = browser
        .element(By::Css("#choose-system[role='combobox']"))
        .await?;
    place(&input).await?;
    // A click that lands while the narrow layout settles opens nothing, so
    // the input is clicked again until the list shows.
    until("the combobox opens at 390 px", async || {
        if input.attr("aria-expanded").await?.as_deref() != Some("true") {
            input.click().await?;
        }
        Ok(input.attr("aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    // NOTE: no specification governs this: our own design; the click that opens the list
    // may scroll the page, so the input is placed again before the shot (#434).
    place(&input).await?;
    shot_of_view(browser, brand, theme, "combobox-narrow").await?;
    escape(browser).await?;
    window(browser, WIDTH, HEIGHT).await
}
