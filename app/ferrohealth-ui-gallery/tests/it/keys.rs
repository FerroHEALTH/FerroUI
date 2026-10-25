// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the keys: the top bar's search and its `/` (#144), the
//! docked decision bar (#145), and the shortcut legend a console's listener
//! opens over the page (#141), each at a desktop width and on a phone.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

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

/// Moves the focus to the page itself, outside every control.
async fn blur(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute("document.activeElement.blur();", vec![])
        .await?;
    Ok(())
}

/// Waits until `/` from the page focuses the search, which also waits for
/// its island to hydrate.
async fn slash_reaches_the_search(browser: &Browser) -> Result<(), Box<dyn Error>> {
    until("the slash focuses the search", async || {
        blur(browser).await?;
        press(browser, "/").await?;
        Ok(focused(browser).await? == "gallery-search")
    })
    .await
}

/// The computed `display` of the element `css` names.
async fn display(browser: &Browser, css: &str) -> Result<String, Box<dyn Error>> {
    let shown: String = browser
        .driver()
        .execute(
            "return getComputedStyle(document.querySelector(arguments[0])).display;",
            vec![serde_json::json!(css)],
        )
        .await?
        .convert()?;
    Ok(shown)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_slash_focuses_the_search_and_escape_clears_it_keeping_the_focus()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        assert_ne!(
            "none",
            display(browser, ".fui-search-hint").await?,
            "the cap shows"
        );
        slash_reaches_the_search(browser).await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        assert_eq!(
            Some(String::new()),
            search.prop("value").await?,
            "the slash is not typed into the field"
        );
        assert_eq!(
            "none",
            display(browser, ".fui-search-hint").await?,
            "focused, the cap goes"
        );
        press(browser, "abc").await?;
        assert_eq!(Some(String::from("abc")), search.prop("value").await?);
        press(browser, Key::Escape).await?;
        assert_eq!(
            Some(String::new()),
            search.prop("value").await?,
            "Escape clears"
        );
        // WCAG 2.2 SC 2.4.3: the focus never drops to the page.
        assert_eq!(
            "gallery-search",
            focused(browser).await?,
            "Escape keeps the focus in the field"
        );
        // A slash typed into a field is the field's text.
        let field = browser.element(By::Id("long-field-1")).await?;
        browser
            .driver()
            .execute("arguments[0].focus();", vec![field.to_json()?])
            .await?;
        press(browser, "/").await?;
        assert_eq!(
            "long-field-1",
            focused(browser).await?,
            "the field keeps the focus"
        );
        let typed = field.prop("value").await?.unwrap_or_default();
        assert!(
            typed.contains('/') && typed.len() == "value-1/".len(),
            "the slash is the field's text: {typed}"
        );
        browser.console_clean("the slash").await
    })
    .await
}

/// Types `words` into the search once its island has hydrated, and waits
/// until the panel under it holds an answer.
pub(crate) async fn typed(browser: &Browser, words: &str) -> Result<WebElement, Box<dyn Error>> {
    let search = browser
        .element(By::Css("#gallery-search[role='combobox']"))
        .await?;
    search.click().await?;
    press(browser, words).await?;
    let panel = browser.element(By::Id("gallery-search-panel")).await?;
    until("the panel holds the answer", async || {
        let shown = panel.is_displayed().await?;
        let waiting = !browser
            .driver()
            .find_all(By::Css("#gallery-search-panel .fui-spinner-ring"))
            .await?
            .is_empty();
        Ok(shown && !waiting)
    })
    .await?;
    Ok(panel)
}

/// WAI-ARIA Authoring Practices 1.2, the combobox pattern: with no option
/// active Enter submits the form, and on an active option it follows it.
#[tokio::test(flavor = "multi_thread")]
async fn enter_submits_to_the_find_page_and_on_an_active_hit_follows_it()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let panel = typed(browser, "synthetic-00").await?;
        assert!(panel.is_displayed().await?, "the panel opens as you type");
        press(browser, Key::Enter).await?;
        browser
            .at(&gallery.url("/ferroterm/find?q=synthetic-00"))
            .await?;
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        typed(browser, "synthetic-00").await?;
        press(browser, Key::Down).await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        assert_eq!(
            Some(String::from("gallery-search-hit-1")),
            search.attr("aria-activedescendant").await?
        );
        press(browser, Key::Enter).await?;
        browser
            .at(&gallery.url("/ferroterm/tables?q=synthetic-001.model.yml"))
            .await?;
        browser.console_clean("Enter on the panel").await
    })
    .await
}

