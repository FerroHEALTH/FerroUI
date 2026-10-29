// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the overview page.

use super::Msg;

messages! {
    /// The panel's header action that leads to every file.
    AllFiles = "all-files",
    /// `Answer`
    Answer = "answer",
    /// `The answer as the server sent it.`
    AnswerServerSent = "answer-server-sent",
    /// `Appearance`
    Appearance = "appearance",
    /// `Asking the bridge…`
    AskingBridge = "asking-bridge",
    /// `Back to the files`
    BackFiles = "back-files",
    /// A banner's title in the overview's specimen.
    BannerSandbox = "banner-sandbox",
    /// A banner's sentence around a URL longer than a phone is wide.
    BannerSandboxBody = "banner-sandbox-body",
    /// `Banners`
    Banners = "banners",
    /// `Past five tabs, a phone shows the sections as a select once the page has loaded its script.`
    BinderManyTabsSelect = "binder-many-tabs-select",
    /// `The tab you are on opens into this panel, so the two read as one sheet.`
    BinderPanelJoined = "binder-panel-joined",
    /// `Notifications holds a draft; Keyboard leaves the row on a phone, which has no keyboard.`
    BinderPreferencesDraft = "binder-preferences-draft",
    /// `the bridge refused the request.`
    BridgeRefusedRequest = "bridge-refused-request",
    /// `Built`
    Built = "built",
    /// `A bullet list`
    BulletList = "bullet-list",
    /// `Bundle.entry[0].resource`
    BundleEntry0Resource = "bundle-entry-0-resource",
    /// `Business Source License 1.1`
    BusinessSourceLicense11 = "business-source-license-1-1",
    /// `Buttons`
    Buttons = "buttons",
    /// `A card titled with a file's path.`
    CardTitledFileSPath = "card-titled-file-s-path",
    /// `the CDR did not answer within 30 seconds.`
    CdrDidNotAnswerWithin = "cdr-did-not-answer-within",
    /// `Commit`
    Commit = "commit",
    /// `Compiled`
    Compiled = "compiled",
    /// `Compositions`
    Compositions = "compositions",
    /// `The console cannot reach the bridge.`
    ConsoleCannotReachBridge = "console-cannot-reach-bridge",
    /// `Contributions`
    Contributions = "contributions",
    /// `Default language`
    DefaultLanguage = "default-language",
    /// `Definition grid`
    DefinitionGrid = "definition-grid",
    /// `Degraded`
    Degraded = "degraded",
    /// `Detail`
    Detail = "detail",
    /// `Diagnostics`
    Diagnostics = "diagnostics",
    /// `Directory`
    Directory = "directory",
    /// `Disabled`
    Disabled = "disabled",
    /// `Disclosure and well`
    DisclosureWell = "disclosure-well",
    /// `Down`
    Down = "down",
    /// `Empty state`
    EmptyState = "empty-state",
    /// `Every static view of the library, in this brand's theme.`
    EveryStaticViewLibraryBrand = "every-static-view-library-brand",
    /// `Expand a value set`
    ExpandValueSet = "expand-value-set",
    /// `Facts across a row`
    FactsAcrossRow = "facts-across-row",
    /// `FHIR`
    Fhir = "fhir",
    /// `FHIRconnect`
    FhirConnect = "fhir-connect",
    /// `FHIR R4`
    FhirR4 = "fhir-r4",
    /// `FHIR version`
    FhirVersion = "fhir-version",
    /// `{ $count } findings`
    FindingsCount = "findings-count",
    /// `FLAT`
    FlatUpper = "flat-upper",
    /// `Format`
    Format = "format",
    /// `From the care services directory`
    FromDirectory = "from-directory",
    /// `Full`
    Full = "full",
    /// `GET https://term.example.org/fhir/ValueSet/$expand?url=http://example.org/vs`
    GetHttpsTermExampleOrg = "get-https-term-example-org",
    /// `Icons`
    Icons = "icons",
    /// `Keyboard`
    Keyboard = "keyboard",
    /// `Keyboard tabs: Left, Right, Home and End move between them.`
    KeyboardTabsLeftRightHome = "keyboard-tabs-left-right-home",
    /// `Languages`
    Languages = "languages",
    /// `Last changed`
    LastChanged = "last-changed",
    /// `Licence`
    Licence = "licence",
    /// `A link as a button`
    LinkButton = "link-button",
    /// `Loading the files.`
    LoadingFiles = "loading-files",
    /// `Locale`
    Locale = "locale",
    /// `Mapping files`
    MappingFiles = "mapping-files",
    /// `Excluded`
    MarkExcluded = "mark-excluded",
    /// `2,940 ms`
    MarkLatency = "mark-latency",
    /// `Not checked`
    MarkNotChecked = "mark-not-checked",
    /// `Reachable`
    MarkReachable = "mark-reachable",
    /// `Unreachable`
    MarkUnreachable = "mark-unreachable",
    /// `Members`
    Members = "members",
    /// `Minimal`
    Minimal = "minimal",
    /// `MSG_UNKNOWN_TYPE`
    UnknownTypeCode = "msg-unknown-type",
    /// `1 of 3 lanes down`
    N13LanesDown = "n-1-3-lanes-down",
    /// `1 failing`
    N1Failing = "n-1-failing",
    /// `New file`
    NewFile = "new-file",
    /// `No file matches the filter.`
    NoFileMatchesFilter = "no-file-matches-filter",
    /// `No match`
    NoMatch = "no-match",
    /// `No program maps the template gallery.template.v1.`
    NoProgramMapsTemplateGallery = "no-program-maps-template-gallery",
    /// `Notices`
    Notices = "notices",
    /// `Notifications`
    Notifications = "notifications",
    /// `POST https://bridge.example.org/fhir/Bundle?_format=json`
    PostHttpsBridgeExampleOrg = "post-https-bridge-example-org",
    /// `A plain form post, with or without script`
    PostsTheDensity = "posts-the-density",
    /// `Product`
    ProductColumn = "product-column",
    /// `Profile`
    Profile = "profile",
    /// `A quiet empty section`
    QuietEmptySection = "quiet-empty-section",
    /// `R4`
    R4 = "r4",
    /// `R4B`
    R4b = "r4b",
    /// `R5`
    R5 = "r5",
    /// `R6`
    R6 = "r6",
    /// `Raw`
    Raw = "raw",
    /// `The raw document, folded until it is asked for.`
    RawDocumentFoldedUntilAsked = "raw-document-folded-until-asked",
    /// `Read`
    ReadOperation = "read-operation",
    /// `Reading the capability statement`
    ReadingCapabilityStatement = "reading-capability-statement",
    /// The heading of the panel that previews the files.
    RecentFiles = "recent-files",
    /// `Record`
    Record = "record",
    /// `Refusal and fault`
    RefusalFault = "refusal-fault",
    /// `Request`
    Request = "request",
    /// `revision 3 pending`
    Revision3Pending = "revision-3-pending",
    /// `The rows of the answer, as a table.`
    RowsAnswerTable = "rows-answer-table",
    /// `Run $expand over a value set this server holds.`
    RunExpandOverValueSet = "run-expand-over-value-set",
    /// `Running revision on every federation node`
    RunningRevisionEveryFederationNode = "running-revision-every-federation-node",
    /// A refusal notice with its issues, a line each.
    SaveRefusedIssues = "save-refused-issues",
    /// `Segmented controls`
    SegmentedControls = "segmented-controls",
    /// `self, with no external subject reference, so the record names no person outside the server`
    SelfNoExternalSubjectReference = "self-no-external-subject-reference",
    /// `Serving`
    Serving = "serving",
    /// `The set compiled and replaced the running one.`
    SetCompiledReplacedRunningOne = "set-compiled-replaced-running-one",
    /// `Settings`
    Settings = "settings",
    /// The line naming the format `$format` the document is shown in.
    ShownAs = "shown-as",
    /// `Signing in`
    Signing = "signing",
    /// `SMART configuration returned, but it is not valid JSON.`
    SmartConfigurationReturnedButNot = "smart-configuration-returned-but-not",
    /// `SMART is not enabled on this synthetic server.`
    SmartNotEnabledSyntheticServer = "smart-not-enabled-synthetic-server",
    /// `Source`
    Source = "source",
    /// `Status history`
    StatusHistory = "status-history",
    /// `STRUCTURED`
    Structured = "structured",
    /// `Subject`
    Subject = "subject",
    /// `Switch to the compact density`
    SwitchToCompact = "switch-to-compact",
    /// `Synthetic evaluation`
    SyntheticEvaluation = "synthetic-evaluation",
    /// `This synthetic version is deleted.`
    SyntheticVersionDeleted = "synthetic-version-deleted",
    /// `Table`
    Table = "table",
    /// `Tabs`
    Tabs = "tabs",
    /// `Tags`
    Tags = "tags",
    /// `Target`
    Target = "target",
    /// `the terminology server answered with no readable outcome.`
    TerminologyServerAnsweredNoReadable = "terminology-server-answered-no-readable",
    /// `Three changes wait to be applied to the running set.`
    ThreeChangesWaitAppliedRunning = "three-changes-wait-applied-running",
    /// `$tofhir http://fhirconnect.org/fhir/OperationDefinition/ToFhirSynthetic`
    TofhirHttpFhirconnectOrgFhir = "tofhir-http-fhirconnect-org-fhir",
    /// `Uid`
    Uid = "uid",
    /// `$validate-code on the value sets the server loaded`
    ValidateCodeValueSetsServer = "validate-code-value-sets-server",
    /// `Version uid`
    VersionUid = "version-uid",
    /// `Views of a file`
    ViewsFile = "views-file",
    /// `wasm32-unknown-unknown`
    Wasm32UnknownUnknown = "wasm32-unknown-unknown",
    /// `A well sets a passage apart inside a card, such as a note on what the figures mean.`
    WellSetsPassageApartInside = "well-sets-passage-apart-inside",
    /// `What the bridge answered, as JSON`
    WhatBridgeAnsweredJson = "what-bridge-answered-json",
    /// `What the console asked.`
    WhatConsoleAsked = "what-console-asked",
    /// `With a warning`
    WithAWarning = "with-a-warning",
    /// `Your preferences`
    YourPreferences = "your-preferences",
}
