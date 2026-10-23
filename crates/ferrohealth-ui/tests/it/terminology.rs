// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The terminology parts rendered to HTML: the coding (M2), the version
//! select (M3), the value set clause (M4), the root matrix (M5), the
//! mapping row (M6) and the repeatable rows (F7).

use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::table::view::{Body, DataTable, Sorting, data_table};
use ferrohealth_ui::view::clause::{
    Clause, ClauseCode, ClauseIssue, ClauseKind, clause, clause_codes, clauses,
};
use ferrohealth_ui::view::coding::{Coding, Layout, MoreCodes, canonical, coding, inactive_mark};
use ferrohealth_ui::view::mapping::{
    MapElement, Mapping, Relationship, map_element, map_elements, mapping_columns, mapping_row,
    relation,
};
use ferrohealth_ui::view::root_matrix::{MatrixRoot, MatrixRow, RootMatrix, root_matrix};
use ferrohealth_ui::widget::repeat::{Fields, ROW, Repeat, RepeatAdd, RepeatRow, RowEnd, repeat};
use ferrohealth_ui::widget::version::{VersionRoot, VersionSelect, version_select};
use leptos::prelude::*;

use crate::markup::{has, tags};
use crate::render::{html, page_html};

/// The text of `html` with its tags taken out.
fn text(html: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for each in html.chars() {
        match each {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(each),
            _ => {}
        }
    }
    out
}

#[test]
fn a_coding_is_its_display_over_its_code_and_system() {
    let stacked = html(|| {
        coding(
            Coding::new("SF-1210", "Cough <dry>").system("Synthetic findings"),
            Layout::Stacked,
        )
    });
    assert!(
        has(&stacked, "span", &[("class", "fui-coding")]),
        "{stacked}"
    );
    assert!(
        has(&stacked, "code", &[("class", "fui-coding-code")]),
        "the code in the code face: {stacked}"
    );
    assert!(stacked.contains(">SF-1210</code>"), "{stacked}");
    assert!(stacked.contains("Cough &lt;dry&gt;"), "escaped: {stacked}");
    assert!(
        has(&stacked, "span", &[("class", "fui-coding-system")]),
        "{stacked}"
    );
    let inline = html(|| coding(Coding::new("SF-1210", "Cough"), Layout::Inline));
    assert!(
        has(
            &inline,
            "span",
            &[("class", "fui-coding fui-coding-inline")]
        ),
        "{inline}"
    );
    assert!(!inline.contains("fui-coding-system"), "no system: {inline}");
    assert!(!inline.contains("style="), "{inline}");
}

#[test]
fn a_coding_with_no_display_says_so_and_a_linked_one_is_a_row_link() {
    let bare = html(|| {
        coding(
            Coding {
                display: None,
                code: String::from("X-1"),
                system: None,
                inactive: false,
                href: None,
                current: false,
            },
            Layout::Stacked,
        )
    });
    assert!(bare.contains("no display"), "{bare}");
    let linked = html(|| coding(Coding::new("X-1", "One").href("/c/x-1"), Layout::Stacked));
    assert!(
        has(
            &linked,
            "a",
            &[("class", "fui-row-link"), ("href", "/c/x-1")]
        ),
        "{linked}"
    );
}

#[test]
fn an_inactive_coding_reads_as_the_word_inactive() {
    // NOTE: COMPONENTS.md family rule 12: a state is a shape, a colour and a word together.
    let inactive = html(|| coding(Coding::new("SF-9", "Old").inactive(), Layout::Inline));
    assert!(
        has(
            &inactive,
            "span",
            &[("class", "fui-coding fui-coding-inline is-inactive")]
        ),
        "{inactive}"
    );
    let mark = html(inactive_mark);
    assert_eq!(
        r#"<span class="fui-inactive-mark">Inactive</span>"#, mark,
        "the ring is the stylesheet's, with no text; the word is the mark's only text"
    );
}

#[test]
fn a_canonical_breaks_only_after_a_slash() {
    let shown = html(|| canonical("https://example.org/fhir/ValueSet/a-b"));
    assert!(
        has(&shown, "code", &[("class", "fui-canonical")]),
        "{shown}"
    );
    assert_eq!(
        5,
        tags(&shown, "wbr").len(),
        "one break after each slash: {shown}"
    );
    assert!(
        shown.contains("ValueSet/<wbr>a-b"),
        "the last segment stays whole: {shown}"
    );
    assert_eq!(
        "https://example.org/fhir/ValueSet/a-b",
        text(&shown),
        "the text is the canonical whole"
    );
}

