// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The state mark (F12) as the server draws it: each mark's shape and
//! tone, its word, its quiet detail, and the escaping of what it is given.

use ferrohealth_ui::view::flow::FlowState;
use ferrohealth_ui::view::state::{Mark, state};

use crate::markup::has;

/// The HTML of the state mark `mark` beside `word`, with `detail`.
fn drawn(mark: Mark, word: &str, detail: Option<&str>) -> String {
    state(mark, word, detail.map(str::to_owned)).to_string()
}

#[test]
fn each_mark_draws_its_shape_in_its_tone_beside_its_word() {
    // Family rule 12; WCAG 2.2 SC 1.4.1: a shape, a colour and a word together.
    for (mark, tone, word) in [
        (Mark::Ok, "fui-tone-ok", "Reachable"),
        (Mark::Warn, "fui-tone-warn", "Degraded"),
        (Mark::Bad, "fui-tone-bad", "Unreachable"),
        (Mark::Neutral, "fui-tone-neutral", "Not checked"),
    ] {
        let html = drawn(mark, word, None);
        assert!(
            has(&html, "span", &[("class", &format!("fui-state {tone}"))]),
            "{html}"
        );
        assert!(
            has(
                &html,
                "span",
                &[
                    ("class", &format!("fui-dot {tone}")),
                    ("aria-hidden", "true")
                ]
            ),
            "the shape is hidden, the word names the state: {html}"
        );
        assert!(html.contains(word), "{html}");
        assert!(!html.contains("style="), "{html}");
    }
}

#[test]
fn a_state_switched_off_is_the_hollow_ring_with_its_word_muted() {
    let html = drawn(Mark::Off, "Excluded", None);
    assert!(
        has(
            &html,
            "span",
            &[("class", "fui-state fui-tone-neutral is-muted")]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "span",
            &[("class", "fui-dot fui-tone-neutral is-muted")]
        ),
        "{html}"
    );
}

#[test]
fn a_detail_follows_the_word_quietly_and_is_text() {
    let html = drawn(Mark::Warn, "1 warning", Some("<b>2,940 ms</b>"));
    assert!(
        html.contains(r#"<span class="fui-status-detail">&#60;b&#62;2,940 ms&#60;/b&#62;</span>"#),
        "{html}"
    );
    let plain = drawn(Mark::Ok, "Compiles", None);
    assert!(!plain.contains("fui-status-detail"), "{plain}");
}

#[test]
fn a_flow_state_takes_the_mark_of_its_shape() {
    assert_eq!(Mark::Ok, Mark::from(FlowState::Up));
    assert_eq!(Mark::Warn, Mark::from(FlowState::Degraded));
    assert_eq!(Mark::Bad, Mark::from(FlowState::Unreachable));
    assert_eq!(Mark::Neutral, Mark::from(FlowState::NotChecked));
    assert_eq!(Mark::Off, Mark::from(FlowState::Off));
}
