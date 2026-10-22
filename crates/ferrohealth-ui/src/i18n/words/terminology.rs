// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the terminology parts: the coding, the version
//! select, the value set clause, the root matrix and the mapping row.

use crate::i18n::Text;

texts! {
    /// The mark of an exclude clause.
    ClauseExclude = "clause-exclude",
    /// The name of an exclude clause, by its source `$title`.
    ClauseExcludeLabel = "clause-exclude-label",
    /// The mark of an include clause.
    ClauseInclude = "clause-include",
    /// The name of an include clause, by its source `$title`.
    ClauseIncludeLabel = "clause-include-label",
    /// The word of the inactive mark.
    CodingInactive = "coding-inactive",
    /// A coding with no display.
    CodingNoDisplay = "coding-no-display",
    /// The header of a mapping's comment.
    MappingComment = "mapping-comment",
    /// The name of the dash a mapping to nothing shows as its target.
    MappingNoTarget = "mapping-no-target",
    /// The header of a mapping's relationship.
    MappingRelationship = "mapping-relationship",
    /// The header of a mapping's source.
    MappingSource = "mapping-source",
    /// The header of a mapping's target.
    MappingTarget = "mapping-target",
    /// R5 `source-is-broader-than-target`.
    RelationBroader = "relation-broader",
    /// R4 `disjoint`.
    RelationDisjoint = "relation-disjoint",
    /// R4 `equal`.
    RelationEqual = "relation-equal",
    /// R4 and R5 `equivalent`.
    RelationEquivalent = "relation-equivalent",
    /// R4 `inexact`.
    RelationInexact = "relation-inexact",
    /// R5 `source-is-narrower-than-target`.
    RelationNarrower = "relation-narrower",
    /// A mapping to nothing: R4 `unmatched`, R5 `noMap`.
    RelationNone = "relation-none",
    /// R5 `not-related-to`.
    RelationNotRelated = "relation-not-related",
    /// R4 `relatedto`, R5 `related-to`.
    RelationRelated = "relation-related",
    /// R4 `specializes`.
    RelationSpecializes = "relation-specializes",
    /// R4 `subsumes`.
    RelationSubsumes = "relation-subsumes",
    /// R4 `narrower`.
    RelationTargetNarrower = "relation-target-narrower",
    /// R4 `wider`.
    RelationTargetWider = "relation-target-wider",
    /// The state of a root that answered.
    RootAnswered = "root-answered",
    /// A cell of a root that did not answer.
    RootCellNoAnswer = "root-cell-no-answer",
    /// The state of a root that did not answer.
    RootNoAnswer = "root-no-answer",
    /// A cell a root does not declare.
    RootNotDeclared = "root-not-declared",
    /// The release line of the default root, by its release `$note`.
    VersionDefault = "version-default",
    /// The version select's name and tooltip: its `$label`, the root in
    /// view `$value` and its release `$note`.
    VersionName = "version-name",
}
