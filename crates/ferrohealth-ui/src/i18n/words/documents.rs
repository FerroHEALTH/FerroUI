// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the document pane, the code viewer, the copy
//! button and the upload field.

use crate::i18n::Text;

texts! {
    /// The word before an error drawn under its line, for assistive
    /// technology.
    CodeError = "code-error",
    /// A code viewer's line count, by `$count`.
    CodeLines = "code-lines",
    /// The accessible name of a code viewer's box: `$code` is its name.
    CodeRegion = "code-region",
    /// The word before a warning drawn under its line, for assistive
    /// technology.
    CodeWarning = "code-warning",
    /// The copy button once the text is on the clipboard.
    CopyCopied = "copy-copied",
    /// The words beside a refused copy.
    CopyRefused = "copy-refused",
    /// The announcement of a copy.
    CopySaidCopied = "copy-said-copied",
    /// The announcement of a refused copy.
    CopySaidRefused = "copy-said-refused",
    /// The document pane's copy button.
    DocumentCopy = "document-copy",
    /// The accessible name of the format choice.
    DocumentFormat = "document-format",
    /// The document pane's highlighted view.
    DocumentHighlighted = "document-highlighted",
    /// The highlighted view of the `$document`.
    DocumentHighlightedRegion = "document-highlighted-region",
    /// The document pane's raw view.
    DocumentRaw = "document-raw",
    /// The raw view of the `$document`.
    DocumentRawRegion = "document-raw-region",
    /// The document pane's rendered view.
    DocumentRendered = "document-rendered",
    /// The accessible name of the view choice.
    DocumentView = "document-view",
    /// A size in bytes.
    SizeBytes = "size-bytes",
    /// A size in kibibytes.
    SizeKib = "size-kib",
    /// A size in mebibytes.
    SizeMib = "size-mib",
    /// The line while the console sends.
    UploadBusy = "upload-busy",
    /// The drop zone's words.
    UploadDropInvite = "upload-drop-invite",
    /// The drop zone's words while a file is over it.
    UploadDropOver = "upload-drop-over",
    /// The line once the `$file` is read.
    UploadRead = "upload-read",
    /// The line under it when the `$file` was not UTF-8 text.
    UploadReadLossy = "upload-read-lossy",
    /// The line once the `$file`, the first of several dropped, is read.
    UploadReadOne = "upload-read-one",
    /// A `$file` of a kind the field does not take.
    UploadRefusedKind = "upload-refused-kind",
    /// A `$file` of `$size` over the `$limit`.
    UploadRefusedSize = "upload-refused-size",
    /// The kinds of file and the limit the field takes.
    UploadRule = "upload-rule",
    /// The limit of a field that takes a file of any kind.
    UploadRuleAny = "upload-rule-any",
    /// A `$file` the browser could not read.
    UploadUnreadable = "upload-unreadable",
}
