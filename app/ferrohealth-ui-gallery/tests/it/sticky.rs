// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journey that holds the sticky top bar off what the page brings into
//! view (#408): every control the keyboard reaches on a long page, and every
//! one scrolled into view, ends fully below the bar, at 1440 and 390 px (WCAG
//! 2.2 SC 2.4.11, Focus Not Obscured).

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// How many times the journey presses Tab through the page.
const TABS: usize = 90;

/// Where the focused element sits against the top bar: its id or tag, its
/// top, and the bar's bottom, when the element is a visible part of the
/// page's main content.
async fn focused_against_the_bar(
    browser: &Browser,
) -> Result<Option<(String, f64, f64)>, Box<dyn Error>> {
    let read = browser
        .driver()
        .execute(
            r"const element = document.activeElement;
            if (!element || !element.closest('main')) { return null; }
            const box = element.getBoundingClientRect();
            if (box.width === 0 || box.height === 0) { return null; }
            const bar = document.querySelector('.fui-topbar').getBoundingClientRect();
            return [element.id || element.tagName, box.top, Math.max(bar.bottom, 0)];",
            vec![],
        )
        .await?
        .json()
        .clone();
    Ok(serde_json::from_value(read)?)
}

/// Tabs through the forms page at `width` by `height` and scrolls each of
/// its links into view, asserting each ends fully below the bar.
async fn clear_of_the_bar(
    browser: &Browser,
    gallery: &Gallery,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    window(browser, width, height).await?;
    browser.goto(&gallery.url("/ferrobridge/forms")).await?;
    browser.hydrated().await?;
    let mut checked = 0_u32;
    for _ in 0..TABS {
        browser
            .driver()
            .action_chain()
            .send_keys(Key::Tab)
            .perform()
            .await?;
        if let Some((name, top, bar)) = focused_against_the_bar(browser).await? {
            checked += 1;
            assert!(
                top >= bar - 0.5,
                "{name} is focused under the top bar at {width} px: its top {top}, the bar's foot {bar}"
            );
        }
    }
    assert!(
        checked > 20,
        "the journey reached the page's controls: {checked}"
    );
    let hidden: Vec<String> = browser
        .driver()
        .execute(
            r"const bar = document.querySelector('.fui-topbar');
            const under = [];
            for (const element of document.querySelectorAll('main a[href], main button, main input')) {
              const box = element.getBoundingClientRect();
              if (box.width === 0 || box.height === 0) { continue; }
              element.scrollIntoView();
              const top = element.getBoundingClientRect().top;
              const foot = Math.max(bar.getBoundingClientRect().bottom, 0);
              if (top < foot - 0.5) { under.push((element.id || element.tagName) + ' ' + top + ' < ' + foot); }
            }
            return under;",
            vec![],
        )
        .await?
        .convert()?;
    assert!(
        hidden.is_empty(),
        "scrolled into view under the top bar at {width} px: {hidden:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_focused_or_scrolled_to_control_ends_below_the_sticky_top_bar()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        clear_of_the_bar(browser, &gallery, 1440, 900).await?;
        clear_of_the_bar(browser, &gallery, 390, 844).await?;
        browser.console_clean("the sticky bar").await
    })
    .await
}
