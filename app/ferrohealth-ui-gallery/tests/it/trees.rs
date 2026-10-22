// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the trees page: the concept browser's tree walked and
//! folded by the keyboard, an unread level read by the server, the folder
//! tree in an edit, and the path tree following a node's link.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The row text of the tree item that holds the focus, or `None` when the
/// focus is elsewhere.
async fn focused_row(browser: &Browser) -> Result<Option<String>, Box<dyn Error>> {
    let active = browser.driver().active_element().await?;
    if active.attr("role").await?.as_deref() != Some("treeitem") {
        return Ok(None);
    }
    Ok(Some(
        active.find(By::Css(".fui-tree-row")).await?.text().await?,
    ))
}

/// Presses `key` on the element that holds the focus.
async fn press(browser: &Browser, key: Key) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .active_element()
        .await?
        .send_keys(key)
        .await?;
    Ok(())
}

/// Focuses the item `selector` of a tree until `key` moves the focus to the
/// row that starts with `words`, which also waits for the island to
/// hydrate.
async fn walk_from(
    browser: &Browser,
    selector: &str,
    key: Key,
    words: &str,
) -> Result<(), Box<dyn Error>> {
    until("the tree answers the keyboard", async || {
        let item = browser.element(By::Css(selector)).await?;
        browser
            .driver()
            .execute("arguments[0].focus();", vec![item.to_json()?])
            .await?;
        item.send_keys(key.clone()).await?;
        Ok(focused_row(browser)
            .await?
            .is_some_and(|text| text.starts_with(words)))
    })
    .await
}

/// Whether the tree item whose row starts with `words` in the tree `tree`
/// is open.
async fn open_item(browser: &Browser, tree: &str, words: &str) -> Result<bool, Box<dyn Error>> {
    let item = browser
        .element(By::XPath(format!(
            "//ul[@id = '{tree}']//li[@role = 'treeitem'][starts-with(normalize-space(div[contains(@class, 'fui-tree-row')]), '{words}')]"
        )))
        .await?;
    Ok(item.attr("aria-expanded").await?.as_deref() == Some("true"))
}

#[tokio::test(flavor = "multi_thread")]
async fn the_concept_tree_folds_in_place_and_reads_an_unread_level_from_the_server()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        let page = gallery.url("/ferroterm/trees");
        browser.goto(&page).await?;
        let stops = browser
            .driver()
            .find_all(By::Css("#trees-deep [tabindex='0']"))
            .await?;
        assert_eq!(1, stops.len(), "one tab stop");
        // The selected concept holds the tab stop; Up walks to its parent.
        walk_from(browser, "#trees-deep [tabindex='0']", Key::Up, "Cough").await?;
        press(browser, Key::Left).await?;
        until("Left closes the open item in place", async || {
            Ok(!open_item(browser, "trees-deep", "Cough").await?)
        })
        .await?;
        assert!(
            browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .ends_with("/ferroterm/trees"),
            "a read level folds with no page load"
        );
        press(browser, Key::Right).await?;
        until("Right opens it again", async || {
            open_item(browser, "trees-deep", "Cough").await
        })
        .await?;
        // Home, then Down to the skin, an unread level: Right reads it on the server.
        press(browser, Key::Home).await?;
        press(browser, Key::Down).await?;
        until("Down reaches the unread level", async || {
            Ok(focused_row(browser)
                .await?
                .is_some_and(|text| text.starts_with("Finding of the skin")))
        })
        .await?;
        press(browser, Key::Right).await?;
        until("the server reads the level", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("SF-1100"))
        })
        .await?;
        browser.text("a", "Rash").await?;
        assert!(open_item(browser, "trees-deep", "Finding of the skin").await?);
        browser.console_clean("the concept tree").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_folder_tree_walks_past_its_actions_and_the_path_tree_follows_a_node()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr/trees")).await?;
        // The edit's root item: Down walks to Episodes, Left closes it.
        walk_from(
            browser,
            "#trees-directory-edit > li",
            Key::Down,
            "Episodes",
        )
        .await?;
        press(browser, Key::Left).await?;
        until("Left closes the folder", async || {
            Ok(!open_item(browser, "trees-directory-edit", "Episodes").await?)
        })
        .await?;
        press(browser, Key::Right).await?;
        until("Right opens it again", async || {
            open_item(browser, "trees-directory-edit", "Episodes").await
        })
        .await?;
        // Typing in the rename field moves no tree focus.
        let field = browser.element(By::Id("trees-directory-edit-rename")).await?;
        field.click().await?;
        field.send_keys(Key::Left).await?;
        assert_eq!(
            Some(String::from("trees-directory-edit-rename")),
            browser.driver().active_element().await?.attr("id").await?,
            "the arrow keys stay in the field"
        );
        assert!(open_item(browser, "trees-directory-edit", "Episodes").await?);
        browser
            .element(By::Css(
                "#trees-directory-edit button[name='remove'][aria-label='Remove the reference Vital signs']",
            ))
            .await?;
        // In the path tree, Enter follows the selected node's link.
        walk_from(browser, "#trees-paths [tabindex='0']", Key::Up, "Diastolic").await?;
        press(browser, Key::Enter).await?;
        until("Enter follows the node's link", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .ends_with("#diastolic"))
        })
        .await?;
        browser.console_clean("the folder and path trees").await
    })
    .await
}
