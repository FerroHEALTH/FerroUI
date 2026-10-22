// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The terminology page's journeys: the FHIR version select in the top bar
//! and its keyboard path, the same choices in the phone's user menu, the
//! repeatable rows adding and removing in place with the focus moved on,
//! and the page at a phone width: Remove on a row's first line, the root
//! matrix scrolling inside its card with its first column in place, and a
//! mapping's parts in reading order. Every width is measured against the
//! layout viewport, which a classic scrollbar narrows.

use std::error::Error;

use thirtyfour::ElementRect;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::menus::hydrated_trigger;
use crate::stack::Gallery;

/// The page, in the brand whose console reads through FHIR versions.
const PAGE: &str = "/ferroterm/terminology";

/// The rectangle of the first element `css` names.
async fn rect(browser: &Browser, css: &str) -> Result<ElementRect, Box<dyn Error>> {
    Ok(browser.element(By::Css(css)).await?.rect().await?)
}

/// Presses `key` on whatever holds the focus.
async fn press(browser: &Browser, key: Key) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(key)
        .perform()
        .await?;
    Ok(())
}

/// The `id` of the element that holds the focus, or its `href`, or its
/// `data-repeat`, whichever it has first.
async fn focused(browser: &Browser) -> Result<String, Box<dyn Error>> {
    let active = browser.driver().active_element().await?;
    for name in ["id", "href", "data-repeat"] {
        if let Some(value) = active.attr(name).await? {
            return Ok(value);
        }
    }
    Ok(String::new())
}

