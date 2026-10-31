// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keyboard paths written twice (#548): the tree's walk and moves, the
//! tabs, the menu and the listbox, the work-list, the topology and the
//! masked input's eye, each in the library's Rust and again in the module
//! that answers the keys in the browser, held to one answer over tables of
//! keys and places (`parity` holds the harness).

use std::collections::BTreeSet;
use std::error::Error;

use ferrohealth_ui::i18n::Locale;
use ferrohealth_ui::view::action_menu;
use ferrohealth_ui::view::masked::holds;
use ferrohealth_ui::view::menu;
use ferrohealth_ui::view::shortcuts::tab_at;
use ferrohealth_ui::view::tabs;
use ferrohealth_ui::view::topology::{self, Ask, STEPS, asked, stepped};
use ferrohealth_ui::view::tree::{
    More, Move, Shown, Tree, TreeKind, TreeNode, TreeWords, in_sight, moved, path_key, tree_view,
};
use ferrohealth_ui::view::worklist::{self, Selection, selection};
use serde_json::{Value, json};

use crate::parity::held;

/// The keys every table presses: the arrows, the ends, the activators, a
/// letter and the vowel the work-list walks by, and keys no path takes.
const KEYS: &[&str] = &[
    "ArrowDown",
    "ArrowUp",
    "ArrowLeft",
    "ArrowRight",
    "Home",
    "End",
    "Enter",
    " ",
    "j",
    "k",
    "Escape",
    "Tab",
    "a",
    "",
];

/// A tree with every kind of item: links and plain items, a read level and
/// an unread one, a paged level, a removed item, and keys that hold the
/// separator and the escape of a path key.
fn nodes() -> Vec<TreeNode> {
    vec![
        TreeNode::new("root", "Root").href("/c/root").children(vec![
            TreeNode::new("chest", "Chest").href("/c/chest").paged(
                vec![
                    TreeNode::new("cough", "Cough").href("/c/cough"),
                    TreeNode::new("wheeze", "Wheeze"),
                ],
                More::of(&Locale::english(), 2, 9, 2, "/more"),
            ),
            TreeNode::new("unread", "Unread")
                .toggle_href("/open?unread")
                .unread(),
            TreeNode::new("a/b", "Slash").children(vec![
                TreeNode::new("50%", "Percent").href("/c/percent"),
                TreeNode::new("gone", "Gone").href("/c/gone").removed(),
            ]),
        ]),
        TreeNode::new("leaf", "Leaf"),
        TreeNode::new("linked", "Linked leaf").href("/c/linked"),
        TreeNode::new("empty", "Empty folder").children(Vec::new()),
    ]
}

/// The sets of open items the tables draw the tree with.
fn opens() -> Vec<BTreeSet<String>> {
    let set = |keys: &[&[&str]]| keys.iter().map(|path| path_key(path)).collect();
    vec![
        set(&[]),
        set(&[&["root"]]),
        set(&[&["root"], &["root", "chest"]]),
        set(&[&["root", "chest"]]),
        set(&[&["root"], &["root", "a/b"], &["root", "unread"]]),
        set(&[&["root"], &["root", "chest"], &["root", "a/b"], &["empty"]]),
    ]
}

