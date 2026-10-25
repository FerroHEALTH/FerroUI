// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tree view: nested items that fold, one tab stop, and the arrow keys.
//!
//! It follows the tree view pattern of the WAI-ARIA Authoring Practices 1.2,
//! drawn as M1's concept tree, E4's folder tree or E5's path tree
//! (`design/COMPONENTS.md`).
//!
//! The tree is a `role="tree"` list of `treeitem`s, each child list a
//! `group`, every item carrying `aria-level`, the selected one
//! `aria-selected="true"`, and an item that opens `aria-expanded`. The
//! keyboard reaches the tree once (a roving `tabindex`): Down and Up move
//! between the items in sight, Home and End to the ends, Right opens an item
//! or moves to its first child, Left closes it or moves to its parent, and
//! Enter or Space follows an item's link or folds it. The focus ring is on
//! the row. A name is a name in a list (family rule 4) that never wraps, so
//! a deep level scrolls sideways under the scroll cue.
//!
//! A tree holds plain data ([`TreeNode`]), so the ready island
//! [`crate::island::TreeView`] takes it as props. A level is read by the
//! console's server, one at a time: an item whose children are not read yet
//! is [`Children::Unread`], and opening it follows its toggle's link, which
//! the console answers with the level read; meanwhile the level shows its
//! loading row. A level larger than a page ends in the more row ([`More`]).
//! The open items are state the server can read: the toggle is a link to the
//! page with the item opened or closed, so a tree folds with no
//! WebAssembly, and a tree in a form can post its open items as hidden
//! fields. A console that keeps the open items in its address watches
//! [`Tree::expanded`] with `Effect::watch` (the Leptos book,
//! `reactivity/14_create_effect`).
//!
//! An item's key is the console's data-derived id, never a position, and is
//! unique among its siblings. The tree addresses an item by its path key
//! ([`path_key`]): the keys from the top to it, so the same concept under
//! two parents (a polyhierarchy, as in SNOMED CT) is two items, each with
//! its own id, its own fold, its own selection and its own place in the
//! keyboard's walk. [`Tree::expanded`], [`Tree::selected`] and the open
//! fields hold path keys; a top item's path key is its key.

use std::collections::BTreeSet;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};

/// The children of an item.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Children {
    /// A leaf: the item cannot open.
    #[default]
    None,
    /// The item can open, and its children are not read yet: opening it
    /// shows the loading row and follows its toggle's link.
    Unread,
    /// The children read so far, and the more row when the level ends
    /// before its last child.
    Read {
        /// The children, in the order the level reads them.
        nodes: Vec<TreeNode>,
        /// The more row, when the server pages the level.
        more: Option<More>,
    },
}

/// The more row that ends a level larger than a page: "25 of 40 children
/// shown. Show the other 15".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct More {
    /// The sentence that counts the children shown.
    pub words: Word,
    /// The link's words.
    pub link: Word,
    /// The page with the next children read.
    pub href: String,
}

impl More {
    /// The more row of a level that shows `shown` of its `total` children,
    /// reading `next` more at `href`, in the locale of the page being drawn.
    #[must_use]
    pub fn of_page(shown: u32, total: u32, next: u32, href: impl Into<String>) -> Self {
        let rest = total.saturating_sub(shown);
        let link = if next >= rest {
            tr_with(Text::TreeMoreRest, &[("count", rest.into())])
        } else {
            tr_with(Text::TreeMoreNext, &[("count", next.into())])
        };
        Self {
            words: tr_with(
                Text::TreeMore,
                &[("shown", shown.into()), ("total", total.into())],
            ),
            link,
            href: href.into(),
        }
    }
}

/// What an item of a folder tree is (E4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mark {
    /// A folder, with the folder glyph.
    Folder,
    /// A reference, with the file glyph in the accent.
    Reference,
}

/// An action at the end of an item's row in an edit (E4): a submit button
/// of the console's form, named [`Edit::name`] with the item's key as its
/// value, so the edit posts with no WebAssembly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edit {
    /// Add a folder in the folder.
    AddFolder,
    /// Add a reference in the folder.
    AddReference,
    /// Rename the folder.
    Rename,
    /// Remove the item.
    Remove,
    /// Keep an item an edit removed.
    Undo,
}

impl Edit {
    /// The name of the action's button, the field the console's form reads.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AddFolder => "add_folder",
            Self::AddReference => "add_reference",
            Self::Rename => "rename",
            Self::Remove => "remove",
            Self::Undo => "undo",
        }
    }

    /// The action's glyph.
    const fn glyph(self) -> Glyph {
        match self {
            Self::AddFolder => Glyph::FOLDER_PLUS,
            Self::AddReference => Glyph::FILE_TEXT,
            Self::Rename => Glyph::PENCIL,
            Self::Remove => Glyph::TRASH,
            Self::Undo => Glyph::UNDO,
        }
    }
}

