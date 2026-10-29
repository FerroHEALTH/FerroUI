// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The configuration page in a browser: the countdown counting down once
//! it is enhanced, the secret field's choice moved by the keyboard revealing
//! its control, the lifecycle stacked on a phone inside the layout
//! viewport, and the page's floor with no script.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The time the trial countdown shows.
async fn time_left(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .element(By::Css("#trial-countdown .fui-countdown-words strong"))
        .await?
        .text()
        .await?)
}

/// The page with no script: the countdown shows the time the server
/// drew, both trial actions are links, the secret field shows the chosen
/// choice's control alone, and no value is filled.
pub(crate) async fn config_floor(browser: &Browser) -> Result<(), Box<dyn Error>> {
    assert_eq!("1:42", time_left(browser).await?, "the server's time left");
    browser
        .element(By::Css("#pending a.fui-button-primary[href$='#confirmed']"))
        .await?;
    let refused = browser.element(By::Id("secret-refused-value")).await?;
    assert!(refused.is_displayed().await?, "the chosen control shows");
    assert_eq!(
        Some(String::from("off")),
        refused.attr("autocomplete").await?
    );
    assert_eq!(
        Some(String::new()),
        refused.prop("value").await?,
        "the value field is never filled"
    );
    let kept = browser.element(By::Id("secret-kept-value")).await?;
    assert!(!kept.is_displayed().await?, "Keep reveals no control");
    browser
        .element(By::Css("#provenance .fui-tag-locked[title]"))
        .await?;
    Ok(())
}

/// The provisioned file with no script: the lock tag in its title row
/// names its owner in its tooltip.
pub(crate) async fn provisioned_floor(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser
        .element(By::Css("main .fui-tag-locked[title]"))
        .await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_countdown_counts_down_and_the_secret_choice_reveals_its_control()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrofed/configuration"))
            .await?;
        let words = browser
            .element(By::Css("#trial-countdown [role='timer'][aria-live='off']"))
            .await?;
        assert!(words.is_displayed().await?);
        until("the countdown counts down", async || {
            Ok(time_left(browser).await? != "1:42")
        })
        .await?;
        let status = browser
            .element(By::Css("#trial-countdown [role='status']"))
            .await?;
        assert!(
            status.text().await?.trim().is_empty(),
            "the polite region says nothing between its set points"
        );

        let keep = browser.element(By::Id("secret-kept-mode-keep")).await?;
        let value = browser.element(By::Id("secret-kept-value")).await?;
        assert!(!value.is_displayed().await?, "Keep reveals no control");
        browser
            .driver()
            .execute("arguments[0].focus();", vec![keep.to_json()?])
            .await?;
        browser
            .driver()
            .action_chain()
            .send_keys(Key::Right)
            .perform()
            .await?;
        let chosen = browser.element(By::Id("secret-kept-mode-value")).await?;
        until("the arrow key chooses Set a value", async || {
            Ok(chosen.prop("checked").await?.as_deref() == Some("true"))
        })
        .await?;
        assert!(
            value.is_displayed().await?,
            "the radio group pattern: the chosen choice reveals its control"
        );
        browser.console_clean("the configuration page").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_lifecycle_stacks_on_a_phone_inside_the_layout_viewport() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 900).await?;
        browser
            .goto(&gallery.url("/ferrobridge/configuration"))
            .await?;
        browser.element(By::Css("main h1")).await?;
        // NOTE: CSSOM View, `clientWidth`: a classic scrollbar narrows the root's client box.
        let [first, second, right, client]: [f64; 4] = browser
            .driver()
            .execute(
                "const steps = document.querySelectorAll('.fui-lifecycle')[0].children;
                 const wide = [...document.querySelectorAll('.fui-pending, .fui-secret, .fui-lifecycle')]
                     .map((part) => part.getBoundingClientRect().right);
                 return [steps[0].getBoundingClientRect().left,
                         steps[1].getBoundingClientRect().left,
                         Math.max(...wide), document.documentElement.clientWidth];",
                Vec::new(),
            )
            .await?
            .convert()?;
        assert!(
            (first - second).abs() <= 1.0,
            "the steps stack at 390 px ({first} and {second})"
        );
        assert!(
            right <= client + 0.5,
            "the configuration parts fit the layout viewport ({right} in {client})"
        );
        browser.console_clean("the configuration page on a phone").await
    })
    .await
}
