// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code editor beside its diagnostics, with its draft at rest and with
//! a diagnostic followed; the empty editor of a new file; and the
//! diagnostics list by file: each at 1440 and 390 px.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, press, shot_of};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "code-editor-draft",
    "code-editor-current",
    "code-editor-empty",
    "diagnostics-by-file",
    "code-editor-draft-390",
    "code-editor-empty-390",
    "diagnostics-by-file-390",
];

/// Opens the forms page afresh and waits for the code editor to hydrate.
async fn forms(browser: &Browser, gallery: &Gallery, brand: Brand) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let draft = browser.element(By::Id("gallery-mapping")).await?;
    until("the code editor may be edited", async || {
        Ok(draft.attr("readonly").await?.is_none())
    })
    .await
}

/// Takes the code editor at rest, the empty one and the list by file at
/// the width the window has, each named with `suffix`.
async fn at_rest(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    suffix: &str,
) -> Result<(), Box<dyn Error>> {
    let split = browser.element(By::Css(".fui-editor-split")).await?;
    shot_of(
        browser,
        brand,
        theme,
        &format!("code-editor-draft{suffix}"),
        &split,
    )
    .await?;
    let empty = browser
        .element(By::Css("section.fui-card[aria-label='New mapping']"))
        .await?;
    shot_of(
        browser,
        brand,
        theme,
        &format!("code-editor-empty{suffix}"),
        &empty,
    )
    .await?;
    let by_file = browser
        .element(By::Css(
            "section.fui-card[aria-label='Diagnostics by file']",
        ))
        .await?;
    shot_of(
        browser,
        brand,
        theme,
        &format!("diagnostics-by-file{suffix}"),
        &by_file,
    )
    .await
}

/// The code editor and the list by file at 1440 and 390 px, and the code
/// editor with its error's diagnostic followed, its line and row current.
pub(super) async fn editor(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    forms(browser, gallery, brand).await?;
    at_rest(browser, brand, theme, "").await?;
    let error = browser
        .element(By::Css(
            ".fui-editor-split a.fui-diag-where[href='#gallery-mapping-L16']",
        ))
        .await?;
    press(browser, &error).await?;
    browser
        .element(By::Css("#gallery-mapping-L16.is-current"))
        .await?;
    let split = browser.element(By::Css(".fui-editor-split")).await?;
    shot_of(browser, brand, theme, "code-editor-current", &split).await?;
    window(browser, 390, HEIGHT).await?;
    forms(browser, gallery, brand).await?;
    at_rest(browser, brand, theme, "-390").await?;
    window(browser, WIDTH, HEIGHT).await
}
