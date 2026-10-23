// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the language switch, the name of each
//! catalog's language and the marks a number is written with.

use crate::i18n::Text;

texts! {
    /// The name of a catalog's language in that language.
    LanguageName = "language-name",
    /// The accessible name of the language switch.
    LanguageSwitch = "language-switch",
    /// The decimal mark of a number, after the CLDR number symbols.
    NumberDecimal = "number-decimal",
    /// The mark between groups of three digits, after the CLDR number
    /// symbols.
    NumberGroup = "number-group",
}
