// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of navigation in the page (FerroUI #410): a link keeps the
//! document and the router draws the next page, back and forward restore
//! the pages, the new page takes the focus and is announced, and hovering a
//! link loads its route's code before the click.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The text of the page's heading.
async fn heading(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser.element(By::Css("main h1")).await?.text().await?)
}

/// Whether the marker the journey set on the window survives.
async fn marked(browser: &Browser) -> Result<bool, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute("return window.fuiKept === true;", vec![])
        .await?
        .convert()?)
}

/// Follows the sidebar's link to `path` and waits for the page titled
/// `title`.
async fn follow(browser: &Browser, path: &str, title: &str) -> Result<(), Box<dyn Error>> {
    browser
        .element(By::Css(format!("#sidebar a[href='{path}']")))
        .await?
        .click()
        .await?;
    until(&format!("the router draws {title}"), async || {
        Ok(heading(browser).await? == title)
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_link_keeps_the_document_and_back_and_forward_restore_the_pages()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        browser.hydrated().await?;
        let document = browser.loader().await?;
        browser
            .driver()
            .execute("window.fuiKept = true;", vec![])
            .await?;
        follow(browser, "/ferrobridge/charts", "Charts").await?;
        assert!(marked(browser).await?, "the link loaded no new document");
        // WCAG 2.2 SC 2.4.3 and SC 4.1.3: the new page takes the focus and is said.
        until("the new page's heading takes the focus", async || {
            Ok(browser
                .driver()
                .execute(
                    "return document.activeElement === document.querySelector('main h1');",
                    vec![],
                )
                .await?
                .convert::<bool>()?)
        })
        .await?;
        until("the region says the new page's title", async || {
            let said: String = browser
                .driver()
                .execute(
                    "return document.querySelector(\"[data-ferroui-announcer][role='status']\").textContent;",
                    vec![],
                )
                .await?
                .convert()?;
            Ok(said.starts_with("Charts"))
        })
        .await?;
        assert_eq!(document, browser.loader().await?);
        browser.at(&gallery.url("/ferrobridge/charts")).await?;
        browser
            .element(By::Css(
                "#sidebar a[href='/ferrobridge/charts'][aria-current='page']",
            ))
            .await?;
        follow(browser, "/ferrobridge/forms", "Forms and editor").await?;
        browser.driver().back().await?;
        until("back restores the charts", async || {
            Ok(heading(browser).await? == "Charts")
        })
        .await?;
        browser.driver().forward().await?;
        until("forward restores the forms", async || {
            Ok(heading(browser).await? == "Forms and editor")
        })
        .await?;
        assert!(marked(browser).await?, "back and forward load no document");
        browser.console_clean("navigation in the page").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn hovering_a_link_loads_its_routes_code_before_the_click() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroterm/tables")).await?;
        // The browser's resource timeline keeps 250 entries by default, fewer than the
        // chunks the idle prefetch loads.
        browser
            .driver()
            .execute("performance.setResourceTimingBufferSize(10000);", vec![])
            .await?;
        browser.hydrated().await?;
        let list = browser
            .element(By::Css("#sidebar a[href='/ferroterm/templates/list']"))
            .await?;
        // NOTE: the templates' route is in the group of pages aside, which the idle prefetch
        // leaves, so only the hover loads the chunks it alone needs; the split manifest
        // names them.
        let waiting: u64 = browser
            .driver()
            .execute_async(
                r"const done = arguments[arguments.length - 1];
                fetch('/pkg/__wasm_split_manifest.json').then((answer) => answer.json()).then((manifest) => {
                  const key = Object.keys(manifest).find((name) => name.includes('l_aside'));
                  const loaded = performance.getEntriesByType('resource').map((entry) => entry.name);
                  window.fuiWaiting = manifest[key].filter((chunk) => !loaded.some((name) => name.endsWith('/' + chunk + '.wasm')));
                  performance.clearResourceTimings();
                  done(window.fuiWaiting.length);
                });",
                vec![],
            )
            .await?
            .convert()?;
        assert!(waiting > 0, "the template page's own chunks wait for a reason to load");
        browser
            .driver()
            .action_chain()
            .move_to_element_center(&list)
            .perform()
            .await?;
        until("the hover loads the template page's chunks", async || {
            Ok(browser
                .driver()
                .execute(
                    r"const loaded = performance.getEntriesByType('resource').map((entry) => entry.name);
                    return window.fuiWaiting.every((chunk) => loaded.some((name) => name.endsWith('/' + chunk + '.wasm')));",
                    vec![],
                )
                .await?
                .convert::<bool>()?)
        })
        .await?;
        browser.console_clean("the prefetch").await
    })
    .await
}
