// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the configuration parts: the revision lifecycle
//! and its pending card, the commit-confirm countdown, the write-only
//! secret field and the provenance tag.

use crate::i18n::Text;

texts! {
    /// The countdown's line once the time is up: `$time` is the deadline.
    CountdownEnded = "countdown-ended",
    /// The countdown's line under the time left: `$time` is the deadline.
    CountdownLeft = "countdown-left",
    /// What the live region says at a set point: `$count` seconds left.
    CountdownSaid = "countdown-said",
    /// What the live region says once the time is up.
    CountdownUp = "countdown-up",
    /// The hidden state word of the current step.
    LifecycleCurrent = "lifecycle-current",
    /// The hidden state word of a step done.
    LifecycleDone = "lifecycle-done",
    /// The hidden state word of a step that failed.
    LifecycleFailed = "lifecycle-failed",
    /// The hidden state word of a step the revision does not take.
    LifecycleSkipped = "lifecycle-skipped",
    /// The hidden state word of a step still to come.
    LifecycleWaiting = "lifecycle-waiting",
    /// The state of a secret whose file cannot be read: `$file` is its path.
    SecretBroken = "secret-broken",
    /// The Clear choice of a secret field.
    SecretClear = "secret-clear",
    /// The warning under the Clear choice.
    SecretClearNote = "secret-clear-note",
    /// The Read from a file choice of a secret field.
    SecretFile = "secret-file",
    /// The help under the path of the file a secret is read from.
    SecretFileHelp = "secret-file-help",
    /// The label of the path of the file a secret is read from.
    SecretFileLabel = "secret-file-label",
    /// The Keep choice of a secret field that is set.
    SecretKeep = "secret-keep",
    /// The Keep choice of a secret field that is not set.
    SecretLeaveUnset = "secret-leave-unset",
    /// The refusal of a value where no secrets directory is set.
    SecretNoDirectory = "secret-no-directory",
    /// The state of a secret that is set where the page may not say where.
    SecretRedacted = "secret-redacted",
    /// The state of a secret that is set: `$file` is the path it is read from.
    SecretSet = "secret-set",
    /// The state of a secret that is not set.
    SecretUnset = "secret-unset",
    /// The Set a value choice of a secret field.
    SecretValue = "secret-value",
    /// The help under a secret's new value.
    SecretValueHelp = "secret-value-help",
    /// The label of a secret's new value.
    SecretValueLabel = "secret-value-label",
}
