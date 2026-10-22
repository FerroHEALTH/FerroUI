// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tables page: a row and a sort header under the pointer and the focus,
//! the editor's diagnostics, and the compact density.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{away, card, focus, hover, shot_of, shot_of_view};
use crate::browser::Browser;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "hover-row",
    "focus-row",
    "focus-sort",
    "editor-diagnostic",
    "density-compact",
];

/// The row under the pointer, and the editor with its diagnostics.
pub(super) async fn tables_and_editor(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    let region = browser.element(By::Css(".fui-table-scroll")).await?;
    let row = browser
        .element(By::XPath("(//table)[1]/tbody/tr[3]"))
        .await?;
    hover(browser, &row).await?;
    shot_of(browser, brand, theme, "hover-row", &region).await?;
    away(browser).await?;
    let link = row.find(By::Css("a.fui-row-link")).await?;
    focus(browser, &link).await?;
    shot_of(browser, brand, theme, "focus-row", &row).await?;
    let header = browser.element(By::XPath("(//table)[1]/thead/tr")).await?;
    let sort = header
        .find(By::XPath(".//th[contains(., 'Lines')]//button"))
        .await?;
    focus(browser, &sort).await?;
    shot_of(browser, brand, theme, "focus-sort", &header).await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let editor = browser.element(card("Highlighted editor")).await?;
    shot_of(browser, brand, theme, "editor-diagnostic", &editor).await
}

/// The tables page in the compact density, its cookie set for the one shot.
pub(super) async fn compact(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let mut cookie = Cookie::new("gallery-density", "compact");
    cookie.set_path("/");
    browser.driver().add_cookie(cookie).await?;
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    browser.element(By::Css("main h1")).await?;
    shot_of_view(browser, brand, theme, "density-compact").await?;
    browser.driver().delete_cookie("gallery-density").await?;
    Ok(())
}
