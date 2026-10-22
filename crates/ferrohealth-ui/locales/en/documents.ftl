# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# The English words of the document pane, the code viewer, the copy button and
# the upload field: one message per `Text` of `src/i18n/words/documents.rs`,
# in the order of their ids.

code-error = Error:
code-lines =
    { $count ->
        [one] { $count } line
       *[other] { $count } lines
    }
code-region = { $code }, one numbered line per line
code-warning = Warning:
copy-copied = Copied
copy-refused = The browser refused the clipboard.
copy-said-copied = Copied to the clipboard.
copy-said-refused = Copy failed: the browser refused the clipboard.
document-copy = Copy
document-format = Format
document-highlighted = Highlighted
document-highlighted-region = { $document }, highlighted
document-raw = Raw
document-raw-region = { $document }, raw
document-rendered = Rendered
document-view = View
size-bytes =
    { $count ->
        [one] { $count } byte
       *[other] { $count } bytes
    }
size-kib = { $count } KiB
size-mib = { $count } MiB
upload-busy = Uploading…
upload-drop-invite = Drop a file here, or
upload-drop-over = Drop to read the file
upload-read = { $file } read; check the text below before you send it.
upload-refused-kind = { $file } was not read: the field takes { $kinds } files.
upload-refused-size = { $file } was not read: it is { $size } and the field takes up to { $limit }.
upload-rule = { $kinds } files, up to { $limit }.
upload-unreadable = { $file } was not read: the browser could not read it.
