// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The skip link on a page that hydrates whole (#482): pressing it moves
//! the sequential focus navigation starting point to the main area, so the
//! next Tab lands on the main area's first focusable element (HTML Living
//! Standard, "scroll to the fragment"; WCAG 2.2 SC 2.4.1 Bypass Blocks).

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, with_browser};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// Marks the main area's first element a Tab can reach with
/// `data-fui-first`, and returns whether there is one.
async fn mark_the_first_in_main(browser: &Browser) -> Result<bool, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute(
            r"const main = document.getElementById('main');
            const first = Array.from(main.querySelectorAll(
              'a[href], button, input, select, textarea, summary, [tabindex]'
            )).find((element) => element.tabIndex >= 0
              && !element.disabled
              && element.checkVisibility());
            if (first) {
              first.setAttribute('data-fui-first', '');
            }
            return first !== undefined;",
            Vec::new(),
        )
        .await?
        .convert()?)
}

/// Presses `keys` on whatever holds the focus.
async fn press(browser: &Browser, keys: impl Into<TypingData>) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(keys)
        .perform()
        .await?;
    Ok(())
}

/// Activates the skip link from the keyboard, presses Tab, and checks the
/// focus landed on the main area's first focusable element.
pub(crate) async fn the_skip_link_reaches_main(browser: &Browser) -> Result<(), Box<dyn Error>> {
    assert!(
        mark_the_first_in_main(browser).await?,
        "the main area holds a focusable element"
    );
    let skip = browser.element(By::Css("a.fui-skip")).await?;
    skip.focus().await?;
    press(browser, Key::Enter).await?;
    press(browser, Key::Tab).await?;
    let active = browser.driver().active_element().await?;
    let inside: bool = browser
        .driver()
        .execute(
            "return document.activeElement.hasAttribute('data-fui-first');",
            Vec::new(),
        )
        .await?
        .convert()?;
    assert!(
        inside,
        "the Tab after the skip link lands on the main area's first focusable element, not on {:?} ({:?})",
        active.tag_name().await?,
        active.text().await?
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_skip_link_moves_the_focus_to_the_main_area_under_hydration()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        browser.hydrated().await?;
        the_skip_link_reaches_main(browser).await?;
        assert!(
            browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .ends_with("/ferrobridge/tables#main"),
            "the address names the fragment"
        );
        browser.console_clean("the skip link").await
    })
    .await
}
