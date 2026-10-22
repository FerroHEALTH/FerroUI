// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The status tones: the ground, the text and the dot of a pill, a badge, a
//! status indicator, a banner or a toast.
//!
//! A tone repeats the word it sits beside and never stands for it (WCAG 2.2
//! SC 1.4.1).

/// A status tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Tone {
    /// It works.
    Ok,
    /// It may not work.
    Warn,
    /// It does not work.
    Bad,
    /// A fact to know, neither good nor bad.
    Info,
    /// Nothing is known yet, or it does not apply.
    #[default]
    Neutral,
    /// The brand's accent, for a mark that is no status.
    Accent,
}

impl Tone {
    /// Every tone, in the order the gallery shows them.
    pub const ALL: [Self; 6] = [
        Self::Ok,
        Self::Warn,
        Self::Bad,
        Self::Info,
        Self::Neutral,
        Self::Accent,
    ];

    /// The class that sets the tone's custom properties.
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::Ok => "fui-tone-ok",
            Self::Warn => "fui-tone-warn",
            Self::Bad => "fui-tone-bad",
            Self::Info => "fui-tone-info",
            Self::Neutral => "fui-tone-neutral",
            Self::Accent => "fui-tone-accent",
        }
    }

    /// The classes of a status pill, a dot and a word, in this tone.
    #[must_use]
    pub const fn pill(self) -> &'static str {
        match self {
            Self::Ok => "fui-pill fui-tone-ok",
            Self::Warn => "fui-pill fui-tone-warn",
            Self::Bad => "fui-pill fui-tone-bad",
            Self::Info => "fui-pill fui-tone-info",
            Self::Neutral => "fui-pill fui-tone-neutral",
            Self::Accent => "fui-pill fui-tone-accent",
        }
    }

    /// The classes of a badge, a word with no dot, in this tone.
    #[must_use]
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Ok => "fui-badge fui-tone-ok",
            Self::Warn => "fui-badge fui-tone-warn",
            Self::Bad => "fui-badge fui-tone-bad",
            Self::Info => "fui-badge fui-tone-info",
            Self::Neutral => "fui-badge fui-tone-neutral",
            Self::Accent => "fui-badge fui-tone-accent",
        }
    }

    /// The tone's name, as the gallery labels it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Bad => "bad",
            Self::Info => "info",
            Self::Neutral => "neutral",
            Self::Accent => "accent",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tone;

    #[test]
    fn every_tone_names_its_own_class() {
        for tone in Tone::ALL {
            assert!(tone.pill().ends_with(tone.class()), "{tone:?}");
            assert!(tone.badge().ends_with(tone.class()), "{tone:?}");
            assert!(tone.class().ends_with(tone.name()), "{tone:?}");
        }
    }
}
