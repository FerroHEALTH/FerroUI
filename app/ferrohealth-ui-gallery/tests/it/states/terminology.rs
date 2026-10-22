// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The terminology parts: the FHIR version select open in the top bar and
//! its group in the phone's user menu, a repeatable row just added with
//! the focus in its first field, and the root matrix scrolled sideways on
//! a phone with its first column in place.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, escape, hover, press, shot_of, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "version-select-open",
    "repeat-row-added",
    "version-user-menu-390",
    "root-matrix-scrolled-390",
];

/// The version select's menu open, a filter row added to the composer,
/// then on a phone the user menu with the version group and the matrix
/// scrolled sideways.
pub(super) async fn terminology(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let page = gallery.url(&format!("/{}/terminology", brand.key()));
    browser.goto(&page).await?;
    let select = crate::menus::hydrated_trigger(browser, "fhir-version").await?;
    press(browser, &select).await?;
    until("the version menu opens", async || {
        Ok(select.attr("aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    let heading = browser.element(By::Css("main h1")).await?;
    hover(browser, &heading).await?;
    shot_of_view(browser, brand, theme, "version-select-open").await?;
    escape(browser).await?;
    let list = browser.element(By::Id("include-filters")).await?;
    let add = list.find(By::Css("[data-repeat='add']")).await?;
    press(browser, &add).await?;
    until("Add inserts a row", async || {
        Ok(!list
            .find_all(By::Css("[data-row='new-1']"))
            .await?
            .is_empty())
    })
    .await?;
    let include = browser
        .element(By::Css("#value-set-composer .fui-clause"))
        .await?;
    shot_of(browser, brand, theme, "repeat-row-added", &include).await?;

    window(browser, 390, 844).await?;
    browser.goto(&page).await?;
    let menu = browser
        .element(By::Css("button[popovertarget='user-menu']"))
        .await?;
    menu.click().await?;
    let group = browser
        .element(By::Css("#user-menu .fui-menu-version + .fui-menu-version"))
        .await?;
    until("the user menu shows the version group", async || {
        group.is_displayed().await.map_err(Into::into)
    })
    .await?;
    shot_of_view(browser, brand, theme, "version-user-menu-390").await?;
    escape(browser).await?;
    let matrix = browser.element(By::Css(".fui-root-matrix")).await?;
    browser
        .driver()
        .execute(
            "arguments[0].querySelector('.fui-table-scroll').scrollLeft = 160;",
            vec![matrix.to_json()?],
        )
        .await?;
    shot_of(browser, brand, theme, "root-matrix-scrolled-390", &matrix).await?;
    window(browser, WIDTH, HEIGHT).await
}
