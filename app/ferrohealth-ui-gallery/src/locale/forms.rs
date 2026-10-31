// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the forms page.

use super::Msg;

messages! {
    /// `Accept a partial answer`
    AcceptPartialAnswer = "accept-partial-answer",
    /// `An action answered on its page`
    ActionAnsweredOnPage = "action-answered-on-page",
    /// `Actions between`
    ActionsBetween = "actions-between",
    /// `Active concepts only`
    ActiveConceptsOnly = "active-concepts-only",
    /// `Add`
    Add = "add",
    /// `Add value`
    AddValue = "add-value",
    /// `The address must be absolute, starting https://.`
    AddressMustAbsoluteStartingHttps = "address-must-absolute-starting-https",
    /// `Administrative gender`
    AdministrativeGender = "administrative-gender",
    /// `Allowed audiences`
    AllowedAudiences = "allowed-audiences",
    /// `Another version`
    AnotherVersion = "another-version",
    /// `Archetype id`
    ArchetypeId = "archetype-id",
    /// `Checked on the server as you type, and when you press Check.`
    ArchetypeIdCheckedAsYouType = "archetype-id-checked-as-you-type",
    /// `The archetype id is empty.`
    ArchetypeIdEmpty = "archetype-id-empty",
    /// `Not an archetype id: …`
    ArchetypeIdRefused = "archetype-id-refused",
    /// `Well formed: the { $kind } archetype { $name }, version { $version }.`
    ArchetypeIdWellFormed = "archetype-id-well-formed",
    /// `Ask these nodes`
    AskTheseNodes = "ask-these-nodes",
    /// `ATC`
    Atc = "atc",
    /// `Back in the queue on { $day }.`
    BackInQueueOn = "back-in-queue-on",
    /// `Base address`
    BaseAddress = "base-address",
    /// The label of the base URL of the member `$member`.
    BaseUrlOf = "base-url-of",
    /// `BCP 47 languages`
    Bcp47Languages = "bcp-47-languages",
    /// `blood pressure`
    BloodPressure = "blood-pressure",
    /// `Bundle, JSON`
    BundleJson = "bundle-json",
    /// `canonical`
    Canonical = "canonical",
    /// `A canonical and a clause are what a save needs.`
    CanonicalClauseWhatSaveNeeds = "canonical-clause-what-save-needs",
    /// `Check`
    Check = "check",
    /// `An archetype id checked as you type`
    CheckArchetypeId = "check-archetype-id",
    /// `Choose a code system`
    ChooseCodeSystem = "choose-code-system",
    /// `Choose from the list`
    ChooseFromList = "choose-from-list",
    /// `Choose a member first`
    ChooseMemberFirst = "choose-member-first",
    /// `Choose OPT file`
    ChooseOptFile = "choose-opt-file",
    /// The combobox label of the clause at `$place`.
    ClauseCodeSystem = "clause-code-system",
    /// `The clauses`
    Clauses = "clauses",
    /// `Clear`
    Clear = "clear",
    /// `Client credentials`
    ClientCredentials = "client-credentials",
    /// `Client id`
    ClientId = "client-id",
    /// `Its client secret`
    ClientSecret = "client-secret",
    /// `Code editor`
    CodeEditor = "code-editor",
    /// `Code system`
    CodeSystem = "code-system",
    /// `A code system this server holds, or the canonical of another.`
    CodeSystemServerHoldsCanonical = "code-system-server-holds-canonical",
    ///  codes
    Codes = "codes",
    /// `Codes the system retired still match.`
    CodesSystemRetiredStillMatch = "codes-system-retired-still-match",
    /// `Combobox`
    Combobox = "combobox",
    /// `a comment after a value is kept but not read`
    CommentAfterValueKeptBut = "comment-after-value-kept-but",
    /// `Composition format`
    CompositionFormat = "composition-format",
    /// `Create`
    Create = "create",
    /// `Danger`
    Danger = "danger",
    /// `Danger, disabled`
    DangerDisabled = "danger-disabled",
    /// `Date fields`
    DateFields = "date-fields",
    /// `Days after today are not offered.`
    DaysAfterTodayNotOffered = "days-after-today-not-offered",
    /// `days`
    DaysUnit = "days-unit",
    /// `Defer until`
    DeferUntil = "defer-until",
    /// `Diagnostics by file`
    DiagnosticsByFile = "diagnostics-by-file",
    /// `Diagnostics at real size`
    DiagnosticsRealSize = "diagnostics-real-size",
    /// `Download the whole configuration as TOML`
    DownloadWholeConfigurationToml = "download-whole-configuration-toml",
    /// The label of the draft of the file `$name`.
    DraftOf = "draft-of",
    /// The heading of the card of the two editor headers.
    EditorHeaders = "editor-headers",
    /// `EHR id (optional)`
    EhrIdOptional = "ehr-id-optional",
    /// `ehr_id (UUID), blank lets the server mint one`
    EhrIdUuidBlankLets = "ehr-id-uuid-blank-lets",
    /// `Add endpoint`
    EpAdd = "ep-add",
    /// `Add profile`
    EpAddProfile = "ep-add-profile",
    /// `Base URL`
    EpBase = "ep-base",
    /// `Query API endpoint { $name }`
    EpHeading = "ep-heading",
    /// `Endpoint id`
    EpId = "ep-id",
    /// `No endpoints yet.`
    EpNone = "ep-none",
    /// `Profile URL`
    EpProfile = "ep-profile",
    /// `Profiles of { $name }`
    EpProfiles = "ep-profiles",
    /// `No profiles yet.`
    EpProfilesNone = "ep-profiles-none",
    /// `Remove the endpoint { $name }`
    EpRemove = "ep-remove",
    /// `Remove the profile { $value }`
    EpRemoveProfile = "ep-remove-profile",
    /// `Endpoints of a member`
    EpTitle = "ep-title",
    /// `Exclude`
    Exclude = "exclude",
    /// `external subject id`
    ExternalSubjectId = "external-subject-id",
    /// `A federated query`
    FederatedQuery = "federated-query",
    /// `A federation member`
    FederationMember = "federation-member",
    /// `Inside the field's edge`
    FieldEdgeParts = "field-edge-parts",
    /// `Field shapes`
    FieldShapes = "field-shapes",
    /// `Fields`
    Fields = "fields",
    /// `Fields, grouped fields, the upload field, the highlighted editor and the filter.`
    FieldsGroupedFieldsUploadField = "fields-grouped-fields-upload-field",
    /// `Fields per row`
    FieldsPerRow = "fields-per-row",
    /// `Fields in a row`
    FieldsRow = "fields-row",
    /// `The file`
    File = "file",
    /// The title of the file editor: `$mode` new or edit, `$name` the file changed.
    FileEditor = "file-editor",
    /// The filter's count: `$kept` of the `$total` files kept.
    FilesOnPage = "files-on-page",
    /// `Filter`
    Filter = "filter",
    /// `Filter the files`
    FilterFiles = "filter-files",
    /// `Find a code in that code system`
    FindCodeCodeSystem = "find-code-code-system",
    /// `flat`
    Flat = "flat",
    /// `From`
    FromDay = "from-day",
    /// `Ghost`
    Ghost = "ghost",
    /// `The grant`
    Grant = "grant",
    /// `Grouped fields`
    GroupedFields = "grouped-fields",
    /// `Highlighted editor`
    HighlightedEditor = "highlighted-editor",
    /// `ICD-10`
    Icd10 = "icd-10",
    /// `In a week`
    InAWeek = "in-a-week",
    /// `In 3 days`
    InThreeDays = "in-three-days",
    /// `Include`
    Include = "include",
    /// `Include inactive codes`
    IncludeInactiveCodes = "include-inactive-codes",
    /// `ISO 3166 countries`
    Iso3166Countries = "iso-3166-countries",
    /// The link to the line `$line` an issue points at.
    IssueLine = "issue-line",
    /// `Keep`
    Keep = "keep",
    /// `Keep the draft until`
    KeepDraftUntil = "keep-draft-until",
    /// `LOINC`
    Loinc = "loinc",
    /// `A long button label`
    LongButtonLabel = "long-button-label",
    /// The crumb of the files listing above the file editor.
    MappingFilesCrumb = "mapping-files-crumb",
    /// `Patient identifier`
    MaskedIdentifier = "masked-identifier",
    /// `It stays masked; hold the eye to check what you typed.`
    MaskedIdentifierHelp = "masked-identifier-help",
    /// `Media types`
    MediaTypes = "media-types",
    /// `The member signs in with no secret once this is saved.`
    MemberSignsNoSecretOnce = "member-signs-no-secret-once",
    /// `ms`
    Milliseconds = "milliseconds",
    /// `namespace`
    Namespace = "namespace",
    /// `New mapping`
    NewMapping = "new-mapping",
    /// `The new mapping file`
    NewMappingFile = "new-mapping-file",
    /// `The new secret`
    NewSecret = "new-secret",
    /// `Next Monday`
    NextMonday = "next-monday",
    /// `metadata.note is no key of the grammar: it is kept and never read.`
    NoteKeptNeverRead = "note-kept-never-read",
    /// `Note to check`
    NoteToCheck = "note-to-check",
    /// `The note has { $count } words.`
    NoteWords = "note-words",
    /// `Observation status`
    ObservationStatus = "observation-status",
    /// `The one this server resolves to`
    OneServerResolves = "one-server-resolves",
    /// `Operational template`
    OperationalTemplate = "operational-template",
    /// A finding whose message holds a long openEHR path.
    PathNamesNoNode = "path-names-no-node",
    /// `Pick or type`
    PickType = "pick-type",
    /// `Primary`
    Primary = "primary",
    /// `Primary, disabled`
    PrimaryDisabled = "primary-disabled",
    /// `The file names no profile the bridge knows.`
    ProfileUnknown = "profile-unknown",
    /// `Remove this clause`
    RemoveClause = "remove-clause",
    /// The name of the button that removes the value `$value`.
    RemoveValue = "remove-value",
    /// `Replace`
    Replace = "replace",
    /// `Retention`
    Retention = "retention",
    /// `retries must be a whole number, such as 3.`
    RetriesMustBeNumber = "retries-must-be-number",
    /// `A reveal inside a reveal`
    RevealInsideReveal = "reveal-inside-reveal",
    /// `Row limit`
    RowLimit = "row-limit",
    /// `rows`
    RowsUnit = "rows-unit",
    /// `Run the query`
    RunQuery = "run-query",
    /// `RxNorm`
    Rxnorm = "rxnorm",
    /// `Save this draft and reload the set. The bridge refuses a draft that does not compile.`
    SaveDraftReloadSetBridge = "save-draft-reload-set-bridge",
    /// `Save this value set`
    SaveValueSet = "save-value-set",
    /// `Secondary`
    Secondary = "secondary",
    /// `Secondary, disabled`
    SecondaryDisabled = "secondary-disabled",
    /// `Set by the deployment.`
    SetByDeployment = "set-by-deployment",
    /// `Show the actions`
    ShowActions = "show-actions",
    /// `Show the identifier while held`
    ShowIdentifierWhileHeld = "show-identifier-while-held",
    /// The editor's size: `$bytes` bytes in `$lines` lines.
    SizeLines = "size-lines",
    /// `Small`
    Small = "small",
    /// `SNOMED CT`
    SnomedCt = "snomed-ct",
    /// `Subject id (optional)`
    SubjectIdOptional = "subject-id-optional",
    /// `Subject namespace (optional)`
    SubjectNamespaceOptional = "subject-namespace-optional",
    /// `Synthetic gallery codes`
    SyntheticGalleryCodes = "synthetic-gallery-codes",
    /// `The system this clause draws its codes from.`
    SystemClauseDrawsCodesFrom = "system-clause-draws-codes-from",
    /// `Target path (optional)`
    TargetPathOptional = "target-path-optional",
    /// `the template gallery.template.v1 is not loaded`
    TemplateGalleryTemplateV1Not = "template-gallery-template-v1-not",
    /// `Ten values at most.`
    TenValuesMost = "ten-values-most",
    /// `At time`
    Time = "time",
    /// `Time travel (UTC)`
    TimeTravelUtc = "time-travel-utc",
    /// `To`
    ToDay = "to-day",
    /// `A token`
    Token = "token",
    /// `Token`
    TokenLabel = "token-label",
    /// `Tomorrow`
    Tomorrow = "tomorrow",
    /// `Two hundred characters at most.`
    TwoHundredCharactersMost = "two-hundred-characters-most",
    /// `Two letters or more.`
    TwoLettersMore = "two-letters-more",
    /// `Type a whole number.`
    TypeWholeNumber = "type-whole-number",
    /// `UCUM`
    Ucum = "ucum",
    /// `Upload`
    Upload = "upload",
    /// `UTF-8 · LF`
    Utf8Lf = "utf-8-lf",
    /// `Version 1`
    Version1 = "version-1",
    /// `A version this server holds, or another one you type.`
    VersionServerHoldsAnotherOne = "version-server-holds-another-one",
    /// `When a node does not answer, the rows of the others still show.`
    WhenNodeDoesNotAnswer = "when-node-does-not-answer",
    /// `Where the gateway reaches the member.`
    WhereGatewayReachesMember = "where-gateway-reaches-member",
    /// `Why you save it, one line`
    WhyYouSaveOneLine = "why-you-save-one-line",
    /// `YAML · FHIRconnect context`
    YamlFhirconnectContext = "yaml-fhirconnect-context",
}
