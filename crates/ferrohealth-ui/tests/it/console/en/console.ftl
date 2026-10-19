# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# A console's catalog as the tests read it: the nouns its listings are
# named by and the title of its editor.

value-sets =
    { $count ->
        [one] value set
       *[other] value sets
    }
    .empty = No value sets
    .opening = Value sets
ehrs =
    { $count ->
        [one] EHR
       *[other] EHRs
    }
    .empty = No EHRs
    .opening = EHRs
file-editor =
    { $mode ->
        [new] New mapping file
       *[edit] Edit { $name }
    }
