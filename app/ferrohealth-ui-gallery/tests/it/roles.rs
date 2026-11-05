// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! What Chrome's accessibility tree reads from the native elements the
//! components draw where an ARIA role stood before (#624): a live region
//! is an `<output>`, a named scroll box a `<section>`, the user menu's
//! separator an `<hr>` (ARIA in HTML § 4, the implicit semantics of
//! `output`, `section` with an accessible name, and `hr`). A control that
//! shows words is named starting with them, and an icon control shows no
//! text, so WCAG 2.2 SC 2.5.3 Label in Name has no visible label to hold its
//! name to.

use std::error::Error;

use serde_json::{Value, json};
use thirtyfour::prelude::*;

use crate::browser::{Browser, with_browser};
use crate::dates::{open_picker, open_range};
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

/// The accessibility tree's reading of every rendered element `selector`
/// matches on the page the browser holds, in document order.
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
        if !rendered(browser, &element).await? {
            continue;
        }
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
    assert!(!nodes.is_empty(), "{selector} matches no rendered element");
    Ok(nodes)
}

/// Whether the browser renders `element`: neither it nor an ancestor is
/// `display: none` (CSSOM View, `checkVisibility()`). An empty live region
/// and a visually hidden one are rendered, though WebDriver counts neither
/// as displayed.
async fn rendered(browser: &Browser, element: &WebElement) -> Result<bool, Box<dyn Error>> {
    Ok(browser
        .driver()
        .execute(
            "return arguments[0].checkVisibility();",
            vec![element.to_json()?],
        )
        .await?
        .convert()?)
}

/// Waits until the loader has marked every element that asks for a module
/// `data-fui-ready`, so what a module draws is there to read.
async fn ready(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let left: Vec<String> = browser
        .driver()
        .execute_async(
            r#"const done = arguments[arguments.length - 1];
            const waiting = () => [...document.querySelectorAll("[data-fui]:not([data-fui-ready])")];
            (async () => {
              for (let n = 0; n < 600 && waiting().length; n++) await new Promise((r) => requestAnimationFrame(r));
              done(waiting().map((e) => e.dataset.fui));
            })();"#,
            Vec::new(),
        )
        .await?
        .convert()?;
    assert!(left.is_empty(), "not enhanced: {left:?}");
    Ok(())
}

/// Every page and the live regions on it, each a component's own.
const LIVE: &[(&str, &[&str])] = &[
    (
        "/ferrobridge/",
        &["output.fui-spinner", "[aria-busy] > output.fui-sr-only"],
    ),
    ("/ferrobridge/code", &[".fui-copy > output"]),
    (
        "/ferrobridge/configuration",
        &[".fui-key-hint + output", "[data-fui='countdown'] output"],
    ),
    (
        "/ferrobridge/federation",
        &["output.fui-topology-zoom-level"],
    ),
    (
        "/ferrobridge/forms",
        &[
            ".fui-editor-status output",
            "[data-fui~='identifier'] > output.fui-sr-only",
            "output.fui-identifier-refused",
            "form[data-fui='search'] > output",
            "output[data-upload='read']",
            ".fui-table-tools > output",
            ".fui-help > output",
        ],
    ),
    (
        "/ferrobridge/forms?dock=done",
        &["output.fui-dock-message.fui-dock-done"],
    ),
    (
        "/ferrobridge/preferences?saved=density",
        &["output.fui-setting-saved"],
    ),
    ("/ferrobridge/tables", &[".fui-pager > output"]),
    ("/ferrobridge/trees", &[".fui-muted > output"]),
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
            ready(browser).await?;
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

// WAI-ARIA 1.2 § status: the date pickers' status lines, which their
// modules draw as a picker opens, are polite statuses too.
#[tokio::test(flavor = "multi_thread")]
async fn a_date_pickers_status_line_is_an_output_the_tree_reads_as_a_polite_status()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for range in [false, true] {
            browser.goto(&gallery.url("/ferrobridge/forms")).await?;
            ready(browser).await?;
            let picker = if range {
                open_range(browser).await?
            } else {
                open_picker(browser, "audit-from").await?
            };
            let picker = picker.id().await?.ok_or("the picker has an id")?;
            browser
                .driver()
                .execute(
                    "document.querySelectorAll('.fui-datepicker-status').forEach((o) => o.hidden = false);",
                    Vec::new(),
                )
                .await?;
            for node in read(browser, &format!("#{picker} .fui-datepicker-status")).await? {
                assert_eq!(
                    ("OUTPUT", "status", "polite"),
                    (node.tag.as_str(), node.role.as_str(), node.live.as_str()),
                    "#{picker}: {node:?}"
                );
            }
        }
        Ok(())
    })
    .await
}

