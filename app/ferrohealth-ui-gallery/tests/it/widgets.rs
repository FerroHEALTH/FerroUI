// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the 0.1.1 widgets and forms: the help switch, the picker,
//! the tree's keyboard path, the search box, the density switch and the
//! pager of a listing with no stated total.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// Presses the button `button` until `done` holds, which also waits for
/// the page to hydrate.
async fn press_until(
    what: &str,
    button: &WebElement,
    done: impl AsyncFn() -> Result<bool, Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    until(what, async || {
        if !done().await? {
            button.click().await?;
        }
        done().await
    })
    .await
}

/// The attribute `name` of the element `id`.
async fn attr(browser: &Browser, id: &str, name: &str) -> Result<Option<String>, Box<dyn Error>> {
    Ok(browser.element(By::Id(id)).await?.attr(name).await?)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_help_switch_shows_the_help_and_keeps_it_named() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/layouts")).await?;
        let help = browser.element(By::Id("run-count-help")).await?;
        let hidden = async || -> Result<bool, Box<dyn Error>> {
            Ok(help
                .attr("class")
                .await?
                .is_some_and(|class| class.contains("fui-sr-only")))
        };
        assert!(hidden().await?, "visually hidden at first, still read");
        assert_eq!(
            Some(String::from("run-count-help")),
            attr(browser, "run-count", "aria-describedby").await?
        );
        let switch = browser
            .element(By::XPath(
                "//button[normalize-space(.) = 'Explain these parameters']",
            ))
            .await?;
        press_until("the help shows", &switch, async || Ok(!hidden().await?)).await?;
        assert_eq!(
            Some(String::from("true")),
            switch.attr("aria-pressed").await?
        );
        assert_eq!(
            Some(String::from("run-count-help")),
            attr(browser, "run-count", "aria-describedby").await?,
            "the control names its help on every render"
        );
        browser.console_clean("the help switch").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_picker_writes_a_choice_and_keeps_a_typed_value() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/layouts")).await?;
        let choice = browser.element(By::Id("run-value-set-choice")).await?;
        let input = browser.element(By::Id("run-value-set")).await?;
        until("the picker hydrates and hides its input", async || {
            Ok(!input.is_displayed().await?)
        })
        .await?;
        browser

            .driver()

            .execute("arguments[0].value = 'http://example.org/vs/sites'; arguments[0].dispatchEvent(new Event('change', {bubbles: true}));", vec![choice.to_json()?])

            .await?;
        until("the choice is written into the input", async || {
            Ok(input.prop("value").await?.as_deref() == Some("http://example.org/vs/sites"))
        })
        .await?;
        browser

            .driver()

            .execute("arguments[0].value = arguments[0].options[arguments[0].options.length - 1].value; arguments[0].dispatchEvent(new Event('change', {bubbles: true}));", vec![choice.to_json()?])

            .await?;
        until("another opens the input", async || Ok(input.is_displayed().await?)).await?;
        input.clear().await?;
        input.send_keys("http://example.org/vs/mine").await?;
        assert!(input.is_displayed().await?, "an unlisted value stays on the input");
        assert_eq!(
            Some(String::from("http://example.org/vs/mine")),
            input.prop("value").await?
        );
        assert_eq!(None, choice.attr("name").await?, "the select submits nothing");
        browser.console_clean("the picker").await
    })
    .await
}

/// The row text of the tree item that holds the focus, or `None` when the
/// focus is elsewhere.
async fn focused_row(browser: &Browser) -> Result<Option<String>, Box<dyn Error>> {
    let active = browser.driver().active_element().await?;
    if active.attr("role").await?.as_deref() != Some("treeitem") {
        return Ok(None);
    }
    Ok(Some(
        active.find(By::Css(".fui-tree-row")).await?.text().await?,
    ))
}

