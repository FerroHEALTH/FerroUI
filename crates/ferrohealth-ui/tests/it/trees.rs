// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The trees (M1, E4, E5), the browser and the path inspector rendered to
//! HTML on the host, as a console's server draws them.

use std::collections::BTreeSet;

use ferrohealth_ui::island::TreeView;
use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::table::filter::Find;
use ferrohealth_ui::table::model::StateSource;
use ferrohealth_ui::view::browser::{
    BrowserPane, Heading, Panes, PathNode, browser, browser_find, match_list, pane_body, path_node,
};
use ferrohealth_ui::view::coding::{Coding, Layout, MoreCodes, coding};
use ferrohealth_ui::view::path::{archetype, path};
use ferrohealth_ui::widget::tree::{
    Edit, Mark, More, Tree, TreeData, TreeKind, TreeNode, TreeWords, path_key, tree_view,
};
use leptos::prelude::*;

use crate::markup::{has, tags};
use crate::render::{html, page_html};

/// The tree `nodes` drawn as `kind`, with `open` open and `selected`
/// selected.
fn drawn(
    kind: TreeKind,
    editing: bool,
    nodes: Vec<TreeNode>,
    open: &[&str],
    selected: Option<&str>,
) -> String {
    let open: BTreeSet<String> = open.iter().map(|key| (*key).to_owned()).collect();
    let selected = selected.map(str::to_owned);
    html(move || {
        tree_view(Tree {
            id: String::from("browse"),
            label: String::from("Concepts"),
            kind,
            editing,
            nodes: Signal::stored(nodes),
            expanded: RwSignal::new(open),
            selected: Signal::stored(selected),
            open_field: Some(String::from("open")),
            words: TreeWords::of_page(),
        })
    })
}

/// A concept hierarchy: a paged level, an unread level, an inactive leaf.
fn concepts() -> Vec<TreeNode> {
    vec![
        TreeNode::new("root", "Synthetic finding")
            .code("SF-1000")
            .href("/c/root")
            .toggle_href("/browse?open=")
            .children(vec![
                TreeNode::new("chest", "Finding of the chest")
                    .code("SF-1200")
                    .href("/c/chest")
                    .paged(
                        vec![
                            TreeNode::new("cough", "Cough")
                                .code("SF-1210")
                                .href("/c/cough"),
                        ],
                        More::of_page(25, 40, 25, "/browse?more=chest"),
                    ),
                TreeNode::new("general", "General finding")
                    .code("SF-1300")
                    .toggle_href("/browse?open=root,general")
                    .unread(),
                TreeNode::new("old", "Malaise, old code")
                    .code("SF-1390")
                    .inactive(),
            ]),
    ]
}

