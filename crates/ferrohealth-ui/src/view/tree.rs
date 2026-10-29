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
//! The server draws the tree as [`Tree`] holds it: the open items with
//! their children, a closed item with none. A level is read by the
//! console's server, one at a time: an item whose children are not read yet
//! is [`Children::Unread`], and opening it follows its toggle's link, which
//! the console answers with the level read; an open unread level shows its
//! loading row. A level larger than a page ends in the more row ([`More`]).
//! The open items are state the server can read: the toggle is a link to
//! the page with the item opened or closed, so a tree folds with no script,
//! and a tree in a form posts its open items as hidden fields. The `tree`
//! module folds a drawn level in place, walks the keys, keeps the hidden
//! fields in step and keeps the page's folds for the session; opening a
//! level the server did not draw follows the toggle's link.
//!
//! An item's key is the console's data-derived id, never a position, and is
//! unique among its siblings. The tree addresses an item by its path key
//! ([`path_key`]): the keys from the top to it, so the same concept under
//! two parents (a polyhierarchy, as in SNOMED CT) is two items, each with
//! its own id, its own fold, its own selection and its own place in the
//! keyboard's walk. [`Tree::open`], [`Tree::selected`] and the open fields
//! hold path keys; a top item's path key is its key.

use std::collections::BTreeSet;

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
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
    /// reading `next` more at `href`, in `locale`.
    #[must_use]
    pub fn of(locale: &Locale, shown: u32, total: u32, next: u32, href: impl Into<String>) -> Self {
        let rest = total.saturating_sub(shown);
        let link = if next >= rest {
            tr_with(locale, Text::TreeMoreRest, &[("count", rest.into())])
        } else {
            tr_with(locale, Text::TreeMoreNext, &[("count", next.into())])
        };
        Self {
            words: tr_with(
                locale,
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
/// value, so the edit posts with no script.
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
    /// tree folds with no script and the console reads an unread level.
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
/// locale.
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
    /// The tree's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        let around =
            |text: Text| Phrase::new(tr_with(locale, text, &[("label", Phrase::open("label"))]));
        Self {
            expand: around(Text::TreeExpand),
            collapse: around(Text::TreeCollapse),
            inactive: tr(locale, Text::TreeInactive),
            reading: tr(locale, Text::TreeReading),
            removed: tr(locale, Text::TreeRemoved),
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
/// use ferrohealth_ui::view::tree::path_key;
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
#[cfg(test)]
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

/// What a tree shows: its items, its open items and its selection.
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
    pub nodes: Vec<TreeNode>,
    /// The path keys ([`path_key`]) of the open items; the server draws
    /// the tree as given.
    pub open: BTreeSet<String>,
    /// The path key of the selected item; the items above it are its path.
    pub selected: Option<String>,
    /// The name of the hidden fields that carry the open items in the
    /// form around the tree, one field per path key.
    pub open_field: Option<String>,
    /// Its words, in the page's locale ([`TreeWords::of`]).
    pub words: TreeWords,
}

/// What every item of one drawing shares.
struct Drawing<'a> {
    tree: &'a Tree,
    /// The path keys of the items above the selected one.
    path: BTreeSet<String>,
    /// The path key of the one tab stop: the selection, or the first item.
    stop: Option<String>,
}

/// The classes `each` names, in order, one space apart.
fn classes(each: &[Option<&str>]) -> String {
    each.iter().flatten().copied().collect::<Vec<_>>().join(" ")
}

#[derive(Template)]
#[template(path = "tree/tree.html", block = "item")]
struct ItemView<'a> {
    id: String,
    key: &'a str,
    class: String,
    expanded: Option<bool>,
    selected: bool,
    stop: bool,
    level: u16,
    toggle: Html,
    glyph: Option<Html>,
    name: Html,
    code: Option<&'a str>,
    tag: Option<&'a str>,
    inactive: Option<&'a Word>,
    count: Option<&'a str>,
    actions: Option<Html>,
    children: Option<Html>,
}

#[derive(Template)]
#[template(path = "tree/tree.html", block = "toggle")]
struct ToggleView<'a> {
    href: Option<&'a str>,
    name: Word,
    expand: Word,
    collapse: Word,
    glyph: Html,
}

