// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The locale a page renders in, and the catalogs of the strings the
//! components draw themselves.
//!
//! Every word a component draws on its own (a pager step, a dialog's
//! cancel button, the theme toggle's name, an empty table's range) is a
//! message of a Fluent catalog under the crate's `locales/` directory, one
//! directory per language and one file per area of components
//! (`locales/en/shell.ftl`), each area's messages declared in `words/`. The
//! catalogs are embedded at build time by `fluent_templates::static_loader!`,
//! which reads the directory, so a language joins as one catalog directory
//! and no code changes. English (`en`) is the reference catalog: complete,
//! and the one every lookup falls back to. [`Locale::negotiate`] chooses a
//! locale by an `Accept-Language` header.
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

mod negotiate;
pub mod number;
mod words;

fluent_templates::static_loader! {
    static CATALOG = {
        locales: "./locales",
        fallback_language: "en",
        // NOTE: Fluent's guide, "Bidirectionality": the isolation marks around a
        // placeable are for mixed-direction text; the English catalog draws none.
        customise: |bundle| {
            bundle.set_use_isolating(false);
            bundle.set_formatter(Some(number::format_number));
        },
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
    /// A number, written in the marks of the page's language (its digits
    /// grouped in threes) and selectable by plural rule.
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
        u64::try_from(number).map_or_else(|_| Self::Text(number.to_string()), Self::from)
    }
}

impl From<u64> for Arg {
    fn from(number: u64) -> Self {
        // NOTE: no specification governs this: our own design; a count past what a float
        // holds exactly is written grouped as text, which a plural reads as its default.
        number::exact(number).map_or_else(|| Self::Text(number::grouped(number)), Self::Number)
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
        self.library_word(id, args).0
    }

    /// The library's message `id` in this locale, with `args` filled in,
    /// and whether it is English drawn on a page in another language
    /// because no catalog of the page's language holds it.
    fn library_word(&self, id: &str, args: &[(&'static str, Arg)]) -> (String, bool) {
        let args = fluent_args(args);
        // NOTE: no specification governs this: our own design; a message a catalog
        // lacks, or cannot format, is looked for in the next one, ending in English.
        let own = self
            .served
            .as_ref()
            .and_then(|served| CATALOG.lookup_single_language(served, id, Some(&args)).ok())
            .or_else(|| {
                self.supplement.and_then(|catalog| {
                    catalog.try_lookup_complete(&self.language, id, Some(&args))
                })
            });
        if let Some(own) = own {
            return (own, false);
        }
        let fell_back = self.language.language != english().language;
        let text = CATALOG
            .lookup_single_language(&english(), id, Some(&args))
            .ok()
            .unwrap_or_else(|| missing(id));
        (text, fell_back)
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

/// A string a component draws itself, by its message in the catalog. Each
/// message is an associated constant declared by its area's file under
/// `words/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Text(&'static str);

impl Text {
    /// The message's id in the catalog.
    pub(crate) const fn id(self) -> &'static str {
        self.0
    }
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

/// The library's message `text` in the page's locale with `args` filled in,
/// drawn as text, inside `<span lang="en">` when it is English on a page in
/// another language because no catalog of the page's language holds it
/// (WCAG 2.2 SC 3.1.2).
pub(crate) fn tr_word(text: Text, args: &[(&'static str, Arg)]) -> AnyView {
    let (words, fell_back) = locale().library_word(text.id(), args);
    if fell_back {
        view! { <span lang="en">{words}</span> }.into_any()
    } else {
        words.into_any()
    }
}

#[cfg(test)]
mod tests {
    use super::{Locale, Text};

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
}