#[test]
fn a_tree_item_states_its_level_by_name() {
    let tree = drawn(
        TreeKind::Concept,
        false,
        concepts(),
        &["root", "root/chest"],
        None,
    );
    assert!(tree.contains(r#"aria-level="1""#), "{tree}");
    assert!(tree.contains(r#"aria-level="2""#), "{tree}");
    assert!(tree.contains(r#"aria-level="3""#), "{tree}");
    assert!(!tree.contains("attr:aria-level"), "{tree}");
}

#[test]
fn the_concept_tree_marks_the_path_reads_a_level_and_ends_a_long_one() {
    let tree = drawn(
        TreeKind::Concept,
        false,
        concepts(),
        &["root", "root/chest", "root/general"],
        Some("root/chest/cough"),
    );
    assert!(
        has(&tree, "div", &[("class", "fui-concept-tree")]),
        "{tree}"
    );
    assert!(
        has(&tree, "ul", &[("role", "tree"), ("aria-label", "Concepts")]),
        "{tree}"
    );
    let items = tags(&tree, "li");
    let item = |words: &str| {
        items
            .iter()
            .find(|item| item.get("id").is_some_and(|id| id.ends_with(words)))
            .cloned()
            .unwrap_or_default()
    };
    // The path keys' bytes in hex: root, root/chest, root/chest/cough.
    assert!(
        item("item-726f6f74")["class"].contains("is-on-path"),
        "{tree}"
    );
    assert!(
        item("item-726f6f742f6368657374")["class"].contains("is-on-path"),
        "{tree}"
    );
    let cough = item("item-726f6f742f63686573742f636f756768");
    assert_eq!("true", cough["aria-selected"], "{tree}");
    assert_eq!("0", cough["tabindex"], "one tab stop, on the selection");
    assert!(!cough["class"].contains("is-on-path"), "{tree}");
    assert!(
        tree.contains("Reading the children"),
        "an unread open level reads: {tree}"
    );
    assert!(
        has(
            &tree,
            "li",
            &[("class", "fui-tree-loading"), ("role", "none")]
        ),
        "{tree}"
    );
    assert!(tree.contains("25 of 40 children shown."), "{tree}");
    assert!(has(&tree, "a", &[("href", "/browse?more=chest")]), "{tree}");
    assert!(tree.contains("Show the other 15"), "{tree}");
    assert!(
        has(&tree, "span", &[("class", "fui-inactive-mark")]),
        "{tree}"
    );
    assert!(
        item("item-726f6f742f6f6c64")["class"].contains("is-inactive"),
        "{tree}"
    );
    assert_eq!(
        1,
        items
            .iter()
            .filter(|item| item.get("tabindex").is_some_and(|stop| stop == "0"))
            .count(),
        "{tree}"
    );
}

#[test]
fn a_toggle_is_a_link_where_the_console_gives_one_and_names_its_item() {
    let tree = drawn(TreeKind::Concept, false, concepts(), &["root"], None);
    assert!(
        has(
            &tree,
            "a",
            &[
                ("class", "fui-tree-toggle"),
                ("href", "/browse?open=root,general"),
                ("aria-label", "Expand General finding"),
                ("tabindex", "-1"),
            ]
        ),
        "{tree}"
    );
    assert!(
        has(
            &tree,
            "a",
            &[
                ("class", "fui-tree-toggle"),
                ("aria-label", "Collapse Synthetic finding")
            ]
        ),
        "{tree}"
    );
    // A toggle with no link waits for the page to hydrate.
    let bare = drawn(
        TreeKind::Concept,
        false,
        vec![TreeNode::new("a", "A").children(vec![TreeNode::new("b", "B")])],
        &["a"],
        None,
    );
    assert!(
        has(
            &bare,
            "button",
            &[("class", "fui-tree-toggle"), ("disabled", "")]
        ),
        "{bare}"
    );
    assert!(
        has(
            &bare,
            "span",
            &[("class", "fui-tree-spacer"), ("aria-hidden", "true")]
        ),
        "a leaf keeps the toggle's room: {bare}"
    );
}

#[test]
fn a_tree_posts_its_open_items_as_hidden_fields() {
    let tree = drawn(
        TreeKind::Concept,
        false,
        concepts(),
        &["root/chest", "root"],
        None,
    );
    assert!(
        has(
            &tree,
            "input",
            &[
                ("type", "hidden"),
                ("name", "open"),
                ("value", "root/chest")
            ]
        ),
        "{tree}"
    );
    assert!(
        has(
            &tree,
            "input",
            &[("type", "hidden"), ("name", "open"), ("value", "root")]
        ),
        "{tree}"
    );
}

#[test]
fn a_concept_under_two_parents_draws_two_items_with_distinct_ids_and_one_selection() {
    let pneumonia = || {
        TreeNode::new("233604007", "Pneumonia")
            .code("233604007")
            .children(vec![TreeNode::new("75570004", "Viral pneumonia")])
    };
    let tree = drawn(
        TreeKind::Concept,
        false,
        vec![
            TreeNode::new("19829001", "Disorder of lung").children(vec![pneumonia()]),
            TreeNode::new("40733004", "Infectious disease").children(vec![pneumonia()]),
        ],
        &["19829001", "40733004", "19829001/233604007"],
        Some(&path_key(&["40733004", "233604007"])),
    );
    let items = tags(&tree, "li");
    let ids: BTreeSet<&str> = items
        .iter()
        .filter_map(|item| item.get("id").map(String::as_str))
        .collect();
    assert_eq!(items.len(), ids.len(), "every item has its own id: {tree}");
    assert_eq!(
        1,
        tree.matches("Viral pneumonia").count(),
        "only the copy under the lung is open: {tree}"
    );
    let selected: Vec<_> = items
        .iter()
        .filter(|item| {
            item.get("aria-selected")
                .is_some_and(|value| value == "true")
        })
        .collect();
    assert_eq!(1, selected.len(), "one item is selected: {tree}");
    assert!(
        selected
            .first()
            .and_then(|item| item.get("id"))
            .is_some_and(|id| id.ends_with("item-34303733333030342f323333363034303037")),
        "the copy under the infectious disease is the selected one: {tree}"
    );
    assert_eq!(
        1,
        items
            .iter()
            .filter(|item| item.get("tabindex").is_some_and(|stop| stop == "0"))
            .count(),
        "one tab stop: {tree}"
    );
}

/// A directory: a folder over two references, one removed and one renamed.
fn directory() -> Vec<TreeNode> {
    vec![
        TreeNode::new("root", "root")
            .mark(Mark::Folder)
            .edits(vec![Edit::AddFolder, Edit::AddReference])
            .children(vec![
                TreeNode::new("episodes", "2026-10 Follow-up")
                    .mark(Mark::Folder)
                    .count("2 items")
                    .renaming("2026-10 Follow-up, week 43")
                    .edits(vec![
                        Edit::AddFolder,
                        Edit::AddReference,
                        Edit::Rename,
                        Edit::Remove,
                    ])
                    .children(vec![
                        TreeNode::new("vitals", "Vital signs")
                            .mark(Mark::Reference)
                            .code("0199f1d0…7c11")
                            .href("/composition/vitals")
                            .edits(vec![Edit::Remove]),
                        TreeNode::new("summary", "Discharge summary")
                            .mark(Mark::Reference)
                            .code("01995f22…2b53")
                            .href("/composition/summary")
                            .removed()
                            .edits(vec![Edit::Undo]),
                    ]),
            ]),
    ]
}

#[test]
fn the_folder_tree_draws_folders_and_references_and_names_every_action_in_an_edit() {
    let read = drawn(
        TreeKind::Folder,
        false,
        directory(),
        &["root", "root/episodes"],
        None,
    );
    assert!(
        has(
            &read,
            "div",
            &[("class", "fui-concept-tree fui-folder-tree")]
        ),
        "{read}"
    );
    assert!(has(&read, "span", &[("class", "fui-tree-count")]), "{read}");
    assert!(read.contains("2 items"), "{read}");
    assert!(has(&read, "span", &[("class", "fui-tree-glyph")]), "{read}");
    assert!(
        tags(&read, "li")
            .iter()
            .any(|item| item["class"].contains("is-ref")),
        "{read}"
    );
    assert!(
        !read.contains("fui-tree-actions"),
        "no action outside an edit: {read}"
    );
    let edit = drawn(
        TreeKind::Folder,
        true,
        directory(),
        &["root", "root/episodes"],
        None,
    );
    assert!(
        has(
            &edit,
            "div",
            &[("class", "fui-concept-tree fui-folder-tree is-editing")]
        ),
        "{edit}"
    );
    for (name, value, words) in [
        ("add_folder", "root", "Add a folder in root"),
        (
            "add_reference",
            "episodes",
            "Add a reference in 2026-10 Follow-up",
        ),
        ("rename", "episodes", "Rename 2026-10 Follow-up"),
        ("remove", "episodes", "Remove 2026-10 Follow-up"),
        ("remove", "vitals", "Remove the reference Vital signs"),
        ("undo", "summary", "Undo removing Discharge summary"),
    ] {
        assert!(
            has(
                &edit,
                "button",
                &[
                    ("type", "submit"),
                    ("name", name),
                    ("value", value),
                    ("aria-label", words),
                    ("title", words)
                ]
            ),
            "{name} {value}: {edit}"
        );
    }
    assert!(
        has(
            &edit,
            "input",
            &[
                ("class", "fui-input"),
                ("name", "new_name"),
                ("value", "2026-10 Follow-up, week 43"),
                ("aria-label", "New name for 2026-10 Follow-up"),
            ]
        ),
        "{edit}"
    );
    assert!(edit.contains("(removed)"), "a removed item says so: {edit}");
    assert!(
        !has(&edit, "a", &[("href", "/composition/summary")]),
        "a removed reference is no link: {edit}"
    );
}

#[test]
fn the_path_tree_tags_each_row_with_its_rm_type() {
    let tree = drawn(
        TreeKind::Path,
        false,
        vec![
            TreeNode::new("bp", "Blood pressure")
                .tag("OBSERVATION")
                .children(vec![
                    TreeNode::new("sys", "Systolic")
                        .tag("DV_QUANTITY")
                        .href("/p/sys"),
                ]),
        ],
        &["bp"],
        Some("bp/sys"),
    );
    assert!(
        has(&tree, "div", &[("class", "fui-concept-tree fui-path-tree")]),
        "{tree}"
    );
    assert!(has(&tree, "span", &[("class", "fui-rm-type")]), "{tree}");
    assert!(tree.contains("DV_QUANTITY"), "{tree}");
}

#[test]
fn the_tree_view_draws_its_props_on_the_server() {
    let page = page_html(|| {
        view! {
            <TreeView
                tree=TreeData {
                    id: String::from("isle"),
                    label: String::from("Paths"),
                    kind: TreeKind::Path,
                    editing: false,
                    nodes: vec![TreeNode::new("a", "A").children(vec![TreeNode::new("b", "B")])],
                    open: BTreeSet::from([String::from("a")]),
                    selected: Some(String::from("a/b")),
                    open_field: None,
                }
                words=TreeWords::of_page()
            />
        }
        .into_any()
    });
    assert!(
        has(&page, "ul", &[("id", "isle"), ("role", "tree")]),
        "{page}"
    );
    assert!(page.contains(r#"aria-selected="true""#), "{page}");
}

#[test]
fn an_archetype_id_breaks_after_dots_and_a_path_after_slashes_dots_and_brackets() {
    let id = html(|| archetype("openEHR-EHR-OBSERVATION.blood_pressure.v2"));
    assert!(
        has(
            &id,
            "code",
            &[
                ("class", "fui-archetype"),
                ("title", "openEHR-EHR-OBSERVATION.blood_pressure.v2")
            ]
        ),
        "{id}"
    );
    assert!(
        id.contains("<span>openEHR-EHR-OBSERVATION.</span><wbr"),
        "{id}"
    );
    assert_eq!(3, tags(&id, "wbr").len(), "{id}");
    let at = html(|| path("/content[id0.1]/value"));
    assert!(has(&at, "span", &[("class", "fui-path")]), "{at}");
    assert!(at.contains("<span>content[</span><wbr"), "{at}");
    assert!(at.contains("<span>id0.</span><wbr"), "{at}");
    let tricky = html(|| path("/a<b>"));
    assert!(tricky.contains("a&lt;b&gt;"), "a path is escaped: {tricky}");
}

#[test]
fn the_browser_lays_out_its_panes_and_the_inspector_reads_a_coded_node() {
    let three = html(|| {
        browser(
            Panes::Three,
            Heading::Two,
            vec![
                BrowserPane::new(
                    "Search",
                    view! {
                        {pane_body(browser_find(Find {
                            id: "b-q",
                            name: "q",
                            label: "Search the designations",
                            value: "cough",
                            source: StateSource::Url,
                            action: None,
                            fields: &[],
                        }))}
                        {match_list(
                            "Concepts the search matched",
                            vec!["Cough".into_any()],
                        )}
                    },
                ),
                BrowserPane::new("Hierarchy", ()),
                BrowserPane::new("The concept", ()),
            ],
        )
    });
    assert!(has(&three, "div", &[("class", "fui-browser")]), "{three}");
    assert_eq!(3, tags(&three, "section").len(), "{three}");
    assert!(
        has(
            &three,
            "section",
            &[("class", "fui-browser-pane"), ("aria-label", "Hierarchy")]
        ),
        "{three}"
    );
    assert!(
        has(&three, "h2", &[("class", "fui-browser-title")]),
        "{three}"
    );
    assert!(
        has(
            &three,
            "form",
            &[
                ("class", "fui-browser-find"),
                ("role", "search"),
                ("method", "get")
            ]
        ),
        "{three}"
    );
    let flat = html(|| browser(Panes::Flat, Heading::Two, vec![]));
    assert!(
        has(&flat, "div", &[("class", "fui-browser is-two-pane")]),
        "{flat}"
    );
    let two = html(|| {
        browser(
            Panes::Two,
            Heading::Three,
            vec![
                BrowserPane::new("Paths", ()),
                path_node(PathNode {
                    name: String::from("Position"),
                    path: String::from("/content[id0.1]/data[id2]/value"),
                    rm_type: String::from("DV_CODED_TEXT"),
                    occurrences: String::from("0..1"),
                    units: None,
                    flat_path: Some(String::from("vital_signs/blood_pressure/position")),
                    codes: vec!["Standing".into_any(), "Sitting".into_any()],
                    more: None,
                    query: Some(String::from("/query?path=position")),
                }),
            ],
        )
    });
    assert!(
        has(&two, "div", &[("class", "fui-browser is-two")]),
        "{two}"
    );
    assert!(has(&two, "h3", &[("class", "fui-browser-title")]), "{two}");
    for words in [
        "Path",
        "RM type",
        "Occurrences",
        "FLAT path",
        "Value set · 2 codes",
        "Build a query on this path",
    ] {
        assert!(two.contains(words), "{words}: {two}");
    }
    assert!(!two.contains(">Units<"), "no units on a coded node: {two}");
    assert!(has(&two, "a", &[("href", "/query?path=position")]), "{two}");
    assert!(has(&two, "ul", &[("class", "fui-code-list")]), "{two}");
}

#[test]
fn a_path_node_with_no_query_draws_no_build_a_query() {
    let node = html(|| {
        browser(
            Panes::Two,
            Heading::Three,
            vec![path_node(PathNode {
                name: String::from("Name"),
                path: String::from(
                    "/identities[openEHR-DEMOGRAPHIC-PARTY_IDENTITY.person_name.v1]",
                ),
                rm_type: String::from("PARTY_IDENTITY"),
                occurrences: String::from("1..*"),
                units: None,
                flat_path: None,
                codes: Vec::new(),
                more: None,
                query: None,
            })],
        )
    });
    assert!(
        !node.contains("Build a query on this path"),
        "a demographic template's node offers no query: {node}"
    );
    assert!(
        !node.contains("fui-button"),
        "the head holds no action at all: {node}"
    );
    assert!(node.contains("PARTY_IDENTITY"), "the facts stay: {node}");
}

#[test]
fn a_large_value_set_on_a_path_node_ends_in_the_more_row_and_counts_its_total() {
    let node = html(|| {
        browser(
            Panes::Two,
            Heading::Three,
            vec![path_node(PathNode {
                name: String::from("Finding"),
                path: String::from("/content[id0.1]/data[id2]/value"),
                rm_type: String::from("DV_CODED_TEXT"),
                occurrences: String::from("0..1"),
                units: None,
                flat_path: None,
                codes: vec!["Cough".into_any(), "Fever".into_any()],
                more: Some(MoreCodes::of_page(2, 4000, 25, "/node?codes=2")),
                query: Some(String::from("/query")),
            })],
        )
    });
    assert!(node.contains("Value set · 4,000 codes"), "{node}");
    assert!(
        has(&node, "li", &[("class", "fui-code-more")]),
        "the more row is the list's last item: {node}"
    );
    assert!(node.contains("2 of 4,000 codes shown."), "{node}");
    assert!(has(&node, "a", &[("href", "/node?codes=2")]), "{node}");
    assert!(node.contains("Show the next 25"), "{node}");
}

#[test]
fn every_tree_keeps_the_width_of_a_name_its_code_and_its_count() {
    // NOTE: COMPONENTS.md M1: a name never wraps and a deep level scrolls, so nothing overlaps it.
    let every = ":is(.fui-concept-tree, .fui-folder-tree, .fui-path-tree)";
    let rule = format!(
        "  {every} .fui-tree-label,\n  {every} .fui-tree-code,\n  {every} .fui-tree-count,\n  {every} .fui-rm-type {{\n    flex: none;\n  }}"
    );
    assert!(
        STYLESHEET.contains(&rule),
        "a concept tree's name shrinks under its code"
    );
}

#[test]
fn the_match_in_view_is_current_on_its_link_and_no_row_is_selected() {
    let three = html(|| {
        match_list(
            "Concepts the search matched",
            vec![
                coding(
                    Coding::new("SF-1210", "Cough")
                        .href("/c/cough")
                        .current(true),
                    Layout::Inline,
                ),
                coding(
                    Coding::new("SF-1211", "Dry cough").href("/c/dry"),
                    Layout::Inline,
                ),
            ],
        )
    });
    // NOTE: WAI-ARIA 1.2 allows aria-selected on no plain list item; aria-current is global.
    assert!(!three.contains("aria-selected"), "{three}");
    assert!(
        has(
            &three,
            "a",
            &[("href", "/c/cough"), ("aria-current", "true")]
        ),
        "the match in view is current on its link: {three}"
    );
    assert!(
        !has(&three, "a", &[("href", "/c/dry"), ("aria-current", "true")]),
        "{three}"
    );
    assert!(
        STYLESHEET
            .contains("  .fui-match-list > li:has([aria-current]:not([aria-current=\"false\"])) {"),
        "the current match's row takes the tint"
    );
}
