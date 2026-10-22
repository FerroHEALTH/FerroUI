// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The stylesheet and the brand themes, as text a console serves.
//!
//! A console serves [`STYLESHEET`] and its brand's theme as two stylesheets
//! of its own origin (so `style-src 'self'` holds), the theme after the
//! stylesheet, or writes both into its build and imports them into its
//! Tailwind entry; [`TAILWIND_THEME`] maps the roles to Tailwind utilities
//! for a console whose own views use them. The files are `css/` and
//! `themes/` of this crate.

/// The role tokens, the base layer and every component class.
pub const STYLESHEET: &str = include_str!("../css/ferrohealth-ui.css");

/// The roles as Tailwind 4 theme variables, for a console's Tailwind entry.
pub const TAILWIND_THEME: &str = include_str!("../css/tailwind-theme.css");

/// A FerroHEALTH console's brand, and its theme file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Brand {
    /// FerroBRIDGE, Indigo and Iron.
    Bridge,
    /// FerroFED, Azure and Iron.
    Fed,
    /// FerroEHR, the teal of its viewer.
    Ehr,
    /// FerroTERM, teal.
    Term,
    /// FerroPIX, Plum and Iron.
    Pix,
}

impl Brand {
    /// Every brand, in the order the gallery shows them.
    pub const ALL: [Self; 5] = [Self::Bridge, Self::Fed, Self::Ehr, Self::Term, Self::Pix];

    /// The brand's key, as a URL path segment or a file name carries it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Bridge => "ferrobridge",
            Self::Fed => "ferrofed",
            Self::Ehr => "ferroehr",
            Self::Term => "ferroterm",
            Self::Pix => "ferropix",
        }
    }

    /// The product's name as an operator reads it.
    #[must_use]
    pub const fn product(self) -> &'static str {
        match self {
            Self::Bridge => "FerroBRIDGE",
            Self::Fed => "FerroFED",
            Self::Ehr => "FerroEHR",
            Self::Term => "FerroTERM",
            Self::Pix => "FerroPIX",
        }
    }

    /// The brand's theme file: the accent of each theme over the roles of
    /// [`STYLESHEET`].
    #[must_use]
    pub const fn theme(self) -> &'static str {
        match self {
            Self::Bridge => include_str!("../themes/ferrobridge.css"),
            Self::Fed => include_str!("../themes/ferrofed.css"),
            Self::Ehr => include_str!("../themes/ferroehr.css"),
            Self::Term => include_str!("../themes/ferroterm.css"),
            Self::Pix => include_str!("../themes/ferropix.css"),
        }
    }

    /// The brand `key` names, or `None` for a key that names none.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|brand| brand.key() == key)
    }
}
