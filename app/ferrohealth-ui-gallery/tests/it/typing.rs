// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Typing into a large document: the editor's work per keystroke follows
//! the change, not the length of the text (#458), the code editor's panel
//! keeps its line count and changed state by the edit (#477), and a
//! console that takes each edit keeps its own copy of the text in step
//! (#478).

use std::error::Error;

use thirtyfour::prelude::*;

use crate::browser::{Browser, until, with_browser};
use crate::journeys_enabled;
use crate::stack::Gallery;

/// How many keys each round types.
const KEYS: usize = 300;

/// What one round of typing measured in the page.
#[derive(Debug, serde::Deserialize)]
struct Round {
    /// The milliseconds each key took, from the dispatch of its input
    /// event until the editor and the console had taken it, before the
    /// browser draws.
    times: Vec<f64>,
    /// The keys after which the layer had not changed when the time was
    /// read.
    unseen: usize,
    /// The text area's text after the round.
    value: String,
    /// The text of each line of the layer after the round.
    lines: Vec<String>,
}

/// The synthetic YAML of the gallery, repeated to at least `bytes` bytes.
fn document(bytes: usize) -> String {
    let unit = ferrohealth_ui_gallery::data::YAML;
    unit.repeat(bytes.div_ceil(unit.len()).max(1))
}

/// Writes `text` into the editor whose text area is `id` as one input, then
/// types [`KEYS`] keys in the middle of it, one input event each, and reads
/// the time each took and the layer the editor drew.
async fn round(browser: &Browser, id: &str, text: &str) -> Result<Round, Box<dyn Error>> {
    let answer = browser
        .driver()
        .execute_async(
            r"const done = arguments[arguments.length - 1];
            const [id, text, keys] = [arguments[0], arguments[1], arguments[2]];
            const area = document.getElementById(id);
            const layer = area.previousElementSibling;
            let changes = 0;
            const observer = new MutationObserver((records) => {
              changes += records.length;
            });
            observer.observe(layer, { childList: true, subtree: true, attributes: true });
            // A task after every microtask, with no timer clamp, in which the
            // browser may draw.
            const settle = () => new Promise((resolve) => {
              const channel = new MessageChannel();
              channel.port1.onmessage = () => resolve();
              channel.port2.postMessage(0);
            });
            // The browser has changed the text by the time it dispatches the
            // input event, so the time runs from there.
            let heard = 0;
            const hear = () => {
              heard = performance.now();
            };
            window.addEventListener('input', hear, true);
            (async () => {
              area.focus();
              area.select();
              document.execCommand('insertText', false, text);
              await settle();
              observer.takeRecords();
              changes = 0;
              const middle = text.indexOf('\n', text.length >> 1) + 1;
              area.setSelectionRange(middle, middle);
              const times = [];
              let unseen = 0;
              for (let key = 0; key < keys; key += 1) {
                const typed = key % 50 === 49 ? '\n' : 'x';
                document.execCommand('insertText', false, typed);
                area.dispatchEvent(new KeyboardEvent('keyup', { key: typed, bubbles: true }));
                // The editor's effects run in microtasks after its handler and
                // before the browser draws, so this times the scripts alone.
                for (let turn = 0; turn < 16; turn += 1) {
                  await Promise.resolve();
                }
                times.push(performance.now() - heard);
                if (changes + observer.takeRecords().length === 0) {
                  unseen += 1;
                }
                changes = 0;
                await settle();
                observer.takeRecords();
                changes = 0;
              }
              observer.disconnect();
              window.removeEventListener('input', hear, true);
              const lines = Array.from(layer.children).map((line) => line.textContent);
              done({ times, unseen, value: area.value, lines });
            })();",
            vec![
                serde_json::to_value(id)?,
                serde_json::to_value(text)?,
                serde_json::to_value(KEYS)?,
            ],
        )
        .await?;
    Ok(serde_json::from_value(answer.json().clone())?)
}

/// The median of `times`.
fn median(times: &[f64]) -> f64 {
    let mut sorted = times.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted.get(sorted.len() >> 1).copied().unwrap_or_default()
}

/// Types a round into the editor `id` at 2 KB and at 64 KB, checks each
/// layer draws the text area's text, and returns the median time of a key
/// at each size and the text the last round left.
async fn rounds(browser: &Browser, id: &str) -> Result<([f64; 2], String), Box<dyn Error>> {
    let mut medians = [0.0; 2];
    let mut last = String::new();
    for (bytes, median_of) in [2 * 1024, 64 * 1024].into_iter().zip(&mut medians) {
        let text = document(bytes);
        let typed = round(browser, id, &text).await?;
        assert_eq!(KEYS, typed.times.len());
        assert_eq!(
            0, typed.unseen,
            "the layer of {id} took every key in the time read"
        );
        assert_eq!(
            text.len() + KEYS,
            typed.value.len(),
            "the text area {id} holds every key"
        );
        assert_eq!(
            typed.value,
            typed.lines.join("\n"),
            "the layer of {id} draws the text area's text, line by line"
        );
        *median_of = median(&typed.times);
        last = typed.value;
    }
    Ok((medians, last))
}

/// Writes `text` into the editor whose text area is `id` as one input.
async fn put(browser: &Browser, id: &str, text: &str) -> Result<(), Box<dyn Error>> {
    browser
        .driver()
        .execute(
            r"const area = document.getElementById(arguments[0]);
            area.focus();
            area.select();
            document.execCommand('insertText', false, arguments[1]);",
            vec![serde_json::to_value(id)?, serde_json::to_value(text)?],
        )
        .await?;
    Ok(())
}

/// Waits until the code editor panel around the text area `id` counts
/// the lines of `text` in its toolbar and its status line says `state`.
async fn panel_says(
    browser: &Browser,
    id: &str,
    text: &str,
    state: &str,
) -> Result<(), Box<dyn Error>> {
    let lines = ferrohealth_ui::view::code::line_count(text).to_string();
    until(
        &format!("the panel of {id} counts {lines} lines and says {state:?}"),
        async || {
            let [bar, status]: [String; 2] = browser
                .driver()
                .execute(
                    r"const panel = document.getElementById(arguments[0]).closest('section');
                    return [
                      panel.querySelector('.fui-editor-bar > span').textContent,
                      panel.querySelector('.fui-editor-status').textContent,
                    ];",
                    vec![serde_json::to_value(id)?],
                )
                .await?
                .convert()?;
            let count = bar.rsplit('·').next().unwrap_or_default();
            let digits: String = count.chars().filter(char::is_ascii_digit).collect();
            Ok(digits == lines && status.contains(state))
        },
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_key_typed_into_a_large_document_costs_what_it_costs_in_a_small_one()
-> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url("/ferroterm/forms")).await?;
        for id in ["gallery-draft", "gallery-mapping"] {
            let area = browser.element(By::Id(id)).await?;
            until("the editor hydrates", async || {
                browser.editor_live(&area).await
            })
            .await?;
            let ([small, large], last) = rounds(browser, id).await?;
            // A text 32 times longer costs a little more per key for the copies of
            // the text, never 32 times more; the floor is three ticks of the clock.
            assert!(
                large <= 5.0 * small.max(0.3),
                "a key in {id} costs {large:.3} ms in 64 KB against {small:.3} ms in 2 KB"
            );
            if id == "gallery-mapping" {
                panel_says(browser, id, &last, "Changed since opened").await?;
                let opened = ferrohealth_ui_gallery::data::DRAFT_OPENED;
                put(browser, id, opened).await?;
                panel_says(browser, id, opened, "Unchanged").await?;
            }
        }
        browser.console_clean("typing into a large document").await
    })
    .await
}