/// Whether the page scrolls sideways in the layout viewport.
async fn page_scrolls_sideways(browser: &Browser) -> Result<bool, Box<dyn Error>> {
    // NOTE: CSSOM View, `clientWidth`: a classic scrollbar narrows the root's client box.
    let wider: bool = browser
        .driver()
        .execute(
            "const root = document.documentElement; return root.scrollWidth > root.clientWidth + 1;",
            Vec::new(),
        )
        .await?
        .convert()?;
    Ok(wider)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_version_select_names_itself_walks_by_the_keyboard_and_returns_the_focus()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url(PAGE)).await?;
        let select = hydrated_trigger(browser, "fhir-version").await?;
        let name = "FHIR version: R4, FHIR 4.0.1";
        assert_eq!(Some(String::from(name)), select.attr("aria-label").await?);
        assert_eq!(Some(String::from(name)), select.attr("title").await?);
        assert_eq!("R4", select.text().await?.trim(), "the version alone");
        browser
            .driver()
            .execute("arguments[0].focus();", vec![select.to_json()?])
            .await?;
        press(browser, Key::Down).await?;
        until("Down opens the menu on its first root", async || {
            Ok(focused(browser).await?.ends_with("?fhir=R4"))
        })
        .await?;
        press(browser, Key::Down).await?;
        assert!(focused(browser).await?.ends_with("?fhir=R4B"));
        press(browser, Key::End).await?;
        assert!(focused(browser).await?.ends_with("?fhir=R6"));
        press(browser, Key::Down).await?;
        assert!(focused(browser).await?.ends_with("?fhir=R4"), "wraps");
        let checked = browser
            .element(By::Css(
                "#fhir-version a[role='menuitemradio'][aria-checked='true']",
            ))
            .await?;
        assert!(
            checked.text().await?.contains("FHIR 4.0.1 · default"),
            "the root in view is checked and says it is the default"
        );
        press(browser, Key::Escape).await?;
        until("Escape returns the focus to the select", async || {
            let active = browser.driver().active_element().await?;
            Ok(active.attr("aria-controls").await?.as_deref() == Some("fhir-version"))
        })
        .await?;
        select.click().await?;
        browser
            .element(By::Css("#fhir-version a[href$='?fhir=R5']"))
            .await?
            .click()
            .await?;
        until("choosing R5 reads through it", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .ends_with("?fhir=R5"))
        })
        .await?;
        let select = hydrated_trigger(browser, "fhir-version").await?;
        assert_eq!("R5", select.text().await?.trim());
        browser.console_clean("the version select").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn on_a_phone_the_version_leaves_the_top_bar_for_the_user_menu() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url(PAGE)).await?;
        let select = browser
            .element(By::Css("summary[aria-controls='fhir-version']"))
            .await?;
        assert!(
            !select.is_displayed().await?,
            "below 40 rem the select leaves the top bar"
        );
        browser
            .element(By::Css("button[popovertarget='user-menu']"))
            .await?
            .click()
            .await?;
        let group = browser
            .element(By::XPath(
                "//*[@id = 'user-menu']//div[@class = 'fui-menu-version'][div[normalize-space(.) = 'FHIR version']]",
            ))
            .await?;
        until("the menu shows the version group", async || {
            group.is_displayed().await.map_err(Into::into)
        })
        .await?;
        let chosen = group
            .find(By::Css("a[role='menuitemradio'][aria-checked='true']"))
            .await?;
        assert_eq!("R4", chosen.text().await?.trim());
        assert_eq!(
            4,
            group.find_all(By::Css("a[role='menuitemradio']")).await?.len(),
            "one segment per root"
        );
        browser.console_clean("the phone's version group").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_repeatable_row_adds_and_removes_in_place_and_moves_the_focus_on()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url(PAGE)).await?;
        hydrated_trigger(browser, "fhir-version").await?;
        let at = browser.driver().current_url().await?;
        let list = browser.element(By::Id("include-filters")).await?;
        let add = list.find(By::Css("[data-repeat='add']")).await?;
        add.click().await?;
        until(
            "Add inserts a row and focuses its first field",
            async || Ok(focused(browser).await? == "filter-new-1-property"),
        )
        .await?;
        assert_eq!(
            2,
            list.find_all(By::Css(".fui-repeat-item")).await?.len(),
            "the first row and the new one"
        );
        let label = list
            .find(By::Css("label[for='filter-new-1-value']"))
            .await?;
        assert_eq!(
            "Value",
            label.text().await?.trim(),
            "the new row's labels name its fields"
        );
        list.find(By::Css("[data-row='new-1'] [data-repeat='remove']"))
            .await?
            .click()
            .await?;
        until(
            "Remove of the last row moves the focus to the row before",
            async || Ok(focused(browser).await? == "filter-f1-property"),
        )
        .await?;
        list.find(By::Css("[data-row='f1'] [data-repeat='remove']"))
            .await?
            .click()
            .await?;
        until(
            "Remove of the only row moves the focus to Add",
            async || Ok(focused(browser).await? == "add"),
        )
        .await?;
        assert_eq!(
            0,
            list.find_all(By::Css(".fui-repeat-item")).await?.len(),
            "no row is left"
        );
        let empty = list.find(By::Css("[data-repeat='empty']")).await?;
        assert!(empty.is_displayed().await?, "an empty list says so");
        assert_eq!(at, browser.driver().current_url().await?, "nothing posted");
        browser.console_clean("the repeatable rows").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn on_a_phone_remove_keeps_its_line_the_matrix_scrolls_in_its_card_and_a_mapping_stacks()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 900).await?;
        browser.goto(&gallery.url(PAGE)).await?;
        browser.element(By::Css("main h1")).await?;
        assert!(!page_scrolls_sideways(browser).await?, "the page never scrolls sideways");
        let field = rect(browser, "#include-filters .fui-clause-filter > .fui-field").await?;
        let remove = rect(browser, "#include-filters .fui-clause-filter > [data-repeat='remove']")
            .await?;
        assert!(
            remove.y < field.y + field.height && remove.y + remove.height > field.y,
            "Remove stays on the row's first line: {field:?} {remove:?}"
        );
        assert!(remove.x > field.x + field.width - 1.0, "beside the first field");
        let [scrolled, first, region]: [f64; 3] = browser
            .driver()
            .execute(
                "const box = document.querySelector('.fui-root-matrix .fui-table-scroll');
                 box.scrollLeft = 200;
                 const first = box.querySelector('tbody th');
                 return [box.scrollLeft, first.getBoundingClientRect().left, box.getBoundingClientRect().left];",
                Vec::new(),
            )
            .await?
            .convert()?;
        assert!(scrolled > 0.0, "the matrix scrolls sideways inside its card");
        assert!(
            (first - region).abs() <= 1.0,
            "the first column stays in place ({first} at {region})"
        );
        assert!(!page_scrolls_sideways(browser).await?, "the page still does not");
        let tops: Vec<f64> = browser
            .driver()
            .execute(
                "const row = [...document.querySelectorAll('.fui-table tbody tr')]
                   .find(tr => tr.querySelector('.fui-relation-cell'));
                 return [row.querySelector('th'), row.querySelector('.fui-relation-cell'),
                         row.querySelector('td:nth-of-type(2)')]
                   .map(cell => cell.getBoundingClientRect().top);",
                Vec::new(),
            )
            .await?
            .convert()?;
        assert!(
            tops.windows(2).all(|pair| pair[0] < pair[1]),
            "source, relationship and target stack in reading order: {tops:?}"
        );
        browser.console_clean("the terminology page on a phone").await
    })
    .await
}
