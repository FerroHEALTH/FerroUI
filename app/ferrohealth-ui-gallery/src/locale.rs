// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's locales, English and a pseudo-locale, and the catalog of
//! the gallery's own words.
//!
//! The gallery's page strings are messages of its own Fluent catalog
//! (`locales/en/ferrohealth-ui-gallery.ftl`), looked up as a console looks
//! up its own, so the gallery is held to the rule it shows. English is the
//! one catalog. The pseudo-locale draws every library string and every
//! gallery string accented and lengthened, so a page shows where a longer
//! translation would overflow or wrap.
//!
//! The pseudo-locale is the gallery's own and never ships: it is a catalog
//! the gallery hands the library with [`Locale::with_catalog`], drawing on
//! the library's English catalog and the gallery's. Its tag is `qps`, a private-use
//! language subtag (BCP 47, `qaa` to `qtz`), so no catalog the library ships
//! can serve it. No specification governs pseudo-localization: our own
//! design, after the common practice of accenting and padding each string.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

use ferrohealth_ui::i18n::{Arg, Locale, Message, library_catalog, locale};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader};

fluent_templates::static_loader! {
    static GALLERY = {
        locales: "./locales",
        fallback_language: "en",
        // NOTE: Fluent's guide, "Bidirectionality": the isolation marks around a
        // placeable are for mixed-direction text; the English catalog draws none.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// The tag of the pseudo-locale.
pub const PSEUDO_TAG: &str = "qps";

/// The pseudo-locale's language.
static PSEUDO_LANGUAGE: LazyLock<LanguageIdentifier> =
    LazyLock::new(|| fluent_templates::langid!("qps"));

/// The gallery's catalog: its own words in every language it holds, and in
/// the pseudo-locale the library's English and its own, stretched.
#[derive(Debug, Clone, Copy)]
pub struct Catalog;

/// The one catalog the gallery looks its words up in and hands the library.
static CATALOG: Catalog = Catalog;

/// The gallery's catalog, for a [`Message`] the library draws.
#[must_use]
pub fn catalog() -> &'static (dyn Loader + Send + Sync) {
    &CATALOG
}

impl Loader for Catalog {
    fn lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> String {
        self.try_lookup_complete(lang, text_id, args)
            .unwrap_or_else(|| text_id.to_owned())
    }

    fn try_lookup_complete(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
    ) -> Option<String> {
        if *lang != *PSEUDO_LANGUAGE {
            return GALLERY.try_lookup_complete(lang, text_id, args);
        }
        let english = fluent_templates::langid!("en");
        library_catalog()
            .try_lookup_complete(&english, text_id, args)
            .or_else(|| GALLERY.try_lookup_complete(&english, text_id, args))
            .map(|english| stretch(&english))
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(GALLERY.locales().chain(std::iter::once(&*PSEUDO_LANGUAGE)))
    }
}

/// `english` accented letter by letter and padded by two fifths of its
/// length (at least three), in brackets, so a cut end shows.
#[must_use]
pub fn stretch(english: &str) -> String {
    let accented: String = english.chars().map(accent).collect();
    let length = english.chars().count();
    let pad = (length.saturating_mul(2).div_ceil(5)).max(3);
    format!("[{accented} {}]", "~".repeat(pad))
}

/// The accented look-alike of `letter`, the letter itself when it has none.
const fn accent(letter: char) -> char {
    match letter {
        'a' => 'á',
        'c' => 'ç',
        'e' => 'é',
        'i' => 'í',
        'n' => 'ñ',
        'o' => 'ö',
        'u' => 'ü',
        'y' => 'ý',
        'A' => 'Å',
        'C' => 'Ç',
        'E' => 'É',
        'I' => 'Î',
        'N' => 'Ñ',
        'O' => 'Ö',
        'U' => 'Ü',
        'Y' => 'Ý',
        other => other,
    }
}

/// The locale the tag `tag` names among the gallery's: the pseudo-locale
/// for its tag, English for any other.
#[must_use]
pub fn gallery_locale(tag: &str) -> Locale {
    if tag == PSEUDO_TAG {
        Locale::parse(PSEUDO_TAG).map_or_else(
            |_| Locale::english(),
            |locale| locale.with_catalog(&CATALOG),
        )
    } else {
        Locale::english()
    }
}

/// The locales the gallery's language switch offers.
#[must_use]
pub fn choices() -> Vec<Locale> {
    vec![Locale::english(), gallery_locale(PSEUDO_TAG)]
}

/// Declares the gallery's messages: each variant names one message of the
/// catalog by its id.
macro_rules! messages {
    ($($(#[$meta:meta])+ $name:ident = $id:literal,)+) => {
        /// A string the gallery draws, by its message in the catalog.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Msg {
            $($(#[$meta])+ $name,)+
        }

        impl Msg {
            /// Every message, in declaration order.
            pub const ALL: &[Self] = &[$(Self::$name,)+];

            /// The message's id in the catalog.
            #[must_use]
            pub const fn id(self) -> &'static str {
                match self {
                    $(Self::$name => $id,)+
                }
            }
        }
    };
}

messages! {
    /// The gallery's product name in the footer.
    Product = "product",

    /// `Back to the gallery`
    BackGallery = "back-gallery",
    /// `404: the gallery has no page at this address.`
    NotFoundBody = "not-found-body",
    /// `Not found`
    NotFound = "not-found",
    /// `Every component of the library in every product theme. Choose a brand.`
    EveryComponentLibraryEveryProduct = "every-component-library-every-product",
    /// `FerroHEALTH UI`
    FerrohealthUi = "ferrohealth-ui",
    /// `Gallery`
    Gallery = "gallery",
    /// `Pages`
    Pages = "pages",
    /// `every row and text on these pages is invented for the gallery; none is patient data.`
    EveryRowTextThesePages = "every-row-text-these-pages",
    /// `A file, a template or a path`
    FileTemplatePath = "file-template-path",
    /// `Search the mapping files`
    SearchMappingFiles = "search-mapping-files",
    /// `Signed in to the gallery`
    SignedGallery = "signed-gallery",
    /// `Library v0.1.7`
    LibraryV017 = "library-v0-1-7",
    /// `Brand`
    Brand = "brand",
    /// `TERM`
    Term = "term",
    /// `EHR`
    Ehr = "ehr",
    /// `FED`
    Fed = "fed",
    /// `BRIDGE`
    Bridge = "bridge",
    /// `Sign-in`
    SignInGroup = "sign-in-group",
    /// `Components`
    Components = "components",
    /// `Both methods`
    BothMethods = "both-methods",
    /// `Provider sign-in`
    ProviderSign = "provider-sign",
    /// `Basic sign-in`
    BasicSign = "basic-sign",
    /// `Layouts`
    Layouts = "layouts",
    /// `Charts`
    Charts = "charts",
    /// `Overlays`
    Overlays = "overlays",
    /// `Documents`
    Documents = "documents",
    /// `Tables and paging`
    TablesPaging = "tables-paging",
    /// `Forms and editor`
    FormsEditor = "forms-editor",
    /// `Overview`
    Overview = "overview",
    /// `Drawn as SVG on the server, scaled to the card, each with its figures for assistive...`
    DrawnSvgServerScaledCard = "drawn-svg-server-scaled-card",
    /// `Sep 19`
    Sep19 = "sep-19",
    /// `Oct 18`
    Oct18 = "oct-18",
    /// `48 synthetic commits over 30 days, most on day 21.`
    N48SyntheticCommitsOver30 = "n-48-synthetic-commits-over-30",
    /// `Commits per day`
    CommitsPerDay = "commits-per-day",
    /// `Activity`
    Activity = "activity",
    /// `Line chart`
    LineChart = "line-chart",
    /// `Refusals`
    Refusals = "refusals",
    /// `Writes`
    Writes = "writes",
    /// `Reads`
    Reads = "reads",
    /// `Three synthetic series over a week: reads rise to 48 on Friday, writes stay near 12,...`
    ThreeSyntheticSeriesOverWeek = "three-synthetic-series-over-week",
    /// `Requests per day`
    RequestsPerDay = "requests-per-day",
    /// `Mon`
    Mon = "mon",
    /// `Tue`
    Tue = "tue",
    /// `Wed`
    Wed = "wed",
    /// `Thu`
    Thu = "thu",
    /// `Fri`
    Fri = "fri",
    /// `Sat`
    Sat = "sat",
    /// `Sun`
    Sun = "sun",
    /// `Code and documents`
    CodeDocuments = "code-documents",
    /// `Answered by the synthetic replica gallery-1.`
    AnsweredSyntheticReplicaGallery1 = "answered-synthetic-replica-gallery-1",
    /// `Bounded, scrollable and highlighted, never at the page's full height.`
    BoundedScrollableHighlightedNeverPage = "bounded-scrollable-highlighted-never-page",
    /// `The mapping file, numbered`
    MappingFileNumbered = "mapping-file-numbered",
    /// `A diagnostic links to its line:`
    DiagnosticLinksLine = "diagnostic-links-line",
    /// `line 13`
    Line13 = "line-13",
    /// `Numbered lines`
    NumberedLines = "numbered-lines",
    /// `AQL`
    Aql = "aql",
    /// `A query, AQL`
    QueryAql = "query-aql",
    /// `XML`
    Xml = "xml",
    /// `A composition, XML`
    CompositionXml = "composition-xml",
    /// `JSON`
    Json = "json",
    /// `A resource, JSON`
    ResourceJson = "resource-json",
    /// `YAML`
    Yaml = "yaml",
    /// `A mapping file, YAML`
    MappingFileYaml = "mapping-file-yaml",
    /// `Observation`
    Observation = "observation",
    /// `Document pane`
    DocumentPane = "document-pane",
    /// `Value`
    Value = "value",
    /// `72.5 beats/min`
    N725BeatsMin = "n-72-5-beats-min",
    /// `Code`
    Code = "code",
    /// `A synthetic measurement`
    SyntheticMeasurement = "synthetic-measurement",
    /// `Status`
    Status = "status",
    /// `Resource`
    Resource = "resource",
    /// `Fields, grouped fields, the upload field, the highlighted editor and the filter.`
    FieldsGroupedFieldsUploadField = "fields-grouped-fields-upload-field",
    /// `Files`
    Files = "files",
    /// `Filter the files`
    FilterFiles = "filter-files",
    /// `Filter`
    Filter = "filter",
    /// `Read it again`
    ReadAgain = "read-again",
    /// `An editor that reloads`
    EditorReloads = "editor-reloads",
    /// `The file`
    File = "file",
    /// `Highlighted editor`
    HighlightedEditor = "highlighted-editor",
    /// `the template gallery.template.v1 is not loaded`
    TemplateGalleryTemplateV1Not = "template-gallery-template-v1-not",
    /// `a comment after a value is kept but not read`
    CommentAfterValueKeptBut = "comment-after-value-kept-but",
    /// `Choose OPT file`
    ChooseOptFile = "choose-opt-file",
    /// `Operational template`
    OperationalTemplate = "operational-template",
    /// `Upload`
    Upload = "upload",
    /// `Clear`
    Clear = "clear",
    /// `Download the whole configuration as TOML`
    DownloadWholeConfigurationToml = "download-whole-configuration-toml",
    /// `A long button label`
    LongButtonLabel = "long-button-label",
    /// `Save the settings`
    SaveSettings = "save-settings",
    /// `A long form`
    LongForm = "long-form",
    /// `Client credentials`
    ClientCredentials = "client-credentials",
    /// `A token`
    Token = "token",
    /// `The grant`
    Grant = "grant",
    /// `A reveal inside a reveal`
    RevealInsideReveal = "reveal-inside-reveal",
    /// `Token`
    TokenLabel = "token-label",
    /// `Client id`
    ClientId = "client-id",
    /// `The new secret`
    NewSecret = "new-secret",
    /// `Keep`
    Keep = "keep",
    /// `Replace`
    Replace = "replace",
    /// `Its client secret`
    ClientSecret = "client-secret",
    /// `The member signs in with no secret once this is saved.`
    MemberSignsNoSecretOnce = "member-signs-no-secret-once",
    /// `Remove`
    Remove = "remove",
    /// `Search`
    Search = "search",
    /// `Choose a code system`
    ChooseCodeSystem = "choose-code-system",
    /// `Find a code in that code system`
    FindCodeCodeSystem = "find-code-code-system",
    /// `Choose a member first`
    ChooseMemberFirst = "choose-member-first",
    /// `Code system`
    CodeSystem = "code-system",
    /// `A federation member`
    FederationMember = "federation-member",
    /// `Grouped fields`
    GroupedFields = "grouped-fields",
    /// `Base URL`
    BaseUrl = "base-url",
    /// `Member`
    Member = "member",
    /// `A canonical and a clause are what a save needs.`
    CanonicalClauseWhatSaveNeeds = "canonical-clause-what-save-needs",
    /// `Remove this clause`
    RemoveClause = "remove-clause",
    /// `Save this value set`
    SaveValueSet = "save-value-set",
    /// `Active concepts only`
    ActiveConceptsOnly = "active-concepts-only",
    /// `The code`
    TheCode = "the-code",
    /// `The clauses`
    Clauses = "clauses",
    /// `Combobox`
    Combobox = "combobox",
    /// `Synthetic gallery codes`
    SyntheticGalleryCodes = "synthetic-gallery-codes",
    /// `Observation status`
    ObservationStatus = "observation-status",
    /// `Administrative gender`
    AdministrativeGender = "administrative-gender",
    /// `Media types`
    MediaTypes = "media-types",
    /// `BCP 47 languages`
    Bcp47Languages = "bcp-47-languages",
    /// `ISO 3166 countries`
    Iso3166Countries = "iso-3166-countries",
    /// `RxNorm`
    Rxnorm = "rxnorm",
    /// `ATC`
    Atc = "atc",
    /// `ICD-10`
    Icd10 = "icd-10",
    /// `UCUM`
    Ucum = "ucum",
    /// `SNOMED CT`
    SnomedCt = "snomed-ct",
    /// `LOINC`
    Loinc = "loinc",
    /// `The system this clause draws its codes from.`
    SystemClauseDrawsCodesFrom = "system-clause-draws-codes-from",
    /// `Fields per row`
    FieldsPerRow = "fields-per-row",
    /// `Codes the system retired still match.`
    CodesSystemRetiredStillMatch = "codes-system-retired-still-match",
    /// `Include inactive codes`
    IncludeInactiveCodes = "include-inactive-codes",
    /// `Add`
    Add = "add",
    /// `blood pressure`
    BloodPressure = "blood-pressure",
    /// `Two letters or more.`
    TwoLettersMore = "two-letters-more",
    ///  codes
    Codes = "codes",
    /// `Include`
    Include = "include",
    /// `Exclude`
    Exclude = "exclude",
    /// `Danger, disabled`
    DangerDisabled = "danger-disabled",
    /// `Secondary, disabled`
    SecondaryDisabled = "secondary-disabled",
    /// `Primary, disabled`
    PrimaryDisabled = "primary-disabled",
    /// `Small`
    Small = "small",
    /// `Ghost`
    Ghost = "ghost",
    /// `Danger`
    Danger = "danger",
    /// `Secondary`
    Secondary = "secondary",
    /// `Primary`
    Primary = "primary",
    /// `At time`
    Time = "time",
    /// `Target path (optional)`
    TargetPathOptional = "target-path-optional",
    /// `Time travel (UTC)`
    TimeTravelUtc = "time-travel-utc",
    /// `Version`
    Version = "version",
    /// `Create`
    Create = "create",
    /// `Subject namespace (optional)`
    SubjectNamespaceOptional = "subject-namespace-optional",
    /// `Subject id (optional)`
    SubjectIdOptional = "subject-id-optional",
    /// `EHR id (optional)`
    EhrIdOptional = "ehr-id-optional",
    /// `Fields in a row`
    FieldsRow = "fields-row",
    /// `Version 1`
    Version1 = "version-1",
    /// `Latest`
    Latest = "latest",
    /// `namespace`
    Namespace = "namespace",
    /// `external subject id`
    ExternalSubjectId = "external-subject-id",
    /// `ehr_id (UUID), blank lets the server mint one`
    EhrIdUuidBlankLets = "ehr-id-uuid-blank-lets",
    /// `Save`
    Save = "save",
    /// `Save this draft and reload the set. The bridge refuses a draft that does not compile.`
    SaveDraftReloadSetBridge = "save-draft-reload-set-bridge",
    /// `Keep the draft until`
    KeepDraftUntil = "keep-draft-until",
    /// `Bundle, JSON`
    BundleJson = "bundle-json",
    /// `Composition format`
    CompositionFormat = "composition-format",
    /// `Why you save it, one line`
    WhyYouSaveOneLine = "why-you-save-one-line",
    /// `Two hundred characters at most.`
    TwoHundredCharactersMost = "two-hundred-characters-most",
    /// `Fields`
    Fields = "fields",
    /// `flat`
    Flat = "flat",
    /// `canonical`
    Canonical = "canonical",
    /// `Pick or type`
    PickType = "pick-type",
    /// `A code system this server holds, or the canonical of another.`
    CodeSystemServerHoldsCanonical = "code-system-server-holds-canonical",
    /// `Choose one this server holds`
    ChooseOneServerHolds = "choose-one-server-holds",
    /// `Another canonical`
    AnotherCanonical = "another-canonical",
    /// `Choose from the list`
    ChooseFromList = "choose-from-list",
    /// `A version this server holds, or another one you type.`
    VersionServerHoldsAnotherOne = "version-server-holds-another-one",
    /// `Another version`
    AnotherVersion = "another-version",
    /// `The one this server resolves to`
    OneServerResolves = "one-server-resolves",
    /// `Ten values at most.`
    TenValuesMost = "ten-values-most",
    /// `Add value`
    AddValue = "add-value",
    /// `Allowed audiences`
    AllowedAudiences = "allowed-audiences",
    /// `Field shapes`
    FieldShapes = "field-shapes",
    /// `The address must be absolute, starting https://.`
    AddressMustAbsoluteStartingHttps = "address-must-absolute-starting-https",
    /// `Where the gateway reaches the member.`
    WhereGatewayReachesMember = "where-gateway-reaches-member",
    /// `Run the query`
    RunQuery = "run-query",
    /// `When a node does not answer, the rows of the others still show.`
    WhenNodeDoesNotAnswer = "when-node-does-not-answer",
    /// `Accept a partial answer`
    AcceptPartialAnswer = "accept-partial-answer",
    /// `Ask these nodes`
    AskTheseNodes = "ask-these-nodes",
    /// `A federated query`
    FederatedQuery = "federated-query",
    /// `Main and aside`
    MainAside = "main-aside",
    /// `Three panes and the tree, a list beside its item, a form beside its answer, rows of...`
    ThreePanesTreeListBeside = "three-panes-tree-list-beside",
    /// `Section`
    Section = "section",
    /// `Loose`
    Loose = "loose",
    /// `Default`
    Default = "default",
    /// `Tight`
    Tight = "tight",
    /// `Stacks`
    Stacks = "stacks",
    /// `A third line.`
    ThirdLine = "third-line",
    /// `A second line.`
    SecondLine = "second-line",
    /// `System`
    System = "system",
    /// `Definition`
    Definition = "definition",
    /// `A finding invented for the gallery.`
    FindingInventedGallery = "finding-invented-gallery",
    /// `Display`
    Display = "display",
    /// `Synthetic finding of the leg`
    SyntheticFindingLeg = "synthetic-finding-leg",
    /// `Synthetic finding of the arm`
    SyntheticFindingArm = "synthetic-finding-arm",
    /// `Rows of fields`
    RowsFields = "rows-fields",
    /// `Up`
    Up = "up",
    /// `30 s`
    N30S = "n-30-s",
    /// `timed out`
    TimedOut = "timed-out",
    /// `12 ms`
    N12Ms = "n-12-ms",
    /// `The nodes that answered`
    NodesAnswered = "nodes-answered",
    /// `Nodes`
    Nodes = "nodes",
    /// `No time spent, 0 percent`
    NoTimeSpent0Percent = "no-time-spent-0-percent",
    /// `Latency per operation`
    LatencyPerOperation = "latency-per-operation",
    /// `Latency`
    Latency = "latency",
    /// `31 ms`
    N31Ms = "n-31-ms",
    /// `96 ms`
    N96Ms = "n-96-ms",
    /// `List`
    List = "list",
    /// `Include inactive concepts`
    IncludeInactiveConcepts = "include-inactive-concepts",
    /// `An inactive concept is listed and marked.`
    InactiveConceptListedMarked = "inactive-concept-listed-marked",
    /// `Why you run it`
    WhyYouRun = "why-you-run",
    /// `A note kept beside the run.`
    NoteKeptBesideRun = "note-kept-beside-run",
    /// `Display language`
    DisplayLanguage = "display-language",
    /// `The language of the displays.`
    LanguageDisplays = "language-displays",
    /// `Page size`
    PageSize = "page-size",
    /// `How many concepts one page lists, 1 to 1000.`
    HowManyConceptsOnePage = "how-many-concepts-one-page",
    /// `List a value set`
    ListValueSet = "list-value-set",
    /// `Dutch`
    Dutch = "dutch",
    /// `English`
    English = "english",
    /// `The canonical of the value set to list.`
    CanonicalValueSetList = "canonical-value-set-list",
    /// `Choose a value set this server holds`
    ChooseValueSetServerHolds = "choose-value-set-server-holds",
    /// `Value set`
    ValueSet = "value-set",
    /// `Synthetic units`
    SyntheticUnits = "synthetic-units",
    /// `Synthetic sites`
    SyntheticSites = "synthetic-sites",
    /// `Synthetic signs`
    SyntheticSigns = "synthetic-signs",
    /// `Excludes the inactive concepts.`
    ExcludesInactiveConcepts = "excludes-inactive-concepts",
    /// `Composed of two synthetic code systems.`
    ComposedTwoSyntheticCodeSystems = "composed-two-synthetic-code-systems",
    /// `Concepts`
    Concepts = "concepts",
    /// `Name`
    Name = "name",
    /// `Value sets`
    ValueSets = "value-sets",
    /// `Master and detail`
    MasterDetail = "master-detail",
    /// `Concept`
    Concept = "concept",
    /// `Hierarchy`
    Hierarchy = "hierarchy",
    /// `Three panes and the tree`
    ThreePanesTree = "three-panes-tree",
    /// `Parent`
    Parent = "parent",
    /// `Matches`
    Matches = "matches",
    /// `Find a concept`
    FindConcept = "find-concept",
    /// `Synthetic hierarchy`
    SyntheticHierarchy = "synthetic-hierarchy",
    /// `Another synthetic child`
    AnotherSyntheticChild = "another-synthetic-child",
    /// `Synthetic child, read on opening`
    SyntheticChildReadOpening = "synthetic-child-read-opening",
    /// `Synthetic body site`
    SyntheticBodySite = "synthetic-body-site",
    /// `Synthetic procedure`
    SyntheticProcedure = "synthetic-procedure",
    /// `Synthetic finding`
    SyntheticFinding = "synthetic-finding",
    /// `Overlays and feedback`
    OverlaysFeedback = "overlays-feedback",
    /// `Dialogs, modals, the side panel and toasts.`
    DialogsModalsSidePanelToasts = "dialogs-modals-side-panel-toasts",
    /// `Toasts`
    Toasts = "toasts",
    /// `Refusal`
    Refusal = "refusal",
    /// `The save was refused: 412.`
    SaveWasRefused412 = "save-was-refused-412",
    /// `Warning`
    Warning = "warning",
    /// `The set compiled with 2 warnings.`
    SetCompiled2Warnings = "set-compiled-2-warnings",
    /// `Information`
    Information = "information",
    /// `The set reloads in the background.`
    SetReloadsBackground = "set-reloads-background",
    /// `Success`
    Success = "success",
    /// `Saved, and the set reloaded.`
    SavedSetReloaded = "saved-set-reloaded",
    /// `Access`
    Access = "access",
    /// `View access`
    ViewAccess = "view-access",
    /// `Content that is no navigation, from the end of the page.`
    ContentNoNavigationFromEnd = "content-no-navigation-from-end",
    /// `Side panel`
    SidePanel = "side-panel",
    /// `Scopes`
    Scopes = "scopes",
    /// `Policy`
    Policy = "policy",
    /// `The operator scope decides what the console may do.`
    OperatorScopeDecidesWhatConsole = "operator-scope-decides-what-console",
    /// `Method`
    Method = "method",
    /// `OpenID Connect`
    OpenidConnect = "openid-connect",
    /// `Signed in as`
    Signed = "signed",
    /// `Synthetic operator`
    SyntheticOperator = "synthetic-operator",
    /// `Rename`
    Rename = "rename",
    /// `A path below fhirconnect/ or omocl/.`
    PathBelowFhirconnectOmocl = "path-below-fhirconnect-omocl",
    /// `New name`
    NewName = "new-name",
    /// `Rename the file`
    RenameFile = "rename-file",
    /// `Rename a file`
    RenameTheFile = "rename-the-file",
    /// `Modal dialogs`
    ModalDialogs = "modal-dialogs",
    /// `Delete`
    Delete = "delete",
    /// `The file goes for good, and the running set reloads without it.`
    FileGoesGoodRunningSet = "file-goes-good-running-set",
    /// `the file`
    TheFile = "the-file",
    /// `Delete the file`
    DeleteFile = "delete-file",
    /// `Reload the set`
    ReloadSet = "reload-set",
    /// `Opened by their trigger with no script; Escape and Cancel close them and the focus...`
    OpenedTheirTriggerNoScript = "opened-their-trigger-no-script",
    /// `Confirmation dialogs`
    ConfirmationDialogs = "confirmation-dialogs",
    /// `The synthetic file was deleted.`
    SyntheticFileWasDeleted = "synthetic-file-was-deleted",
    /// `The synthetic file goes for good; the running set reloads without it.`
    SyntheticFileGoesGoodRunning = "synthetic-file-goes-good-running",
    /// `Delete the file?`
    DeleteTheFile = "delete-the-file",
    /// `Reload`
    Reload = "reload",
    /// `The set reloaded.`
    SetReloaded = "set-reloaded",
    /// `The files on disk compile and replace the running set when they compile.`
    FilesDiskCompileReplaceRunning = "files-disk-compile-replace-running",
    /// `Reload the set?`
    ReloadTheSet = "reload-the-set",
    /// `New file`
    NewFile = "new-file",
    /// `Every static view of the library, in this brand's theme.`
    EveryStaticViewLibraryBrand = "every-static-view-library-brand",
    /// `Icons`
    Icons = "icons",
    /// `The pseudo-locale draws every library string accented and longer, to check the layout.`
    PseudoLocaleDrawsEveryLibrary = "pseudo-locale-draws-every-library",
    /// `Language`
    Language = "language",
    /// `Density`
    Density = "density",
    /// `Theme`
    Theme = "theme",
    /// `Settings`
    Settings = "settings",
    /// `Loading the files.`
    LoadingFiles = "loading-files",
    /// `Signing in`
    Signing = "signing",
    /// `Sign in`
    SignIn = "sign-in",
    /// `Asking the bridge…`
    AskingBridge = "asking-bridge",
    /// `Reading the capability statement`
    ReadingCapabilityStatement = "reading-capability-statement",
    /// `Loading`
    Loading = "loading",
    /// `No match`
    NoMatch = "no-match",
    /// `No file matches the filter.`
    NoFileMatchesFilter = "no-file-matches-filter",
    /// `A quiet empty section`
    QuietEmptySection = "quiet-empty-section",
    /// `The bridge serves no mapping file yet.`
    BridgeServesNoMappingFile = "bridge-serves-no-mapping-file",
    /// `No mapping file`
    NoMappingFile = "no-mapping-file",
    /// `Empty state`
    EmptyState = "empty-state",
    /// `The console cannot reach the bridge.`
    ConsoleCannotReachBridge = "console-cannot-reach-bridge",
    /// `Refusal and fault`
    RefusalFault = "refusal-fault",
    /// `the CDR did not answer within 30 seconds.`
    CdrDidNotAnswerWithin = "cdr-did-not-answer-within",
    /// `GET https://term.example.org/fhir/ValueSet/$expand?url=http://example.org/vs`
    GetHttpsTermExampleOrg = "get-https-term-example-org",
    /// `the terminology server answered with no readable outcome.`
    TerminologyServerAnsweredNoReadable = "terminology-server-answered-no-readable",
    /// `POST https://bridge.example.org/fhir/Bundle?_format=json`
    PostHttpsBridgeExampleOrg = "post-https-bridge-example-org",
    /// `MSG_UNKNOWN_TYPE`
    UnknownTypeCode = "msg-unknown-type",
    /// `Bundle.entry[0].resource`
    BundleEntry0Resource = "bundle-entry-0-resource",
    /// `No program maps the template gallery.template.v1.`
    NoProgramMapsTemplateGallery = "no-program-maps-template-gallery",
    /// `the bridge refused the request.`
    BridgeRefusedRequest = "bridge-refused-request",
    /// `Segmented controls`
    SegmentedControls = "segmented-controls",
    /// `R4`
    R4 = "r4",
    /// `R4B`
    R4b = "r4b",
    /// `R5`
    R5 = "r5",
    /// `R6`
    R6 = "r6",
    /// `FHIR`
    Fhir = "fhir",
    /// `FHIR version`
    FhirVersion = "fhir-version",
    /// `Full`
    Full = "full",
    /// `Minimal`
    Minimal = "minimal",
    /// `Detail`
    Detail = "detail",
    /// `Format`
    Format = "format",
    /// `Keyboard tabs: Left, Right, Home and End move between them.`
    KeyboardTabsLeftRightHome = "keyboard-tabs-left-right-home",
    /// `Tabs`
    Tabs = "tabs",
    /// `What the console asked.`
    WhatConsoleAsked = "what-console-asked",
    /// `Request`
    Request = "request",
    /// `The answer as the server sent it.`
    AnswerServerSent = "answer-server-sent",
    /// `Raw`
    Raw = "raw",
    /// `The rows of the answer, as a table.`
    RowsAnswerTable = "rows-answer-table",
    /// `Table`
    Table = "table",
    /// `Answer`
    Answer = "answer",
    /// `Tags`
    Tags = "tags",
    /// `Commit`
    Commit = "commit",
    /// `Status history`
    StatusHistory = "status-history",
    /// `Directory`
    Directory = "directory",
    /// `Contributions`
    Contributions = "contributions",
    /// `Compositions`
    Compositions = "compositions",
    /// `Record`
    Record = "record",
    /// `History`
    History = "history",
    /// `Diagnostics`
    Diagnostics = "diagnostics",
    /// `Source`
    Source = "source",
    /// `Views of a file`
    ViewsFile = "views-file",
    /// `Three changes wait to be applied to the running set.`
    ThreeChangesWaitAppliedRunning = "three-changes-wait-applied-running",
    /// `Staged changes`
    StagedChanges = "staged-changes",
    /// `A well sets a passage apart inside a card, such as a note on what the figures mean.`
    WellSetsPassageApartInside = "well-sets-passage-apart-inside",
    /// `The raw document, folded until it is asked for.`
    RawDocumentFoldedUntilAsked = "raw-document-folded-until-asked",
    /// `What the bridge answered, as JSON`
    WhatBridgeAnsweredJson = "what-bridge-answered-json",
    /// `Disclosure and well`
    DisclosureWell = "disclosure-well",
    /// `$tofhir http://fhirconnect.org/fhir/OperationDefinition/ToFhirSynthetic`
    TofhirHttpFhirconnectOrgFhir = "tofhir-http-fhirconnect-org-fhir",
    /// `$validate-code on the value sets the server loaded`
    ValidateCodeValueSetsServer = "validate-code-value-sets-server",
    /// `A bullet list`
    BulletList = "bullet-list",
    /// `A card titled with a file's path.`
    CardTitledFileSPath = "card-titled-file-s-path",
    /// `Run $expand over a value set this server holds.`
    RunExpandOverValueSet = "run-expand-over-value-set",
    /// `Expand a value set`
    ExpandValueSet = "expand-value-set",
    /// `self, with no external subject reference, so the record names no person outside the server`
    SelfNoExternalSubjectReference = "self-no-external-subject-reference",
    /// `Subject`
    Subject = "subject",
    /// `Version uid`
    VersionUid = "version-uid",
    /// `wasm32-unknown-unknown`
    Wasm32UnknownUnknown = "wasm32-unknown-unknown",
    /// `Target`
    Target = "target",
    /// `Locale`
    Locale = "locale",
    /// `Built`
    Built = "built",
    /// `Licence`
    Licence = "licence",
    /// `Business Source License 1.1`
    BusinessSourceLicense11 = "business-source-license-1-1",
    /// `Profile`
    Profile = "profile",
    /// `Product`
    ProductColumn = "product-column",
    /// `Definition grid`
    DefinitionGrid = "definition-grid",
    /// `Uid`
    Uid = "uid",
    /// `Last changed`
    LastChanged = "last-changed",
    /// `Languages`
    Languages = "languages",
    /// `Default language`
    DefaultLanguage = "default-language",
    /// `Synthetic evaluation`
    SyntheticEvaluation = "synthetic-evaluation",
    /// `Facts across a row`
    FactsAcrossRow = "facts-across-row",
    /// `The set compiled and replaced the running one.`
    SetCompiledReplacedRunningOne = "set-compiled-replaced-running-one",
    /// `SMART is not enabled on this synthetic server.`
    SmartNotEnabledSyntheticServer = "smart-not-enabled-synthetic-server",
    /// `SMART configuration returned, but it is not valid JSON.`
    SmartConfigurationReturnedButNot = "smart-configuration-returned-but-not",
    /// `Notices`
    Notices = "notices",
    /// `Back to the files`
    BackFiles = "back-files",
    /// `This synthetic version is deleted.`
    SyntheticVersionDeleted = "synthetic-version-deleted",
    /// `Down`
    Down = "down",
    /// `Degraded`
    Degraded = "degraded",
    /// `1 of 3 lanes down`
    N13LanesDown = "n-1-3-lanes-down",
    /// `Serving`
    Serving = "serving",
    /// `Disabled`
    Disabled = "disabled",
    /// `Buttons`
    Buttons = "buttons",
    /// `A link as a button`
    LinkButton = "link-button",
    /// `revision 3 pending`
    Revision3Pending = "revision-3-pending",
    /// `Running revision on every federation node`
    RunningRevisionEveryFederationNode = "running-revision-every-federation-node",
    /// `1 failing`
    N1Failing = "n-1-failing",
    /// `Members`
    Members = "members",
    /// `With a warning`
    WithAWarning = "with-a-warning",
    /// `Compiled`
    Compiled = "compiled",
    /// `Mapping files`
    MappingFiles = "mapping-files",
    /// `STRUCTURED`
    Structured = "structured",
    /// `FLAT`
    FlatUpper = "flat-upper",
    /// `API reference`
    ApiReference = "api-reference",
    /// `Documentation`
    Documentation = "documentation",
    /// `Console`
    Console = "console",
    /// `Wrong username or password.`
    WrongUsernamePassword = "wrong-username-password",
    /// `Could not read the server's sign-in methods, so both are offered.`
    CouldNotReadServerS = "could-not-read-server-s",
    /// `Your session ended. Sign in again to continue.`
    YourSessionEndedSignAgain = "your-session-ended-sign-again",
    /// `Sign in with OIDC`
    SignOidc = "sign-oidc",
    /// `One table and pager model: in the URL, or in the page for a screen that can show...`
    OneTablePagerModelUrl = "one-table-pager-model-url",
    /// `The staged configuration against the running one`
    StagedConfigurationAgainstRunningOne = "staged-configuration-against-running-one",
    /// `Apply`
    Apply = "apply",
    /// `Loading and empty`
    LoadingEmpty = "loading-empty",
    /// `No rows`
    NoRows = "no-rows",
    /// `No stated total`
    NoStatedTotal = "no-stated-total",
    /// `Pages of the EHRs, in the page`
    PagesEhrsPage = "pages-ehrs-page",
    /// `Pages of the EHRs, as links`
    PagesEhrsLinks = "pages-ehrs-links",
    /// `Members, each with an Edit link and a Remove form`
    MembersEachEditLinkRemove = "members-each-edit-link-remove",
    /// `Row actions`
    RowActions = "row-actions",
    /// `Edit`
    Edit = "edit",
    /// `Actions`
    Actions = "actions",
    /// `Health indicators, unstacked at every width`
    HealthIndicatorsUnstackedEveryWidth = "health-indicators-unstacked-every-width",
    /// `Fits a phone as it is`
    FitsPhone = "fits-phone",
    /// `Indicator`
    Indicator = "indicator",
    /// `The synthetic query's rows, a body per page`
    SyntheticQuerySRowsBody = "synthetic-query-s-rows-body",
    /// `A result in pages`
    ResultPages = "result-pages",
    /// `Rate`
    Rate = "rate",
    /// `Committed`
    Committed = "committed",
    /// `Composition`
    Composition = "composition",
    /// `The rows the synthetic query answered`
    RowsSyntheticQueryAnswered = "rows-synthetic-query-answered",
    /// `A result set`
    ResultSet = "result-set",
    /// `count`
    Count = "count",
    /// `name`
    NameLower = "name-lower",
    /// `Synthetic profile`
    SyntheticProfile = "synthetic-profile",
    /// `http://example.org/fhir/StructureDefinition/ferrohealth-synthetic-journey-diagnose`
    HttpExampleOrgFhirStructuredefinition = "http-example-org-fhir-structuredefinition",
    /// `Synthetic vital signs`
    SyntheticVitalSigns = "synthetic-vital-signs",
    /// `Paged in the page`
    PagedPage = "paged-page",
    /// `Pages of the mapping files, in the page`
    PagesMappingFilesPage = "pages-mapping-files-page",
    /// `Mapping files, paged in the page`
    MappingFilesPagedPage = "mapping-files-paged-page",
    /// `Paged in the URL`
    PagedUrl = "paged-url",
    /// `Pages of the mapping files`
    PagesMappingFiles = "pages-mapping-files",
    /// `Mapping files, paged in the URL`
    MappingFilesPagedUrl = "mapping-files-paged-url",
    /// `Open`
    Open = "open",
    /// `State`
    State = "state",
    /// `Changed`
    Changed = "changed",
    /// `Lines`
    Lines = "lines",
    /// `Kind`
    Kind = "kind",
    /// `File`
    FileColumn = "file-column",
    /// `The noun of the mapping files, by count, with its empty and opening forms.`
    NounMappingFiles = "noun-mapping-files",
    /// `The noun of the EHRs, by count, with its empty and opening forms.`
    NounEhrs = "noun-ehrs",
    /// The combobox label of the clause at `$place`.
    ClauseCodeSystem = "clause-code-system",
    /// The filter's count: `$kept` of the `$total` files kept.
    FilesOnPage = "files-on-page",
    /// The link to the line `$line` an issue points at.
    IssueLine = "issue-line",
    /// The editor's size: `$bytes` bytes in `$lines` lines.
    SizeLines = "size-lines",
    /// The label of the synthetic setting at `$index`.
    SyntheticSetting = "synthetic-setting",
    /// The name of the button that removes the value `$value`.
    RemoveValue = "remove-value",
    /// The label of the base URL of the member `$member`.
    BaseUrlOf = "base-url-of",
    /// The toast once the row `$row` is deleted.
    RowDeleted = "row-deleted",
    /// The question of the dialog that deletes the row `$row`.
    DeleteRow = "delete-row",
    /// The line naming the format `$format` the document is shown in.
    ShownAs = "shown-as",
    /// A refusal notice with its issues, a line each.
    SaveRefusedIssues = "save-refused-issues",
    /// The sign-in card's notice of the synthetic deployment.
    SyntheticDeployment = "synthetic-deployment",
    /// The reload count: read `$count` times again.
    ReadTimesAgain = "read-times-again",
    /// The heading of the panel that previews the files.
    RecentFiles = "recent-files",
    /// The panel's header action that leads to every file.
    AllFiles = "all-files",
    /// The heading of the card of the two editor headers.
    EditorHeaders = "editor-headers",
    /// The title of the file editor: `$mode` new or edit, `$name` the file changed.
    FileEditor = "file-editor",
    /// The crumb of the files listing above the file editor.
    MappingFilesCrumb = "mapping-files-crumb",
    /// `A hierarchy in a table`
    HierarchyInTable = "hierarchy-in-table",
    /// `The concepts of a synthetic hierarchy, each a level under its parent`
    ConceptsSyntheticHierarchy = "concepts-synthetic-hierarchy",
    /// `Root of the hierarchy`
    HierarchyRoot = "hierarchy-root",
    /// `A branch`
    HierarchyBranchA = "hierarchy-branch-a",
    /// `A leaf of branch A`
    HierarchyLeafA1 = "hierarchy-leaf-a1",
    /// `Another leaf of branch A`
    HierarchyLeafA2 = "hierarchy-leaf-a2",
    /// `B branch`
    HierarchyBranchB = "hierarchy-branch-b",
    /// `A twig of branch B`
    HierarchyTwigB1 = "hierarchy-twig-b1",
    /// `A leaf of that twig`
    HierarchyLeafB1x = "hierarchy-leaf-b1x",
    /// `Nine levels down, at the deepest step`
    HierarchyDeep = "hierarchy-deep",
}

/// The gallery's message `msg` in the page's locale.
#[must_use]
pub fn tr(msg: Msg) -> String {
    locale().lookup(&CATALOG, msg.id())
}

/// The gallery's message `msg` in the page's locale, with the placeables
/// `args` filled in.
#[must_use]
pub fn tr_with(msg: Msg, args: &[(&'static str, Arg)]) -> String {
    locale().lookup_with(&CATALOG, msg.id(), args)
}

/// The gallery's message `msg` in `locale`, with the placeables `args`
/// filled in: for a handler or a task, where no page tree is in reach.
#[must_use]
pub fn tr_in(locale: &Locale, msg: Msg, args: &[(&'static str, Arg)]) -> String {
    locale.lookup_with(&CATALOG, msg.id(), args)
}

/// The gallery's message `msg` as a [`Message`] a component looks up as it
/// draws, such as a listing's noun.
#[must_use]
pub const fn message(msg: Msg) -> Message {
    Message::new(&CATALOG, msg.id())
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::path::Path;

    use fluent_templates::Loader;
    use fluent_templates::fluent_bundle::FluentValue;

    use super::{GALLERY, Msg, PSEUDO_TAG, gallery_locale, stretch};

    #[test]
    fn the_catalog_holds_every_message_and_no_other() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("locales/en/ferrohealth-ui-gallery.ftl");
        let source = std::fs::read_to_string(path).expect("the catalog");
        let mut held: Vec<&str> = source
            .lines()
            .filter_map(|line| line.split_once(" ="))
            .map(|(id, _)| id)
            .filter(|id| id.starts_with(|c: char| c.is_ascii_lowercase()) && !id.contains(' '))
            .collect();
        held.sort_unstable();
        let mut declared: Vec<&str> = Msg::ALL.iter().map(|msg| msg.id()).collect();
        declared.sort_unstable();
        assert_eq!(
            declared, held,
            "every message is declared once and held once"
        );
        let english = fluent_templates::langid!("en");
        // Every placeable a message names, so each formats whole.
        let args: HashMap<Cow<'static, str>, FluentValue<'static>> = [
            "bytes", "count", "format", "index", "kept", "line", "lines", "member", "mode", "name",
            "place", "row", "total", "value",
        ]
        .into_iter()
        .map(|name| (Cow::Borrowed(name), FluentValue::from(3)))
        .collect();
        for msg in Msg::ALL {
            assert!(
                GALLERY
                    .try_lookup_with_args(&english, msg.id(), &args)
                    .is_some_and(|words| !words.trim().is_empty()),
                "en holds {}",
                msg.id()
            );
        }
    }

    #[test]
    fn the_pseudo_locale_stretches_the_gallerys_own_words_too() {
        let pseudo = gallery_locale(PSEUDO_TAG);
        assert_eq!(
            "[Övérvíéw ~~~~]",
            pseudo.lookup(&super::CATALOG, Msg::Overview.id())
        );
        let english = gallery_locale("en");
        assert_eq!(
            "Overview",
            english.lookup(&super::CATALOG, Msg::Overview.id())
        );
    }

    #[test]
    fn the_pseudo_locale_accents_and_lengthens_every_string() {
        assert_eq!("[Çáñçél ~~~]", stretch("Cancel"));
        assert_eq!("[Págé 1 öf 2 ~~~~~]", stretch("Page 1 of 2"));
        assert_eq!(PSEUDO_TAG, gallery_locale(PSEUDO_TAG).tag());
        assert_eq!(
            "en",
            gallery_locale("sepia").tag(),
            "an unknown tag is English"
        );
    }
}