/// The version select over four roots, `current` in view.
fn select(current: &str) -> VersionSelect {
    VersionSelect {
        id: String::from("fhir-version"),
        label: String::from("FHIR version"),
        roots: [
            ("R4", "FHIR 4.0.1"),
            ("R4B", "FHIR 4.3.0"),
            ("R5", "FHIR 5.0.0"),
        ]
        .into_iter()
        .map(|(name, release)| VersionRoot {
            name: name.to_owned(),
            release: release.to_owned(),
            href: format!("?fhir={name}"),
            default: name == "R4",
        })
        .collect(),
        current: current.to_owned(),
    }
}

#[test]
fn the_version_select_shows_the_version_alone_with_its_label_as_name_and_tooltip() {
    let shown = page_html(|| version_select(select("R5")));
    let name = "FHIR version: R5, FHIR 5.0.0";
    assert!(
        has(
            &shown,
            "summary",
            &[
                ("class", "fui-version-select"),
                ("aria-label", name),
                ("title", name),
                ("aria-haspopup", "menu"),
                ("aria-controls", "fhir-version"),
            ]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "details",
            &[("class", "fui-anchor fui-version-anchor")]
        ),
        "the top bar hides it below 40 rem: {shown}"
    );
    assert!(
        has(&shown, "div", &[("id", "fhir-version"), ("role", "menu")]),
        "{shown}"
    );
}

#[test]
fn the_version_menu_checks_the_root_in_view_and_marks_the_default() {
    let shown = page_html(|| version_select(select("R5")));
    let items = tags(&shown, "a");
    assert_eq!(3, items.len(), "one choice per root: {shown}");
    assert!(
        items
            .iter()
            .all(|item| item.get("role").map(String::as_str) == Some("menuitemradio")),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "a",
            &[("href", "?fhir=R5"), ("aria-checked", "true")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "a",
            &[("href", "?fhir=R4"), ("aria-checked", "false")]
        ),
        "{shown}"
    );
    assert!(shown.contains("FHIR 4.0.1 · default"), "{shown}");
    assert!(shown.contains(">FHIR 5.0.0</span>"), "{shown}");
    assert_eq!(
        3,
        tags(&shown, "svg")
            .iter()
            .filter(|svg| svg.get("class").map(String::as_str)
                == Some("fui-icon fui-version-check"))
            .count(),
        "every choice keeps the check's room: {shown}"
    );
}

#[test]
fn the_version_choices_are_the_user_menus_group_below_40rem() {
    let choices = select("R4B").choices();
    assert_eq!("FHIR version", choices.label);
    let chosen: Vec<&str> = choices
        .choices
        .iter()
        .filter(|choice| choice.chosen)
        .map(|choice| choice.words.as_str())
        .collect();
    assert_eq!(vec!["R4B"], chosen);
    let unknown = select("R9").choices();
    assert!(
        unknown.choices.first().is_some_and(|choice| choice.chosen),
        "an unknown root falls back to the first"
    );
    let none = page_html(|| {
        version_select(VersionSelect {
            roots: Vec::new(),
            ..select("R4")
        })
    });
    assert!(
        !none.contains("fui-version-select") && !none.contains("menuitemradio"),
        "no root, no control: {none}"
    );
}

/// A clause of `kind`, broken when `broken`.
fn a_clause(kind: ClauseKind, broken: bool) -> String {
    html(move || {
        clause(Clause {
            kind,
            title: view! { "From " <a href="/cs">"Findings"</a> }.into_any(),
            source: String::from("From Findings <v3>"),
            sentence: "Includes everything below SF-1200.".into_any(),
            body: Some(clause_codes(
                vec![ClauseCode {
                    coding: Coding::new("SF-1310", "Fever"),
                    remove: None,
                }],
                None,
            )),
            actions: None,
            issues: if broken {
                vec![ClauseIssue {
                    key: String::from("vsd-3"),
                    text: String::from("Cannot have both concept and filter."),
                }]
            } else {
                Vec::new()
            },
        })
    })
}

