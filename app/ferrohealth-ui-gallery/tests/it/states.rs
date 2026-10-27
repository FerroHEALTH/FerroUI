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

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::screenshots::Shard;
use crate::stack::Gallery;

// The areas, one file each, kept in alphabetical order.
mod charts;
mod clinical;
mod combobox;
mod configuration;
mod dates;
mod document;
mod editor;
mod federation;
mod fields;
mod keys;
mod layouts;
mod menus;
mod overlays;
mod overview;
mod phone;
mod preferences;
mod rail;
mod records;
mod tables;
mod terminology;
mod trees;
mod upload;

/// Every area's states.
const AREAS: &[&[&str]] = &[
    charts::STATES,
    clinical::STATES,
    combobox::STATES,
    configuration::STATES,
    dates::STATES,
    document::STATES,
    editor::STATES,
    federation::STATES,
    fields::STATES,
    keys::STATES,
    layouts::STATES,
    menus::STATES,
    overlays::STATES,
    overview::STATES,
    phone::STATES,
    preferences::STATES,
    rail::STATES,
    records::STATES,
    tables::STATES,
    terminology::STATES,
    trees::STATES,
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

/// Every listed state the capture did not write in some brand of `shard`
/// and theme.
fn unwritten(shard: Shard) -> Vec<String> {
    let written = WRITTEN.lock().unwrap_or_else(PoisonError::into_inner);
    let mut missing = Vec::new();
    for brand in Brand::ALL.into_iter().filter(|brand| shard.takes(*brand)) {
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

/// The capture's own rule: the text caret, which blinks, is not drawn, so
/// two captures of a state hold the same pixels (FerroUI #434). It goes in
/// through the DevTools protocol, which the page's Content-Security-Policy
/// leaves alone. No specification governs this: our own design.
const STEADY: &str = "*, *::before, *::after { caret-color: transparent !important; }";

/// Adds [`STEADY`] to the page in view and waits for the page to settle.
async fn steady(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let cdp = browser.driver().cdp();
    cdp.send_raw("DOM.enable", json!({})).await?;
    cdp.send_raw("CSS.enable", json!({})).await?;
    let tree = cdp.send_raw("Page.getFrameTree", json!({})).await?;
    let frame = tree
        .pointer("/frameTree/frame/id")
        .and_then(serde_json::Value::as_str)
        .ok_or("the page has no frame")?
        .to_owned();
    let sheet = cdp
        .send_raw("CSS.createStyleSheet", json!({ "frameId": frame }))
        .await?;
    let id = sheet
        .get("styleSheetId")
        .and_then(serde_json::Value::as_str)
        .ok_or("the capture's style sheet has no id")?
        .to_owned();
    cdp.send_raw(
        "CSS.setStyleSheetText",
        json!({ "styleSheetId": id, "text": STEADY }),
    )
    .await?;
    browser.settled().await
}

/// Writes the screenshot of `element` as `state`, once the page's islands
/// have hydrated.
async fn shot_of(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
    element: &WebElement,
) -> Result<(), Box<dyn Error>> {
    browser.hydrated().await?;
    shot_of_unhydrated(browser, brand, theme, state, element).await
}

/// Writes the screenshot of `element` as `state` without waiting for the
/// islands, for a page whose WebAssembly is withheld.
async fn shot_of_unhydrated(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
    element: &WebElement,
) -> Result<(), Box<dyn Error>> {
    let png = in_view_and_held(browser, state, element).await?;
    write(brand, state, theme, &png)
}

/// The script that centres `arguments[0]` in the viewport at a whole
/// pixel of scroll, since a fractional one draws the edges another way each
/// time (#434). No specification governs this: our own design.
const CENTRE: &str = "arguments[0].scrollIntoView({block: 'center'}); window.scrollTo(Math.round(scrollX), Math.round(scrollY));";

/// The shot of `element` centred in the viewport, once its box, the scroll
/// and its pixels hold still.
///
/// A part above the element that settles late moves it, sometimes after it
/// was centred, so the element is centred again until a held shot finds it
/// still in full view (#434).
async fn in_view_and_held(
    browser: &Browser,
    state: &str,
    element: &WebElement,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let place = async || -> Result<String, Box<dyn Error>> {
        Ok(browser
            .driver()
            .execute_async(
                "const [element, done] = arguments; requestAnimationFrame(() => requestAnimationFrame(() => { const box = element.getBoundingClientRect(); done([box.x, box.y, box.width, box.height, scrollX, scrollY].join(' ')); }));",
                vec![element.to_json()?],
            )
            .await?
            .convert()?)
    };
    let in_view = async || -> Result<bool, Box<dyn Error>> {
        Ok(browser
            .driver()
            .execute(
                "const box = arguments[0].getBoundingClientRect(); return box.height > innerHeight || (box.top >= 0 && box.bottom <= innerHeight);",
                vec![element.to_json()?],
            )
            .await?
            .convert()?)
    };
    let mut shot = None;
    until(&format!("{state} stays in full view"), async || {
        browser
            .driver()
            .execute(CENTRE, vec![element.to_json()?])
            .await?;
        steady(browser).await?;
        let mut last = place().await?;
        until("the element's box and the scroll hold", async || {
            let now = place().await?;
            let held = now == last;
            last = now;
            Ok(held)
        })
        .await?;
        let png = held(browser, state, Part::Element(element)).await?;
        if in_view().await? {
            shot = Some(png);
            return Ok(true);
        }
        Ok(false)
    })
    .await?;
    shot.ok_or_else(|| format!("no shot of {state} held").into())
}

/// The part of the viewport `element` covers, cut from a shot of the
/// viewport.
///
/// chromedriver's own element shot scrolls an element taller than the
/// viewport into view each time, a pixel further for a box at a fractional
/// offset, so two shots of it differ; the viewport, scrolled where the
/// capture put it, holds still (#434).
pub(super) async fn element_png(
    browser: &Browser,
    element: &WebElement,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let (rect, shot) = element_shot(browser, element).await?;
    cut(&shot, rect)
}

/// The box of `element` in device pixels, as left, top, right and bottom,
/// and a shot of the viewport taken right after it was measured.
async fn element_shot(
    browser: &Browser,
    element: &WebElement,
) -> Result<([usize; 4], Vec<u8>), Box<dyn Error>> {
    let rect: Vec<usize> = browser
        .driver()
        .execute(
            "const box = arguments[0].getBoundingClientRect(), ratio = devicePixelRatio; return [Math.floor(box.left * ratio), Math.floor(box.top * ratio), Math.ceil(box.right * ratio), Math.ceil(box.bottom * ratio)].map((edge) => Math.max(0, edge));",
            vec![element.to_json()?],
        )
        .await?
        .convert()?;
    let [left, top, right, bottom] = rect[..] else {
        return Err(format!("the element's box is no rectangle: {rect:?}").into());
    };
    let shot = browser.driver().screenshot_as_png().await?;
    Ok(([left, top, right, bottom], shot))
}

/// The part `[left, top, right, bottom]` of the viewport shot `shot`, as a
/// PNG of its own.
fn cut(shot: &[u8], [left, top, right, bottom]: [usize; 4]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(shot));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("the shot is too large")?];
    let frame = reader.next_frame(&mut buffer)?;
    let channels = match frame.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => return Err(format!("a shot is RGB or RGBA, not {other:?}").into()),
    };
    let (width, height) = (
        usize::try_from(frame.width)?,
        usize::try_from(frame.height)?,
    );
    let (x0, y0) = (left.min(width), top.min(height));
    let (x1, y1) = (right.min(width), bottom.min(height));
    if x1 <= x0 || y1 <= y0 {
        return Err(format!(
            "the element lies outside the viewport: {:?}",
            [left, top, right, bottom]
        )
        .into());
    }
    let stride = width * channels;
    let mut cut = Vec::with_capacity((x1 - x0) * (y1 - y0) * channels);
    for row in y0..y1 {
        let start = row * stride + x0 * channels;
        cut.extend_from_slice(&buffer[start..start + (x1 - x0) * channels]);
    }
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, u32::try_from(x1 - x0)?, u32::try_from(y1 - y0)?);
    encoder.set_color(frame.color_type);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&cut)?;
    Ok(png)
}

