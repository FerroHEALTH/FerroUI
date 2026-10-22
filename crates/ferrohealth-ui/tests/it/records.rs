// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The patient-record components rendered as a console's server draws
//! them: the record compare, the timeline, the masked identifier and the
//! work-list. Every value here is invented and follows no national scheme.

use ferrohealth_ui::table::view::{Body, Column, DataTable, Role, Sorting, cell, data_table};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::compare::{
    Compare, CompareField, CompareRecord, Verdict, compare, compare_pair, compare_stacked,
};
use ferrohealth_ui::view::timeline::{Event, timeline};
use ferrohealth_ui::widget::identifier::{
    Domain, Identifier, IdentifierWords, Masked, Reveal, identifier, identifier_list,
    identifier_static,
};
use ferrohealth_ui::widget::worklist::{WorkList, WorkRow, row, worklist, worklist_foot};
use leptos::prelude::*;

use crate::markup::{has, tags};

/// The HTML `view` renders to, under a reactive owner as a server render
/// runs.
fn html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new();
    owner.with(|| view().to_html())
}

/// A record titled `title` with `values`, each a plain text or none.
fn record(key: &str, title: &str, values: &[Option<&'static str>]) -> CompareRecord {
    CompareRecord {
        key: key.to_owned(),
        title: title.to_owned(),
        sub: None,
        values: values
            .iter()
            .map(|value| value.map(|value| ViewFn::from(move || value.into_any())))
            .collect(),
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
    html(|| {
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
            compare_stacked(table, 1)
        } else {
            compare(table)
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
    let table = html(|| {
        let fields = fields();
        let leading = record("a", "Source A", &[Some("x"), Some("y"), Some("z")]);
        let others = [record("b", "Source B", &[Some("x"), Some("y"), Some("z")])];
        compare(Compare {
            id: "agree",
            caption: "Two sources",
            fields: &fields,
            leading: &leading,
            others: &others,
            verdict: &|_, _| Verdict::Agree,
            differ_first: true,
        })
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
        stacked.contains(r#"<span class="fui-compare-value"><!></span><span title="absent""#),
        "{stacked}"
    );
    let pair = html(|| {
        let fields = fields();
        let leading = record("a", "Source A", &[Some("x"), None, Some("z")]);
        let others = [record("b", "Source B", &[Some("x"), Some("y"), Some("w")])];
        compare_pair(
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
    let table = html(|| {
        let fields = vec![CompareField {
            key: String::from("k"),
            label: String::from("<b>field</b>"),
        }];
        let leading = record("a", "<i>lead</i>", &[Some("<script>")]);
        let others = [record("b", "B", &[None])];
        compare(Compare {
            id: "esc",
            caption: "<caption>",
            fields: &fields,
            leading: &leading,
            others: &others,
            verdict: &|_, _| Verdict::Absent,
            differ_first: false,
        })
    });
    assert!(
        !table.contains("<b>field") && !table.contains("<i>lead") && !table.contains("<script>"),
        "{table}"
    );
    assert!(table.contains("&lt;b&gt;field"), "{table}");
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
    let list = html(|| {
        timeline(
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
    let list = html(|| {
        timeline(
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
    let list = html(|| {
        let mut full = event("full", "2026-01-01T08:00:00Z", "merged");
        full.code = Some(String::from("merge"));
        full.actor = Some(String::from("Steward"));
        full.reason = Some(String::from("Same person"));
        full.identifiers = vec![Masked::new(Domain::new("Local"), "AB-0000-7", 3)];
        full.body = Some(ViewFn::from(|| view! { <p>"Diff"</p> }.into_any()));
        timeline("History", &[full])
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
    let list = html(|| timeline("History", &events));
    assert_eq!(50, tags(&list, "li").len());
}

/// The words of a masked identifier.
fn words() -> IdentifierWords {
    IdentifierWords::of_page()
}

/// The masked identifier drawn by the server in `state`.
fn masked_identifier(state: Reveal) -> String {
    html(|| {
        identifier(
            Identifier {
                id: String::from("id-1"),
                masked: Masked::new(
                    Domain::new("Local MRN").system("urn:example:mrn"),
                    "SYN-7Q2-XYZ",
                    3,
                ),
                show_domain: true,
                state: RwSignal::new(state),
                fallback: String::from("/reveal/1"),
                words: words(),
            },
            Callback::new(|()| {}),
        )
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
    // NOTE: leptos-ui.md §15.8: before hydration the reveal is a link to the console's page.
    assert!(has(&drawn, "a", &[("href", "/reveal/1")]), "{drawn}");
    assert!(
        has(&drawn, "button", &[("aria-pressed", "false")]),
        "{drawn}"
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
        3, disabled,
        "the busy twin, Hide and Copy are disabled while masked: {drawn}"
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
        has(&busy, "button", &[("aria-busy", "true"), ("disabled", "")]),
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
fn the_listing_form_has_no_control_and_a_list_keeps_its_order() {
    let one = html(|| identifier_static(&Masked::new(Domain::new("Local"), "SYN-0042", 3), false));
    assert!(!one.contains("<button") && !one.contains("<a "), "{one}");
    assert!(
        one.contains(r#"<span class="fui-sr-only">Local </span>"#),
        "{one}"
    );
    let list = html(|| {
        identifier_list(&[
            Masked::new(Domain::new("Second"), "SYN-0002", 3),
            Masked::new(Domain::new("First"), "SYN-0001", 3),
        ])
    });
    assert!(list.find("Second") < list.find("First"), "{list}");
}

/// A work-list of three rows inside its island part.
fn queue() -> String {
    html(|| {
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
                    cells: view! { <td></td> {cell(name, key)} }.into_any(),
                })
            })
            .collect_view()
            .into_any();
        let table = data_table(DataTable {
            caption: "Queue",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(rows),
        })
        .into_any();
        let list = worklist(
            WorkList {
                focused: RwSignal::new(None),
                on_open: Callback::new(|_: String| {}),
            },
            table,
        );
        view! {
            {list}
            {worklist_foot(None, None)}
        }
        .into_any()
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
