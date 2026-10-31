// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the patient-record components: the work-list walked by
//! the keyboard with each row's actions on its one line (#540), the masked
//! identifier revealed, hidden and refused, the stacked compare's choice
//! and a timeline event's details, and the floor each keeps with its
//! scripts withheld.

use std::error::Error;

use ferrohealth_ui::script;
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

/// Opens the tables page of FerroPIX and waits until the work-list's module
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
            "the select-all shows once enhanced"
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
        browser.at(&gallery.url("/ferropix/layouts")).await?;
        Ok(())
    })
    .await
}

/// The rows of the work-list whose glance opener and row menu do not share
/// one line, or that stand taller than one line of them, each with its
/// height and the height of each cell.
async fn wrapped_rows(browser: &Browser) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute(
            r"return Array.from(document.querySelectorAll(arguments[0])).flatMap((row) => {
                const opener = row.querySelector('td.fui-row-menu button[commandfor]').getBoundingClientRect();
                const menu = row.querySelector('td.fui-row-menu summary').getBoundingClientRect();
                const height = row.getBoundingClientRect().height;
                const level = Math.abs(opener.top + opener.bottom - menu.top - menu.bottom) <= 2;
                const one = level && opener.right <= menu.left && height < 1.5 * Math.max(opener.height, menu.height);
                const tall = Array.from(row.cells).map((cell) => cell.getAttribute('data-role') + ' ' + cell.scrollHeight);
                return one ? [] : [row.getAttribute('data-key') + ' at ' + height + ' px: ' + tall.join(', ')];
              });",
            vec![serde_json::to_value(ROWS)?],
        )
        .await?
        .convert()?)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_work_list_row_keeps_its_glance_opener_and_menu_on_one_line() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        open_worklist(browser, &gallery).await?;
        browser
            .element(By::Css(
                "button[commandfor='pair-20-glance'][data-fui-ready]",
            ))
            .await?;
        let wrapped = wrapped_rows(browser).await?;
        assert!(
            wrapped.is_empty(),
            "every row keeps its opener and its menu on one line at 1440 px: {wrapped:?}"
        );
        focus_first_row(browser).await?;
        press(browser, Key::End).await?;
        until("End moves to the last row", async || {
            Ok(focused_row(browser).await? == "pair-20")
        })
        .await?;
        assert!(
            focused_row_in_view(browser).await?,
            "the last row is in view, clear of the foot"
        );
        let shown = browser
            .driver()
            .execute(
                "return document.activeElement.matches(':focus-visible');",
                vec![],
            )
            .await?;
        assert_eq!(
            Some(true),
            shown.json().as_bool(),
            "the last row shows its focus ring"
        );
        let start = browser
            .driver()
            .execute(
                "return getComputedStyle(document.getElementById('pair-20-glance')).textAlign;",
                vec![],
            )
            .await?;
        assert_eq!(
            Some("start"),
            start.json().as_str(),
            "the glance opened from the row's actions starts its lines"
        );
        // The capture centres the queue's card, so the last row shows in it
        // only while the rows keep their one line.
        let [top, bottom, view, card]: [f64; 4] = browser
            .driver()
            .execute(
                "const card = document.querySelector('.fui-list-card:has(.fui-worklist-grid)'); \
                 card.scrollIntoView({block: 'center'}); \
                 const row = document.activeElement.getBoundingClientRect(); \
                 return [row.top, row.bottom, document.documentElement.clientHeight, \
                   card.getBoundingClientRect().height];",
                vec![],
            )
            .await?
            .convert()?;
        assert!(
            top >= 0.0 && bottom <= view,
            "the last row stays in view with the queue's card centred: \
             the row spans {top} to {bottom} px of a {view} px window, the card {card} px"
        );
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

/// Opens the forms page of `brand` and waits until the identifier module
/// has enhanced every identifier on it.
pub(crate) async fn open_identifiers(
    browser: &Browser,
    gallery: &Gallery,
    brand: &str,
) -> Result<(), Box<dyn Error>> {
    browser
        .goto(&gallery.url(&format!("/{brand}/forms")))
        .await?;
    browser.hydrated().await?;
    identifiers_ready(browser).await
}