#[test]
fn a_clause_names_its_kind_in_words_beside_its_edge() {
    let include = a_clause(ClauseKind::Include, false);
    assert!(
        has(
            &include,
            "section",
            &[
                ("class", "fui-clause"),
                ("aria-label", "Include: From Findings &lt;v3&gt;")
            ]
        ),
        "{include}"
    );
    assert!(
        has(&include, "span", &[("class", "fui-badge fui-tone-accent")]),
        "{include}"
    );
    assert!(include.contains(">Include</span>"), "the word: {include}");
    assert!(
        has(&include, "h3", &[("class", "fui-clause-title")]),
        "{include}"
    );
    assert!(
        has(&include, "ul", &[("class", "fui-clause-codes")]),
        "{include}"
    );
    assert!(
        include.contains("fui-coding fui-coding-inline"),
        "each code an inline coding: {include}"
    );
    assert!(!include.contains("fui-clause-issue"), "{include}");
    let exclude = a_clause(ClauseKind::Exclude, false);
    assert!(
        has(&exclude, "section", &[("class", "fui-clause is-exclude")]),
        "{exclude}"
    );
    assert!(exclude.contains(">Exclude</span>"), "{exclude}");
    assert!(
        has(&exclude, "span", &[("class", "fui-badge fui-tone-warn")]),
        "{exclude}"
    );
}

#[test]
fn a_broken_invariant_is_named_by_its_number_inline() {
    // NOTE: HL7 FHIR R4 (4.0.1), ValueSet invariant vsd-3: "Cannot have both concept and filter".
    let broken = a_clause(ClauseKind::Include, true);
    assert!(
        has(
            &broken,
            "p",
            &[("class", "fui-clause-issue"), ("role", "note")]
        ),
        "{broken}"
    );
    assert!(
        broken.contains("<code>vsd-3</code>Cannot have both concept and filter."),
        "the number first, then the sentence: {broken}"
    );
    let together = html(|| clauses(view! { <section></section> }));
    assert!(
        has(&together, "div", &[("class", "fui-clauses")]),
        "{together}"
    );
}

#[test]
fn two_broken_invariants_are_a_list_of_issues_whose_numbers_never_break() {
    // NOTE: HL7 FHIR R4 (4.0.1), ValueSet vsd-2 and vsd-3 can fail on one clause together.
    let broken = html(|| {
        clause(Clause {
            kind: ClauseKind::Include,
            title: "No code system".into_any(),
            source: String::from("No code system"),
            sentence: "Lists codes and a filter.".into_any(),
            body: None,
            actions: None,
            issues: vec![
                ClauseIssue {
                    key: String::from("vsd-2"),
                    text: String::from("Choose the code system its codes and filter come from."),
                },
                ClauseIssue {
                    key: String::from("vsd-3"),
                    text: String::from("Split the codes and the filter into two clauses."),
                },
            ],
        })
    });
    assert!(
        has(&broken, "ul", &[("class", "fui-clause-issues")]),
        "{broken}"
    );
    assert_eq!(
        2,
        tags(&broken, "li")
            .iter()
            .filter(|item| item
                .get("class")
                .is_some_and(|class| class == "fui-clause-issue"))
            .count(),
        "one issue each: {broken}"
    );
    assert!(broken.contains("<code>vsd-2</code>"), "{broken}");
    assert!(broken.contains("<code>vsd-3</code>"), "{broken}");
    assert!(
        !has(&broken, "p", &[("class", "fui-clause-issue")]),
        "{broken}"
    );
    assert!(
        STYLESHEET.contains(
            "  .fui-clause-issue code {\n    margin-inline-end: 0.25rem;\n    color: var(--bad);\n    font-weight: 600;\n    white-space: nowrap;"
        ),
        "an invariant's number breaks at its hyphen"
    );
}

#[test]
fn a_clause_of_thousands_of_codes_ends_in_the_more_row() {
    let codes = html(|| {
        clause_codes(
            vec![ClauseCode {
                coding: Coding::new("SF-1310", "Fever"),
                remove: None,
            }],
            Some(MoreCodes::of_page(1, 4000, 25, "/value-set?codes=1")),
        )
    });
    assert!(has(&codes, "li", &[("class", "fui-code-more")]), "{codes}");
    assert!(codes.contains("1 of 4,000 codes shown."), "{codes}");
    assert!(
        has(&codes, "a", &[("href", "/value-set?codes=1")]),
        "{codes}"
    );
    assert!(codes.contains("Show the next 25"), "{codes}");
    assert_eq!(
        "Show the other 10",
        MoreCodes::of_page(3990, 4000, 25, "/").link
    );
    assert_eq!(
        "Show the other one",
        MoreCodes::of_page(3999, 4000, 25, "/").link
    );
    assert_eq!(4000, MoreCodes::of_page(1, 4000, 25, "/").total);
}

