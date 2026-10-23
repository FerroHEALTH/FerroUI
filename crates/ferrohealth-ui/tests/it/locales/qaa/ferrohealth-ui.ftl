# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# A throwaway catalog for the tests alone, in the private-use language `qaa`
# (ISO 639-2, reserved for local use), with invented words. It holds a few
# of the library's messages, so the rest fall back to English. It never
# ships: the library's catalogs are the directories under `locales/`.

language-name = Qaa
step-first = Qirst
step-previous = Qrevious
step-next = Qext
step-last = Qast
rows-per-page = Qows per qage
page-of = Qage { $page } qof { $pages }
range-of = { $first }-{ $last } qof { $total } { $noun }
rows =
    { $count ->
        [one] qow
       *[other] qows
    }
cancel = Qancel
date-order = dmy
