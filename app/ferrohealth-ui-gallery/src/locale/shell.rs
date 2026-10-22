// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the gallery's shell and those more than one page draws.

use super::Msg;

messages! {
    /// `Actions for $name`
    ActionsFor = "actions-for",
    /// `Another canonical`
    AnotherCanonical = "another-canonical",
    /// `Back to the gallery`
    BackGallery = "back-gallery",
    /// `Base URL`
    BaseUrl = "base-url",
    /// `Basic sign-in`
    BasicSign = "basic-sign",
    /// `Brand`
    Brand = "brand",
    /// `BRIDGE`
    Bridge = "bridge",
    /// `The bridge serves no mapping file yet.`
    BridgeServesNoMappingFile = "bridge-serves-no-mapping-file",
    /// `Charts`
    Charts = "charts",
    /// `Choose one this server holds`
    ChooseOneServerHolds = "choose-one-server-holds",
    /// `Clinical records`
    ClinicalRecords = "clinical-records",
    /// `Code`
    Code = "code",
    /// `Components`
    Components = "components",
    /// `Concept`
    Concept = "concept",
    /// `Configuration`
    Configuration = "configuration",
    /// `Console`
    Console = "console",
    /// `Delete`
    Delete = "delete",
    /// `Display`
    Display = "display",
    /// `Documents`
    Documents = "documents",
    /// `EHR`
    Ehr = "ehr",
    /// `Every component of the library in every product theme. Choose a brand.`
    EveryComponentLibraryEveryProduct = "every-component-library-every-product",
    /// `every row and text on these pages is invented for the gallery; none is patient data.`
    EveryRowTextThesePages = "every-row-text-these-pages",
    /// `FED`
    Fed = "fed",
    /// `FerroHEALTH UI`
    FerrohealthUi = "ferrohealth-ui",
    /// `A file, a template or a path`
    FileTemplatePath = "file-template-path",
    /// `Files`
    Files = "files",
    /// `The anchor start is read again as the fallback.`
    FindingStartTwice = "finding-start-twice",
    /// `Forms and editor`
    FormsEditor = "forms-editor",
    /// `Gallery`
    Gallery = "gallery",
    /// `Gallery operator · a synthetic identity`
    GalleryOperatorLine = "gallery-operator-line",
    /// `History`
    History = "history",
    /// `JSON`
    Json = "json",
    /// `Language`
    Language = "language",
    /// `Latest`
    Latest = "latest",
    /// `Layouts`
    Layouts = "layouts",
    /// `Library up`
    LibraryUp = "library-up",
    /// `ferrohealth-ui { $version }, rendered on the server`
    LibraryVersion = "library-version",
    /// `Loading`
    Loading = "loading",
    /// `A mapping file, YAML`
    MappingFileYaml = "mapping-file-yaml",
    /// `Member`
    Member = "member",
    /// `More actions`
    MoreActions = "more-actions",
    /// `More actions for the mapping set`
    MoreActionsSet = "more-actions-set",
    /// `Name`
    Name = "name",
    /// `About`
    NavAbout = "nav-about",
    /// `T2 Detail`
    NavDetail = "nav-detail",
    /// `Error 500`
    NavFault = "nav-fault",
    /// `Error 403`
    NavForbidden = "nav-forbidden",
    /// `{ $count } issues to look at`
    NavIssues = "nav-issues",
    /// `T1 List`
    NavList = "nav-list",
    /// `T1 Empty`
    NavListEmpty = "nav-list-empty",
    /// `T1 Many items`
    NavListMany = "nav-list-many",
    /// `T1 With tabs`
    NavListTabs = "nav-list-tabs",
    /// `Error 404`
    NavNotFound = "nav-not-found",
    /// `Not known: the gallery does not probe the charts`
    NavNotProbed = "nav-not-probed",
    /// `T3 Your preferences`
    NavPreferences = "nav-preferences",
    /// `T3 From a file`
    NavSettings = "nav-settings",
    /// `T3 Managed`
    NavSettingsManaged = "nav-settings-managed",
    /// `No mapping file`
    NoMappingFile = "no-mapping-file",
    /// `Not found`
    NotFound = "not-found",
    /// `404: the gallery has no page at this address.`
    NotFoundBody = "not-found-body",
    /// `Open`
    Open = "open",
    /// `Open the document`
    OpenTheDocument = "open-the-document",
    /// `Operations`
    Operations = "operations",
    /// `Overlays`
    Overlays = "overlays",
    /// `Overview`
    Overview = "overview",
    /// `Pages`
    Pages = "pages",
    /// `PIX`
    Pix = "pix",
    /// The gallery's product name in the footer.
    Product = "product",
    /// `Provider sign-in`
    ProviderSign = "provider-sign",
    /// `Remove`
    Remove = "remove",
    /// `Save`
    Save = "save",
    /// `Search`
    Search = "search",
    /// `All results in Tables`
    SearchAllResults = "search-all-results",
    /// `Browse every file`
    SearchBrowse = "search-browse",
    /// `{ $kind } · { $lines } lines`
    SearchFileLine = "search-file-line",
    /// `Names that start with “{ $query }”`
    SearchFoundHead = "search-found-head",
    /// `Search waits while the index is rebuilding`
    SearchIndexRebuilding = "search-index-rebuilding",
    /// `Mapping files`
    SearchKindFiles = "search-kind-files",
    /// `Templates`
    SearchKindTemplates = "search-kind-templates",
    /// `Search the mapping files`
    SearchMappingFiles = "search-mapping-files",
    /// `No file found for “{ $query }”`
    SearchNoFile = "search-no-file",
    /// `Check the spelling, or browse every file in Tables.`
    SearchNoFileLine = "search-no-file-line",
    /// `Operational template`
    SearchTemplateLine = "search-template-line",
    /// `Sign in`
    SignIn = "sign-in",
    /// `Sign-in`
    SignInGroup = "sign-in-group",
    /// `Sign-in states`
    SignInStates = "sign-in-states",
    /// `Staged changes`
    StagedChanges = "staged-changes",
    /// `The staged configuration against the running one`
    StagedConfigurationAgainstRunningOne = "staged-configuration-against-running-one",
    /// `Status`
    Status = "status",
    /// `Tables and paging`
    TablesPaging = "tables-paging",
    /// `Page templates`
    Templates = "templates",
    /// `TERM`
    Term = "term",
    /// `Terminology`
    Terminology = "terminology",
    /// `The code`
    TheCode = "the-code",
    /// `Timeout`
    Timeout = "timeout",
    /// `Trees`
    Trees = "trees",
    /// `Try again`
    TryAgain = "try-again",
    /// `Value`
    Value = "value",
    /// `Version`
    Version = "version",
    /// `XML`
    Xml = "xml",
    /// `YAML`
    Yaml = "yaml",
}
