// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keys the `keys` module binds (#526, #530): a chord a control
//! announces pressed from inside a field (Run's Ctrl Enter, the dock's
//! Ctrl S), a single key outside one (the dock's reason on `r`, Discard on
//! `d`), a go-to
//! pair of the legend, and a work-list row's `o` that opens the focused
//! row's side panel and returns the focus to the row; none below 40 rem,
//! nothing with the modules refused, and the work-list reflowed at 320 px.

use std::error::Error;

use serde_json::json;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The CSS of the work-list's rows.
const ROWS: &str = ".fui-worklist-grid tbody tr[data-key]";

/// Presses `keys` on whatever holds the focus.
async fn press(browser: &Browser, keys: impl Into<TypingData>) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .send_keys(keys)
        .perform()
        .await?;
    Ok(())
}

/// Presses `key` with Control held.
async fn chord(browser: &Browser, key: impl Into<TypingData>) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .action_chain()
        .key_down(Key::Control)
        .send_keys(key)
        .key_up(Key::Control)
        .perform()
        .await?;
    Ok(())
}

/// Runs `script` in the page with `args` and returns what it answers.
async fn run(
    browser: &Browser,
    script: &str,
    args: Vec<serde_json::Value>,
) -> Result<serde_json::Value, Box<dyn Error>> {
    Ok(browser.driver().execute(script, args).await?.json().clone())
}

/// The id of the element that holds the focus, or its `data-key`.
async fn focused(browser: &Browser) -> Result<String, Box<dyn Error>> {
    let active = browser.driver().active_element().await?;
    Ok(match active.attr("id").await? {
        Some(id) if !id.is_empty() => id,
        _ => active.attr("data-key").await?.unwrap_or_default(),
    })
}

/// Moves the focus to the page itself, outside every control.
async fn blur(browser: &Browser) -> Result<(), Box<dyn Error>> {
    run(browser, "document.activeElement.blur();", vec![]).await?;
    Ok(())
}

/// Waits until the element `css` names has been enhanced by its modules.
async fn ready(browser: &Browser, css: &str) -> Result<WebElement, Box<dyn Error>> {
    browser
        .element(By::Css(format!("{css}[data-fui-ready]")))
        .await
}

/// Keeps every submit in the page from leaving it, and records which
/// control submitted, as `window.submitted`.
async fn catch_submits(browser: &Browser) -> Result<(), Box<dyn Error>> {
    run(
        browser,
        "window.submitted = ''; document.addEventListener('submit', (event) => { \
           event.preventDefault(); window.submitted = event.submitter ? event.submitter.id : '?'; });",
        vec![],
    )
    .await?;
    Ok(())
}

/// The control that submitted last, once [`catch_submits`] runs.
async fn submitted(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(run(browser, "return window.submitted;", vec![])
        .await?
        .as_str()
        .unwrap_or_default()
        .to_owned())
}

