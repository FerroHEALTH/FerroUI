// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The family's personal preferences: the theme, the density, reduced
//! motion, the language, the first day of the week and the shortcut hints.
//!
//! Each preference is a cookie of its own (family rule 9), named by its
//! [`Preference::COOKIE`], with a default and a parse that reads a missing
//! or unknown value as the default. A console keeps each one in its cookie
//! (`SameSite=Lax`, `Secure`, one year, no patient data) when Your
//! preferences posts it, reads them all with [`Preferences::read`] while it
//! renders, and writes the theme, the density, the motion and the hints on
//! `<html>` as `data-theme`, `data-density`, `data-motion` and
//! `data-hints`, so a page arrives drawn in them before any script runs;
//! `system` follows the operating system through `prefers-color-scheme` and
//! `prefers-reduced-motion`, which the stylesheet reads. The library writes
//! no cookie and touches no browser storage: it holds the vocabulary alone.
//! No specification governs this: our own design.

use crate::calendar::Weekday;
use crate::i18n::{Locale, Text, Word, tr};

/// A personal preference kept in a cookie of its own (family rule 9).
pub trait Preference: Default + Sized {
    /// The name of the cookie the preference is kept in.
    const COOKIE: &'static str;

    /// The value the cookie holds for this choice.
    fn cookie_value(&self) -> &str;

    /// The choice the cookie value `value` names, or `None` for a value
    /// that names none.
    fn from_cookie(value: &str) -> Option<Self>;

    /// The choice the cookie value `value` names, the default for a cookie
    /// that is absent or names nothing.
    #[must_use]
    fn read(value: Option<&str>) -> Self {
        value.and_then(Self::from_cookie).unwrap_or_default()
    }
}

/// Implements [`Preference`] for a closed set whose inherent `key` and
/// `parse` are the cookie's value and its parse.
macro_rules! keyed_preference {
    ($type:ty, $cookie:literal) => {
        impl Preference for $type {
            const COOKIE: &'static str = $cookie;

            fn cookie_value(&self) -> &str {
                self.key()
            }

            fn from_cookie(value: &str) -> Option<Self> {
                Self::parse(value)
            }
        }
    };
}

/// A theme the operator can choose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// The light theme.
    Light,
    /// The dark theme.
    Dark,
    /// The theme the operating system asks for.
    #[default]
    System,
}

impl Theme {
    /// Every theme, in the order the switch offers them.
    pub const ALL: [Self; 3] = [Self::Light, Self::Dark, Self::System];

    /// The theme's key, as `data-theme` and a cookie carry it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }

    /// The theme's name in the page's locale, as the switch shows it.
    #[must_use]
    pub fn label(self, locale: &Locale) -> Word {
        tr(locale, match self {
            Self::Light => Text::ThemeLight,
            Self::Dark => Text::ThemeDark,
            Self::System => Text::ThemeSystem,
        })
    }

    /// The theme `key` names, or `None` for a key that names none.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.key() == key)
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

impl std::str::FromStr for Theme {
    type Err = UnknownTheme;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        Self::parse(key).ok_or(UnknownTheme)
    }
}

/// A key that names no [`Theme`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownTheme;

impl std::fmt::Display for UnknownTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the key names no theme: light, dark or system")
    }
}

impl std::error::Error for UnknownTheme {}

/// How tightly the page is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    /// The default spacing.
    #[default]
    Comfortable,
    /// Tighter spacing and rows, for a dense listing.
    Compact,
}

impl Density {
    /// Both densities, in the order a switch offers them.
    pub const ALL: [Self; 2] = [Self::Comfortable, Self::Compact];

    /// The density's name in the page's locale, as a reader sees it.
    #[must_use]
    pub fn label(self, locale: &Locale) -> Word {
        tr(locale, match self {
            Self::Comfortable => Text::DensityComfortable,
            Self::Compact => Text::DensityCompact,
        })
    }

    /// The density's key, as `data-density` carries it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Comfortable => "comfortable",
            Self::Compact => "compact",
        }
    }

    /// The density `key` names, or `None` for a key that names none.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|density| density.key() == key)
    }
}

impl std::fmt::Display for Density {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

impl std::str::FromStr for Density {
    type Err = UnknownDensity;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        Self::parse(key).ok_or(UnknownDensity)
    }
}

/// A key that names no [`Density`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownDensity;

impl std::fmt::Display for UnknownDensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the key names no density: comfortable or compact")
    }
}

impl std::error::Error for UnknownDensity {}

keyed_preference!(Theme, "theme");
keyed_preference!(Density, "density");

/// Whether the page moves less than the operating system asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    /// The motion the operating system asks for, through
    /// `prefers-reduced-motion`.
    #[default]
    System,
    /// Reduced motion, whatever the operating system asks.
    Reduce,
}

impl Motion {
    /// Both choices, in the order the preference offers them.
    pub const ALL: [Self; 2] = [Self::System, Self::Reduce];

