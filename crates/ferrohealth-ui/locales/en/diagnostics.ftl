# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# The English words of the diagnostics list: one message per `Text` of
# `src/i18n/words/diagnostics.rs`, in the order of their ids.

diagnostics-error = Error
diagnostics-errors =
    { $count ->
        [one] { $count } error
       *[other] { $count } errors
    }
diagnostics-errors-warnings = { $errors }, { $warnings }
diagnostics-line = line { $line }, column { $column }
diagnostics-mapping = Mapping { $mapping }
diagnostics-more = { $shown } of { $total } findings shown.
diagnostics-more-all = Show all { $total }
diagnostics-none = No errors or warnings.
diagnostics-warning = Warning
diagnostics-warnings =
    { $count ->
        [one] { $count } warning
       *[other] { $count } warnings
    }
diagnostics-whole = the whole document