#[tokio::test(flavor = "multi_thread")]
async fn ctrl_enter_runs_the_form_it_is_pressed_in_from_inside_a_field()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrobridge/operations"))
            .await?;
        ready(browser, "#pair-answered-run").await?;
        catch_submits(browser).await?;
        let format = browser.element(By::Id("pair-answered-format")).await?;
        run(browser, "arguments[0].focus();", vec![format.to_json()?]).await?;
        chord(browser, Key::Enter).await?;
        until(
            "Ctrl Enter presses the Run of the focused field's form",
            async || Ok(submitted(browser).await? == "pair-answered-run"),
        )
        .await?;
        assert_eq!(
            "pair-answered-format",
            focused(browser).await?,
            "a chord in a field leaves the focus in the field"
        );
        run(browser, "window.submitted = '';", vec![]).await?;
        blur(browser).await?;
        chord(browser, Key::Enter).await?;
        until(
            "outside a form Ctrl Enter presses the first Run",
            async || Ok(submitted(browser).await? == "pair-waiting-run"),
        )
        .await?;
        browser.console_clean("Ctrl Enter").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn ctrl_s_saves_through_the_dock_and_r_moves_to_its_reason() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferroehr/forms?dock=idle"))
            .await?;
        ready(browser, "#gallery-dock button[value='save']").await?;
        let reason = browser.element(By::Id("dock-reason")).await?;
        let before = reason.prop("value").await?.unwrap_or_default();
        let length = before.chars().count();
        blur(browser).await?;
        press(browser, "r").await?;
        until("r moves the focus to the reason", async || {
            Ok(focused(browser).await? == "dock-reason")
        })
        .await?;
        assert_eq!(
            Some(before),
            reason.prop("value").await?,
            "the key is not typed into the field"
        );
        press(browser, "r").await?;
        let typed = reason.prop("value").await?.unwrap_or_default();
        assert_eq!(
            length + 1,
            typed.chars().count(),
            "in the field r is text: {typed}"
        );
        chord(browser, "s").await?;
        until("Ctrl S from inside the reason saves", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("decision=save"))
        })
        .await?;
        // A single key presses its decision outside a field.
        browser
            .goto(&gallery.url("/ferroehr/forms?dock=idle"))
            .await?;
        ready(browser, "#gallery-dock button[value='discard']").await?;
        blur(browser).await?;
        press(browser, "d").await?;
        until("d presses Discard", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("decision=discard"))
        })
        .await?;
        // A phone has no keyboard: r and d are no shortcuts there (family rule 6).
        window(browser, 390, 844).await?;
        browser
            .goto(&gallery.url("/ferroehr/forms?dock=idle"))
            .await?;
        ready(browser, "#dock-reason").await?;
        blur(browser).await?;
        press(browser, "r").await?;
        assert_ne!(
            "dock-reason",
            focused(browser).await?,
            "no key below 40 rem"
        );
        blur(browser).await?;
        press(browser, "d").await?;
        assert!(
            !browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("decision="),
            "no decision below 40 rem"
        );
        browser.console_clean("the dock's keys").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_modules_refused_the_dock_s_keys_do_nothing_and_its_save_still_posts()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        let cdp = browser.driver().cdp();
        cdp.send_raw("Network.enable", json!({})).await?;
        cdp.send_raw(
            "Network.setBlockedURLs",
            json!({ "urls": ["*/ferrohealth-ui/*/scripts/*"] }),
        )
        .await?;
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferroehr/forms?dock=idle"))
            .await?;
        let save = browser
            .element(By::Css("#gallery-dock button[value='save']"))
            .await?;
        assert_eq!(
            Some(String::from("Control+S")),
            save.attr("aria-keyshortcuts").await?,
            "the key is announced with no script"
        );
        blur(browser).await?;
        press(browser, "r").await?;
        assert_ne!("dock-reason", focused(browser).await?, "no module, no key");
        press(browser, "d").await?;
        assert!(
            !browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("decision="),
            "no module, no decision by key"
        );
        save.click().await?;
        until("Save posts by the pointer", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("decision=save"))
        })
        .await?;
        browser.goto("about:blank").await?;
        let others: Vec<String> = browser
            .console_errors()
            .await?
            .into_iter()
            .filter(|entry| !entry.contains("/scripts/"))
            .collect();
        assert!(others.is_empty(), "{others:?}");
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_go_to_pair_of_the_legend_follows_its_link_outside_a_field() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        ready(browser, "dialog#shortcuts").await?;
        let pair = browser
            .element(By::Css(
                "#shortcuts [data-fui-go='g'] a[data-fui-go-key='t']",
            ))
            .await?;
        assert_eq!(
            Some(String::from("/ferrofed/tables")),
            pair.attr("href").await?
        );
        // Typed into a field the pair is text.
        let search = browser.element(By::Id("gallery-search")).await?;
        search.click().await?;
        press(browser, "gt").await?;
        assert_eq!(Some(String::from("gt")), search.prop("value").await?);
        assert!(
            browser
                .driver()
                .current_url()
                .await?
                .path()
                .ends_with("/overlays")
        );
        // A first key with no pair after it leads nowhere.
        blur(browser).await?;
        press(browser, "gz").await?;
        assert!(
            browser
                .driver()
                .current_url()
                .await?
                .path()
                .ends_with("/overlays")
        );
        blur(browser).await?;
        press(browser, "gt").await?;
        browser.at(&gallery.url("/ferrofed/tables")).await?;
        browser.console_clean("a go-to pair").await
    })
    .await
}

/// Opens FerroPIX's tables page and gives the work-list's first row the
/// focus once its modules have run.
async fn first_row(browser: &Browser, gallery: &Gallery) -> Result<(), Box<dyn Error>> {
    browser.goto(&gallery.url("/ferropix/tables")).await?;
    ready(browser, ".fui-worklist-grid").await?;
    ready(browser, "button[commandfor='pair-02-glance']").await?;
    let first = browser
        .element(By::Css(format!("{ROWS}[tabindex='0']")))
        .await?;
    run(
        browser,
        "arguments[0].scrollIntoView({block: 'center'}); arguments[0].focus();",
        vec![first.to_json()?],
    )
    .await?;
    Ok(())
}

