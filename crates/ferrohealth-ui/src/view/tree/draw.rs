// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The drawing of a tree: its items, their toggles, names and edits, level by
//! level.

use std::collections::BTreeSet;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::Word;
use crate::icon::{Glyph, icon};

use super::path::{below, item_id};
use super::{Children, Mark, Tree, TreeNode, TreeWords};

/// What every item of one drawing shares.
pub(super) struct Drawing<'a> {
    pub(super) tree: &'a Tree,
    /// The path keys of the items above the selected one.
    pub(super) path: BTreeSet<String>,
    /// The path key of the one tab stop: the selection in sight, or the
    /// first item.
    pub(super) stop: Option<String>,
}

/// The classes `each` names, in order, one space apart.
pub(super) fn classes(each: &[Option<&str>]) -> String {
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
pub(super) struct TreeView<'a> {
    pub(super) class: String,
    pub(super) id: &'a str,
    pub(super) label: &'a str,
    pub(super) open_field: Option<&'a str>,
    pub(super) open: &'a BTreeSet<String>,
    pub(super) reading: &'a Word,
    pub(super) shut: Html,
    pub(super) opened: Html,
    pub(super) items: Html,
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
pub(super) fn items(
    drawing: &Drawing<'_>,
    nodes: &[TreeNode],
    parent: Option<&str>,
    level: u16,
) -> Html {
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
    // NOTE: no specification governs this: our own design; a closed level the
    // server read is drawn hidden, so the `tree` module opens it in place.
    let drawn = open || matches!(node.children, Children::Read { .. });
    let children = drawn.then(|| {
        Html::raw(format!(
            "<ul role=\"group\"{} class=\"fui-tree-group\">{}{}</ul>",
            if open { "" } else { " hidden" },
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
