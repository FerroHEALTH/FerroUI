// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The configuration parts rendered to HTML: the revision lifecycle and
//! its pending card (F1), the commit-confirm countdown (F2), the write-only
//! secret field (F3) and the provenance tag (F4).

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::lifecycle::{
    Lifecycle, PendingCard, Step, StepState, lifecycle, pending_card, trial_actions,
};
use ferrohealth_ui::view::secret::{
    SecretChoice, SecretField, SecretState, secret_field, secret_state,
};
use ferrohealth_ui::view::tag::Tag;
use ferrohealth_ui::widget::action_menu::{Act, Action};
use ferrohealth_ui::widget::countdown::Countdown;
use leptos::prelude::*;

use crate::markup::{has, tags};
use crate::render::{html, page_html};

/// A secret that is set, read from a file.
const SET: &str = "/run/secrets/node-b.secret";

/// The steps of a revision on trial, its confirmation current.
fn on_trial() -> Lifecycle {
    Lifecycle {
        label: String::from("Where this revision stands"),
        steps: vec![
            Step::new(StepState::Done, "Saved", "10:02"),
            Step::new(StepState::Done, "Approved", "Demo Reviewer, 10:09"),
            Step::new(StepState::Done, "Applied", "10:12:42"),
            Step::new(StepState::Current, "Confirm", "by 10:14:42"),
        ],
    }
}