/// Every page, the control pressed first, if any, and the named boxes the
/// page then shows.
const REGIONS: &[(&str, Option<&str>, &str)] = &[
    ("/ferrobridge/", None, ".fui-alert-evidence"),
    ("/ferrobridge/code", None, ".fui-code-panel-numbered"),
    (
        "/ferrobridge/code",
        Some(".fui-document-switch > [data-view='raw']"),
        ".fui-document-view > .fui-code-panel",
    ),
    ("/ferrobridge/code", None, ".fui-linediff"),
    ("/ferrobridge/forms", None, "#identifiers"),
    ("/ferrobridge/forms?dock=idle", None, ".fui-dock"),
    ("/ferrobridge/layouts", None, ".fui-pane"),
    (
        "/ferrobridge/tables",
        None,
        "section.fui-table-scroll:has(> .fui-table)",
    ),
    ("/ferrobridge/tables", None, "section:has(> .fui-compare)"),
    (
        "/ferrobridge/terminology",
        None,
        ".fui-root-matrix > section",
    ),
    (
        "/ferrobridge/templates/about",
        None,
        ".fui-self-description",
    ),
];

// ARIA in HTML § 4: a section with an accessible name is a region.
#[tokio::test(flavor = "multi_thread")]
async fn a_named_box_is_a_section_the_tree_reads_as_a_region_by_its_name()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for (page, press, selector) in REGIONS {
            browser.goto(&gallery.url(page)).await?;
            ready(browser).await?;
            if let Some(press) = press {
                browser.element(By::Css(*press)).await?.click().await?;
            }
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

/// The pages whose menus draw a separator: the user menu on every page,
/// an action menu on the overlays and a filter menu on the federation.
const SEPARATED: &[&str] = &[
    "/ferrobridge/code",
    "/ferrobridge/federation",
    "/ferrobridge/overlays",
];

// ARIA in HTML § 4: an hr is a separator, which WAI-ARIA 1.2 § menu lets a
// menu own.
#[tokio::test(flavor = "multi_thread")]
async fn a_menus_separator_is_an_hr_the_tree_reads_as_a_separator() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for page in SEPARATED {
            browser.goto(&gallery.url(page)).await?;
            ready(browser).await?;
            let count: usize = browser
                .driver()
                .execute(
                    "return document.querySelectorAll('hr.fui-menu-sep').length;",
                    Vec::new(),
                )
                .await?
                .convert()?;
            assert!(count > 0, "{page} draws a menu separator");
            for at in 0..count {
                // Opens the menu that holds the separator `at`, alone.
                browser
                    .driver()
                    .execute(
                        r#"document.querySelectorAll(":popover-open").forEach((p) => p.hidePopover());
                        document.querySelectorAll("details[open]").forEach((d) => d.open = false);
                        document.querySelectorAll("[data-pin]").forEach((e) => e.removeAttribute("data-pin"));
                        const sep = document.querySelectorAll("hr.fui-menu-sep")[arguments[0]];
                        for (let up = sep.parentElement; up; up = up.parentElement) {
                          if (up.matches("[popover]")) up.showPopover();
                          if (up.matches("details")) up.open = true;
                        }
                        sep.setAttribute("data-pin", "");"#,
                        vec![json!(at)],
                    )
                    .await?;
                for node in read(browser, "[data-pin]").await? {
                    assert_eq!(
                        ("HR", "separator"),
                        (node.tag.as_str(), node.role.as_str()),
                        "{page} separator {at}: {node:?}"
                    );
                }
            }
        }
        Ok(())
    })
    .await
}

/// Every page, the window width it is read at, and the controls on it whose
/// only visible part is an icon. The drawer's button shows below 64rem, and
/// the search's submit drops its word on a phone.
const ICON_ONLY: &[(&str, u32, &str)] = &[
    ("/ferrobridge/code", 390, ".fui-drawer-only"),
    ("/ferrobridge/code", 390, ".fui-search-submit"),
    (
        "/ferrobridge/forms",
        1440,
        ".fui-date > button[aria-haspopup='dialog']",
    ),
    (
        "/ferrobridge/overlays?toast=ok",
        1440,
        ".fui-toast :is(summary, button)",
    ),
    (
        "/ferrobridge/templates/list-many",
        1440,
        ".fui-filter-chip > a",
    ),
    ("/ferrobridge/trees", 1440, ".fui-tree-toggle"),
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
        for (page, width, selector) in ICON_ONLY {
            browser.driver().set_window_rect(0, 0, *width, 900).await?;
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

/// The controls on the code page at 1440 px whose visible label is words:
/// the user menu's button that shows a name, and the search's submit with
/// its word.
const WORDED: &[&str] = &[
    ".fui-operator:has(.fui-operator-name)",
    ".fui-search-submit",
];

// WCAG 2.2 SC 2.5.3: a control whose label is words is named starting with
// them.
#[tokio::test(flavor = "multi_thread")]
async fn a_control_that_shows_words_is_named_by_them() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.driver().set_window_rect(0, 0, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrobridge/code")).await?;
        for selector in WORDED {
            for node in read(browser, selector).await? {
                assert!(
                    !node.text.is_empty() && node.name.starts_with(&node.text),
                    "{selector}: {node:?}"
                );
            }
        }
        Ok(())
    })
    .await
}
