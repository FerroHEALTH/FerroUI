// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys: every interactive component driven in the browser, every
//! page of every brand loaded with no console error, the narrow screen's
//! drawer, and the 320 px reflow.

use std::error::Error;

use ferrohealth_ui::style::Brand;
use serde_json::json;
use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The pages of a brand, after its key, in ascending order
/// (`scripts/checks/mod-order.sh`).
pub(crate) const PAGES: &[&str] = &[
    "",
    "/charts",
    "/clinical",
    "/code",
    "/configuration",
    "/configuration/provisioned",
    "/federation",
    "/find",
    "/forms",
    "/layouts",
    "/operations",
    "/overlays",
    "/preferences",
    "/preferences/keyboard",
    "/preferences/language",
    "/tables",
    "/templates/about",
    "/templates/detail",
    "/templates/forbidden",
    "/templates/list",
    "/templates/list-empty",
    "/templates/list-many",
    "/templates/list-tabs",
    "/templates/not-found",
    "/templates/settings",
    "/templates/settings-managed",
    "/terminology",
    "/trees",
];

/// The sign-in states the gallery draws.
pub(crate) const SIGN_INS: [&str; 12] = ferrohealth_ui_gallery::pages::sign_in::STATES;

/// Lays the page out `width` CSS pixels wide and `height` tall.
pub(crate) async fn window(
    browser: &Browser,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .cdp()
        .send_raw(
            "Emulation.setDeviceMetricsOverride",
            json!({ "width": width, "height": height, "deviceScaleFactor": 1, "mobile": false }),
        )
        .await?;
    Ok(())
}

/// Waits until the overlays page has hydrated: its ok toast button raises a
/// toast, which carries an action and so waits for its Dismiss.
pub(crate) async fn hydrated_toasts(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let raise = browser.element(By::Css("button[data-toast='ok']")).await?;
    until("the toasts answer a press", async || {
        raise.click().await?;
        Ok(!browser
            .driver()
            .find_all(By::XPath(
                "//div[contains(@class, 'fui-toast ')]/div/strong[contains(., 'Saved, and the set reloaded.')]",
            ))
            .await?
            .is_empty())
    })
    .await
}

/// Waits until the overview has hydrated: its keyboard tabs answer an
/// arrow key.
pub(crate) async fn hydrated_tabs(browser: &Browser) -> Result<(), Box<dyn Error>> {
    let first = browser.element(By::Id("answer-tab-0")).await?;
    until("the tabs answer the keyboard", async || {
        first.click().await?;
        first.send_keys(Key::Right).await?;
        let second = browser.driver().find(By::Id("answer-tab-1")).await?;
        Ok(second.attr("aria-selected").await?.as_deref() == Some("true"))
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn every_page_of_every_brand_loads_with_no_console_error() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/")).await?;
        browser.text("p", "Choose a brand").await?;
        browser.console_clean("the front page").await?;
        for brand in Brand::ALL {
            for &page in PAGES {
                let path = format!("/{}{page}", brand.key());
                browser.goto(&gallery.url(&path)).await?;
                browser.element(By::Css("main h1")).await?;
                browser.console_clean(&path).await?;
            }
            for variant in SIGN_INS {
                let path = format!("/sign-in/{}/{variant}", brand.key());
                browser.goto(&gallery.url(&path)).await?;
                browser.element(By::Css(".fui-sign-in")).await?;
                browser.console_clean(&path).await?;
            }
        }
        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_keyboard_tabs_follow_the_arrows_home_and_end() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed")).await?;
        hydrated_tabs(browser).await?;
        let second = browser.element(By::Id("answer-tab-1")).await?;
        second.send_keys(Key::End).await?;
        let last = browser.element(By::Id("answer-tab-2")).await?;
        until("End selects the last tab", async || {
            Ok(last.attr("aria-selected").await?.as_deref() == Some("true"))
        })
        .await?;
        assert!(
            browser
                .element(By::Id("answer-panel-2"))
                .await?
                .is_displayed()
                .await?
        );
        assert!(
            !browser
                .element(By::Id("answer-panel-0"))
                .await?
                .is_displayed()
                .await?
        );
        last.send_keys(Key::Right).await?;
        let first = browser.element(By::Id("answer-tab-0")).await?;
        until("Right wraps to the first tab", async || {
            Ok(first.attr("aria-selected").await?.as_deref() == Some("true"))
        })
        .await?;
        browser.console_clean("the tabs").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_dialog_confirms_and_a_toast_says_so() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr/overlays")).await?;
        hydrated_toasts(browser).await?;
        // The toasts' button scrolled the page; the title row's More sits at its top.
        browser
            .driver()
            .execute("window.scrollTo(0, 0);", Vec::new())
            .await?;
        // The delete is More's last item, never a button beside the primary.
        crate::menus::hydrated_trigger(browser, "overlays-more")
            .await?
            .click()
            .await?;
        browser
            .element(By::XPath(
                "//button[@role = 'menuitem' and contains(., 'Delete the file')]",
            ))
            .await?
            .click()
            .await?;
        let dialog = browser.element(By::Id("delete-file")).await?;
        until("the dialog opens", async || {
            Ok(dialog.is_displayed().await?)
        })
        .await?;
        dialog
            .find(By::Id("confirm-delete-file"))
            .await?
            .click()
            .await?;
        browser
            .text("strong", "The synthetic file was deleted.")
            .await?;
        until("the dialog closes", async || {
            Ok(!dialog.is_displayed().await?)
        })
        .await?;
        browser.console_clean("the dialog").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn an_informational_dialog_opens_on_close_and_gives_the_focus_back()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/overlays")).await?;
        let trigger = browser
            .element(By::Css("button[commandfor='how-reloading']"))
            .await?;
        trigger.scroll_into_view().await?;
        trigger.click().await?;
        let dialog = browser.element(By::Id("how-reloading")).await?;
        until("the dialog opens", async || {
            Ok(dialog.is_displayed().await?)
        })
        .await?;
        // NOTE: WAI-ARIA Authoring Practices, the dialog (modal) pattern: a dialog that asks nothing focuses Close.
        let active = browser.driver().active_element().await?;
        assert_eq!(Some(String::from("close")), active.attr("command").await?);
        assert_eq!(
            1,
            dialog.find_all(By::Css("button")).await?.len(),
            "Close alone"
        );
        active.send_keys(Key::Enter).await?;
        until("Close closes it", async || {
            Ok(!dialog.is_displayed().await?)
        })
        .await?;
        let back = browser.driver().active_element().await?;
        assert_eq!(
            Some(String::from("how-reloading")),
            back.attr("commandfor").await?,
            "the focus returns to the trigger"
        );
        browser.console_clean("the informational dialog").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_keeps_every_byte_typed_and_a_diagnostic_leads_to_its_line()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let draft = browser.element(By::Id("gallery-draft")).await?;
        until("the editor hydrates", async || {
            browser.editor_live(&draft).await
        })
        .await?;
        draft.clear().await?;
        let expected = format!(
            "{}# typed: naïve ✓ <b>&\n",
            ferrohealth_ui_gallery::data::YAML
        );
        draft.send_keys(&expected).await?;
        until("the text area holds every byte typed", async || {
            Ok(draft.prop("value").await?.as_deref() == Some(expected.as_str()))
        })
        .await?;
        browser
            .text("p", &format!("{} bytes", expected.len()))
            .await?;
        let layer = browser
            .element(By::XPath(
                "//textarea[@id = 'gallery-draft']/preceding-sibling::pre[contains(@class, 'fui-editor-layer')]",
            ))
            .await?;
        assert!(
            layer.text().await?.contains("# typed: naïve ✓ <b>&"),
            "the layer draws what was typed"
        );
        browser.follow("line 13").await?;
        let url = browser.driver().current_url().await?;
        assert_eq!(Some("gallery-draft-L13"), url.fragment(), "{url}");
        browser
            .element(By::Css("#gallery-draft-L13.fui-mark-error"))
            .await?;
        browser.console_clean("the editor").await
    })
    .await
}

/// The editor `gallery-draft`'s layer as the page holds it: each line's id
/// and text, and the text area's value.
#[derive(Debug, serde::Deserialize)]
struct Layer {
    /// The id of each line element, in order.
    ids: Vec<String>,
    /// The text of each line element, in order.
    lines: Vec<String>,
    /// The text area's value.
    value: String,
    /// The mark each kept line element still carries, by its id.
    kept: Vec<Option<u32>>,
}

