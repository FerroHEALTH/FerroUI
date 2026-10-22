// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The interaction states, captured behind the `FERROUI_SCREENSHOTS` gate:
//! the pointer on a navigation entry, a table row, each button variant, a
//! linked card, a card-header action and a tab; the keyboard focus on a
//! control, a navigation entry, a card-header action and the keyboard tabs; every overlay open (the dialogs, the modal
//! a signal opens, the side panel, the user menu and its phone
//! sheet, the drawer); the sidebar folded to its rail; the toasts; the upload field empty, with a file over it, with a
//! file read and with a file refused; the editor with its diagnostics; the
//! help switch off and on, the picker open on its input, the tree walked by the
//! keyboard, the search box focused, the tables in the compact density, the
//! compact stat tiles and the stacked timestamps of a phone at 320 and
//! 390 px, the disabled fields inside a well at 1440 and 390 px, the
//! title row's action group with More open by the pointer and the keyboard,
//! the split button's variants, a row menu and the select button's listbox,
//! with the action group, More's bottom sheet and the listbox's at 390 px,
//! the document pane switched to its raw view, and the states of the keys
//! ([`crate::key_states`]).
//!
//! Each state is written as `states/<brand>/<state>.png` and its `-dark`
//! twin under the screenshots folder, every brand in the light and the
//! dark theme, so the hover and focus treatment is looked at as often as
//! the pages are. The capture writes only a state [`STATES`] lists and
//! fails unless it wrote every one of them, so the list and the capture
//! cannot drift apart.

use std::collections::BTreeSet;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::stack::Gallery;

/// Every state, by the name its images take.
pub(crate) const STATES: [&str; 85] = [
    "hover-nav",
    "focus-nav",
    "hover-stat",
    "hover-card-action",
    "focus-card-action",
    "hover-primary",
    "hover-secondary",
    "hover-danger",
    "hover-destructive",
    "hover-ghost",
    "focus-button",
    "hover-tab",
    "focus-keyboard-tabs",
    "tabs-select-390",
    "hover-row",
    "focus-row",
    "focus-sort",
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
    "user-menu",
    "user-menu-sheet",
    "focus-user-menu",
    "rail-hover",
    "toasts",
    "popover-form",
    "popover-form-390",
    "drawer-open",
    "focus-search",
    "help-off",
    "help-on",
    "picker-other",
    "tree-keyboard",
    "density-compact",
    "reveal-replace",
    "combobox-open",
    "combobox-hover",
    "combobox-keyboard",
    "combobox-narrow",
    "stats-compact-320",
    "stats-compact-390",
    "timestamp-stacked-320",
    "disabled-well",
    "field-shapes",
    "pick-or-type-typed",
    "pick-or-type-combobox-typed",
    "disabled-well-390",
    "document-raw",
    "action-group",
    "menu-open",
    "menu-keyboard",
    "split-open",
    "row-menu-open",
    "listbox-open",
    "action-group-390",
    "menu-sheet-390",
    "listbox-sheet-390",
    "hover-search",
    "search-filled",
    "search-busy",
    "search-disabled",
    "search-answer",
    "search-none",
    "search-390",
    "search-answer-390",
    "dock-idle",
    "dock-missing",
    "dock-disabled",
    "dock-submitting",
    "dock-done",
    "dock-stale",
    "dock-bare",
    "dock-path",
    "dock-idle-390",
    "dock-disabled-390",
    "dock-path-390",
    "legend-open",
    "shortcut-hint",
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

/// Every image the capture wrote, by brand, theme and state.
static WRITTEN: Mutex<BTreeSet<(&'static str, String, String)>> = Mutex::new(BTreeSet::new());

/// Refuses a state [`STATES`] does not list, whose image the screenshot
/// guard would reject.
fn listed(state: &str) -> Result<(), Box<dyn Error>> {
    if STATES.contains(&state) {
        Ok(())
    } else {
        Err(format!("the capture writes {state}, which STATES does not list").into())
    }
}

/// Every listed state the capture did not write in some brand and theme.
fn unwritten() -> Vec<String> {
    let written = WRITTEN.lock().unwrap_or_else(PoisonError::into_inner);
    let mut missing = Vec::new();
    for brand in Brand::ALL {
        for theme in ["light", "dark"] {
            for state in STATES {
                if !written.contains(&(brand.key(), theme.to_owned(), state.to_owned())) {
                    missing.push(format!("{}/{state} ({theme})", brand.key()));
                }
            }
        }
    }
    missing
}

/// Writes `png` as the image of `state`.
fn write(brand: Brand, state: &str, theme: &str, png: &[u8]) -> Result<(), Box<dyn Error>> {
    listed(state)?;
    WRITTEN
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert((brand.key(), theme.to_owned(), state.to_owned()));
    let path = file(brand, state, theme);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, png)?;
    Ok(())
}

/// The card whose title is `title`.
pub(crate) fn card(title: &str) -> By {
    By::XPath(format!(
        "//section[contains(@class, 'fui-card')][h2[normalize-space(.) = '{title}'] or div/h2[normalize-space(.) = '{title}']]"
    ))
}

/// Moves the pointer onto `element`.
pub(crate) async fn hover(browser: &Browser, element: &WebElement) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .move_to_element_center(element)
        .perform()
        .await?;
    Ok(())
}

