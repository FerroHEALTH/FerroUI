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
async fn more_walks_by_the_keyboard_reaches_a_disabled_item_and_returns_the_focus()
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
        // The toggle event that says More opened fires after the key.
        until("More says it is open", async || {
            Ok(more.attr("aria-expanded").await?.as_deref() == Some("true"))
        })
        .await?;
        press(browser, Key::Down).await?;
        // WAI-ARIA Authoring Practices, "Focusability of disabled controls": a disabled menu
        // item stays focusable, so its reason is read.
        let disabled = browser.driver().active_element().await?;
        assert!(
            focused_words(browser)
                .await?
                .starts_with("Download the file"),
            "the disabled download is reached"
        );
        assert_eq!(
            Some(String::from("true")),
            disabled.attr("aria-disabled").await?,
            "and announced disabled"
        );
        press(browser, Key::Enter).await?;
        assert!(
            is_open(browser, "overlays-more").await?,
            "a disabled item takes no press"
        );
        press(browser, Key::Down).await?;
        assert_eq!(
            "Delete the file\u{2026}",
            focused_words(browser).await?,
            "the destructive item is last"
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
async fn a_copy_item_copies_its_text_and_says_so_in_a_toast() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser
            .driver()
            .cdp()
            .send_raw(
                "Browser.grantPermissions",
                serde_json::json!({ "permissions": ["clipboardReadWrite", "clipboardSanitizedWrite"] }),
            )
            .await?;
        browser.goto(&gallery.url("/ferrobridge/overlays")).await?;
        let more = hydrated_trigger(browser, "alone-more").await?;
        more.scroll_into_view().await?;
        more.click().await?;
        let copy = browser
            .element(By::XPath(
                "//button[@role = 'menuitem' and contains(., \"Copy the set's name\")]",
            ))
            .await?;
        until("the copy item shows once the menu hydrates", async || {
            Ok(copy.is_displayed().await?)
        })
        .await?;
        copy.click().await?;
        browser.text("strong", "Copied to the clipboard.").await?;
        assert!(
            !is_open(browser, "alone-more").await?,
            "choosing the copy item closes the menu"
        );
        let held = browser
            .driver()
            .execute_async(
                "navigator.clipboard.readText().then(arguments[0], () => arguments[0](null));",
                Vec::new(),
            )
            .await?;
        assert_eq!(
            Some(ferrohealth_ui_gallery::data::SET_NAME),
            held.json().as_str(),
            "the clipboard holds the set's name"
        );
        browser.console_clean("the copy item").await
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
                return Math.abs(box.bottom - document.documentElement.clientHeight) < 1 && Math.abs(box.width - document.documentElement.clientWidth) < 1;",
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
            let active = browser.driver().active_element().await?;
            Ok(active.attr("role").await?.as_deref() == Some("option")
                && active.attr("aria-selected").await?.as_deref() == Some("true"))
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

/// The id of the element that holds the focus.
async fn focused_id(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .driver()
        .active_element()
        .await?
        .attr("id")
        .await?
        .unwrap_or_default())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_row_menus_remove_asks_why_and_removes_the_row_by_keyboard() -> Result<(), Box<dyn Error>>
{
    // HTML Living Standard, the `command` attribute and constraint validation: the item opens
    // the dialog with no script, and a required field holds the post until it is filled.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        let menu = hydrated_trigger(browser, "member-menu-node-b").await?;
        focus(browser, &menu).await?;
        press(browser, Key::Down).await?;
        until("Down opens the row menu on Remove…", async || {
            Ok(focused_words(browser).await? == "Remove\u{2026}")
        })
        .await?;
        press(browser, Key::Enter).await?;
        let dialog = browser.element(By::Id("remove-node-b")).await?;
        until("Remove… opens the row's own confirmation", async || {
            Ok(dialog.attr("open").await?.is_some())
        })
        .await?;
        until("the focus moves to the Reason", async || {
            Ok(focused_id(browser).await? == "remove-node-b-reason")
        })
        .await?;
        press(browser, Key::Tab).await?;
        press(browser, Key::Tab).await?;
        assert_eq!("Remove node-b", focused_words(browser).await?);
        press(browser, Key::Enter).await?;
        until(
            "a removal with no reason is held at the Reason",
            async || Ok(focused_id(browser).await? == "remove-node-b-reason"),
        )
        .await?;
        assert!(
            dialog.attr("open").await?.is_some(),
            "the dialog stays open"
        );
        browser
            .driver()
            .active_element()
            .await?
            .send_keys("Replaced by node-b2")
            .await?;
        press(browser, Key::Tab).await?;
        press(browser, Key::Tab).await?;
        press(browser, Key::Enter).await?;
        browser
            .at(&gallery.url("/ferrofed/overlays?removed=node-b"))
            .await?;
        browser.element(By::Id("member-removed")).await?;
        assert!(
            browser
                .driver()
                .find_all(By::Id("remove-node-b"))
                .await?
                .is_empty(),
            "the removed row is gone"
        );
        browser.element(By::Id("remove-node-c")).await?;
        browser.console_clean("a row menu's removal").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_confirmation_in_a_rows_actions_cell_wraps_its_sentence_on_a_phone()
-> Result<(), Box<dyn Error>> {
    // WCAG 2.2 SC 1.4.10 Reflow: at 390 px the dialog's sentence wraps inside it, whatever the
    // actions cell around it sets for its own buttons.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        let dialog = browser
            .element(By::Css("#member-removals td.fui-table-actions dialog#remove-node-b"))
            .await?;
        browser
            .driver()
            .execute(
                "document.getElementById('remove-node-b').showModal();",
                vec![],
            )
            .await?;
        until("the confirmation opens", async || {
            Ok(dialog.attr("open").await?.is_some())
        })
        .await?;
        let laid: Vec<String> = browser
            .driver()
            .execute(
                "const dialog = document.getElementById('remove-node-b');
                const edge = dialog.getBoundingClientRect().right;
                const style = getComputedStyle(dialog);
                const out = [style.whiteSpace, style.textAlign];
                for (const node of dialog.querySelectorAll('h2, p')) {
                    if (node.getBoundingClientRect().right > edge + 0.5 || node.scrollWidth > node.clientWidth) {
                        out.push(node.textContent.trim());
                    }
                }
                return out;",
                vec![],
            )
            .await?
            .convert()?;
        assert_eq!(
            vec![String::from("normal"), String::from("start")],
            laid,
            "the dialog resets the cell's nowrap and end alignment, and no line runs past its edge"
        );
        browser.console_clean("a confirmation in an actions cell").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_row_menu_in_an_actions_cell_sits_on_the_rows_first_line() -> Result<(), Box<dyn Error>> {
    // NOTE: design/COMPONENTS.md, The table; an action in the actions cell sits on the row's
    // first line, a row menu as a button does.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        browser
            .element(By::Css("#member-removals td.fui-table-actions summary"))
            .await?;
        let offset: f64 = browser
            .driver()
            .execute(
                "const row = document.querySelector('#member-removals tbody tr');
                const name = row.querySelector('th code').getBoundingClientRect();
                const menu = row.querySelector('td.fui-table-actions summary').getBoundingClientRect();
                return (menu.top + menu.height / 2) - (name.top + name.height / 2);",
                vec![],
            )
            .await?
            .convert()?;
        assert!(
            offset.abs() <= 1.5,
            "the row menu's centre meets the centre of the row's first line: {offset} px apart"
        );
        browser.console_clean("a row menu in an actions cell").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_results_export_menu_submits_the_query_form_to_its_format() -> Result<(), Box<dyn Error>>
{
    // HTML Living Standard, the `formaction` attribute: the item submits the form it sits in
    // to another address, its fields riding along.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr/clinical")).await?;
        let export = browser
            .element(By::Css("#results-form summary[aria-haspopup='menu']"))
            .await?;
        until("Export hydrates", async || {
            Ok(export.attr("aria-expanded").await?.is_some())
        })
        .await?;
        assert_eq!("Export", export.text().await?.trim());
        focus(browser, &export).await?;
        press(browser, Key::Down).await?;
        until("Down opens Export on CSV", async || {
            Ok(focused_words(browser).await? == "CSV")
        })
        .await?;
        press(browser, Key::Down).await?;
        assert_eq!("JSON", focused_words(browser).await?);
        press(browser, Key::Up).await?;
        press(browser, Key::Enter).await?;
        let csv = gallery.url("/export/csv?");
        until("CSV submits the query's form to its address", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .starts_with(&csv))
        })
        .await?;
        // The export is a download the browser loads as a new document, never a page the
        // router draws.
        until("the query rode along", async || {
            let body = browser.element(By::Tag("body")).await?.text().await?;
            Ok(body.starts_with("csv\nSELECT "))
        })
        .await?;
        browser.console_clean("the results pane's Export").await
    })
    .await
}