/// Reads the layer of the editor `gallery-draft`.
async fn layer(browser: &Browser) -> Result<Layer, Box<dyn Error>> {
    measure(
        browser,
        r"const area = document.getElementById('gallery-draft');
        const lines = Array.from(area.previousElementSibling.children);
        return { ids: lines.map((line) => line.id), lines: lines.map((line) => line.textContent),
          value: area.value, kept: lines.map((line) => line.fuiKept ?? null) };",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_redraws_only_the_lines_an_edit_changes() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let draft = browser.element(By::Id("gallery-draft")).await?;
        until("the editor hydrates", async || {
            browser.editor_live(&draft).await
        })
        .await?;
        // Each line element carries its number as a mark the page never
        // writes, so a redrawn element comes back without it.
        browser
            .driver()
            .execute(
                r"const area = document.getElementById('gallery-draft');
                Array.from(area.previousElementSibling.children)
                  .forEach((line, at) => { line.fuiKept = at + 1; });
                area.focus();
                const start = area.value.split('\n').slice(0, 2).join('\n').length + 1;
                area.setSelectionRange(start, start);",
                Vec::new(),
            )
            .await?;
        let before = layer(browser).await?;
        let count = before.ids.len();
        browser
            .driver()
            .action_chain()
            .send_keys("x")
            .send_keys(Key::Enter)
            .perform()
            .await?;
        until("the layer draws the new line", async || {
            Ok(layer(browser).await?.ids.len() == count + 1)
        })
        .await?;
        let after = layer(browser).await?;
        assert_eq!(
            after.value,
            after.lines.join("\n"),
            "the layer draws the text area's text, line by line"
        );
        let expected: Vec<String> = (1..=count + 1)
            .map(|line| format!("gallery-draft-L{line}"))
            .collect();
        assert_eq!(expected, after.ids, "the lines are numbered in order");
        assert_eq!(
            vec![Some(1), Some(2)],
            after.kept[..2],
            "the lines above the edit keep their elements"
        );
        assert_eq!(
            (4..=u32::try_from(count)?).map(Some).collect::<Vec<_>>(),
            after.kept[4..],
            "the lines below the edit keep their elements under their new numbers"
        );
        browser.console_clean("the editor's redraw").await
    })
    .await
}

/// The text of the code editor's live region, which says the diagnostic of
/// the line the cursor is on.
async fn said(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .element(By::Css(
            ".fui-editor-split .fui-editor-status [role='status']",
        ))
        .await?
        .text()
        .await?)
}

/// Follows the code editor's diagnostic of line `line`, beside it.
async fn follow_diagnostic(browser: &Browser, line: u32) -> Result<(), Box<dyn Error>> {
    let place = browser
        .element(By::Css(format!(
            ".fui-editor-split a.fui-diag-where[href='#gallery-mapping-L{line}']"
        )))
        .await?;
    // A row the page scrolled under the sticky top bar is scrolled back
    // first, as a reader would, since WebDriver clicks it where it lies.
    place.scroll_into_view().await?;
    place.click().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_code_editor_keeps_its_cursor_its_marks_and_its_diagnostics_in_step()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        let draft = browser.element(By::Id("gallery-mapping")).await?;
        until("the code editor hydrates", async || {
            browser.editor_live(&draft).await
        })
        .await?;
        browser.text("span", "Changed since opened").await?;
        assert!(said(browser).await?.is_empty(), "the status says nothing at rest");
        // Following a diagnostic moves the caret to its line and column, and
        // the line and its row turn current together.
        follow_diagnostic(browser, 16).await?;
        browser
            .element(By::Css("#gallery-mapping-L16.fui-mark-error.is-current"))
            .await?;
        let row = browser
            .element(By::Css(".fui-editor-split li.fui-diag.is-current[aria-current='true']"))
            .await?;
        assert!(row.text().await?.contains("FC-201"), "the error's row is current");
        browser.text("span", "Ln 16, Col 12").await?;
        until("the status says the line's error", async || {
            Ok(said(browser).await? == "Line 16 has an error.")
        })
        .await?;
        // The editor is the one tab stop and takes the focus.
        until("the editor takes the focus", async || {
            let focused = browser
                .driver()
                .execute("return document.activeElement.id;", Vec::new())
                .await?;
            Ok(focused.json().as_str() == Some("gallery-mapping"))
        })
        .await?;
        // Typing at the caret: the text area and the layer follow, and the
        // caret moves on.
        draft.send_keys("0").await?;
        until("the text area holds what was typed", async || {
            Ok(draft
                .prop("value")
                .await?
                .is_some_and(|value| value.contains("retries: 0three")))
        })
        .await?;
        let layer = browser
            .element(By::XPath(
                "//textarea[@id = 'gallery-mapping']/preceding-sibling::pre[contains(@class, 'fui-editor-layer')]",
            ))
            .await?;
        assert!(
            layer.text().await?.contains("retries: 0three"),
            "the layer draws what was typed"
        );
        browser.text("span", "Ln 16, Col 13").await?;
        // The warning's row takes its turn.
        follow_diagnostic(browser, 6).await?;
        until("the status says the line's warning", async || {
            Ok(said(browser).await? == "Line 6 has a warning.")
        })
        .await?;
        browser
            .element(By::Css("#gallery-mapping-L6.fui-mark-warning.is-current"))
            .await?;
        assert_eq!(
            1,
            browser
                .driver()
                .find_all(By::Css(".fui-editor-split li.fui-diag.is-current"))
                .await?
                .len(),
            "one row is current"
        );
        browser.console_clean("the code editor").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_url_pager_is_links_the_sort_a_get_form_and_the_session_pager_posts()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/tables")).await?;
        browser.text("p", "1-25 of 137").await?;
        let url_rows = By::XPath("(//table)[1]/tbody/tr");
        assert_eq!(25, browser.driver().find_all(url_rows).await?.len());
        // The label names what a reader sees: the box shows the whole table.
        let region = browser.element(By::Css(".fui-table-scroll")).await?;
        let table = region.find(By::Css("table")).await?;
        assert!(
            region.rect().await?.height >= table.rect().await?.height,
            "the table box clips its rows"
        );
        let next =
            By::XPath("//nav[@aria-label = 'Pages of the mapping files']//a[contains(., 'Next')]");
        browser.element(next).await?.click().await?;
        until("the page is in the URL", async || {
            Ok(browser
                .driver()
                .current_url()
                .await?
                .as_str()
                .contains("page=2"))
        })
        .await?;
        browser.text("p", "26-50 of 137").await?;
        // design/COMPONENTS.md, § The table: a sort is a button in a GET form.
        browser
            .element(By::XPath("(//table)[1]//th[contains(., 'Lines')]//button"))
            .await?
            .click()
            .await?;
        until("the sort is in the URL", async || {
            let url = browser.driver().current_url().await?;
            Ok(url.as_str().contains("sort=lines") && !url.as_str().contains("page="))
        })
        .await?;
        browser
            .element(By::XPath(
                "//th[@aria-sort = 'ascending' and contains(., 'Lines')]",
            ))
            .await?;
        browser.console_clean("the URL pager and the sort").await?;
        session_pager_posts(browser, &gallery).await?;
        browser.console_clean("the session pager").await
    })
    .await
}

/// Pages the session listing to its last page through its `POST` form, and
/// checks that neither the address nor the history names the position.
async fn session_pager_posts(browser: &Browser, gallery: &Gallery) -> Result<(), Box<dyn Error>> {
    browser.goto(&gallery.url("/ferrobridge/tables")).await?;
    let before = browser.driver().current_url().await?;
    let last = By::XPath(
        "//nav[@aria-label = 'Pages of the mapping files, in the page']//button[contains(., 'Last')]",
    );
    let last = browser.element(last).await?;
    browser
        .click_in_page(&last, "the session pager draws the last page", async || {
            Ok(browser
                .driver()
                .find_all(By::XPath("//p[contains(., '126-137 of 137')]"))
                .await?
                .len()
                == 1)
        })
        .await?;
    let session_rows = By::XPath("(//table)[2]/tbody/tr");
    assert_eq!(12, browser.driver().find_all(session_rows).await?.len());
    assert_eq!(
        before,
        browser.driver().current_url().await?,
        "the session pager puts no position in the URL"
    );
    browser.driver().back().await?;
    let earlier = browser.driver().current_url().await?;
    assert!(
        !earlier.as_str().contains("page=") && !earlier.as_str().contains("size="),
        "no entry of the history names the position: {earlier}"
    );
    Ok(())
}

