// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the top bar's typeahead: the panel anchored under the
//! field at 1440 and 390 px, floating over the page without moving it; the
//! keys that walk its options across the kinds with the focus kept in the
//! field (WAI-ARIA Authoring Practices 1.2, the combobox pattern); the
//! pointer; and the rows it shows while it waits, when nothing matched and
//! when the console refused.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::keys::typed;
use crate::stack::Gallery;

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

/// The id of the element that holds the focus.
async fn focused(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .driver()
        .active_element()
        .await?
        .attr("id")
        .await?
        .unwrap_or_default())
}

/// The id of the option the search field names active, if any.
async fn active(browser: &Browser) -> Result<Option<String>, Box<dyn Error>> {
    Ok(browser
        .element(By::Id("gallery-search"))
        .await?
        .attr("aria-activedescendant")
        .await?)
}

/// Where the panel sits against the search box and the page, as the
/// browser lays them out.
#[derive(Debug, serde::Deserialize)]
struct Placed {
    /// The search box's left edge, right edge and bottom.
    field: (f64, f64, f64),
    /// The panel's left edge, right edge, top and width.
    panel: (f64, f64, f64, f64),
    /// The window's width.
    view: f64,
    /// Whether the panel is the element drawn at its own middle, so nothing
    /// covers or clips it.
    on_top: bool,
    /// Whether the panel is out of the flow.
    floats: bool,
}

/// Measures the open panel.
async fn placed(browser: &Browser) -> Result<Placed, Box<dyn Error>> {
    let placed = browser
        .driver()
        .execute(
            r"const field = document.querySelector('.fui-topbar .fui-search').getBoundingClientRect();
            const element = document.getElementById('gallery-search-panel');
            const panel = element.getBoundingClientRect();
            const middle = document.elementFromPoint(panel.left + panel.width / 2, panel.top + Math.min(panel.height / 2, 40));
            return {
              field: [field.left, field.right, field.bottom],
              panel: [panel.left, panel.right, panel.top, panel.width],
              view: document.documentElement.clientWidth,
              on_top: !!middle && element.contains(middle),
              floats: getComputedStyle(element).position === 'absolute',
            };",
            vec![],
        )
        .await?
        .json()
        .clone();
    Ok(serde_json::from_value(placed)?)
}

