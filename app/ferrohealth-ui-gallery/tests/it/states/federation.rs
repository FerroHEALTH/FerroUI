// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The federation page: the topology hovered, zoomed, with its quick look,
//! folded, large with its minimap and as its List, at the size of a real
//! federation folded and unfolded and as its List on a phone; the fan-out
//! outcome at a phone's width; the replica line on the self-description;
//! and the sidebar inside a node, beside the page and in the phone's
//! drawer.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, away, card, hover, press, shot_of, shot_of_view};
use crate::browser::{Browser, until};
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "fanout-partial-390",
    "replica-line",
    "sidebar-node",
    "sidebar-node-drawer",
    "topology-collapsed",
    "topology-hover",
    "topology-large",
    "topology-list",
    "topology-look",
    "topology-real",
    "topology-real-390",
    "topology-real-open",
    "topology-zoomed",
];

/// The topology's card on the page, after `query` loads.
async fn topology(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    query: &str,
) -> Result<WebElement, Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/federation{query}", brand.key())))
        .await?;
    browser.element(card("Topology")).await
}

/// Every state of the federation page, in `brand` and `theme`.
pub(super) async fn federation(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let map = topology(browser, gallery, brand, "").await?;
    let zoom = browser.element(By::Css("button[data-zoom='in']")).await?;
    until("the zoom cluster is enabled", async || {
        Ok(zoom.attr("disabled").await?.is_none())
    })
    .await?;
    let node = browser
        .element(By::Css(format!(
            "a.fui-tnode-link[href='/{}/federation/node-b']",
            brand.key()
        )))
        .await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![map.to_json()?],
        )
        .await?;
    hover(browser, &node).await?;
    shot_of(browser, brand, theme, "topology-hover", &map).await?;
    away(browser).await?;
    press(browser, &zoom).await?;
    press(browser, &zoom).await?;
    shot_of(browser, brand, theme, "topology-zoomed", &map).await?;
    for (query, state) in [
        ("?look=node-b&view=map", "topology-look"),
        ("?collapse=all", "topology-collapsed"),
        (
            "?large=1&minimap=on&view=map&collapse=org-01,org-02,org-03,org-04,org-06,org-07,org-09,org-10,org-11,org-12",
            "topology-large",
        ),
        ("?view=list", "topology-list"),
        (
            "?large=50&minimap=on&view=map&collapse=all",
            "topology-real",
        ),
        ("?large=50&minimap=on&view=map", "topology-real-open"),
    ] {
        let map = topology(browser, gallery, brand, query).await?;
        shot_of(browser, brand, theme, state, &map).await?;
    }
    browser
        .goto(&gallery.url(&format!("/{}/federation/self-description", brand.key())))
        .await?;
    browser.element(By::Css(".fui-replica-line")).await?;
    shot_of_view(browser, brand, theme, "replica-line").await?;
    browser
        .goto(&gallery.url(&format!("/{}/federation/node-a", brand.key())))
        .await?;
    browser.element(By::Css("#sidebar .fui-nav-tree")).await?;
    shot_of_view(browser, brand, theme, "sidebar-node").await?;
    window(browser, 390, HEIGHT).await?;
    browser
        .goto(&gallery.url(&format!("/{}/federation/node-a", brand.key())))
        .await?;
    let open = browser
        .element(By::XPath("//button[@aria-label = 'Menu']"))
        .await?;
    open.click().await?;
    let drawer = browser.element(By::Id("sidebar")).await?;
    until("the drawer opens", async || {
        Ok(drawer.is_displayed().await?)
    })
    .await?;
    shot_of_view(browser, brand, theme, "sidebar-node-drawer").await?;
    browser
        .goto(&gallery.url(&format!("/{}/federation", brand.key())))
        .await?;
    let partial = browser.element(card("One node failing")).await?;
    shot_of(browser, brand, theme, "fanout-partial-390", &partial).await?;
    let real = topology(browser, gallery, brand, "?large=50&collapse=all").await?;
    shot_of(browser, brand, theme, "topology-real-390", &real).await?;
    window(browser, WIDTH, HEIGHT).await
}
