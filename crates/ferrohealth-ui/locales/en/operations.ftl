# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# The English words of the load report, the transform pair, the OMOP row
# preview and the HL7 v2 viewer: one message per `Text` of
# `src/i18n/words/operations.rs`, in the order of their ids.

rows-concept-zero = 0, no matching concept
rows-count =
    { $count ->
        [one] { $count } row
       *[other] { $count } rows
    }
rows-empty = empty text
rows-null = NULL
