// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! What Chrome's accessibility tree reads from the native elements the
//! components draw where an ARIA role stood before (#624): a live region
//! is an `<output>`, a named scroll box a `<section>`, the user menu's
//! separator an `<hr>` (WAI-ARIA in HTML, § 4, the implicit semantics of
//! `output`, `section` with an accessible name, and `hr`), and an icon
//! control shows no text, so WCAG 2.2 SC 2.5.3 Label in Name has no visible
//! label to hold its name to.

use std::error::Error;

use serde_json::{Value, json};
use thirtyfour::prelude::*;

use crate::browser::{Browser, with_browser};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// What the accessibility tree says of one element.
#[derive(Debug)]
struct Node {
    /// The computed role, such as `status`.
    role: String,
    /// The computed accessible name.
    name: String,
    /// The `live` property, empty when the node has none.
    live: String,
    /// The element's tag name, upper case.
    tag: String,
    /// The element's rendered text, trimmed.
    text: String,
}

/// The accessibility tree's reading of every element `selector` matches on
/// the page the browser holds, in document order.
async fn read(browser: &Browser, selector: &str) -> Result<Vec<Node>, Box<dyn Error>> {
    let cdp = browser.driver().cdp();
    cdp.send_raw("DOM.enable", json!({})).await?;
    cdp.send_raw("Accessibility.enable", json!({})).await?;
    let document = cdp.send_raw("DOM.getDocument", json!({})).await?;
    let root = document["root"]["nodeId"].clone();
    let found = cdp
        .send_raw(
            "DOM.querySelectorAll",
            json!({ "nodeId": root, "selector": selector }),
        )
        .await?;
    let ids: Vec<Value> = found["nodeIds"].as_array().cloned().unwrap_or_default();
    let shown = browser.driver().find_all(By::Css(selector)).await?;
    assert_eq!(
        ids.len(),
        shown.len(),
        "{selector}: the DOM and the driver agree"
    );
    let mut nodes = Vec::new();
    for (id, element) in ids.into_iter().zip(shown) {
        let tree = cdp
            .send_raw(
                "Accessibility.getPartialAXTree",
                json!({ "nodeId": id, "fetchRelatives": false }),
            )
            .await?;
        let node = &tree["nodes"][0];
        let text = |value: &Value| value["value"].as_str().unwrap_or_default().to_owned();
        let live = node["properties"]
            .as_array()
            .and_then(|properties| properties.iter().find(|p| p["name"] == "live"))
            .map(|property| text(&property["value"]))
            .unwrap_or_default();
        nodes.push(Node {
            role: text(&node["role"]),
            name: text(&node["name"]),
            live,
            tag: element.tag_name().await?.to_uppercase(),
            text: element.text().await?.trim().to_owned(),
        });
    }
    assert!(!nodes.is_empty(), "{selector} matches nothing");
    Ok(nodes)
}

/// Every page and the live regions on it, each a component's own.
const LIVE: &[(&str, &[&str])] = &[
    ("/ferrobridge/code", &[".fui-copy > output"]),
    ("/ferrobridge/configuration", &[".fui-key-hint + output"]),
    (
        "/ferrobridge/forms",
        &[
            ".fui-editor-status output",
            "[data-fui~='identifier'] > output.fui-sr-only",
            "output.fui-identifier-refused",
            "form[data-fui='search'] > output",
            "output[data-upload='read']",
        ],
    ),
    (
        "/ferrobridge/forms?dock=done",
        &["output.fui-dock-message.fui-dock-done"],
    ),
    ("/ferrobridge/operations", &["output.fui-spinner"]),
    (
        "/ferrobridge/preferences?saved=density",
        &["output.fui-setting-saved"],
    ),
];

