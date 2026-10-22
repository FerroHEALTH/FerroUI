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
    for button in browser
        .driver()
        .find_all(By::Css(".fui-document button"))
        .await?
    {
        assert!(
            !button.is_displayed().await?,
            "no control in the pane that works only with script shows"
        );
    }
    copy_waits(browser, "document-copy").await
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

/// Checks that the side panel's opener is a link to its content's route,
/// and the button that opens it in place waits for the WebAssembly.
async fn side_panel_floor(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let link = browser
        .element(By::Css("a[href$='/overlays/access']"))
        .await?;
    assert!(link.is_displayed().await?, "the opener is a link");
    let button = browser
        .element(By::Css(
            "button[popovertarget='access'][aria-controls='access']",
        ))
        .await?;
    assert!(
        !button.is_displayed().await?,
        "the panel's button waits for the WebAssembly"
    );
    Ok(())
}

/// Checks that a setting's key reads as text and its copy button waits
/// for the WebAssembly.
async fn key_hint_floor(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let key = browser
        .element(By::Css(".fui-key-hint.is-text code"))
        .await?;
    let text = key.prop("textContent").await?.unwrap_or_default();
    assert!(!text.is_empty(), "the key reads as text");
    let hint = browser.element(By::Css("button.fui-key-hint")).await?;
    assert!(
        hint.attr("hidden").await?.is_some(),
        "the key's copy button waits for the WebAssembly"
    );
    Ok(())
}

