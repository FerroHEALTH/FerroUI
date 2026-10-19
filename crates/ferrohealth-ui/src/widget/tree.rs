// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tree view: nested items that fold, one tab stop, and the arrow keys
//! (WAI-ARIA Authoring Practices 1.2, the tree view pattern).
//!
//! The tree is a `role="tree"` list of `treeitem`s, each child list a
//! `group`, every item carrying `aria-level`, the selected one
//! `aria-selected="true"`, and an item that opens `aria-expanded`. The
//! keyboard reaches the tree once (a roving `tabindex`): Down and Up move
//! between the items in sight, Home and End to the ends, Right opens an item
//! or moves to its first child, Left closes it or moves to its parent, and
//! Enter or Space follows an item's link or folds it. A row may be a link the
//! console gives, so selecting works before the page hydrates and the
//! selection is shareable; children may arrive later, so a row can open
//! before any child is read. The indentation comes from the nesting, never a
//! style attribute. Keys are the console's data-derived ids, never
//! positions.

use std::collections::BTreeSet;

use std::borrow::Cow;

use leptos::prelude::*;

use crate::icon::{Glyph, icon};

/// One item of a tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    /// The item's stable key, derived from its data.
    pub key: String,
    /// Its words.
    pub label: String,
    /// A code shown muted after the words, such as a concept's code.
    pub code: Option<String>,
    /// The page the item opens, when it is a link.
    pub href: Option<String>,
    /// Whether it can open, even before its children are read.
    pub expandable: bool,
    /// The children read so far.
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// A leaf `key` with the words `label`.
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            code: None,
            href: None,
            expandable: false,
            children: Vec::new(),
        }
    }

    /// This item, with the code `code`.
    #[must_use]
    pub fn code(self, code: impl Into<String>) -> Self {
        Self {
            code: Some(code.into()),
            ..self
        }
    }

    /// This item, a link to `href`.
    #[must_use]
    pub fn href(self, href: impl Into<String>) -> Self {
        Self {
            href: Some(href.into()),
            ..self
        }
    }

    /// This item, with the children `children`; it can open.
    #[must_use]
    pub fn children(self, children: Vec<Self>) -> Self {
        Self {
            expandable: true,
            children,
            ..self
        }
    }

    /// This item, able to open before its children are read.
    #[must_use]
    pub fn expandable(self) -> Self {
        Self {
            expandable: true,
            ..self
        }
    }
}

/// What a tree shows and the state it shares with its console.
#[derive(Debug, Clone)]
pub struct Tree {
    /// The tree's id; each item's id is built from it.
    pub id: &'static str,
    /// The tree's accessible name.
    pub label: Cow<'static, str>,
    /// The items at the top.
    pub nodes: Signal<Vec<TreeNode>>,
    /// The keys of the open items; the server draws the tree as given.
    pub expanded: RwSignal<BTreeSet<String>>,
    /// The key of the selected item.
    pub selected: Signal<Option<String>>,
    /// Called with an item's key when it opens, so the console can read its
    /// children.
    pub on_expand: Option<Callback<String>>,
    /// The actions at the end of an item's row, such as rename and delete.
    pub actions: Option<Callback<TreeNode, AnyView>>,
}

/// The part of a [`Tree`] its items and handlers share: all of it but the
/// accessible name, so it is `Copy`.
#[derive(Debug, Clone, Copy)]
struct State {
    id: &'static str,
    nodes: Signal<Vec<TreeNode>>,
    expanded: RwSignal<BTreeSet<String>>,
    selected: Signal<Option<String>>,
    on_expand: Option<Callback<String>>,
    actions: Option<Callback<TreeNode, AnyView>>,
}

impl State {
    /// The state of `tree`, and its accessible name.
    fn of(tree: Tree) -> (Self, Cow<'static, str>) {
        let Tree {
            id,
            label,
            nodes,
            expanded,
            selected,
            on_expand,
            actions,
        } = tree;
        (
            Self {
                id,
                nodes,
                expanded,
                selected,
                on_expand,
                actions,
            },
            label,
        )
    }
}

/// One item in sight, in the order the keyboard walks them.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Shown {
    /// Its key.
    key: String,
    /// Its parent's key.
    parent: Option<String>,
    /// Whether it can open.
    expandable: bool,
    /// Whether it is open.
    open: bool,
    /// Whether it is a link.
    link: bool,
}

