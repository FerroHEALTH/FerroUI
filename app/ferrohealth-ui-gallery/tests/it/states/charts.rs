// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page's figures: a flow's card hovered and focused, which
//! shows its path, the busy bridge folded and unfolded, and the run report,
//! the flows' lists and the score breakdown on a phone; the eight series
//! chart and the score among close thresholds on a phone, and the charts
//! and the forms under forced colours.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, away, card, focus, shot_of};
use crate::browser::Browser;
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "flow-busy",
    "flow-busy-390",
    "flow-busy-open",
    "flow-hover",
    "flow-focus",
    "flow-list-390",
    "run-report-390",
    "score-390",
    "many-series-390",
    "score-close-390",
    "forced-colours-charts",
    "forced-colours-forms",
];

/// Emulates forced colours, or ends it, in `theme`'s colour scheme with
/// reduced motion, as the capture runs.
async fn forced_colours(browser: &Browser, theme: &str, on: bool) -> Result<(), Box<dyn Error>> {
    let forced = if on { "active" } else { "none" };
    browser
        .driver()
        .cdp()
        .send_raw(
            "Emulation.setEmulatedMedia",
            json!({ "features": [
                { "name": "forced-colors", "value": forced },
                { "name": "prefers-color-scheme", "value": theme },
                { "name": "prefers-reduced-motion", "value": "reduce" },
            ] }),
        )
        .await?;
    Ok(())
}

/// The flow with problem edges, its CDR card hovered and then focused, and
/// at 390 px its list, the run report and the score breakdown.
pub(super) async fn charts(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let page = gallery.url(&format!("/{}/charts", brand.key()));
    window(browser, WIDTH, HEIGHT).await?;
    browser.goto(&page).await?;
    let flow = browser.element(card("Flow with problem edges")).await?;
    let cdr = flow.find(By::Css(".fui-tnode-link[href='#cdr']")).await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![flow.to_json()?],
        )
        .await?;
    browser.settled().await?;
    // A pointer move to an element can scroll the page and leave the card
    // from under the pointer, so the pointer goes to the card's centre in
    // the viewport, which scrolls nothing.
    let centre = browser
        .driver()
        .execute(
            "const box = arguments[0].getBoundingClientRect(); return [Math.round(box.x + box.width / 2), Math.round(box.y + box.height / 2)];",
            vec![cdr.to_json()?],
        )
        .await?;
    let (x, y): (i64, i64) = serde_json::from_value(centre.json().clone())?;
    browser
        .driver()
        .action_chain()
        .move_to(x, y)
        .perform()
        .await?;
    browser.settled().await?;
    let png = flow.screenshot_as_png().await?;
    super::write(brand, "flow-hover", theme, &png)?;
    away(browser).await?;
    focus(browser, &cdr).await?;
    shot_of(browser, brand, theme, "flow-focus", &flow).await?;
    for (query, state) in [
        ("", "flow-busy"),
        ("?open=callers,upstreams", "flow-busy-open"),
    ] {
        browser.goto(&format!("{page}{query}")).await?;
        let busy = browser.element(By::Id("busy-flow")).await?;
        shot_of(browser, brand, theme, state, &busy).await?;
    }
    window(browser, 390, HEIGHT).await?;
    browser.goto(&page).await?;
    let busy = browser.element(By::Id("busy-flow")).await?;
    shot_of(browser, brand, theme, "flow-busy-390", &busy).await?;
    for (title, state) in [
        ("Flow with problem edges", "flow-list-390"),
        ("Run report", "run-report-390"),
        ("Score breakdown", "score-390"),
        ("Line chart with eight series", "many-series-390"),
        ("Score with close thresholds", "score-close-390"),
    ] {
        let section = browser.element(card(title)).await?;
        shot_of(browser, brand, theme, state, &section).await?;
    }
    window(browser, WIDTH, HEIGHT).await?;
    forced_colours(browser, theme, true).await?;
    browser.goto(&page).await?;
    let lines = browser
        .element(card("Line chart with eight series"))
        .await?;
    shot_of(browser, brand, theme, "forced-colours-charts", &lines).await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let query = browser.element(card("A federated query")).await?;
    shot_of(browser, brand, theme, "forced-colours-forms", &query).await?;
    forced_colours(browser, theme, false).await
}
