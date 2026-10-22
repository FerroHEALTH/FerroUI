// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The trees page: the concept tree walked by the keyboard, and the folder
//! tree in an edit with the focus on a row.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{card, focus, shot_of};
use crate::browser::{Browser, until};
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &["trees-folder-focus", "trees-keyboard"];

/// Focuses the item `selector` until `key` moves the focus off it, which
/// also waits for its island to hydrate.
async fn walk(browser: &Browser, selector: &str, key: Key) -> Result<(), Box<dyn Error>> {
    until("the tree answers the keyboard", async || {
        let item = browser.element(By::Css(selector)).await?;
        focus(browser, &item).await?;
        item.send_keys(key.clone()).await?;
        let active = browser.driver().active_element().await?;
        Ok(active.attr("role").await?.as_deref() == Some("treeitem")
            && active.attr("id").await? != item.attr("id").await?)
    })
    .await
}

/// The trees page: the concept tree walked by the keyboard, and the folder
/// tree in an edit with the focus on a row.
pub(super) async fn trees(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/trees", brand.key())))
        .await?;
    walk(browser, "#trees-deep [tabindex='0']", Key::Up).await?;
    let deep = browser
        .element(card("A deep path and an inactive concept"))
        .await?;
    shot_of(browser, brand, theme, "trees-keyboard", &deep).await?;
    walk(browser, "#trees-directory-edit > li", Key::Down).await?;
    let directory = browser.element(By::Id("trees-directory")).await?;
    shot_of(browser, brand, theme, "trees-folder-focus", &directory).await
}