#[derive(Template)]
#[template(path = "tree/tree.html", block = "tree")]
struct TreeView<'a> {
    class: String,
    id: &'a str,
    label: &'a str,
    open_field: Option<&'a str>,
    open: &'a BTreeSet<String>,
    reading: &'a Word,
    shut: Html,
    opened: Html,
    items: Html,
}

/// Returns the toggle that folds and unfolds the item `node`, out of the
/// tab order (the arrow keys fold an item): a link to its
/// [`TreeNode::toggle_href`], or a button the `tree` module enables; or a
/// spacer for an item that cannot open.
fn toggle(words: &TreeWords, node: &TreeNode, open: bool) -> Html {
    if !node.opens() {
        return Html::raw(String::from(
            "<span aria-hidden=\"true\" class=\"fui-tree-spacer\"></span>",
        ));
    }
    let label = [("label", node.label.as_str())];
    let expand = words.expand.fill(&label);
    let collapse = words.collapse.fill(&label);
    ToggleView {
        href: node.toggle_href.as_deref(),
        name: if open {
            collapse.clone()
        } else {
            expand.clone()
        },
        expand,
        collapse,
        glyph: icon(
            if open {
                Glyph::CHEVRON_DOWN
            } else {
                Glyph::CHEVRON_RIGHT
            },
            "14",
        ),
    }
    .html()
}

/// Returns the name of the item `node`: the rename field while it is
/// renamed, a link when it opens a page, and words otherwise.
fn name(tree: &Tree, node: &TreeNode) -> Html {
    if let Some(typed) = &node.rename {
        let label = tree.words.rename_field.fill(&[("label", &node.label)]);
        return Html::raw(format!(
            "<input id=\"{}-rename\" name=\"new_name\" type=\"text\" value=\"{}\" aria-label=\"{}\" autocomplete=\"off\" class=\"fui-input\">",
            Html::text(&tree.id),
            Html::text(typed),
            Html::text(label.text())
        ));
    }
    let label = Html::text(&node.label);
    if node.removed {
        return Html::raw(format!(
            "<span class=\"fui-tree-label\">{label}</span><span class=\"fui-sr-only\"> {}</span>",
            tree.words.removed
        ));
    }
    Html::raw(match &node.href {
        Some(href) => format!(
            "<a href=\"{}\" tabindex=\"-1\" class=\"fui-tree-label\">{label}</a>",
            Html::text(href)
        ),
        None => format!("<span class=\"fui-tree-label\">{label}</span>"),
    })
}

/// Returns the actions of the item `node` while `tree` is in an edit.
fn edits(tree: &Tree, node: &TreeNode) -> Option<Html> {
    if !tree.editing || node.edits.is_empty() {
        return None;
    }
    let buttons: Vec<String> = node
        .edits
        .iter()
        .map(|edit| {
            let words = Html::text(tree.words.edit(*edit, node).text());
            format!(
                "<button type=\"submit\" name=\"{}\" value=\"{}\" aria-label=\"{words}\" title=\"{words}\" class=\"fui-icon-button\">{}</button>",
                edit.name(),
                Html::text(&node.key),
                icon(edit.glyph(), "16")
            )
        })
        .collect();
    Some(Html::raw(format!(
        "<span class=\"fui-tree-actions\">{}</span>",
        buttons.concat()
    )))
}