/// One item of a tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeNode {
    /// The item's stable key, derived from its data and unique among its
    /// siblings; the tree addresses the item by its [`path_key`].
    pub key: String,
    /// Its name.
    pub label: String,
    /// A code shown muted in mono after the name, such as a concept's code
    /// or a reference's short uid; it never breaks.
    pub code: Option<String>,
    /// A quiet mono tag after the name, such as a path's RM type (E5).
    pub tag: Option<String>,
    /// The count after a folder's name, in words ("2 items").
    pub count: Option<String>,
    /// The glyph before the name, in a folder tree.
    pub mark: Option<Mark>,
    /// Whether the concept is inactive: muted, with M2's inactive mark.
    pub inactive: bool,
    /// Whether an edit removed the item: struck through, and said so.
    pub removed: bool,
    /// The name being typed, while the item is renamed in an edit.
    pub rename: Option<String>,
    /// The actions at the end of its row while the tree is in an edit.
    pub edits: Vec<Edit>,
    /// The page the item opens, when it is a link.
    pub href: Option<String>,
    /// The page with this item opened, or closed when it is open, so the
    /// tree folds with no WebAssembly and the console reads an unread level.
    pub toggle_href: Option<String>,
    /// Its children.
    pub children: Children,
}

impl TreeNode {
    /// A leaf `key` named `label`.
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            code: None,
            tag: None,
            count: None,
            mark: None,
            inactive: false,
            removed: false,
            rename: None,
            edits: Vec::new(),
            href: None,
            toggle_href: None,
            children: Children::None,
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

    /// This item, with the tag `tag`.
    #[must_use]
    pub fn tag(self, tag: impl Into<String>) -> Self {
        Self {
            tag: Some(tag.into()),
            ..self
        }
    }

    /// This item, with the count `count`.
    #[must_use]
    pub fn count(self, count: impl Into<String>) -> Self {
        Self {
            count: Some(count.into()),
            ..self
        }
    }

    /// This item, marked `mark`.
    #[must_use]
    pub fn mark(self, mark: Mark) -> Self {
        Self {
            mark: Some(mark),
            ..self
        }
    }

    /// This item, inactive.
    #[must_use]
    pub fn inactive(self) -> Self {
        Self {
            inactive: true,
            ..self
        }
    }

    /// This item, removed by an edit.
    #[must_use]
    pub fn removed(self) -> Self {
        Self {
            removed: true,
            ..self
        }
    }

    /// This item, being renamed to `name`.
    #[must_use]
    pub fn renaming(self, name: impl Into<String>) -> Self {
        Self {
            rename: Some(name.into()),
            ..self
        }
    }

    /// This item, with the actions `edits` in an edit.
    #[must_use]
    pub fn edits(self, edits: Vec<Edit>) -> Self {
        Self { edits, ..self }
    }

    /// This item, a link to `href`.
    #[must_use]
    pub fn href(self, href: impl Into<String>) -> Self {
        Self {
            href: Some(href.into()),
            ..self
        }
    }

    /// This item, whose toggle leads to `href`.
    #[must_use]
    pub fn toggle_href(self, href: impl Into<String>) -> Self {
        Self {
            toggle_href: Some(href.into()),
            ..self
        }
    }

    /// This item, with the children `children` read; it can open.
    #[must_use]
    pub fn children(self, children: Vec<Self>) -> Self {
        Self {
            children: Children::Read {
                nodes: children,
                more: None,
            },
            ..self
        }
    }

    /// This item, with the children `children` read and the more row
    /// `more` after them.
    #[must_use]
    pub fn paged(self, children: Vec<Self>, more: More) -> Self {
        Self {
            children: Children::Read {
                nodes: children,
                more: Some(more),
            },
            ..self
        }
    }

    /// This item, able to open before its children are read.
    #[must_use]
    pub fn unread(self) -> Self {
        Self {
            children: Children::Unread,
            ..self
        }
    }

    /// Whether the item can open.
    #[must_use]
    pub const fn opens(&self) -> bool {
        !matches!(self.children, Children::None)
    }

    /// The children read so far.
    fn read(&self) -> &[Self] {
        match &self.children {
            Children::Read { nodes, .. } => nodes,
            Children::None | Children::Unread => &[],
        }
    }
}

/// Which design a tree draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TreeKind {
    /// M1's concept tree.
    #[default]
    Concept,
    /// E4's folder tree of a directory.
    Folder,
    /// E5's path tree of a template.
    Path,
}

