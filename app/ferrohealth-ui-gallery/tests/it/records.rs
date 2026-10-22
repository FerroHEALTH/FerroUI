// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the patient-record components: the work-list walked by
//! the keyboard, the masked identifier revealed, hidden and refused, the
//! stacked compare's choice and a timeline event's details, and the floor
//! each keeps with its WebAssembly withheld.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The CSS of the work-list's rows.
const ROWS: &str = ".fui-worklist-grid tbody tr[data-key]";

/// Presses `keys` on whatever holds the focus.
async fn press(browser: &Browser, keys: impl Into<TypingData>) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(keys)
        .perform()
        .await?;
    Ok(())
}

/// The `data-key` of the row that holds the focus, if a row does.
async fn focused_row(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .driver()
        .active_element()
        .await?
        .attr("data-key")
        .await?
        .unwrap_or_default())
}

/// The `data-key` of the row marked `aria-current`.
async fn current_row(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .element(By::Css(format!("{ROWS}[aria-current='true']")))
        .await?
        .attr("data-key")
        .await?
        .unwrap_or_default())
}

/// Whether the focused row lies clear of the viewport's edges and above
/// the work-list's foot.
async fn focused_row_in_view(browser: &Browser) -> Result<bool, Box<dyn Error>> {
    let clear = browser
        .driver()
        .execute(
            "const row = document.activeElement.getBoundingClientRect(); \
             const foot = document.querySelector('.fui-worklist-foot').getBoundingClientRect(); \
             const view = document.documentElement.clientHeight; \
             return row.top >= 0 && row.bottom <= Math.min(view, foot.top) + 1;",
            vec![],
        )
        .await?;
    Ok(clear.json().as_bool() == Some(true))
}

/// Opens the tables page of FerroPIX and waits until the work-list's island
/// has made its table a grid.
pub(crate) async fn open_worklist(
    browser: &Browser,
    gallery: &Gallery,
) -> Result<(), Box<dyn Error>> {
    browser.goto(&gallery.url("/ferropix/tables")).await?;
    browser
        .element(By::Css(".fui-worklist-grid table[role='grid']"))
        .await?;
    Ok(())
}

/// Gives the first row of the work-list the focus.
pub(crate) async fn focus_first_row(browser: &Browser) -> Result<WebElement, Box<dyn Error>> {
    let first = browser
        .element(By::Css(format!("{ROWS}[tabindex='0']")))
        .await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'}); arguments[0].focus();",
            vec![first.to_json()?],
        )
        .await?;
    Ok(first)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_work_list_moves_its_focused_row_by_the_keys_and_opens_it() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        open_worklist(browser, &gallery).await?;
        focus_first_row(browser).await?;
        assert_eq!("pair-01", focused_row(browser).await?);
        press(browser, "j").await?;
        until("j moves to the second row", async || {
            Ok(focused_row(browser).await? == "pair-02")
        })
        .await?;
        assert_eq!(
            "pair-02",
            current_row(browser).await?,
            "aria-current follows the focus"
        );
        press(browser, Key::Down).await?;
        until("Down moves to the third row", async || {
            Ok(focused_row(browser).await? == "pair-03")
        })
        .await?;
        press(browser, "k").await?;
        press(browser, Key::Up).await?;
        until("k and Up move back to the first row", async || {
            Ok(focused_row(browser).await? == "pair-01")
        })
        .await?;
        // NOTE: WAI-ARIA Authoring Practices 1.2, the grid pattern: one tab stop.
        let stops = browser
            .driver()
            .find_all(By::Css(format!("{ROWS}[tabindex='0']")))
            .await?;
        assert_eq!(1, stops.len(), "one row is the tab stop");
        press(browser, Key::End).await?;
        until("End moves to the last row", async || {
            Ok(focused_row(browser).await? == "pair-20")
        })
        .await?;
        // NOTE: WCAG 2.2 SC 2.4.11: the focused row is not hidden under the sticky foot.
        assert!(
            focused_row_in_view(browser).await?,
            "the last row is in view, clear of the foot"
        );
        press(browser, Key::Home).await?;
        until("Home moves to the first row", async || {
            Ok(focused_row(browser).await? == "pair-01")
        })
        .await?;
        assert!(
            focused_row_in_view(browser).await?,
            "the first row is in view again"
        );
        let header = browser
            .element(By::Css(".fui-worklist-grid thead .fui-select-all"))
            .await?;
        assert!(
            header.is_displayed().await?,
            "the select-all shows once hydrated"
        );
        let mixed = browser
            .driver()
            .execute(
                "return arguments[0].indeterminate;",
                vec![header.to_json()?],
            )
            .await?;
        assert_eq!(
            Some(true),
            mixed.json().as_bool(),
            "two of twenty selected is mixed"
        );
        header.click().await?;
        let selected = browser
            .driver()
            .find_all(By::Css(format!("{ROWS}[aria-selected='true']")))
            .await?;
        assert_eq!(20, selected.len(), "the select-all selects every row");
        focus_first_row(browser).await?;
        press(browser, Key::Enter).await?;
        browser.at(&gallery.url("/ferropix/templates")).await?;
        Ok(())
    })
    .await
}

