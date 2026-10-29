// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of navigation in the server-rendered gallery (FerroUI
//! #510): a link loads its page as one document and back and forward
//! restore the pages; a tab switch inside a page loads no document, draws
//! no fallback and moves no layout; a tree's level opens and closes in
//! place and its folds survive a reload; and a same-path navigation keeps
//! its scroll.

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

/// Whether the marker the journey set on the window survives, which it
/// does only while no new document loads.
async fn marked(browser: &Browser) -> Result<bool, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute("return window.fuiKept === true;", vec![])
        .await?
        .convert()?)
}

/// Sets the marker [`marked`] reads.
async fn mark(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute("window.fuiKept = true;", vec![])
        .await?;
    Ok(())
}

/// Follows the sidebar's link to `path` and waits for the page titled
/// `title`.
async fn follow(browser: &Browser, path: &str, title: &str) -> Result<(), Box<dyn Error>> {
    browser
        .element(By::Css(format!("#sidebar a[href='{path}']")))
        .await?
        .click()
        .await?;
    until(&format!("the link loads {title}"), async || {
        Ok(heading(browser).await? == title)
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_link_loads_its_page_and_back_and_forward_restore_the_pages() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        browser.hydrated().await?;
        let document = browser.loader().await?;
        follow(browser, "/ferrobridge/charts", "Charts").await?;
        assert_ne!(
            document,
            browser.loader().await?,
            "a link is one document the browser paints whole"
        );
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
        browser.console_clean("a link and the history").await
    })
    .await
}

/// Starts counting the layout shifts the page makes from now (Layout
/// Instability API, `layout-shift` entries with no recent input).
const WATCH_SHIFTS: &str = "window.fuiShift = 0;
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) if (!entry.hadRecentInput) window.fuiShift += entry.value;
    }).observe({ type: 'layout-shift' });";

#[tokio::test(flavor = "multi_thread")]
async fn a_tab_switch_inside_a_page_loads_no_document_draws_no_fallback_and_moves_no_layout()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge")).await?;
        browser.hydrated().await?;
        let document = browser.loader().await?;
        let navigations: i64 = browser
            .driver()
            .execute(
                "return performance.getEntriesByType('navigation').length;",
                vec![],
            )
            .await?
            .convert()?;
        mark(browser).await?;
        browser.driver().execute(WATCH_SHIFTS, vec![]).await?;
        browser
            .element(By::Id("answer-tab-1"))
            .await?
            .click()
            .await?;
        until("the second panel shows", async || {
            browser
                .element(By::Id("answer-panel-1"))
                .await?
                .is_displayed()
                .await
                .map_err(Into::into)
        })
        .await?;
        let text = browser
            .element(By::Id("answer-panel-1"))
            .await?
            .text()
            .await?;
        assert!(
            !text.trim().is_empty(),
            "the panel shows its words, no fallback"
        );
        assert!(
            !browser
                .element(By::Id("answer-panel-0"))
                .await?
                .is_displayed()
                .await?,
            "the first panel hides"
        );
        assert!(marked(browser).await?, "the switch loaded no document");
        assert_eq!(document, browser.loader().await?);
        let after: i64 = browser
            .driver()
            .execute(
                "return performance.getEntriesByType('navigation').length;",
                vec![],
            )
            .await?
            .convert()?;
        assert_eq!(navigations, after, "no navigation entry was added");
        let shift: f64 = browser
            .driver()
            .execute("return window.fuiShift;", vec![])
            .await?
            .convert()?;
        assert!(shift <= 0.0, "the switch moved no layout: {shift}");
        browser.console_clean("a tab switch in place").await
    })
    .await
}

/// Whether the tree item at the path key `key` of the tree `tree` is open.
async fn open(browser: &Browser, tree: &str, key: &str) -> Result<bool, Box<dyn Error>> {
    Ok(browser
        .element(By::Css(format!(
            "#{tree} li[role='treeitem'][data-key='{key}']"
        )))
        .await?
        .attr("aria-expanded")
        .await?
        .as_deref()
        == Some("true"))
}

/// Clicks the toggle of the tree item at the path key `key` of the tree
/// `tree`.
async fn toggle(browser: &Browser, tree: &str, key: &str) -> Result<(), Box<dyn Error>> {
    browser
        .element(By::Css(format!(
            "#{tree} li[role='treeitem'][data-key='{key}'] > .fui-tree-row > .fui-tree-toggle"
        )))
        .await?
        .click()
        .await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_tree_level_folds_in_place_and_its_folds_survive_a_reload() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroterm/trees")).await?;
        browser.hydrated().await?;
        // The long browser's chest level is read and open as the server draws it.
        let tree = "trees-long";
        let chest = "NF-100/NF-200";
        let document = browser.loader().await?;
        mark(browser).await?;
        assert!(open(browser, tree, chest).await?, "the level arrives open");
        toggle(browser, tree, chest).await?;
        until("the level closes", async || {
            Ok(!open(browser, tree, chest).await?)
        })
        .await?;
        assert!(marked(browser).await?, "the fold loaded no document");
        assert_eq!(document, browser.loader().await?);
        browser.driver().refresh().await?;
        browser.hydrated().await?;
        until("the reload keeps the level closed", async || {
            Ok(!open(browser, tree, chest).await?)
        })
        .await?;
        // The server drew the level, so it opens again in place.
        mark(browser).await?;
        toggle(browser, tree, chest).await?;
        until("the level opens in place", async || {
            open(browser, tree, chest).await
        })
        .await?;
        assert!(
            browser
                .element(By::Css(format!(
                    "#{tree} li[data-key='{chest}'] > ul[role='group']"
                )))
                .await?
                .is_displayed()
                .await?,
            "its children show"
        );
        assert!(marked(browser).await?, "the open loaded no document");
        browser.console_clean("a tree's folds").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_same_path_navigation_keeps_its_scroll() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge/overlays")).await?;
        browser.hydrated().await?;
        let link = browser.element(By::Css("a[data-toast='info']")).await?;
        link.scroll_into_view().await?;
        let before: f64 = browser
            .driver()
            .execute("return window.scrollY;", vec![])
            .await?
            .convert()?;
        assert!(before > 0.0, "the link sits below the fold");
        link.click().await?;
        browser
            .at(&gallery.url("/ferrobridge/overlays?toast=info"))
            .await?;
        browser.hydrated().await?;
        until("the page opens where it was", async || {
            let after: f64 = browser
                .driver()
                .execute("return window.scrollY;", vec![])
                .await?
                .convert()?;
            Ok((after - before).abs() < 2.0)
        })
        .await?;
        browser.console_clean("the scroll kept").await
    })
    .await
}
