// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Every page with its WebAssembly withheld: its content is all there and
//! its links and forms still lead where they lead.
//!
//! The browser is refused every file of the site bundle (`/pkg/`), so no
//! island hydrates, as on a page whose WebAssembly has not loaded or will
//! not. The interaction contract's third layer is what this holds:
//! complete server-rendered HTML, and a link or a form behind every
//! important action (no specification governs this: our own design). A
//! refused bundle file is the one browser error a page may log here.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::{PAGES, SIGN_INS, window};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// Refuses the browser every file of the site bundle.
async fn withhold_the_webassembly(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let cdp = browser.driver().cdp();
    cdp.send_raw("Network.enable", json!({})).await?;
    cdp.send_raw("Network.setBlockedURLs", json!({ "urls": ["*/pkg/*"] }))
        .await?;
    Ok(())
}

/// Fails on any browser error other than a refused bundle file, naming
/// `step`.
async fn only_the_bundle_refused(browser: &Browser, step: &str) -> Result<(), Box<dyn Error>> {
    let others: Vec<String> = browser
        .console_errors()
        .await?
        .into_iter()
        .filter(|entry| !entry.contains("/pkg/"))
        .collect();
    if others.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the browser logged {} error(s) other than the withheld bundle during {step}:\n{}",
        others.len(),
        others.join("\n")
    )
    .into())
}

/// Leaves the page for a blank one, so the island loader, which imports
/// the bundle when the browser is next idle, logs its refusal before the
/// journey ends or never; then fails on any error but a refused bundle file,
/// naming `step`.
async fn leave(browser: &Browser, step: &str) -> Result<(), Box<dyn Error>> {
    browser.goto("about:blank").await?;
    only_the_bundle_refused(browser, step).await
}

/// The trimmed text of the element `by` names, waiting for it.
async fn text_of(browser: &Browser, by: By) -> Result<String, Box<dyn Error>> {
    Ok(browser.element(by).await?.text().await?.trim().to_owned())
}

