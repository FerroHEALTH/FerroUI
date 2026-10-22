// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the trees (M1, E4) and the path browser (E5).

use crate::i18n::Text;

texts! {
    /// The facts heading of a node's FLAT path.
    PathNodeFlat = "path-node-flat",
    /// The facts heading of a node's occurrences.
    PathNodeOccurrences = "path-node-occurrences",
    /// The facts heading of a node's path.
    PathNodePath = "path-node-path",
    /// The link in a node's head to the query builder.
    PathNodeQuery = "path-node-query",
    /// The facts heading of a node's RM type.
    PathNodeRmType = "path-node-rm-type",
    /// The facts heading of a node's units.
    PathNodeUnits = "path-node-units",
    /// The heading over a coded node's value set: `$count` its codes.
    PathNodeValueSet = "path-node-value-set",
    /// The name of a toggle that closes an item: `$label` its name.
    TreeCollapse = "tree-collapse",
    /// The name of the action that adds a folder in a folder: `$label` its
    /// name.
    TreeEditAddFolder = "tree-edit-add-folder",
    /// The name of the action that adds a reference in a folder: `$label`
    /// its name.
    TreeEditAddReference = "tree-edit-add-reference",
    /// The name of the action that removes a folder: `$label` its name.
    TreeEditRemove = "tree-edit-remove",
    /// The name of the action that removes a reference: `$label` its name.
    TreeEditRemoveReference = "tree-edit-remove-reference",
    /// The name of the action that renames a folder: `$label` its name.
    TreeEditRename = "tree-edit-rename",
    /// The name of the action that keeps a removed item: `$label` its name.
    TreeEditUndo = "tree-edit-undo",
    /// The name of a toggle that opens an item: `$label` its name.
    TreeExpand = "tree-expand",
    /// An inactive concept's mark.
    TreeInactive = "tree-inactive",
    /// A level that ends before its last child: `$shown` the children it
    /// shows of `$total`.
    TreeMore = "tree-more",
    /// The link that reads the next page of a level: `$count` children.
    TreeMoreNext = "tree-more-next",
    /// The link that reads the rest of a level: `$count` children.
    TreeMoreRest = "tree-more-rest",
    /// The row of a level being read.
    TreeReading = "tree-reading",
    /// What an item removed in an edit says to assistive technology.
    TreeRemoved = "tree-removed",
    /// The name of the field that renames an item: `$label` its name.
    TreeRenameField = "tree-rename-field",
}
