// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms and controls at their edges: a long segment label, the
//! upload's focus, a refusal with thousands of issues, the picker's labels,
//! and a word that falls back to English on a page in another language.

use std::borrow::Cow;

use ferrohealth_ui::i18n::{Locale, provide_locale};
use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::view::refusal::{ISSUES_DRAWN, ISSUES_OPEN, Issue, Refusal, Severity, refusal};
use ferrohealth_ui::widget::date::DateWords;
use ferrohealth_ui::widget::picker::{Picker, picker_field};
use leptos::prelude::*;

use crate::markup::{has, tags};
use crate::render::html;

fluent_templates::static_loader! {
    static THROWAWAY = {
        locales: "./tests/it/locales",
        fallback_language: "qaa",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// `view` rendered in a page in the throwaway language `qaa`, whose catalog
/// holds a few of the library's messages.
fn in_qaa(view: impl FnOnce() -> AnyView) -> String {
    Owner::new().with(|| {
        provide_locale(
            Locale::parse("qaa")
                .expect("a private-use language")
                .with_catalog(&*THROWAWAY),
        );
        view().to_html()
    })
}

/// The block of the rule whose selector line is `selector`.
fn rule(selector: &str) -> &'static str {
    let at = STYLESHEET
        .find(&format!("{selector} {{\n"))
        .unwrap_or_else(|| panic!("no rule {selector}"));
    let rest = STYLESHEET.get(at..).unwrap_or_default();
    rest.get(..rest.find('}').unwrap_or(rest.len()))
        .unwrap_or_default()
}

#[test]
fn a_segments_words_wrap_and_a_wrapped_row_keeps_its_dividers() {
    // WCAG 2.2 SC 1.4.10: a long label wraps inside its segment, never clipped by the group.
    let segment = rule("  .fui-segment");
    assert!(
        segment.contains("overflow-wrap: anywhere;")
            && segment.contains("min-width: 0;")
            && !segment.contains("nowrap"),
        "{segment}"
    );
    let edges = rule("  .fui-segmented > .fui-segment");
    for line in [
        "margin-block-start: -1px;",
        "margin-inline-start: -1px;",
        "border-block-start: 1px solid var(--edge-strong);",
        "border-inline-start: 1px solid var(--edge-strong);",
    ] {
        assert!(edges.contains(line), "{line}: {edges}");
    }
}

#[test]
fn the_upload_shows_its_hidden_inputs_focus_on_the_chooser() {
    // WCAG 2.2 SC 2.4.7: the file input is visually hidden, so its focus shows on the label.
    let ring = rule("  .fui-drop:has(> input[type=\"file\"]:focus-visible) > label.fui-button");
    assert!(ring.contains("outline: 2px solid var(--focus);"), "{ring}");
}

/// A refusal answered with `count` warnings.
fn refusal_of(count: usize) -> String {
    html(move || {
        let issues = (0..count)
            .map(|place| {
                Issue::new(Severity::Warning, "business-rule").text(format!("issue {place}"))
            })
            .chain([Issue::new(Severity::Error, "processing").text("the gravest")])
            .collect();
        refusal(&Refusal {
            status: Some(422),
            summary: String::from("the server refused the bundle."),
            issues,
            ..Refusal::default()
        })
        .into_any()
    })
}

#[test]
fn a_refusal_of_thousands_of_issues_shows_the_gravest_and_folds_and_counts_the_rest() {
    let shown = refusal_of(5000);
    let items = tags(&shown, "li").len();
    assert_eq!(
        ISSUES_DRAWN, items,
        "at most {ISSUES_DRAWN} issues are drawn"
    );
    let open = shown
        .split("<details")
        .next()
        .map_or(0, |before| before.matches("<li>").count());
    assert_eq!(ISSUES_OPEN, open, "the first {ISSUES_OPEN} are in the open");
    assert!(
        shown.find("the gravest") < shown.find("issue 0"),
        "the gravest first"
    );
    assert!(shown.contains("180 more issues"), "{shown}");
    assert!(
        shown.contains("4801 more issues are not shown here."),
        "{shown}"
    );
}

#[test]
fn a_refusal_is_announced_by_its_title_alone() {
    // WAI-ARIA 1.2, `alert`: the live region holds the title, never the list of issues.
    let shown = refusal_of(3);
    let alerts = tags(&shown, "div")
        .into_iter()
        .chain(tags(&shown, "p"))
        .filter(|tag| tag.get("role").map(String::as_str) == Some("alert"))
        .collect::<Vec<_>>();
    assert_eq!(1, alerts.len(), "{shown}");
    assert!(
        has(
            &shown,
            "p",
            &[("class", "fui-alert-title"), ("role", "alert")]
        ),
        "{shown}"
    );
}

#[test]
fn the_pickers_select_has_a_visible_label() {
    // WCAG 2.2 SC 3.3.2: each control of the picker carries a label a reader sees.
    let shown = html(|| {
        picker_field(Picker {
            id: String::from("vs"),
            name: String::from("url"),
            label: Cow::Borrowed("Value set"),
            choose_label: Cow::Borrowed("Choose a value set"),
            help: None,
            choices: Signal::stored(Vec::new()),
            value: RwSignal::new(String::new()),
            empty_word: Cow::Borrowed("Choose one"),
            other_word: Cow::Borrowed("Another"),
        })
    });
    assert!(
        has(
            &shown,
            "label",
            &[("for", "vs-choice"), ("class", "fui-label")]
        ),
        "{shown}"
    );
}

#[test]
fn a_word_that_falls_back_to_english_is_marked_english() {
    // WCAG 2.2 SC 3.1.2: the qaa catalog holds no severity word, so it is drawn in English.
    let shown = in_qaa(|| {
        refusal(&Refusal {
            status: Some(400),
            summary: String::from("qefused."),
            issues: vec![Issue::new(Severity::Error, "invalid").text("qad")],
            ..Refusal::default()
        })
        .into_any()
    });
    assert!(shown.contains(r#"<span lang="en">error</span>"#), "{shown}");
    let english = refusal_of(1);
    assert!(
        !english.contains(r#"lang="en""#),
        "an English page marks nothing: {english}"
    );
}

#[test]
fn a_date_is_typed_in_the_order_its_pages_language_writes() {
    let english = Owner::new().with(|| DateWords::of_page(false));
    assert_eq!(
        ferrohealth_ui::calendar::DateOrder::Ymd,
        english.order,
        "English takes ISO 8601"
    );
    let qaa = Owner::new().with(|| {
        provide_locale(
            Locale::parse("qaa")
                .expect("a private-use language")
                .with_catalog(&*THROWAWAY),
        );
        DateWords::of_page(false)
    });
    assert_eq!(ferrohealth_ui::calendar::DateOrder::Dmy, qaa.order);
}
