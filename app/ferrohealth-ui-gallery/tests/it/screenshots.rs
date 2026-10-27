// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's screenshots, taken behind the `FERROUI_SCREENSHOTS` gate,
//! and the guard that every one is on disk.
//!
//! With `FERROUI_SCREENSHOTS=1` the capture drives the gallery through every
//! page of every brand and every sign-in card, in the light and the dark
//! theme, at 1440 and at 390 CSS pixels wide and the full height of the page,
//! and writes each as `<page>.png` and `<page>-dark.png`, the narrow ones
//! `<page>-narrow.png` and `<page>-narrow-dark.png`, under [`IMAGES`]. Without the
//! gate the capture returns at once. [`PAGES_FILTER`] limits it to the pages
//! a change touches.
//!
//! The guard, every shot and state has both images and every image is a
//! shot's or a state's, runs behind `FERROUI_SCREENSHOT_GUARD=1`: in CI on the
//! refresh pull request (branch `chore/screenshots`), on a release pull
//! request and on a manual run. A feature pull request commits no image
//! (`scripts/checks/screenshot-paths.sh`), so between refreshes the committed
//! set may lack a state a merged change added.
//!
//! The capture runs with `prefers-reduced-motion: reduce`, so a spinner or a
//! skeleton holds still and an unchanged page captures pixel-identical. On a
//! release pull request the pixel check (`FERROUI_SCREENSHOT_PIXELS=1`)
//! compares every committed image, the pages and the states, with a fresh
//! capture by its pixels.

use std::collections::BTreeSet;
use std::error::Error;
use std::path::{Path, PathBuf};

use ferrohealth_ui::style::Brand;
use serde::Deserialize;
use serde_json::json;
use thirtyfour::{By, Cookie};

use crate::browser::{Browser, with_browser};
use crate::journeys::{PAGES, SIGN_INS, window};
use crate::stack::Gallery;

/// The environment variable that opts in to the capture.
const SCREENSHOTS_GATE: &str = "FERROUI_SCREENSHOTS";

/// The environment variable that limits the capture to some pages: a
/// comma-separated list of page names, each matched against the last part
/// of an image's name (`tables` takes the tables page of every brand). The
/// whole capture runs when it is unset.
const PAGES_FILTER: &str = "FERROUI_SCREENSHOT_PAGES";

/// The environment variable that opts in to the screenshot guard.
const GUARD_GATE: &str = "FERROUI_SCREENSHOT_GUARD";

/// The environment variable that opts in to the pseudo-locale capture,
/// which checks the layout of longer words once and is never committed.
const PSEUDO_GATE: &str = "FERROUI_PSEUDO";

/// The folder of the pseudo-locale capture, from the workspace root.
const PSEUDO_IMAGES: &str = "target/pseudo-screenshots";

/// The folder of the screenshots, from the workspace root.
pub(crate) const IMAGES: &str = "docs/screenshots";

/// The widths every screenshot is taken at, in CSS pixels, with the suffix
/// each image name takes: a wide screen and a phone.
const WIDTHS: [(u32, &str); 2] = [(1440, ""), (390, "-narrow")];

/// The width the window returns to between shots.
const WIDTH: u32 = 1440;

/// The height of the window a page is laid out in before its full height is
/// measured.
const HEIGHT: u32 = 900;

/// The tallest screenshot, under the browser's 16384 pixel texture limit.
const TALLEST: u32 = 16000;

/// One screenshot: the image's name without the theme suffix, the path the
/// browser loads, and a selector the page shows once it has loaded.
#[derive(Debug, Clone)]
struct Shot {
    /// The image's name, such as `ferrobridge/tables`.
    image: String,
    /// The path the browser loads.
    path: String,
    /// A CSS selector of an element the page shows once loaded.
    ready: &'static str,
}

/// The name of the page `page` of a brand's pages.
fn page_name(page: &str) -> &str {
    page.strip_prefix('/')
        .filter(|name| !name.is_empty())
        .unwrap_or("overview")
}

/// Every shot the capture takes.
fn shots() -> Vec<Shot> {
    let mut shots = vec![
        Shot {
            image: String::from("index"),
            path: String::from("/"),
            ready: ".fui-sign-in",
        },
        Shot {
            image: String::from("not-found"),
            path: String::from("/no/such/page"),
            ready: ".fui-empty",
        },
    ];
    for brand in Brand::ALL {
        for &page in PAGES {
            shots.push(Shot {
                image: format!("{}/{}", brand.key(), page_name(page)),
                path: format!("/{}{page}", brand.key()),
                ready: "main h1",
            });
        }
        shots.push(Shot {
            image: format!("{}/fault", brand.key()),
            path: format!("/fault/{}", brand.key()),
            ready: ".fui-status-page h1",
        });
        for variant in SIGN_INS {
            shots.push(Shot {
                image: format!("{}/sign-in-{variant}", brand.key()),
                path: format!("/sign-in/{}/{variant}", brand.key()),
                ready: ".fui-sign-in",
            });
        }
    }
    shots
}