#[test]
fn a_long_code_and_a_urn_canonical_break_between_their_pieces() {
    let uid = "8849182c-82ad-4088-a07f-48ead4180515::ferroehr.example.org::12";
    let shown = html(move || coding(Coding::new(uid, "Encounter"), Layout::Stacked));
    assert!(
        shown.contains("<span>8849182c-</span><wbr"),
        "a long code breaks after its hyphens: {shown}"
    );
    assert!(
        shown.contains("<span>ferroehr.</span><wbr"),
        "and its dots: {shown}"
    );
    assert_eq!(
        9,
        tags(&shown, "wbr").len(),
        "a break after each piece, and none inside a double colon: {shown}"
    );
    assert!(
        text(&shown).contains(uid),
        "the text is the code whole: {shown}"
    );
    assert!(
        STYLESHEET.contains("  .fui-coding-code:has(> wbr) {\n    white-space: normal;\n  }"),
        "a long code's breaks are held back by nowrap"
    );
    let short = html(|| coding(Coding::new("SF-1210", "Cough"), Layout::Inline));
    assert!(
        !short.contains("<wbr"),
        "an ordinary code never breaks: {short}"
    );
    let urn = "urn:uuid:8849182c-82ad-4088-a07f-48ead4180515";
    let canonical_urn = html(move || canonical(urn));
    assert_eq!(
        6,
        tags(&canonical_urn, "wbr").len(),
        "after each colon and hyphen: {canonical_urn}"
    );
    assert_eq!(urn, text(&canonical_urn));
    let oid = "urn:oid:2.16.840.1.113883.3.1937.777.63.10";
    assert!(
        !tags(&html(move || canonical(oid)), "wbr").is_empty(),
        "an OID breaks after its dots"
    );
}

#[test]
fn the_root_matrix_heads_each_column_with_its_root_release_and_state() {
    let roots = vec![
        MatrixRoot {
            name: String::from("R4"),
            release: String::from("FHIR 4.0.1"),
            answered: true,
        },
        MatrixRoot {
            name: String::from("R6"),
            release: String::from("FHIR 6.0.0-ballot5"),
            answered: false,
        },
    ];
    let shown = html(move || {
        root_matrix(RootMatrix {
            caption: String::from("What each root declares"),
            corner: String::from("Declaration"),
            roots,
            rows: vec![
                MatrixRow {
                    name: "Code systems".into_any(),
                    cells: vec![Some("15".into_any()), Some("0".into_any())],
                },
                MatrixRow {
                    name: "Purpose".into_any(),
                    cells: vec![None, None],
                },
            ],
        })
    });
    assert!(
        has(&shown, "div", &[("class", "fui-root-matrix")]),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "div",
            &[
                ("class", "fui-table-scroll"),
                ("role", "region"),
                ("aria-label", "What each root declares"),
                ("tabindex", "0")
            ]
        ),
        "the matrix scrolls inside its own region: {shown}"
    );
    assert!(
        shown.contains(r#"<caption class="fui-sr-only">What each root declares</caption>"#),
        "{shown}"
    );
    assert_eq!(
        3,
        tags(&shown, "th")
            .iter()
            .filter(|th| th.get("scope").map(String::as_str) == Some("col"))
            .count(),
        "the corner and one header cell per root: {shown}"
    );
    assert!(shown.contains("<strong>R4</strong>"), "{shown}");
    assert!(shown.contains("FHIR 6.0.0-ballot5"), "{shown}");
    assert!(
        has(&shown, "span", &[("class", "fui-state fui-tone-ok")]),
        "{shown}"
    );
    assert!(
        shown.contains("</span>Answered</span>"),
        "the state's word: {shown}"
    );
    assert!(
        has(&shown, "span", &[("class", "fui-state fui-tone-bad")]),
        "{shown}"
    );
    assert!(shown.contains("</span>No answer</span>"), "{shown}");
    assert!(
        has(
            &shown,
            "span",
            &[("class", "fui-dot fui-tone-bad"), ("aria-hidden", "true")]
        ),
        "the dot is the word's shape, hidden: {shown}"
    );
    assert!(
        has(&shown, "td", &[("data-label", "R4"), ("class", "is-none")]),
        "a cell a root does not declare: {shown}"
    );
    assert!(shown.contains(">not declared</td>"), "{shown}");
    assert_eq!(
        2,
        shown.matches(">no answer</td>").count(),
        "every cell of a root that did not answer: {shown}"
    );
    assert!(
        !shown.contains(">0</td>"),
        "a root with no answer shows none of its cells: {shown}"
    );
}

