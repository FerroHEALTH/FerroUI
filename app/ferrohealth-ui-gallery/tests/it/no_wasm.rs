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

/// Reaches each view of the code page's document pane through its link,
/// and its other format through the format's link.
async fn document_views(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let highlighted = browser.element(By::Id("document-highlighted")).await?;
    let raw = browser.element(By::Id("document-raw")).await?;
    assert!(
        !raw.is_displayed().await?,
        "the raw view waits for its link"
    );
    browser
        .element(By::Css(".fui-document-switch a[data-view='raw']"))
        .await?
        .click()
        .await?;
    until("the raw view shows by its link", async || {
        Ok(raw.is_displayed().await? && !highlighted.is_displayed().await?)
    })
    .await?;
    let rendered = browser.element(By::Id("document-rendered")).await?;
    browser
        .element(By::Css(".fui-document-switch a[data-view='rendered']"))
        .await?
        .click()
        .await?;
    until("the rendered view shows by its link", async || {
        Ok(rendered.is_displayed().await? && !raw.is_displayed().await?)
    })
    .await?;
    browser
        .element(By::XPath(
            "//nav[contains(@class, 'fui-segmented')]/a[text() = 'XML']",
        ))
        .await?
        .click()
        .await?;
    browser
        .element(By::Css("#document-highlighted .fui-tok-tag"))
        .await?;
    assert!(
        browser
            .driver()
            .find_all(By::Css(".fui-document button"))
            .await?
            .is_empty(),
        "no control in the pane works only with script"
    );
    Ok(())
}

