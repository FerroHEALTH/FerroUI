// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the sign-in pages.

use super::Msg;

messages! {
    /// `or, on this development deployment`
    BasicDivider = "basic-divider",
    /// `Documentation`
    Documentation = "documentation",
    /// `Privacy notice`
    PrivacyNotice = "privacy-notice",
    /// `Read without signing in`
    ReadWithout = "read-without",
    /// `Every screen is open to read.`
    ReadWithoutLine = "read-without-line",
    /// `The gallery of the FerroHEALTH consoles. It holds no clinical data.`
    SignInCaption = "sign-in-caption",
    /// `You sign in at your organisation's identity provider; the gallery never sees your password.`
    SignInNote = "sign-in-note",
    /// `Taking you to FerroHEALTH ID…`
    SignInTaking = "sign-in-taking",
    /// `Sign in to the gallery`
    SignInTitle = "sign-in-title",
    /// `Sign in with FerroHEALTH ID`
    SignInWith = "sign-in-with",
    /// `Your session ended after 30 minutes without a request. Sign in again to go on.`
    StateExpired = "state-expired",
    /// `The gallery holds as many sessions as it allows, so it cannot start yours. Try again in a few minutes; sessions end after 30 minutes without a request.`
    StateFull = "state-full",
    /// `Sign-in is paused for this username after 5 failed tries. Try again later, or sign in with FerroHEALTH ID.`
    StateLocked = "state-locked",
    /// `The gallery could not reach FerroHEALTH ID. Try again in a moment; it may be down.`
    StateProviderUnreachable = "state-provider-unreachable",
    /// `You come back here once you are signed in.`
    StateRedirecting = "state-redirecting",
    /// `Not signed in: the sign-in could not be completed at FerroHEALTH ID. Try again; if it keeps failing, the provider may be down.`
    StateRefused = "state-refused",
    /// `The gallery could not exchange the code for tokens (502).`
    StateRefusedDetail = "state-refused-detail",
    /// `Not signed in: FerroHEALTH ID did not sign you in, so nothing was kept. If it keeps happening, ask your provider's administrator.`
    StateRejected = "state-rejected",
    /// `For the administrator: the ID token did not pass the gallery's checks (401).`
    StateRejectedDetail = "state-rejected-detail",
    /// `You are signed out of the gallery and of FerroHEALTH ID.`
    StateSignedOut = "state-signed-out",
    /// `Sign-in is not set up yet. Whoever runs the gallery adds an identity provider to its configuration file.`
    StateUnavailable = "state-unavailable",
    /// `No [oidc] section in the configuration file (503).`
    StateUnavailableDetail = "state-unavailable-detail",
    /// `The server behind the gallery did not answer. You can still try to sign in.`
    StateUpstream = "state-upstream",
    /// `Not signed in: the username or the password is wrong.`
    StateWrong = "state-wrong",
    /// `4 tries are left before a 15-minute pause.`
    StateWrongTries = "state-wrong-tries",
}
