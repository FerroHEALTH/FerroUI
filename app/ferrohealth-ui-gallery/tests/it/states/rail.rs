// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The sidebar folded to its rail, an entry under the pointer and an entry
//! with the keyboard focus, each showing its tooltip.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{away, focus, hover, shot_of_view};
use crate::browser::Browser;
use crate::browser::until;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &["rail-focus", "rail-hover"];

/// The rail entry that leads to the forms page.
const FORMS: &str =
    "//*[@id = 'sidebar']//a[span[@class = 'fui-tooltip' and . = 'Forms and editor']]";

/// The sidebar folded to its rail, an entry under the pointer, then one
/// with the keyboard focus, its cookie set for the two shots.
pub(super) async fn rail(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let mut cookie = Cookie::new("gallery-sidebar", "rail");
    cookie.set_path("/");
    browser.driver().add_cookie(cookie).await?;
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    browser
        .element(By::Css("#sidebar.fui-sidebar-rail"))
        .await?;
    let entry = browser.element(By::XPath(FORMS)).await?;
    hover(browser, &entry).await?;
    shot_of_view(browser, brand, theme, "rail-hover").await?;
    away(browser).await?;
    focus(browser, &entry).await?;
    let tooltip = entry.find(By::ClassName("fui-tooltip")).await?;
    until("the focused entry shows its tooltip", async || {
        Ok(tooltip.is_displayed().await?)
    })
    .await?;
    shot_of_view(browser, brand, theme, "rail-focus").await?;
    browser.driver().delete_cookie("gallery-sidebar").await?;
    Ok(())
}