/// Scrolls the element `by` names to the middle of the viewport, clear of
/// the sticky top bar, and clicks it.
async fn press(browser: &Browser, by: By) -> Result<(), Box<dyn Error>> {
    let element = browser.element(by).await?;
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

/// Checks that the overlays page's confirmation trigger is there and its
/// More opens with no script.
async fn more_opens(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser
        .element(By::Css("button[popovertarget='reload-set']"))
        .await?;
    let more = browser
        .element(By::Css("summary[aria-controls='overlays-more']"))
        .await?;
    more.click().await?;
    let item = browser
        .element(By::XPath(
            "//div[@id = 'overlays-more']//a[@role = 'menuitem']",
        ))
        .await?;
    assert!(item.is_displayed().await?, "More opens with no script");
    more.click().await?;
    Ok(())
}

/// Checks that the chart shows its colour key to a sighted reader.
async fn colour_key(browser: &Browser) -> Result<(), Box<dyn Error>> {
    browser.element(By::Css("main svg")).await?;
    let keys = browser
        .driver()
        .find_all(By::Css(".fui-legend-key"))
        .await?;
    assert_eq!(3, keys.len(), "the colour key names every series");
    for key in keys {
        assert!(key.is_displayed().await?, "a sighted reader sees the key");
    }
    Ok(())
}

/// Checks the forms page's date fields with no WebAssembly: a date field
/// is its typed text with the calendar disabled and saying to type the
/// date (R23), and a range shows its From and To fields in its button's
/// place (R22).
async fn date_fields(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let from = browser.element(By::Id("audit-from")).await?;
    assert_eq!(Some(String::from("2026-10-14")), from.prop("value").await?);
    let calendar = from.find(By::XPath("following-sibling::button")).await?;
    assert!(!calendar.is_enabled().await?, "the calendar is disabled");
    assert!(
        calendar
            .attr("aria-label")
            .await?
            .is_some_and(|name| name.starts_with("Type the date")),
        "the calendar's name says to type the date"
    );
    assert!(
        browser
            .element(By::Id("audit-range-from"))
            .await?
            .is_displayed()
            .await?
    );
    assert!(
        browser
            .element(By::Id("audit-range-to"))
            .await?
            .is_displayed()
            .await?
    );
    Ok(())
}

/// Checks the forms page with no WebAssembly: the combobox's native select,
/// the editor's text and the date fields.
async fn forms_content(browser: &Browser) -> Result<(), Box<dyn Error>> {
    // The combobox's native select stands until its island hydrates,
    // so finding it proves the WebAssembly was withheld.
    browser.element(By::Css("select#choose-system")).await?;
    let draft = browser.element(By::Id("gallery-draft")).await?;
    let held = draft.prop("value").await?.unwrap_or_default();
    assert!(!held.is_empty(), "the editor shows its text");
    date_fields(browser).await
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
            // NOTE: leptos-ui.md §15.8: the phone's tab select waits disabled, and
            // every binder tab is a link to its own route.
            let select = browser.element(By::Id("fui-tabs-select")).await?;
            assert!(
                select.attr("disabled").await?.is_some(),
                "the tab select cannot work yet"
            );
            browser
                .element(By::Css(
                    "nav.fui-tabs a[href='?tab=commit'][data-tab='commit']",
                ))
                .await?;
        }
        "/forms" => forms_content(browser).await?,
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
            document_views(browser).await?;
        }
        "/overlays" => {
            more_opens(browser).await?;
            let opener = browser.element(By::Id("open-shortcuts")).await?;
            assert!(
                !opener.is_displayed().await?,
                "the legend's opener, which needs its script, waits hidden"
            );
            browser.element(By::Css(".fui-shortcut-hint kbd")).await?;
        }
        "/charts" => colour_key(browser).await?,
        "/layouts" => {
            let items = browser
                .driver()
                .find_all(By::Css("#gallery-tree [role='treeitem']"))
                .await?;
            assert!(!items.is_empty(), "the tree shows its items");
            crate::widgets::help_switch_journey(browser).await?;
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
                browser.element(By::Css(".fui-sign-in h1")).await?;
                browser
                    .element(By::Css(
                        ".fui-sign-in a.fui-button-primary, .fui-sign-in button.fui-button-primary",
                    ))
                    .await?;
                if variant != "provider" {
                    let notice = text_of(browser, By::Id("sign-in-notice")).await?;
                    assert!(!notice.is_empty(), "{path} names its state");
                }
                if variant == "basic" || variant == "locked" {
                    let form = browser
                        .element(By::Css(".fui-sign-in form[method='post']"))
                        .await?;
                    form.find(By::Css("input[autocomplete='username']")).await?;
                    form.find(By::Css("input[autocomplete='current-password']"))
                        .await?;
                } else {
                    assert!(
                        browser
                            .driver()
                            .find_all(By::Css("input[type='password']"))
                            .await?
                            .is_empty(),
                        "{path} holds no password field"
                    );
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
        // The link pager is links, and the sortable header a GET form.
        browser
            .element(By::XPath(
                "//nav[@aria-label = 'Pages of the mapping files']//a[contains(., 'Next')]",
            ))
            .await?
            .click()
            .await?;
        browser.text("p", "26-50 of 137").await?;
        browser
            .element(By::XPath("(//table)[1]//th[contains(., 'Lines')]//button"))
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

/// Follows a hit of the search's answer and saves through the docked bar,
/// both drawn by the server.
async fn answer_and_dock(browser: &Browser, gallery: &Gallery) -> Result<(), Box<dyn Error>> {
    // The search's answer is drawn by the server and its hits are links.
    browser
        .goto(&gallery.url("/ferrofed/forms?search=hits"))
        .await?;
    browser
        .element(By::Id("gallery-search-hit-1"))
        .await?
        .click()
        .await?;
    browser
        .at(&gallery.url("/ferrofed/tables?q=synthetic-001.model.yml"))
        .await?;
    // The docked bar's decisions submit the form they name.
    browser
        .goto(&gallery.url("/ferrofed/forms?dock=missing"))
        .await?;
    browser
        .element(By::Id("dock-reason"))
        .await?
        .send_keys("Synthetic night thresholds")
        .await?;
    browser
        .element(By::Css("button[value='save']"))
        .await?
        .click()
        .await?;
    browser
        .text("p", "Saved: the settings apply at once.")
        .await?;
    Ok(())
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
        answer_and_dock(browser, &gallery).await?;
        // The user menu opens as a popover, its theme posts, and the page
        // comes back in the choice; so does the density switch.
        browser.goto(&gallery.url("/ferrofed")).await?;
        browser
            .element(By::Css("button[popovertarget='user-menu']"))
            .await?
            .click()
            .await?;
        let dark = browser
            .element(By::Css("#user-menu button[name='theme'][value='dark']"))
            .await?;
        until("the user menu opens", async || {
            Ok(dark.is_displayed().await?)
        })
        .await?;
        dark.click().await?;
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
        // The sidebar's Collapse posts, and the page comes back as the rail.
        browser
            .element(By::XPath("//button[@name = 'sidebar' and @value = 'rail']"))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css("#sidebar.fui-sidebar-rail"))
            .await?;
        // Sign out posts and leads to the signed-out card.
        browser
            .element(By::Css("button[popovertarget='user-menu']"))
            .await?
            .click()
            .await?;
        let out = browser
            .element(By::XPath(
                "//*[@id = 'user-menu']//button[contains(., 'Sign out')]",
            ))
            .await?;
        until("the user menu opens again", async || {
            Ok(out.is_displayed().await?)
        })
        .await?;
        out.click().await?;
        browser
            .at(&gallery.url("/sign-in/ferrofed/signed-out"))
            .await?;
        // A history filter's range posts its From and To fields as typed.
        browser.goto(&gallery.url("/ferropix/forms")).await?;
        let from = browser.element(By::Id("audit-range-from")).await?;
        from.clear().await?;
        from.send_keys("2026-10-01").await?;
        let to = browser.element(By::Id("audit-range-to")).await?;
        to.clear().await?;
        to.send_keys("2026-10-07").await?;
        browser
            .element(By::XPath(
                "//form[@id = 'date-range']//button[@type = 'submit']",
            ))
            .await?
            .click()
            .await?;
        browser
            .at(&gallery.url("/ferropix/forms?from=2026-10-01&to=2026-10-07#date-range"))
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
        // The user menu is a popover the browser opens.
        browser.goto(&gallery.url("/ferrofed")).await?;
        browser
            .element(By::Css("button[popovertarget='user-menu']"))
            .await?
            .click()
            .await?;
        let menu = browser.element(By::Id("user-menu")).await?;
        until("the user menu opens", async || {
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
        // The popover form is a popover its trigger opens, and Escape closes.
        browser
            .element(By::Css("button[popovertarget='save-query']"))
            .await?
            .click()
            .await?;
        let form = browser.element(By::Id("save-query")).await?;
        until("the popover form opens", async || {
            Ok(form.is_displayed().await?)
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

#[tokio::test(flavor = "multi_thread")]
async fn a_listing_filters_sorts_and_pages_with_the_webassembly_withheld()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_webassembly(browser).await?;
        browser.goto(&gallery.url("/ferropix/tables")).await?;
        let find = browser.element(By::Id("gallery-find")).await?;
        find.send_keys("synthetic-01").await?;
        find.send_keys(Key::Enter).await?;
        until("the filter is in the URL", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("q=synthetic-01"))
        })
        .await?;
        browser.text("p", "1-10 of 10").await?;
        browser
            .element(By::XPath("(//table)[1]//th[contains(., 'Lines')]//button"))
            .await?
            .click()
            .await?;
        until("the sort keeps the filter", async || {
            let url = browser.driver().current_url().await?;
            Ok(url.as_str().contains("sort=lines") && url.as_str().contains("q=synthetic-01"))
        })
        .await?;
        browser
            .element(By::XPath(
                "//th[@aria-sort = 'ascending' and contains(., 'Lines')]",
            ))
            .await?;
        let before = browser.driver().current_url().await?;
        let next = By::XPath(
            "//nav[@aria-label = 'Pages of the mapping files, in the page']//button[contains(., 'Next')]",
        );
        browser.element(next).await?.click().await?;
        browser.text("p", "26-50 of 137").await?;
        let after = browser.driver().current_url().await?;
        assert!(
            !after.as_str().contains("page=") && after.path() == before.path(),
            "the session pager posts and the URL names no position: {after}"
        );
        leave(browser, "the listing").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn every_item_of_more_works_with_the_webassembly_withheld() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_webassembly(browser).await?;
        // A form button inside the menu posts.
        browser.goto(&gallery.url("/ferrobridge")).await?;
        press(browser, By::Css("summary[aria-controls='overview-more']")).await?;
        press(
            browser,
            By::XPath(
                "//button[@role = 'menuitem' and contains(., 'Switch to the compact density')]",
            ),
        )
        .await?;
        browser
            .element(By::Css(r#"html[data-density="compact"]"#))
            .await?;
        // A link inside the menu leads on.
        browser.goto(&gallery.url("/ferrobridge/overlays")).await?;
        press(browser, By::Css("summary[aria-controls='overlays-more']")).await?;
        press(
            browser,
            By::XPath("//a[@role = 'menuitem' and contains(., 'Open the document')]"),
        )
        .await?;
        browser.at(&gallery.url("/ferrobridge/code")).await?;
        // The destructive item opens its confirmation.
        browser.goto(&gallery.url("/ferrobridge/overlays")).await?;
        press(browser, By::Css("summary[aria-controls='overlays-more']")).await?;
        press(
            browser,
            By::XPath("//button[@role = 'menuitem' and contains(., 'Delete the file')]"),
        )
        .await?;
        let dialog = browser.element(By::Id("delete-file")).await?;
        until("the confirmation opens", async || {
            Ok(dialog.is_displayed().await?)
        })
        .await?;
        // A choice of the listbox posts too.
        browser.goto(&gallery.url("/ferrobridge/overlays")).await?;
        press(browser, By::Css("summary[aria-controls='language-list']")).await?;
        press(
            browser,
            By::XPath("//div[@id = 'language-list']//button[@role = 'option' and @lang = 'qps']"),
        )
        .await?;
        browser.element(By::Css(r#"html[lang="qps"]"#)).await?;
        leave(browser, "the menus").await
    })
    .await
}
