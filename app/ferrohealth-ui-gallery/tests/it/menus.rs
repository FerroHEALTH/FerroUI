// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the menus and listboxes: the title row's More (F13,
//! F14) and its keyboard path, the phone's bottom sheet, the select button
//! and its listbox (R29), and the combobox's listbox, which keeps its rows
//! as the highlight moves (WAI-ARIA Authoring Practices, the menu button,
//! listbox and combobox patterns).

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The trigger of the menu `id`, once its island has hydrated, which the
/// trigger's `aria-expanded` says.
pub(crate) async fn hydrated_trigger(
    browser: &Browser,
    id: &str,
) -> Result<WebElement, Box<dyn Error>> {
    let trigger = browser
        .element(By::Css(format!("summary[aria-controls='{id}']")))
        .await?;
    until(&format!("the menu {id} hydrates"), async || {
        Ok(trigger.attr("aria-expanded").await?.is_some())
    })
    .await?;
    Ok(trigger)
}

/// The words of the element that holds the focus.
async fn focused_words(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .driver()
        .active_element()
        .await?
        .text()
        .await?
        .trim()
        .to_owned())
}

/// Whether the `<details>` around the menu `id` is open.
async fn is_open(browser: &Browser, id: &str) -> Result<bool, Box<dyn Error>> {
    let open = browser
        .driver()
        .execute(
            "return document.getElementById(arguments[0]).closest('details').open;",
            vec![serde_json::Value::from(id)],
        )
        .await?;
    Ok(open.convert()?)
}

/// Gives `element` the keyboard focus.
async fn focus(browser: &Browser, element: &WebElement) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute("arguments[0].focus();", vec![element.to_json()?])
        .await?;
    Ok(())
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

