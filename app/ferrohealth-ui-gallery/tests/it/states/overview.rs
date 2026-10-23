// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The pointer and focus states of the overview.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, away, card, focus, hover, shot_of};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "hover-nav",
    "focus-nav",
    "hover-stat",
    "hover-card-action",
    "focus-card-action",
    "hover-primary",
    "hover-secondary",
    "hover-danger",
    "hover-destructive",
    "hover-ghost",
    "focus-button",
    "hover-tab",
    "focus-keyboard-tabs",
    "tabs-select-390",
    "focus-user-menu",
];

/// The pointer and focus states of the overview.
pub(super) async fn overview(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}", brand.key())))
        .await?;
    crate::journeys::hydrated_tabs(browser).await?;
    let sidebar = browser.element(By::Id("sidebar")).await?;
    let entry = browser.element(By::LinkText("Forms and editor")).await?;
    hover(browser, &entry).await?;
    shot_of(browser, brand, theme, "hover-nav", &sidebar).await?;
    away(browser).await?;
    focus(browser, &entry).await?;
    shot_of(browser, brand, theme, "focus-nav", &sidebar).await?;
    let opener = browser
        .element(By::Css("button[popovertarget='user-menu']"))
        .await?;
    focus(browser, &opener).await?;
    let bar = browser.element(By::Css(".fui-topbar")).await?;
    shot_of(browser, brand, theme, "focus-user-menu", &bar).await?;
    let stats = browser.element(By::Css(".fui-grid-stats")).await?;
    let first = browser.element(By::Css("a.fui-stat")).await?;
    hover(browser, &first).await?;
    shot_of(browser, brand, theme, "hover-stat", &stats).await?;
    let preview = browser.element(card("Recent files")).await?;
    let action = browser.element(By::Css("a.fui-card-action")).await?;
    hover(browser, &action).await?;
    shot_of(browser, brand, theme, "hover-card-action", &preview).await?;
    away(browser).await?;
    focus(browser, &action).await?;
    shot_of(browser, brand, theme, "focus-card-action", &preview).await?;
    let buttons = browser.element(card("Buttons")).await?;
    for (variant, state) in [
        ("Primary", "hover-primary"),
        ("Secondary", "hover-secondary"),
        ("Danger", "hover-danger"),
        ("Destructive", "hover-destructive"),
        ("Ghost", "hover-ghost"),
    ] {
        let button = browser
            .element(By::Css(format!("button[data-variant='{variant}']")))
            .await?;
        hover(browser, &button).await?;
        shot_of(browser, brand, theme, state, &buttons).await?;
    }
    away(browser).await?;
    let primary = browser
        .element(By::Css("button[data-variant='Primary']"))
        .await?;
    focus(browser, &primary).await?;
    shot_of(browser, brand, theme, "focus-button", &buttons).await?;
    let tabs = browser.element(card("Tabs")).await?;
    let tab = browser.element(By::PartialLinkText("Diagnostics")).await?;
    hover(browser, &tab).await?;
    shot_of(browser, brand, theme, "hover-tab", &tabs).await?;
    away(browser).await?;
    let first_tab = browser.element(By::Id("answer-tab-0")).await?;
    focus(browser, &first_tab).await?;
    first_tab.send_keys(Key::Right).await?;
    shot_of(browser, brand, theme, "focus-keyboard-tabs", &tabs).await?;
    away(browser).await?;
    window(browser, 390, HEIGHT).await?;
    let select = browser.element(By::Css(".fui-tabs-select select")).await?;
    until("the tab select takes the row's place", async || {
        Ok(select.is_displayed().await?)
    })
    .await?;
    let tabs = browser.element(card("Tabs")).await?;
    shot_of(browser, brand, theme, "tabs-select-390", &tabs).await?;
    window(browser, WIDTH, HEIGHT).await
}
