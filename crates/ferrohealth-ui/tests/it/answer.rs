// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The marker of a form answered in place (#546): the attributes a console
//! writes in its `<form>` tag, and the module they ask the loader for.

use ferrohealth_ui::script::{HOOK, module};
use ferrohealth_ui::view::answer::{ANSWER_JOB, Answer};
use ferrohealth_ui::view::shortcuts::KEYS_JOB;

use crate::markup::{has, tags};

/// A post form carrying `answer`'s attributes, as a console writes it.
fn form(answer: &Answer) -> String {
    format!(
        "<form method=\"post\" action=\"/check\"{}></form>",
        answer.attributes()
    )
}

#[test]
fn the_marker_asks_for_the_answer_module_and_names_its_region() {
    let html = form(&Answer::new("check-answer"));
    assert!(
        has(
            &html,
            "form",
            &[
                ("method", "post"),
                (HOOK, ANSWER_JOB),
                ("data-fui-answer", "check-answer")
            ]
        ),
        "{html}"
    );
    assert!(
        !tags(&html, "form")[0].contains_key("data-fui-settle"),
        "a form with no rest posts on submit alone: {html}"
    );
    let shipped = module(&format!("{ANSWER_JOB}.js"));
    assert!(
        shipped.is_some_and(|module| module.text.contains("export function enhance(form)")),
        "the job names a module the library ships"
    );
}

#[test]
fn a_rest_a_second_region_and_another_job_ride_in_the_same_attributes() {
    let html = form(
        &Answer::new("check-answer")
            .with_region("check-count")
            .with_settle(300)
            .with_job(KEYS_JOB),
    );
    assert!(
        has(
            &html,
            "form",
            &[
                (HOOK, "answer keys"),
                ("data-fui-answer", "check-answer check-count"),
                ("data-fui-settle", "300")
            ]
        ),
        "{html}"
    );
    let none = form(&Answer::new("check-answer").with_settle(0));
    assert!(
        !none.contains("data-fui-settle"),
        "a rest of 0 posts on submit alone: {none}"
    );
}

#[test]
fn a_region_id_is_escaped_into_its_attribute() {
    let html = form(&Answer::new("a\"b<c"));
    assert!(html.contains("data-fui-answer=\"a&#34;b&#60;c\""), "{html}");
    assert_eq!(1, tags(&html, "form").len(), "{html}");
}