/// Moves the pointer away, to the page's top-left corner.
pub(crate) async fn away(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let body = browser.element(By::Tag("footer")).await?;
    hover(browser, &body).await
}

/// Gives `element` the keyboard focus.
pub(crate) async fn focus(browser: &Browser, element: &WebElement) -> Result<(), Box<dyn Error>> {
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
pub(crate) async fn escape(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(Key::Escape)
        .perform()
        .await?;
    Ok(())
}

/// Writes the screenshot of `element` as `state`.
pub(crate) async fn shot_of(
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
    browser.settled().await?;
    let png = element.screenshot_as_png().await?;
    write(brand, state, theme, &png)
}

/// Writes the screenshot of the viewport as `state`.
pub(crate) async fn shot_of_view(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
) -> Result<(), Box<dyn Error>> {
    browser.settled().await?;
    let png = browser.driver().screenshot_as_png().await?;
    write(brand, state, theme, &png)
}

/// The document pane switched in place to its raw view.
async fn document(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/code", brand.key())))
        .await?;
    browser
        .element(By::Css(".fui-document-views.is-live"))
        .await?;
    browser
        .element(By::Css(".fui-document-switch a[data-view='raw']"))
        .await?
        .click()
        .await?;
    let raw = browser.element(By::Id("document-raw")).await?;
    until("the raw view shows", async || {
        Ok(raw.is_displayed().await?)
    })
    .await?;
    let pane = browser.element(By::Css(".fui-document")).await?;
    shot_of(browser, brand, theme, "document-raw", &pane).await
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
    let opener = browser
        .element(By::Css("button[popovertarget='user-menu']"))
        .await?;
    focus(browser, &opener).await?;
    let bar = browser.element(By::Css(".fui-topbar")).await?;
    shot_of(browser, brand, theme, "focus-user-menu", &bar).await?;
    let stats = browser.element(By::Css(".fui-grid-stats")).await?;
    let first = browser.element(By::Css("a.fui-stat")).await?;
    hover(browser, &first).await?;
    shot_of(browser, brand, theme, "hover-stat", &stats).await?;
    let preview = browser.element(card("Recent files")).await?;
    let action = browser.element(By::Css("a.fui-card-action")).await?;
    hover(browser, &action).await?;
    shot_of(browser, brand, theme, "hover-card-action", &preview).await?;
    away(browser).await?;
    focus(browser, &action).await?;
    shot_of(browser, brand, theme, "focus-card-action", &preview).await?;
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
    let tab = browser.element(By::PartialLinkText("Diagnostics")).await?;
    hover(browser, &tab).await?;
    shot_of(browser, brand, theme, "hover-tab", &tabs).await?;
    away(browser).await?;
    let first_tab = browser.element(By::Id("answer-tab-0")).await?;
    focus(browser, &first_tab).await?;
    first_tab.send_keys(Key::Right).await?;
    shot_of(browser, brand, theme, "focus-keyboard-tabs", &tabs).await?;
    away(browser).await?;
    window(browser, 390, HEIGHT).await?;
    let select = browser.element(By::Id("fui-tabs-select")).await?;
    until("the tab select takes the row's place", async || {
        Ok(select.is_displayed().await?)
    })
    .await?;
    let tabs = browser.element(card("Tabs")).await?;
    shot_of(browser, brand, theme, "tabs-select-390", &tabs).await?;
    window(browser, WIDTH, HEIGHT).await
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
    away(browser).await?;
    let link = row.find(By::Css("a.fui-row-link")).await?;
    focus(browser, &link).await?;
    shot_of(browser, brand, theme, "focus-row", &row).await?;
    let header = browser.element(By::XPath("(//table)[1]/thead/tr")).await?;
    let sort = header
        .find(By::XPath(".//th[contains(., 'Lines')]//button"))
        .await?;
    focus(browser, &sort).await?;
    shot_of(browser, brand, theme, "focus-sort", &header).await?;
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
    // The refusal replaces the read's state in place; two frames let the
    // browser paint the new icon in the refusal's tone before the capture.
    browser
        .driver()
        .execute_async(
            "const done = arguments[arguments.length - 1];
            requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(done, 150)));",
            vec![],
        )
        .await?;
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

