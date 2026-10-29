// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's locales, English and a pseudo-locale, and the catalog of
//! the gallery's own words.
//!
//! The gallery's page strings are messages of its own Fluent catalog, looked
//! up as a console looks up its own, so the gallery is held to the rule it
//! shows. Each area of the gallery keeps its messages in a file of its own,
//! `locale/<area>.rs` with `locales/en/<area>.ftl` beside it, so two changes
//! to different areas touch different files. English is the one catalog.
//! The pseudo-locale draws every library string and every gallery string
//! accented and lengthened, so a page shows where a longer translation would
//! overflow or wrap.
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

use ferrohealth_ui::i18n::{Arg, Locale, Message, library_catalog};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader};

fluent_templates::static_loader! {
    static GALLERY = {
        locales: "./locales",
        fallback_language: "en",
        // NOTE: Fluent's guide, "Bidirectionality": the isolation marks around a
        // placeable are for mixed-direction text; the English catalog draws none.
        customise: |bundle| {
            bundle.set_use_isolating(false);
            bundle.set_formatter(Some(ferrohealth_ui::i18n::number::format_number));
        },
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

/// A string the gallery draws, by its message in the catalog. Each message
/// is an associated constant declared by its area's file under `locale/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Msg(&'static str);

impl Msg {
    /// Every message, area by area and by id within an area.
    pub fn all() -> impl Iterator<Item = Self> {
        AREAS.iter().flat_map(|area| area.iter().copied())
    }

    /// The message's id in the catalog.
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.0
    }
}

/// Declares one area's messages: each constant names one message of the
/// catalog by its id, and the area's `ALL` lists them for [`AREAS`].
macro_rules! messages {
    ($($(#[$meta:meta])+ $name:ident = $id:literal,)+) => {
        #[expect(
            non_upper_case_globals,
            reason = "a message is named as a word of the page, Msg::Overview, as when Msg was an enum"
        )]
        impl Msg {
            $($(#[$meta])+ pub const $name: Self = Self($id);)+
        }

        /// The area's messages, in the order of their ids.
        pub(super) const ALL: &[Msg] = &[$(Msg::$name,)+];
    };
}

// The areas, one file each, kept in alphabetical order.
mod charts;
mod clinical;
mod code;
mod configuration;
mod dock;
mod federation;
mod find;
mod forms;
mod layouts;
mod operations;
mod overlays;
mod overview;
mod preferences;
mod records;
mod shell;
mod sign_in;
mod tables;
mod templates;
mod terminology;
mod trees;

/// Every area's messages.
const AREAS: &[&[Msg]] = &[
    charts::ALL,
    clinical::ALL,
    code::ALL,
    configuration::ALL,
    dock::ALL,
    federation::ALL,
    find::ALL,
    forms::ALL,
    layouts::ALL,
    operations::ALL,
    overlays::ALL,
    overview::ALL,
    preferences::ALL,
    records::ALL,
    shell::ALL,
    sign_in::ALL,
    tables::ALL,
    templates::ALL,
    terminology::ALL,
    trees::ALL,
];

/// The gallery's message `msg` in `locale`, with the placeables `args`
/// filled in.
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
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales/en");
        let mut sources = Vec::new();
        for entry in std::fs::read_dir(folder).expect("the catalog folder") {
            let path = entry.expect("an entry").path();
            assert_eq!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("ftl"),
                "{} is no catalog file",
                path.display()
            );
            sources.push(std::fs::read_to_string(path).expect("a catalog file"));
        }
        let mut held: Vec<&str> = sources
            .iter()
            .flat_map(|source| source.lines())
            .filter_map(|line| line.split_once(" ="))
            .map(|(id, _)| id)
            .filter(|id| id.starts_with(|c: char| c.is_ascii_lowercase()) && !id.contains(' '))
            .collect();
        held.sort_unstable();
        let mut declared: Vec<&str> = Msg::all().map(Msg::id).collect();
        declared.sort_unstable();
        assert_eq!(
            declared, held,
            "every message is declared once and held once"
        );
        let english = fluent_templates::langid!("en");
        // Every placeable a message names, so each formats whole.
        let args: HashMap<Cow<'static, str>, FluentValue<'static>> = [
            "bytes", "count", "day", "format", "index", "kept", "kind", "line", "lines", "member",
            "mode", "name", "place", "query", "row", "total", "value", "version",
        ]
        .into_iter()
        .map(|name| (Cow::Borrowed(name), FluentValue::from(3)))
        .collect();
        for msg in Msg::all() {
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