/// The loading row and the more row that end the open level of `node`.
fn level_end(words: &TreeWords, node: &TreeNode) -> String {
    match &node.children {
        Children::Unread => format!(
            "<li role=\"none\" class=\"fui-tree-loading\"><span aria-hidden=\"true\" class=\"fui-spinner-ring\"></span>{}</li>",
            words.reading
        ),
        Children::Read {
            more: Some(more), ..
        } => format!(
            "<li role=\"none\" class=\"fui-tree-more\"><span>{} <a href=\"{}\">{}</a></span></li>",
            more.words,
            Html::text(&more.href),
            more.link
        ),
        Children::Read { more: None, .. } | Children::None => String::new(),
    }
}

/// Returns the items `nodes` under the item at the path key `parent` (the
/// top for `None`) at `level`.
fn items(drawing: &Drawing<'_>, nodes: &[TreeNode], parent: Option<&str>, level: u16) -> Html {
    nodes
        .iter()
        .map(|node| item(drawing, node, &below(parent, &node.key), level))
        .collect()
}

/// Returns the item `node` at the path key `key` and the level `level`
/// and, while it is open, its children.
fn item(drawing: &Drawing<'_>, node: &TreeNode, key: &str, level: u16) -> Html {
    let tree = drawing.tree;
    let open = node.opens() && tree.open.contains(key);
    let children = open.then(|| {
        Html::raw(format!(
            "<ul role=\"group\" class=\"fui-tree-group\">{}{}</ul>",
            items(drawing, node.read(), Some(key), level.saturating_add(1)),
            level_end(&tree.words, node)
        ))
    });
    let glyph = node.mark.map(|mark| {
        let glyph = match mark {
            Mark::Folder => Glyph::FOLDER,
            Mark::Reference => Glyph::FILE_TEXT,
        };
        Html::raw(format!(
            "<span class=\"fui-tree-glyph\">{}</span>",
            icon(glyph, "14")
        ))
    });
    ItemView {
        id: item_id(&tree.id, key),
        key,
        class: classes(&[
            Some("fui-tree-item"),
            node.inactive.then_some("is-inactive"),
            (node.mark == Some(Mark::Reference)).then_some("is-ref"),
            node.removed.then_some("is-removed"),
            drawing.path.contains(key).then_some("is-on-path"),
        ]),
        expanded: node.opens().then_some(open),
        selected: tree.selected.as_deref() == Some(key),
        stop: drawing.stop.as_deref() == Some(key),
        level,
        toggle: toggle(&tree.words, node, open),
        glyph,
        name: name(tree, node),
        code: node.code.as_deref(),
        tag: node.tag.as_deref(),
        inactive: node.inactive.then_some(&tree.words.inactive),
        count: node.count.as_deref(),
        actions: edits(tree, node),
        children,
    }
    .html()
}

/// Returns the tree `tree`, drawn as it holds it, for the `tree` module to
/// fold and walk.
#[must_use]
pub fn tree_view(tree: &Tree) -> Html {
    let path = tree
        .selected
        .as_deref()
        .and_then(|key| path_to(&tree.nodes, key))
        .unwrap_or_default()
        .into_iter()
        .collect();
    let stop = tree
        .selected
        .clone()
        .or_else(|| tree.nodes.first().map(|node| step(&node.key)));
    let drawing = Drawing { tree, path, stop };
    TreeView {
        class: classes(&[
            Some("fui-concept-tree"),
            (tree.kind == TreeKind::Folder).then_some("fui-folder-tree"),
            (tree.kind == TreeKind::Path).then_some("fui-path-tree"),
            tree.editing.then_some("is-editing"),
        ]),
        id: &tree.id,
        label: &tree.label,
        open_field: tree.open_field.as_deref(),
        open: &tree.open,
        reading: &tree.words.reading,
        shut: icon(Glyph::CHEVRON_RIGHT, "14"),
        opened: icon(Glyph::CHEVRON_DOWN, "14"),
        items: items(&drawing, &tree.nodes, None, 1),
    }
    .html()
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
    fn a_tree_round_trips_through_json() {
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
