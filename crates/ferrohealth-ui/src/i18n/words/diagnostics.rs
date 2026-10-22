// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the diagnostics list.

use crate::i18n::Text;

texts! {
    /// The word of an error.
    DiagnosticsError = "diagnostics-error",
    /// A file's count of errors, by `$count`.
    DiagnosticsErrors = "diagnostics-errors",
    /// A file's counts of `$errors` and of `$warnings`, each already in
    /// words.
    DiagnosticsErrorsWarnings = "diagnostics-errors-warnings",
    /// A finding's place, its `$line` and `$column`.
    DiagnosticsLine = "diagnostics-line",
    /// The `$mapping` a finding belongs to.
    DiagnosticsMapping = "diagnostics-mapping",
    /// The word of a warning.
    DiagnosticsWarning = "diagnostics-warning",
    /// A file's count of warnings, by `$count`.
    DiagnosticsWarnings = "diagnostics-warnings",
    /// The place of a finding about the whole document.
    DiagnosticsWhole = "diagnostics-whole",
}
