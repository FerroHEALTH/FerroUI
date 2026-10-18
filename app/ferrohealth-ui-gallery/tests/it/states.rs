// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The interaction states, captured behind the `FERROUI_SCREENSHOTS` gate:
//! the pointer on a navigation entry, a table row, each button variant, a
//! linked card and a tab; the keyboard focus on a control, a navigation
//! entry and the keyboard tabs; every overlay open (the dialogs, the modal
//! a signal opens, the side panel, the operator menu, the
//! drawer); the toasts; the upload field empty, with a file over it, with a
//! file read and with a file refused; the editor with its diagnostics; the
//! help switch on, the picker open on its input, the tree walked by the
//! keyboard, the search box focused, and the tables in the compact density.
//!
//! Each state is written as `states/<brand>/<state>.png` and its `-dark`
//! twin under the screenshots folder, every brand in the light and the
//! dark theme, so the hover and focus treatment is looked at as often as
//! the pages are.

use std::error::Error;
use std::path::PathBuf;

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::stack::Gallery;

/// Every state, by the name its images take.
pub(crate) const STATES: [&str; 32] = [
    "hover-nav",
    "focus-nav",
    "hover-stat",
    "hover-primary",
    "hover-secondary",
    "hover-danger",
    "hover-destructive",
    "hover-ghost",
    "focus-button",
    "hover-tab",
    "focus-keyboard-tabs",
    "hover-row",
    "editor-diagnostic",
    "drop-empty",
    "drop-over",
    "drop-chosen",
    "drop-refused",
    "dialog-open",
    "dialog-danger",
    "modal-open",
    "modal-wide",
    "side-panel",
    "operator-menu",
    "focus-theme-toggle",
    "toasts",
    "drawer-open",
    "focus-search",
    "help-on",
    "picker-other",
    "tree-keyboard",
    "density-compact",
    "reveal-replace",
];

/// The viewport the states are taken in.
const WIDTH: u32 = 1440;
const HEIGHT: u32 = 900;

/// Where the image of `state` of `brand` in `theme` goes.
fn file(brand: Brand, state: &str, theme: &str) -> PathBuf {
    let suffix = if theme == "dark" { "-dark" } else { "" };
    crate::screenshots::root()
        .join(crate::screenshots::IMAGES)
        .join("states")
        .join(brand.key())
        .join(format!("{state}{suffix}.png"))
}

/// Writes `png` as the image of `state`.
fn write(brand: Brand, state: &str, theme: &str, png: &[u8]) -> Result<(), Box<dyn Error>> {
    let path = file(brand, state, theme);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, png)?;
    Ok(())
}

/// The card whose title is `title`.
fn card(title: &str) -> By {
    By::XPath(format!(
        "//section[contains(@class, 'fui-card')][h2[normalize-space(.) = '{title}'] or div/h2[normalize-space(.) = '{title}']]"
    ))
}

/// Moves the pointer onto `element`.
async fn hover(browser: &Browser, element: &WebElement) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .move_to_element_center(element)
        .perform()
        .await?;
    Ok(())
}

/// Moves the pointer away, to the page's top-left corner.
async fn away(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let body = browser.element(By::Tag("footer")).await?;
    hover(browser, &body).await
}

/// Gives `element` the keyboard focus.
async fn focus(browser: &Browser, element: &WebElement) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute("arguments[0].focus();", vec![element.to_json()?])
        .await?;
    Ok(())
}

/// Scrolls `element` to the middle of the viewport, clear of the sticky top
/// bar, and clicks it.
async fn press(browser: &Browser, element: &WebElement) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![element.to_json()?],
        )
        .await?;
    element.click().await?;
    Ok(())
}

/// Presses Escape, which closes the overlay that is open.
async fn escape(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(Key::Escape)
        .perform()
        .await?;
    Ok(())
}

/// Writes the screenshot of `element` as `state`.
async fn shot_of(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
    element: &WebElement,
) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![element.to_json()?],
        )
        .await?;
    let png = element.screenshot_as_png().await?;
    write(brand, state, theme, &png)
}

