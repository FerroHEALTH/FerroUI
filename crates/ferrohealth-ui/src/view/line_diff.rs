// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The line diff (B11): one file's change as a unified diff, in the code
//! viewer's box.
//!
//! Each line carries its number before and its number after, its sign and
//! its text. A removed line is marked "−" on the bad tone's ground and an
//! added line "+" on the ok tone's ground, and each carries its word for
//! assistive technology, so a change never rides on colour alone (WCAG 2.2
//! SC 1.4.1). A hunk's head is a muted row. A line never wraps: a long one
//! scrolls sideways inside the box under the scroll cue, and the box is
//! focusable so the keyboard scrolls it (SC 1.4.10, SC 2.1.1).
//!
//! The diff is read as the unified format writes it: the file headers
//! before the first hunk (`diff`, `index`, `---`, `+++`) are left out, a
//! hunk's head (`@@ -12,7 +12,9 @@`) sets the numbers of the lines after
//! it, and a line is context, removed or added by its first character. A
//! line before any hunk head is numbered from 1. No specification governs
//! the drawing: our own design.

use crate::html::Html;
use crate::i18n::{Locale, Text, tr, tr_with};

/// What one line of a unified diff is.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Line<'a> {
    /// A hunk's head, or a note such as `\ No newline at end of file`.
    Head(&'a str),
    /// A line both sides hold, at its number before and after.
    Context(u32, u32, &'a str),
    /// A line only the side before holds.
    Removed(u32, &'a str),
    /// A line only the side after holds.
    Added(u32, &'a str),
}

/// The first line numbers a hunk's head `head` names, before and after;
/// `None` for a head that names none.
fn starts(head: &str) -> Option<(u32, u32)> {
    let mut parts = head.split_whitespace().skip(1);
    let before = parts.next()?.strip_prefix('-')?;
    let after = parts.next()?.strip_prefix('+')?;
    let first = |range: &str| range.split(',').next()?.parse::<u32>().ok();
    Some((first(before)?, first(after)?))
}

/// The lines of the unified diff `diff`, each numbered.
fn lines(diff: &str) -> Vec<Line<'_>> {
    let (mut before, mut after) = (1_u32, 1_u32);
    let mut in_hunk = false;
    let mut out = Vec::new();
    for line in diff.lines() {
        if !in_hunk
            && ["diff ", "index ", "--- ", "+++ "]
                .iter()
                .any(|header| line.starts_with(header))
        {
            continue;
        }
        if line.starts_with("@@") {
            // A head that names no numbers keeps the count of the lines
            // above it, and still shows.
            if let Some((from, to)) = starts(line) {
                (before, after) = (from, to);
            }
            in_hunk = true;
            out.push(Line::Head(line));
        } else if let Some(text) = line.strip_prefix('-') {
            out.push(Line::Removed(before, text));
            before = before.saturating_add(1);
        } else if let Some(text) = line.strip_prefix('+') {
            out.push(Line::Added(after, text));
            after = after.saturating_add(1);
        } else if line.starts_with('\\') {
            out.push(Line::Head(line));
        } else {
            out.push(Line::Context(
                before,
                after,
                line.strip_prefix(' ').unwrap_or(line),
            ));
            before = before.saturating_add(1);
            after = after.saturating_add(1);
        }
    }
    out
}

/// One line of the diff: its two numbers, its sign, its word for assistive
/// technology and its text, in the class `class`.
fn row(
    class: &str,
    (before, after): (Option<u32>, Option<u32>),
    sign: &str,
    word: Option<&str>,
    text: &str,
) -> String {
    let number = |n: Option<u32>| n.map_or_else(String::new, |n| n.to_string());
    let said = word.map_or_else(String::new, |word| {
        format!("<span class=\"fui-sr-only\">{} </span>", Html::text(word))
    });
    format!(
        "<span class=\"{class}\"><span aria-hidden=\"true\" class=\"fui-diff-num\">{}</span><span aria-hidden=\"true\" class=\"fui-diff-num\">{}</span><span aria-hidden=\"true\" class=\"fui-diff-sign\">{sign}</span><span class=\"fui-diff-text\">{said}{}</span></span>",
        number(before),
        number(after),
        Html::text(text)
    )
}

/// Returns the line diff of the unified diff `diff` of the file `file`;
/// a diff with no line in it reads as a sentence.
#[must_use]
pub fn line_diff(locale: &Locale, file: &str, diff: &str) -> Html {
    let lines = lines(diff);
    if lines.is_empty() {
        return Html::raw(format!(
            "<p class=\"fui-muted\">{}</p>",
            tr(locale, Text::DiffNone)
        ));
    }
    let removed = tr(locale, Text::DiffRemoved);
    let added = tr(locale, Text::DiffAdded);
    let rows: String = lines
        .iter()
        .map(|line| match line {
            Line::Head(text) => format!(
                "<span class=\"fui-diff-hunk\"><span class=\"fui-diff-text\">{}</span></span>",
                Html::text(text)
            ),
            Line::Context(before, after, text) => row(
                "fui-diff-line",
                (Some(*before), Some(*after)),
                "",
                None,
                text,
            ),
            Line::Removed(before, text) => row(
                "fui-diff-line fui-diff-removed",
                (Some(*before), None),
                "\u{2212}",
                Some(removed.text()),
                text,
            ),
            Line::Added(after, text) => row(
                "fui-diff-line fui-diff-added",
                (None, Some(*after)),
                "+",
                Some(added.text()),
                text,
            ),
        })
        .collect();
    let label = tr_with(locale, Text::DiffRegion, &[("file", file.into())]);
    Html::raw(format!(
        "<section aria-label=\"{}\" tabindex=\"0\" class=\"fui-code-panel fui-linediff fui-scroll-cue\"><pre>{rows}</pre></section>",
        Html::text(label.text())
    ))
}

#[cfg(test)]
mod tests {
    use super::{Line, lines, starts};

    #[test]
    fn a_hunk_head_names_where_its_lines_start() {
        assert_eq!(Some((12, 14)), starts("@@ -12,7 +14,9 @@ mappings:"));
        assert_eq!(Some((1, 1)), starts("@@ -1 +1 @@"));
        assert_eq!(None, starts("@@ nothing @@"));
    }

    #[test]
    fn the_lines_are_numbered_on_each_side_they_stand_on() {
        let diff = "--- a/f.yml\n+++ b/f.yml\n@@ -3,3 +3,3 @@ top\n keep\n-old\n+new\n\\ No newline at end of file\n";
        assert_eq!(
            vec![
                Line::Head("@@ -3,3 +3,3 @@ top"),
                Line::Context(3, 3, "keep"),
                Line::Removed(4, "old"),
                Line::Added(4, "new"),
                Line::Head("\\ No newline at end of file"),
            ],
            lines(diff)
        );
    }

    #[test]
    fn a_line_before_any_hunk_is_numbered_from_one() {
        assert_eq!(
            vec![Line::Removed(1, "a"), Line::Context(2, 1, "b")],
            lines("-a\n b\n")
        );
        assert_eq!(Vec::<Line<'_>>::new(), lines(""));
    }
}