/// What a held shot is cut from: the whole viewport, or the box of an
/// element in it.
#[derive(Debug, Clone, Copy)]
enum Part<'a> {
    /// The whole viewport.
    View,
    /// The box of this element.
    Element(&'a WebElement),
}

/// A viewport shot and the box of the part it is cut to, if any.
type Taken = (Option<[usize; 4]>, Vec<u8>);

/// Takes a viewport shot of `part`.
async fn take(browser: &Browser, part: Part<'_>) -> Result<Taken, Box<dyn Error>> {
    Ok(match part {
        Part::View => (None, browser.driver().screenshot_as_png().await?),
        Part::Element(element) => {
            let (rect, shot) = element_shot(browser, element).await?;
            (Some(rect), shot)
        }
    })
}

/// The image of the taken shot `taken`.
fn image((rect, shot): &Taken) -> Result<Vec<u8>, Box<dyn Error>> {
    match rect {
        Some(rect) => cut(shot, *rect),
        None => Ok(shot.clone()),
    }
}

/// The image of `part` that two shots in a row hold, the page settled
/// before each, so a scroll, a hover or a transition that ends late is
/// never caught half-way (#434). A shot of an element taller than the
/// viewport makes the browser lay the page out again, which the settling
/// waits out.
///
/// Chrome encodes the same pixels to the same bytes, so two viewport shots
/// with the same bytes and the same box hold the same image, and only a
/// pair whose bytes differ is cut and compared by its image.
async fn held(browser: &Browser, state: &str, part: Part<'_>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut last = take(browser, part).await?;
    let mut kept = None;
    until(
        &format!("two shots of {state} in a row hold the same pixels"),
        async || {
            browser.settled().await?;
            let now = take(browser, part).await?;
            if now == last {
                kept = Some(image(&now)?);
                return Ok(true);
            }
            let cut_now = image(&now)?;
            if image(&last)? == cut_now {
                kept = Some(cut_now);
                return Ok(true);
            }
            last = now;
            Ok(false)
        },
    )
    .await?;
    kept.ok_or_else(|| "no shot held".into())
}

