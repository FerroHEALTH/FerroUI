// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The mapping row (M6): one mapping of a concept map, its source coding,
//! the relationship as a word with its arrow, the target coding and the
//! comment.
//!
//! A mapping to nothing says so in words with its own glyph, and its target
//! cell holds a named dash, never an empty cell. The row is a table row of
//! [`crate::table::view::data_table`], so on a phone the source, the
//! relationship and the target stack in reading order. In an editor each
//! source code is one block ([`map_element`]) with its targets under it as
//! F7 rows ([`crate::view::repeat`]). No specification governs the
//! layout: our own design. The relationships are the HL7 FHIR
//! specification's: R4 `ConceptMapEquivalence` and R5
//! `ConceptMapRelationship`.

use crate::html::Html;
use crate::i18n::{Locale, Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::table::view::{Column, Role, cell, title_cell};
use crate::view::coding::{Coding, Layout, coding};

/// How a mapping's target relates to its source.
///
/// R4 reads its codes from the target to the source and R5 from the source
/// to the target; each variant says its words the way its version reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relationship {
    /// R4 `relatedto`, R5 `related-to`.
    RelatedTo,
    /// R4 and R5 `equivalent`.
    Equivalent,
    /// R4 `equal`.
    Equal,
    /// R4 `wider`: the target is wider in meaning than the source.
    Wider,
    /// R4 `subsumes`: the target subsumes the source.
    Subsumes,
    /// R4 `narrower`: the target is narrower in meaning than the source.
    Narrower,
    /// R4 `specializes`: the target specialises the source.
    Specializes,
    /// R4 `inexact`.
    Inexact,
    /// R4 `disjoint`: an explicit assertion that the two do not map.
    Disjoint,
    /// R5 `source-is-narrower-than-target`.
    SourceNarrower,
    /// R5 `source-is-broader-than-target`.
    SourceBroader,
    /// R5 `not-related-to`.
    NotRelatedTo,
    /// No match: R4 `unmatched`, or an R5 element with `noMap` set.
    Unmatched,
}

impl Relationship {
    /// The relationship an R4 or R4B `ConceptMap` writes as `code`, or
    /// `None` for a code the value set does not hold.
    #[must_use]
    pub fn from_r4(code: &str) -> Option<Self> {
        // NOTE: HL7 FHIR R4 (4.0.1), ValueSet concept-map-equivalence: the ten codes.
        Some(match code {
            "relatedto" => Self::RelatedTo,
            "equivalent" => Self::Equivalent,
            "equal" => Self::Equal,
            "wider" => Self::Wider,
            "subsumes" => Self::Subsumes,
            "narrower" => Self::Narrower,
            "specializes" => Self::Specializes,
            "inexact" => Self::Inexact,
            "unmatched" => Self::Unmatched,
            "disjoint" => Self::Disjoint,
            _ => return None,
        })
    }

    /// The relationship an R5 `ConceptMap` writes as `code`, or `None` for
    /// a code the value set does not hold; `noMap` is
    /// [`Relationship::Unmatched`].
    #[must_use]
    pub fn from_r5(code: &str) -> Option<Self> {
        // NOTE: HL7 FHIR R5 (5.0.0), ValueSet concept-map-relationship: the five codes.
        Some(match code {
            "related-to" => Self::RelatedTo,
            "equivalent" => Self::Equivalent,
            "source-is-narrower-than-target" => Self::SourceNarrower,
            "source-is-broader-than-target" => Self::SourceBroader,
            "not-related-to" => Self::NotRelatedTo,
            _ => return None,
        })
    }

    /// The words of the relationship in the page's locale.
    #[must_use]
    pub fn words(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::RelatedTo => Text::RelationRelated,
                Self::Equivalent => Text::RelationEquivalent,
                Self::Equal => Text::RelationEqual,
                Self::Wider => Text::RelationTargetWider,
                Self::Subsumes => Text::RelationSubsumes,
                Self::Narrower => Text::RelationTargetNarrower,
                Self::Specializes => Text::RelationSpecializes,
                Self::Inexact => Text::RelationInexact,
                Self::Disjoint => Text::RelationDisjoint,
                Self::SourceNarrower => Text::RelationNarrower,
                Self::SourceBroader => Text::RelationBroader,
                Self::NotRelatedTo => Text::RelationNotRelated,
                Self::Unmatched => Text::RelationNone,
            },
        )
    }
}

/// Returns the relationship `relationship` as its arrow and its words, or,
/// for a mapping to nothing, its own glyph and the words "maps to nothing".
#[must_use]
pub fn relation(locale: &Locale, relationship: Relationship) -> Html {
    let none = relationship == Relationship::Unmatched;
    let glyph = if none { Glyph::BAN } else { Glyph::ARROW_RIGHT };
    Html::raw(format!(
        "<span class=\"fui-relation{}\">{}{}</span>",
        if none { " is-none" } else { "" },
        icon(glyph, "14"),
        relationship.words(locale)
    ))
}

/// One mapping of a concept map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    /// The source code.
    pub source: Coding,
    /// How the target relates to it.
    pub relationship: Relationship,
    /// The target code, or `None` for a mapping to nothing, which draws as
    /// [`Relationship::Unmatched`] whatever `relationship` says.
    pub target: Option<Coding>,
    /// The comment, if the map writes one.
    pub comment: Option<String>,
}

