// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The locale a page renders in, and the catalogs of the strings the
//! components draw themselves.
//!
//! Every word a component draws on its own (a pager step, a dialog's
//! cancel button, the theme toggle's name, an empty table's range) is a
//! message of a Fluent catalog under the crate's `locales/` directory, one
//! directory per language (`locales/en/ferrohealth-ui.ftl`). The catalogs
//! are embedded at build time by `fluent_templates::static_loader!`, which
//! reads the directory, so a language joins as one catalog file and no code
//! changes. English (`en`) is the reference catalog: complete, and the one
//! every lookup falls back to.
//!
//! A console chooses the locale once per render: it builds a [`Locale`] from
//! its configuration or the request and hands it to [`provide_locale`] at
//! the root of its page tree on the server, and writes [`Locale::tag`] as
//! `<html lang>` (WCAG 2.2 SC 3.1.1). A component reads it with [`locale`],
//! and a page that provides none renders in English. An island never looks a
//! word up: the server resolves each word it draws, through a widget's
//! `*Words::of_page()` or a [`Phrase`] for a sentence around a value the
//! browser learns, and hands them over as props, so the browser carries no
//! catalog. A console catalogs its own page strings the same
//! way: its own `static_loader!` over its own `locales/`, looked up through
//! [`Locale::lookup`] so its words follow the same choice.
//!
//! No specification governs the catalog layout or the lookup order: our own
//! design. The message syntax is Project Fluent's
//! (<https://projectfluent.org/fluent/guide/>).

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader, StaticLoader};
use leptos::prelude::*;

fluent_templates::static_loader! {
    static CATALOG = {
        locales: "./locales",
        fallback_language: "en",
        // NOTE: Fluent's guide, "Bidirectionality": the isolation marks around a
        // placeable are for mixed-direction text; the English catalog draws none.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// The library's catalogs, every language it ships, so a catalog of a
/// console's can draw from them (a pseudo-locale over the English one).
#[must_use]
pub fn library_catalog() -> &'static StaticLoader {
    &CATALOG
}

/// The language every lookup falls back to, and the one a page that
/// provides no locale renders in.
fn english() -> LanguageIdentifier {
    fluent_templates::langid!("en")
}

/// A value a message's placeable takes: a word or a number, which a
/// catalog may select a plural form by.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    /// Text, written as it stands.
    Text(String),
    /// A number, written without grouping and selectable by plural rule.
    Number(f64),
}

impl From<&str> for Arg {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl From<String> for Arg {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<u32> for Arg {
    fn from(number: u32) -> Self {
        Self::Number(f64::from(number))
    }
}

impl From<i32> for Arg {
    fn from(number: i32) -> Self {
        Self::Number(f64::from(number))
    }
}

impl From<usize> for Arg {
    fn from(number: usize) -> Self {
        // NOTE: no specification governs this: our own design; a count past u32 is
        // written as text, which a plural selector reads as its default form.
        u32::try_from(number).map_or_else(|_| Self::Text(number.to_string()), Self::from)
    }
}

impl From<u64> for Arg {
    fn from(number: u64) -> Self {
        u32::try_from(number).map_or_else(|_| Self::Text(number.to_string()), Self::from)
    }
}

impl From<f64> for Arg {
    fn from(number: f64) -> Self {
        Self::Number(number)
    }
}

impl Arg {
    /// The value as Fluent formats it.
    fn fluent(&self) -> FluentValue<'static> {
        match self {
            Self::Text(text) => FluentValue::from(text.clone()),
            Self::Number(number) => FluentValue::from(*number),
        }
    }
}

/// The arguments of a lookup as the Fluent loader takes them.
fn fluent_args(args: &[(&'static str, Arg)]) -> HashMap<Cow<'static, str>, FluentValue<'static>> {
    args.iter()
        .map(|(name, value)| (Cow::Borrowed(*name), value.fluent()))
        .collect()
}

/// The error a tag that is no language identifier parses to.
type TagError = <LanguageIdentifier as std::str::FromStr>::Err;

/// A tag that names no language (BCP 47).
#[derive(Debug)]
pub struct UnknownLanguage {
    tag: String,
    source: TagError,
}

impl fmt::Display for UnknownLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the tag {:?} is no language identifier", self.tag)
    }
}

impl std::error::Error for UnknownLanguage {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// The locale a page renders in: a language, the library catalog that
/// serves it, and the console's catalogs for a language the library does
/// not ship.
#[derive(Clone)]
pub struct Locale {
    language: LanguageIdentifier,
    served: Option<LanguageIdentifier>,
    supplement: Option<&'static (dyn Loader + Send + Sync)>,
}

impl fmt::Debug for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Locale")
            .field("language", &self.language.to_string())
            .field("served", &self.served.as_ref().map(ToString::to_string))
            .field("supplement", &self.supplement.is_some())
            .finish()
    }
}

