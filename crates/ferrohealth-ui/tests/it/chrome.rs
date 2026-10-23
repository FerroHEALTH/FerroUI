// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page chrome at its edges: binder tabs that outrun their sheet, a
//! long title or crumb, a long primary action, a squeezed or long search,
//! the dock in a short window, and paths known only at run time. Each rule
//! keeps a 320 px screen and 400% zoom from scrolling sideways (WCAG 2.2
//! SC 1.4.10).

use std::borrow::Cow;

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::view::shell::{Footer, footer, lockup, mark};
use leptos::prelude::*;

use crate::markup::has;
use crate::render::page_html;

/// One rule of the stylesheet: the at-rules and the selector around it,
/// outermost first, and its declarations.
#[derive(Debug)]
struct Rule {
    within: Vec<String>,
    body: String,
}

impl Rule {
    /// The rule's own selector.
    fn selector(&self) -> &str {
        self.within.last().map_or("", String::as_str)
    }

    /// Whether one of the at-rules around it is `at`.
    fn inside(&self, at: &str) -> bool {
        self.within.iter().any(|prelude| prelude == at)
    }
}

/// Every rule of the stylesheet that holds a declaration, with its
/// comments taken out.
fn rules() -> Vec<Rule> {
    let mut text = String::new();
    let mut rest = STYLESHEET;
    while let Some(open) = rest.find("/*") {
        text.push_str(&rest[..open]);
        let close = rest[open..].find("*/").expect("a closed comment");
        rest = &rest[open + close + 2..];
    }
    text.push_str(rest);
    let mut found = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut run = String::new();
    for c in text.chars() {
        match c {
            '{' => {
                stack.push(run.split_whitespace().collect::<Vec<_>>().join(" "));
                run.clear();
            }
            '}' => {
                if !run.trim().is_empty() && run.contains(':') {
                    found.push(Rule {
                        within: stack.clone(),
                        body: run.trim().to_owned(),
                    });
                }
                run.clear();
                stack.pop();
            }
            _ => run.push(c),
        }
    }
    found
}

/// The declarations of the rule `selector` under exactly the at-rules
/// `at`, outermost first, all joined.
fn body(at: &[&str], selector: &str) -> String {
    rules()
        .into_iter()
        .filter(|rule| {
            rule.selector() == selector
                && rule.within.len() == at.len() + 1
                && rule
                    .within
                    .iter()
                    .zip(at)
                    .all(|(prelude, at)| prelude == at)
        })
        .map(|rule| rule.body)
        .collect::<Vec<_>>()
        .join("\n")
}

const LAYER: &str = "@layer components";

#[test]
fn a_binder_row_scrolls_inside_itself_at_every_width() {
    let row = body(&[LAYER], ".fui-tabbed > .fui-tabs");
    assert!(
        row.contains("overflow-x: auto;"),
        "a row wider than its sheet widens the page from 40 rem: {row}"
    );
    assert!(row.contains("flex-wrap: nowrap;"), "{row}");
    assert!(
        !row.contains("overflow: visible;"),
        "the row lets its tabs out: {row}"
    );
}

#[test]
fn a_tab_is_the_first_scroll_target_only_of_a_row_that_scrolls() {
    // NOTE: CSS Scroll Snap 2, scroll-initial-target: a target scrolls its nearest scroll
    // container, which for a row that does not scroll is the page itself.
    let targets: Vec<Rule> = rules()
        .into_iter()
        .filter(|rule| rule.body.contains("scroll-initial-target"))
        .collect();
    assert!(!targets.is_empty());
    for rule in targets {
        assert!(
            rule.selector().starts_with(".fui-tabbed > .fui-tabs > ")
                || rule.inside("@media (width < 40rem)"),
            "a tab row that wraps would scroll the page to its tab: {rule:?}"
        );
    }
}

#[test]
fn a_long_title_or_crumb_wraps_inside_the_header() {
    let title = body(
        &[LAYER],
        ".fui-page-heading > :is(h1, .fui-page-title-line), .fui-page-title-line > h1",
    );
    assert!(
        title.contains("min-width: 0;") && title.contains("overflow-wrap: break-word;"),
        "a long code title widens the page: {title}"
    );
    let crumb = body(&[LAYER], ".fui-crumbs a");
    assert!(
        crumb.contains("min-width: 0;") && crumb.contains("overflow-wrap: anywhere;"),
        "a canonical crumb widens the page: {crumb}"
    );
    assert!(body(&[LAYER], ".fui-crumbs li").contains("min-width: 0;"));
}

