// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of the forms and controls at their edges: a long segment
//! label, the upload's focus, the combobox cleared, the picker under the
//! arrow keys and the copy button on a page with no clipboard.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// Runs the script `script` over `args` and reads its answer as `T`.
async fn script<T: serde::de::DeserializeOwned>(
    browser: &Browser,
    script: &str,
    args: Vec<serde_json::Value>,
) -> Result<T, Box<dyn Error>> {
    let answer = browser.driver().execute(script, args).await?;
    Ok(serde_json::from_value(answer.json().clone())?)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_long_segment_label_wraps_inside_its_group() -> Result<(), Box<dyn Error>> {
    // WCAG 2.2 SC 1.4.10: at 320 px a segment's words wrap, never clipped by the group's edge.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/")).await?;
        let mut cookie = Cookie::new("gallery-language", "qps");
        cookie.set_path("/");
        browser.driver().add_cookie(cookie).await?;
        browser.driver().set_window_rect(0, 0, 320, 900).await?;
        browser.goto(&gallery.url("/ferrobridge")).await?;
        browser.element(By::Css(r#"html[lang="qps"]"#)).await?;
        let clipped: Vec<String> = script(
            browser,
            "return [...document.querySelectorAll('.fui-segmented > .fui-segment')]
               .filter(segment => segment.getClientRects().length > 0)
               .filter(segment => {
                 const group = segment.parentElement.getBoundingClientRect();
                 const own = segment.getBoundingClientRect();
                 return own.right > group.right + 1 || own.left < group.left - 2
                   || segment.scrollWidth > segment.clientWidth + 1;
               })
               .map(segment => segment.textContent.trim());",
            Vec::new(),
        )
        .await?;
        assert!(clipped.is_empty(), "segments past their group: {clipped:?}");
        browser.console_clean("the segmented controls").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_upload_shows_its_focus_on_the_chooser() -> Result<(), Box<dyn Error>> {
    // WCAG 2.2 SC 2.4.7: the file input is visually hidden, so the chooser shows its focus.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        let input = browser.element(By::Id("gallery-upload-file")).await?;
        let body = browser.element(By::Css("body")).await?;
        body.send_keys(Key::Shift).await?;
        script::<serde_json::Value>(browser, "arguments[0].focus();", vec![input.to_json()?])
            .await?;
        let outline: String = script(
            browser,
            "return getComputedStyle(document.querySelector(\"label[for='gallery-upload-file']\")).outlineStyle;",
            Vec::new(),
        )
        .await?;
        assert_eq!("solid", outline, "the chooser draws the focus ring");
        browser.console_clean("the upload's focus").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn clearing_the_combobox_clears_its_value() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        let input = browser
            .element(By::Css("#choose-system[role='combobox']"))
            .await?;
        let posted = browser
            .element(By::Css("input[type='hidden'][name='system']"))
            .await?;
        assert_eq!(Some(String::from("snomed")), posted.prop("value").await?);
        input.click().await?;
        script::<serde_json::Value>(
            browser,
            "arguments[0].value = ''; arguments[0].dispatchEvent(new Event('input', {bubbles: true}));",
            vec![input.to_json()?],
        )
        .await?;
        input.send_keys(Key::Escape).await?;
        until("clearing the words clears the value", async || {
            Ok(posted.prop("value").await?.as_deref() == Some(""))
        })
        .await?;
        assert_eq!(Some(String::new()), input.prop("value").await?);
        browser.console_clean("the cleared combobox").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn arrowing_through_the_picker_keeps_the_focus_on_its_select() -> Result<(), Box<dyn Error>> {
    // WCAG 2.2 SC 3.2.2: a closed select that changes per arrow key must not move the focus.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/layouts")).await?;
        let choice = browser.element(By::Id("run-value-set-choice")).await?;
        let input = browser.element(By::Id("run-value-set")).await?;
        until("the picker hydrates and hides its input", async || {
            Ok(!input.is_displayed().await?)
        })
        .await?;
        let count: usize = script(
            browser,
            "return arguments[0].options.length;",
            vec![choice.to_json()?],
        )
        .await?;
        script::<serde_json::Value>(browser, "arguments[0].focus();", vec![choice.to_json()?])
            .await?;
        // NOTE: HTML Living Standard, the select element: a closed select may change, and
        // fire `change`, at each arrow key, as Chrome does on Windows and Linux.
        for _ in 1..count {
            script::<serde_json::Value>(
                browser,
                "const select = arguments[0];
                 select.dispatchEvent(new KeyboardEvent('keydown', {key: 'ArrowDown', bubbles: true}));
                 select.selectedIndex = Math.min(select.selectedIndex + 1, select.options.length - 1);
                 select.dispatchEvent(new Event('input', {bubbles: true}));
                 select.dispatchEvent(new Event('change', {bubbles: true}));
                 select.dispatchEvent(new KeyboardEvent('keyup', {key: 'ArrowDown', bubbles: true}));",
                vec![choice.to_json()?],
            )
            .await?;
        }
        until("the arrow onto another shows the input", async || {
            Ok(input.is_displayed().await?)
        })
        .await?;
        let last: i64 = script(
            browser,
            "return arguments[0].selectedIndex;",
            vec![choice.to_json()?],
        )
        .await?;
        assert_eq!(
            i64::try_from(count)? - 1,
            last,
            "the arrows walk the closed select to its last option"
        );
        let focused: String =
            script(
                browser,
                "return new Promise(done => requestAnimationFrame(() => requestAnimationFrame(() => done(document.activeElement.id))));",
                Vec::new(),
            )
            .await?;
        assert_eq!(
            "run-value-set-choice", focused,
            "the focus stays on the select (selected index {last} of {count})"
        );
        browser.console_clean("the picker's arrows").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_copy_button_says_refused_where_the_page_has_no_clipboard() -> Result<(), Box<dyn Error>>
{
    // W3C Clipboard API: `navigator.clipboard` is undefined outside a secure context.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser
            .goto(&gallery.url("/ferrobridge/operations"))
            .await?;
        let button = browser.element(By::Id("pair-answer-copy")).await?;
        until("the copy button hydrates", async || {
            Ok(button.is_displayed().await?)
        })
        .await?;
        script::<serde_json::Value>(browser, "delete Navigator.prototype.clipboard;", Vec::new())
            .await?;
        button.click().await?;
        browser.element(By::Css(".fui-copy-refused")).await?;
        browser.console_clean("the copy with no clipboard").await
    })
    .await
}

/// The masked identifier's input and its eye, once the eye works.
async fn masked(browser: &Browser) -> Result<(WebElement, WebElement), Box<dyn Error>> {
    let input = browser.element(By::Id("edge-identifier")).await?;
    let eye = browser
        .element(By::Css("#edge-identifier + leptos-island > button"))
        .await?;
    until("the eye works", async || {
        eye.is_enabled().await.map_err(Into::into)
    })
    .await?;
    Ok((input, eye))
}

/// Waits until the input shows its text when `shown`, or masks it, with
/// the eye's pressed state in step.
async fn shows(input: &WebElement, eye: &WebElement, shown: bool) -> Result<(), Box<dyn Error>> {
    let (kind, pressed) = if shown {
        ("text", "true")
    } else {
        ("password", "false")
    };
    until(&format!("the input is {kind}"), async || {
        Ok(input.attr("type").await?.as_deref() == Some(kind)
            && eye.attr("aria-pressed").await?.as_deref() == Some(pressed))
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_masked_identifier_shows_while_the_eye_is_held_by_the_pointer()
-> Result<(), Box<dyn Error>> {
    // WAI-ARIA 1.2 `button`, `aria-pressed`: the eye says it is pressed while it shows the text.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        let (input, eye) = masked(browser).await?;
        input.send_keys("123456782").await?;
        assert_eq!(
            Some(String::from("Show the identifier while held")),
            eye.attr("aria-label").await?,
            "the eye names what it shows"
        );
        shows(&input, &eye, false).await?;
        browser
            .driver()
            .action_chain()
            .move_to_element_center(&eye)
            .click_and_hold()
            .perform()
            .await?;
        shows(&input, &eye, true).await?;
        browser.driver().action_chain().release().perform().await?;
        shows(&input, &eye, false).await?;
        assert_eq!(
            Some(String::from("123456782")),
            input.prop("value").await?,
            "holding never changes what was typed"
        );
        browser.console_clean("the eye held by the pointer").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_masked_identifier_shows_while_space_or_enter_is_held_on_the_eye()
-> Result<(), Box<dyn Error>> {
    // WAI-ARIA Authoring Practices, button pattern: Space and Enter press the button.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        let (input, eye) = masked(browser).await?;
        input.send_keys("123456782").await?;
        for key in [Key::Space, Key::Enter] {
            browser
                .driver()
                .execute("arguments[0].focus();", vec![eye.to_json()?])
                .await?;
            browser
                .driver()
                .action_chain()
                .key_down(key.clone())
                .perform()
                .await?;
            shows(&input, &eye, true).await?;
            browser
                .driver()
                .action_chain()
                .key_up(key)
                .perform()
                .await?;
            shows(&input, &eye, false).await?;
        }
        browser
            .driver()
            .action_chain()
            .key_down(Key::Space)
            .perform()
            .await?;
        shows(&input, &eye, true).await?;
        browser
            .driver()
            .execute("arguments[0].focus();", vec![input.to_json()?])
            .await?;
        shows(&input, &eye, false).await?;
        browser
            .driver()
            .action_chain()
            .key_up(Key::Space)
            .perform()
            .await?;
        browser.console_clean("the eye held by the keyboard").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_choice_legend_is_drawn_as_a_field_label() -> Result<(), Box<dyn Error>> {
    // A choice's legend names one field, so it takes the field label's style, not a group heading's.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        browser.element(By::Css("fieldset.fui-choice > legend")).await?;
        let styles: Vec<String> = script(
            browser,
            "const legend = getComputedStyle(document.querySelector('fieldset.fui-choice > legend'));
             const label = getComputedStyle(document.querySelector('.fui-field > label.fui-label'));
             return [legend.fontSize, label.fontSize, legend.fontWeight, label.fontWeight, legend.borderBottomStyle];",
            Vec::new(),
        )
        .await?;
        assert_eq!(styles[0], styles[1], "the legend's size is the label's: {styles:?}");
        assert_eq!(styles[2], styles[3], "the legend's weight is the label's: {styles:?}");
        assert_eq!("none", styles[4], "no rule under the legend: {styles:?}");
        browser.console_clean("the choice legend").await
    })
    .await
}

/// How many rows the list `id` holds, its nested lists' rows left out.
async fn rows_of(browser: &Browser, id: &str) -> Result<u32, Box<dyn Error>> {
    script(
        browser,
        "const id = arguments[0];
         const rows = [...document.querySelectorAll(`#${id} [data-repeat='rows']`)]
           .find(each => each.closest('.fui-repeat').id === id);
         return rows.querySelectorAll(':scope > .fui-repeat-item').length;",
        vec![serde_json::Value::from(id)],
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_list_inside_a_sub_card_adds_and_removes_its_own_rows_alone() -> Result<(), Box<dyn Error>>
{
    // A press inside a nested list is that list's: the sub-card list around it keeps its rows.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferrofed/forms")).await?;
        let add = browser
            .element(By::Css("#ep-e2-profiles button[data-repeat='add']"))
            .await?;
        // The page's islands hydrate from one entry, so the eye working
        // means the lists work too.
        masked(browser).await?;
        assert_eq!(2, rows_of(browser, "member-endpoints").await?);
        assert_eq!(2, rows_of(browser, "ep-e2-profiles").await?);
        script::<serde_json::Value>(
            browser,
            "arguments[0].scrollIntoView({block: 'center'});",
            vec![add.to_json()?],
        )
        .await?;
        add.click().await?;
        until("a profile row is added", async || {
            Ok(rows_of(browser, "ep-e2-profiles").await? == 3)
        })
        .await?;
        assert_eq!(
            2,
            rows_of(browser, "member-endpoints").await?,
            "adding a profile adds no endpoint"
        );
        browser
            .element(By::Css(
                "#ep-e2-profiles [data-row='p1'] button[data-repeat='remove']",
            ))
            .await?
            .click()
            .await?;
        until("the profile row is removed", async || {
            Ok(rows_of(browser, "ep-e2-profiles").await? == 2)
        })
        .await?;
        assert_eq!(
            2,
            rows_of(browser, "member-endpoints").await?,
            "removing a profile removes no endpoint"
        );
        let focused: String = script(
            browser,
            "return document.activeElement.closest('.fui-repeat').id;",
            Vec::new(),
        )
        .await?;
        assert_eq!("ep-e2-profiles", focused, "the focus stays in the profiles");
        browser.console_clean("a nested list").await
    })
    .await
}
