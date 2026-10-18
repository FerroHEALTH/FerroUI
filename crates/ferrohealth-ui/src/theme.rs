// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The light, dark and system themes, and the comfortable and compact
//! densities.
//!
//! A console writes the operator's choice as `data-theme` (and
//! `data-density`) on `<html>` while it renders, so a page arrives drawn in
//! it before any script runs; `system` follows the operating system through
//! `prefers-color-scheme`, which the stylesheet reads. Where the choice is
//! kept is the console's policy (a server cookie, never browser storage, in
//! the consoles that forbid it), so the library holds the vocabulary alone.
//! No specification governs this: our own design.

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

    /// The theme's name, as the switch shows it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::System => "System",
        }
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
        [Self::Comfortable, Self::Compact]
            .into_iter()
            .find(|density| density.key() == key)
    }
}

#[cfg(test)]
mod tests {
    use super::{Density, Theme};

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
