// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The layouts page: the search box focused, the help switch on, the picker
//! open on its input, and the tree walked by the keyboard.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{card, focus, press, shot_of};
use crate::browser::{Browser, until};
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "focus-search",
    "help-off",
    "help-on",
    "picker-other",
    "tree-keyboard",
];

/// The layouts page: the search box focused, the help switch off and on, the
/// picker open on its input, and the tree walked by the keyboard.
pub(super) async fn layouts(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/layouts", brand.key())))
        .await?;
    let search = browser.element(By::Id("gallery-search")).await?;
    focus(browser, &search).await?;
    let bar = browser.element(By::Css(".fui-topbar")).await?;
    shot_of(browser, brand, theme, "focus-search", &bar).await?;
    let form = browser.element(card("Main and aside")).await?;
    shot_of(browser, brand, theme, "help-off", &form).await?;
    let switch = browser.element(By::Id("run-help")).await?;
    until("the help switch answers", async || {
        if !switch.is_selected().await? {
            press(browser, &switch).await?;
        }
        Ok(switch.is_selected().await?)
    })
    .await?;
    shot_of(browser, brand, theme, "help-on", &form).await?;
    let choice = browser.element(By::Id("run-value-set-choice")).await?;
    browser

        .driver()

        .execute("arguments[0].selectedIndex = arguments[0].options.length - 1; arguments[0].dispatchEvent(new Event('change', {bubbles: true}));", vec![choice.to_json()?])

        .await?;
    let input = browser.element(By::Id("run-value-set")).await?;
    until("the picker shows its input", async || {
        Ok(input.is_displayed().await?)
    })
    .await?;
    input.clear().await?;
    input.send_keys("http://example.org/vs/mine").await?;
    shot_of(browser, brand, theme, "picker-other", &form).await?;
    let first = browser
        .element(By::Css("#gallery-tree [role='treeitem']"))
        .await?;
    focus(browser, &first).await?;
    first.send_keys(Key::Down).await?;
    let pane = browser.element(card("Three panes and the tree")).await?;
    shot_of(browser, brand, theme, "tree-keyboard", &pane).await
}