/// Whether the side panel `id` is open as a modal.
async fn panel_open(browser: &Browser, id: &str) -> Result<bool, Box<dyn Error>> {
    Ok(run(
        browser,
        "return document.getElementById(arguments[0]).matches(':modal');",
        vec![json!(id)],
    )
    .await?
    .as_bool()
        == Some(true))
}

#[tokio::test(flavor = "multi_thread")]
async fn o_opens_the_focused_row_s_panel_and_closing_it_returns_the_focus_to_the_row()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        first_row(browser, &gallery).await?;
        press(browser, "j").await?;
        until("j moves to the second row", async || {
            Ok(focused(browser).await? == "pair-02")
        })
        .await?;
        press(browser, "o").await?;
        until("o opens the focused row's panel", async || {
            panel_open(browser, "pair-02-glance").await
        })
        .await?;
        assert!(
            !panel_open(browser, "pair-01-glance").await?,
            "only the focused row's panel opens"
        );
        press(browser, Key::Escape).await?;
        until("Escape closes the panel", async || {
            Ok(!panel_open(browser, "pair-02-glance").await?)
        })
        .await?;
        until("the focus returns to the row", async || {
            Ok(focused(browser).await? == "pair-02")
        })
        .await?;
        // Outside a row the row's key reaches nothing.
        blur(browser).await?;
        press(browser, "o").await?;
        let open = run(
            browser,
            "return document.querySelector('dialog:modal') !== null;",
            vec![],
        )
        .await?;
        assert_eq!(Some(false), open.as_bool(), "o outside a row opens nothing");
        // A phone has no keyboard (family rule 6).
        window(browser, 390, 844).await?;
        first_row(browser, &gallery).await?;
        press(browser, "o").await?;
        assert!(
            !panel_open(browser, "pair-01-glance").await?,
            "no key below 40 rem"
        );
        browser.console_clean("the row's key").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_modules_refused_a_row_s_glance_is_a_link_and_no_key_acts()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        let cdp = browser.driver().cdp();
        cdp.send_raw("Network.enable", json!({})).await?;
        cdp.send_raw(
            "Network.setBlockedURLs",
            json!({ "urls": ["*/ferrohealth-ui/*/scripts/*"] }),
        )
        .await?;
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferropix/tables")).await?;
        let opener = browser
            .element(By::Css("button[commandfor='pair-01-glance']"))
            .await?;
        assert!(
            !opener.is_displayed().await?,
            "the button waits for its module"
        );
        let link = browser
            .element(By::XPath(
                "//button[@commandfor = 'pair-01-glance']/preceding-sibling::a[1]",
            ))
            .await?;
        assert!(
            link.is_displayed().await?,
            "the glance is a link with no script"
        );
        assert_eq!(
            Some(String::from("/ferropix/layouts")),
            link.attr("href").await?
        );
        let row = browser
            .element(By::Css(format!("{ROWS}[data-key='pair-01']")))
            .await?;
        run(browser, "arguments[0].focus();", vec![row.to_json()?]).await?;
        press(browser, "o").await?;
        assert!(
            !panel_open(browser, "pair-01-glance").await?,
            "no module, no key"
        );
        browser.goto("about:blank").await?;
        let others: Vec<String> = browser
            .console_errors()
            .await?
            .into_iter()
            .filter(|entry| !entry.contains("/scripts/"))
            .collect();
        assert!(others.is_empty(), "{others:?}");
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_work_list_with_its_glances_reflows_at_320_pixels() -> Result<(), Box<dyn Error>> {
    // WCAG 2.2 SC 1.4.10: the row's opener and menu stay inside the page.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 320, 800).await?;
        browser.goto(&gallery.url("/ferropix/tables")).await?;
        ready(browser, ".fui-worklist-grid").await?;
        let widths = run(
            browser,
            "const root = document.documentElement; return [root.scrollWidth, root.clientWidth];",
            vec![],
        )
        .await?;
        let page = widths[0].as_f64().unwrap_or_default();
        let view = widths[1].as_f64().unwrap_or_default();
        assert!(
            page <= view + 0.5,
            "the page scrolls sideways: {page} in {view}"
        );
        browser.console_clean("the work-list at 320 px").await
    })
    .await
}
