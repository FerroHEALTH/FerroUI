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
//! a plain link to the console's route, marked `rel="external"` so a router
//! leaves it to the server, and the basic form is a plain `POST` with plain,
//! uncontrolled inputs and the autocomplete tokens a password manager reads
//! (HTML, `autocomplete`, `username` and `current-password`), so the page
//! works with no script and hydration never wipes what was typed. No
//! placeholder repeats a label, and no password field is drawn unless the
//! console passes a [`Basic`] form.

use leptos::either::Either;
use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::button::Variant;
use crate::view::form::{ERROR, INPUT};

/// The id of the notice of the sign-in's state, for a journey to find it.
pub const NOTICE_ID: &str = "sign-in-notice";

/// A link the notice offers as the way on, such as Try again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retry {
    /// Its words.
    pub label: String,
    /// Where it leads, such as the sign-in page itself.
    pub href: String,
}

/// The words of a state's notice.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
    pub lockup: AnyView,
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
    pub line: Option<AnyView>,
    /// How sign-in works, muted, such as "You sign in at your
    /// organisation's identity provider; the console never sees your
    /// password.".
    pub note: Option<String>,
    /// The deployment's links, such as Documentation and Privacy notice, as
    /// words and href; each leaves the console's router alone.
    pub links: Vec<(String, String)>,
    /// The caption under the card, kept to its width.
    pub caption: Option<String>,
}

/// Returns the notice of `state`, if it has one.
fn notice(state: &State) -> Option<impl IntoView + use<>> {
    let words = state.notice()?.clone();
    let role = if state.alerts() { "alert" } else { "status" };
    let class = format!("fui-sign-in-notice {}", state.tone().class());
    let detail = words.detail.map(|detail| {
        view! {
            " "
            <span class="fui-small fui-muted">{detail}</span>
        }
    });
    let retry = words.retry.map(|retry| {
        view! {
            " "
            <a href=retry.href rel="external" class=Variant::Secondary.small()>
                {icon(Glyph::REFRESH_CW, "16")}
                {retry.label}
            </a>
        }
    });
    Some(view! {
        <div role=role id=NOTICE_ID class=class>
            {words.text}
            {detail}
            {retry}
        </div>
    })
}

/// Returns the provider's action in `state`: a link, a busy button while
/// redirecting, a disabled button while sign-in is unavailable.
fn provider_action(provider: Provider, state: &State) -> impl IntoView + use<> {
    let class = Variant::Primary.class();
    match state {
        State::Redirecting(_) => Either::Left(Either::Left(view! {
            <button type="button" class=class aria-busy="true" disabled>
                <span class="fui-spinner-ring" aria-hidden="true"></span>
                {provider.busy}
            </button>
        })),
        State::Unavailable(_) => Either::Left(Either::Right(view! {
            <button type="button" class=class disabled>
                {icon(provider.glyph, "16")}
                {provider.label}
            </button>
        })),
        _ => Either::Right(view! {
            <a href=provider.href rel="external" class=class>
                {icon(provider.glyph, "16")}
                {provider.label}
            </a>
        }),
    }
}

/// Returns the basic form, its divider first when it follows the
/// provider's action, disabled while `disabled`.
fn basic_form(basic: Basic, after_provider: bool, disabled: bool) -> impl IntoView + use<> {
    let divider = after_provider.then(|| {
        let words = basic.divider.unwrap_or_else(|| tr(Text::SignInOr));
        view! { <div class="fui-divider">{words}</div> }
    });
    let submit = basic.submit.unwrap_or_else(|| tr(Text::SignInSubmit));
    let invalid = basic.error.is_some().then_some("true");
    let described = basic.error.is_some().then_some("sign-in-password-error");
    let error = basic.error.map(|error| {
        view! {
            <p id="sign-in-password-error" class=ERROR>
                {icon(Glyph::CIRCLE_ALERT, "14")}
                " "
                {error}
            </p>
        }
    });
    view! {
        {divider}
        <form method="post" action=basic.action class="fui-form">
            <div class="fui-field">
                <label for="sign-in-username" class="fui-label">
                    {tr(Text::SignInUsername)}
                </label>
                <input
                    id="sign-in-username"
                    name="username"
                    class=INPUT
                    value=basic.username
                    autocomplete="username"
                    autocapitalize="off"
                    spellcheck="false"
                    disabled=disabled
                />
            </div>
            <div class="fui-field">
                <label for="sign-in-password" class="fui-label">
                    {tr(Text::SignInPassword)}
                </label>
                <input
                    id="sign-in-password"
                    name="password"
                    type="password"
                    class=INPUT
                    autocomplete="current-password"
                    aria-invalid=invalid
                    aria-describedby=described
                    disabled=disabled
                />
                {error}
            </div>
            <input type="hidden" name="next" value=basic.next />
            <button type="submit" class=Variant::Secondary.class() disabled=disabled>
                {icon(Glyph::LOG_IN, "16")}
                {submit}
            </button>
        </form>
    }
}

/// Returns the sign-in page's card and caption, `sign_in`, centred by the
/// page around it ([`crate::view::shell::centred`]).
#[must_use]
pub fn sign_in(sign_in: SignIn) -> impl IntoView {
    let state = sign_in.state;
    let no_method = sign_in.provider.is_none() && sign_in.basic.is_none();
    let title = sign_in
        .title
        .map(|title| view! { <h1 class="fui-sign-in-title">{title}</h1> });
    let notice = notice(&state);
    let after_provider = sign_in.provider.is_some();
    let provider = sign_in
        .provider
        .map(|provider| provider_action(provider, &state));
    let locked = matches!(state, State::Redirecting(_) | State::Locked(_));
    let basic = sign_in
        .basic
        .map(|basic| basic_form(basic, after_provider, locked));
    let unavailable = (no_method && state == State::Ready).then(|| {
        view! {
            <div role="alert" class="fui-sign-in-notice fui-tone-warn">
                {tr(Text::SignInNone)}
            </div>
        }
    });
    let note = sign_in
        .note
        .map(|note| view! { <p class="fui-small fui-muted fui-sign-in-line">{note}</p> });
    let links = (!sign_in.links.is_empty()).then(|| {
        let links = sign_in
            .links
            .into_iter()
            .map(|(label, href)| {
                view! {
                    <a href=href rel="external noopener">
                        {label}
                    </a>
                }
            })
            .collect_view();
        view! { <div class="fui-sign-in-links">{links}</div> }
    });
    let caption = sign_in
        .caption
        .map(|caption| view! { <p class="fui-sign-in-caption">{caption}</p> });
    view! {
        <div class="fui-stack fui-sign-in-stack">
            <div class="fui-card fui-sign-in">
                <div class="fui-sign-in-lockup">{sign_in.lockup}</div>
                {title}
                {notice}
                {unavailable}
                {provider}
                {basic}
                {sign_in.line}
                {note}
                {links}
            </div>
            {caption}
        </div>
    }
}
