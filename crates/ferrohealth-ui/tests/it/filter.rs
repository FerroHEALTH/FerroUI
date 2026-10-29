// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The in-place filter rendered to HTML on the host: the count in sight,
//! and the status region assistive technology hears, apart.

use ferrohealth_ui::i18n::Phrase;
use ferrohealth_ui::view::filter::{SETTLE, TableFilter, table_filter};

use crate::markup::has;
use crate::render::html;

#[test]
fn the_filter_shows_its_count_and_says_it_from_a_status_region_of_its_own() {
    let phrase = format!(
        "{} of {} files on this page.",
        Phrase::open_mark("kept"),
        Phrase::open_mark("total")
    );
    let shown = html(table_filter(TableFilter {
        id: "files",
        label: String::from("Filter the files"),
        rows: String::from("file-rows"),
        count: String::from("12 of 12 files on this page."),
        phrase: Phrase::new(phrase),
        words: String::new(),
    }));
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

#[test]
fn the_filter_names_the_rows_it_narrows_and_carries_its_open_sentence() {
    let phrase = format!(
        "{} of {} kept.",
        Phrase::open_mark("kept"),
        Phrase::open_mark("total")
    );
    let shown = html(table_filter(TableFilter {
        id: "nodes",
        label: String::from("Filter"),
        rows: String::from("node-rows"),
        count: String::from("3 of 3 kept."),
        phrase: Phrase::new(phrase.clone()),
        words: String::from("obs"),
    }));
    assert!(
        has(
            &shown,
            "div",
            &[("data-fui", "filter"), ("data-fui-rows", "node-rows")]
        ),
        "the module finds the rows by their holder's id: {shown}"
    );
    assert!(
        shown.contains(&phrase),
        "the open sentence rides along: {shown}"
    );
    assert!(
        has(&shown, "input", &[("type", "search"), ("value", "obs")]),
        "the words a reload kept stay in the box: {shown}"
    );
}
