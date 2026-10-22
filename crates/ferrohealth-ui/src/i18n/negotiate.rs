// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The choice of a [`Locale`] by an `Accept-Language` header
//! (RFC 9110 §12.5.4).

use fluent_templates::{LanguageIdentifier, Loader};

use crate::i18n::Locale;

impl Locale {
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

#[cfg(test)]
mod tests {
    use super::quality;
    use crate::i18n::{CATALOG, Locale};

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