#[test]
fn every_relationship_is_a_word_with_its_arrow() {
    let narrower = html(|| relation(Relationship::Narrower));
    assert!(
        has(&narrower, "span", &[("class", "fui-relation")]),
        "{narrower}"
    );
    assert!(narrower.contains("the target is narrower"), "{narrower}");
    assert_eq!(1, tags(&narrower, "svg").len(), "{narrower}");
    let broader = html(|| relation(Relationship::SourceBroader));
    assert!(broader.contains("broader than the target"), "{broader}");
    let nothing = html(|| relation(Relationship::Unmatched));
    assert!(
        has(&nothing, "span", &[("class", "fui-relation is-none")]),
        "{nothing}"
    );
    assert!(nothing.contains("maps to nothing"), "{nothing}");
    for each in [
        Relationship::RelatedTo,
        Relationship::Equivalent,
        Relationship::Equal,
        Relationship::Wider,
        Relationship::Subsumes,
        Relationship::Narrower,
        Relationship::Specializes,
        Relationship::Inexact,
        Relationship::Disjoint,
        Relationship::SourceNarrower,
        Relationship::SourceBroader,
        Relationship::NotRelatedTo,
        Relationship::Unmatched,
    ] {
        let words = Owner::new().with(|| each.words());
        assert!(
            !words.is_empty()
                && words
                    .text()
                    .chars()
                    .all(|c| !c.is_ascii_uppercase() || c == 'I'),
            "{each:?} reads as words: {}",
            words.text()
        );
    }
}

#[test]
fn a_mapping_row_reads_source_relationship_target_and_comment() {
    let shown = html(|| {
        let columns = mapping_columns(true);
        let rows = view! {
            {mapping_row(
                &columns,
                Mapping {
                    source: Coding::new("LF-04", "Chest complaint"),
                    relationship: Relationship::Narrower,
                    target: Some(Coding::new("SF-1220", "Wheeze")),
                    comment: Some(String::from("Only the wheeze part.")),
                },
            )}
            {mapping_row(
                &columns,
                Mapping {
                    source: Coding::new("LF-07", "Feeling unwell"),
                    relationship: Relationship::Equivalent,
                    target: None,
                    comment: None,
                },
            )}
        }
        .into_any();
        data_table(DataTable {
            caption: "Every mapping",
            columns: &columns,
            sorting: Sorting::default(),
            body: Body::Rows(rows),
        })
        .into_any()
    });
    let headers: Vec<String> = ["Source", "Relationship", "Target", "Comment"]
        .iter()
        .map(|word| format!(">{word}</th>"))
        .collect();
    let mut at = 0;
    for header in &headers {
        let found = shown[at..].find(header.as_str()).map(|found| found + at);
        assert!(found.is_some(), "{header} in order: {shown}");
        at = found.unwrap_or(at);
    }
    assert!(
        has(&shown, "th", &[("scope", "row"), ("data-role", "title")]),
        "the source heads the row: {shown}"
    );
    assert!(
        has(
            &shown,
            "td",
            &[
                ("data-label", "Relationship"),
                ("class", "fui-relation-cell")
            ]
        ),
        "{shown}"
    );
    assert!(shown.contains("the target is narrower"), "{shown}");
    assert!(shown.contains("Only the wheeze part."), "{shown}");
    assert!(
        shown.contains("maps to nothing"),
        "a mapping with no target maps to nothing: {shown}"
    );
    assert!(!shown.contains(">equivalent<"), "{shown}");
    assert!(
        has(
            &shown,
            "span",
            &[("role", "img"), ("aria-label", "no target")]
        ),
        "the target cell is a named dash, never empty: {shown}"
    );
    let bare = mapping_columns(false);
    assert_eq!(3, bare.len(), "no comment column");
}