/// Returns the columns of a table of mappings: the source, the
/// relationship, the target and, when `comments`, the comment, which leaves
/// the compact phone row.
#[must_use]
pub fn mapping_columns(locale: &Locale, comments: bool) -> Vec<Column> {
    let mut columns = vec![
        Column::owned("source", tr(locale, Text::MappingSource)).role(Role::Title),
        Column::owned("relationship", tr(locale, Text::MappingRelationship)).role(Role::Sub),
        Column::owned("target", tr(locale, Text::MappingTarget)).role(Role::Quiet),
    ];
    if comments {
        columns.push(Column::owned("comment", tr(locale, Text::MappingComment)).role(Role::Wide));
    }
    columns
}

/// Returns the row of `mapping` under `columns` from [`mapping_columns`].
///
/// The source heads the row; the comment cell is drawn when the columns
/// have one, empty for a mapping with no comment.
#[must_use]
pub fn mapping_row(locale: &Locale, columns: &[Column], mapping: Mapping) -> Html {
    let Mapping {
        source,
        relationship,
        target,
        comment,
    } = mapping;
    let relationship = if target.is_some() {
        relationship
    } else {
        Relationship::Unmatched
    };
    let head = title_cell(None, coding(locale, source, Layout::Stacked), None);
    let related = columns
        .get(1)
        .map(|column| {
            format!(
                "<td data-label=\"{}\" data-role=\"{}\" class=\"fui-relation-cell\">{}</td>",
                Html::text(column.label.text()),
                column.role.key(),
                relation(locale, relationship)
            )
        })
        .unwrap_or_default();
    let target = columns
        .get(2)
        .map(|column| {
            let shown = match target {
                Some(target) => coding(locale, target, Layout::Stacked),
                None => Html::raw(format!(
                    "<span role=\"img\" aria-label=\"{}\" class=\"fui-muted\">\u{2013}</span>",
                    Html::text(tr(locale, Text::MappingNoTarget).text())
                )),
            };
            String::from(cell(column, shown))
        })
        .unwrap_or_default();
    let comment = columns
        .get(3)
        .map(|column| String::from(cell(column, Html::text(comment.unwrap_or_default()))))
        .unwrap_or_default();
    Html::raw(format!("<tr>{head}{related}{target}{comment}</tr>"))
}

/// One source code of a concept map in an editor, with its targets.
#[derive(Debug)]
pub struct MapElement {
    /// The source code.
    pub source: Coding,
    /// The actions at the end of its head, such as Preview and Remove.
    pub actions: Option<Html>,
    /// Its targets, F7 rows from [`crate::view::repeat::repeat`], each a
    /// target code, a relationship and a comment. A target with no
    /// relationship says so in the line after its row.
    pub targets: Html,
    /// What shows under the targets, such as a Translate preview.
    pub after: Option<Html>,
}

/// Returns the source codes of a concept map in an editor, one block each.
#[must_use]
pub fn map_elements(elements: Html) -> Html {
    let elements = String::from(elements);
    Html::raw(format!("<ul class=\"fui-map-elements\">{elements}</ul>"))
}

/// Returns the block of `element`: its source coding and actions over its
/// target rows.
#[must_use]
pub fn map_element(locale: &Locale, element: MapElement) -> Html {
    let MapElement {
        source,
        actions,
        targets,
        after,
    } = element;
    let actions = actions
        .map(|actions| format!("<div class=\"fui-actions\">{actions}</div>"))
        .unwrap_or_default();
    Html::raw(format!(
        "<li class=\"fui-map-element\"><div class=\"fui-map-element-head\">{}{actions}</div>{targets}{}</li>",
        coding(locale, source, Layout::Inline),
        after.unwrap_or_default()
    ))
}

#[cfg(test)]
mod tests {
    use super::Relationship;

    #[test]
    fn every_r4_code_reads_and_an_unknown_one_does_not() {
        for code in [
            "relatedto",
            "equivalent",
            "equal",
            "wider",
            "subsumes",
            "narrower",
            "specializes",
            "inexact",
            "unmatched",
            "disjoint",
        ] {
            assert!(Relationship::from_r4(code).is_some(), "{code}");
        }
        assert_eq!(None, Relationship::from_r4("related-to"), "an R5 code");
        assert_eq!(
            Some(Relationship::Unmatched),
            Relationship::from_r4("unmatched")
        );
    }

    #[test]
    fn every_r5_code_reads_and_an_r4_one_does_not() {
        assert_eq!(
            Some(Relationship::SourceBroader),
            Relationship::from_r5("source-is-broader-than-target")
        );
        assert_eq!(
            Some(Relationship::SourceNarrower),
            Relationship::from_r5("source-is-narrower-than-target")
        );
        assert_eq!(
            Some(Relationship::RelatedTo),
            Relationship::from_r5("related-to")
        );
        assert_eq!(
            Some(Relationship::NotRelatedTo),
            Relationship::from_r5("not-related-to")
        );
        assert_eq!(
            Some(Relationship::Equivalent),
            Relationship::from_r5("equivalent")
        );
        assert_eq!(None, Relationship::from_r5("wider"), "an R4 code");
    }
}
