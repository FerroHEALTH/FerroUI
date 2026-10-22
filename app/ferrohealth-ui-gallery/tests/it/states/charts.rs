// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page's figures: a flow's card hovered and focused, which
//! shows its path, and the run report, the flow's list and the score
//! breakdown on a phone.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use thirtyfour::prelude::*;

use super::{HEIGHT, WIDTH, away, card, focus, hover, shot_of};
use crate::browser::Browser;
use crate::journeys::window;
use crate::stack::Gallery;

/// The area's states, by the name their images take.
pub(super) const STATES: &[&str] = &[
    "flow-hover",
    "flow-focus",
    "flow-list-390",
    "run-report-390",
    "score-390",
];

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
    hover(browser, &cdr).await?;
    shot_of(browser, brand, theme, "flow-hover", &flow).await?;
    away(browser).await?;
    focus(browser, &cdr).await?;
    shot_of(browser, brand, theme, "flow-focus", &flow).await?;
    window(browser, 390, HEIGHT).await?;
    browser.goto(&page).await?;
    for (title, state) in [
        ("Flow with problem edges", "flow-list-390"),
        ("Run report", "run-report-390"),
        ("Score breakdown", "score-390"),
    ] {
        let section = browser.element(card(title)).await?;
        shot_of(browser, brand, theme, state, &section).await?;
    }
    window(browser, WIDTH, HEIGHT).await
}