    /// The choice's key, as `data-motion` and its cookie carry it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Reduce => "reduce",
        }
    }

    /// The choice `key` names, or `None` for a key that names none.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|motion| motion.key() == key)
    }

    /// The choice's words in the page's locale, as the preference offers
    /// it.
    #[must_use]
    pub fn label(self, locale: &Locale) -> Word {
        tr(locale, match self {
            Self::System => Text::MotionSystem,
            Self::Reduce => Text::MotionReduce,
        })
    }
}

keyed_preference!(Motion, "motion");

/// The day a week starts on in every calendar the page draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeekStart {
    /// The first day the page's language uses.
    #[default]
    Language,
    /// Monday, the first day of the ISO 8601 week.
    Monday,
    /// Sunday.
    Sunday,
}

impl WeekStart {
    /// Every choice, in the order the preference offers them.
    pub const ALL: [Self; 3] = [Self::Language, Self::Monday, Self::Sunday];

    /// The choice's key, as its cookie carries it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Language => "language",
            Self::Monday => "monday",
            Self::Sunday => "sunday",
        }
    }

    /// The choice `key` names, or `None` for a key that names none.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|start| start.key() == key)
    }

    /// The first day of the week, where the language starts it on
    /// `from_language`; a calendar takes it as its first day.
    #[must_use]
    pub const fn first_day(self, from_language: Weekday) -> Weekday {
        match self {
            Self::Language => from_language,
            Self::Monday => Weekday::Monday,
            Self::Sunday => Weekday::Sunday,
        }
    }
}

keyed_preference!(WeekStart, "week-start");

/// Where the key caps after a label show (R1); phones never show them
/// (family rule 6), and the shortcut legend keeps its keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Hints {
    /// On a wide screen with a keyboard: from 40 rem up, and not on a
    /// touch-only device.
    #[default]
    Wide,
    /// On every screen from 40 rem up, touch-only ones included.
    Always,
    /// Nowhere but in the legend.
    Never,
}

impl Hints {
    /// Every choice, in the order the preference offers them.
    pub const ALL: [Self; 3] = [Self::Wide, Self::Always, Self::Never];

    /// The choice's key, as `data-hints` and its cookie carry it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Wide => "wide",
            Self::Always => "always",
            Self::Never => "never",
        }
    }

    /// The choice `key` names, or `None` for a key that names none.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|hints| hints.key() == key)
    }

    /// The choice's words in the page's locale, as the preference offers
    /// it.
    #[must_use]
    pub fn label(self, locale: &Locale) -> Word {
        tr(locale, match self {
            Self::Wide => Text::HintsWide,
            Self::Always => Text::HintsAlways,
            Self::Never => Text::HintsNever,
        })
    }
}

keyed_preference!(Hints, "hints");

/// The language of the console's words.
///
/// A chosen language is kept as its BCP 47 tag; the console checks it is
/// one it offers (`crate::i18n::Locale::parse`) and follows the browser's
/// `Accept-Language` otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Language {
    /// The language the browser asks for, through `Accept-Language`.
    #[default]
    Browser,
    /// The language this BCP 47 tag names, such as `nl`.
    Chosen(String),
}

impl Language {
    /// The key of [`Language::Browser`], as its cookie carries it.
    pub const BROWSER: &'static str = "browser";

    /// The tag of the chosen language, `None` when the browser decides.
    #[must_use]
    pub fn tag(&self) -> Option<&str> {
        match self {
            Self::Browser => None,
            Self::Chosen(tag) => Some(tag),
        }
    }
}

impl Preference for Language {
    const COOKIE: &'static str = "language";

    fn cookie_value(&self) -> &str {
        self.tag().unwrap_or(Self::BROWSER)
    }

    fn from_cookie(value: &str) -> Option<Self> {
        // NOTE: BCP 47 §2.1: a tag is subtags of one to eight ASCII letters and
        // digits joined by hyphens; anything else names no language.
        let tag = value.split('-').all(|subtag| {
            (1..=8).contains(&subtag.len()) && subtag.chars().all(|c| c.is_ascii_alphanumeric())
        });
        if value == Self::BROWSER {
            Some(Self::Browser)
        } else if tag {
            Some(Self::Chosen(value.to_owned()))
        } else {
            None
        }
    }
}

/// Every family preference, as the console read them for one render.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Preferences {
    /// The theme.
    pub theme: Theme,
    /// The density.
    pub density: Density,
    /// Whether motion is reduced.
    pub motion: Motion,
    /// The language.
    pub language: Language,
    /// The first day of the week.
    pub week_start: WeekStart,
    /// Where the shortcut hints show.
    pub hints: Hints,
}

impl Preferences {
    /// The names of every preference's cookie, which Reset to defaults
    /// clears.
    pub const COOKIES: [&'static str; 6] = [
        Theme::COOKIE,
        Density::COOKIE,
        Motion::COOKIE,
        Language::COOKIE,
        WeekStart::COOKIE,
        Hints::COOKIE,
    ];

