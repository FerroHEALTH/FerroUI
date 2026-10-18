// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys: every interactive component driven in the browser, every
//! page of every brand loaded with no console error, the narrow screen's
//! drawer, and the 320 px reflow.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The pages of a brand, after its key.
pub(crate) const PAGES: [&str; 4] = ["", "/forms", "/tables", "/code"];

/// The sign-in variants.
pub(crate) const SIGN_INS: [&str; 3] = ["basic", "provider", "both"];

/// Lays the page out `width` CSS pixels wide and `height` tall.
pub(crate) async fn window(
    browser: &Browser,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .cdp()
        .send_raw(
            "Emulation.setDeviceMetricsOverride",
            json!({ "width": width, "height": height, "deviceScaleFactor": 1, "mobile": false }),
        )
        .await?;
    Ok(())
}

/// Waits until the overview has hydrated: its keyboard tabs answer an
/// arrow key.
pub(crate) async fn hydrated_tabs(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let first = browser.element(By::Id("answer-tab-0")).await?;
    until("the tabs answer the keyboard", async || {
        first.click().await?;
        first.send_keys(Key::Right).await?;
        let second = browser.driver().find(By::Id("answer-tab-1")).await?;
        Ok(second.attr("aria-selected").await?.as_deref() == Some("true"))
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn every_page_of_every_brand_loads_with_no_console_error() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/")).await?;
        browser.text("p", "Choose a brand").await?;
        browser.console_clean("the front page").await?;
        for brand in Brand::ALL {
            for page in PAGES {
                let path = format!("/{}{page}", brand.key());
                browser.goto(&gallery.url(&path)).await?;
                browser.element(By::Css("main h1")).await?;
                browser.console_clean(&path).await?;
            }
            for variant in SIGN_INS {
                let path = format!("/sign-in/{}/{variant}", brand.key());
                browser.goto(&gallery.url(&path)).await?;
                browser.element(By::Css(".fui-sign-in")).await?;
                browser.console_clean(&path).await?;
            }
        }
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_keyboard_tabs_follow_the_arrows_home_and_end() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed")).await?;
        hydrated_tabs(browser).await?;
        let second = browser.element(By::Id("answer-tab-1")).await?;
        second.send_keys(Key::End).await?;
        let last = browser.element(By::Id("answer-tab-2")).await?;
        until("End selects the last tab", async || {
            Ok(last.attr("aria-selected").await?.as_deref() == Some("true"))
        })
        .await?;
        assert!(
            browser
                .element(By::Id("answer-panel-2"))
                .await?
                .is_displayed()
                .await?
        );
        assert!(
            !browser
                .element(By::Id("answer-panel-0"))
                .await?
                .is_displayed()
                .await?
        );
        last.send_keys(Key::Right).await?;
        let first = browser.element(By::Id("answer-tab-0")).await?;
        until("Right wraps to the first tab", async || {
            Ok(first.attr("aria-selected").await?.as_deref() == Some("true"))
        })
        .await?;
        browser.console_clean("the tabs").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_dialog_confirms_and_a_toast_says_so() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr")).await?;
        hydrated_tabs(browser).await?;
        browser
            .element(By::XPath(
                "//button[normalize-space(.) = 'Delete the file']",
            ))
            .await?
            .click()
            .await?;
        let dialog = browser.element(By::Id("delete-file")).await?;
        until("the dialog opens", async || {
            Ok(dialog.is_displayed().await?)
        })
        .await?;
        dialog
            .find(By::XPath(".//button[normalize-space(.) = 'Delete']"))
            .await?
            .click()
            .await?;
        browser.text("p", "The synthetic file was deleted.").await?;
        until("the dialog closes", async || {
            Ok(!dialog.is_displayed().await?)
        })
        .await?;
        browser.console_clean("the dialog").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_keeps_every_byte_typed_and_a_diagnostic_leads_to_its_line()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let draft = browser.element(By::Id("gallery-draft")).await?;
        until("the editor may be edited", async || {
            Ok(draft.attr("readonly").await?.is_none())
        })
        .await?;
        draft.clear().await?;
        let expected = format!(
            "{}# typed: naïve ✓ <b>&\n",
            ferrohealth_ui_gallery::data::YAML
        );
        draft.send_keys(&expected).await?;
        until("the text area holds every byte typed", async || {
            Ok(draft.prop("value").await?.as_deref() == Some(expected.as_str()))
        })
        .await?;
        browser
            .text("p", &format!("{} bytes", expected.len()))
            .await?;
        let layer = browser.element(By::Css(".fui-editor-layer")).await?;
        assert!(
            layer.text().await?.contains("# typed: naïve ✓ <b>&"),
            "the layer draws what was typed"
        );
        browser.follow("line 13").await?;
        let url = browser.driver().current_url().await?;
        assert_eq!(Some("gallery-draft-L13"), url.fragment(), "{url}");
        browser
            .element(By::Css("#gallery-draft-L13.fui-mark-error"))
            .await?;
        browser.console_clean("the editor").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_url_pager_and_sort_are_links_and_the_session_pager_stays_out_of_the_url()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        browser.text("p", "1-25 of 137").await?;
        let url_rows = By::XPath("(//table)[1]/tbody/tr");
        assert_eq!(25, browser.driver().find_all(url_rows).await?.len());
        // The label names what a reader sees: the box shows the whole table.
        let region = browser.element(By::Css(".fui-table-scroll")).await?;
        let table = region.find(By::Css("table")).await?;
        assert!(
            region.rect().await?.height >= table.rect().await?.height,
            "the table box clips its rows"
        );
        let next = By::XPath(
            "//nav[@aria-label = 'Pages of the mapping files']//a[contains(., 'Next')]",
        );
        browser.element(next).await?.click().await?;
        until("the page is in the URL", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("page=2"))
        })
        .await?;
        browser.text("p", "26-50 of 137").await?;
        browser
            .element(By::XPath("//th[contains(., 'Lines')]//a"))
            .await?
            .click()
            .await?;
        until("the sort is in the URL", async || {
            let url = browser.driver().current_url().await?;
            Ok(url.as_str().contains("sort=lines") && !url.as_str().contains("page="))
        })
        .await?;
        browser
            .element(By::XPath(
                "//th[@aria-sort = 'ascending' and contains(., 'Lines')]",
            ))
            .await?;
        let before = browser.driver().current_url().await?;
        let last = By::XPath(
            "//nav[@aria-label = 'Pages of the mapping files, in the page']//button[contains(., 'Last')]",
        );
        until("the session pager moves to the last page", async || {
            browser.driver().find(last.clone()).await?.click().await?;
            Ok(!browser
                .driver()
                .find_all(By::XPath("//p[contains(., '126-137 of 137')]"))
                .await?
                .is_empty())
        })
        .await?;
        let session_rows = By::XPath("(//table)[2]/tbody/tr");
        assert_eq!(12, browser.driver().find_all(session_rows).await?.len());
        assert_eq!(
            before,
            browser.driver().current_url().await?,
            "the session pager puts no position in the URL"
        );
        browser.console_clean("the pagers").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_theme_switch_posts_and_the_page_returns_in_the_theme() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed/code")).await?;
        browser
            .element(By::XPath("//button[@name = 'theme' and @value = 'dark']"))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css(r#"html[data-theme="dark"]"#))
            .await?;
        browser.at(&gallery.url("/ferrofed/code")).await?;
        browser
            .element(By::XPath(
                "//button[@value = 'dark' and @aria-pressed = 'true']",
            ))
            .await?;
        browser.console_clean("the theme switch").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_drawer_opens_takes_the_focus_and_closes_on_a_link() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 600, 900).await?;
        browser.goto(&gallery.url("/ferrobridge")).await?;
        hydrated_tabs(browser).await?;
        browser
            .element(By::XPath("//button[normalize-space(.) = 'Menu']"))
            .await?
            .click()
            .await?;
        let nav = browser.element(By::Id("sidebar")).await?;
        until("the drawer opens", async || Ok(nav.is_displayed().await?)).await?;
        until("the focus is in the drawer", async || {
            let active = browser.driver().active_element().await?;
            Ok(active.text().await? == "Overview")
        })
        .await?;
        browser.follow("Code panels").await?;
        browser.at(&gallery.url("/ferrobridge/code")).await?;
        until("the drawer closes", async || {
            Ok(!browser
                .driver()
                .find(By::Id("sidebar"))
                .await?
                .is_displayed()
                .await?)
        })
        .await?;
        browser.console_clean("the drawer").await
    })
    .await
}

