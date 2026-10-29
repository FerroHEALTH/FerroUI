// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The docked decision bar (#145): every decision a form submit, the fields
//! it takes, a disabled decision that says why in words, its states, and
//! the stylesheet rules that dock it and keep a focused control clear.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::dock::{
    Decision, DecisionBar, DecisionState, DockField, DockForm, DockLine, DockLink, decision_bar,
};

use crate::markup::{has, tags};
use crate::render::html;

/// The reason field, changed by `change`.
fn reason(change: impl FnOnce(&mut DockField)) -> DockField {
    let mut field = DockField {
        id: String::from("reason"),
        name: String::from("reason"),
        label: String::from("Reason"),
        value: String::from("Same address history"),
        placeholder: String::from("Why you decided this"),
        shortcut: Some(String::from("r")),
        ..DockField::default()
    };
    change(&mut field);
    field
}

/// Link (the primary, `l`) and Not a match, in `link` and `other` states.
fn decisions(link: DecisionState, other: DecisionState) -> Vec<Decision> {
    vec![
        Decision {
            value: String::from("link"),
            label: String::from("Link"),
            variant: Variant::Primary,
            glyph: Some(Glyph::LINK),
            shortcut: Some(String::from("l")),
            state: link,
            ..Decision::default()
        },
        Decision {
            value: String::from("not-a-match"),
            label: String::from("Not a match"),
            state: other,
            ..Decision::default()
        },
    ]
}

/// The bar of its own form at `/decide`, with `fields`, `line` and
/// `decisions`.
fn bar(fields: Vec<DockField>, line: Option<DockLine>, decisions: Vec<Decision>) -> String {
    html({
        decision_bar(DecisionBar {
            id: String::from("dock"),
            label: String::from("Decision"),
            form: DockForm::Own(String::from("/decide")),
            fields,
            line,
            decisions,
        })
    })
}

/// The bar in its idle state, with the reason.
fn idle() -> String {
    bar(
        vec![reason(|_| {})],
        None,
        decisions(DecisionState::Ready, DecisionState::Ready),
    )
}

#[test]
fn the_bar_is_a_named_region_whose_decisions_submit_its_post_form() {
    let html = idle();
    assert!(
        has(
            &html,
            "div",
            &[
                ("id", "dock"),
                ("class", "fui-dock"),
                ("role", "region"),
                ("aria-label", "Decision")
            ]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "form",
            &[
                ("id", "dock-form"),
                ("method", "post"),
                ("action", "/decide"),
                ("hidden", "")
            ]
        ),
        "{html}"
    );
    let buttons = tags(&html, "button");
    assert_eq!(2, buttons.len(), "{html}");
    for button in &buttons {
        assert_eq!(
            Some("submit"),
            button.get("type").map(String::as_str),
            "{html}"
        );
        assert_eq!(
            Some("dock-form"),
            button.get("form").map(String::as_str),
            "{html}"
        );
        assert_eq!(
            Some("decision"),
            button.get("name").map(String::as_str),
            "{html}"
        );
        assert!(
            !button.contains_key("disabled"),
            "idle decisions are ready: {html}"
        );
    }
    assert!(
        has(&html, "input", &[("id", "reason"), ("form", "dock-form")]),
        "{html}"
    );
    assert!(!html.contains("style="), "{html}");
}

#[test]
fn a_bar_on_the_pages_form_joins_it_and_draws_no_form_of_its_own() {
    let html = html({
        decision_bar(DecisionBar {
            id: String::from("dock"),
            label: String::from("Your changes"),
            form: DockForm::Page(String::from("editor")),
            fields: vec![reason(|_| {})],
            line: None,
            decisions: decisions(DecisionState::Ready, DecisionState::Ready),
        })
    });
    assert!(tags(&html, "form").is_empty(), "{html}");
    assert!(
        has(&html, "button", &[("value", "link"), ("form", "editor")]),
        "{html}"
    );
    assert!(
        has(&html, "input", &[("id", "reason"), ("form", "editor")]),
        "{html}"
    );
}

