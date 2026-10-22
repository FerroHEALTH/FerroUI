// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the tables page.

use super::Msg;

messages! {
    /// `Actions`
    Actions = "actions",
    /// `Apply`
    Apply = "apply",
    /// `Changed`
    Changed = "changed",
    /// `Committed`
    Committed = "committed",
    /// `Composition`
    Composition = "composition",
    /// `The concepts of a synthetic hierarchy, each a level under its parent`
    ConceptsSyntheticHierarchy = "concepts-synthetic-hierarchy",
    /// `count`
    Count = "count",
    /// `Edit`
    Edit = "edit",
    /// `File`
    FileColumn = "file-column",
    /// `Find a mapping file`
    FindFile = "find-file",
    /// `Fits a phone as it is`
    FitsPhone = "fits-phone",
    /// `Health indicators, unstacked at every width`
    HealthIndicatorsUnstackedEveryWidth = "health-indicators-unstacked-every-width",
    /// `A branch`
    HierarchyBranchA = "hierarchy-branch-a",
    /// `B branch`
    HierarchyBranchB = "hierarchy-branch-b",
    /// `Nine levels down, at the deepest step`
    HierarchyDeep = "hierarchy-deep",
    /// `A hierarchy in a table`
    HierarchyInTable = "hierarchy-in-table",
    /// `A leaf of branch A`
    HierarchyLeafA1 = "hierarchy-leaf-a1",
    /// `Another leaf of branch A`
    HierarchyLeafA2 = "hierarchy-leaf-a2",
    /// `A leaf of that twig`
    HierarchyLeafB1x = "hierarchy-leaf-b1x",
    /// `Root of the hierarchy`
    HierarchyRoot = "hierarchy-root",
    /// `A twig of branch B`
    HierarchyTwigB1 = "hierarchy-twig-b1",
    /// `http://example.org/fhir/StructureDefinition/ferrohealth-synthetic-journey-diagnose`
    HttpExampleOrgFhirStructuredefinition = "http-example-org-fhir-structuredefinition",
    /// `Indicator`
    Indicator = "indicator",
    /// `Kind`
    Kind = "kind",
    /// `Lines`
    Lines = "lines",
    /// `Loading and empty`
    LoadingEmpty = "loading-empty",
    /// `Mapping files, paged in the page`
    MappingFilesPagedPage = "mapping-files-paged-page",
    /// `Mapping files, paged in the URL`
    MappingFilesPagedUrl = "mapping-files-paged-url",
    /// `Members, each with an Edit link and a Remove form`
    MembersEachEditLinkRemove = "members-each-edit-link-remove",
    /// `name`
    NameLower = "name-lower",
    /// `No rows`
    NoRows = "no-rows",
    /// `No stated total`
    NoStatedTotal = "no-stated-total",
    /// `The noun of the EHRs, by count, with its empty and opening forms.`
    NounEhrs = "noun-ehrs",
    /// `The noun of a file's lines, by count, with its empty and opening forms.`
    NounLines = "noun-lines",
    /// `The noun of the mapping files, by count, with its empty and opening forms.`
    NounMappingFiles = "noun-mapping-files",
    /// `One table and pager model: in the URL, or in the page for a screen that can show...`
    OneTablePagerModelUrl = "one-table-pager-model-url",
    /// `Paged in the page`
    PagedPage = "paged-page",
    /// `Paged in the URL`
    PagedUrl = "paged-url",
    /// `Pages of the EHRs, as links`
    PagesEhrsLinks = "pages-ehrs-links",
    /// `Pages of the EHRs, in the page`
    PagesEhrsPage = "pages-ehrs-page",
    /// `Pages of the mapping files`
    PagesMappingFiles = "pages-mapping-files",
    /// `Pages of the mapping files, in the page`
    PagesMappingFilesPage = "pages-mapping-files-page",
    /// `Rate`
    Rate = "rate",
    /// `A result in pages`
    ResultPages = "result-pages",
    /// `A result set`
    ResultSet = "result-set",
    /// `Mapping files to review, sorted by state, then lines, then name`
    ReviewCaption = "review-caption",
    /// `Mapping files to review`
    ReviewFiles = "review-files",
    /// `Sorted by state, then lines, then name`
    ReviewOrder = "review-order",
    /// `Row actions`
    RowActions = "row-actions",
    /// `The rows the synthetic query answered`
    RowsSyntheticQueryAnswered = "rows-synthetic-query-answered",
    /// `Select $name`
    SelectFile = "select-file",
    /// `Selected`
    SelectedColumn = "selected-column",
    /// `State`
    State = "state",
    /// `Synthetic profile`
    SyntheticProfile = "synthetic-profile",
    /// `The synthetic query's rows, a body per page`
    SyntheticQuerySRowsBody = "synthetic-query-s-rows-body",
    /// `Synthetic vital signs`
    SyntheticVitalSigns = "synthetic-vital-signs",
}