/// Checks what a brand's page at `page` must show with no WebAssembly,
/// beyond its heading and its navigation.
async fn page_content(browser: &Browser, page: &str) -> Result<(), Box<dyn Error>> {
    match page {
        "" => {
            assert!(
                browser
                    .element(By::Id("answer-panel-0"))
                    .await?
                    .is_displayed()
                    .await?,
                "the first keyboard tab's panel shows"
            );
        }
        "/forms" => {
            // The combobox's native select stands until its island hydrates,
            // so finding it proves the WebAssembly was withheld.
            browser.element(By::Css("select#choose-system")).await?;
            let draft = browser.element(By::Id("gallery-draft")).await?;
            let held = draft.prop("value").await?.unwrap_or_default();
            assert!(!held.is_empty(), "the editor shows its text");
        }
        "/tables" => {
            browser.text("p", "1-25 of 137").await?;
            let rows = browser
                .driver()
                .find_all(By::XPath("(//table)[1]/tbody/tr"))
                .await?;
            assert_eq!(25, rows.len(), "the first table shows its page");
        }
        "/code" => {
            let shown = text_of(browser, By::Id("document-highlighted")).await?;
            assert!(!shown.is_empty(), "the document pane shows its document");
        }
        "/overlays" => {
            browser
                .element(By::Css("button[popovertarget='reload-set']"))
                .await?;
        }
        "/charts" => {
            browser.element(By::Css("main svg")).await?;
        }
        "/layouts" => {
            let items = browser
                .driver()
                .find_all(By::Css("#gallery-tree [role='treeitem']"))
                .await?;
            assert!(!items.is_empty(), "the tree shows its items");
        }
        "/templates/list" | "/templates/list-tabs" => {
            let rows = browser
                .driver()
                .find_all(By::Css(".fui-worklist tbody tr"))
                .await?;
            assert_eq!(5, rows.len(), "the list shows its five locations");
        }
        "/templates/list-many" => {
            browser.element(By::Css(".fui-filter-chip a[href]")).await?;
            browser.element(By::Css(".fui-list-foot a[href]")).await?;
        }
        "/templates/list-empty" => {
            browser
                .element(By::Css(".fui-transform-empty a[href]"))
                .await?;
        }
        "/templates/detail" => {
            browser.element(By::Css(".fui-crumbs a[href]")).await?;
            browser.element(By::Css(".fui-stepper a[href]")).await?;
        }
        "/templates/settings" => {
            browser.element(By::Css(".fui-key-hint code")).await?;
            browser.text("p", "Set in").await?;
        }
        "/templates/settings-managed" => {
            browser.element(By::Css(".fui-revision-compact")).await?;
        }
        "/templates/about" => {
            browser.element(By::Css("a[download]")).await?;
            let shown = text_of(browser, By::Css(".fui-self-description")).await?;
            assert!(!shown.is_empty(), "the self-description shows");
        }
        "/templates/forbidden" | "/templates/not-found" => {
            browser
                .element(By::Css(".fui-status-page a.fui-button-primary[href]"))
                .await?;
        }
        other => return Err(format!("no content check for the page {other:?}").into()),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn every_page_keeps_its_content_with_the_webassembly_withheld() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_webassembly(browser).await?;
        browser.goto(&gallery.url("/")).await?;
        browser.text("p", "Choose a brand").await?;
        for brand in Brand::ALL {
            browser
                .element(By::Css(format!("a[href='/{}']", brand.key())))
                .await?;
        }
        only_the_bundle_refused(browser, "the front page").await?;
        for brand in Brand::ALL {
            for page in PAGES {
                let path = format!("/{}{page}", brand.key());
                browser.goto(&gallery.url(&path)).await?;
                let heading = text_of(browser, By::Css("main h1")).await?;
                assert!(!heading.is_empty(), "{path} has a heading");
                let current = browser
                    .element(By::Css("#sidebar a[aria-current='page']"))
                    .await?;
                assert_eq!(
                    Some(path.clone()),
                    current.attr("href").await?,
                    "the navigation marks {path}"
                );
                browser.element(By::Css("footer")).await?;
                page_content(browser, page).await?;
                only_the_bundle_refused(browser, &path).await?;
            }
            for variant in SIGN_INS {
                let path = format!("/sign-in/{}/{variant}", brand.key());
                browser.goto(&gallery.url(&path)).await?;
                browser.element(By::Css(".fui-sign-in")).await?;
                if variant == "provider" {
                    browser
                        .element(By::Css(".fui-sign-in a, .fui-sign-in form"))
                        .await?;
                } else {
                    let form = browser
                        .element(By::Css(".fui-sign-in form[method='post']"))
                        .await?;
                    form.find(By::Css("input[autocomplete='username']")).await?;
                    form.find(By::Css("input[autocomplete='current-password']"))
                        .await?;
                }
                only_the_bundle_refused(browser, &path).await?;
            }
        }
        // The page the gallery does not serve answers 404, which the browser
        // logs beside the withheld bundle.
        browser.goto(&gallery.url("/nowhere")).await?;
        browser.element(By::Css("a[href='/']")).await?;
        browser.goto("about:blank").await?;
        let others: Vec<String> = browser
            .console_errors()
            .await?
            .into_iter()
            .filter(|entry| !entry.contains("/pkg/") && !entry.contains("/nowhere - "))
            .collect();
        assert!(others.is_empty(), "the 404 page logged {others:?}");
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn links_lead_on_with_the_webassembly_withheld() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_webassembly(browser).await?;
        // The navigation's links load their page and mark it.
        browser.goto(&gallery.url("/ferrobridge")).await?;
        browser.follow("Tables and paging").await?;
        browser.at(&gallery.url("/ferrobridge/tables")).await?;
        browser
            .element(By::Css(
                "#sidebar a[aria-current='page'][href='/ferrobridge/tables']",
            ))
            .await?;
        // The link pager and the sortable header are links.
        browser
            .element(By::XPath(
                "//nav[@aria-label = 'Pages of the mapping files']//a[contains(., 'Next')]",
            ))
            .await?
            .click()
            .await?;
        browser.text("p", "26-50 of 137").await?;
        browser
            .element(By::XPath("//th[contains(., 'Lines')]//a"))
            .await?
            .click()
            .await?;
        browser
            .element(By::XPath(
                "//th[@aria-sort = 'ascending' and contains(., 'Lines')]",
            ))
            .await?;
        // The brand switch is links to the same page.
        browser
            .element(By::Css("a[href='/ferrofed/tables']"))
            .await?
            .click()
            .await?;
        browser.at(&gallery.url("/ferrofed/tables")).await?;
        // A concept is a link, and the server draws it selected.
        browser.goto(&gallery.url("/ferroehr/layouts")).await?;
        browser
            .element(By::Css("a[href='/ferroehr/layouts?concept=SYN-110']"))
            .await?
            .click()
            .await?;
        browser
            .at(&gallery.url("/ferroehr/layouts?concept=SYN-110"))
            .await?;
        browser.text("h2", "SYN-110").await?;
        leave(browser, "the links").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn forms_post_and_submit_with_the_webassembly_withheld() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_webassembly(browser).await?;
        browser
            .driver()
            .cdp()
            .send_raw(
                "Emulation.setEmulatedMedia",
                json!({ "features": [{ "name": "prefers-color-scheme", "value": "light" }] }),
            )
            .await?;
        // The search box is a GET form.
        browser.goto(&gallery.url("/ferrofed/tables")).await?;
        let search = browser.element(By::Id("gallery-search")).await?;
        search.send_keys("synthetic-007").await?;
        search.send_keys(Key::Enter).await?;
        browser
            .at(&gallery.url("/ferrofed/tables?q=synthetic-007"))
            .await?;
        // The theme toggle and the density switch post, and the page comes
        // back in the choice.
        browser.goto(&gallery.url("/ferrofed")).await?;
        browser
            .element(By::Id("theme-to-dark"))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css(r#"html[data-theme="dark"]"#))
            .await?;
        browser
            .element(By::XPath(
                "//button[@name = 'density' and @value = 'compact']",
            ))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css(r#"html[data-density="compact"]"#))
            .await?;
        leave(browser, "the forms").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn popovers_open_with_the_webassembly_withheld() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_webassembly(browser).await?;
        // The operator menu is a popover the browser opens.
        browser.goto(&gallery.url("/ferrofed")).await?;
        browser
            .element(By::Css("button[popovertarget='operator-menu']"))
            .await?
            .click()
            .await?;
        let menu = browser.element(By::Id("operator-menu")).await?;
        until("the operator menu opens", async || {
            Ok(menu.is_displayed().await?)
        })
        .await?;
        // A confirmation dialog is a popover its trigger opens.
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        browser
            .element(By::Css("button[popovertarget='reload-set']"))
            .await?
            .click()
            .await?;
        let dialog = browser.element(By::Id("reload-set")).await?;
        until("the confirmation opens", async || {
            Ok(dialog.is_displayed().await?)
        })
        .await?;
        // On a narrow screen the drawer is a popover the menu button opens,
        // and its links still lead on.
        window(browser, 600, 900).await?;
        browser.goto(&gallery.url("/ferroehr")).await?;
        browser
            .element(By::XPath("//button[@aria-label = 'Menu']"))
            .await?
            .click()
            .await?;
        let nav = browser.element(By::Id("sidebar")).await?;
        until("the drawer opens", async || Ok(nav.is_displayed().await?)).await?;
        browser.follow("Documents").await?;
        browser.at(&gallery.url("/ferroehr/code")).await?;
        leave(browser, "the popovers").await
    })
    .await
}