/// Whether the capture takes `image` under the page list `filter`, the value
/// of [`PAGES_FILTER`] (`None` takes every image).
fn wanted(image: &str, filter: Option<&str>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    let page = image.rsplit('/').next().unwrap_or(image);
    filter.split(',').map(str::trim).any(|name| name == page)
}

/// The workspace root.
pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The file of `image` in the theme `theme`.
fn file(image: &str, theme: &str) -> PathBuf {
    file_in(IMAGES, image, theme)
}

/// The file of `image` in the theme `theme` under `folder`.
fn file_in(folder: &str, image: &str, theme: &str) -> PathBuf {
    let suffix = if theme == "dark" { "-dark" } else { "" };
    root().join(folder).join(format!("{image}{suffix}.png"))
}

/// The part of `Page.getLayoutMetrics` the capture reads.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayoutMetrics {
    /// The size of the page's content.
    css_content_size: ContentSize,
}

/// A height in CSS pixels.
#[derive(Debug, Deserialize)]
struct ContentSize {
    /// The height.
    height: f64,
}

/// The height of the page's content, from [`HEIGHT`] to [`TALLEST`].
async fn content_height(browser: &Browser) -> Result<u32, Box<dyn Error>> {
    let metrics: LayoutMetrics = serde_json::from_value(
        browser
            .driver()
            .cdp()
            .send_raw("Page.getLayoutMetrics", json!({}))
            .await?,
    )?;
    let content = metrics.css_content_size.height;
    Ok((HEIGHT..=TALLEST)
        .find(|tall| f64::from(*tall) >= content)
        .unwrap_or(TALLEST))
}

/// Loads `shot` in `theme` and writes it at the page's full height under
/// `folder`.
async fn capture(
    browser: &Browser,
    gallery: &Gallery,
    shot: &Shot,
    theme: &str,
    (width, suffix): (u32, &str),
    folder: &str,
) -> Result<(), Box<dyn Error>> {
    window(browser, width, HEIGHT).await?;
    browser.goto(&gallery.url(&shot.path)).await?;
    browser.element(By::Css(shot.ready)).await?;
    browser
        .element(By::Css(format!(r#"html[data-theme="{theme}"]"#)))
        .await?;
    browser.hydrated().await?;
    // A taller window can lay the page out taller (the footer follows the
    // viewport), so the height is measured again until it holds.
    browser.settled().await?;
    let mut height = content_height(browser).await?;
    for _ in 0..4 {
        window(browser, width, height).await?;
        browser.settled().await?;
        let measured = content_height(browser).await?;
        if measured <= height {
            break;
        }
        height = measured;
    }
    let png = browser.driver().screenshot_as_png().await?;
    window(browser, WIDTH, HEIGHT).await?;
    let path = file_in(folder, &format!("{}{suffix}", shot.image), theme);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, png)?;
    if shot.image == "not-found" {
        let errors = browser.console_errors().await?;
        return match errors.as_slice() {
            [error] if error.contains("404") => Ok(()),
            _ => Err(format!("the unknown path logged {errors:?}, not its one 404").into()),
        };
    }
    browser.console_clean(&shot.image).await
}

#[tokio::test(flavor = "multi_thread")]
async fn every_page_is_captured_in_every_brand_light_and_dark() -> Result<(), Box<dyn Error>> {
    if !std::env::var(SCREENSHOTS_GATE).is_ok_and(|value| value == "1") {
        return Ok(());
    }
    let filter = std::env::var(PAGES_FILTER).ok();
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
                    json!({ "features": [
                        { "name": "prefers-color-scheme", "value": theme },
                        { "name": "prefers-reduced-motion", "value": "reduce" },
                    ] }),
                )
                .await?;
            for shot in shots()
                .into_iter()
                .filter(|shot| wanted(&shot.image, filter.as_deref()))
            {
                for width in WIDTHS {
                    capture(browser, &gallery, &shot, theme, width, IMAGES).await?;
                }
            }
        }
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn one_brand_is_captured_in_the_pseudo_locale_light_and_dark() -> Result<(), Box<dyn Error>> {
    if !std::env::var(PSEUDO_GATE).is_ok_and(|value| value == "1") {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, WIDTH, HEIGHT).await?;
        for theme in ["light", "dark"] {
            browser.goto(&gallery.url("/")).await?;
            browser.driver().delete_all_cookies().await?;
            for (name, value) in [
                ("gallery-theme", theme),
                (
                    "gallery-language",
                    ferrohealth_ui_gallery::locale::PSEUDO_TAG,
                ),
            ] {
                let mut cookie = Cookie::new(name, value);
                cookie.set_path("/");
                browser.driver().add_cookie(cookie).await?;
            }
            browser
                .driver()
                .cdp()
                .send_raw(
                    "Emulation.setEmulatedMedia",
                    json!({ "features": [
                        { "name": "prefers-color-scheme", "value": theme },
                        { "name": "prefers-reduced-motion", "value": "reduce" },
                    ] }),
                )
                .await?;
            for shot in shots()
                .into_iter()
                .filter(|shot| shot.image.starts_with(Brand::Bridge.key()))
            {
                for width in WIDTHS {
                    capture(browser, &gallery, &shot, theme, width, PSEUDO_IMAGES).await?;
                }
            }
        }
        Ok(())
    })
    .await
}

