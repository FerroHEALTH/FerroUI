// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of Your preferences: the brands' accents the theme cards
//! name, and the rows a console adds to the library's three tabs.

use super::Msg;

messages! {
    /// `Azure`
    AccentAzure = "accent-azure",
    /// `Indigo`
    AccentIndigo = "accent-indigo",
    /// `Plum`
    AccentPlum = "accent-plum",
    /// `Teal`
    AccentTeal = "accent-teal",
    /// `Concepts are asked for in`
    ConceptLanguage = "concept-language",
    /// `The language tag the server answers a concept's display in, such as nl-NL.`
    ConceptLanguageDesc = "concept-language-desc",
    /// `en-GB`
    ConceptLanguageHint = "concept-language-hint",
    /// `Dates`
    Dates = "dates",
    /// `Written year first, with today said in words.`
    DatesDesc = "dates-desc",
    /// `2026-10-21 · today 09:42`
    DatesExample = "dates-example",
    /// `Hide them when I leave the page`
    HideOnLeave = "hide-on-leave",
    /// `Masks every revealed identifier as soon as you open another page or tab.`
    HideOnLeaveDesc = "hide-on-leave-desc",
    /// `Hide revealed identifiers when I leave the page`
    HideOnLeaveSwitch = "hide-on-leave-switch",
    /// `Revealed identifiers stay for`
    RevealFor = "reveal-for",
    /// `How long a revealed identifier shows before it is masked again; the server caps every reveal at 30 s.`
    RevealForDesc = "reveal-for-desc",
    /// `{ $count } seconds`
    RevealSeconds = "reveal-seconds",
    /// `Rows per page`
    RowsPerPage = "rows-per-page",
    /// `How many rows a listing shows on one page, from 10 to 200.`
    RowsPerPageDesc = "rows-per-page-desc",
    /// `Time zone`
    TimeZone = "time-zone",
    /// `Times are shown in the server's zone; your browser's is the same.`
    TimeZoneDesc = "time-zone-desc",
    /// `Europe/Amsterdam · CEST, UTC+2`
    TimeZoneValue = "time-zone-value",
}