/// The text of the element that has the focus.
async fn focused_text(browser: &Browser) -> Result<String, Box<dyn Error>> {
    Ok(browser
        .driver()
        .active_element()
        .await?
        .text()
        .await?
        .trim()
        .to_owned())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_user_menu_posts_the_theme_and_the_choice_is_kept() -> Result<(), Box<dyn Error>> {
    // Family rule 5: the theme is set in the user menu; rule 9: kept in a cookie.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser
            .driver()
            .cdp()
            .send_raw(
                "Emulation.setEmulatedMedia",
                json!({ "features": [{ "name": "prefers-color-scheme", "value": "light" }] }),
            )
            .await?;
        browser.goto(&gallery.url("/ferrofed")).await?;
        browser
            .element(By::Css("button[popovertarget='user-menu']"))
            .await?
            .click()
            .await?;
        let dark = browser
            .element(By::XPath(
                "//*[@id = 'user-menu']//button[@role = 'menuitemradio' and @value = 'dark']",
            ))
            .await?;
        until("the menu opens", async || Ok(dark.is_displayed().await?)).await?;
        browser
            .click_in_page(&dark, "the theme the post kept takes effect", async || {
                Ok(!browser
                    .driver()
                    .find_all(By::Css(r#"html[data-theme="dark"]"#))
                    .await?
                    .is_empty())
            })
            .await?;
        browser.at(&gallery.url("/ferrofed")).await?;
        browser
            .element(By::Css(
                r#"#user-menu button[value="dark"][aria-checked="true"]"#,
            ))
            .await?;
        until("the choice reaches the cookie", async || {
            Ok(browser
                .driver()
                .get_named_cookie("gallery-theme")
                .await
                .is_ok_and(|cookie| cookie.value == "dark"))
        })
        .await?;
        browser.goto(&gallery.url("/ferrofed/code")).await?;
        browser
            .element(By::Css(r#"html[data-theme="dark"]"#))
            .await?;
        assert!(
            browser
                .driver()
                .find_all(By::Css(
                    "#theme-to-dark, #theme-to-light, .fui-theme-toggle"
                ))
                .await?
                .is_empty(),
            "the top bar holds no theme toggle (family rule 5)"
        );
        browser.console_clean("the theme in the user menu").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_keyboard_walks_the_user_menu_and_the_focus_returns_to_its_button()
-> Result<(), Box<dyn Error>> {
    // WAI-ARIA Authoring Practices 1.2, the menu button pattern.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge")).await?;
        hydrated_tabs(browser).await?;
        let button = browser
            .element(By::Css("button[popovertarget='user-menu']"))
            .await?;
        button.focus().await?;
        button.send_keys(Key::Enter).await?;
        until("the button says the menu is open", async || {
            Ok(button.attr("aria-expanded").await?.as_deref() == Some("true"))
        })
        .await?;
        until("the focus is on the first item", async || {
            Ok(focused_text(browser).await? == "Your preferences")
        })
        .await?;
        let active = browser.driver().active_element().await?;
        active.send_keys(Key::Down).await?;
        until("Down moves to the console's own link", async || {
            Ok(focused_text(browser).await? == "Access")
        })
        .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Down)
            .await?;
        until("Down moves to the theme", async || {
            Ok(focused_text(browser).await? == "Light")
        })
        .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::End)
            .await?;
        until("End goes to Sign out, the last item", async || {
            Ok(focused_text(browser).await? == "Sign out")
        })
        .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Down)
            .await?;
        until("Down wraps to the first item", async || {
            Ok(focused_text(browser).await? == "Your preferences")
        })
        .await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Escape)
            .await?;
        let menu = browser.driver().find(By::Id("user-menu")).await?;
        until("Escape closes the menu", async || {
            Ok(!menu.is_displayed().await?)
        })
        .await?;
        until("the focus returns to the button", async || {
            let active = browser.driver().active_element().await?;
            Ok(active.attr("popovertarget").await?.as_deref() == Some("user-menu"))
        })
        .await?;
        until("the button says the menu is closed", async || {
            Ok(button.attr("aria-expanded").await?.as_deref() == Some("false"))
        })
        .await?;
        browser.console_clean("the user menu's keyboard").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn on_a_phone_the_user_menu_is_a_sheet_with_cancel() -> Result<(), Box<dyn Error>> {
    // F13: below 40 rem the user menu opens as a bottom sheet with Cancel.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferropix")).await?;
        browser
            .element(By::Css("button[popovertarget='user-menu']"))
            .await?
            .click()
            .await?;
        let menu = browser.element(By::Id("user-menu")).await?;
        until("the sheet opens", async || Ok(menu.is_displayed().await?)).await?;
        let (bottom, left, right): (f64, f64, f64) = serde_json::from_value(
            browser
                .driver()
                .execute(
                    r"const box = document.getElementById('user-menu').getBoundingClientRect();
                    const view = document.documentElement;
                    return [view.clientHeight - box.bottom, box.left, view.clientWidth - box.right];",
                    vec![],
                )
                .await?
                .json()
                .clone(),
        )?;
        assert!(bottom.abs() < 1.0, "the sheet sits on the screen's foot");
        assert!(left.abs() < 1.0 && right.abs() < 1.0, "the sheet spans the screen");
        let brands = browser
            .element(By::XPath(
                "//*[@id = 'user-menu']//a[@role = 'menuitemradio' and @aria-checked = 'true']",
            ))
            .await?;
        assert!(brands.is_displayed().await?, "the brand choice moves into the sheet");
        let shortcuts = browser
            .driver()
            .find(By::XPath("//*[@id = 'user-menu']//a[contains(., 'Keyboard shortcuts')]"))
            .await?;
        assert!(!shortcuts.is_displayed().await?, "no key hints on a phone");
        browser
            .element(By::Css("#user-menu .fui-menu-cancel"))
            .await?
            .click()
            .await?;
        until("Cancel closes the sheet", async || {
            Ok(!menu.is_displayed().await?)
        })
        .await?;
        browser.console_clean("the user menu's sheet").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_sidebar_folds_to_its_rail_and_back() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/tables")).await?;
        browser
            .element(By::XPath("//button[@name = 'sidebar' and @value = 'rail']"))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css("#sidebar.fui-sidebar-rail"))
            .await?;
        browser.at(&gallery.url("/ferroterm/tables")).await?;
        let width: f64 = serde_json::from_value(
            browser
                .driver()
                .execute(
                    "return document.getElementById('sidebar').getBoundingClientRect().width;",
                    vec![],
                )
                .await?
                .json()
                .clone(),
        )?;
        assert!(width < 70.0, "the rail is the glyphs alone, {width} px");
        browser
            .element(By::XPath(
                "//*[@id = 'sidebar']//a[@aria-current = 'page'][span[@class = 'fui-tooltip' and . = 'Tables and paging']]",
            ))
            .await?;
        browser.goto(&gallery.url("/ferroterm/code")).await?;
        browser
            .element(By::Css("#sidebar.fui-sidebar-rail"))
            .await?;
        browser
            .element(By::XPath("//button[@name = 'sidebar' and @value = 'full']"))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css("#sidebar:not(.fui-sidebar-rail)"))
            .await?;
        browser.console_clean("the rail").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rail_entry_shows_its_words_on_keyboard_focus_until_escape() -> Result<(), Box<dyn Error>>
{
    // WCAG 2.2 SC 1.4.13; shell edge audit, row 13: the collapsed rail showed no words on focus.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroehr/tables")).await?;
        browser
            .element(By::XPath("//button[@name = 'sidebar' and @value = 'rail']"))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css("#sidebar.fui-sidebar-rail"))
            .await?;
        let entry = browser
            .element(By::Css("#sidebar a.fui-nav-entry[aria-current='page']"))
            .await?;
        browser
            .driver()
            .execute("arguments[0].focus();", vec![entry.to_json()?])
            .await?;
        let tooltip = entry.find(By::ClassName("fui-tooltip")).await?;
        until("the focused entry shows its words", async || {
            Ok(tooltip.is_displayed().await?)
        })
        .await?;
        let [tip_left, tip_top, tip_bottom, rail_right, entry_top, entry_bottom]: [f64; 6] =
            browser
                .driver()
                .execute(
                    "const tip = arguments[0].getBoundingClientRect();
                    const rail = document.getElementById('sidebar').getBoundingClientRect();
                    const entry = arguments[1].getBoundingClientRect();
                    return [tip.left, tip.top, tip.bottom, rail.right, entry.top, entry.bottom];",
                    vec![tooltip.to_json()?, entry.to_json()?],
                )
                .await?
                .convert()?;
        assert!(
            tip_left >= rail_right - 1.0 && tip_top < entry_bottom && tip_bottom > entry_top,
            "the tooltip sits beside its entry, past the rail: tooltip {tip_left} {tip_top}-{tip_bottom}, rail right {rail_right}, entry {entry_top}-{entry_bottom}"
        );
        // Escape is the island's, so it is pressed again until the island has hydrated.
        until("Escape hides the tooltip", async || {
            browser
                .driver()
                .action_chain()
                .send_keys(Key::Escape)
                .perform()
                .await?;
            Ok(!tooltip.is_displayed().await?)
        })
        .await?;
        browser
            .driver()
            .action_chain()
            .send_keys(Key::Tab)
            .perform()
            .await?;
        let next = browser.driver().active_element().await?;
        let shown = next.find(By::ClassName("fui-tooltip")).await?;
        until("the next entry shows its words", async || {
            Ok(shown.is_displayed().await?)
        })
        .await?;
        browser
            .element(By::XPath("//button[@name = 'sidebar' and @value = 'full']"))
            .await?
            .click()
            .await?;
        browser
            .element(By::Css("#sidebar:not(.fui-sidebar-rail)"))
            .await?;
        browser.console_clean("the rail's tooltips").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_drawer_opens_takes_the_focus_and_closes_on_a_link() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 600, 900).await?;
        browser.goto(&gallery.url("/ferrobridge")).await?;
        hydrated_tabs(browser).await?;
        browser
            .element(By::XPath("//button[@aria-label = 'Menu']"))
            .await?
            .click()
            .await?;
        let nav = browser.element(By::Id("sidebar")).await?;
        until("the drawer opens", async || Ok(nav.is_displayed().await?)).await?;
        until("the focus is in the drawer", async || {
            let active = browser.driver().active_element().await?;
            Ok(active.text().await? == "Overview")
        })
        .await?;
        browser.follow("Documents").await?;
        browser.at(&gallery.url("/ferrobridge/code")).await?;
        until("the drawer closes", async || {
            Ok(!browser
                .driver()
                .find(By::Id("sidebar"))
                .await?
                .is_displayed()
                .await?)
        })
        .await?;
        browser.console_clean("the drawer").await
    })
    .await
}

/// Waits until the keyboard focus is on the side panel's Close button.
async fn focus_on_close(browser: &Browser) -> Result<(), Box<dyn Error>> {
    until("the focus is on the panel's Close button", async || {
        let active = browser.driver().active_element().await?;
        let class = active.attr("class").await?.unwrap_or_default();
        Ok(class.contains("fui-side-panel-close"))
    })
    .await
}

/// Waits until the keyboard focus is back on the side panel's opener.
async fn focus_on_opener(browser: &Browser) -> Result<(), Box<dyn Error>> {
    until("the focus is back on the panel's opener", async || {
        let active = browser.driver().active_element().await?;
        Ok(active.attr("aria-controls").await?.as_deref() == Some("access"))
    })
    .await
}

