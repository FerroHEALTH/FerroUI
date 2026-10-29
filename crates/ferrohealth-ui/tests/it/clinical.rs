// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The clinical record parts rendered to HTML: the subject line (E1), the
//! version history with At a time (E2), the clinical reading (E3), the
//! condition groups (E6), the results pane (E7) and the scope grant (E8).

use ferrohealth_ui::html::Html;
use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::view::conditions::{
    ADD_CONDITION, ADD_GROUP, Choice, Condition, ConditionGroup, Match, Operand, REMOVE,
    REMOVE_GROUP, condition_groups,
};
use ferrohealth_ui::view::identifier::{Domain, Masked, identifier_static};
use ferrohealth_ui::view::reading::{Element, INDENTED, Level, Reading, Section, Value, reading};
use ferrohealth_ui::view::results::{Export, ResultsPane, SHOWN, Shown, Switch, results_pane};
use ferrohealth_ui::view::scope::{
    Compartment, Grant, Operation, Operations, Request, Scope, ScopeError, ScopeGrant, scope_grants,
};
use ferrohealth_ui::view::subject::{Subject, SubjectLine, subject_line};
use ferrohealth_ui::view::versions::{
    AtTime, Standing, Version, VersionHistory, older_version, version_cell, version_history,
};

use crate::markup::{has, tags};
use crate::render::{en, html};

// ---- E1 ----

#[test]
fn a_subject_line_draws_the_identifier_and_both_states() {
    let line = html({
        subject_line(
            &en(),
            SubjectLine {
                subject: Subject::External(identifier_static(
                    &en(),
                    &Masked::new(Domain::new("demo.hospital.mrn"), "P-20481-537", 3),
                    true,
                )),
                queryable: true,
                modifiable: true,
            },
        )
    });
    assert!(has(&line, "div", &[("class", "fui-subject")]), "{line}");
    assert!(line.contains(">Subject</span>"), "the label: {line}");
    assert!(line.contains("demo.hospital.mrn"), "the namespace: {line}");
    assert!(!line.contains("P-20481"), "the value stays masked: {line}");
    assert!(
        line.contains("Queryable") && line.contains("Modifiable"),
        "{line}"
    );
    assert!(
        has(
            &line,
            "span",
            &[("class", "fui-dot fui-tone-ok"), ("aria-hidden", "true")]
        ),
        "a filled dot beside the word: {line}"
    );
    assert!(!line.contains("style="), "{line}");
}

#[test]
fn a_cleared_state_is_the_hollow_ring_and_its_negation() {
    // NOTE: design/COMPONENTS.md family rule 12: a state is a shape, a colour and a word.
    let line = html({
        subject_line(
            &en(),
            SubjectLine {
                subject: Subject::None,
                queryable: false,
                modifiable: true,
            },
        )
    });
    assert!(line.contains("No external subject"), "{line}");
    assert!(line.contains("Not queryable"), "{line}");
    assert!(
        has(
            &line,
            "span",
            &[("class", "fui-state fui-tone-neutral is-muted")]
        ),
        "{line}"
    );
    assert!(
        has(
            &line,
            "span",
            &[("class", "fui-dot fui-tone-neutral is-muted")]
        ),
        "the hollow ring: {line}"
    );
}

// ---- E2 ----

/// A version `number` standing `standing`, read when `read`.
fn version(number: &str, standing: Standing, read: bool) -> Version {
    Version {
        number: number.to_owned(),
        standing,
        href: (standing != Standing::Deleted).then(|| format!("/versions/{number}")),
        change: String::from("Modification"),
        committer: String::from("Demo Nurse <b>"),
        when: Html::text("5 min ago"),
        contribution: Html::raw(String::from(
            r#"<a href="/contributions/1">0199f1d0…4f16</a>"#,
        )),
        read,
    }
}

