// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the 0.1.1 widgets and forms: the help switch, the picker,
//! the tree's keyboard path, the search box, the density switch, the pager
//! of a listing with no stated total and the line chart's legend.

use std::error::Error;

use thirtyfour::components::SelectElement;
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

/// Whether the element `id` is drawn larger than the one pixel a visually
/// hidden text takes.
pub(crate) async fn in_sight(browser: &Browser, id: &str) -> Result<bool, Box<dyn Error>> {
    let answer = browser
        .driver()
        .execute(
            "const box = document.getElementById(arguments[0]).getBoundingClientRect();
             return box.width > 1 && box.height > 1;",
            vec![serde_json::Value::from(id)],
        )
        .await?;
    Ok(serde_json::from_value(answer.json().clone())?)
}

/// Checks the help switch of the runner's form on the layouts page: the
/// help lines hidden from sight and still named at first, shown with the
/// switch on, hidden again with it off.
pub(crate) async fn help_switch_journey(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let switch = browser
        .element(By::Css("input#run-help[type='checkbox'][role='switch']"))
        .await?;
    assert_eq!(
        None,
        switch.attr("name").await?,
        "the form submits nothing for it"
    );
    for help in ["run-count-help", "run-inactive-help", "run-value-set-help"] {
        assert!(
            !in_sight(browser, help).await?,
            "{help} is hidden from sight at first"
        );
    }
    assert_eq!(
        Some(String::from("run-count-help")),
        attr(browser, "run-count", "aria-describedby").await?,
        "a hidden help line is still named"
    );
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![switch.to_json()?],
        )
        .await?;
    browser
        .element(By::Css("label.fui-help-switch"))
        .await?
        .click()
        .await?;
    until("the help shows", async || {
        in_sight(browser, "run-count-help").await
    })
    .await?;
    assert!(switch.is_selected().await?, "the switch reads as on");
    for help in ["run-inactive-help", "run-value-set-help"] {
        assert!(
            in_sight(browser, help).await?,
            "{help} shows with the others"
        );
    }
    assert_eq!(
        Some(String::from("run-inactive-help")),
        attr(browser, "run-inactive", "aria-describedby").await?,
        "the control names its help on every render"
    );
    switch.click().await?;
    until("the help hides again", async || {
        Ok(!in_sight(browser, "run-count-help").await?)
    })
    .await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_help_switch_shows_the_help_and_keeps_it_named() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/layouts")).await?;
        help_switch_journey(browser).await?;
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

            .execute("arguments[0].selectedIndex = arguments[0].options.length - 1; arguments[0].dispatchEvent(new Event('change', {bubbles: true}));", vec![choice.to_json()?])

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