/// A row `key`, headed `heading` when given, around `inside`.
fn shaped_row(key: &str, heading: Option<&str>, inside: AnyView) -> RepeatRow {
    RepeatRow {
        key: key.to_owned(),
        heading: heading.map(str::to_owned),
        fields: inside,
        end: RowEnd::Remove {
            label: format!("Remove {key}"),
            name: String::from("remove"),
            value: key.to_owned(),
        },
        after: None,
    }
}

/// A list `id` of `fields` rows, with no blank row and no Add.
fn shaped(id: &str, fields: Fields, rows: Vec<RepeatRow>) -> AnyView {
    repeat(Repeat {
        id: id.to_owned(),
        label: String::from("Rows"),
        fields,
        head: Vec::new(),
        rows,
        blank: None,
        add: None,
        empty: String::from("No rows yet."),
        form: None,
    })
}

#[test]
fn a_one_field_row_keeps_its_field_and_its_remove_on_one_line() {
    let shown = page_html(|| {
        shaped(
            "profiles",
            Fields::One,
            vec![shaped_row(
                "p1",
                None,
                view! { <input id="p1" name="p[p1]" /> }.into_any(),
            )],
        )
    });
    assert!(
        has(
            &shown,
            "div",
            &[("class", "fui-repeater-row fui-repeater-one")]
        ),
        "{shown}"
    );
    assert!(
        STYLESHEET.contains(
            "  .fui-repeater-row.fui-repeater-one {\n    grid-template-columns: minmax(0, 1fr) auto;"
        ),
        "one field takes the row, Remove its end"
    );
}

