// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The upload field empty, with a file over it, with a file read and with a
//! file refused, and the revealed secret's replace field.

use std::error::Error;
use std::path::PathBuf;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{card, press, shot_of};
use crate::browser::{Browser, until};
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "drop-empty",
    "drop-over",
    "drop-chosen",
    "drop-refused",
    "reveal-replace",
];

/// The synthetic files the upload field is given: one it reads and one it
/// refuses.
fn sample_files() -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let dir = std::env::temp_dir().join("ferroui-gallery-states");
    std::fs::create_dir_all(&dir)?;
    let read = dir.join("synthetic-vital-signs.opt");
    std::fs::write(
        &read,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<template>\n  <concept>Synthetic vital signs</concept>\n</template>\n",
    )?;
    let refused = dir.join("notes.txt");
    std::fs::write(&refused, "not a template\n")?;
    Ok((read, refused))
}

/// The upload field empty, with a file over it, with a file read and with a
/// file refused.
pub(super) async fn upload(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let (read, refused) = sample_files()?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let grouped = browser.element(card("Grouped fields")).await?;
    let replace = browser
        .element(By::Css("label[for='secret-replace']"))
        .await?;
    press(browser, &replace).await?;
    browser.element(By::Id("member-secret")).await?;
    shot_of(browser, brand, theme, "reveal-replace", &grouped).await?;
    let field = browser.element(card("Upload")).await?;
    let zone = browser.element(By::Id("gallery-upload-drop")).await?;
    shot_of(browser, brand, theme, "drop-empty", &field).await?;
    until("the drop zone takes a file over it", async || {
        browser
            .driver()
            .execute(
                "arguments[0].dispatchEvent(new DragEvent('dragover', {bubbles: true, cancelable: true}));",
                vec![zone.to_json()?],
            )
            .await?;
        Ok(zone
            .attr("class")
            .await?
            .is_some_and(|class| class.contains("fui-drop-over")))
    })
    .await?;
    shot_of(browser, brand, theme, "drop-over", &field).await?;
    browser
        .driver()
        .execute(
            "arguments[0].dispatchEvent(new DragEvent('dragleave', {bubbles: true}));",
            vec![zone.to_json()?],
        )
        .await?;
    let input = browser.element(By::Id("gallery-upload-file")).await?;
    input.send_keys(read.to_string_lossy()).await?;
    browser.text("p", "synthetic-vital-signs.opt read").await?;
    shot_of(browser, brand, theme, "drop-chosen", &field).await?;
    input.send_keys(refused.to_string_lossy()).await?;
    browser.text("p", "notes.txt was not read").await?;
    // The refusal replaces the read's state in place; two frames let the
    // browser paint the new icon in the refusal's tone before the capture.
    browser
        .driver()
        .execute_async(
            "const done = arguments[arguments.length - 1];
            requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(done, 150)));",
            vec![],
        )
        .await?;
    shot_of(browser, brand, theme, "drop-refused", &field).await
}