/// The words a tree draws itself, resolved on the server in the page's
/// locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeWords {
    /// The name of a toggle that opens an item, around its `label`.
    pub expand: Phrase,
    /// The name of a toggle that closes an item, around its `label`.
    pub collapse: Phrase,
    /// An inactive concept's mark.
    pub inactive: Word,
    /// The loading row of a level being read.
    pub reading: Word,
    /// What a removed item says to assistive technology.
    pub removed: Word,
    /// The name of the rename field, around the item's `label`.
    pub rename_field: Phrase,
    /// The name of [`Edit::AddFolder`], around the item's `label`.
    pub add_folder: Phrase,
    /// The name of [`Edit::AddReference`], around the item's `label`.
    pub add_reference: Phrase,
    /// The name of [`Edit::Rename`], around the item's `label`.
    pub rename: Phrase,
    /// The name of [`Edit::Remove`] on a folder, around its `label`.
    pub remove: Phrase,
    /// The name of [`Edit::Remove`] on a reference, around its `label`.
    pub remove_reference: Phrase,
    /// The name of [`Edit::Undo`], around the item's `label`.
    pub undo: Phrase,
}

impl TreeWords {
    /// The tree's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        let around = |text: Text| Phrase::new(tr_with(text, &[("label", Phrase::open("label"))]));
        Self {
            expand: around(Text::TreeExpand),
            collapse: around(Text::TreeCollapse),
            inactive: tr(Text::TreeInactive),
            reading: tr(Text::TreeReading),
            removed: tr(Text::TreeRemoved),
            rename_field: around(Text::TreeRenameField),
            add_folder: around(Text::TreeEditAddFolder),
            add_reference: around(Text::TreeEditAddReference),
            rename: around(Text::TreeEditRename),
            remove: around(Text::TreeEditRemove),
            remove_reference: around(Text::TreeEditRemoveReference),
            undo: around(Text::TreeEditUndo),
        }
    }

    /// The name of the action `edit` on the item `node`.
    fn edit(&self, edit: Edit, node: &TreeNode) -> Word {
        let phrase = match edit {
            Edit::AddFolder => &self.add_folder,
            Edit::AddReference => &self.add_reference,
            Edit::Rename => &self.rename,
            Edit::Remove if node.mark == Some(Mark::Reference) => &self.remove_reference,
            Edit::Remove => &self.remove,
            Edit::Undo => &self.undo,
        };
        phrase.fill(&[("label", &node.label)])
    }
}

/// What a tree shows and the state it shares with its console.
#[derive(Debug, Clone)]
pub struct Tree {
    /// The tree's id, on its `role="tree"` list; each item's id is built
    /// from it.
    pub id: String,
    /// The tree's accessible name.
    pub label: String,
    /// Which design it draws.
    pub kind: TreeKind,
    /// Whether it is in an edit: each row ends in its [`TreeNode::edits`].
    pub editing: bool,
    /// The items at the top.
    pub nodes: Signal<Vec<TreeNode>>,
    /// The path keys ([`path_key`]) of the open items; the server draws
    /// the tree as given.
    pub expanded: RwSignal<BTreeSet<String>>,
    /// The path key of the selected item; the items above it are its path.
    pub selected: Signal<Option<String>>,
    /// The name of the hidden fields that carry the open items in the
    /// form around the tree, one field per path key.
    pub open_field: Option<String>,
    /// Its words, in the page's locale ([`TreeWords::of_page`]).
    pub words: TreeWords,
}

/// A tree as plain data, the props of [`crate::island::TreeView`]: what
/// [`Tree`] holds, its state as it arrives from the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeData {
    /// The tree's id.
    pub id: String,
    /// The tree's accessible name.
    pub label: String,
    /// Which design it draws.
    pub kind: TreeKind,
    /// Whether it is in an edit.
    pub editing: bool,
    /// The items at the top.
    pub nodes: Vec<TreeNode>,
    /// The path keys of the open items.
    pub open: BTreeSet<String>,
    /// The path key of the selected item.
    pub selected: Option<String>,
    /// The name of the hidden fields that carry the open items.
    pub open_field: Option<String>,
}