#[tokio::test(flavor = "multi_thread")]
async fn the_tree_walks_by_the_keyboard_and_reads_children_on_opening() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr/layouts")).await?;
        let picked = browser.element(By::Id("run-value-set")).await?;
        until("the page hydrates", async || {
            Ok(!picked.is_displayed().await?)
        })
        .await?;
        let items = || {
            browser
                .driver()
                .find_all(By::Css("#gallery-tree [role='treeitem']"))
        };
        let stops = browser
            .driver()
            .find_all(By::Css("#gallery-tree [tabindex='0']"))
            .await?;
        assert_eq!(1, stops.len(), "one tab stop");
        let focused = async || focused_row(browser).await;
        until("the tree answers the keyboard", async || {
            let first = items().await?.remove(0);
            browser
                .driver()
                .execute("arguments[0].focus();", vec![first.to_json()?])
                .await?;
            first.send_keys(Key::Down).await?;
            Ok(focused()
                .await?
                .is_some_and(|text| text.contains("SYN-110")))
        })
        .await?;
        let active = browser.driver().active_element().await?;
        active.send_keys(Key::End).await?;
        until("End moves to the last item", async || {
            Ok(focused()
                .await?
                .is_some_and(|text| text.contains("SYN-300")))
        })
        .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Up)
            .await?;
        until("Up moves to the closed branch", async || {
            Ok(focused()
                .await?
                .is_some_and(|text| text.starts_with("Synthetic procedure")))
        })
        .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Right)
            .await?;
        browser
            .text("a", "Synthetic child, read on opening")
            .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Home)
            .await?;
        until("Home moves to the first item", async || {
            Ok(focused()
                .await?
                .is_some_and(|text| text.starts_with("Synthetic finding")))
        })
        .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Down)
            .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Enter)
            .await?;
        browser
            .at(&gallery.url("/ferroehr/layouts?concept=SYN-110"))
            .await?;
        browser
            .element(By::Css("#gallery-tree [aria-selected='true']"))
            .await?;
        browser.console_clean("the tree").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_search_box_submits_and_the_density_switch_posts() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm")).await?;
        crate::journeys::hydrated_tabs(browser).await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        search.send_keys("synthetic-007").await?;
        search.send_keys(Key::Enter).await?;
        browser
            .at(&gallery.url("/ferroterm/tables?q=synthetic-007"))
            .await?;
        browser.goto(&gallery.url("/ferroterm")).await?;
        crate::journeys::hydrated_tabs(browser).await?;
        browser
            .element(By::XPath(
                "//button[@name = 'density' and @value = 'compact']",
            ))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css(r#"html[data-density="compact"]"#))
            .await?;
        browser
            .element(By::XPath(
                "//button[@value = 'compact' and @aria-pressed = 'true']",
            ))
            .await?;
        browser
            .element(By::XPath(
                "//button[@name = 'density' and @value = 'comfortable']",
            ))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css(r#"html[data-density="comfortable"]"#))
            .await?;
        browser.console_clean("the search and the density").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_open_pager_steps_with_no_total() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr/tables")).await?;
        let pager = browser
            .element(By::Css("nav[aria-label='Pages of the EHRs, in the page']"))
            .await?;
        let next = pager.find(By::Css("[data-page='next']")).await?;
        let status = pager.find(By::Css("[role='status']")).await?;
        press_until("the next page shows", &next, async || {
            Ok(status.text().await? == "EHRs 26 to 50")
        })
        .await?;
        pager
            .find(By::Css("[data-page='prev']"))
            .await?
            .click()
            .await?;
        until("the first page shows", async || {
            Ok(status.text().await? == "EHRs 1 to 25")
        })
        .await?;
        let linked = browser
            .element(By::Css(
                "nav[aria-label='Pages of the EHRs, as links'] a[data-page='next']",
            ))
            .await?;
        linked.click().await?;
        browser.text("p", "EHRs 26 to 50").await?;
        browser.console_clean("the open pager").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_combobox_filters_as_typed_and_the_keys_choose() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        // Hydrated, the native select gives way to the combobox.
        let input = browser
            .element(By::Css("#choose-system[role='combobox']"))
            .await?;
        assert_eq!(
            Some(String::from("SNOMED CT")),
            input.prop("value").await?,
            "the chosen option's words show"
        );
        input.click().await?;
        until("the list opens", async || {
            Ok(input.attr("aria-expanded").await?.as_deref() == Some("true"))
        })
        .await?;
        input.send_keys("loinc").await?;
        let options = browser
            .driver()
            .find_all(By::Css("#choose-system-list [role='option']"))
            .await?;
        assert_eq!(1, options.len(), "typing keeps the matching option alone");
        input.send_keys(Key::Escape).await?;
        until("Escape closes the list", async || {
            Ok(input.attr("aria-expanded").await?.as_deref() == Some("false"))
        })
        .await?;
        input.send_keys(Key::Down).await?;
        input.send_keys(Key::End).await?;
        let active = input.attr("aria-activedescendant").await?;
        assert_eq!(Some(String::from("choose-system-option-11")), active);
        input.send_keys(Key::Enter).await?;
        let posted = browser
            .element(By::Css("input[type='hidden'][name='system']"))
            .await?;
        until("Enter chooses the highlighted option", async || {
            Ok(posted.prop("value").await?.as_deref() == Some("synthetic"))
        })
        .await?;
        input.clear().await?;
        input.send_keys("nothing like it").await?;
        browser.element(By::Css(".fui-combobox-empty")).await?;
        browser.console_clean("the combobox").await
    })
    .await
}