#[test]
fn more_wraps_under_a_long_primary_action() {
    let group = body(&[LAYER], ".fui-action-group");
    assert!(
        group.contains("flex-wrap: wrap;") && group.contains("max-width: 100%;"),
        "More is pushed past the screen's edge: {group}"
    );
    assert!(
        body(&[LAYER], ".fui-page-heading > .fui-actions").contains("max-width: 100%;"),
        "the actions outgrow the header"
    );
}

#[test]
fn a_stat_cards_mark_never_outgrows_its_card() {
    let mark = body(&[LAYER], ".fui-stat-label > :not(:first-child)");
    assert!(
        mark.contains("max-width: 100%;") && mark.contains("white-space: normal;"),
        "a pill after the label runs past a narrow card: {mark}"
    );
}

#[test]
fn the_search_answer_scrolls_inside_itself_and_wraps_its_names() {
    let answer = body(&[LAYER], ".fui-search-results");
    assert!(
        answer.contains("max-height: min(32rem, calc(100dvh - var(--bar) - 1rem));")
            && answer.contains("overflow-y: auto;"),
        "forty hits run past the window's foot under the sticky bar: {answer}"
    );
    let name = body(
        &[LAYER],
        ".fui-search-results-name, .fui-search-results li > a > .fui-small",
    );
    assert!(
        name.contains("overflow-wrap: break-word;"),
        "a long name runs under the tag: {name}"
    );
}

#[test]
fn a_squeezed_search_gives_up_its_word_then_its_key_cap() {
    assert!(
        body(&[LAYER], ".fui-search").contains("container: fui-search / inline-size;"),
        "the search cannot measure itself"
    );
    assert!(
        body(
            &[LAYER, "@container fui-search (width < 22rem)"],
            ".fui-search-word"
        )
        .contains("clip-path: inset(50%);"),
        "the submit keeps its word while the field shrinks to a letter"
    );
    assert!(
        body(
            &[LAYER, "@container fui-search (width < 16rem)"],
            ".fui-search-hint"
        )
        .contains("display: none;"),
        "the key cap keeps its room in a narrow field"
    );
    assert!(
        body(&[LAYER, "@media (width >= 48rem)"], ".fui-topbar-middle")
            .contains("min-width: 12rem;"),
        "the field gives way below a usable width"
    );
}

#[test]
fn the_dock_stops_sticking_in_a_short_window() {
    // NOTE: WCAG 2.2 SC 2.4.11: the bar sticks only where the scroll padding keeps a
    // focused control clear of it.
    let dock = body(
        &[
            LAYER,
            "@media (height < 30rem), (width < 40rem) and (height < 40rem)",
        ],
        ".fui-dock",
    );
    assert!(
        dock.contains("position: static;"),
        "the bar covers most of a 320 by 256 window: {dock}"
    );
    assert!(
        body(
            &[LAYER, "@media (width >= 40rem) and (height >= 30rem)"],
            "html:has(.fui-dock)"
        )
        .contains("scroll-padding-bottom: 9rem;"),
        "a sticking bar hides the control the keyboard reaches"
    );
}

#[test]
fn the_footer_and_the_lockup_take_paths_known_at_run_time() {
    let base = String::from("/console");
    let page = page_html(|| {
        let about = format!("{base}/about");
        view! {
            {lockup(format!("{base}/"), mark(Glyph::LAYERS), "FerroUI", "Gallery")}
            {footer(Footer {
                product: "FerroUI Gallery".into(),
                version: Cow::Owned(format!("0.3.{}", 2)),
                about: about.into(),
            })}
        }
        .into_any()
    });
    assert!(
        has(
            &page,
            "a",
            &[("href", "/console/"), ("class", "fui-lockup")]
        ),
        "{page}"
    );
    assert!(has(&page, "a", &[("href", "/console/about")]), "{page}");
    assert!(page.contains("FerroUI Gallery 0.3.2"), "{page}");
}