/// Writes the screenshot of the viewport as `state`.
async fn shot_of_view(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
) -> Result<(), Box<dyn Error>> {
    let png = browser.driver().screenshot_as_png().await?;
    write(brand, state, theme, &png)
}

/// The pointer and focus states of the overview.
async fn overview(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}", brand.key())))
        .await?;
    crate::journeys::hydrated_tabs(browser).await?;
    let sidebar = browser.element(By::Id("sidebar")).await?;
    let entry = browser.element(By::LinkText("Forms and editor")).await?;
    hover(browser, &entry).await?;
    shot_of(browser, brand, theme, "hover-nav", &sidebar).await?;
    away(browser).await?;
    focus(browser, &entry).await?;
    shot_of(browser, brand, theme, "focus-nav", &sidebar).await?;
    let toggle = browser.element(By::Id("theme-to-dark")).await?;
    let shown = if toggle.is_displayed().await? {
        toggle
    } else {
        browser.element(By::Id("theme-to-light")).await?
    };
    focus(browser, &shown).await?;
    let bar = browser.element(By::Css(".fui-topbar")).await?;
    shot_of(browser, brand, theme, "focus-theme-toggle", &bar).await?;
    let stats = browser.element(By::Css(".fui-grid-stats")).await?;
    let first = browser.element(By::Css("a.fui-stat")).await?;
    hover(browser, &first).await?;
    shot_of(browser, brand, theme, "hover-stat", &stats).await?;
    let buttons = browser.element(card("Buttons")).await?;
    for (variant, state) in [
        ("Primary", "hover-primary"),
        ("Secondary", "hover-secondary"),
        ("Danger", "hover-danger"),
        ("Destructive", "hover-destructive"),
        ("Ghost", "hover-ghost"),
    ] {
        let button = browser
            .element(By::Css(format!("button[data-variant='{variant}']")))
            .await?;
        hover(browser, &button).await?;
        shot_of(browser, brand, theme, state, &buttons).await?;
    }
    away(browser).await?;
    let primary = browser
        .element(By::Css("button[data-variant='Primary']"))
        .await?;
    focus(browser, &primary).await?;
    shot_of(browser, brand, theme, "focus-button", &buttons).await?;
    let tabs = browser.element(card("Tabs")).await?;
    let tab = browser.element(By::LinkText("Diagnostics")).await?;
    hover(browser, &tab).await?;
    shot_of(browser, brand, theme, "hover-tab", &tabs).await?;
    away(browser).await?;
    let first_tab = browser.element(By::Id("answer-tab-0")).await?;
    focus(browser, &first_tab).await?;
    first_tab.send_keys(Key::Right).await?;
    shot_of(browser, brand, theme, "focus-keyboard-tabs", &tabs).await
}

/// The row under the pointer, and the editor with its diagnostics.
async fn tables_and_editor(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    let region = browser.element(By::Css(".fui-table-scroll")).await?;
    let row = browser
        .element(By::XPath("(//table)[1]/tbody/tr[3]"))
        .await?;
    hover(browser, &row).await?;
    shot_of(browser, brand, theme, "hover-row", &region).await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let editor = browser.element(card("Highlighted editor")).await?;
    shot_of(browser, brand, theme, "editor-diagnostic", &editor).await
}

/// The synthetic files the upload field is given: one it reads and one it
/// refuses.
fn sample_files() -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let dir = std::env::temp_dir().join("ferroui-gallery-states");
    std::fs::create_dir_all(&dir)?;
    let read = dir.join("synthetic-vital-signs.opt");
    std::fs::write(
        &read,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<template>\n  <concept>Synthetic vital signs</concept>\n</template>\n",
    )?;
    let refused = dir.join("notes.txt");
    std::fs::write(&refused, "not a template\n")?;
    Ok((read, refused))
}

