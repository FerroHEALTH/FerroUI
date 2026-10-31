// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The journeys of a form answered in place (#546): the forms page's
//! archetype id check, posted with `fetch` as you type and on Check, its
//! answer moved into the region the page drew, with the focus kept, an
//! older answer dropped, and the plain post of the whole page when the
//! modules are refused or the answer is no page. No specification governs
//! the module: our own design, held to WCAG 2.2 SC 1.4.10 and SC 4.1.3.

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys::window;
use crate::journeys_enabled;
use crate::no_script::{leave, withhold_the_modules};
use crate::stack::Gallery;

/// A well-formed archetype id and what the page answers for it.
const WELL_FORMED: &str = "openEHR-EHR-OBSERVATION.blood_pressure.v2";
const WELL_FORMED_SAID: &str = "Well formed: the OBSERVATION archetype blood_pressure, version 2.";

/// What the page answers for an id that is none.
const REFUSED_SAID: &str = "Not an archetype id: write openEHR-EHR-CLASS.concept.vN, such as openEHR-EHR-OBSERVATION.blood_pressure.v2.";

/// Counts the page's requests in `window.fuiAsked`, holding the first one's
/// answer back `late_ms` milliseconds when it is above 0 and marking
/// `window.fuiLateDone` once the module has had it; or answering every
/// request `500` when `refuse` holds.
const WATCH: &str = "const [late, refuse] = arguments; const plain = window.fetch; window.fuiAsked = 0; window.fuiLateDone = false; window.fetch = (...args) => { const n = ++window.fuiAsked; if (refuse) return Promise.resolve(new Response('refused', { status: 500, headers: { 'Content-Type': 'text/plain' } })); const answer = plain(...args); if (n !== 1 || late <= 0) return answer; return answer.then((response) => new Promise((done) => setTimeout(() => { done(response); setTimeout(() => { window.fuiLateDone = true; }, 50); }, late))); };";

/// Loads the forms page and waits until the check form's module runs, with
/// the region held in `window.fuiRegion` to tell it is the same element
/// later.
async fn open(browser: &Browser, gallery: &Gallery) -> Result<(), Box<dyn Error>> {
    browser.goto(&gallery.url("/ferrobridge/forms")).await?;
    browser
        .element(By::Css("#check-form[data-fui-ready]"))
        .await?;
    browser
        .driver()
        .execute(
            "window.fuiRegion = document.getElementById('check-answer');",
            Vec::new(),
        )
        .await?;
    Ok(())
}

/// Wraps the page's `fetch` as [`WATCH`] says.
async fn watch(browser: &Browser, late_ms: u32, refuse: bool) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute(
            WATCH,
            vec![serde_json::json!(late_ms), serde_json::json!(refuse)],
        )
        .await?;
    Ok(())
}

/// Runs `script` and reads its answer as `T`.
async fn read<T: serde::de::DeserializeOwned>(
    browser: &Browser,
    script: &str,
) -> Result<T, Box<dyn Error>> {
    let answer = browser.driver().execute(script, Vec::new()).await?;
    Ok(serde_json::from_value(answer.json().clone())?)
}

/// The trimmed text of the answer region.
async fn said(browser: &Browser) -> Result<String, Box<dyn Error>> {
    read(
        browser,
        "return document.getElementById('check-answer').textContent.trim();",
    )
    .await
}

/// Waits until the answer region says `words`.
async fn until_said(browser: &Browser, words: &str) -> Result<(), Box<dyn Error>> {
    until(&format!("the answer says {words:?}"), async || {
        Ok(said(browser).await? == words)
    })
    .await
}

/// Asserts the answer arrived in the region the page drew, still a polite
/// live region, in the document the journey started on.
async fn in_place(browser: &Browser, loader: &str) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        loader,
        browser.loader().await?,
        "the answer came in place, with no new document"
    );
    let kept: bool = read(
        browser,
        "const region = document.getElementById('check-answer'); return region === window.fuiRegion && region.getAttribute('aria-live') === 'polite' && !region.hasAttribute('aria-busy');",
    )
    .await?;
    assert!(
        kept,
        "the region is the element the page drew, a polite live region at rest"
    );
    Ok(())
}