/// The size of the row of the identifier `key`, width and height.
async fn row_size(browser: &Browser, key: &str) -> Result<String, Box<dyn Error>> {
    let rect = browser
        .element(By::Css(format!("#identifiers tr[data-key='{key}']")))
        .await?
        .rect()
        .await?;
    Ok(format!("{:.0} by {:.0}", rect.width, rect.height))
}

/// Opens the forms page of `brand` and waits until the identifiers island
/// has hydrated, its Reveal a button.
pub(crate) async fn open_identifiers(
    browser: &Browser,
    gallery: &Gallery,
    brand: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{brand}/forms")))
        .await?;
    let press = browser
        .element(By::Css("#identifier-bsn .fui-identifier-press"))
        .await?;
    until("the identifiers have hydrated", async || {
        Ok(press.attr("hidden").await?.is_none())
    })
    .await
}

/// Reveals the identifier `key` and waits for its countdown.
pub(crate) async fn reveal(browser: &Browser, key: &str) -> Result<(), Box<dyn Error>> {
    let button = browser
        .element(By::Css(format!(
            "#identifier-{key} button[aria-pressed='false']"
        )))
        .await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![button.to_json()?],
        )
        .await?;
    button.click().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn an_identifier_reveals_counts_down_and_hides_without_moving_its_row()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        open_identifiers(browser, &gallery, "ferropix").await?;
        let before = row_size(browser, "bsn").await?;
        let value = browser
            .element(By::Css("#identifier-bsn .fui-identifier-value"))
            .await?;
        assert!(
            !value.text().await?.contains("999"),
            "the value is not on the page while masked"
        );
        reveal(browser, "bsn").await?;
        until("the value shows", async || {
            Ok(value.text().await?.contains("999 000 124"))
        })
        .await?;
        let countdown = browser
            .element(By::Css("#identifier-bsn .fui-identifier-countdown"))
            .await?;
        assert!(
            countdown.text().await?.contains("s left"),
            "the countdown shows"
        );
        // NOTE: the reserved slot (#140): revealing changes no size around it.
        assert_eq!(
            before,
            row_size(browser, "bsn").await?,
            "revealing moves nothing"
        );
        browser
            .element(By::Css("#identifier-bsn button[aria-pressed='true']"))
            .await?
            .click()
            .await?;
        until("Hide masks it again", async || {
            Ok(!value.text().await?.contains("999"))
        })
        .await?;
        assert_eq!(
            before,
            row_size(browser, "bsn").await?,
            "hiding moves nothing"
        );
        reveal(browser, "ssn").await?;
        browser
            .text("span", "Not revealed: the hourly limit is reached")
            .await?;
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_stacked_compare_reads_the_record_chosen_and_an_event_opens()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferropix/tables")).await?;
        let parts = browser
            .driver()
            .find_all(By::Css(".fui-compare-alone .fui-revealed"))
            .await?;
        assert_eq!(3, parts.len(), "one part per other record");
        assert!(parts[0].is_displayed().await? && !parts[1].is_displayed().await?);
        browser
            .element(By::Css("label[for='records-compare-stacked-read-1']"))
            .await?
            .click()
            .await?;
        until("the second record shows", async || {
            Ok(parts[1].is_displayed().await?)
        })
        .await?;
        assert!(!parts[0].is_displayed().await?, "one record at a time");
        browser.goto(&gallery.url("/ferropix/layouts")).await?;
        let details = browser.element(By::Css(".fui-timeline details")).await?;
        details.find(By::Css("summary")).await?.click().await?;
        until("the event's details open", async || {
            Ok(details.attr("open").await?.is_some())
        })
        .await?;
        Ok(())
    })
    .await
}

/// The floor of the forms page's identifiers with no WebAssembly: each
/// Reveal is a link to the console's reveal page, and no value is shown.
pub(crate) async fn identifiers_floor(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let link = browser
        .element(By::Css("#identifier-bsn a.fui-identifier-reveal"))
        .await?;
    assert!(
        link.is_displayed().await?,
        "the reveal link stands in for the button"
    );
    let href = link.attr("href").await?.unwrap_or_default();
    assert!(href.contains("?reveal=bsn"), "{href}");
    let table = browser.element(By::Id("identifiers")).await?.text().await?;
    assert!(!table.contains("999 000 124"), "no value reaches the page");
    Ok(())
}

/// The floor of the tables page's work-list with no WebAssembly: a table
/// whose rows open by their links, with the select-all hidden.
pub(crate) async fn worklist_floor(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let rows = browser.driver().find_all(By::Css(ROWS)).await?;
    assert_eq!(20, rows.len(), "every row of the queue shows");
    browser
        .element(By::Css(".fui-worklist-grid tbody a.fui-row-link"))
        .await?;
    let header = browser
        .element(By::Css(".fui-worklist-grid thead .fui-select-all"))
        .await?;
    assert!(
        !header.is_displayed().await?,
        "the select-all waits for its island"
    );
    Ok(())
}