#[test]
fn a_decision_that_leaves_skips_the_forms_checks() {
    let mut decisions = decisions(DecisionState::Ready, DecisionState::Ready);
    decisions.push(Decision {
        value: String::from("discard"),
        label: String::from("Discard"),
        leaves: true,
        ..Decision::default()
    });
    let html = bar(vec![reason(|_| {})], None, decisions);
    let buttons = tags(&html, "button");
    let discard = buttons
        .iter()
        .find(|button| button.get("value").map(String::as_str) == Some("discard"))
        .expect("Discard is drawn");
    assert!(discard.contains_key("formnovalidate"), "{html}");
    assert_eq!(
        1,
        buttons
            .iter()
            .filter(|button| button.contains_key("formnovalidate"))
            .count(),
        "a decision that keeps what the form holds is checked: {html}"
    );
}

#[test]
fn a_shortcut_is_a_quiet_cap_and_announced() {
    let html = idle();
    assert!(
        has(
            &html,
            "button",
            &[("value", "link"), ("aria-keyshortcuts", "L")]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "input",
            &[("id", "reason"), ("aria-keyshortcuts", "R")]
        ),
        "{html}"
    );
    assert!(
        html.contains(r#"Link<kbd class="fui-kbd">L</kbd>"#),
        "{html}"
    );
    assert!(
        html.contains(r#"Reason<kbd class="fui-kbd">R</kbd>"#),
        "{html}"
    );
}

#[test]
fn a_disabled_decision_says_why_in_visible_words_it_names() {
    let html = bar(
        vec![reason(|field| field.value = String::new())],
        None,
        decisions(
            DecisionState::Blocked(String::from("Give a reason to save.")),
            DecisionState::Ready,
        ),
    );
    assert!(
        has(
            &html,
            "button",
            &[
                ("value", "link"),
                ("disabled", ""),
                ("aria-describedby", "dock-why-1")
            ]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "p",
            &[("id", "dock-why-1"), ("class", "fui-dock-message")]
        ),
        "{html}"
    );
    assert!(
        html.contains("<span>Give a reason to save.</span>"),
        "{html}"
    );
    let link = tags(&html, "button").remove(0);
    assert!(!link.contains_key("title"), "never only a tooltip: {html}");
}

#[test]
fn a_bar_with_no_field_puts_its_line_in_the_fields_place() {
    let html = bar(
        Vec::new(),
        None,
        decisions(
            DecisionState::Ready,
            DecisionState::Blocked(String::from("Fix the path to create the file.")),
        ),
    );
    let row = html.find(r#"<div class="fui-dock-row">"#).expect("the row");
    let why = html.find(r#"id="dock-why-2""#).expect("the reason line");
    assert!(why > row, "the reason sits in the row: {html}");
    assert!(tags(&html, "input").is_empty(), "{html}");
    let state = bar(
        Vec::new(),
        Some(DockLine::State {
            glyph: Some(Glyph::PENCIL),
            said: String::from("2 changes in this draft"),
        }),
        decisions(DecisionState::Ready, DecisionState::Ready),
    );
    let row = state
        .find(r#"<div class="fui-dock-row">"#)
        .expect("the row");
    let line = state
        .find("2 changes in this draft")
        .expect("the state line");
    assert!(line > row, "the state line sits in the row: {state}");
}

#[test]
fn a_reason_refused_is_marked_invalid_named_and_takes_the_focus() {
    let html = bar(
        vec![reason(|field| {
            field.value = String::new();
            field.error = Some(String::from("Write why <you> decided this."));
            field.help = Some(String::from("One line"));
            field.focus = true;
        })],
        None,
        decisions(DecisionState::Ready, DecisionState::Ready),
    );
    assert!(
        has(
            &html,
            "input",
            &[
                ("id", "reason"),
                ("aria-invalid", "true"),
                ("aria-describedby", "reason-error"),
                ("autofocus", "")
            ]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "p",
            &[("id", "reason-error"), ("class", "fui-error")]
        ),
        "{html}"
    );
    assert!(
        html.contains("Write why &#60;you&#62; decided this."),
        "{html}"
    );
    assert!(
        !html.contains("One line"),
        "the error takes the help's place: {html}"
    );
    assert!(
        tags(&html, "button")
            .iter()
            .all(|button| !button.contains_key("disabled")),
        "the decisions stay enabled: {html}"
    );
}

#[test]
fn the_chosen_decision_is_busy_and_the_others_wait() {
    let html = bar(
        vec![reason(|field| field.disabled = true)],
        None,
        decisions(DecisionState::Busy, DecisionState::Held),
    );
    assert!(
        has(
            &html,
            "button",
            &[("value", "link"), ("aria-busy", "true"), ("disabled", "")]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "span",
            &[("class", "fui-spinner-ring"), ("aria-hidden", "true")]
        ),
        "{html}"
    );
    assert!(!html.contains(r#"<kbd class="fui-kbd">L</kbd>"#), "{html}");
    let other = tags(&html, "button").remove(1);
    assert!(other.contains_key("disabled"), "{html}");
    assert!(!other.contains_key("aria-describedby"), "{html}");
    assert!(
        has(&html, "input", &[("id", "reason"), ("disabled", "")]),
        "{html}"
    );
}

#[test]
fn a_done_bar_confirms_in_words_and_a_stale_one_says_what_happened() {
    let done = bar(
        vec![reason(|field| field.disabled = true)],
        Some(DockLine::Done(String::from("Linked."))),
        decisions(DecisionState::Held, DecisionState::Held),
    );
    assert!(
        has(
            &done,
            "p",
            &[
                ("class", "fui-dock-message fui-dock-done"),
                ("role", "status")
            ]
        ),
        "{done}"
    );
    let stale = bar(
        vec![reason(|_| {})],
        Some(DockLine::Stale {
            said: String::from("Someone decided first."),
            ways: vec![DockLink {
                href: String::from("/next"),
                words: String::from("Open the next candidate"),
                variant: Variant::Secondary,
            }],
        }),
        decisions(DecisionState::Held, DecisionState::Held),
    );
    assert!(stale.contains(r#"role="alert""#), "{stale}");
    assert!(stale.contains("Someone decided first."), "{stale}");
    assert!(has(&stale, "a", &[("href", "/next")]), "{stale}");
}

#[test]
fn the_bar_takes_a_reason_and_a_path() {
    let html = bar(
        vec![
            reason(|_| {}),
            DockField {
                id: String::from("path"),
                name: String::from("path"),
                label: String::from("Path"),
                value: String::from("fhirconnect/a.yml"),
                ..DockField::default()
            },
        ],
        None,
        decisions(DecisionState::Ready, DecisionState::Ready),
    );
    assert_eq!(2, tags(&html, "input").len(), "{html}");
    assert!(has(&html, "label", &[("for", "path")]), "{html}");
}

#[test]
fn the_bar_docks_two_fields_stacks_on_a_phone_and_keeps_a_focused_control_clear() {
    assert!(
        STYLESHEET.contains("  .fui-dock {\n    position: sticky;\n    bottom: 0;"),
        "the bar is not docked"
    );
    assert!(
        STYLESHEET.contains(
            "  .fui-dock-row:has(> .fui-dock-field ~ .fui-dock-field) {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) auto;"
        ),
        "two fields do not share the row"
    );
    assert!(
        STYLESHEET.contains(
            "    .fui-dock-row .fui-dock-actions {\n      display: flex;\n      flex-direction: column;"
        ),
        "the decisions do not stack on a phone"
    );
    assert!(
        STYLESHEET.contains("    .fui-dock-actions > .fui-button-primary {\n      order: -1;"),
        "the primary is not first on a phone"
    );
    // WCAG 2.2 SC 2.4.11: a control the keyboard reaches is never hidden under the bar.
    assert!(
        STYLESHEET.contains("    html:has(.fui-dock) {\n      scroll-padding-bottom:"),
        "no scroll padding above the bar"
    );
    assert!(
        !STYLESHEET.contains("fui-action-bar"),
        "the card's action bar is gone"
    );
}
