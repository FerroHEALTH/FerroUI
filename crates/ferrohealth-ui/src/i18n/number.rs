// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Numbers written as the page's language writes them: the digits grouped
//! in threes and the decimal mark of the language.
//!
//! The marks are messages of each language's catalog (`number-group`,
//! `number-decimal` in `locales/<tag>/language.ftl`), so a language brings
//! its own with no code change. A number a message takes as a placeable is
//! written by [`format_number`], the formatter of every Fluent bundle the
//! library builds; a number a component draws outside a message, such as a
//! count beside a title, is written by [`grouped`]. A catalog keeps a number
//! ungrouped with `NUMBER($n, useGrouping: "false")` (Fluent's guide,
//! "Functions"). No specification governs the marks: our own design, after
//! the CLDR number symbols.

use std::convert::Infallible;

use fluent_templates::LanguageIdentifier;
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::fluent_bundle::memoizer::MemoizerKind;
use intl_memoizer::Memoizable;

use super::{Locale, Text, locale};

/// The marks a language writes a number with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Marks {
    group: String,
    decimal: String,
}

impl Marks {
    /// The marks of `locale`, from its catalog.
    pub(super) fn of(locale: &Locale) -> Self {
        Self {
            group: locale.text(Text::NumberGroup),
            decimal: locale.text(Text::NumberDecimal),
        }
    }

    /// The number `plain`, written as Rust writes a float or an integer
    /// ("-1234567.5"), in these marks, grouped when `grouped`.
    pub(super) fn write(&self, plain: &str, grouped: bool) -> String {
        let (sign, unsigned) = plain
            .strip_prefix('-')
            .map_or(("", plain), |rest| ("-", rest));
        let (whole, fraction) = unsigned
            .split_once('.')
            .map_or((unsigned, None), |(whole, fraction)| {
                (whole, Some(fraction))
            });
        let mut written = String::from(sign);
        let digits = whole.chars().count();
        for (index, digit) in whole.chars().enumerate() {
            let left = digits - index;
            if grouped && index > 0 && left % 3 == 0 {
                written.push_str(&self.group);
            }
            written.push(digit);
        }
        if let Some(fraction) = fraction {
            written.push_str(&self.decimal);
            written.push_str(fraction);
        }
        written
    }
}

/// Returns `count` written in the page's language: "312,000" in English.
///
/// For a number a component draws outside a message, such as the count
/// after a title or on a tab; a number inside a message is written by the
/// catalog's formatter.
#[must_use]
pub fn grouped(count: u64) -> String {
    Marks::of(&locale()).write(&count.to_string(), true)
}

/// The mark the page's language groups the digits of a count by, resolved
/// on the server, so an island writes a count it learns in the browser as
/// the server writes one.
///
/// An island carries no catalog (`leptos-ui.md` §15.4), so a widget whose
/// island counts takes this as part of its words.
///
/// # Examples
///
/// ```
/// use ferrohealth_ui::i18n::number::Grouping;
///
/// assert_eq!("12,480", Grouping::default().write(12_480));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Grouping {
    group: String,
}

impl Default for Grouping {
    fn default() -> Self {
        Self {
            group: String::from(","),
        }
    }
}

impl Grouping {
    /// The grouping of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            group: locale().text(Text::NumberGroup),
        }
    }

    /// Returns `count` with its digits grouped in threes: "12,480" in
    /// English.
    #[must_use]
    pub fn write(&self, count: u64) -> String {
        Marks {
            group: self.group.clone(),
            decimal: String::new(),
        }
        .write(&count.to_string(), true)
    }
}

/// The language of the bundle a formatter runs in, read from the bundle's
/// memoizer, which builds each of its formatters for that language.
struct Language(LanguageIdentifier);

impl Memoizable for Language {
    type Args = ();
    type Error = Infallible;

    fn construct(language: LanguageIdentifier, (): ()) -> Result<Self, Infallible> {
        Ok(Self(language))
    }
}

/// Writes a number a message takes in the marks of the bundle's language:
/// the formatter the library sets on its own catalogs, and that a console
/// sets on its own in the `customise` of its `static_loader!`.
///
/// It returns `None`, leaving Fluent's own writing, for anything but a
/// finite number.
///
/// # Examples
///
/// ```text
/// fluent_templates::static_loader! {
///     static CATALOG = {
///         locales: "./locales",
///         fallback_language: "en",
///         customise: |bundle| {
///             bundle.set_use_isolating(false);
///             bundle.set_formatter(Some(ferrohealth_ui::i18n::number::format_number));
///         },
///     };
/// }
/// ```
#[must_use]
pub fn format_number<M: MemoizerKind>(value: &FluentValue<'_>, memoizer: &M) -> Option<String> {
    let FluentValue::Number(number) = value else {
        return None;
    };
    if !number.value.is_finite() {
        return None;
    }
    let language = memoizer
        .with_try_get_threadsafe::<Language, _, _>((), |language| language.0.clone())
        .ok()?;
    // NOTE: no specification governs this: our own design; the page's locale, with a
    // console's catalog of its language, gives the marks when the languages agree.
    let page = locale();
    let marks = if page.language == language {
        Marks::of(&page)
    } else {
        Marks::of(&Locale::of(language))
    };
    Some(marks.write(&number.as_string(), number.options.use_grouping))
}

/// The value of `count` exactly as a float, when a float holds it exactly
/// (up to 2^53), so a large count still selects its plural and is grouped.
pub(super) fn exact(count: u64) -> Option<f64> {
    const LIMIT: u64 = 1 << 53;
    if count > LIMIT {
        return None;
    }
    let high = u32::try_from(count >> 32).ok()?;
    let low = u32::try_from(count & u64::from(u32::MAX)).ok()?;
    Some(f64::from(high).mul_add(4_294_967_296.0, f64::from(low)))
}

#[cfg(test)]
mod tests {
    use super::{Marks, exact};

    fn english() -> Marks {
        Marks {
            group: String::from(","),
            decimal: String::from("."),
        }
    }

    #[test]
    fn digits_are_grouped_in_threes_from_the_decimal_mark() {
        let marks = english();
        assert_eq!("0", marks.write("0", true));
        assert_eq!("999", marks.write("999", true));
        assert_eq!("1,000", marks.write("1000", true));
        assert_eq!("312,000", marks.write("312000", true));
        assert_eq!("4,000,000,000", marks.write("4000000000", true));
        assert_eq!("-1,234,567.25", marks.write("-1234567.25", true));
        assert_eq!("1234567.25", marks.write("1234567.25", false));
    }

    #[test]
    fn a_language_writes_its_own_marks() {
        let german = Marks {
            group: String::from("."),
            decimal: String::from(","),
        };
        assert_eq!("12.345.678,5", german.write("12345678.5", true));
    }

    #[test]
    fn a_count_a_float_holds_exactly_is_kept_as_a_number() {
        assert_eq!(Some(12_345_678_901.0), exact(12_345_678_901));
        assert_eq!(Some(9_007_199_254_740_992.0), exact(1 << 53));
        assert_eq!(None, exact((1 << 53) + 1));
    }
}
