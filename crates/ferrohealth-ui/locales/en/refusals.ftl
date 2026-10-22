# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# The English words of a refusal and its issues: one message per `Text` of
# `src/i18n/words/refusals.rs`, in the order of their ids.

issue-no-text = The issue carries no text.
issue-place = at
issues-left-out =
    { $count ->
        [one] { $count } information issue with no text left out.
       *[other] { $count } information issues with no text left out.
    }
refusal-evidence = What the server sent
refusal-no-answer = No answer: { $summary }
refusal-status = { $status }: { $summary }
severity-error = error
severity-fatal = fatal
severity-information = information
severity-warning = warning
timestamp-utc = { $date } { $time } UTC