/// `shown` as the `tree` module reads its items.
fn shown_json(shown: &[Shown]) -> Value {
    json!(
        shown
            .iter()
            .map(|item| json!({
                "key": item.key,
                "parent": item.parent,
                "expandable": item.expandable,
                "open": item.open,
                "link": item.link,
            }))
            .collect::<Vec<_>>()
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn the_tree_items_in_sight_are_the_items_drawn_in_sight() -> Result<(), Box<dyn Error>> {
    let english = Locale::english();
    let mut cases = Vec::new();
    for open in opens() {
        for kind in [TreeKind::Concept, TreeKind::Folder, TreeKind::Path] {
            let drawn = tree_view(&Tree {
                id: String::from("parity"),
                label: String::from("Parity"),
                kind,
                editing: false,
                nodes: nodes(),
                open: open.clone(),
                selected: None,
                open_field: None,
                words: TreeWords::of(&english),
            });
            let open_keys: Vec<&String> = open.iter().collect();
            cases.push((
                json!([drawn.to_string(), open_keys]),
                shown_json(&in_sight(&nodes(), &open)),
            ));
        }
    }
    held(
        "tree",
        "(m, html) => { const holder = document.createElement('div'); holder.innerHTML = html; return m.inSight(holder.querySelector(\"ul[role='tree']\")); }",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_tree_keys_move_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for open in opens() {
        let shown = in_sight(&nodes(), &open);
        for (index, item) in shown.iter().enumerate() {
            for key in KEYS {
                let answer = match moved(key, &item.key, &shown) {
                    Some(Move::Focus(to)) => json!({"focus": to}),
                    Some(Move::Open(to)) => json!({"open": to}),
                    Some(Move::Close(to)) => json!({"close": to}),
                    Some(Move::Follow(to)) => json!({"follow": to}),
                    None => Value::Null,
                };
                cases.push((json!([key, index, shown_json(&shown)]), answer));
            }
        }
    }
    held(
        "tree",
        "(m, key, at, shown) => m.moved(key, at, shown)",
        cases,
    )
    .await
}

/// The digits and the other keys a tab row is pressed by.
const DIGITS: &[&str] = &[
    "0", "1", "2", "5", "9", "10", "\u{661}", "\u{ff11}", "a", "", "1 ",
];

#[tokio::test(flavor = "multi_thread")]
async fn the_tabs_and_the_shortcuts_digits_move_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for count in 0..=10_usize {
        for key in DIGITS {
            cases.push((json!(["tabAt", key, count, 0]), json!(tab_at(key, count))));
        }
        for current in 0..count.max(1) {
            for key in KEYS {
                cases.push((
                    json!(["moved", key, count, current]),
                    json!(tabs::moved(key, current, count)),
                ));
            }
        }
    }
    held(
        "tabs",
        "(m, ask, key, count, current) => ask === 'tabAt' ? m.tabAt(key, count) : m.moved(key, current, count)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_menu_and_the_action_menu_and_the_user_menu_move_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for count in 0..=4_u32 {
        let places = std::iter::once(None).chain((0..count).map(Some));
        for current in places {
            for key in KEYS {
                let wide = usize::try_from(count)?;
                let at = current.map(usize::try_from).transpose()?;
                let rust = action_menu::moved(key, at, wide);
                assert_eq!(
                    rust.map(u32::try_from).transpose()?,
                    menu::step(key, current, count),
                    "the action menu and the user menu move alike in Rust on {key:?} from {current:?} of {count}"
                );
                cases.push((json!([key, current, count]), json!(rust)));
            }
        }
    }
    held(
        "menu",
        "(m, key, current, count) => m.moved(key, current, count)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_menu_and_the_action_menu_find_a_typed_letter_alike() -> Result<(), Box<dyn Error>> {
    let words: Vec<String> = [
        "Copy",
        "  cut",
        "\u{c9}dit",
        "\u{e9}tat",
        "Delete",
        "copy again",
        "\u{130}stanbul",
        "\u{85}next",
        "\u{feff}bom",
        "",
    ]
    .map(String::from)
    .to_vec();
    let letters = [
        "c", "C", "d", "\u{e9}", "\u{c9}", "i", "\u{130}", "n", "b", "x", " ", "", "cu", "\u{85}",
    ];
    let mut cases = Vec::new();
    for letter in letters {
        for current in std::iter::once(None).chain((0..words.len()).map(Some)) {
            cases.push((
                json!([letter, current, words]),
                json!(action_menu::typed(letter, current, &words)),
            ));
        }
    }
    held(
        "menu",
        "(m, letter, current, words) => m.typed(letter, current, words)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_menu_and_the_action_menu_flip_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for (top, bottom) in [(0.0, 30.0), (100.0, 130.0), (700.0, 730.0), (770.0, 800.0)] {
        for (left, right, height) in [
            (10.0, 210.0, 100.0),
            (-1.0, 199.0, 200.0),
            (1100.0, 1300.0, 700.0),
            (0.0, 1280.0, 770.0),
        ] {
            let (up, side) =
                action_menu::flips((top, bottom), (left, right, height), (1280.0, 800.0));
            cases.push((
                json!([
                    {"top": top, "bottom": bottom},
                    {"left": left, "right": right, "height": height},
                    {"width": 1280.0, "height": 800.0}
                ]),
                json!({"up": up, "side": side}),
            ));
        }
    }
    held(
        "menu",
        "(m, anchor, popup, viewport) => m.flips(anchor, popup, viewport)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_worklist_rows_move_and_select_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for count in 0..=4_usize {
        for current in 0..count.max(1) {
            for key in KEYS {
                cases.push((
                    json!(["moved", key, current, count]),
                    json!(worklist::moved(key, current, count)),
                ));
            }
        }
        for checked in 0..=count.saturating_add(1) {
            let named = match selection(checked, count) {
                Selection::None => "none",
                Selection::Some => "some",
                Selection::All => "all",
            };
            cases.push((json!(["selection", "", checked, count]), json!(named)));
        }
    }
    held(
        "worklist",
        "(m, ask, key, a, count) => ask === 'moved' ? m.moved(key, a, count) : m.selection(a, count)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_topology_zooms_asks_and_moves_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = vec![(json!(["steps"]), json!(STEPS))];
    let scales = [
        0.1, 0.25, 0.254, 0.256, 0.3, 0.33, 0.5, 0.995, 1.0, 1.004, 1.006, 1.9, 2.0, 3.0,
    ];
    for scale in scales {
        for inward in [true, false] {
            cases.push((
                json!(["stepped", scale, inward]),
                json!(stepped(scale, inward)),
            ));
        }
    }
    for key in KEYS.iter().chain(&["+", "=", "-", "_", "0", "i"]) {
        for on_card in [true, false] {
            let ask = asked(key, on_card).map(|ask| match ask {
                Ask::ZoomIn => "in",
                Ask::ZoomOut => "out",
                Ask::Fit => "fit",
                Ask::Look => "look",
            });
            cases.push((json!(["asked", key, on_card]), json!(ask)));
        }
        for count in 0..=3_usize {
            for at in 0..count.max(1) {
                cases.push((
                    json!(["moved", key, at, count]),
                    json!(topology::moved(key, at, count)),
                ));
            }
        }
    }
    held(
        "topology",
        "(m, ask, a, b, c) => ask === 'steps' ? m.STEPS : ask === 'stepped' ? m.stepped(a, b) : ask === 'asked' ? m.asked(a, b) : m.moved(a, b, c)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_masked_eye_is_held_by_the_same_keys() -> Result<(), Box<dyn Error>> {
    let cases = KEYS
        .iter()
        .chain(&["Spacebar", "space", "enter", "NumpadEnter"])
        .map(|key| (json!([key]), json!(holds(key))))
        .collect();
    held("masked", "(m, key) => m.holds(key)", cases).await
}
