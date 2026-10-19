// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's locales: English and a pseudo-locale.
//!
//! English is the library's one catalog. The pseudo-locale draws every
//! library string accented and lengthened, so a page shows where a longer
//! translation would overflow or wrap.
//!
//! The pseudo-locale is the gallery's own and never ships: it is a catalog
//! the gallery hands the library with [`Locale::with_catalog`], drawing on
//! the library's English catalog. Its tag is `qps`, a private-use
//! language subtag (BCP 47, `qaa` to `qtz`), so no catalog the library ships
//! can serve it. No specification governs pseudo-localization: our own
//! design, after the common practice of accenting and padding each string.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

use ferrohealth_ui::i18n::{Locale, library_catalog};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{LanguageIdentifier, Loader};

/// The tag of the pseudo-locale.
pub const PSEUDO_TAG: &str = "qps";

/// The pseudo-locale's language.
static PSEUDO_LANGUAGE: LazyLock<LanguageIdentifier> =
    LazyLock::new(|| fluent_templates::langid!("qps"));

/// The pseudo-locale's catalog: the library's English, stretched.
#[derive(Debug, Clone, Copy)]
pub struct Pseudo;

/// The one pseudo catalog the gallery hands the library.
static PSEUDO: Pseudo = Pseudo;

impl Loader for Pseudo {
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
            return None;
        }
        library_catalog()
            .try_lookup_complete(&fluent_templates::langid!("en"), text_id, args)
            .map(|english| stretch(&english))
    }

    fn locales(&self) -> Box<dyn Iterator<Item = &LanguageIdentifier> + '_> {
        Box::new(std::iter::once(&*PSEUDO_LANGUAGE))
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
        Locale::parse(PSEUDO_TAG)
            .map_or_else(|_| Locale::english(), |locale| locale.with_catalog(&PSEUDO))
    } else {
        Locale::english()
    }
}

/// The locales the gallery's language switch offers.
#[must_use]
pub fn choices() -> Vec<Locale> {
    vec![Locale::english(), gallery_locale(PSEUDO_TAG)]
}

#[cfg(test)]
mod tests {
    use super::{PSEUDO_TAG, gallery_locale, stretch};

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