    /// The preferences the `Cookie` header value `cookies` holds, each
    /// cookie named `prefix` then its [`Preference::COOKIE`] (an empty
    /// `prefix` for the names as they are), and each one absent or
    /// unreadable at its default.
    #[must_use]
    pub fn read(cookies: &str, prefix: &str) -> Self {
        let value = |name: &str| cookie(cookies, &format!("{prefix}{name}"));
        Self {
            theme: Theme::read(value(Theme::COOKIE)),
            density: Density::read(value(Density::COOKIE)),
            motion: Motion::read(value(Motion::COOKIE)),
            language: Language::read(value(Language::COOKIE)),
            week_start: WeekStart::read(value(WeekStart::COOKIE)),
            hints: Hints::read(value(Hints::COOKIE)),
        }
    }
}

/// The value of the cookie `name` in the `Cookie` header value `cookies`,
/// `None` when it holds none.
#[must_use]
pub fn cookie<'a>(cookies: &'a str, name: &str) -> Option<&'a str> {
    // NOTE: RFC 6265 §5.4: cookies arrive as `name=value` pairs joined by `; `.
    cookies
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    use super::{
        Density, Hints, Language, Motion, Preference, Preferences, Theme, WeekStart, cookie,
    };
    use crate::calendar::Weekday;

    /// Every choice of `P` in `all` reads back from its cookie value.
    fn reads_back<P: Preference + PartialEq + std::fmt::Debug>(all: &[P]) {
        for choice in all {
            assert_eq!(
                Some(choice),
                P::from_cookie(choice.cookie_value()).as_ref(),
                "{choice:?} in the cookie {}",
                P::COOKIE
            );
        }
    }

    #[test]
    fn every_preference_reads_back_from_its_cookie_value() {
        reads_back(&Theme::ALL);
        reads_back(&Density::ALL);
        reads_back(&Motion::ALL);
        reads_back(&WeekStart::ALL);
        reads_back(&Hints::ALL);
        reads_back(&[Language::Browser, Language::Chosen(String::from("nl-BE"))]);
    }

    #[test]
    fn an_absent_or_unknown_cookie_is_the_default() {
        assert_eq!(Motion::System, Motion::read(None), "absent");
        assert_eq!(Motion::System, Motion::read(Some("always")), "unknown");
        assert_eq!(Hints::Wide, Hints::read(Some("")), "empty");
        assert_eq!(WeekStart::Language, WeekStart::read(Some("Monday")));
        assert_eq!(Language::Browser, Language::read(Some("")), "empty");
        assert_eq!(Language::Browser, Language::read(Some("not a tag")));
        assert_eq!(Language::Browser, Language::read(Some("en_GB")));
        assert_eq!(
            Language::Browser,
            Language::read(Some("abcdefghi")),
            "a subtag is at most eight characters"
        );
    }

    #[test]
    fn the_defaults_are_the_family_defaults() {
        let defaults = Preferences::default();
        assert_eq!(
            Theme::System,
            defaults.theme,
            "the theme follows the system"
        );
        assert_eq!(
            Density::Comfortable,
            defaults.density,
            "comfortable is the default (family rule 10)"
        );
        assert_eq!(Motion::System, defaults.motion);
        assert_eq!(Language::Browser, defaults.language);
        assert_eq!(WeekStart::Language, defaults.week_start);
        assert_eq!(Hints::Wide, defaults.hints);
    }

    #[test]
    fn each_preference_has_a_cookie_of_its_own() {
        let mut names = Preferences::COOKIES.to_vec();
        names.sort_unstable();
        names.dedup();
        assert_eq!(Preferences::COOKIES.len(), names.len(), "{names:?}");
    }

    #[test]
    fn the_preferences_are_read_from_a_cookie_header() {
        let read = Preferences::read(
            "a=1; app-theme=dark; app-density=compact; app-motion=reduce; \
             app-language=nl; app-week-start=sunday; app-hints=never; theme=light",
            "app-",
        );
        assert_eq!(
            Preferences {
                theme: Theme::Dark,
                density: Density::Compact,
                motion: Motion::Reduce,
                language: Language::Chosen(String::from("nl")),
                week_start: WeekStart::Sunday,
                hints: Hints::Never,
            },
            read
        );
        assert_eq!(Preferences::default(), Preferences::read("", ""));
        assert_eq!(
            Some("light"),
            cookie("app-theme=dark; theme=light", "theme")
        );
    }

    #[test]
    fn the_week_starts_on_the_language_s_day_unless_chosen() {
        assert_eq!(
            Weekday::Sunday,
            WeekStart::Language.first_day(Weekday::Sunday)
        );
        assert_eq!(
            Weekday::Monday,
            WeekStart::Monday.first_day(Weekday::Sunday)
        );
        assert_eq!(
            Weekday::Sunday,
            WeekStart::Sunday.first_day(Weekday::Monday)
        );
    }

    #[test]
    fn every_theme_and_density_reads_back_from_its_key() {
        for theme in Theme::ALL {
            assert_eq!(Some(theme), Theme::parse(theme.key()), "{theme:?}");
        }
        assert_eq!(None, Theme::parse("Dark"), "keys are lower case");
        assert_eq!(
            Theme::System,
            Theme::default(),
            "the default follows the system"
        );
        for density in [Density::Comfortable, Density::Compact] {
            assert_eq!(Some(density), Density::parse(density.key()), "{density:?}");
        }
        assert_eq!(None, Density::parse("cosy"), "only the two densities");
    }
}
