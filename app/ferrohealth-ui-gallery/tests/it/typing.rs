// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Typing into a large document: the editor's work per keystroke follows
//! the change, not the length of the text, in YAML (#458) and in JSON and
//! XML, whose tokens run past their line (#531); an edit draws the layer a
//! whole pass over the text draws (#531); the code editor's panel keeps its
//! line count and changed state by the edit (#477), and a console that
//! takes each edit keeps its own copy of the text in step (#478).

use std::error::Error;

use ferrohealth_ui_gallery::data;
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

/// The editors a round types into: the page, the text area and the
/// gallery sample their document repeats. The upload field's editor reads
/// its grammar from the text, so the XML sample makes it an XML editor.
const EDITORS: &[(&str, &str, &str)] = &[
    ("/ferroterm/forms", "gallery-draft", data::YAML),
    ("/ferroterm/forms", "gallery-mapping", data::YAML),
    ("/ferroterm/operations", "pair-waiting-input", data::JSON),
    ("/ferroterm/forms", "gallery-upload-text", data::XML),
];

/// The gallery sample `unit`, repeated to at least `bytes` bytes.
fn document(unit: &str, bytes: usize) -> String {
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

/// Types a round into the editor `id` at 2 KB and at 64 KB of `unit`,
/// checks each layer draws the text area's text, and returns the median
/// time of a key at each size and the text the last round left.
async fn rounds(
    browser: &Browser,
    id: &str,
    unit: &str,
) -> Result<([f64; 2], String), Box<dyn Error>> {
    let mut medians = [0.0; 2];
    let mut last = String::new();
    for (bytes, median_of) in [2 * 1024, 64 * 1024].into_iter().zip(&mut medians) {
        let text = document(unit, bytes);
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
        for &(page, id, unit) in EDITORS {
            browser.goto(&gallery.url(page)).await?;
            let area = browser.element(By::Id(id)).await?;
            until("the editor is enhanced", async || {
                browser.editor_live(&area).await
            })
            .await?;
            let ([small, large], last) = rounds(browser, id, unit).await?;
            // A text 32 times longer costs a little more per key for the copies of
            // the text, never 32 times more; the floor is three ticks of the clock.
            eprintln!("PROBE {id}: small {small:.4} large {large:.4}");
            assert!(
                large <= 5.0 * small.max(0.3),
                "a key in {id} costs {large:.3} ms in 64 KB against {small:.3} ms in 2 KB"
            );
            if id == "gallery-mapping" {
                panel_says(browser, id, &last, "Changed since opened").await?;
                let opened = data::DRAFT_OPENED;
                put(browser, id, opened).await?;
                panel_says(browser, id, opened, "Unchanged").await?;
            }
        }
        browser.console_clean("typing into a large document").await
    })
    .await
}

/// The size of the document typed key by key, an openEHR composition's.
const TYPED_BYTES: usize = 20 * 1024;

/// How many keys one command types, so no command nears the driver's
/// script timeout.
const CHUNK: usize = 1024;

/// The most a key of the typed document may cost, at the median, in
/// milliseconds.
const KEY_BUDGET: f64 = 1.0;

/// The most the editor may spend on the whole typed document, in
/// milliseconds: the 10 s a console's harness gives one command.
const DOCUMENT_BUDGET: f64 = 10_000.0;

/// Types `text` at the end of the editor `id`, one key and one input event
/// each, and returns the time each key took from the dispatch of its input
/// event until the editor had taken it, before the browser draws.
async fn type_out(browser: &Browser, id: &str, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    put(browser, id, "").await?;
    let keys: Vec<char> = text.chars().collect();
    let mut times = Vec::with_capacity(keys.len());
    for chunk in keys.chunks(CHUNK) {
        let chunk: String = chunk.iter().collect();
        let answer = browser
            .driver()
            .execute_async(
                r"const done = arguments[arguments.length - 1];
                const area = document.getElementById(arguments[0]);
                const keys = Array.from(arguments[1]);
                const settle = () => new Promise((resolve) => {
                  const channel = new MessageChannel();
                  channel.port1.onmessage = () => resolve();
                  channel.port2.postMessage(0);
                });
                let heard = 0;
                const hear = () => {
                  heard = performance.now();
                };
                window.addEventListener('input', hear, true);
                (async () => {
                  area.focus();
                  const times = [];
                  for (const key of keys) {
                    area.setSelectionRange(area.value.length, area.value.length);
                    document.execCommand('insertText', false, key);
                    for (let turn = 0; turn < 16; turn += 1) {
                      await Promise.resolve();
                    }
                    times.push(performance.now() - heard);
                    await settle();
                  }
                  window.removeEventListener('input', hear, true);
                  done(times);
                })();",
                vec![serde_json::to_value(id)?, serde_json::to_value(chunk)?],
            )
            .await?;
        let chunk_times: Vec<f64> = serde_json::from_value(answer.json().clone())?;
        times.extend(chunk_times);
    }
    Ok(times)
}