/// Checks that the Copy button `id` waits hidden, its text on the page.
async fn copy_waits(browser: &Browser, id: &str) -> Result<(), Box<dyn Error>> {
    let copy = browser.element(By::Id(id)).await?;
    assert!(
        !copy.is_displayed().await?,
        "{id} waits for the WebAssembly"
    );
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

/// Checks that the tab of Your preferences at `page` offers its
/// preferences as plain forms, each posted by its Save button.
async fn preferences_content(browser: &Browser, page: &str) -> Result<(), Box<dyn Error>> {
    let control = match page {
        "/preferences" => "form.fui-preference-form input[type='radio'][name='theme']",
        "/preferences/language" => "form.fui-preference-form select[name='week-start']",
        _ => "form.fui-preference-form select[name='hints']",
    };
    browser.element(By::Css(control)).await?;
    let save = browser
        .element(By::Css("form.fui-preference-form button[type='submit']"))
        .await?;
    assert!(
        save.is_displayed().await?,
        "a preference posts through its Save button before the page hydrates"
    );
    if page == "/preferences/language" {
        browser
            .element(By::Css("form.fui-menu-form input[name='language']"))
            .await?;
    }
    browser
        .element(By::Css("button[popovertarget='preferences-reset']"))
        .await?;
    Ok(())
}

/// Chooses the compact density on Your preferences through its Save button,
/// then resets every preference through the confirmation.
async fn preferences_post_and_reset(
    browser: &Browser,
    gallery: &Gallery,
) -> Result<(), Box<dyn Error>> {
    browser.goto(&gallery.url("/ferrofed/preferences")).await?;
    press(
        browser,
        By::XPath("//label[contains(@class, 'fui-choice-card')][input[@value = 'compact']]"),
    )
    .await?;
    let save = browser
        .element(By::XPath(
            "//form[contains(@class, 'fui-preference-form')][.//input[@value = 'compact']]//button[@type = 'submit']",
        ))
        .await?;
    browser.click_and_load(&save).await?;
    browser
        .at(&gallery.url("/ferrofed/preferences?saved=density"))
        .await?;
    browser
        .element(By::Css(".fui-setting-saved[role='status']"))
        .await?;
    browser
        .element(By::Css(r#"html[data-density="compact"]"#))
        .await?;
    press(
        browser,
        By::Css("button[popovertarget='preferences-reset']"),
    )
    .await?;
    let confirm = browser
        .element(By::XPath(
            "//div[@id = 'preferences-reset']//button[@type = 'submit']",
        ))
        .await?;
    until("the reset dialog opens", async || {
        Ok(confirm.is_displayed().await?)
    })
    .await?;
    browser.click_and_load(&confirm).await?;
    browser
        .element(By::Css(
            r#"html[data-density="comfortable"][data-theme="system"]"#,
        ))
        .await?;
    Ok(())
}

/// Checks what the page template at `page` must show with no WebAssembly.
async fn templates_content(browser: &Browser, page: &str) -> Result<(), Box<dyn Error>> {
    match page {
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
            key_hint_floor(browser).await?;
            browser.text("p", "Set in").await?;
        }
        "/templates/settings-managed" => {
            browser.element(By::Css(".fui-revision-compact")).await?;
        }
        "/templates/about" => {
            browser.element(By::Css("a[download]")).await?;
            copy_waits(browser, "about-copy").await?;
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

/// Checks what the terminology page must show with no WebAssembly: the
/// version select opens its menu of links, and every Remove and Add of the
/// repeatable rows is a submit button of the form around it.
async fn terminology_content(browser: &Browser) -> Result<(), Box<dyn Error>> {
    // NOTE: leptos-ui.md §15.8: a `<details>` opens with no script, and each root is a link.
    let select = browser
        .element(By::Css("summary.fui-version-select"))
        .await?;
    select.click().await?;
    let root = browser
        .element(By::Css(
            "#fhir-version a[role='menuitemradio'][href$='?fhir=R5']",
        ))
        .await?;
    assert!(
        root.is_displayed().await?,
        "the version menu opens with no script"
    );
    select.click().await?;
    let buttons = browser
        .driver()
        .find_all(By::Css(
            "form [data-repeat='remove'], form [data-repeat='add']",
        ))
        .await?;
    assert!(
        buttons.len() >= 4,
        "the composer's and the editor's rows post"
    );
    for button in buttons {
        assert_eq!(
            Some(String::from("submit")),
            button.attr("type").await?,
            "Remove and Add post the form"
        );
    }
    browser
        .element(By::Css(".fui-root-matrix td.is-none"))
        .await?;
    Ok(())
}

/// Checks what the overlays page must show with no WebAssembly: the More
/// menu and the side panel open, and the legend's opener waits hidden.
async fn overlays_content(browser: &Browser) -> Result<(), Box<dyn Error>> {
    more_opens(browser).await?;
    side_panel_floor(browser).await?;
    let opener = browser.element(By::Id("open-shortcuts")).await?;
    assert!(
        !opener.is_displayed().await?,
        "the legend's opener, which needs its script, waits hidden"
    );
    browser.element(By::Css(".fui-shortcut-hint kbd")).await?;
    Ok(())
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
        "/forms" => {
            forms_content(browser).await?;
            crate::records::identifiers_floor(browser).await?;
        }
        "/tables" => {
            browser.text("p", "1-25 of 137").await?;
            let rows = browser
                .driver()
                .find_all(By::XPath("(//table)[1]/tbody/tr"))
                .await?;
            assert_eq!(25, rows.len(), "the first table shows its page");
            crate::records::worklist_floor(browser).await?;
        }
        "/code" => {
            let shown = text_of(browser, By::Id("document-highlighted")).await?;
            assert!(!shown.is_empty(), "the document pane shows its document");
            document_views(browser).await?;
            copy_waits(browser, "file-copy").await?;
        }
        "/configuration" => crate::configuration::config_floor(browser).await?,
        "/configuration/provisioned" => crate::configuration::provisioned_floor(browser).await?,
        "/operations" => {
            // NOTE: leptos-ui.md §5: a text area carries its text in the server HTML, so
            // Run posts the input with no WebAssembly.
            let input = browser.element(By::Id("pair-answered-input")).await?;
            assert!(
                input
                    .prop("value")
                    .await?
                    .is_some_and(|text| text.contains("ctx/language")),
                "the input pane's editor holds its text"
            );
            browser
                .element(By::Css("#pair-answered-run[type='submit']"))
                .await?;
            let raw = text_of(browser, By::Css(".fui-hl7-raw")).await?;
            assert!(raw.starts_with("MSH|"), "the raw message shows: {raw}");
        }
        "/clinical" => {
            // NOTE: leptos-ui.md §15.8: At a time, the switch and the builder are links
            // and forms, so each works with no WebAssembly.
            let subject = text_of(browser, By::Css("#subject .fui-subject")).await?;
            assert!(
                subject.contains("demo.hospital.mrn"),
                "the subject line names the namespace: {subject}"
            );
            browser
                .element(By::Css(
                    "form.fui-at-time[method='get'] button[type='submit']",
                ))
                .await?;
            browser
                .element(By::Css("#results nav.fui-segmented a[aria-current='true']"))
                .await?;
            browser
                .element(By::Css(
                    "#query button[type='submit'][name='remove'][value='c1']",
                ))
                .await?;
        }
        "/terminology" => terminology_content(browser).await?,
        "/overlays" => overlays_content(browser).await?,
        "/charts" => colour_key(browser).await?,
        "/layouts" => {
            let items = browser
                .driver()
                .find_all(By::Css("#gallery-tree [role='treeitem']"))
                .await?;
            assert!(!items.is_empty(), "the tree shows its items");
            crate::widgets::help_switch_journey(browser).await?;
        }
        "/trees" => trees_content(browser).await?,
        "/preferences" | "/preferences/language" | "/preferences/keyboard" => {
            preferences_content(browser, page).await?;
        }
        templates if templates.starts_with("/templates/") => {
            templates_content(browser, templates).await?;
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
                // A section's entry (Your preferences) marks each of its tabs.
                let marked = current.attr("href").await?.unwrap_or_default();
                assert!(
                    marked == path || path.starts_with(&format!("{marked}/")),
                    "the navigation marks {path}, not {marked}"
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
        // A tree folds by its toggle's link, and the server reads the level.
        browser
            .goto(&gallery.url("/ferroterm/trees?open=SF-1000&concept=SF-1000"))
            .await?;
        browser
            .element(By::Css(
                "#trees-deep a.fui-tree-toggle[aria-label='Expand Finding of the skin']",
            ))
            .await?
            .click()
            .await?;
        browser.text("a", "Rash").await?;
        browser
            .element(By::Css(
                "#trees-deep a.fui-tree-toggle[aria-label='Collapse Finding of the skin']",
            ))
            .await?;
        // The side panel's opener leads to its content's own page.
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        browser
            .element(By::Css("a[href='/ferrofed/overlays/access']"))
            .await?
            .click()
            .await?;
        browser
            .at(&gallery.url("/ferrofed/overlays/access"))
            .await?;
        browser.text("h1", "Access").await?;
        browser.text("dd", "Synthetic operator").await?;
        leave(browser, "the links").await
    })
    .await
}

/// The trees page with no WebAssembly: every tree draws its items, the
/// concept browser's toggles are links, and the edit's actions are submit
/// buttons of its form.
async fn trees_content(browser: &Browser) -> Result<(), Box<dyn Error>> {
    for tree in [
        "trees-deep",
        "trees-long",
        "trees-directory",
        "trees-directory-edit",
        "trees-paths",
    ] {
        let items = browser
            .driver()
            .find_all(By::Css(format!("#{tree} [role='treeitem']")))
            .await?;
        assert!(!items.is_empty(), "the tree {tree} shows its items");
    }
    browser
        .element(By::Css("#trees-deep a.fui-tree-toggle[href*='open=']"))
        .await?;
    browser.text("li", "Reading the children").await?;
    browser
        .element(By::Css(
            "form #trees-directory-edit button[type='submit'][name='rename'][value='follow-up']",
        ))
        .await?;
    Ok(())
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
        // comes back in the choice; so does the density on Your preferences,
        // posted by its Save button.
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
        browser.click_and_load(&dark).await?;
        browser
            .element(By::Css(r#"html[data-theme="dark"]"#))
            .await?;
        preferences_post_and_reset(browser, &gallery).await?;
        // The sidebar's Collapse posts, and the page comes back as the rail.
        let collapse = browser
            .element(By::XPath("//button[@name = 'sidebar' and @value = 'rail']"))
            .await?;
        browser.click_and_load(&collapse).await?;
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
async fn the_code_editor_posts_its_draft_unchanged_with_the_webassembly_withheld()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_webassembly(browser).await?;
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        let draft = browser.element(By::Id("gallery-mapping")).await?;
        assert_eq!(
            Some(String::from(ferrohealth_ui_gallery::data::DRAFT_TEXT)),
            draft.prop("value").await?,
            "the server wrote the draft into its text area, markup and all"
        );
        let check = browser
            .element(By::XPath(
                "//form[.//textarea[@id = 'gallery-mapping']]//button[@type = 'submit']",
            ))
            .await?;
        browser.click_and_load(&check).await?;
        browser.at(&gallery.url("/draft")).await?;
        browser.element(By::Id("received")).await?;
        let received = browser
            .driver()
            .execute(
                "return document.getElementById('received').textContent;",
                Vec::new(),
            )
            .await?;
        assert_eq!(
            Some(ferrohealth_ui_gallery::data::DRAFT_TEXT),
            received.json().as_str(),
            "the server received the draft unchanged"
        );
        leave(browser, "the code editor's post").await
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
        let next = browser.element(next).await?;
        browser.click_and_load(&next).await?;
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
