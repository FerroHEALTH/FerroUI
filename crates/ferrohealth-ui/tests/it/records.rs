// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The patient-record components rendered as a console's server draws
//! them: the record compare, the timeline, the masked identifier and the
//! work-list. Every value here is invented and follows no national scheme.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::table::view::{Body, Column, DataTable, Role, Sorting, cell, data_table};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::compare::{
    Compare, CompareField, CompareRecord, Verdict, compare, compare_pair, compare_stacked,
};
use ferrohealth_ui::view::identifier::{
    Domain, Identifier, IdentifierWords, Masked, Reveal, identifier, identifier_list,
    identifier_static,
};
use ferrohealth_ui::view::timeline::{Event, Window, timeline, timeline_window};
use ferrohealth_ui::view::worklist::{WorkRow, row, worklist, worklist_foot};

use crate::markup::{has, tags};
use crate::render::{en, html};

/// A record titled `title` with `values`, each a plain text or none.
fn record(key: &str, title: &str, values: &[Option<&'static str>]) -> CompareRecord {
    CompareRecord {
        key: key.to_owned(),
        title: title.to_owned(),
        sub: None,
        values: values.iter().map(|value| value.map(Html::text)).collect(),
    }
}

/// The fields of the compares: a name, a date and a code.
fn fields() -> Vec<CompareField> {
    ["name", "born", "code"]
        .into_iter()
        .map(|key| CompareField {
            key: key.to_owned(),
            label: format!("Field {key}"),
        })
        .collect()
}

/// The verdicts: the code differs in the first other record, the date is
/// absent in the second, the rest agree.
fn verdict(field: &str, other: usize) -> Verdict {
    match (field, other) {
        ("code", 0) => Verdict::Differ,
        ("born", 1) => Verdict::Absent,
        _ => Verdict::Agree,
    }
}

/// The compare of a leading record and `count` others.
fn drawn(count: usize, differ_first: bool, stacked: bool) -> String {
    html({
        let fields = fields();
        let leading = record(
            "lead",
            "Source zero",
            &[Some("Alpha"), Some("2000-01-01"), Some("SYN-1")],
        );
        let others: Vec<CompareRecord> = (1..=count)
            .map(|index| {
                record(
                    &format!("r{index}"),
                    &format!("Source {index}"),
                    &[
                        Some("Alpha"),
                        (index != 2).then_some("2000-01-01"),
                        Some("SYN-2"),
                    ],
                )
            })
            .collect();
        let table = Compare {
            id: "cmp",
            caption: "Records of one person",
            fields: &fields,
            leading: &leading,
            others: &others,
            verdict: &verdict,
            differ_first,
        };
        if stacked {
            compare_stacked(&en(), table, 1)
        } else {
            compare(&en(), table)
        }
    })
}

#[test]
fn a_compare_draws_two_to_six_records_with_row_headers_and_a_caption() {
    for count in 1..=5 {
        let table = drawn(count, false, false);
        assert_eq!(
            count + 2,
            tags(&table, "th")
                .iter()
                .filter(|th| th.get("scope").is_some_and(|scope| scope == "col"))
                .count(),
            "{table}"
        );
        assert_eq!(
            3,
            tags(&table, "th")
                .iter()
                .filter(|th| th.get("scope").is_some_and(|scope| scope == "row"))
                .count()
        );
        assert!(
            table.contains("<caption>Records of one person</caption>"),
            "{table}"
        );
        assert!(has(
            &table,
            "div",
            &[("role", "region"), ("aria-label", "Records of one person")]
        ));
        assert!(!table.contains("style="), "{table}");
    }
}

#[test]
fn every_mark_is_a_word_and_a_glyph_with_a_title() {
    let table = drawn(2, false, false);
    // NOTE: WCAG 2.2 SC 1.4.1: each verdict is a word, never colour alone.
    assert!(
        has(
            &table,
            "span",
            &[
                ("class", "fui-compare-mark fui-compare-differ"),
                ("title", "differs")
            ]
        ),
        "{table}"
    );
    assert!(
        has(
            &table,
            "span",
            &[
                ("class", "fui-compare-mark fui-compare-absent"),
                ("title", "absent")
            ]
        ),
        "{table}"
    );
    assert!(
        has(
            &table,
            "span",
            &[
                ("class", "fui-compare-mark fui-compare-agree"),
                ("title", "agrees")
            ]
        ),
        "{table}"
    );
    assert!(
        table.contains(r#"<span class="fui-sr-only">agrees</span>"#),
        "{table}"
    );
    assert!(table.contains(">differs</span>"), "{table}");
}

#[test]
fn the_differing_rows_come_first_when_asked() {
    let table = drawn(2, true, false);
    let code = table.find(">Field code<").expect("the code row");
    let born = table.find(">Field born<").expect("the date row");
    let name = table.find(">Field name<").expect("the name row");
    assert!(
        code < born && born < name,
        "differ, then absent, then agree"
    );
    let plain = drawn(2, false, false);
    assert!(
        plain.find(">Field name<") < plain.find(">Field code<"),
        "the given order"
    );
    assert!(table.contains("fui-compare-row-differ"), "{table}");
}

#[test]
fn a_compare_whose_fields_all_agree_says_so_in_its_caption() {
    let table = html({
        let fields = fields();
        let leading = record("a", "Source A", &[Some("x"), Some("y"), Some("z")]);
        let others = [record("b", "Source B", &[Some("x"), Some("y"), Some("z")])];
        compare(
            &en(),
            Compare {
                id: "agree",
                caption: "Two sources",
                fields: &fields,
                leading: &leading,
                others: &others,
                verdict: &|_, _| Verdict::Agree,
                differ_first: true,
            },
        )
    });
    assert!(
        table.contains("<caption>Two sources: every field agrees</caption>"),
        "{table}"
    );
}

#[test]
fn the_columns_have_fixed_widths_so_a_revealed_value_moves_nothing() {
    let table = drawn(3, false, false);
    assert_eq!(
        1,
        tags(&table, "col")
            .iter()
            .filter(|col| col
                .get("class")
                .is_some_and(|class| class == "fui-compare-col-field"))
            .count()
    );
    assert_eq!(
        4,
        tags(&table, "col")
            .iter()
            .filter(|col| col
                .get("class")
                .is_some_and(|class| class == "fui-compare-col"))
            .count()
    );
    assert!(table.contains("fui-compare-scroll"), "{table}");
}

#[test]
fn the_stacked_form_is_a_radio_choice_with_every_mark_at_its_line_end() {
    let stacked = drawn(3, true, true);
    let radios = tags(&stacked, "input");
    assert_eq!(3, radios.len(), "{stacked}");
    assert!(
        radios
            .iter()
            .all(|radio| radio.get("type").is_some_and(|kind| kind == "radio"))
    );
    assert_eq!(Some(&String::new()), radios[1].get("checked"), "{stacked}");
    assert!(stacked.contains("fui-compare-alone"), "{stacked}");
    assert!(
        stacked.contains("Marked against Source zero, the leading record."),
        "{stacked}"
    );
    // The absent value keeps its empty place before its mark.
    assert!(
        stacked.contains(r#"<span class="fui-compare-value"></span><span title="absent""#),
        "{stacked}"
    );
    let pair = html({
        let fields = fields();
        let leading = record("a", "Source A", &[Some("x"), None, Some("z")]);
        let others = [record("b", "Source B", &[Some("x"), Some("y"), Some("w")])];
        compare_pair(
            &en(),
            Compare {
                id: "pair",
                caption: "Pair",
                fields: &fields,
                leading: &leading,
                others: &others,
                verdict: &verdict,
                differ_first: false,
            },
            0,
        )
    });
    assert!(pair.contains("fui-compare-wide"), "{pair}");
    assert!(!pair.contains("fui-compare-alone"), "{pair}");
}

#[test]
fn a_compare_escapes_what_it_is_given() {
    let table = html({
        let fields = vec![CompareField {
            key: String::from("k"),
            label: String::from("<b>field</b>"),
        }];
        let leading = record("a", "<i>lead</i>", &[Some("<script>")]);
        let others = [record("b", "B", &[None])];
        compare(
            &en(),
            Compare {
                id: "esc",
                caption: "<caption>",
                fields: &fields,
                leading: &leading,
                others: &others,
                verdict: &|_, _| Verdict::Absent,
                differ_first: false,
            },
        )
    });
    assert!(
        !table.contains("<b>field") && !table.contains("<i>lead") && !table.contains("<script>"),
        "{table}"
    );
    assert!(table.contains("&#60;b&#62;field"), "{table}");
}

/// The stacked compare of a leading record and `count` others, `shown`
/// read first.
fn stacked_of(count: usize, shown: usize) -> String {
    html({
        let fields = fields();
        let leading = record("lead", "Source zero", &[Some("Alpha"), None, Some("SYN-1")]);
        let others: Vec<CompareRecord> = (1..=count)
            .map(|index| {
                record(
                    &format!("r{index}"),
                    &format!("Source {index}"),
                    &[Some("Alpha"), None, Some("SYN-2")],
                )
            })
            .collect();
        compare_stacked(
            &en(),
            Compare {
                id: "many",
                caption: "Records of one person",
                fields: &fields,
                leading: &leading,
                others: &others,
                verdict: &verdict,
                differ_first: false,
            },
            shown,
        )
    })
}

#[test]
fn the_stacked_form_offers_every_record_past_five_as_disclosures() {
    // NOTE: WCAG 2.2 SC 1.4.10: below 64 rem the stacked form is the only form, so it
    // reaches every record, each a native disclosure that opens with no script.
    let many = stacked_of(7, 2);
    let disclosures = tags(&many, "details");
    assert_eq!(7, disclosures.len(), "one per other record: {many}");
    assert!(
        disclosures
            .iter()
            .all(|each| each.get("name").is_some_and(|name| name == "many-read")),
        "one at a time: {many}"
    );
    let open: Vec<usize> = disclosures
        .iter()
        .enumerate()
        .filter(|(_, each)| each.contains_key("open"))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(vec![2], open, "the record shown first opens: {many}");
    assert!(
        many.contains("Source 7"),
        "the seventh record is offered: {many}"
    );
    assert!(!many.contains(r#"type="radio""#), "{many}");
}

#[test]
fn a_shown_record_past_the_end_reads_the_last_one() {
    let few = stacked_of(3, 9);
    let radios = tags(&few, "input");
    assert_eq!(
        Some(&String::new()),
        radios.get(2).and_then(|radio| radio.get("checked")),
        "the last record is chosen: {few}"
    );
    let many = stacked_of(7, 40);
    let disclosures = tags(&many, "details");
    assert!(
        disclosures
            .get(6)
            .is_some_and(|last| last.contains_key("open")),
        "the last record opens: {many}"
    );
}

#[test]
fn a_compare_with_no_other_record_says_so_and_never_that_all_agree() {
    let table = drawn(0, false, false);
    assert!(
        table.contains("Records of one person: no other record to compare"),
        "{table}"
    );
    assert!(!table.contains("every field agrees"), "{table}");
    let stacked = stacked_of(0, 0);
    assert!(
        stacked.contains("Records of one person: no other record to compare"),
        "{stacked}"
    );
    assert!(!stacked.contains("<fieldset"), "no empty choice: {stacked}");
}

/// An event at `at` of `kind`.
fn event(key: &str, at: &str, kind: &str) -> Event {
    Event {
        key: key.to_owned(),
        at: at.to_owned(),
        kind: kind.to_owned(),
        code: None,
        tone: Tone::Info,
        actor: None,
        title: format!("Event {key}"),
        identifiers: Vec::new(),
        reason: None,
        body: None,
    }
}

#[test]
fn a_timeline_is_a_labelled_list_newest_first_each_time_a_time_element() {
    let list = html({
        timeline(
            &en(),
            "History",
            &[
                event("old", "2026-01-01T08:00:00Z", "created"),
                event("new", "2026-03-01T08:00:00Z", "merged"),
                event("mid", "2026-02-01T08:00:00Z", "linked automatically"),
            ],
        )
    });
    assert!(
        has(
            &list,
            "ol",
            &[("class", "fui-timeline"), ("aria-label", "History")]
        ),
        "{list}"
    );
    let keys: Vec<String> = tags(&list, "li")
        .iter()
        .filter_map(|li| li.get("data-key").cloned())
        .collect();
    assert_eq!(vec!["new", "mid", "old"], keys, "newest first");
    assert_eq!(3, tags(&list, "time").len(), "{list}");
    assert!(has(
        &list,
        "span",
        &[("class", "fui-timeline-dot"), ("aria-hidden", "true")]
    ));
    assert!(
        list.contains(">Merged<") && list.contains(">Linked automatically<"),
        "{list}"
    );
    assert!(!list.contains("style="), "{list}");
}

#[test]
fn an_event_with_no_actor_and_no_body_draws_neither() {
    let list = html({
        timeline(
            &en(),
            "History",
            &[event("one", "2026-01-01T08:00:00Z", "created")],
        )
    });
    assert!(!list.contains("fui-timeline-body"), "{list}");
    assert!(!list.contains("<details"), "{list}");
    assert_eq!(1, tags(&list, "li").len());
}

#[test]
fn an_event_draws_its_code_actor_identifiers_reason_and_a_body_that_opens() {
    let list = html({
        let mut full = event("full", "2026-01-01T08:00:00Z", "merged");
        full.code = Some(String::from("merge"));
        full.actor = Some(String::from("Steward"));
        full.reason = Some(String::from("Same person"));
        full.identifiers = vec![Masked::new(Domain::new("Local"), "AB-0000-7", 3)];
        full.body = Some(Html::raw(String::from("<p>Diff</p>")));
        timeline(&en(), "History", &[full])
    });
    assert!(
        list.contains(r#"<span class="fui-timeline-code">merge</span>"#),
        "{list}"
    );
    assert!(list.contains("Steward"), "{list}");
    assert!(list.contains(">Reason</span>"), "{list}");
    assert!(list.contains("<details"), "{list}");
    assert!(list.contains("fui-identifier-static"), "{list}");
}

#[test]
fn fifty_events_draw_whole_with_no_cap() {
    let events: Vec<Event> = (0..50)
        .map(|index| {
            event(
                &format!("e{index:02}"),
                &format!("2026-01-01T08:{index:02}:00Z"),
                "version",
            )
        })
        .collect();
    let list = html(timeline(&en(), "History", &events));
    assert_eq!(50, tags(&list, "li").len());
}

#[test]
fn events_sort_by_the_moment_they_name_whatever_their_offsets() {
    // 10:00+02:00 is 08:00 UTC, before 09:30Z, though its text sorts after it.
    let list = html({
        timeline(
            &en(),
            "History",
            &[
                event("east", "2026-01-01T10:00:00+02:00", "linked"),
                event("utc", "2026-01-01T09:30:00Z", "merged"),
                event("west", "2026-01-01T05:00:00-05:00", "split"),
                event("odd", "yesterday", "noted"),
            ],
        )
    });
    let order: Vec<String> = tags(&list, "li")
        .iter()
        .filter_map(|li| li.get("data-key").cloned())
        .collect();
    assert_eq!(
        vec!["west", "utc", "east", "odd"],
        order,
        "newest first by the instant, no date-time last: {list}"
    );
}

#[test]
fn a_windowed_timeline_shows_its_newest_and_links_to_the_older() {
    let events: Vec<Event> = (0..12)
        .map(|index| {
            event(
                &format!("e{index:02}"),
                &format!("2026-01-01T08:{index:02}:00Z"),
                "version",
            )
        })
        .collect();
    let list = html({
        timeline_window(
            &en(),
            "History",
            &events,
            Window {
                limit: 5,
                total: Some(3120),
                older: "/history?events=all",
            },
        )
    });
    let keys: Vec<String> = tags(&list, "li")
        .iter()
        .filter_map(|li| li.get("data-key").cloned())
        .collect();
    assert_eq!(vec!["e11", "e10", "e09", "e08", "e07"], keys, "{list}");
    assert!(list.contains("The newest 5 of 3"), "{list}");
    assert!(
        has(&list, "a", &[("href", "/history?events=all")]),
        "{list}"
    );
    let whole = html({
        timeline_window(
            &en(),
            "History",
            &events,
            Window {
                limit: 20,
                total: None,
                older: "/history",
            },
        )
    });
    assert_eq!(12, tags(&whole, "li").len(), "{whole}");
    assert!(
        !whole.contains("fui-timeline-more"),
        "nothing older: {whole}"
    );
}

/// The words of a masked identifier.
fn words() -> IdentifierWords {
    IdentifierWords::of(&en())
}

/// The masked identifier drawn by the server in `state`.
fn masked_identifier(state: Reveal) -> String {
    html({
        identifier(Identifier {
            id: String::from("id-1"),
            masked: Masked::new(
                Domain::new("Local MRN").system("urn:example:mrn"),
                "SYN-7Q2-XYZ",
                3,
            ),
            show_domain: true,
            state,
            fallback: String::from("/reveal/1"),
            words: words(),
        })
    })
}

#[test]
fn the_server_draws_the_identifier_masked_with_its_domain_and_no_value() {
    let drawn = masked_identifier(Reveal::Masked);
    assert!(drawn.contains("masked, ends in XYZ"), "{drawn}");
    assert!(
        !drawn.contains("SYN-7Q2"),
        "the whole value never reaches the page: {drawn}"
    );
    assert!(
        has(
            &drawn,
            "span",
            &[("class", "fui-tag"), ("title", "urn:example:mrn")]
        ),
        "{drawn}"
    );
    assert!(
        has(&drawn, "a", &[("href", "/reveal/1")]),
        "the reveal is a link to the console's page: {drawn}"
    );
    assert!(!drawn.contains("style="), "{drawn}");
}

#[test]
fn the_copy_button_waits_in_an_inert_face_while_masked() {
    let drawn = masked_identifier(Reveal::Masked);
    let disabled = tags(&drawn, "button")
        .into_iter()
        .filter(|button| button.contains_key("disabled"))
        .count();
    assert_eq!(
        2, disabled,
        "Hide and Copy are disabled while masked: {drawn}"
    );
    assert!(drawn.contains(">Copy</button>"), "{drawn}");
    let faces: Vec<_> = tags(&drawn, "span")
        .into_iter()
        .filter(|span| {
            span.get("class")
                .is_some_and(|class| class.starts_with("fui-identifier-face"))
        })
        .collect();
    assert_eq!(2, faces.len(), "{drawn}");
    assert!(
        faces[1].contains_key("inert"),
        "the revealed face is inert: {drawn}"
    );
    assert!(!faces[0].contains_key("inert"), "{drawn}");
}

#[test]
fn a_reveal_under_way_is_busy_and_a_refusal_is_a_line_in_words() {
    let busy = masked_identifier(Reveal::Busy);
    assert!(
        has(
            &busy,
            "button",
            &[("aria-busy", "true"), ("aria-disabled", "true")]
        ),
        "{busy}"
    );
    let refused = masked_identifier(Reveal::Refused(String::from("the limit is reached")));
    assert!(
        refused.contains("Not revealed: the limit is reached"),
        "{refused}"
    );
    assert!(has(&refused, "span", &[("role", "status")]), "{refused}");
}

#[test]
fn a_long_revealed_value_breaks_inside_its_row() {
    // NOTE: WCAG 2.2 SC 1.4.10: a long value breaks inside the slot, never widening the page.
    let rule = ferrohealth_ui::style::STYLESHEET
        .split("\n  .fui-identifier-shown > .fui-identifier-value {\n")
        .nth(1)
        .and_then(|rest| rest.split("\n  }").next())
        .expect("the revealed value's rule");
    assert!(rule.contains("white-space: normal;"), "{rule}");
    assert!(rule.contains("overflow-wrap: anywhere;"), "{rule}");
    assert!(
        ferrohealth_ui::style::STYLESHEET
            .contains("  .fui-identifier-slot {\n    display: inline-grid;\n    grid-template-columns: minmax(0, auto);\n    max-width: 100%;"),
        "the slot never grows past its row"
    );
}

#[test]
fn the_listing_form_has_no_control_and_a_list_keeps_its_order() {
    let one = html(identifier_static(
        &en(),
        &Masked::new(Domain::new("Local"), "SYN-0042", 3),
        false,
    ));
    assert!(!one.contains("<button") && !one.contains("<a "), "{one}");
    assert!(
        one.contains(r#"<span class="fui-sr-only">Local </span>"#),
        "{one}"
    );
    let list = html({
        identifier_list(
            &en(),
            &[
                Masked::new(Domain::new("Second"), "SYN-0002", 3),
                Masked::new(Domain::new("First"), "SYN-0001", 3),
            ],
        )
    });
    assert!(list.find("Second") < list.find("First"), "{list}");
}

/// A work-list of three rows inside its island part.
fn queue() -> String {
    html({
        let columns = [
            Column::text("select", "Select every row").role(Role::Check),
            Column::text("name", "Name").role(Role::Title),
        ];
        let [_, name] = &columns;
        let rows = ["a", "b", "c"]
            .into_iter()
            .enumerate()
            .map(|(index, key)| {
                row(WorkRow {
                    key,
                    focused: index == 0,
                    class: None,
                    cells: Html::raw(String::from("<td></td>")).then(cell(name, Html::text(key))),
                })
            })
            .collect::<Html>();
        let table = data_table(DataTable {
            caption: "Queue",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(rows),
        });
        worklist(table).then(worklist_foot(None, None))
    })
}

#[test]
fn the_server_draws_the_first_row_focused_with_one_tab_stop() {
    let list = queue();
    let rows: Vec<_> = tags(&list, "tr")
        .into_iter()
        .filter(|tr| tr.contains_key("data-key"))
        .collect();
    assert_eq!(3, rows.len(), "{list}");
    // NOTE: WAI-ARIA Authoring Practices 1.2, the grid pattern: a roving tabindex.
    assert_eq!(Some(&String::from("0")), rows[0].get("tabindex"));
    assert_eq!(Some(&String::from("true")), rows[0].get("aria-current"));
    assert!(
        rows[1..]
            .iter()
            .all(|row| row.get("tabindex").is_some_and(|tab| tab == "-1")
                && !row.contains_key("aria-current"))
    );
    assert!(list.contains(r#"class="fui-worklist-foot""#), "{list}");
}

#[test]
fn the_select_all_waits_hidden_until_a_work_list_shows_it() {
    let list = queue();
    assert!(
        has(
            &list,
            "input",
            &[
                ("class", "fui-select-all"),
                ("aria-label", "Select every row"),
                ("hidden", "")
            ]
        ),
        "{list}"
    );
}