/// The part of `Page.getLayoutMetrics` the reflow check reads.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    /// The size of the page's content.
    css_content_size: Size,
    /// The layout viewport.
    css_layout_viewport: Viewport,
}

/// A width.
#[derive(Debug, serde::Deserialize)]
struct Size {
    /// The width, in CSS pixels.
    width: f64,
}

/// The layout viewport.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Viewport {
    /// Its width, in CSS pixels.
    client_width: f64,
}

/// A numeric DOM property as WebDriver returns it, or 0 when absent.
fn number(value: Option<String>) -> f64 {
    value.and_then(|text| text.parse().ok()).unwrap_or(0.0)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_table_header_sticks_below_the_top_bar_on_a_wide_screen() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrobridge/tables?size=100"))
            .await?;
        browser.text("p", "1-100 of 137").await?;
        let rows = browser
            .driver()
            .find_all(By::XPath("(//table)[1]/tbody/tr"))
            .await?;
        assert_eq!(100, rows.len());
        let row = rows.get(60).ok_or("no 61st row")?;
        row.scroll_into_view().await?;
        let bar = browser
            .element(By::Css(".fui-topbar"))
            .await?
            .rect()
            .await?;
        let head = browser
            .element(By::XPath("(//table)[1]/thead//th[1]"))
            .await?
            .rect()
            .await?;
        // Rects are in page coordinates; the sticky bar's top is the scroll.
        assert!(bar.y > 100.0, "the page did not scroll ({})", bar.y);
        let below_bar = head.y - (bar.y + bar.height);
        assert!(
            (-1.0..=2.0).contains(&below_bar),
            "the header sits {below_bar} px from the top bar's foot"
        );
        browser.console_clean("the sticky header").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn every_page_reflows_at_320_pixels_with_no_sideways_scroll() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 320, 800).await?;
        for page in PAGES {
            let path = format!("/ferroehr{page}");
            browser.goto(&gallery.url(&path)).await?;
            browser.element(By::Css("main h1")).await?;
            let metrics: Metrics = serde_json::from_value(
                browser
                    .driver()
                    .cdp()
                    .send_raw("Page.getLayoutMetrics", json!({}))
                    .await?,
            )?;
            assert!(
                metrics.css_content_size.width <= metrics.css_layout_viewport.client_width + 0.5,
                "{path} scrolls sideways at 320 px: {metrics:?}"
            );
            if page == "/tables" {
                // The table, wider than its card, scrolls sideways inside it.
                let region = browser.element(By::Css(".fui-table-scroll")).await?;
                let scroll = number(region.prop("scrollWidth").await?);
                let client = number(region.prop("clientWidth").await?);
                assert!(
                    scroll > client,
                    "the table box does not scroll sideways at 320 px ({scroll} in {client})"
                );
            }
        }
        browser.console_clean("the reflow").await
    })
    .await
}