/// The four toasts raised at once, then dismissed.
async fn raised_toasts(browser: &Browser, brand: Brand, theme: &str) -> Result<(), Box<dyn Error>> {
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
    .await?;
    Ok(())
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
        .find_all(By::Css(".fui-toast > .fui-icon-button"))
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
            "//button[@popovertarget = 'reload-set' and not(@role)]",
            "reload-set",
        ),
        (
            "dialog-danger",
            "//button[@role = 'menuitem' and contains(., 'Delete the file')]",
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
            "user-menu",
            "//button[@popovertarget = 'user-menu']",
            "user-menu",
        ),
        (
            "popover-form",
            "//button[@popovertarget = 'save-query']",
            "save-query",
        ),
    ];
    for (state, opener, id) in opened {
        if state == "dialog-danger" {
            // The delete is More's last item, so More opens first.
            press(
                browser,
                &crate::menus::hydrated_trigger(browser, "overlays-more").await?,
            )
            .await?;
        }
        overlay(browser, brand, theme, state, By::XPath(opener), By::Id(id)).await?;
    }
    raised_toasts(browser, brand, theme).await?;
    window(browser, 390, HEIGHT).await?;
    overlay(
        browser,
        brand,
        theme,
        "popover-form-390",
        By::XPath("//button[@popovertarget = 'save-query']"),
        By::Id("save-query"),
    )
    .await?;
    overlay(
        browser,
        brand,
        theme,
        "drawer-open",
        By::XPath("//button[@aria-label = 'Menu']"),
        By::Id("sidebar"),
    )
    .await?;
    overlay(
        browser,
        brand,
        theme,
        "user-menu-sheet",
        By::XPath("//button[@popovertarget = 'user-menu']"),
        By::Id("user-menu"),
    )
    .await?;
    window(browser, WIDTH, HEIGHT).await
}

/// The layouts page: the search box focused, the help switch off and on, the
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
    let form = browser.element(card("Main and aside")).await?;
    shot_of(browser, brand, theme, "help-off", &form).await?;
    let switch = browser.element(By::Id("run-help")).await?;
    until("the help switch answers", async || {
        if !switch.is_selected().await? {
            press(browser, &switch).await?;
        }
        Ok(switch.is_selected().await?)
    })
    .await?;
    shot_of(browser, brand, theme, "help-on", &form).await?;
    let choice = browser.element(By::Id("run-value-set-choice")).await?;
    browser

        .driver()

        .execute("arguments[0].selectedIndex = arguments[0].options.length - 1; arguments[0].dispatchEvent(new Event('change', {bubbles: true}));", vec![choice.to_json()?])

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