/// The upload field empty, with a file over it, with a file read and with a
/// file refused.
async fn upload(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let (read, refused) = sample_files()?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let grouped = browser.element(card("Grouped fields")).await?;
    let replace = browser
        .element(By::Css("label[for='secret-replace']"))
        .await?;
    press(browser, &replace).await?;
    browser.element(By::Id("member-secret")).await?;
    shot_of(browser, brand, theme, "reveal-replace", &grouped).await?;
    let field = browser.element(card("Upload")).await?;
    let zone = browser.element(By::Id("gallery-upload-drop")).await?;
    shot_of(browser, brand, theme, "drop-empty", &field).await?;
    until("the drop zone takes a file over it", async || {
        browser
            .driver()
            .execute(
                "arguments[0].dispatchEvent(new DragEvent('dragover', {bubbles: true, cancelable: true}));",
                vec![zone.to_json()?],
            )
            .await?;
        Ok(zone
            .attr("class")
            .await?
            .is_some_and(|class| class.contains("fui-drop-over")))
    })
    .await?;
    shot_of(browser, brand, theme, "drop-over", &field).await?;
    browser
        .driver()
        .execute(
            "arguments[0].dispatchEvent(new DragEvent('dragleave', {bubbles: true}));",
            vec![zone.to_json()?],
        )
        .await?;
    let input = browser.element(By::Id("gallery-upload-file")).await?;
    input.send_keys(read.to_string_lossy()).await?;
    browser.text("p", "synthetic-vital-signs.opt read").await?;
    shot_of(browser, brand, theme, "drop-chosen", &field).await?;
    input.send_keys(refused.to_string_lossy()).await?;
    browser.text("p", "notes.txt was not read").await?;
    shot_of(browser, brand, theme, "drop-refused", &field).await
}

/// Opens the overlay a press on `opener` shows, writes the viewport as
/// `state`, and closes it with Escape.
async fn overlay(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
    opener: By,
    shown: By,
) -> Result<(), Box<dyn Error>> {
    let button = browser.element(opener).await?;
    let target = browser.element(shown).await?;
    until(&format!("{state} opens"), async || {
        if !target.is_displayed().await? {
            press(browser, &button).await?;
        }
        Ok(target.is_displayed().await?)
    })
    .await?;
    away(browser).await?;
    shot_of_view(browser, brand, theme, state).await?;
    escape(browser).await?;
    until(&format!("{state} closes"), async || {
        Ok(!target.is_displayed().await?)
    })
    .await
}

/// Every overlay open, and the toasts.
async fn overlays(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/overlays", brand.key())))
        .await?;
    crate::journeys::hydrated_toasts(browser).await?;
    for close in browser
        .driver()
        .find_all(By::Css(".fui-toast button"))
        .await?
    {
        close.click().await?;
    }
    until("the probe's toast is gone", async || {
        Ok(browser
            .driver()
            .find_all(By::Css(".fui-toast"))
            .await?
            .is_empty())
    })
    .await?;
    let opened = [
        (
            "dialog-open",
            "//button[normalize-space(.) = 'Reload the set']",
            "reload-set",
        ),
        (
            "dialog-danger",
            "//button[normalize-space(.) = 'Delete the file']",
            "delete-file",
        ),
        (
            "modal-open",
            "//button[@data-delete = 'synthetic-007.model.yml']",
            "delete-row",
        ),
        ("modal-wide", "//button[@id = 'open-rename']", "rename-file"),
        (
            "side-panel",
            "//button[normalize-space(.) = 'View access']",
            "access",
        ),
        (
            "operator-menu",
            "//button[@aria-controls = 'operator-menu']",
            "operator-menu",
        ),
    ];
    for (state, opener, id) in opened {
        overlay(browser, brand, theme, state, By::XPath(opener), By::Id(id)).await?;
    }
    for tone in ["ok", "info", "warn", "bad"] {
        let raise = browser
            .element(By::Css(format!("button[data-toast='{tone}']")))
            .await?;
        press(browser, &raise).await?;
    }
    let region = browser.element(By::Css(".fui-toasts")).await?;
    until("four toasts show", async || {
        Ok(region.find_all(By::Css(".fui-toast")).await?.len() >= 4)
    })
    .await?;
    shot_of(browser, brand, theme, "toasts", &region).await?;
    for close in browser
        .driver()
        .find_all(By::Css(".fui-toast button"))
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
    .await?;
    window(browser, 390, HEIGHT).await?;
    overlay(
        browser,
        brand,
        theme,
        "drawer-open",
        By::XPath("//button[@aria-label = 'Menu']"),
        By::Id("sidebar"),
    )
    .await?;
    window(browser, WIDTH, HEIGHT).await
}

