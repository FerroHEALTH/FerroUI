// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keys and paths of a tree's nodes: what is in sight, where a key moves
//! and how a path is written.

use std::collections::BTreeSet;

use super::TreeNode;

/// One item in sight, in the order the keyboard walks them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    /// Its path key.
    pub key: String,
    /// Its parent's path key.
    pub parent: Option<String>,
    /// Whether it can open.
    pub expandable: bool,
    /// Whether it is open.
    pub open: bool,
    /// Whether it is a link.
    pub link: bool,
}

/// Returns the items of `nodes` in sight when the items `expanded` are
/// open, in document order: the walk the `tree` module's keys take over the
/// items drawn.
#[must_use]
pub fn in_sight(nodes: &[TreeNode], expanded: &BTreeSet<String>) -> Vec<Shown> {
    fn walk(
        nodes: &[TreeNode],
        parent: Option<&str>,
        expanded: &BTreeSet<String>,
        out: &mut Vec<Shown>,
    ) {
        for node in nodes {
            let key = below(parent, &node.key);
            let open = node.opens() && expanded.contains(&key);
            out.push(Shown {
                key: key.clone(),
                parent: parent.map(str::to_owned),
                expandable: node.opens(),
                open,
                link: node.href.is_some() && !node.removed,
            });
            if open {
                walk(node.read(), Some(&key), expanded, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(nodes, None, expanded, &mut out);
    out
}

/// The separator between the keys of a path key.
const SEPARATOR: char = '/';

/// The key `key` as one step of a path key: a `%` and the separator
/// percent-encoded, so no key can be read as two.
pub(super) fn step(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    for character in key.chars() {
        match character {
            '%' => out.push_str("%25"),
            SEPARATOR => out.push_str("%2F"),
            other => out.push(other),
        }
    }
    out
}

/// The path key of the item `key` under the item `parent`, or at the top
/// for `None`.
pub(super) fn below(parent: Option<&str>, key: &str) -> String {
    match parent {
        Some(parent) => format!("{parent}{SEPARATOR}{}", step(key)),
        None => step(key),
    }
}

/// Returns the path key of the item reached by `keys`, from the top item's
/// key to the item's own.
///
/// It is the keys joined by `/`, with a `%` and a `/` inside a key
/// percent-encoded, so a key holding either still makes a distinct path
/// key. A top item's path key is its key when the key holds neither.
///
/// # Examples
///
/// ```
/// use ferrohealth_ui::view::tree::path::path_key;
///
/// assert_eq!("SF-1000", path_key(&["SF-1000"]));
/// assert_eq!("SF-1000/SF-1200", path_key(&["SF-1000", "SF-1200"]));
/// assert_eq!("a%2Fb/c", path_key(&["a/b", "c"]));
/// ```
#[must_use]
pub fn path_key<S: AsRef<str>>(keys: &[S]) -> String {
    keys.iter().fold(String::new(), |path, key| {
        if path.is_empty() {
            step(key.as_ref())
        } else {
            below(Some(&path), key.as_ref())
        }
    })
}

/// Returns the keys a path key `path` joins, from the top item's key: the
/// inverse of [`path_key`], for a console reading the open fields a tree
/// posts.
///
/// A `%` not followed by `25` or `2F` is kept as it is.
#[must_use]
pub fn path_keys(path: &str) -> Vec<String> {
    if path.is_empty() {
        return Vec::new();
    }
    path.split(SEPARATOR)
        .map(|step| step.replace("%2F", "/").replace("%25", "%"))
        .collect()
}

/// The item at the path key `path` among `nodes` and the children read
/// under them.
pub(super) fn find<'a>(nodes: &'a [TreeNode], path: &str) -> Option<&'a TreeNode> {
    let mut level = nodes;
    let mut found = None;
    for key in path_keys(path) {
        let node = level.iter().find(|node| node.key == key)?;
        level = node.read();
        found = Some(node);
    }
    found
}

/// The children of the item at the path key `path` in `nodes`, or the top
/// items for `None`.
#[cfg(test)]
pub(super) fn children_of(nodes: &[TreeNode], path: Option<&str>) -> Vec<TreeNode> {
    match path {
        None => nodes.to_vec(),
        Some(path) => find(nodes, path)
            .map(|node| node.read().to_vec())
            .unwrap_or_default(),
    }
}

/// The path keys of the items above the item at the path key `path` in
/// `nodes`, from the top, or `None` when no item is there.
pub(super) fn path_to(nodes: &[TreeNode], path: &str) -> Option<Vec<String>> {
    find(nodes, path)?;
    let keys = path_keys(path);
    let above = keys.len().saturating_sub(1);
    Some(
        (1..=above)
            .map(|depth| path_key(keys.get(..depth).unwrap_or_default()))
            .collect(),
    )
}

/// What a key press does to the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    /// Move the focus to the item.
    Focus(String),
    /// Open the item.
    Open(String),
    /// Close the item.
    Close(String),
    /// Follow the item's link.
    Follow(String),
}

/// Returns the move the key `key` makes from the item `at` among the items
/// `shown`, or `None` for a key the tree leaves alone.
///
/// The `tree` module moves the same way (WAI-ARIA Authoring Practices 1.2,
/// tree view, keyboard interaction).
#[must_use]
pub fn moved(key: &str, at: &str, shown: &[Shown]) -> Option<Move> {
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
pub(super) fn item_id(tree: &str, key: &str) -> String {
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
