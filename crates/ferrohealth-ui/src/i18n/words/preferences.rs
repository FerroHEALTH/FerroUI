// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of Your preferences.

use crate::i18n::Text;

texts! {
    /// Shortcut hints on every wide screen.
    HintsAlways = "hints-always",
    /// No shortcut hints.
    HintsNever = "hints-never",
    /// Shortcut hints on wide screens with a keyboard, the default.
    HintsWide = "hints-wide",
    /// Motion always reduced.
    MotionReduce = "motion-reduce",
    /// Motion as the operating system asks for it, the default.
    MotionSystem = "motion-system",
    /// The Appearance tab.
    PreferencesAppearance = "preferences-appearance",
    /// The comfortable density's card.
    PreferencesDensityComfortableNote = "preferences-density-comfortable-note",
    /// The compact density's card.
    PreferencesDensityCompactNote = "preferences-density-compact-note",
    /// What the density governs.
    PreferencesDensityDesc = "preferences-density-desc",
    /// The line at the foot of Your preferences.
    PreferencesFoot = "preferences-foot",
    /// The shortcut hints preference.
    PreferencesHints = "preferences-hints",
    /// What the shortcut hints preference governs.
    PreferencesHintsDesc = "preferences-hints-desc",
    /// The line under Your preferences' title.
    PreferencesIntro = "preferences-intro",
    /// The Keyboard tab.
    PreferencesKeyboard = "preferences-keyboard",
    /// The language choice that follows the browser.
    PreferencesLanguageBrowser = "preferences-language-browser",
    /// The `$language` the browser asks for, under its choice.
    PreferencesLanguageBrowserNote = "preferences-language-browser-note",
    /// What the language governs.
    PreferencesLanguageDesc = "preferences-language-desc",
    /// The Language and region tab.
    PreferencesLanguageRegion = "preferences-language-region",
    /// The motion preference.
    PreferencesMotion = "preferences-motion",
    /// What the motion preference governs.
    PreferencesMotionDesc = "preferences-motion-desc",
    /// The Language and region tab as FerroEHR names it.
    PreferencesReading = "preferences-reading",
    /// The button that asks to reset every preference.
    PreferencesReset = "preferences-reset",
    /// What a reset does and leaves alone.
    PreferencesResetBody = "preferences-reset-body",
    /// The reset dialog's confirming button.
    PreferencesResetConfirm = "preferences-reset-confirm",
    /// The question of the reset dialog.
    PreferencesResetTitle = "preferences-reset-title",
    /// The button that posts a preference before the `preference` module runs.
    PreferencesSave = "preferences-save",
    /// The mark of a preference just kept.
    PreferencesSaved = "preferences-saved",
    /// The accessible name of Your preferences' tabs.
    PreferencesSections = "preferences-sections",
    /// The row that leads to the shortcut legend.
    PreferencesShortcuts = "preferences-shortcuts",
    /// What the shortcut legend holds.
    PreferencesShortcutsDesc = "preferences-shortcuts-desc",
    /// The link to the shortcut legend.
    PreferencesShortcutsOpen = "preferences-shortcuts-open",
    /// The dark theme's card: the brand's `$accent` on ink.
    PreferencesThemeDarkNote = "preferences-theme-dark-note",
    /// What the theme governs.
    PreferencesThemeDesc = "preferences-theme-desc",
    /// The light theme's card: the brand's `$accent` on white.
    PreferencesThemeLightNote = "preferences-theme-light-note",
    /// The system theme's card.
    PreferencesThemeSystemNote = "preferences-theme-system-note",
    /// The first-day-of-the-week preference.
    PreferencesWeek = "preferences-week",
    /// What the first day of the week governs.
    PreferencesWeekDesc = "preferences-week-desc",
    /// The week starting on the language's `$weekday`, the default.
    WeekStartLanguage = "week-start-language",
}