#[test]
fn the_page_filter_takes_the_named_pages_of_every_brand_alone() {
    assert!(wanted("ferrobridge/tables", None));
    assert!(wanted("ferrobridge/tables", Some("tables")));
    assert!(wanted("ferroehr/tables", Some("forms, tables")));
    assert!(wanted("index", Some("index")));
    assert!(!wanted("ferrobridge/forms", Some("tables")));
    assert!(!wanted("ferrobridge/sign-in-basic", Some("sign-in")));
}

#[test]
fn every_shot_has_its_light_and_dark_image_and_no_other_image_is_kept() {
    if !std::env::var(GUARD_GATE).is_ok_and(|value| value == "1") {
        return;
    }
    let mut expected = BTreeSet::new();
    for shot in shots() {
        for (theme, (_, suffix)) in ["light", "dark"]
            .into_iter()
            .flat_map(|theme| WIDTHS.map(|width| (theme, width)))
        {
            let image = file(&format!("{}{suffix}", shot.image), theme);
            assert!(
                image.is_file(),
                "{} is missing: run the capture with {SCREENSHOTS_GATE}=1",
                image.display()
            );
            expected.insert(image);
        }
    }
    for brand in Brand::ALL {
        for state in crate::states::all() {
            for theme in ["light", "dark"] {
                let image = file(&format!("states/{}/{state}", brand.key()), theme);
                assert!(
                    image.is_file(),
                    "{} is missing: run the capture with {SCREENSHOTS_GATE}=1",
                    image.display()
                );
                expected.insert(image);
            }
        }
    }
    let mut folders = vec![root().join(IMAGES)];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(&folder).expect("the screenshots folder") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                folders.push(path);
            } else {
                let path = folder.join(path.file_name().expect("a file name"));
                assert!(
                    expected.contains(&path),
                    "{} is no shot's image",
                    path.display()
                );
            }
        }
    }
}

/// The environment variable that opts in to the pixel check: a fresh
/// capture against the committed images, on a release branch.
const PIXELS_GATE: &str = "FERROUI_SCREENSHOT_PIXELS";

/// The red, green and blue of every pixel of the PNG `bytes`, row by row,
/// with its width and height; an alpha channel is left out, since every
/// capture is opaque and the alpha alone never tells two captures apart.
fn rgb(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), Box<dyn Error>> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let size = reader
        .output_buffer_size()
        .ok_or("the image is too large to decode")?;
    let mut buffer = vec![0; size];
    let frame = reader.next_frame(&mut buffer)?;
    buffer.truncate(frame.buffer_size());
    let channels = match frame.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        other => return Err(format!("a capture is RGB or RGBA, not {other:?}").into()),
    };
    let pixels = buffer
        .chunks_exact(channels)
        .flat_map(|pixel| pixel.iter().take(3).copied())
        .collect();
    Ok((frame.width, frame.height, pixels))
}

/// Every PNG under `folder`, as paths relative to the workspace root.
fn images_under(folder: &Path, into: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in std::fs::read_dir(root().join(folder))? {
        let path = entry?.path();
        let relative = path.strip_prefix(root())?.to_path_buf();
        if path.is_dir() {
            images_under(&relative, into)?;
        } else if path.extension().is_some_and(|extension| extension == "png") {
            into.push(relative);
        }
    }
    Ok(())
}

#[test]
fn every_committed_screenshot_shows_what_a_fresh_capture_draws() -> Result<(), Box<dyn Error>> {
    if !std::env::var(PIXELS_GATE).is_ok_and(|value| value == "1") {
        return Ok(());
    }
    let mut images = Vec::new();
    images_under(Path::new(IMAGES), &mut images)?;
    images.sort();
    let mut differ = Vec::new();
    for image in &images {
        let name = image.to_string_lossy().replace('\\', "/");
        let committed = std::process::Command::new("git")
            .arg("-C")
            .arg(root())
            .arg("show")
            .arg(format!("HEAD:{name}"))
            .output()?;
        if !committed.status.success() {
            differ.push(format!("{name} (not committed)"));
            continue;
        }
        let fresh = std::fs::read(root().join(image))?;
        if rgb(&committed.stdout)? != rgb(&fresh)? {
            differ.push(name);
        }
    }
    assert!(
        differ.is_empty(),
        "{} of {} committed screenshots no longer show what the gallery draws; land the \
         refresh pull request (branch chore/screenshots) first (docs/release.md):\n{}",
        differ.len(),
        images.len(),
        differ.join("\n")
    );
    Ok(())
}