/// The top of the page's heading.
async fn heading_top(browser: &Browser) -> Result<f64, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute(
            "return document.querySelector('main h1').getBoundingClientRect().top;",
            vec![],
        )
        .await?
        .convert()?)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_panel_opens_anchored_under_the_field_and_moves_nothing_on_the_page()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        let before = heading_top(browser).await?;
        typed(browser, "synthetic").await?;
        let at = placed(browser).await?;
        let (left, right, bottom) = at.field;
        let (panel_left, panel_right, top, width) = at.panel;
        assert!(at.floats, "the panel is out of the flow: {at:?}");
        assert!(at.on_top, "nothing covers or clips the panel: {at:?}");
        assert!(
            top >= bottom && top - bottom <= 12.0,
            "the panel opens right under the field: {at:?}"
        );
        assert!(
            (panel_left - left).abs() <= 1.5,
            "the panel starts at the field's start edge: {at:?}"
        );
        assert!(
            width >= right - left - 0.5,
            "at least the field's width: {at:?}"
        );
        assert!(width <= 642.5, "at most 40rem: {at:?}");
        assert!(panel_right <= at.view - 15.5, "inside the gutter: {at:?}");
        assert!(
            (heading_top(browser).await? - before).abs() < 0.5,
            "the open panel moves nothing on the page"
        );
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        let before = heading_top(browser).await?;
        typed(browser, "synthetic").await?;
        let at = placed(browser).await?;
        let (left, right, bottom) = at.field;
        let (panel_left, panel_right, top, _) = at.panel;
        assert!(at.on_top, "nothing covers or clips the panel: {at:?}");
        assert!(
            top >= bottom && top - bottom <= 12.0,
            "on a phone too the panel opens under the field: {at:?}"
        );
        assert!(
            (panel_left - left).abs() <= 1.5 && (panel_right - right).abs() <= 1.5,
            "on a phone the panel takes the field's full width: {at:?}"
        );
        assert!(panel_left >= 0.0 && panel_right <= at.view, "{at:?}");
        assert!(
            (heading_top(browser).await? - before).abs() < 0.5,
            "the open panel moves nothing on the page"
        );
        browser.console_clean("the anchored panel").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_arrows_walk_every_kind_with_the_focus_kept_in_the_field() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroehr/forms")).await?;
        typed(browser, "synthetic").await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        assert_eq!(Some(String::from("combobox")), search.attr("role").await?);
        assert_eq!(
            Some(String::from("true")),
            search.attr("aria-expanded").await?
        );
        assert_eq!(
            Some(String::from("gallery-search-results")),
            search.attr("aria-controls").await?
        );
        let options = browser
            .driver()
            .find_all(By::Css("#gallery-search-results [role='option']"))
            .await?;
        assert_eq!(
            7,
            options.len(),
            "five files, the template and every result"
        );
        let groups = browser
            .driver()
            .find_all(By::Css(
                "#gallery-search-results [role='group'][aria-labelledby]",
            ))
            .await?;
        assert_eq!(
            2,
            groups.len(),
            "one group per kind, each named by its heading"
        );
        assert_eq!(
            None,
            active(browser).await?,
            "no option is active as you type"
        );
        press(browser, Key::Down).await?;
        assert_eq!(
            Some(String::from("gallery-search-hit-1")),
            active(browser).await?
        );
        let first = browser.element(By::Id("gallery-search-hit-1")).await?;
        assert_eq!(
            Some(String::from("true")),
            first.attr("aria-selected").await?
        );
        for _ in 0..5 {
            press(browser, Key::Down).await?;
        }
        assert_eq!(
            Some(String::from("gallery-search-hit-6")),
            active(browser).await?,
            "the arrow crosses from the files to the templates"
        );
        assert_eq!(
            Some(String::from("false")),
            first.attr("aria-selected").await?,
            "one option is active at a time"
        );
        press(browser, Key::Down).await?;
        assert_eq!(
            Some(String::from("gallery-search-all")),
            active(browser).await?
        );
        press(browser, Key::Down).await?;
        assert_eq!(
            Some(String::from("gallery-search-hit-1")),
            active(browser).await?,
            "past the last option the first"
        );
        press(browser, Key::Up).await?;
        assert_eq!(
            Some(String::from("gallery-search-all")),
            active(browser).await?
        );
        assert_eq!(
            "gallery-search",
            focused(browser).await?,
            "the focus never leaves the field"
        );
        press(browser, Key::Enter).await?;
        browser
            .at(&gallery.url("/ferroehr/find?q=synthetic"))
            .await?;
        browser.console_clean("the arrows").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_pointer_highlights_and_follows_and_tab_or_a_press_outside_closes()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferropix/forms")).await?;
        let panel = typed(browser, "synthetic-00").await?;
        press(browser, Key::Tab).await?;
        until("Tab closes the panel", async || {
            Ok(!panel.is_displayed().await?)
        })
        .await?;
        browser
            .element(By::Id("gallery-search"))
            .await?
            .click()
            .await?;
        until("the field opens its answer again", async || {
            Ok(panel.is_displayed().await?)
        })
        .await?;
        browser
            .driver()
            .action_chain()
            .move_to(10, 10)
            .click()
            .perform()
            .await?;
        until("a press outside closes the panel", async || {
            Ok(!panel.is_displayed().await?)
        })
        .await?;
        browser
            .element(By::Id("gallery-search"))
            .await?
            .click()
            .await?;
        let second = browser.element(By::Id("gallery-search-hit-2")).await?;
        until("the panel opens", async || {
            Ok(second.is_displayed().await?)
        })
        .await?;
        browser
            .driver()
            .action_chain()
            .move_to_element_center(&second)
            .perform()
            .await?;
        until("the pointer highlights the hit", async || {
            Ok(second.attr("aria-selected").await?.as_deref() == Some("true"))
        })
        .await?;
        assert_eq!("gallery-search", focused(browser).await?);
        second.click().await?;
        browser
            .at(&gallery.url("/ferropix/tables?q=synthetic-002.extension.yml"))
            .await?;
        browser.console_clean("the pointer").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_panel_says_while_it_waits_when_nothing_matched_and_when_the_console_refused()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let search = browser
            .element(By::Css("#gallery-search[role='combobox']"))
            .await?;
        search.click().await?;
        press(browser, "zz-slow").await?;
        browser
            .element(By::Css(
                "#gallery-search-panel .fui-search-results-status .fui-spinner-ring",
            ))
            .await?;
        browser.text("p", "Searching").await?;
        browser.text("strong", "No file found for").await?;
        browser.text("div", "Browse every file").await?;
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        typed(browser, "zz-fail").await?;
        browser
            .text("p", "The search did not answer (503).")
            .await?;
        let errors = browser.console_errors().await?;
        assert!(
            errors.iter().all(|error| error.contains("503")),
            "only the refused answer is logged: {errors:?}"
        );
        browser.console_clean("the panel's rows").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_session_that_is_over_loads_the_signed_out_page() -> Result<(), Box<dyn Error>> {
    // RFC 9110 §15.5.2: the answer route's 401 hands the page to the console's sign-in.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        let search = browser
            .element(By::Css("#gallery-search[role='combobox']"))
            .await?;
        search.click().await?;
        press(browser, "zz-signed-out").await?;
        browser
            .at(&gallery.url("/sign-in/ferrobridge/expired"))
            .await?;
        let errors = browser.console_errors().await?;
        assert!(
            errors.iter().all(|error| error.contains("401")),
            "only the refused answer is logged: {errors:?}"
        );
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_console_names_the_option_that_submits_its_search() -> Result<(), Box<dyn Error>> {
    // FerroPIX #214: the submit runs the person search, which the panel never runs.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferropix/forms")).await?;
        typed(browser, "synthetic").await?;
        browser
            .text("span", "Search patients for “synthetic”")
            .await?;
        browser.console_clean("the named submit option").await
    })
    .await
}

/// The router draws the find page in place, so the search box lives on: a
/// submit made before the words rest drops the ask still waiting, and the
/// panel stays closed over the page (#494).
#[tokio::test(flavor = "multi_thread")]
async fn a_submit_before_the_words_rest_leaves_the_panel_closed_over_the_page()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let search = browser
            .element(By::Css("#gallery-search[role='combobox']"))
            .await?;
        search.click().await?;
        press(browser, "synthetic-00" + Key::Enter).await?;
        browser
            .at(&gallery.url("/ferroterm/find?q=synthetic-00"))
            .await?;
        tokio::time::sleep(ferrohealth_ui::widget::search::SETTLE * 5).await;
        let panel = browser
            .driver()
            .find(By::Id("gallery-search-panel"))
            .await?;
        assert!(
            !panel.is_displayed().await?,
            "the ask the submit overtook opens no panel over the find page"
        );
        assert_eq!(
            Some(String::from("false")),
            browser
                .element(By::Id("gallery-search"))
                .await?
                .attr("aria-expanded")
                .await?
        );
        browser
            .console_clean("the submit before the words rest")
            .await
    })
    .await
}
