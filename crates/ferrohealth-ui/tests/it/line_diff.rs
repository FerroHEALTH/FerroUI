// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The line diff (B11): a unified diff drawn line by line, each change in
//! a sign, a tone and a word, in a box the keyboard scrolls.

use ferrohealth_ui::view::line_diff::line_diff;

use crate::markup::{has, tags};
use crate::render::{en, html};

/// A synthetic diff of a mapping file: headers, one hunk, a context line,
/// a removed line and an added line that needs escaping.
const DIFF: &str = "diff --git a/m.yml b/m.yml\n--- a/m.yml\n+++ b/m.yml\n@@ -12,3 +12,3 @@ mappings:\n   - name: weight\n-    unit: kg\n+    unit: <g & kg>\n";

#[test]
fn a_diff_is_a_region_the_keyboard_scrolls_named_by_its_file() {
    let page = html(line_diff(&en(), "fhirconnect/weight.context.yml", DIFF));
    assert!(
        has(
            &page,
            "div",
            &[
                ("role", "region"),
                ("tabindex", "0"),
                (
                    "aria-label",
                    "fhirconnect/weight.context.yml, its removed and added lines"
                ),
                ("class", "fui-code-panel fui-linediff fui-scroll-cue"),
            ]
        ),
        "{page}"
    );
}

#[test]
fn each_change_carries_a_sign_a_tone_and_a_word() {
    let page = html(line_diff(&en(), "m.yml", DIFF));
    let removed = tags(&page, "span")
        .into_iter()
        .filter(|span| {
            span.get("class").map(String::as_str) == Some("fui-diff-line fui-diff-removed")
        })
        .count();
    let added = tags(&page, "span")
        .into_iter()
        .filter(|span| {
            span.get("class").map(String::as_str) == Some("fui-diff-line fui-diff-added")
        })
        .count();
    assert_eq!((1, 1), (removed, added), "{page}");
    assert!(
        page.contains("<span class=\"fui-sr-only\">removed </span>    unit: kg"),
        "{page}"
    );
    assert!(
        page.contains("<span class=\"fui-sr-only\">added </span>    unit: &#60;g &#38; kg&#62;"),
        "the added line is escaped: {page}"
    );
    assert!(
        page.contains(">\u{2212}</span>"),
        "the removed sign: {page}"
    );
    assert!(!page.contains("<g & kg>"), "{page}");
}

#[test]
fn the_lines_carry_their_numbers_before_and_after_and_the_headers_go() {
    let page = html(line_diff(&en(), "m.yml", DIFF));
    assert!(
        !page.contains("--- a/m.yml"),
        "the file headers are left out: {page}"
    );
    assert!(
        page.contains("<span class=\"fui-diff-hunk\"><span class=\"fui-diff-text\">@@ -12,3 +12,3 @@ mappings:</span></span>"),
        "{page}"
    );
    assert!(
        page.contains("<span aria-hidden=\"true\" class=\"fui-diff-num\">12</span><span aria-hidden=\"true\" class=\"fui-diff-num\">12</span>"),
        "the context line keeps both numbers: {page}"
    );
    assert!(
        page.contains("<span aria-hidden=\"true\" class=\"fui-diff-num\">13</span><span aria-hidden=\"true\" class=\"fui-diff-num\"></span>"),
        "the removed line has no number after: {page}"
    );
    assert!(
        page.contains("<span aria-hidden=\"true\" class=\"fui-diff-num\"></span><span aria-hidden=\"true\" class=\"fui-diff-num\">13</span>"),
        "the added line has no number before: {page}"
    );
}

#[test]
fn a_diff_with_no_line_reads_as_a_sentence() {
    assert_eq!(
        "<p class=\"fui-muted\">No line changes.</p>",
        html(line_diff(&en(), "m.yml", "--- a/m.yml\n+++ b/m.yml\n"))
    );
}
