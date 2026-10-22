// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The in-place filter rendered to HTML on the host: the count in sight,
//! and the status region assistive technology hears, apart.

use ferrohealth_ui::widget::filter::{SETTLE, table_filter};
use leptos::prelude::*;

use crate::markup::has;
use crate::render::html;

#[test]
fn the_filter_shows_its_count_and_says_it_from_a_status_region_of_its_own() {
    let shown = html(|| {
        let words = RwSignal::new(String::new());
        let count = Signal::stored(String::from("12 of 12 files on this page."));
        table_filter("files", "Filter the files", words, count).into_any()
    });
    assert!(
        has(
            &shown,
            "p",
            &[("class", "fui-muted"), ("aria-hidden", "true")]
        ),
        "the count in sight is not read twice: {shown}"
    );
    assert!(
        has(&shown, "p", &[("role", "status"), ("class", "fui-sr-only")]),
        "{shown}"
    );
    assert_eq!(
        2,
        shown.matches("12 of 12 files on this page.").count(),
        "the server draws both with the count: {shown}"
    );
    assert!(
        SETTLE.as_millis() >= 500,
        "a pause between two keys is shorter than the wait"
    );
}