/// The items of `nodes` in sight when the items `expanded` are open, in
/// document order.
fn in_sight(nodes: &[TreeNode], expanded: &BTreeSet<String>) -> Vec<Shown> {
    fn walk(
        nodes: &[TreeNode],
        parent: Option<&str>,
        expanded: &BTreeSet<String>,
        out: &mut Vec<Shown>,
    ) {
        for node in nodes {
            let open = node.expandable && expanded.contains(&node.key);
            out.push(Shown {
                key: node.key.clone(),
                parent: parent.map(str::to_owned),
                expandable: node.expandable,
                open,
                link: node.href.is_some(),
            });
            if open {
                walk(&node.children, Some(&node.key), expanded, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(nodes, None, expanded, &mut out);
    out
}

/// The children of the item `key` in `nodes`, or the top items for `None`.
fn children_of(nodes: &[TreeNode], key: Option<&str>) -> Vec<TreeNode> {
    fn find<'a>(nodes: &'a [TreeNode], key: &str) -> Option<&'a TreeNode> {
        nodes.iter().find_map(|node| {
            if node.key == key {
                Some(node)
            } else {
                find(&node.children, key)
            }
        })
    }
    match key {
        None => nodes.to_vec(),
        Some(key) => find(nodes, key)
            .map(|node| node.children.clone())
            .unwrap_or_default(),
    }
}

/// What a key press does to the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Move {
    /// Move the focus to the item.
    Focus(String),
    /// Open the item.
    Open(String),
    /// Close the item.
    Close(String),
    /// Follow the item's link.
    Follow(String),
}

/// The move the key `key` makes from the item `at` among the items `shown`,
/// or `None` for a key the tree leaves alone.
fn moved(key: &str, at: &str, shown: &[Shown]) -> Option<Move> {
    let index = shown.iter().position(|item| item.key == at)?;
    let item = shown.get(index)?;
    let focus = |item: Option<&Shown>| item.map(|item| Move::Focus(item.key.clone()));
    match key {
        "ArrowDown" => focus(shown.get(index.saturating_add(1))),
        "ArrowUp" => index
            .checked_sub(1)
            .and_then(|before| focus(shown.get(before))),
        "Home" => focus(shown.first()),
        "End" => focus(shown.last()),
        "ArrowRight" if item.expandable && !item.open => Some(Move::Open(item.key.clone())),
        "ArrowRight" if item.open => focus(
            shown
                .get(index.saturating_add(1))
                .filter(|next| next.parent.as_deref() == Some(at)),
        ),
        "ArrowLeft" if item.open => Some(Move::Close(item.key.clone())),
        "ArrowLeft" => item.parent.clone().map(Move::Focus),
        "Enter" | " " if item.link => Some(Move::Follow(item.key.clone())),
        "Enter" | " " if item.open => Some(Move::Close(item.key.clone())),
        "Enter" | " " if item.expandable => Some(Move::Open(item.key.clone())),
        _ => None,
    }
}

/// The id of the item `key` of the tree `tree`: the key's bytes in hex, so
/// any key makes a valid, distinct id.
fn item_id(tree: &str, key: &str) -> String {
    let mut id = format!("{tree}-item-");
    for byte in key.bytes() {
        for digit in [byte >> 4, byte & 0x0f] {
            if let Some(digit) = char::from_digit(u32::from(digit), 16) {
                id.push(digit);
            }
        }
    }
    id
}

/// Opens the item `key` of `tree` and tells the console.
fn open(tree: State, key: &str) {
    tree.expanded.update(|open| {
        open.insert(key.to_owned());
    });
    if let Some(on_expand) = tree.on_expand {
        on_expand.run(key.to_owned());
    }
}

/// Moves the keyboard focus to the element `id`.
fn focus_element(id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let element = document()
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(element) = element
            && element.focus().is_err()
        {
            leptos::logging::warn!("the browser refused the focus on a tree item");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = id;
}

/// Follows the link of the item whose element is `id`.
fn follow(id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let link = document()
            .get_element_by_id(id)
            .and_then(|item| item.query_selector("a").ok().flatten())
            .and_then(|link| link.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(link) = link {
            link.click();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = id;
}

/// Returns the items under `parent` (the top for `None`) at `level`.
fn items(
    tree: State,
    parent: Option<String>,
    level: u16,
    focused: RwSignal<Option<String>>,
) -> AnyView {
    let each = move || {
        tree.nodes
            .with(|nodes| children_of(nodes, parent.as_deref()))
    };
    view! {
        <For each=each key=|node: &TreeNode| node.key.clone() let:node>
            {item(tree, &node, level, focused)}
        </For>
    }
    .into_any()
}

/// Returns the item `node` at `level` and, while it is open, its children.
/// Returns the button that folds and unfolds the item `key`, out of the tab
/// order (the arrow keys fold an item), or a spacer for an item that cannot
/// open.
fn toggle(
    tree: State,
    key: String,
    open: Option<impl Fn() -> bool + Clone + Send + Sync + 'static>,
) -> AnyView {
    let Some(open) = open else {
        return view! { <span class="fui-tree-spacer"></span> }.into_any();
    };
    let shown = open.clone();
    view! {
        <button
            type="button"
            class="fui-tree-toggle"
            tabindex="-1"
            aria-hidden="true"
            on:click=move |_| {
                if open() {
                    tree.expanded.update(|expanded| {
                        expanded.remove(&key);
                    });
                } else {
                    self::open(tree, &key);
                }
            }
        >
            {move || icon(if shown() { Glyph::CHEVRON_DOWN } else { Glyph::CHEVRON_RIGHT }, "14")}
        </button>
    }
    .into_any()
}

/// Returns the item `node` at `level` and, while it is open, its children.
fn item(tree: State, node: &TreeNode, level: u16, focused: RwSignal<Option<String>>) -> AnyView {
    let key = node.key.clone();
    let id = item_id(tree.id, &key);
    let expandable = node.expandable;
    // NOTE: no specification governs this: our own design; a memo per item, so
    // opening one item never rebuilds the child list of another.
    let open = {
        let key = key.clone();
        let memo = Memo::new(move |_| expandable && tree.expanded.with(|open| open.contains(&key)));
        move || memo.get()
    };
    let selected = {
        let key = key.clone();
        move || {
            tree.selected
                .with(|selected| selected.as_deref() == Some(key.as_str()))
        }
    };
    let stop = {
        let key = key.clone();
        move || {
            let at = focused.get().or_else(|| tree.selected.get()).or_else(|| {
                tree.nodes
                    .with(|nodes| nodes.first().map(|node| node.key.clone()))
            });
            if at.as_deref() == Some(key.as_str()) {
                "0"
            } else {
                "-1"
            }
        }
    };
    let toggle = toggle(tree, key.clone(), expandable.then_some(open));
    let words = match node.href.clone() {
        Some(href) => view! {
            <a href=href class="fui-tree-label" tabindex="-1">
                {node.label.clone()}
            </a>
        }
        .into_any(),
        None => view! { <span class="fui-tree-label">{node.label.clone()}</span> }.into_any(),
    };
    let code = node
        .code
        .clone()
        .map(|code| view! { <code class="fui-tree-code">{code}</code> });
    let actions = tree
        .actions
        .map(|actions| view! { <span class="fui-tree-actions">{actions.run(node.clone())}</span> });
    let children = {
        let key = key.clone();
        move || {
            open().then(|| {
                view! {
                    <ul role="group" class="fui-tree-group">
                        {items(tree, Some(key.clone()), level.saturating_add(1), focused)}
                    </ul>
                }
            })
        }
    };
    let expanded = { move || expandable.then(|| open().to_string()) };
    let focus_key = key.clone();
    view! {
        <li
            id=id
            role="treeitem"
            class="fui-tree-item"
            aria-expanded=expanded
            aria-selected=move || selected().to_string()
            tabindex=stop
            on:focusin=move |event| {
                event.stop_propagation();
                focused.set(Some(focus_key.clone()));
            }
        >
            <div class="fui-tree-row">{toggle} {words} {code} {actions}</div>
            {children}
        </li>
    }
    // Leptos names no typed `aria-level` attribute, so it is set by name.
    .attr("aria-level", level.to_string())
    .into_any()
}

/// Returns the tree [`tree_view`] draws, telling `on_collapse` what closes.
///
/// `on_collapse` receives an item's key whenever it closes, by the toggle,
/// the keyboard or the console, so a console that keeps the open set in its
/// address can remove it.
#[must_use]
pub fn tree_view_collapsing(tree: Tree, on_collapse: Callback<String>) -> AnyView {
    let expanded = tree.expanded;
    Effect::new(move |previous: Option<BTreeSet<String>>| {
        let now = expanded.get();
        if let Some(previous) = previous {
            for key in previous.difference(&now) {
                on_collapse.run(key.clone());
            }
        }
        now
    });
    tree_view(tree)
}

/// Returns the tree `tree`.
#[must_use]
pub fn tree_view(tree: Tree) -> AnyView {
    let (tree, label) = State::of(tree);
    let focused = RwSignal::new(None::<String>);
    let on_key = move |event: leptos::ev::KeyboardEvent| {
        let Some(at) = focused
            .get_untracked()
            .or_else(|| tree.selected.get_untracked())
        else {
            return;
        };
        let shown = tree
            .nodes
            .with_untracked(|nodes| tree.expanded.with_untracked(|open| in_sight(nodes, open)));
        let Some(step) = moved(&event.key(), &at, &shown) else {
            return;
        };
        event.prevent_default();
        match step {
            Move::Focus(key) => {
                focus_element(&item_id(tree.id, &key));
                focused.set(Some(key));
            }
            Move::Open(key) => open(tree, &key),
            Move::Close(key) => tree.expanded.update(|open| {
                open.remove(&key);
            }),
            Move::Follow(key) => follow(&item_id(tree.id, &key)),
        }
    };
    view! {
        <ul id=tree.id role="tree" aria-label=label class="fui-tree" on:keydown=on_key>
            {items(tree, None, 1, focused)}
        </ul>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Move, TreeNode, children_of, in_sight, item_id, moved};

    fn nodes() -> Vec<TreeNode> {
        vec![
            TreeNode::new("a", "Body site").children(vec![
                TreeNode::new("a1", "Arm").href("/c/a1"),
                TreeNode::new("a2", "Leg").expandable(),
            ]),
            TreeNode::new("b", "Finding"),
        ]
    }

    #[test]
    fn the_items_in_sight_follow_what_is_open() {
        let closed = in_sight(&nodes(), &BTreeSet::new());
        let keys: Vec<&str> = closed.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(vec!["a", "b"], keys);
        let open = BTreeSet::from([String::from("a")]);
        let keys: Vec<String> = in_sight(&nodes(), &open)
            .into_iter()
            .map(|item| item.key)
            .collect();
        assert_eq!(vec!["a", "a1", "a2", "b"], keys);
        assert_eq!(2, children_of(&nodes(), Some("a")).len());
        assert!(children_of(&nodes(), Some("a2")).is_empty(), "not read yet");
    }

    #[test]
    fn the_keys_walk_the_tree_as_the_pattern_has_it() {
        let open = BTreeSet::from([String::from("a")]);
        let shown = in_sight(&nodes(), &open);
        let focus = |key: &str| Some(Move::Focus(key.to_owned()));
        assert_eq!(focus("a1"), moved("ArrowDown", "a", &shown));
        assert_eq!(focus("a"), moved("ArrowUp", "a1", &shown));
        assert_eq!(
            None,
            moved("ArrowUp", "a", &shown),
            "no item above the first"
        );
        assert_eq!(focus("b"), moved("End", "a", &shown));
        assert_eq!(focus("a"), moved("Home", "b", &shown));
        assert_eq!(
            focus("a1"),
            moved("ArrowRight", "a", &shown),
            "into an open item"
        );
        assert_eq!(
            Some(Move::Open(String::from("a2"))),
            moved("ArrowRight", "a2", &shown)
        );
        assert_eq!(
            Some(Move::Close(String::from("a"))),
            moved("ArrowLeft", "a", &shown)
        );
        assert_eq!(
            focus("a"),
            moved("ArrowLeft", "a1", &shown),
            "to the parent"
        );
        assert_eq!(
            Some(Move::Follow(String::from("a1"))),
            moved("Enter", "a1", &shown)
        );
        assert_eq!(
            Some(Move::Open(String::from("a2"))),
            moved(" ", "a2", &shown)
        );
        assert_eq!(None, moved("x", "a", &shown));
    }

    #[test]
    fn every_key_makes_a_distinct_valid_id() {
        assert_eq!("t-item-6120622f63", item_id("t", "a b/c"));
        assert_ne!(item_id("t", "a b"), item_id("t", "a_b"));
    }
}
