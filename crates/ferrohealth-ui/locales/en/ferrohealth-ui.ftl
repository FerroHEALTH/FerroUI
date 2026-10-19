# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# The English catalog of the strings the FerroHEALTH UI components draw
# themselves: the reference catalog, complete, and the one every lookup
# falls back to. Each message is one `Text` of the crate's `i18n` module;
# a catalog of another language holds the same messages in a directory
# named by its BCP 47 tag beside this one.

## The locale

language-name = English
language-switch = Language

## The shell

skip-to-page = Skip to the page
open-menu = Menu
close = Close
footer-about = About
sign-out = Sign out
find = Find
breadcrumb = Breadcrumb
none-value = none

## Themes and densities

theme-light = Light
theme-dark = Dark
theme-system = System
theme-switch = Theme
theme-to-dark = Switch to the dark theme
theme-to-light = Switch to the light theme
density-comfortable = Comfortable
density-compact = Compact
density-switch = Density

## Buttons and dialogs

variant-primary = Primary
variant-secondary = Secondary
variant-danger = Danger
variant-destructive = Destructive
variant-ghost = Ghost
cancel = Cancel
toast-region = Notices
toast-dismiss = Dismiss
help-show = Explain these parameters
help-hide = Hide help
combobox-no-match = No match

## Sign-in

sign-in-username = Username
sign-in-password = Password
sign-in-submit = Sign in
sign-in-or = or
sign-in-none = No sign-in method is available: the console has none configured.

## Tables and paging

step-first = First
step-previous = Previous
step-next = Next
step-last = Last
step-unavailable = (unavailable)
rows-per-page = Rows per page
page-of = Page { $page } of { $pages }
range = { $first }-{ $last } of { $total }
range-empty = No rows
range-of = { $first }-{ $last } of { $total } { $noun }
open-range-of = { $nouns } { $first } to { $last }
partial-note = The server counted { $stated } { $noun }, and this listing holds { $held }.
range-with-note = { $range }. { $note }
rows =
    { $count ->
        [one] row
       *[other] rows
    }
rows-opening = Rows
change-setting = Setting
change-before = Before
change-after = After
change-removed = removed
change-added = added
change-not-set = not set
change-none = Nothing changes.

## Refusals

severity-fatal = fatal
severity-error = error
severity-warning = warning
severity-information = information
issue-no-text = The issue carries no text.
issue-place = at
issues-left-out =
    { $count ->
        [one] { $count } information issue with no text left out.
       *[other] { $count } information issues with no text left out.
    }
refusal-status = { $status }: { $summary }
refusal-no-answer = No answer: { $summary }
refusal-evidence = What the server sent
timestamp-utc = { $date } { $time } UTC

## Charts

chart-no-value = No value
chart-no-value-cell = no value
chart-as-table = { $chart }, as a table
chart-x = X
chart-series = Series

## Documents and uploads

document-highlighted = Highlighted
document-raw = Raw
document-rendered = Rendered
document-format = Format
document-view = View
document-copy = Copy
document-highlighted-region = { $document }, highlighted
document-raw-region = { $document }, raw
copy-copied = Copied
copy-said-copied = Copied to the clipboard.
copy-said-refused = Copy failed: the browser refused the clipboard.
copy-refused = The browser refused the clipboard.
upload-drop-invite = Drop a file here, or
upload-drop-over = Drop to read the file
upload-rule = { $kinds } files, up to { $limit }.
upload-read = { $file } read; check the text below before you send it.
upload-busy = Uploading…
upload-refused-kind = { $file } was not read: the field takes { $kinds } files.
upload-refused-size = { $file } was not read: it is { $size } and the field takes up to { $limit }.
upload-unreadable = { $file } was not read: the browser could not read it.
size-bytes =
    { $count ->
        [one] { $count } byte
       *[other] { $count } bytes
    }
size-kib = { $count } KiB
size-mib = { $count } MiB