/// The combobox on the forms page: open, an option under the pointer, an
/// option the keyboard highlights, and open at 390 px, each with the page
/// around it.
async fn combobox_states(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let input = browser
        .element(By::Css("#choose-system[role='combobox']"))
        .await?;
    let place = |input: &WebElement| {
        let script = "arguments[0].scrollIntoView({block: 'start'}); window.scrollBy(0, -120);";
        let json = input.to_json();
        async move {
            browser.driver().execute(script, vec![json?]).await?;
            Ok::<(), Box<dyn Error>>(())
        }
    };
    place(&input).await?;
    input.click().await?;
    until("the combobox opens", async || {
        Ok(input.attr("aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    let label = browser
        .element(By::Css("label[for='choose-system']"))
        .await?;
    hover(browser, &label).await?;
    shot_of_view(browser, brand, theme, "combobox-open").await?;
    let third = browser
        .element(By::Css("#choose-system-list [role='option']:nth-child(3)"))
        .await?;
    hover(browser, &third).await?;
    shot_of_view(browser, brand, theme, "combobox-hover").await?;
    escape(browser).await?;
    focus(browser, &input).await?;
    input.send_keys(Key::Down).await?;
    input.send_keys(Key::Down).await?;
    input.send_keys(Key::Down).await?;
    hover(browser, &label).await?;
    shot_of_view(browser, brand, theme, "combobox-keyboard").await?;
    escape(browser).await?;
    window(browser, 390, 844).await?;
    let input = browser
        .element(By::Css("#choose-system[role='combobox']"))
        .await?;
    place(&input).await?;
    // A click that lands while the narrow layout settles opens nothing, so
    // the input is clicked again until the list shows.
    until("the combobox opens at 390 px", async || {
        if input.attr("aria-expanded").await?.as_deref() != Some("true") {
            input.click().await?;
        }
        Ok(input.attr("aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    shot_of_view(browser, brand, theme, "combobox-narrow").await?;
    escape(browser).await?;
    window(browser, WIDTH, HEIGHT).await
}

/// The phone's compact stat tiles at 320 and 390 px, the stacked table's
/// timestamps at 320 px, and the disabled fields inside the forms page's
/// well.
async fn phone(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    for (width, state) in [(320, "stats-compact-320"), (390, "stats-compact-390")] {
        window(browser, width, HEIGHT).await?;
        browser
            .goto(&gallery.url(&format!("/{}", brand.key())))
            .await?;
        let stats = browser.element(By::Css(".fui-grid-stats")).await?;
        shot_of(browser, brand, theme, state, &stats).await?;
    }
    window(browser, 320, HEIGHT).await?;
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    let first = browser
        .element(By::XPath("(//table)[1]/tbody/tr[1]"))
        .await?;
    browser
        .driver()
        .execute(
            "window.scrollTo(0, arguments[0].getBoundingClientRect().top + window.scrollY - 72);",
            vec![first.to_json()?],
        )
        .await?;
    shot_of_view(browser, brand, theme, "timestamp-stacked-320").await?;
    window(browser, WIDTH, HEIGHT).await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let grouped = browser.element(card("Grouped fields")).await?;
    shot_of(browser, brand, theme, "disabled-well", &grouped).await?;
    window(browser, 390, HEIGHT).await?;
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    let grouped = browser.element(card("Grouped fields")).await?;
    shot_of(browser, brand, theme, "disabled-well-390", &grouped).await?;
    window(browser, WIDTH, HEIGHT).await
}

/// Opens the menu `id` by a press on its trigger, scrolled to the middle of
/// the viewport, and writes the viewport as `state`.
async fn opened_menu(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
    id: &str,
) -> Result<(), Box<dyn Error>> {
    let trigger = crate::menus::hydrated_trigger(browser, id).await?;
    press(browser, &trigger).await?;
    until(&format!("{state} opens"), async || {
        Ok(trigger.attr("aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    let heading = browser.element(By::Css("main h1")).await?;
    hover(browser, &heading).await?;
    shot_of_view(browser, brand, theme, state).await
}

/// Closes the menu `id` with Escape from its trigger.
async fn closed_menu(browser: &Browser, id: &str) -> Result<(), Box<dyn Error>> {
    escape(browser).await?;
    let trigger = crate::menus::hydrated_trigger(browser, id).await?;
    until(&format!("{id} closes"), async || {
        Ok(trigger.attr("aria-expanded").await?.as_deref() == Some("false"))
    })
    .await
}

/// The title row's action group, More open by the pointer and by the
/// keyboard, the split button's variants, a row menu and the select
/// button's listbox, then the action group, More's bottom sheet and the
/// listbox's at 390 px.
async fn menu_states(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let page = gallery.url(&format!("/{}/overlays", brand.key()));
    browser.goto(&page).await?;
    crate::menus::hydrated_trigger(browser, "overlays-more").await?;
    let header = browser.element(By::Css(".fui-page-header")).await?;
    shot_of(browser, brand, theme, "action-group", &header).await?;
    browser
        .driver()
        .execute("window.scrollTo(0, 0);", vec![])
        .await?;
    opened_menu(browser, brand, theme, "menu-open", "overlays-more").await?;
    browser
        .driver()
        .action_chain()
        .send_keys(Key::Down)
        .perform()
        .await?;
    shot_of_view(browser, brand, theme, "menu-keyboard").await?;
    closed_menu(browser, "overlays-more").await?;
    opened_menu(browser, brand, theme, "split-open", "save-variants").await?;
    closed_menu(browser, "save-variants").await?;
    opened_menu(browser, brand, theme, "row-menu-open", "row-menu-0").await?;
    closed_menu(browser, "row-menu-0").await?;
    opened_menu(browser, brand, theme, "listbox-open", "version-list").await?;
    closed_menu(browser, "version-list").await?;
    window(browser, 390, 844).await?;
    browser.goto(&page).await?;
    crate::menus::hydrated_trigger(browser, "overlays-more").await?;
    let header = browser.element(By::Css(".fui-page-header")).await?;
    shot_of(browser, brand, theme, "action-group-390", &header).await?;
    opened_menu(browser, brand, theme, "menu-sheet-390", "overlays-more").await?;
    closed_menu(browser, "overlays-more").await?;
    opened_menu(browser, brand, theme, "listbox-sheet-390", "version-list").await?;
    closed_menu(browser, "version-list").await?;
    window(browser, WIDTH, HEIGHT).await
}

/// The field shapes card with its error lines showing, and the two
/// pick-or-type fields switched to their typed inputs.
async fn field_states(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{}/forms", brand.key())))
        .await?;
    browser
        .element(By::Css("#pick-canonical-choice[role='combobox']"))
        .await?;
    let shapes = browser.element(card("Field shapes")).await?;
    shot_of(browser, brand, theme, "field-shapes", &shapes).await?;
    let picks = browser.element(card("Pick or type")).await?;
    let select = browser.element(By::Id("pick-version-choice")).await?;
    select
        .find(By::Css("option[value='another']"))
        .await?
        .click()
        .await?;
    let typed = browser.element(By::Id("pick-version")).await?;
    until("the typed version shows", async || {
        typed.is_displayed().await.map_err(Into::into)
    })
    .await?;
    focus(browser, &typed).await?;
    shot_of(browser, brand, theme, "pick-or-type-typed", &picks).await?;
    browser
        .element(By::Id("pick-version-back"))
        .await?
        .click()
        .await?;
    let combo = browser
        .element(By::Css("#pick-canonical-choice[role='combobox']"))
        .await?;
    combo.click().await?;
    browser
        .element(By::Css(
            "#pick-canonical-choice-list li[role='option']:last-child",
        ))
        .await?
        .click()
        .await?;
    let canonical = browser.element(By::Id("pick-canonical")).await?;
    until("the typed canonical shows", async || {
        canonical.is_displayed().await.map_err(Into::into)
    })
    .await?;
    shot_of(browser, brand, theme, "pick-or-type-combobox-typed", &picks).await
}

/// The sidebar folded to its rail, an entry under the pointer, its cookie
/// set for the one shot.
async fn rail(
    browser: &Browser,
    gallery: &Gallery,
    brand: Brand,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    let mut cookie = Cookie::new("gallery-sidebar", "rail");
    cookie.set_path("/");
    browser.driver().add_cookie(cookie).await?;
    browser
        .goto(&gallery.url(&format!("/{}/tables", brand.key())))
        .await?;
    browser
        .element(By::Css("#sidebar.fui-sidebar-rail"))
        .await?;
    let entry = browser
        .element(By::Css("#sidebar a[title='Forms and editor']"))
        .await?;
    hover(browser, &entry).await?;
    shot_of_view(browser, brand, theme, "rail-hover").await?;
    away(browser).await?;
    browser.driver().delete_cookie("gallery-sidebar").await?;
    Ok(())
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
                combobox_states(browser, &gallery, brand, theme).await?;
                field_states(browser, &gallery, brand, theme).await?;
                compact(browser, &gallery, brand, theme).await?;
                rail(browser, &gallery, brand, theme).await?;
                phone(browser, &gallery, brand, theme).await?;
                document(browser, &gallery, brand, theme).await?;
                menu_states(browser, &gallery, brand, theme).await?;
                crate::key_states::keys(browser, &gallery, brand, theme).await?;
            }
        }
        let missing = unwritten();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!("STATES lists states the capture never wrote: {missing:?}").into())
        }
    })
    .await
}

#[test]
fn a_state_states_does_not_list_is_refused() {
    assert!(listed("hover-card-action").is_ok());
    assert!(listed("focus-card-action").is_ok());
    assert!(
        listed("hover-unlisted").is_err(),
        "the capture would write an image the screenshot guard rejects"
    );
}

#[test]
fn every_state_is_listed_once() {
    let unique: BTreeSet<&str> = STATES.into_iter().collect();
    assert_eq!(unique.len(), STATES.len(), "STATES names a state twice");
}