/// Opens the side panel from `opener`, waits for the focus on its Close
/// button, and returns the panel.
async fn open_panel(browser: &Browser, opener: &WebElement) -> Result<WebElement, Box<dyn Error>> {
    let panel = browser.element(By::Id("access")).await?;
    opener.click().await?;
    until("the panel opens", async || Ok(panel.is_displayed().await?)).await?;
    focus_on_close(browser).await?;
    Ok(panel)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_side_panel_takes_the_focus_to_its_close_button_and_gives_it_back()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        let opener = browser
            .element(By::Css(
                "button[commandfor='access'][aria-controls='access']",
            ))
            .await?;
        // The button shows once the island has hydrated, and its link hides.
        until("the panel's button shows", async || {
            Ok(opener.is_displayed().await?)
        })
        .await?;
        let link = browser
            .element(By::Css("main a[href='/ferrofed/overlays/access']"))
            .await?;
        assert!(
            !link.is_displayed().await?,
            "the link gives way to the button"
        );
        let panel = open_panel(browser, &opener).await?;
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Escape)
            .await?;
        until("Escape closes the panel", async || {
            Ok(!panel.is_displayed().await?)
        })
        .await?;
        focus_on_opener(browser).await?;
        let panel = open_panel(browser, &opener).await?;
        browser
            .element(By::Css("#access .fui-side-panel-close"))
            .await?
            .click()
            .await?;
        until("Close closes the panel", async || {
            Ok(!panel.is_displayed().await?)
        })
        .await?;
        focus_on_opener(browser).await?;
        browser.console_clean("the side panel").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_side_panel_is_a_sheet_the_size_of_the_viewport_on_a_phone()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 390, 844).await?;
        browser.goto(&gallery.url("/ferropix/overlays")).await?;
        let opener = browser
            .element(By::Css(
                "button[commandfor='access'][aria-controls='access']",
            ))
            .await?;
        until("the panel's button shows", async || {
            Ok(opener.is_displayed().await?)
        })
        .await?;
        opener.scroll_into_view().await?;
        open_panel(browser, &opener).await?;
        // NOTE: CSSOM View, `clientWidth`: a classic scrollbar narrows the root's client box.
        let measured = browser
            .driver()
            .execute(
                "const root = document.documentElement;
                 const panel = document.getElementById('access').getBoundingClientRect();
                 const body = document.getElementById('access-body').getBoundingClientRect();
                 return [panel.left, panel.top, panel.right, panel.bottom,
                         root.clientWidth, root.clientHeight, body.bottom];",
                Vec::new(),
            )
            .await?;
        let [left, top, right, bottom, width, height, body]: [f64; 7] =
            serde_json::from_value(measured.json().clone())?;
        assert!(
            left.abs() < 1.0 && top.abs() < 1.0,
            "the sheet starts at the corner"
        );
        assert!(
            (right - width).abs() < 1.0,
            "the sheet is the viewport's width: {right} against {width}"
        );
        assert!(
            (bottom - height).abs() < 1.0,
            "the sheet is the viewport's height: {bottom} against {height}"
        );
        assert!(
            (bottom - body).abs() < 1.0,
            "the body fills the sheet, with nothing below it: {body} against {bottom}"
        );
        browser.console_clean("the phone's side panel").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn every_overlay_with_a_scrim_holds_the_focus_and_gives_it_back() -> Result<(), Box<dyn Error>>
{
    // WAI-ARIA Authoring Practices, the dialog (modal) pattern; WCAG 2.2 SC 2.4.3 and SC 2.4.11:
    // Tab never reaches the page under the scrim, and closing returns the focus to the opener.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for (width, height) in [(1440, 900), (390, 844)] {
            window(browser, width, height).await?;
            browser.goto(&gallery.url("/ferrofed/overlays")).await?;
            let panel = browser
                .element(By::Css(
                    "button[commandfor='access'][aria-controls='access']",
                ))
                .await?;
            until("the panel's button shows", async || {
                Ok(panel.is_displayed().await?)
            })
            .await?;
            for id in ["reload-set", "save-query", "access"] {
                held(browser, id).await?;
            }
            // Shell edge audit, row 4: below 48 rem the navigation is a drawer.
            if width < 768 {
                held(browser, "sidebar").await?;
            }
        }
        browser.console_clean("the overlays' focus").await
    })
    .await
}

/// Opens the overlay `id` from its opener, walks Tab through it more times
/// than it has controls, and closes it with Escape, checking the focus
/// never lands under it and returns to the opener.
async fn held(browser: &Browser, id: &str) -> Result<(), Box<dyn Error>> {
    let opener = browser
        .element(By::Css(format!(
            "button[command='show-modal'][commandfor='{id}']:not([role])"
        )))
        .await?;
    browser
        .driver()
        .execute(
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![opener.to_json()?],
        )
        .await?;
    opener.click().await?;
    let overlay = browser.element(By::Id(id)).await?;
    until(&format!("{id} opens"), async || {
        Ok(overlay.is_displayed().await?)
    })
    .await?;
    let modal: bool = browser
        .driver()
        .execute(
            "return document.getElementById(arguments[0]).matches(':modal');",
            vec![json!(id)],
        )
        .await?
        .convert()?;
    assert!(modal, "{id} is modal, so the page under it is inert");
    for step in 0..12 {
        browser
            .driver()
            .action_chain()
            .send_keys(Key::Tab)
            .perform()
            .await?;
        let behind: bool = browser
            .driver()
            .execute(
                "const active = document.activeElement;
                return active !== null && active !== document.body
                    && !document.getElementById(arguments[0]).contains(active);",
                vec![json!(id)],
            )
            .await?
            .convert()?;
        assert!(!behind, "Tab {step} left {id} for the page under it");
    }
    browser
        .driver()
        .action_chain()
        .send_keys(Key::Escape)
        .perform()
        .await?;
    until(&format!("Escape closes {id}"), async || {
        Ok(!overlay.is_displayed().await?)
    })
    .await?;
    until(
        &format!("the focus returns to the opener of {id}"),
        async || {
            let active = browser.driver().active_element().await?;
            Ok(
                active.attr("command").await?.as_deref() == Some("show-modal")
                    && active.attr("commandfor").await?.as_deref() == Some(id),
            )
        },
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_puts_the_text_on_the_clipboard_and_says_so() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser
            .driver()
            .cdp()
            .send_raw(
                "Browser.grantPermissions",
                json!({ "permissions": ["clipboardReadWrite", "clipboardSanitizedWrite"] }),
            )
            .await?;
        browser.goto(&gallery.url("/ferrobridge/code")).await?;
        let copy = browser.element(By::Id("file-copy")).await?;
        until("Copy shows once it works", async || {
            Ok(copy.is_displayed().await?)
        })
        .await?;
        copy.scroll_into_view().await?;
        copy.click().await?;
        until("Copy says the text is copied", async || {
            Ok(copy.text().await? == "Copied")
        })
        .await?;
        let held = browser
            .driver()
            .execute_async(
                "navigator.clipboard.readText().then(arguments[0], () => arguments[0](null));",
                Vec::new(),
            )
            .await?;
        assert_eq!(
            Some(ferrohealth_ui_gallery::data::MAPPING),
            held.json().as_str(),
            "the clipboard holds the file"
        );
        browser.console_clean("the copy button").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_reads_the_document_from_the_raw_view_the_page_draws() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser
            .driver()
            .cdp()
            .send_raw(
                "Browser.grantPermissions",
                json!({ "permissions": ["clipboardReadWrite", "clipboardSanitizedWrite"] }),
            )
            .await?;
        browser.goto(&gallery.url("/ferrobridge/code")).await?;
        let copy = browser.element(By::Id("document-copy")).await?;
        until("Copy shows once it works", async || {
            Ok(copy.is_displayed().await?)
        })
        .await?;
        copy.scroll_into_view().await?;
        copy.click().await?;
        until("Copy says the text is copied", async || {
            Ok(copy.text().await? == "Copied")
        })
        .await?;
        let held = browser
            .driver()
            .execute_async(
                "navigator.clipboard.readText().then(arguments[0], () => arguments[0](null));",
                Vec::new(),
            )
            .await?;
        assert_eq!(
            Some(ferrohealth_ui_gallery::data::JSON),
            held.json().as_str(),
            "the clipboard holds the document as the raw view draws it"
        );
        browser.console_clean("the document's copy button").await
    })
    .await
}

/// The part of `Page.getLayoutMetrics` the reflow check reads.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    /// The size of the page's content.
    css_content_size: Size,
    /// The layout viewport.
    css_layout_viewport: Viewport,
}

/// A width.
#[derive(Debug, serde::Deserialize)]
struct Size {
    /// The width, in CSS pixels.
    width: f64,
}

/// The layout viewport.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Viewport {
    /// Its width, in CSS pixels.
    client_width: f64,
}