#[test]
fn a_version_history_marks_current_deleted_and_the_version_read() {
    let table = html({
        version_history(
            &en(),
            VersionHistory {
                caption: String::from("Versions"),
                versions: vec![
                    version("4", Standing::Deleted, false),
                    version("3", Standing::Current, false),
                    version("2", Standing::Older, true),
                ],
                at_time: None,
            },
        )
    });
    assert!(table.contains("Version 3"), "{table}");
    assert!(
        has(&table, "span", &[("class", "fui-badge fui-tone-accent")]),
        "the Current mark: {table}"
    );
    assert!(table.contains("Deleted"), "{table}");
    assert_eq!(
        1,
        tags(&table, "tr")
            .iter()
            .filter(|row| row.get("aria-current").map(String::as_str) == Some("true"))
            .count(),
        "one row is the version read: {table}"
    );
    assert!(has(&table, "a", &[("href", "/versions/2")]), "{table}");
    assert!(
        !has(&table, "a", &[("href", "/versions/4")]),
        "a deletion has nothing to read"
    );
    for header in ["Version", "Change", "Committed by", "When", "Contribution"] {
        assert!(
            table.contains(&format!(">{header}</th>")),
            "{header}: {table}"
        );
    }
    assert!(table.contains("Demo Nurse &#60;b&#62;"), "escaped: {table}");
}

#[test]
fn a_version_cell_keeps_its_mark_beside_its_number() {
    let cell = html(version_cell(&en(), "1.1.0", Standing::Current));
    assert!(
        has(&cell, "span", &[("class", "fui-version-cell")]),
        "{cell}"
    );
    assert!(cell.contains("Version 1.1.0"), "{cell}");
}

