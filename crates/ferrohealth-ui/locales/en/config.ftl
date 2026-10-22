# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1

# The English words of the revision lifecycle, the commit-confirm countdown,
# the write-only secret field and the provenance tag: one message per `Text`
# of `src/i18n/words/config.rs`, in the order of their ids.

countdown-ended = ended at { $time }
countdown-left = left, until { $time }
countdown-said =
    { $count ->
        [one] { $count } second left
       *[other] { $count } seconds left
    }
countdown-up = The time is up.
lifecycle-current = Current:
lifecycle-done = Done:
lifecycle-failed = Failed:
lifecycle-skipped = Not needed:
lifecycle-waiting = Waiting:
secret-broken = set, but { $file } cannot be read
secret-clear = Clear
secret-clear-note = The secret and its reference are removed. It is not used from the apply on.
secret-file = Read from a file
secret-file-help = A path the server can read; it checks that it can read it before it saves.
secret-file-label = Absolute path of the file it is read from
secret-keep = Keep
secret-leave-unset = Leave unset
secret-no-directory = No secrets directory is set, so a value cannot be written: read it from a file.
secret-redacted = set, not shown
secret-set = set, read from { $file }
secret-unset = not set
secret-value = Set a value
secret-value-help = Written to a new file only the server reads; the configuration holds only its path.
secret-value-label = Value, never shown again