/// The text area's text and the text of each line of its layer.
async fn drawn(browser: &Browser, id: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
    let answer = browser
        .driver()
        .execute(
            r"const area = document.getElementById(arguments[0]);
            const layer = area.previousElementSibling;
            return [area.value, Array.from(layer.children).map((line) => line.textContent)];",
            vec![serde_json::to_value(id)?],
        )
        .await?;
    Ok(serde_json::from_value(answer.json().clone())?)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_composition_typed_key_by_key_stays_inside_its_budget() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for &(page, id, unit) in EDITORS.iter().filter(|(_, _, unit)| *unit != data::YAML) {
            browser.goto(&gallery.url(page)).await?;
            let area = browser.element(By::Id(id)).await?;
            until("the editor is enhanced", async || {
                browser.editor_live(&area).await
            })
            .await?;
            let text = document(unit, TYPED_BYTES);
            let times = type_out(browser, id, &text).await?;
            assert_eq!(text.chars().count(), times.len());
            let (value, lines) = drawn(browser, id).await?;
            assert_eq!(text, value, "the text area {id} holds every key");
            assert_eq!(
                value,
                lines.join("\n"),
                "the layer of {id} draws the text area's text, line by line"
            );
            let key = median(&times);
            let whole: f64 = times.iter().sum();
            eprintln!("PROBE {id}: median {key:.4} ms, sum {whole:.0} ms, max {:.2}", times.iter().copied().fold(0.0, f64::max));
            assert!(
                key <= KEY_BUDGET,
                "a key typed into {id} costs {key:.3} ms at the median, over {KEY_BUDGET} ms"
            );
            assert!(
                whole <= DOCUMENT_BUDGET,
                "typing {} bytes into {id} key by key costs {whole:.0} ms, over {DOCUMENT_BUDGET} ms",
                text.len()
            );
        }
        browser.console_clean("typing a composition key by key").await
    })
    .await
}

/// A JSON text whose string, key and value run past their lines.
const JSON_EDGES: &str = r#"{
  "note": "a string
that runs past
its line",
  "key"

    : 1,
  "list": [true, false, null, -1.5e3],
  "tail": "end"
}"#;

/// An XML text whose comment, tag, attribute value and CDATA section run
/// past their lines.
const XML_EDGES: &str = r#"<?xml version="1.0"?>
<!-- a comment
that runs over
three lines -->
<composition
    archetype_node_id="openEHR-EHR-COMPOSITION.gallery.v1"
    note='a value
over two lines'>
  <![CDATA[ raw <text>
  over lines ]]>
  <name><value>Gallery</value></name>
</composition>"#;