/// Writes the screenshot of the viewport as `state`.
async fn shot_of_view(
    browser: &Browser,
    brand: Brand,
    theme: &str,
    state: &str,
) -> Result<(), Box<dyn Error>> {
    browser.hydrated().await?;
    steady(browser).await?;
    let png = held(browser, state, Part::View).await?;
    write(brand, state, theme, &png)
}

#[tokio::test(flavor = "multi_thread")]
async fn every_state_is_captured_in_every_brand_light_and_dark() -> Result<(), Box<dyn Error>> {
    if !std::env::var("FERROUI_SCREENSHOTS").is_ok_and(|value| value == "1") {
        return Ok(());
    }
    let shard = Shard::from_env()?;
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, WIDTH, HEIGHT).await?;
        browser.hide_scrollbars().await?;
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
            for brand in Brand::ALL.into_iter().filter(|brand| shard.takes(*brand)) {
                // The areas in alphabetical order, so two new areas land on
                // different lines (`scripts/checks/mod-order.sh`).
                charts::charts(browser, &gallery, brand, theme).await?;
                clinical::clinical(browser, &gallery, brand, theme).await?;
                combobox::combobox_states(browser, &gallery, brand, theme).await?;
                configuration::configuration(browser, &gallery, brand, theme).await?;
                dates::date_states(browser, &gallery, brand, theme).await?;
                document::document(browser, &gallery, brand, theme).await?;
                editor::editor(browser, &gallery, brand, theme).await?;
                federation::federation(browser, &gallery, brand, theme).await?;
                fields::field_states(browser, &gallery, brand, theme).await?;
                keys::keys(browser, &gallery, brand, theme).await?;
                layouts::layouts(browser, &gallery, brand, theme).await?;
                menus::menu_states(browser, &gallery, brand, theme).await?;
                overlays::overlays(browser, &gallery, brand, theme).await?;
                overview::overview(browser, &gallery, brand, theme).await?;
                phone::phone(browser, &gallery, brand, theme).await?;
                preferences::preferences(browser, &gallery, brand, theme).await?;
                rail::rail(browser, &gallery, brand, theme).await?;
                records::records(browser, &gallery, brand, theme).await?;
                tables::compact(browser, &gallery, brand, theme).await?;
                tables::tables_and_editor(browser, &gallery, brand, theme).await?;
                terminology::terminology(browser, &gallery, brand, theme).await?;
                trees::trees(browser, &gallery, brand, theme).await?;
                upload::upload(browser, &gallery, brand, theme).await?;
            }
        }
        let missing = unwritten(shard);
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