#[test]
fn at_a_time_is_a_get_form_with_its_field_open_and_quiet_line() {
    let table = html({
        version_history(
            &en(),
            VersionHistory {
                caption: String::from("Versions"),
                versions: vec![version("1", Standing::Current, false)],
                at_time: Some(AtTime {
                    action: String::from("/compositions/c1"),
                    field: Html::raw(String::from(r#"<div class="fui-field">field</div>"#)),
                    opens: String::from("Opens the version current then."),
                }),
            },
        )
    });
    assert!(
        has(
            &table,
            "form",
            &[
                ("class", "fui-at-time"),
                ("method", "get"),
                ("action", "/compositions/c1")
            ]
        ),
        "{table}"
    );
    assert!(
        has(&table, "button", &[("type", "submit")]),
        "Open: {table}"
    );
    assert!(table.contains(">Open</button>"), "{table}");
    assert!(table.contains("Opens the version current then."), "{table}");
    let table_at = table.find("<table").unwrap();
    let form_at = table.find("<form").unwrap();
    assert!(table_at < form_at, "At a time sits under the table");
}

#[test]
fn a_version_history_with_no_version_says_so_in_its_table() {
    let table = html({
        version_history(
            &en(),
            VersionHistory {
                caption: String::from("Versions"),
                versions: vec![],
                at_time: None,
            },
        )
    });
    assert!(
        has(&table, "td", &[("colspan", "5")]),
        "one row across the columns: {table}"
    );
    assert!(
        table.contains("No version has been committed."),
        "a phone reads the line too: {table}"
    );
}

#[test]
fn an_older_version_says_so_once_with_the_way_to_the_latest() {
    let shown = html(older_version(&en(), "2", 3, "/compositions/c1"));
    assert!(shown.contains("You are reading version 2 of 3."), "{shown}");
    assert!(has(&shown, "a", &[("href", "/compositions/c1")]), "{shown}");
    assert!(shown.contains("Read the latest"), "{shown}");
    assert!(
        has(&shown, "div", &[("class", "fui-notice fui-tone-info")]),
        "{shown}"
    );
}

// ---- E3 ----

/// A composition read with a nested cluster two levels deep.
fn composition() -> Reading {
    Reading {
        sections: vec![Section {
            title: String::from("Blood pressure"),
            node: Some(String::from("openEHR-EHR-OBSERVATION.blood_pressure.v2")),
            elements: vec![
                Element {
                    label: String::from("Systolic"),
                    value: Value::Quantity {
                        magnitude: String::from("128"),
                        unit: String::from("mm Hg"),
                        ucum: String::from("mm[Hg]"),
                    },
                },
                Element {
                    label: String::from("Position"),
                    value: Value::Coded(Html::raw(String::from(
                        r#"<span class="coded">Sitting</span>"#,
                    ))),
                },
                Element {
                    label: String::from("Cuff size"),
                    value: Value::Null(String::from("No information: unknown")),
                },
            ],
            sections: vec![Section {
                title: String::from("Device"),
                node: Some(String::from("openEHR-EHR-CLUSTER.device.v1")),
                elements: vec![Element {
                    label: String::from("Device name"),
                    value: Value::Text(String::from("Upper-arm cuff <i>")),
                }],
                sections: vec![Section {
                    title: String::from("Calibration"),
                    node: None,
                    elements: Vec::new(),
                    sections: Vec::new(),
                }],
            }],
        }],
        level: Level::Three,
        folded: Some(Html::text("The uids are folded away.")),
    }
}

#[test]
fn a_reading_draws_its_elements_with_their_units_codes_and_null_flavours() {
    let shown = html(reading(&en(), composition()));
    assert!(has(&shown, "div", &[("class", "fui-reading")]), "{shown}");
    assert!(
        has(&shown, "dl", &[("class", "fui-reading-rows")]),
        "{shown}"
    );
    assert!(shown.contains("<dt>Systolic</dt>"), "{shown}");
    assert!(
        has(
            &shown,
            "span",
            &[("class", "fui-reading-unit"), ("title", "UCUM mm[Hg]")]
        ),
        "the UCUM code is the tooltip: {shown}"
    );
    assert!(shown.contains(">mm Hg</span>"), "{shown}");
    assert!(
        has(&shown, "span", &[("class", "coded")]),
        "the coding: {shown}"
    );
    assert!(
        has(&shown, "span", &[("class", "fui-reading-null")]),
        "the null flavour: {shown}"
    );
    assert!(
        shown.contains("Upper-arm cuff &#60;i&#62;"),
        "escaped: {shown}"
    );
    assert!(shown.contains("The uids are folded away."), "{shown}");
}

#[test]
fn a_reading_nests_its_headings_as_the_document_nests() {
    // NOTE: WCAG 2.2 SC 1.3.1: the outline follows the document's nesting.
    let shown = html(reading(&en(), composition()));
    let blood = shown.find("<h3").unwrap();
    let device = shown.find("<h4").unwrap();
    let calibration = shown.find("<h5").unwrap();
    assert!(blood < device && device < calibration, "{shown}");
    assert_eq!(3, tags(&shown, "section").len(), "{shown}");
    assert!(
        has(
            &shown,
            "code",
            &[
                ("class", "fui-archetype fui-reading-node"),
                ("title", "openEHR-EHR-CLUSTER.device.v1")
            ]
        ),
        "{shown}"
    );
    assert!(shown.contains(".<wbr>"), "breaks only after a dot: {shown}");
}

/// A cluster nested `depth` levels under a top section, each with one
/// long unbroken value.
fn nested(depth: usize) -> Section {
    let mut inner = Section {
        title: format!("Cluster {depth}"),
        node: None,
        elements: vec![Element {
            label: String::from("Note"),
            value: Value::Text(String::from(
                "SYN0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
            )),
        }],
        sections: Vec::new(),
    };
    for level in (0..depth).rev() {
        inner = Section {
            title: format!("Cluster {level}"),
            node: None,
            elements: Vec::new(),
            sections: vec![inner],
        };
    }
    inner
}

#[test]
fn a_cluster_past_four_levels_indents_no_further() {
    // NOTE: WCAG 2.2 SC 1.4.10: a cluster nested twelve deep stays inside a phone's width.
    let shown = html({
        reading(
            &en(),
            Reading {
                sections: vec![nested(12)],
                level: Level::Three,
                folded: None,
            },
        )
    });
    let sections = tags(&shown, "section");
    assert_eq!(13, sections.len(), "{shown}");
    let deep = sections
        .iter()
        .filter(|section| {
            section
                .get("class")
                .is_some_and(|class| class == "fui-reading-section is-deep")
        })
        .count();
    assert_eq!(
        12 - INDENTED,
        deep,
        "every cluster past {INDENTED} levels: {shown}"
    );
    assert!(
        STYLESHEET.contains(
            "  .fui-reading-section .fui-reading-section.is-deep {\n    margin-inline-start: 0;\n    padding-inline-start: 0;\n    border-inline-start: 0;"
        ),
        "a deep cluster indents no further"
    );
    assert!(
        STYLESHEET.contains("  .fui-reading-rows dd {\n    margin: 0;\n    min-width: 0;\n    overflow-wrap: anywhere;"),
        "an unbroken value wraps inside its row"
    );
}

#[test]
fn a_reader_hears_a_heading_and_its_node_apart_and_the_ucum_code() {
    let shown = html(reading(&en(), composition()));
    assert!(
        shown.contains(r#"Blood pressure<span class="fui-sr-only">, </span><code"#),
        "a hidden comma between the title and its node: {shown}"
    );
    assert!(
        shown.contains(r#"<span class="fui-sr-only">, UCUM mm[Hg]</span>"#),
        "the UCUM code is said after the unit: {shown}"
    );
}

// ---- E6 ----

/// A choice of `options`, `chosen` chosen.
fn choice(options: &[&str], chosen: &str) -> Choice {
    Choice {
        options: options
            .iter()
            .map(|each| ((*each).to_owned(), (*each).to_owned()))
            .collect(),
        chosen: chosen.to_owned(),
    }
}

/// A query: All of a quantity and a coded value, and a negated Any group.
fn query() -> ConditionGroup {
    ConditionGroup {
        key: String::from("g1"),
        number: 1,
        matching: Match::All,
        negated: false,
        conditions: vec![
            Condition {
                key: String::from("c1"),
                path: choice(&["Blood pressure › Systolic"], "Blood pressure › Systolic"),
                archetype_path: String::from("…/items[at0004]/value/magnitude"),
                operator: choice(&[">", "="], ">"),
                operand: Operand::Typed {
                    value: String::from("140"),
                    unit: Some(String::from("mm Hg")),
                },
            },
            Condition {
                key: String::from("c2"),
                path: choice(&["Position"], "Position"),
                archetype_path: String::from("…/items[at0008]/value"),
                operator: choice(&["="], "="),
                operand: Operand::Coded(choice(&["at1000", "at1001"], "at1001")),
            },
        ],
        groups: vec![ConditionGroup {
            key: String::from("g2"),
            number: 2,
            matching: Match::Any,
            negated: true,
            conditions: vec![Condition {
                key: String::from("c3"),
                path: choice(&["Pulse › Rate"], "Pulse › Rate"),
                archetype_path: String::from("…/items[at0004]/value/magnitude"),
                operator: choice(&[">"], ">"),
                operand: Operand::Typed {
                    value: String::from("100"),
                    unit: None,
                },
            }],
            groups: Vec::new(),
        }],
    }
}

#[test]
fn the_groups_draw_all_any_negated_and_nested_with_their_conditions() {
    let shown = html(condition_groups(&en(), query()));
    assert!(
        has(&shown, "div", &[("class", "fui-cond-group")]),
        "{shown}"
    );
    assert!(
        has(&shown, "div", &[("class", "fui-cond-group is-not")]),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "input",
            &[("name", "g1-match"), ("value", "all"), ("checked", "")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "input",
            &[("name", "g2-match"), ("value", "any"), ("checked", "")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "input",
            &[("name", "g2-not"), ("type", "checkbox"), ("checked", "")]
        ),
        "the negation is a word: {shown}"
    );
    assert!(shown.contains("Match the opposite (NOT)"), "{shown}");
    assert_eq!(
        3,
        tags(&shown, "li")
            .iter()
            .filter(|li| li.get("class").map(String::as_str) == Some("fui-cond"))
            .count()
    );
    assert!(
        has(&shown, "span", &[("class", "fui-cond-path")]),
        "{shown}"
    );
    assert!(
        has(&shown, "option", &[("value", "at1001"), ("selected", "")]),
        "{shown}"
    );
    assert!(shown.contains("mm Hg"), "the unit suffix: {shown}");
    assert!(
        has(
            &shown,
            "button",
            &[
                ("name", REMOVE),
                ("value", "c1"),
                (
                    "aria-label",
                    "Remove the condition on Blood pressure › Systolic"
                )
            ]
        ),
        "{shown}"
    );
    assert!(
        has(&shown, "button", &[("name", REMOVE_GROUP), ("value", "g2")]),
        "{shown}"
    );
    assert!(
        !has(&shown, "button", &[("name", REMOVE_GROUP), ("value", "g1")]),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[
                ("name", ADD_CONDITION),
                ("value", "g2"),
                ("aria-label", "Add a condition to group 2")
            ]
        ),
        "{shown}"
    );
    assert_eq!(
        1,
        tags(&shown, "button")
            .iter()
            .filter(|b| b.get("name").map(String::as_str) == Some(ADD_GROUP))
            .count(),
        "{shown}"
    );
}

#[test]
fn a_group_head_reads_as_one_phrase() {
    // NOTE: WAI-ARIA 1.2, `radiogroup` named by `aria-labelledby`: the head's
    // visible words are said once, as the choice's name.
    let shown = html(condition_groups(&en(), query()));
    assert!(
        has(
            &shown,
            "div",
            &[
                ("role", "radiogroup"),
                ("aria-labelledby", "g1-match-label")
            ]
        ),
        "{shown}"
    );
    assert!(
        shown.contains("Group 1: match all or any of these conditions"),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "span",
            &[("class", "fui-cond-phrase"), ("aria-hidden", "true")]
        ),
        "{shown}"
    );
    assert!(
        has(&shown, "label", &[("for", "c1-path")]),
        "each control is labelled: {shown}"
    );
}

#[test]
fn a_chosen_value_its_select_does_not_list_stays_chosen() {
    let mut group = query();
    if let Some(first) = group.conditions.first_mut() {
        first.operator = choice(&["=", ">"], "matches");
    }
    let shown = html(condition_groups(&en(), group));
    let select = shown
        .split(r#"<select id="c1-op""#)
        .nth(1)
        .and_then(|rest| rest.split("</select>").next())
        .expect("the operator's select");
    let options = tags(select, "option");
    let chosen: Vec<_> = options
        .iter()
        .filter(|option| option.contains_key("selected"))
        .filter_map(|option| option.get("value").cloned())
        .collect();
    assert!(
        chosen.contains(&String::from("matches")),
        "the value the condition has posts, never the first option: {shown}"
    );
    assert!(
        shown.contains("matches (not in the list)"),
        "the select says it is not listed: {shown}"
    );
    assert!(
        !options.iter().any(
            |option| option.get("value").is_some_and(|value| value == "=")
                && option.contains_key("selected")
        ),
        "{shown}"
    );
}

#[test]
fn a_condition_row_stacks_in_a_group_too_narrow_for_its_fields() {
    // NOTE: WCAG 2.2 SC 1.4.10: between the phone layout and a width where a nested row
    // fits, the row stacks inside its group instead of widening the page (#307).
    assert!(
        STYLESHEET.contains("  .fui-cond-list {\n    container-type: inline-size;"),
        "the list measures the room its group leaves"
    );
    assert!(
        STYLESHEET.contains(
            "  @container (width < 30rem) {\n    .fui-cond {\n      grid-template-columns: minmax(0, 1fr) auto;"
        ),
        "a narrow group stacks its rows"
    );
}

// ---- E7 ----

/// A pane showing `shown`, its switch `switch`.
fn pane(shown: Shown, switch: Switch) -> String {
    html({
        results_pane(
            &en(),
            ResultsPane {
                title: String::from("Results"),
                count: 6,
                numeric: true,
                shown,
                switch,
                csv: Export::Link(String::from("/answer.csv")),
                json: Export::Link(String::from("/answer.json")),
                window: String::from("Rows 1 to 6 of 6, inside the query's own LIMIT 50."),
                body: Html::raw(String::from(r#"<div class="fui-table-scroll">table</div>"#)),
            },
        )
    })
}

#[test]
fn a_results_pane_draws_its_count_window_switch_and_exports() {
    let shown = pane(
        Shown::Table,
        Switch::Url {
            table: String::from("?shown=table"),
            chart: String::from("?shown=chart"),
        },
    );
    assert!(shown.contains("Results"), "{shown}");
    assert!(shown.contains("· 6"), "the count: {shown}");
    assert!(
        has(&shown, "p", &[("class", "fui-results-window")]),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "summary",
            &[
                ("class", "fui-button fui-button-secondary fui-button-small"),
                ("aria-haspopup", "menu"),
                ("aria-label", "Export")
            ]
        ),
        "one Export opens the menu of the formats (F13): {shown}"
    );
    assert!(
        has(
            &shown,
            "a",
            &[("role", "menuitem"), ("href", "/answer.csv")]
        ) && shown.contains(">CSV<"),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "a",
            &[("role", "menuitem"), ("href", "/answer.json")]
        ) && shown.contains(">JSON<"),
        "{shown}"
    );
    assert!(
        !shown.contains("Export CSV") && !shown.contains("Export JSON"),
        "no export button of its own beside the menu: {shown}"
    );
    assert!(
        has(
            &shown,
            "a",
            &[("href", "?shown=table"), ("aria-current", "true")]
        ),
        "the view in force: {shown}"
    );
    assert!(
        !shown.contains("fui-panel-body"),
        "a table runs edge to edge: {shown}"
    );
}

#[test]
fn a_session_switch_posts_and_a_chart_keeps_its_inset() {
    let shown = pane(
        Shown::Chart,
        Switch::Session {
            action: String::from("/results/shown"),
            back: String::from("/query"),
        },
    );
    assert!(
        has(
            &shown,
            "form",
            &[("method", "post"), ("action", "/results/shown")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[
                ("name", SHOWN),
                ("value", "chart"),
                ("aria-pressed", "true")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(&shown, "div", &[("class", "fui-panel-body")]),
        "{shown}"
    );
}

#[test]
fn a_pane_rides_the_consoles_form_with_no_form_of_its_own() {
    let shown = html({
        results_pane(
            &en(),
            ResultsPane {
                title: String::from("Results"),
                count: 6,
                numeric: true,
                shown: Shown::Chart,
                switch: Switch::Form {
                    form: Some(String::from("query")),
                },
                csv: Export::Submit {
                    action: String::from("/query/answer.csv"),
                    form: Some(String::from("query")),
                },
                json: Export::Submit {
                    action: String::from("/query/answer.json"),
                    form: Some(String::from("query")),
                },
                window: String::from("Rows 1 to 6 of 6."),
                body: Html::raw(String::from(r#"<div class="fui-table-scroll">chart</div>"#)),
            },
        )
    });
    assert!(
        tags(&shown, "form").is_empty(),
        "a form inside the console's form is dropped by the parser: {shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[
                ("type", "submit"),
                ("form", "query"),
                ("name", SHOWN),
                ("value", "chart"),
                ("aria-pressed", "true")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[
                ("form", "query"),
                ("value", "table"),
                ("aria-pressed", "false")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[
                ("type", "submit"),
                ("form", "query"),
                ("formaction", "/query/answer.csv"),
                ("role", "menuitem")
            ]
        ) && shown.contains(">CSV<"),
        "an Export item rides the console's form by formaction: {shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[("formaction", "/query/answer.json"), ("role", "menuitem")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "div",
            &[("role", "group"), ("aria-label", "Show the results as")]
        ),
        "{shown}"
    );
}

/// A table pane over `count` rows, with or without a numeric column.
fn answered(count: u64, numeric: bool) -> String {
    html({
        results_pane(
            &en(),
            ResultsPane {
                title: String::from("Results"),
                count,
                numeric,
                shown: Shown::Chart,
                switch: Switch::Url {
                    table: String::from("?shown=table"),
                    chart: String::from("?shown=chart"),
                },
                csv: Export::Link(String::from("/answer.csv")),
                json: Export::Link(String::from("/answer.json")),
                window: String::from("No rows."),
                body: Html::raw(String::from(r#"<div class="fui-table-scroll">table</div>"#)),
            },
        )
    })
}

#[test]
fn an_answer_with_no_row_offers_no_chart_and_no_export_and_says_why() {
    let shown = answered(0, true);
    assert!(!shown.contains("?shown=chart"), "no chart: {shown}");
    assert!(
        !has(&shown, "a", &[("href", "/answer.csv")])
            && !has(&shown, "a", &[("href", "/answer.json")]),
        "no export link: {shown}"
    );
    assert!(
        has(&shown, "span", &[("aria-disabled", "true")])
            && shown.contains("Export")
            && shown.contains("(unavailable)"),
        "Export stays as a disabled control: {shown}"
    );
    assert!(
        !shown.contains("<summary"),
        "a disabled Export opens no menu: {shown}"
    );
    assert!(
        shown.contains("There is nothing to chart or export: the query returned no rows."),
        "the reason is visible text (design/COMPONENTS.md, R4): {shown}"
    );
    assert!(
        !shown.contains("fui-panel-body"),
        "the table shows: {shown}"
    );
}

#[test]
fn an_answer_with_no_numeric_column_offers_no_chart_and_says_why() {
    let shown = answered(3, false);
    assert!(!shown.contains("?shown=chart"), "no chart: {shown}");
    assert!(
        has(&shown, "a", &[("href", "/answer.csv")]),
        "its rows export: {shown}"
    );
    assert!(
        shown.contains("There is no chart: the answer has no numeric column."),
        "{shown}"
    );
    assert!(
        !shown.contains("fui-panel-body"),
        "the table shows: {shown}"
    );
}

#[test]
fn a_results_count_is_grouped_and_never_breaks() {
    let shown = answered(12_345_678_901, true);
    assert!(shown.contains("· 12,345,678,901"), "{shown}");
    assert!(
        has(&shown, "span", &[("class", "fui-list-count")]),
        "{shown}"
    );
}

// ---- E8 ----

#[test]
fn a_scope_parses_by_the_smart_grammar() {
    // NOTE: SMART App Launch 2.2.0, Scopes and Launch Context: a subset of the in-order
    // string `cruds`; the SMART 1 suffixes read as `rs`, `cud` and `cruds`.
    let scope = Scope::parse("system/aql-org.openehr::bp.v1.rs").unwrap();
    assert_eq!(Compartment::System, scope.compartment);
    assert_eq!(
        "aql-org.openehr::bp.v1", scope.resource,
        "the permissions follow the last dot"
    );
    assert!(scope.operations.permits(Operation::Read));
    assert!(!scope.operations.permits(Operation::Create));
    let v1 = Scope::parse("patient/Observation.write").unwrap();
    assert!(v1.operations.permits(Operation::Delete) && !v1.operations.permits(Operation::Read));
    let all = Scope::parse("user/*.*").unwrap();
    assert!(
        Operation::ALL
            .into_iter()
            .all(|each| all.operations.permits(each))
    );
    let constrained = Scope::parse("patient/Observation.rs?category=laboratory").unwrap();
    assert_eq!(
        Some("category=laboratory"),
        constrained.constraint.as_deref()
    );
    for (raw, error) in [
        ("Observation.rs", ScopeError::Compartment),
        ("doctor/Observation.rs", ScopeError::Compartment),
        ("patient/.rs", ScopeError::Resource),
        ("patient/Observation", ScopeError::Resource),
        ("patient/Observation.dus", ScopeError::Permissions),
        ("patient/Observation.rr", ScopeError::Permissions),
        ("patient/Observation.", ScopeError::Permissions),
    ] {
        assert_eq!(Err(error), Scope::parse(raw), "{raw}");
    }
}

#[test]
fn a_scope_a_console_parsed_itself_draws_as_the_parsed_one() {
    let built = Scope {
        compartment: Compartment::Patient,
        resource: String::from("composition-*"),
        operations: Operations::of(&[Operation::Read, Operation::Search]),
        constraint: None,
    };
    assert_eq!(Scope::parse("patient/composition-*.rs").unwrap(), built);
    let draw = |scope: Scope| {
        html({
            scope_grants(
                &en(),
                "Scopes",
                vec![ScopeGrant {
                    raw: String::from("patient/composition-*.rs"),
                    grant: Grant::Granted {
                        scope,
                        resource: String::from("Compositions of any template"),
                        broad: false,
                    },
                }],
            )
        })
    };
    let parsed = Scope::parse("patient/composition-*.rs").unwrap();
    assert_eq!(draw(parsed), draw(built));
    let every: Operations = Operation::ALL.into_iter().collect();
    assert!(Operation::ALL.into_iter().all(|each| every.permits(each)));
    assert_eq!(Operations::default(), Operations::of(&[]));
}

#[test]
fn a_grant_strikes_an_absent_operation_and_marks_a_broad_one() {
    let narrow = Scope::parse("patient/composition-*.rs").unwrap();
    let broad = Scope::parse("system/aql-*.rs").unwrap();
    let shown = html({
        scope_grants(
            &en(),
            "Scopes",
            vec![
                ScopeGrant {
                    raw: String::from("patient/composition-*.rs"),
                    grant: Grant::Granted {
                        scope: narrow,
                        resource: String::from("Compositions of any template"),
                        broad: false,
                    },
                },
                ScopeGrant {
                    raw: String::from("system/aql-*.rs"),
                    grant: Grant::Granted {
                        scope: broad,
                        resource: String::from("Every stored query"),
                        broad: true,
                    },
                },
                ScopeGrant {
                    raw: String::from("doctor/x.rs"),
                    grant: Grant::Refused(ScopeError::Compartment.expected(&en())),
                },
            ],
        )
    });
    assert!(
        has(
            &shown,
            "ul",
            &[("class", "fui-scopes"), ("aria-label", "Scopes")]
        ),
        "{shown}"
    );
    assert!(
        has(&shown, "code", &[("class", "fui-scope-raw")]),
        "{shown}"
    );
    assert!(
        shown.contains("Patient · Compositions of any template"),
        "{shown}"
    );
    assert_eq!(
        6,
        tags(&shown, "del").len(),
        "three struck per grant: {shown}"
    );
    assert!(shown.contains("not permitted"), "said absent: {shown}");
    assert_eq!(
        4,
        tags(&shown, "span")
            .iter()
            .filter(|s| s.get("class").map(String::as_str) == Some("fui-scope-op is-on"))
            .count()
    );
    assert_eq!(1, shown.matches("Broad access").count(), "{shown}");
    assert!(
        has(&shown, "li", &[("class", "fui-scope is-refused")]),
        "{shown}"
    );
    assert!(
        shown.contains("Expected patient, user or system, then a slash."),
        "{shown}"
    );
}

#[test]
fn a_scope_that_asks_for_no_resource_reads_as_what_it_asks_for() {
    // NOTE: SMART App Launch 2.2.0, Scopes and Launch Context: identity, launch context and
    // refresh token scopes grant no resource and are no malformed resource scope.
    for (raw, request) in [
        ("openid", Request::OpenId),
        ("profile", Request::Profile),
        ("fhirUser", Request::FhirUser),
        ("launch", Request::Launch),
        ("online_access", Request::OnlineAccess),
        ("offline_access", Request::OfflineAccess),
        (
            "launch/patient",
            Request::LaunchContext {
                resource: String::from("patient"),
                role: None,
            },
        ),
        (
            "launch/relatedperson?role=friend",
            Request::LaunchContext {
                resource: String::from("relatedperson"),
                role: Some(String::from("friend")),
            },
        ),
    ] {
        assert_eq!(Some(request), Request::parse(raw), "{raw}");
    }
    for raw in ["launch/", "launch/patient?x=1", "patient/Observation.rs"] {
        assert_eq!(None, Request::parse(raw), "{raw}");
    }
    let shown = html({
        scope_grants(
            &en(),
            "Scopes",
            [
                "openid",
                "profile",
                "fhirUser",
                "launch/patient",
                "offline_access",
            ]
            .into_iter()
            .map(|raw| ScopeGrant::read(&en(), raw, |_| (String::new(), false)))
            .collect(),
        )
    });
    assert!(!shown.contains("is-refused"), "none is malformed: {shown}");
    assert!(shown.contains("Who you are, by OpenID Connect"), "{shown}");
    assert!(
        shown.contains("Your profile: your name, picture and the like"),
        "OpenID Connect Core 1.0, § 5.4: profile is a standard scope value: {shown}"
    );
    assert!(
        shown.contains("The patient in context at launch"),
        "{shown}"
    );
    assert!(
        shown.contains("A refresh token that outlasts your session"),
        "{shown}"
    );
}

#[test]
fn a_constrained_grant_shows_its_constraint_and_its_raw_scope_breaks() {
    let raw = "patient/Observation.rs?category=http://terminology.hl7.org/CodeSystem/observation-category|laboratory";
    let shown = html({
        scope_grants(
            &en(),
            "Scopes",
            vec![ScopeGrant::read(&en(), raw, |_| {
                (String::from("Observations"), false)
            })],
        )
    });
    assert!(
        has(&shown, "span", &[("class", "fui-scope-constraint")]),
        "the constraint narrows the grant: {shown}"
    );
    assert!(shown.contains("Only where"), "{shown}");
    assert!(shown.contains("laboratory"), "{shown}");
    assert!(
        shown.contains("patient/<wbr>Observation.<wbr>rs?<wbr>category=<wbr>"),
        "the raw scope breaks after its marks: {shown}"
    );
    assert!(
        STYLESHEET.contains("  .fui-scope-raw {\n    min-width: 0;"),
        "the raw scope may shrink"
    );
}