/// A numeric DOM property as WebDriver returns it, or 0 when absent.
fn number(value: Option<String>) -> f64 {
    value.and_then(|text| text.parse().ok()).unwrap_or(0.0)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_table_header_sticks_below_the_top_bar_on_a_wide_screen() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser
            .goto(&gallery.url("/ferrobridge/tables?size=100"))
            .await?;
        browser.text("p", "1-100 of 137").await?;
        let rows = browser
            .driver()
            .find_all(By::XPath("(//table)[1]/tbody/tr"))
            .await?;
        assert_eq!(100, rows.len());
        let row = rows.get(60).ok_or("no 61st row")?;
        row.scroll_into_view().await?;
        let bar = browser
            .element(By::Css(".fui-topbar"))
            .await?
            .rect()
            .await?;
        let head = browser
            .element(By::XPath("(//table)[1]/thead//th[1]"))
            .await?
            .rect()
            .await?;
        // Rects are in page coordinates; the sticky bar's top is the scroll.
        assert!(bar.y > 100.0, "the page did not scroll ({})", bar.y);
        let below_bar = head.y - (bar.y + bar.height);
        assert!(
            (-1.0..=2.0).contains(&below_bar),
            "the header sits {below_bar} px from the top bar's foot"
        );
        browser.console_clean("the sticky header").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_table_page_reads_whole_beside_the_sidebar_from_64rem() -> Result<(), Box<dyn Error>> {
    // WCAG 2.2 SC 1.4.10: a listing, a query's result set and an empty
    // answer never scroll the page sideways where the table box stops
    // scrolling; the page's own width is the client width, a classic
    // scrollbar left out.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for width in [1024, 1152, 1280] {
            window(browser, width, 900).await?;
            for page in ["/tables", "/clinical"] {
                let path = format!("/ferrobridge{page}");
                browser.goto(&gallery.url(&path)).await?;
                browser.element(By::Css("main h1")).await?;
                let root = browser.element(By::Css("html")).await?;
                let scroll = number(root.prop("scrollWidth").await?);
                let client = number(root.prop("clientWidth").await?);
                assert!(
                    scroll <= client + 1.0,
                    "{path} scrolls sideways at {width} px ({scroll} in {client})"
                );
            }
        }
        browser.console_clean("the table pages from 64rem").await
    })
    .await
}

/// Asserts every table box of the tables page reads whole at 320 px.
async fn table_boxes_read_whole(browser: &Browser) -> Result<(), Box<dyn Error>> {
    // Every table reads whole at 320 px: a data table stacks its rows as
    // cards, and the one that opts out fits its box. The record compare
    // keeps its columns and scrolls inside its own box, a focusable region
    // with the field column held at the start (view::compare::compare,
    // WCAG 2.2 SC 1.4.10).
    let regions = browser
        .driver()
        .find_all(By::Css(".fui-table-scroll"))
        .await?;
    assert!(regions.len() >= 6, "{} table boxes", regions.len());
    for region in regions {
        let class = region.attr("class").await?.unwrap_or_default();
        if class
            .split_whitespace()
            .any(|name| name == "fui-compare-scroll")
        {
            assert_eq!(Some(String::from("0")), region.attr("tabindex").await?);
            assert_eq!(Some(String::from("region")), region.attr("role").await?);
            let field = region.find(By::Css("tbody th")).await?;
            assert_eq!("sticky", field.css_value("position").await?);
            continue;
        }
        let scroll = number(region.prop("scrollWidth").await?);
        let client = number(region.prop("clientWidth").await?);
        assert!(
            scroll <= client + 1.0,
            "a table box scrolls sideways at 320 px ({scroll} in {client})"
        );
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn every_page_reflows_at_320_pixels_with_no_sideways_scroll() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 320, 800).await?;
        for &page in PAGES {
            let path = format!("/ferroehr{page}");
            browser.goto(&gallery.url(&path)).await?;
            browser.element(By::Css("main h1")).await?;
            let metrics: Metrics = serde_json::from_value(
                browser
                    .driver()
                    .cdp()
                    .send_raw("Page.getLayoutMetrics", json!({}))
                    .await?,
            )?;
            assert!(
                metrics.css_content_size.width <= metrics.css_layout_viewport.client_width + 0.5,
                "{path} scrolls sideways at 320 px: {metrics:?}"
            );
            if page == "/tables" {
                table_boxes_read_whole(browser).await?;
                let row = browser
                    .element(By::XPath(
                        "(//table[not(ancestor::*[contains(@class, 'fui-worklist')])])[1]/tbody/tr[1]",
                    ))
                    .await?;
                assert_eq!("block", row.css_value("display").await?);
                // NOTE: design/COMPONENTS.md, The table, Phone; in a list card the row is the
                // work-list line: the name and the row menu first, then the facts.
                let line = browser
                    .element(By::XPath("(//table)[1]/tbody/tr[1]"))
                    .await?;
                assert_eq!("flex", line.css_value("display").await?);
                let named = browser
                    .element(By::XPath(
                        "(//table)[1]/tbody/tr[1]/td[@data-label = 'Lines']",
                    ))
                    .await?;
                assert!(named.is_displayed().await?, "a stacked cell is hidden");
                let wide = browser
                    .element(By::XPath(
                        "(//table)[1]/tbody/tr[1]/td[@data-role = 'wide']",
                    ))
                    .await?;
                assert!(!wide.is_displayed().await?, "a wide cell leaves the phone row");
                let title = browser
                    .element(By::XPath("(//table)[1]/tbody/tr[1]/th"))
                    .await?
                    .rect()
                    .await?;
                let menu = browser
                    .element(By::XPath("(//table)[1]/tbody/tr[1]/td[@data-role = 'menu']"))
                    .await?
                    .rect()
                    .await?;
                assert!(
                    menu.y < title.y + title.height,
                    "the row menu sits on the row's first line"
                );
                let check = browser
                    .element(By::XPath("(//table)[3]/tbody/tr[2]/td[@data-role = 'check']"))
                    .await?;
                assert!(check.is_displayed().await?, "a phone row keeps its checkbox");
                let checked = check.rect().await?;
                let named_row = browser
                    .element(By::XPath("(//table)[3]/tbody/tr[2]/th"))
                    .await?
                    .rect()
                    .await?;
                assert!(
                    checked.y < named_row.y + named_row.height && checked.x < named_row.x,
                    "the checkbox leads the row's first line"
                );
                // NOTE: design/COMPONENTS.md, The table, Phone; a table of several bodies stacks
                // each body's rows as hairline-separated lines, the bodies joined as its rows are.
                let paged = browser
                    .element(By::Css("tbody[data-page='2'] > tr"))
                    .await?;
                assert_eq!("block", paged.css_value("display").await?);
                assert_eq!("1px", paged.css_value("border-top-width").await?);
                let second = browser.element(By::Css("tbody[data-page='2']")).await?;
                assert_eq!("0px", second.css_value("margin-top").await?);
            }
        }
        browser.console_clean("the reflow").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_control_tabbed_to_above_the_fold_lands_below_the_sticky_top_bar()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroehr/forms")).await?;
        browser.element(By::Css("main h1")).await?;
        // NOTE: Leptos 0.8 `#[island(lazy)]` hydrates after the page loads, and a
        // masked identifier swaps its fallback link for its button then.
        until("the identifiers have hydrated", async || {
            let pending = browser
                .driver()
                .execute(
                    "return document.querySelectorAll('.fui-identifier-press[hidden]').length;",
                    vec![],
                )
                .await?;
            Ok(*pending.json() == json!(0))
        })
        .await?;
        // From the foot of the page, the focus goes to the control after one
        // well above the fold without scrolling; Shift+Tab then moves to
        // that control, which the browser scrolls into view at the top.
        let marked = browser
            .driver()
            .execute(
                r"window.scrollTo(0, document.documentElement.scrollHeight);
                const all = [...document.querySelectorAll(
                  'main a[href], main button:not([disabled]), main input:not([disabled]):not([type=hidden]), main select:not([disabled]), main textarea:not([disabled])'
                )].filter((e) => e.offsetParent !== null && e.tabIndex >= 0);
                const above = all.filter((e) => e.getBoundingClientRect().bottom < -200);
                if (above.length === 0) { return false; }
                const target = above[above.length - 1];
                const after = all[all.indexOf(target) + 1];
                target.setAttribute('data-journey', 'target');
                after.focus({ preventScroll: true });
                return document.activeElement === after;",
                vec![],
            )
            .await?;
        assert_eq!(
            serde_json::Value::Bool(true),
            *marked.json(),
            "no control above the fold"
        );
        browser
            .driver()
            .active_element()
            .await?
            .send_keys(Key::Shift + Key::Tab)
            .await?;
        let landed = browser
            .driver()
            .execute(
                r"const target = document.activeElement;
                const bar = document.querySelector('.fui-topbar').getBoundingClientRect();
                return [target.getAttribute('data-journey'), target.getBoundingClientRect().top, bar.bottom];",
                vec![],
            )
            .await?;
        let landed: (Option<String>, f64, f64) = serde_json::from_value(landed.json().clone())?;
        assert_eq!(Some(String::from("target")), landed.0, "Shift+Tab missed the control");
        assert!(
            landed.1 >= landed.2,
            "the focused control's top ({}) is under the top bar's foot ({})",
            landed.1,
            landed.2
        );
        browser.console_clean("the scroll padding").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_top_bar_status_keeps_its_shape_and_shows_its_words_from_64_rem()
-> Result<(), Box<dyn Error>> {
    // F12: below 64 rem the pill keeps its shape, its words its accessible name.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for width in [390, 640, 768, 900, 1024, 1100, 1280, 1440] {
            window(browser, width, 900).await?;
            browser.goto(&gallery.url("/ferroehr/forms")).await?;
            browser.element(By::Css("main h1")).await?;
            let measured = browser
                .driver()
                .execute(
                    r"const pill = document.querySelector('.fui-topbar-end .fui-topbar-status');
                    const words = pill.querySelector('.fui-roomy-text');
                    const end = document.querySelector('.fui-topbar-end').getBoundingClientRect();
                    const middle = document.querySelector('.fui-topbar-middle');
                    const box = pill.getBoundingClientRect();
                    const search = middle.getBoundingClientRect();
                    const clear = middle.offsetTop !== pill.offsetTop
                      ? true
                      : box.left >= search.right;
                    const read = words.getBoundingClientRect();
                    return [pill.getClientRects().length > 0, read.width > 1,
                      pill.scrollWidth <= pill.clientWidth + 1, end.right <= document.documentElement.clientWidth + 0.5,
                      clear, words.textContent];",
                    vec![],
                )
                .await?;
            let (shown, worded, whole, inside, clear, name): (bool, bool, bool, bool, bool, String) =
                serde_json::from_value(measured.json().clone())?;
            assert!(shown, "the status pill keeps its shape at {width} px");
            assert_eq!("Library up", name, "the words stay its name at {width} px");
            assert!(whole, "the status pill is cut short at {width} px");
            assert!(inside, "the top bar's end runs past the window at {width} px");
            assert!(clear, "the status pill overlaps the search box at {width} px");
            assert_eq!(
                width >= 1024,
                worded,
                "the words show from 64 rem up alone, at {width} px"
            );
        }
        browser.console_clean("the top bar status").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_top_bar_never_covers_the_search_at_any_width() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for width in [390, 480, 640, 700, 768, 900, 1024, 1100, 1280, 1440, 1920] {
            window(browser, width, 900).await?;
            browser.goto(&gallery.url("/ferroterm/forms")).await?;
            browser.element(By::Css("main h1")).await?;
            let (covered, outside, sideways, name, title, searched, shared): (
                bool,
                bool,
                bool,
                String,
                String,
                f64,
                bool,
            ) = measure(
                browser,
                r"const search = document.querySelector('.fui-topbar .fui-search').getBoundingClientRect();
                const overlaps = (a, b) => a.left < b.right - 0.5 && b.left < a.right - 0.5
                  && a.top < b.bottom - 0.5 && b.top < a.bottom - 0.5;
                const parts = Array.from(document.querySelectorAll('.fui-topbar-end > *, .fui-topbar-row > .fui-lockup'))
                  .filter((el) => el.getClientRects().length > 0)
                  .map((el) => el.getBoundingClientRect());
                const end = document.querySelector('.fui-topbar-end').getBoundingClientRect();
                const operator = document.querySelector('.fui-topbar .fui-operator');
                const label = operator.querySelector('.fui-operator-name');
                return [parts.some((part) => overlaps(part, search)),
                  end.right > document.documentElement.clientWidth + 0.5,
                  document.documentElement.scrollWidth > document.documentElement.clientWidth + 0.5,
                  label.textContent, operator.getAttribute('title') || '',
                  search.width,
                  Math.abs(document.querySelector('.fui-topbar-row > .fui-lockup').getBoundingClientRect().top - end.top) < 12];",
            )
            .await?;
            assert!(!covered, "the top bar covers the search at {width} px");
            assert!(!outside, "the top bar's end runs past the window at {width} px");
            if sideways {
                let wide: String = measure(
                    browser,
                    r"const w = document.documentElement.clientWidth;
                    return Array.from(document.querySelectorAll('body *'))
                      .filter((el) => el.getBoundingClientRect().right > w + 0.5)
                      .slice(0, 8)
                      .map((el) => el.tagName + '.' + el.className + ' ' + Math.round(el.getBoundingClientRect().right))
                      .join(' | ');",
                )
                .await?;
                panic!("the page scrolls sideways at {width} px: {wide}");
            }
            assert_eq!(
                title, name,
                "the operator's button carries the whole name as its title at {width} px"
            );
            // NOTE: no specification governs this: our own design; the search
            // keeps 11rem (176 px) on one row with the end, and its 28rem cap
            // at 1920 px, where every brand's end leaves room for it.
            assert!(shared, "the lockup and the end do not share a row at {width} px");
            if width >= 768 {
                assert!(searched >= 175.5, "the search is {searched} px wide at {width} px");
            }
            if width >= 1920 {
                assert!(searched >= 447.5, "the search gave way with room at {width} px");
            }
        }
        browser.console_clean("the top bar at every width").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_keyed_combobox_names_its_help_and_keeps_its_own_choice() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let second = browser
            .element(By::Css("#clause-2-system[role='combobox']"))
            .await?;
        let described = second.attr("aria-describedby").await?.unwrap_or_default();
        assert_eq!(
            described, "clause-2-system-help",
            "the input names its help line"
        );
        browser.element(By::Id("clause-2-system-help")).await?;
        second.click().await?;
        browser
            .element(By::Css(
                "#clause-2-system-list li[role='option']:nth-child(2)",
            ))
            .await?
            .click()
            .await?;
        let (first, chosen): (String, String) = measure(
            browser,
            r"return [document.querySelector('input[name=clause1]').value,
              document.querySelector('input[name=clause2]').value];",
        )
        .await?;
        assert!(
            first.is_empty(),
            "the first clause keeps its own empty choice, not `{first}`"
        );
        assert!(
            !chosen.is_empty(),
            "the second clause posts the option chosen in it"
        );
        browser.console_clean("the keyed combobox").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn an_editor_inside_a_reloading_transition_keeps_taking_keys() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let draft = browser.element(By::Id("reloading-draft")).await?;
        until("the editor hydrates", async || {
            browser.editor_live(&draft).await
        })
        .await?;
        draft.send_keys("x").await?;
        for round in 1..=2 {
            browser
                .element(By::Id("reload-draft"))
                .await?
                .click()
                .await?;
            // The old editor stays on screen while the read is pending, and a
            // key typed into it then reached a disposed signal.
            browser
                .element(By::Id("reloading-draft"))
                .await?
                .send_keys("y")
                .await?;
            browser
                .text("p[@id='reloaded']", &format!("Read {round} times again."))
                .await?;
        }
        let typed: String = measure(
            browser,
            "return document.getElementById('reloading-draft').value;",
        )
        .await?;
        assert!(
            typed.trim_end().ends_with("xyy"),
            "the editor kept `{typed}`"
        );
        browser.console_clean("the reloading editor").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn stat_cards_keep_their_glyphs_and_figures_in_line() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for width in [390, 1440] {
            window(browser, width, 900).await?;
            browser.goto(&gallery.url("/ferrofed")).await?;
            browser.element(By::Css(".fui-stat")).await?;
            let (glyphs, values, marked): (Vec<f64>, Vec<f64>, usize) = measure(
                browser,
                r"const cards = Array.from(document.querySelectorAll('.fui-grid-stats .fui-stat'));
                const offset = (card, part) => card.querySelector(part).getBoundingClientRect().top
                  - card.getBoundingClientRect().top;
                return [cards.map((card) => offset(card, '.fui-stat-glyph')),
                  cards.map((card) => offset(card, '.fui-stat-value')),
                  cards.filter((card) => card.querySelector('.fui-stat-label > :nth-child(2)')).length];",
            )
            .await?;
            assert_eq!(marked, 2, "the row holds two marked cards at {width} px");
            let spread = |offsets: &[f64]| {
                offsets.iter().copied().fold(f64::MIN, f64::max)
                    - offsets.iter().copied().fold(f64::MAX, f64::min)
            };
            assert!(spread(&glyphs) < 0.5, "the glyphs are out of line at {width} px: {glyphs:?}");
            assert!(spread(&values) < 0.5, "the figures are out of line at {width} px: {values:?}");
        }
        browser.console_clean("the stat cards").await
    })
    .await
}