/// One edit: its name, the text it finds, how far past the found text it
/// starts, how many characters it removes, and what it inserts.
type Edit = (&'static str, &'static str, usize, usize, &'static str);

/// The edits at the start, the middle and the end of each JSON token that
/// runs past its line.
const JSON_EDITS: &[Edit] = &[
    ("the string's opening quote goes", "\"a string", 0, 1, ""),
    (
        "a quote closes the string in its middle",
        "runs past",
        5,
        0,
        "\"",
    ),
    ("the string's closing quote goes", "line\",", 4, 1, ""),
    ("the key's colon on a later line goes", "    : 1", 4, 1, ""),
    (
        "a colon follows the key on its own line",
        "\"key\"",
        5,
        0,
        ":",
    ),
    ("a quote opens a string at the start", "{", 1, 0, "\""),
    ("a quote opens a string at the end", "\"end\"", 5, 0, " \""),
    ("a line break splits the string", "that runs", 4, 1, "\n"),
];

/// The edits at the start, the middle and the end of each XML token that
/// runs past its line.
const XML_EDITS: &[Edit] = &[
    ("the comment's opening goes", "<!--", 1, 1, ""),
    ("the comment closes in its middle", "runs over", 5, 0, "-->"),
    ("the comment's close goes", "lines -->", 6, 3, ""),
    ("the tag's opening goes", "<composition", 0, 1, ""),
    ("the tag closes in its middle", "    archetype", 0, 0, ">"),
    ("the tag's close goes", "lines'>", 6, 1, ""),
    ("the value's opening quote goes", "note='", 5, 1, ""),
    (
        "a quote opens a value in its middle",
        "over two",
        4,
        0,
        "\"",
    ),
    (
        "the CDATA section closes in its middle",
        "raw <text>",
        4,
        0,
        "]]>",
    ),
    ("the CDATA section's close goes", "lines ]]>", 6, 3, ""),
    ("a tag opens at the end", "</composition>", 14, 0, "\n<open"),
];

/// How many seeded random edits follow the named ones.
const RANDOM_EDITS: usize = 300;

/// Puts `text` into the editor `id`, then makes each edit and undoes it,
/// then makes [`RANDOM_EDITS`] seeded random edits, and returns every
/// edit after which the layer's markup differs from a whole highlighting
/// of the text area's text in the editor's grammar.
async fn misdrawn(
    browser: &Browser,
    id: &str,
    text: &str,
    edits: &[Edit],
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut placed = Vec::new();
    for &(name, find, past, removed, inserted) in edits {
        let at = text
            .find(find)
            .ok_or_else(|| format!("the edit {name:?} finds {find:?}"))?
            + past;
        placed.push(serde_json::json!([name, at, removed, inserted]));
    }
    let answer = browser
        .driver()
        .execute_async(
            r"const done = arguments[arguments.length - 1];
            const [id, text, edits, random, alphabet] = arguments;
            (async () => {
              const loader = document.querySelector('script[src*=\'fui.js\']');
              const { linesHtml } = await import(new URL('highlight.js', loader.src).href);
              const area = document.getElementById(id);
              const root = area.closest('[data-language]');
              const layer = area.previousElementSibling;
              const normal = (html) => {
                const template = document.createElement('template');
                template.innerHTML = html;
                return template.innerHTML;
              };
              const misses = [];
              const check = (name) => {
                const want = linesHtml(root.getAttribute('data-language'), area.value).map(normal);
                const got = Array.from(layer.children).map((line) => line.innerHTML);
                const at = want.findIndex((line, n) => line !== got[n]);
                if (want.length !== got.length || at >= 0) {
                  misses.push(name + ': line ' + (at + 1) + ' draws ' + JSON.stringify(got[at]) + ', a whole pass ' + JSON.stringify(want[at]));
                }
              };
              const edit = (at, removed, inserted) => {
                area.setSelectionRange(at, at + removed);
                if (inserted) document.execCommand('insertText', false, inserted);
                else if (removed) document.execCommand('delete');
              };
              area.focus();
              area.select();
              document.execCommand('insertText', false, text);
              check('the text put whole');
              for (const [name, at, removed, inserted] of edits) {
                const gone = area.value.slice(at, at + removed);
                edit(at, removed, inserted);
                check(name);
                edit(at, inserted.length, gone);
                check(name + ', undone');
              }
              let seed = 531;
              const next = (bound) => {
                seed = (seed * 1103515245 + 12345) % 2147483648;
                return seed % bound;
              };
              for (let n = 0; n < random; n += 1) {
                const at = 1 + next(Math.max(1, area.value.length - 1));
                const removed = next(3) === 0 ? 1 + next(3) : 0;
                const inserted = removed && next(2) === 0 ? '' : Array.from({ length: 1 + next(3) }, () => alphabet[next(alphabet.length)]).join('');
                edit(at, Math.min(removed, area.value.length - at), inserted);
                check('random edit ' + (n + 1) + ' at ' + at);
              }
              done(misses);
            })();",
            vec![
                serde_json::to_value(id)?,
                serde_json::to_value(text)?,
                serde_json::to_value(placed)?,
                serde_json::to_value(RANDOM_EDITS)?,
                serde_json::to_value("\"'<>/!-:[]{}?= \nax")?,
            ],
        )
        .await?;
    Ok(serde_json::from_value(answer.json().clone())?)
}

#[tokio::test(flavor = "multi_thread")]
async fn an_edit_draws_the_layer_a_whole_pass_draws() -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        for (page, id, text, edits) in [
            (
                "/ferroterm/operations",
                "pair-waiting-input",
                JSON_EDGES,
                JSON_EDITS,
            ),
            (
                "/ferroterm/forms",
                "gallery-upload-text",
                XML_EDGES,
                XML_EDITS,
            ),
        ] {
            browser.goto(&gallery.url(page)).await?;
            let area = browser.element(By::Id(id)).await?;
            until("the editor is enhanced", async || {
                browser.editor_live(&area).await
            })
            .await?;
            let misses = misdrawn(browser, id, text, edits).await?;
            assert!(
                misses.is_empty(),
                "the layer of {id} differs from a whole pass after {} edits: {misses:#?}",
                misses.len()
            );
        }
        browser
            .console_clean("an edit draws what a whole pass draws")
            .await
    })
    .await
}
