// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the forms page's long form and its decision dock.

use super::Msg;

messages! {
    /// `Discard`
    DockDiscard = "dock-discard",
    /// `2 changes in this draft`
    DockDraftChanges = "dock-draft-changes",
    /// `Give a reason to save.`
    DockGiveReason = "dock-give-reason",
    /// `Your changes`
    DockLabel = "dock-label",
    /// `Open the current settings`
    DockOpenCurrent = "dock-open-current",
    /// `Path`
    DockPath = "dock-path",
    /// `Below fhirconnect/ or omocl/, ending in .yml.`
    DockPathHelp = "dock-path-help",
    /// `Reason`
    DockReason = "dock-reason",
    /// `One line, at most 200 characters`
    DockReasonHelp = "dock-reason-help",
    /// `Write why you change the settings: the reason goes into the audit trail with your name.`
    DockReasonMissing = "dock-reason-missing",
    /// `Why you change the settings`
    DockReasonPlaceholder = "dock-reason-placeholder",
    /// `Synthetic thresholds for the night shift`
    DockReasonSample = "dock-reason-sample",
    /// `Saved: the settings apply at once.`
    DockSaved = "dock-saved",
    /// `Saving`
    DockSaving = "dock-saving",
    /// `Someone saved first: Demo Steward changed these settings at 09:41 today, so your changes were not sent.`
    DockStale = "dock-stale",
    /// `A long form`
    LongForm = "long-form",
    /// `Save the settings`
    SaveSettings = "save-settings",
    /// The label of the synthetic setting at `$index`.
    SyntheticSetting = "synthetic-setting",
}
