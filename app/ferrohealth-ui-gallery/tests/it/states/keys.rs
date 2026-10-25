// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The states of the keys: the top bar's search under the pointer, filled,
//! busy, disabled, with its answer, an active hit, none and an error, at
//! 1440 and 390 px; the
//! docked decision bar in each state, and on a phone; the shortcut legend
//! open over the page and the hint at a listing's foot.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, away, card, escape, hover, shot_of, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "hover-search",
    "search-filled",
    "search-busy",
    "search-disabled",
    "search-answer",
    "search-active",
    "search-none",
    "search-error",
    "search-390",
    "search-answer-390",
    "dock-idle",
    "dock-missing",
    "dock-disabled",
    "dock-submitting",
    "dock-done",
    "dock-stale",
    "dock-bare",
    "dock-path",
    "dock-idle-390",
    "dock-disabled-390",
    "dock-path-390",
    "legend-open",
    "shortcut-hint",
];

/// The top bar of `brand`'s forms page with the search in the state
/// `search` names, written as `state`.
async fn top_bar(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
    search: &str,
    state: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms?search={search}", brand.key())))
        .await?;
    let bar = browser.element(By::Css(".fui-topbar")).await?;
    shot_of(browser, brand, theme, state, &bar).await
}

/// The search's states.
async fn search(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let bar = browser.element(By::Css(".fui-topbar")).await?;
    let group = browser.element(By::Css(".fui-topbar .fui-search")).await?;
    hover(browser, &group).await?;
    shot_of(browser, brand, theme, "hover-search", &bar).await?;
    away(browser).await?;
    let field = browser.element(By::Id("gallery-search")).await?;
    field.send_keys("synthetic-007").await?;
    browser
        .driver()
        .execute("document.activeElement.blur();", vec![])
        .await?;
    shot_of(browser, brand, theme, "search-filled", &bar).await?;
    top_bar(browser, gallery, brand, theme, "busy", "search-busy").await?;
    top_bar(
        browser,
        gallery,
        brand,
        theme,
        "disabled",
        "search-disabled",
    )
    .await?;
    let forms = gallery.url(&format!("/{}/forms", brand.key()));
    browser.goto(&forms).await?;
    crate::keys::typed(browser, "synthetic").await?;
    shot_of_view(browser, brand, theme, "search-answer").await?;
    for _ in 0..6 {
        press_down(browser).await?;
    }
    shot_of_view(browser, brand, theme, "search-active").await?;
    for (words, state) in [("zz-unknown", "search-none"), ("zz-fail", "search-error")] {
        browser.goto(&forms).await?;
        crate::keys::typed(browser, words).await?;
        shot_of_view(browser, brand, theme, state).await?;
    }
    browser.console_errors().await?;
    window(browser, 390, HEIGHT).await?;
    top_bar(browser, gallery, brand, theme, "", "search-390").await?;
    browser.goto(&forms).await?;
    crate::keys::typed(browser, "synthetic").await?;
    shot_of_view(browser, brand, theme, "search-answer-390").await?;
    window(browser, WIDTH, HEIGHT).await
}

/// Presses the down arrow on the search field.
async fn press_down(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(Key::Down)
        .perform()
        .await?;
    Ok(())
}

/// The docked bar of `brand`'s forms page in the state `dock` names,
/// written as `state`.
async fn bar(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
    dock: &str,
    state: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms?dock={dock}", brand.key())))
        .await?;
    let bar = browser.element(By::Css(".fui-dock")).await?;
    shot_of(browser, brand, theme, state, &bar).await
}

/// The docked bar's states.
async fn dock(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    for (dock, state) in [
        ("idle", "dock-idle"),
        ("missing", "dock-missing"),
        ("disabled", "dock-disabled"),
        ("submitting", "dock-submitting"),
        ("done", "dock-done"),
        ("stale", "dock-stale"),
        ("bare", "dock-bare"),
        ("path", "dock-path"),
    ] {
        bar(browser, gallery, brand, theme, dock, state).await?;
    }
    window(browser, 390, HEIGHT).await?;
    for (dock, state) in [
        ("idle", "dock-idle-390"),
        ("disabled", "dock-disabled-390"),
        ("path", "dock-path-390"),
    ] {
        bar(browser, gallery, brand, theme, dock, state).await?;
    }
    window(browser, WIDTH, HEIGHT).await
}

/// The legend open over the overlays page, and the hint.
async fn legend(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/overlays", brand.key())))
        .await?;
    let shortcuts = browser.element(card("Keyboard shortcuts")).await?;
    shot_of(browser, brand, theme, "shortcut-hint", &shortcuts).await?;
    let opener = browser.element(By::Id("open-shortcuts")).await?;
    let legend = browser.element(By::Id("shortcuts")).await?;
    until("the legend opens", async || {
        if !legend.is_displayed().await? && opener.is_displayed().await? {
            opener.click().await?;
        }
        Ok(legend.is_displayed().await?)
    })
    .await?;
    away(browser).await?;
    shot_of_view(browser, brand, theme, "legend-open").await?;
    escape(browser).await?;
    until("the legend closes", async || {
        Ok(!legend.is_displayed().await?)
    })
    .await
}

/// Writes every state of the keys of `brand` in `theme`.
pub(super) async fn keys(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    search(browser, gallery, brand, theme).await?;
    dock(browser, gallery, brand, theme).await?;
    legend(browser, gallery, brand, theme).await
}
