// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the configuration page and its provisioned file.

use super::Msg;

messages! {
    /// `About this file`
    CfAboutFile = "cf-about-file",
    /// `after the approval`
    CfAfterApproval = "cf-after-approval",
    /// `another operator`
    CfAnotherOperator = "cf-another-operator",
    /// `yours, or any operator's`
    CfAnyOperator = "cf-any-operator",
    /// `Applied`
    CfApplied = "cf-applied",
    /// `Apply`
    CfApply = "cf-apply",
    /// `Apply revision 10`
    CfApply10 = "cf-apply-10",
    /// `Apply opens once an operator other than you approves it.`
    CfApplyWhy = "cf-apply-why",
    /// `Approval`
    CfApproval = "cf-approval",
    /// `Approved`
    CfApproved = "cf-approved",
    /// `Broker`
    CfBroker = "cf-broker",
    /// `The AMQP URL, from an environment variable; never shown.`
    CfBrokerDesc = "cf-broker-desc",
    /// `by 10:14:42, or it rolls back`
    CfBy141442 = "cf-by-141442",
    /// `by Demo Operator, 10:02`
    CfByOperator1002 = "cf-by-operator-1002",
    /// `by you, 09:51`
    CfByYou0951 = "cf-by-you-0951",
    /// `by you, 10:13:58`
    CfByYou1358 = "cf-by-you-1358",
    /// `Clear`
    CfClear = "cf-clear",
    /// `Client secret`
    CfClientSecret = "cf-client-secret",
    /// `Compare 10 with 11`
    CfCompare = "cf-compare",
    /// `Confirm`
    CfConfirm = "cf-confirm",
    /// `Confirm revision 10`
    CfConfirm10 = "cf-confirm-10",
    /// `Confirmed`
    CfConfirmed = "cf-confirmed",
    /// `Confirmed by you at 10:13:58, 44 seconds before the deadline; the server no longer rolls it back.`
    CfConfirmedLine = "cf-confirmed-line",
    /// `Confirmed`
    CfConfirmedStep = "cf-confirmed-step",
    /// `Revision 10 runs, confirmed`
    CfConfirmedTitle = "cf-confirmed-title",
    /// `Database`
    CfDatabase = "cf-database",
    /// `The PostgreSQL URL; read from a file and never shown.`
    CfDatabaseDesc = "cf-database-desc",
    /// `the deadline passed at 10:14:42`
    CfDeadline = "cf-deadline",
    /// `Draft`
    CfDraft = "cf-draft",
    /// `File`
    CfFile = "cf-file",
    /// `Read from a file`
    CfFromFile = "cf-from-file",
    /// `Set, kept`
    CfKept = "cf-kept",
    /// `Revision lifecycle`
    CfLifecycle = "cf-lifecycle",
    /// `Mapping files`
    CfMappingFiles = "cf-mapping-files",
    /// `Saved, no approval needed`
    CfNoApproval = "cf-no-approval",
    /// `Not confirmed`
    CfNotConfirmed = "cf-not-confirmed",
    /// `not needed`
    CfNotNeeded = "cf-not-needed",
    /// `Not set`
    CfNotSet = "cf-not-set",
    /// `OIDC client secret`
    CfOidc = "cf-oidc",
    /// `Proves the console to the identity provider; read from a file and never shown.`
    CfOidcDesc = "cf-oidc-desc",
    /// `On trial`
    CfOnTrial = "cf-on-trial",
    /// `A provisioned file`
    CfOpenProvisioned = "cf-open-provisioned",
    /// `Owned by the provisioned mapping directory, read at start`
    CfOwnedByDirectory = "cf-owned-by-directory",
    /// `Password`
    CfPassword = "cf-password",
    /// `Private key`
    CfPrivateKey = "cf-private-key",
    /// `Provenance tag`
    CfProvenance = "cf-provenance",
    /// `Provisioned`
    CfProvisioned = "cf-provisioned",
    /// `The console shows a provisioned file and never changes it: the lock in the title row says so, and its tooltip names the owner. Change the file in its directory and restart.`
    CfProvisionedBody = "cf-provisioned-body",
    /// `A mapping file from the provisioned directory`
    CfProvisionedSubtitle = "cf-provisioned-subtitle",
    /// `Provisioned file`
    CfProvisionedTitle = "cf-provisioned-title",
    /// `Read-only settings, file mode`
    CfReadOnly = "cf-read-only",
    /// `Value refused`
    CfRefused = "cf-refused",
    /// `Demo Reviewer, 10:09`
    CfReviewer1009 = "cf-reviewer-1009",
    /// `Roll back now`
    CfRollBackNow = "cf-roll-back-now",
    /// `Rolled back`
    CfRolledBack = "cf-rolled-back",
    /// `The server saved revision 9's content as revision 11 and applied it at 10:14:42, with the reason that revision 10 was not confirmed by its deadline.`
    CfRolledBackLine = "cf-rolled-back-line",
    /// `Revision 10 was rolled back: no one confirmed it in time`
    CfRolledBackTitle = "cf-rolled-back-title",
    /// `Saved`
    CfSaved = "cf-saved",
    /// `Secret field`
    CfSecrets = "cf-secrets",
    /// `Session key`
    CfSessionKey = "cf-session-key",
    /// `Seals the session cookie; every replica holds the same key.`
    CfSessionKeyDesc = "cf-session-key-desc",
    /// `Set a value`
    CfSetValue = "cf-set-value",
    /// `Source`
    CfSource = "cf-source",
    /// `The revision lifecycle and its pending card, the commit-confirm countdown, the write-only secret field and the provenance tag.`
    CfSubtitle = "cf-subtitle",
    /// `10:02`
    CfT1002 = "cf-t1002",
    /// `10:12:42`
    CfT121242 = "cf-t121242",
    /// `Template`
    CfTemplate = "cf-template",
    /// `It sets the credentials of node-a-lab-pub. Unless an operator confirms it by 10:14:42, the server restores revision 9's content.`
    CfTrialLine = "cf-trial-line",
    /// `Revision 10 runs on trial`
    CfTrialTitle = "cf-trial-title",
    /// `Set, unreadable`
    CfUnreadable = "cf-unreadable",
    /// `It changes the credentials of node-a-lab-pub. You saved it, so another operator approves it.`
    CfWaitLine = "cf-wait-line",
    /// `Revision 10 waits for a second operator`
    CfWaitTitle = "cf-wait-title",
    /// `Waiting for approval`
    CfWaitingApproval = "cf-waiting-approval",
    /// `Where this revision stands`
    CfWhere = "cf-where",
    /// `within 2 min of the apply`
    CfWithin2Min = "cf-within-2-min",
}