#[test]
fn a_lifecycle_draws_every_step_state_in_glyph_class_and_word() {
    let drawn = html(|| {
        lifecycle(Lifecycle {
            label: String::from("Where this revision stands"),
            steps: vec![
                Step::new(StepState::Done, "Saved", "by you, 10:02"),
                Step::new(StepState::Current, "Approval", "another operator"),
                Step::new(StepState::Waiting, "Apply", "after the approval"),
                Step::new(StepState::Skipped, "Confirm", "not needed"),
                Step::new(StepState::Failed, "Not confirmed", "deadline 10:14:42"),
            ],
        })
    });
    assert!(
        has(
            &drawn,
            "ol",
            &[
                ("class", "fui-lifecycle"),
                ("aria-label", "Where this revision stands")
            ]
        ),
        "an ordered list named for assistive technology: {drawn}"
    );
    let items = tags(&drawn, "li");
    let classes: Vec<&str> = items
        .iter()
        .filter_map(|item| item.get("class").map(String::as_str))
        .collect();
    assert_eq!(
        vec![
            "is-done",
            "is-current",
            "is-waiting",
            "is-skipped",
            "is-failed"
        ],
        classes,
        "{drawn}"
    );
    for word in [
        "Done: ",
        "Current: ",
        "Waiting: ",
        "Not needed: ",
        "Failed: ",
    ] {
        assert!(
            drawn.contains(&format!(r#"<span class="fui-sr-only">{word}</span>"#)),
            "family rule 12: each state is a word too, {word}: {drawn}"
        );
    }
    assert_eq!(
        3,
        tags(&drawn, "svg").len(),
        "a check, a dash and a cross; the current and waiting dots draw none: {drawn}"
    );
    assert!(
        tags(&drawn, "span")
            .iter()
            .filter(|span| span.get("class").map(String::as_str) == Some("fui-lifecycle-dot"))
            .all(|dot| dot.get("aria-hidden").map(String::as_str) == Some("true")),
        "the dots repeat the words and are hidden: {drawn}"
    );
}

#[test]
fn only_the_current_step_is_aria_current_step() {
    let drawn = html(|| lifecycle(on_trial()));
    let current: Vec<_> = tags(&drawn, "li")
        .into_iter()
        .filter(|item| item.contains_key("aria-current"))
        .collect();
    assert_eq!(1, current.len(), "{drawn}");
    assert_eq!(
        Some("step"),
        current[0].get("aria-current").map(String::as_str),
        "WAI-ARIA 1.2, aria-current=step: {drawn}"
    );
    assert_eq!(
        Some("is-current"),
        current[0].get("class").map(String::as_str)
    );
}

#[test]
fn a_revision_that_skips_approval_draws_the_step_as_not_needed() {
    let drawn = html(|| {
        lifecycle(Lifecycle {
            label: String::from("Where this revision stands"),
            steps: vec![
                Step::new(StepState::Done, "Saved", "by you, 09:51"),
                Step::new(StepState::Skipped, "Approval", "not needed"),
                Step::new(StepState::Current, "Apply", "yours, or any operator's"),
                Step::new(StepState::Skipped, "Confirm", "not needed"),
            ],
        })
    });
    assert_eq!(4, tags(&drawn, "li").len(), "never left out: {drawn}");
    assert!(has(&drawn, "li", &[("class", "is-skipped")]), "{drawn}");
    assert!(drawn.contains("Approval"), "{drawn}");
}

#[test]
fn a_pending_card_names_itself_and_holds_its_actions_lifecycle_and_body() {
    let drawn = html(|| {
        pending_card(
            PendingCard {
                title: String::from("Revision 9 is saved, not applied"),
                line: String::from("It changes 2 settings, both at the next start."),
                glyph: Glyph::PENCIL,
                tone: None,
                lifecycle: on_trial(),
            },
            Some(view! { <a href="/apply">"Apply revision 9"</a> }.into_any()),
            Some(view! { <p class="fui-small">"Check it first."</p> }.into_any()),
        )
    });
    assert!(
        has(
            &drawn,
            "section",
            &[
                ("class", "fui-pending"),
                ("aria-label", "Revision 9 is saved, not applied")
            ]
        ),
        "{drawn}"
    );
    assert!(
        !tags(&drawn, "section")[0].contains_key("role"),
        "a revision going its normal way is no live region: {drawn}"
    );
    assert!(has(&drawn, "div", &[("class", "fui-actions")]), "{drawn}");
    assert!(has(&drawn, "ol", &[("class", "fui-lifecycle")]), "{drawn}");
    assert!(drawn.contains("Check it first."), "{drawn}");
    let bare = html(|| {
        pending_card(
            PendingCard {
                title: String::from("Revision 10 was rolled back"),
                line: String::from("No one confirmed it in time."),
                glyph: Glyph::ROTATE_CCW,
                tone: Some(Tone::Bad),
                lifecycle: on_trial(),
            },
            None,
            None,
        )
    });
    assert!(
        has(
            &bare,
            "section",
            &[("class", "fui-pending fui-tone-bad"), ("role", "status")]
        ),
        "a toned card is a polite status region: {bare}"
    );
    assert!(
        !has(&bare, "div", &[("class", "fui-actions")]),
        "no empty action row: {bare}"
    );
}

/// The actions of a revision on trial with `left` of 120 seconds left.
fn trial(left: u32) -> String {
    page_html(move || {
        trial_actions(
            Countdown::new("trial", left, 120, "10:14:42"),
            Action::new(
                Some(Glyph::ROTATE_CCW),
                "Roll back now",
                Act::Link(String::from("/rolled-back")),
            ),
            Action::new(
                Some(Glyph::CHECK),
                "Confirm revision 10",
                Act::Post {
                    action: String::from("/confirm"),
                    fields: vec![(String::from("revision"), String::from("10"))],
                },
            ),
        )
    })
}

/// The text of the first polite status region in `html`.
fn status_text(html: &str) -> &str {
    let at = html.find(r#"role="status""#).expect("a status region");
    let rest = &html[at..];
    let open = rest.find('>').expect("its start tag ends") + 1;
    let close = rest.find("</span>").expect("it ends");
    &rest[open..close]
}

#[test]
fn the_server_draws_the_time_left_with_the_ring_and_both_actions() {
    let drawn = trial(102);
    assert!(drawn.contains("<strong>1:42</strong>"), "{drawn}");
    assert!(drawn.contains("left, until 10:14:42"), "{drawn}");
    assert!(
        has(&drawn, "circle", &[("class", "fui-countdown-left")]),
        "{drawn}"
    );
    assert!(
        has(&drawn, "svg", &[("aria-hidden", "true")]),
        "the ring is decorative: {drawn}"
    );
    let primary = drawn
        .find("fui-button fui-button-primary fui-button-small")
        .expect("Confirm is a small primary button");
    let secondary = drawn
        .find("fui-button fui-button-secondary fui-button-small")
        .expect("Roll back now is a small secondary button");
    assert!(
        secondary < primary,
        "Roll back now comes before Confirm: {drawn}"
    );
    assert!(
        has(
            &drawn,
            "form",
            &[("method", "post"), ("action", "/confirm")]
        ),
        "Confirm posts with no WebAssembly: {drawn}"
    );
    assert!(has(&drawn, "a", &[("href", "/rolled-back")]), "{drawn}");
}

#[test]
fn the_time_left_is_a_timer_and_a_polite_region_says_the_set_points() {
    let drawn = trial(102);
    assert!(
        has(
            &drawn,
            "span",
            &[
                ("class", "fui-countdown-words"),
                ("role", "timer"),
                ("aria-live", "off")
            ]
        ),
        "WAI-ARIA 1.2, timer: the time itself is not announced each second: {drawn}"
    );
    assert!(
        has(
            &drawn,
            "span",
            &[("class", "fui-sr-only"), ("role", "status")]
        ) && status_text(&drawn).trim().is_empty(),
        "the polite region starts silent and says only the set points: {drawn}"
    );
    let ended = trial(0);
    assert!(ended.contains("<strong>0:00</strong>"), "{ended}");
    assert!(ended.contains("ended at 10:14:42"), "{ended}");
}

#[test]
fn a_set_secret_offers_four_choices_and_shows_only_its_reference() {
    let drawn = html(|| {
        secret_field(
            SecretField::new("password", "Password", SecretState::Set(SET.into())),
            None,
        )
    });
    let radios: Vec<_> = tags(&drawn, "input")
        .into_iter()
        .filter(|input| input.get("type").map(String::as_str) == Some("radio"))
        .collect();
    assert_eq!(
        4,
        radios.len(),
        "Keep, Set a value, Read from a file, Clear: {drawn}"
    );
    assert!(
        radios
            .iter()
            .all(|radio| radio.get("name").map(String::as_str) == Some("password[mode]")),
        "one radio group: {drawn}"
    );
    assert!(
        has(
            &drawn,
            "input",
            &[("id", "password-mode-keep"), ("checked", "")]
        ),
        "Keep is chosen: {drawn}"
    );
    assert!(
        drawn.contains(&format!("set, read from <code>{SET}</code>")),
        "by reference: {drawn}"
    );
    assert!(
        has(
            &drawn,
            "fieldset",
            &[
                ("aria-labelledby", "password-label"),
                ("aria-describedby", "password-now")
            ]
        ),
        "the group is named by the secret and described by its state: {drawn}"
    );
    for word in ["Keep", "Set a value", "Read from a file", "Clear"] {
        assert!(drawn.contains(word), "{word}: {drawn}");
    }
}

#[test]
fn no_state_puts_a_value_in_the_markup_and_the_value_field_is_never_filled() {
    for choice in [
        SecretChoice::Keep,
        SecretChoice::Value,
        SecretChoice::File,
        SecretChoice::Clear,
    ] {
        let drawn = html(move || {
            secret_field(
                SecretField::new("password", "Password", SecretState::Set(SET.into()))
                    .choose(choice),
                None,
            )
        });
        let value = tags(&drawn, "input")
            .into_iter()
            .find(|input| input.get("type").map(String::as_str) == Some("password"))
            .expect("the value field");
        assert_eq!(
            Some("password[value]"),
            value.get("name").map(String::as_str)
        );
        assert_eq!(
            Some("off"),
            value.get("autocomplete").map(String::as_str),
            "{drawn}"
        );
        assert!(
            !value.contains_key("value"),
            "the page never fills a secret's value: {drawn}"
        );
        assert!(
            has(
                &drawn,
                "input",
                &[
                    ("id", &format!("password-mode-{}", choice.key())),
                    ("checked", "")
                ]
            ),
            "{drawn}"
        );
    }
}

#[test]
fn every_id_of_a_secret_field_is_unique_and_each_label_names_one_control() {
    let drawn = html(|| {
        secret_field(
            SecretField::new("password", "Password", SecretState::Set(SET.into())).refuse_value(),
            None,
        )
    });
    let mut ids: Vec<&str> = drawn
        .match_indices(" id=\"")
        .map(|(at, found)| {
            let rest = &drawn[at + found.len()..];
            &rest[..rest.find('"').expect("the id ends")]
        })
        .collect();
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(count, ids.len(), "HTML ids are unique: {drawn}");
    for label in tags(&drawn, "label") {
        let target = label.get("for").expect("every label names its control");
        assert!(
            ids.contains(&target.as_str()),
            "the label for {target} names a control: {drawn}"
        );
    }
}

#[test]
fn an_unset_secret_offers_leave_unset_and_no_clear() {
    let drawn = html(|| {
        secret_field(
            SecretField::new("client-secret", "Client secret", SecretState::Unset),
            None,
        )
    });
    assert!(drawn.contains("Leave unset"), "{drawn}");
    assert!(!drawn.contains(">Clear<"), "nothing to clear: {drawn}");
    assert!(
        !has(&drawn, "input", &[("id", "client-secret-mode-clear")]),
        "{drawn}"
    );
    assert!(
        has(&drawn, "span", &[("class", "fui-secret-now is-unset")]),
        "{drawn}"
    );
    assert!(drawn.contains("not set"), "{drawn}");
}

/// The value input is marked secret, so an answer form's page never carries
/// what was typed into it back to the browser (#499).
#[test]
fn a_secret_value_input_carries_its_secret_mark() {
    let drawn = html(|| {
        secret_field(
            SecretField::new("client-secret", "Client secret", SecretState::Unset),
            None,
        )
    });
    assert!(
        has(
            &drawn,
            "input",
            &[
                ("type", "hidden"),
                (
                    "name",
                    "ferroui_secret_636c69656e742d7365637265745b76616c75655d"
                ),
                ("value", "client-secret[value]"),
            ]
        ),
        "{drawn}"
    );
}

#[test]
fn a_file_choice_holds_its_path_and_clear_warns() {
    let file = html(|| {
        secret_field(
            SecretField::new("key", "Private key", SecretState::Set(SET.into()))
                .choose(SecretChoice::File)
                .path("/run/secrets/node-a-2026-11.pem"),
            None,
        )
    });
    assert!(
        has(
            &file,
            "input",
            &[
                ("id", "key-file"),
                ("name", "key[file]"),
                ("value", "/run/secrets/node-a-2026-11.pem")
            ]
        ),
        "{file}"
    );
    let clear = html(|| {
        secret_field(
            SecretField::new("password", "Password", SecretState::Set(SET.into()))
                .choose(SecretChoice::Clear),
            None,
        )
    });
    assert!(
        has(&clear, "p", &[("class", "fui-secret-clear")]),
        "{clear}"
    );
}

#[test]
fn an_unreadable_secret_says_so_in_the_tone_of_a_fault() {
    let drawn = html(|| {
        secret_field(
            SecretField::new("key", "Private key", SecretState::Unreadable(SET.into())),
            None,
        )
    });
    assert!(
        has(&drawn, "span", &[("class", "fui-secret-now is-broken")]),
        "{drawn}"
    );
    assert!(
        drawn.contains(&format!("set, but <code>{SET}</code> cannot be read")),
        "{drawn}"
    );
}

#[test]
fn a_value_is_refused_where_no_secrets_directory_is_set() {
    let drawn = html(|| {
        secret_field(
            SecretField::new("password", "Password", SecretState::Set(SET.into())).refuse_value(),
            None,
        )
    });
    assert!(
        has(
            &drawn,
            "input",
            &[("id", "password-mode-value"), ("checked", "")]
        ),
        "{drawn}"
    );
    assert!(
        has(
            &drawn,
            "input",
            &[
                ("name", "password[value]"),
                ("aria-invalid", "true"),
                (
                    "aria-describedby",
                    "password-value-help password-value-error"
                )
            ]
        ),
        "{drawn}"
    );
    assert!(
        has(
            &drawn,
            "p",
            &[
                ("id", "password-value-error"),
                ("class", "fui-help fui-error")
            ]
        ),
        "{drawn}"
    );
    assert!(drawn.contains("No secrets directory is set"), "{drawn}");
    assert!(
        !has(
            &drawn,
            "input",
            &[("id", "password-file"), ("aria-invalid", "true")]
        ),
        "the file choice is not refused: {drawn}"
    );
}

#[test]
fn the_read_only_state_names_the_reference_alone() {
    let set = html(|| secret_state(&SecretState::Set(String::from("/run/secrets/db-url"))));
    assert!(has(&set, "span", &[("class", "fui-secret-now")]), "{set}");
    assert!(
        set.contains("set, read from <code>/run/secrets/db-url</code>"),
        "{set}"
    );
    let redacted = html(|| secret_state(&SecretState::Redacted));
    assert!(redacted.contains("set, not shown"), "{redacted}");
    assert!(!redacted.contains("<code>"), "{redacted}");
    let escaped = html(|| secret_state(&SecretState::Set(String::from("/a<b>"))));
    assert!(escaped.contains("<code>/a&lt;b&gt;</code>"), "{escaped}");
}

#[test]
fn the_lock_tag_names_its_owner_in_its_tooltip_and_accessible_name() {
    let drawn = html(|| {
        Tag::new("Provisioned")
            .locked("Provisioned from mappings/observation.yml")
            .view()
    });
    assert!(
        has(
            &drawn,
            "span",
            &[
                ("class", "fui-tag fui-tag-locked"),
                ("title", "Provisioned from mappings/observation.yml")
            ]
        ),
        "{drawn}"
    );
    assert!(
        drawn.contains(
            r#"<span class="fui-sr-only"> (Provisioned from mappings/observation.yml)</span></span>"#
        ) && drawn.contains("</svg>Provisioned<"),
        "the word, then the owner for assistive technology: {drawn}"
    );
    assert!(
        has(&drawn, "svg", &[("aria-hidden", "true")]),
        "the lock never stands alone: {drawn}"
    );
}