// WAI-ARIA 1.2 § status: a status has an implicit aria-live of polite.
#[tokio::test(flavor = "multi_thread")]
async fn a_live_region_is_an_output_the_tree_reads_as_a_polite_status() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for (page, selectors) in LIVE {
            browser.goto(&gallery.url(page)).await?;
            // An output the module shows later is read as it will be shown.
            browser
                .driver()
                .execute(
                    "document.querySelectorAll('output[hidden]').forEach((o) => o.hidden = false);",
                    Vec::new(),
                )
                .await?;
            for selector in *selectors {
                for node in read(browser, selector).await? {
                    assert_eq!(
                        ("OUTPUT", "status", "polite"),
                        (node.tag.as_str(), node.role.as_str(), node.live.as_str()),
                        "{page} {selector}: {node:?}"
                    );
                }
            }
        }
        Ok(())
    })
    .await
}

/// Every page and the named scroll boxes on it.
const REGIONS: &[(&str, &str)] = &[
    ("/ferrobridge/", ".fui-alert-evidence"),
    ("/ferrobridge/code", ".fui-code-panel-numbered"),
    ("/ferrobridge/code", ".fui-document-view > .fui-code-panel"),
    ("/ferrobridge/forms?dock=idle", ".fui-dock"),
    ("/ferrobridge/templates/about", ".fui-self-description"),
];

// HTML-ARIA § 4: a section with an accessible name is a region.
#[tokio::test(flavor = "multi_thread")]
async fn a_named_box_is_a_section_the_tree_reads_as_a_region_by_its_name()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for (page, selector) in REGIONS {
            browser.goto(&gallery.url(page)).await?;
            for node in read(browser, selector).await? {
                assert_eq!(
                    ("SECTION", "region"),
                    (node.tag.as_str(), node.role.as_str()),
                    "{page} {selector}: {node:?}"
                );
                assert!(!node.name.is_empty(), "{page} {selector}: {node:?}");
            }
        }
        Ok(())
    })
    .await
}

// HTML-ARIA § 4: an hr is a separator, which WAI-ARIA 1.2 § menu lets a
// menu own.
#[tokio::test(flavor = "multi_thread")]
async fn the_user_menus_separator_is_an_hr_the_tree_reads_as_a_separator()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/code")).await?;
        browser
            .driver()
            .execute(
                "document.querySelector('.fui-user-menu').showPopover();",
                Vec::new(),
            )
            .await?;
        for node in read(browser, "[role='menu'] > .fui-menu-sep").await? {
            assert_eq!(
                ("HR", "separator"),
                (node.tag.as_str(), node.role.as_str()),
                "{node:?}"
            );
        }
        Ok(())
    })
    .await
}

/// Every page and the controls on it whose only visible part is an icon.
const ICON_ONLY: &[(&str, &str)] = &[
    ("/ferrobridge/code", ".fui-drawer-only"),
    ("/ferrobridge/code", ".fui-search-submit"),
    (
        "/ferrobridge/forms",
        ".fui-date > button[aria-haspopup='dialog']",
    ),
    ("/ferrobridge/overlays?toast=ok", ".fui-toast summary"),
    ("/ferrobridge/templates/list-many", ".fui-filter-chip > a"),
    ("/ferrobridge/trees", ".fui-tree-toggle"),
];

// WCAG 2.2 Understanding SC 2.5.3: where a component has no visible text
// label the criterion does not apply, and its name says what it does.
#[tokio::test(flavor = "multi_thread")]
async fn an_icon_control_shows_no_text_and_its_name_says_what_it_does() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for (page, selector) in ICON_ONLY {
            browser.goto(&gallery.url(page)).await?;
            for node in read(browser, selector).await? {
                assert!(
                    node.text.is_empty() && !node.name.is_empty(),
                    "{page} {selector}: {node:?}"
                );
            }
        }
        Ok(())
    })
    .await
}

// WCAG 2.2 SC 2.5.3: the user menu's button that shows a name is named by
// that name.
#[tokio::test(flavor = "multi_thread")]
async fn the_user_menus_button_is_named_by_the_name_it_shows() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/code")).await?;
        for node in read(browser, ".fui-operator:has(.fui-operator-name)").await? {
            assert!(
                !node.text.is_empty() && node.name.starts_with(&node.text),
                "{node:?}"
            );
        }
        Ok(())
    })
    .await
}
