// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The federation page in a real browser: the topology's zoom by its
//! cluster and by the keys, the quick look on "i", the arrows along the
//! List twin, hovering that moves nothing, and the nested sidebar section
//! inside a node, with the rail leaving it out. Every width is measured
//! against the layout viewport, which a classic scrollbar narrows.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The readout of the zoom cluster.
async fn readout(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .element(By::Css("[data-zoom-readout]"))
        .await?
        .text()
        .await?
        .trim()
        .to_owned())
}

/// Waits until the topology's module runs: its Zoom in is enabled.
async fn hydrated(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let zoom = browser.element(By::Css("button[data-zoom='in']")).await?;
    until("the zoom cluster is enabled", async || {
        Ok(zoom.attr("disabled").await?.is_none())
    })
    .await
}

/// The width of the first drawing, as its attribute says.
async fn drawn_width(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .element(By::Css(".fui-pipeline-stage > svg"))
        .await?
        .attr("width")
        .await?
        .unwrap_or_default())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_map_zooms_by_its_cluster_and_by_the_keys_and_fits_again() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrofed/federation")).await?;
        hydrated(browser).await?;
        let natural = drawn_width(browser).await?;
        let before = readout(browser).await?;
        browser
            .element(By::Css("button[data-zoom='in']"))
            .await?
            .click()
            .await?;
        let zoomed = drawn_width(browser).await?;
        assert_ne!(natural, zoomed, "Zoom in widens the drawing");
        let icon = browser
            .element(By::Css(".fui-tnode-glyph svg"))
            .await?
            .attr("width")
            .await?;
        assert_eq!(Some(String::from("18")), icon, "a card's glyph keeps its size");
        let after = readout(browser).await?;
        assert!(after.ends_with('%'), "{after}");
        assert_ne!(before, after, "the readout follows the zoom");
        let canvas = browser.element(By::Css(".fui-topology-scroll")).await?;
        canvas.focus().await?;
        canvas.send_keys("+").await?;
        let further = drawn_width(browser).await?;
        assert_ne!(zoomed, further, "+ zooms in again");
        canvas.send_keys("0").await?;
        assert_eq!(natural, drawn_width(browser).await?, "0 fits the drawing again");
        let wide = browser
            .driver()
            .execute(
                "return document.documentElement.scrollWidth <= document.documentElement.clientWidth;",
                Vec::new(),
            )
            .await?;
        assert_eq!(
            Some(true),
            wide.json().as_bool(),
            "a zoomed map scrolls inside its canvas, never the page"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn i_on_a_card_opens_its_quick_look_and_hovering_moves_nothing() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrofed/federation")).await?;
        hydrated(browser).await?;
        let card = browser
            .element(By::Css(
                "a.fui-tnode-link[href='/ferrofed/federation/node-b']",
            ))
            .await?;
        browser
            .driver()
            .execute(
                "arguments[0].scrollIntoView({block: 'center'});",
                vec![card.to_json()?],
            )
            .await?;
        let before = card.rect().await?;
        until("hovering a card raises its path", async || {
            browser
                .driver()
                .action_chain()
                .move_to_element_center(&card)
                .perform()
                .await?;
            let raised = browser
                .driver()
                .execute(
                    "return getComputedStyle(arguments[0].closest('.fui-pipeline-card').querySelector('.fui-pipeline-path')).visibility;",
                    vec![card.to_json()?],
                )
                .await?;
            Ok(raised.json().as_str() == Some("visible"))
        })
        .await?;
        let after = card.rect().await?;
        assert!(
            (before.x - after.x).abs() < 0.5 && (before.y - after.y).abs() < 0.5,
            "hovering a card moves nothing"
        );
        card.focus().await?;
        card.send_keys("i").await?;
        browser.element(By::Css("aside.fui-topology-panel")).await?;
        assert!(
            browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("look=node-b")
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_arrows_walk_the_list_twin() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrofed/federation?view=list"))
            .await?;
        hydrated(browser).await?;
        let first = browser
            .element(By::Css(".fui-topology-list .fui-tree-name > a"))
            .await?;
        first.focus().await?;
        first.send_keys(Key::Down).await?;
        let active = browser.driver().active_element().await?;
        assert_eq!(
            Some(String::from("/ferrofed/federation/self-description")),
            active.attr("href").await?,
            "Down moves to the next row"
        );
        active.send_keys(Key::End).await?;
        let last = browser.driver().active_element().await?;
        assert_eq!(
            Some(String::from("/ferrofed/federation/node-e")),
            last.attr("href").await?
        );
        last.send_keys(Key::Left).await?;
        let parent = browser.driver().active_element().await?;
        assert_eq!(
            Some(String::from("/ferrofed/federation/testmeer")),
            parent.attr("href").await?,
            "Left moves to the parent"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn inside_a_node_the_sidebar_nests_its_organisation_and_the_rail_leaves_it_out()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrofed/federation/node-a"))
            .await?;
        let tree = browser.element(By::Css("#sidebar .fui-nav-tree")).await?;
        assert!(tree.is_displayed().await?);
        let here = browser
            .element(By::Css("#sidebar a[aria-current='page']"))
            .await?;
        assert_eq!(
            Some(String::from("/ferrofed/federation/node-a")),
            here.attr("href").await?
        );
        let mut cookie = Cookie::new("gallery-sidebar", "rail");
        cookie.set_path("/");
        browser.driver().add_cookie(cookie).await?;
        browser
            .goto(&gallery.url("/ferrofed/federation/node-a"))
            .await?;
        let folded = browser.element(By::Css("#sidebar .fui-nav-tree")).await?;
        assert!(!folded.is_displayed().await?, "the rail leaves it out");
        browser.driver().delete_cookie("gallery-sidebar").await?;
        Ok(())
    })
    .await
}