/// WAI-ARIA Authoring Practices 1.2, the combobox pattern: Escape dismisses
/// the popup when it is visible, and clears the field only when it is not.
#[tokio::test(flavor = "multi_thread")]
async fn escape_closes_the_answer_keeping_the_words_and_the_focus() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrofed/forms")).await?;
        let answer = typed(browser, "synthetic-00").await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        let words = search.prop("value").await?;
        assert_eq!(Some(String::from("synthetic-00")), words);
        press(browser, Key::Escape).await?;
        until("Escape closes the answer", async || {
            Ok(!answer.is_displayed().await?)
        })
        .await?;
        assert_eq!(words, search.prop("value").await?, "the words stay");
        assert_eq!("gallery-search", focused(browser).await?);
        assert_eq!(
            Some(String::from("false")),
            search.attr("aria-expanded").await?
        );
        press(browser, Key::Escape).await?;
        assert_eq!(
            Some(String::new()),
            search.prop("value").await?,
            "a second Escape clears"
        );
        assert_eq!("gallery-search", focused(browser).await?);
        browser.console_clean("Escape on the answer").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_disabled_search_takes_no_slash_and_a_phone_shows_no_cap() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferropix/overlays?search=disabled"))
            .await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        assert!(
            !search.is_enabled().await?,
            "{:?}",
            search.attr("placeholder").await?
        );
        assert_eq!(
            Some(String::from("Search waits while the index is rebuilding")),
            search.attr("placeholder").await?,
            "the field shows its reason"
        );
        assert!(
            browser
                .driver()
                .find_all(By::Css(".fui-search-hint"))
                .await?
                .is_empty(),
            "no cap while disabled"
        );
        browser.element(By::Id("open-shortcuts")).await?;
        blur(browser).await?;
        press(browser, "/").await?;
        assert_ne!("gallery-search", focused(browser).await?);
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferropix/overlays")).await?;
        let hint = browser.element(By::Css(".fui-search-hint")).await?;
        assert!(!hint.is_displayed().await?, "a phone shows no cap");
        let submit = browser.element(By::Css(".fui-search-submit")).await?;
        assert_eq!(Some(String::from("Find")), submit.attr("aria-label").await?);
        browser.console_clean("the disabled search").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_legend_opens_over_the_page_and_returns_the_focus() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        let opener = browser.element(By::Id("open-shortcuts")).await?;
        until("the opener shows once the page hydrates", async || {
            Ok(opener.is_displayed().await?)
        })
        .await?;
        let legend = browser.element(By::Id("shortcuts")).await?;
        opener.click().await?;
        until("the legend opens", async || {
            Ok(legend.is_displayed().await?)
        })
        .await?;
        assert!(
            browser
                .driver()
                .find(By::Css("dialog#shortcuts:modal"))
                .await
                .is_ok(),
            "the legend is a modal dialog over the page"
        );
        press(browser, Key::Escape).await?;
        until("the legend closes", async || {
            Ok(!legend.is_displayed().await?)
        })
        .await?;
        assert_eq!(
            "open-shortcuts",
            focused(browser).await?,
            "the focus comes back"
        );
        // The console's own listener opens it on ?, never while typing.
        blur(browser).await?;
        press(browser, "?").await?;
        until("? opens the legend", async || {
            Ok(legend.is_displayed().await?)
        })
        .await?;
        press(browser, Key::Escape).await?;
        until("the legend closes again", async || {
            Ok(!legend.is_displayed().await?)
        })
        .await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        search.click().await?;
        press(browser, "?").await?;
        assert!(
            !legend.is_displayed().await?,
            "a ? typed into a field is text"
        );
        // A phone shows no shortcut, and nothing opens the legend there.
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        let opener = browser.element(By::Id("open-shortcuts")).await?;
        let legend = browser.element(By::Id("shortcuts")).await?;
        assert!(!opener.is_displayed().await?, "a phone has no opener");
        blur(browser).await?;
        press(browser, "?").await?;
        assert!(!legend.is_displayed().await?, "? opens nothing on a phone");
        let hint = browser.element(By::Css(".fui-shortcut-hint")).await?;
        assert!(!hint.is_displayed().await?, "a phone shows no hint");
        browser.console_clean("the legend").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_save_without_a_reason_asks_for_it_and_a_save_with_one_is_done()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferroehr/forms?dock=missing"))
            .await?;
        let reason = browser.element(By::Id("dock-reason")).await?;
        assert_eq!(
            Some(String::from("true")),
            reason.attr("aria-invalid").await?
        );
        until("the reason takes the focus", async || {
            Ok(focused(browser).await? == "dock-reason")
        })
        .await?;
        press(browser, Key::Tab).await?;
        let save = browser.element(By::Css("button[value='save']")).await?;
        let active = browser.driver().active_element().await?;
        assert_eq!(
            Some(String::from("discard")),
            active.attr("value").await?,
            "the field, then Discard"
        );
        press(browser, Key::Tab).await?;
        let active = browser.driver().active_element().await?;
        assert_eq!(
            Some(String::from("save")),
            active.attr("value").await?,
            "then Save"
        );
        reason.send_keys("Synthetic night thresholds").await?;
        save.click().await?;
        browser
            .text("p", "Saved: the settings apply at once.")
            .await?;
        assert!(
            browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("decision=save"),
            "the decision is a form submit"
        );
        // A disabled Save says why as words in the bar.
        browser
            .goto(&gallery.url("/ferroehr/forms?dock=disabled"))
            .await?;
        let save = browser.element(By::Css("button[value='save']")).await?;
        assert!(!save.is_enabled().await?);
        let why = save
            .attr("aria-describedby")
            .await?
            .ok_or("Save names no reason")?;
        let line = browser.element(By::Id(&why)).await?;
        assert!(line.is_displayed().await?, "the reason shows");
        assert_eq!("Give a reason to save.", line.text().await?.trim());
        browser.console_clean("the dock").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn on_a_phone_the_dock_stacks_its_decisions_primary_first_at_the_foot()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 844).await?;
        for state in ["?dock=idle", "?dock=path"] {
            browser
                .goto(&gallery.url(&format!("/ferroterm/forms{state}")))
                .await?;
            let (save, discard, dock, bottom, wide): (f64, f64, f64, f64, f64) = browser
                .driver()
                .execute(
                    r"const top = (css) => document.querySelector(css).getBoundingClientRect().top;
                    const dock = document.querySelector('.fui-dock').getBoundingClientRect();
                    const save = document.querySelector('button[value=save]').getBoundingClientRect();
                    return [top('button[value=save]'), top('button[value=discard]'), dock.bottom, document.documentElement.clientHeight, save.width / dock.width];",
                    vec![],
                )
                .await?
                .convert()?;
            assert!(save < discard, "the primary comes first {state}: {save} {discard}");
            assert!(wide > 0.85, "a decision takes the row {state}: {wide}");
            assert!((dock - bottom).abs() < 1.0, "the dock sits at the window foot {state}: {dock} {bottom}");
            let fields = browser.driver().find_all(By::Css(".fui-dock input")).await?;
            assert_eq!(if state.ends_with("path") { 2 } else { 1 }, fields.len());
        }
        browser.console_clean("the phone's dock").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_user_menu_opens_the_legend_where_the_page_has_one() -> Result<(), Box<dyn Error>> {
    // FerroUI #413: the menu's Keyboard shortcuts opens the legend in place of its link.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        browser.hydrated().await?;
        browser
            .element(By::Css("button[popovertarget='user-menu']"))
            .await?
            .click()
            .await?;
        let shortcuts = browser
            .element(By::Css("#user-menu a[data-ferroui-shortcuts]"))
            .await?;
        until("the menu opens", async || {
            Ok(shortcuts.is_displayed().await?)
        })
        .await?;
        shortcuts.click().await?;
        until("the legend opens", async || {
            Ok(browser
                .driver()
                .find(By::Css("dialog#shortcuts:modal"))
                .await
                .is_ok())
        })
        .await?;
        browser.at(&gallery.url("/ferrofed/overlays")).await?;
        browser.console_clean("the legend from the menu").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_digit_follows_the_binder_tab_it_names() -> Result<(), Box<dyn Error>> {
    // FerroUI #413: `2` follows the second link tab of the page's first binder.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge")).await?;
        browser.hydrated().await?;
        blur(browser).await?;
        press(browser, "2").await?;
        until("the second tab's address is reached", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .fragment()
                .is_some_and(|fragment| fragment == "tabs-diagnostics"))
        })
        .await?;
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferrobridge")).await?;
        browser.hydrated().await?;
        blur(browser).await?;
        press(browser, "3").await?;
        assert!(
            browser.driver().current_url().await?.fragment().is_none(),
            "a phone follows no digit"
        );
        browser.console_clean("the digit keys").await
    })
    .await
}