/// The id of the focused element.
async fn focused(browser: &Browser) -> Result<String, Box<dyn Error>> {
    read(browser, "return document.activeElement.id;").await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_check_answers_in_place_as_you_type_with_the_focus_kept_and_one_post_per_rest()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        open(browser, &gallery).await?;
        watch(browser, 0, false).await?;
        let loader = browser.loader().await?;
        let field = browser.element(By::Id("check-id")).await?;
        field.click().await?;
        field.send_keys(WELL_FORMED).await?;
        until_said(browser, WELL_FORMED_SAID).await?;
        in_place(browser, &loader).await?;
        assert_eq!("check-id", focused(browser).await?, "the focus stays in the field");
        assert_eq!(
            Some(String::from(WELL_FORMED)),
            field.prop("value").await?,
            "what you typed stays as you typed it"
        );
        let asked: u32 = read(browser, "return window.fuiAsked;").await?;
        let keys = u32::try_from(WELL_FORMED.len())?;
        assert!(
            asked >= 1 && asked.saturating_mul(4) < keys,
            "the form posts once the field rests, not per key: {asked} posts for {keys} keys"
        );
        browser
            .driver()
            .execute(
                "document.getElementById('check-id').value = '';",
                Vec::new(),
            )
            .await?;
        let check = browser
            .element(By::Css("#check-form button[type='submit']"))
            .await?;
        check.click().await?;
        until_said(browser, "The archetype id is empty.").await?;
        in_place(browser, &loader).await?;
        let on_check: bool = read(
            browser,
            "return document.activeElement === document.querySelector(\"#check-form button[type='submit']\");",
        )
        .await?;
        assert!(on_check, "Check keeps the focus it took");
        browser.console_clean("the check answered in place").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn an_answer_that_arrives_after_a_newer_one_is_dropped() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        open(browser, &gallery).await?;
        watch(browser, 1500, false).await?;
        let field = browser.element(By::Id("check-id")).await?;
        field.click().await?;
        field.send_keys("blood_pressure").await?;
        until("the first post has left", async || {
            Ok(read::<u32>(browser, "return window.fuiAsked;").await? == 1)
        })
        .await?;
        browser
            .driver()
            .execute(
                "document.getElementById('check-id').value = '';",
                Vec::new(),
            )
            .await?;
        field.send_keys(WELL_FORMED).await?;
        until_said(browser, WELL_FORMED_SAID).await?;
        until("the first post's answer has arrived", async || {
            read::<bool>(browser, "return window.fuiLateDone;").await
        })
        .await?;
        assert_eq!(
            WELL_FORMED_SAID,
            said(browser).await?,
            "the older answer, a refusal, never replaced the newer one"
        );
        let busy: bool = read(
            browser,
            "return document.getElementById('check-answer').hasAttribute('aria-busy');",
        )
        .await?;
        assert!(!busy, "the region is at rest");
        browser.console_clean("an older answer dropped").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn an_answer_that_is_no_page_posts_the_form_and_the_page_answers_whole()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        open(browser, &gallery).await?;
        watch(browser, 0, true).await?;
        browser
            .driver()
            .execute(
                "document.getElementById('check-id').value = arguments[0];",
                vec![serde_json::json!(WELL_FORMED)],
            )
            .await?;
        let check = browser
            .element(By::Css("#check-form button[type='submit']"))
            .await?;
        browser.click_and_load(&check).await?;
        browser.at(&gallery.url("/ferrobridge/forms")).await?;
        until_said(browser, WELL_FORMED_SAID).await?;
        assert_eq!(
            Some(String::from(WELL_FORMED)),
            browser
                .element(By::Id("check-id"))
                .await?
                .prop("value")
                .await?,
            "the page drew the posted id back into its field"
        );
        browser
            .console_clean("a refused answer posted the form")
            .await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_modules_refused_the_check_posts_and_the_page_answers_whole()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        withhold_the_modules(browser).await?;
        browser.goto(&gallery.url("/ferrobridge/forms")).await?;
        browser
            .element(By::Id("check-id"))
            .await?
            .send_keys("blood_pressure")
            .await?;
        let check = browser
            .element(By::Css("#check-form button[type='submit']"))
            .await?;
        browser.click_and_load(&check).await?;
        browser.at(&gallery.url("/ferrobridge/forms")).await?;
        until_said(browser, REFUSED_SAID).await?;
        assert_eq!(
            Some(String::from("blood_pressure")),
            browser
                .element(By::Id("check-id"))
                .await?
                .prop("value")
                .await?,
            "the posted id is kept in its field"
        );
        leave(browser, "the check with the modules refused").await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn at_320_pixels_the_check_and_its_longest_answer_reflow_with_no_sideways_scroll()
-> Result<(), Box<dyn Error>> {
    // WCAG 2.2 SC 1.4.10: the refusal, the longest answer, wraps inside the card at 320 px.
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        window(browser, 320, 800).await?;
        open(browser, &gallery).await?;
        let field = browser.element(By::Id("check-id")).await?;
        field.click().await?;
        field.send_keys("blood_pressure").await?;
        until_said(browser, REFUSED_SAID).await?;
        let fits: bool = read(
            browser,
            "const root = document.documentElement; const region = document.getElementById('check-answer').getBoundingClientRect(); const card = document.getElementById('check-form').closest('.fui-card').getBoundingClientRect(); return root.scrollWidth <= root.clientWidth && region.right <= card.right + 0.5 && card.right <= root.clientWidth + 0.5;",
        )
        .await?;
        assert!(
            fits,
            "the answer wraps inside its card and the page scrolls no sideways at 320 px"
        );
        browser.console_clean("the check at 320 px").await
    })
    .await
}