impl PartialEq for Locale {
    fn eq(&self, other: &Self) -> bool {
        self.language == other.language
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::of(english())
    }
}

impl Locale {
    /// English, the reference locale.
    #[must_use]
    pub fn english() -> Self {
        Self::default()
    }

    /// The locale of the language `language`, served by the library's
    /// catalog that matches it best: its own, or the catalog of its bare
    /// language (`en` serves `en-GB`).
    fn of(language: LanguageIdentifier) -> Self {
        let mut shipped: Vec<&LanguageIdentifier> = CATALOG.locales().collect();
        shipped.sort_by_key(ToString::to_string);
        let served = shipped
            .iter()
            .find(|each| **each == &language)
            .or_else(|| {
                shipped
                    .iter()
                    .find(|each| each.matches(&language, true, false))
            })
            .map(|each| (*each).clone());
        Self {
            language,
            served,
            supplement: None,
        }
    }

    /// The locale the BCP 47 tag `tag` names (`en`, `en-GB`).
    ///
    /// # Errors
    ///
    /// [`UnknownLanguage`] when `tag` is no language identifier.
    pub fn parse(tag: &str) -> Result<Self, UnknownLanguage> {
        tag.parse::<LanguageIdentifier>()
            .map(Self::of)
            .map_err(|source| UnknownLanguage {
                tag: tag.to_owned(),
                source,
            })
    }

    /// The best locale for the `Accept-Language` header value `header`
    /// among the languages the library ships, English when none matches
    /// (RFC 9110 §12.5.4).
    #[must_use]
    pub fn negotiate(header: &str) -> Self {
        Self::ranked(header)
            .into_iter()
            .map(Self::of)
            .find(|locale| locale.served.is_some())
            .unwrap_or_default()
    }

    /// The best locale for the `Accept-Language` header value `header`
    /// among the languages the console's `catalog` holds, and `fallback`,
    /// the console's configured locale, when none matches.
    ///
    /// The console's catalog names the languages the console ships; the
    /// library draws its own words in the same language, or in English
    /// where it has no catalog for it.
    #[must_use]
    pub fn negotiate_with(header: &str, catalog: &dyn Loader, fallback: Self) -> Self {
        let held: Vec<&LanguageIdentifier> = catalog.locales().collect();
        Self::ranked(header)
            .into_iter()
            .find(|language| {
                held.iter()
                    .any(|each| *each == language || each.matches(language, true, false))
            })
            .map_or(fallback, Self::of)
    }

    /// The language ranges of the `Accept-Language` value `header` that are
    /// language identifiers with a weight above zero, best first and in
    /// their order within a weight (RFC 9110 §12.5.4).
    fn ranked(header: &str) -> Vec<LanguageIdentifier> {
        let mut ranked: Vec<(u16, usize, LanguageIdentifier)> = header
            .split(',')
            .enumerate()
            .filter_map(|(order, range)| {
                let mut parts = range.split(';');
                let tag = parts.next()?.trim();
                let weight = parts
                    .find_map(|part| part.trim().strip_prefix("q="))
                    .map_or(Some(1000), quality)?;
                // NOTE: RFC 9110 §12.5.4: a range that is no language identifier,
                // or the wildcard, names no catalog to choose and is passed over.
                let language = tag.parse::<LanguageIdentifier>().ok()?;
                (weight > 0).then_some((weight, order, language))
            })
            .collect();
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        ranked
            .into_iter()
            .map(|(_, _, language)| language)
            .collect()
    }