#[tokio::test(flavor = "multi_thread")]
async fn more_walks_by_the_keyboard_skips_a_disabled_item_and_returns_the_focus()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        let more = hydrated_trigger(browser, "overlays-more").await?;
        assert_eq!(
            Some(String::from("false")),
            more.attr("aria-expanded").await?
        );
        focus(browser, &more).await?;
        press(browser, Key::Down).await?;
        until("Down opens More on its first item", async || {
            Ok(focused_words(browser).await? == "Open the document")
        })
        .await?;
        assert!(is_open(browser, "overlays-more").await?);
        assert_eq!(
            Some(String::from("true")),
            more.attr("aria-expanded").await?
        );
        press(browser, Key::Down).await?;
        assert_eq!(
            "Delete the file\u{2026}",
            focused_words(browser).await?,
            "the disabled download is skipped, the destructive item is last"
        );
        press(browser, Key::Down).await?;
        assert_eq!("Open the document", focused_words(browser).await?, "wraps");
        press(browser, Key::End).await?;
        assert_eq!("Delete the file\u{2026}", focused_words(browser).await?);
        press(browser, Key::Home).await?;
        assert_eq!("Open the document", focused_words(browser).await?);
        press(browser, Key::Escape).await?;
        until("Escape closes More", async || {
            Ok(!is_open(browser, "overlays-more").await?)
        })
        .await?;
        let active = browser.driver().active_element().await?;
        assert_eq!(
            Some(String::from("overlays-more")),
            active.attr("aria-controls").await?,
            "the focus returns to More"
        );
        more.click().await?;
        until("a press opens More", async || {
            is_open(browser, "overlays-more").await
        })
        .await?;
        browser.element(By::Css("main h1")).await?.click().await?;
        until("a press outside closes More", async || {
            Ok(!is_open(browser, "overlays-more").await?)
        })
        .await?;
        more.click().await?;
        browser
            .element(By::XPath(
                "//button[@role = 'menuitem' and contains(., 'Delete the file')]",
            ))
            .await?
            .click()
            .await?;
        let dialog = browser.element(By::Id("delete-file")).await?;
        until("the delete opens its confirmation", async || {
            Ok(dialog.is_displayed().await?)
        })
        .await?;
        assert!(
            !is_open(browser, "overlays-more").await?,
            "choosing an item closes More"
        );
        press(browser, Key::Escape).await?;
        until("the confirmation closes", async || {
            Ok(!dialog.is_displayed().await?)
        })
        .await?;
        browser.console_clean("the More menu").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn more_is_a_bottom_sheet_with_its_cancel_on_a_phone() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferropix/overlays")).await?;
        let more = hydrated_trigger(browser, "overlays-more").await?;
        more.click().await?;
        let cancel = browser
            .element(By::XPath(
                "//summary[@aria-controls = 'overlays-more']/following-sibling::div//button[@class = 'fui-menu-cancel']",
            ))
            .await?;
        until("the sheet shows its Cancel", async || {
            cancel.is_displayed().await.map_err(Into::into)
        })
        .await?;
        let docked: bool = browser
            .driver()
            .execute(
                "const box = document.getElementById('overlays-more').parentElement.getBoundingClientRect();
                return Math.abs(box.bottom - window.innerHeight) < 1 && Math.abs(box.width - document.documentElement.clientWidth) < 1;",
                vec![],
            )
            .await?
            .convert()?;
        assert!(docked, "the sheet spans the foot of the screen");
        cancel.click().await?;
        until("Cancel closes the sheet", async || {
            Ok(!is_open(browser, "overlays-more").await?)
        })
        .await?;
        let active = browser.driver().active_element().await?;
        assert_eq!(
            Some(String::from("overlays-more")),
            active.attr("aria-controls").await?
        );
        browser.console_clean("the sheet").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_listbox_opens_on_its_choice_and_a_choice_follows_its_link()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr/overlays")).await?;
        let select = hydrated_trigger(browser, "version-list").await?;
        select.click().await?;
        until("the listbox opens on the choice in force", async || {
            Ok(focused_words(browser).await?.starts_with("1.1.0"))
        })
        .await?;
        press(browser, Key::Down).await?;
        assert_eq!("1.0.0", focused_words(browser).await?);
        press(browser, Key::Enter).await?;
        browser
            .at(&gallery.url("/ferroehr/overlays?version=1.0.0"))
            .await?;
        browser.console_clean("the listbox").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_combobox_keeps_its_rows_as_the_highlight_moves() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let input = browser
            .element(By::Css("#choose-system[role='combobox']"))
            .await?;
        input.click().await?;
        until("the list opens", async || {
            Ok(input.attr("aria-expanded").await?.as_deref() == Some("true"))
        })
        .await?;
        // Each row is marked as a DOM node; a row drawn again loses its mark.
        browser
            .driver()
            .execute(
                "document.querySelectorAll('#choose-system-list [role=option]').forEach((row) => { row.dataset.kept = 'yes'; row.keptNode = true; });",
                vec![],
            )
            .await?;
        for _ in 0..3 {
            input.send_keys(Key::Down).await?;
        }
        let counts: Vec<u64> = browser
            .driver()
            .execute(
                "const rows = [...document.querySelectorAll('#choose-system-list .fui-combobox-option')];
                return [rows.length, rows.filter((row) => row.keptNode !== true).length, rows.filter((row) => row.dataset.active === 'true').length];",
                vec![],
            )
            .await?
            .convert()?;
        assert_eq!(
            vec![12, 0, 1],
            counts,
            "no row is replaced, and one row is highlighted"
        );
        input.send_keys("o").await?;
        let replaced: u64 = browser
            .driver()
            .execute(
                "return [...document.querySelectorAll('#choose-system-list .fui-combobox-option')].filter((row) => row.keptNode !== true).length;",
                vec![],
            )
            .await?
            .convert()?;
        assert_eq!(0, replaced, "a row the filter keeps is the same row");
        browser.console_clean("the keyed combobox").await
    })
    .await
}