/// Runs `script` in the page and reads its answer as `T`.
async fn measure<T: serde::de::DeserializeOwned>(
    browser: &Browser,
    script: &str,
) -> Result<T, Box<dyn Error>> {
    let answer = browser.driver().execute(script, vec![]).await?;
    Ok(serde_json::from_value(answer.json().clone())?)
}

#[tokio::test(flavor = "multi_thread")]
async fn at_320_pixels_a_long_pill_in_a_stacked_card_wraps_inside_the_card()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 320, 800).await?;
        browser.goto(&gallery.url("/ferrofed/tables")).await?;
        browser.element(By::Css("main h1")).await?;
        // The script below rewrites a cell, so it waits for the page to hydrate first.
        browser.hydrated().await?;
        // NOTE: WCAG 2.2 SC 1.4.10; a state longer than the value column stays inside the card.
        let [pill, card, edge]: [f64; 3] = browser
            .driver()
            .execute(
                "const table = [...document.querySelectorAll('.fui-table-stack .fui-table')]
                   .find(each => !each.closest('.fui-worklist'));
                 const cell = table.querySelector('tbody tr td[data-label]');
                 cell.innerHTML = '<span class=\"fui-pill fui-tone-warn\">saved, not applied to the running configuration</span>';
                 const pill = cell.querySelector('.fui-pill').getBoundingClientRect();
                 const card = cell.closest('tr').getBoundingClientRect();
                 return [pill.right, card.right, document.documentElement.clientWidth];",
                Vec::new(),
            )
            .await?
            .convert()?;
        assert!(pill <= card + 0.5, "the pill ends at {pill}, past its card at {card}");
        assert!(card <= edge + 0.5, "the card ends at {card}, past the page at {edge}");
        let wider: bool = browser
            .driver()
            .execute(
                "const root = document.documentElement; return root.scrollWidth > root.clientWidth + 1;",
                Vec::new(),
            )
            .await?
            .convert()?;
        assert!(!wider, "the page scrolls sideways");
        browser.console_clean("a long pill at 320 px").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_long_button_label_wraps_at_320_pixels_and_a_short_one_keeps_its_line()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 320, 800).await?;
        browser.goto(&gallery.url("/ferrofed/forms")).await?;
        browser.element(By::Id("long-label")).await?;
        let (right, inner, height, short, page, view): (f64, f64, f64, f64, f64, f64) = measure(
            browser,
            r"const long = document.getElementById('long-label').getBoundingClientRect();
            const short = document.getElementById('long-label').nextElementSibling.getBoundingClientRect();
            return [long.right, document.documentElement.clientWidth, long.height, short.height,
              document.documentElement.scrollWidth, document.documentElement.clientWidth];",
        )
        .await?;
        assert!(right <= inner + 0.5, "the long button runs past the window ({right} > {inner})");
        assert!(page <= view + 0.5, "the page scrolls sideways ({page} > {view})");
        assert!(height > short + 4.0, "the long label did not wrap ({height} vs {short})");
        browser.console_clean("the long label").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_pick_or_type_field_shows_one_control_and_its_own_label_at_a_time()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        // The combobox replaces its select once the page hydrates, which is
        // when the select's change listener is there too.
        browser
            .element(By::Css("#pick-canonical-choice[role='combobox']"))
            .await?;
        let shown = async |css: &str| -> Result<bool, Box<dyn Error>> {
            Ok(browser.element(By::Css(css)).await?.is_displayed().await?)
        };
        assert!(shown("#pick-version-choice").await?, "the list shows first");
        assert!(shown("label[for='pick-version-choice']").await?);
        assert!(!shown("#pick-version").await?);
        assert!(
            !shown("label[for='pick-version']").await?,
            "a hidden control's label hides with it"
        );
        browser
            .element(By::Css("#pick-version-choice option[value='another']"))
            .await?
            .click()
            .await?;
        until("the typed input shows", async || {
            shown("#pick-version").await
        })
        .await?;
        assert!(shown("label[for='pick-version']").await?);
        assert!(!shown("#pick-version-choice").await?);
        assert!(!shown("label[for='pick-version-choice']").await?);
        let described = browser
            .element(By::Id("pick-version"))
            .await?
            .attr("aria-describedby")
            .await?;
        assert_eq!(described.as_deref(), Some("pick-version-help"));
        browser
            .element(By::Id("pick-version-back"))
            .await?
            .click()
            .await?;
        until("the list shows again", async || {
            shown("#pick-version-choice").await
        })
        .await?;
        assert!(!shown("label[for='pick-version']").await?);
        // The combobox labels itself, and its last option opens the input.
        let combo = browser
            .element(By::Css("#pick-canonical-choice[role='combobox']"))
            .await?;
        combo.click().await?;
        browser
            .element(By::Css(
                "#pick-canonical-choice-list li[role='option']:last-child",
            ))
            .await?
            .click()
            .await?;
        until("the typed canonical shows", async || {
            shown("#pick-canonical").await
        })
        .await?;
        assert!(!shown("label[for='pick-canonical-choice']").await?);
        browser.console_clean("the pick-or-type field").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_reveal_inside_a_reveal_answers_to_its_own_choice_alone() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        let shown = async |id: &str| -> Result<bool, Box<dyn Error>> {
            Ok(browser.element(By::Id(id)).await?.is_displayed().await?)
        };
        assert!(
            shown("grant-client-id").await?,
            "the client part opens chosen"
        );
        assert!(!shown("grant-token-value").await?);
        assert!(!shown("grant-secret-value").await?);
        // The inner choice shows its own part and leaves the outer one be.
        browser
            .element(By::Css("label[for='grant-secret-replace']"))
            .await?
            .click()
            .await?;
        until("the inner reveal shows its part", async || {
            shown("grant-secret-value").await
        })
        .await?;
        assert!(
            shown("grant-client-id").await?,
            "the inner choice hid the outer part"
        );
        assert!(
            !shown("grant-token-value").await?,
            "the inner choice showed the outer part"
        );
        // The outer choice swaps the outer parts alone.
        browser
            .element(By::Css("label[for='grant-token']"))
            .await?
            .click()
            .await?;
        until("the outer reveal shows its second part", async || {
            shown("grant-token-value").await
        })
        .await?;
        assert!(!shown("grant-client-id").await?);
        browser.console_clean("the nested reveal").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_form_in_the_actions_cell_sits_on_the_row_beside_its_link() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroehr/tables")).await?;
        browser.element(By::Css("a[href='#edit']")).await?;
        let (link, button): (f64, f64) = measure(
            browser,
            r"const link = document.querySelector('a[href=\'#edit\']');
            const button = link.closest('td').querySelector('form button');
            return [link.getBoundingClientRect().top, button.getBoundingClientRect().top];",
        )
        .await?;
        assert!(
            (link - button).abs() <= 1.0,
            "Edit at {link}, Remove at {button}"
        );
        browser.console_clean("the row actions").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn cards_in_a_start_grid_keep_their_own_heights() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 900).await?;
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        browser.element(By::Css(".fui-grid-start")).await?;
        let heights: Vec<f64> = measure(
            browser,
            r"return [...document.querySelector('.fui-grid-start').children]
              .map((card) => card.getBoundingClientRect().height);",
        )
        .await?;
        let [short, tall] = heights.as_slice() else {
            return Err(format!("two cards expected: {heights:?}").into());
        };
        assert!(tall - short > 20.0, "the short card stretched: {heights:?}");
        browser.console_clean("the start grid").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn tabbing_through_a_long_form_never_leaves_the_focus_under_the_dock()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 1440, 700).await?;
        browser.goto(&gallery.url("/ferroehr/forms?dock=idle")).await?;
        let first = browser.element(By::Id("long-field-1")).await?;
        browser
            .driver()
            .execute(
                "arguments[0].scrollIntoView({block: 'start'}); window.scrollBy(0, -200); arguments[0].focus({preventScroll: true});",
                vec![first.to_json()?],
            )
            .await?;
        for step in 2..=14 {
            browser
                .driver()
                .active_element()
                .await?
                .send_keys(Key::Tab)
                .await?;
            let (id, bottom, bar): (String, f64, f64) = measure(
                browser,
                r"const focused = document.activeElement;
                const bar = document.querySelector('.fui-dock').getBoundingClientRect();
                return [focused.id, focused.getBoundingClientRect().bottom, bar.top];",
            )
            .await?;
            assert_eq!(format!("long-field-{step}"), id, "the focus left the form");
            assert!(
                bottom <= bar + 0.5,
                "{id} ends at {bottom}, under the dock from {bar}"
            );
        }
        browser.console_clean("the dock").await
    })
    .await
}