    /// This locale, with the console's `catalog` consulted for a message
    /// the library has no catalog for in this language, before English.
    ///
    /// A language reaches the consoles as a library catalog first; this is
    /// for a catalog the library does not ship, such as a pseudo-locale the
    /// gallery draws to check the layout, or a test's throwaway language.
    #[must_use]
    pub fn with_catalog(mut self, catalog: &'static (dyn Loader + Send + Sync)) -> Self {
        self.supplement = Some(catalog);
        self
    }

    /// The BCP 47 tag of the locale, as `<html lang>` carries it.
    #[must_use]
    pub fn tag(&self) -> String {
        self.language.to_string()
    }

    /// The languages the library ships a catalog for, sorted by tag.
    #[must_use]
    pub fn shipped() -> Vec<String> {
        let mut tags: Vec<String> = CATALOG.locales().map(ToString::to_string).collect();
        tags.sort();
        tags
    }

    /// The message `id` of the console's `catalog` in this locale, falling
    /// back as the catalog's own fallback language says, and the id itself
    /// when no catalog holds it, which the console's tests keep from
    /// happening.
    #[must_use]
    pub fn lookup(&self, catalog: &dyn Loader, id: &str) -> String {
        self.lookup_with(catalog, id, &[])
    }

    /// [`Locale::lookup`] with the placeables `args` filled in.
    #[must_use]
    pub fn lookup_with(
        &self,
        catalog: &dyn Loader,
        id: &str,
        args: &[(&'static str, Arg)],
    ) -> String {
        let args = fluent_args(args);
        catalog
            .try_lookup_complete(&self.language, id, Some(&args))
            .unwrap_or_else(|| missing(id))
    }

    /// The library's message `id` in this locale, with `args` filled in.
    // NOTE: the Rust reference, `inline` attribute: one copy serves every component,
    // so the lookup is not repeated in each view that draws a word.
    #[inline(never)]
    fn library(&self, id: &str, args: &[(&'static str, Arg)]) -> String {
        let args = fluent_args(args);
        // NOTE: no specification governs this: our own design; a message a catalog
        // lacks, or cannot format, is looked for in the next one, ending in English.
        self.served
            .as_ref()
            .and_then(|served| CATALOG.lookup_single_language(served, id, Some(&args)).ok())
            .or_else(|| {
                self.supplement.and_then(|catalog| {
                    catalog.try_lookup_complete(&self.language, id, Some(&args))
                })
            })
            .or_else(|| {
                CATALOG
                    .lookup_single_language(&english(), id, Some(&args))
                    .ok()
            })
            .unwrap_or_else(|| missing(id))
    }
}

/// A message of a console's catalog, named by its id: words a console hands
/// a component to draw in the page's locale, such as a listing's noun or an
/// editor's title.
///
/// The component looks the message up as it draws, so the words follow the
/// locale the page provides. The catalog is the console's own
/// `static_loader!`; the message may select a form by the arguments the
/// component fills in (a plural by `$count`, a mode by `$mode`) and carry
/// attributes (`.empty`), each a whole sentence in the console's words.
#[derive(Clone, Copy)]
pub struct Message {
    catalog: &'static (dyn Loader + Send + Sync),
    id: &'static str,
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Message").field(&self.id).finish()
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Message {}

impl Message {
    /// The message `id` of the console's `catalog`.
    #[must_use]
    pub const fn new(catalog: &'static (dyn Loader + Send + Sync), id: &'static str) -> Self {
        Self { catalog, id }
    }

    /// The message's id in its catalog.
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.id
    }

    /// The message in the page's locale.
    #[must_use]
    pub fn text(self) -> String {
        locale().lookup(self.catalog, self.id)
    }

    /// The message in the page's locale, with the placeables `args` filled
    /// in.
    #[must_use]
    pub fn text_with(self, args: &[(&'static str, Arg)]) -> String {
        locale().lookup_with(self.catalog, self.id, args)
    }

    /// The message's attribute `name` (`.empty`) in the page's locale, with
    /// the placeables `args` filled in.
    #[must_use]
    pub fn attribute(self, name: &str, args: &[(&'static str, Arg)]) -> String {
        locale().lookup_with(self.catalog, &format!("{}.{name}", self.id), args)
    }
}

/// The text drawn for the message `id` no catalog holds: the id itself,
/// never nothing, with a warning in the log.
fn missing(id: &str) -> String {
    leptos::logging::warn!("no catalog holds the message {id}");
    id.to_owned()
}

/// The weight of an `Accept-Language` quality value in thousandths, or
/// `None` for one that is no quality value (RFC 9110 §12.4.2).
fn quality(value: &str) -> Option<u16> {
    let value = value.trim();
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if fraction.len() > 3 || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let thousandths = format!("{fraction:0<3}").parse::<u16>().ok()?;
    match whole {
        "0" => Some(thousandths),
        "1" if thousandths == 0 => Some(1000),
        _ => None,
    }
}

/// Makes `locale` the locale of the page tree under the calling component:
/// call it once, at the root of the page tree the server draws.
pub fn provide_locale(locale: Locale) {
    provide_context(locale);
}

/// The locale of the page being drawn: the one the page tree provides, and
/// English when there is none.
#[must_use]
pub fn locale() -> Locale {
    // NOTE: no specification governs this: our own design; a page that names no
    // locale, or a string rendered outside a page tree, draws the reference one.
    use_context::<Locale>().unwrap_or_default()
}

/// The marks around the name of a placeable a [`Phrase`] leaves open:
/// Unicode private-use characters, which no catalog writes.
const OPEN: char = '\u{E000}';
/// The mark after the name of an open placeable.
const SHUT: char = '\u{E001}';

/// Words resolved on the server with placeables left open, which an island
/// fills with values it learns only as it runs, such as a file's name.
///
/// An island takes its words as props, resolved on the server, so it carries
/// no catalog (`leptos-ui.md` §15.4). Where a sentence names a value the
/// browser learns, the server looks the message up with [`Phrase::open`] in
/// place of that value, and the island fills it with [`Phrase::fill`]. A
/// catalog never selects a form by an open placeable: a plural is chosen on
/// the server, by a count it knows.
///
/// # Examples
///
/// ```
/// use ferrohealth_ui::i18n::Phrase;
///
/// let read = Phrase::new(format!("{} read.", Phrase::open_mark("file")));
/// assert_eq!("notes.txt read.", read.fill(&[("file", "notes.txt")]));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Phrase(String);

impl Phrase {
    /// The phrase `resolved`, a message looked up with [`Phrase::open`] for
    /// each placeable left to the browser.
    #[must_use]
    pub fn new(resolved: impl Into<String>) -> Self {
        Self(resolved.into())
    }

    /// The argument that leaves the placeable `name` open in a lookup.
    #[must_use]
    pub fn open(name: &str) -> Arg {
        Arg::Text(Self::open_mark(name))
    }

    /// The mark that stands for the open placeable `name` in a phrase.
    #[must_use]
    pub fn open_mark(name: &str) -> String {
        format!("{OPEN}{name}{SHUT}")
    }

    /// The words with each open placeable named in `values` filled in.
    #[must_use]
    pub fn fill(&self, values: &[(&str, &str)]) -> String {
        values.iter().fold(self.0.clone(), |words, (name, value)| {
            words.replace(&Self::open_mark(name), value)
        })
    }
}

/// Declares the library's messages: each variant names one message of the
/// catalog by its id.
macro_rules! texts {
    ($($(#[$meta:meta])+ $name:ident = $id:literal,)+) => {
        /// A string a component draws itself, by its message in the catalog.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum Text {
            $($(#[$meta])+ $name,)+
        }

        impl Text {
            /// Every message, in declaration order.
            #[cfg(test)]
            pub(crate) const ALL: &[Self] = &[$(Self::$name,)+];

            /// The message's id in the catalog.
            pub(crate) const fn id(self) -> &'static str {
                match self {
                    $(Self::$name => $id,)+
                }
            }
        }
    };
}

texts! {
    /// The name of a catalog's language in that language.
    LanguageName = "language-name",
    /// The accessible name of the language switch.
    LanguageSwitch = "language-switch",
    /// The link that skips the chrome to the page.
    SkipToPage = "skip-to-page",
    /// The name of the button that opens the navigation drawer.
    OpenMenu = "open-menu",
    /// A button that closes the drawer or a side panel.
    Close = "close",
    /// The footer's link to the About page.
    FooterAbout = "footer-about",
    /// The user menu's sign-out item.
    SignOut = "sign-out",
    /// The user menu's link to the preferences page.
    YourPreferences = "your-preferences",
    /// The user menu's link to the keyboard shortcuts.
    KeyboardShortcuts = "keyboard-shortcuts",
    /// The sidebar's button that folds it to the rail of glyphs.
    SidebarCollapse = "sidebar-collapse",
    /// The rail's button that unfolds it to the full sidebar.
    SidebarExpand = "sidebar-expand",
    /// The search box's button.
    Find = "find",
    /// The accessible name of the breadcrumb trail.
    Breadcrumb = "breadcrumb",
    /// A definition whose value is absent.
    NoneValue = "none-value",
    /// The light theme.
    ThemeLight = "theme-light",
    /// The dark theme.
    ThemeDark = "theme-dark",
    /// The theme the operating system asks for.
    ThemeSystem = "theme-system",
    /// The accessible name of the theme switch.
    ThemeSwitch = "theme-switch",
    /// The default density.
    DensityComfortable = "density-comfortable",
    /// The compact density.
    DensityCompact = "density-compact",
    /// The accessible name of the density switch.
    DensitySwitch = "density-switch",
    /// The primary button variant, as the gallery names it.
    VariantPrimary = "variant-primary",
    /// The secondary button variant.
    VariantSecondary = "variant-secondary",
    /// The danger button variant.
    VariantDanger = "variant-danger",
    /// The destructive button variant.
    VariantDestructive = "variant-destructive",
    /// The ghost button variant.
    VariantGhost = "variant-ghost",
    /// A dialog's button that leaves without acting.
    Cancel = "cancel",
    /// The accessible name of the toast region.
    ToastRegion = "toast-region",
    /// A toast's dismiss button.
    ToastDismiss = "toast-dismiss",
    /// The badge of a tab whose section holds a draft.
    TabDraft = "tab-draft",
    /// A `$tab` holding a draft, as the phone's tab select names it.
    TabDraftChoice = "tab-draft-choice",
    /// The help switch while the help lines are hidden.
    HelpShow = "help-show",
    /// The help switch while the help lines show.
    HelpHide = "help-hide",
    /// The combobox list when the filter keeps no option.
    ComboboxNoMatch = "combobox-no-match",
    /// The title row's button that opens the More menu.
    MenuMore = "menu-more",
    /// The name of a split button's toggle: the variants of `$action`.
    MenuVariants = "menu-variants",
    /// The name of a select button: its `$label` and the `$value` chosen.
    SelectName = "select-name",
    /// The basic form's user name field.
    SignInUsername = "sign-in-username",
    /// The basic form's password field.
    SignInPassword = "sign-in-password",
    /// The basic form's button.
    SignInSubmit = "sign-in-submit",
    /// The divider between the basic form and the provider's button.
    SignInOr = "sign-in-or",
    /// The card of a console with no sign-in method.
    SignInNone = "sign-in-none",
    /// The pager's step to the first page.
    StepFirst = "step-first",
    /// The pager's step to the page before.
    StepPrevious = "step-previous",
    /// The pager's step to the page after.
    StepNext = "step-next",
    /// The pager's step to the last page.
    StepLast = "step-last",
    /// The hidden words of a step that cannot be taken.
    StepUnavailable = "step-unavailable",
    /// The pager's size choices.
    RowsPerPage = "rows-per-page",
    /// The pager's position: `$page` of `$pages`.
    PageOf = "page-of",
    /// The range of a page: `$first` to `$last` of `$total` rows.
    Range = "range",
    /// The range of an empty listing.
    RangeEmpty = "range-empty",
    /// The range naming the rows: `$noun` is the word for `$total` rows.
    RangeOf = "range-of",
    /// The range of a listing with no total: `$nouns` opens it.
    OpenRangeOf = "open-range-of",
    /// The note when the server counted `$stated` rows and the listing holds `$held`.
    PartialNote = "partial-note",
    /// The range followed by its partial note.
    RangeWithNote = "range-with-note",
    /// The library's word for `$count` rows.
    Rows = "rows",
    /// The library's word for rows, opening a sentence.
    RowsOpening = "rows-opening",
    /// The change table's setting column.
    ChangeSetting = "change-setting",
    /// The change table's value before.
    ChangeBefore = "change-before",
    /// The change table's value after.
    ChangeAfter = "change-after",
    /// The hidden word before a removed value.
    ChangeRemoved = "change-removed",
    /// The hidden word before an added value.
    ChangeAdded = "change-added",
    /// A side of a change that holds no value.
    ChangeNotSet = "change-not-set",
    /// A change table with no change.
    ChangeNone = "change-none",
    /// An issue's severity: fatal.
    SeverityFatal = "severity-fatal",
    /// An issue's severity: error.
    SeverityError = "severity-error",
    /// An issue's severity: warning.
    SeverityWarning = "severity-warning",
    /// An issue's severity: information.
    SeverityInformation = "severity-information",
    /// An issue that carries no text.
    IssueBlank = "issue-no-text",
    /// The word between an issue's code and where it points.
    IssuePlace = "issue-place",
    /// The information issues with no text that are counted, `$count` of them.
    IssuesLeftOut = "issues-left-out",
    /// A refusal's title: the upstream's `$status` and the `$summary`.
    RefusalStatus = "refusal-status",
    /// A refusal's title when no answer came.
    RefusalNoAnswer = "refusal-no-answer",
    /// The accessible name of the body a refusal keeps as evidence.
    RefusalEvidence = "refusal-evidence",
    /// An instant read in UTC: its `$date` and `$time`.
    TimestampUtc = "timestamp-utc",
    /// The legend's note for a gap in a series.
    ChartNoValue = "chart-no-value",
    /// A cell of a chart's table with no value.
    ChartNoValueCell = "chart-no-value-cell",
    /// The caption of a chart's table: `$chart` is the chart's name.
    ChartAsTable = "chart-as-table",
    /// The header of a chart table's X column.
    ChartX = "chart-x",
    /// The accessible name of a chart's legend.
    ChartSeries = "chart-series",
    /// The document pane's highlighted view.
    DocumentHighlighted = "document-highlighted",
    /// The document pane's raw view.
    DocumentRaw = "document-raw",
    /// The document pane's rendered view.
    DocumentRendered = "document-rendered",
    /// The accessible name of the format choice.
    DocumentFormat = "document-format",
    /// The accessible name of the view choice.
    DocumentView = "document-view",
    /// The document pane's copy button.
    DocumentCopy = "document-copy",
    /// The highlighted view of the `$document`.
    DocumentHighlightedRegion = "document-highlighted-region",
    /// The raw view of the `$document`.
    DocumentRawRegion = "document-raw-region",
    /// A code viewer's line count, by `$count`.
    CodeLines = "code-lines",
    /// The accessible name of a code viewer's box: `$code` is its name.
    CodeRegion = "code-region",
    /// The word before an error drawn under its line, for assistive
    /// technology.
    CodeError = "code-error",
    /// The word before a warning drawn under its line, for assistive
    /// technology.
    CodeWarning = "code-warning",
    /// The copy button once the text is on the clipboard.
    CopyCopied = "copy-copied",
    /// The announcement of a copy.
    CopySaidCopied = "copy-said-copied",
    /// The announcement of a refused copy.
    CopySaidRefused = "copy-said-refused",
    /// The words beside a refused copy.
    CopyRefused = "copy-refused",
    /// The drop zone's words.
    UploadDropInvite = "upload-drop-invite",
    /// The drop zone's words while a file is over it.
    UploadDropOver = "upload-drop-over",
    /// The kinds of file and the limit the field takes.
    UploadRule = "upload-rule",
    /// The line once the `$file` is read.
    UploadRead = "upload-read",
    /// The line while the console sends.
    UploadBusy = "upload-busy",
    /// A `$file` of a kind the field does not take.
    UploadRefusedKind = "upload-refused-kind",
    /// A `$file` of `$size` over the `$limit`.
    UploadRefusedSize = "upload-refused-size",
    /// A `$file` the browser could not read.
    UploadUnreadable = "upload-unreadable",
    /// A size in bytes.
    SizeBytes = "size-bytes",
    /// A size in kibibytes.
    SizeKib = "size-kib",
    /// A size in mebibytes.
    SizeMib = "size-mib",
    /// The words before the active filters of a list.
    FilterShowing = "filter-showing",
    /// The words before the active filters, with the rows they keep.
    FilterShowingOf = "filter-showing-of",
    /// The accessible name of the link that removes one filter.
    FilterRemove = "filter-remove",
    /// The link that removes every filter.
    FilterClear = "filter-clear",
    /// The accessible name of a setting's key hint.
    KeyCopy = "key-copy",
    /// The tooltip of a setting's key hint.
    KeyCopyTitle = "key-copy-title",
    /// A setting that takes effect at the next start.
    NextStart = "next-start",
    /// The file-mode line's words before the file.
    ConfigSetIn = "config-set-in",
    /// The file-mode line's words on when the file is read.
    ConfigReadAtStart = "config-read-at-start",
    /// The file-mode line's link to how configuration works.
    ConfigHow = "config-how",
    /// The step to the previous item of a queue, named with the item.
    StepperPrevious = "stepper-previous",
    /// The step to the next item of a queue, named with the item.
    StepperNext = "stepper-next",
    /// The About page's card of versions.
    AboutVersions = "about-versions",
    /// The About page's card of the maker.
    AboutManufacturer = "about-manufacturer",
    /// The About page's row of the console's own version.
    AboutConsole = "about-console",
    /// The About page's row of the maker.
    AboutMadeBy = "about-made-by",
    /// The About page's row of the licence.
    AboutLicence = "about-licence",
    /// The About page's row of where to ask for help.
    AboutSupport = "about-support",
    /// The About page's row of the documentation.
    AboutDocumentation = "about-documentation",
    /// The button that downloads a document.
    Download = "download",
    /// The words of a 403.
    ErrorForbidden = "error-forbidden",
    /// The words of a 404.
    ErrorNotFound = "error-not-found",
    /// The words of a 500.
    ErrorFault = "error-fault",
    /// An error's status: its words, then its number.
    ErrorStatus = "error-status",
    /// The summary of an error's technical detail.
    ErrorDetail = "error-detail",
    /// The time an error happened.
    ErrorTime = "error-time",
    /// The id of the request that failed.
    ErrorRequest = "error-request",
}

/// The library's message `text` in the page's locale.
#[inline(never)]
pub(crate) fn tr(text: Text) -> String {
    locale().library(text.id(), &[])
}

/// The library's message `text` in the page's locale, with `args` filled in.
#[inline(never)]
pub(crate) fn tr_with(text: Text, args: &[(&'static str, Arg)]) -> String {
    locale().library(text.id(), args)
}

impl Locale {
    /// The library's message `text` in this locale.
    pub(crate) fn text(&self, text: Text) -> String {
        self.library(text.id(), &[])
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Arg, CATALOG, Locale, Text, fluent_args, quality};

    /// The names of the placeables the library's messages take.
    const PLACEABLES: &[&str] = &[
        "action", "chart", "code", "count", "date", "document", "file", "filter", "first", "held",
        "key", "kinds", "label", "last", "limit", "note", "noun", "nouns", "page", "pages",
        "range", "shown", "size", "stated", "status", "summary", "tab", "time", "total", "value",
        "what", "words",
    ];

    /// The ids of the messages the catalog file at `path` defines.
    fn ids(path: &Path) -> Vec<String> {
        let source = std::fs::read_to_string(path).expect("the catalog should be readable");
        let mut ids: Vec<String> = source
            .lines()
            .filter_map(|line| line.split_once(" ="))
            .map(|(id, _)| id)
            .filter(|id| {
                id.starts_with(|first: char| first.is_ascii_lowercase())
                    && id
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            })
            .map(str::to_owned)
            .collect();
        ids.sort();
        ids
    }

    #[test]
    fn the_shipped_languages_are_the_catalog_directories() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        let mut found: Vec<String> = std::fs::read_dir(&root)
            .expect("the locales directory should be readable")
            .map(|entry| {
                entry
                    .expect("an entry should be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        found.sort();
        assert_eq!(
            found,
            Locale::shipped(),
            "a catalog directory is a language"
        );
        assert!(
            found.contains(&String::from("en")),
            "English is the reference"
        );
    }

    #[test]
    fn every_catalog_holds_every_message_and_no_other() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        let mut declared: Vec<String> = Text::ALL.iter().map(|text| text.id().to_owned()).collect();
        declared.sort();
        for tag in Locale::shipped() {
            let catalog = ids(&root.join(&tag).join("ferrohealth-ui.ftl"));
            assert_eq!(
                declared, catalog,
                "the catalog {tag} is complete and holds nothing else"
            );
            let language = tag.parse().expect("a catalog directory is a language");
            // Every placeable a message names, so each formats whole.
            let args: Vec<(&'static str, Arg)> = PLACEABLES
                .iter()
                .map(|name| (*name, Arg::from(3)))
                .collect();
            let args = fluent_args(&args);
            for text in Text::ALL {
                let words = CATALOG
                    .lookup_single_language(&language, text.id(), Some(&args))
                    .unwrap_or_else(|error| panic!("{tag} formats {}: {error}", text.id()));
                assert!(
                    !words.trim().is_empty(),
                    "{tag} draws {} as words",
                    text.id()
                );
            }
        }
    }

    #[test]
    fn a_language_is_served_by_its_catalog_or_its_bare_language() {
        let british = Locale::parse("en-GB").expect("a tag");
        assert_eq!(
            "en-GB",
            british.tag(),
            "the page keeps the tag it asked for"
        );
        assert_eq!("English", british.text(Text::LanguageName));
        assert!(
            Locale::parse("not a tag").is_err(),
            "no language identifier"
        );
        let unknown = Locale::parse("qaa").expect("a private-use language");
        assert_eq!(
            "English",
            unknown.text(Text::LanguageName),
            "falls back to English"
        );
    }

    #[test]
    fn accept_language_picks_the_best_shipped_language() {
        assert_eq!("en-US", Locale::negotiate("en-US,en;q=0.9").tag());
        assert_eq!(
            "en",
            Locale::negotiate("qaa, en;q=0.5").tag(),
            "a language the library lacks is passed over"
        );
        assert_eq!("en", Locale::negotiate("").tag(), "no header is English");
        assert_eq!(
            "en",
            Locale::negotiate("en;q=0, *").tag(),
            "a zero weight is refused"
        );
        let configured = Locale::parse("en-GB").expect("a tag");
        assert_eq!(
            "en-US",
            Locale::negotiate_with("qaa, en-US;q=0.8", &*CATALOG, configured.clone()).tag(),
            "the best language the console's catalog holds"
        );
        assert_eq!(
            "en-GB",
            Locale::negotiate_with("qaa", &*CATALOG, configured).tag(),
            "none held is the configured locale"
        );
        assert_eq!(Some(500), quality("0.5"));
        assert_eq!(Some(1000), quality("1.000"));
        assert_eq!(None, quality("1.5"));
        assert_eq!(None, quality("0.1234"));
    }
}
