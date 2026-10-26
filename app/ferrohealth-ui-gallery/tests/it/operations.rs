// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The operations page laid out at a wide and at a phone width: the
//! transform pair side by side and stacked with Run across the row, the
//! load report's glyph beside its title, the OMOP row preview's columns
//! over their values, and the HL7 v2 viewer's field names dropped on a
//! phone. Every width is measured against the layout viewport, which a
//! classic scrollbar narrows.

use std::error::Error;

use thirtyfour::ElementRect;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The rectangle of the first element `css` names.
async fn rect(browser: &Browser, css: &str) -> Result<ElementRect, Box<dyn Error>> {
    Ok(browser.element(By::Css(css)).await?.rect().await?)
}

/// Whether `a` and `b` start within a pixel of each other.
fn level(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

#[tokio::test(flavor = "multi_thread")]
async fn the_pair_stands_side_by_side_wide_and_stacks_on_a_phone_with_run_across_the_row()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrobridge/operations"))
            .await?;
        browser.element(By::Css("main h1")).await?;
        // The measures wait for the layout to hold over two frames, fonts loaded.
        browser.settled().await?;
        let input = rect(browser, ".fui-transform > .fui-transform-pane:first-child").await?;
        let answer = rect(browser, ".fui-transform > .fui-transform-pane:last-child").await?;
        assert!(level(input.y, answer.y), "side by side at 1440 px: {input:?} {answer:?}");
        assert!(answer.x > input.x + input.width - 2.0, "{input:?} {answer:?}");
        let name = browser.element(By::Css(".fui-hl7-fields .fui-hl7-name")).await?;
        assert!(name.is_displayed().await?, "a field's name shows at 1440 px");
        let column = rect(browser, ".fui-rowgraph-fields dt").await?;
        let value = rect(browser, ".fui-rowgraph-fields dd").await?;
        assert!(level(column.y, value.y), "a column beside its value: {column:?} {value:?}");

        window(browser, 390, 900).await?;
        browser
            .goto(&gallery.url("/ferrobridge/operations"))
            .await?;
        browser.element(By::Css("main h1")).await?;
        browser.settled().await?;
        let input = rect(browser, ".fui-transform > .fui-transform-pane:first-child").await?;
        let answer = rect(browser, ".fui-transform > .fui-transform-pane:last-child").await?;
        assert!(level(input.x, answer.x), "stacked at 390 px: {input:?} {answer:?}");
        assert!(answer.y >= input.y + input.height - 1.0, "{input:?} {answer:?}");
        // NOTE: CSSOM View, `clientWidth`: a classic scrollbar narrows the root's client box.
        let [run, inner, right, client]: [f64; 4] = browser
            .driver()
            .execute(
                "const row = document.querySelector('.fui-transform-options');
                 const run = row.querySelector('.fui-transform-run .fui-button');
                 const style = getComputedStyle(row);
                 return [run.getBoundingClientRect().width,
                         row.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight),
                         row.getBoundingClientRect().right, document.documentElement.clientWidth];",
                Vec::new(),
            )
            .await?
            .convert()?;
        assert!(level(run, inner), "Run spans the options row on a phone ({run} of {inner})");
        assert!(right <= client + 0.5, "the row fits the layout viewport ({right} in {client})");
        let glyph = rect(browser, ".fui-load-report > .fui-icon").await?;
        let title = rect(browser, ".fui-load-report > .fui-load-report-title").await?;
        assert!(
            glyph.x < title.x && glyph.y < title.y + title.height,
            "the glyph keeps its line beside the title: {glyph:?} {title:?}"
        );
        let column = rect(browser, ".fui-rowgraph-fields dt").await?;
        let value = rect(browser, ".fui-rowgraph-fields dd").await?;
        assert!(level(column.x, value.x), "a column over its value: {column:?} {value:?}");
        assert!(value.y >= column.y + column.height - 1.0, "{column:?} {value:?}");
        let name = browser.element(By::Css(".fui-hl7-fields .fui-hl7-name")).await?;
        assert!(!name.is_displayed().await?, "a field's name drops on a phone");
        browser.console_clean("the operations page").await
    })
    .await
}

/// The last line the editor `pair-running-input`'s layer draws, which follows
/// the console's text.
async fn drawn_last_line(browser: &Browser) -> Result<String, Box<dyn Error>> {
    let line: String = browser
        .driver()
        .execute(
            "const layer = document.getElementById('pair-running-input').previousElementSibling;
             return layer.lastElementChild.textContent.trimEnd();",
            Vec::new(),
        )
        .await?
        .convert()?;
    Ok(line)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_keeps_a_key_typed_while_its_store_answers_an_earlier_one()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrobridge/operations"))
            .await?;
        let input = browser.element(By::Id("pair-running-input")).await?;
        until("the editor hydrates", async || {
            browser.editor_live(&input).await
        })
        .await?;
        let start = input.prop("value").await?.unwrap_or_default();
        input.click().await?;
        browser
            .driver()
            .execute(
                "const area = document.getElementById('pair-running-input');
                 area.setSelectionRange(area.value.length, area.value.length);",
                Vec::new(),
            )
            .await?;
        input.send_keys("XYZ").await?;
        // The store answers one edit at a time: once the layer draws the first, the rest wait.
        let last = start
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .trim_end()
            .to_owned();
        let first = format!("{last}X");
        until("the store answers the first key", async || {
            Ok(drawn_last_line(browser).await? == first)
        })
        .await?;
        input.send_keys("!").await?;
        until("the store answers the last key", async || {
            Ok(drawn_last_line(browser).await?.ends_with('!'))
        })
        .await?;
        assert_eq!(
            format!("{last}XYZ!"),
            drawn_last_line(browser).await?,
            "the console holds every key typed"
        );
        let expected = format!("{start}XYZ!");
        assert_eq!(
            Some(expected.clone()),
            input.prop("value").await?,
            "no key typed while the store answered was lost"
        );
        browser.console_clean("the editor and its late store").await
    })
    .await
}
