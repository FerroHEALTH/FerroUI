// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The interaction states, captured behind the `FERROUI_SCREENSHOTS` gate:
//! the pointer and the keyboard focus on the controls, every overlay and
//! menu open, the fields in each of their states, and the phone layouts.
//!
//! Each area of the gallery keeps its states in a file of its own under
//! `states/`: the names in its `STATES` and the function that captures
//! them, so two changes to different areas touch different files. [`all`]
//! is every area's list in turn.
//!
//! Each state is written as `states/<brand>/<state>.png` and its `-dark`
//! twin under the screenshots folder, every brand in the light and the
//! dark theme, so the hover and focus treatment is looked at as often as
//! the pages are. The capture writes only a state [`all`] lists and fails
//! unless it wrote every one of them, so the lists and the capture cannot
//! drift apart.

use std::collections::BTreeSet;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::prelude::*;

use crate::browser::{Browser, with_browser};
use crate::journeys::window;
use crate::stack::Gallery;

// The areas, one file each, kept in alphabetical order.
mod combobox;
mod dates;
mod document;
mod fields;
mod keys;
mod layouts;
mod menus;
mod overlays;
mod overview;
mod phone;
mod preferences;
mod rail;
mod tables;
mod upload;

/// Every area's states.
const AREAS: &[&[&str]] = &[
    combobox::STATES,
    dates::STATES,
    document::STATES,
    fields::STATES,
    keys::STATES,
    layouts::STATES,
    menus::STATES,
    overlays::STATES,
    overview::STATES,
    phone::STATES,
    preferences::STATES,
    rail::STATES,
    tables::STATES,
    upload::STATES,
];

/// Every state, by the name its images take: each area's in turn.
pub(crate) fn all() -> impl Iterator<Item = &'static str> {
    AREAS.iter().flat_map(|area| area.iter().copied())
}

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

/// Refuses a state [`all`] does not list, whose image the screenshot
/// guard would reject.
fn listed(state: &str) -> Result<(), Box<dyn Error>> {
    if all().any(|listed| listed == state) {
        Ok(())
    } else {
        Err(format!("the capture writes {state}, which no area's STATES lists").into())
    }
}

/// Every listed state the capture did not write in some brand and theme.
fn unwritten() -> Vec<String> {
    let written = WRITTEN.lock().unwrap_or_else(PoisonError::into_inner);
    let mut missing = Vec::new();
    for brand in Brand::ALL {
        for theme in ["light", "dark"] {
            for state in all() {
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
fn card(title: &str) -> By {
    By::XPath(format!(
        "//section[contains(@class, 'fui-card')][h2[normalize-space(.) = '{title}'] or div/h2[normalize-space(.) = '{title}']]"
    ))
}

/// Moves the pointer onto `element`.
async fn hover(browser: &Browser, element: &WebElement) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .move_to_element_center(element)
        .perform()
        .await?;
    Ok(())
}

/// Moves the pointer away, to the page's top-left corner.
async fn away(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let body = browser.element(By::Tag("footer")).await?;
    hover(browser, &body).await
}

/// Gives `element` the keyboard focus.
async fn focus(browser: &Browser, element: &WebElement) -> Result<(), Box<dyn Error>> {
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
async fn escape(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(Key::Escape)
        .perform()
        .await?;
    Ok(())
}

/// Writes the screenshot of `element` as `state`.
async fn shot_of(
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
async fn shot_of_view(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
) -> Result<(), Box<dyn Error>> {
    browser.settled().await?;
    let png = browser.driver().screenshot_as_png().await?;
    write(brand, state, theme, &png)
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
                overview::overview(browser, &gallery, brand, theme).await?;
                tables::tables_and_editor(browser, &gallery, brand, theme).await?;
                upload::upload(browser, &gallery, brand, theme).await?;
                overlays::overlays(browser, &gallery, brand, theme).await?;
                layouts::layouts(browser, &gallery, brand, theme).await?;
                combobox::combobox_states(browser, &gallery, brand, theme).await?;
                fields::field_states(browser, &gallery, brand, theme).await?;
                dates::date_states(browser, &gallery, brand, theme).await?;
                tables::compact(browser, &gallery, brand, theme).await?;
                rail::rail(browser, &gallery, brand, theme).await?;
                phone::phone(browser, &gallery, brand, theme).await?;
                document::document(browser, &gallery, brand, theme).await?;
                menus::menu_states(browser, &gallery, brand, theme).await?;
                keys::keys(browser, &gallery, brand, theme).await?;
                preferences::preferences(browser, &gallery, brand, theme).await?;
            }
        }
        let missing = unwritten();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!("the areas list states the capture never wrote: {missing:?}").into())
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
    let unique: BTreeSet<&str> = all().collect();
    assert_eq!(unique.len(), all().count(), "a state is listed twice");
}
