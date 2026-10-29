// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the date field and its picker (R23, R25) and of a
//! history filter's range (R22): the grid's keyboard path, the days a field
//! forbids, a typed text that is no date, the quick choices, the presets
//! and the phone's bottom sheet. Each follows the WAI-ARIA Authoring
//! Practices date picker dialog pattern.

use std::error::Error;

use serde_json::Value;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The forms page of the brand the journeys use.
const FORMS: &str = "/ferropix/forms";

/// The calendar button of the date field `id`, once the page has been enhanced
/// and turned it on.
pub(crate) async fn calendar(browser: &Browser, id: &str) -> Result<WebElement, Box<dyn Error>> {
    let button = browser
        .element(By::XPath(format!(
            "//input[@id = '{id}']/following-sibling::button"
        )))
        .await?;
    until(
        "the calendar turns on as the page is enhanced",
        async || Ok(button.is_enabled().await?),
    )
    .await?;
    Ok(button)
}

/// Opens the picker of the date field `id` and returns it, pressing its
/// button again should the page still be scrolling to a focus it moved.
pub(crate) async fn open_picker(browser: &Browser, id: &str) -> Result<WebElement, Box<dyn Error>> {
    let button = calendar(browser, id).await?;
    let picker = By::Id(format!("{id}-picker"));
    until("the picker opens", async || {
        if browser.driver().find_all(picker.clone()).await?.is_empty() {
            browser
                .driver()
                .execute(
                    "arguments[0].scrollIntoView({block: 'center'});",
                    vec![button.to_json()?],
                )
                .await?;
            if button.click().await.is_err() {
                return Ok(false);
            }
        }
        Ok(!browser.driver().find_all(picker.clone()).await?.is_empty())
    })
    .await?;
    browser.element(picker).await
}

/// Opens the range picker of the forms page and returns it.
pub(crate) async fn open_range(browser: &Browser) -> Result<WebElement, Box<dyn Error>> {
    let button = browser
        .element(By::XPath(
            "//form[@id = 'date-range']//button[@aria-haspopup = 'dialog']",
        ))
        .await?;
    until(
        "the range button shows as the page is enhanced",
        async || Ok(button.is_displayed().await?),
    )
    .await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![button.to_json()?],
        )
        .await?;
    button.click().await?;
    browser.element(By::Id("audit-range-picker")).await
}

/// The `data-day` of the element that holds the keyboard focus, if any.
async fn focused_day(browser: &Browser) -> Result<Option<String>, Box<dyn Error>> {
    let value = browser
        .driver()
        .execute(
            "return document.activeElement.getAttribute('data-day');",
            vec![],
        )
        .await?;
    Ok(value.json().as_str().map(str::to_owned))
}

/// The id of the element that holds the keyboard focus.
async fn focused_id(browser: &Browser) -> Result<String, Box<dyn Error>> {
    let value = browser
        .driver()
        .execute("return document.activeElement.id;", vec![])
        .await?;
    Ok(value.json().as_str().unwrap_or_default().to_owned())
}

/// Presses `key` on the element that holds the focus.
async fn press_key(browser: &Browser, key: Key) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(key)
        .perform()
        .await?;
    Ok(())
}

/// Waits until the focus is on the day `day`.
async fn focus_on(browser: &Browser, day: &str) -> Result<(), Box<dyn Error>> {
    until(&format!("the focus moves to {day}"), async || {
        Ok(focused_day(browser).await?.as_deref() == Some(day))
    })
    .await
}

/// The button of `day` in the picker.
async fn day_button(picker: &WebElement, day: &str) -> Result<WebElement, Box<dyn Error>> {
    Ok(picker
        .find(By::Css(format!("button[data-day='{day}']")))
        .await?)
}