/// The layouts page: the search box focused, the help switch on, the
/// picker open on its input, and the tree walked by the keyboard.
async fn layouts(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/layouts", brand.key())))
        .await?;
    let search = browser.element(By::Id("gallery-search")).await?;
    focus(browser, &search).await?;
    let bar = browser.element(By::Css(".fui-topbar")).await?;
    shot_of(browser, brand, theme, "focus-search", &bar).await?;
    let switch = browser
        .element(By::XPath(
            "//button[normalize-space(.) = 'Explain these parameters']",
        ))
        .await?;
    until("the help switch answers", async || {
        if switch.attr("aria-pressed").await?.as_deref() != Some("true") {
            press(browser, &switch).await?;
        }
        Ok(switch.attr("aria-pressed").await?.as_deref() == Some("true"))
    })
    .await?;
    let form = browser.element(card("Main and aside")).await?;
    shot_of(browser, brand, theme, "help-on", &form).await?;
    let choice = browser.element(By::Id("run-value-set-choice")).await?;
    browser

        .driver()

        .execute("arguments[0].value = arguments[0].options[arguments[0].options.length - 1].value; arguments[0].dispatchEvent(new Event('change', {bubbles: true}));", vec![choice.to_json()?])

        .await?;
    let input = browser.element(By::Id("run-value-set")).await?;
    until("the picker shows its input", async || {
        Ok(input.is_displayed().await?)
    })
    .await?;
    input.clear().await?;
    input.send_keys("http://example.org/vs/mine").await?;
    shot_of(browser, brand, theme, "picker-other", &form).await?;
    let first = browser
        .element(By::Css("#gallery-tree [role='treeitem']"))
        .await?;
    focus(browser, &first).await?;
    first.send_keys(Key::Down).await?;
    let pane = browser.element(card("Three panes and the tree")).await?;
    shot_of(browser, brand, theme, "tree-keyboard", &pane).await
}

/// The tables page in the compact density, its cookie set for the one shot.
async fn compact(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let mut cookie = Cookie::new("gallery-density", "compact");
    cookie.set_path("/");
    browser.driver().add_cookie(cookie).await?;
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    browser.element(By::Css("main h1")).await?;
    shot_of_view(browser, brand, theme, "density-compact").await?;
    browser.driver().delete_cookie("gallery-density").await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn every_state_is_captured_in_every_brand_light_and_dark() -> Result<(), Box<dyn Error>> {
    if !std::env::var("FERROUI_SCREENSHOTS").is_ok_and(|value| value == "1") {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, WIDTH, HEIGHT).await?;
        for theme in ["light", "dark"] {
            browser.goto(&gallery.url("/")).await?;
            browser.driver().delete_all_cookies().await?;
            let mut cookie = Cookie::new("gallery-theme", theme);
            cookie.set_path("/");
            browser.driver().add_cookie(cookie).await?;
            browser
                .driver()
                .cdp()
                .send_raw(
                    "Emulation.setEmulatedMedia",
                    json!({ "features": [{ "name": "prefers-color-scheme", "value": theme }, { "name": "prefers-reduced-motion", "value": "reduce" }] }),
                )
                .await?;
            for brand in Brand::ALL {
                overview(browser, &gallery, brand, theme).await?;
                tables_and_editor(browser, &gallery, brand, theme).await?;
                upload(browser, &gallery, brand, theme).await?;
                overlays(browser, &gallery, brand, theme).await?;
                layouts(browser, &gallery, brand, theme).await?;
                compact(browser, &gallery, brand, theme).await?;
            }
        }
        Ok(())
    })
    .await
}
