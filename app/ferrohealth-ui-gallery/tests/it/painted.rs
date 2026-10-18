// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The painted colours: the contrast the browser actually draws, read from
//! the computed style of the elements a reader reads, in every brand, light
//! and dark (WCAG 2.2 SC 1.4.3), beside the stylesheet's own contrast test.
//!
//! A translucent ground is laid over the ground under it as the browser
//! composites it. The styles are read over the Chrome `DevTools` Protocol, so
//! no script is written.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::Cookie;

use crate::browser::{Browser, with_browser};
use crate::journeys::hydrated_tabs;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// A colour as the browser computes it, each channel in 0 to 255.
#[derive(Debug, Clone, Copy)]
struct Rgba(f64, f64, f64, f64);

impl Rgba {
    /// The colour `value` writes, `rgb(r, g, b)` or `rgba(r, g, b, a)`.
    fn parse(value: &str) -> Option<Self> {
        let inner = value
            .strip_prefix("rgba(")
            .or_else(|| value.strip_prefix("rgb("))?
            .strip_suffix(')')?;
        let parts: Vec<f64> = inner
            .split(',')
            .map(|part| part.trim().parse::<f64>())
            .collect::<Result<_, _>>()
            .ok()?;
        match parts.as_slice() {
            [r, g, b] => Some(Self(*r, *g, *b, 1.0)),
            [r, g, b, a] => Some(Self(*r, *g, *b, *a)),
            _ => None,
        }
    }

    /// This colour laid over the opaque `ground`.
    fn over(self, ground: Self) -> Self {
        let mix = |top: f64, under: f64| top * self.3 + under * (1.0 - self.3);
        Self(
            mix(self.0, ground.0),
            mix(self.1, ground.1),
            mix(self.2, ground.2),
            1.0,
        )
    }

    /// The relative luminance WCAG 2.2 defines.
    fn luminance(self) -> f64 {
        let linear = |c: f64| {
            let s = c / 255.0;
            if s <= 0.040_45 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(self.0) + 0.7152 * linear(self.1) + 0.0722 * linear(self.2)
    }
}

/// The contrast ratio of two opaque colours.
fn ratio(one: Rgba, two: Rgba) -> f64 {
    let (a, b) = (one.luminance(), two.luminance());
    let (light, dark) = if a >= b { (a, b) } else { (b, a) };
    (light + 0.05) / (dark + 0.05)
}

/// The computed `property` of the first element `selector` matches.
async fn computed(
    browser: &Browser,
    selector: &str,
    property: &str,
) -> Result<Rgba, Box<dyn Error>> {
    let cdp = browser.driver().cdp();
    cdp.send_raw("DOM.enable", json!({})).await?;
    cdp.send_raw("CSS.enable", json!({})).await?;
    let document = cdp.send_raw("DOM.getDocument", json!({})).await?;
    let root = document["root"]["nodeId"].clone();
    let found = cdp
        .send_raw(
            "DOM.querySelector",
            json!({ "nodeId": root, "selector": selector }),
        )
        .await?;
    let node = found["nodeId"].as_u64().filter(|node| *node != 0);
    let node = node.ok_or_else(|| format!("no element matches {selector}"))?;
    let style = cdp
        .send_raw("CSS.getComputedStyleForNode", json!({ "nodeId": node }))
        .await?;
    let value = style["computedStyle"]
        .as_array()
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry["name"] == property)
                .and_then(|entry| entry["value"].as_str())
        })
        .ok_or_else(|| format!("{selector} has no computed {property}"))?;
    Rgba::parse(value).ok_or_else(|| format!("{selector} {property} is no colour: {value}").into())
}

/// The pairs the check reads: the text's selector, the ground's selector,
/// and the ground under a translucent ground.
const PAIRS: [(&str, &str, &str); 5] = [
    ("main h1", "body", "body"),
    (".fui-card .fui-muted", ".fui-card", ".fui-card"),
    (".fui-button-primary", ".fui-button-primary", ".fui-card"),
    (
        ".fui-nav-entry[aria-current='page']",
        ".fui-nav-entry[aria-current='page']",
        ".fui-sidebar",
    ),
    (".fui-facts dt", ".fui-card", ".fui-card"),
];

#[tokio::test(flavor = "multi_thread")]
async fn the_painted_text_meets_wcag_aa_in_every_brand_light_and_dark() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        let mut failures = Vec::new();
        for theme in ["light", "dark"] {
            browser.goto(&gallery.url("/")).await?;
            let mut cookie = Cookie::new("gallery-theme", theme);
            cookie.set_path("/");
            browser.driver().add_cookie(cookie).await?;
            for brand in Brand::ALL {
                browser
                    .goto(&gallery.url(&format!("/{}", brand.key())))
                    .await?;
                hydrated_tabs(browser).await?;
                for (text, ground, under) in PAIRS {
                    let fg = computed(browser, text, "color").await?;
                    let base = computed(browser, under, "background-color").await?;
                    let page = computed(browser, "body", "background-color").await?;
                    let floor = base.over(page);
                    let bg = computed(browser, ground, "background-color")
                        .await?
                        .over(floor);
                    let measured = ratio(fg.over(bg), bg);
                    if measured < 4.5 {
                        failures.push(format!(
                            "{} {theme}: {text} on {ground} paints {measured:.2}:1",
                            brand.product()
                        ));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
        browser.console_clean("the painted colours").await
    })
    .await
}

#[test]
fn a_computed_colour_reads_and_composites() {
    let half = Rgba::parse("rgba(0, 0, 0, 0.5)").expect("a colour");
    let white = Rgba::parse("rgb(255, 255, 255)").expect("a colour");
    assert!((half.over(white).0 - 127.5).abs() < 1e-9);
    assert!((ratio(Rgba(0.0, 0.0, 0.0, 1.0), white) - 21.0).abs() < 1e-9);
    assert!(Rgba::parse("transparent").is_none());
}
