// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The sidebar folded to its rail, an entry under the pointer.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{away, hover, shot_of_view};
use crate::browser::Browser;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &["rail-hover"];

/// The sidebar folded to its rail, an entry under the pointer, its cookie
/// set for the one shot.
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
    let entry = browser
        .element(By::Css("#sidebar a[title='Forms and editor']"))
        .await?;
    hover(browser, &entry).await?;
    shot_of_view(browser, brand, theme, "rail-hover").await?;
    away(browser).await?;
    browser.driver().delete_cookie("gallery-sidebar").await?;
    Ok(())
}
