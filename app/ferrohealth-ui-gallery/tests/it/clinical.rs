// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the clinical record parts: the condition builder keeps
//! inside the page between the phone layout and the wide one.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The document's client width, its scroll width and the right edge of
/// the widest condition row, under the classic 15 px scrollbar Linux draws
/// (macOS hides its own).
async fn widths(browser: &Browser) -> Result<(f64, f64, f64), Box<dyn Error>> {
    let measured = browser
        .driver()
        .execute(
            r"const bar = new CSSStyleSheet();
            bar.replaceSync('html::-webkit-scrollbar { width: 15px; }');
            document.adoptedStyleSheets = [...document.adoptedStyleSheets, bar];
            const rows = [...document.querySelectorAll('.fui-cond')];
            const right = Math.max(...rows.map((row) => row.getBoundingClientRect().right));
            return [document.documentElement.clientWidth, document.documentElement.scrollWidth, right];",
            vec![],
        )
        .await?;
    Ok(serde_json::from_value(measured.json().clone())?)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_condition_builder_keeps_inside_the_page_at_tablet_widths() -> Result<(), Box<dyn Error>>
{
    // NOTE: WCAG 2.2 SC 1.4.10: a nested condition row stacks inside its group rather than
    // widen the page at 768 px less a classic scrollbar (#307).
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for width in [640, 768, 1024] {
            window(browser, width, 900).await?;
            browser.goto(&gallery.url("/ferropix/clinical")).await?;
            browser.element(By::Css(".fui-cond")).await?;
            let (client, scroll, right) = widths(browser).await?;
            assert!(
                scroll <= client && right <= client,
                "at {width} px the builder reaches {right} and the page {scroll} past {client}"
            );
        }
        browser.console_clean("the condition builder").await
    })
    .await
}
