# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# The English words of the coding and its more row, the version select, the
# value set clause, the root matrix and the mapping row: one message per
# `Text` of `src/i18n/words/terminology.rs`, in the order of their ids. The
# relationships read as the HL7 FHIR specification defines its codes: R4
# ConceptMapEquivalence, read from the target to the source, and R5
# ConceptMapRelationship, read from the source to the target.

clause-exclude = Exclude
clause-exclude-label = Exclude: { $title }
clause-include = Include
clause-include-label = Include: { $title }
codes-more = { $shown } of { $total } codes shown.
codes-more-next = Show the next { $count }
codes-more-rest = { $count ->
    [one] Show the other one
   *[other] Show the other { $count }
}
coding-inactive = Inactive
coding-no-display = no display
mapping-comment = Comment
mapping-no-target = no target
mapping-relationship = Relationship
mapping-source = Source
mapping-target = Target
relation-broader = broader than the target
relation-disjoint = disjoint
relation-equal = equal
relation-equivalent = equivalent
relation-inexact = inexact
relation-narrower = narrower than the target
relation-none = maps to nothing
relation-not-related = not related to the target
relation-related = related to
relation-specializes = the target specialises it
relation-subsumes = the target subsumes it
relation-target-narrower = the target is narrower
relation-target-wider = the target is wider
root-answered = Answered
root-cell-no-answer = no answer
root-no-answer = No answer
root-not-declared = not declared
version-default = { $note } · default
version-name = { $label }: { $value }, { $note }
