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
issues-more =
    { $count ->
        [one] { $count } more issue
       *[other] { $count } more issues
    }
issues-not-drawn =
    { $count ->
        [one] { $count } more issue is not shown here.
       *[other] { $count } more issues are not shown here.
    }
refusal-evidence = What the server sent
refusal-no-answer = No answer:
refusal-status = { $status }: { $summary }
severity-error = error
severity-fatal = fatal
severity-information = information
severity-warning = warning
time-ago-days =
    { $count ->
        [one] { $count } day ago
       *[other] { $count } days ago
    }
time-ago-hours = { $count } h ago
time-ago-minutes = { $count } min ago
time-ago-months =
    { $count ->
        [one] { $count } month ago
       *[other] { $count } months ago
    }
time-ago-seconds = { $count } s ago
time-ago-weeks =
    { $count ->
        [one] { $count } week ago
       *[other] { $count } weeks ago
    }
time-ago-years =
    { $count ->
        [one] { $count } year ago
       *[other] { $count } years ago
    }
time-yesterday = yesterday
timestamp-utc = { $date } { $time } UTC