/// The part of a [`Tree`] its items and handlers share, so it is `Copy`.
#[derive(Debug, Clone, Copy)]
struct State {
    id: StoredValue<String>,
    nodes: Signal<Vec<TreeNode>>,
    expanded: RwSignal<BTreeSet<String>>,
    selected: Signal<Option<String>>,
    /// The path keys of the items above the selected one.
    path: Memo<BTreeSet<String>>,
    editing: bool,
    /// Whether the page has hydrated, so a toggle with no link works.
    hydrated: RwSignal<bool>,
    words: StoredValue<TreeWords>,
    /// The element of each item drawn, by path key; a row drawn again before
    /// its old one is cleaned up comes later, so a key's last entry is the
    /// one in the page.
    drawn: StoredValue<Vec<(String, NodeRef<leptos::html::Li>)>>,
}

/// One item in sight, in the order the keyboard walks them.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Shown {
    /// Its path key.
    key: String,
    /// Its parent's path key.
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
fn step(key: &str) -> String {
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
fn below(parent: Option<&str>, key: &str) -> String {
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
/// use ferrohealth_ui::widget::tree::path_key;
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
fn find<'a>(nodes: &'a [TreeNode], path: &str) -> Option<&'a TreeNode> {
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
fn children_of(nodes: &[TreeNode], path: Option<&str>) -> Vec<TreeNode> {
    match path {
        None => nodes.to_vec(),
        Some(path) => find(nodes, path)
            .map(|node| node.read().to_vec())
            .unwrap_or_default(),
    }
}

/// The path keys of the items above the item at the path key `path` in
/// `nodes`, from the top, or `None` when no item is there.
fn path_to(nodes: &[TreeNode], path: &str) -> Option<Vec<String>> {
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

/// Opens the item `key` of `tree`; an unread level is read by the console
/// behind the item's toggle link, which the page then follows.
fn open(tree: State, key: &str) {
    tree.expanded.update(|open| {
        open.insert(key.to_owned());
    });
    let unread = tree.nodes.with_untracked(|nodes| {
        find(nodes, key)
            .filter(|node| node.children == Children::Unread)
            .and_then(|node| node.toggle_href.clone())
    });
    if let Some(href) = unread {
        navigate(&href);
    }
}

/// Closes the item `key` of `tree`.
fn close(tree: State, key: &str) {
    tree.expanded.update(|open| {
        open.remove(key);
    });
}

/// Leaves the page for `href`.
fn navigate(href: &str) {
    #[cfg(target_arch = "wasm32")]
    crate::widget::submit::follow(href);
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = href;
}

/// The element of the item `key` of `tree`, if it is drawn.
fn drawn(tree: State, key: &str) -> Option<NodeRef<leptos::html::Li>> {
    tree.drawn.with_value(|drawn| {
        drawn
            .iter()
            .rev()
            .find_map(|(drawn, item)| (drawn == key).then_some(*item))
    })
}

/// Moves the keyboard focus to the item `key` of `tree`.
fn focus_item(tree: State, key: &str) {
    let item = drawn(tree, key);
    #[cfg(target_arch = "wasm32")]
    if let Some(item) = item.and_then(|item| item.get_untracked())
        && item.focus().is_err()
    {
        leptos::logging::warn!("the browser refused the focus on a tree item");
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: Option<NodeRef<leptos::html::Li>> = item;
}

/// Follows the link of the item `key` of `tree`.
fn follow(tree: State, key: &str) {
    let item = drawn(tree, key);
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        // NOTE: the item's own row comes before its children, so its first label is its own.
        let link = item
            .and_then(|item| item.get_untracked())
            .and_then(|item| item.query_selector("a.fui-tree-label").ok().flatten())
            .and_then(|link| link.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(link) = link {
            link.click();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: Option<NodeRef<leptos::html::Li>> = item;
}

/// Whether the key press `event` reached the tree from an item itself, not
/// from a field or a button in its row.
fn from_an_item(event: &leptos::ev::KeyboardEvent) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .and_then(|element| element.get_attribute("role"))
            .is_some_and(|role| role == "treeitem")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: &leptos::ev::KeyboardEvent = event;
        true
    }
}

/// Returns the items under the item at the path key `parent` (the top for
/// `None`) at `level`.
fn items(
    tree: State,
    parent: Option<String>,
    level: u16,
    focused: RwSignal<Option<String>>,
) -> AnyView {
    let each = {
        let parent = parent.clone();
        move || {
            tree.nodes
                .with(|nodes| children_of(nodes, parent.as_deref()))
        }
    };
    view! {
        <For each=each key=|node: &TreeNode| node.key.clone() let:node>
            {item(tree, &node, below(parent.as_deref(), &node.key), level, focused)}
        </For>
    }
    .into_any()
}

/// Returns the toggle that folds and unfolds the item `node` at the path
/// key `key`, out of the tab order (the arrow keys fold an item): a link to
/// its [`TreeNode::toggle_href`], or a button that works once the page has
/// hydrated; or a spacer for an item that cannot open.
fn toggle(
    tree: State,
    node: &TreeNode,
    key: String,
    open: impl Fn() -> bool + Clone + Send + Sync + 'static,
) -> AnyView {
    if !node.opens() {
        return view! { <span class="fui-tree-spacer" aria-hidden="true"></span> }.into_any();
    }
    let label = node.label.clone();
    let name = {
        let open = open.clone();
        move || {
            tree.words.with_value(|words| {
                let phrase = if open() {
                    &words.collapse
                } else {
                    &words.expand
                };
                String::from(phrase.fill(&[("label", &label)]))
            })
        }
    };
    let glyph = {
        let open = open.clone();
        move || {
            icon(
                if open() {
                    Glyph::CHEVRON_DOWN
                } else {
                    Glyph::CHEVRON_RIGHT
                },
                "14",
            )
        }
    };
    let flip = move |event: leptos::ev::MouseEvent| {
        event.prevent_default();
        if open() {
            close(tree, &key);
        } else {
            self::open(tree, &key);
        }
    };
    match node.toggle_href.clone() {
        Some(href) => view! {
            <a href=href class="fui-tree-toggle" tabindex="-1" aria-label=name on:click=flip>
                {glyph}
            </a>
        }
        .into_any(),
        None => view! {
            <button
                type="button"
                class="fui-tree-toggle"
                tabindex="-1"
                aria-label=name
                disabled=move || !tree.hydrated.get()
                on:click=flip
            >
                {glyph}
            </button>
        }
        .into_any(),
    }
}

/// Returns the name of the item `node`: the rename field while it is
/// renamed, a link when it opens a page, and words otherwise.
fn name(tree: State, node: &TreeNode) -> AnyView {
    if let Some(typed) = node.rename.clone() {
        let id = tree.id.with_value(|id| format!("{id}-rename"));
        let label = tree
            .words
            .with_value(|words| words.rename_field.fill(&[("label", &node.label)]));
        return view! {
            <input
                id=id
                name="new_name"
                type="text"
                class="fui-input"
                value=typed
                aria-label=label
                autocomplete="off"
            />
        }
        .into_any();
    }
    if node.removed {
        let removed = tree.words.with_value(|words| words.removed.clone());
        return view! {
            <span class="fui-tree-label">{node.label.clone()}</span>
            <span class="fui-sr-only">" " {removed}</span>
        }
        .into_any();
    }
    match node.href.clone() {
        Some(href) => view! {
            <a href=href class="fui-tree-label" tabindex="-1">
                {node.label.clone()}
            </a>
        }
        .into_any(),
        None => view! { <span class="fui-tree-label">{node.label.clone()}</span> }.into_any(),
    }
}

/// Returns the actions of the item `node` while `tree` is in an edit.
fn edits(tree: State, node: &TreeNode) -> Option<AnyView> {
    if !tree.editing || node.edits.is_empty() {
        return None;
    }
    let buttons = node
        .edits
        .iter()
        .map(|edit| {
            let words = tree.words.with_value(|words| words.edit(*edit, node));
            view! {
                <button
                    type="submit"
                    class="fui-icon-button"
                    name=edit.name()
                    value=node.key.clone()
                    aria-label=words.clone()
                    title=words
                >
                    {icon(edit.glyph(), "16")}
                </button>
            }
        })
        .collect_view();
    Some(view! { <span class="fui-tree-actions">{buttons}</span> }.into_any())
}

/// The loading row and the more row that end the open level of the item
/// at the path key `key`, as the nodes of `tree` have it now.
fn level_end(tree: State, key: &str) -> AnyView {
    let (unread, more) = tree.nodes.with(|nodes| {
        find(nodes, key).map_or((false, None), |node| match &node.children {
            Children::Unread => (true, None),
            Children::Read { more, .. } => (false, more.clone()),
            Children::None => (false, None),
        })
    });
    let reading = unread.then(|| {
        let words = tree.words.with_value(|words| words.reading.clone());
        view! {
            <li class="fui-tree-loading" role="none">
                <span class="fui-spinner-ring" aria-hidden="true"></span>
                {words}
            </li>
        }
    });
    let more = more.map(|more| {
        view! {
            <li class="fui-tree-more" role="none">
                <span>{more.words} " " <a href=more.href>{more.link}</a></span>
            </li>
        }
    });
    view! { {reading} {more} }.into_any()
}

/// The classes `each` names, in order, one space apart.
fn classes(each: &[Option<&str>]) -> String {
    each.iter().flatten().copied().collect::<Vec<_>>().join(" ")
}

/// Returns the row of the item `node` at the path key `key`: its toggle,
/// glyph, name, code, tag, inactive mark, count and actions.
fn row(
    tree: State,
    node: &TreeNode,
    key: String,
    open: impl Fn() -> bool + Clone + Send + Sync + 'static,
) -> AnyView {
    let toggle = toggle(tree, node, key, open);
    let glyph = node.mark.map(|mark| {
        let glyph = match mark {
            Mark::Folder => Glyph::FOLDER,
            Mark::Reference => Glyph::FILE_TEXT,
        };
        view! { <span class="fui-tree-glyph">{icon(glyph, "14")}</span> }
    });
    let name = name(tree, node);
    let code = node
        .code
        .clone()
        .map(|code| view! { <span class="fui-tree-code">{code}</span> });
    let tag = node
        .tag
        .clone()
        .map(|tag| view! { <span class="fui-rm-type">{tag}</span> });
    let inactive = node.inactive.then(|| {
        let words = tree.words.with_value(|words| words.inactive.clone());
        view! { <span class="fui-inactive-mark">{words}</span> }
    });
    let count = node
        .count
        .clone()
        .map(|count| view! { <span class="fui-tree-count">{count}</span> });
    let actions = edits(tree, node);
    view! {
        <div class="fui-tree-row">
            {toggle} {glyph} {name} {code} {tag} {inactive} {count} {actions}
        </div>
    }
    .into_any()
}

/// Returns the item `node` at the path key `key` and the level `level`
/// and, while it is open, its children.
fn item(
    tree: State,
    node: &TreeNode,
    key: String,
    level: u16,
    focused: RwSignal<Option<String>>,
) -> AnyView {
    let id = tree.id.with_value(|id| item_id(id, &key));
    let expandable = node.opens();
    let element = NodeRef::<leptos::html::Li>::new();
    tree.drawn.update_value(|drawn| {
        drawn.push((key.clone(), element));
    });
    {
        let key = key.clone();
        on_cleanup(move || {
            tree.drawn.try_update_value(|drawn| {
                if let Some(at) = drawn.iter().position(|(drawn, _)| *drawn == key) {
                    drawn.remove(at);
                }
            });
        });
    }
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
    let on_path = {
        let key = key.clone();
        move || tree.path.with(|path| path.contains(&key))
    };
    let stop = {
        let key = key.clone();
        move || {
            let at = focused.get().or_else(|| tree.selected.get()).or_else(|| {
                tree.nodes
                    .with(|nodes| nodes.first().map(|node| step(&node.key)))
            });
            if at.as_deref() == Some(key.as_str()) {
                "0"
            } else {
                "-1"
            }
        }
    };
    let row = row(tree, node, key.clone(), open);
    let children = {
        let key = key.clone();
        move || {
            open().then(|| {
                let end = {
                    let key = key.clone();
                    move || level_end(tree, &key)
                };
                view! {
                    <ul role="group" class="fui-tree-group">
                        {items(tree, Some(key.clone()), level.saturating_add(1), focused)}
                        {end}
                    </ul>
                }
            })
        }
    };
    let expanded = move || expandable.then(|| open().to_string());
    let focus_key = key;
    view! {
        <li
            id=id
            role="treeitem"
            class=classes(&[
                Some("fui-tree-item"),
                node.inactive.then_some("is-inactive"),
                (node.mark == Some(Mark::Reference)).then_some("is-ref"),
                node.removed.then_some("is-removed"),
            ])
            class:is-on-path=on_path
            node_ref=element
            aria-expanded=expanded
            aria-selected=move || selected().to_string()
            tabindex=stop
            on:focusin=move |event| {
                event.stop_propagation();
                focused.set(Some(focus_key.clone()));
            }
        >
            {row}
            {children}
        </li>
    }
    // Leptos names no typed `aria-level` attribute, so it is set by name.
    .attr("aria-level", level.to_string())
    .into_any()
}

/// Returns the tree `tree`.
#[must_use]
pub fn tree_view(tree: Tree) -> AnyView {
    let Tree {
        id,
        label,
        kind,
        editing,
        nodes,
        expanded,
        selected,
        open_field,
        words,
    } = tree;
    let path = Memo::new(move |_| {
        selected.with(|selected| {
            selected
                .as_deref()
                .and_then(|key| nodes.with(|nodes| path_to(nodes, key)))
                .unwrap_or_default()
                .into_iter()
                .collect()
        })
    });
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let state = State {
        id: StoredValue::new(id.clone()),
        nodes,
        expanded,
        selected,
        path,
        editing,
        hydrated,
        words: StoredValue::new(words),
        drawn: StoredValue::new(Vec::new()),
    };
    let focused = RwSignal::new(None::<String>);
    let on_key = move |event: leptos::ev::KeyboardEvent| {
        if !from_an_item(&event) {
            return;
        }
        // NOTE: no specification governs this: our own design; an item focused before
        // the page hydrated is the tab stop, the selected item or the first.
        let Some(at) = focused
            .get_untracked()
            .or_else(|| state.selected.get_untracked())
            .or_else(|| {
                state
                    .nodes
                    .with_untracked(|nodes| nodes.first().map(|node| step(&node.key)))
            })
        else {
            return;
        };
        let shown = state
            .nodes
            .with_untracked(|nodes| state.expanded.with_untracked(|open| in_sight(nodes, open)));
        let Some(step) = moved(&event.key(), &at, &shown) else {
            return;
        };
        event.prevent_default();
        match step {
            Move::Focus(key) => {
                focus_item(state, &key);
                focused.set(Some(key));
            }
            Move::Open(key) => open(state, &key),
            Move::Close(key) => close(state, &key),
            Move::Follow(key) => follow(state, &key),
        }
    };
    let fields = open_field.map(|name| {
        let each = move || expanded.get().into_iter().collect::<Vec<_>>();
        view! {
            <For each=each key=String::clone let:key>
                <input type="hidden" name=name.clone() value=key />
            </For>
        }
    });
    view! {
        <div class=classes(
            &[
                Some("fui-concept-tree"),
                (kind == TreeKind::Folder).then_some("fui-folder-tree"),
                (kind == TreeKind::Path).then_some("fui-path-tree"),
                editing.then_some("is-editing"),
            ],
        )>
            {fields}
            <ul id=id role="tree" aria-label=label class="fui-tree" on:keydown=on_key>
                {items(state, None, 1, focused)}
            </ul>
        </div>
    }
    .into_any()
}

/// Returns the tree `data` with its words `words`, its open items and its
/// selection its own state: the body of [`crate::island::TreeView`].
#[must_use]
pub fn tree_island(data: TreeData, words: TreeWords) -> AnyView {
    let TreeData {
        id,
        label,
        kind,
        editing,
        nodes,
        open,
        selected,
        open_field,
    } = data;
    tree_view(Tree {
        id,
        label,
        kind,
        editing,
        nodes: Signal::stored(nodes),
        expanded: RwSignal::new(open),
        selected: Signal::stored(selected),
        open_field,
        words,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        Children, Move, TreeNode, children_of, find, in_sight, item_id, moved, path_key, path_keys,
        path_to,
    };

    fn nodes() -> Vec<TreeNode> {
        vec![
            TreeNode::new("a", "Body site").children(vec![
                TreeNode::new("a1", "Arm").href("/c/a1"),
                TreeNode::new("a2", "Leg").unread(),
            ]),
            TreeNode::new("b", "Finding"),
        ]
    }

    #[test]
    fn the_items_in_sight_follow_what_is_open() {
        let closed = in_sight(&nodes(), &BTreeSet::new());
        let keys: Vec<&str> = closed.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(vec!["a", "b"], keys);
        let open = BTreeSet::from([String::from("a"), String::from("a/a2")]);
        let keys: Vec<String> = in_sight(&nodes(), &open)
            .into_iter()
            .map(|item| item.key)
            .collect();
        assert_eq!(
            vec!["a", "a/a1", "a/a2", "b"],
            keys,
            "an unread level shows no item"
        );
        assert_eq!(2, children_of(&nodes(), Some("a")).len());
        assert!(
            children_of(&nodes(), Some("a/a2")).is_empty(),
            "not read yet"
        );
    }

    #[test]
    fn the_path_is_every_item_above_the_selected_one() {
        assert_eq!(Some(vec![String::from("a")]), path_to(&nodes(), "a/a2"));
        assert_eq!(Some(Vec::new()), path_to(&nodes(), "b"));
        assert_eq!(None, path_to(&nodes(), "z"));
        assert_eq!(None, path_to(&nodes(), "a2"), "a child is under its parent");
    }

    /// One concept under two parents, as SNOMED CT's polyhierarchy has it.
    fn polyhierarchy() -> Vec<TreeNode> {
        vec![
            TreeNode::new("lung", "Disorder of lung").children(vec![
                TreeNode::new("pneumonia", "Pneumonia")
                    .children(vec![TreeNode::new("viral", "Viral")]),
            ]),
            TreeNode::new("infection", "Infectious disease").children(vec![
                TreeNode::new("pneumonia", "Pneumonia")
                    .children(vec![TreeNode::new("viral", "Viral")]),
            ]),
        ]
    }

    #[test]
    fn a_concept_under_two_parents_is_two_items_each_with_its_own_fold_and_place() {
        let open = BTreeSet::from([
            String::from("lung"),
            String::from("infection"),
            String::from("lung/pneumonia"),
        ]);
        let shown = in_sight(&polyhierarchy(), &open);
        let keys: Vec<&str> = shown.iter().map(|item| item.key.as_str()).collect();
        assert_eq!(
            vec![
                "lung",
                "lung/pneumonia",
                "lung/pneumonia/viral",
                "infection",
                "infection/pneumonia",
            ],
            keys,
            "the copy under infection stays closed"
        );
        assert_ne!(
            item_id("t", "lung/pneumonia"),
            item_id("t", "infection/pneumonia")
        );
        let focus = |key: &str| Some(Move::Focus(key.to_owned()));
        assert_eq!(
            focus("infection/pneumonia"),
            moved("ArrowDown", "infection", &shown),
            "Down from the second parent reaches its own copy"
        );
        assert_eq!(
            focus("infection"),
            moved("ArrowLeft", "infection/pneumonia", &shown),
            "Left from the second copy reaches its own parent"
        );
        assert_eq!(
            Some(Move::Open(String::from("infection/pneumonia"))),
            moved("ArrowRight", "infection/pneumonia", &shown)
        );
        assert_eq!(
            Some(vec![String::from("infection")]),
            path_to(&polyhierarchy(), "infection/pneumonia")
        );
        assert_eq!(
            Some("Viral"),
            find(&polyhierarchy(), "infection/pneumonia/viral").map(|node| node.label.as_str())
        );
    }

    #[test]
    fn a_path_key_reads_back_whatever_its_keys_hold() {
        for keys in [
            vec!["a"],
            vec!["a/b", "c"],
            vec!["100%", "%2F", "x"],
            vec!["/content[id0.1]", "/data[id2]/value"],
        ] {
            assert_eq!(keys, path_keys(&path_key(&keys)), "{keys:?}");
        }
        assert_ne!(path_key(&["a/b"]), path_key(&["a", "b"]));
        assert_eq!(Vec::<String>::new(), path_keys(""));
    }

    #[test]
    fn the_keys_walk_the_tree_as_the_pattern_has_it() {
        let open = BTreeSet::from([String::from("a")]);
        let shown = in_sight(&nodes(), &open);
        let focus = |key: &str| Some(Move::Focus(key.to_owned()));
        assert_eq!(focus("a/a1"), moved("ArrowDown", "a", &shown));
        assert_eq!(focus("a"), moved("ArrowUp", "a/a1", &shown));
        assert_eq!(
            None,
            moved("ArrowUp", "a", &shown),
            "no item above the first"
        );
        assert_eq!(focus("b"), moved("End", "a", &shown));
        assert_eq!(focus("a"), moved("Home", "b", &shown));
        assert_eq!(
            focus("a/a1"),
            moved("ArrowRight", "a", &shown),
            "into an open item"
        );
        assert_eq!(
            Some(Move::Open(String::from("a/a2"))),
            moved("ArrowRight", "a/a2", &shown)
        );
        assert_eq!(
            Some(Move::Close(String::from("a"))),
            moved("ArrowLeft", "a", &shown)
        );
        assert_eq!(
            focus("a"),
            moved("ArrowLeft", "a/a1", &shown),
            "to the parent"
        );
        assert_eq!(
            Some(Move::Follow(String::from("a/a1"))),
            moved("Enter", "a/a1", &shown)
        );
        assert_eq!(
            Some(Move::Open(String::from("a/a2"))),
            moved(" ", "a/a2", &shown)
        );
        assert_eq!(None, moved("x", "a", &shown));
    }

    #[test]
    fn a_removed_item_is_no_link_to_follow() {
        let nodes = vec![TreeNode::new("r", "Report").href("/r").removed()];
        let shown = in_sight(&nodes, &BTreeSet::new());
        assert_eq!(None, moved("Enter", "r", &shown));
    }

    #[test]
    fn a_tree_round_trips_through_json_as_an_island_prop() {
        let json = serde_json::to_string(&nodes()).expect("a tree serializes");
        let back: Vec<TreeNode> = serde_json::from_str(&json).expect("a tree reads back");
        assert_eq!(nodes(), back);
        assert!(matches!(
            back.first().map(|node| &node.children),
            Some(Children::Read { .. })
        ));
    }

    #[test]
    fn every_key_makes_a_distinct_valid_id() {
        assert_eq!("t-item-6120622f63", item_id("t", "a b/c"));
        assert_ne!(item_id("t", "a b"), item_id("t", "a_b"));
    }
}