/// The value the input `id` holds.
async fn value_of(browser: &Browser, id: &str) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .element(By::Id(id))
        .await?
        .prop("value")
        .await?
        .unwrap_or_default())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_grid_moves_by_the_keys_and_a_chosen_day_fills_the_field() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url(FORMS)).await?;
        let picker = open_picker(browser, "audit-from").await?;
        assert_eq!(Some(String::from("dialog")), picker.attr("role").await?);
        focus_on(browser, "2026-10-14").await?;
        let chosen = day_button(&picker, "2026-10-14").await?;
        assert_eq!(
            Some(String::from("Wednesday 14 October 2026")),
            chosen.attr("aria-label").await?,
            "a day is named by its full date"
        );
        press_key(browser, Key::Right).await?;
        focus_on(browser, "2026-10-15").await?;
        press_key(browser, Key::Up).await?;
        focus_on(browser, "2026-10-08").await?;
        press_key(browser, Key::Home).await?;
        focus_on(browser, "2026-10-05").await?;
        press_key(browser, Key::End).await?;
        focus_on(browser, "2026-10-11").await?;
        press_key(browser, Key::PageUp).await?;
        focus_on(browser, "2026-09-11").await?;
        press_key(browser, Key::Enter).await?;
        until("the chosen day fills the field", async || {
            Ok(value_of(browser, "audit-from").await? == "2026-09-11")
        })
        .await?;
        picker
            .find(By::XPath(".//button[normalize-space(.) = 'Apply']"))
            .await?
            .click()
            .await?;
        until("the focus returns to the field", async || {
            Ok(focused_id(browser).await? == "audit-from")
        })
        .await?;
        assert!(
            browser
                .driver()
                .find_all(By::Id("audit-from-picker"))
                .await?
                .is_empty(),
            "Apply closes the picker"
        );
        browser.console_clean("the date picker's keys").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn escape_puts_the_field_back_and_returns_the_focus() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url(FORMS)).await?;
        let picker = open_picker(browser, "audit-from").await?;
        day_button(&picker, "2026-10-02").await?.click().await?;
        until("the click fills the field", async || {
            Ok(value_of(browser, "audit-from").await? == "2026-10-02")
        })
        .await?;
        press_key(browser, Key::Escape).await?;
        until("Escape puts the field back", async || {
            Ok(value_of(browser, "audit-from").await? == "2026-10-14")
        })
        .await?;
        until("the focus returns to the field", async || {
            Ok(focused_id(browser).await? == "audit-from")
        })
        .await?;
        browser.console_clean("the date picker's Escape").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_field_that_forbids_the_future_offers_no_later_day_or_month() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url(FORMS)).await?;
        let picker = open_picker(browser, "audit-from").await?;
        assert!(
            day_button(&picker, "2026-10-21")
                .await?
                .is_enabled()
                .await?,
            "today is allowed"
        );
        assert!(
            !day_button(&picker, "2026-10-22")
                .await?
                .is_enabled()
                .await?,
            "tomorrow is disabled"
        );
        let next = picker
            .find(By::Css("button[aria-label='Next month']"))
            .await?;
        assert!(!next.is_enabled().await?, "no later month is offered");
        picker
            .find(By::Css("button.fui-datepicker-title"))
            .await?
            .click()
            .await?;
        let november = picker
            .find(By::Css("button[aria-label='November 2026']"))
            .await?;
        assert!(
            !november.is_enabled().await?,
            "the chooser offers no later month"
        );
        let september = picker
            .find(By::Css("button[aria-label='September 2026']"))
            .await?;
        september.click().await?;
        picker
            .find(By::Css("table[aria-label='September 2026']"))
            .await?;
        browser.console_clean("the forbidden future").await
    })
    .await
}

