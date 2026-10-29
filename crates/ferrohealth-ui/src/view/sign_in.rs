// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The sign-in page (R19), one card under the product's lockup.
//!
//! The card holds the title, the notice of the state the sign-in is in, the
//! provider's action, the basic form when the console passes one, a line
//! on how sign-in works and the deployment's links, with a caption kept to
//! the card's width under it.
//!
//! Every state is a typed [`State`]: each names its cause in the console's
//! words and the way on, in its tone, as a status or, for a refusal, an
//! alert. The page keeps nothing (family rule 9): the provider's action is
//! a plain link to the console's route, and the basic form is a plain `POST` with plain,
//! uncontrolled inputs and the autocomplete tokens a password manager reads
//! (HTML, `autocomplete`, `username` and `current-password`), so the page
//! works with no script. No
//! placeholder repeats a label, and no password field is drawn unless the
//! console passes a [`Basic`] form.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::button::Variant;
use crate::view::form::{ERROR, INPUT};

/// The id of the notice of the sign-in's state, for a journey to find it.
pub const NOTICE_ID: &str = "sign-in-notice";

/// A link the notice offers as the way on, such as Try again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retry {
    /// Its words.
    pub label: String,
    /// Where it leads, such as the sign-in page itself.
    pub href: String,
}

/// The words of a state's notice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    /// The cause and the way on, as whole sentences, such as "Your session
    /// ended after 30 minutes without a request. Sign in again to go on.".
    pub text: String,
    /// A muted line after it for whoever runs the console, such as "The
    /// console could not exchange the code for tokens (502).".
    pub detail: Option<String>,
    /// A link that tries again, if one helps.
    pub retry: Option<Retry>,
}

impl Notice {
    /// A notice saying `text`, with no detail and no retry.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            detail: None,
            retry: None,
        }
    }
}

/// The state the sign-in is in, each with its notice.
///
/// It serializes, so a console can carry a state its server chose, such as
/// a refused callback, across a redirect.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum State {
    /// Nothing to say: the card offers its methods.
    #[default]
    Ready,
    /// The browser is on its way to the identity provider: the provider's
    /// action is busy and every method is disabled.
    Redirecting(Option<Notice>),
    /// The operator signed out.
    SignedOut(Notice),
    /// The session ended.
    Expired(Notice),
    /// The console holds as many sessions as it allows.
    Full(Notice),
    /// Sign-in is not set up, so the provider's action is disabled.
    Unavailable(Notice),
    /// The sign-in could not be completed, such as a failed code exchange
    /// or a wrong password.
    Refused(Notice),
    /// The identity provider refused the operator, or its answer did not
    /// pass the console's checks.
    Rejected(Notice),
    /// The console could not reach the identity provider.
    ProviderUnreachable(Notice),
    /// The product behind the console did not answer; sign-in may still
    /// work.
    UpstreamUnreachable(Notice),
    /// Sign-in is paused for the username after too many failed tries, so
    /// the basic form is disabled.
    Locked(Notice),
}

impl State {
    /// The tone of the state's notice.
    #[must_use]
    pub const fn tone(&self) -> Tone {
        match self {
            Self::Ready | Self::Redirecting(_) => Tone::Info,
            Self::SignedOut(_) => Tone::Ok,
            Self::Expired(_)
            | Self::Full(_)
            | Self::Unavailable(_)
            | Self::UpstreamUnreachable(_)
            | Self::Locked(_) => Tone::Warn,
            Self::Refused(_) | Self::Rejected(_) | Self::ProviderUnreachable(_) => Tone::Bad,
        }
    }

    /// Whether the notice interrupts as an alert: a refusal, or a state
    /// that stops this sign-in (WAI-ARIA 1.2, `alert`); the others are a
    /// status.
    #[must_use]
    pub const fn alerts(&self) -> bool {
        matches!(
            self,
            Self::Full(_)
                | Self::Refused(_)
                | Self::Rejected(_)
                | Self::ProviderUnreachable(_)
                | Self::Locked(_)
        )
    }

    /// The state's notice, if it has one.
    #[must_use]
    pub const fn notice(&self) -> Option<&Notice> {
        match self {
            Self::Ready | Self::Redirecting(None) => None,
            Self::Redirecting(Some(notice))
            | Self::SignedOut(notice)
            | Self::Expired(notice)
            | Self::Full(notice)
            | Self::Unavailable(notice)
            | Self::Refused(notice)
            | Self::Rejected(notice)
            | Self::ProviderUnreachable(notice)
            | Self::UpstreamUnreachable(notice)
            | Self::Locked(notice) => Some(notice),
        }
    }
}

/// The provider's action: the link that starts the sign-in at the console's
/// own route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    /// The console's route, with the page to return to in its query.
    pub href: String,
    /// What it says, such as "Sign in with FerroHEALTH ID", or "Try again"
    /// after a refusal.
    pub label: String,
    /// What it says while the browser is on its way, such as "Taking you
    /// to FerroHEALTH ID…".
    pub busy: String,
    /// The glyph beside its words.
    pub glyph: Glyph,
}