/// Sets the gallery's theme cookie to `theme` for every later page.
async fn theme(browser: &Browser, gallery: &Gallery, theme: &str) -> Result<(), Box<dyn Error>> {
    browser.goto(&gallery.url("/")).await?;
    let mut cookie = Cookie::new("gallery-theme", theme.to_owned());
    cookie.set_path("/");
    browser.driver().add_cookie(cookie).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_timestamp_keeps_one_line_in_tables_cards_and_facts_in_every_brand()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for width in [320, 390, 1440] {
            window(browser, width, 900).await?;
            for brand in Brand::ALL {
                for page in ["/tables", ""] {
                    let path = format!("/{}{page}", brand.key());
                    browser.goto(&gallery.url(&path)).await?;
                    browser.element(By::Css("time.fui-time")).await?;
                    // A value broken over lines draws one box per line; each
                    // must keep inside the card, row or definition it sits in.
                    // A wide column a phone row leaves out draws no box at all.
                    let (count, broken, outside, page, view): (usize, Vec<String>, Vec<String>, f64, f64) =
                        measure(
                            browser,
                            r"const times = Array.from(document.querySelectorAll('time.fui-time'))
                              .filter((time) => time.checkVisibility());
                            const broken = times.filter((time) => time.getClientRects().length !== 1)
                              .map((time) => time.textContent);
                            const outside = times.filter((time) => {
                              const box = time.closest('tr, .fui-facts, .fui-facts-row, .fui-card');
                              return time.getBoundingClientRect().right > box.getBoundingClientRect().right + 0.5;
                            }).map((time) => time.textContent);
                            return [times.length, broken, outside,
                              document.documentElement.scrollWidth, document.documentElement.clientWidth];",
                        )
                        .await?;
                    assert!(count >= 2, "{path} at {width} px draws {count} timestamps");
                    assert!(broken.is_empty(), "{path} at {width} px breaks {broken:?}");
                    assert!(outside.is_empty(), "{path} at {width} px runs {outside:?} past its box");
                    assert!(page <= view + 0.5, "{path} scrolls sideways at {width} px ({page} > {view})");
                }
            }
        }
        let time = browser.element(By::Css("time.fui-time")).await?;
        let title = time.attr("title").await?.unwrap_or_default();
        let datetime = time.attr("datetime").await?.unwrap_or_default();
        assert!(title.ends_with("+00:00") || title.ends_with("+02:00"), "the title is the raw value: {title}");
        assert!(datetime.ends_with('Z'), "the datetime is the instant in UTC: {datetime}");
        assert!(time.text().await?.ends_with(" UTC"), "the text names UTC");
        browser.console_clean("the timestamps").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stat_tile_below_30rem_is_one_compact_line() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for scheme in ["light", "dark"] {
            theme(browser, &gallery, scheme).await?;
            for width in [320, 390, 480] {
                window(browser, width, 900).await?;
                browser.goto(&gallery.url("/ferroterm")).await?;
                browser.element(By::Css(".fui-grid-stats .fui-stat")).await?;
                let tiles: Vec<(f64, bool, bool, bool, bool, bool, bool)> = measure(
                    browser,
                    r"const overlap = (a, b) => a.top < b.bottom && b.top < a.bottom;
                    return Array.from(document.querySelectorAll('.fui-grid-stats .fui-stat')).map((tile) => {
                      const box = (part) => tile.querySelector(part).getBoundingClientRect();
                      const label = tile.querySelector('.fui-stat-label > span:first-child');
                      const glyph = box('.fui-stat-glyph');
                      const value = box('.fui-stat-value');
                      const words = label.getBoundingClientRect();
                      return [tile.getBoundingClientRect().height,
                        overlap(glyph, value) && overlap(value, words),
                        words.top >= value.bottom - 0.5,
                        label.getClientRects().length === 1,
                        label.title === label.textContent,
                        tile.querySelector('.fui-stat-label > :nth-child(2)') !== null,
                        label.scrollWidth > label.clientWidth
                          && getComputedStyle(label).textOverflow === 'ellipsis'];
                    });",
                )
                .await?;
                assert_eq!(tiles.len(), 5, "five tiles at {width} px");
                assert_eq!(
                    tiles.iter().filter(|tile| tile.5).count(),
                    2,
                    "two tiles carry a mark at {width} px"
                );
                if width == 320 {
                    assert!(
                        tiles.iter().any(|tile| tile.6),
                        "no label ends in an ellipsis at 320 px {scheme}"
                    );
                }
                for (index, (height, one_line, stacked, single, titled, _, _)) in tiles.iter().enumerate() {
                    assert!(*titled, "tile {index} names its whole label in its title");
                    if width < 480 {
                        assert!(*height <= 64.5, "tile {index} is {height} px tall at {width} px {scheme}");
                        assert!(*one_line, "tile {index} is not one line at {width} px {scheme}");
                        assert!(*single, "tile {index} wraps its label at {width} px {scheme}");
                    } else {
                        assert!(*stacked, "tile {index} lost its desktop layout at {width} px {scheme}");
                    }
                }
                let (page, view): (f64, f64) = measure(
                    browser,
                    "return [document.documentElement.scrollWidth, document.documentElement.clientWidth];",
                )
                .await?;
                assert!(page <= view + 0.5, "the dashboard scrolls sideways at {width} px ({page} > {view})");
            }
        }
        browser.console_clean("the compact stat tiles").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_field_and_its_button_stack_below_30rem_and_share_a_line_above()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for width in [320, 390, 480, 1440] {
            window(browser, width, 900).await?;
            browser.goto(&gallery.url("/ferroterm/forms")).await?;
            browser.element(By::Id("member-code")).await?;
            let (label, input, button, row): (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) = measure(
                browser,
                r"const box = (e) => { const r = e.getBoundingClientRect(); return [r.top, r.bottom, r.left, r.width]; };
                const input = document.getElementById('member-code');
                const row = input.closest('.fui-field-flow');
                return [box(document.querySelector('label[for=member-code]')), box(input),
                  box(row.querySelector(':scope > button')), box(row)];",
            )
            .await?;
            assert!(label[1] <= input[0] + 0.5, "the label is not above the input at {width} px");
            if width < 480 {
                assert!(button[0] >= input[1] - 0.5, "the button is not below the input at {width} px");
                assert!((button[3] - row[3]).abs() <= 1.0, "the button is not the row's width at {width} px");
                assert!((input[3] - row[3]).abs() <= 1.0, "the input is not the row's width at {width} px");
            } else if width == 1440 {
                // At 480 px the row's select, input and button wrap as the
                // flex row has room; only the wide row shares one line.
                assert!(
                    (button[1] - input[1]).abs() <= 1.0,
                    "the button left the input's line at {width} px"
                );
            }
        }
        browser.console_clean("the field and its button").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_field_drawn_per_row_names_its_own_control_and_help() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        browser.element(By::Id("compose-search-exclude-2")).await?;
        // HTML Living Standard, the label element: `control` is the element
        // its `for` names, so each label of a row reaches that row's input.
        let named: Vec<(String, String, bool)> = measure(
            browser,
            r"return [...document.querySelectorAll('label[for^=compose-]')].map((label) => {
                const control = label.control;
                const help = control && control.getAttribute('aria-describedby');
                return [label.htmlFor, control ? control.id : '', !!(help && document.getElementById(help))];
              });",
        )
        .await?;
        assert_eq!(4, named.len(), "two rows of a search and a switch: {named:?}");
        for (target, control, described) in &named {
            assert_eq!(target, control, "the label names its own row's control");
            assert!(*described, "{control} names a help line that is on the page");
        }
        browser.console_clean("the fields per row").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_result_header_carries_its_path_as_its_title_and_no_cell_repeats_it()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed/tables")).await?;
        let (titles, cells): (Vec<String>, usize) = measure(
            browser,
            r"const table = [...document.querySelectorAll('table')].find((t) =>
                t.caption && t.caption.textContent.startsWith('The rows the synthetic query'));
              return [[...table.querySelectorAll('thead th')].map((th) => th.title),
                table.querySelectorAll('tbody [title]').length];",
        )
        .await?;
        assert_eq!(
            vec!["c/uid/value", "c/name/value", "count(*)"],
            titles,
            "each header is titled with the path it reads"
        );
        assert_eq!(0, cells, "no cell repeats a header's title");
        browser.console_clean("the titled result headers").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_button_beside_a_field_with_help_stays_on_the_controls_line() -> Result<(), Box<dyn Error>>
{
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for width in [480, 768, 1440] {
            window(browser, width, 900).await?;
            browser.goto(&gallery.url("/ferroterm/forms")).await?;
            browser.element(By::Id("compose-search-include-1")).await?;
            let (input, button): (Vec<f64>, Vec<f64>) = measure(
                browser,
                r"const box = (e) => { const r = e.getBoundingClientRect(); return [r.top, r.bottom]; };
                const input = document.getElementById('compose-search-include-1');
                return [box(input), box(input.closest('.fui-field-flow').querySelector(':scope > button'))];",
            )
            .await?;
            assert!(
                (button[0] - input[0]).abs() <= 1.0 && (button[1] - input[1]).abs() <= 1.0,
                "the button {button:?} left the input's line {input:?} at {width} px"
            );
        }
        browser.console_clean("the button beside a field with help").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_user_menus_theme_segments_keep_their_words_on_one_line() -> Result<(), Box<dyn Error>>
{
    // WCAG 2.2 SC 1.4.12: a segment's word never breaks; the menu grows to the row instead.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/")).await?;
        for (width, height) in [(1440, 900), (390, 844)] {
            window(browser, width, height).await?;
            for theme in ["light", "dark", "system"] {
                browser.driver().delete_all_cookies().await?;
                for (name, value) in [("gallery-theme", theme), ("gallery-language", "qps")] {
                    let mut cookie = Cookie::new(name, value);
                    cookie.set_path("/");
                    browser.driver().add_cookie(cookie).await?;
                }
                browser.goto(&gallery.url("/ferrobridge")).await?;
                browser
                    .element(By::Css("button[popovertarget='user-menu']"))
                    .await?
                    .click()
                    .await?;
                let menu = browser.element(By::Id("user-menu")).await?;
                until("the menu opens", async || Ok(menu.is_displayed().await?)).await?;
                let lines: Vec<u32> = serde_json::from_value(
                    browser
                        .driver()
                        .execute(
                            r"return [...document.querySelectorAll('#user-menu .fui-menu-theme .fui-segment')].map(segment => {
                                const walk = document.createTreeWalker(segment, NodeFilter.SHOW_TEXT);
                                const tops = new Set();
                                while (walk.nextNode()) {
                                    const range = document.createRange();
                                    range.selectNodeContents(walk.currentNode);
                                    for (const rect of range.getClientRects()) {
                                        if (rect.width > 0) tops.add(Math.round(rect.top));
                                    }
                                }
                                return tops.size;
                            });",
                            vec![],
                        )
                        .await?
                        .json()
                        .clone(),
                )?;
                assert_eq!(
                    vec![1, 1, 1],
                    lines,
                    "each theme segment holds one line at {width} px with {theme} chosen"
                );
            }
        }
        browser.console_clean("the theme segments").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn each_rows_remove_opens_its_own_confirmation() -> Result<(), Box<dyn Error>> {
    // HTML Living Standard, the `command` attribute: a trigger opens the dialog its id names.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed/overlays")).await?;
        let menu = crate::menus::hydrated_trigger(browser, "member-menu-node-c").await?;
        browser
            .driver()
            .execute(
                "arguments[0].scrollIntoView({block: 'center'});",
                vec![menu.to_json()?],
            )
            .await?;
        menu.click().await?;
        let second = browser
            .element(By::Css(
                "#member-removals button[commandfor='remove-node-c']",
            ))
            .await?;
        until("the row menu opens", async || {
            Ok(second.is_displayed().await?)
        })
        .await?;
        second.click().await?;
        let opened = browser.element(By::Id("remove-node-c")).await?;
        until("the second row's dialog opens", async || {
            Ok(opened.attr("open").await?.is_some())
        })
        .await?;
        let first = browser.element(By::Id("remove-node-b")).await?;
        assert!(
            first.attr("open").await?.is_none(),
            "the first row's dialog stays closed"
        );
        browser.console_clean("a row's own confirmation").await
    })
    .await
}
