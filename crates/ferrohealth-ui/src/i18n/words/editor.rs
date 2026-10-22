// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the code editor's toolbar and status line.

use crate::i18n::Text;

texts! {
    /// The status line's state of a draft changed since it was opened.
    EditorChanged = "editor-changed",
    /// What the status line says as the cursor reaches the `$line` an
    /// error names.
    EditorErrorOn = "editor-error-on",
    /// The status line's state of a file not saved yet.
    EditorNew = "editor-new",
    /// The cursor's `$line` and `$column`.
    EditorPosition = "editor-position",
    /// The status line's state of a draft as it was opened.
    EditorUnchanged = "editor-unchanged",
    /// What the status line says as the cursor reaches the `$line` a
    /// warning names.
    EditorWarningOn = "editor-warning-on",
}