/// The basic method: the form that posts a username and a password.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Basic {
    /// The console's route the form posts to.
    pub action: String,
    /// The page to return to once signed in, posted as `next`.
    pub next: String,
    /// The words between the provider's action and the form, such as "or,
    /// on this development deployment"; `None` says "or".
    pub divider: Option<String>,
    /// The form's button, such as "Sign in with a CDR account"; `None`
    /// says "Sign in".
    pub submit: Option<String>,
    /// The username the field holds again after a refusal.
    pub username: String,
    /// What is wrong with the last try, under the password, such as "4
    /// tries are left before a 15-minute pause.".
    pub error: Option<String>,
}

/// What the sign-in page shows.
#[derive(Debug)]
pub struct SignIn {
    /// The product's lockup, plain text while signed out, from
    /// [`crate::view::shell::lockup_plain`].
    pub lockup: Html,
    /// The page's heading, such as "Sign in to the console".
    pub title: Option<String>,
    /// The state the sign-in is in.
    pub state: State,
    /// The provider's action, when the console has one.
    pub provider: Option<Provider>,
    /// The basic form, when the console has one.
    pub basic: Option<Basic>,
    /// A line of the console's own under the methods, such as a link to
    /// read without signing in.
    pub line: Option<Html>,
    /// How sign-in works, muted, such as "You sign in at your
    /// organisation's identity provider; the console never sees your
    /// password.".
    pub note: Option<String>,
    /// The deployment's links, such as Documentation and Privacy notice, as
    /// words and href.
    pub links: Vec<(String, String)>,
    /// The caption under the card, kept to its width.
    pub caption: Option<String>,
}

struct NoticeView {
    role: &'static str,
    tone: &'static str,
    text: String,
    detail: Option<String>,
    retry: Option<Retry>,
}

/// How the provider's action is drawn in the state the sign-in is in.
enum ProviderView {
    /// A busy, disabled button while the browser is on its way.
    Busy(String),
    /// A disabled button while sign-in is unavailable.
    Off(Html, String),
    /// The link that starts the sign-in.
    Link(String, Html, String),
}

struct BasicView {
    divider: Option<Word>,
    action: String,
    username_label: Word,
    username: String,
    password_label: Word,
    error: Option<String>,
    next: String,
    disabled: bool,
    submit: Word,
    login: Html,
    alert: Html,
}

#[derive(Template)]
#[template(path = "sign_in/card.html")]
struct Card {
    input: &'static str,
    error_class: &'static str,
    primary: &'static str,
    secondary: &'static str,
    small: &'static str,
    notice_id: &'static str,
    refresh: Html,
    lockup: Html,
    title: Option<String>,
    notice: Option<NoticeView>,
    unavailable: Option<Word>,
    provider: Option<ProviderView>,
    basic: Option<BasicView>,
    line: Option<Html>,
    note: Option<String>,
    links: Vec<(String, String)>,
    caption: Option<String>,
}

/// Returns the sign-in page's card and caption, `sign_in`, centred by the
/// page around it ([`crate::view::shell::centred`]).
#[must_use]
pub fn sign_in(locale: &Locale, sign_in: SignIn) -> Html {
    let SignIn {
        lockup,
        title,
        state,
        provider,
        basic,
        line,
        note,
        links,
        caption,
    } = sign_in;
    let no_method = provider.is_none() && basic.is_none();
    let notice = state.notice().cloned().map(|words| NoticeView {
        role: if state.alerts() { "alert" } else { "status" },
        tone: state.tone().class(),
        text: words.text,
        detail: words.detail,
        retry: words.retry,
    });
    let after_provider = provider.is_some();
    let provider = provider.map(|provider| match state {
        State::Redirecting(_) => ProviderView::Busy(provider.busy),
        State::Unavailable(_) => ProviderView::Off(icon(provider.glyph, "16"), provider.label),
        _ => ProviderView::Link(provider.href, icon(provider.glyph, "16"), provider.label),
    });
    let disabled = matches!(state, State::Redirecting(_) | State::Locked(_));
    let basic = basic.map(|basic| BasicView {
        divider: after_provider.then(|| {
            basic
                .divider
                .map_or_else(|| tr(locale, Text::SignInOr), Word::from)
        }),
        action: basic.action,
        username_label: tr(locale, Text::SignInUsername),
        username: basic.username,
        password_label: tr(locale, Text::SignInPassword),
        error: basic.error,
        next: basic.next,
        disabled,
        submit: basic
            .submit
            .map_or_else(|| tr(locale, Text::SignInSubmit), Word::from),
        login: icon(Glyph::LOG_IN, "16"),
        alert: icon(Glyph::CIRCLE_ALERT, "14"),
    });
    Card {
        input: INPUT,
        error_class: ERROR,
        primary: Variant::Primary.class(),
        secondary: Variant::Secondary.class(),
        small: Variant::Secondary.small(),
        notice_id: NOTICE_ID,
        refresh: icon(Glyph::REFRESH_CW, "16"),
        lockup,
        title,
        notice,
        unavailable: (no_method && state == State::Ready).then(|| tr(locale, Text::SignInNone)),
        provider,
        basic,
        line,
        note,
        links,
        caption,
    }
    .html()
}
