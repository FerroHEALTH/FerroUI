// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The binder tabs on a phone, the toasts with their line and action, and
//! the popover form, driven in the browser.

use std::error::Error;

use serde_json::Value;
use thirtyfour::components::SelectElement;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::{hydrated_tabs, hydrated_toasts, window};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The bounding box of the element `id`: left, top, right and bottom.
async fn rect(browser: &Browser, id: &str) -> Result<[f64; 4], Box<dyn Error>> {
    let script = "const r = document.getElementById(arguments[0]).getBoundingClientRect(); return [r.left, r.top, r.right, r.bottom];";
    let found = browser
        .driver()
        .execute(script, vec![Value::from(id)])
        .await?;
    let sides: Vec<f64> = found.convert()?;
    Ok([sides[0], sides[1], sides[2], sides[3]])
}

/// The value the page's script reads for `expression`.
async fn read(browser: &Browser, expression: &str) -> Result<Value, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute(&format!("return {expression};"), Vec::new())
        .await?
        .json()
        .clone())
}

/// Dismisses every toast the region shows.
async fn dismiss_all(browser: &Browser) -> Result<(), Box<dyn Error>> {
    for close in browser
        .driver()
        .find_all(By::Css(".fui-toast > .fui-icon-button"))
        .await?
    {
        close.click().await?;
    }
    until("the toasts are gone", async || {
        Ok(browser
            .driver()
            .find_all(By::Css(".fui-toast"))
            .await?
            .is_empty())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_phone_shows_many_tabs_as_a_select_whose_go_button_leads_to_the_tab_chosen()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed")).await?;
        hydrated_tabs(browser).await?;
        let keyboard = browser
            .element(By::Css("a.fui-tab[data-tab='keyboard']"))
            .await?;
        assert!(
            keyboard.is_displayed().await?,
            "a wide screen shows Keyboard"
        );
        window(browser, 390, 900).await?;
        let select = browser.element(By::Id("fui-tabs-select")).await?;
        until("the select takes the row's place", async || {
            Ok(select.is_displayed().await? && select.attr("disabled").await?.is_none())
        })
        .await?;
        let row = browser
            .element(By::Css(".fui-tabbed-many > nav.fui-tabs"))
            .await?;
        assert!(
            !row.is_displayed().await?,
            "the row gives way to the select"
        );
        assert!(
            !keyboard.is_displayed().await?,
            "a phone has no keyboard, so no Keyboard tab"
        );
        // A mark on the window, which a navigation would clear.
        read(browser, "(window.fuiStayed = true)").await?;
        SelectElement::new(&select)
            .await?
            .select_by_value("?tab=commit")
            .await?;
        browser.settled().await?;
        assert_eq!(
            read(browser, "window.fuiStayed === true && !location.search").await?,
            Value::Bool(true),
            "choosing an option alone changes no page (WCAG 2.2 SC 3.2.2)"
        );
        browser
            .element(By::Css("form.fui-tabs-select button[type='submit']"))
            .await?
            .click()
            .await?;
        until("the Go button leads to the tab chosen", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .ends_with("?tab=commit"))
        })
        .await?;
        window(browser, 1440, 900).await?;
        browser.console_clean("the tab select").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_toast_with_an_action_waits_and_the_bad_tone_is_an_alert() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/overlays")).await?;
        hydrated_toasts(browser).await?;
        dismiss_all(browser).await?;
        browser
            .element(By::Css("button[data-toast='ok']"))
            .await?
            .click()
            .await?;
        let link = browser.element(By::LinkText("Open the tables")).await?;
        assert!(
            link.attr("href")
                .await?
                .unwrap_or_default()
                .ends_with("/ferrobridge/tables"),
            "the action leads to the tables page"
        );
        browser
            .element(By::Css("button[data-toast='info']"))
            .await?
            .click()
            .await?;
        browser
            .text("strong", "The set reloads in the background.")
            .await?;
        // WCAG 2.2 SC 2.2.1: the toast without an action goes on its own, the one
        // with an action waits for Dismiss.
        until("the information toast goes on its own", async || {
            Ok(browser
                .driver()
                .find_all(By::XPath(
                    "//strong[contains(., 'The set reloads in the background.')]",
                ))
                .await?
                .is_empty())
        })
        .await?;
        assert!(link.is_displayed().await?, "the toast with an action stays");
        // At 390 px the region sits at the foot and the page keeps clear of it.
        window(browser, 390, 800).await?;
        let region = browser.element(By::Css(".fui-toasts")).await?;
        let foot: f64 = read(
            browser,
            "document.querySelector('.fui-toasts').getBoundingClientRect().bottom",
        )
        .await?
        .as_f64()
        .unwrap_or_default();
        assert!(
            foot <= 800.0 && foot > 760.0,
            "the region docks to the foot: {foot}"
        );
        let kept = read(
            browser,
            "getComputedStyle(document.querySelector('.fui-main')).paddingBottom",
        )
        .await?;
        assert_eq!(Value::from("112px"), kept, "the page keeps its foot clear");
        assert!(region.is_displayed().await?);
        window(browser, 1440, 900).await?;
        dismiss_all(browser).await?;
        // WAI-ARIA 1.2, the alert role.
        browser
            .element(By::Css("button[data-toast='bad']"))
            .await?
            .click()
            .await?;
        let alert = browser.element(By::Css(".fui-toast[role='alert']")).await?;
        alert
            .find(By::XPath(".//button[normalize-space(.) = 'Try again']"))
            .await?
            .click()
            .await?;
        until("the action runs and dismisses its toast", async || {
            Ok(browser
                .driver()
                .find_all(By::Css(".fui-toast[role='alert']"))
                .await?
                .is_empty())
        })
        .await?;
        browser
            .text("strong", "Saved, and the set reloaded.")
            .await?;
        browser.console_clean("the toasts").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_popover_form_opens_under_its_trigger_or_as_a_sheet_and_escape_returns_the_focus()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        // A tall window leaves room under the trigger, so the form does not
        // flip above it.
        window(browser, 1440, 1600).await?;
        browser.goto(&gallery.url("/ferroehr/overlays")).await?;
        let trigger = browser
            .element(By::Css("button[popovertarget='save-query']"))
            .await?;
        browser
            .driver()
            .execute(
                "window.scrollBy(0, arguments[0].getBoundingClientRect().top - 200);",
                vec![trigger.to_json()?],
            )
            .await?;
        trigger.click().await?;
        let form = browser.element(By::Id("save-query")).await?;
        until("the popover form opens", async || {
            Ok(form.is_displayed().await?)
        })
        .await?;
        browser
            .driver()
            .execute(
                "arguments[0].id = 'save-query-trigger';",
                vec![trigger.to_json()?],
            )
            .await?;
        let [_, _, opener_right, opener_bottom] = rect(browser, "save-query-trigger").await?;
        let [_, top, right, _] = rect(browser, "save-query").await?;
        assert!(
            (top - opener_bottom - 4.0).abs() < 1.5,
            "4 px under its trigger: {top} against {opener_bottom}"
        );
        assert!(
            (right - opener_right).abs() < 1.5,
            "aligned to its trigger's end edge: {right} against {opener_right}"
        );
        browser
            .driver()
            .action_chain()
            .send_keys(Key::Escape)
            .perform()
            .await?;
        until("Escape closes it", async || {
            Ok(!form.is_displayed().await?)
        })
        .await?;
        let focused = read(browser, "document.activeElement.id").await?;
        assert_eq!(
            Value::from("save-query-trigger"),
            focused,
            "the focus returns to the trigger"
        );
        window(browser, 390, 800).await?;
        trigger.click().await?;
        until("the sheet opens", async || Ok(form.is_displayed().await?)).await?;
        let [left, _, right, bottom] = rect(browser, "save-query").await?;
        // The layout viewport, which leaves out a classic scrollbar.
        let wide = read(browser, "document.documentElement.clientWidth")
            .await?
            .as_f64()
            .unwrap_or_default();
        let high = read(browser, "document.documentElement.clientHeight")
            .await?
            .as_f64()
            .unwrap_or_default();
        assert!(
            (left - 12.0).abs() < 1.5 && (wide - right - 12.0).abs() < 1.5,
            "the sheet takes the content width: {left} to {right} of {wide}"
        );
        assert!(
            (high - bottom - 12.0).abs() < 1.5,
            "the sheet rises from the foot: {bottom} of {high}"
        );
        window(browser, 1440, 900).await?;
        browser.console_clean("the popover form").await
    })
    .await
}
