// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The in-place filter's journey: the count shows at once, and its status
//! region takes the count only once the typing pauses, so assistive
//! technology hears one count for a burst of keys (WCAG 2.2 SC 4.1.3).

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The count the filter keeps with nothing typed.
const ALL: &str = "12 of 12 files on this page.";
/// The count the filter keeps for `d`: the four model files.
const MODELS: &str = "4 of 12 files on this page.";

/// The text of the first element `css` names.
async fn text_of(browser: &Browser, css: &str) -> Result<String, Box<dyn Error>> {
    Ok(browser.element(By::Css(css)).await?.text().await?)
}

/// The text the filter's status region holds, read from the page as
/// assistive technology reads it, though the region is out of sight.
async fn status(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute(
            "return document.querySelector('.fui-table-tools [role=status]').textContent;",
            Vec::new(),
        )
        .await?
        .convert()?)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_filter_says_its_count_once_the_typing_pauses() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        let field = browser.element(By::Id("gallery-filter")).await?;
        let shown = ".fui-table-tools p[aria-hidden='true']";
        until("the filter hydrates and narrows the files", async || {
            field.clear().await?;
            field.send_keys("d").await?;
            Ok(text_of(browser, shown).await? == MODELS)
        })
        .await?;
        until(
            "the status takes the count the typing paused on",
            async || Ok(status(browser).await? == MODELS),
        )
        .await?;
        browser
            .driver()
            .execute(
                "const region = document.querySelector('.fui-table-tools [role=status]');
                 window.saidCounts = [];
                 new MutationObserver(() => window.saidCounts.push(region.textContent))
                   .observe(region, { childList: true, characterData: true, subtree: true });",
                Vec::new(),
            )
            .await?;
        field
            .send_keys(Key::Backspace + "d" + Key::Backspace)
            .await?;
        until("the count shows at once", async || {
            Ok(text_of(browser, shown).await? == ALL)
        })
        .await?;
        until("the status takes the last count", async || {
            Ok(status(browser).await? == ALL)
        })
        .await?;
        let said: Vec<String> = browser
            .driver()
            .execute("return window.saidCounts;", Vec::new())
            .await?
            .convert()?;
        assert!(
            said.iter().all(|count| count == ALL),
            "three keys, one count said: {said:?}"
        );
        browser.console_clean("the filter").await
    })
    .await
}
