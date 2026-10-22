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
diagnostics-warning = Warning
diagnostics-warnings =
    { $count ->
        [one] { $count } warning
       *[other] { $count } warnings
    }
diagnostics-whole = the whole document