/// Waits until the identifier module has enhanced every identifier of the
/// page, which the loader marks [`script::READY`].
///
/// The loader's own mark ([`Browser::hydrated`]) is written before any
/// module has arrived, so a Reveal pressed by the keyboard at that moment
/// finds no listener, stores no focus to hand on, and the reveal page
/// opens with the focus at its top (#523).
async fn identifiers_ready(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let waiting = format!("[{}~='identifier']:not([{}])", script::HOOK, script::READY);
    until(
        "the identifier module has enhanced every identifier",
        async || {
            let left = browser
                .driver()
                .execute(
                    "return document.querySelectorAll(arguments[0]).length;",
                    vec![serde_json::Value::from(waiting.as_str())],
                )
                .await?;
            Ok(left.json().as_u64() == Some(0))
        },
    )
    .await
}

/// The Reveal of the identifier `key`: a link to the console's reveal page.
fn reveal_link(key: &str) -> By {
    By::Css(format!("#identifier-{key} a.fui-identifier-reveal"))
}

/// Reveals the identifier `key` by its link and waits for the page its
/// reveal draws.
pub(crate) async fn reveal(browser: &Browser, key: &str) -> Result<(), Box<dyn Error>> {
    let link = browser.element(reveal_link(key)).await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![link.to_json()?],
        )
        .await?;
    browser.click_and_load(&link).await?;
    browser.hydrated().await
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
        let value = browser
            .element(By::Css("#identifier-bsn .fui-identifier-value"))
            .await?;
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

/// Whether the focused element matches the CSS `selector`.
async fn focus_is(browser: &Browser, selector: &str) -> Result<bool, Box<dyn Error>> {
    let held = browser
        .driver()
        .execute(
            "return document.activeElement !== null && document.activeElement.matches(arguments[0]);",
            vec![serde_json::Value::from(selector)],
        )
        .await?;
    Ok(held.json().as_bool().unwrap_or(false))
}

#[tokio::test(flavor = "multi_thread")]
async fn the_focus_follows_reveal_hide_and_a_refusal_to_the_face_that_shows()
-> Result<(), Box<dyn Error>> {
    // NOTE: WCAG 2.2 SC 2.4.3: a pressed button that goes away hands the focus to the
    // button of the face that takes its place, never to the page.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        open_identifiers(browser, &gallery, "ferropix").await?;
        let reveal = browser.element(reveal_link("bsn")).await?;
        browser
            .driver()
            .execute(
                "arguments[0].scrollIntoView({block: 'center'}); arguments[0].focus();",
                vec![reveal.to_json()?],
            )
            .await?;
        let old = browser.loader().await?;
        press(browser, Key::Enter).await?;
        until("the reveal page loads", async || {
            Ok(browser.loader().await? != old)
        })
        .await?;
        browser.hydrated().await?;
        identifiers_ready(browser).await?;
        until("the focus moves to Hide once the value shows", async || {
            focus_is(browser, "#identifier-bsn button[aria-pressed='true']").await
        })
        .await?;
        press(browser, Key::Enter).await?;
        until("the focus returns to Reveal once hidden", async || {
            focus_is(browser, "#identifier-bsn a.fui-identifier-reveal").await
        })
        .await?;
        let refused = browser.element(reveal_link("ssn")).await?;
        browser
            .driver()
            .execute(
                "arguments[0].scrollIntoView({block: 'center'}); arguments[0].focus();",
                vec![refused.to_json()?],
            )
            .await?;
        let old = browser.loader().await?;
        press(browser, Key::Enter).await?;
        until("the refusal's page loads", async || {
            Ok(browser.loader().await? != old)
        })
        .await?;
        browser.hydrated().await?;
        identifiers_ready(browser).await?;
        browser
            .text("span", "Not revealed: the hourly limit is reached")
            .await?;
        until("the focus returns to Reveal after a refusal", async || {
            focus_is(browser, "#identifier-ssn a.fui-identifier-reveal").await
        })
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

/// The floor of the forms page's identifiers with no script: each
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

/// The floor of the tables page's work-list with no script: a table
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
        "the select-all waits for its module"
    );
    Ok(())
}