/// What the title of `picker` draws after its text: the path of the glyph
/// it ends with, whether that glyph is hidden from assistive technology,
/// whether it is drawn in the title's own colour, and the text before it.
async fn title_glyph(browser: &Browser, picker: &WebElement) -> Result<Value, Box<dyn Error>> {
    let title = picker.find(By::Css("button.fui-datepicker-title")).await?;
    let found = browser
        .driver()
        .execute(
            "const title = arguments[0];
             const svg = title.lastElementChild;
             if (!svg || svg.tagName.toLowerCase() !== 'svg') return null;
             const box = svg.getBoundingClientRect();
             return {
               path: svg.querySelector('path').getAttribute('d'),
               hidden: svg.getAttribute('aria-hidden'),
               stroke: svg.getAttribute('stroke'),
               same_colour: getComputedStyle(svg).color === getComputedStyle(title).color,
               drawn: box.width > 0 && box.height > 0,
               text: title.textContent.trim(),
               label: title.getAttribute('aria-label'),
             };",
            vec![title.to_json()?],
        )
        .await?;
    Ok(found.json().clone())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_month_title_shows_the_chevron_of_the_chooser_it_opens() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url(FORMS)).await?;
        let picker = open_picker(browser, "audit-from").await?;
        let month = title_glyph(browser, &picker).await?;
        assert_eq!(
            Some("m6 9 6 6 6-6"),
            month["path"].as_str(),
            "the month title ends with the chevron-down glyph: {month}"
        );
        assert_eq!(
            Some("true"),
            month["hidden"].as_str(),
            "the glyph is hidden, the title's name says the action (WCAG 2.2 SC 1.1.1): {month}"
        );
        assert_eq!(Some("currentColor"), month["stroke"].as_str(), "{month}");
        assert_eq!(
            Some(true),
            month["same_colour"].as_bool(),
            "the glyph takes the title's colour: {month}"
        );
        assert_eq!(Some(true), month["drawn"].as_bool(), "{month}");
        assert_eq!(Some("October 2026"), month["text"].as_str(), "{month}");
        assert_eq!(
            Some("Choose the month and year, now October 2026"),
            month["label"].as_str(),
            "{month}"
        );
        picker
            .find(By::Css("button.fui-datepicker-title"))
            .await?
            .click()
            .await?;
        picker.find(By::Css("div.fui-datepicker-months")).await?;
        let year = title_glyph(browser, &picker).await?;
        assert_eq!(
            Some("m18 15-6-6-6 6"),
            year["path"].as_str(),
            "the year title, which goes back to the days, ends with chevron-up: {year}"
        );
        assert_eq!(Some("true"), year["hidden"].as_str(), "{year}");
        assert_eq!(Some(true), year["same_colour"].as_bool(), "{year}");
        assert_eq!(Some("2026"), year["text"].as_str(), "{year}");
        browser.console_clean("the month title's chevron").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_typed_text_that_is_no_date_disables_apply_and_says_why() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url(FORMS)).await?;
        calendar(browser, "audit-from").await?;
        let input = browser.element(By::Id("audit-from")).await?;
        input.clear().await?;
        input.send_keys("2026-10-41").await?;
        let error = browser.element(By::Id("audit-from-error")).await?;
        until("the field says why", async || {
            Ok(error.text().await?.contains("October has 31 days"))
        })
        .await?;
        assert_eq!(
            Some(String::from("true")),
            input.attr("aria-invalid").await?
        );
        let picker = open_picker(browser, "audit-from").await?;
        let apply = picker
            .find(By::XPath(".//button[normalize-space(.) = 'Apply']"))
            .await?;
        assert!(!apply.is_enabled().await?, "Apply is disabled");
        let status = picker.find(By::Css("p[role='status']")).await?;
        assert_eq!("Type a date as YYYY-MM-DD.", status.text().await?.trim());
        day_button(&picker, "2026-10-13").await?.click().await?;
        until("a chosen day makes the field valid", async || {
            Ok(apply.is_enabled().await?)
        })
        .await?;
        browser.console_clean("an invalid typed date").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_quick_choice_fills_the_field_and_the_line_names_the_day() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url(FORMS)).await?;
        let picker = open_picker(browser, "defer-until").await?;
        assert!(
            !day_button(&picker, "2026-10-21")
                .await?
                .is_enabled()
                .await?,
            "a deferral cannot be today"
        );
        picker
            .find(By::XPath(".//button[normalize-space(.) = 'Next Monday']"))
            .await?
            .click()
            .await?;
        until("the quick choice fills the field", async || {
            Ok(value_of(browser, "defer-until").await? == "2026-10-26")
        })
        .await?;
        let status = picker.find(By::Css("p[role='status']")).await?;
        assert_eq!(
            "Back in the queue on Monday 26 October 2026.",
            status.text().await?.trim()
        );
        browser.console_clean("a quick choice").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_range_takes_a_preset_or_two_days_and_apply_submits_it() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url(FORMS)).await?;
        let picker = open_range(browser).await?;
        until("the range opens inside the screen", async || {
            Ok(picker.rect().await?.x >= 0.0)
        })
        .await?;
        let status = picker.find(By::Css("p[role='status']")).await?;
        picker
            .find(By::XPath(".//button[normalize-space(.) = 'Last 30 days']"))
            .await?
            .click()
            .await?;
        until("the preset fills the range", async || {
            Ok(status.text().await?.trim() == "2026-09-22 to 2026-10-21 · 30 days.")
        })
        .await?;
        day_button(&picker, "2026-10-12").await?.click().await?;
        until("the first day asks for the end", async || {
            Ok(status.text().await?.trim() == "From 2026-10-12: now choose the end.")
        })
        .await?;
        let apply = picker
            .find(By::XPath(".//button[normalize-space(.) = 'Apply']"))
            .await?;
        assert!(!apply.is_enabled().await?, "a half range cannot apply");
        day_button(&picker, "2026-10-10").await?.click().await?;
        until("an earlier second day starts the range", async || {
            Ok(status.text().await?.trim() == "2026-10-10 to 2026-10-12 · 3 days.")
        })
        .await?;
        apply.click().await?;
        browser
            .at(&gallery.url(&format!("{FORMS}?from=2026-10-10&to=2026-10-12#date-range")))
            .await?;
        browser.console_clean("the range").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn on_a_phone_the_range_is_a_full_width_sheet_with_one_month_and_both_arrows()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url(FORMS)).await?;
        let picker = open_range(browser).await?;
        let rect = picker.rect().await?;
        // The layout viewport, which leaves out a classic scrollbar.
        let view: f64 = browser
            .driver()
            .execute("return document.documentElement.clientWidth;", vec![])
            .await?
            .convert()?;
        assert!(
            rect.x.abs() < 1.0 && (rect.width - view).abs() < 1.0,
            "full width of {view}: {rect:?}"
        );
        let months = picker.find_all(By::Css(".fui-datepicker-month")).await?;
        assert!(
            !months[0].is_displayed().await?,
            "the earlier month is left out"
        );
        let back = months[1]
            .find(By::Css("button[aria-label='Previous month']"))
            .await?;
        assert!(
            back.is_displayed().await?,
            "the month shown has its back arrow"
        );
        let on = months[1]
            .find(By::Css("button[aria-label='Next month']"))
            .await?;
        assert!(on.is_displayed().await?);
        back.click().await?;
        picker
            .find(By::Css("table[aria-label='September 2026']"))
            .await?;
        browser.console_clean("the phone sheet").await
    })
    .await
}
