// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The title row's action group, its menus and the select button's listbox, at
//! 1440 and 390 px.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, escape, hover, press, shot_of, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "action-group",
    "menu-open",
    "menu-keyboard",
    "split-open",
    "row-menu-open",
    "copy-menu-open",
    "listbox-open",
    "action-group-390",
    "menu-sheet-390",
    "listbox-sheet-390",
];

/// Opens the menu `id` by a press on its trigger, scrolled to the middle of
/// the viewport, and writes the viewport as `state`.
async fn opened_menu(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
    id: &str,
) -> Result<(), Box<dyn Error>> {
    let trigger = crate::menus::hydrated_trigger(browser, id).await?;
    press(browser, &trigger).await?;
    until(&format!("{state} opens"), async || {
        Ok(trigger.attr("aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    let heading = browser.element(By::Css("main h1")).await?;
    hover(browser, &heading).await?;
    shot_of_view(browser, brand, theme, state).await
}

/// Closes the menu `id` with Escape from its trigger.
async fn closed_menu(browser: &Browser, id: &str) -> Result<(), Box<dyn Error>> {
    escape(browser).await?;
    let trigger = crate::menus::hydrated_trigger(browser, id).await?;
    until(&format!("{id} closes"), async || {
        Ok(trigger.attr("aria-expanded").await?.as_deref() == Some("false"))
    })
    .await
}

/// The title row's action group, More open by the pointer and by the
/// keyboard, the split button's variants, a row menu, the menu with a copy
/// item and the select button's listbox, then the action group, More's
/// bottom sheet and the listbox's at 390 px.
pub(super) async fn menu_states(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let page = gallery.url(&format!("/{}/overlays", brand.key()));
    browser.goto(&page).await?;
    crate::menus::hydrated_trigger(browser, "overlays-more").await?;
    let header = browser.element(By::Css(".fui-page-header")).await?;
    shot_of(browser, brand, theme, "action-group", &header).await?;
    browser
        .driver()
        .execute("window.scrollTo(0, 0);", vec![])
        .await?;
    opened_menu(browser, brand, theme, "menu-open", "overlays-more").await?;
    browser
        .driver()
        .action_chain()
        .send_keys(Key::Down)
        .perform()
        .await?;
    shot_of_view(browser, brand, theme, "menu-keyboard").await?;
    closed_menu(browser, "overlays-more").await?;
    opened_menu(browser, brand, theme, "split-open", "save-variants").await?;
    closed_menu(browser, "save-variants").await?;
    opened_menu(browser, brand, theme, "row-menu-open", "row-menu-0").await?;
    closed_menu(browser, "row-menu-0").await?;
    opened_menu(browser, brand, theme, "copy-menu-open", "alone-more").await?;
    closed_menu(browser, "alone-more").await?;
    opened_menu(browser, brand, theme, "listbox-open", "version-list").await?;
    closed_menu(browser, "version-list").await?;
    window(browser, 390, 844).await?;
    browser.goto(&page).await?;
    crate::menus::hydrated_trigger(browser, "overlays-more").await?;
    let header = browser.element(By::Css(".fui-page-header")).await?;
    shot_of(browser, brand, theme, "action-group-390", &header).await?;
    opened_menu(browser, brand, theme, "menu-sheet-390", "overlays-more").await?;
    closed_menu(browser, "overlays-more").await?;
    opened_menu(browser, brand, theme, "listbox-sheet-390", "version-list").await?;
    closed_menu(browser, "version-list").await?;
    window(browser, WIDTH, HEIGHT).await
}
