// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the terminology page.

use super::Msg;

messages! {
    /// `Add an exclude`
    TmAddExclude = "tm-add-exclude",
    /// `Add filter`
    TmAddFilter = "tm-add-filter",
    /// `Add an include`
    TmAddInclude = "tm-add-include",
    /// `Add a target`
    TmAddTarget = "tm-add-target",
    /// `The FHIR R6 ballot (6.0.0-ballot5). Behaviour grounded only in the ballot may change when R6 is published.`
    TmBallot = "tm-ballot",
    /// `Base address`
    TmBase = "tm-base",
    /// `A long canonical`
    TmCanonical = "tm-canonical",
    /// `Choose one`
    TmChoose = "tm-choose",
    /// `Choose a relationship for { $value }.`
    TmChooseRelationship = "tm-choose-relationship",
    /// `Code system`
    TmCodeSystem = "tm-code-system",
    /// `Code systems`
    TmCodeSystems = "tm-code-systems",
    /// `Codes`
    TmCodes = "tm-codes",
    /// `Codings`
    TmCodings = "tm-codings",
    /// `Comment`
    TmComment = "tm-comment",
    /// `Includes and excludes`
    TmComposer = "tm-composer",
    /// `Includes add codes; excludes take them back out.`
    TmComposerIntro = "tm-composer-intro",
    /// `Legacy findings to synthetic findings`
    TmConceptMap = "tm-concept-map",
    /// `Declaration`
    TmDeclaration = "tm-declaration",
    /// `What each root declares`
    TmDeclares = "tm-declares",
    /// `Edit the concept map`
    TmEditConceptMap = "tm-edit-concept-map",
    /// `Edit the value set`
    TmEditValueSet = "tm-edit-value-set",
    /// `Group 1: { $name } to { $value }`
    TmEditorGroup = "tm-editor-group",
    /// `{ $count -> [one] Excludes { $count } code: *[other] Excludes { $count } codes: }`
    TmExcludesCodes = "tm-excludes-codes",
    /// `FHIR version`
    TmFhirVersion = "tm-fhir-version",
    /// `Filters`
    TmFilters = "tm-filters",
    /// `Filters of the include`
    TmFiltersOf = "tm-filters-of",
    /// `From { $name }, version 3`
    TmFrom = "tm-from",
    /// `From { $name } to { $value }`
    TmGroup = "tm-group",
    /// `Inactive, with its system`
    TmInactive = "tm-inactive",
    /// `Includes { $value } { $name } and everything below it.`
    TmIncludesBelow = "tm-includes-below",
    /// `{ $count -> [one] Includes { $count } code. *[other] Includes { $count } codes. }`
    TmIncludesCodes = "tm-includes-codes",
    /// `Inline`
    TmInline = "tm-inline",
    /// `Every mapping the map writes`
    TmMappings = "tm-mappings",
    /// `With no display`
    TmNoDisplay = "tm-no-display",
    /// `No filters yet.`
    TmNoFilters = "tm-no-filters",
    /// `No targets yet.`
    TmNoTargets = "tm-no-targets",
    /// `Below (descendent-of)`
    TmOpBelow = "tm-op-below",
    /// `Equals (=)`
    TmOpEquals = "tm-op-equals",
    /// `Is a (is-a)`
    TmOpIsA = "tm-op-is-a",
    /// `Operator`
    TmOperator = "tm-operator",
    /// `Optional`
    TmOptional = "tm-optional",
    /// `Concept`
    TmPropConcept = "tm-prop-concept",
    /// `Inactive`
    TmPropInactive = "tm-prop-inactive",
    /// `Parent`
    TmPropParent = "tm-prop-parent",
    /// `Property`
    TmProperty = "tm-property",
    /// `Relationship`
    TmRelationship = "tm-relationship",
    /// `Remove this clause`
    TmRemoveClause = "tm-remove-clause",
    /// `Remove { $value }`
    TmRemoveCode = "tm-remove-code",
    /// `Remove this filter`
    TmRemoveFilter = "tm-remove-filter",
    /// `Remove the code { $value }`
    TmRemoveSource = "tm-remove-source",
    /// `Remove this target of { $value }`
    TmRemoveTarget = "tm-remove-target",
    /// `Security service`
    TmSecurity = "tm-security",
    /// `What the definition selects`
    TmSelects = "tm-selects",
    /// `Server`
    TmServer = "tm-server",
    /// `What the root says it serves`
    TmServes = "tm-serves",
    /// `Software`
    TmSoftware = "tm-software",
    /// `Source system`
    TmSourceSystem = "tm-source-system",
    /// `Stacked, with its system`
    TmStacked = "tm-stacked",
    /// `Coded values, value set clauses, the FHIR roots of a server and a concept map's mappings. The top bar reads through one FHIR version.`
    TmSubtitle = "tm-subtitle",
    /// `Synthetic encounter`
    TmSyntheticEncounter = "tm-synthetic-encounter",
    /// `Target code`
    TmTargetCode = "tm-target-code",
    /// `Target system`
    TmTargetSystem = "tm-target-system",
    /// `Targets of { $value }`
    TmTargetsOf = "tm-targets-of",
    /// `Title`
    TmTitle = "tm-title",
    /// `A URN canonical`
    TmUrnCanonical = "tm-urn-canonical",
    /// `Value`
    TmValue = "tm-value",
    /// `Synthetic respiratory findings`
    TmValueSet = "tm-value-set",
    /// `Value sets`
    TmValueSets = "tm-value-sets",
    /// `Version`
    TmVersion = "tm-version",
    /// `3 (default)`
    TmVersionDefault = "tm-version-default",
    /// `A versioned uid as its code`
    TmVersionedUid = "tm-versioned-uid",
    /// `This clause lists codes and filters at once. Split it into two clauses; the two clauses add together.`
    TmVsd3 = "tm-vsd3",
}
