// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's screenshots, taken behind the `FERROUI_SCREENSHOTS` gate,
//! and the guard that every one is on disk.
//!
//! With `FERROUI_SCREENSHOTS=1` the capture drives the gallery through every
//! page of every brand and every sign-in card, in the light and the dark
//! theme, at 1440 CSS pixels wide and the full height of the page, and writes
//! each as `<page>.png` and `<page>-dark.png` under [`IMAGES`]. Without the
//! gate the capture returns at once. The guard runs in the ordinary suite:
//! every shot has both images, and every image is a shot's.

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

/// The folder of the screenshots, from the workspace root.
pub(crate) const IMAGES: &str = "docs/screenshots";

/// The width every screenshot is taken at, in CSS pixels.
const WIDTH: u32 = 1440;

/// The height of the window a page is laid out in before its full height is
/// measured.
const HEIGHT: u32 = 900;

/// The tallest screenshot.
const TALLEST: u32 = 8000;

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
        for page in PAGES {
            shots.push(Shot {
                image: format!("{}/{}", brand.key(), page_name(page)),
                path: format!("/{}{page}", brand.key()),
                ready: "main h1",
            });
        }
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

/// The workspace root.
pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The file of `image` in the theme `theme`.
fn file(image: &str, theme: &str) -> PathBuf {
    let suffix = if theme == "dark" { "-dark" } else { "" };
    root().join(IMAGES).join(format!("{image}{suffix}.png"))
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

/// Loads `shot` in `theme` and writes it at the page's full height.
async fn capture(
    browser: &Browser,
    gallery: &Gallery,
    shot: &Shot,
    theme: &str,
) -> Result<(), Box<dyn Error>> {
    browser.goto(&gallery.url(&shot.path)).await?;
    browser.element(By::Css(shot.ready)).await?;
    browser
        .element(By::Css(format!(r#"html[data-theme="{theme}"]"#)))
        .await?;
    // A taller window can lay the page out taller (the footer follows the
    // viewport), so the height is measured again until it holds.
    let mut height = content_height(browser).await?;
    for _ in 0..4 {
        window(browser, WIDTH, height).await?;
        let measured = content_height(browser).await?;
        if measured <= height {
            break;
        }
        height = measured;
    }
    let png = browser.driver().screenshot_as_png().await?;
    window(browser, WIDTH, HEIGHT).await?;
    let path = file(&shot.image, theme);
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
                    json!({ "features": [{ "name": "prefers-color-scheme", "value": theme }] }),
                )
                .await?;
            for shot in shots() {
                capture(browser, &gallery, &shot, theme).await?;
            }
        }
        Ok(())
    })
    .await
}

#[test]
fn every_shot_has_its_light_and_dark_image_and_no_other_image_is_kept() {
    let mut expected = BTreeSet::new();
    for shot in shots() {
        for theme in ["light", "dark"] {
            let image = file(&shot.image, theme);
            assert!(
                image.is_file(),
                "{} is missing: run the capture with {SCREENSHOTS_GATE}=1",
                image.display()
            );
            expected.insert(image);
        }
    }
    for brand in Brand::ALL {
        for state in crate::states::STATES {
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