#[tokio::test(flavor = "multi_thread")]
async fn the_pickers_select_follows_a_value_set_after_a_manual_choice() -> Result<(), Box<dyn Error>>
{
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
        // A manual choice of the value set the console sets later, then of
        // another: an option the reader chose ignores its `selected`
        // attribute from then on (HTML Living Standard, the option's
        // dirtiness), so only the select's `value` property can follow.
        for chosen in [
            "http://example.org/vs/sites",
            "http://example.org/vs/signs",
            "http://example.org/vs/sites",
        ] {
            SelectElement::new(&choice)
                .await?
                .select_by_value(chosen)
                .await?;
            until("the manual choice is written into the input", async || {
                Ok(input.prop("value").await?.as_deref() == Some(chosen))
            })
            .await?;
        }
        browser
            .element(By::Id("run-value-set-reset"))
            .await?
            .click()
            .await?;
        until("the select follows the value the console set", async || {
            Ok(choice.prop("value").await?.as_deref() == Some("http://example.org/vs/signs"))
        })
        .await?;
        assert_eq!(
            Some(String::from("http://example.org/vs/signs")),
            input.prop("value").await?
        );
        assert_eq!(
            Some(String::from("http://example.org/vs/signs")),
            SelectElement::new(&choice)
                .await?
                .first_selected_option()
                .await?
                .attr("value")
                .await?,
            "the option shown is the value's"
        );
        assert!(
            !input.is_displayed().await?,
            "a listed value keeps the input hidden"
        );
        SelectElement::new(&choice)
            .await?
            .select_by_exact_text("Another canonical")
            .await?;
        until("another opens the input", async || {
            Ok(input.is_displayed().await?)
        })
        .await?;
        assert_eq!(
            Some(String::from("http://example.org/vs/signs")),
            input.prop("value").await?,
            "another keeps the value for editing"
        );
        assert_eq!(
            "Another canonical",
            SelectElement::new(&choice)
                .await?
                .first_selected_option()
                .await?
                .text()
                .await?,
            "the select shows the choice that opened the input"
        );
        browser
            .console_clean("the picker following its value")
            .await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_chart_legend_hides_and_shows_a_series_in_place() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/charts")).await?;
        let chip = browser
            .element(By::Css(".fui-legend-chip[data-series='Writes']"))
            .await?;
        until("the legend shows once the page hydrates", async || {
            chip.is_displayed().await.map_err(Into::into)
        })
        .await?;
        let lines = browser
            .driver()
            .find_all(By::Css("[data-series-lines] .fui-chart-line"))
            .await?;
        assert_eq!(3, lines.len(), "every series is drawn");
        let writes = browser
            .element(By::Css("[data-series-lines] .fui-chart-line.fui-series-5"))
            .await?;
        assert!(writes.is_displayed().await?);
        chip.click().await?;
        until("the series hides", async || {
            Ok(!writes.is_displayed().await?)
        })
        .await?;
        assert_eq!(
            Some(String::from("false")),
            chip.attr("aria-pressed").await?
        );
        assert_eq!(
            3,
            browser
                .driver()
                .find_all(By::Css("[data-series-lines] .fui-chart-line"))
                .await?
                .len(),
            "a hidden series stays drawn"
        );
        chip.click().await?;
        until("the same line shows again", async || {
            writes.is_displayed().await.map_err(Into::into)
        })
        .await?;
        assert_eq!(Some(String::from("true")), chip.attr("aria-pressed").await?);
        browser.console_clean("the chart legend").await
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
async fn the_search_box_submits_and_the_preferences_density_posts_on_change()
-> Result<(), Box<dyn Error>> {
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
        browser.goto(&gallery.url("/ferroterm/preferences")).await?;
        preferences_hydrated(browser).await?;
        let compact = density_card(browser, "compact").await?;
        browser.click_and_load(&compact).await?;
        browser
            .at(&gallery.url("/ferroterm/preferences?saved=density"))
            .await?;
        browser
            .element(By::Css(r#"html[data-density="compact"]"#))
            .await?;
        browser
            .element(By::Css(".fui-setting-saved[role='status']"))
            .await?;
        preferences_hydrated(browser).await?;
        until("the row just saved has its choice focused", async || {
            let focused = browser.driver().active_element().await?;
            Ok(focused.attr("name").await?.as_deref() == Some("density")
                && focused.attr("value").await?.as_deref() == Some("compact"))
        })
        .await?;
        let comfortable = density_card(browser, "comfortable").await?;
        browser.click_and_load(&comfortable).await?;
        browser
            .element(By::Css(r#"html[data-density="comfortable"]"#))
            .await?;
        browser.console_clean("the search and the density").await
    })
    .await
}

/// Waits until Your preferences has hydrated: every Save button has left,
/// since a change now posts by itself.
async fn preferences_hydrated(browser: &Browser) -> Result<(), Box<dyn Error>> {
    until("the preferences post on change", async || {
        let saves = browser
            .driver()
            .find_all(By::Css("form.fui-preference-form > button[type='submit']"))
            .await?;
        for save in &saves {
            if save.is_displayed().await? {
                return Ok(false);
            }
        }
        Ok(!saves.is_empty())
    })
    .await
}

/// The density preview card that chooses `density`.
async fn density_card(browser: &Browser, density: &str) -> Result<WebElement, Box<dyn Error>> {
    let card = browser
        .element(By::XPath(format!(
            "//label[contains(@class, 'fui-choice-card')][input[@name = 'density' and @value = '{density}']]"
        )))
        .await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![card.to_json()?],
        )
        .await?;
    Ok(card)
}

#[tokio::test(flavor = "multi_thread")]
async fn your_preferences_keeps_its_keyboard_tab_off_a_phone_and_its_cards_full_width()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        crate::journeys::window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferrobridge/preferences")).await?;
        let keyboard = browser
            .element(By::Css("nav.fui-tabs a[data-tab='keyboard']"))
            .await?;
        assert!(
            !keyboard.is_displayed().await?,
            "a phone has no keyboard tab (R3)"
        );
        let widths: Vec<f64> = serde_json::from_value(
            browser
                .driver()
                .execute(
                    "const row = document.querySelector('.fui-choice-cards').getBoundingClientRect().width; \
                     return [row, ...[...document.querySelectorAll('.fui-choice-cards')[0].querySelectorAll('.fui-choice-card')].map(c => c.getBoundingClientRect().width)];",
                    vec![],
                )
                .await?
                .json()
                .clone(),
        )?;
        let (row, cards) = widths.split_first().ok_or("no preview cards")?;
        assert_eq!(3, cards.len(), "{widths:?}");
        for card in cards {
            assert!(
                (row - card).abs() < 1.0,
                "each theme card takes the phone's row: {widths:?}"
            );
        }
        let overflow: bool = serde_json::from_value(
            browser
                .driver()
                .execute(
                    "return document.documentElement.scrollWidth > document.documentElement.clientWidth;",
                    vec![],
                )
                .await?
                .json()
                .clone(),
        )?;
        assert!(!overflow, "the page reflows with no sideways scroll");
        crate::journeys::window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrobridge/preferences/keyboard"))
            .await?;
        assert!(
            browser
                .element(By::Css("nav.fui-tabs a[data-tab='keyboard']"))
                .await?
                .is_displayed()
                .await?,
            "a wide screen keeps it"
        );
        browser.console_clean("Your preferences").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn your_preferences_saved_mark_never_moves_its_control() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        crate::journeys::window(browser, 1440, 900).await?;
        let place = async |path: &str| -> Result<Vec<f64>, Box<dyn Error>> {
            browser.goto(&gallery.url(path)).await?;
            preferences_hydrated(browser).await?;
            Ok(serde_json::from_value(
                browser
                    .driver()
                    .execute(
                        "const r = document.getElementById('week-start').getBoundingClientRect(); \
                         return [r.left + window.scrollX, r.top + window.scrollY, r.width, r.height];",
                        vec![],
                    )
                    .await?
                    .json()
                    .clone(),
            )?)
        };
        let quiet = place("/ferrofed/preferences/language").await?;
        let saved = place("/ferrofed/preferences/language?saved=week-start").await?;
        browser
            .element(By::Css(".fui-setting-saved[role='status']"))
            .await?;
        assert_eq!(quiet, saved, "Saved has its own slot (R28)");
        browser.console_clean("the saved mark").await
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
        let posted = "nav[aria-label='Pages of the EHRs, in the page']";
        let linked = "nav[aria-label='Pages of the EHRs, as links']";
        let step = async |pager: &str, key: &str| -> Result<(), Box<dyn Error>> {
            let control = browser
                .element(By::Css(format!("{pager} [data-page='{key}']")))
                .await?;
            browser.click_and_load(&control).await
        };
        let range = async |pager: &str| -> Result<String, Box<dyn Error>> {
            Ok(browser
                .element(By::Css(format!("{pager} [role='status']")))
                .await?
                .text()
                .await?)
        };
        step(posted, "next").await?;
        assert_eq!(
            "EHRs 26 to 50",
            range(posted).await?,
            "Next posts the page after"
        );
        step(posted, "prev").await?;
        assert_eq!(
            "EHRs 1 to 25",
            range(posted).await?,
            "Previous posts the page before"
        );
        step(linked, "next").await?;
        assert_eq!(
            "EHRs 26 to 50",
            range(linked).await?,
            "the linked Next leads on"
        );
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

/// Whether the tree item whose row starts with `words` is open.
async fn open_item(browser: &Browser, words: &str) -> Result<bool, Box<dyn Error>> {
    let item = browser
        .element(By::XPath(format!(
            "//ul[@id = 'gallery-tree']//li[@role = 'treeitem'][starts-with(normalize-space(div[contains(@class, 'fui-tree-row')]), '{words}')]"
        )))
        .await?;
    Ok(item.attr("aria-expanded").await?.as_deref() == Some("true"))
}

/// The current address of the page.
async fn address(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser.driver().current_url().await?.to_string())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_tree_keeps_its_open_items_in_the_address_across_a_reload() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/layouts")).await?;
        let picked = browser.element(By::Id("run-value-set")).await?;
        until("the page hydrates", async || {
            Ok(!picked.is_displayed().await?)
        })
        .await?;
        assert!(open_item(browser, "Synthetic finding").await?);
        // Closing by the toggle removes the item from the address.
        let toggle = browser
            .element(By::Css("#gallery-tree > li:first-child .fui-tree-toggle"))
            .await?;
        press_until("the toggle closes the item", &toggle, async || {
            Ok(!open_item(browser, "Synthetic finding").await?)
        })
        .await?;
        until("the closed item leaves the address", async || {
            let url = address(browser).await?;
            Ok(url.contains("open=") && !url.contains("SYN-100"))
        })
        .await?;
        let closed = address(browser).await?;
        browser.goto(&closed).await?;
        assert!(
            !open_item(browser, "Synthetic finding").await?,
            "the reload opened the closed item again ({closed})"
        );
        assert!(
            browser
                .driver()
                .find_all(By::XPath(
                    "//a[contains(., 'Synthetic finding of the arm')]"
                ))
                .await?
                .is_empty(),
            "a closed item's children show after the reload"
        );
        // Opening adds the item; the reload reads its children again.
        let picked = browser.element(By::Id("run-value-set")).await?;
        until("the page hydrates again", async || {
            Ok(!picked.is_displayed().await?)
        })
        .await?;
        let branch = browser
            .element(By::XPath(
                "//ul[@id = 'gallery-tree']/li[2]//button[contains(@class, 'fui-tree-toggle')]",
            ))
            .await?;
        press_until("the toggle opens the branch", &branch, async || {
            open_item(browser, "Synthetic procedure").await
        })
        .await?;
        until("the opened item joins the address", async || {
            Ok(address(browser).await?.contains("SYN-200"))
        })
        .await?;
        let opened = address(browser).await?;
        browser.goto(&opened).await?;
        assert!(open_item(browser, "Synthetic procedure").await?);
        browser
            .text("a", "Synthetic child, read on opening")
            .await?;
        // Closing by the keyboard (Left) removes it again.
        let picked = browser.element(By::Id("run-value-set")).await?;
        until("the page hydrates a third time", async || {
            Ok(!picked.is_displayed().await?)
        })
        .await?;
        let item = browser
            .element(By::XPath("//ul[@id = 'gallery-tree']/li[2]"))
            .await?;
        until("Left closes the branch", async || {
            browser
                .driver()
                .execute("arguments[0].focus();", vec![item.to_json()?])
                .await?;
            item.send_keys(Key::Left).await?;
            Ok(!open_item(browser, "Synthetic procedure").await?)
        })
        .await?;
        until("the branch leaves the address", async || {
            Ok(!address(browser).await?.contains("SYN-200"))
        })
        .await?;
        browser.console_clean("the tree's address").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_document_views_switch_in_place_once_the_island_hydrates() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr/code")).await?;
        browser
            .element(By::Css(".fui-document-views.is-live"))
            .await?;
        let highlighted = browser.element(By::Id("document-highlighted")).await?;
        let raw = browser.element(By::Id("document-raw")).await?;
        assert!(
            highlighted.is_displayed().await?,
            "the highlighted view opens"
        );
        assert!(!raw.is_displayed().await?, "the raw view waits");
        browser
            .element(By::Css(".fui-document-switch a[data-view='raw']"))
            .await?
            .click()
            .await?;
        until("the raw view shows in place", async || {
            Ok(raw.is_displayed().await? && !highlighted.is_displayed().await?)
        })
        .await?;
        let url = browser.driver().current_url().await?;
        assert!(
            url.fragment().is_none(),
            "no jump and no history entry: {url}"
        );
        assert_eq!(
            Some(String::from("true")),
            browser
                .element(By::Css(".fui-document-switch a[data-view='raw']"))
                .await?
                .attr("aria-current")
                .await?
        );
        let rendered = browser.element(By::Id("document-rendered")).await?;
        browser
            .element(By::Css(".fui-document-switch a[data-view='rendered']"))
            .await?
            .click()
            .await?;
        until("the rendered view shows", async || {
            Ok(rendered.is_displayed().await? && !raw.is_displayed().await?)
        })
        .await?;
        browser.console_clean("the document views").await
    })
    .await
}
