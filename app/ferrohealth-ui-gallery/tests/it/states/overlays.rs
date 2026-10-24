// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Every overlay open, and the toasts.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, away, escape, press, shot_of, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "dialog-open",
    "dialog-danger",
    "dialog-reason",
    "dialog-info",
    "modal-open",
    "modal-wide",
    "side-panel",
    "user-menu",
    "user-menu-sheet",
    "toasts",
    "popover-form",
    "popover-form-390",
    "dialog-reason-390",
    "side-panel-390",
    "drawer-open",
];

/// Opens the overlay a press on `opener` shows, writes the viewport as
/// `state`, and closes it with Escape.
async fn overlay(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
    opener: By,
    shown: By,
) -> Result<(), Box<dyn Error>> {
    let button = browser.element(opener).await?;
    let target = browser.element(shown).await?;
    until(&format!("{state} opens"), async || {
        if !target.is_displayed().await? {
            press(browser, &button).await?;
        }
        Ok(target.is_displayed().await?)
    })
    .await?;
    away(browser).await?;
    shot_of_view(browser, brand, theme, state).await?;
    escape(browser).await?;
    until(&format!("{state} closes"), async || {
        Ok(!target.is_displayed().await?)
    })
    .await
}

/// The four toasts raised at once, of which the region shows the newest
/// three, then dismissed.
async fn raised_toasts(browser: &Browser, brand: Brand, theme: &str) -> Result<(), Box<dyn Error>> {
    for tone in ["ok", "info", "warn", "bad"] {
        let raise = browser
            .element(By::Css(format!("button[data-toast='{tone}']")))
            .await?;
        press(browser, &raise).await?;
    }
    let region = browser.element(By::Css(".fui-toasts")).await?;
    until("the region shows as many toasts as it holds", async || {
        Ok(region.find_all(By::Css(".fui-toast")).await?.len()
            >= ferrohealth_ui::widget::toast::SHOWN_MAX)
    })
    .await?;
    shot_of(browser, brand, theme, "toasts", &region).await?;
    for close in browser
        .driver()
        .find_all(By::Css(".fui-toast > .fui-icon-button"))
        .await?
    {
        close.click().await?;
    }
    until("the toasts are gone", async || {
        Ok(browser
            .driver()
            .find_all(By::Css(".fui-toast"))
            .await?
            .is_empty())
    })
    .await?;
    Ok(())
}

/// Every overlay open, and the toasts.
pub(super) async fn overlays(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/overlays", brand.key())))
        .await?;
    crate::journeys::hydrated_toasts(browser).await?;
    for close in browser
        .driver()
        .find_all(By::Css(".fui-toast > .fui-icon-button"))
        .await?
    {
        close.click().await?;
    }
    until("the probe's toast is gone", async || {
        Ok(browser
            .driver()
            .find_all(By::Css(".fui-toast"))
            .await?
            .is_empty())
    })
    .await?;
    let opened = [
        (
            "dialog-open",
            "//button[@commandfor = 'reload-set' and not(@role)]",
            "reload-set",
        ),
        (
            "dialog-danger",
            "//button[@role = 'menuitem' and contains(., 'Delete the file')]",
            "delete-file",
        ),
        (
            "dialog-reason",
            "//button[@role = 'menuitem' and @commandfor = 'remove-node-b']",
            "remove-node-b",
        ),
        (
            "dialog-info",
            "//button[@commandfor = 'how-reloading']",
            "how-reloading",
        ),
        (
            "modal-open",
            "//button[@data-delete = 'synthetic-007.model.yml']",
            "delete-row",
        ),
        ("modal-wide", "//button[@id = 'open-rename']", "rename-file"),
        (
            "side-panel",
            "//button[normalize-space(.) = 'View access']",
            "access",
        ),
        (
            "user-menu",
            "//button[@popovertarget = 'user-menu']",
            "user-menu",
        ),
        (
            "popover-form",
            "//button[@commandfor = 'save-query']",
            "save-query",
        ),
    ];
    for (state, opener, id) in opened {
        if state == "dialog-danger" {
            // The delete is More's last item, so More opens first.
            press(
                browser,
                &crate::menus::hydrated_trigger(browser, "overlays-more").await?,
            )
            .await?;
        }
        if state == "dialog-reason" {
            // Remove… is the row menu's item, so the row menu opens first.
            press(
                browser,
                &crate::menus::hydrated_trigger(browser, "member-menu-node-b").await?,
            )
            .await?;
        }
        overlay(browser, brand, theme, state, By::XPath(opener), By::Id(id)).await?;
    }
    raised_toasts(browser, brand, theme).await?;
    phone_overlays(browser, brand, theme).await
}

/// The overlays that open differently on a phone, 390 px wide.
async fn phone_overlays(
    browser: &Browser,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    window(browser, 390, HEIGHT).await?;
    overlay(
        browser,
        brand,
        theme,
        "popover-form-390",
        By::XPath("//button[@commandfor = 'save-query']"),
        By::Id("save-query"),
    )
    .await?;
    // The confirmation sits in its row's actions cell, opened from the row menu.
    press(
        browser,
        &crate::menus::hydrated_trigger(browser, "member-menu-node-b").await?,
    )
    .await?;
    overlay(
        browser,
        brand,
        theme,
        "dialog-reason-390",
        By::XPath("//button[@role = 'menuitem' and @commandfor = 'remove-node-b']"),
        By::Id("remove-node-b"),
    )
    .await?;
    overlay(
        browser,
        brand,
        theme,
        "side-panel-390",
        By::XPath("//button[@commandfor = 'access']"),
        By::Id("access"),
    )
    .await?;
    overlay(
        browser,
        brand,
        theme,
        "drawer-open",
        By::XPath("//button[@aria-label = 'Menu']"),
        By::Id("sidebar"),
    )
    .await?;
    overlay(
        browser,
        brand,
        theme,
        "user-menu-sheet",
        By::XPath("//button[@popovertarget = 'user-menu']"),
        By::Id("user-menu"),
    )
    .await?;
    window(browser, WIDTH, HEIGHT).await
}