#[test]
fn a_sub_card_row_is_a_group_named_by_its_heading_with_remove_beside_it() {
    // WAI-ARIA 1.2 `group`, `aria-labelledby`: the heading names the row's fields.
    let shown = page_html(|| {
        let nested = shaped(
            "ep-e1-profiles",
            Fields::One,
            vec![shaped_row(
                "p1",
                None,
                view! { <input id="pr" /> }.into_any(),
            )],
        );
        shaped(
            "endpoints",
            Fields::Card,
            vec![shaped_row(
                "e1",
                Some("Query API endpoint node-c-pub"),
                view! { <input id="e1-id" /> {nested} }.into_any(),
            )],
        )
    });
    assert!(
        has(
            &shown,
            "div",
            &[
                ("class", "fui-repeater-card"),
                ("role", "group"),
                ("aria-labelledby", "endpoints-e1-heading")
            ]
        ),
        "{shown}"
    );
    let head = shown
        .split(r#"<div class="fui-repeater-card-head">"#)
        .nth(1)
        .and_then(|rest| rest.split("</div>").next())
        .unwrap_or_default();
    assert!(
        head.contains(r#"<h3 id="endpoints-e1-heading">Query API endpoint node-c-pub</h3>"#),
        "{shown}"
    );
    assert!(
        head.contains(r#"aria-label="Remove e1""#),
        "Remove sits beside the heading: {shown}"
    );
    assert!(
        has(
            &shown,
            "div",
            &[("id", "ep-e1-profiles"), ("role", "group")]
        ),
        "the sub-card holds a list of its own: {shown}"
    );
}

/// A list of `rows` filter rows with a blank row and Add.
fn filters(rows: &[&str]) -> String {
    let rows: Vec<String> = rows.iter().map(|row| (*row).to_owned()).collect();
    page_html(move || {
        let row = |key: &str| RepeatRow {
            key: key.to_owned(),
            heading: None,
            fields: view! { <input id=format!("f-{key}") name=format!("f[{key}]") /> }.into_any(),
            end: RowEnd::Remove {
                label: format!("Remove the filter {key}"),
                name: String::from("remove"),
                value: key.to_owned(),
            },
            after: None,
        };
        repeat(Repeat {
            id: String::from("filters"),
            label: String::from("Filters"),
            fields: Fields::Three,
            head: Vec::new(),
            rows: rows.iter().map(|key| row(key)).collect(),
            blank: Some(row(ROW)),
            add: Some(RepeatAdd {
                label: String::from("Add filter"),
                name: String::from("add"),
                value: String::from("filter"),
            }),
            empty: String::from("No filters yet."),
            form: None,
        })
    })
}

#[test]
fn a_repeatable_row_ends_in_its_remove_which_posts_without_webassembly() {
    // NOTE: leptos-ui.md §15.8: Remove and Add are submit buttons, so they post with no script.
    let shown = filters(&["a", "b"]);
    assert!(shown.contains("<leptos-island"), "an island: {shown}");
    assert!(
        has(
            &shown,
            "div",
            &[
                ("id", "filters"),
                ("role", "group"),
                ("aria-label", "Filters")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "div",
            &[("class", "fui-repeat-item"), ("data-row", "a")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[
                ("type", "submit"),
                ("name", "remove"),
                ("value", "b"),
                ("aria-label", "Remove the filter b"),
                ("data-repeat", "remove"),
                ("formnovalidate", ""),
            ]
        ),
        "{shown}"
    );
    let row = shown
        .find(r#"<div class="fui-clause-filter">"#)
        .expect("a filter row");
    let input = shown[row..].find("<input").expect("its field");
    let remove = shown[row..]
        .find("data-repeat=\"remove\"")
        .expect("its remove");
    assert!(input < remove, "Remove is the row's last control: {shown}");
    assert!(
        has(
            &shown,
            "button",
            &[
                ("type", "submit"),
                ("name", "add"),
                ("value", "filter"),
                ("data-repeat", "add")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(&shown, "p", &[("data-repeat", "empty"), ("hidden", "")]),
        "the quiet line waits hidden: {shown}"
    );
    assert!(
        has(&shown, "template", &[("data-repeat", "blank")]),
        "{shown}"
    );
    assert!(
        shown.contains(&format!("data-row=\"{ROW}\"")),
        "the blank row keeps the key the island replaces: {shown}"
    );
}

#[test]
fn an_empty_list_says_so_and_a_kept_row_says_why() {
    let empty = filters(&[]);
    assert!(
        has(
            &empty,
            "p",
            &[("class", "fui-transform-empty"), ("data-repeat", "empty")]
        ),
        "{empty}"
    );
    assert!(
        !empty.contains(r#"data-repeat="empty" hidden"#) && empty.contains("No filters yet."),
        "{empty}"
    );
    let kept = page_html(|| {
        repeat(Repeat {
            id: String::from("properties"),
            label: String::from("Properties"),
            fields: Fields::Two,
            head: vec![String::from("Code"), String::from("Type")],
            rows: vec![RepeatRow {
                key: String::from("inactive"),
                heading: None,
                fields: view! { <input id="p-1" /> }.into_any(),
                end: RowEnd::Kept {
                    reason: String::from("The editor keeps this property"),
                },
                after: None,
            }],
            blank: None,
            add: None,
            empty: String::from("No properties yet."),
            form: Some(String::from("editor")),
        })
    });
    assert!(
        has(
            &kept,
            "button",
            &[
                ("type", "button"),
                ("disabled", ""),
                ("aria-hidden", "true")
            ]
        ),
        "the lock is shown, never heard: {kept}"
    );
    // NOTE: design F13: a disabled control says its reason as visible text, never in a
    // tooltip or a name alone.
    assert!(
        kept.contains(r#"<p class="fui-help fui-repeat-kept">The editor keeps this property</p>"#),
        "the reason is a visible line: {kept}"
    );
    assert!(
        !kept.contains(r#"title="The editor keeps this property""#),
        "{kept}"
    );
    assert!(
        has(
            &kept,
            "div",
            &[("class", "fui-repeater-head"), ("aria-hidden", "true")]
        ),
        "{kept}"
    );
    assert!(
        has(&kept, "div", &[("class", "fui-repeat-rows fui-repeater")]),
        "{kept}"
    );
    assert!(!kept.contains("<template"), "no blank row: {kept}");
}

#[test]
fn a_map_element_is_its_source_over_its_target_rows() {
    let shown = html(|| {
        map_elements(map_element(MapElement {
            source: Coding::new("LF-04", "Chest complaint"),
            actions: Some(view! { <button type="button">"Remove"</button> }.into_any()),
            targets: view! { <div class="fui-clause-filter"></div> }.into_any(),
            after: None,
        }))
    });
    assert!(
        has(&shown, "ul", &[("class", "fui-map-elements")]),
        "{shown}"
    );
    assert!(
        has(&shown, "li", &[("class", "fui-map-element")]),
        "{shown}"
    );
    let head = shown.find("fui-map-element-head").expect("its head");
    let source = shown.find("LF-04").expect("its source");
    let targets = shown.find("fui-clause-filter").expect("its targets");
    assert!(head < source && source < targets, "{shown}");
}
